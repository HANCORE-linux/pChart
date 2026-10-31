"""What the video shows: when each key is typed (seconds after pChart starts), when the Omarchy theme
switches to Harbor, and the scenes with their captions and keys, as approved in the storyboard
(picture 48)."""


def typed(at, text, step):
    return [(at + index * step, "space" if char == " " else char) for index, char in enumerate(text)]


LEAD = 1.0
THEME = 36.0
KEYS = [
    (3.0, "down"), (4.6, "up"),
    (7.0, "2"), (8.6, "enter"), (9.6, "down"), (10.2, "down"), *typed(11.0, "4 jan", 0.15), (15.6, "esc"),
    *[(17.0 + index * 0.18, "right") for index in range(12)], (23.6, "esc"),
    (25.0, "n"), (26.0, "down"), (26.4, "down"), (26.8, "down"), (27.3, "left"), (27.6, "left"), (27.9, "left"),
    (28.8, "i"), (29.8, "enter"), *typed(30.4, "Client signs in New York", 0.09), (33.6, "enter"), (34.6, "esc"),
    (41.0, "3"), (43.5, "down"), (45.2, "up"),
    (47.0, "4"), (48.5, "left"), (49.5, "down"), (50.2, "right"), (50.9, "right"), (52.2, "enter"),
]
END = 56.0
SCENES = [
    (LEAD, 7.0, "Opening a plan says where it stands, what is next and which chain decides the date.", ["↑↓"]),
    (7.0, 17.0, "Enter asks when an item starts. Type a date as you say it, and see what follows first.", ["2", "Enter", "↑↓", "Esc"]),
    (17.0, 25.0, "←→ tries an item longer or shorter. Nothing is saved until Enter; Esc goes back.", ["←→", "Esc"]),
    (25.0, 35.0, "n moves a cursor through the roadmap to add notes; i marks a milestone important.", ["n", "←→", "i", "Enter"]),
    (35.0, 41.0, "pChart follows the Omarchy theme: switched to Harbor, it recolours at once.", []),
    (41.0, 47.0, "Checks say what to look at, what happens, and what you can do.", ["3", "↑↓"]),
    (47.0, END, "The preview is the file: pick a view and an audience, and Enter writes it.", ["4", "←→", "↑↓", "Enter"]),
]
