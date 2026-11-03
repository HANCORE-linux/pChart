<img src="assets/icon.svg" width="64" height="64" alt="">

# pChart

Project charts in the terminal that check themselves.

Pick a template, set the target date, and pChart schedules the rest backwards and forwards with holidays per team and place. It shows in plain language whether the date holds, who is waiting on whom and what is due next, and exports the same picture as PDF, SVG, PNG and Markdown. Real graphics in kitty, Ghostty and foot, never ASCII.

**Status:** early. The plan is in [docs/PLAN.md](docs/PLAN.md), the approved design in [docs/design](docs/design/). What exists so far: the spike, which schedules a real plan file and draws it as a roadmap in the terminal and as files, and the core with a CLI for people and agents that says whether the date holds, who waits on whom and what is next, checks the plan and changes it (`pchart help`).

![pChart in use: pchart starts in a workspace, opens a project, why a date is set, the buffer added from the checks, the export, how busy each person is, and a report from the team taken into the plan](docs/pchart.gif)

<p>
  <img src="docs/roadmap.png" width="49%" alt="The swimlane roadmap of the sample plan: phases, lanes per team, milestones and the chain that decides the date">
  <img src="docs/export.png" width="49%" alt="The export screen: the milestone chart as it will be written to PDF and SVG">
</p>

## Try the spike

```sh
cargo build --release
./target/release/pchart samples/customer-portal.pchart --today 2026-10-29
./target/release/pchart new portal.pchart --template new-application --target 2027-03-01
./target/release/pchart situation samples/customer-portal.pchart --today 2026-10-29
./target/release/pchart schedule samples/customer-portal.pchart --today 2026-10-29
./target/release/pchart export samples/customer-portal.pchart --today 2026-10-29 --level standard --png roadmap.png --pdf roadmap.pdf
./target/release/pchart export samples/customer-portal.pchart --today 2026-10-29 --view gantt --pdf gantt.pdf --md ROADMAP.md
```

`pchart` alone starts a new plan: ↑↓ picks a template, the date typed is the target (or with ←→ the start), and Enter writes the plan to the current folder and opens it.

1–6 or Tab switch between the situation, the roadmap, the checks, the export, the people and, in a team, the reports; ↑↓ chooses a statement, an item, a check or a person, Enter shows it in the roadmap, Space shows the details of an item, d switches between overview and full, F5 presents the situation in a meeting, o goes to the projects or another plan, q quits. On the export screen ↑↓ chooses the view, the audience or the formats, ←→ changes them, Space switches a format on or off and Enter writes the files to `export/` next to the plan. Enter on the People screen makes the plan's folder a team.

## Install

```sh
make
sudo make install
```

`make package` builds the Arch package and a pacman repository from it in `dist/repo`.

`man pchart` describes every screen, key, command, edit and the plan file; before installing, `man ./man/pchart.1`.

## Terminals

kitty, Ghostty and foot. Terminals without kitty graphics or sixel pictures are not supported.

## License

MIT. Bundled fonts under the SIL Open Font License: JetBrains Mono (`crates/pchart-draw/assets/OFL-JetBrainsMono.txt`) and Noto Sans (`crates/pchart-draw/assets/OFL-NotoSans.txt`). Holiday data comes from [python-holidays](https://github.com/vacanza/holidays) (MIT); see [data/README.md](data/README.md).
