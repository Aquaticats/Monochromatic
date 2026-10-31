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
  Tests check the list against the generated `mise.toml` and the `mpm-shadow` command grammar,
  and fail when a fetching task or verb is missing.
- The toolchain paths:
  An executable under the watched user's `.local/share/monochromatic-dev/toolchain/` or `toolchain-global/`
  is moved the same way.
  The home comes from the system user database for the watched uid,
  in the spelling the user database gives and its resolved spelling.
  A custom `XDG_DATA_HOME` is not seen by the root watcher.

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
are trusted.
Delivery is lossy when the watcher falls behind:
an overrun is reported and every process is rescanned,
and a full scan also runs every two seconds as the backstop.

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

The matched process moves together with its live descendants of the same user.
The pid start time is checked again before the request,
so a reused pid is not moved.
After the manager accepts the job,
the watcher waits up to two seconds until `/proc/<pid>/cgroup` names the slice.

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
A line that starts with `wg-quicker-exempt: POLICY NOT IN FORCE` names the process-watch policy and what failed:
the slice cannot be created,
the process-event subscription failed (entry points are then found only by the periodic scan),
events were dropped,
reading events failed,
a matched process is outside the user manager's subtree,
or the adoption failed.
A failing BPF attach or socket-marking rescan is reported under the socket-marking policy.
No failure invents a fallback route.

### Verification

`mise run //package/wg/cli-quicker-exempt:test:unit` runs matcher,
policy,
adoption and connector tests on fake procfs trees and byte fixtures.
Two ignored tests need a live kernel and user manager:
the first starts a child and receives its exec event,
the second adopts a sleeping child into a throwaway slice through the real manager.
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
