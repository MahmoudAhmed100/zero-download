//! The helper programs that come with Zero Download: yt-dlp (finds the video
//! links) and ffmpeg (joins video + audio), both from their GitHub releases. They live in the "tools" folder
//! of the install folder. Once a day, silently, Zero Download checks for new
//! versions and updates them, and puts them back if one went missing.

use crate::mux::hide_window;
use anyhow::Context;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

const YTDLP_URL: &str = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe";
/// the SHA-256 of every file of that release, published by yt-dlp
const YTDLP_SUMS_URL: &str = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/SHA2-256SUMS";
/// the SHA-256 of every build, published with the ffmpeg builds
const FFMPEG_SUMS_NAME: &str = "checksums.sha256";
/// none of the helpers is anywhere near this big
const MAX_TOOL_BYTES: u64 = 400 << 20;
/// ffmpeg builds published on GitHub (rolling release "latest")
const FFMPEG_RELEASE_API: &str = "https://api.github.com/repos/BtbN/FFmpeg-Builds/releases/tags/latest";

fn exe(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}

/// Find a helper next to the program (`tools/<name>.exe` or `<name>.exe`),
/// then in PATH, then in the per-user data folder.
pub fn find_tool(name: &str) -> Option<PathBuf> {
    bundled(name).or_else(|| which::which(name).ok())
}

/// A helper installed with the program itself or downloaded earlier into its
/// data folder (not an arbitrary copy elsewhere on the PATH).
pub fn bundled(name: &str) -> Option<PathBuf> {
    let target = exe(name);
    for dir in tool_dirs() {
        let p = dir.join(&target);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

/// Folders where the helper programs are looked for and updated.
pub fn tool_dirs() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Ok(exe_path) = std::env::current_exe() {
        let dir = exe_path.parent().unwrap_or(Path::new("."));
        v.push(dir.join("tools"));
        v.push(dir.to_path_buf());
    }
    if let Some(p) = directories::ProjectDirs::from("com", "Zero", "ZeroDownload") {
        v.push(p.data_dir().join("tools"));
    }
    v
}

/// First folder in `tool_dirs()` that exists and can be written to.
fn writable_dir(name: &str) -> Option<PathBuf> {
    for d in tool_dirs() {
        if d.is_dir() && can_write(&d) {
            return Some(d);
        }
    }
    // try to make the tools folder in the app data directory
    if let Some(p) = directories::ProjectDirs::from("com", "Zero", "ZeroDownload") {
        let d = p.data_dir().join("tools");
        if std::fs::create_dir_all(&d).is_ok() && can_write(&d) {
            return Some(d);
        }
    }
    let _ = name;
    None
}

fn can_write(dir: &Path) -> bool {
    let probe = dir.join(".zero-write-test");
    let ok = std::fs::write(&probe, b"x").is_ok();
    let _ = std::fs::remove_file(&probe);
    ok
}

fn due(stamp: &Path, every: Duration) -> bool {
    std::fs::metadata(stamp)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| SystemTime::now().duration_since(t).ok())
        .map(|age| age >= every)
        .unwrap_or(true)
}

/// The SHA-256 of `name` in a checksum list ("<sha256>  <name>" per line).
pub fn sum_for(list: &str, name: &str) -> Option<String> {
    list.lines().find_map(|l| {
        let mut it = l.split_whitespace();
        let (sum, file) = (it.next()?, it.next()?);
        (file.trim_start_matches('*') == name && sum.len() == 64 && sum.chars().all(|c| c.is_ascii_hexdigit()))
            .then(|| sum.to_ascii_lowercase())
    })
}

async fn get_text(client: &reqwest::Client, url: &str) -> anyhow::Result<String> {
    let text = client
        .get(url)
        .header(reqwest::header::USER_AGENT, crate::config::DEFAULT_USER_AGENT)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    if text.len() > 1 << 20 {
        anyhow::bail!("unexpected answer from {url}");
    }
    Ok(text)
}

/// Download `url` to `dest`. It is used only when its SHA-256 matches `sha256` (if given).
async fn download(client: &reqwest::Client, url: &str, dest: &Path, sha256: Option<&str>) -> anyhow::Result<()> {
    use sha2::{Digest, Sha256};
    let mut resp = client
        .get(url)
        .header(reqwest::header::USER_AGENT, crate::config::DEFAULT_USER_AGENT)
        .send()
        .await?
        .error_for_status()?;
    use std::io::Write;
    let tmp = dest.with_extension("download");
    let mut file = std::fs::File::create(&tmp)?;
    let mut h = Sha256::new();
    let mut total: u64 = 0;
    while let Some(chunk) = resp.chunk().await? {
        total += chunk.len() as u64;
        if total > MAX_TOOL_BYTES {
            drop(file);
            let _ = std::fs::remove_file(&tmp);
            anyhow::bail!("the download of {url} is far bigger than expected");
        }
        h.update(&chunk);
        file.write_all(&chunk)?;
    }
    drop(file);
    if total < 1024 {
        let _ = std::fs::remove_file(&tmp);
        anyhow::bail!("the download of {url} was too small");
    }
    if let Some(expected) = sha256 {
        let got: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
        if !got.eq_ignore_ascii_case(expected) {
            let _ = std::fs::remove_file(&tmp);
            anyhow::bail!("the download of {url} does not match its published SHA-256");
        }
    }
    replace(&tmp, dest)
}

fn replace(from: &Path, to: &Path) -> anyhow::Result<()> {
    #[cfg(windows)]
    {
        // a running program on Windows cannot be overwritten: rename the
        // current one to ".old" (which Windows allows) and drop the new one in
        let old = to.with_extension("old");
        let _ = std::fs::remove_file(&old);
        if to.exists() {
            let _ = std::fs::rename(to, &old);
        }
        std::fs::rename(from, to).with_context(|| format!("could not install {}", to.display()))?;
        let _ = std::fs::remove_file(&old);
        Ok(())
    }
    #[cfg(not(windows))]
    {
        std::fs::rename(from, to).with_context(|| format!("could not install {}", to.display()))
    }
}

/// Download yt-dlp once when it is missing; otherwise ask it to update
/// itself (`yt-dlp -U`) once a day.
async fn yt_dlp(client: &reqwest::Client, data_dir: &Path) {
    if bundled("yt-dlp").is_none() {
        if cfg!(windows) {
            if let Some(dir) = writable_dir("yt-dlp") {
                if let Ok(sums) = get_text(client, YTDLP_SUMS_URL).await {
                    if let Some(sum) = sum_for(&sums, "yt-dlp.exe") {
                        let _ = download(client, YTDLP_URL, &dir.join(exe("yt-dlp")), Some(&sum)).await;
                    }
                }
            }
        }
        return;
    }
    let stamp = data_dir.join("yt-dlp-checked");
    if !due(&stamp, Duration::from_secs(24 * 3600)) {
        return;
    }
    let Some(tool) = bundled("yt-dlp") else { return };
    let ok = tokio::task::spawn_blocking(move || {
        let mut cmd = Command::new(&tool);
        cmd.args(["--ignore-config", "-U"]);
        hide_window(&mut cmd);
        cmd.output().map(|o| o.status.success()).unwrap_or(false)
    })
    .await
    .unwrap_or(false);
    if ok {
        let _ = std::fs::write(&stamp, b"ok");
    }
}

#[derive(Deserialize)]
struct GhRelease {
    tag_name: String,
    assets: Vec<GhAsset>,
}

#[derive(Deserialize)]
struct GhAsset {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    updated_at: String,
}

fn pick_ffmpeg(assets: &[GhAsset]) -> Option<&GhAsset> {
    let win = assets.iter().filter(|a| a.name.ends_with(".zip") && a.name.contains("win64-gpl"));
    // prefer a released version like ffmpeg-n7.1-latest-win64-gpl-7.1.zip
    // over rolling master
    let release = win.clone().find(|a| !a.name.contains("shared") && a.name.starts_with("ffmpeg-n"));
    release.or_else(|| win.into_iter().find(|a| !a.name.contains("shared")))
}

/// Download a new ffmpeg when missing or when BtbN published a new release
/// (checked once a day).
async fn ffmpeg(client: &reqwest::Client, data_dir: &Path) -> anyhow::Result<()> {
    if !cfg!(windows) {
        return Ok(());
    }
    let present = bundled("ffmpeg");
    let stamp = data_dir.join("ffmpeg-checked");
    if present.is_some() && !due(&stamp, Duration::from_secs(24 * 3600)) {
        return Ok(());
    }
    let release: GhRelease = client
        .get(FFMPEG_RELEASE_API)
        .header(reqwest::header::USER_AGENT, crate::config::DEFAULT_USER_AGENT)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let asset = pick_ffmpeg(&release.assets).ok_or_else(|| anyhow::anyhow!("no ffmpeg build in release"))?;
    let zip_url = asset.browser_download_url.clone();
    if !zip_url.starts_with("https://github.com/") {
        anyhow::bail!("unexpected ffmpeg download address");
    }
    let sums_url = release
        .assets
        .iter()
        .find(|a| a.name == FFMPEG_SUMS_NAME)
        .map(|a| a.browser_download_url.clone())
        .filter(|u| u.starts_with("https://github.com/"))
        .ok_or_else(|| anyhow::anyhow!("the ffmpeg release has no checksum list"))?;
    let Some(dir) = writable_dir("ffmpeg") else { return Ok(()) };
    let version_file = dir.join("ffmpeg-version.txt");
    let current = std::fs::read_to_string(&version_file).unwrap_or_default();
    let new_tag = format!("{} {}", release.tag_name, asset.updated_at);
    if present.is_some() && current.trim() == new_tag.trim() {
        let _ = std::fs::write(&stamp, b"ok");
        return Ok(());
    }
    // download the zip, unpack it with Windows' own tar, take ffmpeg.exe
    let work = std::env::temp_dir().join(format!("zero-ffmpeg-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work)?;
    let zip = work.join("ffmpeg.zip");
    let sum = sum_for(&get_text(client, &sums_url).await?, &asset.name)
        .ok_or_else(|| anyhow::anyhow!("no checksum for {}", asset.name))?;
    download(client, &zip_url, &zip, Some(&sum)).await?;
    let (zip2, work2) = (zip.clone(), work.clone());
    let unpacked = tokio::task::spawn_blocking(move || {
        let mut cmd = Command::new(crate::system::windows_tool("tar.exe"));
        cmd.arg("-xf").arg(&zip2).arg("-C").arg(&work2);
        hide_window(&mut cmd);
        cmd.output().map(|o| o.status.success()).unwrap_or(false)
    })
    .await
    .unwrap_or(false);
    let found = unpacked.then(|| find_file(&work, &exe("ffmpeg"))).flatten();
    let result = match found {
        Some(f) => {
            let staged = dir.join("ffmpeg.new");
            std::fs::copy(&f, &staged)?;
            replace(&staged, &dir.join(exe("ffmpeg")))?;
            let _ = std::fs::write(&version_file, &new_tag);
            let _ = std::fs::write(&stamp, b"ok");
            Ok(())
        }
        None => Err(anyhow::anyhow!("ffmpeg.exe was not found in the download")),
    };
    let _ = std::fs::remove_dir_all(&work);
    result
}

fn find_file(dir: &Path, name: &str) -> Option<PathBuf> {
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let p = entry.path();
        if p.is_dir() {
            if let Some(f) = find_file(&p, name) {
                return Some(f);
            }
        } else if p.file_name().map(|n| n.eq_ignore_ascii_case(name)).unwrap_or(false) {
            return Some(p);
        }
    }
    None
}

/// Check tools now (only does work when a day has passed or one is
/// missing). Never shows anything; errors just mean "try again later".
async fn aria2(client: &reqwest::Client, data_dir: &Path) -> anyhow::Result<()> {
    if !cfg!(windows) {
        return Ok(());
    }
    let present = bundled("aria2c");
    let stamp = data_dir.join("aria2-checked");
    if present.is_some() && !due(&stamp, Duration::from_secs(24 * 3600)) {
        return Ok(());
    }
    let zip_url = "https://github.com/aria2/aria2/releases/download/release-1.37.0/aria2-1.37.0-win-64bit-build1.zip";
    let Some(dir) = writable_dir("aria2c") else { return Ok(()) };
    let work = std::env::temp_dir().join(format!("zero-aria2-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work)?;
    let zip = work.join("aria2.zip");
    download(client, zip_url, &zip, None).await?;
    let (zip2, work2) = (zip.clone(), work.clone());
    let unpacked = tokio::task::spawn_blocking(move || {
        let mut cmd = Command::new(crate::system::windows_tool("tar.exe"));
        cmd.arg("-xf").arg(&zip2).arg("-C").arg(&work2);
        hide_window(&mut cmd);
        cmd.output().map(|o| o.status.success()).unwrap_or(false)
    })
    .await
    .unwrap_or(false);
    let found = unpacked.then(|| find_file(&work, &exe("aria2c"))).flatten();
    let result = match found {
        Some(f) => {
            let staged = dir.join("aria2c.new");
            std::fs::copy(&f, &staged)?;
            replace(&staged, &dir.join(exe("aria2c")))?;
            let _ = std::fs::write(&stamp, b"ok");
            Ok(())
        }
        None => Err(anyhow::anyhow!("aria2c.exe was not found in the download")),
    };
    let _ = std::fs::remove_dir_all(&work);
    result
}

pub async fn update_all(client: reqwest::Client, data_dir: PathBuf) {
    let _ = aria2(&client, &data_dir).await;
    yt_dlp(&client, &data_dir).await;
    let _ = ffmpeg(&client, &data_dir).await;
    // leftovers of earlier updates
    for dir in tool_dirs() {
        for name in ["yt-dlp", "ffmpeg", "aria2c"] {
            let _ = std::fs::remove_file(dir.join(exe(name)).with_extension("old"));
        }
    }
}

/// Run `update_all` now and then every few hours while the app is open.
pub fn spawn_updater(rt: &tokio::runtime::Handle, client: reqwest::Client, data_dir: PathBuf) {
    rt.spawn(async move {
        // let the app start first
        tokio::time::sleep(Duration::from_secs(20)).await;
        loop {
            update_all(client.clone(), data_dir.clone()).await;
            tokio::time::sleep(Duration::from_secs(6 * 3600)).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn picks_stable_ffmpeg() {
        let a = |n: &str| GhAsset { name: n.into(), browser_download_url: String::new(), updated_at: String::new() };
        let list = vec![
            a("ffmpeg-master-latest-win64-gpl.zip"),
            a("ffmpeg-n7.1-latest-win64-gpl-7.1.zip"),
            a("ffmpeg-n7.0-latest-win64-gpl-7.0.zip"),
            a("ffmpeg-n7.1-latest-win64-gpl-shared-7.1.zip"),
            a("ffmpeg-n7.1-latest-linux64-gpl-7.1.tar.xz"),
        ];
        assert_eq!(pick_ffmpeg(&list).unwrap().name, "ffmpeg-n7.1-latest-win64-gpl-7.1.zip");
        assert_eq!(pick_ffmpeg(&list[..1]).unwrap().name, "ffmpeg-master-latest-win64-gpl.zip");
    }

    #[test]
    fn replace_and_find() {
        let dir = std::env::temp_dir().join(format!("zero-tools-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("a/b")).unwrap();
        std::fs::write(dir.join("a/b/ffmpeg.exe"), b"new").unwrap();
        assert_eq!(find_file(&dir, "ffmpeg.exe").unwrap(), dir.join("a/b/ffmpeg.exe"));
        assert!(find_file(&dir, "missing.exe").is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
