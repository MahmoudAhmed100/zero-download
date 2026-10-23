import os
import re

BASE_DIR = r"c:\Users\Mahmoud\Documents\antigravity\noble-heisenberg\ezoo-download"

def update_file(rel_path, replacements):
    path = os.path.join(BASE_DIR, rel_path)
    if not os.path.exists(path):
        print(f"File not found: {path}")
        return
    with open(path, "r", encoding="utf-8", errors="ignore") as f:
        content = f.read()

    new_content = content
    for old, new in replacements:
        new_content = new_content.replace(old, new)

    if new_content != content:
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(new_content)
        print(f"Updated: {rel_path}")
    else:
        print(f"No changes: {rel_path}")

def main():
    # 1. Update dialogs.slint AboutDialog specifically
    dialogs_path = os.path.join(BASE_DIR, "crates", "app", "ui", "dialogs.slint")
    with open(dialogs_path, "r", encoding="utf-8") as f:
        d_content = f.read()

    # In dialogs.slint:
    d_content = d_content.replace('@tr("About Ezoo Download")', '@tr("About Zero Download")')
    d_content = d_content.replace('text: "Ezoo Download";', 'text: "Zero Download";')
    d_content = d_content.replace('@tr("Coded by Ahmed Ezzat")', '@tr("Developed by Mahmoud")')
    d_content = d_content.replace('@tr("Ezoo Download")', '@tr("Zero Download")')
    d_content = d_content.replace('Ezoo Download', 'Zero Download')
    d_content = d_content.replace('EzooDownload', 'ZeroDownload')
    d_content = d_content.replace('https://www.facebook.com/profile.php?id=61558130707674', 'https://www.facebook.com/share/19gJCRExaT')

    # Remove YouTube button from AboutDialog in dialogs.slint
    yt_button_pattern = r'Button\s*\{\s*text:\s*"YouTube";\s*clicked\s*=>\s*\{[^}]*\}\s*\}'
    d_content = re.sub(yt_button_pattern, '', d_content)

    with open(dialogs_path, "w", encoding="utf-8", newline="\n") as f:
        f.write(d_content)
    print("Updated dialogs.slint")

    # 2. Update app.slint
    update_file(os.path.join("crates", "app", "ui", "app.slint"), [
        ("Ezoo Download", "Zero Download"),
        ("EzooDownload", "ZeroDownload"),
    ])

    # 3. Update common.slint
    update_file(os.path.join("crates", "app", "ui", "common.slint"), [
        ("Ezoo Download", "Zero Download"),
        ("EzooDownload", "ZeroDownload"),
    ])

    # 4. Update Rust source files in crates/app/src
    for fn in ["browser.rs", "extra.rs", "grab.rs", "options.rs", "tray.rs", "update.rs", "i18n.rs"]:
        update_file(os.path.join("crates", "app", "src", fn), [
            ("Restore Ezoo Download", "Restore Zero Download"),
            ("Ezoo Download", "Zero Download"),
            ("EzooDownload", "ZeroDownload"),
            ("Ezoo", "Zero"),
            ("Ahmed Ezzat", "Mahmoud"),
            ("AhmedEzo686", "Mahmoud"),
            ("https://www.youtube.com/@Ezoo-Tech", "https://www.facebook.com/share/19gJCRExaT"),
            ("https://www.facebook.com/profile.php?id=61558130707674", "https://www.facebook.com/share/19gJCRExaT"),
        ])

    # 5. Update translations
    po_path = os.path.join("crates", "app", "lang", "ar", "LC_MESSAGES", "swiftget.po")
    update_file(po_path, [
        ("msgid \"Ezoo Download\"", "msgid \"Zero Download\""),
        ("msgstr \"Ezoo Download\"", "msgstr \"Zero Download\""),
        ("msgid \"About Ezoo Download\"", "msgid \"About Zero Download\""),
        ("msgstr \"عن Ezoo Download\"", "msgstr \"عن Zero Download\""),
        ("msgid \"Coded by Ahmed Ezzat\"", "msgid \"Developed by Mahmoud\""),
        ("msgstr \"Coded by Ahmed Ezzat\"", "msgstr \"برمجة وتطوير: Mahmoud\""),
        ("Ezoo Download", "Zero Download"),
        ("EzooDownload", "ZeroDownload"),
        ("Ahmed Ezzat", "Mahmoud"),
    ])

    update_file(os.path.join("crates", "app", "lang", "ar_ui.py"), [
        ("Ezoo Download", "Zero Download"),
        ("EzooDownload", "ZeroDownload"),
        ("Ahmed Ezzat", "Mahmoud"),
    ])

    # 6. Update Browser Extension
    update_file(os.path.join("extension", "manifest.json"), [
        ('"name": "Ezoo Download"', '"name": "Zero Download"'),
        ('"description": "Ezoo Download extension."', '"description": "Zero Download extension."'),
        ('"default_title": "Ezoo Download"', '"default_title": "Zero Download"'),
    ])

    update_file(os.path.join("extension", "manifest.firefox.json"), [
        ('"name": "Ezoo Download"', '"name": "Zero Download"'),
        ('"description": "Ezoo Download extension."', '"description": "Zero Download extension."'),
        ('"default_title": "Ezoo Download"', '"default_title": "Zero Download"'),
    ])

    update_file(os.path.join("extension", "popup.html"), [
        ("Ezoo Download", "Zero Download"),
    ])

    update_file(os.path.join("extension", "background.js"), [
        ("Ezoo Download", "Zero Download"),
        ("EzooDownload", "ZeroDownload"),
    ])

    # 7. Update .cargo/config.toml
    update_file(os.path.join(".cargo", "config.toml"), [
        ("EzooDownload", "ZeroDownload"),
    ])

    # 8. Update THIRD-PARTY-NOTICES.txt
    update_file("THIRD-PARTY-NOTICES.txt", [
        ("Ezoo Download", "Zero Download"),
    ])

    # 9. Update Inno Setup script (installer/zero.iss)
    iss_in = os.path.join(BASE_DIR, "installer", "ezoo.iss")
    iss_out = os.path.join(BASE_DIR, "installer", "zero.iss")
    with open(iss_in, "r", encoding="utf-8") as f:
        iss_content = f.read()

    # Rebrand installer
    iss_content = iss_content.replace('#define MyAppName "Ezoo Download"', '#define MyAppName "Zero Download"')
    iss_content = iss_content.replace('#define MyAppExe "EzooDownload.exe"', '#define MyAppExe "ZeroDownload.exe"')
    iss_content = iss_content.replace('AppId={{F5837666-2925-4FD9-8CE9-728BE9C5D7E4}', 'AppId={{B4791E88-518A-42F7-98C3-3E9C1D729F41}')
    iss_content = iss_content.replace('AppPublisher=Ahmed Ezzat', 'AppPublisher=Mahmoud')
    iss_content = iss_content.replace('DefaultDirName={localappdata}\\Programs\\Ezoo Download', 'DefaultDirName={localappdata}\\Programs\\Zero Download')
    iss_content = iss_content.replace('OutputBaseFilename=EzooDownload-Setup', 'OutputBaseFilename=ZeroDownload-Setup')
    iss_content = iss_content.replace('VersionInfoCompany=Ezoo', 'VersionInfoCompany=Zero Download')
    iss_content = iss_content.replace('VersionInfoDescription=Ezoo Download Setup', 'VersionInfoDescription=Zero Download Setup')
    iss_content = iss_content.replace('VersionInfoCopyright=Copyright (C) 2026 Ahmed Ezzat', 'VersionInfoCopyright=Copyright (C) 2026 Mahmoud')
    iss_content = iss_content.replace('AppPublisherURL=https://www.youtube.com/@Ezoo-Tech', 'AppPublisherURL=https://www.facebook.com/share/19gJCRExaT')
    iss_content = iss_content.replace('Subkey: "Software\\Ezoo\\EzooDownload"', 'Subkey: "Software\\Zero\\ZeroDownload"')
    iss_content = iss_content.replace('ValueName: "EzooDownload"', 'ValueName: "ZeroDownload"')
    iss_content = iss_content.replace('RunOnceId: "StopEzoo"', 'RunOnceId: "StopZero"')
    iss_content = iss_content.replace('Ezoo Download', 'Zero Download')
    iss_content = iss_content.replace('EzooDownload', 'ZeroDownload')

    with open(iss_out, "w", encoding="utf-8", newline="\n") as f:
        f.write(iss_content)
    print("Created installer/zero.iss")

    # Also keep ezoo.iss in sync in case scripts look for it
    with open(iss_in, "w", encoding="utf-8", newline="\n") as f:
        f.write(iss_content)
    print("Updated installer/ezoo.iss")

    # 10. Update build-windows.ps1 & BUILD.bat
    update_file("build-windows.ps1", [
        ("Ezoo Download", "Zero Download"),
        ("EzooDownload", "ZeroDownload"),
        ("ezoo.iss", "zero.iss"),
        ("ezoo.pfx", "zero.pfx"),
        ("ezoo-setup-build", "zero-setup-build"),
    ])

    update_file("BUILD.bat", [
        ("SwiftGet.exe", "ZeroDownload.exe"),
    ])

    # 11. Update .github/workflows/build.yml
    update_file(os.path.join(".github", "workflows", "build.yml"), [
        ("Build Ezoo Download for Windows", "Build Zero Download for Windows"),
        ("EzooDownload.exe", "ZeroDownload.exe"),
        ("EzooDownload-Setup.exe", "ZeroDownload-Setup.exe"),
        ("EzooDownload-windows", "ZeroDownload-windows"),
        ("Ezoo Download", "Zero Download"),
        ("ezoo.iss", "zero.iss"),
    ])

    # 12. Update README.md
    readme_path = os.path.join(BASE_DIR, "README.md")
    with open(readme_path, "r", encoding="utf-8") as f:
        readme = f.read()

    readme = readme.replace("# Ezoo Download", "# Zero Download")
    readme = readme.replace("**Ahmed Ezzat**", "**Mahmoud**")
    readme = readme.replace("**Ezoo Download**", "**Zero Download**")
    readme = readme.replace("Ezoo Download", "Zero Download")
    readme = readme.replace("EzooDownload", "ZeroDownload")
    readme = readme.replace("AhmedEzo686/ezoo-download", "Mahmoud/zero-download")
    readme = readme.replace("https://www.youtube.com/@Ezoo-Tech", "https://www.facebook.com/share/19gJCRExaT")
    readme = readme.replace("https://www.facebook.com/profile.php?id=61558130707674", "https://www.facebook.com/share/19gJCRExaT")
    readme = readme.replace("© 2026 Ahmed Ezzat", "© 2026 Mahmoud")

    with open(readme_path, "w", encoding="utf-8", newline="\n") as f:
        f.write(readme)
    print("Updated README.md")

if __name__ == "__main__":
    main()
