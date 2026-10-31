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

It also marks the toolchain updates slice,
 `app-monochromatic\x2dupdates.slice`,
 so toolchain sync and update runs and everything they start use the physical route.
The slice name is a contract with the TypeScript and Rust toolchain entry points,
 which re-run themselves in a `systemd-run --user --scope` below it.
The `app-` prefix puts the slice directly under `app.slice`,
 the only directory the watcher scans and watches,
 and `\x2d` is the escaped literal dash.
A cgroup program applies to every descendant cgroup,
 so the watcher attaches the slice once and each per-run scope inherits the mark.
`list-targets` lists the slice while it exists.

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
