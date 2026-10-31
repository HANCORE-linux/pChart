# pChart plan

Planning state as of 2026-10-30. The spike is built (branch `spike`); see [Spike results](#spike-results). Items marked **open** are not decided.

## Promise

From a template to a checked, presentation-ready project plan in minutes, in the terminal, in real graphics.

think-cell is fast but does not know whether a plan holds. pChart is as fast and also checks the plan.

Guiding idea for the user experience (maintainer, 2026-10-29): the hardest discipline in project management is translating complex technical dependencies so that everyone sees at once what the situation is, who is waiting on whom, and what is burning next — clear enough for people outside the field and for management.

## First thing you see

Proposal; the look needs the maintainer's approval as pictures first.

Opening a plan does not start with the Gantt chart. It starts with the **situation screen**, which answers three questions in plain language. Every statement is a link: Enter jumps to the item in the chart.

1. **Where do we stand?** One verdict sentence with the deciding number.
   - "Launch on Mon 01 Mar holds. 12 working days to spare, if Requirements start by Mon 16 Nov."
   - What-if example: "Launch on Mon 01 Mar does not hold: 3 working days late. Cause: Frontend +15 wd."
   - Verdict states: holds, tight, does not hold. The thresholds are settings.
2. **Who is waiting on whom?** Handoffs between lanes (teams, people, the client), not between tasks. A Gantt chart hides these; for management they are the point.
   - "Frontend (Bangalore) waits on the client: Design sign-off due Fri 18 Dec."
   - "Test/QA waits on Frontend: Feature freeze due Wed 10 Feb."
   - "The client waits on Test/QA: Beta test due Fri 26 Feb."
   - The handoffs on the deciding chain come first; the rest is folded.
3. **What is burning next?** A short ranked list: the deciding chain first, then the nearest due dates, then open warnings. Only what is really due; an empty list is a good answer.
   - "Requirements must start by Mon 16 Nov: 12 working days from today."
   - "No buffer before the fixed launch: Acceptance Fri 26 Feb, Launch Mon 01 Mar."

Below the three answers, **the chain that decides the date** is drawn as one simple line of steps with owner and date (Requirements → UX → UI → Design sign-off → Frontend → Integration → Feature freeze → Beta → Acceptance → Launch). This is the critical path, but the screen never uses that word.

Plain-language rules:

- No project management jargon on this screen: "critical path" becomes "this chain decides the date", "float" becomes "to spare", "dependency" becomes "waits on".
- Name owners and places, since "who" is what people outside the project ask first.
- Sentences come from the plan model, never typed by hand, so they cannot go stale.
- Holidays appear only where they matter ("the client in New York is off on Mon 15 Feb").
- Since the last snapshot, changes lead: "Since 22 Oct: slack fell from 9 to 4 working days; Frontend +5 wd."

Flow:

- `pchart` with no plan opens the templates; with a plan, the situation screen; one key leads to the chart views.
- **Present mode** steps through the same content one statement per screen for a meeting: situation, deciding chain, who waits on whom, what burns next, roadmap overview.
- **Briefing export:** the situation screen as a one or two page PDF for people who never open pChart.

This ties the existing parts together: the why key draws the deciding chain, "what changed" feeds the lead sentence, the detail levels and export audiences decide how much is shown.

## Navigation and controls

The same rule applies to menus and controls (maintainer, 2026-10-29): someone from outside the field must be able to use pChart without learning it first. Proposal; the look needs approval as pictures.

- **Menus name tasks, not features.** "Change a date", "What if …?", "Why this date?", "Share as PDF" instead of "Toggle constraint" or "Layer: dependencies".
- **Always visible where you are.** The screens sit as tabs in the title bar (1 Situation, 2 Roadmap, 3 Checks, 4 Export); the current mode (for example WHAT-IF) is always shown.
- **Arrows, Enter and Esc are enough** to do everything. Letter keys are shortcuts for people who plan daily, never the only way.
- **The key bar shows only what fits this moment**, at most six hints. Everything else is in the help and the command palette.
- **Same key, same meaning everywhere:** Enter opens or confirms, Esc goes one step back, Space switches on/off, Tab changes the view, `/` searches, `?` explains this screen, Ctrl+P finds any action by typing what you want ("move launch", "pdf").
- **Mouse works too:** click an item or a sentence to jump to it, scroll to move through time. kitty, Ghostty and foot report mouse events.
- **Questions instead of forms.** Changing a date asks "When does Frontend start?" and offers plain choices: "after Design sign-off (calculated)", "on a fixed date", "no earlier than", "no later than". Those are the fixed date types without the jargon.
- **Forgiving input with an echo.** Dates and durations can be typed loosely ("1 mar", "next friday", "W12", "3w", "3–5 weeks"); pChart shows how it understood them ("→ Mon 01 Mar 2027") before anything changes.
- **Nothing breaks.** Every change can be undone; what-if is a sandbox until Enter. Undo replaces "Are you sure?" dialogs.
- **Read-only for viewers.** Opening a plan to look at it (for example `pchart view plan`) cannot change it by accident.
- **First start guides in three steps:** pick a template, set the target date, see the situation. No tutorial pages; the key bar teaches the rest.
- **Help in sentences.** `?` says what this screen answers and what can be done here.

The approved mockups show ten key hints in the bar; under this rule the bar would show fewer and context-dependent ones. That is a change to the approved design and needs approval first.

## Small windows

Approved 2026-10-30 as pictures 15–21 in [docs/design](design/), with the rules below. pChart has to work in any window, from a full screen down to a tiled corner. Measured here (2560×1440, 9 pt font, 7×16 px cells): full screen 364×88 cells, half 182×88, quarter 182×43, eighth 91×43; the classic terminal is 80×24. The approved design is 170×50.

What the spike does today: at 182×43 everything shows but the chain row is cut off; at 120×36 the side panels keep their width and squeeze the chart to 72 columns, week numbers run into each other, phase names shrink to "Re…" and no bar has a name; below 100×30 it shows only "pChart needs at least 100×30 cells".

Rules, in the order things give way (cells are terminal cells):

- **Always:** the title bar, the chart or the statement, the key hints (dropping from the right, `? help` stays).
- **Title bar:** the path needs 135 columns ("saved" stays); below 100 the tabs show their numbers, the current tab's name and the warning count.
- **Tiles** need 120×44; otherwise one summary line: "Launch Mon 01 Mar 2027 · fixed  holds · 12 wd to spare  start by Mon 16 Nov  ▲ 1 warning", shortened from the right.
- **Roadmap side column** (Selection over Checks) needs 150 columns; Checks needs 38 rows of body. Without the column, two rows under the chart name the selected item, its dates and what it has to spare, so a name thinned out of the picture is still readable.
- **Legend row** needs 110×36; **chain row** needs 30 rows and is cut around the selected item. The panel title shortens to fit ("ROADMAP · LANES 1–6 OF 7").
- **In the picture:** the whole plan fits the width; the lane name column narrows from 22 to 14 columns; week numbers thin to every second week, then none; phase names need 4 letters; the place line needs Full and 3 rows per lane.
- **Detail level:** the user picks it; the picture shows at most what fits (Full needs 4 rows per lane, the others 2) and the detail switch shows both. If lanes do not fit at 2 rows, they scroll ("lanes 1–6 of 7") and keep the selection in view; the lane with the fixed target date always stays in view.
- **Situation screen:** the verdict always; "Since" needs 40 rows; below 150 columns the big number goes and only the small scale stays above the sentence that has the number; below 140 columns "What is burning next" and the chain as a vertical list sit side by side with "Who is waiting" underneath, listing only the chain's handoffs; the chain panel needs 30 rows; the next-6-weeks strip needs 100×44.
- **Checks screen:** the roadmap around the chosen check, with what it means and what can be done, needs 150 columns; without it, the chosen check unfolds in the list with one line for each; the holidays get their own panel under the list, otherwise they follow the list as far as there is room (pictures 22, 23, approved 2026-10-31).
- **Export screen:** the preview needs 110 columns; below 30 rows, page, style and holidays give way and the files share one line (pictures 24, 25, approved 2026-10-31).
- **Below 60×16:** rumb's "Terminal size too small" with the current and the needed size, the short side in red.

## Why pChart

Research on 2026-10-29 found no Linux or terminal tool that combines templates, simple handling and presentation-ready output.

| Tool | What it is | Gap |
|---|---|---|
| think-cell, Office Timeline | PowerPoint add-ins, Windows/macOS | No Linux; data locked in .pptx; no plan checks; holidays only as manual shades |
| taskdog | Python/Textual task manager with a Gantt view (MIT, 359 stars, active) | Task manager, text-drawn charts, no templates, no export |
| gantt-cli | Rust/ratatui Gantt TUI (5 stars) | Small hobby project, no templates, no export |
| andrew-ls/gantt, cplt | Print a text Gantt from a file | Not interactive |
| Mermaid, PlantUML, TaskJuggler | Charts from a text syntax | Syntax to learn, no live preview, generic look |
| GanttProject, ProjectLibre, LibrePlan, OpenProject | GUI or web project management suites | Heavy, chart output is secondary |

Each single feature below exists somewhere (MS Project has baselines and driving-path highlighting, LiquidPlanner has ranged estimates; from general knowledge, not verified in this research). The combination exists nowhere, and none of it exists in a terminal.

### Five pillars

Every feature must strengthen one of these. Useful ideas that strengthen none stay out.

1. **Real graphics in the terminal.** Charts are images (like rumb), never ASCII blocks. The preview is the export.
2. **Project templates, not a blank page.** Full plans with phases, lanes, milestones, dependencies and rules.
3. **One plan, many views.** The same file becomes Gantt chart, milestone chart, swimlane roadmap, phase plan or Now/Next/Later with one key.
4. **The plan checks itself.** Overlaps, broken dependencies, missed deadlines, holidays.
5. **The plan has a memory.** Snapshots give planned-vs-actual and the milestone trend analysis automatically.

### Not pChart

| pChart is not | That is |
|---|---|
| A task manager (time tracking, daily planning) | taskdog, Taskwarrior |
| A project management suite (costs, budgets, permissions, team sync) | GanttProject, ProjectLibre, OpenProject |
| A chart language to learn | Mermaid, PlantUML, TaskJuggler |
| An Office add-in or PPTX generator | think-cell, Office Timeline |
| An ASCII chart | gantt-cli, cplt |
| A cloud service with an account | Asana, monday.com |

The plan file is readable text, but nobody has to write it by hand.

## Functions that think with you

v1 has 1, 2, 4 and 5 (decided 2026-10-30); the others come after v1.

1. **What-if.** Move or stretch an item in a trial mode. The old plan stays visible as a ghost; moved milestones and broken deadlines show at once. Apply, discard, or save as variant B and export both.
2. **Why key.** On any date, show the chain that sets it and the slack of each item in it.
3. **Honest dates.** Durations as ranges ("3–5 weeks"). A Monte Carlo run turns them into a date range with a confidence ("between W18 and W21, 85 % by W20"). Only for forecasting; it never reschedules.
4. **What changed.** Compare with the last snapshot: moved milestones with their old position and an arrow. Exportable as a status slide. The milestone trend analysis comes from the same snapshots.
5. **Plan lint with project management knowledge.** Beyond conflicts: no buffer before a fixed date, phase without a closing milestone, block longer than 6 weeks, launch without security review. Templates carry their own rules. `pchart check plan` returns an exit code for CI.
6. **Target date first.** Enter the fixed date; pChart schedules backwards and says plainly when the latest start is already past.
7. **Plan meets reality.** For software plans, link milestones to git tags: once `v0.2` exists, the milestone is reached with its real date. Read-only, like rumb.
8. **Detail per audience.** Export for Team (everything), Management (phases, milestones, ranges) or Client (no internal slack or notes).

Each function sits on one key and stays invisible until needed. The beginner path stays: pick template, adjust, export.

## Charts

**Time charts** share one data model (items with dates in rows or lanes): Gantt chart, milestone chart, timeline, swimlane roadmap, phase plan, release plan, Now/Next/Later, goal roadmap, portfolio roadmap.

**Structure charts:** work breakdown structure, network diagram (PERT/CPM), RACI matrix.

**Control charts:** milestone trend analysis, burndown/burnup, resource load, cost trend.

Proposed scope:

- **v1:** Gantt chart, milestone chart, swimlane roadmap, phase plan, Now/Next/Later; the checks; the "New application" template plus two or three general ones.
- **Later:** milestone trend analysis, network diagram, work breakdown structure (both need automatic box-and-arrow layout), burndown, resource load.

## Scheduling

Per item: a **duration** (fixed, optionally a range), **predecessors**, and optionally a **fixed date**. pChart calculates every date that is not fixed, forwards from a start and backwards from a target. **Fixed dates are never moved**; conflicts are reported.

Fixed date types:

| Type | Example |
|---|---|
| on | Trade fair on 15 May, go-live on 1 Mar |
| no later than | Delivery to the client by end of Q1 |
| no earlier than | Budget from April, hardware arrives 10 Jan |

Behaviour by number of fixed dates:

| Fixed dates | Calculation | Result |
|---|---|---|
| none | relative | Plan in "week 1–n"; one date attaches it to the calendar later |
| start only | forwards | Earliest dates; the end follows |
| target only | backwards | Latest start per item, above all "start no later than …"; warns if that is in the past |
| start and target | both | Slack per item, or how much time is missing |
| one in the middle | split | Backwards before it, forwards after it, checked against start and target |

When there is a conflict, pChart shows the chain that causes it and offers ways out (shorten items on the chain, parallelise, cut scope to Next, move a date). The user decides.

Decided 2026-10-30:

- **Placement of items with slack:** as early as possible from the plan start (the fixed start, or with only a target, the latest start of the deciding chain). The slack is drawn hatched behind the bar, and the selection shows "start no later than …". The rule can be switched per plan.
- **Milestones are points in time** and occupy no day of their own. A calculated milestone is reached when its last predecessor finishes; on a fixed milestone date, its predecessors finish by the working day before. Events that take time, such as an acceptance meeting, are items of one day.
- **The chain that decides the date** is traced backwards from the target: at each item, follow the predecessor that actually sets its start. No slack tolerance is needed, so lanes with different calendars do not blur the chain.

Durations count working days per lane calendar (weekends and holidays).

## Checks

| Check | Example |
|---|---|
| Overlap | Anna is planned on "API" and "Login" in the same week |
| Overload | Backend team at 150 % in W14 |
| Dependency broken | "Frontend integration" starts before "API done" |
| Circular dependency | A waits for B, B waits for A |
| Milestone too early | "Beta" before the work it needs |
| Deadline missed | Plan end after the fixed target |
| Non-working day | Start or end on a weekend or holiday of that lane |
| Overdue | Item before today, not marked done |
| Implausible | End before start, missing data |
| Handover on a holiday | Handover between lanes lands on a holiday of the receiving lane |

Overlap and overload need an optional "who" field; simple plans stay simple without it.

Helpers that are not warnings: highlight the critical path, pull successors along when an item moves, today line.

Principles: pChart warns but never changes the plan on its own (no automatic levelling). Warnings appear only in the editor; exports stay clean.

## Holidays

Requirement: worldwide (Europe, USA, Asia, …), offline.

- **Source:** [vacanza/holidays](https://github.com/vacanza/holidays) (python-holidays), MIT, 250 country codes with subdivisions (federal states, US states, cantons), very active. Handles substitute days, China's swapped working days around Spring Festival, estimated Islamic dates (marked "estimated"), and non-Saturday/Sunday weekends.
- **Delivery:** pChart does not run Python. At each release a script generates a compact data file for a year window (for example 20 years back, 25 ahead) that pChart reads. Each release brings updated data.
- **Calendar per lane, not per plan.** Each lane or person has a location; backward scheduling counts working days per lane.
- **Own days:** company shutdowns, bridge days, personal leave.
- **Override file** for local corrections without waiting for a release.
- **Estimated dates** (moon sighting) are shown as provisional.
- **In the chart:** holidays as shading in their lane only; on/off in exports.

Rejected sources: Nager.Date (offline use needs a paid licence key), date-holidays (data under CC BY-SA 3.0), holidays-rs (data ends 2023), py-holidays-rs (0.1.3 beta). Later addition: [OpenHolidays](https://github.com/openpotato/openholidaysapi.data) (ODbL, 37 countries, mostly Europe) as the only source with **school holidays**.

Measured in the spike (2026-10-30, python-holidays 0.107, 2006–2051): 721 regions, 212,728 lines; 7.4 MB as text, 836 KB with gzip, 439 KB with xz. Generating takes 4 s.

- A subdivision stores only how it differs from its country. 1,930 national holidays are **not** kept by a subdivision (Columbus Day in several US states, some in Malaysia and Australia, Holi in Maharashtra), so the data marks them explicitly.
- python-holidays lists every country twice (DE and DEU); the generator keeps the two-letter codes only.
- India is only available up to 2035.
- **Decided 2026-10-30:** the window is 2006–2051, and the generated file [data/holidays.tsv.gz](../data/) lives in git and is embedded in the binary, so building pChart needs no Python. A command reads only the regions of its lanes; that costs about 20 ms per command, half of it unpacking. `--holidays FILE` reads another file.

## Template: New application

- **Phases:** Requirements → Design → Development → Beta & acceptance → Launch.
- **Lanes:** Design/UX, Frontend, Backend/API, Database, Infrastructure & CI/CD, Test/QA, Security & privacy, Docs & release (the mockup uses a shorter set).
- **Milestones and gates:** requirements fixed, clickable prototype, architecture review, alpha, feature freeze, beta, security review, v1.0.
- **Built-in dependencies:** data model → API → frontend integration; login early; CI/CD before beta; security and privacy review before launch; app store review as its own block for mobile apps.
- **Priorities (must / should / could / won't):** musts form the MVP; the rest goes to Next or Later, which also yields the Now/Next/Later roadmap.
- **Options:** scale in sprints instead of calendar weeks; buffer as a visible block; risks as markers on the timeline.

## From taskdog

| taskdog feature | pChart |
|---|---|
| Dependencies with cycle detection | Take |
| Workload analysis | Take, as the overload check |
| Command palette and fuzzy search | Take |
| Status (open, running, done) | Reduced to what "overdue" and a progress bar need |
| Planned vs actual by time tracking | Replaced by snapshots |
| Nine scheduling algorithms | Reduced to forwards and backwards scheduling, as proposals |
| Markdown notes | Reduced to a remark field, optional in exports |
| REST API server, MCP, SQLite, audit log | Leave out; a readable plan file, git and snapshots cover it; headless `pchart export` and `pchart check` instead |

Lesson from taskdog's design notes: hierarchies made its scheduling unpredictable. pChart groups on one level only (lane or group, then items), purely visually, with no status or duration propagation.

## Building blocks and samples

Proposal, from the maintainer's idea (2026-10-29): nobody can say at the start of a project that they want animation before they know what the application can do. Plans must grow with knowledge, and pChart should suggest the next pieces when they become relevant.

This is an established technique: **rolling wave planning** or progressive elaboration (plan the near future in detail and later phases coarsely, refine as knowledge grows), combined with **decision points** (stage gates) and **spikes** (short, time-boxed research tasks that remove uncertainty before committing, like the pChart spike above).

### How it works

- **Coarse first.** Templates start later phases as placeholders ("Polish: to be detailed") instead of inventing detail nobody knows yet.
- **Decision points** are milestones with a question: "After the prototype: what must the app show?", "After the MVP: what polish is worth it?"
- **Building blocks** are small plan pieces: a few items, their dependencies, rules, a plain-language reason and a sample picture. Example: "Pictures in the terminal" adds a spike for kitty and sixel, theme support, and an optional animation item that waits on both.
- **Suggestions by progress:** when a phase comes near, a decision point is answered, or an item is done, the situation screen shows "Ready to detail: Polish, 3 suggested blocks". Blocks are only suggested, never added on their own, and every suggestion says why.
- Blocks are data like templates (`when` the plan has this state or answer, `adds` these items and rules, `because` this reason). The CLI offers them to agents too (`pchart blocks suggest --json`, `pchart blocks add`).

Example, an app like rumb:

1. Start: "New application" gives Idea → Prototype → MVP → Release, with Polish as a placeholder.
2. At the prototype decision point, the answers "shows live system data" and "runs in a terminal" bring up blocks such as data collection, event stream, terminal UI, and a spike for pictures in the terminal.
3. After the MVP, the polish decision point brings up themes, animation, packaging for Arch/Omarchy, and a man page. Animation appears only now, when it can be judged.

### What pChart knows and what it does not

- **pChart ships project management knowledge**, which stays valid for years: phases, gates, buffers, reviews before launch, typical dependencies.
- **pChart does not ship advice on stacks and architectures.** That knowledge goes stale within months and would turn pChart into a knowledge product that needs constant curation. It comes from three other places instead:
  - **Own blocks:** "Save as block" turns part of a finished plan into a block, with the durations it really took ("pictures in the terminal took 8 wd in rumb"). Plans get more realistic with every project.
  - **Block packs:** shareable sets of blocks as data files (for example a "Rust TUI" pack), kept in a git repository by whoever knows the domain.
  - **Agents:** an AI assistant brings current stack and architecture knowledge and adds blocks through the CLI. pChart provides the mechanism; the agent provides the knowledge of the day.

### Samples

- Every template and chart type comes with a **sample plan** with realistic data, like "Customer portal v1" in the mockups: new application, website relaunch, product launch, trade fair, software release, office move.
- The samples feed the template preview, the documentation pictures, and the golden-image tests, so they are always correct (like rumb's demo data).
- **pChart's own roadmap** becomes the first real sample: pChart is planned in pChart.

### Scope

- **v1:** samples, placeholders, decision points.
- **v1.x:** building blocks and suggestions by progress, with a small set of project management blocks.
- **Later:** save as block, block packs, durations learned from past projects.

## Detail levels

A full chart gets crowded fast. The view must let the user hide detail. The look is approved as pictures 6–8 (2026-10-30):

- **Three levels on one key (`d`):**
  - **Overview:** phases, milestones, fixed dates, today line. Each lane folds into one summary bar. No dependency lines, holiday shading or lane locations.
  - **Standard:** bars with names in each lane, the chain that decides the date stands out, warning markers.
  - **Full:** adds dependency lines, holiday shading, slack, dates on milestones, lane location and calendar. The approved editor mockup is this level.
- **Layers one by one:** dependencies, holidays, slack, dates, phases, locations, warnings, today line.
- **Fold** a single lane or group into one summary bar, and unfold it again.
- **Focus** on the selected item: everything outside its chain is dimmed.
- **Hide side panels** (selection, checks) so the chart gets the full width, like the panel keys in rumb.
- **Thin out automatically:** a name that does not fit its bar moves to the selection panel instead of overlapping; when zoomed out to months or quarters, scales and labels thin out on their own.
- Detail level and layers are stored per view in the plan file. The export audiences (Team, Management, Client) are saved detail levels, so editor and export share one concept.

## Export

- Formats: PDF, SVG, PNG and Markdown. No PPTX (SVG goes into Impress and PowerPoint, PNG anywhere, PDF for sending and print).
- Pick the view and the audience; the preview is the file.
- **Audiences** (approved 2026-10-31): the team sees everything; management sees no warnings; the client sees no warnings and not what single items have to spare. Views without either (milestone chart, phase plan, Now / Next / Later) look the same for everyone. In Markdown the team also gets every item per lane, and the client reads the verdict without its number ("Launch on Mon 01 Mar holds.").
- Labels are placed automatically so nothing overlaps.
- Headless: `pchart export plan --pdf`, `pchart export plan --md ROADMAP.md`, `pchart check plan`.

### Markdown

Proposal (maintainer's request, 2026-10-29). `plan.md` for READMEs, GitHub and GitLab, wikis, tickets, and as context for coding agents.

- **Content follows the audience:** situation (verdict, who waits on whom, what burns next), phases with dates, milestones with fixed or calculated dates, items per lane with duration and predecessors, open checks. The sentences are the same statements as on the situation screen.
- **The chart comes along as a picture:** the export writes the SVG next to the Markdown and links it (`![Roadmap](roadmap.svg)`), so GitHub shows exactly the pChart drawing. No additional Mermaid gantt block (decided 2026-10-30).
- **One direction only.** Markdown is an output, never a second source: the file says "Generated by pChart from portal.pchart, 29 Oct 2026. Edit the plan, not this file." Importing Markdown back is not planned.
- **Stays current by itself:** run `pchart export --md` in a git hook or CI, so `ROADMAP.md` in a repository always matches the plan.
- **For coding agents:** with `--audience agents` the items carry their ids, so an agent that reads what is next can report progress back through the CLI (`pchart set portal.pchart frontend --done`).

Example (sample plan, Management audience):

```markdown
# Customer portal v1

Generated by pChart from customer-portal.pchart, 29 Oct 2026. Edit the plan, not this file.

**Launch on Mon 01 Mar 2027 holds.** 12 working days to spare, if Requirements start by Mon 16 Nov.

![Milestone chart](customer-portal-milestones.svg)

## Who waits on whom

- Frontend (Bangalore) waits on the client: Design sign-off due Fri 18 Dec.
- Test/QA waits on Frontend: Feature freeze due Wed 10 Feb.
- The client waits on Test/QA: Beta test due Fri 26 Feb.

## Milestones

| Milestone | Date | |
|---|---|---|
| Design sign-off | Fri 18 Dec 2026 | calculated |
| Feature freeze | Wed 10 Feb 2027 | calculated |
| Acceptance | Fri 26 Feb 2027 | calculated |
| Launch | Mon 01 Mar 2027 | fixed |
```

## Look and technology

Design baseline: the mockups in [docs/design](design/). The maintainer approved them on 2026-10-29: "if the design looks exactly like this, we build it."

![Templates](design/1-templates.png)
![Editor](design/2-editor.png)
![What-if](design/3-what-if.png)
![Export](design/4-export.png)

The second round, approved on 2026-10-30: the situation screen, the three detail levels, a question asked in the chart, and the palette.

![Situation](design/5-situation.png)
![Overview](design/6-overview.png)
![Standard](design/7-standard.png)
![Full](design/8-full.png)
![Question](design/9-question.png)
![Palette](design/10-palette.png)

The third round, approved on 2026-10-30: the export views without a look of their own so far, and pChart in small windows (see [Small windows](#small-windows)).

![Gantt chart](design/11-gantt.png)
![Phase plan](design/12-phase-plan.png)
![Now / Next / Later](design/13-now-next-later.png)
![Swimlane roadmap, light](design/14-roadmap-light.png)
![Roadmap 182×43](design/15-roadmap-182x43.png)
![Roadmap 120×36](design/16-roadmap-120x36.png)
![Roadmap 91×43](design/17-roadmap-91x43.png)
![Roadmap 80×24](design/18-roadmap-80x24.png)
![Situation 120×36](design/19-situation-120x36.png)
![Situation 80×24](design/20-situation-80x24.png)
![Too small](design/21-too-small.png)

- Graphical, never ASCII.
- The chart area is an image; panels, lists and key hints are terminal text.
- Terminals: **kitty, Ghostty and foot must show everything**. Alacritty is not supported (decided 2026-10-29); it cannot show images.
- Image protocols: kitty graphics in kitty and Ghostty, sixel in foot. Render at the terminal's real pixel size and replace pictures in place in kitty and Ghostty, as rumb does since commit `1224a14`; that is what made Ghostty fast enough for rumb.
- Colour is a signal only (Labra palette, colours from the current Omarchy theme).
- UI language English, and the generated sentences too. Date format `01 Mar 2027` on screen and in exports, ISO `2027-03-01` in the plan file and the JSON; unit `wd` for working days.
- Export style: light background, Noto Sans in the mockup.

The mockups were rendered from `visualizations/plan.py` (a small forward/backward scheduler with per-lane calendars and real holidays from python-holidays 0.107) and `visualizations/render.py`. That folder is local and excluded from git.

## Stack

Decided 2026-10-30 (maintainer's go); versions checked against crates.io on 2026-10-29.

| Layer | Choice | Why |
|---|---|---|
| Language | Rust 2024 | Same as rumb; one binary; the Omarchy/AUR packaging path is known from rumb |
| Terminal UI | ratatui 0.30.2, crossterm 0.29.0 | Proven in rumb |
| Pictures in the terminal | ratatui-image 11.1.0 (kitty protocol for kitty and Ghostty, sixel for foot) | Proven in rumb, including in-place replacement (`1224a14`) and painting off the UI thread |
| Chart drawing | One SVG scene per view (usvg), rasterised with resvg (tiny-skia 0.12, same as rumb) | The same drawing becomes the terminal picture, PNG, SVG and PDF, so "preview = export" holds by construction; usvg also lays out text with real fonts |
| PDF | krilla 0.8.2 with krilla-svg 0.8.1 (Typst ecosystem) | svg2pdf 0.13.0 is stale (last release March 2025, built on usvg 0.45 and tiny-skia 0.11) |
| Fonts | Bundled in the binary (OFL fonts such as Noto Sans, JetBrains Mono) | Exports look the same on every machine |
| Dates | jiff 0.2.38 | Current, ISO week dates, time zones |
| Plan file | TOML via serde (toml 1.1.8) | Readable, diffable, fits git. Decided 2026-10-30 |
| Holidays | python-holidays only in the release tooling; the generated data is embedded | No Python at runtime |

Version catch: krilla-svg 0.8.1 is built on resvg/usvg 0.47, while resvg is at 0.49.0. Pin resvg and usvg to 0.47 so the terminal picture and the PDF share one SVG tree, until krilla-svg catches up.

Structure: see [Architecture](#architecture).

Tests as in rumb: frame tests (cell grid plus pictures), golden SVG files for the drawings, unit and property tests for the scheduler, `clippy -D warnings`.

Considered and not chosen (from general knowledge, not verified in detail): Go with Bubble Tea (no built-in picture support, no reuse of rumb); Python with Textual (fast to prototype and the stack of taskdog, but slower start, harder Arch packaging, no reuse of rumb).

First step before any feature: a **spike** that draws the approved editor mockup from a real plan file through this pipeline in kitty, Ghostty and foot, and exports it as PDF, SVG and PNG. It measures what decides the stack:

1. Time from a key press (move a bar) to the new picture on screen, in each terminal.
2. CPU in Ghostty while moving bars repeatedly.
3. Sixel encoding time in foot for a full-width chart.
4. Whether the PDF from krilla-svg matches the resvg picture (fonts, positions).
5. Size of the embedded holiday data.

### Spike results

Built on 2026-10-30 (branch `spike`): `samples/customer-portal.pchart` is read, scheduled with real holidays per lane and drawn as the swimlane roadmap at the Full level, in the terminal and as SVG, PNG and PDF. Measured on an i7-8700K at 1700×1050 px windows (the terminals' own fonts give 7×16 or 7×17 px cells), 100 changes in a row, each like a key press that makes the selected item one day longer or shorter:

| | kitty | Ghostty | foot |
|---|---|---|---|
| Protocol | kitty graphics | kitty graphics | sixel |
| Chart picture | 1323×768 px | 1323×768 px | 1330×765 px |
| Change to picture, median / p90 | 29 / 36 ms | 25 / 28 ms | 33 / 35 ms |
| of which: text layout and parse, raster, encode | 6, 9, 6 ms | 6, 9, 6 ms | 6, 9, 6 ms |
| of which: writing, then the terminal reading it | 2, 6 ms | 1, 2 ms | 11, 1 ms |
| CPU while changing nonstop (pChart, terminal) | 77 %, 24 % | 90 %, 40 % | 67 %, 43 % |
| CPU idle for 10 s (pChart, terminal) | 0 %, 0.1 % | 0 %, 1 % | 0 %, 0 % |
| Memory, peak | 27 MB | 27 MB | 19 MB |

1. **Speed:** every terminal shows a change within about 30 ms, so the stack holds. Caching shaped labels is not needed yet.
2. **CPU:** a held key keeps one core busy, as expected; idle is zero, since pChart only draws on change.
3. **Sixel in foot:** a fixed palette from the drawing colours, as in rumb, encodes in 6 ms instead of 63 ms for icy_sixel's quantising with dithering. The drawing colours come out exact, anti-aliased edges take the nearest mix, and no dithering is needed.
4. **PDF:** krilla-svg writes the same page as the picture, 1220×756 pt with JetBrains Mono embedded as selectable text. 0.15 % of the pixels differ by more than 10 % after rasterising with poppler, all at glyph edges, in the phase of the hatching and of the dashed line.
5. **Holiday data:** see [Holidays](#holidays).

Cross-check: the Rust scheduler and an independent Python implementation of the same rules agree on all 56 dates and slack values of the sample plan and on the deciding chain.

Not measured: HiDPI (the monitor here has scale 1), other machines, plans larger than the sample, the painting time inside the terminal after it has read the picture.

## Projects, roadmaps and storage

Proposal. Projects are documents, like a spreadsheet or a think-cell slide: **one file per project** (`portal.pchart`), stored wherever the user wants, opened at any time.

- **Why a file and not a database:** taskdog keeps one ever-growing personal task list for all projects, shared by several clients through its server, so SQLite fits there. A pChart plan is bounded (tens to hundreds of items) and gets opened, edited, sent by mail, attached to a ticket and kept in git next to the code. A readable text file does all of that; agents can read and diff it; there is nothing to migrate or lock in.
- **Saving:** every change is saved at once (atomic write), and undo goes back step by step, so nothing gets lost and nobody has to think about saving. What-if stays unsaved until Enter. Decided 2026-10-30.
- **Backups:** the last versions of each plan are kept in `~/.local/state/pchart/`, including changes made by agents through the CLI. `pchart undo portal.pchart` reverts the last change, whoever made it.
- **Opening:** `pchart portal.pchart`; `pchart` alone shows recent plans and templates (the RECENT list in the mockup); a file browser inside pChart finds plans in folders.
- **Where:** the folder pChart was started in, otherwise a default folder that the user picks once.
- **Several projects:** see the workspace below.

### Workspace

Managing projects and roadmaps happens in a **workspace**: a folder that pChart treats as one place. The files stay the truth; the user never sees how they are stored.

```
~/Documents/pchart/                  a workspace (can be a git repository)
  workspace.toml                     people, teams, places and calendars, company days, defaults
  projects/
    customer-portal.pchart
    payment-sdk.pchart
  roadmaps/
    product-2027.pchart-roadmap      which projects, grouped by theme, over which quarters
  archive/
  .pchart/                           cache and backups, never committed
```

What managing means, and how it works without a server:

| Need | How |
|---|---|
| List all projects with their state | Start screen "Projects": verdict (holds, tight, does not hold), target date, what burns next, owner |
| Search, filter, sort | By name, client, state, date range, person |
| New, copy, archive, restore | From a template, from an existing project ("last quarter's roadmap"), move to `archive/` and back |
| Roadmaps over several projects | A roadmap file groups projects by theme; portfolio view and Now/Next/Later across projects, each project as a summary bar with its milestones and state |
| People shared across projects | People and teams live in `workspace.toml`; lanes refer to them. The overlap and overload checks then work **across projects** ("Anna is planned in Portal and SDK in W12") |
| Dependencies between projects | An item can wait on a milestone of another project (`payment-sdk#release`); moving it shows the effect in both |
| Situation across all projects | "3 of 12 projects tight; Anna overbooked in W12" as the first sentence of the portfolio screen |

CLI: `pchart list --json`, `pchart situation --all`, `pchart archive customer-portal`, `pchart new … --roadmap product-2027`.

**Database or not:**

- A database server (PostgreSQL, MySQL) is out: it needs a running service and breaks "offline and simple".
- SQLite is not a server but a single file inside the program. It is the right tool for a **search index**: `.pchart/index.sqlite` holds state, people and dates of all projects for fast lists and cross-project checks. It is rebuilt from the plan files at any time and is never the source of truth.
- Start without the index, since reading a few dozen plan files takes milliseconds. Add it when the projects list gets measurably slow; the user's files do not change when it arrives.
- A real database becomes relevant only with live team editing on a shared server, which is a non-goal.

The file itself (TOML, decided 2026-10-30; the field names are still a proposal):

```toml
format = 1
name = "Customer portal v1"

[[lanes]]
id = "frontend"
name = "Frontend"
place = "IN-KA"          # Bangalore: holidays of Karnataka, India

[[items]]
id = "frontend"
lane = "frontend"
name = "Frontend"
duration = "25wd"
after = ["design-sign-off"]

[[items]]
id = "launch"
lane = "client"
name = "Launch"
milestone = true
on = 2027-03-01          # fixed date
```

## CLI for people and agents

Proposal. Everything that can be done in the TUI can be done on the command line, so scripts and AI agents (Claude Code, Codex and others) can plan with pChart without a screen. Both go through the same action layer.

### Commands

| Purpose | Command |
|---|---|
| Create | `pchart new portal.pchart --template new-application --target 2027-03-01` |
| Situation | `pchart situation portal.pchart` (verdict, who waits on whom, what burns next) |
| Questions | `pchart why portal.pchart launch`, `pchart waits …`, `pchart next …`, `pchart schedule …`, `pchart holidays … --lane frontend` |
| Checks | `pchart check portal.pchart` |
| Change | `pchart set portal.pchart frontend --duration 37`, `pchart set … launch --on 2027-03-01`, `pchart link … api integration`, `pchart add …`, `pchart rename …` |
| Batch | `pchart apply portal.pchart edits.json` (all edits or none) |
| History | `pchart snapshot portal.pchart`, `pchart diff portal.pchart --since 2026-10-22` |
| Export | `pchart export portal.pchart --view milestones --audience management --pdf briefing.pdf` |
| Discover | `pchart help --json`, `pchart schema` |

### Rules that make it work for agents

- **`--json` on every command**, with a versioned schema (`"schema": 1`). Default output is plain-language text for people; `--text` gives the situation sentences ready to paste into a mail.
- **Never asks.** No prompts, no pager, no colour when not attached to a terminal (`NO_COLOR` respected).
- **Dry run on every change:** `--dry-run` returns the new schedule, the changed statements and the checks without writing. This is what-if for agents: try, read the consequences, then apply.
- **Items are addressed by stable ids** (`frontend`, `launch`), not by display names.
- **Clear exit codes:** 0 fine, 1 checks found warnings, 2 a fixed date cannot be held, 3 usage error, 4 file error. Errors go to stderr as JSON with a code, a message and a hint.
- **Safe writes:** atomic (temporary file, then rename) with a lock. An open TUI notices the change and reloads; if it has unsaved edits, it says so instead of overwriting.
- **Self-describing:** `pchart help --json` lists every command, edit type and exit code; `pchart schema` prints the JSON schema of the plan file and of the outputs. A short guide for agents ships with pChart (`pchart guide`), and later possibly a ready-made skill for Claude Code.

Example (numbers from the sample plan):

```json
{
  "schema": 1,
  "verdict": { "state": "holds", "target": "2027-03-01", "slack_wd": 12,
               "latest_start": "2026-11-16", "item": "requirements" },
  "waits": [
    { "who": "frontend", "on": "client", "item": "design-sign-off", "due": "2026-12-18" },
    { "who": "test-qa", "on": "frontend", "item": "feature-freeze", "due": "2027-02-10" }
  ],
  "next": [
    { "kind": "start-by", "item": "requirements", "date": "2026-11-16", "in_wd": 12 },
    { "kind": "check", "rule": "buffer-before-fixed-date", "item": "launch" }
  ]
}
```

## Architecture

Decided 2026-10-30 (maintainer's go).

Goal: modular and extendable without becoming a framework. New views, rules, templates, calendars and export formats must be additions, never rewrites.

### Layers

```
pchart            CLI: view · check · export · (no args) TUI
  │
pchart-tui        screens, actions, keymap, mouse, picture slots (kitty/sixel)
  │
pchart-draw       views → scene → SVG · raster (resvg) · PDF (krilla-svg)
  │
pchart-store      workspace, plan files, locks, backups, recent list, later the SQLite index
  │
pchart-core       model · file format · calendars · scheduler · analysis · statements
```

- Dependencies point downwards only. `pchart-core` knows nothing about terminals, pictures or files on disk; the crate boundaries enforce this at compile time. `pchart-store` is the only part that touches the disk.
- Start with these five crates and strict modules inside them. Split a module into its own crate only when it needs to be used or tested alone.

### Functional core, thin shell

- `pchart-core` is pure: plan in, results out. "Today" and the holiday data are passed in, never read from the clock or the disk, so every result is reproducible in tests (like rumb's demo data).
- Pipeline: **plan → schedule** (earliest and latest dates, slack per item) **→ analysis** (checks, deciding chain, handoffs, what burns next, changes since a snapshot) **→ statements**.
- **Statements are data, not text:** for example `Verdict { holds, slack_wd, latest_start, item }` or `Handoff { from, to, item, due }`. The text layer turns them into sentences per language; the TUI makes every item in them clickable; the briefing export prints them. One source for the situation screen, present mode and the briefing.

### One drawing for everything

- Views do not draw pixels. Each view turns plan, schedule and view settings (detail level, layers, audience) into a **scene**: a small own list of shapes (bars, diamonds, lines, text, shades), each tagged with the item it belongs to.
- The scene is written as SVG once. resvg rasterises that SVG for the terminal and PNG; krilla-svg turns the same SVG into PDF. Preview and export cannot drift apart.
- The item tags make the drawing interactive: a mouse click maps from cell to pixel to shape to item.
- Label placement and automatic thinning out live in the scene layer, shared by all views.

### One action layer

- Everything the user can do is an **action** with an id, a plain-language name ("Change a date"), a short help text and a context (which screen, which mode).
- Keys, mouse, the command palette, the key bar, the `?` help and the CLI all go through the same actions. The key bar shows the actions of the current context; the palette searches their names; the keymap is a data file that maps keys to action ids.
- State changes follow one direction: action → update state → draw. No screen changes data behind another screen's back.

### Edits, undo, what-if, snapshots

- Every change to a plan is an **edit** (move, set duration, set fixed date, add dependency …). Edits make undo and redo trivial and can be recorded.
- What-if is a copy of the plan with its own edit list; Enter applies the edits to the real plan, Esc drops the copy.
- Snapshots are saved copies with a date. Items have **stable ids**, never their names, so diffs, the milestone trend analysis and renamed items keep working.

### Plan file

- Versioned (`format = 1`). New fields are read with defaults, as rumb does with its event stream, so old files keep loading. Each format change gets a migration and a test with an old file.
- **Open:** whether snapshots live inside the plan file or next to it.

### Extension points

| Extension | Mechanism | Added by |
|---|---|---|
| Project templates | Data files (TOML); bundled ones embedded, own ones in `~/.config/pchart/templates` | Users |
| Checks and lint | `Rule` trait in core; common rule kinds (A before B, buffer before a fixed date, maximum duration, required item) usable from templates as data | Code, or users via data |
| Views and chart types | `View` trait: plan, schedule, settings → scene | Code |
| Export formats | `Exporter` trait: scene and statements → file (PDF, SVG, PNG from the scene; Markdown from statements, schedule and an SVG) | Code |
| Calendars | `CalendarSource` trait: holiday data, company days, personal leave, overrides; later school holidays | Code, users via data |
| Actual dates | `Actuals` trait: git tags first | Code |
| Importers | `Importer` trait: CSV and clipboard first | Code |
| Languages | Message catalogs per language (Fluent, `fluent` 0.17) | Users |
| Styles | Data files; terminal colours from the Omarchy theme | Users |
| Keymap | Data file mapping keys to action ids | Users |

- Code extensions are traits plus a registry inside the workspace: a new view is one new module that implements `View` and registers itself.
- Most user extensions are **data, not code**: templates, rule settings, calendars, languages, styles, keymaps.
- Other tools integrate through the CLI and machine-readable output (`pchart check --json`, schedule as JSON), not through plugins.
- **AI assistants:** no MCP server in v1. The plan file is readable text and the CLI is built for agents (see [CLI for people and agents](#cli-for-people-and-agents)), which is all an AI assistant needs to read and edit plans. If a direct integration is wanted later, `pchart mcp` is a thin adapter over the same core and actions (official Rust SDK `rmcp`, 3.8.0 on 2026-10-26), read-only first: situation, why, checks, what-if as a sandbox that never writes.
- **No server and no background service.** TUI and CLI call the core directly inside one binary and work on the plan file. taskdog needs a REST API with WebSocket and API keys because its CLI and TUI are clients of a shared local server and database; pChart gets the same safety from atomic writes, a file lock and the TUI reloading on change. A server only becomes relevant for live team editing or a web UI, which are non-goals.
- **No runtime plugin system in v1** (no dynamic libraries, no WASM). It costs a stable API and opens a security surface. Revisit only when someone needs code that cannot live in the workspace.

### Speed

- Plans have tens to a few hundred items, so a full reschedule on every edit is fine; no incremental engine.
- Painting and encoding pictures run off the UI thread, as in rumb. While a key is held, intermediate pictures are dropped and only the latest is drawn.
- **Idle means idle:** pChart draws only when something changes (key, mouse, resize, the plan file changed on disk). No animation, no polling, no timer except the date change at midnight.

First measurement (2026-10-30), before the spike: the approved editor mockup as one full-screen picture (1700×1050 px, 177 text labels), rendered with resvg 0.47 and encoded the way ratatui-image 11.1.0 does it, one core of an i7-8700K (2017), median of 15 runs:

| Step | 1700×1050 | 3400×2100 (HiDPI) |
|---|---|---|
| Text layout and SVG parse (usvg) | 35 ms | 34 ms |
| Rasterise (resvg) | 18 ms | 38 ms |
| kitty and Ghostty: zlib and base64 | 5 ms, 344 KB | 16 ms, 977 KB |
| foot: sixel with dithering (ratatui-image default) | 63 ms, 382 KB | 254 ms, 804 KB |
| foot: sixel without dithering | 37 ms, 302 KB | 105 ms, 672 KB |
| Peak memory of the whole run | 44 MB | 102 MB without dithering, 150 MB with |

- A change costs about 60 ms of one core in kitty and Ghostty and about 90 ms in foot; on HiDPI about 90 and 180 ms. The spike then measured about 30 ms with the real chart (fewer labels, rumb's fixed sixel palette); see [Spike results](#spike-results).
- Text layout is almost all of the parse time (0.3 ms with the labels removed), so caching shaped labels is the first lever. Not tried yet.
- Sixel without dithering is 1.7 to 2.4 times faster and fits flat chart colours. The picture quality without dithering is not checked yet.
- Upper bound: in pChart only the chart area is a picture, panels are terminal text.
- Not measured: decoding in the terminal (spike point 2), the PDF via krilla, the scheduler, the holiday data, the real binary.
- For comparison, measured on the same machine: rumb uses 7.6 MB and about 10 % of one core all the time in foot (live data and animation); foot itself about 4 %.

## Build order

Proposal with a rough estimate (2026-10-30). The estimate compares sizes and is not measured; after the spike and the core it can be narrowed.

| Step | Content | Needs first | Active days |
|---|---|---|---|
| 0 | Second mockup round: situation screen, detail levels, navigation. Approved 2026-10-30 | — | 1–2 |
| 1 | Spike (see [Stack](#stack)), done 2026-10-30 | — | 2–4 |
| 2 | Core and CLI: model, plan file, calendars and holiday data, scheduler, checks, statements, `pchart check`, `situation`, `schedule` with `--json`. Built 2026-10-30 on branch `core` | — | 5–8 |
| 3 | Drawing and export: scene, the five v1 views, label placement and thinning, detail levels, PDF, SVG, PNG, Markdown. Built 2026-10-30 on branch `draw`: all five views, the roadmap at three levels and in small pictures, label placement, Markdown | Step 0 approved | 6–10 |
| 4 | TUI: templates, situation screen, roadmap editor, what-if, why, checks, export, palette, mouse, date input. Started 2026-10-30 on branch `tui`: the frame, the situation, roadmap, checks and export screens | Step 0 approved | 8–12 |
| 5 | Templates and samples, projects list and workspace, snapshots and "what changed", AUR package, man page, docs, a short video of pChart in use (maintainer's request, 2026-10-30: the real TUI driven by a script and drawn off screen, so no desktop shows; the storyboard is approved as pictures first) | Workspace layout decided | 4–6 |

- **v1:** about 26–42 active days. rumb has about 9,600 lines of Rust today; pChart v1 is estimated at two to three times that.
- **After v1** (everything marked "later"): milestone trend analysis, network diagram and work breakdown structure (automatic box layout), honest dates (Monte Carlo), building-block suggestions and packs, roadmaps and checks across projects, git tags as actual dates, `pchart mcp`, school holidays. Roughly as much again as v1.
- **Largest risks for the estimate:** the look "exactly like the mockups" for any plan, not just the one the mockups were tuned for (label placement without overlaps, thinning out); picture speed with sixel in foot and in Ghostty; the PDF matching the terminal picture. The spike measures the last two.

## Decisions

2026-10-30, maintainer:

- Stack and architecture as proposed; the spike is the first step.
- Plan file in TOML.
- Autosave with undo.
- Milestones are points in time; the deciding chain follows the driving predecessor; items with slack as early as possible (see [Scheduling](#scheduling)).
- Detail levels, situation screen and navigation get a second mockup round before they are drawn.

2026-10-30, maintainer, after the second mockup round:

- Pictures 5–10 are approved, with these choices:
  - Design sign-off falls on the day UI design ends (milestones are points); if the client needs time, it becomes a one-day item.
  - Calculated milestones are drawn at the end of their day, fixed ones in the middle.
  - The hatching shows free slack only (how far a bar can slip before it pushes a successor); what an item has to spare is in the selection panel.
  - Wording: "to spare" instead of "slack", "decides the date" instead of "critical", "Waits on / Then" instead of "Predecessor / Successor".
  - The key bar shows six hints; the why key and Tab move to Ctrl+P.
  - Overview, Standard and Full keep the same frame; only the chart changes.
  - Overview shows dates only on fixed milestones.
  - The situation screen adds a holds / tight / does not hold scale ("tight" below 5 wd), a next-6-weeks strip, "due in N weeks" on each handoff, and owner bands under the chain.
  - The current tab: dark background, pink number and underline; the Checks tab carries the warning count.
- **To spare:** on the deciding chain, what the chain has (the verdict's number); for every other item, counted from where it is planned (the situation mockup: Data model 11 wd).
- **Verdict wording:** "12 working days to spare, with Requirements starting by Mon 16 Nov."
- **A buffer** is an item marked `buffer = true`, not a name.
- **Holiday data** for 2006–2051 lives in git and is embedded (see [Holidays](#holidays)).
- **Date format:** `01 Mar 2027` on screen and in exports, ISO in the plan file and the JSON.
- **Generated sentences** are English only in v1.
- **v1 functions:** what-if, why key, what changed, plan lint (1, 2, 4, 5 in [Functions that think with you](#functions-that-think-with-you)).
- **Building blocks:** decision points in v1, suggestions in v1.x, packs later.
- **Markdown export:** no Mermaid block.

2026-10-30, maintainer, after the third mockup round:

- Pictures 11–14 are approved, with these choices: the Gantt chart groups rows by phase, shows what an item has to spare as a number for items off the chain and the chain's number once in the legend; the phase plan draws phases to scale with dates and length in weeks (working days ÷ 5, holidays not counted) and the milestones as gates between them; Now / Next / Later: Now is running or starting within six weeks, Next the six weeks after, Later the rest, the rule printed once in the column heads; the light roadmap is the Standard level with the places of the lanes, the fixed target in pink, warnings as dark text on a yellow tag, and its legend on the page.
- Pictures 15–21 and the rules under [Small windows](#small-windows) are approved; the lane with the fixed target date stays in view when lanes scroll.

2026-10-31, maintainer, after the fifth mockup round:

- Picture 27 is approved (second proposal; a name under its arrow was turned down): a phase name stays whole inside its arrow. Its text gets smaller as needed, down to 8 pixels; if that is not enough, it breaks into two lines at a space; if that is not enough either, the arrow shows the phase's short name from the plan (`short = "…"` under `[[phases]]`, optional); only without one is the name cut. This is for export pages only: in the terminal, names may still be cut with "…" as in pictures 7 and 8 (the maintainer, after a first version had changed the terminal's phase row too).
- Picture 29 is approved: on export pages long names are never cut but wrap at their spaces. A row of the Gantt table grows for a long item or lane name, a Now / Next / Later card grows for a long name, and when lane and dates do not fit one line, the lane gets its own line and the dates stay whole below it; on the light roadmap a long lane name wraps and the lane grows when it needs to. Only a single word wider than its room is still cut. Pages grow taller where they need to.
- Pictures 22–25 are approved: the Checks screen lists problems, warnings and hints, what was checked and found nothing (and what cannot be checked, such as two items of one person when the plan names no one), and the holidays the plan accounts for; for the chosen check it shows the roadmap around it, what it means and what can be done, answered with the plan as it would be ("A buffer of 5 working days before Launch … The plan then holds with 7 working days to spare"). The Export screen is picture 4 in the frame with tabs, with Markdown as a format and the audiences under [Export](#export); files go to `export/` next to the plan, named after the plan and the view.

### Step 2 as built

What the core and the CLI do now, and choices made while building them that the maintainer should know:

- **Commands:** `situation`, `why`, `waits`, `next`, `check`, `schedule`, `holidays`, and the changes `set`, `add`, `remove`, `link`, `unlink`, `apply` (a JSON list of edits, all or none). Every command has `--json`; every change has `--dry-run`. `pchart help --json` lists commands, options, exit codes and edits.
- **Targets** are fixed milestones (with predecessors) and deadlines (`no_later_than`). A task fixed on a date ("Frontend starts on 1 Dec") is a constraint: if its predecessors are not ready, the check "fixed before predecessor" reports it.
- **Checks:** target too late, deadline missed, fixed before predecessor (problems); no buffer before a fixed date, fixed date on a day off, overdue, two items of the same person at once (warnings); items longer than six weeks, a phase that ends without a milestone (hints). The sample plan gets the warning from the mockups and one hint: its Requirements phase ends with a task.
- **A buffer** is an item with `buffer = true` (`pchart set PLAN ITEM --buffer`, or `"buffer": true` in an edit); a milestone cannot be one.
- **Exit codes 1 and 2 come from `check` only.** A change that works exits 0, even when the plan then does not hold; the output and the JSON say so.
- **Wording of the verdict:** "12 working days to spare, with Requirements starting by Mon 16 Nov." The approved mockup said "if Requirements start by", which breaks for singular names ("if Frontend start"); the new wording is approved.
- **To spare** (`spare_wd` in the JSON): on the deciding chain what the chain has, elsewhere from where the item is planned. In the sample: Data model and API 11 wd, SDK integration 19 wd, Security review 7 wd, as in the situation mockup.
- **Only fixed dates limit the latest dates of the items before them.** An item no fixed date waits on (a user manual after Requirements) is measured against the planned last day instead, so it can show 0 wd to spare without being late. Before this rule, such an item turned "holds, 12 working days to spare" into "tight, 0".
- **Output into a closed pipe** (`pchart schedule PLAN | head`) ends quietly with exit code 0; any other error writing the output exits with 4.
- **Tight** means less than 5 working days to spare (the value assumed in the mockups).
- **Writing the plan file** keeps the fields and their order but not comments a person typed into the file.
- **`why`** says what the chain has to spare once, not per step: per step, the count differs by lane calendar (the client in New York has other holidays) and looks contradictory.

### Step 3 so far

Built 2026-10-30 on branch `draw`, for the views that have approved pictures:

- **Roadmap at three detail levels** (pictures 6–8): `pchart export PLAN --level overview|standard|full`. At the export size of 1220 px, the sample's chart area matches the mockups except for two things: the selection, which only the TUI draws, and the Development phase, which starts on Mon 21 Dec instead of Sat 19 Dec, because a phase now begins with its first item (Design now starts on Mon 30 Nov, as in the mockup; the Markdown no longer lists phases starting on a Saturday).
- **Any plan, not only the sample:** items at the same time in a lane get their own row, and the lane grows by 0.6 of its height per row; two milestones on the same day are rows too. Without `--size`, the export is as tall as the lanes need. A bar's name goes above it, else below it, else ends at the bar's end; a milestone's name goes to its right, else above or below to its left. A name that fits nowhere is left out; the selection panel shows it. Lane names are cut with "…". A test draws the sample and a crowded plan at three sizes and all levels and finds no names covering each other.
- **Milestone chart** (picture 4): `--view milestones`, the 16:9 page in Noto Sans, now bundled (SIL Open Font License); text widths come from the font, so names are placed by their real width. The fixed target is pink; a milestone that is too late would be red.
- **Markdown:** `--md FILE` writes the plan as Markdown and the milestone chart as SVG next to it, following the example under [Markdown](#markdown); `--audience agents` adds every item with its id and how to report progress. The verdict is the sentence of the situation screen, so its headline has no year ("Launch on Mon 01 Mar holds.").
- **Measured again** (`pchart bench`, 100 changes, same windows as the spike): change to picture, median / p90: kitty 26 / 29 ms, Ghostty 24 / 32 ms, foot 33 / 37 ms. Peak memory 31, 31 and 24 MB, about 4 MB more than the spike: the worldwide holiday text while it is read.
- **The other views** (pictures 11–14): `--view gantt`, `--view phases`, `--view now-next-later`, and `--view roadmap --style light`. All five light pages share one frame (`page.rs`): title, view and day, the footer that says where the dates come from, month lines that stop around text, and a page that grows taller when a plan needs more rows or cards. Against the pictures, the pages differ by a few pixels where text widths are measured from the font instead of by Pango. The warning triangle is drawn, since Noto Sans has none. For any plan: long names, lanes and phase names were cut with "…" (on a Now / Next / Later card the lane was cut, never the dates; since pictures 27 and 29 they stay whole), labels on the light roadmap and gates on the phase plan are placed like those of the dark roadmap, a fixed milestone that is not the target is pink too, and items that are done are left out of Now / Next / Later. The phase plan shows only the milestones that close a phase and the target; other milestones have no place in picture 12.
- **Small pictures** (pictures 15–18, the picture side): the lane name column narrows below 1000 units, week numbers thin out, phase names need four letters, Full steps down to Standard below four rows per lane, and below two rows the lanes around the selection are shown together with the lane of the fixed target (`roadmap::shown` tells the TUI which level and lanes it got). In short lanes the bar sits lower so its name fits above, and a name may also stand to the right of its bar or to the left of a milestone at its height. The TUI uses this already; its frame for small windows is step 4.
- **Measured again** after these changes: change to picture, median / p90: kitty 26 / 29 ms, Ghostty 22 / 25 ms, foot 33 / 41 ms.
- **Open, not in any approved picture:** the footer of the milestone chart for a plan without a fixed date ("Dates scheduled forwards from the start of the plan"), the red of a late milestone on the light page, and the Markdown sections beyond the example ("What is burning next", "Phases", the agents' "Items").

### Step 4 so far

Started 2026-10-30 on branch `tui`:

- **Pictures:** a screen now has several pictures, each in its own place with its own kitty image id, painted off the UI thread, as in rumb. Pictures have no ground of their own, so the terminal's background, also a transparent one, shows through; in foot, half transparent edges are mixed with the page colour, since sixel pixels are either drawn or not.
- **Frame** (pictures 7, 8, 15–18, 21): the title bar with the tabs (the current one on a raised ground with a pink number and underline, Checks with its count) and the path, which needs 135 columns; the four tiles with their values as pictures, or the summary line below 120×44; the key bar; rumb's "Terminal size too small" below 60×16.
- **Roadmap screen** (pictures 6–8, 15–18): the detail switch (`d`) highlights the level shown and underlines the one picked when the window shows less; legend symbols are small pictures; the chain row is cut around the selection; the selection and checks panels need 150 columns, otherwise two rows under the chart name the selection; when lanes scroll, the title says so ("ROADMAP · LANES 3–7 OF 7") and the lane of the selection and of the fixed target stay in view.
- **Situation screen** (pictures 5, 19, 20), now the first screen: the headline, the number and the scale as pictures, the sentence with its numbers in bold; who waits on whom on the deciding chain with "in N weeks"; what is burning next with large numerals, the chosen statement on a raised band, and the next six weeks; the chain as a picture with the lanes bracketed under it, or as a list when a step gets less than 14 columns; "everything else has time to spare". From 150×44 it is laid out as picture 5, below that as pictures 19 and 20. `↑↓` chooses a statement, Enter shows its item in the roadmap, `1` and `2` switch screens, Esc goes back to the situation.
- **Not like the pictures:** the "Since 22 Oct" lines wait for snapshots (step 5); "Enter unfolds" under the handoffs and `p present` are not built yet and so not shown. The chain row uses the items' names and cuts them around the selection, since the plan has no short names ("UX", "Sign-off" in the mockups); text larger than a cell (the selected item's name) is bold cell text, only tile values are pictures; the detail level starts at Standard.
- **Checks screen** (pictures 22, 23), tab 3: the checks by weight, the chosen one on a raised band; "Checked, nothing found" lists the rules that ran; the holidays the plan accounts for; the roadmap around the chosen check at the Full level (from five weeks before the week it starts to the Friday of the week after it ends, with every lane that has work then); what it means and what can be done. The words come from `pchart-core` (`text::advice`), so the CLI can use them too. `↑↓` chooses a check, Enter shows its first item in the roadmap.
- **Export screen** (pictures 24, 25), tab 4: view, audience and formats; `↑↓` moves between them, `←→` changes the view or the audience or moves to the next format, Space switches a format on or off, Enter writes the files away from the screen and then says "written 08:45" under them. The preview is the page itself, scaled into the panel over a shadow. The CLI has `--audience team|management|client|agents` for every view now; the default is team, which sees everything, so the pictures 11–14 stay as they were.
- **Not in a picture, chosen while building:** which part `↑↓` is on shows as its heading in brighter ink, the format `←→` is on as a raised ground; the words for the checks the sample does not have (target too late, deadline missed, fixed before its predecessor, fixed on a day off, overdue, two items at once, longer than six weeks) follow those of the pictures; a page taller than 16:9 says its size ("1124 × 780 px · taller than a slide"); the key bar of the Checks screen shows `1 situation` and `q quit` instead of `^P` and `?`, which are not built yet.
- **Names on export pages** (pictures 27 and 29): on the milestone chart, "Requirements" now stands whole in its two-week arrow at 10.4 px, as in picture 4, where the code had stopped at 10.5 px and cut it. Long item, lane and card names wrap. A test checks that no text on the five pages of the sample and of the crowded test plan ends in "…"; the sample's pages are byte for byte as before. The roadmap in the terminal is as in pictures 7 and 8 again.
- **Fixed after recording the storyboard:** on the situation screen the scale's segments are 15, 10 and 15 columns, since "does not hold" filled the 13 of before and ran into "tight" (terminal cells are wider than the mockup's letters); at the Overview level a lane with only milestones stands for them ("Lane · 3 milestones", their dates, "decides the date") instead of "0 items folded" and today's date.
- **Not yet:** on the roadmap the keys Enter, ←→, Ctrl+P and ?, what-if, the date question, the palette, present mode, mouse, autosave with undo, and a key on the Checks screen that adds the buffer (it waits for undo). The key bar shows only keys that work, and `q` quits for now.
- **Measured** at a load of about 3 (change to picture, median / p90): kitty 32 / 40 ms, Ghostty 27 / 29 ms, foot 30 / 31 ms. Transparent pictures cost kitty and Ghostty about 4 ms more to encode than opaque ones (8.5 instead of 4 ms); foot is as fast as in the spike.
- **Sixel and light pages:** the sixel palette is fixed and built from the dark drawing colours, so the white export page came out in the beige of the ink in foot. A light page now gets a palette of its own from the colours of the light pages, their edges, and the page and its shadow over the dark ground; the preview says it shows one.
- **Measured** after the Checks and Export screens (change to picture, median / p90): kitty 29.7 / 30.5 ms, Ghostty 26.6 / 27.8 ms, foot 28.5 / 30.7 ms; peak memory 28, 28 and 23 MB.

## Open decisions

1. Workspace layout and whether roadmaps are their own files.
2. Whether snapshots live inside the plan file or next to it.
