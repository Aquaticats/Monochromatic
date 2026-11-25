# Vet report: recursive file watcher for the Garage bisync

## Report metadata

- Status:
  in progress,
  not reviewed,
  not approved by the owner,
  no recommendation yet.
- Lifecycle phase:
  discovery.
  Known candidates are listed in "Discovery ledger";
  broad discovery is delegated to a Pi subagent run,
  as the owner chose on 2026-11-25T17:19:41-05:00.
- Subject:
  recursive file watcher for garage bisync.
- Scope:
  pick the tool that watches `~/Plain` recursively
  and runs `systemctl --user start rclone-bisync-garage.service` after changes,
  on this CachyOS PC now and on the macOS and Windows clients later.
- Started:
  2026-11-25T17:19:41-05:00.
- Last updated:
  2026-11-25T17:19:41-05:00.
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
- whether it reports hidden files.

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

## Query log

Filled by the discovery run.

## Discovery ledger

Known before broad discovery,
from memory and repository precedent.
Screening below is from metadata or source already read;
the discovery run confirms or corrects each with an evidence record.

### watchexec

- Discovery source:
  repository precedent (`toolchain.lock.json`, version 2.8.0).
- Base category:
  inspectable open-source local executable.
- Overlays:
  multi-platform claim;
  prebuilt binary boundary.
- Screening:
  pending targeted evidence.

### Chokidar (a library run by a small Node program)

- Discovery source:
  repository precedent (`package/watch-restart/cli`, `doc/planning/kopia-source-watch-package.md`, catalog entry in `pnpm-workspace.yaml`).
- Base category:
  inspectable open-source library.
- Overlays:
  multi-platform claim.
- Screening:
  pending targeted evidence.
  The program around it is custom code,
  which the governing skill allows only when every ready-to-use tool fails a named hard constraint.

### watch-restart (this repository's package)

- Discovery source:
  repository precedent.
- Base category:
  inspectable open-source local executable.
- Overlays:
  multi-platform claim.
- Screening:
  pending;
  category fit is in question because it restarts a long-running child
  rather than running a command to completion.

### systemd path units

- Discovery source:
  memory and repository precedent.
- Screening:
  exits on two hard constraints.
  Not recursive:
  systemd v262 `src/core/path.c:38-98` (`path_spec_watch`) adds one inotify watch on the configured path and its parents.
  Linux only.

### inotifywait (inotify-tools)

- Discovery source:
  memory and repository precedent.
- Screening:
  expected to exit as Linux only (inotify is a Linux kernel interface);
  evidence record pending.

### incron

- Discovery source:
  memory.
- Screening:
  expected to exit as Linux only;
  evidence record pending.

### lsyncd

- Discovery source:
  memory.
- Screening:
  pending;
  platform claims to check.

### fswatch

- Discovery source:
  memory.
- Screening:
  pending;
  platform claims to check, Windows especially.

### entr

- Discovery source:
  memory.
- Screening:
  pending;
  recursion and Windows support to check.

### Watchman

- Discovery source:
  memory and repository precedent.
- Screening:
  pending.

### Node `fs.watch` with `recursive`

- Discovery source:
  memory and repository precedent.
- Screening:
  pending;
  custom code around it falls under the same rule as Chokidar's.

## Evidence records

Filled per gate and per candidate.

## Terminal result

None yet.
