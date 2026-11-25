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

## Owner answers, round 2 (2026-11-25T17:19:41-05:00)

- Portability is a hard requirement:
  the same watcher must run on Linux,
  macOS and Windows,
  so the later clients only add a service wrapper.
  Linux-only tools (systemd path units, `inotifywait`, `incron`) leave the candidate list.
- The broad discovery for the watcher vet is delegated to Pi as a subagent
  (the owner answered with `/pi-as-subagent`).
  The vet report holds the settled context and frozen query schedule the delegate starts from.
