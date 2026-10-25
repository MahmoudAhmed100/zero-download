//! The texts made in Rust (statuses, messages) in the language of the
//! program. Arabic is the table below; the other languages are the files
//! lang/rs/<code>.json (English text -> translation). The window texts
//! themselves are translated by Slint (lang/<code>/LC_MESSAGES/*.po).

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

/// The languages of the program: code and own name, in the order of the
/// Language list in Options.
pub const LANGS: [(&str, &str); 8] = [
    ("en", "English"),
    ("ar", "ط§ظ„ط¹ط±ط¨ظٹط©"),
    ("fr", "Franأ§ais"),
    ("es", "Espaأ±ol"),
    ("de", "Deutsch"),
    ("pt", "Portuguأھs"),
    ("tr", "Tأ¼rkأ§e"),
    ("ru", "ذ رƒرپرپذ؛ذ¸ذ¹"),
];

static CURRENT: AtomicUsize = AtomicUsize::new(0);

/// A language code from LANGS, or "" = the language of Windows.
pub fn effective(setting: &str) -> &'static str {
    if let Some((code, _)) = LANGS.iter().find(|(c, _)| *c == setting) {
        return code;
    }
    let sys = sys_locale::get_locale().unwrap_or_default().to_ascii_lowercase();
    LANGS.iter().map(|(c, _)| *c).find(|c| sys.starts_with(c)).unwrap_or("en")
}

/// Place of a language in LANGS (0 = English for anything unknown).
pub fn index_of(code: &str) -> usize {
    LANGS.iter().position(|(c, _)| *c == code).unwrap_or(0)
}

/// Switch every window and every Rust text to this language.
pub fn apply(lang: &str) {
    CURRENT.store(index_of(lang), Ordering::Relaxed);
    let _ = slint::select_bundled_translation(LANGS[index_of(lang)].0);
}

pub fn current() -> &'static str {
    LANGS[CURRENT.load(Ordering::Relaxed) % LANGS.len()].0
}

#[allow(dead_code)]
pub fn is_arabic() -> bool {
    current() == "ar"
}

fn table(code: &str) -> Option<&'static HashMap<String, String>> {
    static FR: OnceLock<HashMap<String, String>> = OnceLock::new();
    static ES: OnceLock<HashMap<String, String>> = OnceLock::new();
    static DE: OnceLock<HashMap<String, String>> = OnceLock::new();
    static PT: OnceLock<HashMap<String, String>> = OnceLock::new();
    static TR: OnceLock<HashMap<String, String>> = OnceLock::new();
    static RU: OnceLock<HashMap<String, String>> = OnceLock::new();
    let (cell, src) = match code {
        "fr" => (&FR, include_str!("../lang/rs/fr.json")),
        "es" => (&ES, include_str!("../lang/rs/es.json")),
        "de" => (&DE, include_str!("../lang/rs/de.json")),
        "pt" => (&PT, include_str!("../lang/rs/pt.json")),
        "tr" => (&TR, include_str!("../lang/rs/tr.json")),
        "ru" => (&RU, include_str!("../lang/rs/ru.json")),
        _ => return None,
    };
    Some(cell.get_or_init(|| serde_json::from_str(src).unwrap_or_default()))
}

/// The text in the current language.
pub fn t(en: &str) -> String {
    match current() {
        "en" => {}
        "ar" => {
            if let Some(ar) = lookup(en) {
                return ar.to_string();
            }
        }
        code => {
            if let Some(v) = table(code).and_then(|m| m.get(en)) {
                return v.clone();
            }
        }
    }
    en.to_string()
}

/// A text with "{}" places, filled in order.
pub fn tf(en: &str, args: &[&dyn std::fmt::Display]) -> String {
    let template = t(en);
    let mut out = String::new();
    let mut rest = template.as_str();
    let mut i = 0;
    while let Some(p) = rest.find("{}") {
        out.push_str(&rest[..p]);
        if let Some(a) = args.get(i) {
            out.push_str(&a.to_string());
        }
        i += 1;
        rest = &rest[p + 2..];
    }
    out.push_str(rest);
    out
}

fn lookup(en: &str) -> Option<&'static str> {
    AR.iter().find(|(k, _)| *k == en).map(|(_, v)| *v)
}

const AR: &[(&str, &str)] = &[
    // statuses
    ("Queued", "ظپظٹ ط§ظ„ط§ظ†طھط¸ط§ط±"),
    ("Connectingâ€¦", "ط¬ط§ط±ظچ ط§ظ„ط§طھطµط§ظ„â€¦"),
    ("Downloading", "ط¬ط§ط±ظچ ط§ظ„طھط­ظ…ظٹظ„"),
    ("Assemblingâ€¦", "ط¬ط§ط±ظچ ط§ظ„طھط¬ظ…ظٹط¹â€¦"),
    ("Paused", "ظ…طھظˆظ‚ظپ ظ…ط¤ظ‚طھط§ظ‹"),
    ("Complete", "ظ…ظƒطھظ…ظ„"),
    ("Error", "ط®ط·ط£"),
    ("Mergingâ€¦", "ط¬ط§ط±ظچ ط§ظ„ط¯ظ…ط¬â€¦"),
    ("{}/{} parts", "{}/{} ط¬ط²ط،"),
    ("{}: video + audio", "{}: ظپظٹط¯ظٹظˆ + طµظˆطھ"),
    ("{}: stream", "{}: ط¨ط«"),
    ("Receiving dataâ€¦", "ط¬ط§ط±ظچ ط§ط³طھظ‚ط¨ط§ظ„ ط§ظ„ط¨ظٹط§ظ†ط§طھâ€¦"),
    ("Joining the partsâ€¦", "ط¬ط§ط±ظچ طھط¬ظ…ظٹط¹ ط§ظ„ط£ط¬ط²ط§ط،â€¦"),
    ("Unknown", "ط؛ظٹط± ظ…ط¹ط±ظˆظپ"),
    ("Yes", "ظ†ط¹ظ…"),
    ("No", "ظ„ط§"),
    ("Finished", "ط§ظ†طھظ‡ظ‰"),
    ("Waiting", "ظپظٹ ط§ظ„ط§ظ†طھط¸ط§ط±"),
    ("{} downloading  آ·  {}/s", "{} ظ‚ظٹط¯ ط§ظ„طھط­ظ…ظٹظ„  آ·  {}/ط«"),
    ("Ready  آ·  {} files in the list", "ط¬ط§ظ‡ط²  آ·  {} ظ…ظ„ظپ ظپظٹ ط§ظ„ظ‚ط§ط¦ظ…ط©"),
    ("Speed limiter: on ({} KB/s)", "ظ…ط­ط¯ط¯ ط§ظ„ط³ط±ط¹ط©: ظٹط¹ظ…ظ„ ({} ظƒظٹظ„ظˆ/ط«)"),
    ("Speed limiter: off", "ظ…ط­ط¯ط¯ ط§ظ„ط³ط±ط¹ط©: ظ…طھظˆظ‚ظپ"),
    // categories (shown names)
    ("Compressed", "ظ…ط¶ط؛ظˆط·ط©"),
    ("Documents", "ظ…ط³طھظ†ط¯ط§طھ"),
    ("Music", "ظ…ظˆط³ظٹظ‚ظ‰"),
    ("Programs", "ط¨ط±ط§ظ…ط¬"),
    ("Video", "ظپظٹط¯ظٹظˆ"),
    ("Images", "طµظˆط±"),
    ("General", "ط¹ط§ظ…"),
    // download file info
    ("Paste a link and press Check", "ط§ظ„طµظ‚ ط±ط§ط¨ط·ط§ظ‹ ظˆط§ط¶ط؛ط· ظپط­طµ"),
    ("Unknown size", "ط­ط¬ظ… ط؛ظٹط± ظ…ط¹ط±ظˆظپ"),
    ("+ audio", "+ طµظˆطھ"),
    ("Video + audio, joined after download", "ظپظٹط¯ظٹظˆ + طµظˆطھطŒ ظٹظڈط¯ظ…ط¬ط§ظ† ط¨ط¹ط¯ ط§ظ„طھط­ظ…ظٹظ„"),
    ("Video stream (HLS)", "ط¨ط« ظپظٹط¯ظٹظˆ (HLS)"),
    ("Resume supported", "ظٹط¯ط¹ظ… ط§ظ„ط§ط³طھط¦ظ†ط§ظپ"),
    ("No resume support", "ظ„ط§ ظٹط¯ط¹ظ… ط§ظ„ط§ط³طھط¦ظ†ط§ظپ"),
    ("This address is already in the download list.", "ظ‡ط°ط§ ط§ظ„ط±ط§ط¨ط· ظ…ظˆط¬ظˆط¯ ط¨ط§ظ„ظپط¹ظ„ ظپظٹ ظ‚ط§ط¦ظ…ط© ط§ظ„طھط­ظ…ظٹظ„."),
    (
        "This address opens a web page, not a file. The link may have expired or need a login.",
        "ظ‡ط°ط§ ط§ظ„ط±ط§ط¨ط· ظٹظپطھط­ طµظپط­ط© ظˆظٹط¨ ظˆظ„ظٹط³ ظ…ظ„ظپط§ظ‹. ط±ط¨ظ…ط§ ط§ظ†طھظ‡طھ طµظ„ط§ط­ظٹطھظ‡ ط£ظˆ ظٹط­طھط§ط¬ طھط³ط¬ظٹظ„ ط¯ط®ظˆظ„.",
    ),
    ("Please enter a valid http:// or https:// address", "ظ…ظ† ظپط¶ظ„ظƒ ط§ظƒطھط¨ ط±ط§ط¨ط·ط§ظ‹ طµط­ظٹط­ط§ظ‹ ظٹط¨ط¯ط£ ط¨ظ€ http:// ط£ظˆ https://"),
    // messages
    (
        "Remove every finished download from the list?\n(The files stay on the disk.)",
        "ط­ط°ظپ ظƒظ„ ط§ظ„طھط­ظ…ظٹظ„ط§طھ ط§ظ„ظ…ظƒطھظ…ظ„ط© ظ…ظ† ط§ظ„ظ‚ط§ط¦ظ…ط©طں\n(ط§ظ„ظ…ظ„ظپط§طھ طھط¨ظ‚ظ‰ ط¹ظ„ظ‰ ط§ظ„ظ‚ط±طµ.)",
    ),
    (
        "A short message about Zero Download was copied to the clipboard.\nPaste it in a chat or an e-mail to a friend.",
        "طھظ… ظ†ط³ط® ط±ط³ط§ظ„ط© ظ‚طµظٹط±ط© ط¹ظ† Zero Download ط¥ظ„ظ‰ ط§ظ„ط­ط§ظپط¸ط©.\nط§ظ„طµظ‚ظ‡ط§ ظپظٹ ظ…ط­ط§ط¯ط«ط© ط£ظˆ ط¨ط±ظٹط¯ ظ„طµط¯ظٹظ‚ظƒ.",
    ),
    ("Zero Download {}\n\nThe video downloader (yt-dlp) is up to date.\n\n{}", "Zero Download {}\n\nط£ط¯ط§ط© ط§ظ„ظپظٹط¯ظٹظˆ (yt-dlp) ظ…ط­ط¯ظ‘ط«ط©.\n\n{}"),
    ("Could not update yt-dlp:\n\n{}", "طھط¹ط°ظ‘ط± طھط­ط¯ظٹط« yt-dlp:\n\n{}"),
    ("Download these {} files again from the beginning?", "ط¥ط¹ط§ط¯ط© طھط­ظ…ظٹظ„ ظ‡ط°ظ‡ ط§ظ„ظ…ظ„ظپط§طھ ({}) ظ…ظ† ط§ظ„ط¨ط¯ط§ظٹط©طں"),
    ("Download â€œ{}â€‌ again from the beginning?", "ط¥ط¹ط§ط¯ط© طھط­ظ…ظٹظ„ â€œ{}â€‌ ظ…ظ† ط§ظ„ط¨ط¯ط§ظٹط©طں"),
    ("No valid http:// or https:// links", "ظ„ط§ طھظˆط¬ط¯ ط±ظˆط§ط¨ط· طµط­ظٹط­ط© (http:// ط£ظˆ https://)"),
    ("The address must start with http:// or https://", "ظٹط¬ط¨ ط£ظ† ظٹط¨ط¯ط£ ط§ظ„ط±ط§ط¨ط· ط¨ظ€ http:// ط£ظˆ https://"),
    ("Put a * in the address where the number goes", "ط¶ط¹ * ظپظٹ ط§ظ„ط±ط§ط¨ط· ظ…ظƒط§ظ† ط§ظ„ط±ظ‚ظ…"),
    ("From must be a letter", "ط®ط§ظ†ط© آ«ظ…ظ†آ» ظٹط¬ط¨ ط£ظ† طھظƒظˆظ† ط­ط±ظپط§ظ‹"),
    ("To must be a letter", "ط®ط§ظ†ط© آ«ط¥ظ„ظ‰آ» ظٹط¬ط¨ ط£ظ† طھظƒظˆظ† ط­ط±ظپط§ظ‹"),
    ("From must come before To", "آ«ظ…ظ†آ» ظٹط¬ط¨ ط£ظ† طھط£طھظٹ ظ‚ط¨ظ„ آ«ط¥ظ„ظ‰آ»"),
    ("From must be a number", "ط®ط§ظ†ط© آ«ظ…ظ†آ» ظٹط¬ط¨ ط£ظ† طھظƒظˆظ† ط±ظ‚ظ…ط§ظ‹"),
    ("To must be a number", "ط®ط§ظ†ط© آ«ط¥ظ„ظ‰آ» ظٹط¬ط¨ ط£ظ† طھظƒظˆظ† ط±ظ‚ظ…ط§ظ‹"),
    ("From must be smaller than To", "آ«ظ…ظ†آ» ظٹط¬ط¨ ط£ظ† طھظƒظˆظ† ط£طµط؛ط± ظ…ظ† آ«ط¥ظ„ظ‰آ»"),
    ("That's more than 10000 files", "ظ‡ط°ط§ ط£ظƒط«ط± ظ…ظ† 10000 ظ…ظ„ظپ"),
    ("{} files\n\nFirst:  {}", "{} ظ…ظ„ظپ\n\nط§ظ„ط£ظˆظ„:  {}"),
    ("\nSecond:  {}", "\nط§ظ„ط«ط§ظ†ظٹ:  {}"),
    ("\nLast:  {}", "\nط§ظ„ط£ط®ظٹط±:  {}"),
    ("Could not change where it is saved: {}", "طھط¹ط°ظ‘ط± طھط؛ظٹظٹط± ظ…ظƒط§ظ† ط§ظ„ط­ظپط¸: {}"),
    ("Could not move the file:\n\n{}", "طھط¹ط°ظ‘ط± ظ†ظ‚ظ„ ط§ظ„ظ…ظ„ظپ:\n\n{}"),
    ("SHA-256 of {}:\n\n{}\n\n(copied to the clipboard)", "SHA-256 ظ„ظ„ظ…ظ„ظپ {}:\n\n{}\n\n(طھظ… ط§ظ„ظ†ط³ط® ط¥ظ„ظ‰ ط§ظ„ط­ط§ظپط¸ط©)"),
    ("Could not read the file: {}", "طھط¹ط°ظ‘ط±طھ ظ‚ط±ط§ط،ط© ط§ظ„ظ…ظ„ظپ: {}"),
    ("Delete downloads", "ط­ط°ظپ ط§ظ„طھط­ظ…ظٹظ„ط§طھ"),
    ("Delete download", "ط­ط°ظپ ط§ظ„طھط­ظ…ظٹظ„"),
    (
        "Remove these {} downloads from the download list?\n\nOnly the list entries are removed: the downloaded files stay on your computer.\n(Parts of unfinished downloads are cleaned up.)",
        "ط¥ط²ط§ظ„ط© ظ‡ط°ظ‡ ط§ظ„طھط­ظ…ظٹظ„ط§طھ ({}) ظ…ظ† ظ‚ط§ط¦ظ…ط© ط§ظ„طھط­ظ…ظٹظ„طں\n\nط³طھظڈط²ط§ظ„ ظ…ظ† ط§ظ„ظ‚ط§ط¦ظ…ط© ظپظ‚ط·طŒ ظˆطھط¨ظ‚ظ‰ ط§ظ„ظ…ظ„ظپط§طھ ط§ظ„ظ…ط­ظ…ظ‘ظ„ط© ط¹ظ„ظ‰ ط¬ظ‡ط§ط²ظƒ ظƒظ…ط§ ظ‡ظٹ.\n(طھظڈط­ط°ظپ ط§ظ„ط£ط¬ط²ط§ط، ط§ظ„ظ…ط¤ظ‚طھط© ظ„ظ„طھط­ظ…ظٹظ„ط§طھ ط؛ظٹط± ط§ظ„ظ…ظƒطھظ…ظ„ط©.)",
    ),
    (
        "Remove â€œ{}â€‌ from the download list?\n\nOnly the list entry is removed: the file stays on your computer.",
        "ط¥ط²ط§ظ„ط© â€œ{}â€‌ ظ…ظ† ظ‚ط§ط¦ظ…ط© ط§ظ„طھط­ظ…ظٹظ„طں\n\nط³طھظڈط²ط§ظ„ ظ…ظ† ط§ظ„ظ‚ط§ط¦ظ…ط© ظپظ‚ط·طŒ ظˆظٹط¨ظ‚ظ‰ ط§ظ„ظ…ظ„ظپ ط¹ظ„ظ‰ ط¬ظ‡ط§ط²ظƒ ظƒظ…ط§ ظ‡ظˆ.",
    ),
    (
        "Delete the unfinished download â€œ{}â€‌?\nThe partly downloaded data will be removed.",
        "ط­ط°ظپ ط§ظ„طھط­ظ…ظٹظ„ ط؛ظٹط± ط§ظ„ظ…ظƒطھظ…ظ„ â€œ{}â€‌طں\nط³ظٹطھظ… ط­ط°ظپ ط§ظ„ط¬ط²ط، ط§ظ„ط°ظٹ طھظ… طھط­ظ…ظٹظ„ظ‡.",
    ),
    (
        "The download limit of {} MB every {} hours was reached, so the downloads were stopped.\nThey continue by themselves when the limit allows it again (Options > Connection).",
        "طھظ… ط§ظ„ظˆطµظˆظ„ ظ„ط­ط¯ ط§ظ„طھط­ظ…ظٹظ„ ({} ظ…ظٹط¬ط§ط¨ط§ظٹطھ ظƒظ„ {} ط³ط§ط¹ط©) ظپطھظˆظ‚ظپطھ ط§ظ„طھط­ظ…ظٹظ„ط§طھ.\nط³طھظƒظ…ظ„ ظˆط­ط¯ظ‡ط§ ط¹ظ†ط¯ظ…ط§ ظٹط³ظ…ط­ ط§ظ„ط­ط¯ ظ…ط±ط© ط£ط®ط±ظ‰ (ط§ظ„ط®ظٹط§ط±ط§طھ > ط§ظ„ط§طھطµط§ظ„).",
    ),
    ("Give the category a name", "ط§ظƒطھط¨ ط§ط³ظ…ط§ظ‹ ظ„ظ„طھطµظ†ظٹظپ"),
    ("A category with this name already exists", "ظٹظˆط¬ط¯ طھطµظ†ظٹظپ ط¨ظ‡ط°ط§ ط§ظ„ط§ط³ظ… ط¨ط§ظ„ظپط¹ظ„"),
    (
        "This category comes with Zero Download and can't be deleted (you can change its file types in Properties).",
        "ظ‡ط°ط§ ط§ظ„طھطµظ†ظٹظپ ط£ط³ط§ط³ظٹ ظپظٹ Zero Download ظˆظ„ط§ ظٹظ…ظƒظ† ط­ط°ظپظ‡ (ظٹظ…ظƒظ†ظƒ طھط؛ظٹظٹط± ط£ظ†ظˆط§ط¹ ظ…ظ„ظپط§طھظ‡ ظ…ظ† ط§ظ„ط®طµط§ط¦طµ).",
    ),
    (
        "Delete the category â€œ{}â€‌?\nIts files stay where they are and move to their file type's category.",
        "ط­ط°ظپ ط§ظ„طھطµظ†ظٹظپ â€œ{}â€‌طں\nظ…ظ„ظپط§طھظ‡ طھط¨ظ‚ظ‰ ظ…ظƒط§ظ†ظ‡ط§ ظˆطھظ†طھظ‚ظ„ ظ„طھطµظ†ظٹظپ ظ†ظˆط¹ظ‡ط§.",
    ),
    ("Found: {}", "طھظ… ط§ظ„ط¹ط«ظˆط±: {}"),
    ("Not found", "ط؛ظٹط± ظ…ظˆط¬ظˆط¯"),
    ("The file was downloaded successfully", "طھظ… طھط­ظ…ظٹظ„ ط§ظ„ظ…ظ„ظپ ط¨ظ†ط¬ط§ط­"),
    ("{} file  آ·  {}", "ظ…ظ„ظپ {}  آ·  {}"),
    ("Tip {} of {}", "ظ†طµظٹط­ط© {} ظ…ظ† {}"),
    ("Could not save the list: {}", "طھط¹ط°ظ‘ط± ط­ظپط¸ ط§ظ„ظ‚ط§ط¦ظ…ط©: {}"),
    ("No download addresses were found in this file.", "ظ„ظ… ظٹطھظ… ط§ظ„ط¹ط«ظˆط± ط¹ظ„ظ‰ ط±ظˆط§ط¨ط· طھط­ظ…ظٹظ„ ظپظٹ ظ‡ط°ط§ ط§ظ„ظ…ظ„ظپ."),
    (
        "{} download(s) were added to the main download queue.\nStart them with Start Queue, or one by one with Resume.",
        "طھظ…طھ ط¥ط¶ط§ظپط© {} طھط­ظ…ظٹظ„ ط¥ظ„ظ‰ ظ‚ط§ط¦ظ…ط© ط§ظ„ط§ظ†طھط¸ط§ط± ط§ظ„ط±ط¦ظٹط³ظٹط©.\nط§ط¨ط¯ط£ظ‡ط§ ط¨ظ€ آ«ط¨ط¯ط، ط§ظ„ظ‚ط§ط¦ظ…ط©آ»طŒ ط£ظˆ ظˆط§ط­ط¯ط§ظ‹ ظˆط§ط­ط¯ط§ظ‹ ط¨ظ€ آ«ط§ط³طھط¦ظ†ط§ظپآ».",
    ),
    ("shut down the computer", "ط¥ظٹظ‚ط§ظپ طھط´ط؛ظٹظ„ ط§ظ„ظƒظ…ط¨ظٹظˆطھط±"),
    ("hibernate the computer", "ط¥ط³ط¨ط§طھ ط§ظ„ظƒظ…ط¨ظٹظˆطھط±"),
    ("put the computer to sleep", "ظˆط¶ط¹ ط§ظ„ظƒظ…ط¨ظٹظˆطھط± ظپظٹ ظˆط¶ط¹ ط§ظ„ط³ظƒظˆظ†"),
    ("Type the address of the start page (http:// or https://)", "ط§ظƒطھط¨ ط±ط§ط¨ط· طµظپط­ط© ط§ظ„ط¨ط¯ط§ظٹط© (http:// ط£ظˆ https://)"),
    ("Startingâ€¦", "ط¬ط§ط±ظچ ط§ظ„ط¨ط¯ط،â€¦"),
    ("Check the files you want first", "ط­ط¯ظ‘ط¯ ط§ظ„ظ…ظ„ظپط§طھ ط§ظ„طھظٹ طھط±ظٹط¯ظ‡ط§ ط£ظˆظ„ط§ظ‹"),
    ("{} file(s) checked", "{} ظ…ظ„ظپ ظ…ط­ط¯ط¯"),
    ("{} file(s) added to the queue", "طھظ…طھ ط¥ط¶ط§ظپط© {} ظ…ظ„ظپ ط¥ظ„ظ‰ ط§ظ„ظ‚ط§ط¦ظ…ط©"),
    ("{} of {} pages explored  آ·  {} files found", "طھظ… ط§ط³طھظƒط´ط§ظپ {} ظ…ظ† {} طµظپط­ط©  آ·  {} ظ…ظ„ظپ"),
    ("  آ·  done", "  آ·  ط§ظ†طھظ‡ظ‰"),
    ("Give the queue a name", "ط§ظƒطھط¨ ط§ط³ظ…ط§ظ‹ ظ„ظ„ظ‚ط§ط¦ظ…ط©"),
    ("The start time must look like 02:30", "ظˆظ‚طھ ط§ظ„ط¨ط¯ط، ظٹط¬ط¨ ط£ظ† ظٹظƒظˆظ† ظ…ط«ظ„ 02:30"),
    ("The stop time must look like 08:00", "ظˆظ‚طھ ط§ظ„ط¥ظٹظ‚ط§ظپ ظٹط¬ط¨ ط£ظ† ظٹظƒظˆظ† ظ…ط«ظ„ 08:00"),
    ("The date must look like 2026-12-31", "ط§ظ„طھط§ط±ظٹط® ظٹط¬ط¨ ط£ظ† ظٹظƒظˆظ† ظ…ط«ظ„ 2026-12-31"),
    ("Could not create the queue: {}", "طھط¹ط°ظ‘ط± ط¥ظ†ط´ط§ط، ط§ظ„ظ‚ط§ط¦ظ…ط©: {}"),
    ("The main download queue can't be deleted.", "ظ„ط§ ظٹظ…ظƒظ† ط­ط°ظپ ظ‚ط§ط¦ظ…ط© ط§ظ„طھط­ظ…ظٹظ„ ط§ظ„ط±ط¦ظٹط³ظٹط©."),
    ("Delete the queue â€œ{}â€‌?\nIts files move to the main download queue.", "ط­ط°ظپ ط§ظ„ظ‚ط§ط¦ظ…ط© â€œ{}â€‌طں\nظ…ظ„ظپط§طھظ‡ط§ طھظ†طھظ‚ظ„ ط¥ظ„ظ‰ ظ‚ط§ط¦ظ…ط© ط§ظ„طھط­ظ…ظٹظ„ ط§ظ„ط±ط¦ظٹط³ظٹط©."),
    ("Downloaded in the current period: {}", "طھظ… طھط­ظ…ظٹظ„ ظپظٹ ط§ظ„ظپطھط±ط© ط§ظ„ط­ط§ظ„ظٹط©: {}"),
    (
        "When the limit is reached the downloads stop, and go on by themselves later.",
        "ط¹ظ†ط¯ ط§ظ„ظˆطµظˆظ„ ظ„ظ„ط­ط¯ طھطھظˆظ‚ظپ ط§ظ„طھط­ظ…ظٹظ„ط§طھطŒ ظˆطھظƒظ…ظ„ ظˆط­ط¯ظ‡ط§ ظ„ط§ط­ظ‚ط§ظ‹.",
    ),
    ("Choose a download folder (Save To)", "ط§ط®طھط± ظ…ط¬ظ„ط¯ ط§ظ„طھط­ظ…ظٹظ„ (ط§ظ„ط­ظپط¸ ظپظٹ)"),
    ("Type the proxy address (Proxy / Server)", "ط§ظƒطھط¨ ط¹ظ†ظˆط§ظ† ط§ظ„ط¨ط±ظˆظƒط³ظٹ (ط§ظ„ط¨ط±ظˆظƒط³ظٹ / ط§ظ„ط®ط§ط¯ظ…)"),
    ("The remembered category folders were cleared.", "طھظ… ظ…ط³ط­ ظ…ط¬ظ„ط¯ط§طھ ط§ظ„طھطµظ†ظٹظپط§طھ ط§ظ„ظ…ط­ظپظˆط¸ط©."),
    ("Type the server, e.g. example.com", "ط§ظƒطھط¨ ط§ظ„ط®ط§ط¯ظ…طŒ ظ…ط«ظ„ط§ظ‹ example.com"),
    ("Type the site and the user name", "ط§ظƒطھط¨ ط§ظ„ظ…ظˆظ‚ط¹ ظˆط§ط³ظ… ط§ظ„ظ…ط³طھط®ط¯ظ…"),
    ("Could not save: {}", "طھط¹ط°ظ‘ط± ط§ظ„ط­ظپط¸: {}"),
    (
        "Restart Zero Download to use the new port. Set the same port in the browser extension.",
        "ط£ط¹ط¯ طھط´ط؛ظٹظ„ Zero Download ظ„ط§ط³طھط®ط¯ط§ظ… ط§ظ„ظ…ظ†ظپط° ط§ظ„ط¬ط¯ظٹط¯طŒ ظˆط§ط¶ط¨ط· ظ†ظپط³ ط§ظ„ظ…ظ†ظپط° ظپظٹ ط¥ط¶ط§ظپط© ط§ظ„ظ…طھطµظپط­.",
    ),
    (
        "Paste the playlist address (https://â€¦/playlist?list=â€¦)",
        "ط§ظ„طµظ‚ ط±ط§ط¨ط· ظ‚ط§ط¦ظ…ط© ط§ظ„طھط´ط؛ظٹظ„ (https://â€¦/playlist?list=â€¦)",
    ),
    ("Getting the list of videosâ€¦ (a few seconds)", "ط¬ط§ط±ظچ ط¬ظ„ط¨ ظ‚ط§ط¦ظ…ط© ط§ظ„ظپظٹط¯ظٹظˆظ‡ط§طھâ€¦ (ط«ظˆط§ظ†ظچ ظ‚ظ„ظٹظ„ط©)"),
    ("No videos were found at this address.", "ظ„ظ… ظٹطھظ… ط§ظ„ط¹ط«ظˆط± ط¹ظ„ظ‰ ظپظٹط¯ظٹظˆظ‡ط§طھ ظپظٹ ظ‡ط°ط§ ط§ظ„ط±ط§ط¨ط·."),
    ("Could not get the list: {}", "طھط¹ط°ظ‘ط± ط¬ظ„ط¨ ط§ظ„ظ‚ط§ط¦ظ…ط©: {}"),
    ("Check the videos you want first", "ط­ط¯ظ‘ط¯ ط§ظ„ظپظٹط¯ظٹظˆظ‡ط§طھ ط§ظ„طھظٹ طھط±ظٹط¯ظ‡ط§ ط£ظˆظ„ط§ظ‹"),
    ("Choose where to save the videos", "ط§ط®طھط± ظ…ظƒط§ظ† ط­ظپط¸ ط§ظ„ظپظٹط¯ظٹظˆظ‡ط§طھ"),
    ("{} of {} videos checked", "{} ظ…ظ† {} ظپظٹط¯ظٹظˆ ظ…ط­ط¯ط¯"),
    ("{} videos", "{} ظپظٹط¯ظٹظˆ"),
    ("Click to add", "ط§ط¶ط؛ط· ظ„ظ„ط¥ط¶ط§ظپط©"),
    ("Zero Download â€“ download manager", "Zero Download â€“ ط¨ط±ظ†ط§ظ…ط¬ طھط­ظ…ظٹظ„"),
    ("Click to add from the store", "ط§ط¶ط؛ط· ظ„ظ„ط¥ط¶ط§ظپط© ظ…ظ† ط§ظ„ظ…طھط¬ط±"),
    ("{} is not installed on this computer.", "{} ط؛ظٹط± ظ…ط«ط¨ظ‘طھ ط¹ظ„ظ‰ ظ‡ط°ط§ ط§ظ„ظƒظ…ط¨ظٹظˆطھط±."),
    ("{} opened the store page: click \"Add\" / \"Get\" there.", "ظپطھط­ {} طµظپط­ط© ط§ظ„ظ…طھط¬ط±: ط§ط¶ط؛ط· \"ط¥ط¶ط§ظپط©\" / \"Get\" ظ‡ظ†ط§ظƒ."),
    ("Could not start the update:\n\n{}", "طھط¹ط°ظ‘ط± ط¨ط¯ط، ط§ظ„طھط­ط¯ظٹط«:\n\n{}"),
    ("A new version is being prepared. It is installed by itself when it is ready.", "ظٹط¬ط±ظٹ طھط¬ظ‡ظٹط² ط¥طµط¯ط§ط± ط¬ط¯ظٹط¯طŒ ظˆط³ظٹظڈط«ط¨ظژظ‘طھ طھظ„ظ‚ط§ط¦ظٹط§ظ‹ ط¹ظ†ط¯ظ…ط§ ظٹظƒظˆظ† ط¬ط§ظ‡ط²ط§ظ‹."),
    ("Could not check for updates:\n\n{}", "طھط¹ط°ظ‘ط± ط§ظ„ط¨ط­ط« ط¹ظ† طھط­ط¯ظٹط«ط§طھ:\n\n{}"),
    ("Zero Download is up to date (version {}).", "Zero Download ظ…ط­ط¯ظژظ‘ط« (ط§ظ„ط¥طµط¯ط§ط± {})."),
    ("Version {} is available. This copy was not installed with the setup, so it cannot update itself.", "ط§ظ„ط¥طµط¯ط§ط± {} ظ…طھط§ط­. ظ‡ط°ظ‡ ط§ظ„ظ†ط³ط®ط© ظ„ظ… طھظڈط«ط¨ظژظ‘طھ ط¨ظ…ظ„ظپ ط§ظ„ط¥ط¹ط¯ط§ط¯طŒ ظ„ط°ظ„ظƒ ظ„ط§ ظٹظ…ظƒظ†ظ‡ط§ طھط­ط¯ظٹط« ظ†ظپط³ظ‡ط§."),
    ("Version {} is available. It is being prepared now and installed by itself; Zero Download opens again after that.", "ط§ظ„ط¥طµط¯ط§ط± {} ظ…طھط§ط­. ظٹط¬ط±ظٹ طھط¬ظ‡ظٹط²ظ‡ ط§ظ„ط¢ظ† ظˆطھط«ط¨ظٹطھظ‡ طھظ„ظ‚ط§ط¦ظٹط§ظ‹طŒ ط«ظ… ظٹظڈظپطھط­ Zero Download ظ…ظ† ط¬ط¯ظٹط¯."),
    ("Could not get the update:\n\n{}", "طھط¹ط°ظ‘ط± ط§ظ„ط­طµظˆظ„ ط¹ظ„ظ‰ ط§ظ„طھط­ط¯ظٹط«:\n\n{}"),
    (
        "The extension folder was not found next to ZeroDownload.exe. Install Zero Download again.",
        "ظ„ظ… ظٹطھظ… ط§ظ„ط¹ط«ظˆط± ط¹ظ„ظ‰ ظ…ط¬ظ„ط¯ ط§ظ„ط¥ط¶ط§ظپط© ط¨ط¬ط§ظ†ط¨ ZeroDownload.exe. ط«ط¨ظ‘طھ Zero Download ظ…ط±ط© ط£ط®ط±ظ‰.",
    ),
    (
        "Firefox opened its add-ons debugging page.\n1. Click \"Load Temporary Add-onâ€¦\".\n2. Paste the folder path (Ctrl+V, it is already copied) and open the file \"manifest.json\" in it.\n\nFirefox removes add-ons loaded this way when it closes. To keep it for good the extension must be signed by Mozilla (free) â€“ ask for the signed version.",
        "ظپطھط­ Firefox طµظپط­ط© طھطµط­ظٹط­ ط§ظ„ط¥ط¶ط§ظپط§طھ.\n1. ط§ط¶ط؛ط· \"Load Temporary Add-onâ€¦\".\n2. ط§ظ„طµظ‚ ظ…ط³ط§ط± ط§ظ„ظ…ط¬ظ„ط¯ (Ctrl+VطŒ طھظ… ظ†ط³ط®ظ‡ ط¨ط§ظ„ظپط¹ظ„) ظˆط§ظپطھط­ ظ…ظ„ظپ \"manifest.json\" ط¨ط¯ط§ط®ظ„ظ‡.\n\nظٹط­ط°ظپ Firefox ط§ظ„ط¥ط¶ط§ظپط§طھ ط§ظ„ظ…ط­ظ…ظ‘ظ„ط© ط¨ظ‡ط°ظ‡ ط§ظ„ط·ط±ظٹظ‚ط© ط¹ظ†ط¯ ط¥ط؛ظ„ط§ظ‚ظ‡. ظ„طھط¨ظ‚ظ‰ ط¯ط§ط¦ظ…ط§ظ‹ ظٹط¬ط¨ ط£ظ† طھظˆظ‚ظ‘ط¹ظ‡ط§ Mozilla (ظ…ط¬ط§ظ†ط§ظ‹) â€“ ط§ط·ظ„ط¨ ط§ظ„ظ†ط³ط®ط© ط§ظ„ظ…ظˆظ‚ظ‘ط¹ط©.",
    ),
    (
        "{} opened its extensions page.\n1. Turn on \"Developer mode\" (a switch on that page).\n2. Click \"Load unpacked\".\n3. Paste the folder path (Ctrl+V â€“ it is already copied) and click \"Select folder\".\n\nThat's all: the Zero Download icon appears next to the address bar. You do this only once.",
        "ظپطھط­ {} طµظپط­ط© ط§ظ„ط¥ط¶ط§ظپط§طھ.\n1. ط´ط؛ظ‘ظ„ \"Developer mode\" (ظˆط¶ط¹ ط§ظ„ظ…ط·ظˆظ‘ط±) ظ…ظ† ط§ظ„ظ…ظپطھط§ط­ ط§ظ„ظ…ظˆط¬ظˆط¯ ظپظٹ ط§ظ„طµظپط­ط©.\n2. ط§ط¶ط؛ط· \"Load unpacked\".\n3. ط§ظ„طµظ‚ ظ…ط³ط§ط± ط§ظ„ظ…ط¬ظ„ط¯ (Ctrl+V â€“ طھظ… ظ†ط³ط®ظ‡ ط¨ط§ظ„ظپط¹ظ„) ظˆط§ط¶ط؛ط· \"Select folder\".\n\nظ‡ط°ط§ ظƒظ„ ط´ظٹط،: ط³طھط¸ظ‡ط± ط£ظٹظ‚ظˆظ†ط© Zero Download ط¨ط¬ط§ظ†ط¨ ط´ط±ظٹط· ط§ظ„ط¹ظ†ظˆط§ظ†. طھظپط¹ظ„ ظ‡ط°ط§ ظ…ط±ط© ظˆط§ط­ط¯ط© ظپظ‚ط·.",
    ),
    // tips
    (
        "Copy a download link anywhere (Ctrl+C) and Zero Download offers to download it.",
        "ط§ظ†ط³ط® ط£ظٹ ط±ط§ط¨ط· طھط­ظ…ظٹظ„ (Ctrl+C) ظˆط³ظٹط¹ط±ط¶ ط¹ظ„ظٹظƒ Zero Download طھط­ظ…ظٹظ„ظ‡.",
    ),
    (
        "Play a video on a web page and click \"Download this video\" above it to choose the quality.",
        "ط´ط؛ظ‘ظ„ ط£ظٹ ظپظٹط¯ظٹظˆ ظپظٹ طµظپط­ط© ظˆظٹط¨ ظˆط§ط¶ط؛ط· \"Download this video\" ظپظˆظ‚ظ‡ ظ„طھط®طھط§ط± ط§ظ„ط¬ظˆط¯ط©.",
    ),
    (
        "Right click a web page in Chrome or Edge and choose \"Download all links with Zero Download\" to pick files from the page.",
        "ط§ط¶ط؛ط· ظƒظ„ظٹظƒ ظٹظ…ظٹظ† ط¹ظ„ظ‰ ط£ظٹ طµظپط­ط© ظپظٹ ظƒط±ظˆظ… ط£ظˆ ط¥ظٹط¯ط¬ ظˆط§ط®طھط± \"Download all links with Zero Download\" ظ„طھط®طھط§ط± ظ…ظ„ظپط§طھ ظ…ظ† ط§ظ„طµظپط­ط©.",
    ),
    (
        "Every download uses up to 32 connections. If a server is slow with many, add it to Options > Connection > Exceptions.",
        "ظƒظ„ طھط­ظ…ظٹظ„ ظٹط³طھط®ط¯ظ… ط­طھظ‰ 32 ط§طھطµط§ظ„ط§ظ‹. ط¥ط°ط§ ظƒط§ظ† ط®ط§ط¯ظ… ظ…ط§ ط¨ط·ظٹط¦ط§ظ‹ ظ…ط¹ ط§طھطµط§ظ„ط§طھ ظƒط«ظٹط±ط©طŒ ط£ط¶ظپظ‡ ظپظٹ ط§ظ„ط®ظٹط§ط±ط§طھ > ط§ظ„ط§طھطµط§ظ„ > ط§ظ„ط§ط³طھط«ظ†ط§ط،ط§طھ.",
    ),
    (
        "Make a queue (Queues > New queue) and set \"1 file at the same time\" to download files one after another.",
        "ط£ظ†ط´ط¦ ظ‚ط§ط¦ظ…ط© ط§ظ†طھط¸ط§ط± ظˆط§ط¶ط¨ط· \"ظ…ظ„ظپ ظˆط§ط­ط¯ ظپظٹ ظ†ظپط³ ط§ظ„ظˆظ‚طھ\" ظ„طھط­ظ…ظٹظ„ ط§ظ„ظ…ظ„ظپط§طھ ظˆط§ط­ط¯ط§ظ‹ طھظ„ظˆ ط§ظ„ط¢ط®ط±.",
    ),
    (
        "The Scheduler can start a queue at night and shut down the computer when everything is finished.",
        "ط§ظ„ظ…ط¬ط¯ظˆظ„ ظٹط³طھط·ظٹط¹ ط¨ط¯ط، ظ‚ط§ط¦ظ…ط© ط¨ط§ظ„ظ„ظٹظ„ ظˆط¥ظٹظ‚ط§ظپ ط§ظ„ظƒظ…ط¨ظٹظˆطھط± ط¹ظ†ط¯ ط§ظ†طھظ‡ط§ط، ظƒظ„ ط´ظٹط،.",
    ),
    (
        "Downloads > When all downloads finish lets you shut down, sleep or hibernate the computer after the last file.",
        "ط§ظ„طھط­ظ…ظٹظ„ط§طھ > ط¹ظ†ط¯ ط§ظ†طھظ‡ط§ط، ظƒظ„ ط§ظ„طھط­ظ…ظٹظ„ط§طھ طھطھظٹط­ ظ„ظƒ ط¥ظٹظ‚ط§ظپ ط§ظ„ظƒظ…ط¨ظٹظˆطھط± ط£ظˆ ط§ظ„ط³ظƒظˆظ† ط£ظˆ ط§ظ„ط¥ط³ط¨ط§طھ ط¨ط¹ط¯ ط¢ط®ط± ظ…ظ„ظپ.",
    ),
    (
        "The speed limiter on the toolbar keeps your internet usable for other things while downloading.",
        "ظ…ط­ط¯ط¯ ط§ظ„ط³ط±ط¹ط© ظپظٹ ط´ط±ظٹط· ط§ظ„ط£ط¯ظˆط§طھ ظٹط¨ظ‚ظٹ ط§ظ„ط¥ظ†طھط±ظ†طھ ظ…طھط§ط­ط§ظ‹ ظ„ط£ط´ظٹط§ط، ط£ط®ط±ظ‰ ط£ط«ظ†ط§ط، ط§ظ„طھط­ظ…ظٹظ„.",
    ),
    (
        "Add Batch > \"Address with a pattern\" downloads img001.jpg â€¦ img100.jpg from one address with a *.",
        "ط¥ط¶ط§ظپط© ظ…ط¬ظ…ظˆط¹ط© > \"ط±ط§ط¨ط· ط¨ظ†ظ…ط·\" ظٹط­ظ…ظ‘ظ„ img001.jpg â€¦ img100.jpg ظ…ظ† ط±ط§ط¨ط· ظˆط§ط­ط¯ ظپظٹظ‡ *.",
    ),
    (
        "An old link expired? Right click the download > \"Refresh download address\" and continue from where it stopped.",
        "ط§ظ†طھظ‡طھ طµظ„ط§ط­ظٹط© ط±ط§ط¨ط· ظ‚ط¯ظٹظ…طں ظƒظ„ظٹظƒ ظٹظ…ظٹظ† ط¹ظ„ظ‰ ط§ظ„طھط­ظ…ظٹظ„ > \"طھط­ط¯ظٹط« ط±ط§ط¨ط· ط§ظ„طھط­ظ…ظٹظ„\" ظˆط£ظƒظ…ظ„ ظ…ظ† ط­ظٹط« طھظˆظ‚ظپ.",
    ),
    (
        "Right click a category in the sidebar > Properties to change its file types and its own folder.",
        "ظƒظ„ظٹظƒ ظٹظ…ظٹظ† ط¹ظ„ظ‰ طھطµظ†ظٹظپ ظپظٹ ط§ظ„ط´ط±ظٹط· ط§ظ„ط¬ط§ظ†ط¨ظٹ > ط§ظ„ط®طµط§ط¦طµ ظ„طھط؛ظٹظٹط± ط£ظ†ظˆط§ط¹ ظ…ظ„ظپط§طھظ‡ ظˆظ…ط¬ظ„ط¯ظ‡.",
    ),
    (
        "View > Columns shows more columns, like Date Added, Save To and Referer.",
        "ط¹ط±ط¶ > ط§ظ„ط£ط¹ظ…ط¯ط© ظٹط¸ظ‡ط± ط£ط¹ظ…ط¯ط© ط£ظƒط«ط± ظ…ط«ظ„ طھط§ط±ظٹط® ط§ظ„ط¥ط¶ط§ظپط© ظˆط§ظ„ط­ظپط¸ ظپظٹ ظˆط§ظ„ظ…ظڈط­ظٹظ„.",
    ),
    (
        "Tasks > Run site grabber collects every image, video or archive of a web site.",
        "ط§ظ„ظ…ظ‡ط§ظ… > طھط´ط؛ظٹظ„ ط¬ط§ظ…ط¹ ط§ظ„ظ…ظˆط§ظ‚ط¹ ظٹط¬ظ…ط¹ ظƒظ„ طµظˆط± ط£ظˆ ظپظٹط¯ظٹظˆظ‡ط§طھ ط£ظˆ ظ…ظ„ظپط§طھ ظ…ظˆظ‚ط¹ ظ…ط§.",
    ),
    (
        "Hold Alt while clicking a link in the browser to let the browser download it by itself.",
        "ط§ط¶ط؛ط· Alt ط£ط«ظ†ط§ط، ط§ظ„ط¶ط؛ط· ط¹ظ„ظ‰ ط±ط§ط¨ط· ظپظٹ ط§ظ„ظ…طھطµظپط­ ظ„ظٹط­ظ…ظ‘ظ„ظ‡ ط§ظ„ظ…طھطµظپط­ ط¨ظ†ظپط³ظ‡.",
    ),
    // 1.0.4
    (
        "Firefox opened the Zero Download add-on.\n1. Click \"Add\" in the message Firefox shows.\n2. Then click \"OK\".\n\nThe add-on is signed by Mozilla, so it stays in Firefox after you close it and updates by itself.",
        "ظپطھط­ ظپط§ظٹط±ظپظˆظƒط³ ط¥ط¶ط§ظپط© Zero Download.\n1. ط§ط¶ط؛ط· \"ط¥ط¶ط§ظپط©\" ظپظٹ ط§ظ„ط±ط³ط§ظ„ط© ط§ظ„طھظٹ ظٹط¹ط±ط¶ظ‡ط§ ظپط§ظٹط±ظپظˆظƒط³.\n2. ط«ظ… ط§ط¶ط؛ط· \"ط­ط³ظ†ظ‹ط§\".\n\nط§ظ„ط¥ط¶ط§ظپط© ظ…ظˆظ‚ظ‘ط¹ط© ظ…ظ† MozillaطŒ ظ„ط°ظ„ظƒ طھط¨ظ‚ظ‰ ظپظٹ ظپط§ظٹط±ظپظˆظƒط³ ط¨ط¹ط¯ ط¥ط؛ظ„ط§ظ‚ظ‡ ظˆطھطھط­ط¯ظ‘ط« ظˆط­ط¯ظ‡ط§.",
    ),
    ("Zero Download was updated to version {} by itself.", "طھظ… طھط­ط¯ظٹط« Zero Download طھظ„ظ‚ط§ط¦ظٹظ‹ط§ ط¥ظ„ظ‰ ط§ظ„ط¥طµط¯ط§ط± {}."),
    ("What's new:", "ط§ظ„ط¬ط¯ظٹط¯ ظپظٹ ظ‡ط°ط§ ط§ظ„ط¥طµط¯ط§ط±:"),
    (
        "A \"Download this video\" button on every video of a page, and you can drag it anywhere on the video.",
        "ط²ط± \"طھط­ظ…ظٹظ„ ظ‡ط°ط§ ط§ظ„ظپظٹط¯ظٹظˆ\" ط¹ظ„ظ‰ ظƒظ„ ظپظٹط¯ظٹظˆ ظپظٹ ط§ظ„طµظپط­ط©طŒ ظˆظٹظ…ظƒظ†ظƒ ط³ط­ط¨ظ‡ ط¥ظ„ظ‰ ط£ظٹ ظ…ظƒط§ظ† ط¹ظ„ظ‰ ط§ظ„ظپظٹط¯ظٹظˆ.",
    ),
    (
        "Videos with several sound tracks: the original language is downloaded, not a dubbed one.",
        "ط§ظ„ظپظٹط¯ظٹظˆظ‡ط§طھ ط°ط§طھ ط§ظ„ظ…ط³ط§ط±ط§طھ ط§ظ„طµظˆطھظٹط© ط§ظ„ظ…طھط¹ط¯ط¯ط©: ظٹظڈط­ظ…ظژظ‘ظ„ ط§ظ„طµظˆطھ ط¨ط§ظ„ظ„ط؛ط© ط§ظ„ط£طµظ„ظٹط© ظˆظ„ظٹط³ ط§ظ„ظ…ط¯ط¨ظ„ط¬.",
    ),
    (
        "New languages: French, Spanish, German, Portuguese, Turkish and Russian.",
        "ظ„ط؛ط§طھ ط¬ط¯ظٹط¯ط©: ط§ظ„ظپط±ظ†ط³ظٹط© ظˆط§ظ„ط¥ط³ط¨ط§ظ†ظٹط© ظˆط§ظ„ط£ظ„ظ…ط§ظ†ظٹط© ظˆط§ظ„ط¨ط±طھط؛ط§ظ„ظٹط© ظˆط§ظ„طھط±ظƒظٹط© ظˆط§ظ„ط±ظˆط³ظٹط©.",
    ),
    (
        "Five new themes and a window opacity setting (View > Theme).",
        "ط®ظ…ط³ط© ظ…ط¸ط§ظ‡ط± ط¬ط¯ظٹط¯ط© ظˆط¥ط¹ط¯ط§ط¯ ظ„ط´ظپط§ظپظٹط© ط§ظ„ظ†ط§ظپط°ط© (ط¹ط±ط¶ > ط§ظ„ظ…ط¸ظ‡ط±).",
    ),
    (
        "Brave is supported (Help > Add Zero Download to my browser).",
        "ط¯ط¹ظ… ظ…طھطµظپط­ Brave (ظ…ط³ط§ط¹ط¯ط© > ط¥ط¶ط§ظپط© Zero Download ط¥ظ„ظ‰ ظ…طھطµظپط­ظٹ).",
    ),
    ("Firefox: the add-on stays in Firefox after it is closed.", "ظپط§ظٹط±ظپظˆظƒط³: ط§ظ„ط¥ط¶ط§ظپط© طھط¨ظ‚ظ‰ ط¨ط¹ط¯ ط¥ط؛ظ„ط§ظ‚ ط§ظ„ظ…طھطµظپط­."),
    (
        "The browser add-in window is bigger; its settings moved to the add-in's own settings page.",
        "ظ†ط§ظپط°ط© ط¥ط¶ط§ظپط© ط§ظ„ظ…طھطµظپط­ ط£ظƒط¨ط±طŒ ظˆط¥ط¹ط¯ط§ط¯ط§طھظ‡ط§ ط§ظ†طھظ‚ظ„طھ ط¥ظ„ظ‰ طµظپط­ط© ط§ظ„ط¥ط¹ط¯ط§ط¯ط§طھ ط§ظ„ط®ط§طµط© ط¨ط§ظ„ط¥ط¶ط§ظپط©.",
    ),
    ("Windows of the program open in front of the main window.", "ظ†ظˆط§ظپط° ط§ظ„ط¨ط±ظ†ط§ظ…ط¬ طھظپطھط­ ط£ظ…ط§ظ… ط§ظ„ظ†ط§ظپط°ط© ط§ظ„ط±ط¦ظٹط³ظٹط©."),
    (
        "A new playlist icon, and a smaller icon on the window title.",
        "ط£ظٹظ‚ظˆظ†ط© ط¬ط¯ظٹط¯ط© ظ„ظ‚ظˆط§ط¦ظ… ط§ظ„طھط´ط؛ظٹظ„طŒ ظˆط£ظٹظ‚ظˆظ†ط© ط£طµط؛ط± ظپظٹ ط´ط±ظٹط· ط¹ظ†ظˆط§ظ† ط§ظ„ظ†ط§ظپط°ط©.",
    ),
    ("Support Zero Download with PayPal from Help > About.", "ط§ط¯ط¹ظ… Zero Download ط¹ط¨ط± PayPal ظ…ظ† ظ…ط³ط§ط¹ط¯ط© > ط­ظˆظ„."),
    // 1.0.5
    (
        "The main window can be maximized to the full screen.",
        "ظ†ط§ظپط°ط© ط§ظ„ط¨ط±ظ†ط§ظ…ط¬ ط§ظ„ط±ط¦ظٹط³ظٹط© ظٹظ…ظƒظ† طھظƒط¨ظٹط±ظ‡ط§ ظ„ظ…ظ„ط، ط§ظ„ط´ط§ط´ط©.",
    ),
    (
        "Columns of the list can be resized and moved by dragging their titles.",
        "ط£ط¹ظ…ط¯ط© ط§ظ„ظ‚ط§ط¦ظ…ط© ظٹظ…ظƒظ† طھظƒط¨ظٹط±ظ‡ط§ ظˆطھطµط؛ظٹط±ظ‡ط§ ظˆظ†ظ‚ظ„ظ‡ط§ ط¨ط³ط­ط¨ ط¹ظ†ط§ظˆظٹظ†ظ‡ط§.",
    ),
    (
        "Toolbar names are bold, and the size of the video button can be chosen in the add-in's window.",
        "ط£ط³ظ…ط§ط، ط£ط²ط±ط§ط± ط§ظ„ط´ط±ظٹط· ط¨ط®ط· ط¹ط±ظٹط¶طŒ ظˆظٹظ…ظƒظ† ط§ط®طھظٹط§ط± ط­ط¬ظ… ط²ط± ط§ظ„ظپظٹط¯ظٹظˆ ظ…ظ† ظ†ط§ظپط°ط© ط§ظ„ط¥ط¶ط§ظپط©.",
    ),
    (
        "Pictures",
        "ط§ظ„طµظˆط±",
    ),
    (
        "Checking for updatesâ€¦",
        "ط¬ط§ط±ظچ ط§ظ„ط¨ط­ط« ط¹ظ† طھط­ط¯ظٹط«ط§طھâ€¦",
    ),
    (
        "This is version {}.",
        "ظ‡ط°ط§ ظ‡ظˆ ط§ظ„ط¥طµط¯ط§ط± {}.",
    ),
    (
        "Could not check for updates",
        "طھط¹ط°ظ‘ط± ط§ظ„ط¨ط­ط« ط¹ظ† طھط­ط¯ظٹط«ط§طھ",
    ),
    (
        "Zero Download is up to date",
        "Zero Download ظ…ط­ط¯ظژظ‘ط«",
    ),
    (
        "Version {} is the newest version.",
        "ط§ظ„ط¥طµط¯ط§ط± {} ظ‡ظˆ ط£ط­ط¯ط« ط¥طµط¯ط§ط±.",
    ),
    (
        "Version {} is available",
        "ط§ظ„ط¥طµط¯ط§ط± {} ظ…طھط§ط­",
    ),
    (
        "This copy was not installed with the setup, so it cannot update itself.",
        "ظ‡ط°ظ‡ ط§ظ„ظ†ط³ط®ط© ظ„ظ… طھظڈط«ط¨ظژظ‘طھ ط¨ط¨ط±ظ†ط§ظ…ط¬ ط§ظ„ط¥ط¹ط¯ط§ط¯طŒ ظ„ط°ظ„ظƒ ظ„ط§ ظٹظ…ظƒظ†ظ‡ط§ طھط­ط¯ظٹط« ظ†ظپط³ظ‡ط§.",
    ),
    (
        "Downloading version {}â€¦",
        "ط¬ط§ط±ظچ طھط­ظ…ظٹظ„ ط§ظ„ط¥طµط¯ط§ط± {}â€¦",
    ),
    (
        "Installing version {}â€¦",
        "ط¬ط§ط±ظچ طھط«ط¨ظٹطھ ط§ظ„ط¥طµط¯ط§ط± {}â€¦",
    ),
    (
        "Zero Download closes now and opens again when it is installed.",
        "ط³ظٹظڈط؛ظ„ظ‚ Zero Download ط§ظ„ط¢ظ† ظˆظٹظپطھط­ ظ…ظ† ط¬ط¯ظٹط¯ ط¨ط¹ط¯ ط§ظ„طھط«ط¨ظٹطھ.",
    ),
    (
        "Could not get the update",
        "طھط¹ط°ظ‘ط± ط§ظ„ط­طµظˆظ„ ط¹ظ„ظ‰ ط§ظ„طھط­ط¯ظٹط«",
    ),
    (
        "Text size, font and a picture behind the main window (Options > Appearance).",
        "ط­ط¬ظ… ط§ظ„ط®ط· ظˆط§ظ„ط®ط· ظˆطµظˆط±ط© ط®ظ„ظپ ط§ظ„ظ†ط§ظپط°ط© ط§ظ„ط±ط¦ظٹط³ظٹط© (ط§ظ„ط®ظٹط§ط±ط§طھ > ط§ظ„ظ…ط¸ظ‡ط±).",
    ),
    (
        "Windows open in the middle of the screen; only the main window can be maximized.",
        "ط§ظ„ظ†ظˆط§ظپط° طھظپطھط­ ظپظٹ ظ…ظ†طھطµظپ ط§ظ„ط´ط§ط´ط©طŒ ظˆط§ظ„ظ†ط§ظپط°ط© ط§ظ„ط±ط¦ظٹط³ظٹط© ظˆط­ط¯ظ‡ط§ ظٹظ…ظƒظ† طھظƒط¨ظٹط±ظ‡ط§.",
    ),
    (
        "Drag a finished file from the list onto the desktop or into a folder.",
        "ط§ط³ط­ط¨ ظ…ظ„ظپظ‹ط§ ظ…ظƒطھظ…ظ„ظ‹ط§ ظ…ظ† ط§ظ„ظ‚ط§ط¦ظ…ط© ط¥ظ„ظ‰ ط³ط·ط­ ط§ظ„ظ…ظƒطھط¨ ط£ظˆ ط¥ظ„ظ‰ ط£ظٹ ظ…ط¬ظ„ط¯.",
    ),
    (
        "The progress of a download shows on its taskbar button.",
        "طھظ‚ط¯ظ‘ظ… ط§ظ„طھط­ظ…ظٹظ„ ظٹط¸ظ‡ط± ط¹ظ„ظ‰ ط²ط± ط§ظ„ظ†ط§ظپط°ط© ظپظٹ ط´ط±ظٹط· ط§ظ„ظ…ظ‡ط§ظ….",
    ),
    (
        "Check for updates shows the check, the download and the installation in its own window.",
        "ط§ظ„ط¨ط­ط« ط¹ظ† طھط­ط¯ظٹط«ط§طھ ظٹط¹ط±ط¶ ط§ظ„ط¨ط­ط« ظˆط§ظ„طھط­ظ…ظٹظ„ ظˆط§ظ„طھط«ط¨ظٹطھ ظپظٹ ظ†ط§ظپط°ط© ط®ط§طµط©.",
    ),
    (
        "Playlist videos are saved with their own original titles and their original sound.",
        "ظپظٹط¯ظٹظˆظ‡ط§طھ ظ‚ظˆط§ط¦ظ… ط§ظ„طھط´ط؛ظٹظ„ طھظڈط­ظپط¸ ط¨ط¹ظ†ط§ظˆظٹظ†ظ‡ط§ ط§ظ„ط£طµظ„ظٹط© ظˆطµظˆطھظ‡ط§ ط§ظ„ط£طµظ„ظٹ.",
    ),
    (
        "Closing a download's window (X) stops the download; Resume opens it again.",
        "ط¥ط؛ظ„ط§ظ‚ ظ†ط§ظپط°ط© ط§ظ„طھط­ظ…ظٹظ„ (X) ظٹظˆظ‚ظپ ط§ظ„طھط­ظ…ظٹظ„طŒ ظˆآ«ط§ط³طھط¦ظ†ط§ظپآ» ظٹظپطھط­ظ‡ط§ ظ…ظ† ط¬ط¯ظٹط¯.",
    ),
    (
        "The download button on videos in the browser is smaller, and its size can be changed in the add-in's settings.",
        "ط²ط± ط§ظ„طھط­ظ…ظٹظ„ ط¹ظ„ظ‰ ط§ظ„ظپظٹط¯ظٹظˆظ‡ط§طھ ظپظٹ ط§ظ„ظ…طھطµظپط­ ط£طµط¨ط­ ط£طµط؛ط±طŒ ظˆظٹظ…ظƒظ† طھط؛ظٹظٹط± ط­ط¬ظ…ظ‡ ظ…ظ† ط¥ط¹ط¯ط§ط¯ط§طھ ط§ظ„ط¥ط¶ط§ظپط©.",
    ),
    (
        "Updates are signed: Zero Download installs only updates made by its developer.",
        "ط§ظ„طھط­ط¯ظٹط«ط§طھ ط£طµط¨ط­طھ ظ…ظˆظ‚ظ‘ط¹ط©: Zero Download ظ„ط§ ظٹط«ط¨ظ‘طھ ط¥ظ„ط§ ط§ظ„طھط­ط¯ظٹط«ط§طھ ط§ظ„طµط§ط¯ط±ط© ظ…ظ† ظ…ط·ظˆظ‘ط±ظ‡.",
    ),
    (
        "Downloaded files are marked as coming from the internet, so Windows warns before opening an unsafe file.",
        "ط§ظ„ظ…ظ„ظپط§طھ ط§ظ„ظ…ط­ظ…ظ‘ظ„ط© طھظڈط¹ظ„ظژظ‘ظ… ط¨ط£ظ†ظ‡ط§ ظ…ظ† ط§ظ„ط¥ظ†طھط±ظ†طھطŒ ظپظٹط­ط°ظ‘ط±ظƒ ظˆظٹظ†ط¯ظˆط² ظ‚ط¨ظ„ ظپطھط­ ظ…ظ„ظپ ط؛ظٹط± ط¢ظ…ظ†.",
    ),
    (
        "Saved passwords and site logins are encrypted with your Windows account.",
        "ظƒظ„ظ…ط§طھ ط§ظ„ظ…ط±ظˆط± ظˆط¨ظٹط§ظ†ط§طھ ط¯ط®ظˆظ„ ط§ظ„ظ…ظˆط§ظ‚ط¹ ط§ظ„ظ…ط­ظپظˆط¸ط© ط£طµط¨ط­طھ ظ…ط´ظپظ‘ط±ط© ط¨ط­ط³ط§ط¨ ظˆظٹظ†ط¯ظˆط² ط§ظ„ط®ط§طµ ط¨ظƒ.",
    ),
    (
        "The video helpers (yt-dlp and ffmpeg) are checked before they are used.",
        "ط£ط¯ظˆط§طھ ط§ظ„ظپظٹط¯ظٹظˆ (yt-dlp ظˆ ffmpeg) ظٹطھظ… ط§ظ„طھط­ظ‚ظ‚ ظ…ظ†ظ‡ط§ ظ‚ط¨ظ„ ط§ط³طھط®ط¯ط§ظ…ظ‡ط§.",
    ),
    (
        "More protection against harmful web pages and links.",
        "ط­ظ…ط§ظٹط© ط£ظƒط¨ط± ظ…ظ† طµظپط­ط§طھ ط§ظ„ظˆظٹط¨ ظˆط§ظ„ط±ظˆط§ط¨ط· ط§ظ„ط¶ط§ط±ط©.",
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fills_places() {
        assert_eq!(tf("Tip {} of {}", &[&1, &14]), "Tip 1 of 14");
        CURRENT.store(index_of("ar"), Ordering::Relaxed);
        assert_eq!(tf("Tip {} of {}", &[&1, &14]), "ظ†طµظٹط­ط© 1 ظ…ظ† 14");
        assert_eq!(t("not in the list"), "not in the list");
        CURRENT.store(index_of("fr"), Ordering::Relaxed);
        assert_eq!(tf("Tip {} of {}", &[&1, &14]), "Astuce 1 sur 14");
        CURRENT.store(0, Ordering::Relaxed);
    }

    #[test]
    fn every_language_has_every_text() {
        // the tables are read from JSON: a broken file would silently fall back to English
        for (code, _) in LANGS.iter().skip(2) {
            let m = table(code).expect("table");
            assert!(m.len() > 150, "{code}: only {} texts", m.len());
            for (en, tr) in m {
                assert_eq!(en.matches("{}").count(), tr.matches("{}").count(), "{code}: {en}");
            }
        }
        assert_eq!(index_of("xx"), 0);
        assert_eq!(effective("es"), "es");
    }
}

