#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "docs" / "social-preview.png"
WIDTH = 1280
HEIGHT = 640
BG = "#080b10"
PANEL = "#0e131b"
PANEL_2 = "#121923"
BORDER = "#2d3948"
TEXT = "#f5f1e8"
MUTED = "#9ea8b6"
AMBER = "#f6b756"
AMBER_2 = "#f2cf83"
TEAL = "#6ed6cf"
RED = "#ef7b73"
GREEN = "#74d99f"


def load_font(size: int, bold: bool = False, mono: bool = False) -> ImageFont.FreeTypeFont:
    candidates = []
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


def rounded_box(draw: ImageDraw.ImageDraw, box, radius, fill, outline=None, width=1):
    draw.rounded_rectangle(box, radius=radius, fill=fill, outline=outline, width=width)


def add_glow(base: Image.Image, center: tuple[int, int], radius: int, color: str, alpha: int):
    glow = Image.new("RGBA", base.size, (0, 0, 0, 0))
    overlay = ImageDraw.Draw(glow)
    x, y = center
    overlay.ellipse(
        (x - radius, y - radius, x + radius, y + radius),
        fill=hex_to_rgba(color, alpha),
    )
    glow = glow.filter(ImageFilter.GaussianBlur(radius=60))
    base.alpha_composite(glow)


def hex_to_rgba(color: str, alpha: int):
    color = color.lstrip("#")
    return tuple(int(color[i : i + 2], 16) for i in (0, 2, 4)) + (alpha,)


def draw_grid(draw: ImageDraw.ImageDraw):
    for x in range(0, WIDTH, 32):
        draw.line((x, 0, x, HEIGHT), fill=hex_to_rgba("#17202b", 100), width=1)
    for y in range(0, HEIGHT, 32):
        draw.line((0, y, WIDTH, y), fill=hex_to_rgba("#17202b", 100), width=1)


def draw_header(draw: ImageDraw.ImageDraw, font_small, font_body, font_title):
    draw.text((84, 82), "Eidra", font=font_title, fill=TEXT)
    draw.text(
        (86, 166),
        "Local-first trust layer for AI development",
        font=font_body,
        fill=AMBER_2,
    )
    body = (
        "Scan, control, and explain AI traffic before it leaves your machine.\n"
        "Proxy + MCP firewall + live dashboard."
    )
    draw.multiline_text((86, 228), body, font=font_body, fill=TEXT, spacing=14)

    badges = [
        ("OBSERVE", TEAL),
        ("MASK / BLOCK", AMBER),
        ("ROUTE LOCAL", GREEN),
    ]
    x = 86
    for label, color in badges:
        w = draw.textlength(label, font=font_small) + 28
        rounded_box(draw, (x, 318, x + int(w), 352), 12, PANEL_2, outline=color, width=2)
        draw.text((x + 14, 327), label, font=font_small, fill=color)
        x += int(w) + 12


def draw_panel_labels(draw: ImageDraw.ImageDraw, font_small, font_mono):
    draw.text((86, 390), "LOCAL TRUST BOUNDARY", font=font_small, fill=MUTED)
    draw.text((844, 116), "LIVE CONTAINMENT", font=font_small, fill=MUTED)
    draw.text((844, 416), "DECISION RAIL", font=font_small, fill=MUTED)
    draw.text((88, 573), "tool -> Eidra -> cloud/local", font=font_mono, fill=MUTED)


def draw_flow(base: Image.Image, draw: ImageDraw.ImageDraw, font_label, font_small, font_mono):
    left = (84, 420, 670, 556)
    rounded_box(draw, left, 26, PANEL, outline=BORDER, width=2)

    nodes = [
        ("AI TOOL", 126, 447, "#253240", TEAL),
        ("EIDRA", 316, 447, "#261d14", AMBER),
        ("CLOUD", 506, 435, "#1c2430", RED),
        ("LOCAL", 506, 483, "#15271e", GREEN),
    ]

    for label, x, y, fill, stroke in nodes:
        rounded_box(draw, (x, y, x + 126, y + 42), 16, fill, outline=stroke, width=2)
        draw.text((x + 18, y + 11), label, font=font_label, fill=TEXT)

    draw.line((252, 468, 316, 468), fill=AMBER, width=4)
    draw.line((442, 468, 506, 456), fill=RED, width=4)
    draw.line((442, 468, 506, 504), fill=GREEN, width=4)
    draw.text((340, 431), "inspect", font=font_small, fill=AMBER_2)
    draw.text((458, 437), "mask / block", font=font_small, fill=RED)
    draw.text((460, 495), "route local", font=font_small, fill=GREEN)

    log_lines = [
        "[ok] cloud prompt sanitized",
        "[ok] mcp write tool blocked",
        "[ok] secrets routed local",
    ]
    log_y = 580
    for idx, line in enumerate(log_lines):
        draw.text((720, log_y + idx * 18), line, font=font_mono, fill=MUTED)

    glow = Image.new("RGBA", base.size, (0, 0, 0, 0))
    glow_draw = ImageDraw.Draw(glow)
    glow_draw.rounded_rectangle((314, 445, 444, 491), radius=18, outline=hex_to_rgba(AMBER, 180), width=5)
    glow = glow.filter(ImageFilter.GaussianBlur(10))
    base.alpha_composite(glow)


def draw_dashboard_panel(draw: ImageDraw.ImageDraw, font_small, font_mono):
    panel = (820, 144, 1188, 366)
    rounded_box(draw, panel, 24, PANEL, outline=BORDER, width=2)

    bars = [
        ("allow", 0.78, TEAL),
        ("mask", 0.58, AMBER),
        ("route", 0.42, GREEN),
        ("block", 0.18, RED),
    ]
    y = 180
    for label, ratio, color in bars:
        draw.text((846, y), label.upper(), font=font_small, fill=MUTED)
        rounded_box(draw, (930, y - 4, 1162, y + 20), 10, "#111821", outline="#1d2632", width=1)
        rounded_box(draw, (934, y, 934 + int(220 * ratio), y + 16), 8, color)
        y += 44

    metrics = [
        ("traffic", "proxied"),
        ("mcp", "firewalled"),
        ("audit", "local sqlite"),
    ]
    y = 314
    for key, value in metrics:
        draw.text((846, y), key, font=font_mono, fill=MUTED)
        draw.text((980, y), value, font=font_mono, fill=TEXT)
        y += 18


def draw_rail(draw: ImageDraw.ImageDraw, font_small, font_mono):
    panel = (820, 444, 1188, 556)
    rounded_box(draw, panel, 24, PANEL, outline=BORDER, width=2)
    entries = [
        ("scan", AMBER_2),
        ("policy", AMBER),
        ("local route", GREEN),
        ("audit", TEAL),
    ]
    x = 852
    for idx, (label, color) in enumerate(entries):
        if idx:
            draw.line((x - 18, 500, x - 2, 500), fill=BORDER, width=3)
        rounded_box(draw, (x, 478, x + 72, 522), 16, PANEL_2, outline=color, width=2)
        draw.text((x + 10, 493), label, font=font_small, fill=color)
        x += 90
    draw.text((850, 533), "Designed for quick first-glance trust decisions.", font=font_mono, fill=MUTED)


def main():
    image = Image.new("RGBA", (WIDTH, HEIGHT), BG)
    add_glow(image, (280, 190), 220, AMBER, 80)
    add_glow(image, (1020, 150), 160, TEAL, 40)
    add_glow(image, (960, 500), 180, AMBER, 30)

    draw = ImageDraw.Draw(image, "RGBA")
    draw_grid(draw)

    outer = (48, 46, WIDTH - 48, HEIGHT - 46)
    rounded_box(draw, outer, 32, hex_to_rgba("#091019", 210), outline=hex_to_rgba(BORDER, 255), width=2)

    font_title = load_font(64, bold=True)
    font_body = load_font(28)
    font_label = load_font(20, bold=True)
    font_small = load_font(16, bold=True)
    font_mono = load_font(15, mono=True)

    draw_header(draw, font_small, font_body, font_title)
    draw_panel_labels(draw, font_small, font_mono)
    draw_flow(image, draw, font_label, font_small, font_mono)
    draw_dashboard_panel(draw, font_small, font_mono)
    draw_rail(draw, font_small, font_mono)

    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    image.convert("RGB").save(OUTPUT, quality=95)
    print(f"wrote {OUTPUT}")


if __name__ == "__main__":
    main()
