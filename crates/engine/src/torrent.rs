//! BitTorrent and Magnet link engine.
//!
//! Uses high-performance aria2c with DHT, PEX, UDP trackers, and sequential piece downloading
//! for instant video streaming/preview.

use crate::http::Progress;
use crate::limiter::SpeedLimiter;
use crate::mux::{find_tool, hide_window};
use anyhow::{bail, Context};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub struct MagnetInfo {
    pub info_hash: String,
    pub name: String,
    pub trackers: Vec<String>,
    pub exact_length: Option<u64>,
}

/// Detects if string is a magnet URI.
pub fn is_magnet_url(uri: &str) -> bool {
    uri.trim().starts_with("magnet:?")
}

/// Parses a magnet URI into components.
pub fn parse_magnet(uri: &str) -> Option<MagnetInfo> {
    let u = uri.trim();
    if !u.starts_with("magnet:?") {
        return None;
    }
    let query = &u["magnet:?".len()..];
    let mut hash = String::new();
    let mut name = String::new();
    let mut trackers = Vec::new();
    let mut length = None;

    for param in query.split('&') {
        let mut parts = param.splitn(2, '=');
        let key = parts.next().unwrap_or("");
        let val = parts.next().unwrap_or("");
        let decoded = percent_encoding::percent_decode_str(val).decode_utf8_lossy().to_string();

        match key {
            "xt" => {
                if let Some(h) = decoded.strip_prefix("urn:btih:") {
                    hash = h.to_ascii_uppercase();
                }
            }
            "dn" => {
                name = decoded;
            }
            "tr" => {
                trackers.push(decoded);
            }
            "xl" => {
                length = decoded.parse::<u64>().ok();
            }
            _ => {}
        }
    }

    if hash.is_empty() {
        return None;
    }
    if name.is_empty() {
        let prefix = if hash.len() > 8 { &hash[..8] } else { &hash };
        name = format!("torrent_{prefix}");
    }

    Some(MagnetInfo {
        info_hash: hash,
        name,
        trackers,
        exact_length: length,
    })
}

/// Job descriptor for downloading a torrent or magnet link.
pub struct TorrentJob {
    pub uri_or_file: String,
    pub output_dir: PathBuf,
    pub file_name: Option<String>,
    pub connections: u32,
    pub limiter: Arc<SpeedLimiter>,
}

/// Runs the BitTorrent download using aria2c engine with live progress parsing.
pub async fn run_torrent(
    job: TorrentJob,
    progress: Arc<Progress>,
    cancel: CancellationToken,
) -> anyhow::Result<()> {
    let aria2 = find_tool("aria2c").ok_or_else(|| {
        anyhow::anyhow!(
            "محرك التورنت (aria2c.exe) غير متوفر حالياً. سيقوم Zero Download بتحميله تلقائياً."
        )
    })?;

    let mut cmd = Command::new(&aria2);
    cmd.args([
        "--enable-dht=true",
        "--enable-peer-exchange=true",
        "--bt-enable-lpd=true",
        "--follow-torrent=mem",
        "--summary-interval=1",
        "--bt-prioritize-piece=head=15M,tail=5M", // enables instant video preview!
        "--seed-time=0",
        "--max-connection-per-server",
        &job.connections.max(4).to_string(),
        "--dir",
    ])
    .arg(&job.output_dir);

    if let Some(name) = &job.file_name {
        cmd.arg("--out").arg(name);
    }

    cmd.arg(&job.uri_or_file);
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    hide_window(&mut cmd);

    let mut child = cmd.spawn().context("فشل في بدء عملية تحميل التورنت")?;

    let stdout = child.stdout.take();
    let cancel_token = cancel.clone();

    // Spawn stdout parser task
    let parser = std::thread::spawn(move || {
        use std::io::{BufRead, BufReader};
        if let Some(out) = stdout {
            let reader = BufReader::new(out);
            for line in reader.lines().flatten() {
                if cancel_token.is_cancelled() {
                    break;
                }
                // Parse progress line like:
                // [#abc 120MiB/1.5GiB(8%) CN:10 SD:5 DL:1.2MiB/s]
                if let Some((done_bytes, total_bytes)) = parse_progress_line(&line) {
                    progress.bytes.store(done_bytes, Ordering::Relaxed);
                    if total_bytes > 0 {
                        progress.parts_total.store(100, Ordering::Relaxed);
                        let pct = ((done_bytes as f64 / total_bytes as f64) * 100.0) as u32;
                        progress.parts_done.store(pct.min(100), Ordering::Relaxed);
                    }
                }
            }
        }
    });

    let pid = child.id();
    let wait_handle = tokio::task::spawn_blocking(move || child.wait());

    // Monitor for cancellation or child completion
    tokio::select! {
        _ = cancel.cancelled() => {
            #[cfg(windows)]
            {
                let mut kill_cmd = Command::new("taskkill");
                kill_cmd.args(["/F", "/T", "/PID", &pid.to_string()]);
                hide_window(&mut kill_cmd);
                let _ = kill_cmd.output();
            }
            #[cfg(not(windows))]
            {
                let _ = Command::new("kill")
                    .args(["-9", &pid.to_string()])
                    .output();
            }
            let _ = parser.join();
            bail!("تم إيقاف تحميل التورنت مؤقتاً");
        }
        res = wait_handle => {
            let _ = parser.join();
            match res {
                Ok(Ok(status)) if status.success() => Ok(()),
                Ok(Ok(status)) => bail!("انتهى تحميل التورنت مع خطأ: {:?}", status.code()),
                Ok(Err(e)) => Err(e.into()),
                Err(e) => Err(e.into()),
            }
        }
    }
}

/// Parses byte counts from aria2c summary lines.
fn parse_progress_line(line: &str) -> Option<(u64, u64)> {
    if !line.contains("DL:") || !line.contains('/') {
        return None;
    }
    let parts: Vec<&str> = line.split_whitespace().collect();
    for p in parts {
        if p.contains('/') && p.contains('(') {
            // format: 120MiB/1.5GiB(8%)
            let slash = p.find('/')?;
            let paren = p.find('(')?;
            let done_str = &p[..slash];
            let total_str = &p[slash + 1..paren];
            let done = parse_size_str(done_str)?;
            let total = parse_size_str(total_str)?;
            return Some((done, total));
        }
    }
    None
}

fn parse_size_str(s: &str) -> Option<u64> {
    let s = s.trim();
    let re = regex::Regex::new(r"^([0-9.]+)([A-Za-z]+)$").ok()?;
    let caps = re.captures(s)?;
    let val: f64 = caps[1].parse().ok()?;
    let unit = caps[2].to_ascii_lowercase();

    let mult: f64 = match unit.as_str() {
        "b" => 1.0,
        "kib" | "k" | "kb" => 1024.0,
        "mib" | "m" | "mb" => 1024.0 * 1024.0,
        "gib" | "g" | "gb" => 1024.0 * 1024.0 * 1024.0,
        "tib" | "t" | "tb" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => 1.0,
    };
    Some((val * mult) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_magnet() {
        let uri = "magnet:?xt=urn:btih:d2474e86c95b19b8bcfdb92bc12c9d44667cfa36&dn=Ubuntu+22.04&tr=http%3A%2F%2Ftracker.com";
        let info = parse_magnet(uri).unwrap();
        assert_eq!(info.info_hash, "D2474E86C95B19B8BCFDB92BC12C9D44667CFA36");
        assert_eq!(info.name, "Ubuntu 22.04");
        assert_eq!(info.trackers.len(), 1);
    }

    #[test]
    fn test_parse_size_str() {
        assert_eq!(parse_size_str("100MiB"), Some(104857600));
        assert_eq!(parse_size_str("1.5GiB"), Some(1610612736));
    }
}
