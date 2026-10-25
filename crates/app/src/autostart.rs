//! "Start with Windows": written to the user's registry (HKCU\Run),
//! so no admin rights are asked for.

pub fn is_enabled() -> bool {
    imp::is_enabled()
}

pub fn set(enabled: bool) -> bool {
    imp::set(enabled)
}

#[allow(dead_code)]
pub fn installed_by_setup() -> bool {
    imp::installed_by_setup()
}

#[cfg(windows)]
mod imp {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    const KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
    const NAME: &str = "ZeroDownload";
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    fn reg(args: &[&str]) -> bool {
        Command::new(swiftget_engine::system::windows_tool("reg.exe"))
            .args(args)
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn is_enabled() -> bool {
        reg(&["query", KEY, "/v", NAME])
    }

    /// Installed with the setup program (which asked about autostart itself).
    pub fn installed_by_setup() -> bool {
        reg(&["query", r"HKCU\Software\Zero\ZeroDownload", "/v", "Installed"])
            || reg(&["query", r"HKCU\Software\Ezoo\EzooDownload", "/v", "Installed"])
    }

    pub fn set(enabled: bool) -> bool {
        if enabled {
            let exe = match std::env::current_exe() {
                Ok(p) => p.display().to_string(),
                Err(_) => return false,
            };
            // starts minimized next to the clock
            let val = format!("\"{exe}\" --background");
            reg(&["add", KEY, "/v", NAME, "/t", "REG_SZ", "/d", &val, "/f"])
        } else {
            reg(&["delete", KEY, "/v", NAME, "/f"])
        }
    }
}

#[cfg(not(windows))]
mod imp {
    pub fn is_enabled() -> bool {
        false
    }
    pub fn installed_by_setup() -> bool {
        false
    }
    pub fn set(_: bool) -> bool {
        false
    }
}
