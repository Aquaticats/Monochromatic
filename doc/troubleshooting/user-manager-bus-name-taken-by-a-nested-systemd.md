# A nested `systemd --user` took the user manager's bus name

## What happened

At 2026-11-11T03:51:27-05:00 a probe started `/usr/lib/systemd/systemd --user --log-target=console`
as a child of another agent's session (pid 2477887,
cgroup `…/app-monochromatic\x2dupdates.slice/wg-quicker-update-app.slice-run\x2dp2476658\x2di316681074.scope.scope/init.scope`).
systemd asks for `org.freedesktop.systemd1` on the session bus with
`SD_BUS_NAME_REPLACE_EXISTING|SD_BUS_NAME_ALLOW_REPLACEMENT`
(`src/core/dbus.c`, line 809 of v262),
and the real manager (the main process of `user@1000.service`) never asks for the name back.
The impostor owned the name for almost three hours,
until it was killed and the real manager was re-executed
(`systemctl --user daemon-reexec`) at about 06:40.

While it owned the name:

- `systemctl --user …` and `busctl --user call org.freedesktop.systemd1 …` reached the impostor,
  not the real manager.
- The `wg-quicker-exempt` watcher called the manager by that bus name,
  so its adoptions,
  its scope limits and its leftover sweep talked to two different managers at once.
  A container build was adopted into the impostor's tree,
  where it shared a 100 % CPU, 1 GiB, 256-task scope,
  and a Git wrapper release gate compiled on one throttled core for most of an hour
  (`nr_throttled 71282` of `nr_periods 90513`).
- `systemctl --user status <pid>` printed the impostor's unit for processes that belonged to the real one.

## What to do when it happens again

1. Look for a second manager:
   `pgrep --full 'systemd --user'` shows more than one pid;
   only the main process of `user@<uid>.service` (from `systemctl show --property=MainPID user@<uid>.service`)
   should be there.
2. Kill the extra pid with `SIGKILL`, never `SIGTERM`:
   an impostor manager stops the units it started when it is asked to stop,
   which takes those units with it.
3. Make the real manager take the name again: `systemctl --user daemon-reexec`,
   then check `busctl --user call org.freedesktop.DBus /org/freedesktop/DBus org.freedesktop.DBus GetNameOwner s org.freedesktop.systemd1`
   and that `GetConnectionUnixProcessID` of the answer is the manager's pid.

## What the packages do about it now

- The `wg-quicker-exempt` watcher no longer asks a bus who the manager is:
  every call goes over the manager's own private socket
  (`/run/user/<uid>/systemd/private`),
  and before the first byte the watcher compares `SO_PEERCRED` with the main process the system manager reports.
  A mismatch refuses the call loudly and moves nothing.
- The root watcher starts `__impostor-guard` as the watched user:
  it hears `NameOwnerChanged` for the name (a five-second poll is the backstop),
  kills an owner that is not the manager with `SIGKILL`,
  and asks the manager to take the name again at most once in five minutes.
  Every action and refusal goes to the log, the journal and the desktop.
- `busctl` cannot be used for the private socket at all:
  it is a bus client and answers
  `Call failed: Invalid request descriptor` against it (reproduced on this host),
  which is why the wire format is written in the package (`src/dbus_wire.rs`).
- A client that connects to a user's session bus must run as that user:
  `busctl --address=unix:path=/run/user/1000/bus …` as root fails with
  `Failed to connect to system scope bus via local transport: Broken pipe`,
  because the bus daemon authenticates the peer by its uid.

## Traps

- A nested manager cannot be started by an agent probe on this machine:
  the guard kills it as soon as it takes the name,
  and the run ends with it.
  A probe that needs a manager runs inside a container or a throwaway VM,
  whose bus is not this machine's.
- `systemctl --user` with `XDG_RUNTIME_DIR` of another user fails from root:
  sd-bus refuses the peer on its own,
  before the manager sees anything (`Operation not permitted`).
