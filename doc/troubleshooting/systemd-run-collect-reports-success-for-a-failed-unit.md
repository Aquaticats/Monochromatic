# `systemd-run --collect` makes `systemctl show` report `Result=success` for a unit that failed (systemd 262.1)

## Symptom

A gate or campaign launcher started a transient unit with `systemd-run --user --collect` and later asked
`systemctl --user show <unit> --property=Result,ExecMainStatus` whether it passed.
The answer was `Result=success` and `ExecMainStatus=0` whatever the command had done,
so a waiter that printed
"success" from that answer never meant the run had passed.
The cli-git gate launcher had this defect until it wrote the command's own exit status into the log (2026-11-03);
merges before that were also checked against the nextest summary in the log.

## Root cause

`--collect` sets `CollectMode=inactive-or-failed`,
so the unit is unloaded as soon as it ends,
even when it failed,
and its results are flushed at once.
The manual says so (`man systemd-run`,
`--collect`,
and `man systemd.unit`,
`CollectMode=`,
systemd 262.1-1.fc45):

```text
-G, --collect
    Unload the transient unit after it completed, even if it failed.
```

```text
Note that if this mode is used unit results (such as exit codes, exit signals, consumed resources, ...)
are flushed out immediately after the unit completed.
```

`systemctl show` on an unloaded unit prints the defaults of a unit that does not exist,
which read as a clean exit.

## Verification

Two transient units that each run `sh -c 'sleep 2; exit 3'`,
one with `--collect` and one without,
then
`systemctl --user show` four seconds later (probe run 2026-11-03 on Fedora 45,
systemd 262.1-1.fc45):

```sh
# file: probe (run from any directory)
systemd-run --user --unit=probe-collect --collect /usr/bin/sh -c 'sleep 2; exit 3'
systemd-run --user --unit=probe-keep /usr/bin/sh -c 'sleep 2; exit 3'
sleep 4
systemctl --user show probe-collect --property=LoadState,ActiveState,Result,ExecMainStatus
systemctl --user show probe-keep --property=ActiveState,Result,ExecMainStatus
systemctl --user reset-failed probe-keep
```

```text
LoadState=not-found
ActiveState=inactive
Result=success
ExecMainStatus=0
ActiveState=failed
Result=exit-code
ExecMainStatus=3
```

## Verified workarounds

- Have the unit's own command write its exit status where the waiter reads it,
  as the last line of the log.
  The cli-git gate launcher does this (`gate-exit-status=<n>`),
  and verdicts read that line,
  not the unit.
- Start the unit with `--wait` so `systemd-run` itself exits with the command's status
  (`systemd-run --user --wait --collect sh -c 'exit 3'` exited 3 in the same probe session;
  the manual only says
  `--wait` shows the exit code and status on exit).
- Leave out `--collect` and run `systemctl --user reset-failed <unit>` after reading the result.

## What does not work

- Reading `Result`,
  `ExecMainStatus` or `ActiveState` of a collected unit after it ended.
- Treating a `success` line from a waiter built on those properties as a pass.

## Upstream filing decision

Not filed.
The behavior is documented by systemd itself (`CollectMode=` and `--collect`),
so this is a usage hazard and not a defect.
