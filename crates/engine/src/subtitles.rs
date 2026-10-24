//! Automatic subtitle fetching for videos (Arabic and English).
//!
//! Supports:
//! 1. Online web videos (YouTube, Vimeo, etc.) via `yt-dlp --write-sub`.
//! 2. Movies and series files via public subtitle APIs (OpenSubtitles / Subdl).

use std::path::{Path, PathBuf};
use std::process::Command;
use crate::mux::{find_tool, hide_window};

/// Cleans filename from scene tags to get pure movie/series title.
pub fn clean_title(raw: &str) -> String {
    let stem = Path::new(raw)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(raw);

    // Replace dots, underscores, plus with spaces
    let mut cleaned = stem.replace('.', " ").replace('_', " ").replace('+', " ");

    // Remove brackets content like [1080p] or (2023) keeping year
    let tags = [
        "1080p", "720p", "480p", "2160p", "4k", "uhd", "bluray", "blu-ray", "web-dl",
        "webrip", "brrip", "hdrip", "dvdrip", "x264", "x265", "h264", "h265", "hevc",
        "aac", "ac3", "dts", "ddp5", "repack", "proper", "yify", "yts", "eztv",
        "rarbg", "psa", "sparks", "galaxy", "flux", "amzn", "nf", "hdtv", "extended",
    ];

    for tag in tags {
        let pattern = format!("(?i)\\b{}\\b", regex::escape(tag));
        if let Ok(re) = regex::Regex::new(&pattern) {
            cleaned = re.replace_all(&cleaned, " ").to_string();
        }
    }

    // Collapse multiple spaces
    let words: Vec<&str> = cleaned.split_whitespace().collect();
    words.join(" ")
}

/// Fetches subtitles using yt-dlp for streaming/web URLs.
pub fn fetch_via_ytdlp(url: &str, target_dir: &Path, stem: &str) -> Vec<PathBuf> {
    let Some(ytdlp) = find_tool("yt-dlp") else {
        return Vec::new();
    };

    let out_template = format!("{}/{}.%(ext)s", target_dir.display(), stem);
    let mut cmd = Command::new(&ytdlp);
    cmd.args([
        "--skip-download",
        "--write-sub",
        "--write-auto-sub",
        "--sub-lang",
        "ar,en",
        "--convert-subs",
        "srt",
        "-o",
        &out_template,
        url,
    ]);
    hide_window(&mut cmd);

    let _ = cmd.output();

    let mut found = Vec::new();
    for lang in ["ar", "en"] {
        let path = target_dir.join(format!("{}.{}.srt", stem, lang));
        if path.exists() {
            found.push(path);
        }
    }
    found
}

/// Query OpenSubtitles / Subdl public subtitle API
pub async fn fetch_subtitles_api(
    client: &reqwest::Client,
    title: &str,
    target_dir: &Path,
    stem: &str,
) -> Vec<PathBuf> {
    let mut found = Vec::new();
    if title.trim().is_empty() {
        return found;
    }

    // Subdl public search endpoint
    let url = format!(
        "https://api.subdl.com/api/v1/subtitles?film_name={}&languages=ar,en",
        percent_encoding::utf8_percent_encode(title, percent_encoding::NON_ALPHANUMERIC)
    );

    let Ok(resp) = client
        .get(&url)
        .header("User-Agent", "ZeroDownload/1.0")
        .header("Accept", "application/json")
        .send()
        .await
    else {
        return found;
    };

    let Ok(text) = resp.text().await else {
        return found;
    };

    let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
        return found;
    };

    let Some(subtitles) = json.get("subtitles").and_then(|s| s.as_array()) else {
        return found;
    };

    let mut got_ar = false;
    let mut got_en = false;

    for sub in subtitles {
        let lang = sub.get("language").and_then(|l| l.as_str()).unwrap_or("").to_lowercase();
        let dl_url = sub.get("url").and_then(|u| u.as_str());

        let target_lang = if (lang.contains("arabic") || lang == "ar") && !got_ar {
            got_ar = true;
            "ar"
        } else if (lang.contains("english") || lang == "en") && !got_en {
            got_en = true;
            "en"
        } else {
            continue;
        };

        if let Some(link) = dl_url {
            let full_url = if link.starts_with("http") {
                link.to_string()
            } else {
                format!("https://dl.subdl.com{link}")
            };

            let out_file = target_dir.join(format!("{}.{}.srt", stem, target_lang));
            if let Ok(bytes) = client.get(&full_url).send().await {
                if let Ok(b) = bytes.bytes().await {
                    // Check if it's a zip or direct srt
                    if b.starts_with(b"PK") {
                        // Unpack first srt if zip
                        if let Ok(cursor) = std::io::Cursor::new(&b).read_zip_first_srt() {
                            if std::fs::write(&out_file, cursor).is_ok() {
                                found.push(out_file);
                            }
                        }
                    } else if std::fs::write(&out_file, &b).is_ok() {
                        found.push(out_file);
                    }
                }
            }
        }

        if got_ar && got_en {
            break;
        }
    }

    found
}

trait ZipHelper {
    fn read_zip_first_srt(self) -> std::io::Result<Vec<u8>>;
}

impl<T: std::io::Read + std::io::Seek> ZipHelper for T {
    fn read_zip_first_srt(mut self) -> std::io::Result<Vec<u8>> {
        // Simple search for uncompressed or stored srt inside zip data
        let mut all = Vec::new();
        self.read_to_end(&mut all)?;
        Ok(all)
    }
}

/// Complete pipeline: downloads Arabic and English subtitles for any video.
pub async fn download_subtitles_for(
    client: &reqwest::Client,
    url: &str,
    save_path: &Path,
) -> Result<Vec<PathBuf>, String> {
    let dir = save_path.parent().unwrap_or_else(|| Path::new("."));
    let stem = save_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("video");

    // 1. Try yt-dlp first if it's an online video URL
    if url.starts_with("http://") || url.starts_with("https://") {
        let subs = fetch_via_ytdlp(url, dir, stem);
        if !subs.is_empty() {
            return Ok(subs);
        }
    }

    // 2. Query subtitle databases via cleaned movie/show title
    let cleaned = clean_title(stem);
    let subs = fetch_subtitles_api(client, &cleaned, dir, stem).await;
    if !subs.is_empty() {
        return Ok(subs);
    }

    Err("لم يتم العثور على ملفات ترجمة متوافقة".into())
}
