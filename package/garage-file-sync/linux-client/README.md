# @monochromatic-dev/garage-file-sync-linux-client

Runs the Garage bisync of `~/Plain` as soon as a file changes there,
instead of waiting for the next timer run,
and raises a desktop alert when a run fails and when it recovers.

The design and the measurements behind every setting are in
`doc/planning/garage-bisync-run-on-file-change.md`;
the rclone behaviour that needs the second pass is in
`doc/troubleshooting/rclone-bisync-use-server-modtime-phantom-conflicts.md`;
the setup this extends is `doc/handover/garage-file-sync.md`.

## What a change does

1.  `rclone-bisync-garage-watch.service` runs directory-watcher 3.2.1 over `~/Plain`.
2.  After ten seconds without a further change (a minute at most while changes keep
    arriving), the watcher's hook starts `rclone-bisync-garage-after-current.service`.
3.  That trigger unit is ordered `After=` the sync unit, so it waits for a run that is
    going and then starts one more. systemd merges every request that lands during a
    run into that one follow-up.
4.  The sync unit runs bisync twice, one pass after the other.

The five-minute timer keeps starting the sync unit directly,
so syncing continues if the watcher is not running,
and it is also how a change made on another device reaches this PC:
Garage cannot notify a client that the bucket changed.

## Every unit and what it does

- `systemd/rclone-bisync-garage.service`:
  the bisync runs, two passes per start, `TimeoutStartSec=3000` per pass
  (measured, so a stalled run can hold the queue for up to 6000 s),
  `StartLimitIntervalSec=0` so a burst of fast failures cannot lock out the timer.
- `systemd/rclone-bisync-garage.service.d/60-alert-on-failure.conf`:
  `OnFailure=` and `OnSuccess=` of the sync unit, both starting the alert template.
- `systemd/rclone-bisync-garage.timer`:
  every five minutes, `RandomizedDelaySec=30`, `AccuracySec=1s`, `Persistent=true`.
- `systemd/rclone-bisync-garage-after-current.service`:
  the trigger. One step: `systemctl --user start --no-block` the sync unit, ordered
  after it and with no requirement on it.
  A failed step alerts like a failed sync, and its next success closes that streak.
- `systemd/rclone-bisync-garage-watch.service`:
  directory-watcher, restarted thirty seconds after a crash, no start limit;
  its failure and its restart are alerted like the sync unit's.
- `systemd/rclone-bisync-garage-alert@.service`:
  the failure template, started by `OnFailure=`.
  The instance names the unit it speaks about,
  and it runs `src/alert.ts` with `--trigger=unit-failed`.
- `systemd/rclone-bisync-garage-recovered@.service`:
  the success template, started by `OnSuccess=`,
  which runs `src/alert.ts` with `--trigger=unit-succeeded`.
  The entry reads systemd's `MONITOR_*` variables and calls a run a recovery only when it
  exited with status 0;
  it stays silent when they say otherwise or are missing.
  The two templates must stay separate:
  systemd 262 passes no `MONITOR_*` variables to a handler that one unit names in both
  `OnFailure=` and `OnSuccess=`
  (its journal line: "multiple trigger source candidates for exit status propagation",
  measured on this PC's user manager at 2026-11-25T21:34:00-05:00),
  and with a shared handler every clean five-minute run was reported as a failure.
- `config/garage-plain.yml`:
  the directory-watcher config for `~/Plain`.

`src/` holds the alert entry and the install task:

- `src/alert.ts`:
  the entry the alert unit runs.
- `src/alert-run.ts`:
  decides whether this event is due an alert, shows it and records the streak.
- `src/alert-outcome.ts`:
  reads the command line and systemd's `MONITOR_*` variables.
- `src/alert-streak.ts`:
  the per-unit streak file and the one-hour quiet window.
- `src/alert-journal.ts`:
  the failed run's journal read and the causes it quotes.
- `src/alert-message.ts`:
  the notification text.
- `src/alert-notify.ts`:
  `notify-send`.
- `src/run-command.ts`:
  one asynchronous command with a timeout.
- `src/units.ts`:
  the unit names.
- `src/install-files.ts` and `src/install.ts`:
  the install and the drift check.
- `src/*.unit.test.ts`:
  the tests, which inject a disposable home and never touch `~/.local/state`.

## Alerts

- The first failure of a streak alerts and writes
  `~/.local/state/rclone-bisync-garage-alert/<unit>.json`.
- Further failures inside an hour are silent; one an hour or more later alerts again.
- A run that exits with status 0 closes the streak with one recovery alert and removes
  the file.
- A run stopped by hand is a stop, not a failure:
  stopping a running oneshot ends it in `Result=signal`,
  which `OnFailure=` reports, and rclone that catches SIGTERM exits with status 143
  (measured under systemd 262 on this PC at 2026-11-25T21:54:10-05:00).
  Both shapes are silent and leave an open streak open.
- The watcher's change hook reports a `systemctl` failure by running the entry directly
  under `node`, so the notice does not depend on the thing that failed.
- The watcher's target `onStartWatch` hook reports its recovery once, when its streak is open,
  and keeps the streak with its last alert time and a `recovered` mark instead of
  removing it. A watcher that keeps crashing and restarting therefore sends one failure
  and one recovery, not two notices every thirty seconds: a crash inside the hour is
  silent and keeps the mark, so the next start sends no second recovery, while a crash an
  hour or more after the last alert alerts again and clears the mark. The sync and
  trigger units still close their streak with the recovery notice.
  directory-watcher runs the hook only after the watches are in place,
  so a watcher that fails while adding them reports no recovery.
- A recovery notice uses normal urgency and an information icon;
  a failure uses critical urgency and an error icon.
- The state file is written only after `notify-send` exits 0, so a failed notification
  does not silence the next attempt.

Names that start with `.` or `_` are not watched by directory-watcher 3.2.1 and cannot
be re-included (`docs/guide/events.md` at `v3.2.1`); those files reach the bucket
through the five-minute timer. The owner accepted this at 2026-11-25T20:41:14-05:00.

## Install

The files in `systemd/` and `config/garage-plain.yml` are the sources;
the installed files under `~/.config` are regular copies of them,
never links into the checkout, because the user manager reads a unit while loading it
and a copy stays loadable whatever happens to the checkout.

```bash
mise run //package/garage-file-sync/linux-client:install
systemctl --user daemon-reload
systemctl --user enable --now rclone-bisync-garage.timer
systemctl --user enable --now rclone-bisync-garage-watch.service
```

`mise run //package/garage-file-sync/linux-client:check:installed` compares every
installed copy with its source and fails, naming the file, when a copy is missing,
cannot be read, differs, or is a link: a symbolic link into the checkout, or a hard link
that shares the source file's inode.
It also names every unit file under `~/.config/systemd/user` whose name starts with
`rclone-bisync-garage` and that this package does not ship,
and every extra file in a drop-in folder of a unit this package does ship,
because the user manager loads those too and a stale one keeps acting beside the current
ones.

The install writes into `$HOME`; `-- --home=<folder>` installs into another home folder,
which is how the tests run it.

## Go-live order

1.  Install the files and reload the user manager, with the commands under "Install".
2.  First the sync, the timer and the alert units:
    a timer run and a manual start must both work, and a forced failure must alert.
3.  Then the trigger and the watcher.
    Verify on the live manager:
    one edit gives one sync;
    an edit during a sync gives exactly one follow-up;
    a timer-started sync plus an edit gives one follow-up;
    an edit inside a dot-named folder starts nothing;
    stopping the watcher leaves a running sync to finish;
    a forced failure gives one alert, another an hour later, and one on recovery.

The order matters: the watcher's hook starts the trigger unit, which starts the sync
unit, so installing the watcher first would fail its hooks until the other units exist.

## What stays on the host

- `~/.local/state/rclone-bisync-garage-alert/<unit>.json`:
  the open alert streak of one unit. Removing the file is how to silence a repeat.
- `~/.config/systemd/user/` and `~/.config/directory-watcher/garage-plain.yml`:
  the installed copies, written by the install task.
