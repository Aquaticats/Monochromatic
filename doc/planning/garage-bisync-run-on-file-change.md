# Run the Garage bisync when files change: design in progress

Status:
 agent-drafted during a grilling session that started 2026-11-25T17:04:57-05:00;
 not reviewed,
 not approved by the owner,
 not final.
Nothing in this file is decided until the owner answers the open questions it lists.

## Purpose

The owner asked for their Garage setup to start a manual sync whenever something changes in the relevant folders,
instead of waiting for the hourly timer.
"Manual sync" here means the run `systemctl --user start rclone-bisync-garage.service` starts,
which is the same run the hourly timer starts.

The setup this builds on is the runbook `doc/handover/garage-file-sync.md`.
The live unit on this PC differs from that runbook's Stage D (Linux):
it bisyncs `%h/Plain`, not `%h/Seafile/Plain`,
and it has the drop-in `rclone-bisync-garage.service.d/60-alert-on-failure.conf`
that starts `rclone-bisync-garage-alert@.service` on failure.
Any edit to the units takes its paths from the live files in `~/.config/systemd/user/`.

## Facts measured at 2026-11-25T17:04:57-05:00

Each fact names the command or source it came from.

- The local tree `~/Plain` holds `main.kdbx`,
  four old `main.kdbx.conflict1` to `main.kdbx.conflict4` copies,
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
  3.1 to 6.6 seconds wall clock,
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

## Watcher recommendation (2026-11-25T19:44:55-05:00)

The vet recommends directory-watcher 3.2.1;
see the vet report sections "Finalist validation" and "Recommendation".
Measured on Linux with the design's trigger shape:
one edit gives one sync about 2 s later (2000 ms batch wait),
an edit during a sync gives exactly one more,
a timer-started sync is waited out,
`touch` and `chmod` start nothing (so the settle pass's mtime stamp starts no extra run),
and `.partial` downloads start nothing until renamed.
Not adopted yet.

## Design consequences still to settle

- A start request made while the timer's run is going joins that run (measured, see the systemd container findings),
  so a watcher that queues its own runs still misses a change that lands during a timer run
  unless something starts one more run after it.
  Settled in round 4:
  a queuing watcher plus a trigger unit ordered after the sync unit.
- `TimeoutStartSec=3000` was sized for an hourly timer;
  with 5-minute polling and change triggers,
  a stalled run would hold every start request for up to 50 minutes.
  The value needs the longest legitimate run (a large upload) as its floor.
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
