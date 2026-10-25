//! New versions of Zero Download itself.
//!
//! Every release on GitHub carries two files:
//!
//! * `ZeroDownload-Setup.exe` – the installer
//! * `version.json`           – `{"version":"1.0.1","url":"…/ZeroDownload-Setup.exe","sha256":"…"}`
//!
//! The app reads `version.json` of the latest release, and when the version
//! there is higher than its own it downloads the installer, checks that its
//! SHA-256 matches, and hands it back to the app to run.
//!
//! `version.json` can also carry `sig`: an Ed25519 signature of the version and
//! the SHA-256, made by the release build with a private key that only lives
//! in the repository's secrets.

use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub const REPO: &str = "MahmoudAhmed100/zero-download";
pub const SETUP_NAME: &str = "ZeroDownload-Setup.exe";

pub fn version_url() -> String {
    format!("https://github.com/{REPO}/releases/latest/download/version.json")
}

/// The running version.
pub fn current() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Public half of the release signing key (Ed25519, 32 bytes, hex).
pub const UPDATE_PUBLIC_KEY: &str = "4bd5c564c841b012b39426755d585c48012f9360e193cd81d141f85ad9b58406";

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Release {
    pub version: String,
    #[serde(default)]
    pub url: String,
    pub sha256: String,
    /// Ed25519 signature of `signed_message(version, sha256)`, hex
    #[serde(default)]
    pub sig: String,
}

/// What the release signature covers.
pub fn signed_message(version: &str, sha256: &str) -> String {
    format!("zero-download-update\n{}\n{}\n", version.trim(), sha256.trim().to_ascii_lowercase())
}

pub fn from_hex(s: &str) -> Option<Vec<u8>> {
    let s = s.trim();
    if s.len() % 2 != 0 {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok()).collect()
}

/// The signature of `r` was made with the key whose public half is `public_hex`.
pub fn signature_ok(r: &Release, public_hex: &str) -> bool {
    use ring::signature::{UnparsedPublicKey, ED25519};
    let (Some(key), Some(sig)) = (from_hex(public_hex), from_hex(&r.sig)) else {
        return false;
    };
    if key.len() != 32 || sig.len() != 64 {
        return false;
    }
    UnparsedPublicKey::new(&ED25519, key)
        .verify(signed_message(&r.version, &r.sha256).as_bytes(), &sig)
        .is_ok()
}

impl Release {
    /// Where the installer is downloaded from. Only addresses of our own
    /// repository are accepted.
    pub fn setup_url(&self) -> String {
        let own = format!("https://github.com/{REPO}/");
        if self.url.starts_with(&own) {
            self.url.clone()
        } else {
            format!("https://github.com/{REPO}/releases/download/v{}/{SETUP_NAME}", self.version)
        }
    }
}

/// True when `latest` is a higher version than `now` ("1.2.0" > "1.1.9").
pub fn is_newer(latest: &str, now: &str) -> bool {
    let parse = |s: &str| -> Vec<u64> {
        s.trim_start_matches('v')
            .split('.')
            .map(|x| x.parse().unwrap_or(0))
            .collect()
    };
    parse(latest) > parse(now)
}

/// This copy was installed with the setup (has `unins000.exe` next to it);
/// otherwise it is a development build or portable copy, and won't be replaced.
pub fn installed() -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("unins000.exe").exists()))
        .unwrap_or(false)
}

/// Read and check `version.json`.
pub fn parse(text: &str) -> anyhow::Result<Release> {
    let r: Release = serde_json::from_str(text.trim_start_matches('\u{feff}'))?;
    if r.sha256.len() != 64 || !r.sha256.chars().all(|c| c.is_ascii_hexdigit()) {
        anyhow::bail!("version.json has an invalid sha256");
    }
    if r.version.is_empty() {
        anyhow::bail!("version.json has an empty version");
    }
    Ok(r)
}

/// `parse`, and if signed it must match our key.
pub fn parse_signed(text: &str) -> anyhow::Result<Release> {
    let r = parse(text)?;
    if !r.sig.is_empty() && !signature_ok(&r, UPDATE_PUBLIC_KEY) {
        anyhow::bail!("the update signature is invalid, so it is not installed");
    }
    Ok(r)
}

/// The latest release, if it is newer than this program.
pub async fn check(client: &reqwest::Client) -> anyhow::Result<Option<Release>> {
    let text = client
        .get(version_url())
        .header(reqwest::header::USER_AGENT, crate::config::DEFAULT_USER_AGENT)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let r: Release = serde_json::from_str(text.trim_start_matches('\u{feff}'))?;
    if !is_newer(&r.version, current()) {
        return Ok(None);
    }
    let r = parse_signed(&text)?;
    Ok(Some(r))
}

pub fn sha256_hex(data: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(data);
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

fn file_sha256(path: &Path) -> anyhow::Result<String> {
    use std::io::Read;
    let mut f = std::fs::File::open(path)?;
    let mut buf = [0u8; 64 * 1024];
    let mut h = Sha256::new();
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

/// No installer of ours is anywhere near this big.
const MAX_SETUP_BYTES: u64 = 500 << 20;

/// Download the installer of `r` into `dir` and check its SHA-256.
/// A copy that was already downloaded and checked is reused.
pub async fn download(client: &reqwest::Client, r: &Release, dir: &Path) -> anyhow::Result<PathBuf> {
    download_with_progress(client, r, dir, &|_, _| {}).await
}

/// `download`, telling `progress(bytes so far, total)` while it goes.
pub async fn download_with_progress(
    client: &reqwest::Client,
    r: &Release,
    dir: &Path,
    progress: &(dyn Fn(u64, Option<u64>) + Send + Sync),
) -> anyhow::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let dest = dir.join(format!("ZeroDownload-Setup-{}.exe", r.version.trim_start_matches('v')));
    if dest.exists() {
        if file_sha256(&dest)?.eq_ignore_ascii_case(&r.sha256) {
            return Ok(dest);
        }
        let _ = std::fs::remove_file(&dest);
    }

    let mut resp = client
        .get(r.setup_url())
        .header(reqwest::header::USER_AGENT, crate::config::DEFAULT_USER_AGENT)
        .send()
        .await?
        .error_for_status()?;

    use std::io::Write;
    let tmp = dest.with_extension("download");
    let mut file = std::fs::File::create(&tmp)?;
    let mut h = Sha256::new();
    let mut total: u64 = 0;
    let size = resp.content_length();
    progress(0, size);
    while let Some(chunk) = resp.chunk().await? {
        total += chunk.len() as u64;
        progress(total, size);
        if total > MAX_SETUP_BYTES {
            drop(file);
            let _ = std::fs::remove_file(&tmp);
            anyhow::bail!("the update is far bigger than expected");
        }
        h.update(&chunk);
        file.write_all(&chunk)?;
    }
    drop(file);

    let got: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
    if !got.eq_ignore_ascii_case(&r.sha256) {
        let _ = std::fs::remove_file(&tmp);
        anyhow::bail!("the downloaded update does not match its SHA-256");
    }
    std::fs::rename(&tmp, &dest)?;
    Ok(dest)
}

/// Run the installer without any window. It closes this program, installs
/// the new version over it and starts it again.
/// `visible`: the setup shows its progress (Help > Check for updates);
/// otherwise nothing is shown (the automatic update).
pub fn run_installer(setup: &Path, sha256: &str, visible: bool) -> anyhow::Result<()> {
    // checked again right before it runs: the file sat on the disk meanwhile
    if !file_sha256(setup)?.eq_ignore_ascii_case(sha256) {
        let _ = std::fs::remove_file(setup);
        anyhow::bail!("the downloaded update was changed on the disk, so it is not installed");
    }
    let mut cmd = std::process::Command::new(setup);
    cmd.arg(if visible { "/SILENT" } else { "/VERYSILENT" });
    cmd.args(["/SUPPRESSMSGBOXES", "/NORESTART", "/CLOSEAPPLICATIONS", "/AUTOUPDATE=1"]);
    if let Some(dir) = std::env::current_exe().ok().and_then(|e| e.parent().map(Path::to_path_buf)) {
        cmd.arg(format!("/DIR={}", dir.display()));
    }
    cmd.spawn()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_versions() {
        assert!(is_newer("1.0.1", "1.0.0"));
        assert!(is_newer("v1.2.0", "1.1.9"));
        assert!(!is_newer("1.0.0", "1.0.0"));
        assert!(!is_newer("1.0.0", "1.0.1"));
    }

    #[test]
    fn parse_release_json() {
        let text = r#"{"version":"1.0.1","url":"https://github.com/MahmoudAhmed100/zero-download/releases/download/v1.0.1/ZeroDownload-Setup.exe","sha256":"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"}"#;
        let r = parse(text).unwrap();
        assert_eq!(r.version, "1.0.1");
        assert_eq!(r.setup_url(), format!("https://github.com/{REPO}/releases/download/v1.0.1/{SETUP_NAME}"));

        let r2 = Release { version: "1.0.1".into(), url: String::new(), sha256: String::new(), sig: String::new() };
        assert!(r2.setup_url().starts_with("https://github.com/"));
        assert!(parse("{\"version\":\"1.0.1\",\"sha256\":\"zz\"}").is_err());
    }

    fn signed(version: &str, sha: &str) -> (String, String) {
        use ring::signature::{Ed25519KeyPair, KeyPair};
        let kp = Ed25519KeyPair::from_seed_unchecked(&[7u8; 32]).unwrap();
        let public: String = kp.public_key().as_ref().iter().map(|b| format!("{b:02x}")).collect();
        let sig: String = kp.sign(signed_message(version, sha).as_bytes()).as_ref().iter().map(|b| format!("{b:02x}")).collect();
        (public, sig)
    }

    #[test]
    fn signatures() {
        let sha = sha256_hex(b"setup");
        let (public, sig) = signed("2.1.0", &sha);
        let good = Release { version: "2.1.0".into(), url: String::new(), sha256: sha.clone(), sig: sig.clone() };
        assert!(signature_ok(&good, &public));
        // another version, another file, no signature, a broken one: refused
        assert!(!signature_ok(&Release { version: "2.1.1".into(), ..good.clone() }, &public));
        assert!(!signature_ok(&Release { sha256: sha256_hex(b"evil"), ..good.clone() }, &public));
        assert!(!signature_ok(&Release { sig: String::new(), ..good.clone() }, &public));
        assert!(!signature_ok(&Release { sig: "zz".repeat(64), ..good.clone() }, &public));
        // signed with another key: refused by the key in the program
        assert!(!signature_ok(&good, UPDATE_PUBLIC_KEY));
        let text = format!("{{\"version\":\"2.1.0\",\"sha256\":\"{sha}\",\"sig\":\"{sig}\"}}");
        assert!(parse_signed(&text).is_err());
        assert_eq!(UPDATE_PUBLIC_KEY.len(), 64);
    }

    #[test]
    fn installer_checked_again_before_it_runs() {
        let dir = std::env::temp_dir().join(format!("zero-su-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("setup.exe");
        std::fs::write(&f, b"changed").unwrap();
        let err = run_installer(&f, &sha256_hex(b"original"), false).unwrap_err();
        assert!(format!("{err}").contains("changed"));
        assert!(!f.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
