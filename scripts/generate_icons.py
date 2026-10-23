import os
import math
from PIL import Image, ImageDraw, ImageFilter

def create_zero_download_icon(size=512):
    # Create image with transparent background
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Base dimensions
    padding = int(size * 0.06)
    corner_radius = int(size * 0.22)
    box = [padding, padding, size - padding, size - padding]

    # Background: Smooth dark navy rounded rectangle with subtle gradient
    # We create a mask for the rounded box
    mask = Image.new("L", (size, size), 0)
    mask_draw = ImageDraw.Draw(mask)
    mask_draw.rounded_rectangle(box, radius=corner_radius, fill=255)

    # Base gradient for background
    bg = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    bg_draw = ImageDraw.Draw(bg)
    for y in range(size):
        ratio = y / size
        # From very deep obsidian navy to dark midnight cyan
        r = int(11 + ratio * 12)
        g = int(15 + ratio * 28)
        b = int(28 + ratio * 55)
        bg_draw.line([(0, y), (size, y)], fill=(r, g, b, 255))

    # Composite rounded background
    img.paste(bg, (0, 0), mask)

    # Subtle inner border glow
    border_img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    b_draw = ImageDraw.Draw(border_img)
    b_draw.rounded_rectangle(box, radius=corner_radius, outline=(0, 242, 254, 80), width=max(2, int(size * 0.015)))
    img = Image.alpha_composite(img, border_img)

    # Glowing aura behind the central "Z" / Arrow
    glow = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    g_draw = ImageDraw.Draw(glow)
    cx, cy = size // 2, size // 2
    r_glow = int(size * 0.28)
    g_draw.ellipse([cx - r_glow, cy - r_glow, cx + r_glow, cy + r_glow], fill=(0, 210, 255, 60))
    glow = glow.filter(ImageFilter.GaussianBlur(int(size * 0.1)))
    img = Image.alpha_composite(img, glow)

    # Render Modern "Z" fused with Download Arrow
    # Coordinates mapped in 0..1 range
    # Concept: Top bar of Z, diagonal stroke of Z that transforms into a downward download arrowhead
    fg = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    fg_draw = ImageDraw.Draw(fg)

    # Coordinates
    # Top horizontal bar of Z
    t_left = (int(size * 0.26), int(size * 0.25))
    t_right = (int(size * 0.74), int(size * 0.25))
    bar_h = int(size * 0.11)

    # Top bar polygon with rounded/slanted ends
    fg_draw.polygon([
        (t_left[0], t_left[1]),
        (t_right[0], t_right[1]),
        (t_right[0] - int(size * 0.06), t_right[1] + bar_h),
        (t_left[0], t_left[1] + bar_h),
    ], fill=(255, 255, 255, 255))

    # Diagonal stroke of Z leading toward center-bottom
    diag_poly = [
        (t_right[0], t_right[1]),
        (t_right[0] - int(size * 0.08), t_right[1] + bar_h),
        (int(size * 0.34), int(size * 0.58)),
        (int(size * 0.44), int(size * 0.58)),
    ]
    # Let's create an iconic geometric emblem:
    # A futuristic Z whose bottom sweep forms a downward arrow
    emblem = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    e_draw = ImageDraw.Draw(emblem)

    # Modern Cyber Z Shape
    # Top bar
    e_draw.polygon([
        (int(size * 0.24), int(size * 0.24)),
        (int(size * 0.76), int(size * 0.24)),
        (int(size * 0.68), int(size * 0.36)),
        (int(size * 0.24), int(size * 0.36)),
    ], fill=(255, 255, 255, 255))

    # Diagonal beam
    e_draw.polygon([
        (int(size * 0.76), int(size * 0.24)),
        (int(size * 0.68), int(size * 0.36)),
        (int(size * 0.32), int(size * 0.66)),
        (int(size * 0.44), int(size * 0.66)),
    ], fill=(255, 255, 255, 255))

    # Bottom bar transitioning to an arrow point
    e_draw.polygon([
        (int(size * 0.24), int(size * 0.66)),
        (int(size * 0.54), int(size * 0.66)),
        (int(size * 0.54), int(size * 0.58)),
        (int(size * 0.76), int(size * 0.72)), # Arrowhead tip right/down
        (int(size * 0.54), int(size * 0.86)),
        (int(size * 0.54), int(size * 0.78)),
        (int(size * 0.24), int(size * 0.78)),
    ], fill=(255, 255, 255, 255))

    # Center download arrow shaft inside the Z space
    arrow_w = int(size * 0.10)
    arrow_poly = [
        (cx - arrow_w//2, int(size * 0.38)),
        (cx + arrow_w//2, int(size * 0.38)),
        (cx + arrow_w//2, int(size * 0.58)),
        (cx + int(size * 0.16), int(size * 0.58)),
        (cx, int(size * 0.78)),
        (cx - int(size * 0.16), int(size * 0.78)),
        (cx - arrow_w//2, int(size * 0.58)),
    ]

    # Let's create a pure, pristine, stunning visual:
    # Outer geometric '0' (Zero) ring split by an energetic downward download Arrow,
    # forming both a '0' and a 'Z' symmetry!
    pure = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    p_draw = ImageDraw.Draw(pure)

    # 1. Stylized Upper Arc / Z top
    # Let's paint a vibrant gradient mask
    shape_mask = Image.new("L", (size, size), 0)
    s_draw = ImageDraw.Draw(shape_mask)

    # Top segment of Z
    s_draw.polygon([
        (int(size * 0.22), int(size * 0.24)),
        (int(size * 0.78), int(size * 0.24)),
        (int(size * 0.68), int(size * 0.36)),
        (int(size * 0.22), int(size * 0.36)),
    ], fill=255)

    # Diagonal of Z
    s_draw.polygon([
        (int(size * 0.78), int(size * 0.24)),
        (int(size * 0.65), int(size * 0.24)),
        (int(size * 0.26), int(size * 0.65)),
        (int(size * 0.39), int(size * 0.65)),
    ], fill=255)

    # Bottom segment of Z
    s_draw.polygon([
        (int(size * 0.22), int(size * 0.65)),
        (int(size * 0.78), int(size * 0.65)),
        (int(size * 0.78), int(size * 0.77)),
        (int(size * 0.32), int(size * 0.77)),
    ], fill=255)

    # Dynamic Downward Arrow piercing through
    arrow_head = [
        (cx, int(size * 0.88)),                      # Arrow bottom tip
        (cx + int(size * 0.22), int(size * 0.65)),  # Right wing
        (cx + int(size * 0.08), int(size * 0.65)),  # Right inner
        (cx + int(size * 0.08), int(size * 0.38)),  # Right shaft
        (cx - int(size * 0.08), int(size * 0.38)),  # Left shaft
        (cx - int(size * 0.08), int(size * 0.65)),  # Left inner
        (cx - int(size * 0.22), int(size * 0.65)),  # Left wing
    ]
    s_draw.polygon(arrow_head, fill=255)

    # Gradient shader for the emblem: Cyan (#00F2FE) -> Electric Sky Blue (#4FACFE) -> Neon Violet (#7F00FF)
    grad = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    g_draw = ImageDraw.Draw(grad)
    for y in range(size):
        ratio = y / size
        # RGB interpolation
        r = int(0 + ratio * 130)
        g = int(242 - ratio * 110)
        b = int(254)
        g_draw.line([(0, y), (size, y)], fill=(r, g, b, 255))

    # Apply gradient to the shape mask
    emblem_colored = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    emblem_colored.paste(grad, (0, 0), shape_mask)

    # Add subtle white glossy top highlight on emblem
    highlight = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    h_draw = ImageDraw.Draw(highlight)
    h_draw.polygon([
        (int(size * 0.22), int(size * 0.24)),
        (int(size * 0.78), int(size * 0.24)),
        (int(size * 0.72), int(size * 0.29)),
        (int(size * 0.22), int(size * 0.29)),
    ], fill=(255, 255, 255, 120))
    emblem_colored = Image.alpha_composite(emblem_colored, highlight)

    # Composite onto final image
    img = Image.alpha_composite(img, emblem_colored)

    return img

def main():
    base_dir = r"c:\Users\Mahmoud\Documents\antigravity\noble-heisenberg\ezoo-download"
    master = create_zero_download_icon(512)

    # 1. UI icon (256x256 PNG)
    ui_icon = master.resize((256, 256), Image.Resampling.LANCZOS)
    ui_path = os.path.join(base_dir, "crates", "app", "ui", "icon.png")
    ui_icon.save(ui_path, "PNG")
    print(f"Saved: {ui_path}")

    # 1b. Tray raw RGBA (32x32)
    tray32 = master.resize((32, 32), Image.Resampling.LANCZOS).convert("RGBA")
    tray_path = os.path.join(base_dir, "crates", "app", "ui", "tray32.rgba")
    with open(tray_path, "wb") as f:
        f.write(tray32.tobytes())
    print(f"Saved: {tray_path}")

    # 2. Windows ICO with multiple layers
    ico_sizes = [(256, 256), (128, 128), (64, 64), (48, 48), (32, 32), (24, 24), (16, 16)]
    ico_images = [master.resize(s, Image.Resampling.LANCZOS) for s in ico_sizes]
    ico_path = os.path.join(base_dir, "crates", "app", "icon.ico")
    ico_images[0].save(ico_path, format="ICO", sizes=[s for s in ico_sizes], append_images=ico_images[1:])
    print(f"Saved: {ico_path}")

    # 3. Extension icons
    ext_sizes = [16, 32, 48, 128]
    ext_dir = os.path.join(base_dir, "extension", "icons")
    os.makedirs(ext_dir, exist_ok=True)
    for sz in ext_sizes:
        ext_img = master.resize((sz, sz), Image.Resampling.LANCZOS)
        p = os.path.join(ext_dir, f"icon{sz}.png")
        ext_img.save(p, "PNG")
        print(f"Saved: {p}")

if __name__ == "__main__":
    main()
