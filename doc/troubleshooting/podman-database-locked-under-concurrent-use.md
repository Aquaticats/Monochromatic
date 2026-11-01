# Podman answers `database is locked` while other work uses it

## Symptom

On 2026-10-05,
on a host where several other sessions were running `podman build`,
`podman run` and `podman rm --force` at once and the load average was
between 40 and 100,
`podman inspect` failed:

```text
# podman inspect --format {{.State.Pid}} music-player-settings-pane-fold
Error: beginning transaction: database is locked
```

`podman ps --all` either printed the same message or took longer than two
minutes to answer.
The container named in the command was running throughout.

## What it cost

An emulator visit restores the guest's settings and stops it through a
script that first asked `podman inspect` for the container's process id.
The script died on that message,
so the guest was left running with changed settings until it was restored
by hand.

## Cause

Not established.
The message is Podman's own.
What holds the database,
and whether the concurrent builds and removals were the holders,
was not read from Podman's source or measured.

## Workaround that was verified

Do not make cleanup depend on Podman answering.
The first process of a container started with `podman run --init` is
`/run/podman-init -- <command>`,
and its command line names what the container runs.
`pgrep --full` with a pattern that includes the distinguishing arguments
(for the emulator,
the AVD name and the `-ports` pair) found exactly one process,
and `nsenter --target <pid> --user --net` ran the ADB client inside the
container's namespaces without calling Podman.
The restore script now tries `podman inspect` first,
falls back to that lookup when Podman fails,
and records which way it found the process.

## What does not work

- Trusting one successful answer:
  the visit's own state query had succeeded seconds before the restore
  script's query failed,
  and a query after that hung for more than two minutes before it answered.
- A pattern that names only the AVD:
  it also matched a second,
  unrelated emulator that used the same AVD name on other ports.
