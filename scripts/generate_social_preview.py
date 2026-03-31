#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageChops, ImageDraw, ImageFilter, ImageFont, ImageOps


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "docs" / "assets" / "social-preview-dashboard.png"
OUTPUT = ROOT / "docs" / "social-preview.png"
WIDTH = 1280
HEIGHT = 640

BG = "#080b10"
PANEL = "#0b1118"
BORDER = "#293240"
TEXT = "#f3efe6"
MUTED = "#96a1af"
AMBER = "#f2bd63"
AMBER_SOFT = "#f6d28e"
TEAL = "#74d7d2"


def load_font(size: int, *, bold: bool = False, mono: bool = False) -> ImageFont.FreeTypeFont:
    candidates: list[str] = []
    if mono:
        candidates.extend(
            [
                "/System/Library/Fonts/Menlo.ttc",
                "/System/Library/Fonts/SFNSMono.ttf",
                "/System/Library/Fonts/Supplemental/Courier New.ttf",
            ]
        )
    elif bold:
        candidates.extend(
            [
                "/System/Library/Fonts/Supplemental/Helvetica.ttc",
                "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
                "/System/Library/Fonts/Supplemental/Arial.ttf",
            ]
        )
    else:
        candidates.extend(
            [
                "/System/Library/Fonts/Supplemental/Helvetica.ttc",
                "/System/Library/Fonts/Supplemental/Arial.ttf",
            ]
        )

    for candidate in candidates:
        path = Path(candidate)
        if path.exists():
            return ImageFont.truetype(str(path), size=size)
    return ImageFont.load_default()


def rgba(hex_color: str, alpha: int) -> tuple[int, int, int, int]:
    hex_color = hex_color.lstrip("#")
    return tuple(int(hex_color[index : index + 2], 16) for index in (0, 2, 4)) + (alpha,)


def rounded_box(draw: ImageDraw.ImageDraw, box, radius, fill, outline=None, width=1):
    draw.rounded_rectangle(box, radius=radius, fill=fill, outline=outline, width=width)


def add_soft_glow(base: Image.Image, center: tuple[int, int], radius: int, color: str, alpha: int):
    layer = Image.new("RGBA", base.size, (0, 0, 0, 0))
    overlay = ImageDraw.Draw(layer)
    x, y = center
    overlay.ellipse((x - radius, y - radius, x + radius, y + radius), fill=rgba(color, alpha))
    layer = layer.filter(ImageFilter.GaussianBlur(radius=80))
    base.alpha_composite(layer)


def draw_background(base: Image.Image):
    draw = ImageDraw.Draw(base, "RGBA")

    add_soft_glow(base, (240, 150), 240, AMBER, 70)
    add_soft_glow(base, (1060, 180), 220, TEAL, 30)

    for x in range(0, WIDTH, 40):
        draw.line((x, 0, x, HEIGHT), fill=rgba("#14202c", 80), width=1)
    for y in range(0, HEIGHT, 40):
        draw.line((0, y, WIDTH, y), fill=rgba("#14202c", 80), width=1)

    outer = (40, 40, WIDTH - 40, HEIGHT - 40)
    rounded_box(draw, outer, 30, rgba(BG, 220), outline=rgba(BORDER, 255), width=2)

    draw.line((92, 86, 356, 86), fill=rgba(AMBER, 255), width=3)
    draw.line((924, 86, 1188, 86), fill=rgba(TEAL, 170), width=2)


def draw_screenshot(base: Image.Image, screenshot_path: Path):
    if not screenshot_path.exists():
        raise FileNotFoundError(f"missing screenshot source: {screenshot_path}")

    screenshot = Image.open(screenshot_path).convert("RGB")
    screenshot = ImageOps.fit(screenshot, (742, 440), method=Image.Resampling.LANCZOS)

    frame_box = (500, 104, 1242, 544)
    shadow_box = (516, 120, 1258, 560)

    shadow = Image.new("RGBA", base.size, (0, 0, 0, 0))
    shadow_draw = ImageDraw.Draw(shadow)
    shadow_draw.rounded_rectangle(shadow_box, radius=28, fill=rgba("#000000", 160))
    shadow = shadow.filter(ImageFilter.GaussianBlur(radius=24))
    base.alpha_composite(shadow)

    frame = Image.new("RGBA", base.size, (0, 0, 0, 0))
    frame_draw = ImageDraw.Draw(frame)
    rounded_box(frame_draw, frame_box, 28, rgba(PANEL, 255), outline=rgba(AMBER_SOFT, 130), width=2)
    base.alpha_composite(frame)

    shot = Image.new("RGBA", base.size, (0, 0, 0, 0))
    shot.paste(screenshot, (500, 104))
    mask = Image.new("L", base.size, 0)
    mask_draw = ImageDraw.Draw(mask)
    mask_draw.rounded_rectangle(frame_box, radius=28, fill=255)
    clipped = Image.new("RGBA", base.size, (0, 0, 0, 0))
    clipped.paste(shot, mask=mask)
    base.alpha_composite(clipped)

    gloss = Image.new("RGBA", base.size, (0, 0, 0, 0))
    gloss_draw = ImageDraw.Draw(gloss)
    gloss_draw.polygon(
        [(500, 104), (760, 104), (700, 280), (500, 280)],
        fill=rgba("#ffffff", 24),
    )
    gloss = gloss.filter(ImageFilter.GaussianBlur(radius=10))
    base.alpha_composite(gloss)


def draw_text(base: Image.Image):
    draw = ImageDraw.Draw(base, "RGBA")

    overline = load_font(18, bold=True, mono=True)
    title = load_font(72, bold=True)
    subtitle = load_font(30)
    support = load_font(22)
    body = load_font(17, mono=True)

    draw.text((90, 112), "LOCAL-FIRST TRUST LAYER", font=overline, fill=AMBER)
    draw.text((90, 150), "Eidra", font=title, fill=TEXT)
    draw.text((90, 252), "for AI development", font=subtitle, fill=AMBER_SOFT)

    support_lines = [
        "Proxy + MCP firewall",
        "Live terminal dashboard",
    ]
    y = 324
    for line in support_lines:
        draw.text((92, y), line, font=support, fill=TEXT)
        y += 34

    chip_y = 428
    for label, color, width in [
        ("inspect egress", TEAL, 168),
        ("control mcp", AMBER, 154),
        ("route local", AMBER_SOFT, 150),
    ]:
        rounded_box(draw, (92, chip_y, 92 + width, chip_y + 34), 12, rgba(PANEL, 255), outline=rgba(color, 255), width=2)
        draw.text((108, chip_y + 9), label, font=body, fill=color)
        chip_y += 50

    draw.text((502, 70), "ACTUAL TERMINAL DASHBOARD", font=overline, fill=MUTED)
    draw.text((90, 582), "See what leaves. Decide what goes.", font=body, fill=MUTED)


def main():
    base = Image.new("RGBA", (WIDTH, HEIGHT), BG)
    draw_background(base)
    draw_screenshot(base, SOURCE)
    draw_text(base)

    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    base.convert("RGB").save(OUTPUT, quality=95)
    print(f"wrote {OUTPUT}")


if __name__ == "__main__":
    main()
