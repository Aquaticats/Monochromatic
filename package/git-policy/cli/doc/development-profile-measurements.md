# Development profile measurements for the Git wrapper

Every agent that works on this package builds it in a worktree of its own,
and a build tree of this package was between 5 and 8 GiB earlier in November 2026,
so the size of one tree is what limits how many agents can work at once
(the brief that commissioned this page measured the volume filling from 251 GiB to 107 GiB while
about thirty-five agents ran).
This page holds the measurements behind the development and test profile of
`package/git-policy/cli/Cargo.toml`,
so a later change can be argued with them instead of re-deriving them.

## What was measured

Measured 2026-11-13 with the toolchain this repository pins for that day
(`cargo 1.101.0-nightly`,
`rustc 1.101.0-nightly`),
one clean tree at a time,
on the branch `perf/git-policy-cli-build-tree-of-the-wrapper-package-is-measured-and-made-smaller`.

Each round ran `cargo clean` first,
then the builds an agent of this package runs:

- `mise run //package/git-policy/cli:native:build`,
  which is
  `cargo build --locked --offline --bin monochromatic-git-policy-cli`.
- the library test compile,
  `cargo nextest list --locked --offline --lib --message-format json`,
  which compiles the whole `--lib` test binary and runs nothing.
- the consumer test compile,
  `cargo nextest list --locked --offline --test native_binary --message-format json`.
- a backtrace probe:
  one failing equality assertion and one `panic!` in a temporary
  `tests/backtrace_probe.rs`,
  run with `RUST_BACKTRACE=1` and then removed.

Every step ran in its own `systemd-run --user --scope` with `CPUQuota=600%`,
`MemoryMax=12G`,
`TasksMax=512` and `IOWeight=1`,
and every run was a foreground command with a bound.

The Clippy pass (`native:clippy`) is measured separately,
in the section "What the Clippy pass adds":
it takes tens of minutes on a machine where other agents run the same runs,
and its artifacts are metadata only,
so including it in every round would have made the comparison cost more than it explained.

### How the sizes were taken

Host configuration on this machine sets `build.build-dir` to
`/var/mnt/encrypted/cargo-build/{workspace-path-hash}`,
so a build tree is two places,
not one:

- the worktree's `package/git-policy/cli/target/`,
  which holds final artifacts,
  and
- the workspace's own build directory,
  which holds intermediates.

`cargo metadata` reports only the first,
so the build directory was read from the test binary path
`cargo nextest list` prints:
`<build dir>/debug/build/monochromatic-git-policy-cli/<unit>/out/<binary>`.
`cargo clean` deletes both places,
and the trees below are the two added together.

Sizes are apparent bytes (`du` and `st_size`),
except where an inode count is given.
Debug information was measured per file with `size --format=sysv`,
summing every section whose name starts with `.debug`,
and is quoted against the sum of all sections in that file.
The final binary is not hardlinked into the build directory on this machine:
the worktree copy and the build directory copy are two files with separate inodes,
so one control binary costs its size twice.

## Where the weight is, with Cargo's defaults

The tree after the three builds is 5885 MiB:
build directory 5539 MiB and `target/` 346 MiB.
By kind,
in the build directory:

- incremental compilation data:
  2813 MiB
- executables:
  1575 MiB,
  of which the library test binary is 607 MiB,
  the consumer test binary 328 MiB,
  the control binary a copy of the worktree one at 346 MiB,
  and the proc-macro shared objects about 270 MiB in all
- dependency rlibs:
  531 MiB
- this crate's own rlib:
  381 MiB
- rmeta for every crate:
  228 MiB
- build script outputs:
  8 MiB
- dep-info files:
  2 MiB

The largest single files are the library test binary (607 MiB),
three incremental `dep-graph.bin` files (491,
229 and 209 MiB),
this crate's rlib (381 MiB),
the two copies of the control binary (346 MiB each),
and two incremental `query-cache.bin` files (338 and 125 MiB).

Debug information is most of every executable.
Measured with `size --format=sysv`,
against the sum of that file's sections:

- the control binary holds 270 MiB of `.debug*` sections of 318 MiB,
  85%
- the library test binary 443 MiB of 544 MiB,
  81%
- the consumer test binary 295 MiB of 316 MiB,
  93%
- together,
  1278 MiB of the 1575 MiB of executables

So two settings decide most of the tree:
how much debug information is emitted,
and whether incremental data is kept.

## Every candidate, same builds, one clean tree each

The tree is the build directory plus `target/`.
Wall and CPU time are the `cargo build` step and the library test compile step
(`/usr/bin/time -v`,
CPU is user plus system),
and the CPU figure is the one to compare,
because wall time on this machine also holds whatever the host was doing.

- Cargo's defaults,
  `debug = full`,
  `incremental = true`:
  tree 5885 MiB,
  build 124.8 s wall and 512.5 s CPU,
  library test compile 105.1 s wall and 144.0 s CPU.
- `[profile.dev] debug = "line-tables-only"`:
  tree 3516 MiB,
  a fall of 40%.
  Build 114.6 s wall and 466.3 s CPU,
  library test compile 67.8 s and 96.9 s.
- `[profile.dev] debug = false`:
  tree 2648 MiB,
  a fall of 55%.
  Build 108.7 s wall and 440.2 s CPU,
  library test compile 81.0 s and 91.5 s.
- `[profile.dev.package."*"] debug = false`,
  which leaves this crate at full debug information:
  tree 4691 MiB,
  a fall of 20%.
  Build 108.5 s wall and 423.6 s CPU,
  library test compile 101.2 s and 133.0 s.
- `[profile.dev.package."*"] debug = "line-tables-only"`:
  tree 5349 MiB,
  a fall of 9%.
  Build 114.4 s wall and 439.3 s CPU,
  library test compile 90.0 s and 126.4 s.
- `[profile.dev] incremental = false`:
  tree 2885 MiB,
  a fall of 51%.
  Build 92.3 s wall and 395.0 s CPU,
  library test compile 70.3 s and 106.7 s.
- `[profile.dev] split-debuginfo = "unpacked"`:
  tree 4980 MiB,
  a fall of 15%.
  Build 91.3 s wall and 409.5 s CPU,
  library test compile 77.5 s and 111.1 s.
- `[profile.dev] split-debuginfo = "packed"`:
  tree 5344 MiB,
  a fall of 9%.
  Build 94.0 s wall and 383.8 s CPU,
  library test compile 81.4 s and 116.9 s.
- `[profile.dev] debug = "line-tables-only"` with `incremental = false`,
  the setting this package now uses:
  tree 1562 MiB,
  a fall of 73%.
  Build 83.2 s wall and 377.8 s CPU,
  library test compile 50.3 s and 77.3 s.

The two split forms do not shrink the tree:
they move debug information out of the executables
(the four large executables fall from 1627 MiB to 601 MiB)
into separate files (795 MiB for `packed`,
641 MiB for `unpacked`),
and cost the same compile time.
Reducing debug information for dependencies only is the weakest lever,
because debug information in the executables and incremental data of this crate,
not in dependencies,
are where the bytes are.

## What each candidate costs a debugger

The probe ran the same two tests under every candidate.
With Cargo's defaults,
with `line-tables-only`,
with either split form,
and with incremental off,
every backtrace frame carries its location:

```txt
   2: backtrace_probe::a_panic_carries_a_message
             at ./tests/backtrace_probe.rs:14:5
```

With `debug = false`,
frames keep their names and lose their locations:

```txt
   2: backtrace_probe::a_panic_carries_a_message
```

Both keep the panic message's own location,
`panicked at tests/backtrace_probe.rs:14:5`,
because that comes from the call site the panic machinery records rather than from debug information.
So `debug = false` keeps what a failing assertion prints and drops what a call chain shows.
It also drops every local variable,
which `line-tables-only` drops as well:
Cargo's own profile reference describes the difference as
"minimal amount of debug info for backtraces with filename/line number info,
but not anything else,
i.e. no variable or function parameter info"
(<https://doc.rust-lang.org/cargo/reference/profiles.html>).

Regarding `split-debuginfo`,
the frames still resolve while the separate debug files sit beside the binary.
A binary copied out of the tree without them loses file and line.

Incremental compilation does not change a backtrace.

## What the Clippy pass adds

On 2026-11-12,
with Cargo's defaults,
the tree after the same three builds and then
`mise run //package/git-policy/cli:native:clippy` was 6668 MiB against 5885 MiB for the three builds alone,
so the Clippy pass left about 780 MiB,
which are rmeta files of every crate compiled by the Clippy driver.
With the setting below,
measured 2026-11-13 on one clean tree,
the same three builds and then Clippy leave 1719 MiB:
build directory 1585 MiB and `target/` 134 MiB,
of which the Clippy pass itself is 157 MiB.
Its own wall time is dominated by waiting for Cargo's package cache lock while other agents build,
so it is not a number to compare candidates with.

## What a run sees today beside its own change

The full library suite on this host,
2026-11-13,
gives the same result under Cargo's defaults and under the
setting below:
7550 tests run,
7540 passed,
10 failed,
22 skipped,
and the same ten tests fail both times.
The consumer suite (`native_binary`) fails six of its tests both times as well,
each after a 60 s ceiling
in `commit_timing` and `fault_remote`.
None of the sixteen is about the profile,
so a run that meets them is not meeting a change of this file.

## What else reads these artifacts

- The release profile is untouched:
  `[profile.release]` keeps `panic = "unwind"`,
  `overflow-checks = true` and `strip = "symbols"`,
  and `[profile.release.build-override]` keeps `opt-level = 3`,
  which
  `package/file-enforcer/config` owns and this change does not move.
  `[profile.dev.build-override] opt-level = 3` is also untouched.
- `package/git-policy/cli.fuzz` has profiles of its own and is its own workspace root,
  and Cargo reads profiles only from the workspace root of the build,
  so its `[profile.dev] panic = "unwind"` and its build override are unaffected.
  There is no `*.mutation` or `*.bench` sidecar beside this package,
  and mutation campaigns are paused.
- `native-test-base.Containerfile` only installs tools;
  the gate compiles this package inside the container
  from a bind mount of the worktree,
  so the gate builds with these settings too and its build gets cheaper.
  The image itself does not change.
- `cargo-shadow`'s clean build folders pass reads the profile directories
  `build`,
  `deps`,
  `incremental`,
  `.fingerprint` and `examples`
  (`package/cargo-shadow/cli/src/lean_build_trees.rs`),
  and lists each with a reader that treats a missing directory as no entries.
  With `incremental = false` the `incremental` directory is simply never created,
  which that pass already handles,
  and no other directory changes shape.

## The package cache lock, which is not a property of the tree

Separate from the tree,
Cargo's lock on its package cache held builds of this package for tens of minutes
while other agents built.

Evidence from 2026-11-12:
a `cargo metadata` of another worktree of this package sat in state `D` for 45 minutes with
`/proc/3672158/wchan` reading `wait_for_commit` (a btrfs transaction commit),
and `/proc/locks` showed it holding the `FLOCK WRITE` lock on inode 260928,
which is `/home/user/.cargo/.package-cache`,
with twenty cargo processes queued behind it.
Two of this run's own steps were ended by their bounds while waiting for that lock,
one at 40 minutes and one at 60 minutes,
with `Blocking waiting for file lock on package cache` on stderr and no work done.

Measured 2026-11-13,
holding that lock for 15 s with `flock` on purpose,
one arm at a time,
running one `cargo build --locked --offline` that has nothing to compile:

- shared cargo home,
  nobody holding the lock:
  finished in 0.3 s,
  no wait
- shared cargo home,
  lock held:
  14.8 s,
  waited for the holder,
  and printed
  `Blocking waiting for file lock on package cache`
- shared cargo home with `-Z fine-grain-locking`,
  lock held:
  14.8 s,
  the same wait
- a cargo home of the run's own,
  with `registry` and `git` linked to the shared ones and the same
  `config.toml`,
  lock held:
  no wait,
  and the build left the artifacts it found

So `--offline --locked` does not remove the wait,
and neither does `-Z fine-grain-locking`
("Use fine grain locking instead of locking the entire build cache",
<https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#fine-grain-locking>):
measured against the same held lock it waited exactly as long as the plain build,
so that flag refines other locks and not this one.
The flag did not change what is built:
with it the build compiled nothing and wrote the same binary.

A cargo home of the run's own does remove the wait,
and the Cargo book describes the layout a build
may share (<https://doc.rust-lang.org/cargo/guide/cargo-home.html>).
What it does not yet do is leave the same artifacts under the wrapper's own `cargo`:
with that launcher in front,
switching homes recompiled 266 units and produced a binary with another
sha256,
while the real cargo at the same commit,
given the same two homes,
produced one binary with the
same md5 in all three arms.
That points at the launcher's per-home stamp rather than at Cargo,
so a cargo home per run is worth adopting only after its owner says why the bytes move.
Until then a run can only bound each step,
retry it,
and say how long it waited.

## What would shrink the total further than a profile can

- One build tree shared by agents that start from the same commit.
  Cargo keys `build.build-dir` on the workspace path
  (`doc/decision/cargo-build-dir-host-cache.md`),
  so two worktrees never share one today.
  Sharing needs every agent of a commit to compile in the same worktree,
  or a
  `build.build-dir` that ignores the worktree path,
  which the host configuration cannot express.
  Nothing in this repository does it today,
  and the write side is the obstacle:
  two builds in one directory serialize on Cargo's build directory lock.
- A tree removed when its run ends.
  `cargo-shadow lean` already removes a build tree only after proving nothing uses it
  (`doc/handover/cargo-shadow-lean-build-folders.md`),
  and it removed nothing on 2026-11-12 because every tree belonged to a working run.
  A per-run tree that ends with its run needs the run to say when it ends,
  which the driver does not do today:
  a run that ends its turn is closed about two minutes later,
  and the tree is left.
- A shared cache of compiled dependencies.
  The dependency rlibs and rmeta are 759 MiB of the 5885 MiB tree,
  so sharing them can at most remove that,
  and it needs identical profiles,
  features,
  target and
  compiler for every consumer,
  plus a way to hand the artifacts to a build that did not compile them.
  This machine already shares `build.build-dir` per workspace,
  not per dependency,
  and nothing here runs `sccache` or a similar cache.
  The profile change above already removed the larger halves,
  so this is the smallest of the three.

## The setting this package now uses

```toml
[profile.dev]
debug = "line-tables-only"
incremental = false
```

It is the smallest tree measured,
1562 MiB against 5885 MiB,
and it is also the fastest of the
candidates for every step an agent runs,
because less debug information is emitted and no incremental
data is written.
It keeps file and line in every backtrace frame,
which is the part a person or an agent reading a
failure uses.
It costs the rebuild after an edit:
with incremental compilation off,
one edit rebuilt the control binary
in 23.5 s wall and 26.9 s CPU,
against a few seconds with incremental data present,
and that is the price of 2813 MiB that no longer exists.

The next best single setting is `debug = false`,
at 2648 MiB,
which loses file and line in every backtrace frame and was rejected for that.
`incremental = false` alone,
at 2885 MiB,
keeps full debug information and costs nothing in a backtrace,
so it is the setting to prefer instead if the line tables ever become too expensive.
