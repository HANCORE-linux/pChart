"""The parts of the video that are not pChart (picture 48, in the style of the OmaQ video): the desktop
around the terminal, with the theme's wallpaper and a bar on top, the caption
of a scene on the left under it and its keys on the right, the one just pressed lit, and the end card."""

import html
import json
import subprocess
from pathlib import Path

W, H, BAR, BAND = 1920, 1080, 28, 70
MONO, SANS = "JetBrains Mono, JetBrainsMono Nerd Font, monospace", "Noto Sans"
ROOT = Path(__file__).resolve().parent.parent.parent
THEMES = json.loads((ROOT / "crates/pchart-tui/themes.json").read_text())
WALLPAPERS = {"labra": Path(__file__).resolve().parent / "wallpapers/labra.jpg", "harbor": Path(__file__).resolve().parent / "wallpapers/harbor.jpg"}


def text(x, y, value, size, fill, family=MONO, bold=False, anchor="start"):
    weight = ' font-weight="700"' if bold else ""
    return f'<text x="{x}" y="{y}" font-family="{family}" font-size="{size}" fill="{fill}" text-anchor="{anchor}"{weight}>{html.escape(value)}</text>'


def png(body, out: Path, width=W, height=H):
    svg = out.with_suffix(".svg")
    svg.write_text(f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">{body}</svg>')
    subprocess.run(["rsvg-convert", "-w", str(width), "-h", str(height), str(svg), "-o", str(out)], check=True)
    return out


def wallpaper(name, out: Path):
    """The theme's wallpaper, cut to fill the screen."""
    source = WALLPAPERS[name]
    subprocess.run(["magick", str(source), "-resize", f"{W}x{H}^", "-gravity", "center", "-extent", f"{W}x{H}", str(out)], check=True)
    return out


def bar(name, out: Path):
    """The bar along the top of the desktop in the theme's colours: the workspaces, the day and the time."""
    c = THEMES[name]
    body = [f'<rect width="{W}" height="{BAR}" fill="{c["background"]}"/>']
    for number in range(1, 5):
        body.append(text(18 + (number - 1) * 22, 20, str(number), 15, "#ee585d" if number == 1 and name == "labra" else c["foreground"] if number == 1 else c["muted"]))
    body.append(text(W / 2, 20, "Thursday 09:41", 15, c["foreground"], anchor="middle"))
    for number in range(3):
        body.append(f'<rect x="{W - 30 - number * 24}" y="10" width="12" height="10" rx="2" fill="none" stroke="{c["foreground"]}" stroke-width="1.5"/>')
    return png("".join(body), out, W, BAR)


# How Labra decorates a window in Hyprland (its hyprland.lua): a border of 1 px running from pink at the
# bottom to olive at the top, corners rounded by 4 px, the window at 94 % opacity, and a soft dark shadow
# 20 px wide, 4 px down.
BORDER, ROUNDING, OPACITY, SHADOW, MARGIN = 1, 4, 0.94, 20, 40
GRADIENT = [("0", "#fd5ea0"), ("0.143", "#fd5ea0"), ("0.286", "#89974a"), ("1", "#89974a")]


def window(out: Path, width, height):
    """The decoration around a window of `width` × `height`, drawn on a canvas `MARGIN` larger on every
    side: the shadow under it and the border around it, with the window itself left clear."""
    w, h, m, b, r = width, height, MARGIN, BORDER, ROUNDING
    stops = "".join(f'<stop offset="{at}" stop-color="{color}" stop-opacity="0.7"/>' for at, color in reversed([(f"{1 - float(at):.3f}", color) for at, color in GRADIENT]))
    return png("".join([
        f'<defs><linearGradient id="edge" x1="0" y1="0" x2="0" y2="1">{stops}</linearGradient>',
        f'<filter id="soft" x="-20%" y="-20%" width="140%" height="140%"><feGaussianBlur stdDeviation="{SHADOW / 3}"/></filter>',
        f'<mask id="hole"><rect width="{w + 2 * m}" height="{h + 2 * m}" fill="white"/><rect x="{m}" y="{m}" width="{w}" height="{h}" rx="{r}" fill="black"/></mask></defs>',
        f'<g mask="url(#hole)"><rect x="{m}" y="{m + 4}" width="{w}" height="{h}" rx="{r}" fill="#050604" fill-opacity="0.9" filter="url(#soft)"/>',
        f'<rect x="{m - b / 2}" y="{m - b / 2}" width="{w + b}" height="{h + b}" rx="{r + b / 2}" fill="none" stroke="url(#edge)" stroke-width="{b}"/></g>',
    ]), out, w + 2 * m, h + 2 * m)


def corners(out: Path, width, height):
    """The window's shape as a mask: white inside the rounded corners."""
    return png(f'<rect width="{width}" height="{height}" fill="black"/><rect width="{width}" height="{height}" rx="{ROUNDING}" fill="white"/>', out, width, height)


def keys(right, y, names, lit, c):
    out, x = [], right
    for name in reversed(names):
        w = 22 + len(name) * 12
        x -= w
        on = name == lit
        out.append(f'<rect x="{x}" y="{y - 24}" width="{w}" height="34" rx="6" fill="{c["accent"] if on else "#0d0d0d"}" fill-opacity="{1 if on else 0.82}" stroke="{c["accent"] if on else "#585d58"}" stroke-width="1.5"/>')
        out.append(text(x + w / 2, y, name, 19, "#040704" if on else "#dee1c8", MONO, True, "middle"))
        x -= 10
    return "".join(out)


def band(name, out: Path, caption, names, lit=None):
    """The caption of a scene in a dark box on the left, and its keys on the right."""
    c = THEMES[name]
    body = []
    if caption:
        width = 28 + len(caption) * 12.6
        body += [f'<rect x="20" y="16" width="{width}" height="36" rx="3" fill="#0d0d0d" fill-opacity="0.86"/>', text(34, 40, caption, 21, "#dee1c8")]
    body.append(keys(W - 20, 44, names, lit, c))
    return png("".join(body), out, W, BAND)


def end(out: Path):
    """The end card: the name, what pChart is, and how to start it."""
    icon = f'<rect x="{W / 2 - 330}" y="330" width="150" height="150" rx="33" fill="#c026d3"/>' + text(W / 2 - 255, 437, "P", 92, "#ffffff", SANS, True, "middle")
    name = text(W / 2 - 150, 445, "pChart", 120, "#dee1c8", MONO, True)
    prompt = text(W / 2 - 110, 690, "~ ❯ pchart", 34, "#d3d7b5") + f'<rect x="{W / 2 + 106}" y="662" width="18" height="36" fill="#e33d84"/>'
    return png("".join([
        f'<rect width="{W}" height="{H}" fill="#000000"/>', icon, name,
        text(W / 2, 590, "Project charts that check themselves.", 44, "#d3d7b5", SANS, anchor="middle"), prompt,
        text(W / 2, 760, "Rust · MIT · kitty, Ghostty and foot", 20, "#585d58", MONO, anchor="middle"),
    ]), out)
