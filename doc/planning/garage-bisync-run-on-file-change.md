# Run the Garage bisync when files change: design in progress

Status:
 agent-drafted during a grilling session that started 2026-11-25T17:04:57-05:00;
 not reviewed,
 not approved by the owner,
 not final.
Nothing in this file is decided until the owner answers the open questions it lists.

## Current design, as settled at 2026-11-25T20:41:14-05:00

This section is the design as it stands after eight owner rounds and two reviews;
the sections after it are the record of how each part was decided and measured.
Status: confirmed by the owner at 2026-11-25T20:54:22-05:00 ("Yes, confirmed").
Implemented on this PC (`package/garage-file-sync/linux-client`, merged as `325f2f1ab6`) and live since 2026-11-25T22:01:09-05:00;
the Mac's agent has both passes and `--conflict-resolve newer` since 2026-11-25T21:15:56-05:00.
The first change-triggered sync on live data ran at 2026-11-25T23:19:37-05:00 (a deletion of five conflict copies), two passes, both successful.
Not final until the owner says so.
Decision record: `doc/decision/garage-file-sync-change-watcher.md`.

### Scope

This CachyOS PC now.
The Mac and Windows clients later,
each with its own watcher wrapper,
because the merging below relies on systemd (Sonnet review).
The Mac goes second (round 8): its agent first gains `--conflict-resolve newer` and the settle pass.

### Parts on this PC

1.  Watcher:
    directory-watcher 3.2.1 (`seungyeop-lee/directory-watcher` tag `v3.2.1`),
    installed through the repository toolchain (`toolchain.plan.jsonc`, then `mise run toolchain:update`),
    run by a new user unit, `rclone-bisync-garage-watch.service`:
    `Type=simple`,
    `ExecStart=|@directory-watcher --config-path %h/.config/directory-watcher/garage-plain.yml`
    (the binary resolved through the login shell as the sync unit resolves rclone;
    without `--config-path` it would look for `config.yml` in its working folder, `cmd/cmd.go:31-32` and `:60`),
    `Restart=on-failure` with `RestartSec=30s`
    (so the default limit of five starts in 10 s cannot trip, measured),
    `OnFailure=` the alert template,
    `WantedBy=default.target` (linger is on).
    Config, `~/.config/directory-watcher/garage-plain.yml`:
    one target, the absolute path of `~/Plain`,
    filled in from the home folder when file-enforcer writes the file
    (rule `HOM`: no literal home path in the repository source);
    `hookBatch.waitMillisecond` 10000 and `maxWaitMillisecond` 60000 (round 8);
    `exclude` `**/*.partial`, `**/*.part`, `**/*.tmp`, `**/*.crdownload`;
    run defaults.
    Hook (a shell string, run by `sh -c`):
    `systemctl --user start --no-block rclone-bisync-garage-after-current.service`,
    and when that call fails, the alert script run directly with `node`
    (not through systemd, which may be the thing that failed),
    deduplicated like every other alert.
    directory-watcher only logs a failed hook command and keeps watching,
    so without this nothing would notice.
    The target's `lifeCycle.onStartWatch` runs the alert script's recovery check for the watcher,
    which is how a watcher outage gets its recovery notice;
    it is the target's hook, not the global one, because directory-watcher runs it only after the watches are in place
    (`docs/guide/how-it-works.md` at `v3.2.1`; changed after the short review at 2026-11-25T21:44-05:00).
    Accepted exception (round 8):
    names starting with `.` or `_` (168 entries) are never watched,
    so they reach the bucket only through the 5-minute timer.
2.  Trigger unit, `rclone-bisync-garage-after-current.service`:
    `Type=oneshot`,
    `After=` the sync unit and no `Requires=` or `Wants=`,
    no `RemainAfterExit=`,
    one step:
    `systemctl --user start --no-block` of the sync unit.
    Requests during a sync merge into one follow-up (measured, system scope).
3.  Sync unit, `rclone-bisync-garage.service`, changed:
    a second `ExecStart=` line identical to the first (the settle pass);
    `StartLimitIntervalSec=0`, so fast failures cannot lock out the timer's start (adopted unless vetoed);
    `TimeoutStartSec=3000` kept (round 7), which applies per pass (measured), with its comment corrected;
    the existing `OnFailure=` drop-in kept, plus `OnSuccess=` for the recovery alert.
    A failed first pass skips the second (systemd runs no later `ExecStart=` after a failure),
    which is harmless because the settle pass only matters after a successful upload;
    the unit says so in a comment.
4.  Timer, `rclone-bisync-garage.timer`, changed:
    `OnCalendar=*:0/5`,
    `RandomizedDelaySec=30` and `AccuracySec=1s` (adopted unless vetoed;
    the default `AccuracySec=1min` would let runs slip up to 90 s),
    `Persistent=true` kept.
5.  Alerts:
    `~/.local/bin/rclone-bisync-garage-alert.ts` alerts on the first failure of a streak,
    again every hour while it lasts,
    and once on recovery (round 8).
    From the Sonnet and DeepSeek reviews:
    state per unit under `~/.local/state/rclone-bisync-garage-alert/`, which survives logout;
    state written only after `notify-send` succeeds;
    recovery only for a run that exited with status 0;
    a run stopped by hand reaches the failure handler (measured: `Result=signal`), which stays silent for it
    (`Result=signal` with `TERM`, or rclone's own exit status 143 on SIGTERM);
    its "next hourly run" text and its look-back window are corrected for 5-minute runs.
6.  Where the files live (round 7):
    sources in the repository,
    written to `~/.config` by file-enforcer's per-user rules;
    no rule kind does that today (`file-enforcer.schema.json`),
    so this needs a change to file-enforcer.
    The runbook's Stage D is rewritten from the result.
    Superseded at 2026-11-25T21:00:45-05:00 by the owner's build-time answer:
    a new package, `package/garage-file-sync/linux-client`, following the repository's precedent
    (`package/claude-cpu-cap/daemon/systemd/`):
    unit sources in `systemd/`, the watcher YAML, and the alert script as tested package source,
    installed as plain copies by a task, with a check task that fails when an installed copy drifts.
    The watcher config needs no templating: `WorkingDirectory=%h` and `path: Plain`
    (relative paths resolve against the launch folder, `docs/guide/configuration.md:143` at `v3.2.1`).
    The watcher binary is named by absolute path,
    because neither `/bin/sh -l` (what `|@` runs) nor the fish login shell finds the toolchain's `bin/`.
    The Mac's agent gets `--conflict-resolve newer` and its two passes through `/bin/sh -c` in the plist (owner, same time).

### Verified before go-live, on the real user manager

One edit gives one sync;
an edit during a sync gives exactly one follow-up;
a timer-started sync plus an edit gives one follow-up;
an edit inside a dot-named folder starts nothing (the accepted exception);
stopping the watcher leaves a running sync to finish;
a forced failure gives one alert, an hour later another, and one on recovery.
Then the decision record for directory-watcher is written.

### Accepted risks and open items

- Until the Mac's agent gains `--conflict-resolve newer`,
  a conflict there renames both copies and this PC mirrors that within 5 minutes (round 8: this PC first).
- A save that lands inside an upload or between the two passes still makes a `.conflictN` copy (measured);
  five `main.kdbx.conflict` copies exist now, and removing one needs deleting it on both sides in one window.
- The rclone root cause is drafted upstream and not posted (owner's decision).
- `TimeoutStartSec=3000` applies per pass (measured),
  so a stalled two-pass sync can hold every follow-up and timer start for up to 6000 s before it fails and alerts;
  accepted by the owner at 2026-11-25T20:54:22-05:00 ("Keep 3000, accept 100 min").
- With two writers, `--conflict-resolve newer` compares one side's authoring time with the other side's upload time,
  so it can pick the older content as the winner;
  the loser is kept as a `.conflictN` copy (bisync's default `--conflict-loser num`), so nothing is lost outright.
- One maintainer, unsigned releases;
  a watch-limit failure exits without the watcher's own notice, so the unit's failure alert covers it.

## Purpose

The owner asked for their Garage setup to start a manual sync whenever something changes in the relevant folders,
instead of waiting for the hourly timer.
"Manual sync" here means the run `systemctl --user start rclone-bisync-garage.service` starts,
which is the same run the hourly timer starts.

The setup this builds on is the runbook `doc/handover/garage-file-sync.md`.
The live unit on this PC differed from that runbook's Stage D (Linux):
it bisyncs `%h/Plain`, not `%h/Seafile/Plain` (Stage D corrected at 2026-11-25T20:19:01-05:00),
and it has the drop-in `rclone-bisync-garage.service.d/60-alert-on-failure.conf`
that starts `rclone-bisync-garage-alert@.service` on failure.
Any edit to the units takes its paths from the live files in `~/.config/systemd/user/`.

## Facts measured at 2026-11-25T17:04:57-05:00

Each fact names the command or source it came from.

- The local tree `~/Plain` holds `main.kdbx`,
  four old `main.kdbx.conflict1` to `main.kdbx.conflict4` copies
  (as of this check; the 18:01:21 hourly run added `main.kdbx.conflict5`, see "Review findings"),
  `Music/` and `Text/`:
  1454 folders and 10332 files in all (`find ~/Plain -type d`, `find ~/Plain -type f`).
- `~/Plain` sits on ZFS dataset `zpcachyos/ROOT/cos/home` (`findmnt --target ~/Plain`).
- Local churn:
  in the seven days before the check only `main.kdbx` had a newer mtime (saved 2026-11-25 16:48);
  no file under `Music/` or `Text/` did (`find -mtime -7` per top-level folder).
  Because the unit uses `--use-server-modtime`,
  a downloaded file's local mtime is its upload time,
  so this count also covers files that arrived from the bucket.
- Remote churn:
  objects in `garage:files/Plain` with a `LastModified` in the 14 days before the check were
  `main.kdbx` (2026-11-23 08:01),
  `main.kdbx.conflict3` and `main.kdbx.conflict4` (2026-11-18),
  and one `Music/Tenjin_Kotone/` file (2026-11-17)
  (`rclone lsl --use-server-modtime --max-age 14d`).
- Cost of a run that finds no change,
  from the unit's own accounting lines for the hourly runs on 2026-11-25:
  3.1 to 6.6 seconds wall clock
  (one pass; the settle pass makes each run two passes,
  and the 5-minute timer makes 288 timer runs a day instead of 24),
  0.77 to 1.29 seconds CPU,
  68 to 108 MB peak memory.
- The user journal keeps this unit's lines only from 2026-11-24T20:01:05-05:00,
  so it cannot show how often transfers happened before then.
- inotify limits on this PC:
  `fs.inotify.max_user_watches = 524288`,
  `fs.inotify.max_user_instances = 1024`,
  `fs.inotify.max_queued_events = 16384` (`sysctl`).
  A recursive watch of `~/Plain` needs one watch per folder.
- None of `watchexec`, `inotifywait`, `fswatch` or `entr` is installed (`command -v`).
  `systemd 262`, `rclone v1.76.0` and `keepassxc 2.8.0` are.
  `~/.config/keepassxc/keepassxc.ini` sets no save-related key, so KeePassXC saves with its defaults.
- Garage cannot tell a client that the bucket changed:
  its `doc/book/reference-manual/s3-compatibility.md` marks
  `GetBucketNotificationConfiguration` and `PutBucketNotificationConfiguration` as missing,
  and the same page says missing endpoints return `501 Not Implemented`.
  Changes made by other devices can only be found by listing the bucket.
- `systemd.path(5)` on this PC says a path unit watches through inotify,
  that `PathChanged=` fires when a file open for writing is closed,
  that `PathModified=` also fires on plain writes,
  that files whose names start with a dot are generally ignored,
  and that after the triggered service ends the paths are checked again.
  It does not say whether a path unit watches subfolders,
  or whether a change that lands while the service runs gets a later run;
  both are still to be tested.

## Interactions with known bisync behaviour

`doc/troubleshooting/rclone-bisync-use-server-modtime-phantom-conflicts.md` records that every file bisync uploads
reads back as changed on the next run,
and that the next run then stamps the local file's mtime.
With runs seconds apart instead of an hour apart,
two things need measuring on a disposable Garage before the design settles:

- whether bisync's own downloads and mtime stamps inside `~/Plain` start another run,
  and whether that chain ends;
- whether two quick saves of one file produce new `.conflictN` copies.

## Owner answers, round 1 (2026-11-25T17:15:31-05:00)

- Machines:
  every client should eventually start a sync on change,
  but only this CachyOS PC is in scope for this session.
  The other clients (macOS, Windows) are later work,
  so the design prefers parts that also run there.
- Folders:
  all of `~/Plain`,
  every folder and every folder created later.
- Remote edits:
  the timer polls every 5 minutes instead of hourly,
  because Garage cannot push change notices.
- Trigger:
  a change starts the existing `rclone-bisync-garage.service`,
  never a second rclone command,
  keeping one set of flags,
  the failure alert,
  and systemd's one-run-at-a-time guarantee for a unit.

## Owner answers, round 3 (2026-11-25T17:38:21-05:00)

- Mid-run edits:
  a hard requirement.
  When a change lands while a sync runs,
  one more sync starts after that run ends,
  by the watcher itself or by a small wrapper around the command it runs.
- Settle pass:
  every run of the unit runs bisync twice,
  the second pass straight after the first
  (on Linux a second `ExecStart=` line in `rclone-bisync-garage.service`),
  accepting one more bucket listing per run,
  including the 5-minute timer runs.
- Upstream:
  research whether rclone already tracks or fixed the baseline mtime behaviour,
  and draft a report for the owner to review;
  nothing is posted upstream without the owner's approval of the wording.

## Owner answers, round 4 (2026-11-25T17:59:11-05:00)

- Never interrupt a running sync:
  a hard requirement for the watcher on every client.
  Measured on this PC,
  killing a `systemctl start` client does not stop the sync,
  so on Linux this rules out nothing by itself;
  on macOS and Windows a restart-style watcher would kill rclone mid-run.
  Restart-only tools leave the vet,
  which also makes `watch-restart`'s exit consistent with the rest of the screening.
- Follow-up shape:
  a watcher that queues its own runs natively,
  calling a trigger unit ordered `After=` `rclone-bisync-garage.service`
  whose only step is `systemctl --user start rclone-bisync-garage.service`.
  The trigger unit covers changes during timer runs (measured);
  the watcher's own queue covers changes during runs it started.
  The 5-minute timer keeps starting the sync unit directly,
  so syncing goes on if the watcher dies.
  Native queueing becomes a hard requirement for the watcher.
- Upstream draft:
  before the owner reads it,
  its reproduction steps run exactly as written on a throwaway Garage.

## Owner answer on the upstream draft (2026-11-25T18:04:35-05:00)

"Make the changes but don't post it."
The reproduction fix (start from a seed file) is in
`doc/troubleshooting/rclone-bisync-use-server-modtime-phantom-conflicts.md`;
nothing is posted upstream.

## Repository precedent the owner pointed out (2026-11-25T18:16:16-05:00)

The owner:
"I believe we have already audited watchexec much earlier and found it unsatisfactory?"
and "We wrote a custom watchexec replacement too. A whole package!"
Both are on record:
`package-paused/desktop-daemon/editord/HANDOVER.custom-dev-server-watcher.md:502-507` rates `watchexec` "Fit: rejected"
and `:718-723` says "We cannot adopt a tool whose async runtime we cannot inspect";
`package/watch-restart/cli` is the replacement.
The vet had carried watchexec as a candidate and exited watch-restart as restart-only;
see the vet report section "Repository precedent missed by discovery".

## Owner answers, round 5 (2026-11-25T18:18:47-05:00)

- Watcher pick:
  "Vet 3p, no watchexec.
  Only going back to watch-restart if nothing works,
  because watch-restart has its own critical problems we'd need to fix if we use it."
  So watchexec leaves the vet,
  the third-party vet goes on with Watchman, onchange, modd and directory-watcher,
  and extending `package/watch-restart/cli` is the fallback only if every one of them fails.
- Skill fix:
  tighten source class 4 of `.claude/skills/choosing-technology/SKILL.md`
  (search every candidate's name across `doc/` and package docs,
  record every verdict found as counterevidence,
  and enter a repository package that owns the responsibility as a candidate to extend);
  the owner sees the diff before it is committed.

## What is on record against the watch-restart fallback (2026-11-25T18:18:47-05:00)

The owner gave no list with "its own critical problems";
these are the problems the repository records,
which the fallback would have to clear first:

- open issue #4790:
  Ctrl+C inside a restart's stop-timeout grace abandons the SIGKILL escalation and can orphan the detached child;
- open issue #4793:
  the publish-ready package ships a TypeScript-source bin with no `engines` constraint;
- open issues #4787, #4788, #4789, #4791, #6038 and #4795:
  README and doc claims that disagree with the code (missing exports, event names, signals, the oversized-file sentinel);
- open issue #4796:
  the `start()` and `restart()` re-entry guards for state `stopping` have no tests;
- `package/watch-restart/cli/HANDOVER.implementation-state.md`:
  interactive verification cases 2, 3, 4 and 6 are still pending
  (one restart per real edit, no restart on byte-identical or metadata-only writes, clean shutdown on SIGTERM),
  and the atomic-save watcher test is skipped after failing about 1 run in 5 in isolation.

## The Mac client, probed read-only over `ssh m1` (2026-11-25T18:50:01-05:00)

- macOS 27.2 on arm64;
  `kern.maxfiles: 122880`,
  `kern.maxfilesperproc: 61440`,
  `launchctl limit maxfiles` soft 256 and hard unlimited,
  `ulimit -n` 256.
  directory-watcher's kqueue backend needs about 11786 descriptors for a tree the size of this PC's `~/Plain`:
  under the per-process ceiling,
  far over the default soft limit,
  so a LaunchAgent for it would have to raise its own soft limit.
- `~/Plain` exists there,
  and `launchctl list` shows `cat.aquati.rclone-bisync-garage` loaded with last exit status 7,
  the status bisync exits with when it aborts.
  Read at 2026-11-25T18:51:49-05:00 on the owner's go-ahead:
  every run since the agent's log began (2026-09-14 16:00, 682 runs) aborted
  with "cannot find prior Path1 or Path2 listings",
  none succeeded,
  and `~/Library/Caches/rclone/bisync/` is empty;
  recorded in `doc/handover/garage-file-sync.md` "Client check (macOS, `ssh m1`)".

## Owner answers, round 6 (2026-11-25T18:51:49-05:00)

- onchange exits: its per-event queue meets the round 3 wording but not its purpose.
- modd exits on the provenance gate; building it from source was declined.
- The Mac's failing agent was read, not changed (see the Mac client section).
- directory-watcher 3.2.1 is the only finalist; runtime validation comes next.

## Trigger call mode measured (2026-11-25T19:48:30-05:00)

Under systemd 262 in the `docker.io/cachyos/cachyos:latest` container,
with an 8-second stand-in sync and a trigger oneshot ordered `After=` it whose step is
`systemctl start --no-block` of the sync,
every request sent as `systemctl start --no-block` of the trigger:

- one request with no sync running started one sync;
- five more requests during that sync, a second apart, gave exactly one follow-up sync,
  started 54 ms after the first ended;
  a request after it went idle started one more;
- a sync started directly (the timer's path) plus two requests during it gave exactly one follow-up.

So with non-blocking requests systemd merges every request that lands during a sync into one pending follow-up,
and the watcher's hook returns at once.
directory-watcher's own queue would otherwise add one run per batch window during a long sync
("runs from different waits are never merged", `docs/guide/running.md:51` at `v3.2.1`),
and past `queue.size` (2048) the watcher stops with `dispatch queue full` (`:93-95`);
bisync's own downloads rename `.partial` files to final names inside the watched tree,
so a long download run would feed that queue.

## Watcher recommendation (2026-11-25T19:44:55-05:00)

The vet recommends directory-watcher 3.2.1;
see the vet report sections "Finalist validation" and "Recommendation".
Measured on Linux with the design's trigger shape:
one edit gives one sync about 2 s later (2000 ms batch wait),
an edit during a sync gives exactly one more,
a timer-started sync is waited out,
`touch` and `chmod` start nothing (so the settle pass's mtime stamp starts no extra run),
and `.partial` downloads start nothing until renamed.
Adopted in round 7.

## Owner answers, round 7 (2026-11-25T19:51:58-05:00)

- Watcher:
  adopt directory-watcher 3.2.1,
  installed through the repository toolchain (`toolchain.plan.jsonc`, then `mise run toolchain:update`);
  its decision record is written when the work is implemented.
- Call mode:
  the watcher's hook runs `systemctl --user start --no-block` of the trigger unit,
  and the trigger runs `systemctl --user start --no-block` of the sync unit,
  so requests during a sync merge into one follow-up (measured, "Trigger call mode measured").
- Quiet period:
  `hookBatch.waitMillisecond` 2000.
- Alerts:
  one desktop alert on the first failure of a streak and one on the next success,
  by changing `~/.local/bin/rclone-bisync-garage-alert.ts` to remember the last state;
  the watcher unit gets the same `OnFailure=` alert and `Restart=on-failure`.
- Timeout:
  `TimeoutStartSec=3000` stays on the sync unit.
- Config home:
  the unit files and the watcher config have their sources in the repository,
  written to `~/.config` by file-enforcer's per-user rules;
  if no rule kind fits, that is a change to file-enforcer,
  or the runbook stays the source and the owner is told.
- The Mac:
  compare its `~/Plain` with the bucket read-only first (done; see "The Mac recovered"),
  then the owner decides on a `--resync`.

## Names the watcher would skip by default (2026-11-25T19:51:58-05:00)

directory-watcher excludes names starting with `.` or `_` unless re-included
(vet record GW-19, `docs/guide/configuration.md` at `v3.2.1`).
`find ~/Plain -mindepth 1 \( -name '.*' -o -name '_*' \)` counted 168 such entries,
among them app data folders whose edits matter (a notes app's settings folder, an e-reader's position folder).
Round 1's answer "every folder, including folders created later" decides it:
the watcher config re-includes these names.
Adopted unless the owner vetoes.
(Superseded: directory-watcher cannot re-include them; see "Review findings".)

## The Mac recovered (2026-11-25T20:04:52-05:00)

The owner chose the bucket as the golden copy;
the Mac's 214 Mac-only files were moved aside and its 12 differing files copied aside,
then a `--resync` and a normal agent run both succeeded
(`doc/handover/garage-file-sync.md`, "Client check (macOS, `ssh m1`)").
The Mac writes to the bucket again,
so the bucket now has two writers,
and its agent still lacks `--conflict-resolve newer` and the settle pass.

## Review findings, DeepSeek cross-vendor review (2026-11-25T20:18:41-05:00)

Pi on DeepSeek (`deepseek-v4.1-flash-1017`) read every file of the design and the live units,
read-only, and returned fifteen findings.
Checked by the coordinating session against the sources before acting:

- Verified:
  directory-watcher 3.2.1 cannot watch names starting with `.` or `_`, and `!` cannot re-include them
  (`docs/guide/events.md:82` and `:108` at `v3.2.1`),
  so "the watcher config re-includes these names" (section "Names the watcher would skip by default") cannot be done,
  and the 168 such entries under `~/Plain` would reach the bucket only through the 5-minute timer.
  This fails round 1's "every folder" answer and the vet's hard constraint on recursion;
  the owner decides.
- Verified:
  the sync unit carries systemd's default start limit
  (`systemctl --user show`: `StartLimitIntervalUSec=10s`, `StartLimitBurst=5`),
  which change triggers can reach while every run fails quickly,
  and then the timer's start is refused too.
- Verified:
  the 18:01:21 hourly run on this PC created `main.kdbx.conflict5`;
  its journal shows Path1 at 17:00:35.814 (the previous run's upload) against a local save at 17:25:22,
  the pile-up the throwaway measurements predicted, on the live setup.
- Verified:
  `file-enforcer.schema.json` documents every committed output as "Repo-relative output path",
  and its only per-user rule is the IDE language-server policy,
  so no rule kind writes `~/.config` today.
- Taken as written (reasoning, no measurement needed):
  two writers make `--conflict-resolve newer` compare one side's authoring time with the other side's upload time;
  the Mac's agent still resolves conflicts with the default `none`;
  other growing-file names (`*.part`, `*.tmp`, `*.crdownload`) are not excluded;
  a remembered "failing" state can hide the only alert of a failure that began while logged out;
  a failing hook command (trigger missing, `systemctl --user` unusable) raises no alert;
  the user-manager path with `--no-block` and the watcher together was never measured;
  the watcher unit and the 5-minute timer are not yet specified;
  and several lines of this record and the vet report were stale (fixed in the same change).

## Review findings, Claude Sonnet gate review (2026-11-25T20:33:56-05:00)

Its verdict: not confirmable yet.
Beyond the DeepSeek findings it adds,
each checked or labelled:

- The candidate exits of rounds 4 to 6 rest on the blocking shape.
  With round 7's `--no-block` hook on Linux,
  the watcher's command is a millisecond `systemctl` call and systemd merges the requests,
  so "queues natively" and "never interrupts a running command" no longer separate candidates on this PC;
  they still matter on macOS and Windows, which have no such merge.
  onchange's record shows it watches dot names (`OC-E18`);
  Watchman kills only a `systemctl` child that has already exited.
  Reading of the records, not measured.
- Measured at 2026-11-25T20:33:56-05:00 in a systemd 262 container:
  a unit with `Restart=on-failure` and `RestartSec=1` that always fails
  ran its `OnFailure=` unit on every crash (5 of 5)
  and then stayed failed with `Result=start-limit-hit` after five starts in 10 s.
  So the watcher unit needs a `RestartSec` long enough that the default start limit cannot trip,
  and the alert needs per-unit deduplication.
- Measured, same container: `TimeoutStartSec` applies per `ExecStart=` pass (recorded under "Design consequences").
- KeePassXC 2.8.0 defaults `AutoSaveAfterEveryChange` to true (`src/core/Config.cpp:61`, source reading),
  so an editing session is a burst of saves;
  with a 2 s quiet period each pause starts a sync,
  and a save that lands inside an upload pass still makes a conflict copy (measured on the throwaway).
- The recovery alert needs `OnSuccess=` (systemd 249 and later), not only a script change;
  the remembered state has to be per unit and written only after `notify-send` succeeds.
- Implementation details the record leaves open:
  the trigger unit must not set `RemainAfterExit=yes`
  and must order itself with `After=` only;
  absolute or login-shell-resolved paths for `systemctl` and the watcher binary;
  the 5-minute timer's `OnCalendar`, `RandomizedDelaySec` and `Persistent`;
  whether `~` expands in the watcher YAML;
  `WantedBy=default.target` (linger is on).
- Round 2's "later clients only add a service wrapper" does not hold:
  the Linux shape relies on systemd's job merging.

## Proof on this PC's user manager (2026-11-25T21:57:27-05:00)

Test-named copies of every unit in `package/garage-file-sync/linux-client` (prefix `gfs-proof-`),
on the real user manager,
against a throwaway Garage in podman on `127.0.0.1:39000`, a scratch rclone config and a scratch tree with `.` and `_` names.
The proof sync unit had one added line, `ExecStartPre=/usr/bin/sleep 20`, so an edit could land inside a run;
the alert units ran the package's entry with a scratch home and a stand-in `notify-send` that only logged.
Logs: `~/temp/agent/garage-file-sync-unit-stack-proof-2026-11-25/keep/`.

- Every unit loaded (`LoadState=loaded`) with no warning in the user journal.
- One edit gave one sync run.
- An edit, then three edits during the run that followed, gave exactly two runs.
- A sync started by hand (as the timer starts it) plus an edit during it gave two runs.
- Edits under `.obsidian/`, `_pages/` and a new `.newdot/` folder started no run;
  the next run uploaded them with everything else.
- Stopping the watcher during a run left the run to finish with `Result=success`.
- The timer's next elapse was a five-minute boundary plus 4 s, with `AccuracyUSec=1s` and `Persistent=yes`.
- A failure (filter file missing) sent one critical notice and wrote the streak;
  a second failure inside the hour sent none;
  a failure with the streak backdated by two hours sent another;
  the next clean run sent one normal-urgency "recovered" and removed the streak.
- A failed notification wrote no state, and its handler exited 1.
- With the trigger unit removed, the watcher's hook alerted directly through `node`.
- The watcher killed while its folder was missing kept failing on restart and sent no recovery;
  with the folder back, the next restart sent one recovery and cleared the streak.

Two defects the proof found, both fixed on the branch before go-live:

- systemd 262 passes no `MONITOR_*` variables to a handler that one unit names in both `OnFailure=` and `OnSuccess=`
  (journal: "multiple trigger source candidates for exit status propagation (gfs-proof-sync.service, gfs-proof-sync.service), skipping"),
  so every clean run was reported as a failure.
  The success handler is now its own template, `rclone-bisync-garage-recovered@.service`.
  The DeepSeek re-review had read the source as fine; only running it showed the defect.
- Stopping a running oneshot by hand ends it in `Result=signal` (killed by signal 15), a failure,
  so `OnFailure=` handles it; the design's line that `OnSuccess=` fires for a hand stop was wrong for this unit.

The short DeepSeek review of the branch found that the watcher's global start hook would report a false recovery every 30 s
when adding watches fails, and that the abort notice demanded a `--resync` the troubleshooting doc says is not needed;
both are fixed (the hook is now the target's `onStartWatch`, which directory-watcher runs only after watching starts).

## Coverage and review fixes merged (2026-11-25T23:10:57-05:00)

The coverage branch and its review fixes merged into `main` as `59de5b8c25`;
the installed copies still match the package.
A short DeepSeek review of the fixes (at `a78419f961`) reverted each fix in a throwaway copy and saw its test fail for the named reason.
It found, and the coordinating session left open:

- A `hook-failed` alert's streak closes only when the watcher restarts, so a hook fault that is fixed while the watcher keeps running is never announced as fixed.
- A hand stop that systemd escalates to SIGKILL after `TimeoutStopSec` still raises a failure notice; only SIGTERM stops are silent.
- Untested branches: an unpaired `{` in a listing path, a state file with `recovered: false`, the "stopping it failed" branch of `stopCommand`, and `src/alert.ts` as a command.

## Review of the test-coverage branch (2026-11-25T22:39:01-05:00)

A short DeepSeek review of `test/garage-file-sync-linux-client-coverage` (at `223ad05ef4`) found, and the coordinating session weighed:

- Fixing (branch `fix/garage-file-sync-linux-client-alert-review-findings`):
  the runner can wait forever on a descendant that keeps the pipes open after the command exits;
  a hand stop raises a critical failure notice;
  a watcher that keeps exiting after it started watching sends a failure and a recovery about every 30 s;
  the backup-listing check accepts another local folder's listings for the same remote
  (the proof fixture had left such listings in `~/.cache/rclone/bisync`; they were deleted at once);
  two comments that do not describe the code, and an untested state-file branch.
- Left open, recorded here:
  the drift check does not scan instance drop-in folders (`rclone-bisync-garage-alert@<unit>.service.d/`), which `systemctl --user edit` on a running handler creates;
  a leftover `<target>.new` from an interrupted install is reported as a unit the manager loads, which it does not;
  the alert entry has no test as a command, and a few error branches of `alert-run.ts` and `install-files.ts` have none.

## Owner answers, round 8 (2026-11-25T20:41:14-05:00)

- Watcher: keep directory-watcher 3.2.1;
  names starting with `.` or `_` sync only by the 5-minute timer, an accepted exception to round 1.
- Order: this PC goes live first; the Mac's agent gets `--conflict-resolve newer` and the settle pass afterwards.
- Alerts: first failure of a streak, then hourly while it lasts, plus one on recovery.
- Quiet period: 10 s, capped at 60 s.

## Design consequences still to settle

- A start request made while the timer's run is going joins that run (measured, see the systemd container findings),
  so a watcher that queues its own runs still misses a change that lands during a timer run
  unless something starts one more run after it.
  Settled in round 4:
  a queuing watcher plus a trigger unit ordered after the sync unit.
- `TimeoutStartSec`: settled in round 7 (kept at 3000 s).
  Measured at 2026-11-25T20:33:56-05:00 (systemd 262 container, two 3-second `ExecStart=` passes under `TimeoutStartSec=5`, both finished):
  the timeout applies to each pass,
  so a stalled two-pass sync can hold every follow-up for up to 6000 s.
  An earlier line here said the opposite, from a review claim; it was wrong.
- The reasoning in `doc/troubleshooting/rclone-bisync-use-server-modtime-phantom-conflicts.md` that `--conflict-resolve newer`
  cannot let a stale remote copy win assumes one writer;
  it needs rechecking once other clients write to the bucket on change.

## Facts gathered for round 2

- systemd path units do not recurse:
  `path_spec_watch` in systemd v262 `src/core/path.c:38-98` adds one inotify watch on the configured path
  and on its parent directories,
  with no walk into subfolders,
  so a `PathChanged=` on `~/Plain` sees only entries directly inside it.
  A path unit cannot meet the round 1 answer "every folder,
  including folders created later".
- Repository precedent for the same shape of problem:
  `doc/planning/kopia-source-watch-package.md` (design plan only, never built)
  plans one user daemon that watches backup sources recursively with Chokidar,
  debounces,
  starts a Kopia snapshot,
  and keeps a periodic timer as the safety net;
  it rejects `systemd.path` (not recursive) and a raw `inotifywait` loop (custom debounce and parsing),
  and lists `watchexec` as an alternative.
- `package/watch-restart/cli` is a Chokidar-based package that replaced `watchexec` in a dev loop
  because of a SIGINT hang in `watchexec`'s `-j` filter mode
  and signals not reaching grandchildren (`doc/troubleshooting/mise-watch.md`);
  it restarts a long-running child,
  which is a different job from running a sync to completion.
- `watchexec` 2.8.0 is locked in `toolchain.lock.json`
  and installed at `~/.local/share/monochromatic-dev/toolchain/bin/watchexec`,
  outside this shell's `PATH`.
- No vet report for a file watcher exists under `doc/audit/`.

## Measured on a throwaway Garage (2026-11-25T17:30:22-05:00)

Garage v2.3.0 and the host's `rclone v1.76.0` ran in a podman pod,
with the live unit's bisync flags,
on a small tree bind-mounted from `~/temp/agent` (the same ZFS home dataset as `~/Plain`),
and `inotifywait --monitor --recursive` recording every event in the tree during each step.
Logs and per-step summaries:
`~/temp/agent/garage-bisync-on-change-grilling-2026-11-25/throwaway-garage-test/keep/`.
The pod was removed afterwards.

- A run that finds no change writes nothing in the tree:
  only folder open, read and close events.
- A run that uploads a local edit writes nothing in the tree either.
- The run after an upload stamps the uploaded file's mtime ("Updated modification time in destination"),
  which is one `ATTRIB` event;
  the run after that writes nothing.
  So a watcher that reports `ATTRIB` starts one extra run per upload,
  and that run ends the chain.
- Two saves of `main.kdbx` with one run between them,
  and no other run before the second save's run,
  left `main.kdbx.conflict1` on both sides holding the first save's bytes,
  while `main.kdbx` kept the newer bytes.
  This reproduces the pile-up the troubleshooting doc predicts,
  now that `--conflict-resolve newer` keeps the loser under a `.conflictN` name.
- A second bisync pass right after an uploading run (a "settle pass") prevents it:
  two save cycles,
  each followed by a run and a settle pass,
  added no new `.conflictN` file.
- A save of `main.kdbx` made while a run was uploading a different file was not seen by that run;
  the next run uploaded it without a conflict,
  because `main.kdbx` carried no phantom then.
  So a change that lands mid-run needs one more run after the current one ends.
- A download from the bucket writes `<name>.<8 hex digits>.partial`
  (create, modify, close, attribute change),
  then renames it to the final name;
  the next run writes nothing.
  Downloaded objects get no phantom.
- A save done as "write a temporary file, rename it over the original" shows a create and writes on the temporary name,
  then the rename.

Second round on the same pod (2026-11-25T17:34:03-05:00),
for the cases the settle pass might not cover:

- Deletion,
  positive control:
  a new file uploaded by one run,
  then deleted locally before any other run,
  came back from the bucket on the next run,
  as `doc/troubleshooting/rclone-bisync-use-server-modtime-phantom-conflicts.md` records.
- Deletion with a settle pass between the upload and the delete:
  the next run deleted the object,
  and the file was gone on both sides.
- A second save of the same file while its upload was running:
  the next run raised a conflict and left `.conflict1` on both sides;
  the settle pass after it did not help.
- A second save between the upload pass and the settle pass (53 ms apart in the log):
  the settle pass itself raised the conflict and left `.conflict1` on both sides.

What `systemctl start` does while the unit runs,
measured on a oneshot unit that sleeps 8 seconds,
under systemd as PID 1 in the `docker.io/cachyos/cachyos:latest` container:

- a second `systemctl start` sent 2 seconds into the run joined that run:
  the unit ran once,
  and both callers returned when it ended,
  each with exit status 0;
- a `systemctl start --no-block` sent during the run also joined it;
- a start sent after the run ended started a new run.

So a start request made while any run is going (the timer's or the watcher's) is absorbed by that run,
and a change that lands mid-run needs a new start request after the run ends.

Measured at 2026-11-25T17:57:35-05:00 under systemd 262 (`262.2-1-arch`) in the same container image,
with an 8-second stand-in oneshot for the sync
and a trigger oneshot ordered `After=` it whose only step is `systemctl start` of the stand-in:

- a trigger requested 2 seconds into a running sync waited for that sync to end,
  then started a fresh one:
  two syncs ran,
  and the trigger request returned after the second;
- a trigger requested with no sync running started one sync;
- a second trigger requested while the first trigger was still running joined it:
  one sync ran,
  and both requests returned together.
- killing a blocking `systemctl start` client with SIGTERM (it exited 143) left the run going (`activating`),
  and the run ended normally.

So on Linux a trigger unit ordered after the sync unit covers a change that lands during a timer run without custom code,
a watcher that queues its own runs covers a change that lands during a run it started,
and no watcher can kill a sync it starts through `systemctl`.

So a settle pass narrows the window in which a save makes a conflict copy
from "until the next run" to the length of the upload pass plus the gap before the settle pass,
and it closes the deleted-file-comes-back case;
it does not remove the window.
The root cause stays the rclone behaviour the troubleshooting doc names:
after an upload,
bisync records the source mtime in its baseline but compares the object's `LastModified` next time.

## Owner answers, round 2 (2026-11-25T17:19:41-05:00)

- Portability is a hard requirement:
  the same watcher must run on Linux,
  macOS and Windows,
  so the later clients only add a service wrapper.
  Linux-only tools (systemd path units, `inotifywait`, `incron`) leave the candidate list.
- The broad discovery for the watcher vet is delegated to Pi as a subagent
  (the owner answered with `/pi-as-subagent`).
  The vet report holds the settled context and frozen query schedule the delegate starts from.
