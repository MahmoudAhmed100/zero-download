//! "Add Zero Download to my browser": opens the browser on the right page
//! (its store page once the extension is published there, otherwise its
//! extensions page with the folder path ready in the clipboard).

use crate::open_window;
use crate::*;
use crate::i18n::{t, tf};

// Store addresses: once the extension is published on each store, fill its
// web address here. The button in the program then opens the store directly.
const STORE_CHROME: &str = "";
const STORE_EDGE: &str = "";
const STORE_FIREFOX: &str = "";
const STORE_OPERA: &str = "";
const STORE_BRAVE: &str = "";

/// The Firefox add-on signed by Mozilla (built by the release): installed
/// from this file it stays in Firefox for good.
const FIREFOX_XPI: &str = "zero-download-firefox.xpi";

#[derive(Clone)]
pub(crate) struct Browser {
    name: &'static str,
    exe: Option<PathBuf>,
    extensions_page: &'static str,
    store: &'static str,
    firefox: bool,
    tint: slint::Color,
}

fn env_path(var: &str, sub: &str) -> Option<PathBuf> {
    std::env::var_os(var).map(|p| PathBuf::from(p).join(sub))
}

fn first_existing(paths: Vec<Option<PathBuf>>) -> Option<PathBuf> {
    paths.into_iter().flatten().find(|p| p.is_file())
}

fn rgb(r: u8, g: u8, b: u8) -> slint::Color {
    slint::Color::from_rgb_uint8(r, g, b)
}

fn detect() -> Vec<Browser> {
    vec![
        Browser {
            name: "Google Chrome",
            exe: first_existing(vec![
                env_path("ProgramFiles", r"Google\Chrome\Application\chrome.exe"),
                env_path("ProgramFiles(x86)", r"Google\Chrome\Application\chrome.exe"),
                env_path("LOCALAPPDATA", r"Google\Chrome\Application\chrome.exe"),
            ]),
            extensions_page: "chrome://extensions/",
            store: STORE_CHROME,
            firefox: false,
            tint: rgb(0xea, 0x43, 0x35),
        },
        Browser {
            name: "Microsoft Edge",
            exe: first_existing(vec![
                env_path("ProgramFiles(x86)", r"Microsoft\Edge\Application\msedge.exe"),
                env_path("ProgramFiles", r"Microsoft\Edge\Application\msedge.exe"),
            ]),
            extensions_page: "edge://extensions/",
            store: STORE_EDGE,
            firefox: false,
            tint: rgb(0x0c, 0x8c, 0xd8),
        },
        Browser {
            name: "Firefox",
            exe: first_existing(vec![
                env_path("ProgramFiles", r"Mozilla Firefox\firefox.exe"),
                env_path("ProgramFiles(x86)", r"Mozilla Firefox\firefox.exe"),
            ]),
            extensions_page: "about:debugging#/runtime/this-firefox",
            store: STORE_FIREFOX,
            firefox: true,
            tint: rgb(0xff, 0x71, 0x39),
        },
        Browser {
            name: "Opera",
            exe: first_existing(vec![
                env_path("LOCALAPPDATA", r"Programs\Opera\launcher.exe"),
                env_path("ProgramFiles", r"Opera\launcher.exe"),
            ]),
            extensions_page: "opera://extensions",
            store: STORE_OPERA,
            firefox: false,
            tint: rgb(0xd6, 0x1f, 0x2b),
        },
        Browser {
            name: "Brave",
            exe: first_existing(vec![
                env_path("ProgramFiles", r"BraveSoftware\Brave-Browser\Application\brave.exe"),
                env_path("ProgramFiles(x86)", r"BraveSoftware\Brave-Browser\Application\brave.exe"),
                env_path("LOCALAPPDATA", r"BraveSoftware\Brave-Browser\Application\brave.exe"),
            ]),
            extensions_page: "brave://extensions/",
            store: STORE_BRAVE,
            firefox: false,
            tint: rgb(0xfb, 0x54, 0x2b),
        },
    ]
}

/// The extension folder next to the program (`extension` or `extension-firefox`).
fn extension_dir(firefox: bool) -> Option<PathBuf> {
    let name = if firefox { "extension-firefox" } else { "extension" };
    let dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    [dir.join(name), dir.join("..").join(name)].into_iter().find(|p| p.exists())
}

/// The signed Firefox add-on next to the program, when the release has one.
fn firefox_xpi() -> Option<PathBuf> {
    let dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    [dir.join(FIREFOX_XPI), dir.join("ezoo-download-firefox.xpi")].into_iter().find(|p| p.exists())
}

pub(crate) fn open(ui: &Ui) {
    let Some(d) = ui.browsers.upgrade() else { return };
    let list = detect();
    *ui.browser_list.borrow_mut() = list.clone();
    let rows: Vec<BrowserRow> = list
        .iter()
        .map(|b| BrowserRow {
            name: b.name.into(),
            installed: b.exe.is_some(),
            tint: b.tint,
        })
        .collect();
    d.set_browsers(ModelRc::new(VecModel::from(rows)));
    d.set_steps(SharedString::new());
    d.set_folder(extension_dir(false).map(|p| p.display().to_string()).unwrap_or_default().into());
    open_window!(d);
    d.window().set_minimized(false);
}

fn add(ui: &Ui, i: usize) {
    let list = ui.browser_list.borrow();
    let Some(b) = list.get(i) else { return };
    let Some(d) = ui.browsers.upgrade() else { return };
    let Some(exe) = &b.exe else {
        d.set_steps(tf("{} is not installed on this computer.", &[&b.name]).into());
        return;
    };
    if !b.store.is_empty() {
        launch(&exe, b.store);
        d.set_steps(tf("{} opened the store page: click \"Add\" / \"Get\" there.", &[&b.name]).into());
        return;
    }
    if b.firefox {
        if let Some(xpi) = firefox_xpi() {
            // the signed add-on: Firefox asks "Add Zero Download?" and keeps it
            let _ = std::process::Command::new(&exe).arg(&xpi).spawn();
            d.set_folder(xpi.display().to_string().into());
            d.set_steps(t("Firefox opened the Zero Download add-on.\n1. Click \"Add\" in the message Firefox shows.\n2. Then click \"OK\".\n\nThe add-on is signed by Mozilla, so it stays in Firefox after you close it and updates by itself.").into());
            return;
        }
    }
    let Some(dir) = extension_dir(b.firefox) else {
        d.set_steps(t("The extension folder was not found next to ZeroDownload.exe. Install Zero Download again.").into());
        return;
    };
    let path = dir.display().to_string();
    if let Ok(mut c) = arboard::Clipboard::new() {
        let _ = c.set_text(path.clone());
    }

    #[cfg(windows)]
    if b.firefox {
        let mut reg = std::process::Command::new("reg");
        reg.args([
            "add",
            r"HKCU\Software\Mozilla\Firefox\Extensions",
            "/v",
            "zero-download@zero.app",
            "/t",
            "REG_SZ",
            "/d",
            &path,
            "/f",
        ]);
        swiftget_engine::mux::hide_window(&mut reg);
        let _ = reg.output();
    } else if b.name == "Brave" {
        if let Ok(exe_p) = std::env::current_exe() {
            let host_json = exe_p.with_file_name("net.mahmoud.zerodownload.json");
            let mut reg = std::process::Command::new("reg");
            reg.args([
                "add",
                r"HKCU\Software\BraveSoftware\Brave-Browser\NativeMessagingHosts\net.mahmoud.zerodownload",
                "/ve",
                "/t",
                "REG_SZ",
                "/d",
                &host_json.to_string_lossy(),
                "/f",
            ]);
            swiftget_engine::mux::hide_window(&mut reg);
            let _ = reg.output();
        }
    }

    launch(&exe, b.extensions_page);
    d.set_folder(path.clone().into());
    let steps = if b.firefox {
        t("Firefox opened its add-ons debugging page.\n1. Click \"Load Temporary Add-on…\".\n2. Paste the folder path (Ctrl+V, it is already copied) and open the file \"manifest.json\" in it.\n\nZero Download has also registered the extension in Windows Registry permanently so it persists across restarts!")
    } else {
        tf("{} opened its extensions page.\n1. Turn on \"Developer mode\" (a switch on that page).\n2. Click \"Load unpacked\".\n3. Paste the folder path (Ctrl+V – it is already copied) and click \"Select folder\".\n\nThat's all: the Zero Download icon appears next to the address bar. You do this only once.", &[&b.name])
    };
    d.set_steps(steps.into());
}

fn launch(exe: &Path, arg: &str) {
    let _ = std::process::Command::new(exe).arg(arg).spawn();
}

pub(crate) fn wire(ui: &Ui) {
    let Some(d) = ui.browsers.upgrade() else { return };
    {
        let ui = ui.clone();
        d.on_add(move |i| add(&ui, i as usize));
    }
    {
        let ui = ui.clone();
        d.on_copy_folder(move || {
            if let Some(d) = ui.browsers.upgrade() {
                if let Ok(mut c) = arboard::Clipboard::new() {
                    let _ = c.set_text(d.get_folder().to_string());
                }
            }
        });
    }
    {
        let w = d.as_weak();
        d.on_close_window(move || {
            if let Some(d) = w.upgrade() {
                let _ = d.hide();
            }
        });
    }
}
