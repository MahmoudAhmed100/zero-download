import os

BASE_DIR = r"c:\Users\Mahmoud\Documents\antigravity\noble-heisenberg\ezoo-download"

def update(rel_path, replacements):
    path = os.path.join(BASE_DIR, rel_path)
    if not os.path.exists(path):
        return
    with open(path, "r", encoding="utf-8", errors="ignore") as f:
        content = f.read()

    new_content = content
    for old, new in replacements:
        new_content = new_content.replace(old, new)

    if new_content != content:
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(new_content)
        print(f"Cleaned: {rel_path}")

def main():
    update("build-windows.ps1", [
        ('"ezoo-ffmpeg.zip"', '"zero-ffmpeg.zip"'),
        ('"ezoo-ffmpeg"', '"zero-ffmpeg"'),
        ("https://github.com/AhmedEzo686/ezoo-download", "https://github.com/Mahmoud/zero-download"),
        ("e.g. to C:\\Ezoo", "e.g. to C:\\Zero"),
    ])

    update(os.path.join("crates", "engine", "src", "manager.rs"), [
        ('format!(".ezoo-{id}")', 'format!(".zero-{id}")'),
        ('Ezoo Download', 'Zero Download'),
        ('EzooDownload', 'ZeroDownload'),
        ('Ezoo', 'Zero'),
    ])

    update(os.path.join("crates", "engine", "src", "mux.rs"), [
        ('Ezoo Download', 'Zero Download'),
        ('EzooDownload', 'ZeroDownload'),
    ])

    update(os.path.join("crates", "engine", "src", "tools.rs"), [
        ('ezoo-ffmpeg-', 'zero-ffmpeg-'),
        ('ezoo-tools-test-', 'zero-tools-test-'),
        ('Ezoo Download', 'Zero Download'),
        ('EzooDownload', 'ZeroDownload'),
    ])

    update(os.path.join("crates", "engine", "src", "ytdlp.rs"), [
        ('Ezoo Download', 'Zero Download'),
        ('EzooDownload.exe', 'ZeroDownload.exe'),
        ('ezoo-cookies-', 'zero-cookies-'),
    ])

    update(os.path.join("crates", "app", "src", "api.rs"), [
        ('"ezoo-api"', '"zero-api"'),
        ('Ezoo Download', 'Zero Download'),
        ('EzooDownload', 'ZeroDownload'),
        ('"app":"Ezoo"', '"app":"Zero"'),
    ])

    update(os.path.join("crates", "app", "src", "main.rs"), [
        ('// Ezoo Download', '// Zero Download'),
        ('Ezoo Download', 'Zero Download'),
        ('EzooDownload', 'ZeroDownload'),
    ])

    update(os.path.join("crates", "app", "build.rs"), [
        ('EzooDownload.exe', 'ZeroDownload.exe'),
        ('EzooDownload', 'ZeroDownload'),
        ('Ezoo Download', 'Zero Download'),
    ])

    update(os.path.join("crates", "engine", "src", "ef2.rs"), [
        ('Ezoo Download', 'Zero Download'),
    ])

    update(os.path.join("crates", "engine", "src", "selfupdate.rs"), [
        ('Ezoo Download', 'Zero Download'),
        ('EzooDownload-Setup.exe', 'ZeroDownload-Setup.exe'),
    ])

if __name__ == "__main__":
    main()
