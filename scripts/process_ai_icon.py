import os
from PIL import Image, ImageDraw

def process_icon(src_path, output_dir, artifact_dir):
    img = Image.open(src_path).convert("RGBA")
    w, h = img.size

    # Create high-quality antialiased rounded corner mask
    # Scale up mask by 2x for smooth supersampling
    scale = 2
    mw, mh = w * scale, h * scale
    mask = Image.new("L", (mw, mh), 0)
    draw = ImageDraw.Draw(mask)

    radius = int(mw * 0.20)
    padding = int(mw * 0.02)
    draw.rounded_rectangle([padding, padding, mw - padding, mh - padding], radius=radius, fill=255)
    mask = mask.resize((w, h), Image.Resampling.LANCZOS)

    # Apply mask
    icon_rgba = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    icon_rgba.paste(img, (0, 0), mask)

    # 1. UI icon (256x256 PNG)
    ui_path = os.path.join(output_dir, "crates", "app", "ui", "icon.png")
    icon_256 = icon_rgba.resize((256, 256), Image.Resampling.LANCZOS)
    icon_256.save(ui_path, "PNG")
    print(f"Saved: {ui_path}")

    # Copy to artifact directory for presentation to user
    art_path = os.path.join(artifact_dir, "zero_download_ai_icon.png")
    icon_256.save(art_path, "PNG")
    print(f"Saved: {art_path}")

    # 2. Windows ICO with multiple layers
    ico_sizes = [(256, 256), (128, 128), (64, 64), (48, 48), (32, 32), (24, 24), (16, 16)]
    ico_images = [icon_rgba.resize(s, Image.Resampling.LANCZOS) for s in ico_sizes]
    ico_path = os.path.join(output_dir, "crates", "app", "icon.ico")
    ico_images[0].save(ico_path, format="ICO", sizes=[s for s in ico_sizes], append_images=ico_images[1:])
    print(f"Saved: {ico_path}")

    # 3. System tray 32x32 raw RGBA
    tray32 = icon_rgba.resize((32, 32), Image.Resampling.LANCZOS).convert("RGBA")
    tray_path = os.path.join(output_dir, "crates", "app", "ui", "tray32.rgba")
    with open(tray_path, "wb") as f:
        f.write(tray32.tobytes())
    print(f"Saved: {tray_path}")

    # 4. Extension icons
    ext_dir = os.path.join(output_dir, "extension", "icons")
    os.makedirs(ext_dir, exist_ok=True)
    for sz in [16, 32, 48, 128]:
        p = os.path.join(ext_dir, f"icon{sz}.png")
        icon_rgba.resize((sz, sz), Image.Resampling.LANCZOS).save(p, "PNG")
        print(f"Saved: {p}")

if __name__ == "__main__":
    src = r"C:\Users\Mahmoud\.gemini\antigravity\brain\84c709ea-b312-4cb6-8bce-18c085ca3843\zero_download_icon_1792795030166.jpg"
    base = r"c:\Users\Mahmoud\Documents\antigravity\noble-heisenberg\ezoo-download"
    artifact = r"C:\Users\Mahmoud\.gemini\antigravity\brain\84c709ea-b312-4cb6-8bce-18c085ca3843"
    process_icon(src, base, artifact)
