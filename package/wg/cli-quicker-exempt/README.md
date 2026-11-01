# wg-quicker-exempt

Marks sockets from selected cgroups so policy routing can keep application traffic outside a WireGuard tunnel.

## Why

`wg-quicker` installs a dedicated bypass table and an `ip rule` matching an exemption mark.
This loader applies that mark to sockets created in Ghostty,
 Steam,
 Helium,
 Pale Moon,
 Firefox Nightly,
 ChatGPT,
 Interpreter,
 and Qure cgroups without enumerating their destination IPs.

It also moves update traffic out of the tunnel by process.
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
 Firefox Nightly,
 ChatGPT,
 Interpreter,
 and Qure targets without attaching:

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
  `mise run` of the fetching tasks (`toolchain:sync`,
  `update`,
  `install`,
  `outdated`,
  their `:global` forms,
  and the aliases `bootstrap` and `prepare:android`),
  `mpm-shadow install`,
  `i`,
  `update` and `outdated` with or without `-g`,
  the `mpm` shadow,
  and `node` running the toolchain `sync.ts` or `update.ts`.
  A process matches by the file name of its resolved executable (`mise`,
  `mpm-shadow`,
  `mpm`,
  `node`)
  together with its command-line words.
  The first command-line word alone never matches,
  because `exec -a mpm /usr/bin/curl` sets it freely;
  it counts only for `mpm-shadow` started as `mpm`.
  Tests check the list against the generated `mise.toml` and the `mpm-shadow` command grammar,
  and fail when a fetching task or verb is missing.
  The names `toolchain:install` and `toolchain:outdated` with their `:global` forms are the spellings after the
  toolchain cutover;
  they are on the list before `mise.toml` has them,
  and the test fails the day one of them appears,
  so the pending list shrinks with the cutover.
- The toolchain paths:
  An executable under the watched user's `.local/share/monochromatic-dev/toolchain/` or `toolchain-global/`
  is moved the same way.
  The home comes from the system user database for the watched uid,
  in the spelling the user database gives and its resolved spelling.
  A custom `XDG_DATA_HOME` is not seen by the root watcher.
  A root that is a link,
  or that sits below one,
  also counts as the directory it points to,
  resolved when the watcher starts,
  because a resolved executable path names that place,
  but only when that place lies strictly inside the user's resolved home:
  not the home itself,
  not the filesystem root,
  not another user's home,
  and not through a home that resolves to the filesystem root.
  The directory is writable by the user while the watcher runs as root,
  so a link to anywhere else would make every program there leave the tunnel;
  the reviewer's probe linked the root to `/` and `/usr/bin/curl` matched.
  Such a root is left out of the matching and reported with `POLICY NOT IN FORCE` in the watcher log,
  which `up` relays;
  a root that cannot be resolved for any reason other than not existing,
  such as a link loop,
  is reported the same way.
  A link that is dangling is the same as a root that does not exist yet.
  A link created later is not seen until the next start.
  The start notice lists installed tools only from real directories that stay strictly inside the home.
  `src/install_roots_tests.rs` has the accepted and refused fixtures:
  a link inside the home,
  to `/`,
  to the home itself,
  to another user's home,
  a link loop,
  a dangling link,
  an unresolved home and a home that resolves to `/`.
  The roots cover every program the toolchain installed,
  so every script and dev server run by the toolchain's node leaves the tunnel too,
  not only updates;
  the start notice lists what is installed now.

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
Delivery is lossy when the watcher falls behind:
an overrun is reported and every process is rescanned,
and a full scan also runs every two seconds of wall time as the backstop,
whatever the number of loop wakeups.

### Moving a process

The watcher does not write `cgroup.procs`.
It asks the user's systemd manager to adopt the pids
(`StartTransientUnit` with `PIDs` and `Slice` through `busctl`,
run as the watched user through `setpriv` when the watcher is root).
systemd's cgroup delegation document states the single-writer rule:
each cgroup has one writer,
and a writer that ignores it leaves software fighting over ownership.
A direct write would put processes into a cgroup the manager never created,
which it would not garbage-collect and whose pid-to-unit map would go stale.
The manager creates a scope per adoption and removes it when the processes exit.
The `busctl` client is used instead of hand-written D-Bus wire code in a root process.

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
The scope is named `wg-quicker-update-<pid>-<start time>.scope`,
so a reused pid never collides with a scope that still holds the old process's descendants.
A process of the watched user that the watcher may not inspect is reported once,
and is not mistaken for one that exited.
A process in another pid namespace (a container or a sandbox) is reported and left alone,
and so is a process whose pid namespace link cannot be read,
because it cannot be told apart from one.
A watcher that cannot read its own pid namespace says so at start.

The new scope carries the limits of the cgroup the process left,
so moving a process for routing never loosens a cap such as the CPU quota of `claude-code-bash`.
The limits are `cpu.max` as `CPUQuotaPerSecUSec`,
`memory.max` as `MemoryMax`,
`memory.high` as `MemoryHigh`,
`memory.swap.max` as `MemorySwapMax` and `pids.max` as `TasksMax`.
The smallest finite value on the cgroups between the old place and the nearest ancestor it shares with the new place is used;
limits of the shared ancestors keep applying by themselves.
A limit file that cannot be read stops the move with a report,
and a move that fails is never tried again without the limits.
The cap becomes the moved processes' own cap,
no longer shared with the processes that stayed in the old cgroup.
CPU weight,
I/O limits and cpusets are not copied.

The manager can only move processes inside its own subtree,
`user@<uid>.service`.
A process in a login session scope (`session-N.scope` under `user-<uid>.slice`),
or the main process of a service,
is not moved,
and the report says so by name.

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
the root is opened without following a link,
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
A watcher killed by a clean system shutdown may be reported if the sentinel outlives it by a moment;
the notification then fails harmlessly.
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

### Verification

`mise run //package/wg/cli-quicker-exempt:test:unit` runs matcher,
policy,
adoption and connector tests on fake procfs trees and byte fixtures.
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

The descriptor-keeper fallback uses a two-phase handshake.
A candidate creates all links before the parent writes transition state and sends `COMMIT`.
The child records a commit marker,
 the parent validates and stops the prior holder,
 then finalizes active state.
Recovery retains the prior holder if an uncommitted candidate died
and adopts a live committed candidate after interruption.
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
   Firefox Nightly,
   ChatGPT,
   Interpreter,
   and Qure service enumeration plus future-cgroup inotify coverage.
   It periodically rescans Helium,
   Pale Moon,
   Firefox Nightly,
   ChatGPT,
   Interpreter,
   and Qure processes.
   Firefox Nightly matching accepts exact `firefox` and `firefox-bin` names only under a `firefox-nightly` directory.
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
- Process discovery attaches entire current cgroup.
   If Helium,
   Pale Moon,
   Firefox Nightly,
   ChatGPT,
   Interpreter,
   or Qure shares that cgroup with another process,
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
