<img src="assets/icon.svg" width="64" height="64" alt="">

# pChart

Project charts in the terminal that check themselves.

Pick a template, set the target date, and pChart schedules the rest backwards and forwards with holidays per team and place. It shows in plain language whether the date holds, who is waiting on whom and what is due next, and exports the same picture as PDF, SVG, PNG and Markdown. Real graphics in kitty, Ghostty and foot, never ASCII.

**Status:** early. The plan is in [docs/PLAN.md](docs/PLAN.md), the approved design in [docs/design](docs/design/). What exists so far: the spike, which schedules a real plan file and draws it as a roadmap in the terminal and as files, and the core with a CLI for people and agents that says whether the date holds, who waits on whom and what is next, checks the plan and changes it (`pchart help`).

![Editor](docs/design/2-editor.png)

## Try the spike

```sh
cargo build --release
./target/release/pchart samples/customer-portal.pchart --today 2026-10-29
./target/release/pchart situation samples/customer-portal.pchart --today 2026-10-29
./target/release/pchart schedule samples/customer-portal.pchart --today 2026-10-29
./target/release/pchart export samples/customer-portal.pchart --today 2026-10-29 --png roadmap.png --pdf roadmap.pdf
```

↑↓ selects an item, ←→ makes it shorter or longer, q quits.

## Terminals

kitty, Ghostty and foot. Terminals without kitty graphics or sixel pictures are not supported.

## License

MIT. Bundled fonts: JetBrains Mono under the SIL Open Font License (`crates/pchart-draw/assets/OFL-JetBrainsMono.txt`). Holiday data comes from [python-holidays](https://github.com/vacanza/holidays) (MIT); see [data/README.md](data/README.md).
