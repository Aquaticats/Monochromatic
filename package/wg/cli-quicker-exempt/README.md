# wg-quicker-exempt

Marks sockets from selected cgroups so policy routing can keep application traffic outside a WireGuard tunnel.

## Why

`wg-quicker` installs a dedicated bypass table and an `ip rule` matching an exemption mark.
This loader applies that mark to sockets created in Ghostty,
 Steam,
 Helium,
 Pale Moon,
 Firefox release,
 Firefox Nightly,
 ChatGPT,
 Interpreter,
 Qure,
 and OdyTTY cgroups without enumerating their destination IPs.

It also moves update traffic out of the tunnel by process,
and covers Pi,
the agent harness this repository runs,
whose provider traffic leaves the tunnel too.
See [Update traffic policy](#update-traffic-policy).

## How it works

For each cgroup,
 the Rust loader:

1. creates a one-entry `BPF_MAP_TYPE_ARRAY` containing the socket mark;
2. loads `BPF_PROG_TYPE_CGROUP_SOCK_ADDR` programs for TCP connect and UDP sendmsg,
    IPv4 and IPv6;
3. calls `bpf_setsockopt(SOL_SOCKET, SO_MARK)` with the map-value pointer;
4. denies the socket operation if `bpf_setsockopt` fails;
5. creates one BPF link for each hook;
6. pins all four links under `/sys/fs/bpf/wg-quicker-exempt/`;
7. falls back to a detached descriptor keeper when the detected SELinux regression makes `BPF_OBJ_PIN` return `EINVAL`.

The loader uses raw stable `bpf(2)` UAPI through `libc`.
It does not require libbpf or bpftool.

## Usage

Attach or atomically replace the marker for one or more cgroups:

```sh
wg-quicker-exempt attach <mark> <cgroup-dir>...
```

Detach exact persisted links:

```sh
wg-quicker-exempt detach <cgroup-dir>...
```

List Ghostty,
 Steam,
 Helium,
 Pale Moon,
 Firefox release,
 Firefox Nightly,
 ChatGPT,
 Interpreter,
 Qure,
 OdyTTY
 and `pi-<name>.service` targets without attaching:

```sh
wg-quicker-exempt list-targets <uid>
```

Start or stop detached application watcher owned by tunnel key:

```sh
wg-quicker-exempt watch-start <key> <mark> <uid>
wg-quicker-exempt watch-stop <key>
```

Example:

```sh
wg-quicker-exempt attach 100 \
  /sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/app.slice/app-com.mitchellh.ghostty@id.service \
  /sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/app.slice/app-ghostty-surface-transient-123.scope
```

Root or equivalent BPF,
 network-administration,
 and cgroup attach capabilities are required.

## Update traffic policy

The wg packages decide alone that update traffic leaves outside the tunnel;
the toolchain places nothing and knows nothing about it.
The user's words,
verbatim:

> mpm-shadow should never determine itself goes through clearnet.
> Our wg set of pkgs should determine that!

> Anyone using our wg set of tools should be warned on start and the behavior shouldn't have a toggle.
> It should always enforce our policies,
> even clearnet.

The owner decided that a config without `ExemptMark` is warned about and still brought up,
and that `Table = off` stays a loud report that the policies are not in force.
`wg-quicker` does not start this watcher for a config without `ExemptMark`;
with `Table = off` it still does,
but the marks decide nothing because no bypass rule exists.
The owner's words and the reason are in the `wg-quicker` README,
section "Two configs that run without the policies,
loudly".

This watcher enforces two of the three methods the user chose;
the update-host list is enforced by `wg-quicker`'s route watcher
(see the [wg-quicker README](../cli-quicker/README.md#update-traffic-policy)).

- The process watch:
  A process that is an update entry point is moved into `app-monochromatic\x2dupdates.slice`.
  The list lives in `src/update_entry_points.rs`:
  `mise run` of the fetching tasks (`toolchain:install`,
  `update`,
  `outdated`,
  their `:global` forms,
  and the aliases `bootstrap` and `prepare:android`),
  `mpm-shadow install`,
  `i`,
  `update` with or without `-g`,
  pnpm's own aliases `up` and `upgrade` of `update`,
  `outdated`,
  the `mpm` shadow,
  `node` running the TypeScript `mpm-shadow` launcher,
  `package/monorepo-task/core-exec/src/mpm-shadow.ts` or its built entry,
  which every root `toolchain:*` task runs and which downloads the locked archive when no installed copy is new enough.
  The launcher entry is a proposal open to the owner's veto:
  under `mise` the matched task already carries it,
  and only a direct run outside `mise` needs the entry.
  The offline tasks `toolchain:list`,
  `toolchain:list:global` and `toolchain:lock:check` stay off the list.
  A process matches by the file name of its resolved executable (`mise`,
  `mpm-shadow`,
  `mpm`,
  `node`)
  together with its command-line words.
  The first command-line word alone never matches,
  because `exec -a mpm /usr/bin/curl` sets it freely;
  it counts only for `mpm-shadow` started as `mpm`.
  Pi is an entry point of its own:
  `node` whose first command-line word is `pi`,
  the single word Pi leaves after it rewrites its process title,
  or `node` running Pi's CLI entry,
  the tail `/pi-coding-agent/dist/bundle/cli.js`.
  The word counts only for `node`,
  so `exec -a pi /usr/bin/curl` does not match,
  and a checkout of that package runs the same entry and matches too.
  Tests read the root and package `mise.toml` files,
  the verb grammar in `package/mpm-shadow/cli/src/toolchain/verb_grammar.rs` and `cli.rs`,
  and the launcher script.
  They fail when a fetching task or verb is missing from the list,
  when a listed name no longer exists,
  and when the grammar gains a verb that is not yet classified as fetching or offline.
- The toolchain paths:
  An executable under the watched user's `.local/share/monochromatic-dev/toolchain/` or `toolchain-global/`
  is moved the same way.
  The home comes from the system user database for the watched uid,
  in the spelling the user database gives and its resolved spelling.
  A custom `XDG_DATA_HOME` is not seen by the root watcher.
  A link is never followed.
  The owner chose the option "Refuse links":
  a toolchain folder that is a link is ignored,
  wherever it points,
  and the watcher reports it loudly at every start (`POLICY NOT IN FORCE` for that root,
  in the watcher log,
  which `up` relays).
  This also holds for a link on the way to a root below the home
  (the report then names that link,
  for example `.local/share/monochromatic-dev`),
  for a link whose target is inside the home,
  for a dangling link and for a link loop.
  The directory is writable by the user while the watcher runs as root,
  so a followed link would make every program at its target leave the tunnel;
  the reviewer's probes linked the root to `/` (and `/usr/bin/curl` matched),
  and to `~/.local/share` (and everything below it matched),
  and following only links inside the home still allowed the second.
  The home itself may be reached through a link,
  such as `/home` pointing to `/var/home`,
  because only what lies below the home is examined,
  and both spellings of the home are roots.
  A root that cannot be examined for any reason other than not existing is reported the same way.
  A root that does not exist yet is not reported.
  Plain directories are matched as before.
  A root that is refused is not matched under its own name either.
  A link created later is not seen until the next start,
  and its programs do not match in the meantime,
  because a resolved executable path never names a link.
  The start notice lists installed tools only from roots with no link on the way,
  opened without following a link at the last component and read through that handle.
  `src/install_roots_tests.rs` has the accepted and refused fixtures:
  a link inside the home,
  to the data directory of the home,
  to `/`,
  to the home itself,
  to another user's home,
  a link loop,
  a dangling link,
  a link on the way to a root,
  no resolved home or a home that resolves to `/` (a link is refused either way),
  a home reached through a link,
  and a root or a way that became a link after the start (the listing does not read it).
  The roots cover every program the toolchain installed,
  so every script and dev server run by the toolchain's node leaves the tunnel too,
  not only updates;
  the start notice lists what is installed now.
- The socket-marked cgroups:
  A cgroup of the user's `app.slice` whose name the watcher's own rules select carries the socket mark itself,
  so every process in it,
  and every process of a cgroup the manager creates below it,
  leaves the tunnel without a move.
  The named terminals and desktop applications are selected this way,
  and so is every `pi-<name>.service` Pi agent unit.
  A desktop application's name is believed only while the cgroup it names holds that application's own program
  (`is_application_executable` in `src/application_targets.rs`,
  one program rule per application,
  shared with the process watch so a name and a program cannot disagree about what an application is).
  A unit a user named `app-firefox@anything.service` and filled with another program is therefore not marked,
  where the name alone used to be enough;
  the watcher reports such a cgroup once,
  under `POLICY NOT IN FORCE` for that cgroup,
  naming it and saying that its traffic stays in the tunnel,
  and `list-targets` lists it as not marked.
  That report is how an application whose program this build spells differently is noticed
  instead of losing its exemption in silence.
  The two names that need no evidence are the Pi units and the updates slice,
  which the owner accepted as names alone.
  The user's service manager cannot move a service's main process,
  so a Pi unit is never moved:
  the mark attached to the unit's own cgroup covers the agent,
  its driver and every child process it starts,
  which is the rule the terminals have.
  One predicate answers both the attaching and the process watch,
  so the two can never disagree about which cgroup carries the mark.
  Not covered:
  processes in another pid namespace (containers and sandboxes),
  processes of a login session,
  and the main process of a service whose cgroup the watcher does not mark;
  each one that matches is reported by name.

The slice is marked by the same cgroup programs as the application cgroups,
and a cgroup program applies to every descendant cgroup,
so a scope the user's manager creates below the slice inherits the mark.
The `app-` prefix makes systemd nest the slice directly under `app.slice`,
where the watcher scans and watches,
and `\x2d` is the escaped literal dash.
The watcher creates the slice itself before it first scans.

### Detecting a start

The watcher subscribes to the kernel's process-event connector
(netlink `NETLINK_CONNECTOR`,
`CN_IDX_PROC`,
`PROC_CN_MCAST_LISTEN`)
and reacts to `PROC_EVENT_EXEC`,
which arrives with the exec itself.
Alternatives were weaker:
polling `/proc` alone misses short processes and reacts late;
fanotify on executables sees opens,
not process starts;
a BPF tracepoint needs a second BPF loader and a verifier-tested program the watcher does not otherwise carry.
The connector needs no new dependency and the root watcher already holds the privilege.
Layout constants come from `linux/connector.h` and `linux/cn_proc.h`,
checked with a C program against the target kernel headers
(`nlmsghdr` 16 bytes,
`cn_msg` 20,
`proc_event` 40,
exec data at 16).
Only datagrams from netlink port 0,
the kernel,
are trusted;
a test sends a valid exec event from another local netlink socket and shows it is dropped.
The socket asks the kernel for exec events only.
The plain listen message subscribes to every kind of process event (fork,
exit,
comm,
uid and the others),
and the watcher then dropped all but exec in user space:
measured on this host over half a second,
the plain subscription received 110 exec,
378 fork and 386 exit events,
and the exec-only one 79 exec events.
The listen message is followed by the eight-byte `struct proc_input` form with the exec mask,
which `cn_proc_mcast_ctl` and `cn_filter` of Linux v7.2.11 (`drivers/connector/cn_proc.c`) apply inside the kernel;
a kernel that ignores the form keeps the subscription the first message made.
An ignored test subscribes both ways and shows the difference.
The receive buffer is asked to hold 4 MiB.
When the watcher stops it tells the kernel to ignore the socket (`PROC_CN_MCAST_IGNORE`):
the kernel changes its listener count only on listen and ignore,
and closing a socket leaves the count raised
(`drivers/connector/cn_proc.c` lines 450 and 454,
`connector.c` lines 185 to 190,
Linux v7.2.11),
so each restart used to leave one more listener counted,
and while the count is above zero the kernel builds and broadcasts an event for every fork,
exec and exit.
A watcher that is killed cannot do this;
the next one starts counting from the count it finds.
An ignored test shows the delivery ending.
Delivery is lossy when the watcher falls behind:
an overrun is reported at once,
and every process is rescanned no sooner than one second after the previous scan,
because a scan reads every process of the host (about 26 ms of CPU for 949 processes,
measured).
A full scan also runs every two seconds of wall time as the backstop,
whatever the number of loop wakeups.

### Moving a process

The watcher does not write `cgroup.procs`.
It asks the user's systemd manager to adopt the pids
(`StartTransientUnit` with `PIDs` and `Slice`,
or `AttachProcessesToUnit`).
systemd's cgroup delegation document states the single-writer rule:
each cgroup has one writer,
and a writer that ignores it leaves software fighting over ownership.
A direct write would put processes into a cgroup the manager never created,
which it would not garbage-collect and whose pid-to-unit map would go stale.
The call goes over the manager's own private socket
(`/run/user/<uid>/systemd/private`),
never over a bus name,
and see [One process may not answer for the manager](#one-process-may-not-answer-for-the-manager).
The wire format is written in this program
(`src/dbus_wire.rs`),
because `busctl` is a bus client and asks the bus daemon first,
which a private socket does not answer
(`Call failed: Invalid request descriptor`,
reproduced),
and because the old bus-name call is what a second manager could take over.
One connection and one login precede each call,
and the socket's peer credentials are checked before anything is written.
Each call has one deadline, taken before the connect:
not the socket's read timeout,
which restarts with every read,
so a manager that emits an unrelated message more often than the timeout cannot keep an unanswered call open.
A call that passes its deadline is a `TimedOut` refusal,
which pauses the moves for a few seconds.

The group id handed to `setpriv` is the watched user's,
not the matched process's.

The matched process moves together with its live descendants of the same user that sit in the same cgroup.
A descendant a user placed in another cgroup on purpose stays there.
Every pid of the group,
the matched one included,
has its start time checked again right before the request,
so a reused pid is not moved,
and a matched process that was replaced stops the move.
The group id handed to `setpriv` is the watched user's,
not the matched process's.
After the manager accepts the job,
the watcher waits up to one second until `/proc/<pid>/cgroup` names the slice,
and it notices at once when the process exited meanwhile,
which is not a failure.

A move that fails is tried again after one second,
then two,
then four,
up to a minute between attempts,
and each failure is reported with its attempt number.
One pass spends at most two failed attempts and two seconds on moves,
and postpones the rest;
a postponed move is reported with the reason (`move postponed`),
because until it is tried again its traffic uses the tunnel.
The `busctl` client is killed after three seconds,
and a timeout pauses all moves for ten seconds,
so a hung manager cannot stall the watcher for long.
A shutdown signal is checked between two processes,
so a long pass does not leave it waiting.
A process of the watched user that the watcher may not inspect is reported once,
and is not mistaken for one that exited.
A process in another pid namespace (a container or a sandbox) is reported and left alone,
and so is a process whose pid namespace link cannot be read,
because it cannot be told apart from one.
A watcher that cannot read its own pid namespace says so at start.

The manager can only move processes inside its own subtree,
`user@<uid>.service`.
A process in a login session scope (`session-N.scope` under `user-<uid>.slice`),
or the main process of a service the watcher does not mark,
is not moved,
and the report says so by name.
A process of a cgroup the watcher marks itself,
such as a `pi-<name>.service` unit or a terminal's,
is covered by that cgroup's mark instead and reported as covered without a move.

### Leftover scopes of the earlier design

The first design made a scope per moved process,
named `wg-quicker-update-<pid>-<start time>.scope`.
Scopes that adopted only exiting processes stayed running with no process,
or failed with no cgroup,
and nothing removes them:
the manager stops a scope when its last process exits,
and a scope that never held one gets no such event.
When the watcher starts it sweeps them once.
The scopes of one origin are stopped before the first target scan and the first adoption,
in the watcher's own thread:
a process joining an origin scope is a process a watcher moved there,
and the manager ends every process in a scope it stops,
so a sweep running beside adoption cannot make that decision safely.
Ordering alone does not exclude a second watcher of the same user,
which moves processes into the same origin scopes while the starting watcher sweeps,
so both sides hold one lock file per watched user,
`/run/wg-quicker-exempt-adoption-<uid>.lock`:
an origin batch reads its scopes and sends its stop under that lock,
and every move takes it across its manager call and its confirmation.
A move that cannot take it at once is left for the next scan,
and its log line says its traffic uses the tunnel until then,
which keeps the watcher's loop out of another watcher's slow manager call.
A batch whose lock stays held for five seconds is left for the next watcher start,
and the sweep line says how many scopes that was and what was waited for.
The scopes of the earlier per-process design,
which the current watcher never fills,
are swept on a thread of its own,
so the policy is enforced while the manager works.
It lists the manager's units that match the earlier design's names (two runs of digits,
not a path),
resets the failed ones,
and stops the running ones whose `cgroup.events` says `populated 0`,
25 at a time,
reading each one again right before its batch is sent.
It never touches a scope that holds a process,
which ends with its processes,
a scope whose state cannot be read,
or a scope of the current design,
whose name is a path.
The sweep also stops a scope that holds the processes of one origin while its cgroup holds no process,
including one an earlier build named with a second `.scope`:
the manager stops a scope when its last process exits,
but a scope that was created for a process that was already exiting stays empty for ever,
and nothing else would ever find it again.
A running scope of that kind that holds a process is left alone,
and the manager cannot report an active scope that adopted a process as empty,
because the start job attaches the processes before the scope becomes active.
Its counts are logged once.
Nothing can put a process into an empty scope of the earlier design afterwards,
so a scope that reads empty stays empty.

### One scope per origin

The owner's decision,
2026-11-01,
chosen as "Shared scope per origin":
"Processes moved from the same origin cgroup go into one scope (per origin) that carries that origin's CPU,
memory,
process and IO limits once,
so together they stay within the origin's cap.
Pros:
caps stay shared as intended;
one scope per origin instead of thousands."
The preview the owner saw was `claude-code-bash (cap 28 CPUs, shared) -> wg-quicker-update-claude-code-bash.scope (cap 28 CPUs, shared by all moved processes)`.
This replaces the first design,
a scope per moved process that carried a copy of the cap
(the owner's earlier decision,
"Keep origin caps",
still holds:
a moved process keeps its origin's CPU and memory limits).

The origin is the cgroup the process leaves.
Its scope is named from the path below `user@<uid>.service`,
escaped as `systemd-escape --path` does:
a slash becomes a dash,
and a dash or any byte outside letters,
digits,
colon,
underscore and an inner dot becomes `\xNN`.
`app.slice/claude-code-bash` is `wg-quicker-update-app.slice-claude\x2dcode\x2dbash.scope`
(the owner's preview was illustrative;
the real name keeps the parent,
because two origins with the same last component would otherwise share one cap).
A name that would pass the 255-byte unit name limit is cut and ends in a hash of the whole path.
The name ends in exactly one `.scope`:
the dot of a trailing unit suffix of the origin
(`run-p1-i2.scope`,
`x.service`) is escaped as `\x2e`,
so `app.slice/foo` and `app.slice/foo.scope` keep distinct scopes
instead of one scope with the other origin's cap,
and `app.slice/run-p1-i2.scope` is `wg-quicker-update-app.slice-run\x2dp1\x2di2\x2escope.scope`.
The spelling earlier builds wrote is still found before a new scope is made,
so an origin whose scope is already running under
`wg-quicker-update-app.slice-run\x2dp1\x2di2.scope.scope`
receives its later processes there instead of in a second scope with a copy of the cap.

The description `systemctl status` shows names the origin cgroup and the method that moved the process,
for example
`wg-quicker: adopted from /user.slice/user-1000.slice/user@1000.service/app.slice/claude-code-bash (moved because it is a program under the toolchain install roots)`,
so a test run's scope is not read as an update run of its own.

The first process from an origin makes the scope with `StartTransientUnit`:
with the origin's limits,
`Delegate=yes` and `CollectMode=inactive-or-failed`.
Later processes from that origin are attached with `AttachProcessesToUnit`.
Whether the scope exists is read from its cgroup directory,
which exists exactly while the scope is active,
so the watcher keeps no state about it and a restarted watcher finds the scopes of the one before.
`Delegate=yes` is required:
the manager refuses `AttachProcessesToUnit` for a unit that is not delegated
(`bus_unit_method_attach_processes` in systemd v259.10 `src/core/dbus-unit.c`,
line 1592);
a throwaway scope showed the limits still apply to the scope's own cgroup and `cgroup.subtree_control` stays empty.
The scope ends by itself when its last process exits,
and `CollectMode` makes the manager forget it at once,
where the first design left 982 failed and 1293 empty running scopes in 88 minutes.
BPF programs attached to `app-monochromatic\x2dupdates.slice` apply to its descendant cgroups by the kernel's inheritance of cgroup programs;
that this holds for the delegated scope was not verified with a root watcher in this change.

The limits are `cpu.max` as `CPUQuotaPerSecUSec`,
`memory.max` as `MemoryMax`,
`memory.high` as `MemoryHigh`,
`memory.swap.max` as `MemorySwapMax`,
`pids.max` as `TasksMax`,
the `default` line of `io.weight` as `IOWeight`,
and the per-device limits of `io.max` (`rbps`,
`wbps`,
`riops` and `wiops`)
as `IOReadBandwidthMax`,
`IOWriteBandwidthMax`,
`IOReadIOPSMax` and `IOWriteIOPSMax`,
keyed by device.
The manager takes a device as a path,
so the `major:minor` of the file is sent as `/dev/block/<major>:<minor>`,
a symbolic link that udev keeps for every block device and the manager resolves to the device;
a device that is gone when the limit is applied is skipped by the manager.
A throwaway scope showed the kernel's `io.max` of the scope holding the line the properties describe,
and an empty array cleared it.
Of the origin's cgroups the smallest finite value of each kind and device is used.
The manager writes `IOWeight` to `io.weight` and to `io.bfq.weight`
(`set_io_weight` in `src/core/cgroup.c`,
lines 1443 to 1452),
which matters on this host because its disks use the BFQ scheduler,
which reads only `io.bfq.weight`:
the cap daemon writes `io.weight` 10 on `claude-code-bash`,
whose `io.bfq.weight` stays at 100,
so the moved processes are throttled more on I/O than the processes that stayed.
That is a tightening,
which the policy allows.
A weight above the kernel default of 100 is never carried,
and per-device lines of `io.weight` are not read.
The smallest finite value on the cgroups between the old place and the nearest ancestor it shares with the new place is used;
limits of the shared ancestors keep applying by themselves.
A limit file that cannot be read stops the move with a report,
and a move that fails is never tried again without the limits.
The origin keeps its own cap for the processes that stayed,
so the two together can use up to twice the cap;
the owner's option promises only that the moved processes share one cap.

The cap daemon rewrites `cpu.max` of `claude-code-bash` as the machine heats and cools (its `CAP-APPLIED` log lines),
so the scope follows its origin while it lives:
on every full scan the watcher reads the origin's limits again
and,
when they differ from the ones the scope was last given,
applies them with `SetUnitProperties`,
in both directions and every limit at once
(a limit the origin no longer sets goes back to unlimited,
the I/O weight to 100).
A scope the watcher did not create,
found running after a restart,
is given the origin's limits when processes first join it.
When the origin's directory is gone the scope keeps the limits it has,
and a failure to apply is reported as `scope limits not applied`.
When the scope is gone its record is dropped.
Each application is a call to the user's manager that may run as long as the client timeout,
three seconds,
on the watcher's own thread,
so it is bounded like a move:
a scan spends at most one second on them and leaves the rest for the next scan,
an origin whose application failed waits one second,
then two,
up to a minute,
before it is tried again,
no scan asks the manager while moves are paused after a timeout,
and a call that times out ends the scan and pauses the moves for ten seconds,
so a hung manager cannot stall the watcher for more than one call.

A process that is exiting is never handed to the manager.
The kernel's `cgroup_migrate_add_task` returns early for a `PF_EXITING` task while the write to `cgroup.procs` still succeeds,
and `scope_enter_running` counts that attach and relies on empty notifications afterwards
(Linux v7.2.11 `kernel/cgroup/cgroup.c` lines 2594 to 2596;
systemd v259.10 `src/core/scope.c` line 403 onward),
so a scope that adopts only exiting processes is never populated,
never gets an empty event,
and runs forever.
Reproduced on this host in a throwaway slice,
6 of 6 times.
The watcher skips zombies and tasks flagged `PF_EXITING`,
ends the wait for a move at once when the process is exiting,
and after every create or attach reads the scope's `cgroup.events`:
`populated 0` stops the scope and counts the move as a vanish,
which needs no report (the watcher has no debug log level,
so nothing is written).

The group of a matched process is the members of its cgroup that descend from it,
read from `cgroup.procs`.
Before,
every move listed and read every process of the host.
Measured on this host (949 processes,
release build,
thread CPU time),
one move's group collection went from 12061 read calls and 31.7 ms to 292 and 1.95 ms,
and a pass over every process from 12005 read calls and 29.7 ms to 3302 and 25.9 ms with procfs files read in two calls instead of about five.
The periodic scan and the scan after dropped events still read every process.

### One process may not answer for the manager

On 2026-11-11T03:51:27-05:00 an agent's probe started a second
`/usr/lib/systemd/systemd --user` on this machine.
systemd asks for `org.freedesktop.systemd1` on the session bus with replacement allowed,
and the real manager never asks for it back,
so the impostor owned the name until it was killed hours later.
Every watcher call went to it,
and adopted scopes,
and a container build inside one,
ended up in its tree with its limits.
The owner,
after that:
"Make sure no imposer can never succeed again."

What the watcher does now,
in order:

1. It reads the main process of `user@<uid>.service` from the system manager
   (`systemctl show --property=MainPID`),
   which answers over its own root-owned socket.
2. It opens the user manager's private socket and reads `SO_PEERCRED`.
   A pid that is not that main process is refused loudly,
   naming both pids and the socket,
   and nothing is written,
   so nothing is moved.
   The pid comes from the kernel for the connected socket,
   so a socket the user planted at that path answers with its own pid and is refused.
3. Every call goes over that socket,
   so no name lookup happens anywhere on the path.
4. The root watcher starts the `__impostor-guard` child as the watched user,
   because a user bus refuses a connection from root.
   It asks the session bus who owns `org.freedesktop.systemd1`,
   and hears `NameOwnerChanged` for that name;
   a poll every five seconds is the backstop.
   An owner that is not the user's manager is killed with `SIGKILL`,
   never `SIGTERM`,
   because an impostor manager stops the units it started when it is asked to stop.
   The real manager is then asked to take the name again
   (`systemctl --user daemon-reexec`),
   at most once in five minutes.
   No owner at all asks for that re-execution too,
   once.
   A bus that cannot be asked changes nothing,
   because guessing could kill the manager itself.
   Every action,
   and every refusal,
   goes to the watcher log,
   the journal
   and the desktop of the watched user.

The guard runs as the watched user,
and the re-execution it asks for drops to that user's own primary group,
never to the user id taken as a group id:
a host whose accounts have a primary group number other than the user id,
or no group numbered like the user at all,
made `setpriv --regid <uid>` fail,
so the manager was never asked to take the name again after an impostor had been killed.
`the_reexecution_drops_to_the_users_primary_group` covers it.

Every call to that bus ends at a deadline measured from its start,
and not only at the socket timeout:
a bus that answers with a message of another serial,
or with a signal,
more often than that timeout would restart it on every read,
and the guard would skip those messages for ever without ever finishing its question about the name.
The wait for `NameOwnerChanged` is bounded by one deadline the same way,
so a bus that keeps sending other messages cannot starve the poll that ends it.

Every signal is proven first.
The pid a bus reports is not an identity the kernel keeps:
an owner can exit and the kernel can give its number to another process of the same user,
so the guard asks the bus again whether the same unique connection name still names that pid,
and sends `SIGKILL` through a process descriptor (`pidfd`) opened for it,
which refers to that process and not to the number.
A pid the bus no longer names,
or names with another pid,
is never signalled;
a kernel without process descriptors falls back to the bus proof and a direct signal.
The guard also ends when its watcher does:
the watcher holds the guard's standard input for its whole life,
and the guard's loop ends when that input reaches its end,
which a stop, an exit or a kill of the watcher all close.
Without that, a guard outlived its watcher and kept polling the bus,
and every restart added another one.

The guard's decisions and its signal are tested
(`src/impostor_guard_tests.rs`):
the impostor is killed with the one signal it cannot handle,
a second impostor inside the gap is killed without another re-execution,
and the manager owning the name is left alone.
`src/manager_socket_tests.rs` proves the refusal:
a socket answered by another process is refused,
the refusal names both pids,
and no byte is sent;
the same socket with the right pid is called and answered.
The live ignored test `the_real_manager_creates_attaches_and_follows_limits`
does the whole adoption against the real manager through that socket.

The coordinator's stopgap service `guard-user-manager-bus-name`
does the same from outside the packages until a watcher with this code is deployed.

### Warned on start, loud on failure

The watcher prints the policy notice when it starts,
into its log,
and `watch-start` prints it too.
The notice names the update entry points,
everything the toolchain installed (with the names installed now),
what can imitate a match,
what is not covered,
and that with `Table = off` there is no bypass rule.
Every `wg-quicker-exempt: POLICY NOT IN FORCE` line the new watcher writes to its log while starting
is also printed by `watch-start`,
so `wg-quicker up` shows a policy that failed to start.
A line that starts with `wg-quicker-exempt: POLICY NOT IN FORCE` names the process-watch policy and what failed:
the slice cannot be created,
the process-event subscription failed (entry points are then found only by the periodic scan),
events were dropped,
reading events failed,
the list of processes cannot be read,
a process cannot be inspected,
a matched process is outside the user manager's subtree or in another pid namespace,
or the adoption failed.
A failing BPF attach or socket-marking rescan is reported under the socket-marking policy.
No failure invents a fallback route.

### Failures after startup

The watcher runs detached,
and its standard error is a log file nobody reads while working.
The owner chose "Notification + journal":
every failure goes to the system journal at error level and to a desktop notification for the tunnel's user,
rate limited so a repeating failure does not flood.
A failure after startup is delivered through three channels,
each with its own limit per failure kind and cause:

- the log file,
  every ten seconds;
- the system journal,
  under the identifier `wg-quicker-exempt`,
  at error level (`journalctl --identifier=wg-quicker-exempt`),
  every minute;
- a desktop notification to the watched user through `org.freedesktop.Notifications`,
  sent with `busctl` as that user,
  every ten minutes.

The limit keys on the kind of failure (`move failed`,
`move skipped`,
`move postponed`,
`process unreadable`,
`process list unreadable`,
`process events dropped`,
`process events unreadable`,
`socket marking rescan failed`,
`watcher stopped`,
`watcher died`) and on a normalized cause within the kind,
never on the raw text,
because the text embeds names a user chose,
such as a cgroup path,
and a key on the raw text would let distinct names defeat the limit.
The normalized cause is the line,
at most its first 1024 characters,
with every run of digits made alike and every name a user chose replaced by a placeholder
(an absolute path,
which carries cgroup names,
and a unit name ending in `.service`,
`.scope`,
`.slice`,
`.socket` or `.mount`).
A cgroup name can contain spaces,
so the lines that embed one put it between backticks (`quote_name`,
with a backtick inside the name written as `\u{0060}`),
and the cause replaces everything between the backticks;
a name that is not between backticks is replaced word by word.
So the same reason for another process id,
count,
duration or cgroup name is one cause,
and a different reason is another.
The bound is large because every loud line starts with a fixed phrase of over a hundred characters:
a bound of 200 ended inside it,
and two different `busctl` errors for one kind collapsed into one cause.
A name a user chose cannot make a new cause,
so a user cannot fill the few slots of a kind and hide a real reason.
Each channel delivers the first four distinct causes of a kind in each interval,
so three matched processes skipped for three different reasons are three reports and not one.
Beyond the first cause of every kind,
a channel delivers at most 32 further causes per interval over all kinds together,
and a kind's first cause is always delivered.
The map of remembered kinds is bounded;
when it is full the oldest kind is forgotten.
A repeat or a further cause held back by the limit is counted,
and the next delivery of that kind says "repeated N more time(s) since the last report",
so no occurrence is lost from the count;
the identity of a held-back cause beyond the cap is not kept.
A notification replaces the earlier one only of the same kind and cause,
so distinct causes stay on the desktop side by side;
the note that counts held-back repeats is not part of the cause,
so a repeat that carries it still replaces its first notification.

While the watcher starts,
the log file takes every report without the limit,
because `up` relays that log and a start that hid a finding from `up` would not have said that a policy is not in force.
The journal and the desktop keep their limits during the start.
The relay prints the loud lines of the log,
and the notices that the log itself dropped lines (`LOG LIMIT`),
up to 200 lines,
and then one line saying how many more there are and where the log is.
The log budget bounds the start too:
a start with so many findings that the log budget is reached says so in the log and in `up`,
and the findings beyond the budget are not in the log,
though the journal and the desktop still receive the first four causes of every kind.

A notification never blocks the watcher.
A worker thread sends it through a queue of eight;
when the queue is full the notification is refused and the refusal is named in the log.
A later notification of the same kind replaces the earlier one on the desktop
(the id `Notify` returned is passed as `replaces_id`),
and each notification expires after the interval,
so one persistent failure shows one notification and not six an hour.

Nothing here can panic.
The standard `eprintln!` panics when a write fails,
for example on a full `/run` tmpfs,
and a panic would end every policy the watcher enforces without a word;
every write is a plain `write` whose failure is ignored.
One channel failing never keeps the others from receiving the report,
and the failing channel is named in the log.
The log is bounded too:
256 KiB per hour and 4 MiB over the life of the watcher.
When an hour's budget is reached the log says so once and drops lines until the hour ends.
When the 4 MiB for the whole life is spent the log says so once,
in words that name that budget,
and nothing more is written to it,
success lines included,
so the moves of later update processes are no longer in the log;
the journal and the desktop have their own channels and limits and still receive every failure kind.
A `watch-stop` and a new `watch-start` give a fresh budget.
A start reads at most 1 MiB of the log to relay its startup reports.

Text that a user chose reaches a line escaped.
Every loud line is built through one function that escapes control characters,
escape sequences,
line breaks,
line and paragraph separators (U+2028,
U+2029),
direction marks,
and the invisible tag characters (U+E0000 to U+E007F) as `\u{XXXX}`,
so a cgroup name cannot repaint the terminal that ran `watch-start`,
start a second journal line or end the line early;
the notification builder sanitizes its body itself and then turns `<`,
`>`,
`&` and quotes into entities,
and the startup relay escapes again.
The tool names in the start notice are listed from one directory handle per install root:
the root is first checked for a link on the way below the home and then opened without following a link,
the handle's own place is read from `/proc/self/fd` and must lie strictly inside the home,
and the entries are read through the handle,
so a root swapped for a link after the check cannot make the notice list another place.

The loop ending on an error is reported the same way before the watcher exits,
naming that neither the socket marking nor the process watch is enforced any more.
A process-event socket that fails to be read is dropped and reported once,
so a persistent error cannot spin the loop;
the periodic scan carries on alone.
Successful moves are logged to the file only.

### When the watcher itself dies

The watcher cannot report its own death when `SIGKILL`,
the out-of-memory killer or `systemd-oomd` ends it,
and from then on neither the socket marking nor the process watch is enforced.
A second small process,
the sentinel (`__sentinel`),
is started by `watch-start` after the watcher committed.
It is started outside the cgroup of the watcher:
as its own transient service of the system manager,
`wg-quicker-exempt-sentinel-<watcher pid>-<start time>`,
through `systemd-run --collect` with `ManagedOOMPreference=omit`,
with the program behind `/usr/bin/env`.
On an SELinux-enforcing host the system manager (`init_t`) may not execute a file under a home directory
or a file labelled for containers,
and this program's build and the toolchain sit in such places:
the unit was accepted and then failed with `status=203/EXEC`.
`/usr/bin/env` is executable by the system manager,
and the service then runs in `unconfined_service_t`,
which may execute them.
The cause and the measurements are in `doc/troubleshooting/selinux-denies-system-service-exec-of-home-files.md`.
The watcher runs in the cgroup of the terminal or session that started it,
and a kill of that whole cgroup,
for example by `systemd-oomd`,
which the repository's own notes cite,
would take a sentinel in the same cgroup with it and leave nothing to report.
A service in its own root-owned cgroup survives that,
and `systemd-oomd` is asked not to choose the service itself.
If the manager refuses the unit,
`systemd-run` is missing or hangs (it is ended after ten seconds),
or the manager accepts the unit and the sentinel never confirms (its program can still fail to execute after the start was accepted),
the unit is stopped and the sentinel starts beside the watcher instead,
`watch-start` prints a `POLICY NOT IN FORCE` line saying that a kill of the watcher's whole cgroup then ends both without a report,
and the start goes on.
After `systemctl stop` the unit's state is read whatever `stop` said
(a unit that already ended and was collected is no longer loaded,
which `stop` reports as a failure although nothing runs),
and only `inactive` or `failed` counts as gone.
If the unit cannot be shown to be gone,
a second line says so:
two sentinels may run,
and one death of the watcher may be reported twice.
`/usr/bin/env` reads a first argument that contains `=` as a variable assignment and has no option that prevents it
(`env: '__sentinel': No such file or directory`,
exit 127,
reproduced),
so a program path with `=` in it is not started as a service:
a `POLICY NOT IN FORCE` line says why and the sentinel starts beside the watcher,
which does not go through `env`.
The program path is passed to `systemd-run` as the bytes it has,
so a path that is not valid UTF-8 is not altered.
`systemd-run` and `systemctl` run with a cleared environment and a fixed search path (`/usr/bin:/bin`),
never the caller's.
It waits on the watcher through a process file descriptor (`pidfd_open`),
which refers to the process itself and so cannot be fooled by a reused pid;
where the kernel has none it polls the start time of the pid.
When the watcher is gone the sentinel waits one and a half seconds,
then reads the watcher's state:
if `watch-stop` removed it,
or a newer start replaced it,
the end was asked for and the sentinel exits quietly;
if it still names the dead watcher,
the death was not asked for and the sentinel reports it (kind `watcher died`) through the same channels,
naming both policies as not in force and telling to run `wg-quicker up` again,
and waits for the notification to be sent before it exits.
An unreadable state is reported too,
as unknown.
A shutdown or a reboot is not a policy failure:
it ends the watcher with everything else,
and the state in `/run` and the tunnel go with it.
When the verdict would be a report (the state still names the dead watcher,
or cannot be read)
the sentinel first asks whether the whole system is stopping,
and if it is,
it logs one line and reports nothing.
The system counts as stopping only when `systemctl show --property=ActiveState --value shutdown.target` prints `active` or `activating`.
`/run/nologin` is deliberately not a signal:
`man 8 shutdown` documents that with a time argument the file is created five minutes before the system goes down,
when a real death should still be reported,
and a stale or administrator-made file would hide every death.
A logout while the system keeps running is still reported:
the tunnel stays up with no watcher,
so the update traffic of later sessions is no longer exempted,
which is a real failure.
A probe that cannot tell (`systemctl` fails or runs past five seconds) counts as a system that keeps running,
so a failing probe never hides a death.
The case this can hide is a watcher that dies after `shutdown.target` became active,
which is already part of the shutdown.
A sentinel that was started is not taken for running because its process was created.
When it has loaded it writes a marker (the file `sentinel` in the state directory of the watcher)
with the watcher's pid,
its own pid and its own start time,
and `watch-start` waits up to two seconds for a marker that names this watcher
while the sentinel's process still exists with that start time.
If none appears,
`watch-start` prints a `POLICY NOT IN FORCE` line saying so and where to look
(`journalctl --unit=wg-quicker-exempt-sentinel-<pid>-<start time>`,
because the unit is collected when it ends and `systemctl status` shows nothing for it),
and it prints one line saying where the sentinel runs and whether it confirmed.
The marker is written to a new file that is renamed over `sentinel`,
so a link planted at that name is replaced and never followed.
The service start was exercised on the real host with a throwaway unit and a stand-in watcher process:
the sentinel ran in `unconfined_service_t` in `system.slice` with `ManagedOOMPreference=omit`,
wrote its marker,
and exited when the stand-in watcher died;
the unit that names the program directly failed with `status=203/EXEC` in the same test.
The limit is stated plainly:
if the sentinel is killed first,
or both die together,
nothing reports it;
with the service that now takes a kill of the watcher's whole cgroup out of that case,
and leaves a kill of everything on the machine,
a manager restart,
and a bug in the sentinel itself.
The bypass route watcher of `wg-quicker` has its own sentinel.

### Which marks stay

A mark is exemption from the tunnel,
so it stays attached only while the scan still selects its cgroup
(`src/application_marks.rs`).
A name-selected cgroup whose application has gone,
and an executable-owned cgroup whose program has exited,
lose their mark at the next reconciliation:
the links are dropped,
which is what the kernel unregisters the hooks with,
and the cgroup's traffic belongs in the tunnel like every other cgroup's.
Keeping the mark while the cgroup directory still existed
left the bypass on after the application had exited,
and the report that said the mark was not attached described the opposite of what was in force.
A cgroup is reported once when its name claims an application it does not hold,
and reported again after it regains the application and loses it later.

### Confirming a move

An adoption is confirmed from `/proc/<pid>/cgroup`,
and the answer is the scope's own cgroup path, compared by whole components:
a unit whose name contains the slice's name sits beside the slice and not in it,
so `.../app.slice/app-monochromatic\x2dupdates.slice-task.scope` is not a move
even though it holds every character of `app-monochromatic\x2dupdates.slice`.
A confirmation that asked whether the text contained the slice accepted that lookalike,
recorded a successful adoption,
and left the process outside the updates slice with no retry and no report.
A process that does not appear in the scope within the confirmation limit is a failure that is reported,
and `a_lookalike_cgroup_name_is_not_proof_that_the_process_moved` covers it.

### Verification

`mise run //package/wg/cli-quicker-exempt:test:unit` runs matcher,
policy,
adoption and connector tests on fake procfs trees and byte fixtures.
Each Pi rule has its own test:
`pi_agent_service_names_are_marked_by_name` for the unit name,
`scan_discovers_pi_agent_units` for the scan,
`socket_marked_cgroups_are_direct_named_app_slice_entries` for the predicate the attaching and the process watch share,
`pi_is_recognized_by_its_own_word_and_by_the_entry_it_runs` for the command line,
`a_process_in_a_pi_unit_is_covered_by_the_mark_attached_to_its_own_cgroup` for the gap this fixes,
`a_process_in_an_unmarked_service_is_still_reported_as_not_moved` for the report that must stay,
and `a_pi_process_outside_a_marked_unit_is_moved` for a Pi started anywhere else.
Each guard has a test that fails without it:
the netlink sender check,
the overrun path,
the start-time recheck,
the cleared environment and `PATH` of the client,
the kill of a hung client,
the descendant filters,
the retry and backoff,
the pass budget,
and the executable-name matching.
Two ignored tests need a live kernel and user manager:
the first starts a child and receives its exec event,
the second adopts a sleeping child into a throwaway slice through the real manager (it took 32 ms on the machine of the review).
Run them with `cargo nextest run --run-ignored only`.
Neither needs root on a kernel that allows the connector to unprivileged users.
Root's use of `setpriv` is not covered by a test,
because dropping to another user needs root.

## Persistence and replacement

Canonical cgroup path bytes are encoded as chunked hexadecimal path components.
This mapping is injective and avoids dots,
 which bpffs reserves for future extensions.
It also avoids collisions from replacing slashes with a delimiter.

A new four-link set is created under the tool's staging directory.
First attach renames the completed staging directory into place.
Replacement uses `renameat2(RENAME_EXCHANGE)`,
 then removes only the four old link pins.
A failed attach removes partial new pins and leaves the prior complete set active.

Attach and detach hold `/run/wg-quicker-exempt.lock` through the full lifecycle mutation.
The loader verifies that `/sys/fs/bpf` is a distinct bpffs mount before creating pins.

The descriptor-keeper fallback uses a two-phase handshake,
and both handshakes end at a limit:
the watcher's `READY` and `COMMITTED` and the holder's `READY` and `COMMITTED` are read through one bounded reader
(`src/line_ready.rs`),
a candidate that has not spoken within sixty seconds is refused and ended,
and a candidate that never ends is ended by the same bounded wait.
Without the bound a child that neither spoke nor exited held the lifecycle lock of `watch-start`,
and every later lifecycle command waited for ever for it.
The limit covers every byte of the line and not only the first one:
readiness and the line are separate steps,
a child can write part of a line and keep its stream open,
and a read to the newline then waits for the rest of a line that may never come.
A handshake that stops arriving, mid-line or before it, ends at the sixty seconds with the message it waited for,
and one line is capped at a few kilobytes,
so a stream that never sends a newline cannot fill memory either.
`line_ready_tests.rs` reads both handshakes' sources to keep the blocking read and the unbounded child wait out of them,
because a child that speaks in time proves nothing about the one that never speaks.
A candidate creates all links before the parent writes transition state and sends `COMMIT`.
The child records a commit marker,
 the parent validates and stops the prior holder,
 then finalizes active state.
Recovery retains the prior holder if an uncommitted candidate died
and adopts a live committed candidate after interruption.

A stop of the watcher or of a holder signals the process group it leads,
not only its process:
each one starts its `ip` calls, its `setpriv` and `busctl` clients and the leftover sweep's `systemctl`
as children of a session it leads,
and a signal to the leader alone left an in-flight manager client running,
able to change the user's manager after `watch-stop` returned.
The group is signalled only when the validated process leads it (its `pgrp` is its pid),
and the bounded wait that follows covers the group as well.
`keeper_process_tests.rs` runs a session leader with a child of its own and proves the group is empty afterwards,
which fails when only the leader is signalled.
Detach checks PID,
 process start time,
 full command,
 mark,
 and canonical cgroup before sending `SIGTERM`.

## Platform support

The instruction encoder currently supports little-endian Linux targets only.
A compile-time error rejects big-endian targets rather than producing incorrectly packed register fields.

The raw ABI has compile-time size and offset assertions for every `bpf_attr` arm used by the loader.
The instruction tests verify stack-store width and offset,
 null lookup control flow,
 helper ID,
 and helper-result verdicts.

## Linux 7.1 SELinux regression

Kernel `7.1.3-ogc5.1.fc44.x86_64` was observed returning `EINVAL` from `BPF_OBJ_PIN` after map creation,
 program load,
 and link
creation all succeeded.
The same failure affected maps,
 programs,
 and links at root and nested bpffs paths.

Upstream commit `28254722a459938d97150d3b0712b81e06d0645e` moves the SELinux `SBLABEL_MNT` check before
inode security-state initialization.
Kernel `7.2.0-ogc6.1.fc44.x86_64` was observed pinning all four links successfully.
The loader reports the affected regression commit when `BPF_OBJ_PIN` returns `EINVAL`,
 then automatically uses its
crash-recoverable descriptor keeper.
See [`doc/troubleshooting/linux-bpffs-selinux-object-pin-einval.md`](../../../doc/troubleshooting/linux-bpffs-selinux-object-pin-einval.md).

## Verification

Run unprivileged unit,
 build,
 and lint checks:

```sh
mise run //package/wg/cli-quicker-exempt:buildAndTest
```

Privileged ignored tests use disposable cgroups.
They cover all four protocols,
 process-exit persistence,
 repeated attach,
 exact detach,
 partial-link rollback,
 failed
candidate replacement,
 pre-transition parent death,
 committed and uncommitted transition recovery,
 removed cgroups,
 and wrong-owner cleanup retention.
Fallback-specific debug functional tests inject same typed `BPF_OBJ_PIN EINVAL` boundary,
so every kernel exercises descriptor-keeper behavior deterministically.
Separate public-CLI lifecycle coverage uses current kernel's native pin path.
Release builds omit injection seam.

## Caveats

- Only socket operations occurring after attachment receive the mark.
- Every new cgroup needs its own attachment.
   Detached application watcher owns Ghostty,
   Steam,
   Helium,
   Firefox release,
 Firefox Nightly,
   ChatGPT,
   Interpreter,
   Qure,
   and OdyTTY service enumeration plus future-cgroup inotify coverage.
   It periodically rescans Helium,
   Pale Moon,
   Firefox release,
 Firefox Nightly,
   ChatGPT,
   Interpreter,
   Qure,
   and OdyTTY processes.
   Firefox matching accepts exact `firefox` and `firefox-bin` names only under a `firefox` or `firefox-nightly` directory.
   The release launcher's `app-firefox@*.service` unit is marked while it holds one of those two images.
   Firefox ESR's directory and service remain tunnel-routed.
   ChatGPT matching accepts every executable inside the root-owned `/usr/lib/chatgpt` package tree,
   which includes the subprocess agents that application spawns.
   Interpreter matching accepts every executable whose name begins with `interpreter`,
   which also includes the same vendor's terminal agent.
   Qure matching accepts every executable whose name begins with `qure`,
   which covers its AppImage file,
   its mounted Electron image,
   its bundled pytest runner,
   and the agents its bundled CLI wrappers exec.
   Qure's Chromium-generic helpers,
   such as `chrome_crashpad_handler`,
   carry no such name and bypass only while they share a matched cgroup.
   OdyTTY matching accepts every executable whose name begins with `odytty`,
   which covers its AppImage file and the native image it runs.
- A `pi-<name>.service` unit is marked as a whole:
   every process in it leaves the tunnel,
   not only the Pi process,
   because the mark belongs to the cgroup.
   A unit whose name begins with `pi-` is marked whoever created it,
   because a unit name is a name the user can choose.
   The named desktop applications need their own program in the cgroup:
   a `app-firefox@*.service` or `app-com.mitchellh.ghostty@*.service` unit that holds something else keeps its traffic in the tunnel,
   and the watcher says which cgroup that was.
- Process discovery attaches entire current cgroup.
   If Helium,
   Pale Moon,
   Firefox release,
 Firefox Nightly,
   ChatGPT,
   Interpreter,
   Qure,
   or OdyTTY shares that cgroup with another process,
   every sibling's newly created sockets receive exemption until cgroup disappears or watcher stops.
- A newly started process-discovered application can create sockets before next periodic rescan,
   whose interval is 250 milliseconds.
   Existing applications are attached before watcher readiness.
- Detach removes only the four exact expected pin names and uses `rmdir`,
   so unrelated entries prevent directory removal.
- On affected kernels,
   persistence depends on the validated holder process.
   A holder crash safely detaches its links,
   and the
  next lifecycle command recovers its transition state.
