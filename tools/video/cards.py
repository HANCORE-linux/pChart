"""The parts of the video that are not pChart (picture 48): the title and end card, and the band under
the screen with the caption of a scene and its keys, the one just pressed lit."""

import html
import subprocess
from pathlib import Path

W, H, BAND = 1920, 1080, 90
BG, INK, BRIGHT, DIM, MUTED, RAISED, ACC = "#0d0d0d", "#d3d7b5", "#dee1c8", "#9ea188", "#585d58", "#1d201d", "#e33d84"
SANS, MONO = "Noto Sans", "JetBrains Mono, JetBrainsMono Nerd Font, monospace"


def text(x, y, value, size, fill, family=SANS, bold=False, anchor="start"):
    weight = ' font-weight="700"' if bold else ""
    return f'<text x="{x}" y="{y}" font-family="{family}" font-size="{size}" fill="{fill}" text-anchor="{anchor}"{weight}>{html.escape(value)}</text>'


def keys(right, y, names, lit=None):
    out, x = [], right
    for name in reversed(names):
        w = 22 + len(name) * 13
        x -= w
        on = name == lit
        out.append(f'<rect x="{x}" y="{y - 26}" width="{w}" height="38" rx="7" fill="{ACC if on else RAISED}" stroke="{ACC if on else MUTED}" stroke-width="1.5"/>')
        out.append(text(x + w / 2, y, name, 21, "#040704" if on else BRIGHT, MONO, True, "middle"))
        x -= 12
    return "".join(out)


def png(body, out: Path, width=W, height=H):
    svg = out.with_suffix(".svg")
    svg.write_text(f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">{body}</svg>')
    subprocess.run(["rsvg-convert", "-w", str(width), "-h", str(height), str(svg), "-o", str(out)], check=True)
    return out


def title(out: Path):
    return png("".join([
        f'<rect width="{W}" height="{H}" fill="{BG}"/>',
        text(W / 2, 470, "pChart", 128, BRIGHT, MONO, True, "middle"),
        f'<rect x="{W / 2 - 230}" y="505" width="460" height="4" fill="{ACC}"/>',
        text(W / 2, 580, "Project charts in the terminal, in plain language", 38, INK, anchor="middle"),
        text(W / 2, 640, "Sample plan: Customer portal v1", 26, DIM, anchor="middle"),
    ]), out)


def end(out: Path):
    return png("".join([
        f'<rect width="{W}" height="{H}" fill="{BG}"/>',
        text(W / 2, 400, "pChart", 96, BRIGHT, MONO, True, "middle"),
        text(W / 2, 490, "Situation · Roadmap · Checks · Export", 36, INK, anchor="middle"),
        text(W / 2, 550, "Change a date, try what-if, add notes: every change is saved and can be undone.", 26, DIM, anchor="middle"),
        text(W / 2, 600, "The plan file is the only source of truth; pictures, PDF and Markdown come from it.", 26, DIM, anchor="middle"),
        text(W / 2, 650, "Follows the Omarchy theme · runs in kitty, Ghostty and foot", 26, DIM, anchor="middle"),
    ]), out)


def band(out: Path, left, right, caption, names, lit=None):
    """The band under the screen, from `left` to `right`: the caption, or with `lit` only the keys."""
    body = keys(right, 54, names, lit) if lit else text(left, 54, caption, 25, INK) + keys(right, 54, names)
    return png(body, out, W, BAND)
