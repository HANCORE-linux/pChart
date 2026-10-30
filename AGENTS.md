# AGENTS.md

Rules for AI agents working on pChart. The maintainer decides; when in doubt, ask. The plan and its open decisions are in [docs/PLAN.md](docs/PLAN.md).

## Approval

- Ask the maintainer before every step that records or publishes work: commit, push, pull request, merge, tag and release. One approval covers only the steps it names.
- Never push without the maintainer's consent for that push, never force-push, and never push `main` directly.
- Never create a GitHub repository or add a remote. That is the maintainer's step.
- Design changes need approval before any code is written (see [Design](#design)).
- Decisions the plan marks as open are not made in code. Propose, then record the decision in `docs/PLAN.md`.

## Hard rules

- Commits use the account's GitHub noreply address. Check `git log --format='%ae %ce'` before every push.
- Never commit personal data: no real email addresses, and no screenshots or recordings of a real system. Pictures in `docs/` come from the sample plans.
- No AI attribution in commits: no Co-Authored-By lines for agents, no "Generated with" footers.
- Commit messages have a short imperative subject in English, like the existing history.
- The plan file is the only source of truth. Everything else (SVG, PNG, PDF, Markdown, the index) is output and can be rebuilt from it.
- `pchart-core` never reads the clock or the disk: today, holidays and files are passed in. Dependencies between the crates point downwards only: `pchart` → `pchart-tui` → `pchart-draw` → `pchart-store` → `pchart-core`.
- pChart never moves a fixed date on its own and never changes a plan behind the user's back; it warns.
- Plain Arch Linux keeps working. Omarchy and Hyprland are optional; check that a tool exists before using it.

## Checks

Run what CI runs before asking to commit:

```sh
cargo clippy --locked --release --all-targets -- -D warnings
cargo test --locked --release
```

Verify a change on the real binary as well, in kitty, Ghostty and foot: open `samples/customer-portal.pchart`, and measure with `pchart bench` when drawing or pictures change. Exports: `pchart export samples/customer-portal.pchart --today 2026-10-29 --svg … --png … --pdf …`. Say what you did not check.

## Design

The look of pChart is the maintainer's. **Design changes are not allowed, only proposals.** This covers the layout, colours, text on screen, the charts and the controls.

- The approved design is in [docs/design](docs/design/). Propose a change as before/after pictures. Mockups live in `visualizations/`, which git ignores, never in product code.
- Only a proposal the maintainer has approved goes into the code, exactly as approved.
- The maintainer judges how it looks; passing tests are not that verdict.
- Plain language for people outside project management: "decides the date" instead of "critical path", "to spare" instead of "float", "waits on" instead of "dependency". Menus name tasks, not features.
- Every text on screen is data, a legend or a key hint. No filler. A value that is not known yet stays empty.
- Each fact appears once per screen.
- Colour is a signal only: red means too late, yellow means look. Everything else is neutral ink.
- The preview is the export: the terminal picture, PNG, SVG and PDF come from the same scene.
- UI text is English.

## Code

- Match the surrounding code: naming, density and idiom. Do not add explanatory comments; keep the existing doc comments.
- Read new plan file fields with `#[serde(default)]` so older files still load. A change to `format` needs a migration and a test with an old file.
- Keep heavy work such as painting and encoding pictures off the UI thread, and measure with `pchart bench` when it changes.

## Tools

| Command | Use |
|---|---|
| `pchart help --json` | every command, option, exit code and edit |
| `pchart situation PLAN --json` | verdict, who waits on whom, what is next, the deciding chain |
| `pchart check PLAN` | problems, warnings and hints; exit code 1 or 2 |
| `pchart set … --dry-run`, `pchart apply PLAN - --dry-run` | try a change and read what it does, without writing |
| `pchart schedule PLAN --json` | calculated dates, what each item has to spare, and the deciding chain |
| `pchart export PLAN --svg F --png F --pdf F` | the roadmap picture as files; `--level overview\|standard\|full`, `--view milestones` for the 16:9 page |
| `pchart export PLAN --md F` | the plan as Markdown with the milestone chart next to it; `--audience agents` adds the item ids |
| `pchart bench PLAN --frames N --out F` | inside a terminal: changes a bar N times and times every step up to the terminal |
| `--today YYYY-MM-DD` | calculate from a fixed day, so results repeat |
| `--holidays FILE` | holiday data instead of the bundled data |
| `tools/holidays/generate.py FIRST LAST OUT [REGION …]` | holiday data from python-holidays |
| `PCHART_TRACE=FILE` | the TUI writes every wake-up (picture requests, finished pictures, events) to FILE |
