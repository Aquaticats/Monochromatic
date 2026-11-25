# Vet report: recursive file watcher for the Garage bisync

## Report metadata

- Status:
  in progress,
  not reviewed,
  not approved by the owner,
  no recommendation yet.
- Lifecycle phase:
  discovery and screening complete (saturated);
  targeted evidence, finalist validation and scoring not started.
  Broad discovery ran as the Pi subagent run the owner chose on 2026-11-25T17:19:41-05:00.
  The owner added a mid-run requirement at 2026-11-25T17:38:21-05:00,
  so "Fingerprint input" no longer lists every hard requirement;
  the coordinator re-keys this report after the run.
- Subject:
  recursive file watcher for garage bisync.
- Scope:
  pick the tool that watches `~/Plain` recursively
  and runs `systemctl --user start rclone-bisync-garage.service` after changes,
  on this CachyOS PC now and on the macOS and Windows clients later.
- Started:
  2026-11-25T17:19:41-05:00.
- Last updated:
  2026-11-25T17:41:36-05:00.
- Governing skill:
  `.claude/skills/choosing-technology/SKILL.md` at commit `45d42d9b23d2f2752f9c512683b45905e13afe99`,
  SHA-256 `f4cfa6bc5d1d71f4b136f92ff9fbdab68575c920bd334f824d253336c8782e1b`.
- Compatibility fingerprint:
  `928b2333d4d7cc480b13a355821cf6f6cd7377ecaf5eb58238cf1b9ce3c2b32a`.
  The canonical input is the RFC 8785 serialization of the object in "Fingerprint input".
- Active audit owner:
  the Claude Code session that runs the grilling in `doc/planning/garage-bisync-run-on-file-change.md`;
  the discovery phase is written by the Pi run it starts.
- Prior compatible report:
  none;
  no report under `doc/audit/` covers a file watcher.

## Fingerprint input

```json
{
  "baseCategories": [
    "inspectable open-source local technology"
  ],
  "decisionScope": "Pick the tool that watches ~/Plain recursively and runs systemctl --user start rclone-bisync-garage.service after changes, on this CachyOS PC now and on the macOS and Windows clients later",
  "deployment": {
    "later": "macOS arm64 under launchd; Windows x64 under Task Scheduler",
    "now": "CachyOS Linux x86-64, systemd 262 user manager, home on ZFS"
  },
  "hardConstraints": [
    "open source with inspectable source",
    "runs an arbitrary command, here systemctl --user start rclone-bisync-garage.service",
    "runs on Linux x86-64, macOS arm64 and Windows x64",
    "watches every folder under the root recursively, including folders created after it starts"
  ],
  "incumbent": null,
  "overlays": [
    "multi-platform claim",
    "native or prebuilt binary boundary"
  ],
  "schemaVersion": 1,
  "subject": "recursive file watcher for garage bisync",
  "trustBoundary": "runs as the desktop user; reads names and events under ~/Plain; runs one fixed command; no credentials pass through it"
}
```

## Context

The design record is `doc/planning/garage-bisync-run-on-file-change.md`;
its "Facts measured" and "Facts gathered for round 2" sections hold the measurements this report relies on.
The owner's request:
"Make my Garage setup trigger a manual sync whenever something changes on relevant dirs."
The owner's answers that bind this vet:

- every folder of `~/Plain` counts,
  including folders created later
  (1454 folders and 10332 files measured);
- the watcher starts the existing `rclone-bisync-garage.service`,
  never a second rclone command;
- the same watcher must run on Linux,
  macOS and Windows (a hard requirement);
- the hourly timer becomes a 5-minute timer,
  because Garage cannot push change notices.

## Hard constraints

- Open source with inspectable source.
- Runs an arbitrary command,
  here `systemctl --user start rclone-bisync-garage.service`.
- Runs on Linux x86-64,
  macOS arm64 and Windows x64.
- Watches every folder under the root recursively,
  including folders created after it starts.

### Requirement added during discovery

The owner added this at 2026-11-25T17:38:21-05:00,
recorded in `doc/planning/garage-bisync-run-on-file-change.md` section "Owner answers, round 3":

> When a change lands while a sync runs, one more sync starts after that run ends,
> by the watcher itself or by a small wrapper around the command it runs.

It is a hard requirement of the design
and it does not exit any candidate at screening,
because a wrapper around the command can supply it.
Every ledger entry records whether the candidate meets it natively (with the primary source that says so)
or only through a wrapper.
"Fingerprint input" was frozen before this answer,
so its `hardConstraints` list is incomplete for the current request.

## Base components and overlays

- Base category:
  inspectable open-source local technology (an executable or a library run by a small program).
- Overlay:
  multi-platform claim (Linux, macOS, Windows).
- Overlay:
  native or prebuilt binary boundary,
  for candidates installed as prebuilt release binaries.
- Not applicable:
  the high-trust overlay,
  because no credential passes through the watcher
  (rclone reads its own config inside the unit the watcher starts);
  the sensitive-data overlay,
  because the watcher sees file names only and sends nothing off the machine.

## Capabilities recorded for every candidate

The owner has not yet decided these,
so they are recorded per candidate and are not gates:

- runs the command to completion and never kills a run in progress;
- merges a burst of events into one run (a quiet period);
- queues one more run when events land while a run is going;
- excludes paths by pattern (for example rclone's `*.partial` download names);
- what it does when the inotify watch limit or the macOS and Windows equivalents run out;
- whether it reports hidden files;
- a mid-run change starts one more run after the current run ends
  (owner answer, round 3; a hard requirement that exits no candidate).

For a library candidate these are properties of the program around it,
so its entry records what the library itself documents and what the small program would have to add.

## Unresolved preferences

- Quiet period length,
  follow-up run policy,
  and alert noise:
  later grilling rounds in the design record.
- Soft criteria and weights:
  not frozen yet;
  they freeze before any candidate is rated.

## Query schedule

Frozen at 2026-11-25T17:19:41-05:00,
before any discovery query ran.
Each query is recorded under "Query log" with the fields the governing skill's "Record every query" section lists.
After the initial schedule,
one de-duplicated expansion round is appended from the new taxonomy terms in the ledger,
then the schedule freezes.
Paginated sources continue until two consecutive complete pages add no screening survivor
or the provider reports the end.

### Source class 1: registries

- `mise registry` output filtered for `watch`, `inotify`, `fsevent`, `notify`.
- aqua registry (`aquaproj/aqua-registry`) for `watch`, `file watcher`.
- Arch repositories and the AUR for `file watcher`, `inotify`, `watch command`.
- crates.io for `file watcher`, `watch command`, `run command on change`.
- npm for `file watcher cli`, `watch run command`.
- Homebrew formulae for `file watcher`, `fsevents`, `watch command`.
- winget (`microsoft/winget-pkgs`) for `file watcher`, `watch`.

### Source class 2: repository host

- GitHub topics `file-watcher`, `filesystem-events`, `inotify`, `fsevents`, `file-watching`.
- GitHub code search for `rclone bisync` together with `watchexec`, `inotify`, `fswatch`, `watch`.

### Source class 3: web

- `run command on file change recursive cross-platform`
- `rclone bisync on file change`
- `rclone bisync inotify`
- `watchexec alternative`
- `lsyncd alternative`
- `entr alternative`
- `fswatch alternative windows`

### Source class 4: this repository

Done at 2026-11-25T17:19:41-05:00:
`toolchain.lock.json` locks `watchexec` 2.8.0;
`package/watch-restart/cli` wraps Chokidar to restart a long-running child;
`doc/planning/kopia-source-watch-package.md` (plan only) chose Chokidar for a recursive watch that starts a periodic job;
`package-paused/desktop-daemon/editord/HANDOVER.custom-dev-server-watcher.md` compares Chokidar and Watchman;
no prior watcher vet exists under `doc/audit/`.

### Expansion round (one, from new taxonomy terms)

Appended at 2026-11-25T17:41:36-05:00 from taxonomy terms the ledger recorded during the initial round:
`live reload`, `hot reload`, `nodemon`, `notify` (the Rust library name),
the registry blind spot for description-substring queries,
and the absence of any repository pairing a watcher with `rclone bisync`.
Duplicates of the initial schedule are not repeated.

Source class 1:

- mise registry filtered for `reload`, `hot reload`, `nodemon`.
- aqua registry for `live reload`, `hot reload`, `nodemon`, `reload`.
- crates.io for `notify` and for `live reload`.
- npm for `nodemon alternative`.
- Homebrew formulae on name and description matching `watch`, `notify`, `reload`,
  and descriptions containing `watched files change` or `runs commands`,
  because description-substring queries for `file watcher` missed `watchexec`.
- AUR for `live reload`.
- winget: not re-run;
  the initial round enumerated every manifest directory, so a new query would add no reach.

Source class 2:

- GitHub topics `hot-reload` and `livereload`.
- GitHub code search for `rclone bisync` alone, to catch integration projects the paired terms missed.

Source class 3:

- `nodemon alternative run command on change`
- `watchman trigger example run command`
- `watchman triggers on windows support`
- `hot reload tool runs command windows macos linux`

Schedule frozen at 2026-11-25T17:41:36-05:00.
Terms that appear later are recorded in the ledger without appending further queries.

## Query log

Every query below ran on 2026-11-25 between 17:22 and 17:41 -05:00,
from `/home/user/Monochromatic`,
against providers reached over the network or from the local toolchain.
"New screening survivors" records the outcome after screening,
not the discovery status at query time.
Executed commands and their exit status are in "Evidence records" section "Provider probes".

### Class 1: registries

#### Q1. `mise registry` filtered for watch, inotify, fsevent, notify

- Literal query: `mise registry` piped through `grep -Ei 'watch|inotify|fsevent|notify'`.
- Provider or registry: mise 2026.11.14 registry listing, 1079 rows.
- Include filters: case-insensitive alternation over tool name and backends.
- Exclude filters: none.
- Sort order: provider order (alphabetical by tool).
- Page or cursor: single complete listing; the provider offers no pagination for it.
- Result count: 4 rows.
- New candidates: `watchexec` (known already), `hwatch`, `bazel-watcher`, `tfnotify`.
- New screening survivors: `watchexec`.

#### Q2. aqua registry, watch and notify terms

- Literal query: fetch `registry.yaml` from `aquaproj/aqua-registry` `main`, then filter package name and description
  on `watch|inotify|fsevent|notify|change` in Python.
- Provider or registry: aqua registry, 3,609,579 bytes, 2399 packages.
- Include filters: regex over name and description.
- Exclude filters: none.
- Sort order: file order (package name).
- Page or cursor: single file, complete.
- Result count: 38 of 2399.
- New candidates: `modd`, `cargo-watch`, `viddy`, `hwatch`, `bazel-watcher`, `git-ghost`, `watchexec` (known).
- New screening survivors: `modd`, `watchexec`.

#### Q3. Arch repositories, `file watcher`

- Literal query: `https://archlinux.org/packages/search/json/?q=file+watcher`.
- Provider or registry: Arch Linux package search JSON API.
- Include filters: provider's name and description match.
- Exclude filters: none.
- Sort order: provider default.
- Page or cursor: single page, provider caps at 100 rows.
- Result count: 0.
- New candidates: none.
- New screening survivors: none.
- Blind spot: the provider matches whole words against name and description only;
  a package whose description says "watches" rather than "file watcher" is invisible.

#### Q4. Arch repositories, `inotify`

- Literal query: `https://archlinux.org/packages/search/json/?q=inotify`.
- Provider or registry: Arch Linux package search JSON API.
- Include filters: provider's name and description match.
- Exclude filters: none.
- Sort order: provider default.
- Page or cursor: single page.
- Result count: 7.
- New candidates: `inotify-tools`, `inotify-win`-family note, `kde-inotify-survey` (adjacent tool, not a watcher).
- New screening survivors: none (`inotify-tools` exits on platform later).

#### Q5. Arch repositories, `watch command`

- Literal query: `https://archlinux.org/packages/search/json/?q=watch+command`.
- Provider or registry: Arch Linux package search JSON API.
- Include filters: provider match.
- Exclude filters: none.
- Sort order: provider default.
- Page or cursor: single page.
- Result count: 0.
- New candidates: none.
- New screening survivors: none.

#### Q6. AUR, `file watcher`

- Literal query: `https://aur.archlinux.org/rpc/?v=5&type=search&by=name-desc&arg=file+watcher`.
- Provider or registry: AUR RPC v5.
- Include filters: name and description.
- Exclude filters: none.
- Sort order: provider default (name).
- Page or cursor: single page of 6, provider returns all matches for this size.
- Result count: 6.
- New candidates: `funzzy`, `fyr`, `spook`, `stalk`.
- New screening survivors: `funzzy` (with pending Windows evidence).

#### Q7. AUR, `inotify`

- Literal query: `https://aur.archlinux.org/rpc/?v=5&type=search&by=name-desc&arg=inotify`.
- Provider or registry: AUR RPC v5.
- Include filters: name and description.
- Exclude filters: none.
- Sort order: provider default.
- Page or cursor: single page.
- Result count: 26.
- New candidates: `clsync`, `fsniper`, `iwatch`, `flowhook`, `watchman-bin`, `inotify-tools-git`,
  plus inotify bindings for Ada, C++, OCaml, Perl, PHP and Python.
- New screening survivors: none (all exit on platform or category).

#### Q8. AUR, `watch command`

- Literal query: `https://aur.archlinux.org/rpc/?v=5&type=search&by=name-desc&arg=watch+command`.
- Provider or registry: AUR RPC v5.
- Include filters: name and description.
- Exclude filters: none.
- Sort order: provider default.
- Page or cursor: single page.
- Result count: 9.
- New candidates: `hwatch`, `viddy`, `sasqwatch`, `dwatch`.
- New screening survivors: none (all are periodic output watchers, category mismatch).

#### Q9. crates.io, `file watcher`, pages 1 to 3

- Literal query: `https://crates.io/api/v1/crates?q=file+watcher&per_page=50&page=N&sort=relevance`.
- Provider or registry: crates.io API, User-Agent `watcher-vet-discovery`.
- Include filters: relevance query; page 2 and 3 filtered on `watch|notify|inotify|fsevent|change|entr|lsync`
  for reading, with the unfiltered counts recorded here.
- Exclude filters: none applied to the request.
- Sort order: relevance.
- Page or cursor: pages 1, 2, 3 of 50; provider total 1677 crates.
- Result count: 150 rows read of 1677.
- New candidates: `funzzy`, `watcher` (crates), `file-watcher`, `filesentry`, plus many per-project watcher crates.
- New screening survivors: `funzzy`; the rest are libraries, project scaffolding, or single-purpose crates.

#### Q10. crates.io, `watch command`, pages 1 to 3

- Literal query: `https://crates.io/api/v1/crates?q=watch+command&per_page=50&page=N&sort=relevance`.
- Provider or registry: crates.io API.
- Include filters: as Q9.
- Exclude filters: none.
- Sort order: relevance.
- Page or cursor: pages 1, 2, 3; provider total 6166.
- Result count: 150 rows read of 6166.
- New candidates: `viddy`, `cargo-watch`, `watchexec-cli`, `watchexec` (library), `xtask-watch`.
- New screening survivors: `watchexec`, via `watchexec-cli` 2.8.0 on page 2.

#### Q11. crates.io, `run command on change`, pages 1 to 3

- Literal query: `https://crates.io/api/v1/crates?q=run+command+on+change&per_page=50&page=N&sort=relevance`.
- Provider or registry: crates.io API.
- Include filters: as Q9.
- Exclude filters: none.
- Sort order: relevance.
- Page or cursor: pages 1, 2, 3; provider total 28395.
- Result count: 150 rows read of 28395.
- New candidates: none beyond Q10.
- New screening survivors: none new.

#### Q12. npm, `file watcher cli`, pages 1 to 3

- Literal query: `https://registry.npmjs.org/-/v1/search?text=file%20watcher%20cli&size=50&from=N`.
- Provider or registry: npm registry search.
- Include filters: none on the request; rows filtered on `watch|notify|inotify|change` when reading.
- Exclude filters: none.
- Sort order: provider relevance.
- Page or cursor: `from` 0, 50, 100; provider total 880251.
- Result count: 150 rows read.
- New candidates: `chokidar`, `chokidar-cli`, `sane`, `watchpack`, `@parcel/watcher`, `nsfw`, `watcher`, `filewatcher`,
  `node-watch`, `glob-watcher`, `simple-watcher`, `cheap-watch`, `watch`, `wt`.
- New screening survivors: `chokidar-cli`; `chokidar` and the other libraries are recorded separately.

#### Q13. npm, `watch run command`, pages 1 to 3

- Literal query: `https://registry.npmjs.org/-/v1/search?text=watch%20run%20command&size=50&from=N`.
- Provider or registry: npm registry search.
- Include filters: as Q12.
- Exclude filters: none.
- Sort order: provider relevance.
- Page or cursor: `from` 0, 50, 100; provider total 892540.
- Result count: 150 rows read.
- New candidates: `onchange`, `watch-run`, `watchlist`, `run-when-changed`, `@lerna-lite/watch`, `watch-cli`,
  `npm-watch`, `node-dev`, `nodemon`, `@bazel/ibazel`, `hash-runner`, `light-server`.
- New screening survivors: `onchange`, `watch-run`, `watchlist`, `run-when-changed`.

#### Q14. Homebrew formulae, `file watcher`

- Literal query: local filter of `https://formulae.brew.sh/api/formula.json` (8740 formulae)
  on name or description containing `file watcher`.
- Provider or registry: Homebrew formula API dump.
- Include filters: lowercase substring on name and description.
- Exclude filters: none.
- Sort order: file order (name).
- Page or cursor: single complete dump.
- Result count: 1.
- New candidates: `funzzy`.
- New screening survivors: `funzzy` (with pending Windows evidence).
- Blind spot: `watchexec` is in Homebrew but its description reads "Execute commands when watched files change",
  which the frozen query missed; the expansion round closes this.

#### Q15. Homebrew formulae, `fsevents`

- Literal query: local filter of the same dump on `fsevents`.
- Provider or registry: Homebrew formula API dump.
- Include filters: substring on name and description.
- Exclude filters: none.
- Sort order: file order.
- Page or cursor: single complete dump.
- Result count: 2.
- New candidates: `fsevent_watch`, `fsevents-tools`.
- New screening survivors: none (macOS-only event printers, no command execution).

#### Q16. Homebrew formulae, `watch command`

- Literal query: local filter of the same dump on `watch command`.
- Provider or registry: Homebrew formula API dump.
- Include filters: substring on name and description.
- Exclude filters: none.
- Sort order: file order.
- Page or cursor: single complete dump.
- Result count: 2.
- New candidates: `hwatch`, `viddy`.
- New screening survivors: none (periodic output watchers).

#### Q17. winget, `file watcher` and `watch`

- Literal query: `gh api search/code -f q='repo:microsoft/winget-pkgs watcher'`,
  then `gh api search/code -f q='repo:microsoft/winget-pkgs PowerToys'` as a positive control.
- Provider or registry: GitHub code search over `microsoft/winget-pkgs`.
- Include filters: repository scope.
- Exclude filters: none.
- Sort order: provider default.
- Page or cursor: first page only.
- Result count: 0 for both queries, the control included.
- New candidates: none.
- New screening survivors: none.
- Outcome: the provider returns no rows for this repository at all,
  so code search is unusable here and an independent enumeration replaced it (Q18).

#### Q18. winget, full manifest enumeration

- Literal query: `gh api repos/microsoft/winget-pkgs/git/trees/master:manifests/<letter>?recursive=1`
  for `0` and `a` to `z`, then per-publisher trees for `m`, keeping depth-2 package directories.
- Provider or registry: `microsoft/winget-pkgs` git trees API.
- Include filters: package directory names matched against `watch|notify|inotify|fsevent|entr`.
- Exclude filters: none.
- Sort order: tree order, then sorted unique.
- Page or cursor: 27 letter trees plus 665 publisher subtrees under `m`.
- Result count: 14170 distinct package directories, 75 matching names.
- New candidates: `watchexec`, `fswatch`, `watchman`, `inotifywait` (thekid Windows port), `ImmichFolderWatch`.
- New screening survivors: `watchexec`, `watchman`.
- Note: the `m` letter tree was truncated by the provider,
  so it was enumerated per publisher; every other letter reported `truncated: false`.

### Class 2: repository host

#### Q19. GitHub topic `file-watcher`, pages 1 to 4

- Literal query: `search/repositories?q=topic:file-watcher&sort=stars&order=desc`.
- Provider or registry: GitHub search API, authenticated CLI.
- Include filters: topic.
- Exclude filters: none.
- Sort order: stars, descending.
- Page or cursor: pages 1 to 4 of 100; provider total 325.
- Result count: 400 rows read.
- New candidates: `air`, `wgo`, `funzzy`, `Zippy`, `directory-watcher`, `quay`, `godemon`, `iwatcher`,
  `fswatcher` (Go), `watchful`, `steeloverseer`.
- New screening survivors: `wgo`, `directory-watcher`, `godemon` (pending Windows evidence),
  `quay` (pending), `iwatcher`.
- Saturation: pages 3 and 4 added no screening survivor, meeting the two-consecutive-page rule.

#### Q20. GitHub topic `filesystem-events`, page 1

- Literal query: `search/repositories?q=topic:filesystem-events&sort=stars&order=desc`.
- Provider or registry: GitHub search API.
- Include filters: topic.
- Exclude filters: none.
- Sort order: stars, descending.
- Page or cursor: page 1 of 100; provider total 50, so this is complete.
- Result count: 50.
- New candidates: `notify` (Rust), `watcher` (Go), `fswatcher`, `rjeczalik/notify`, `inotify-win`, `panoptes`.
- New screening survivors: none (libraries or platform-bound tools).

#### Q21. GitHub topic `inotify`, pages 1 to 4

- Literal query: `search/repositories?q=topic:inotify&sort=stars&order=desc`.
- Provider or registry: GitHub search API.
- Include filters: topic.
- Exclude filters: none.
- Sort order: stars, descending.
- Page or cursor: pages 1 to 4 of 100; provider total 363.
- Result count: 400 rows read.
- New candidates: `entr`, `fswatch`, `inotify-tools`, `watchfiles`, `trigger`, `clsync`, `inotify-win`,
  `watchngo`, `watchdog`, `epidemic`.
- New screening survivors: `watchfiles`; `watchngo` with pending platform evidence.
- Saturation: pages 3 and 4 added no screening survivor.

#### Q22. GitHub topic `fsevents`, page 1

- Literal query: `search/repositories?q=topic:fsevents&sort=stars&order=desc`.
- Provider or registry: GitHub search API.
- Include filters: topic.
- Exclude filters: none.
- Sort order: stars, descending.
- Page or cursor: page 1 of 100; provider total 68, so this is complete.
- Result count: 68.
- New candidates: `chokidar`, `fswatcher`, `fsevent-rust`, `godemon`, `jp-watch-c`.
- New screening survivors: none new.

#### Q23. GitHub topic `file-watching`, page 1

- Literal query: `search/repositories?q=topic:file-watching&sort=stars&order=desc`.
- Provider or registry: GitHub search API.
- Include filters: topic.
- Exclude filters: none.
- Sort order: stars, descending.
- Page or cursor: page 1 of 100; provider total 23, so this is complete.
- Result count: 23.
- New candidates: `java-watch`, `file_watcher` (Crystal), `watch-run` variants, `hot-reload` (Go).
- New screening survivors: none.

#### Q24. GitHub code search, `rclone bisync` paired with watcher terms

- Literal queries: `rclone bisync watchexec`, `rclone bisync inotify`, `rclone bisync fswatch`,
  `rclone bisync watch`.
- Provider or registry: GitHub code search, authenticated CLI, limit 15 per query.
- Include filters: none beyond the query text.
- Exclude filters: none.
- Sort order: provider default (best match).
- Page or cursor: first page only.
- Result count: 0, 1, 0, 1.
- New candidates: none; the one external hit is a blog post
  (`nawashiro/nawashiro.github.io`, "Syncing Google Drive on Linux")
  whose unit is named `Rclone Bisync Watch Service`, not a tool.
- New screening survivors: none.
- Reading: no project pairs an rclone bisync run with a file watcher in code searchable here;
  the single hit is documentation of a systemd unit.

### Class 3: web

Provider for every class-3 query: the `web_search` tool,
which resolved to its DeepSeek-backed search and returned ranked result objects
with title, site, URL, date and extracted page text.
No include or exclude domain filters were set.
Sort order was the provider's relevance ranking.
No query offers a cursor; each returned its full result set in one page.

#### Q25. `run command on file change recursive cross-platform`

- Result count: 11.
- New candidates: `codeskyblue/fswatch` (Go), `@lowdep/watch-run`, `iwatcher`, `watchexec-simple`.
- New screening survivors: `iwatcher`, `watchexec-simple` (pending Windows evidence).

#### Q26. `rclone bisync on file change`

- Result count: 4.
- New candidates: none; all hits are rclone's own bisync documentation.
- New screening survivors: none.
- Reading: the capability the owner wants is not an rclone feature.

#### Q27. `rclone bisync inotify`

- Result count: 8.
- New candidates: `namastexlabs/linux-gdrive-sync` (a Linux wrapper around rclone).
- New screening survivors: none (`linux-gdrive-sync` exits on platform).

#### Q28. `watchexec alternative`

- Result count: 12.
- New candidates: `watchexec-simple`, `cargo-watch`.
- New screening survivors: `watchexec-simple` (pending Windows evidence).

#### Q29. `lsyncd alternative`

- Result count: 9.
- New candidates: none new (`foxing` is a sync tool, not a command runner).
- New screening survivors: none.

#### Q30. `entr alternative`

- Result count: 16.
- New candidates: `nodemon` (via comparison pages), `air` (via comparison pages).
- New screening survivors: none at this query; both exit later on category.

#### Q31. `fswatch alternative windows`

- Result count: 9.
- New candidates: none new; the pages confirm `entr` and `fswatch` as the usual pair
  and list no Windows-native equivalent.
- New screening survivors: none.

### Expansion round

#### Q32. `mise registry` filtered for reload terms

- Literal query: `mise registry` piped through `grep -Ei 'reload|hot.?reload|nodemon'`.
- Provider or registry: mise 2026.11.14 registry.
- Include filters: regex above.
- Exclude filters: none.
- Sort order: provider order.
- Page or cursor: single complete listing.
- Result count: 0 of 1079.
- New candidates: none.
- New screening survivors: none.

#### Q33. aqua registry, reload terms

- Literal query: filter of the fetched `registry.yaml` on `live.?reload|hot.?reload|nodemon|reload`.
- Provider or registry: aqua registry, 2399 packages.
- Include filters: regex over name and description.
- Exclude filters: none.
- Sort order: file order.
- Page or cursor: complete file.
- Result count: 2.
- New candidates: `air`, `arelo`.
- New screening survivors: `arelo`.

#### Q34. crates.io, `notify`

- Literal query: `https://crates.io/api/v1/crates?q=notify&per_page=20&sort=relevance`.
- Provider or registry: crates.io API.
- Include filters: none.
- Exclude filters: none.
- Sort order: relevance.
- Page or cursor: page 1 of 20; provider total 3792.
- Result count: 20.
- New candidates: `notify` 9.0.0, `notify-debouncer-full`, `notify-debouncer-mini`.
- New screening survivors: none (library).

#### Q35. crates.io, `live reload`

- Literal query: `https://crates.io/api/v1/crates?q=live+reload&per_page=20&sort=relevance`.
- Provider or registry: crates.io API.
- Include filters: none.
- Exclude filters: none.
- Sort order: relevance.
- Page or cursor: page 1 of 20; provider total 3712.
- Result count: 20.
- New candidates: none in the file-watcher category (all are dev servers or renderers).
- New screening survivors: none.

#### Q36. npm, `nodemon alternative`

- Literal query: `https://registry.npmjs.org/-/v1/search?text=nodemon%20alternative&size=25`.
- Provider or registry: npm registry search.
- Include filters: none.
- Exclude filters: none.
- Sort order: provider relevance.
- Page or cursor: page 1 of 25; provider total 24917.
- Result count: 25.
- New candidates: `nodemon`, `wmon`, `turbowatch`, `@alikilicoglu/fastreload`, `linkemon`.
- New screening survivors: none (all are Node-app reloaders or Node-only).

#### Q37. Homebrew formulae, name and description regex

- Literal query: local filter of the formula dump on name or description matching
  `watch|notify|reload`, or description containing `watched files change` or `runs commands`.
- Provider or registry: Homebrew formula API dump.
- Include filters: as above.
- Exclude filters: none.
- Sort order: file order.
- Page or cursor: complete dump.
- Result count: 27.
- New candidates: `watchexec`, `watchman`, `watcher`, `gitwatch`, `dwatch`, `visionmedia-watch`, `inotify-tools`.
- New screening survivors: `watchexec`, `watchman`.

#### Q38. AUR, `live reload`

- Literal query: `https://aur.archlinux.org/rpc/?v=5&type=search&by=name-desc&arg=live+reload`.
- Provider or registry: AUR RPC v5.
- Include filters: name and description.
- Exclude filters: none.
- Sort order: provider default.
- Page or cursor: single page.
- Result count: 16.
- New candidates: `air`, `wgo`.
- New screening survivors: `wgo`.

#### Q39. GitHub topic `hot-reload`, page 1

- Literal query: `search/repositories?q=topic:hot-reload&sort=stars&order=desc`.
- Provider or registry: GitHub search API.
- Include filters: topic.
- Exclude filters: none.
- Sort order: stars, descending.
- Page or cursor: page 1 of 100; provider total 1120.
- Result count: 100.
- New candidates: none in the command-runner category.
- New screening survivors: none.

#### Q40. GitHub topic `livereload`, page 1

- Literal query: `search/repositories?q=topic:livereload&sort=stars&order=desc`.
- Provider or registry: GitHub search API.
- Include filters: topic.
- Exclude filters: none.
- Sort order: stars, descending.
- Page or cursor: page 1 of 100; provider total 267.
- Result count: 100.
- New candidates: `modd` (known), `realize`, `gowatch`.
- New screening survivors: none new (Go dev runners, category mismatch).

#### Q41. GitHub code search, `rclone bisync` alone

- Literal query: `rclone bisync`.
- Provider or registry: GitHub code search, limit 20.
- Include filters: none.
- Exclude filters: none.
- Sort order: provider default.
- Page or cursor: first page only.
- Result count: 20.
- New candidates: none; every hit is a cron job, a manual command, a wiki note or documentation.
- New screening survivors: none.
- Reading: no public integration pairs a watcher with a bisync run;
  this matches Q24.

#### Q42. `nodemon alternative run command on change`

- Result count: 11.
- New candidates: `nodemon`, `gulp-watch`.
- New screening survivors: none.

#### Q43. `watchman trigger example run command`

- Result count: 19.
- New candidates: none new; confirms the trigger mechanism as the documented way to run a command.
- New screening survivors: `watchman` (confirming evidence).

#### Q44. `watchman triggers on windows support`

- Result count: 13.
- New candidates: none.
- New screening survivors: none new.
- Reading: the returned pages are Watchman's install page, mirrors and a winget listing;
  none documents a Windows exclusion for triggers.

#### Q45. `hot reload tool runs command windows macos linux`

- Result count: 9.
- New candidates: `templier`, `go-watch`, `flux`.
- New screening survivors: none (dev-server or language-bound tools).

## Discovery ledger

Screening here applies the governing skill's screening hard gates
and records each candidate's capabilities from its primary docs.
"Exits" names the gate that ended the candidate;
exited candidates stay in the ledger so the record shows they were found.
A candidate that passes is a screening survivor, not a finalist and not a recommendation.

Round-3 note for every entry:
whether one more run starts after a busy run ends (the owner's mid-run requirement at
2026-11-25T17:38:21-05:00) natively or only through a wrapper around the command.

### Ready-to-use tools that pass screening

#### watchexec 2.8.0

- Discovery source: repository precedent (`toolchain.lock.json`), mise registry (Q1), aqua registry (Q2),
  crates.io `watch command` page 2 (Q10), winget enumeration (Q18), Homebrew expansion (Q37).
- Base category: inspectable open-source local executable.
- Overlays: multi-platform claim (release assets for `x86_64-pc-windows-msvc`, `aarch64-apple-darwin`,
  and `x86_64-unknown-linux-gnu`); prebuilt binary boundary (checksums and `.b3` sums shipped per asset).
- Screening: passes all four gates.
  Apache-2.0; runs the command named after its own options;
  Windows, macOS and Linux are named in the README feature list;
  it monitors the current directory and all subdirectories.
- Capabilities from primary docs:
  runs the command without killing it by default (`--on-busy-update` defaults to `do-nothing`);
  coalesces bursts and adds `--postpone` and `--delay-run`;
  excludes paths with `--ignore`, `--ignore-file`, `.gitignore` and `.ignore` discovery;
  watch-limit exhaustion is not documented in the manual page read;
  hidden-file reporting is not documented in the manual page read.
- Round-3 mid-run change: met natively, `--on-busy-update=queue`
  ("run the command once again when the current run has finished if any events occur while it's running").
  No wrapper needed.

#### Watchman 2026.11.23.00

- Discovery source: memory and repository precedent, confirmed by AUR `watchman-bin`, winget enumeration (Q18),
  Homebrew expansion (Q37), topic `filesystem-events`.
- Base category: inspectable open-source local executable plus a background service.
- Overlays: multi-platform claim; prebuilt binary boundary (weekly date releases, checksum files).
- Screening: passes all four gates.
  MIT; the documented trigger mechanism spawns a process for matched changes;
  the install page names Linux with inotify, macOS with FSEvents, and Windows 10 64-bit and up;
  a watch root covers the whole tree, so recursion is inherent.
- Capabilities from primary docs:
  triggers run one process at a time and re-evaluate after it exits;
  the settle period batches changes before a trigger fires;
  exclusions are query expressions and `ignore_dirs` configuration;
  trigger output goes to the Watchman log by default, not to a terminal;
  watch-limit exhaustion is handled by the documented kernel sysctl advice;
  hidden files are matched by default because queries see the whole tree.
- Round-3 mid-run change: met natively.
  The trigger page states Watchman runs a single instance of the trigger process at a time and,
  after it terminates, respawns for changes that landed in the meantime.

#### chokidar-cli (3.0.0 published on npm, 4.0.0 at repository HEAD)

- Discovery source: npm `file watcher cli` page 1 (Q12), npm `watch run command` page 1 (Q13).
- Base category: inspectable open-source local executable (Node).
- Overlays: multi-platform claim (Chokidar backends); no prebuilt native binary of its own.
- Screening: passes all four gates, with a provenance caveat to carry into validation.
  MIT; runs a command with `--command`; Node and Chokidar cover Windows, macOS and Linux;
  patterns are recursive globs.
  Provenance caveat: the published 3.0.0 declares `repository.url` as `open-npm-tools/chokidar-cli`,
  a repository whose README reads "poc by ks_karem7" with no code,
  while the project's real home moved to `open-cli-tools/chokidar-cli` (tag `v3.0.0`, 874 stars).
  The 4.0.0 code exists only at that home, and npm has no 4.0.0 release.
- Capabilities from primary docs and source:
  runs the command with `spawn` and does not kill a previous run;
  `--debounce` (default 400 ms) and `--throttle` shape bursts;
  `--ignore` accepts strings and regexes;
  watch-limit exhaustion is not documented;
  hidden files are not filtered by default.
- Round-3 mid-run change: needs a wrapper.
  The source's own comment says "commands might be still run concurrently"
  (`index.js:163`), so nothing queues a second run.

#### onchange 7.1.0

- Discovery source: npm `watch run command` page 2 (Q13).
- Base category: inspectable open-source local executable (Node).
- Overlays: multi-platform claim (Node and Chokidar); no native binary.
- Screening: passes all four gates.
  MIT; runs the command after `--`; Node covers all three platforms;
  glob patterns are recursive and Chokidar does the watching.
- Capabilities from primary docs and source:
  runs the command without killing it unless `--kill` is passed;
  `--delay` and Chokidar's `awaitWriteFinish` shape bursts;
  `--exclude` takes globs, and the default excluded set is exactly `**/node_modules/**` and `**/.git/**`
  (`src/index.ts:175-177`);
  watch-limit exhaustion is not documented;
  hidden files are matched unless excluded.
- Round-3 mid-run change: met natively.
  The source keeps a FIFO job queue with `--jobs` concurrency (default 1) and starts the next job
  when the running one finishes (`src/index.ts:151-224`). No wrapper needed.

#### watchfiles 1.3.0

- Discovery source: GitHub topic `inotify` page 1 (Q21).
- Base category: inspectable open-source local executable (Python) plus a library.
- Overlays: multi-platform claim; prebuilt binary boundary (wheels for Windows x86_64 and aarch64,
  macOS x86_64 and arm64, Linux x86_64, aarch64, i686, armv7l and musl).
- Screening: passes all four gates.
  MIT; the CLI takes a command and runs it; wheels cover all three platforms;
  "by default the CLI will watch the current directory and all subdirectories".
- Capabilities from primary docs:
  it restarts the process on change rather than letting one run finish
  (`run_process` is documented as "Run a process and restart it upon file changes",
  with `sigint_timeout` and `sigkill_timeout`);
  debounce defaults to 1600 ms and is adjustable;
  `--filter` selects by extension or custom predicate and ignore patterns are supported;
  watch-limit exhaustion is not documented on the pages read;
  hidden files are watched because the whole tree is walked.
- Round-3 mid-run change: needs a wrapper, and it also conflicts with "never kills a run in progress".
  A change during a run interrupts that run.

#### modd 0.8

- Discovery source: aqua registry (Q2), Homebrew expansion (Q37), topic `livereload` page 1 (Q40).
- Base category: inspectable open-source local executable.
- Overlays: multi-platform claim (release assets for windows32 and windows64, osx64, linux32, linux64,
  linuxARM, netbsd64, openbsd64, freebsd64).
- Screening: passes all four gates.
  MIT; runs commands from `modd.conf`; releases cover Windows, macOS and Linux;
  it watches directory trees by include and exclude patterns.
- Capabilities from primary docs and source:
  prep commands run in order and the block waits for them;
  it batches changes "until there is a lull in filesystem activity", with `lullTime` at 100 ms (`modd.go:20`);
  `!`-prefixed patterns exclude paths (`**/*.go !**/*_test.go` in the README);
  watch-limit exhaustion is not documented;
  hidden files are matched by default.
- Round-3 mid-run change: met natively.
  The runner handles one batch at a time from a buffered channel (`modd.go:202-225`), so a change during a run
  is handled as a further run after the current block returns.

#### iwatcher 0.1.1

- Discovery source: web search (Q25).
- Base category: inspectable open-source local executable.
- Overlays: multi-platform claim (release assets for `x86_64-pc-windows-msvc`, `x86_64-unknown-linux-gnu`,
  `aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`).
- Screening: passes all four gates.
  MIT; runs the command after the watch root; assets cover the three target platforms;
  README states "Recursive: subdirectories are watched automatically".
- Capabilities from primary docs:
  runs the command and does not kill it;
  `--debounce` defaults to 500 ms;
  ignore rules are documented in the README;
  watch-limit exhaustion is not documented;
  hidden files are watched (no filter documented).
- Round-3 mid-run change: needs a wrapper.
  It has "Skip-if-running": "if the previous command is still executing, the new trigger is silently dropped".

#### directory-watcher 3.2.1

- Discovery source: GitHub topic `file-watcher` page 1 (Q19).
- Base category: inspectable open-source local executable.
- Overlays: multi-platform claim (release assets for Darwin arm64 and x86_64, Linux x86_64, i386 and arm64,
  Windows x86_64, i386 and arm64).
- Screening: passes all four gates.
  MIT; hooks run arbitrary commands from a YAML config; assets cover the three platforms;
  it watches configured targets recursively.
- Capabilities from primary docs:
  `run.concurrency` and `run.interrupt` choose whether runs overlap or interrupt;
  `hookBatch.waitMillisecond` (default 100) merges events into one run;
  `watch.exclude` excludes paths;
  a full queue is an explicit error with an `onQueueWarning` hook;
  watch-limit or event-loss conditions stop the watcher and run a notification command.
- Round-3 mid-run change: met natively.
  "Collecting goes on while a hook runs. Each wait that ends puts its runs in the target's queue",
  with `option.queue.size` defaulting to 2048.

#### wgo 0.7.2

- Discovery source: GitHub topic `file-watcher` page 1 (Q19), AUR `live reload` (Q38).
- Base category: inspectable open-source local executable.
- Overlays: multi-platform claim (release assets `wgo-linux`, `wgo-linux-arm`, `wgo-macos`,
  `wgo-macos-apple-silicon`, `wgo-windows.exe`).
- Screening: passes all four gates.
  MIT; runs the command given after its own flags; assets cover the three platforms;
  the watched root is a directory tree with `-dir` filters.
- Capabilities from primary docs:
  it restarts commands on change (the live-reload model);
  debounce defaults to 300 ms (`-debounce`);
  `-dir` and `-xdir` include and exclude directories, with defaults for `.git`, `node_modules` and similar;
  watch-limit exhaustion is not documented;
  hidden files follow the same defaults.
- Round-3 mid-run change: needs a wrapper; a change restarts the command.

#### arelo 1.15.4

- Discovery source: aqua registry expansion (Q33).
- Base category: inspectable open-source local executable.
- Overlays: multi-platform claim (release assets for Darwin arm64 and x86_64, Linux arm64 and x86_64,
  Windows arm64 and x86_64; CI builds for Windows and macOS).
- Screening: passes all four gates.
  MIT; runs an arbitrary `COMMAND` after `--`; assets cover the three platforms;
  "the subdirectories are also monitored unless they match to the ignore patterns".
- Capabilities from primary docs:
  it restarts the command on change (`-d` delay, `-s` signal);
  `-d/--delay` defaults to 1 s;
  `-i/--ignore` and `-p/--pattern` filter paths;
  `--polling` is available as a fallback;
  watch-limit exhaustion is not documented.
- Round-3 mid-run change: needs a wrapper; a change restarts the command.

#### run-when-changed 2.1.0

- Discovery source: npm `watch run command` page 2 (Q13).
- Base category: inspectable open-source local executable (Node).
- Overlays: multi-platform claim (README says "Nix & Win32"); no native binary.
- Screening: passes all four gates.
  ISC; runs `--exec` commands per watch group; Node covers the platforms;
  glob patterns are recursive.
- Capabilities from primary docs:
  runs each command without a kill flag, one per matching file event;
  `--debounce` and `--interval` shape timing;
  `--watch` plus `--match` select paths;
  watch-limit exhaustion is not documented;
  hidden files are matched unless the glob excludes them.
- Round-3 mid-run change: not documented; assume a wrapper until validation shows otherwise.

#### watch-run 1.2.5 (queckezz, the npm package)

- Discovery source: npm `watch run command` page 1 (Q13).
- Base category: inspectable open-source local executable (Node, built on gaze).
- Overlays: multi-platform claim (Node); no native binary.
- Screening: passes all four gates.
  MIT in `package.json`; runs `<cmd>`; Node covers the platforms; `-p` globs are recursive.
  Its own README points readers at `wait-run` and `just-wait` as better and more recent.
- Capabilities from primary docs:
  runs the command per change; `-d` delays execution; `-i` runs once at startup;
  `-s` stops watching on error; no exclusion option beyond the glob;
  watch-limit exhaustion and hidden-file behaviour are not documented.
- Round-3 mid-run change: not documented; assume a wrapper.

#### watchlist 0.3.1

- Discovery source: npm `watch run command` page 1 (Q13).
- Base category: inspectable open-source local executable (Node, an `fs.watch` wrapper).
- Overlays: multi-platform claim (README: "Runs on macOS, Linux, and Windows").
- Screening: passes all four gates.
  MIT; runs a command on change; the README names all three platforms;
  "Recursively monitors all subdirectories".
- Capabilities from primary docs:
  runs the command per change; optional ignore patterns;
  watch-limit exhaustion is not documented;
  hidden files are matched because it walks the tree.
- Round-3 mid-run change: not documented; assume a wrapper.

#### funzzy 2.0.1

- Discovery source: AUR `file watcher` (Q6), crates.io `file watcher` page 1 (Q9), Homebrew (Q14),
  topic `file-watcher` page 1 (Q19).
- Base category: inspectable open-source local executable.
- Overlays: multi-platform claim (prebuilt assets cover darwin arm64 and x86_64, linux arm64 and x86_64 only).
- Screening: passes gates 1, 2 and 4; gate 3 is pending targeted evidence.
  MIT; jobs run commands from `.watch.yaml`; include and ignore globs watch a tree;
  no Windows asset and no Windows section in `docs/INSTALLATION.md` were found,
  and no document says Windows is unsupported, so the platform gate stays pending rather than failed.
- Capabilities from primary docs:
  jobs run in declared order, with named parallel groups;
  `on.change` and `on.ignore` globs plus a socket for control;
  cancellation stops the whole process tree;
  watch-limit exhaustion is not documented.
- Round-3 mid-run change: not documented; assume a wrapper.

#### watchexec-simple (0.2.0 on crates.io, repository last pushed 2023-10-24)

- Discovery source: web search (Q25, Q28).
- Base category: inspectable open-source local executable.
- Overlays: multi-platform claim (no release assets, no platform statement in the README).
- Screening: passes gates 1, 2 and 4; gate 3 pending targeted evidence.
  MIT; it runs the command after `--`; it is built on `notify`, whose backends cover Windows, macOS and Linux;
  the README states it "installs to the same binary name as the `watchexec` project".
- Capabilities from primary docs: a reduced option set compared with Watchexec;
  the README documents no busy-run mode, no debounce flag and no ignore-file handling.
- Round-3 mid-run change: needs a wrapper.

#### godemon

- Discovery source: GitHub topic `file-watcher` page 1 (Q19).
- Base category: inspectable open-source local executable (Go, no releases).
- Overlays: prebuilt binary boundary, but no binaries are published (build from source).
- Screening: passes gates 1, 2 and 4; gate 3 pending targeted evidence.
  Unlicense; it runs an arbitrary command after its flags;
  the README says "godemon has been tested on Mac and Linux. It has not yet been tested on Windows",
  which is an untested claim, not a documented exclusion.
- Capabilities from primary docs: throttles rapid changes to 50 ms by default;
  watches the current directory recursively; `--quiet` suppresses output.
- Round-3 mid-run change: not documented; assume a wrapper.

#### watchngo

- Discovery source: GitHub topic `inotify` page 2 (Q21).
- Base category: inspectable open-source local executable (Go).
- Overlays: prebuilt binary boundary (latest release publishes Linux amd64 only).
- Screening: passes gates 1, 2 and 4; gate 3 pending targeted evidence.
  MIT; it runs a command on change; go source with `fsnotify` could build for Windows and macOS,
  but no Windows or macOS artifact or CI evidence was found.
- Capabilities from primary docs: "Watch files recursively in a directory, with an optional pattern";
  no busy-run, exclusion or hidden-file behaviour documented.
- Round-3 mid-run change: not documented; assume a wrapper.

#### quay 0.2.0

- Discovery source: GitHub topic `file-watcher` page 1 (Q19).
- Base category: inspectable open-source local executable (a dev-server oriented watcher).
- Overlays: multi-platform claim (Rust, no releases); resource-limit flags are Unix-only.
- Screening: passes gates 1, 2 and 4; gate 3 pending targeted evidence.
  MIT in `watcher/Cargo.toml` although the repository has no LICENSE file;
  it runs commands from `quay.yaml`; no Windows artifact or CI evidence was found.
- Capabilities from primary docs: recursive watching with configurable debounce;
  a per-command timeout and process-group termination;
  automatic excludes for `target/`, `.git/`, `node_modules/`;
  WebSocket reload side effects that this use does not need.
- Round-3 mid-run change: not documented; the per-command timeout suggests a single worker; assume a wrapper.

#### Rushabh5000/watch-run (a different project from the npm package)

- Discovery source: GitHub topic `file-watching` page 1 (Q23).
- Base category: inspectable open-source local executable (Node, zero dependencies).
- Overlays: multi-platform claim (README badge: "Windows | macOS | Linux").
- Screening: passes gates 1 and 2; gates 3 and 4 rest on the project's own claims plus `fs.watch` semantics.
  MIT; runs any command; Node covers the platforms; recursive watching relies on `fs.watch` recursive,
  which Node supports on Linux from v19.1.0, macOS and Windows.
- Capabilities from primary docs: debounce, ignore patterns, and the README's comparison to `chokidar-cli`.
  Adoption evidence: 0 stars, no releases; a maintenance signal for scoring, not a screening gate.
- Round-3 mid-run change: not documented; assume a wrapper.

#### prem-the-dev/watch-run (a third project with this name)

- Discovery source: GitHub topic `file-watching` page 1 (Q23).
- Base category: inspectable open-source local executable (Node, zero dependencies).
- Overlays: multi-platform claim (README names Windows and macOS recursive watching,
  per-directory watchers elsewhere).
- Screening: passes gates 1 and 2; gates 3 and 4 rest on the project's own claims.
  MIT; runs any shell command; 0 stars and no releases, same maintenance signal.
- Capabilities from primary docs: 200 ms default debounce, gitignore-style ignores,
  keeps watching after a non-zero exit, cleans up the child on Ctrl+C.
- Round-3 mid-run change: not documented; assume a wrapper.

### Libraries (base category allows them; the existing-tools rule bars them for now)

The skill's "Existing tools before custom implementation" invariant makes a custom program eligible only when
every ready-to-use tool fails a named hard constraint.
Ready-to-use tools pass screening here, so these candidates are recorded with the same gates
but no program around them is eligible in this phase.

#### Chokidar 6.0.0

- Discovery source: repository precedent, npm (Q12), topic `fsevents` (Q22).
- Overlays: multi-platform claim; prebuilt native boundary only for optional native backends.
- Screening: passes the four gates as a library. MIT; it watches recursively,
  "Recursive watching everywhere, with an optional depth limit, path filtering, and symlink support";
  backends are `fs.watch`, FSEvents, and per-directory watching, so all three platforms are covered.
- Capabilities: everything except running a command is present (debounce lives in `awaitWriteFinish`,
  ignores, depth limits); running a command and queueing a rerun would be written in the small program.
  `npm debug` style watch limits are surfaced as `error` events.

#### Rust `notify` 9.0.0

- Discovery source: crates.io expansion `notify` (Q34), topic `filesystem-events` (Q20),
  and it is the watcher Watchexec's own README points to.
- Screening: passes as a library. CC0-1.0 for `notify` (MIT or Apache-2.0 for `notify-types` and the debouncers);
  backends: inotify on Linux and Android, FSEvents or kqueue on macOS, `ReadDirectoryChangesW` on Windows,
  polling everywhere.
- Capabilities: recursion comes from the caller walking directories (`notify` hands out per-path events);
  `notify-debouncer-mini` and `notify-debouncer-full` supply the quiet period.
  Command execution, queueing and watch-limit reporting are the program's job.

#### Go `fsnotify` 1.10.1

- Discovery source: topic `inotify` (Q21), topic `filesystem-events` (Q20).
- Screening: passes as a library. BSD-3-Clause; inotify on Linux, kqueue on BSD and macOS,
  `ReadDirectoryChangesW` on Windows.
- Capabilities: "Are subdirectories watched? No, you must add watches for any directory you want to watch";
  everything else is the program's job. Its README also records fanotify and FSEvents gaps.

#### Python `watchdog` 7.1.1

- Discovery source: topic `inotify` (Q21), topic `filesystem-events` (Q20).
- Screening: passes as a library. Apache-2.0; platforms: Linux 2.6 inotify, macOS FSEvents and kqueue,
  FreeBSD and BSD kqueue, Windows ReadDirectoryChangesW.
- Capabilities: `observer.schedule(..., recursive=True)`; a CLI example exists in its README.
  Command execution and queueing are the program's job.

#### `@parcel/watcher` 2.6.0

- Discovery source: npm (Q12).
- Screening: passes as a library. MIT; "Cross platform: includes backends for macOS, Linux, Windows,
  FreeBSD, and Watchman";
  published per-platform native packages for win32 x64 and arm64, darwin arm64 and x64, linux x64 glibc and musl,
  and others, so the prebuilt boundary maps to source.
- Capabilities: recursive subscription with `subscribe`, plus a historical `getEventsSince` query;
  command execution and queueing are the program's job.

#### Node `fs.watch` with `recursive`

- Discovery source: repository precedent and memory; confirmed in Node's `fs` documentation.
- Screening: passes as a platform API. Node's own docs record recursive support added for Linux
  in v19.1.0, and existing support on macOS and Windows.
- Capabilities: raw events only; debounce, exclusions, command execution, queueing and watch-limit
  reporting would all be written in the program. This is the largest custom-code surface of the library options.

#### Smaller libraries surfaced by the same queries

Recorded for completeness, each passing the gates as a library and each needing the same custom program:
`watchpack` 2.6.4, `sane` 5.0.1, `nsfw` 3.1.0, `fb-watchman` 2.0.4, `wt` 1.2.0, `watcher` 2.3.1,
`filewatcher` 3.0.1, `node-watch` 0.7.4, `glob-watcher` 6.0.0, `simple-watcher` 6.0.1, `cheap-watch` 1.0.4,
`radovskyb/watcher` (Go), `rjeczalik/notify` (Go), `filesentry` 0.2.2 (Rust), `jetbrains-fswatcher` 1.2.0 (Rust),
`watcher` 0.0.1 (crates), `fsevents` 2.3.3 (macOS only, which exits gate 3 on its own).

### Candidates that exit screening

Each entry names the gate that ended it.
The skill's gates: category mismatch, hard-constraint failure, license or terms, unavailable source,
uninspectable high-trust code, unmappable native provenance, unknown build provenance,
unestablishable security boundary, no inspectable validation path, unsupported required platform.

#### systemd path units

- Discovery source: memory and repository precedent.
- Exits: hard constraint 3 (Linux only) and hard constraint 4 (not recursive).
  systemd v262 `src/core/path.c:38-98` (`path_spec_watch`) adds one inotify watch on the configured path
  and on its parents, with no walk into subfolders.

#### inotifywait (inotify-tools 4.26.291)

- Discovery source: memory and repository precedent, Arch (Q4), AUR (Q7), topic `inotify` (Q21), Homebrew (Q37).
- Exits: hard constraint 2 and 3.
  The man page documents `-r, --recursive` with new subdirectories watched,
  but the tool only prints events (a shell loop would run the command),
  and it is a Linux inotify and fanotify client.

#### incron

- Discovery source: memory, AUR `inotify` (Q7).
- Exits: hard constraint 3. `incrond` is the "inotify cron" daemon (`incrond(8)`);
  `incrontab(5)` documents recursive watching by default with a `recursive=false` opt-out,
  but the event source is the Linux kernel only.

#### fswatch 1.22.0

- Discovery source: memory, topic `inotify` (Q21), Homebrew expansion (Q37), winget (Q18).
- Exits: hard constraint 2. It is a monitor that prints changed paths;
  its README documents monitors for macOS FSEvents, BSD kqueue, Solaris, Linux inotify, Linux fanotify,
  Windows ReadDirectoryChangesW and a polling fallback, and "Recursive directory monitoring",
  but nothing that executes a command. A pipeline into `xargs` is custom glue, not the tool.

#### entr 5.x

- Discovery source: memory, topic `inotify` (Q21), web (Q30).
- Exits: hard constraints 3 and 4. Its README gives "Source Installation - BSD, Mac OS, and Linux";
  it reads the file list from stdin (`find src/ | entr -s ...`) and the README's own new-file recipe
  wraps it in `while sleep 0.1; do ls src/*.rb | entr -d make; done`.

#### lsyncd 2.3.1

- Discovery source: memory, web (Q29).
- Exits: category mismatch and hard constraint 3.
  It is a mirror daemon ("aggregates and combines events for a few seconds and then spawns one (or more)
  process(es) to synchronize the changes", by default rsync),
  and its README names an event monitor of "inotify or fsevents" only, with no Windows backend.

#### clsync 0.4.5

- Discovery source: AUR `inotify` (Q7), topic `inotify` (Q21).
- Exits: hard constraint 3 and category mismatch. AUR description: "File live sync daemon based on
  inotify/fanotify/kqueue (Linux, FreeBSD)".

#### spook (0.9.x git, crates 0.1.2)

- Discovery source: AUR `file watcher` (Q6).
- Exits: hard constraint 3. README: "known to build and work well on Linux and Mac OS X.
  It's also verified to work on FreeBSD."

#### FlowHook 0.1.2

- Discovery source: AUR `inotify` (Q7).
- Exits: hard constraint 3. AUR description: "A lightweight C++ file-watcher CLI using Linux inotify";
  the source now sits at `mnasies/FlowHook`.

#### fsniper 1.3.1 and iwatch 0.2.2

- Discovery source: AUR `inotify` (Q7).
- Exits: hard constraint 3. Both are inotify-based Linux daemons ("Daemon to run scripts based on changes
  in files monitored by inotify", "realtime filesystem monitoring program using inotify").

#### inotify-win (thekid)

- Discovery source: topic `filesystem-events` (Q20), winget (Q18 as `thekid/inotifywait`).
- Exits: hard constraints 3 and 4. It is "a port of the inotify-wait tool for Windows" (C#), Windows only,
  and inherits the print-events model.

#### Zippy

- Discovery source: GitHub topic `file-watcher` page 1 (Q19).
- Exits: hard constraint 3. README: "A lightweight Linux file watcher",
  and "Linux/POSIX process model using `fork`, `exec`, `pipe`, and `stat`".

#### stalk 1.0.7

- Discovery source: AUR `file watcher` (Q6).
- Exits: hard constraint 3. README lists "Windows (working progress)" and the release assets are Linux and macOS only.

#### fyr 1.2.0

- Discovery source: AUR `file watcher` (Q6), crates.io (crate metadata).
- Exits: unavailable source. crates.io names `https://github.com/opmr0/fyr` as the repository
  and that repository returns 404, so the artifact cannot be mapped to source.

#### hwatch 0.4.2, viddy 1.3.1, dwatch 0.4.1, sasqwatch 0.2.5

- Discovery source: mise (Q1), aqua (Q2), AUR `watch command` (Q8), Homebrew (Q16).
- Exits: category mismatch. They re-run a command on a timer and show or diff its output;
  none triggers on filesystem events (`hwatch`'s own description: "alternative watch command...
  hook and run commands" against command output).

#### cargo-watch 8.5.3

- Discovery source: aqua (Q2), crates.io (Q10), Homebrew expansion (Q37).
- Exits: category mismatch, and the repository is archived (`archived: true` via the GitHub API).
  It watches Cargo projects and advises downstream users to use `cargo watch` only in that setting;
  Watchexec's own README strikes it from the downstream list.

#### bazel-watcher 0.33.0

- Discovery source: mise (Q1), aqua (Q2).
- Exits: category mismatch. It watches Bazel workspaces and rebuilds Bazel targets.

#### air 1.67.4

- Discovery source: aqua expansion (Q33), AUR `live reload` (Q38), topic `file-watcher` (Q19).
- Exits: category mismatch. "Air is a live-reloading command line utility for developing Go applications",
  driven by `build.cmd`, `build.entrypoint` and Go-specific configuration.

#### npm-watch 0.13.0 and watch-cli 0.2.3

- Discovery source: npm `watch run command` (Q13).
- Exits: category mismatch. Both run `package.json` scripts (npm-watch through nodemon,
  watch-cli through gaze), so they need an npm project and cannot run `systemctl --user start ...`
  as the subject command.

#### @lerna-lite/watch 5.9.0

- Discovery source: npm `watch run command` page 2 (Q13).
- Exits: category mismatch. It is a monorepo task runner scoped to Lerna package graphs.

#### watch-restart (this repository's package)

- Discovery source: repository precedent.
- Exits: category mismatch. It restarts a long-running child process on change
  (the job `package/watch-restart/cli` was built for), not a command run to completion.

#### namastexlabs/linux-gdrive-sync

- Discovery source: web search (Q27).
- Exits: hard constraint 3. A Linux-only turnkey Google Drive sync built on rclone and a local watcher.

#### git-ghost

- Discovery source: aqua (Q2).
- Exits: category mismatch. "Synchronize your working directory efficiently to a remote place without
  committing the changes": a transport tool, not a command runner.

#### TechieGuy12/FileWatcher

- Discovery source: GitHub topic `file-watcher` page 1 (Q19).
- Exits: category mismatch. A C# desktop file and folder monitor that sends notifications and performs actions
  in its own UI, not a headless command runner.

#### fsevent_watch 0.2 and fsevents-tools 1.0.0

- Discovery source: Homebrew `fsevents` (Q15).
- Exits: hard constraints 2 and 3. macOS-only FSEvents clients that print events.

#### sharkdp/trigger

- Discovery source: GitHub topic `inotify` page 1 (Q21).
- Exits: hard constraint 3 and 4. A POSIX `sh` script that reruns a command when listed files change.

### Adjacent tool, not a candidate

`kde-inotify-survey` (Arch, Q4) reports how close a session is to its inotify watch limits.
It is not a watcher and is not screened;
it is recorded because the owner asked what a candidate does when the inotify watch limit runs out,
and this is the tool that answers the same question for the session as a whole.

## Evidence records

Every record below was read on 2026-11-25 between 17:22 and 17:41 -05:00.
No candidate was cloned, installed, built or run:
the skill's external execution gate and finalist validation are a later run,
so no record here comes from executing a candidate.

Common context for every probe:

- Host: CachyOS, Linux 7.3.5-2-cachyos, x86_64, 2 CPUs.
- Working directory: `/home/user/Monochromatic`, branch `main`.
- Evidence kinds: registry API responses, GitHub API metadata and release asset lists,
  project README and documentation pages, and repository source files fetched as text.
- Access date: 2026-11-25 for every URL quoted below.

### Provider probes

Each command ran once from the working directory above unless noted.

- `mise registry` (mise 2026.11.14 linux-x64): exit 0, 1079 rows;
  piped through `grep -Ei 'watch|inotify|fsevent|notify'` (4 rows, exit 0)
  and through `grep -Ei 'reload|hot.?reload|nodemon'` (0 rows, exit 1, an empty result, not an error).
- `curl -sSL https://raw.githubusercontent.com/aquaproj/aqua-registry/main/registry.yaml`: exit 0, 3,609,579 bytes;
  filtered in `python3` with `yaml.safe_load`: 2399 packages, 38 name or description hits, 2 reload hits.
- `curl -sS https://archlinux.org/packages/search/json/?q=<term>` for `file+watcher`, `inotify`, `watch+command`:
  exit 0 each, result counts 0, 7, 0.
- `curl -sS 'https://aur.archlinux.org/rpc/?v=5&type=search&by=name-desc&arg=<term>'` for the same three terms
  and for `live+reload`: exit 0 each, result counts 6, 26, 9, 16.
- `curl -sS -A 'watcher-vet-discovery' 'https://crates.io/api/v1/crates?q=<term>&per_page=50&page=N'`
  for `file watcher`, `watch command`, `run command on change`, `notify`, `live reload`:
  exit 0 each, provider totals 1677, 6166, 28395, 3792, 3712.
- `curl -sS 'https://registry.npmjs.org/-/v1/search?text=<term>&size=50&from=N'` for `file watcher cli`
  and `watch run command`: exit 0, totals 880251 and 892540.
- `curl -sSL https://formulae.brew.sh/api/formula.json`: exit 0, 8740 formulae.
- `gh api -X GET search/repositories -f q='topic:<t>' -f sort=stars -f per_page=100 -f page=N`:
  exit 0 for every topic page; totals 325, 50, 363, 68, 23, 1120, 267.
- `gh api -X GET search/code -f q='repo:microsoft/winget-pkgs <term>'`: exit 0, `total_count` 0 for `watcher`
  and 0 for the positive control `PowerToys`, so code search is unusable on that repository.
- `gh api repos/microsoft/winget-pkgs/git/trees/<ref>?recursive=1` for `0` and `a` to `z`,
  plus 665 per-publisher trees for `m`: exit 0 each; 14170 package directories, 75 name matches;
  every letter tree reported `truncated: false` except `m`, which the per-publisher walk replaced.
- `gh search code --limit 15` for the four `rclone bisync` pairings and `--limit 20` for `rclone bisync` alone:
  exit 0; counts 0, 1, 0, 1, and 20; the single paired hit is this report itself.
- `web_search` for the eleven class-3 and expansion queries: result counts 11, 4, 8, 12, 9, 16, 9, 11, 19, 13, 9.

### Gate 1: open source with inspectable source

- watchexec 2.8.0: Apache-2.0, `https://github.com/watchexec/watchexec` metadata field `license.spdx_id`;
  source repository carries the crates `crates/cli`, `crates/lib` and the manual; pass.
- Watchman 2026.11.23.00: MIT, same metadata field on `facebook/watchman`; source in the repository,
  documentation at `https://meta.github.io/watchman/docs/`; pass.
- chokidar-cli: MIT at `https://github.com/open-cli-tools/chokidar-cli`;
  published 3.0.0 declares a different repository (see the ledger entry); pass with that caveat.
- onchange 7.1.0: MIT, `https://registry.npmjs.org/onchange` `license` field and the repository's `LICENSE`; pass.
- watchfiles 1.3.0: MIT, `https://github.com/samuelcolvin/watchfiles` `license.spdx_id`; pass.
- modd 0.8: MIT, `https://github.com/cortesi/modd` `license.spdx_id`; pass.
- iwatcher 0.1.1: MIT, `https://github.com/CurroRodriguez/iwatcher`; pass.
- directory-watcher 3.2.1: MIT, `https://github.com/seungyeop-lee/directory-watcher`; pass.
- wgo 0.7.2: MIT, `https://github.com/bokwoon95/wgo`; pass.
- arelo 1.15.4: MIT, `https://github.com/makiuchi-d/arelo`; pass.
- run-when-changed 2.1.0: ISC, `https://registry.npmjs.org/run-when-changed`; pass.
- watch-run 1.2.5: MIT in `package.json` at `https://github.com/queckezz/watch-run`
  although the GitHub metadata reports no licence file; pass on the package manifest.
- watchlist 0.3.1: MIT, `https://github.com/lukeed/watchlist`; pass.
- funzzy 2.0.1: MIT, `https://github.com/cristianoliveira/funzzy` README badge and `LICENSE`; pass.
- watchexec-simple 0.2.0: MIT, `https://github.com/kurtbuilds/watchexec-simple`; pass.
- godemon: Unlicense, `https://github.com/bduffany/godemon` README badge; pass.
- watchngo: MIT, `https://github.com/fpeterschmitt/watchngo`; pass.
- quay 0.2.0: MIT in `watcher/Cargo.toml`; no `LICENSE` file at the repository root (probe returned 404 for
  `LICENSE`, `LICENSE.md` and `COPYING`); pass on the manifest field, carried as a caveat.
- Rushabh5000/watch-run and prem-the-dev/watch-run: MIT each, repository metadata; pass.
- Libraries: Chokidar MIT, `notify` CC0-1.0, `fsnotify` BSD-3-Clause, `watchdog` Apache-2.0,
  `@parcel/watcher` MIT, Node itself for `fs.watch`; pass each.

### Gate 2: runs an arbitrary command, here `systemctl --user start rclone-bisync-garage.service`

- watchexec: README "watches a path and runs a command whenever it detects modifications",
  `https://github.com/watchexec/watchexec` section Features; pass.
- Watchman: `https://meta.github.io/watchman/docs/cmd/trigger.html`
  "When files change that match the query expression, Watchman will spawn a process and pass information
  about the change"; pass.
- chokidar-cli: README `--command` option runs a shell command; `index.js` line 163 passes the command
  through `utils.run`; pass.
- onchange: README "Use glob patterns to watch file sets and run a command when anything is added, changed
  or deleted"; `src/index.ts:70` spawns the command; pass.
- watchfiles: `https://watchfiles.helpmanual.io/cli/` "Running and restarting a command", example
  `watchfiles 'pytest --lf'`; pass.
- modd: README section Blocks, prep commands and daemon commands; pass.
- iwatcher: README "A file system monitor that executes a specified command when monitored files change"; pass.
- directory-watcher: README "Each save stops the running server and the processes it started in the hook's
  process group"; hooks run arbitrary commands; pass.
- wgo: README Examples, `wgo -debounce 1s go build -o out main.go :: ./out`; pass.
- arelo: README Usage `arelo [OPTION]... -- COMMAND`; pass.
- run-when-changed: README `--exec <cmd>`; pass.
- watch-run: README `Usage: watch-run <cmd>`; pass.
- watchlist: README "run a command on any file system changes"; pass.
- funzzy: README `.watch.yaml` `jobs[].run`; pass.
- watchexec-simple: README `watchexec -- cargo run`; pass.
- godemon: README "godemon has been tested..." plus `godemon --quiet command`; pass.
- watchngo: README "Watch file or directory, run a command"; pass.
- quay: README "runs commands when files change" and `quay.yaml` command templates; pass.
- Rushabh5000/watch-run: README "Run any command when files change"; pass.
- prem-the-dev/watch-run: README "Run a shell command whenever files in a directory change"; pass.
- fswatch exits here: README lists monitors and "Recursive directory monitoring" but no execution option;
  fail, which is the exit reason.

### Gate 3: runs on Linux x86-64, macOS arm64 and Windows x64

- watchexec: README line 19 "Runs on OS X, Linux, and Windows"; release assets include
  `watchexec-2.8.0-x86_64-pc-windows-msvc.zip`, `...-aarch64-apple-darwin.tar.xz`,
  `...-x86_64-unknown-linux-gnu.deb`; pass.
- Watchman: `https://meta.github.io/watchman/docs/install.html` "Watchman is known to compile and pass its
  test suite on: Linux systems with `inotify`; macOS (uses `FSEvents`...); Windows 10 (64-bit) and up"; pass.
- chokidar-cli: Node plus Chokidar backends; README uses double quotes for cross-platform argument passing;
  `engines` requires Node 20 or newer at HEAD; pass.
- onchange: Node and Chokidar; README notes Windows users may need double quotes; pass.
- watchfiles: `https://watchfiles.helpmanual.io/` lists wheels for Windows x86_64 and aarch64,
  macOS x86_64 and arm64, and Linux x86_64, aarch64, i686, armv7l and musl; pass.
- modd: README "released for OSX, Windows, Linux, FreeBSD, NetBSD and OpenBSD"; assets include
  `modd-0.8-windows64.zip` and `modd-0.8-osx64.tgz`; pass.
- iwatcher: release `v0.1.1` assets include `iwatchr-v0.1.1-x86_64-pc-windows-msvc.zip`,
  `iwatchr-v0.1.1-aarch64-apple-darwin.tar.gz`, `iwatchr-v0.1.1-x86_64-unknown-linux-gnu.tar.gz`; pass.
- directory-watcher: release `v3.2.1` assets include `directory-watcher_Windows_x86_64.zip`,
  `directory-watcher_Darwin_arm64.tar.gz`, `directory-watcher_Linux_x86_64.tar.gz`; pass.
- wgo: assets `wgo-windows.exe`, `wgo-macos-apple-silicon`, `wgo-linux`; pass.
- arelo: assets `arelo_Windows_x86_64.zip`, `arelo_Darwin_arm64.tar.gz`, `arelo_Linux_x86_64.tar.gz`,
  and CI builds for Windows and macOS in `.github/workflows/test.yml`; pass.
- run-when-changed: README "(Nix & Win32)"; Node; pass.
- watch-run 1.2.5: Node, gaze; no platform exclusion documented; pass on Node's portability.
- watchlist: README "Runs on macOS, Linux, and Windows"; pass.
- funzzy: release assets cover darwin arm64 and x86_64 and linux arm64 and x86_64 only;
  `docs/INSTALLATION.md` has macOS and Linux sections only; no statement of exclusion;
  status pending targeted evidence.
- watchexec-simple: no release assets and no platform statement; built on `notify` whose backends cover
  the three platforms; status pending targeted evidence.
- godemon: README "has been tested on Mac and Linux. It has not yet been tested on Windows";
  no release assets; status pending targeted evidence.
- watchngo: README "Quick win with the latest release (Linux amd64 only)", Go source with `fsnotify`;
  status pending targeted evidence.
- quay: Rust with a `notify` dependency; no release assets and no Windows-specific evidence;
  status pending targeted evidence.
- Rushabh5000/watch-run: README badge "Windows | macOS | Linux"; recursive watching relies on Node's
  `fs.watch({recursive})`; status pending targeted evidence.
- prem-the-dev/watch-run: README "Recursive watching on Windows & macOS, per-directory watchers elsewhere"
  (matching Node's documented recursive support); status pending targeted evidence.
- fsevents 2.3.3, fsevent_watch, fsevents-tools, Zippy, spook, FlowHook, fsniper, iwatch, inotify-win,
  lsyncd, clsync, inotifywait, incron, systemd path units, entr, sharkdp/trigger, linux-gdrive-sync:
  each document a platform set that excludes Windows (or, for inotify-win, excludes everything else);
  exit on gate 3.

### Gate 4: watches every folder under the root recursively, including folders created later

- watchexec: README line 20 "Monitors current directory and all subdirectories for changes" and line 31
  repeats "and all subdirectories"; pass.
- Watchman: a watch root covers its tree; `https://meta.github.io/watchman/docs/cmd/trigger.html`
  evaluates queries over the watched root; pass.
- chokidar-cli: Chokidar README line 15 "Recursive watching everywhere, with an optional depth limit";
  glob patterns are matched against the tree; new directories are picked up by Chokidar's per-directory
  watching and `fs.watch({recursive})`; pass.
- onchange: Chokidar with glob patterns; pass.
- watchfiles: `https://watchfiles.helpmanual.io/cli/` "By default the CLI will watch the current directory and
  all subdirectories"; the Rust backend recurses with `recursive: true` by default in `run_process`; pass.
- modd: README "File watch patterns" and "escaped recursively" for include and exclude syntax; pass.
- iwatcher: README "Recursive: subdirectories are watched automatically"; pass.
- directory-watcher: targets are recursive watched paths with `watch.exclude`; pass.
- wgo: README `-root` and `-dir`/`-xdir` filters over the watched root tree; pass.
- arelo: README "The subdirectories are also monitored unless they match to the ignore patterns"; pass.
- run-when-changed: glob watch patterns over the tree; pass.
- watch-run 1.2.5: glob patterns via gaze; pass.
- watchlist: README "Recursively monitors all subdirectories"; pass.
- funzzy: `on.change` globs over the tree; pass.
- watchexec-simple, godemon, watchngo, quay, Rushabh5000/watch-run, prem-the-dev/watch-run:
  pass on the project's documented recursive behaviour; see the ledger for each statement.
- `fsnotify` (Go) exits this gate as a library: "Are subdirectories watched? No, you must add watches for
  any directory you want to watch"; recorded, not deleted.

### Round-3 requirement: one more run after a busy run ends

- watchexec: `doc/watchexec.1.md:539-554` `--on-busy-update`, default `do-nothing`,
  "You can also use 'queue' which will run the command once again when the current run has finished if any
  events occur while it's running"; native.
- Watchman: `https://meta.github.io/watchman/docs/cmd/trigger.html` "Watchman will only run a single instance
  of the trigger process at a time... When the process terminates, watchman will re-evaluate the trigger criteria
  based on the clock at the time the process was last spawned"; native.
- onchange: `src/index.ts:151-224`, excerpt `const running = new Set<Job>(); const queue = new Deque<Job>();`
  and `function dequeue() { ... if (running.size >= jobs) return; ... }` with `jobs` defaulting to 1;
  native, FIFO.
- modd: `modd.go:202` `for mod := range modchan` consumed one batch at a time, with the channel created at
  `modd.go:225` as `make(chan *moddwatch.Mod, 1024)`; native, buffered.
- directory-watcher: `docs/guide/running.md:50-52` "Collecting goes on while a hook runs. Each wait that ends
  puts its runs in the target's queue", with `option.queue.size` default 2048 at lines 93-94; native.
- iwatcher: README line 16 "Skip-if-running: if the previous command is still executing, the new trigger is
  silently dropped" and line 133 the atomic flag; not native, needs a wrapper.
- chokidar-cli: `index.js:163` comment "XXX: commands might be still run concurrently"; not native.
- watchfiles: `run_process` "Run a process and restart it upon file changes" with `sigint_timeout`;
  it interrupts, so a wrapper must add the queued run.
- wgo, arelo: restart the command on change per their READMEs; not native.
- Everyone else: no document states a busy-run mode; recorded as needing a wrapper until validation shows one.

### Evidence records for exits

Each line carries the claim, the gate it decides, the primary URL and the outcome.
All read on 2026-11-25; all statuses are fail for the named gate.

- systemd path units: not recursive, `src/core/path.c:38-98` in systemd v262; Linux-only unit type; gates 3 and 4.
- inotifywait: `https://man.archlinux.org/man/inotifywait.1.en` "Linux's inotify"; `-r` recurses; gate 3
  (and gate 2, since it prints events).
- incron: `https://man.archlinux.org/man/incrond.8.en` "inotify cron (incron) daemon"; gate 3.
- entr: `https://github.com/eradman/entr` "Source Installation - BSD, Mac OS, and Linux"; gate 3.
- lsyncd: `https://github.com/lsyncd/lsyncd` "event monitor interface (inotify or fsevents)"; gates 3 and category.
- clsync: `https://aur.archlinux.org/packages/clsync` "inotify/fanotify/kqueue (Linux, FreeBSD)"; gate 3.
- spook: `https://github.com/johnae/spook` "Linux and Mac OS X... FreeBSD"; gate 3.
- FlowHook: `https://github.com/mnasies/FlowHook` Linux inotify; gate 3.
- fsniper, iwatch: AUR descriptions name inotify; gate 3.
- inotify-win: `https://github.com/thekid/inotify-win` "port of the inotify-wait tool for Windows"; gate 3.
- Zippy: `https://github.com/Voyrox/Zippy` "Linux file watcher"; gate 3.
- stalk: `https://github.com/AppleGamer22/stalk` "Windows (working progress)"; assets Linux and macOS; gate 3.
- fyr: `https://github.com/opmr0/fyr` returns 404 for the repository crates.io names; gate: unavailable source.
- hwatch, viddy, dwatch, sasqwatch: periodic output watchers; gate: category.
- cargo-watch: `archived: true` in repository metadata; Cargo-project scope; gate: category.
- bazel-watcher: Bazel workspace scope; gate: category.
- air: `https://github.com/air-verse/air` "live-reloading command line utility for developing Go applications";
  gate: category.
- npm-watch, watch-cli: `package.json` script runners; gate: category.
- @lerna-lite/watch: Lerna workspace scope; gate: category.
- watch-restart: restarts a long-running child; gate: category.
- namastexlabs/linux-gdrive-sync: Linux-only rclone wrapper; gate 3.
- git-ghost: transport tool; gate: category.
- TechieGuy12/FileWatcher: C# desktop UI monitor; gate: category.
- fsevent_watch, fsevents-tools: macOS-only event printers; gates 2 and 3.
- sharkdp/trigger: POSIX `sh` script over listed files; gates 3 and 4.

## Terminal result

Saturated, with 20 ready-to-use screening survivors and 14 recorded library candidates.

Every required source class is saturated under the skill's page rule:

- Class 1 (registries): every scheduled registry query ran; the paginated registries (crates.io, npm)
  stopped after pages 3 and 4 added no screening survivor, the GitHub topics after pages 3 and 4,
  and the single-page providers reported exhaustion. winget was enumerated in full by git tree
  rather than through its unusable code search.
- Class 2 (repository host): five topics and five code searches completed under the same page rule.
- Class 3 (web): eleven queries returned their full result sets; the provider offers no cursor,
  so saturation is exhaustion of each returned set, with the blind spot that comparison pages
  (LibHunt, AlternativeTo, awesome-alternatives) were read only through the returned extracts.
- Class 4 (this repository): done before discovery, unchanged.

Named blind spots carried into targeted evidence:

- Registry description-substring queries miss tools whose wording differs (the Homebrew `file watcher`
  query missed `watchexec` until the expansion round).
- GitHub code search does not index `microsoft/winget-pkgs`, so winget needed the git tree walk.
- Star-sorted topic pages bury low-adoption tools; pages 3 and 4 of two topics were all 0-star projects.
- The plan record is explicit that no public project pairs a watcher with an rclone bisync run, so no
  prior-art integration exists to copy.

Survivors, ready-to-use tools:
watchexec 2.8.0; Watchman 2026.11.23.00; chokidar-cli; onchange 7.1.0; watchfiles 1.3.0; modd 0.8;
run-when-changed 2.1.0; watch-run 1.2.5; watchlist 0.3.1; iwatcher 0.1.1; directory-watcher 3.2.1;
wgo 0.7.2; arelo 1.15.4; funzzy 2.0.1; watchexec-simple 0.2.0; godemon; watchngo; quay 0.2.0;
Rushabh5000/watch-run; prem-the-dev/watch-run.

Platform evidence is pending for funzzy, watchexec-simple, godemon, watchngo, quay and the two
zero-adoption `watch-run` projects; for the rest, the target platforms are shown by release assets
or by a documented platform list.

Survivors, libraries (recorded; the existing-tools rule bars a custom program while ready-to-use tools pass):
Chokidar 6.0.0; Rust `notify` 9.0.0; Go `fsnotify` 1.10.1; Python `watchdog` 7.1.1; `@parcel/watcher` 2.6.0;
Node `fs.watch`; plus the smaller libraries listed in the ledger.

Not started here: targeted evidence, external execution gate, finalist validation, scoring,
recommendation, and re-keying the fingerprint for the round-3 requirement.
