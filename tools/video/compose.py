#!/usr/bin/env python3
"""Puts the video together in the style of the OmaQ video (picture 48, made anew 2026-11-01): pChart's
window on the desktop with the theme's wallpaper and bar, which switch with the theme, the caption of
each scene on the left under it and its keys on the right, the key just pressed lit, the end card, and
music made here, with nothing downloaded. Writes pchart.mp4 (1920x1080, 30 frames per second, H.264,
AAC) and pchart.gif (1280x720, 12 frames per second, no sound) into the recording's folder.
usage: compose.py OUT"""

import shutil
import struct
import subprocess
import sys
from pathlib import Path

import cards
from timeline import END, KEYS, LEAD, SCENES, THEME

FPS, CARD, FADE, LIT, SWITCH, OPEN, POP = 30, 4.0, 0.4, 0.6, 0.6, 0.9, 0.35
CHIPS = {"up": "↑↓", "down": "↑↓", "left": "←→", "right": "←→", "enter": "Enter", "esc": "Esc"}

missing = [tool for tool in ["ffmpeg", "rsvg-convert", "magick"] if not shutil.which(tool)]
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

main, switch = END - LEAD + OPEN, THEME - LEAD + OPEN
x, y = (cards.W - width) // 2, cards.BAR + (cards.H - cards.BAR - cards.BAND - height) // 2
theme = lambda at: "harbor" if at >= THEME else "labra"


def music(out: Path, length: float):
    """A calm loop in A minor at 96 beats a minute: a pad of the chord, a plucked line and a soft beat.
    Am, F, C and G, two bars each."""
    beat, chords = 60 / 96, [(220.0, 261.63, 329.63), (174.61, 220.0, 261.63), (196.0, 261.63, 329.63), (196.0, 246.94, 293.66)]
    pick = lambda part: "+".join(f"eq(c,{index})*{chord[part]}" for index, chord in enumerate(chords))
    eighth, quarter = beat / 2, beat
    expr = (
        f"st(0,floor(mod(t,{8 * beat * 4})/{8 * beat}));st(1,{pick(0).replace('c', 'ld(0)')});st(2,{pick(1).replace('c', 'ld(0)')});st(3,{pick(2).replace('c', 'ld(0)')});"
        f"st(4,mod(floor(t/{eighth}),4));st(5,if(eq(ld(4),0),ld(1),if(eq(ld(4),1),ld(2),if(eq(ld(4),2),ld(3),ld(2)*2))));"
        f"st(6,mod(t,{quarter}));"
        f"0.055*(sin(2*PI*ld(1)*t)+sin(2*PI*ld(2)*1.003*t)+sin(2*PI*ld(3)*0.997*t))"
        f"+0.10*sin(2*PI*ld(5)*2*t)*exp(-7*mod(t,{eighth}))"
        f"+0.13*sin(2*PI*ld(1)/2*t)*exp(-2.5*ld(6))"
        f"+0.22*sin(2*PI*(48+90*exp(-28*ld(6)))*ld(6))*exp(-11*ld(6))"
    )
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-f", "lavfi", "-i", f"aevalsrc='{expr}':s=44100:d={length:.3f}",
                    "-af", f"lowpass=f=4200,aecho=0.6:0.5:120|240:0.25|0.15,afade=t=in:d=1.5,afade=t=out:st={length - 2.5:.3f}:d=2.5,loudnorm=I=-18:TP=-2,aformat=channel_layouts=stereo",
                    "-ar", "44100", str(out)], check=True)
    return out


bands = []
for index, (start, end, caption, names) in enumerate(SCENES):
    bands.append((start - LEAD + OPEN, end - LEAD + OPEN, cards.band(theme(start), work / f"caption-{index}.png", caption, names)))
for number, (at, key) in enumerate(KEYS):
    start, _, caption, names = next(scene for scene in SCENES if scene[0] <= at < scene[1])
    if CHIPS.get(key, key) in names:
        bands.append((at - LEAD + OPEN, at - LEAD + OPEN + LIT, cards.band(theme(at), work / f"key-{number}.png", caption, names, CHIPS.get(key, key))))

still = lambda path, length: ["-loop", "1", "-t", f"{length:.3f}", "-i", str(path)]
inputs = [
    *still(cards.wallpaper("labra", work / "labra.png"), main), *still(cards.wallpaper("harbor", work / "harbor.png"), main),
    *still(cards.bar("labra", work / "bar-labra.png"), main), *still(cards.bar("harbor", work / "bar-harbor.png"), main),
    "-f", "concat", "-safe", "0", "-i", str(work / "frames.txt"),
]
for _, _, png in bands:
    inputs += still(png, main)
end_input = 5 + len(bands)
inputs += still(cards.end(work / "end.png"), CARD)
audio_input = end_input + 1
inputs += ["-i", str(music(work / "music.wav", main + CARD))]

graph = [
    f"[0:v]fps={FPS},format=rgba[w0]", f"[1:v]fps={FPS},format=rgba,fade=t=in:st={switch:.3f}:d={SWITCH}:alpha=1[w1]", "[w0][w1]overlay[d0]",
    f"[d0][2:v]overlay=0:0:enable='lt(t,{switch:.3f})'[d1]", f"[d1][3:v]overlay=0:0:enable='gte(t,{switch:.3f})'[d2]",
    f"[4:v]fps={FPS},pad=iw+4:ih+4:2:2:color={cards.THEMES['labra']['accent']},format=rgba,setpts=PTS+{OPEN}/TB,"
    f"scale=w='trunc((iw*(0.87+0.13*min(1,max(0,(t-{OPEN})/{POP}))))/2)*2':h='trunc((ih*(0.87+0.13*min(1,max(0,(t-{OPEN})/{POP}))))/2)*2':eval=frame,"
    f"fade=t=in:st={OPEN}:d={POP}:alpha=1[term]",
    f"[d2][term]overlay=x='(W-w)/2':y='{y - 2}+({height + 4}-h)/2':eof_action=repeat:enable='gte(t,{OPEN})'[m0]",
]
for index, (start, end, _) in enumerate(bands):
    graph.append(f"[m{index}][{index + 5}:v]overlay=0:{cards.H - cards.BAND}:enable='between(t,{start:.3f},{end:.3f})'[m{index + 1}]")
graph += [
    f"[m{len(bands)}]format=rgb24,fade=t=in:st=0:d=0.4,fade=t=out:st={main - FADE:.3f}:d={FADE},setsar=1[main]",
    f"[{end_input}:v]fps={FPS},format=rgb24,fade=t=in:st=0:d={FADE},setsar=1[end]",
    "[main][end]concat=n=2:v=1:a=0,format=yuv420p[out]",
]
mp4, gif = recording / "pchart.mp4", recording / "pchart.gif"
subprocess.run(["ffmpeg", "-y", "-loglevel", "error", *inputs, "-filter_complex", ";".join(graph), "-map", "[out]", "-map", f"{audio_input}:a",
                "-c:v", "libx264", "-preset", "slow", "-crf", "18", "-c:a", "aac", "-b:a", "160k", "-shortest", "-movflags", "+faststart", str(mp4)], check=True)
palette = work / "palette.png"
subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-i", str(mp4), "-vf", "fps=12,scale=1280:-1:flags=lanczos,palettegen=stats_mode=diff:max_colors=128", str(palette)], check=True)
subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-i", str(mp4), "-i", str(palette), "-lavfi", "fps=12,scale=1280:-1:flags=lanczos[x];[x][1:v]paletteuse=dither=none:diff_mode=rectangle", "-an", str(gif)], check=True)
print(mp4, f"{main + CARD:.1f} s")
print(gif)
