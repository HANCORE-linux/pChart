#!/usr/bin/env python3
"""Puts the video together as in the storyboard (picture 48): the title card, the recorded screens with
the caption and keys of each scene under them, the key just pressed lit, and the end card. Writes
pchart.mp4 (1920x1080, 30 frames per second, H.264) and pchart.gif (1280x720, 12 frames per second)
into the recording's folder.
usage: compose.py OUT"""

import shutil
import struct
import subprocess
import sys
from pathlib import Path

import cards
from timeline import END, KEYS, LEAD, SCENES

FPS, CARD, FADE, LIT = 30, 3.0, 0.4, 0.6
CHIPS = {"up": "↑↓", "down": "↑↓", "left": "←→", "right": "←→", "enter": "Enter", "esc": "Esc"}

missing = [tool for tool in ["ffmpeg", "rsvg-convert"] if not shutil.which(tool)]
if missing:
    sys.exit(f"compose.py needs {', '.join(missing)}")
recording = Path(sys.argv[1]).resolve()
work = recording / "compose"
work.mkdir(exist_ok=True)

times = [float(line) for line in (recording / "times.txt").read_text().split()]
frames = sorted((recording / "frames").glob("*.png"))
first = next(index for index, t in enumerate(times) if t >= LEAD)
last = max(index for index, t in enumerate(times) if t <= END)
lines = []
for index in range(first, last + 1):
    until = times[index + 1] if index + 1 <= last else END
    lines += [f"file '{frames[index]}'", f"duration {until - times[index]:.4f}"]
lines.append(f"file '{frames[last]}'")
(work / "frames.txt").write_text("\n".join(lines) + "\n")
width, height = struct.unpack(">II", frames[first].read_bytes()[16:24])

main = END - LEAD
scale = min((cards.W - 160) / width, (cards.H - cards.BAND) / height)
width, height = round(width * scale / 2) * 2, round(height * scale / 2) * 2
x, y = (cards.W - width) // 2, (cards.H - cards.BAND - height) // 2

bands = []
for index, (start, end, caption, names) in enumerate(SCENES):
    bands.append((start - LEAD, end - LEAD, cards.band(work / f"caption-{index}.png", x, x + width, caption, names)))
for number, (at, key) in enumerate(KEYS):
    _, _, caption, names = next(scene for scene in SCENES if scene[0] <= at < scene[1])
    if CHIPS.get(key, key) in names:
        bands.append((at - LEAD, at - LEAD + LIT, cards.band(work / f"key-{number}.png", x, x + width, caption, names, CHIPS.get(key, key))))

inputs = ["-loop", "1", "-t", str(CARD), "-i", str(cards.title(work / "title.png")), "-f", "concat", "-safe", "0", "-i", str(work / "frames.txt")]
for _, _, png in bands:
    inputs += ["-loop", "1", "-t", f"{main:.3f}", "-i", str(png)]
inputs += ["-loop", "1", "-t", str(CARD), "-i", str(cards.end(work / "end.png"))]
end_input = 2 + len(bands)
graph = [f"color=c={cards.BG}:s={cards.W}x{cards.H}:r={FPS}:d={main:.3f}[bg]", f"[1:v]fps={FPS},scale={width}:{height}:flags=lanczos[term]", f"[bg][term]overlay={x}:{y}:eof_action=repeat[m0]"]
for index, (start, end, _) in enumerate(bands):
    graph.append(f"[m{index}][{index + 2}:v]overlay=0:{cards.H - cards.BAND}:enable='between(t,{start:.3f},{end:.3f})'[m{index + 1}]")
graph += [
    f"[m{len(bands)}]fade=t=in:st=0:d=0.3,fade=t=out:st={main - 0.3:.3f}:d=0.3,setsar=1[main]",
    f"[0:v]fps={FPS},format=rgb24,fade=t=out:st={CARD - FADE}:d={FADE},setsar=1[title]",
    f"[{end_input}:v]fps={FPS},format=rgb24,fade=t=in:st=0:d={FADE},setsar=1[end]",
    "[title][main][end]concat=n=3:v=1:a=0,format=yuv420p[out]",
]
mp4, gif = recording / "pchart.mp4", recording / "pchart.gif"
subprocess.run(["ffmpeg", "-y", "-loglevel", "error", *inputs, "-filter_complex", ";".join(graph), "-map", "[out]",
                "-c:v", "libx264", "-preset", "slow", "-crf", "18", "-tune", "stillimage", "-movflags", "+faststart", str(mp4)], check=True)
palette = work / "palette.png"
subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-i", str(mp4), "-vf", "fps=12,scale=1280:-1:flags=lanczos,palettegen=stats_mode=diff:max_colors=128", str(palette)], check=True)
subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-i", str(mp4), "-i", str(palette), "-lavfi", "fps=12,scale=1280:-1:flags=lanczos[x];[x][1:v]paletteuse=dither=none:diff_mode=rectangle", str(gif)], check=True)
print(mp4, f"{CARD * 2 + main:.1f} s")
print(gif)
