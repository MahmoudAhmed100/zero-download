//! File security, checksum verification, clipboard matching and antivirus scanning.
//!
//! Features:
//! 1. High-speed SHA-256 and MD5 checksum computation.
//! 2. Automatic clipboard hash matching (detects if copied hash matches downloaded file).
//! 3. Native Windows Defender CLI scanner integration (`MpCmdRun.exe`).

use anyhow::Result;
use sha2::{Digest as ShaDigest, Sha256};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct ChecksumReport {
    pub sha256: String,
    pub md5: String,
    pub matched_clipboard: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanStatus {
    Clean,
    ThreatFound(String),
    NotAvailable,
    Error(String),
}

/// Computes SHA-256 and MD5 simultaneously in a single streaming pass (minimal disk I/O).
pub fn compute_checksums(path: &Path, clipboard_text: Option<&str>) -> Result<ChecksumReport> {
    let mut file = std::fs::File::open(path)?;
    let mut sha_hasher = Sha256::new();
    let mut md5_hasher = md5::Context::new();

    let mut buf = vec![0u8; 1 << 20]; // 1MB buffer
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        sha_hasher.update(&buf[..n]);
        md5_hasher.consume(&buf[..n]);
    }

    let sha256_hex = format!("{:x}", sha_hasher.finalize());
    let md5_hex = format!("{:x}", md5_hasher.compute());

    // Check if clipboard contains matching hash
    let mut matched = None;
    if let Some(clip) = clipboard_text {
        let clean_clip = clip.trim().to_ascii_lowercase();
        if clean_clip == sha256_hex {
            matched = Some("SHA-256");
        } else if clean_clip == md5_hex {
            matched = Some("MD5");
        }
    }

    Ok(ChecksumReport {
        sha256: sha256_hex,
        md5: md5_hex,
        matched_clipboard: matched,
    })
}

/// Locates MpCmdRun.exe (Windows Defender Command Line tool).
pub fn find_defender() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        for p in [
            r"C:\Program Files\Windows Defender\MpCmdRun.exe",
            r"C:\Program Files (x86)\Windows Defender\MpCmdRun.exe",
        ] {
            let path = PathBuf::from(p);
            if path.exists() {
                return Some(path);
            }
        }
    }
    None
}

/// Scans a file with native Windows Defender.
pub fn scan_file(path: &Path) -> ScanStatus {
    let Some(defender) = find_defender() else {
        return ScanStatus::NotAvailable;
    };

    let mut cmd = Command::new(&defender);
    cmd.args(["-Scan", "-ScanType", "3", "-File"]).arg(path);
    crate::mux::hide_window(&mut cmd);

    match cmd.output() {
        Ok(out) => {
            // Exit code 0: clean
            // Exit code 2: threat detected
            match out.status.code() {
                Some(0) => ScanStatus::Clean,
                Some(2) => {
                    let err = String::from_utf8_lossy(&out.stdout);
                    ScanStatus::ThreatFound(err.trim().to_string())
                }
                _ => {
                    // Check output text
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    if stdout.contains("no threats") || stdout.contains("Scan finished") {
                        ScanStatus::Clean
                    } else if stdout.contains("threat") {
                        ScanStatus::ThreatFound("تم العثور على تهديد محتمل".into())
                    } else {
                        ScanStatus::Clean
                    }
                }
            }
        }
        Err(e) => ScanStatus::Error(e.to_string()),
    }
}

// Lightweight embedded MD5 implementation so we don't need external md5 crate
mod md5 {
    #[derive(Clone)]
    pub struct Context {
        count: u64,
        state: [u32; 4],
        buffer: [u8; 64],
    }

    pub struct Digest(pub [u8; 16]);

    impl Context {
        pub fn new() -> Self {
            Self {
                count: 0,
                state: [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476],
                buffer: [0u8; 64],
            }
        }

        pub fn consume(&mut self, mut input: &[u8]) {
            let mut index = ((self.count >> 3) & 0x3f) as usize;
            self.count += (input.len() as u64) << 3;

            let part_len = 64 - index;
            let mut i = 0;

            if input.len() >= part_len {
                self.buffer[index..index + part_len].copy_from_slice(&input[..part_len]);
                Self::transform(&mut self.state, &self.buffer);
                i = part_len;
                while i + 63 < input.len() {
                    Self::transform(&mut self.state, &input[i..i + 64]);
                    i += 64;
                }
                index = 0;
            }

            self.buffer[index..index + input.len() - i].copy_from_slice(&input[i..]);
        }

        pub fn compute(mut self) -> Digest {
            let mut bits = [0u8; 8];
            for (idx, b) in bits.iter_mut().enumerate() {
                *b = (self.count >> (idx * 8)) as u8;
            }

            let index = ((self.count >> 3) & 0x3f) as usize;
            let pad_len = if index < 56 { 56 - index } else { 120 - index };

            let mut padding = [0u8; 64];
            padding[0] = 0x80;
            self.consume(&padding[..pad_len]);
            self.consume(&bits);

            let mut out = [0u8; 16];
            for (i, &val) in self.state.iter().enumerate() {
                out[i * 4] = val as u8;
                out[i * 4 + 1] = (val >> 8) as u8;
                out[i * 4 + 2] = (val >> 16) as u8;
                out[i * 4 + 3] = (val >> 24) as u8;
            }
            Digest(out)
        }

        fn transform(state: &mut [u32; 4], block: &[u8]) {
            let mut x = [0u32; 16];
            for (i, chunk) in block.chunks_exact(4).enumerate() {
                x[i] = u32::from_le_bytes(chunk.try_into().unwrap());
            }

            let [mut a, mut b, mut c, mut d] = *state;

            let ff = |a: &mut u32, b: u32, c: u32, d: u32, x: u32, s: u32, ac: u32| {
                *a = a.wrapping_add((b & c) | (!b & d)).wrapping_add(x).wrapping_add(ac);
                *a = a.rotate_left(s).wrapping_add(b);
            };

            let gg = |a: &mut u32, b: u32, c: u32, d: u32, x: u32, s: u32, ac: u32| {
                *a = a.wrapping_add((b & d) | (c & !d)).wrapping_add(x).wrapping_add(ac);
                *a = a.rotate_left(s).wrapping_add(b);
            };

            let hh = |a: &mut u32, b: u32, c: u32, d: u32, x: u32, s: u32, ac: u32| {
                *a = a.wrapping_add(b ^ c ^ d).wrapping_add(x).wrapping_add(ac);
                *a = a.rotate_left(s).wrapping_add(b);
            };

            let ii = |a: &mut u32, b: u32, c: u32, d: u32, x: u32, s: u32, ac: u32| {
                *a = a.wrapping_add(c ^ (b | !d)).wrapping_add(x).wrapping_add(ac);
                *a = a.rotate_left(s).wrapping_add(b);
            };

            // Round 1
            ff(&mut a, b, c, d, x[0], 7, 0xd76aa478);
            ff(&mut d, a, b, c, x[1], 12, 0xe8c7b756);
            ff(&mut c, d, a, b, x[2], 17, 0x242070db);
            ff(&mut b, c, d, a, x[3], 22, 0xc1bdceee);
            ff(&mut a, b, c, d, x[4], 7, 0xf57c0faf);
            ff(&mut d, a, b, c, x[5], 12, 0x4787c62a);
            ff(&mut c, d, a, b, x[6], 17, 0xa8304613);
            ff(&mut b, c, d, a, x[7], 22, 0xfd469501);
            ff(&mut a, b, c, d, x[8], 7, 0x698098d8);
            ff(&mut d, a, b, c, x[9], 12, 0x8b44f7af);
            ff(&mut c, d, a, b, x[10], 17, 0xffff5bb1);
            ff(&mut b, c, d, a, x[11], 22, 0x895cd7be);
            ff(&mut a, b, c, d, x[12], 7, 0x6b901122);
            ff(&mut d, a, b, c, x[13], 12, 0xfd987193);
            ff(&mut c, d, a, b, x[14], 17, 0xa679438e);
            ff(&mut b, c, d, a, x[15], 22, 0x49b40821);

            // Round 2
            gg(&mut a, b, c, d, x[1], 5, 0xf61e2562);
            gg(&mut d, a, b, c, x[6], 9, 0xc040b340);
            gg(&mut c, d, a, b, x[11], 14, 0x265e5a51);
            gg(&mut b, c, d, a, x[0], 20, 0xe9b6c7aa);
            gg(&mut a, b, c, d, x[5], 5, 0xd62f105d);
            gg(&mut d, a, b, c, x[10], 9, 0x02441453);
            gg(&mut c, d, a, b, x[15], 14, 0xd8a1e681);
            gg(&mut b, c, d, a, x[4], 20, 0xe7d3fbc8);
            gg(&mut a, b, c, d, x[9], 5, 0x21e1cde6);
            gg(&mut d, a, b, c, x[14], 9, 0xc33707d6);
            gg(&mut c, d, a, b, x[3], 14, 0xf4d50d87);
            gg(&mut b, c, d, a, x[8], 20, 0x455a14ed);
            gg(&mut a, b, c, d, x[13], 5, 0xa9e3e905);
            gg(&mut d, a, b, c, x[2], 9, 0xfcefa3f8);
            gg(&mut c, d, a, b, x[7], 14, 0x676f02d9);
            gg(&mut b, c, d, a, x[12], 20, 0x8d2a4c8a);

            // Round 3
            hh(&mut a, b, c, d, x[5], 4, 0xfffa3942);
            hh(&mut d, a, b, c, x[8], 11, 0x8771f681);
            hh(&mut c, d, a, b, x[11], 16, 0x6d9d6122);
            hh(&mut b, c, d, a, x[14], 23, 0xfde5380c);
            hh(&mut a, b, c, d, x[1], 4, 0xa4beea44);
            hh(&mut d, a, b, c, x[4], 11, 0x4bdecfa9);
            hh(&mut c, d, a, b, x[7], 16, 0xf6bb4b60);
            hh(&mut b, c, d, a, x[10], 23, 0xbebfbc70);
            hh(&mut a, b, c, d, x[13], 4, 0x289b7ec6);
            hh(&mut d, a, b, c, x[0], 11, 0xeaa127fa);
            hh(&mut c, d, a, b, x[3], 16, 0xd4ef3085);
            hh(&mut b, c, d, a, x[6], 23, 0x04881d05);
            hh(&mut a, b, c, d, x[9], 4, 0xd9d4d039);
            hh(&mut d, a, b, c, x[12], 11, 0xe6db99e5);
            hh(&mut c, d, a, b, x[15], 16, 0x1fa27cf8);
            hh(&mut b, c, d, a, x[2], 23, 0xc4ac5665);

            // Round 4
            ii(&mut a, b, c, d, x[0], 6, 0xf4292244);
            ii(&mut d, a, b, c, x[7], 10, 0x432aff97);
            ii(&mut c, d, a, b, x[14], 15, 0xab9423a7);
            ii(&mut b, c, d, a, x[5], 21, 0xfc93a039);
            ii(&mut a, b, c, d, x[12], 6, 0x655b59c3);
            ii(&mut d, a, b, c, x[3], 10, 0x8f0ccc92);
            ii(&mut c, d, a, b, x[10], 15, 0xffeff47d);
            ii(&mut b, c, d, a, x[1], 21, 0x85845dd1);
            ii(&mut a, b, c, d, x[8], 6, 0x6fa87e4f);
            ii(&mut d, a, b, c, x[15], 10, 0xfe2ce6e0);
            ii(&mut c, d, a, b, x[6], 15, 0xa3014314);
            ii(&mut b, c, d, a, x[13], 21, 0x4e0811a1);
            ii(&mut a, b, c, d, x[4], 6, 0xf7537e82);
            ii(&mut d, a, b, c, x[11], 10, 0xbd3af235);
            ii(&mut c, d, a, b, x[2], 15, 0x2ad7d2bb);
            ii(&mut b, c, d, a, x[9], 21, 0xeb86d391);

            state[0] = state[0].wrapping_add(a);
            state[1] = state[1].wrapping_add(b);
            state[2] = state[2].wrapping_add(c);
            state[3] = state[3].wrapping_add(d);
        }
    }

    impl std::fmt::LowerHex for Digest {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            for b in self.0 {
                write!(f, "{:02x}", b)?;
            }
            Ok(())
        }
    }
}
