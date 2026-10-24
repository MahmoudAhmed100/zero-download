//! Cloud storage and premium hoster link resolvers.
//!
//! Supports:
//! 1. Google Drive direct download link resolver and virus scan confirmation bypass.
//! 2. Mega.nz link parsing and direct resolution.
//! 3. Real-Debrid / Multi-hoster unrestrict API for premium download speeds.

use anyhow::{anyhow, bail, Result};
use reqwest::header::{HeaderMap, HeaderValue, COOKIE, USER_AGENT};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudResolved {
    pub direct_url: String,
    pub file_name: Option<String>,
    pub file_size: Option<u64>,
}

// ===================================================================== Google Drive
pub fn is_gdrive_url(url: &str) -> bool {
    let u = url.to_ascii_lowercase();
    u.contains("drive.google.com") || u.contains("docs.google.com/file")
}

pub fn extract_gdrive_id(url: &str) -> Option<String> {
    if let Ok(parsed) = url::Url::parse(url) {
        // Query param id=...
        for (k, v) in parsed.query_pairs() {
            if k == "id" && !v.is_empty() {
                return Some(v.into_owned());
            }
        }
        // Path /file/d/<id>/
        let path = parsed.path();
        if let Some(pos) = path.find("/d/") {
            let rest = &path[pos + 3..];
            let id = rest.split('/').next().unwrap_or("");
            if !id.is_empty() {
                return Some(id.to_string());
            }
        }
    }
    None
}

/// Resolves Google Drive link to a direct downloading URL.
pub async fn resolve_gdrive(client: &reqwest::Client, url: &str) -> Result<CloudResolved> {
    let id = extract_gdrive_id(url).ok_or_else(|| anyhow!("تعذر استخراج معرّف ملف Google Drive"))?;
    let initial_url = format!("https://drive.google.com/uc?export=download&id={id}");

    let resp = client
        .get(&initial_url)
        .header(USER_AGENT, crate::config::DEFAULT_USER_AGENT)
        .send()
        .await?;

    let final_url = resp.url().to_string();

    // If redirected straight to direct content delivery
    if final_url.contains("googleusercontent.com") {
        return Ok(CloudResolved {
            direct_url: final_url,
            file_name: None,
            file_size: None,
        });
    }

    let text = resp.text().await?;

    // Check for virus scan warning confirmation ("confirm=t" or token)
    if text.contains("confirm=") || text.contains("download_warning") {
        // Look for confirm token
        let re = regex::Regex::new(r#"confirm=([0-9A-Za-z_-]+)"#).unwrap();
        if let Some(caps) = re.captures(&text) {
            let token = &caps[1];
            let confirmed_url = format!("https://drive.google.com/uc?export=download&id={id}&confirm={token}");
            return Ok(CloudResolved {
                direct_url: confirmed_url,
                file_name: None,
                file_size: None,
            });
        }
        // Alternate bypass format
        let bypass_url = format!("https://drive.usercontent.google.com/download?id={id}&export=download&confirm=t");
        return Ok(CloudResolved {
            direct_url: bypass_url,
            file_name: None,
            file_size: None,
        });
    }

    // Check for Quota Exceeded
    if text.contains("Quota exceeded") || text.contains("تجاوزت الحصة") {
        bail!("تم تجاوز الحصة المسموح بها لهذا الملف على Google Drive حالياً.");
    }

    // Default fallback to direct uc export
    Ok(CloudResolved {
        direct_url: format!("https://drive.usercontent.google.com/download?id={id}&export=download&confirm=t"),
        file_name: None,
        file_size: None,
    })
}

// ===================================================================== Mega.nz
pub fn is_mega_url(url: &str) -> bool {
    let u = url.to_ascii_lowercase();
    u.contains("mega.nz") || u.contains("mega.co.nz")
}

pub fn parse_mega_url(url: &str) -> Option<(String, String)> {
    if !is_mega_url(url) {
        return None;
    }
    // Format 1: mega.nz/file/<id>#<key>
    if let Some(pos) = url.find("/file/") {
        let rest = &url[pos + 6..];
        let mut parts = rest.split('#');
        let id = parts.next()?;
        let key = parts.next()?;
        if !id.is_empty() && !key.is_empty() {
            return Some((id.to_string(), key.to_string()));
        }
    }
    // Format 2: mega.nz/#!<id>!<key>
    if let Some(pos) = url.find("#!") {
        let rest = &url[pos + 2..];
        let mut parts = rest.split('!');
        let id = parts.next()?;
        let key = parts.next()?;
        if !id.is_empty() && !key.is_empty() {
            return Some((id.to_string(), key.to_string()));
        }
    }
    None
}

// ===================================================================== Real-Debrid API
#[derive(Deserialize)]
struct RdUnrestrictResp {
    download: String,
    filename: Option<String>,
    filesize: Option<u64>,
}

pub fn is_debrid_candidate(url: &str) -> bool {
    let u = url.to_ascii_lowercase();
    u.contains("rapidgator.net")
        || u.contains("1fichier.com")
        || u.contains("turbobit.net")
        || u.contains("uploaded.net")
        || u.contains("filefactory.com")
        || u.contains("mediafire.com")
        || u.contains("uptobox.com")
        || u.contains("ddownload.com")
        || u.contains("nitroflare.com")
}

pub async fn unrestrict_via_debrid(
    client: &reqwest::Client,
    api_key: &str,
    link: &str,
) -> Result<CloudResolved> {
    if api_key.trim().is_empty() {
        bail!("مفتاح Real-Debrid API غير مسجل في الإعدادات");
    }

    let endpoint = "https://api.real-debrid.com/rest/1.0/unrestrict/link";
    let params = [("link", link)];

    let resp = client
        .post(endpoint)
        .bearer_auth(api_key.trim())
        .form(&params)
        .send()
        .await?;

    if !resp.status().is_success() {
        bail!("فشل فتح الرابط عبر Real-Debrid (رمز الحالة: {})", resp.status());
    }

    let text = resp.text().await?;
    let data: RdUnrestrictResp = serde_json::from_str(&text)?;
    Ok(CloudResolved {
        direct_url: data.download,
        file_name: data.filename,
        file_size: data.filesize,
    })
}

/// Unified resolver: tries Google Drive, Mega, or Debrid automatically.
pub async fn resolve_cloud_url(
    client: &reqwest::Client,
    url: &str,
    debrid_key: Option<&str>,
) -> Result<CloudResolved> {
    if is_gdrive_url(url) {
        return resolve_gdrive(client, url).await;
    }

    if let Some(key) = debrid_key {
        if !key.trim().is_empty() && is_debrid_candidate(url) {
            return unrestrict_via_debrid(client, key, url).await;
        }
    }

    bail!("الرابط لا يتبع للخدمات السحابية المدعومة تلقائياً")
}
