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
- The README shows pChart in use with at most three pictures, one of them the GIF (`docs/pchart.gif`, made with `tools/video`). They come from the sample plan and are taken anew when the screens they show change.
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
- Colour is a signal: red means too late, yellow means look, and milestones carry the theme's accent so they stand out (picture 45). Everything else is neutral ink.
- The preview is the export: the terminal picture, PNG, SVG and PDF come from the same scene.
- UI text is English.

## Code

- Match the surrounding code: naming, density and idiom. Do not add explanatory comments; keep the existing doc comments.
- Read new plan file fields with `#[serde(default)]` so older files still load. A change to `format` needs a migration and a test with an old file.
- Keep heavy work such as painting and encoding pictures off the UI thread, and measure with `pchart bench` when it changes.
- No text may run over the edge of its shape. `Output::crossings` checks pictures in the tests (`crates/pchart-draw/tests/fit.rs`, and the screens in `pchart-tui`); a new picture or dialog goes into those tests.

## Tools

| Command | Use |
|---|---|
| `make`, `make install`, `make package` | build; install to `PREFIX` (`/usr/local`); the Arch package and a pacman repository of its own in `dist/repo/ARCH` from the last commit, signed when `PCHART_SIGN_KEY` names a GPG key; it uploads nothing |
| `pchart help --json` | every command, option, exit code and edit |
| `pchart new PLAN --template ID --target DATE` | a new plan from a template (`new-application`, `website-relaunch`, `product-launch`, `empty`); never writes over a file; `--no-dates` keeps only how it is built, without lengths |
| `pchart add PLAN ID --lane L --name N` | work without `--duration` has no length yet; the plan has no dates until every piece of work has one, and `situation` names what is missing |
| `pchart list` | in a workspace (a folder with a `workspace.toml`), every project with how it stands, and the people to look at: over their capacity or away during their work; `--json` for agents |
| `workspace.toml` | `name`, `lead` (who answers reports), `teams = [{ name, city }]`, `people = [{ name, team, capacity, away = [{ from, to }] }]`; in a workspace `check`, `situation`, `next` and `list` look at people across all projects |
| `pchart report PLAN ID (--done \| --on-track \| --risk \| --later DAYS) [--comment TEXT] [--by NAME]` | in a workspace, say how a piece of work stands; it waits in `reports/` as a file of its own and the plan stays as it is until the lead takes it; `--by` else `$PCHART_NAME` |
| `pchart reports PLAN` | the reports about a plan, newest first, answered or not, with what taking each would do; `--json` |
| `pchart set PLAN ID --who NAME --share 30` | who does a piece of work and the share of their time it takes (1–100, else all of it); the share counts only for how busy they are, never for dates |
| `pchart snapshot PLAN [--name NAME]` | keep how the plan stands today at the end of the plan file (only dates; the plan becomes format 2); `pchart diff PLAN [--since DATE]` says what changed since, counting the plan and not the days that passed; `--json` |
| `pchart situation PLAN --json` | verdict, who waits on whom, what is next, the deciding chain |
| `pchart check PLAN` | problems, warnings and hints; exit code 1 or 2 |
| `pchart set … --dry-run`, `pchart apply PLAN - --dry-run` | try a change and read what it does, without writing |
| `pchart schedule PLAN --json` | calculated dates, what each item has to spare, and the deciding chain |
| `pchart export PLAN --svg F --png F --pdf F` | the roadmap picture as files; `--level overview\|standard\|full`; `--view milestones\|gantt\|phases\|now-next-later` and `--style light` for the 16:9 pages; `--view structure\|network\|matrix` for how the plan is built, without dates |
| `pchart export PLAN --dot F` | the plan as a Graphviz file: lanes as clusters, who waits on whom, the deciding chain in bold |
| `pchart export PLAN --md F` | the plan as Markdown with the milestone chart next to it; `--audience agents` adds the item ids |
| `--audience team\|management\|client` | leave out what the audience does not see: management no warnings, the client also not what single items have to spare |
| `pchart bench PLAN --frames N --out F` | inside a terminal: changes a bar N times and times every step up to the terminal |
| `--today YYYY-MM-DD` | calculate from a fixed day, so results repeat |
| `--holidays FILE` | holiday data instead of the bundled data |
| `tools/holidays/generate.py FIRST LAST OUT [REGION …]` | holiday data from python-holidays |
| `tools/video/record.py OUT`, then `tools/video/compose.py OUT` | the video of pChart in use (picture 48), recorded off screen in a headless cage; writes `OUT/pchart.mp4` and `OUT/pchart.gif` |
| `PCHART_TRACE=FILE` | the TUI writes every wake-up (picture requests, finished pictures, events) to FILE |
