#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont, ImageOps


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "docs" / "assets" / "social-preview-dashboard.png"
OUTPUT = ROOT / "docs" / "hero-banner.png"
WIDTH = 1600
HEIGHT = 560

BG = "#070b11"
PANEL = "#0c121b"
BORDER = "#273342"
TEXT = "#f3efe6"
MUTED = "#97a3b2"
AMBER = "#f2bd63"
AMBER_SOFT = "#f6d28e"
TEAL = "#79d7d0"


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


def add_soft_glow(base: Image.Image, center: tuple[int, int], radius: int, color: str, alpha: int):
    layer = Image.new("RGBA", base.size, (0, 0, 0, 0))
    draw = ImageDraw.Draw(layer)
    x, y = center
    draw.ellipse((x - radius, y - radius, x + radius, y + radius), fill=rgba(color, alpha))
    layer = layer.filter(ImageFilter.GaussianBlur(radius=100))
    base.alpha_composite(layer)


def draw_background(base: Image.Image):
    draw = ImageDraw.Draw(base, "RGBA")

    add_soft_glow(base, (220, 120), 250, AMBER, 65)
    add_soft_glow(base, (1310, 120), 250, TEAL, 35)
    add_soft_glow(base, (960, 480), 220, AMBER_SOFT, 22)

    for x in range(0, WIDTH, 48):
        draw.line((x, 0, x, HEIGHT), fill=rgba("#13202c", 68), width=1)
    for y in range(0, HEIGHT, 48):
        draw.line((0, y, WIDTH, y), fill=rgba("#13202c", 68), width=1)

    draw.rounded_rectangle(
        (28, 28, WIDTH - 28, HEIGHT - 28),
        radius=30,
        fill=rgba(BG, 220),
        outline=rgba(BORDER, 255),
        width=2,
    )
    draw.line((86, 82, 320, 82), fill=rgba(AMBER, 255), width=3)
    draw.line((1248, 82, 1516, 82), fill=rgba(TEAL, 180), width=2)


def draw_screenshot(base: Image.Image):
    screenshot = Image.open(SOURCE).convert("RGB")
    screenshot = ImageOps.fit(screenshot, (880, 430), method=Image.Resampling.LANCZOS)

    frame_box = (676, 78, 1556, 508)
    shadow_box = (692, 92, 1572, 524)

    shadow = Image.new("RGBA", base.size, (0, 0, 0, 0))
    shadow_draw = ImageDraw.Draw(shadow)
    shadow_draw.rounded_rectangle(shadow_box, radius=30, fill=rgba("#000000", 165))
    shadow = shadow.filter(ImageFilter.GaussianBlur(radius=28))
    base.alpha_composite(shadow)

    frame = Image.new("RGBA", base.size, (0, 0, 0, 0))
    frame_draw = ImageDraw.Draw(frame)
    frame_draw.rounded_rectangle(
        frame_box,
        radius=30,
        fill=rgba(PANEL, 255),
        outline=rgba(AMBER_SOFT, 132),
        width=2,
    )
    base.alpha_composite(frame)

    shot = Image.new("RGBA", base.size, (0, 0, 0, 0))
    shot.paste(screenshot, (676, 78))
    mask = Image.new("L", base.size, 0)
    mask_draw = ImageDraw.Draw(mask)
    mask_draw.rounded_rectangle(frame_box, radius=30, fill=255)
    clipped = Image.new("RGBA", base.size, (0, 0, 0, 0))
    clipped.paste(shot, mask=mask)
    base.alpha_composite(clipped)

    gloss = Image.new("RGBA", base.size, (0, 0, 0, 0))
    gloss_draw = ImageDraw.Draw(gloss)
    gloss_draw.polygon(
        [(676, 78), (948, 78), (880, 260), (676, 260)],
        fill=rgba("#ffffff", 22),
    )
    gloss = gloss.filter(ImageFilter.GaussianBlur(radius=12))
    base.alpha_composite(gloss)


def draw_chip(draw: ImageDraw.ImageDraw, x: int, y: int, text: str, color: str):
    font = load_font(18, mono=True)
    left = x
    top = y
    width = int(draw.textlength(text, font=font)) + 34
    draw.rounded_rectangle(
        (left, top, left + width, top + 38),
        radius=13,
        fill=rgba(PANEL, 255),
        outline=rgba(color, 255),
        width=2,
    )
    draw.text((left + 16, top + 9), text, font=font, fill=color)


def draw_text(base: Image.Image):
    draw = ImageDraw.Draw(base, "RGBA")

    overline = load_font(18, bold=True, mono=True)
    title = load_font(86, bold=True)
    subtitle = load_font(28)
    body = load_font(21)
    footer = load_font(18, mono=True)

    draw.text((86, 110), "LOCALHOST TRUST BOUNDARY", font=overline, fill=AMBER)
    draw.text((86, 150), "Eidra", font=title, fill=TEXT)
    draw.text(
        (88, 256),
        "See what your AI tools are sending.\nMask, block, or route it before it leaves.",
        font=subtitle,
        fill=TEXT,
        spacing=12,
    )

    draw.text(
        (90, 352),
        "Built for Cursor, Claude Code, Codex, SDK workflows, GitHub Actions, and MCP.",
        font=body,
        fill=MUTED,
    )

    chip_y = 412
    draw_chip(draw, 90, chip_y, "inspect egress", TEAL)
    draw_chip(draw, 254, chip_y, "control mcp", AMBER)
    draw_chip(draw, 406, chip_y, "route local", AMBER_SOFT)

    draw.text((680, 42), "ACTUAL EIDRA DASHBOARD", font=overline, fill=MUTED)
    draw.text((88, 508), "Proxy + MCP firewall + live terminal dashboard", font=footer, fill=MUTED)


def main():
    base = Image.new("RGBA", (WIDTH, HEIGHT), BG)
    draw_background(base)
    draw_screenshot(base)
    draw_text(base)
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    base.convert("RGB").save(OUTPUT, quality=95)
    print(f"wrote {OUTPUT}")


if __name__ == "__main__":
    main()
