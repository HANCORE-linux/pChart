#!/usr/bin/env python3
"""Runs CMD, the shell pChart is typed into, in a pty inside this terminal and types the keys of the
timeline at their times.
Writes the start time to START and touches DONE at the end. usage: drive.py START DONE CMD..."""
import fcntl, os, pty, select, sys, termios, time, tty
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from timeline import END, KEYS

NAMES = {"up": b"\x1b[A", "down": b"\x1b[B", "right": b"\x1b[C", "left": b"\x1b[D", "enter": b"\r", "esc": b"\x1b", "space": b" "}
start_file, done_file, cmd = sys.argv[1], sys.argv[2], sys.argv[3:]
size = fcntl.ioctl(0, termios.TIOCGWINSZ, b"\0" * 8)
pid, fd = pty.fork()
if pid == 0:
    fcntl.ioctl(0, termios.TIOCSWINSZ, size)
    os.execvp(cmd[0], cmd)
fcntl.ioctl(fd, termios.TIOCSWINSZ, size)
start = time.time()
Path(start_file).write_text(repr(start))
saved = termios.tcgetattr(0)
tty.setraw(0)


def write(target, data):
    """All of `data`: a terminal busy with a large picture takes only part of a write."""
    while data:
        data = data[os.write(target, data):]


keys = list(KEYS)
try:
    while time.time() - start < END + 0.5:
        now = fcntl.ioctl(0, termios.TIOCGWINSZ, b"\0" * 8)
        if now != size:
            size = now
            fcntl.ioctl(fd, termios.TIOCSWINSZ, size)
        ready, _, _ = select.select([0, fd], [], [], 0.01)
        if fd in ready:
            try:
                data = os.read(fd, 65536)
            except OSError:
                break
            if not data:
                break
            write(1, data)
        if 0 in ready:
            write(fd, os.read(0, 4096))
        if keys and time.time() - start >= keys[0][0]:
            os.write(fd, NAMES.get(keys[0][1], keys[0][1].encode()))
            keys.pop(0)
finally:
    termios.tcsetattr(0, termios.TCSADRAIN, saved)
    Path(done_file).touch()
    os.write(fd, b"q")
