#!/usr/bin/env python3
"""Records pChart off screen for the video (picture 48): kitty in a headless cage, so nothing appears
on the desktop, the keys of the timeline typed by drive.py, and a picture of the whole output taken as
often as grim manages. pChart reads its Omarchy theme from a folder of its own, written from the
themes pChart ships, which switches from Labra to Harbor at THEME together with the colours of that
kitty, as Omarchy does on a desktop. The sample plan is copied, so the recording changes only the copy.
usage: record.py OUT [WIDTHxHEIGHT] [FONT_SIZE]"""

import json
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

from timeline import THEME

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
PCHART = ROOT / "target/release/pchart"
THEMES = json.loads((ROOT / "crates/pchart-tui/themes.json").read_text())

missing = [tool for tool in ["cage", "kitty", "grim", "wlr-randr"] if not shutil.which(tool)]
if missing:
    sys.exit(f"record.py needs {', '.join(missing)}")
if not PCHART.exists():
    sys.exit("record.py needs pChart: cargo build --release")
out = Path(sys.argv[1]).resolve()
size = sys.argv[2] if len(sys.argv) > 2 else "1600x990"
font = sys.argv[3] if len(sys.argv) > 3 else "12"


def theme(current: Path, name: str):
    colors = THEMES[name]
    (current / "theme/colors.toml").write_text("".join(f'{key} = "{value}"\n' for key, value in colors.items()))
    light = current / "theme/light.mode"
    if colors.get("mode") == "light":
        light.touch()
    else:
        light.unlink(missing_ok=True)
    (current / "theme.name").write_text(f"{name}\n")


shutil.rmtree(out, ignore_errors=True)
(out / "frames").mkdir(parents=True)
state, plan = out / "state", out / "plan"
state.mkdir()
plan.mkdir()
shutil.copy(ROOT / "samples/customer-portal.pchart", plan / "customer-portal.pchart")
home = state / "home"
current = home / ".config/omarchy/current"
(current / "theme").mkdir(parents=True)
theme(current, "labra")
labra = THEMES["labra"]
inner = state / "inner.sh"
inner.write_text(f"""#!/bin/bash
echo "$WAYLAND_DISPLAY" > {state}/display
while [ ! -e {state}/go ]; do sleep 0.05; done
cd {plan}
exec kitty -o background_opacity=1.0 -o font_size={font} -o window_padding_width=0 -o remember_window_size=no -o cursor_blink_interval=0 \\
  -o background={labra["background"]} -o foreground={labra["foreground"]} -o allow_remote_control=socket-only --listen-on unix:{state}/kitty.sock \\
  python3 -I {HERE}/drive.py {state}/start {state}/done env HOME={home} {PCHART} customer-portal.pchart --today 2026-10-29
""")
inner.chmod(0o755)
env = dict(os.environ, WLR_BACKENDS="headless", WLR_LIBINPUT_NO_DEVICES="1", WLR_HEADLESS_OUTPUTS="1")
env.pop("WAYLAND_DISPLAY", None)
env.pop("DISPLAY", None)
cage = subprocess.Popen(["cage", "-d", "--", str(inner)], env=env, stdout=open(state / "cage.log", "w"), stderr=subprocess.STDOUT)
try:
    while not (state / "display").exists():
        time.sleep(0.05)
    display = dict(os.environ, WAYLAND_DISPLAY=(state / "display").read_text().strip())
    width, height = size.split("x")
    subprocess.run(["wlr-randr", "--output", "HEADLESS-1", "--custom-mode", f"{width}x{height}"], env=display, check=True)
    time.sleep(0.3)
    (state / "go").touch()
    while not (state / "start").exists():
        time.sleep(0.01)
    start = float((state / "start").read_text())
    times = []
    switched = False
    while not (state / "done").exists():
        if not switched and time.time() - start >= THEME:
            switched = True
            theme(current, "harbor")
            harbor = THEMES["harbor"]
            subprocess.run(["kitty", "@", "--to", f"unix:{state}/kitty.sock", "set-colors", "--all", f"background={harbor['background']}", f"foreground={harbor['foreground']}"], env=display, check=True)
        index = len(times)
        before = time.time()
        subprocess.run(["grim", "-l", "1", str(out / "frames" / f"{index:05}.png")], env=display, check=True)
        times.append((before + time.time()) / 2 - start)
    (out / "times.txt").write_text("\n".join(f"{t:.4f}" for t in times) + "\n")
    print(f"{len(times)} frames, {len(times) / max(times[-1] - times[0], 0.001):.1f} per second")
finally:
    time.sleep(0.5)
    cage.terminate()
    cage.wait(timeout=5)
