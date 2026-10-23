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
        print(f"Cleaned final: {rel_path}")

def main():
    update("README.md", [
        (r"%APPDATA%\Ezoo\ZeroDownload\data", r"%APPDATA%\Zero\ZeroDownload\data"),
        ("**Mahmoud** — Ezoo Tech", "**Mahmoud**"),
    ])

    update(os.path.join("crates", "app", "src", "extra.rs"), [
        ('"ezoo-download-list.ef2"', '"zero-download-list.ef2"'),
        ('"ezoo-download-links.txt"', '"zero-download-links.txt"'),
    ])

    update(os.path.join("crates", "engine", "src", "config.rs"), [
        ("hands to Ezoo Download", "hands to Zero Download"),
    ])

    update(os.path.join("extension", "background.js"), [
        ("Download with Ezoo Download", "Download with Zero Download"),
        ("Download all links with Ezoo\n//     Download", "Download all links with Zero\n//     Download"),
        ("Download video with Ezoo Download", "Download video with Zero Download"),
    ])

    update(os.path.join("extension", "content.js"), [
        ("Ezoo Download is not running", "Zero Download is not running"),
        ("Sent to Ezoo Download", "Sent to Zero Download"),
    ])

    update(os.path.join("extension", "manifest.firefox.json"), [
        ('"id": "ezoo-download@ezoo.app"', '"id": "zero-download@zero.app"'),
    ])

if __name__ == "__main__":
    main()
