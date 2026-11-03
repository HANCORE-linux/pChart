"""What the video shows: when each key is typed (seconds after the terminal opens), and the scenes with
their captions and keys (picture 48, made anew as a use case on 2026-11-01: a terminal opens on the
Omarchy desktop, pchart starts in a workspace and opens one of its projects, and the export shows its
pages; on 2026-11-02 pchart read left it, and the people and the reports of the team came in).
THEME after END keeps the video in Labra."""


def typed(at, text, step=0.08):
    return [(at + index * step, "space" if char == " " else char) for index, char in enumerate(text)]


LEAD = 0.6
THEME = 999.0
KEYS = [
    *typed(1.4, "pchart"), (2.3, "enter"),
    (4.4, "down"), (5.3, "up"), (6.2, "enter"),
    (8.2, "down"), (9.3, "up"),
    (11.0, "2"), (12.6, "y"), (14.4, "down"), (15.8, "down"), (17.4, "esc"),
    (19.0, "3"), (21.4, "b"),
    (24.0, "4"), (25.4, "left"), *[(27.4 + index * 0.3, "right") for index in range(6)], (30.6, "right"),
    (33.0, "5"), (35.0, "down"), *[(37.4 + index * 0.35, "right") for index in range(5)],
    (42.4, "6"), (45.0, "enter"),
    (48.6, "q"),
]
END = 50.6
SCENES = [
    (LEAD, 6.6, "Type pchart in a workspace: its projects, and how each one stands.", ["↑↓", "Enter"]),
    (6.6, 10.6, "A plan says where it stands, what is next and which chain decides the date.", ["↑↓"]),
    (10.6, 18.6, "y shows why a date is set: the chain before it, and what it has to spare.", ["2", "y", "↑↓", "Esc"]),
    (18.6, 23.6, "Checks say what to look at, and b adds the buffer they suggest.", ["3", "b"]),
    (23.6, 33.6, "The preview is the file: Gantt chart, network diagram, dependency matrix.", ["4", "←→"]),
    (33.6, 42.0, "People: who is how busy, week by week, across every project.", ["5", "↑↓", "←→"]),
    (42.0, END, "The team reports how their work stands; Enter takes it into the plan.", ["6", "Enter"]),
]
