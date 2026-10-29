# iproute2 6.1.0 and Tini 0.19 help assumptions stop the Depot WG namespace consumer

## Symptom

The frozen WG privileged profile encountered independent prerequisite and isolation failures.
The complete receipts and failed attempts remain in
[`depot-rust-coverage-2026-10-18.evidence.json`](../audit/depot-rust-coverage-2026-10-18.evidence.json)
and its linked compressed raw archive.

- `42h9g5sczp`:
  `ip --help` printed valid usage and returned `255`.
  The adapter required status `0` and stopped before application tests.
- `8gts453b5g`:
  `mount` returned `32` with
  `mount: /sys/fs/cgroup: none already mounted or mount point busy.`
  No privileged tests ran.
  Independently,
  root-created receipt permissions caused artifact upload `EACCES`.
- `p81kntfxgs`:
  mounts succeeded and all privileged cases ran.
  Five passed and six failed during teardown with messages such as
  `keeper PID 103 command does not match owned process`.
- `4hddgrkrk2`:
  `/usr/bin/tini --help` returned `1` with
  `/usr/bin/tini: invalid option -- '-'`.
  No privileged tests ran.
- `506m7r84kk`:
  corrected prerequisites and a real namespace init passed all 11 privileged cases.
  The 15 ordinary cases were filtered skips in this run.

## Root cause

These failures have different boundaries.
None of the prerequisite-only failures establishes a BPF product failure.

In iproute2 tag `v6.1.0`,
commit `83699d3d5077bbb3957030938f746601317767d2`,
`ip/ip.c:63` defines the usage printer and `ip/ip.c:81` explicitly exits with `-1`:

```c
/* ip/ip.c:63 and 81, iproute2 v6.1.0, abridged. */
static void usage(void)
{
    /* Print Usage: ip [ OPTIONS ] OBJECT { COMMAND | help }. */
    exit(-1);
}
```

The observed process status is `255`.
Accepting that status requires checking the expected usage text;
it does not justify ignoring functional command failures.

Tini source `src/tini.c:248` documents short-only `-h`.
The inspected source commit is `369448a167e8b3da4ca5bca0b3307500c3371828`.
Its parser at `src/tini.c:327` uses `getopt`,
with a dedicated help case:

```c
/* src/tini.c:327, abridged. */
while ((c = getopt(argc, argv, OPT_STRING)) != -1) {
    switch (c) {
        case 'h':
            print_usage(name, stdout);
            *parse_fail_exitcode_ptr = 0;
            return 1;
    }
}
```

The Tini invocation must use `-h`.
The packaged version under test was `0.19.0-1`.

The mount diagnostic is less specific than an underlying kernel-cause conclusion.
In util-linux source commit `d427f58b7a22b960ccc8d02ce30c58bdbe4286d8`,
`libmount/src/context_mount.c:1827` handles `EBUSY` and falls back at line `1842`:

```c
/* libmount/src/context_mount.c:1827 and 1842, abridged. */
case EBUSY:
    if (!*buf)
        snprintf(buf, bufsz, _("%s already mounted or mount point busy"), src);
    break;
```

This identifies the emitting diagnostic,
not the precise reason this kernel rejected a fresh cgroup2 mount.
The verified alternative is a writable private bind of the existing cgroup2 mount.
`sys-utils/mount.8.adoc:239` documents the bind/remount operation
and distinguishes mount-entry flags from filesystem-wide flags.

The six teardown failures were introduced by running Node as PID 1 in the new PID namespace.
The source boundary matters:
frozen `package/wg/cli-quicker-exempt/src/keeper_process.rs:53` reads `/proc/<pid>/cmdline`,
and `keeper_process.rs:82` rejects a command mismatch:

```rust
//! Excerpt of the frozen process identity rejection.
// package/wg/cli-quicker-exempt/src/keeper_process.rs:82, abridged.
if observed_command != expected {
    return Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!("{role} PID {pid} command does not match owned process"),
    ));
}
```

The initial inference that these were product lifecycle failures was unproven.
The controlled retry established the harness defect:
the Node-init positive control retained orphan PID `9` in state `Z`,
while the Tini-init control observed orphan PID `10` absent.
All 11 unchanged application cases then passed.
The product source and identity guards were not changed.
Tini's `README.md:158` documents PID 1 reparenting and zombie reaping;
`src/tini.c:548` implements the reaper:

```c
/* src/tini.c:553, reaper loop, abridged. */
while (1) {
    current_pid = waitpid(-1, &current_status, WNOHANG);
    /* Handle each exited child before checking again. */
}
```

## Verification

Application source is `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`.
The successful executable harness is research commit `d569ca3b16f3e7ef66519fd86939728df7cebcfc`.
Its prepared image digest is
`sha256:1c2b469a9d7fae8bc364e6392f771c28565647a04c40847f345bc140dfce4e88`.
Observed tools include iproute2 `6.1.0`,
libbpf `1.3.0`,
Tini `0.19.0-1` and Linux `6.12.109+`.

The research worktree owns the runnable task:

```sh
# Research branch package/dev-script/task-util/mise.toml, not the main checkout.
mise run //package/dev-script/task-util:research:rust:wg
```

Use its declared ephemeral Depot job:
eight CPUs,
32 GiB,
ten-minute job limit,
eight-minute step limit.
It builds the owning CLI and test executable,
enters private mount/PID/network/cgroup namespaces,
uses Tini as PID 1,
admits actual writable cgroups and bpffs,
then runs ignored Nextest cases with default concurrency.

Working catalog:
verified usage-status/header check,
Tini `-h`,
private writable bind,
root receipt mode `0644`,
both init controls and all 11 privileged cases.
Failing catalog:
unconditional zero-status help checks,
fresh cgroup2 mount,
unreadable root receipts and initless daemon teardown.
Each failure retains its own run ID and diagnostic.

## Verified workarounds

The research harness contains these consumer patches:

```diff
# package/dev-script/task-util/bin/rust-wg-privileged.mjs, abridged.
- run('owned-init-help', '/usr/bin/tini', ['--help']);
+ run('owned-init-help', '/usr/bin/tini', ['-h']);
```

The usage exception applies only to the exact ip help probe.
Functional commands remain strict.
Tini adds an admitted package and process supervisor;
it restores ordinary orphan reaping rather than weakening product identity checks.
Root receipt permissions allow artifact collection in the disposable job;
they are not permissions for real user data.

The mount patch uses `mount --bind` followed by `mount --options remount,bind,rw`
inside the private namespace.
It depends on an accessible existing cgroup2 mount and a verified writable `cgroup.procs` fixture.
It does not prove a fresh cgroup2 mount is supported.
Successful cleanup recorded an empty owned-cgroup inventory and status `0`
for both bpffs and cgroup-bind unmounts.
The implemented private-bind lazy-detach fallback was not used in the successful run.

## What does not work

Treating every nonzero help status as a missing dependency stopped before compilation.
Installing Tini while retaining unsupported `--help` still stopped before tests.
Mount admission alone did not provide correct process lifecycle semantics.
Suppressing the six identity failures or loosening command validation was not used.
The earlier mount and artifact failures remain separate incidents.

## Upstream filing decision

The `.out-of-scope/` inventory has no matching iproute2,
util-linux or Tini exemption.
Read-only tracker searches included iproute2 help/exit queries and
`repo:krallin/tini "help option"`.
The latter returned no matches;
an empty search does not establish an upstream support policy.

1.  Upstream fault is not established:
    the observed help behavior is explicit in source,
    and the lifecycle failure used a harness without an init reaper.
2.  Consumer corrections are demonstrated.
    No claim that upstream cannot change these APIs is made.
3.  The inspected Tini README supports init/reaping;
    util-linux documents the bind/remount alternative.
4.  Filing-specific contributor-policy review was not completed,
    because no upstream defect or filing is proposed.
5.  No maintainer acceptance or rejection is inferred from empty tracker searches.
6.  The consumer patches and actual positive/negative controls were executed.
    An upstream prototype is not required because the upstream-fault gate failed.

Nothing to add upstream.
No issue,
comment or message was sent.
