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

- Formats: PDF, SVG, PNG and Markdown, and DOT for Graphviz (picture 58). No PPTX (SVG goes into Impress and PowerPoint, PNG anywhere, PDF for sending and print).
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
- Colour is a signal (Labra palette, colours from the current Omarchy theme): red means too late, yellow means look, and milestones carry the theme's accent.
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

Decided 2026-11-01 (workspace and roadmaps as files, see [Decisions](#decisions)). Projects are documents, like a spreadsheet or a think-cell slide: **one file per project** (`portal.pchart`), stored wherever the user wants, opened at any time.

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
| History | `pchart snapshot portal.pchart --name "Steering committee"`, `pchart diff portal.pchart --since 2026-10-22` |
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
- Snapshots live inside the plan file and hold only dates (decided 2026-11-01).

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
| 4 | TUI: templates, situation screen, roadmap editor, what-if, why, checks, export, palette, mouse, date input. Started 2026-10-30 on branch `tui`: the frame, the situation, roadmap, checks and export screens, the date question, autosave and undo, what-if, notes, the help and the finder, adding, renaming, linking and removing items, the details beside the roadmap, the start with templates | Step 0 approved | 8–12 |
| 5 | Templates and samples, projects list and workspace, snapshots and "what changed", the Omarchy package and builds for other distributions, man page, docs, a short video of pChart in use (maintainer's request, 2026-10-30: the real TUI driven by a script and drawn off screen, so no desktop shows; the storyboard is approved as pictures first) | Workspace layout decided (2026-11-01) | 4–6 |

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

2026-10-31, maintainer, after the calm round (the screens were too full: "so much text that I have to read first to understand where I am"):

- Pictures 32 and 33 are approved ("more minimalist and clearer") and replace the look of pictures 5, 7, 22 and 24. One main thing per screen; whitespace instead of a frame around every part; every number once; details only for what is selected; symbols drawn into the pictures instead of words (a check mark, a clock, a cross, a play mark, a warning sign, a person, a calendar, a folder), not font glyphs, so every terminal shows the same.
- The verdict is a sentence with one symbol before it, without the three-bar scale: "Launch on Mon 01 Mar is on track." (a check mark, neutral ink), "… is at risk." (a clock, yellow, under 5 working days to spare), "… is late." (a cross, red). Under it the number once: "12 working days to spare" or "4 working days late".
- Situation screen (picture 32): the verdict; "Next" with each statement as a symbol, a title and one line (a clock for "in 12 working days", a person for the team, "Acceptance Fri 26 Feb → Launch Mon 01 Mar" for a missing buffer); the chain that decides the date as a picture. Who waits on whom shows in the chain's teams; the six-week strip and the line of everything else with time to spare are gone.
- Roadmap (picture 33): one quiet line with the verdict, the chart over the whole width, and one line about the selection at the bottom (team, dates, duration, whether it decides the date); Enter shows the details. The tiles, the legend (it moves to `?`), the chain row and the side column with selection and checks are gone.
- Checks (picture 33): "1 thing to look at" with a warning sign; every check one line with its symbol; "6 other checks found nothing" and "5 holidays accounted for" as one quiet line each; for the chosen check the roadmap around it, what happens, and what can be done.
- Export (picture 33): the preview as the main thing; views as a list, audience and formats as boxes that are on or off; the folder and the files; page, style and holidays, which cannot be changed, are gone.
- Then, after looking at both (pictures 34 and 35): every screen keeps a frame with a title around each part, so it is clear where one is and what a part shows; the content inside stays calm. Picture 35 shows all four screens so and is the look to build: the situation screen in three frames ("WHERE DO WE STAND", "NEXT", "THIS CHAIN DECIDES THE DATE"); the roadmap with its four tiles as in picture 7 above the framed chart, and the selection line under it; Checks with the checks on the left and the roadmap around the chosen check on the right; Export with the options on the left and the preview on the right.
- Pictures 37 and 38 are approved and replace pictures 9 and 3. The date question (Enter on an item in the roadmap) asks "When does Frontend start?", lists the four answers with only the chosen one explained, echoes a date typed loosely ("4 jan → Mon 04 Jan 2027"), and says what would follow in the manner of the verdict ("Launch would be 9 working days late." with a cross), before Enter changes anything; Tab tries it as what-if (maintainer, 2026-10-31: the picture's `p` is a letter in the date field). What-if keeps the roadmap with the new values in the tiles, the old places dashed, "WHAT-IF · Frontend +12 wd · not saved" in yellow at the top, and one line with the result at the bottom; Enter keeps, Esc discards, ←→ keeps changing. The WHY and OPTIONS columns of picture 3 are gone; what can be done is on the Checks screen.
- Undo is `u` (maintainer, 2026-10-31); Ctrl+Z would stop the program in many terminals.
- Milestones carry the theme's accent, late ones red, with room around them so they stand apart from the bar they end (picture 45, B; maintainer, 2026-10-31): a diamond in the colour of the bar it ends read as an arrowhead. Built in the roadmap (also its parts on the Checks screen and the roadmap export) and in the chain on the situation screen; the pin of a fixed date takes the colour of its diamond. The export pages follow (maintainer, 2026-10-31; picture 45, pages): every milestone in the page's pink with room around it, late ones red; a fixed date has the pin on the light roadmap and the Gantt chart and in their legends; the milestone page and the phase plan keep the target larger and say "fixed".
- Phase names on the light roadmap follow picture 27 too; it had still put a name that did not fit beside its arrow. On export pages a phase name is at least 10 px now instead of 8 (maintainer, 2026-10-31: 8 px read too small); below that the page shows the plan's `short` name, else the first three letters in capitals ("REQ", as the maintainer suggested), or for a name of several words their first letters ("B&A" for "Beta & acceptance"); only then is it cut. On the milestone page "Requirements" still fits whole at 10.4 px; on the light roadmap it becomes "REQ".
- Packages (maintainer, 2026-10-31): pChart is not published on the AUR but as an Omarchy package through omarchy-pkgs, as rumb is. Arch Linux, Fedora, Debian and Ubuntu are supported (maintainer, 2026-10-31): a release binary that does not need the newest glibc (the one built on Arch needs glibc 2.43 and does not start on Debian 13, Ubuntu 24.04 or Fedora 43), and a build from source with `make` as in rumb. Debian, Ubuntu and Fedora wait until after the first release; it is for Arch Linux and Omarchy first (maintainer, 2026-11-03). pChart goes into Omarchy's package repository later (maintainer, 2026-11-03: https://github.com/omacom/omarchy-pkgs), so that on Omarchy `sudo pacman -S pchart` is all it takes. Its rules, read on 2026-11-03: one recipe in `pkgbuilds/pchart/` with a `PKGBUILD` to Arch standards and `.omarchy/package.json`; the source pinned to a release tag of pChart's public repository with its checksum (`updpkgsums`); every architecture in `arch=()` is built by their CI, so aarch64 must build too; a `github` watch on the release tags (`v(?P<version>…)`) brings new versions without another pull request; one package per pull request, built after a maintainer labels it, and it then reaches `edge`, `rc` and `stable`. Until then the README builds from the source; a package installed from a URL needs a signature (`SigLevel = Required`), a downloaded file does not.
- The first packages come from a pacman repository of the maintainer's own (maintainer, 2026-11-02: "own repo first, not on the AUR"), not yet through omarchy-pkgs. `make package` builds the Arch package from the tarball of the last commit (`make dist`), runs every test in `check()`, and puts it with `pchart.db` into `dist/repo/x86_64`, a folder any web server can serve as `[pchart]`. It signs package and database when `PCHART_SIGN_KEY` names a GPG key, else both stay unsigned. Where the repository is served, whether it is signed and the first version are the maintainer's steps; pChart creates no repository, uploads nothing and adds no remote. `make && sudo make install` builds and installs from source on any distribution with cargo.
- Themes (pictures 44, maintainer, 2026-10-31): pChart takes the colours of the current Omarchy theme and follows it while it changes (checked once a second, as rumb does), and draws in Labra without Omarchy. `pchart PLAN --theme NAME` takes one of the 49 themes rumb ships instead and paints its page, so it also reads in a terminal of another colour; there is no key for themes in the TUI. A theme's palette keeps the contrast each role has in Labra against the page: neutrals are mixed between the theme's background and text, and its accent, yellow and red move towards the text until they read; Labra itself stays as drawn. On a light theme yellow therefore turns brown (#805c4a in Harbor). The export pages keep their own colours in every theme. Pictures 44 in `docs/design` are the real Harbor screens, with the milestones of picture 45.
- The brackets of the selection sit at the ends of its bar, above and below it, instead of 4 px beyond them (picture 46, maintainer, 2026-10-31): beside a bar that starts the next day, as Integration after Frontend, the right bracket lay on that bar.
- Important milestones and notes (pictures 39–43, 2026-10-31): a mode "Add notes" with a cursor that moves through the whole roadmap by day, week and lane; a note on an item moves with it, a note at an empty place stays at its date and lane. An important milestone is drawn as a star instead of the diamond (picture 39, A; maintainer, 2026-10-31), since a diamond with a star beside it would be mistaken for the usual milestone symbols. The mode is called "Add notes" and starts with `n` on the roadmap; `i` sets the star in it. A note at an empty place shows its symbol and the start of its text in the roadmap (maintainer, 2026-10-31). Pictures 39 (A), 40, 41 and 43 are approved (maintainer, 2026-10-31): the exports for Team and Management show notes under their milestones and the client's do not, and an important milestone is never left out, also not at the Overview level and on the pages for management and the client. The roadmap's key bar has `n add notes`; `i` works in the notes mode.
- The help and the finder (pictures 50, maintainer, 2026-10-31): `?` opens the help of the screen over it, dimmed as in pictures 10 and 40: what the screen answers in one sentence, its keys each with a sentence, and its symbols as the screen draws them (on the roadmap the legend that picture 33 took off the screen); in a small window the symbols give way to the keys. Ctrl+P opens the finder of picture 10, now in the look of the calm screens: the chosen line on the raised band with the pink edge of picture 36, the words that matched no longer in pink, which the colour rule keeps for milestones, and only actions that exist, each with what it would do now and its key; picture 10's "What if …? p", "Why this date? w" and "Share as PDF e" are not built. It finds actions by the words people use ("move", "pdf", "holidays") and the items they work on by name, whole words before the starts of words; with nothing typed it lists the actions of the screen. Enter runs the chosen line as its key would: "move launch" asks when Launch is, "holidays" opens the Checks screen on the holidays, "gantt pdf" opens the Export screen with the Gantt chart and PDF on. The finder keeps one size while typing. In the key bars `? help` takes the place of `1 situation` and `q quit`, as picture 35 had it, and stays when a narrow window drops hints from the right; the Export screen gets `? help` at the end.
- Adding, renaming and linking items (pictures 51, maintainer, 2026-10-31): dialogs over the dimmed roadmap like the date question, each saying what would follow before Enter changes the plan. `a` adds an item after the selected one: "What needs doing?" asks its name, work or a milestone, its length typed loosely ("5", "2w", "3 weeks", echoed as working days), its lane (the selected item's to begin with) and where it comes: "after Frontend, before Integration" (what waited on Frontend then waits on the new item), "after Frontend, nothing waits on it", or "before Frontend" (it waits on what Frontend waited on, and Frontend on it); `↑↓` goes from question to question and `←→` changes a choice. Its id is made from its name and never shown. `r` renames ("What is Frontend called?"); `w` asks "What does SDK integration wait on?" with a box for each item, filled when it waits on it, Space switching it; the items that wait on it already are left out, which would make a circle, and one sentence says so. Each dialog is one change that `u` takes back at once. The keys work on the roadmap at the Standard and Full levels; the key bar stays as it is, and the keys are in the help and the finder. Removing an item follows in pictures 53.
- Removing an item (pictures 53, maintainer, 2026-10-31): `x` or Delete on the roadmap asks "Remove Frontend?" in the same dialog, with what changes and what would follow; Enter removes, `u` brings it back, Esc keeps it. What waited on it then waits on what it waited on ("Integration waits on Design sign-off instead of Frontend."), so the chain stays whole; a phase that ended with it ends with the item before it that ends last ("The Development phase ends with Integration instead."); its note goes with it. When a phase ends with it and nothing comes before it, it cannot be removed, and the dialog says so in plain ink. After removing, the selection is what waited on it, or else what it waited on. The plan file gets a new edit for this, `{"op": "phase", "phase": NAME, "until": ID}`, which `pchart apply` takes too. The dialog is kept although undo replaces "Are you sure?": it is no yes-or-no question but shows what removing does to the chain before it happens (maintainer, 2026-10-31).
- The details beside the roadmap (pictures 52, maintainer, 2026-10-31): Space shows or hides a column on the right, as pictures 2 and 15 had it before the calm round took it away. It says only what the line under the chart does not: what the selection waits on and what waits on it, its earliest and latest dates, whether its date is calculated or fixed, the holidays in its time and its note; under it the checks with their symbols. It is off when pChart starts, needs 150 columns (below that pChart says so), and Space is in the help and the finder, not in the key bar.
- The start (pictures 54, maintainer, 2026-11-01): `pchart` without a plan, in a terminal, opens the start, the calm form of picture 1. On the left the templates and under them the plans opened last, the chosen one on the raised band of picture 36 with its lanes, items and milestones, or with where it is. On the right the preview of the template with real dates, and under it "HOW TO START": the name (the template's when none is typed), a target date or a start date (`←→`), typed loosely as in the date question, what follows ("Requirements must start by Thu 12 Nov 2026.", "Launch would be 31 working days late.", or from a start date "Launch on Fri 11 Dec 2026.") and the file that is written. Enter writes the plan to the folder pChart was started in, never over a file (a number is added then), and opens it on the situation screen. A plan opened before shows how it stands, where it is and when it changed, and Enter opens it. A target date goes on the template's last milestone that nothing waits on; without a date the plan starts today. Against picture 1: "No dates (week 1–16)" and the empty charts are gone, since the charts are views of one plan, and "Empty plan" takes their place; the CONTAINS frame is gone, since the chosen line and the preview show it; places per lane are not asked yet, so a new plan counts weekends only until a lane gets a place. The templates: New application (the sample's structure with "Requirements fixed" and a buffer before Launch), Website relaunch, Product launch and Empty plan; each checks clean. Below 30 rows the preview gives way.
- No text runs over the edge of its shape (pictures 55, maintainer, 2026-11-01, after "Launch" ran into the tip of its phase arrow in the start preview: "we need logic so that this never happens anywhere"). A name in a phase arrow is centred between the notch and the slant of the tip at the height of its letters, with air to both, on the roadmap and on the export pages; where the room is short it gets smaller, then shortened, as before. On the milestone page the smallest size is 9 px instead of 10 px, so "Requirements" stays whole (9.3 px) without touching the tip, as picture 27 wanted it whole; in a shorter arrow it becomes "REQ", then "R…", and when not one letter is left the arrow stays empty instead of showing "…" alone, which the Website relaunch template did. Names keep 2 units of air around a diamond and keep off the heads of the "waits on" arrows; the item count of a folded lane gives way to a milestone; a week number that does not fit at the right edge is left out. Against the approved pictures of the sample only the phase rows change.
- Why this date? (pictures 56, maintainer, 2026-11-01): `y` on the roadmap at the Standard and Full levels shows the chain of items that sets the date of the selection. The chain stands out with an arrow from each step to the next, also at the Standard level; everything else, also a warning that belongs to another item, steps back to a third. The frame says "WHY THIS DATE · SWIMLANE ROADMAP …", and the line under the chart says it in one sentence: "Launch · on Mon 01 Mar · after Requirements → … → Beta test → Acceptance · decides the date · 12 wd to spare", a long chain keeping its first and last two steps. What the line says is to spare is the selection's own, as on the line about the selection. `↑↓` chooses another item and its chain follows; `Esc` or `y` goes back. `w` of picture 10 is "waits on" since pictures 51, so the key is `y`; as decided for the key bar, it is in the help and the finder ("Why this date?"), not in the key bar. `pchart why` says the same: what the item has to spare, then when the first step can start.
- How a project is built, without dates (pictures 57, maintainer, 2026-11-01: "if someone wants to know how their project is built, that is time-independent at first"). Two new views, as light pages on the Export screen and with `pchart export --view structure|network`: the project structure (work breakdown, DIN 69901), with the phases as the first level numbered 1, 2, 3 and under each the work and milestones that end in it (1.1, 1.2 …) with their lane and length, items after the last phase going with it and a plan without phases broken down by lane; and the network diagram (DIN 69900), every item as a box below what it waits on, a column for each lane with its team and city on top, the chain that decides the date in ink, milestones as pink diamonds that arrows enter from the left so the pin of a fixed date stays free, and a legend. Neither page has a date or the foot that says where dates come from. The network runs from top to bottom, since the sample's chain has eleven steps and boxes side by side on a 16:9 page would be too narrow for the names; the page grows taller instead. Reading the structure of a folder (an Omarchy theme, a code project) was open decision 3 and is decided with pictures 62.
- The dependency matrix and the Graphviz file (pictures 58, maintainer, 2026-11-01, after asking for the charts known from industry: DSM, UML, C4, SysML, Graphviz). A third view without dates, "Dependency matrix" on the Export screen and `pchart export --view matrix`: the items as rows and as columns in the same order, grouped by phase as in the project structure, each phase a tinted block on the diagonal, and a dot where the row waits on the column, in ink on the chain that decides the date and grey where there is time to spare. A plan has no circles, so every dot lies below the diagonal, and one far from it waits across phases. And `pchart export PLAN --dot FILE` writes the plan for Graphviz: a cluster for each lane with its team and city, a box for each piece of work with its length, a pink diamond for each milestone, an arrow for each item it waits on and the deciding chain in bold. The layout of that picture is Graphviz's, not pChart's. UML, C4 and SysML describe software and systems, not a plan, and are not pChart's.
- Structure without time (pictures 59, maintainer, 2026-11-01: "if someone wants to know how their project is built, that is time-independent at first"). A piece of work may leave out its length; a value not known yet stays empty. Dates come only once every piece of work has a length: until then the plan is only put in order, nothing decides a date and nothing has time to spare, and a known length shows ("10 wd") while an unknown one stays empty. The start offers "No dates" next to "Target date" and "Start date" (←→ target, start or none): the date field goes, one line says "Dates come once every piece of work has a length.", the preview shows the template's project structure, and the plan is written without lengths and without fixed dates. A plan without dates opens on the Export screen with the network diagram, which lists only the three views without dates and no Markdown; the situation, the roadmap and the checks show one panel, "NO DATES YET", with how many pieces of work have no length and which, and the key bar says "4 how it is built". On the command line `pchart new … --no-dates` writes such a plan and `pchart add` takes work without `--duration`; `situation`, `next` and `check` say what is missing, and what needs dates (the dated views, Markdown, `schedule`, `why`) refuses with exit code 3 and says why. Editing the structure on a screen of its own is a later proposal.
- Editing in the network diagram (pictures 60, maintainer, 2026-11-01). A plan without dates opens on screen 2, named "Network" until every piece of work has a length, instead of on the Export screen. It shows the network diagram as wide as the window with the selection between the corner marks of the roadmap, scrolled only as far as the selection and the row after it need; above on the right "11 of 11 without a length", and under it a line about the selection: its team, its length or "no length yet", what comes before it and what after. ↑↓←→ move to the nearest item that way, Enter asks "How long does Frontend take?" and says how many others still have none, and a, w, r and x add, link, rename and remove with the dialogs of the roadmap, over the page shaded dark. When adding, "Takes" may stay empty ("not known yet: it stays empty"), and the line of what would follow, which needs dates, is left out. Screens 1 and 3 say "2 how it is built". When the last length is given, screen 2 becomes the roadmap with its dates and the key bar says "Every piece of work has a length now". The help names the keys of the network diagram, and the finder offers adding, renaming, linking and removing.
- The buffer key (pictures 61, maintainer, 2026-11-01). On the Checks screen, while a missing buffer is chosen, the key bar says "b add the buffer", and `b` adds the buffer that "what you can do" works out: 5 working days in the lane of the item before the fixed date, between it and the date. It is one change, so `u` takes it back; the key bar then says "Buffer added · u takes it back".
- Reading how a folder is built (pictures 62, maintainer, 2026-11-01, deciding what was open decision 3, raised 2026-11-01 with an Omarchy theme as the example). pChart reads a folder itself: `pchart read DIR` writes a plan without dates next to where it runs (`NAME.pchart`, never over a file) that opens in the network diagram of picture 60. It knows Rust (each file under a crate's `src` is an item, each crate a lane, and `use crate::…` or `other_crate::…` waits on that file), Hyprland (`source =`, also from `~/`), Lua (`require`) and CSS (`@import` of files in the folder). A use that would close a circle is left out and named, since a plan cannot wait on itself. The lanes come in the order they build on each other, so the dependency matrix has every dot below the diagonal. It reads one project or one config folder and not the system: not `/`, the system folders or a whole home, at most 1,000 files it can read and 20,000 entries while it counts, and it does not follow links. While it counts and reads, a terminal shows a spinner with the files read and the time left, after 150 ms, so that a quick read shows nothing; real projects took 15 to 55 ms. Theme folders show few waits, since Omarchy reads their files and they rarely take in each other. What stacks pChart knows still goes stale, as the knowledge of stacks does (see [What pChart knows and what it does not](#what-pchart-knows-and-what-it-does-not)). **Parked** on 2026-11-02 (maintainer: "auslagern, später daran arbeiten"): `pchart read` and the pictures 62 live on branch `read`, the other branches no longer have it.
- The workspace (maintainer, 2026-11-01, deciding open decision 1). A folder with a `workspace.toml` is a workspace: its teams, people, places and company days hold for every project in it, its projects live in `projects/`, finished ones in `archive/`, and a roadmap over several projects is a file of its own in `roadmaps/` (`product-2027.pchart-roadmap`), so it can be sent on its own. A workspace is optional: a single plan anywhere keeps working alone, as today. The start shows the workspace's projects with the verdict of each, and the checks across projects ("Anna is planned in Portal and SDK in W12") work only inside a workspace, where people are shared.
- Snapshots (maintainer, 2026-11-01, deciding open decision 2). Snapshots live inside the plan file, at its end, and hold only dates: for each snapshot its day, what the chain had to spare and the planned finish of every item. The history goes wherever the plan goes (mail, ticket, git), and a snapshot takes about as many short lines as the plan has items. To be checked when built: an older pChart must not drop the snapshots when it saves a plan, so a plan with snapshots may need the next `format` and a migration.
- Snapshots and what changed (pictures 67, maintainer, 2026-11-02). `t` on the situation or the roadmap, or `pchart snapshot PLAN [--name NAME]`, keeps how the plan stands today, with a name if one is typed ("Steering committee"); only by hand, never on its own, and `u` takes it back. A snapshot holds the planned first and last day of every item, not only the last as decided on 2026-11-01, since lengths and the dashed old bars need both; besides them what the chain had to spare and the latest start of its first step. A plan with snapshots is format 2, so that an older pChart refuses it instead of dropping them on save; a plan without stays format 1. What changed is measured against the last snapshot up to today and counts only what changed in the plan, not the days that passed: "3 working days less to spare: Requirements must now start by Mon 16 Nov instead of Thu 19 Nov", then work that got longer or shorter ("Frontend +3 wd (22 → 25)"), new and removed items and moved milestones. The situation shows it under the verdict as "Since Kick-off · Thu 22 Oct" in at most three lines (picture 5), `c` shows the roadmap with the old dates dashed, "SINCE KICK-OFF · THU 22 OCT" in its title and "was Thu 24 Dec – Wed 27 Jan" on the line about the selection, and `pchart diff PLAN [--since DATE]` says the same. Not in v1: the milestone trend analysis, a status page to export, and choosing an older snapshot on screen.
- Two things the maintainer missed in the first local test of the package (pictures 68, 2026-11-02: "I could not choose what kind of project to start", "and in the export I could not choose what I want"). On the Export screen `↑↓` walks the views one by one, as the list shows them, and only after the last goes on to "For" and "Format"; a pink bar at the left edge marks the row it is on, `←→` changes the audience or the format, and the key bar says what the keys do there. `o` leaves an open plan, which is saved already, for the start of picture 54, to begin a new plan from a template or open one from before; "o new or other plan" is in the key bar of the situation, and the finder has "New plan or another". Esc on the start ends pChart, as before. In the same test (maintainer: "I pressed d as the key bar said, but Detail stayed grey"): the grey word "Detail" before the detail switch read as a choice that was switched off, so the switch shows only Overview, Standard and Full (maintainer, 2026-11-03: "leave the text Detail out up there"; a `d` in its place and "Space details" in the key bar read as two kinds of detail and were taken back); when the window is too low for the level picked, the key bar says "Full needs a taller window" instead of only underlining it; and `Tab` and `Shift+Tab` go to the next screen of the title bar and the one before, as people try on tabs.
- Adding to a new plan (pictures 69, maintainer, 2026-11-03: "when I start a new plan I am completely lost, I do not know how to add milestones"). A plan with no work in it yet opens on the roadmap, as a plan without dates does; "a add work or a milestone" is always in the key bar of the roadmap and is the last hint to give way in a narrow window; and `a` on the milestone a plan ends with, which nothing waits on, puts the new item before it, with "ADD BEFORE TARGET" over the dialog, which follows the place chosen there.
- The details beside the roadmap and the version (pictures 71, maintainer, 2026-11-03: "I do not see the Space key for the details at the bottom, and the details window is longer than the roadmap on the left"). The column of picture 52 ends where the roadmap ends, above the line of the selection; "Space details" is in the key bar of the roadmap after all, and `d` says what it switches, "d overview/full", so no word stands for two things; in a narrow window `d` gives way first. The title bar shows the whole version, "pchart 0.1.3", on the start and in a plan. The first version is 0.1.3 (maintainer, 2026-11-03), and every test package gets a version of its own. Releases stay at 0.x (maintainer, 2026-11-03): what this plan calls v1 is the scope of the first public release, not its number; 1.0 comes only when the maintainer says so. The first release is 0.2.0 (maintainer, 2026-11-03), tagged `v0.2.0`, with the Arch package attached to its GitHub release.
- People and lanes in a new plan (pictures 72, maintainer, 2026-11-03: "can I not add team members or people?"). `r` renames the selected item and its lane, "Name" and "Lane" with `↑↓` between them; a renamed lane keeps its id, its work and its place. In `a`, `←→` on "Lane" goes past the last lane to "New lane" with a field for its name ("its name, such as Design"); the new lane comes at the bottom, without a place and so without holidays. In `p`, "+ Someone new" stands under "No one named" and typing goes there at once; in a workspace the new person also joins `people` in `workspace.toml`, in the team named as the lane, and the rest of that file stays as it is written. `u` takes back the plan's change but not the line in `workspace.toml`. Agents have the same as edits: `add_lane` and `rename_lane`. Deleting a lane, setting its place in pChart and changing people on the People screen are not part of it.
- A team without writing a file (pictures 73, maintainer, 2026-11-03: "can I create a team in pChart without writing a file first, so I get the overview at once?", and "why can I choose the project? only the person"). Screen 5 "People" is always there; outside a workspace it shows the people the plan names, each in the team of their lane, under "CAPACITY · ONLY THIS PLAN", and the key bar says "Enter make this folder a team". Enter writes `workspace.toml` next to the plan with those people and never over a file; the folder's name is the team's, the overview across its projects is there at once, "6 Reports" appears, and the key bar says "acme is a team now". The projects of a workspace are now the plans next to its `workspace.toml` as well as those in `projects/`. `↑↓` chooses only people: a team is a heading with its sum for each week and is no longer chosen, so the work under the table and the line at the bottom are always one person's (this replaces choosing a team from pictures 64). With no one named the table says "No one named yet · p on a piece of work names someone". Where a terminal draws the underline of the current tab is its own setting (foot draws it a pixel higher than kitty and Ghostty); pChart leaves it as it is (maintainer, 2026-11-03).
- Present mode (pictures 74, maintainer, 2026-11-03: present mode is in the first release, the mouse comes after it). F5 on any screen, or "Present in a meeting" in the finder, shows the situation one statement a screen for a meeting, in the order of [First thing you see](#first-thing-you-see): where we stand, the chain that decides the date, who waits on whom, what is next, and the roadmap at the Overview level. The title bar gives way to the plan's name and today on the left and "2 / 5" on the right, and the key bar says "←→ slide" and "Esc back to pchart"; the arrows, Space and Page Up/Down turn the slides, Esc or F5 goes back where it was. Each slide is drawn as on the situation screen, up to twice as large as the window lets it be, and no larger than its width allows, so a long deciding chain stays the size it has on the situation screen (A; breaking it into two rows was B and was not chosen). "Who waits on whom" shows the handoffs between lanes on the screen for the first time: those on the deciding chain first, as large statements ("Design/UX (Berlin) waits on Product (Berlin)", "Requirements due Fri 27 Nov"), then "and 7 more". The Situation key bar says "F5 present".
- Key hints in a narrow window (pictures 75, maintainer, 2026-11-03: "depending on how big the terminal window is, I do not see the shortcuts at the bottom, such as Space for the details"). When the hints do not fit, the key bar first uses their short words ("Enter date", "a add", "p who", "n notes", "o other plan", "Esc back"), so the roadmap's eleven fit in 136 columns instead of 177; when they still do not fit, hints give way from the right as before and the last one says "? all keys" instead of "? help", so it is clear that there are more. A second row and keeping Space longest were offered and not chosen.
- Details in present mode and the way back to the projects (pictures 76, maintainer, 2026-11-03: "in present mode it should be possible to show details" and "how do I get back to all my project overviews?"). In present mode `↑↓` chooses an entry of the slide: the target on the first, a step of the chain (on a raised ground around its bar with a pink line under its date, so the ground never runs under a neighbour's name), a handoff or a step of "Next" (on a raised band with a pink edge, as on the situation screen), or a piece of work on the roadmap; Space shows its details on the right, the column of picture 52, and hides them again; they need 150 columns, as beside the roadmap. `←→` and Page Up/Down turn the slides, so Space and `↑↓` no longer do. `o` stands in the key bar of every screen, "o projects" in a workspace and "o new or other plan" outside one, just before "? help", so it is among the first to give way in a narrow window.
- What changed, on the first slide (pictures 77, maintainer, 2026-11-03, after the test of present mode with a snapshot). When the plan keeps a snapshot, the first slide shows under the verdict the same lines as the situation screen: "Since Kick-off · Thu 22 Oct", the first change in bold, at most three, then "and N more · c shows them on the roadmap"; they grow with the verdict only as far as the longest of them fits the width, so they are never cut. Without a snapshot the slide stays as it was.
- Phases that overlap (pictures 79, maintainer, 2026-11-03: "why does it overlap at Requirements", choosing A, then "should there not be 1 px of room between them"). A phase runs from the first item after the phase before it to the item that ends it; when that item comes before the end of the phase before, as when a piece of work is fixed too early, the phase starts with the first of its own items instead (those the item that ends it waits on and no earlier phase does), so it is never empty. Phases that then overlap are drawn in a row of their own under the first, on the roadmap, the milestone chart, the phase plan and the light roadmap, only when they overlap; between the rows at least 1 pixel stays free, set in pixels and not in units, so that small fonts keep it. A plan whose phases follow each other looks as before.
- Quitting with a what-if (pictures 78, maintainer, 2026-11-03: "when I press Esc and have not saved changes, should it not ask whether to save before closing?"). Every change is saved at once, and Esc never closes pChart; only a what-if that is not kept is not saved ("WHAT-IF · Frontend +3 wd · not saved"). `q` or Ctrl+C then ask "QUIT · WHAT-IF NOT KEPT", "Keep Frontend +3 wd before you quit?" with what it does ("With it: Launch on Mon 01 Mar is on track, 9 working days to spare."): Enter keeps it and quits, Esc discards it and quits, `←→` goes back to the what-if. Also from present mode, which F5 opens in the middle of a what-if. Esc in a what-if still discards it at once, as its key bar says (chosen over asking there too).
- A calmer situation screen was tried (pictures 80, maintainer, 2026-11-03, after a dashboard on X: one sentence, one row of figures, "First on the list", "Look at", "If it slips", no frames and no capital titles) and not taken: the situation stays as it is (maintainer, 2026-11-03: "leave it as it is").
- The projects of a workspace (pictures 63, maintainer, 2026-11-01). `pchart` started in a workspace, or in a folder inside one, opens the start of picture 54 with the workspace's name in its title ("START · ACME"): first "PROJECTS", every plan in `projects/` with how it stands as a symbol in the signal colours (a check holds, a yellow clock is at risk, a red cross is late, a folder has no verdict yet, such as a plan without dates), with the preview and the stand of the chosen one on the right as for a plan opened before; then "NEW PROJECT FROM" with the templates, and a new project is written to `projects/`. Outside a workspace the start stays as it is. `pchart list` says the same on the command line, with `--json` for agents.
- Teams, people and capacity (pictures 64, maintainer, 2026-11-02: "add teams and people", then "a project lead says employee X has two tasks at 30 % each, so 40 % are still free"). `workspace.toml` names the teams with their city and the people with their team, how much of their time they can work (`capacity = 80`, else 100 %) and when they are away (`away = [{ from = 2027-01-04, to = 2027-01-08 }]`). A piece of work names who does it (`who`, as before) and the share of their time it takes (`share = 30`, else 100 %); the share counts only for how busy someone is and never changes a length or a date (maintainer, 2026-11-02: "only the load"). A person belongs to the team their team name matches, as a lane name does. On the roadmap `p` asks "Who does Frontend?": "It takes 40 % of their time" with `←→` in steps of 10, then the people of the lane's team and the others, each with what they have free then ("40 % free then"), "would be at 140 % of 100 %" or "away Mon 04 Jan – Fri 08 Jan", and "No one named"; the line under the chart then names the person, the share and the city. Two checks look across the projects of the workspace: someone planned at more than their capacity on working days ("Lena Vogel is planned at 130 %", once for each stretch, with the other work, and someone of their team with the time for the whole piece of work: "Give UX concept to Mia Klein, who has the time then"), and someone away during their work ("Priya Nair is away during Frontend", "Plan 5 working days more for Frontend, or give it to someone else"). They replace the plan's own check of two items at once for the people the workspace names; work that together takes no more than 100 % no longer counts as two items at once. They show on the Checks screen with the person's weeks beside them, under "Next", on the start beside the chosen project, and in `pchart check`, `situation`, `next` and `list`. Screen 5 "People", only in a workspace, is the capacity table: a row for each team and under it its people, a column for each week from this one, and in each cell how full the week is on its busiest day, in percent of what they can do, darker when fuller, yellow over 100, hatched where they are away; a team counts its people together against their capacities. `↑↓` choose a team or a person and `←→` a week; under the table the work of the chosen one in the same weeks, and the line at the bottom says "130 % full · 130 % of 100 % planned, 30 % too much" and the work of that week.
- Reports from the team (pictures 65, maintainer, 2026-11-02: "how can others report whether their task is done, still on track, at risk, or needs a new date with its effect on the whole plan", then "we should be able to change a report before taking it with Enter"). Someone says how a piece of work stands: done, on track, at risk, or needs N more working days, with a comment. A report is a small file of its own in the workspace's `reports/` folder (`2026-10-29-0840-priya-nair-frontend.toml`), so no two people write the same file, and the plan does not change until the project lead takes it. Reports exist only in a workspace. The folder reaches the others however the team shares the workspace, such as Syncthing, Git or Nextcloud; pChart has no network code and no service (maintainer, 2026-11-02, instead of building OmaQ's Tox and Signal technology into pChart). On the roadmap `s` asks "How is Frontend going?" with Done, On track, At risk and Needs more time, the days more, a comment and who reports, and says what taking it would do. Screen 6 "Reports", in a workspace beside 5 "People", lists the reports to answer, newest first, with the answered ones under them; for the chosen one it shows what taking it does (for more time the roadmap around it with the earlier places dashed, and "Frontend then takes 28 working days instead of 25 … Launch then holds, with 9 working days to spare"). Enter takes it as one change that `u` takes back, which also opens the report again; `e` opens it in the report dialog ("What do you take from Priya Nair?") to change it first, and the file then says what was taken (`taken = { status = "later", days = 2 }`); `x` declines it. Taking more time makes the work longer, done marks it done, and a risk or "on track" keeps the comment as the item's note. The tab says "6 Reports ●2"; while pChart is open it looks for new reports every 3 seconds and names a new one in the key bar ("Priya Nair reports: Frontend needs 3 more working days"); reports to answer stand under "Next" and on the start ("2 reports to answer"); `pchart report`, `pchart reports` and `pchart list` do the same on the command line. `lead = "Name"` in `workspace.toml` names who answers; who uses pChart on a computer is `$PCHART_NAME` or the first line of `~/.config/pchart/name`. Without a lead everyone may answer. When a message in the key bar does not fit beside the key hints, hints give way from the right until it does, on every screen (maintainer, 2026-11-02).
- No new features until v1 (maintainer, 2026-11-02, after pChart had grown to 16,400 lines in five days: "wait for what others report back"). Until v1 only what the build order still lists is built: snapshots and "what changed", the Omarchy package and builds for other distributions, the man page and the docs, the mouse and present mode, and fixes. The mouse waits until after the first public release; present mode is in it (maintainer, 2026-11-03). So do the builds for other distributions: the first release is for Arch Linux and Omarchy (maintainer, 2026-11-03). Everything else waits for what people who use pChart report: reading a folder (pictures 62, on branch `read`), the diagram of its parts (pictures 66, a draft), the archive, roadmaps over several projects and answering reports on the command line.
- Themes and the video wait until the calm screens are built. Animations are proposed (only to show a change, such as a bar gliding to its new place and the number counting to its new value; no blinking; can be switched off) and not decided.

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
- **`why`** says what the chain has to spare once, not per step: per step, the count differs by lane calendar (the client in New York has other holidays) and looks contradictory. Since pictures 56 that is what the item itself has to spare ("SDK integration has 19 working days to spare."), not what the first step of its chain has, which for an item off the deciding chain was the deciding chain's number; `--json` has it as `spare_wd`.

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
- **Not like the pictures:** the "Since 22 Oct" lines came with snapshots (pictures 67), under the verdict and from 30 rows; "Enter unfolds" under the handoffs and `p present` are not built yet and so not shown. The chain row uses the items' names and cuts them around the selection, since the plan has no short names ("UX", "Sign-off" in the mockups); text larger than a cell (the selected item's name) is bold cell text, only tile values are pictures; the detail level starts at Standard.
- **Checks screen** (pictures 22, 23), tab 3: the checks by weight, the chosen one on a raised band; "Checked, nothing found" lists the rules that ran; the holidays the plan accounts for; the roadmap around the chosen check at the Full level (from five weeks before the week it starts to the Friday of the week after it ends, with every lane that has work then); what it means and what can be done. The words come from `pchart-core` (`text::advice`), so the CLI can use them too. `↑↓` chooses a check, Enter shows its first item in the roadmap.
- **Export screen** (pictures 24, 25), tab 4: view, audience and formats; `↑↓` moves between them, `←→` changes the view or the audience or moves to the next format, Space switches a format on or off, Enter writes the files away from the screen and then says "written 08:45" under them. The preview is the page itself, scaled into the panel over a shadow. The CLI has `--audience team|management|client|agents` for every view now; the default is team, which sees everything, so the pictures 11–14 stay as they were.
- **Not in a picture, chosen while building:** which part `↑↓` is on shows as its heading in brighter ink, the format `←→` is on as a raised ground; the words for the checks the sample does not have (target too late, deadline missed, fixed before its predecessor, fixed on a day off, overdue, two items at once, longer than six weeks) follow those of the pictures; a page taller than 16:9 says its size ("1124 × 780 px · taller than a slide").
- **Calm situation screen** (pictures 32 and 35), now built: the verdict, "Next" and the chain are pictures with drawn symbols, each in a frame with its title. The chosen statement lies on a raised band with a pink edge, as the chosen row of a list (picture 36, B, chosen by the maintainer over a thin pink line and over dimming the others); the key bar shows `↑↓ choose`, `Enter show in roadmap` and `? help`. In a smaller window the verdict's text gets smaller to fit; below 30 rows the verdict takes six rows instead of eight; the chain shows only when every step gets 14 columns and there are rows for it, so at 80 × 24 it is left out. The verdict words are new everywhere the sentence is used: on the screen, in `pchart situation` and in Markdown ("Launch on Mon 01 Mar is on track."); the state in the JSON stays `holds`, `tight` or `does-not-hold`, since programs read it.
- **Calm roadmap** (picture 35), now built: the four tiles as before (or the summary line in a small window), the chart in its frame over the whole width, and under it one line about the selection as a picture with drawn symbols: its name, a person and its team, a calendar and its dates, a clock and its length, and a short bar with "decides the date", what it has to spare, or how late it is; at the Overview level its lane, what it folds and its span. The side column with selection and checks, the legend and the chain row are gone; the key bar keeps the keys that work.
- **Calm Checks screen** (picture 35), now built: in the left frame "1 thing to look at" with a warning sign (a cross when a problem is among them, a check mark and "Nothing to look at" when there is nothing), every check as one line with its symbol (a cross, a warning sign, an information mark for a hint), the chosen one on the raised band with the line that says where it is, and two quiet lines, "6 other checks found nothing" and "5 holidays accounted for"; in the right frame the roadmap around the chosen check and three lines: what happens, the step to take after a play mark, and what the plan is then. Below 150 columns the right frame goes and the three lines follow the list. `↑↓` also chooses the two quiet lines (picture 49, maintainer, 2026-10-31): the right frame, or below 150 columns the room under the list, then shows the checks that found nothing, each with a check mark and the one that did not run in grey, or the holidays the plan accounts for as a table, as in picture 22; Enter does nothing there.
- **Calm Export screen** (picture 35), now built: the left frame is one picture with the views as a list (the chosen one filled like a box that is on, as the maintainer asked, so the frame speaks one language: filled means chosen or on), the audience and the formats as boxes that are filled when on, the sentence what the audience hides or shows (wrapped when long), a folder with the files, and a check mark with "written 13:19" after Enter. The label of the part `↑↓` is on is in brighter ink, and `←→` rings a format in pink (neither is in the picture; both show where the keys act). Page, style and holidays are gone. Below 110 columns the preview goes and the picture says so; below 30 rows the files share one line.
- **Names on export pages** (pictures 27 and 29): on the milestone chart, "Requirements" now stands whole in its two-week arrow at 10.4 px, as in picture 4, where the code had stopped at 10.5 px and cut it. Long item, lane and card names wrap. A test checks that no text on the five pages of the sample and of the crowded test plan ends in "…"; the sample's pages are byte for byte as before. The roadmap in the terminal is as in pictures 7 and 8 again.
- **Fixed after recording the storyboard:** on the situation screen the scale's segments are 15, 10 and 15 columns, since "does not hold" filled the 13 of before and ran into "tight" (terminal cells are wider than the mockup's letters); at the Overview level a lane with only milestones stands for them ("Lane · 3 milestones", their dates, "decides the date") instead of "0 items folded" and today's date.
- **Date question, autosave and undo** (picture 37), now built: Enter on an item in the roadmap at the Standard and Full levels opens the question in the chart's frame over the dimmed roadmap, and the title bar says "CHANGE A DATE · nothing changed yet" in yellow. The answer the item has now says "now" and is chosen when the question opens, with its date filled in; `↑↓` chooses another. A date can be typed loosely: "2027-01-04", "4 jan", "jan 4", "04 Jan 2027", "friday", "next friday", "W12", "3w", "10d", "today", "tomorrow"; a date without a year is the next one from today on (`pchart_core::input::date`, so the CLI can use it too). What would follow is calculated while typing: the verdict for the target, and the last item that moves with its new date. Enter makes the change; the plan file is written at once, away from the UI thread, through `pchart-store` (atomic and locked), and when changes come faster than the disk the newest plan wins. `u` goes back one change at a time and writes again ("Undone: Frontend start"); Esc closes the question without a change. When pChart quits, it waits until the last change is written and says so if it could not be.
- **Chosen while building the date question:** the picture explains only "No earlier than"; the other answers say "calculated, moves when Design sign-off moves", "never moves; pChart warns if the plan cannot hold it" and "must be done by this date; a deadline", since the scheduler limits the finish with `no_later_than`. At 80 × 24 the dialog has no room for what would follow, so that part is left out there.
- **What-if** (picture 38), now built: `←→` on an item in the roadmap at the Standard and Full levels tries it a working day longer or shorter, and Tab in the date question tries the date typed. Nothing is written while trying, also not when pChart quits. The title bar says "WHAT-IF · Frontend +12 wd · not saved" in yellow ("2 changes" when `↑↓` chose another item and `←→` changed it too); the tiles show the tried plan, the target with its verdict word, the latest start in yellow with "today" when it is today and in red when it is past, and "was 12" under what is to spare; the frame says "WHAT-IF · SWIMLANE ROADMAP", and the places items had before are dashed. The line under the chart has the item and how it changed ("25 → 37 wd", or "Mon 21 Dec → Mon 04 Jan" for a date) and what follows ("Launch is at risk · 0 wd to spare · Requirements must start today"; when the target is late only "Launch is 9 wd late", since the start may be held by a fixed date). Enter keeps all changes as one step that `u` takes back ("Undone: Frontend +12 wd"), Esc goes back to the plan as it was.
- **Chosen while building what-if:** picture 35 has `←→ move` in the roadmap's key bar and picture 38 `←→` for the length; the key bar says `←→ length`, as in picture 38. While trying, `u`, the screen keys and Enter for the date question do nothing; `q` quits and drops what was tried. `pchart bench` now times the held arrow key as what-if, with the dashed places.
- **Measured** with what-if, in a headless cage at a load of about 5 (software rendering, so not comparable with the numbers above), against the commit before (change to picture, median / p90): kitty 25.5 / 29.1 ms (before 25.3 / 27.8), Ghostty 31.4 / 33.3 ms (30.9 / 32.6), foot 37.6 / 49.3 and 42.5 / 64.7 ms in two runs (36.9 / 42.6 and 38.9 / 51.4); the dashed places cost foot about 1 ms more to raster.
- **Notes and important milestones** (pictures 39–43), now built: `n` on the roadmap at the Standard and Full levels starts "Add notes". The cursor goes by day (`←→`), week (Shift `←→`) and lane (`↑↓`); its lane lies on a raised band and its day is a dashed line, an empty day is bracketed, and on an item the item is chosen. Enter writes the note where the cursor stands, starting from the one there (picture 40); Enter keeps it, an empty note removes it, Esc keeps it as it was; `i` marks the milestone under the cursor important or no longer; each change is one step for `u`. An important milestone is a star in the roadmap, in the chain of the situation screen and on every export page, whose foot then says "★ important"; its name is placed right after those of fixed dates. Notes show only in the notes mode (picture 47; maintainer, 2026-10-31, for now): there a note stands as plain text under or beside its item, and a note at a place as a stroke at its day with the start of its text, both shortened to fit; the roadmap otherwise shows only the stars. The line under the chart ends with the note of the chosen item. The milestone page shows notes under their milestones, or above when the name stands above, for Team and Management and not for the client. `pchart set --note`, `--important` and `--not-important`, and the edits `note`, `important` and `note_at`, do the same from the command line.
- **Chosen while building the notes:** the line about a selected milestone follows picture 39: the star, its name, its date, whether it decides the date, and the note; it no longer names the lane and its city, which the chart shows. A selected milestone gets the selection brackets, as in pictures 39 and 43. Notes at a place are not on the export pages, and notes do not go into Markdown yet; no picture shows them there. The cursor stays within the days the roadmap shows; at the Overview level `n` does nothing, as Enter. `u` works in the notes mode too.
- **The video** (storyboard picture 48, maintainer, 2026-10-31): about a minute of the real pChart, recorded off screen by `tools/video` from a copy of the sample plan, with a theme folder of its own that switches from Labra to Harbor while it runs. The README shows it as `docs/pchart.gif` (1280 × 720, 12 frames per second, 3.7 MB), next to two screens of the sample plan, the roadmap and the export (maintainer, 2026-11-01: at most three pictures, one of them the GIF); recorded anew on 2026-11-01 with the phase arrows of picture 55; the MP4 (1920 × 1080) is not in git, since it can be made again. The theme switch shows for about half a second a light terminal under pChart's old colours, as on a desktop, since pChart looks at the theme once a second.
- **The video made anew** (maintainer, 2026-11-01: "it should look like a real use case", in the style of the OmaQ video on X). About 55 seconds in Labra: the Omarchy desktop with the theme's wallpaper and a bar, a terminal that opens on it as Hyprland opens a window, a plain prompt `~ ❯` where `pchart` is typed, the start, the situation, why a date is set, the buffer added with `b`, the export with Gantt chart, network diagram and dependency matrix, then `q` and an end card. Until 2026-11-02 it went on with `pchart read ~/projects/pchart` and pChart's own code in the network diagram and the matrix. Since then (maintainer, 2026-11-02: "can you add the teams to the video") `pchart` starts in the sample workspace Acme (`samples/acme`, the one of pictures 63 to 65: its teams, people, four projects and two reports) and opens Customer portal v1 from its projects; after the export come People, with Lena Vogel's week at 150 %, and Reports, where Enter takes Tom Becker's "Data model at risk" into the plan, as the lead Jonas Weber (`PCHART_NAME`); then `q` and the end card. The copies are dated 28 Oct 2026, the day before the video's today, so the start does not say they changed later. The window is drawn as Labra decorates it in Hyprland: a border of 1 px from pink at the bottom to olive at the top, corners rounded by 4 px, 94 % opacity so the wallpaper shows faintly through, a soft shadow, and the terminal's padding of 14 px; the wallpaper is Labra's 02 (maintainer, 2026-11-01). The caption of a scene is on the left under the window and its keys on the right; the MP4 has music made by `compose.py` with ffmpeg, nothing downloaded, and the GIF none. The wallpapers are the Labra and Harbor ones, MIT, in `tools/video/wallpapers`. In two of nine recordings kitty showed the large preview of the dependency matrix as a white strip, while pChart had painted it in 36 ms; it never showed while the output was logged or traced, so its cause is not found. `drive.py` now writes all of pChart's output on, which was one guess and did not end it; when it shows, record again.
- **Help and finder** (pictures 50), now built as decided above. Under them the screen is drawn in the dimmed colours of the date question, and the light page of the export preview steps back, since it cannot dim. In foot a picture drawn over another is drawn again after the one under it changed, so the help and the finder stay on top while the screen under them is painted anew.
- **Adding, renaming, linking and the details** (pictures 51 and 52), now built as decided above. A message after the key hints is cut at the window's edge when the hints fill the line, as "The details need 150 columns" is at 120 columns; this was so before and is not changed yet.
- **Removing an item** (pictures 53), now built as decided above.
- **The start and the templates** (pictures 54), now built as decided above. The templates are plan files bundled in `pchart-store`; the plans opened last are kept in `$XDG_STATE_HOME/pchart/recent`, else `~/.local/state/pchart/recent`, at most eight, and a plan joins them when it is opened in the terminal. `pchart new PLAN --template ID [--name NAME] [--target DATE | --start DATE]` does the same on the command line, with `--dry-run` and `--json`; it never writes over a file either. `--today` and `--theme` work for the start too; without a terminal `pchart` alone still prints the help.
- **Text inside its shape** (pictures 55), now built as decided above. `Output::crossings` reads a picture as it is drawn, with the bundled fonts, and finds every text whose glyphs touch a filled shape without lying wholly inside it, with air of a sixteenth of the text's height and half the shape's edge line; shapes hidden under the ground a text stands on, and the part of a text that a later panel covers, do not count. Tests run it over the roadmap at every level and 19 sizes and over every export page, for the sample, the crowded plan and each template as it is, from a start date and to a target date, and over every screen, the help, the finder, each dialog and the start, at seven window sizes and three cell sizes. Lines through text, such as a "waits on" line through a name at the Full level, are not checked.
- **Why this date?** (pictures 56), now built as decided above. While building it: a frame title that does not fit next to the detail switch takes a shorter form ("WHY THIS DATE · LANES 2–7 OF 7"), as the title with "LANES" ran into "Detail" in small windows before.
- **Project structure and network diagram** (pictures 57), now built as decided above, in `pchart-draw` (`structure.rs`); the finder knows them as "wbs", "psp", "structure", "netzplan" and "network". Not yet: a screen of their own in the TUI (they show in the Export preview), and Markdown. Where two items share a cell of the network their names are shortened ("Analytics a…" in the Website relaunch template), lines that skip rows run along the edge of their column and can lie close together, and a chain that changes lanes draws long lines across.
- **Dependency matrix and Graphviz file** (pictures 58), now built as decided above, in `pchart-draw` (`structure.rs` and `dot.rs`); the finder knows the matrix as "dsm", "matrix" and "dependencies". From about 40 items the cells shrink to their least, 10 points, and only every fifth column has its number; row names longer than 260 points are shortened. With lanes as clusters Graphviz sometimes lays the picture out restlessly.
- **Structure without time** (pictures 59), now built as decided above. Work without a length has no `duration` line in the plan file; older pChart builds refuse such a file ("has no duration"), while older files load as before. On the Export screen the chosen view now fills its whole row, with its name half a point lower, since names with tall or low letters touched its edge ("Network diagram", "Now / Next / Later"); the test now looks at every view. On the situation and checks screens without dates keys do nothing but switch screens, quit and open help and the finder.
- **Editing in the network diagram** (pictures 60), now built as decided above. The page is drawn as a window onto the light page (`Scene::window`), and the network's layout is its own function (`structure::layout`), which the arrow keys use. A dialog is drawn into the same picture, since a second picture over the same cells is not shown. The arrow keys go to the nearest box that way, which in wide plans can be a surprise; the message after the last length is yellow, as every message in the key bar is. The network diagram stays an export page for plans with dates.
- **Not yet:** the mouse, after the first release. The key bar shows only keys that work.
- **The buffer key** (pictures 61), now built as decided above; the advice and the key use the same edits (`text::buffer_edits`), so what the screen promises is what `b` does.
- **Reading a folder** (pictures 62) was built here and is parked since 2026-11-02 on branch `read` (at `b293977`), see above. Its draft follow-up, a diagram of a read project's parts like Greptile's (pictures 66, not approved), is kept outside git with its mockups.
- **Projects of a workspace** (pictures 63), now built as decided above; `pchart-store` finds the workspace (`workspace`) and its projects (`projects`). Not yet: the archive and roadmaps over several projects (teams and people came with pictures 64). At 80 columns the stand of the chosen project is cut at the edge, as it was for a plan opened before.
- **Teams, people and capacity** (pictures 64), now built as decided above; `pchart-core` checks people across projects (`people`), `pchart-store` loads the workspace with its people and projects (`crew`), `pchart-draw` draws the capacity table and the weeks of people (`people`). `pchart set PLAN ID --share 30` and the edit `share` set a share. While building it: a week counts holidays where the work is done as days off, as the checks do; a project that cannot be read or scheduled is left out of the checks across projects; when the work of a chosen team does not fit under the table, it shows the first of them and says how many ("1 OF 2") (until pictures 73, which choose only people). Not yet: teams and places from `workspace.toml` for the lanes, people in exports, a capacity per team over the people it has, and the archive and roadmaps over several projects.
- **Reports from the team** (pictures 65), now built as decided above; `pchart-core` has the report and what taking it changes (`report`), `pchart-store` reads, writes and answers report files (`reports`, `report`, `answer`, `reopen`) and knows who uses pChart here (`me`). Not yet: answering from the command line, a report about several items at once, reports that name a new date instead of days more, and showing the reporter on their own computer which of their reports were changed when taken, beyond the Reports screen.
- **Snapshots and what changed** (pictures 67), now built as decided above; `pchart-core` takes snapshots and works out what changed (`snapshot`), and `Plan::keep` sorts them in and makes the plan format 2. The old dates on the roadmap are the dashed ghosts of the what-if; names now keep clear of a ghost milestone, also in the what-if, where "Design sign-off" covered its old diamond.
- **Choosing in the export, and o** (pictures 68), now built as decided above; `pchart_tui::run` says whether the user asked for the start, and `pchart` then shows it and opens what is chosen there.
- **Adding to a new plan** (pictures 69), built as decided above.
- **The details column, Space in the key bar and the whole version** (pictures 71), built as decided above; version 0.1.3.
- **People and lanes in a new plan** (pictures 72), built as decided above; version 0.1.4.
- **A team without writing a file** (pictures 73), built as decided above; version 0.1.5.
- **Present mode** (pictures 74), built as decided above; version 0.1.6.
- **Key hints in a narrow window** (pictures 75), built as decided above; version 0.1.6.
- **Details in present mode and `o` everywhere** (pictures 76), built as decided above; version 0.1.7.
- **What changed, on the first slide** (pictures 77), built as decided above; version 0.1.9.
- **The man page** (maintainer, 2026-11-03, outline approved): `man/pchart.1`, written by hand as rumb's, installed by `make install` and the package to `share/man/man1`: the screens, the keys everywhere and on each screen and in present mode, every command with its options, the edits, the plan file with an example, the workspace, holidays, themes, environment, files, exit status, terminals and examples. Tests check that it names every command, option, flag, edit and exit code of `pchart help` and every key of the help screens, and that its example plan and workspace load. The README points to it; the help screen named only screens 1–4 and now says "1–6 The screens in the title bar". Version 0.1.10.
- **Phases that overlap** (pictures 79), built as decided above; version 0.1.11.
- **Live theme fixes** (maintainer, 2026-11-03: "live theming does not take all the colours, it needs a restart"): the start (`pchart` without a plan, and `o`) did not follow the Omarchy theme at all; it now looks once a second as an open plan does, and a theme that cannot be read while pChart runs is Labra, as at the start, instead of keeping the last colours. Measured off screen in kitty and in foot (with the OSC colours `omarchy-theme-set-foot` writes): after a switch every screen, the start among them, is the same to the pixel as a fresh start in the new theme; no Omarchy, a theme that cannot be read at the start or while running, and a theme named labra are the same to the pixel as `--theme labra`. Of the 126 themes installed here, pChart reads all; 44 older ones name only color0 to color15 and give red and yellow through color1 and color3. pChart takes from a theme only what its roles use: background, foreground (light, bright), selection (only when it is quiet, contrast 1.1 to 1.6, else a mix), accent (else blue), red and yellow; dim, muted and rule are mixed from background and foreground.
- **Quitting with a what-if** (pictures 78), built as decided above; version 0.1.12.
- **Measured** at a load of about 3 (change to picture, median / p90): kitty 32 / 40 ms, Ghostty 27 / 29 ms, foot 30 / 31 ms. Transparent pictures cost kitty and Ghostty about 4 ms more to encode than opaque ones (8.5 instead of 4 ms); foot is as fast as in the spike.
- **Sixel and light pages:** the sixel palette is fixed and built from the dark drawing colours, so the white export page came out in the beige of the ink in foot. A light page now gets a palette of its own from the colours of the light pages, their edges, and the page and its shadow over the dark ground; the preview says it shows one.
- **Measured** after the Checks and Export screens (change to picture, median / p90): kitty 29.7 / 30.5 ms, Ghostty 26.6 / 27.8 ms, foot 28.5 / 30.7 ms; peak memory 28, 28 and 23 MB.

## Open decisions

None at the moment. Workspace (1) and snapshots (2) were decided on 2026-11-01, reading a folder (3) with pictures 62, which is parked on branch `read` since 2026-11-02.
