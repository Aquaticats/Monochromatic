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
- `native-test-base.Containerfile` installs only tools and caches the pinned browserslist data tarballs,
  neither of which touches a Cargo profile,
  and the image the gate builds is made from a copy of the sources:
  `bin/test-native-container.ts` copies them into a disposable build context
  (`copySourceSnapshot`)
  and `bin/native-test-containerfile.ts` copies that into the image with `COPY`,
  so nothing of the worktree is mounted.
  Two environment settings of that test image override this manifest's profile for every step it runs:
  `CARGO_PROFILE_DEV_DEBUG=0` and `CARGO_INCREMENTAL=0`
  (`bin/native-test-containerfile.ts`,
  in the `environment` list of the one `ENV` line that
  `builderPreludeLines` emits in `bin/native-layered-build.ts`).
  They predate this change,
  so the gate already compiled without debug information and without incremental data,
  and this change leaves its build as it was.
  Measured 2026-11-13 on a dep-free crate whose integration test panics,
  by the `.debug*` sections of the test binary:
  Cargo's defaults leave 5.97 MiB,
  this package's new `[profile.dev]` alone leaves 5.92 MiB,
  the gate's two variables with Cargo's defaults leave 0.00 MiB,
  and they leave 0.00 MiB with this package's new `[profile.dev]` as well,
  writing a binary with the same name as the arm before it.
  So a backtrace inside the gate names frames and gives no file and line,
  because `debug = 0` leaves no line tables to read,
  whatever this package's development profile says.
- `cargo-shadow`'s clean build folders pass reads the profile directories
  `build`,
  `deps`,
  `incremental`,
  `.fingerprint` and `examples`
  (`package/cargo-shadow/cli/src/lean_build_trees.rs`),
  and lists each with a reader that treats a missing directory as no entries.
  With `incremental = false` Cargo still creates the `incremental` directory and leaves it empty
  (measured 2026-11-13 in this package's own build directory:
  no entries,
  no bytes),
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
  a run that ends its turn is closed within one look of a few hundred milliseconds once nothing else holds it
  (about two minutes before the driver's idle window defaulted to 0 on 2026-11-18),
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

## Memory per build, measured 2026-11-25

Status:
measured here on 2026-11-25 by one agent,
unapproved and not final.
It tests the memory questions the research of the same day left open
(`rust-build-peak-across-worktrees-research-2026-11-25`,
in that round's folder)
and changes no profile setting,
no `Cargo.toml` and no launcher;
the numbers are for its owner to choose from.

Every number here is one build of this package in one worktree,
never a test run,
and the default command is the one a round runs after an edit:
`cargo test --locked --offline --lib --no-run`,
which compiles the non-test library and the library test harness and then links them.

### What one arm is

Each arm ran in its own `systemd-run --user --scope` in `agent-work.slice`,
with `MemoryMax=12G`,
`CPUQuota=1200%`,
`TasksMax=1024`,
`IOWeight=1` and `RuntimeMaxSec=1800`.
Before every arm the crate root `src/native/lib.rs` was touched,
so only this crate's own units rebuilt,
as after an edit in a round.
`nproc` inside such a scope reads 12,
so Cargo's default job count is 12 in the control builds.

A memory gate ran before each arm and no arm started until it passed:
memory pressure's `some avg10` at or below its `avg60`,
and `MemFree` plus the part of the ZFS ARC's `size` above `c_min` at least 7,080 MiB.

Three counters are recorded per arm:

- the scope's `memory.peak`,
  which counts page cache as well as anonymous memory and is not the number to compare;
- the maximum of `anon` in the scope's `memory.stat`,
  sampled once a second,
  which is the number compared here;
- the largest resident size of any `rustc` in the scope,
  from the same once-a-second sample of its processes.

Wall time is the scope's own lifetime.
Because this host builds other agents' code at the same time,
each arm is reported as a ratio to the control builds run immediately before and after it,
with the range that dividing by each of the two gives.
Sampling once a second can miss a spike shorter than a second,
so every maximum here is a lower bound;
`memory.peak` is the kernel's own high-water mark and is exact.

### The control's spread

Nine control builds ran between the other arms.
Their `anon` maxima are 5,663,
6,244,
6,275,
6,541,
6,595,
6,703,
6,786,
6,789 and 6,807 MiB,
with a median of 6,595 MiB,
so one build of one unchanged crate can sit 20% above another on this host.
`memory.peak` moves with it,
from 5,962 to 7,159 MiB,
the largest `rustc` from 5,789 to 6,875 MiB,
and wall time from 82.7 to 148.9 s.
A difference smaller than 1.20 is no difference these builds can show.

### Every arm, as a ratio to the control around it

- `-j 4`,
  twice:
  `anon` 6,604 MiB (1.010 times the mean of the controls either side,
  range 0.970 to 1.052) and 6,519 MiB (1.016 times,
  range 0.988 to 1.044),
  `memory.peak` 6,882 and 6,823 MiB,
  the largest `rustc` 6,667 and 6,579 MiB,
  wall 101.4 and 132.3 s.
  A third of the default job count changed nothing outside the spread.
- `-j 1`,
  twice:
  `anon` 5,757 MiB (0.852 times,
  range 0.846 to 0.859) and 5,641 MiB
  (0.866 times,
  range 0.831 to 0.903),
  `memory.peak` 5,985 and 5,851 MiB,
  the largest `rustc` 5,825 and 5,756 MiB,
  wall 145.7 and 193.0 s.
  The direction is down in both runs and wall time rose from a 103.5 s median to 145.7 and 193.0 s,
  but 13% to 15% is under the control's 20% spread,
  so two builds do not settle it.
- `-Z time-passes`:
  `anon` 6,379 MiB (0.946 times,
  range 0.940 to 0.952),
  `memory.peak` 6,570 MiB,
  the largest `rustc` 6,424 MiB,
  wall 138.8 s.
  This is the same build with one more `rustc` flag,
  so a ratio near one is the expected answer.
- Half the test code left out:
  `anon` 4,663 MiB (0.700 times,
  range 0.687 to 0.713),
  `memory.peak` 4,875 MiB,
  the largest `rustc` 4,741 MiB,
  wall 47.1 s.
  Both far outside the spread:
  the lowest peak and the shortest build of the round.
- `cargo check --locked --offline --all-targets`:
  `anon` 4,011 MiB,
  `memory.peak` 4,117 MiB,
  the largest `rustc` 4,049 MiB,
  wall 73.6 s.
- `cargo clippy --locked --offline --all-targets`,
  with `-D warnings` as the round's task passes it:
  `anon` 4,225 MiB,
  `memory.peak` 4,298 MiB,
  the largest `clippy-driver` 4,277 MiB,
  wall 67.9 s.

The two metadata passes match the 4,084 to 4,192 MiB and 4,295 to 4,379 MiB
that the same day's research sampled from other sessions' processes,
which is evidence that these arms measure what that sampling measured.

### Which pass holds the peak

`cargo rustc --locked --offline --lib --profile test -- -Z time-passes`
printed the resident size at the boundaries of each pass of the same test unit.
From one build whose sampled peak was 6,570 MiB,
the largest end-of-pass figures were:

- `macro_expand_crate`,
  1,183 MiB after 5.2 s;
- `type_check_crate`,
  2,827 MiB,
  1,308 MiB of it in 21.2 s;
- `MIR_checking`,
  4,122 MiB,
  1,296 MiB of it in 14.1 s;
- `monomorphization_collector_graph_walk`,
  5,056 MiB,
  1,023 MiB of it in 30.7 s;
- `codegen_to_LLVM_IR`,
  6,410 MiB,
  1,252 MiB of it in 26.7 s;
- `codegen_crate`,
  6,410 MiB,
  2,387 MiB of it in 61.8 s;
- `link_crate`,
  2,262 MiB after 16.5 s,
  since linking runs after code generation and frees memory it no longer needs.

So the front end reaches 4,122 MiB by the end of `MIR_checking`,
code generation adds 2,288 MiB on top of that,
and the sampled peak of 6,570 MiB sits above every end-of-pass figure,
because the resident size between two pass boundaries is not one of the printed numbers.
That is the front-end and code-generation split the research could only infer
from the 1.9 to 2.4 GiB gap between the check and codegen forms of this unit.

### What each result says about the research's ranking

1.  Compile less of the crate in a round.
    Confirmed,
    and measured:
    leaving out 49.9% of the library test unit's test lines
    (165,191 of 331,058 lines,
    265 module declarations in 253 files)
    brought the compared peak to 0.700 times the control and the build to 47.1 s
    against a control median of 103.5 s.
    The research's guess of 4 to 5 GiB for half the test code measured 4,663 MiB of `anon`
    and 4,875 MiB of `memory.peak`.
    The scaling is not linear in lines:
    half the test lines bought 30% of the peak,
    because the non-test library,
    the dependencies and the front end of the remaining tests stay.
    The project's way to compile less per edit is already decided:
    the crate split into reusable crates (`doc/planning/cli-git-source-layout.md`,
    "Splitting into crates (owner,
    2026-11-04)"),
    whose native-layer move waits until the open branches merge (issue 6261,
    "Defer until branches merge").
    This arm measures that lever;
    it adds no new choice.
2.  Fewer parallel jobs per build.
    Weakened to a small effect at the extreme:
    `-j 4` changed nothing outside the spread,
    and `-j 1` gained 13% to 15% at most,
    which the control's own spread cannot separate from host load,
    while wall time went from a 103.5 s median to 145.7 and 193.0 s.
    A round that adopts it should expect no change at four jobs and a longer wait at one.
3.  Lighter debuginfo (`debug = false`).
    Its reach is the code-generation part,
    2,288 MiB of the 6,410 MiB the pass table shows,
    and debug information is written during exactly those passes.
    The arm was not run because the owner rejected the setting on 2026-11-13.
4.  Limit how many peaks overlap.
    Unchanged by these arms,
    and the control's 20% spread is the argument for it:
    two builds of one unchanged crate differed by 1,144 MiB,
    so a cap set from a single measurement is wrong by that much.
5.  `codegen-units`.
    Same reach as item 3 and still unknown in sign;
    not measured.
6.  `sccache` as the wrapper.
    Not measured,
    and nothing here moves it:
    it refuses the unit that sets the peak,
    and a metadata pass alone costs 4,011 to 4,225 MiB of `anon`.
7.  A shared build directory across worktrees.
    Not measured,
    and the same reason holds:
    the unit that sets the peak is this crate's own,
    compiled in every worktree that edits code,
    so a shared directory shares nothing of it.

The cause worth recording is the control's spread itself.
Two faithful builds of one unchanged crate reached 5,663 and 6,807 MiB of `anon`,
and the slow controls read low,
5,663 MiB at 126.5 s and 6,244 MiB at 148.9 s,
against 6,789 MiB at 103.5 s.
That is the shape a token-limited code generation step has:
when the host gives the build less parallelism,
both the peak and the progress fall.
It is the same mechanism `-j` limits,
which is why `-j 1` can only be read as "no more than 15%" here,
and why any future arm on this page needs its control beside it.

### Where the numbers and logs are

The round folder `rust-build-memory-peak-arms-2026-11-25` holds them,
outside the repository:
`keep/arm-summary.json` is every counter of every arm,
`keep/time-passes-rows.tsv` is the pass table,
`keep/arms-so-far.tsv` is one line per arm,
and `scratch/arms/<number>-<arm>/` holds a `arm.json`,
`build.log`,
`samples.tsv` and `gate.log` per arm.
The half-test-code edit was a working-tree edit only,
never committed or stashed,
restored with `git checkout` once it was recorded,
after which `git status --short` was empty;
`keep/half-modules-edit.patch` is that edit.
No test binary was run,
no profile or manifest was changed,
and no container ran.

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

## The setting measured again on the new drive, 2026-11-28

Status:
unapproved and not final.
This section changes no setting and no source,
and it measures the setting above against incremental compilation on.
The proposal it supports is
`doc/planning/cli-git-development-profile-after-the-move-to-the-new-drive.md`
at the repository root.
The owner declares completion,
and what is open is listed in that proposal.

### What was different from the 2026-11-13 measurement

- The home folder moved to the 3.5 TB drive on 2026-11-17
  (`doc/handover/agents-at-once-on-this-machine.md`,
  "The cap now"),
  so a build tree is no longer what limits how many rounds run at once.
- The crate is 1,298,623 lines in 4,012 Rust files below `src/native`,
  and the 1,565 files named `*_tests.rs` hold 680,703 of those lines,
  52.4%.
  Counted with
  `find src/native -type f -name '*.rs' -exec cat {} + | wc -l`
  and the same command with `-name '*_tests.rs'`,
  and the file count with `find src/native -type f -name '*.rs' | wc -l`.
- Memory is the limit now:
  twenty-three rounds starting at once on 2026-11-27 ran the machine out of memory and swap,
  and the record of that night names `rustc` in aggregate,
  "`rustc` holding 28.8 GiB resident",
  beside "eleven `rustc` processes and nineteen rounds",
  so it names no single compiler and no single unit
  (`doc/handover/cli-git-rust-implementation.md`,
  "Memory").

So the two questions are the ones the 2026-11-13 round left open:
whether incremental compilation lowers or raises the compiler's peak memory for this crate,
and what a rebuild after a one-line edit costs either way.

### What one arm is

Each arm is one build in a build directory of its own,
inside a scope of its own,
one build at a time:

```sh
# run in package/git-policy/cli, with the four variables set for the arm
CARGO_TARGET_DIR=<tree>/target \
CARGO_BUILD_BUILD_DIR=<tree>/build \
CARGO_INCREMENTAL=<0 or 1> \
systemd-run --user --scope --slice=agent-work.slice \
  --property=MemoryMax=40G --property=CPUQuota=600% \
  --property=TasksMax=1024 --property=IOWeight=1 \
  --property=RuntimeMaxSec=5400 --unit=<the arm> -- /bin/sh -c '<the build>'
```

`cargo` inside such a scope reads `nproc` as 6,
so Cargo's default job count is 6 in every arm.
The two commands are the brief's:

- `cargo build --locked --offline --lib`
- `cargo test --locked --offline --lib --features test-only-switches --no-run`

Four counters are read per arm,
and none of them needs GNU `time`,
which this machine does not install:

- wall time,
  from `date +%s%N` either side of the build inside the scope;
- processor time,
  from the scope's own `cpu.stat` field `usage_usec`,
  read inside the scope before its last process exits;
- the whole scope's peak resident memory,
  from the scope's `memory.peak` file,
  which is the kernel's high-water mark and includes the page cache the build wrote,
  and its `anon` share from a one-second sample of `memory.stat`;
- the largest `rustc`'s peak resident memory,
  from `VmHWM` in `/proc/<pid>/status`,
  sampled once a second for every `rustc` in the scope,
  whose process numbers come from the scope's `cgroup.procs`.

Sizes are apparent bytes:
`du --summarize --bytes --apparent-size <tree>`,
and the incremental share is the same command on `<tree>/build/debug/incremental`.
With `CARGO_BUILD_BUILD_DIR` set,
the tree holds all intermediates and the final artifacts in the build directory,
and `<tree>/target` holds only `CACHEDIR.TAG`,
the two lock files and an empty `examples` directory (177 bytes).

The arms were gated:
a build started only when the `full` line of `/proc/pressure/memory` read
`avg60` below 10,
and the reading at each start is recorded beside the arm.
That gate waited 782 s before one control,
481 s before one restore build
and 60 s before the extra first build below,
and every other arm found it open.

### The files the one-line edits changed

Each edit adds one line,
`pub fn a_rebuild_measured_after_one_edit() {}`,
at the end of one file,
and the revert removes it,
so the two runs of a control are the same work in opposite directions.
Both files were chosen by counting,
and the counts are the measurement:

- the leaf file,
  `src/native/candidate/candidate_store_commit_listings.rs`,
  is named by no other Rust file:
  `rg -l '\bcandidate_store_commit_listings\b' src/native --glob '*.rs' | wc -l`
  answers 1,
  and that one is its parent's `pub mod` line in `src/native/candidate/mod.rs`
  (the folder's own doc lines name it as a file),
  and the parent re-exports nothing from it.
- the widely used file,
  `src/native/worktree_clone_mount_protocol/mod.rs`,
  is named by 273 Rust files:
  `rg -l '\bworktree_clone_mount_protocol\b' src/native --glob '*.rs' | wc -l`,
  the most of any file of the crate.

### Every arm

Wall is the clock's;
CPU is the scope's processor time;
`rustc` is the largest compiler run's peak resident memory;
`scope` is the scope's `memory.peak`;
tree is the build directory's apparent size after the arm,
with the incremental data's share of it.

The first build,
in a clean build directory:

- `lib-clean-off-a`,
  `cargo build --lib`,
  incremental off:
  wall 94.0 s,
  CPU 435.1 s,
  `rustc` 2853 MiB,
  scope 3817 MiB,
  tree 787 MiB and no incremental data.
- `lib-clean-on`,
  the same build,
  incremental on:
  wall 89.3 s,
  CPU 400.8 s,
  `rustc` 2825 MiB,
  scope 3796 MiB,
  tree 1551 MiB,
  of which 808 MiB incremental.
- `test-clean-off-a`,
  `cargo test --lib --no-run`,
  incremental off:
  wall 125.5 s,
  CPU 481.9 s,
  `rustc` 7488 MiB,
  scope 7617 MiB,
  tree 895 MiB and no incremental data.
- `test-clean-on`,
  the same build,
  incremental on:
  wall 165.7 s,
  CPU 528.5 s,
  `rustc` 7013 MiB,
  scope 7183 MiB,
  tree 2705 MiB,
  of which 1824 MiB incremental.
- `lib-clean-off-b`,
  the control again in a second tree:
  wall 103.2 s,
  CPU 446.1 s,
  `rustc` 2855 MiB,
  tree 787 MiB.
- `test-clean-off-b`,
  the control again in a second tree:
  wall 145.2 s,
  CPU 547.1 s,
  `rustc` 7429 MiB,
  tree 895 MiB.

The rebuild after the leaf file's one line:

- `lib-leaf-off-a-1`,
  incremental off:
  wall 37.2 s,
  CPU 45.4 s,
  `rustc` 3275 MiB.
- `lib-leaf-on`,
  incremental on:
  wall 37.4 s,
  CPU 17.9 s,
  `rustc` 2083 MiB,
  tree 1537 MiB with 794 MiB incremental.
  7.45 s of that wall is Cargo's package cache lock:
  the log holds three `Blocking waiting for file lock on package cache` lines
  before the compiler starts,
  so the build itself took 30.0 s.
- `lib-leaf-off-a-2`,
  the control again,
  reverting the line:
  wall 26.1 s,
  CPU 42.2 s,
  `rustc` 2812 MiB.
- `test-leaf-off-a-1`,
  incremental off:
  wall 66.6 s,
  CPU 125.0 s,
  `rustc` 7348 MiB.
- `test-leaf-on`,
  incremental on:
  wall 32.3 s,
  CPU 31.6 s,
  `rustc` 4969 MiB,
  tree 2658 MiB with 1777 MiB incremental.
- `test-leaf-off-a-2`,
  the control again:
  wall 62.2 s,
  CPU 119.3 s,
  `rustc` 7499 MiB.

The rebuild after the widely used file's one line:

- `lib-wide-off-a-1`,
  incremental off:
  wall 22.2 s,
  CPU 37.9 s,
  `rustc` 2798 MiB.
- `lib-wide-on`,
  incremental on:
  wall 13.4 s,
  CPU 13.4 s,
  `rustc` 2016 MiB.
- `lib-wide-off-a-2`,
  the control again:
  wall 25.3 s,
  CPU 41.1 s,
  `rustc` 2813 MiB.
- `test-wide-off-a-1`,
  incremental off:
  wall 72.3 s,
  CPU 131.9 s,
  `rustc` 7411 MiB.
- `test-wide-on`,
  incremental on:
  wall 72.0 s,
  CPU 45.6 s,
  `rustc` 4899 MiB.
- `test-wide-off-a-2`,
  the control again:
  wall 208.2 s,
  CPU 352.3 s,
  `rustc` 7270 MiB.
  This control compiled exactly one crate,
  as the control before it did,
  the same unit into the same path,
  and the whole difference is the machine:
  its second sample of an unchanged build of one unchanged crate is 2.7 times the CPU of the first.

Two more builds put a tree back to its original bytes before its next arm,
and both are incremental arms,
so their figures are the same arm's work twice over:
`lib-leaf-on-restore` took wall 37.2 s,
CPU 18.6 s and `rustc` 2045 MiB,
and `test-leaf-on-restore` took wall 31.4 s,
CPU 29.5 s and `rustc` 4886 MiB.

### Every arm as a ratio to the controls of its own round

Each cell below is one command and one round,
the control is the setting as it is,
and the ratio is the treatment over the mean of the two control runs,
with the range that dividing by each control alone gives.

The first build:

- library build,
  incremental on:
  wall 0.91,
  range 0.87 to 0.95,
  controls 9.9% apart;
  CPU 0.91,
  range 0.90 to 0.92.
  The control's own spread is 9.9% of wall and 2.5% of CPU.
- library test build,
  incremental on:
  wall 1.22,
  range 1.14 to 1.32,
  controls 15.7% apart;
  CPU 1.03,
  range 0.97 to 1.10.
  So the first test build pays no processor time for incremental data
  and pays about a fifth more wall.
  Its own compile of this crate,
  after the dependency phase ends,
  is 99.2 s against 57.6 s and 66.3 s for the two controls.

The rebuild after the leaf file's line:

- library build,
  incremental on:
  CPU 0.41,
  range 0.39 to 0.42;
  wall 1.18,
  range 1.01 to 1.43,
  which the package cache lock in that arm explains:
  without its 7.45 s the wall is 30.0 s,
  0.95 times the controls,
  range 0.81 to 1.15.
- library test build,
  incremental on:
  wall 0.50,
  range 0.48 to 0.52;
  CPU 0.26,
  range 0.25 to 0.27.

The rebuild after the widely used file's line:

- library build,
  incremental on:
  wall 0.56,
  range 0.53 to 0.60;
  CPU 0.34,
  range 0.33 to 0.35.
- library test build,
  incremental on:
  CPU 0.35 against the control that ran two minutes before it,
  0.13 against the control that ran after it;
  wall 1.00 against the first and 0.35 against the second.
  Only the first pairing is worth reading:
  the second control ran while the machine had slowed every build by about 2.7 times,
  which no property of that arm explains.

### Peak memory, which is the question this round was for

The largest `rustc`,
per cell,
controls first:

- first library build:
  2853 and 2855 MiB without incremental data,
  2825 MiB with it,
  0.99 of the controls,
  which agree to 0.0%.
- first library test build:
  7488 and 7429 MiB without,
  7013 MiB with,
  0.94,
  controls 0.8% apart.
- library rebuild after the leaf edit:
  3275 and 2812 MiB without,
  2083 MiB with,
  0.68.
- library test rebuild after the leaf edit:
  7348 and 7499 MiB without,
  4969 MiB with,
  0.67,
  controls 2.0% apart.
- library rebuild after the wide edit:
  2798 and 2813 MiB without,
  2016 MiB with,
  0.72,
  controls 0.5% apart.
- library test rebuild after the wide edit:
  7411 and 7270 MiB without,
  4899 MiB with,
  0.67,
  controls 1.9% apart.

So incremental compilation does not raise this crate's peak memory at any of the six cells,
and on a rebuild it lowers it by a third.
The whole scope's peak moves the same way,
from 7617 MiB on the first test build without incremental data to 7183 MiB with it,
and from 7330 MiB to 5737 MiB on the test rebuild after the leaf edit.

The test build's compiler is the peak the machine has to hold,
7.3 to 7.5 GiB without incremental data,
and the rebuild of that same unit reaches the same peak,
so a round's ordinary edit-and-test loop costs what its first build costs.
The library build's compiler peaks at 2.8 GiB,
a third of that.

#### The one place two runs of one arm disagreed

The library rebuild after the leaf edit peaked at 3275 MiB in its first run
and 2812 MiB in its second,
16.5% apart,
while every other control pair agrees within 2%.
The two runs compiled the same one crate with the same flags,
and the run with the higher peak also took 42% longer in wall
and 8% more processor time,
with its compiler's resident memory still climbing at 28.1 s of its 36.0 s,
against a peak at 24.1 s of 24.3 s in the other.
That is the shape of code generation starved of threads rather than of different work,
and the run that got the least processor time per second of wall reached the highest peak
(1.22 cores against 1.62).
The five other runs without incremental data of that same unit
sit between 2798 and 2855 MiB.
So the peak is stable to about 2% when the host schedules a build the same way twice,
and moves by up to 17% when it does not,
which is the uncertainty to carry into a cap set from these numbers.

### Disk

Apparent bytes per tree after a first build,
and the incremental data's share:

- library build,
  incremental off:
  787 MiB,
  no incremental data.
- library build,
  incremental on:
  1551 MiB,
  808 MiB of it incremental data.
- library test build,
  incremental off:
  895 MiB,
  no incremental data.
- library test build,
  incremental on:
  2705 MiB,
  1824 MiB of it incremental data.

After a rebuild the on trees read 1537 and 2658 MiB,
with 794 and 1777 MiB of incremental data,
so a rebuild does not grow the incremental directory.

The dependency units are 572 MiB of every tree,
measured as the tree minus this crate's own unit directory
(`build/debug/build/monochromatic-git-policy-cli`):
225 MiB of the library tree's 825 MiB and 339 MiB of the test tree's 939 MiB.
Of the test tree's 1824 MiB of incremental data,
1616 MiB belong to this crate,
161 MiB to the repository's own path dependencies
(`forbidden_strings`,
`forbidden_regex` twice,
`monochromatic_lint`,
`monochromatic_jsonc_edit`,
`monochromatic_catalog_specifier`,
the ZFS request crate and a build script),
and no registry dependency holds any:
the tree holds nine incremental directories,
and all nine name packages whose source is a path,
so the dependency share here is the repository's own path dependencies and nothing else.
The library tree's 808 MiB splits the same way,
647 MiB for this crate and 161 MiB for the path dependencies.

A round builds both commands in one build directory,
which holds the dependency units once,
so one worktree's tree is about 1.14 GiB without incremental data
and about 3.5 GiB with it.
The drive had 1.4 TB free at 2026-11-28T05:04:56-05:00 (`df -h /home`),
which is about 1,200 such trees without incremental data and 400 with it.

### What the worktrees of this repository share today

Nothing of a build.
`cargo-shadow` sets `CARGO_BUILD_BUILD_DIR` for the child Cargo to
(erratum below: only when `RUSTUP_TOOLCHAIN` or `+toolchain` is present)
`<base>/toolchain/<toolchain>/<two hex>/<fourteen hex>`,
where the key is SHA-256 of the canonical workspace root
(`package/cargo-shadow/cli/src/lean_keyed_folder.rs`,
`workspace_key` and `keyed_build_dir`),
and leaves a ledger file `.cargo-shadow-workspace` in it naming that root
(`package/cargo-shadow/cli/README.md`,
"Keeping the build folders lean",
mechanism A;
`doc/decision/cargo-build-dir-host-cache.md`,
"How the template resolves here").
Two worktree paths are two keys.

Measured on 2026-11-28:

- the main checkout's crate,
  `/home/user/Monochromatic/package/git-policy/cli`,
  keys to `toolchain/nightly-2026-11-16/a6/5cc5a0dacf91e3` under
  `/home/user/.cargo/build`,
  2,174,623,752 bytes,
  and its ledger names that path;
- another worktree's crate keys to `61/70104e8fe7cae1`,
  3,228,469,226 bytes,
  and its ledger names that worktree;
- this round's worktree keys to `f6/b033a041ce8843`,
  which holds no tree,
  because every arm of this round built in a directory of its own.

So a second worktree's first build reuses the registry sources
(`~/.cargo/registry`) and the installed toolchain,
and compiles again every one of the crate's 4,012 Rust files and every dependency:
266 to 271 dependency units in 64.3 to 78.8 s,
the dependency phase of the first builds here,
before this crate's own unit starts.
It shares no unit,
no `rmeta`,
no incremental data and no final artifact.
(Erratum below: this is the case with no prewarmed clone in the keyed folder.)

Sharing the dependencies' builds across worktrees would save
572 MiB per tree and that dependency phase,
whose largest `rustc` peaked at 466 MiB with six of them running at once
(measured in one extra first build whose sampler kept a timeline:
the dependency phase ran 185.5 s until this crate's unit started at 255.0 s of wall).
It would save nothing of a rebuild,
whose one compiled unit is this crate's own,
and it would save nothing of the peak memory that limits rounds,
because that peak is this crate's unit.

### What was not run

- No test ran:
  every command was a build.
- `debug = false`,
  `-j`,
  `codegen-units` and `split-debuginfo` were not measured again;
  the 2026-11-13 numbers above stand for them,
  and the owner rejected `debug = false` for what it costs a backtrace.
- No second worktree of the repository was made,
  so what a second worktree pays is argued from the key above
  and from the first builds measured here,
  not from a build at a second workspace path.
- The `--all-targets` form and the `native_binary` consumer target were not measured,
  and their units are larger than the library test build measured here.

### Where each number comes from

- Crate size,
  edits' file counts:
  the `rg` and `find` commands quoted above.
- Wall,
  CPU,
  scope peak,
  `rustc` peak:
  the scope's `cpu.stat`,
  `memory.peak` and `/proc/<pid>/status` as described under "What one arm is".
- Tree sizes:
  `du --summarize --bytes --apparent-size`.
- The worktrees' keys:
  the SHA-256 of each workspace root,
  as `workspace_key` computes it,
  checked against the directories that exist under `/home/user/.cargo/build`
  and the ledger file each holds.
- Free space:
  `df -h /home`.

## The recommended form measured on its own, 2026-11-28

Status:
unapproved and not final.
This section changes no setting and no source.
It measures form D,
`[profile.dev.package.monochromatic-git-policy-cli] incremental = true` beside
`[profile.dev] incremental = false`,
which is the form the proposal recommends,
in its own first build and its own rebuild of the library build and of the library test build,
each beside two form A controls run between them in the same round.
It also observes,
in two rounds that run both of a round's commands in one build directory,
whether the `forbidden-strings` build script runs a second time under form C.
The proposal is `doc/planning/cli-git-development-profile-after-the-move-to-the-new-drive.md`;
the owner declares completion,
and what is open is listed there.

The crate is 1,311,645 lines in 4,049 Rust files below `src/native` at 2026-11-28T08:22-05:00,
688,854 of them in 1,582 `*_tests.rs` files,
counted with the commands under "What was different from the 2026-11-13 measurement" above;
the 1,298,623 lines counted earlier the same day are that round's reading of a crate that grew while these rounds ran.

The arms above measured the everywhere form,
because the harness set `CARGO_INCREMENTAL` and changed no manifest.
This round changes no variable:
each arm writes the manifest text of its own form into `package/git-policy/cli/Cargo.toml` before it starts and puts the
checked-out bytes back after it,
and each arm's record holds the text it added,
so a reader can tell which form it measured.

What the data is,
from Cargo's reference fetched 2026-11-28:
"Incremental compilation causes rustc to save additional information to disk which will be reused when recompiling the
crate,
improving re-compile times".
rustc writes that information under `<build dir>/debug/incremental/<crate>-<hash>/s-<session>/`,
one directory per compiler run,
and garbage-collects it each time it starts in incremental mode,
deleting every session directory it cannot lock and every finalized session except the most recent one
(`compiler/rustc_incremental/src/persist/fs.rs`,
rust-lang/rust,
read 2026-11-28).
So a crate's incremental directory is bounded by one session's size and not by how many rebuilds a tree has seen.
An inherited `CARGO_INCREMENTAL`,
`CARGO_PROFILE_DEV_INCREMENTAL`,
`CARGO_PROFILE_TEST_INCREMENTAL`,
`RUSTFLAGS` or `CARGO_BUILD_JOBS` would have made every arm the everywhere form,
so the harness deletes the three Cargo variables from the child's environment and records all five by name;
every arm records all five as `unset`.

### What one arm is, this round

The shape is the one under "What one arm is" above:
one build in a build directory of its own,
inside `systemd-run --user --scope --slice=agent-work.slice` with `MemoryMax=40G`,
`CPUQuota=600%`,
`TasksMax=1024`,
`IOWeight=1` and `RuntimeMaxSec=5400`,
one build at a time,
behind the memory-pressure gate that opens below `full avg60` 10,
with `CARGO_TARGET_DIR` and `CARGO_BUILD_BUILD_DIR` pointing into a tree of the arm's own.
`cargo` inside such a scope reads `nproc` as 6,
as before.

Three differences from the arms above:

- The two commands are the same two,
  and a "round" arm runs both in one build directory,
  each in a scope of its own,
  so a round's tree is measured as a round builds it.
- The first round pair ran with `-v` and the second with `-vv`,
  because Cargo prints a build script's own run only at `-vv`:
  the line reads ``Running `CARGO=... CARGO_PKG_NAME=forbidden-strings ...` ``.
- Every arm's record keeps its dependency-phase timeline,
  the compiling count and,
  in the round arms,
  the number of build-script run lines and build-script compile lines.

### Every arm

Wall is the clock's;
CPU is the scope's processor time;
`rustc` is the largest compiler run's peak resident memory;
`scope` is the scope's `memory.peak`;
tree is the build directory's apparent size after the arm,
with the incremental data's share of it.
The evidence is the arms' own logs,
and the two restore arms (15 and 19) only put a tree back to its original bytes before its wide-edit arm.

The first build,
in a clean build directory:

- `lib-clean-a-1`,
  form A,
  `cargo build --lib`:
  wall 111.0 s,
  CPU 459.3 s,
  `rustc` 2837 MiB,
  scope 3807 MiB,
  tree 788 MiB and no incremental data.
- `lib-clean-d`,
  form D,
  the same build:
  wall 104.5 s,
  CPU 446.5 s,
  `rustc` 2835 MiB,
  scope 3780 MiB,
  tree 1410 MiB,
  of which 651 MiB incremental data in one directory.
- `lib-clean-a-2`,
  form A again:
  wall 89.2 s,
  CPU 410.4 s,
  `rustc` 2848 MiB,
  scope 3753 MiB,
  tree 788 MiB.
- `test-clean-a-1`,
  form A,
  `cargo test --lib --no-run`:
  wall 136.1 s,
  CPU 518.5 s,
  `rustc` 7445 MiB,
  scope 7578 MiB,
  tree 897 MiB.
- `test-clean-d`,
  form D,
  the same build:
  wall 164.9 s,
  CPU 552.0 s,
  `rustc` 7059 MiB,
  scope 7179 MiB,
  tree 2572 MiB,
  of which 1674 MiB incremental data in one directory.
- `test-clean-a-2`,
  form A again:
  wall 129.7 s,
  CPU 513.5 s,
  `rustc` 7532 MiB,
  scope 7659 MiB,
  tree 897 MiB.

The rebuild after the leaf file's one line
(`src/native/candidate/candidate_store_commit_listings.rs`,
named by no other file of the crate):

- `lib-leaf-a-1`,
  form A:
  wall 20.6 s,
  CPU 35.3 s,
  `rustc` 2853 MiB,
  scope 2762 MiB.
- `lib-leaf-d`,
  form D:
  wall 13.6 s,
  CPU 18.1 s,
  `rustc` 2192 MiB,
  scope 2405 MiB,
  tree 1397 MiB with 638 MiB of incremental data.
- `lib-leaf-a-1-revert`,
  form A,
  the same work in the other direction:
  wall 21.8 s,
  CPU 37.9 s,
  `rustc` 2813 MiB,
  scope 2703 MiB.
- `test-leaf-a-1`,
  form A:
  wall 54.0 s,
  CPU 107.1 s,
  `rustc` 7525 MiB,
  scope 7410 MiB.
- `test-leaf-d`,
  form D:
  wall 23.6 s,
  CPU 27.1 s,
  `rustc` 5057 MiB,
  scope 5737 MiB,
  tree 2524 MiB with 1627 MiB of incremental data.
- `test-leaf-a-1-revert`,
  form A:
  wall 54.5 s,
  CPU 107.9 s,
  `rustc` 7519 MiB,
  scope 7411 MiB.

The rebuild after the widely used file's one line
(`src/native/worktree_clone_mount_protocol/mod.rs`,
which 273 files of the crate name):

- `lib-wide-a-1`,
  form A:
  wall 21.5 s,
  CPU 37.9 s,
  `rustc` 2860 MiB,
  scope 2848 MiB.
- `lib-wide-d`,
  form D:
  wall 11.3 s,
  CPU 14.0 s,
  `rustc` 2071 MiB,
  scope 2328 MiB,
  tree 1395 MiB with 635 MiB of incremental data.
- `lib-wide-a-1-revert`,
  form A:
  wall 23.8 s,
  CPU 39.5 s,
  `rustc` 2815 MiB,
  scope 2751 MiB.
- `test-wide-a-1`,
  form A:
  wall 62.6 s,
  CPU 117.8 s,
  `rustc` 7544 MiB,
  scope 7547 MiB.
- `test-wide-d`,
  form D:
  wall 25.4 s,
  CPU 27.5 s,
  `rustc` 4961 MiB,
  scope 5665 MiB,
  tree 2524 MiB with 1627 MiB of incremental data.
- `test-wide-a-1-revert`,
  form A:
  wall 52.9 s,
  CPU 106.6 s,
  `rustc` 7539 MiB,
  scope 7496 MiB.

The two rounds of both commands in one build directory:

- `round-d-both-commands`,
  form D,
  `-v`:
  `cargo build --lib` wall 87.6 s and CPU 393.6 s,
  compiling 272 units;
  `cargo test --lib --no-run` wall 74.2 s and CPU 133.2 s,
  compiling one unit,
  this crate's.
  The tree after both is 3410 MiB,
  of which 2325 MiB of incremental data in two directories,
  and both name this crate.
- `round-c-both-commands`,
  form C,
  `-v`:
  `cargo build --lib` wall 95.8 s and CPU 432.0 s,
  compiling 270 units;
  `cargo test --lib --no-run` wall 103.2 s and CPU 268.4 s,
  compiling seven units.
  The tree after both is 2998 MiB,
  of which 1836 MiB of incremental data in nine directories:
  this crate,
  the seven path dependencies and their build script,
  and `forbidden_regex` twice.

### Every cell as a ratio to the controls of its own round

Each cell is one command and one round;
the control is form A;
the ratio is the treatment over the mean of the two control runs,
with the range that dividing by each control alone gives,
and the controls' own spread.
A ratio inside the controls' spread,
or inside the 20% these builds can show,
is reported as no difference:
the page's own rule reads "A difference smaller than 1.20 is no difference these builds can show."

The first build:

- library build:
  wall 104.5 s against 111.0 and 89.2 s,
  1.04,
  range 0.94 to 1.17,
  controls 24.5% apart,
  so this cell shows no wall difference and its controls differ by more than the 20% bar;
  CPU 1.03,
  range 0.97 to 1.09,
  controls 11.9% apart,
  no difference shown;
  largest compiler 1.00,
  scope peak 1.00,
  anonymous 1.01.
- library test build:
  wall 164.9 s against 136.1 and 129.7 s,
  1.24,
  range 1.21 to 1.27,
  controls 4.9% apart,
  which clears the bar;
  CPU 1.07,
  range 1.06 to 1.07,
  controls 1.0% apart;
  largest compiler 0.94,
  range 0.94 to 0.95,
  scope peak 0.94,
  anonymous 0.96,
  all three inside the 20% bar,
  so a first build's peak is unchanged as far as this measurement can tell.

The rebuild after the leaf file's line:

- library build:
  wall 0.64,
  range 0.63 to 0.66,
  controls 5.7% apart;
  CPU 0.49,
  range 0.48 to 0.51,
  controls 7.3% apart;
  largest compiler 0.77,
  missing from scope peak (0.88) and clear on anonymous memory (0.72).
- library test build:
  wall 0.44,
  range 0.43 to 0.44,
  controls 0.9% apart;
  CPU 0.25,
  range 0.25 to 0.25,
  controls 0.7% apart;
  largest compiler 0.67,
  scope peak 0.77,
  anonymous 0.63.

The rebuild after the widely used file's line:

- library build:
  wall 0.50,
  range 0.48 to 0.53,
  controls 10.7% apart;
  CPU 0.36,
  range 0.35 to 0.37,
  controls 4.0% apart;
  largest compiler 0.73,
  anonymous 0.68,
  and scope peak 0.83 which is inside the bar.
- library test build:
  wall 0.44,
  range 0.41 to 0.48,
  controls 18.3% apart;
  CPU 0.25,
  range 0.23 to 0.26,
  controls 10.5% apart;
  largest compiler 0.66,
  scope peak 0.75,
  anonymous 0.63.

So the recommended form's rebuild loop is measured for both edits,
and the wall gain holds for both:
0.64 and 0.50 on the library build,
0.44 and 0.44 on the library test build.
The one cell this round did not reproduce from the arms above is the wide-edit test rebuild's wall of 1.00 there;
here the same cell reads 0.44 against two controls 18.3% apart.

### What a round pays and what it gets back

On the library test build,
form D's first build costs 32.0 s more wall (164.9 s against the controls' mean of 132.9 s)
and 36.0 s more processor time (552.0 s against 516.0 s).
Both edits pay that back at about one rebuild:

- leaf edit:
  a rebuild is 23.6 s against 54.2 s of wall and 27.1 s against 107.5 s of processor time,
  so the first build's extra wall is paid back after 1.04 rebuilds and its extra processor time after 0.45;
  at k rebuilds the wall is 1.01,
  0.88,
  0.80,
  0.70 and 0.59 of today for k = 1,
  2,
  3,
  5 and 10,
  and the processor time 0.93,
  0.83,
  0.76,
  0.65 and 0.52.
- wide edit:
  a rebuild is 25.4 s against 57.7 s of wall and 27.5 s against 112.2 s of processor time,
  so the first build's extra wall is paid back after 0.99 rebuilds and its extra processor time after 0.42;
  at the same k the wall is 1.00,
  0.87,
  0.79,
  0.69 and 0.59,
  and the processor time 0.92,
  0.82,
  0.74,
  0.64 and 0.51.

A round that makes no rebuild after its first build pays 1.24 in wall and 1.07 in processor time and gets nothing back.
How many rebuilds a round makes is still unmeasured;
no record holds it.

### The dependency phase, in this round's own timelines

The sampler counts processes named `rustc`,
once a second,
and the scope peak it records is the kernel's `memory.peak` for the scope,
page cache included.
So a peak that is reached while no `rustc` is in the scope is not a compiler's.

In the two form A library first builds,
the scope's peak (3807 and 3753 MiB) is reached during a stretch with no `rustc` in the scope,
13 and 13 one-second samples of it,
while the largest compiler peaked at 2837 and 2848 MiB.
In the two form A library test builds,
the samples with no `rustc` in the scope stood at 3761 and 3748 MiB,
and this crate's unit started at 72.7 s of 136.1 s and 72.3 s of 129.7 s.
That is the level the 2026-11-25 record names,
in the words "the `forbidden-strings` build script's run at 3756 MiB in every worktree where its output is not fresh"
(`doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md`,
"A build's peak is one process").
The sampler cannot name the process,
so the attribution to that build script is the earlier record's,
not this round's;
what this round adds is the level,
the counter that measured it and the samples it was measured in.

### Whether a build script runs a second time

Cargo prints a build script's own run only at `-vv`.
In the first command of both `-vv` rounds,
the detector finds 21 and 20 build-script run lines and 33 build-script compile lines,
(erratum below: the true count is 33 compile lines, 36 run tokens and 33 distinct run paths)
which is the positive control:
it can see a run it should see.
In the second command:

- form D (`round-d-both-commands-vv`):
  no build-script run line,
  no build-script compile line,
  one unit compiled,
  this crate's,
  wall 75.4 s and CPU 140.7 s.
- form C (`round-c-both-commands-vv`):
  two build-script run lines,
  `monochromatic-catalog-specifier`'s and `forbidden-strings`'s
  (erratum below: one run line, `forbidden-strings`'s,
  and `monochromatic-catalog-specifier` has no build script)
  (it reads `.../build/forbidden-strings/c65731ebdc1d74a6/out/build_script_build`
  with `CARGO_PKG_NAME=forbidden-strings`),
  one build-script compile line,
  seven units compiled,
  wall 94.7 s and CPU 263.4 s,
  against form D's one unit and 75.4 s of wall.

So under form C the test command compiles the path dependencies again,
compiles the build script again and runs it again,
and under form D it does none of those.

### What each form writes

The incremental directories a tree holds after the arm,
by name as they stand on disk:

- form A:
  none,
  in either build.
- form D:
  `git_policy_cli-<hash>` only,
  one directory in the library tree and one in the test tree,
  and two in the round's one tree,
  both this crate's.
- form C:
  nine,
  this crate's,
  the seven path dependencies' and their build script's,
  and `forbidden_regex` twice,
  which is the same crate's unit for the two profiles.

That is the scoping D asks for,
observed rather than inferred:
the path dependencies hold no incremental data under D and do hold it under C.

### Disk, from this round's trees

Apparent bytes,
which on this filesystem equal the block count the same `du` reports
(erratum below: the equality is the flag's, not the filesystem's):

- a first library build:
  788 MiB for form A,
  1410 MiB for form D with 651 MiB of incremental data;
- a first library test build:
  897 MiB for form A,
  2572 MiB for form D with 1674 MiB of incremental data;
- a rebuild:
  the incremental directory does not grow,
  638 and 635 MiB on the library tree and 1627 MiB twice on the test tree;
- a round's one tree,
  both commands:
  3410 MiB for form D with 2325 MiB of incremental data,
  and 2998 MiB for form C with 1836 MiB.

So a round that builds both commands in one directory under form D adds about 2.3 GiB to that tree,
and one under form C about 1.8 GiB,
which is smaller only because C writes nothing for the library build of this crate.

### What a live tree holds now, and what removes it

Read on 2026-11-28 between 07:34 and 07:36,
one `cargo metadata` and one stat per workspace,
never a walk of the base:

- of the 29 workspaces this repository has checked out (the main checkout and 28 worktrees),
  24 report a workspace-keyed build directory (`<base>/<2 hex>/<14 hex>`),
  and 23 hold a debug tree there;
  the other five answer with no workspace at that path or a manifest a dependency cannot load.
- those 23 trees total 37,653,572,191 bytes (35.1 GiB),
  from 506 MiB to 3.7 GiB,
  a mean of 1561 MiB.
- every one of those trees holds only `debug/build` and `release/build`:
  the build scripts' outputs,
  from 314 MiB to 3.2 GiB of `debug/build` and 506 MiB of `release/build` per tree,
  and an `incremental` directory that is empty (0 bytes) in all 23.
  No tree holds `deps` or `.fingerprint`,
  so the compiled units a first build makes are not in a build directory at all.
  (Corrected in the erratum below: `debug/build` holds the compiled units.)
- none of the 29 reports a `toolchain/<toolchain>/` path,
  which is the layout the two trees above were measured in;
  the main checkout now reports `<base>/f6/26ded3a61dae16`,
  864,182,952 bytes.
- free space:
  1,548,279,218,176 bytes (1442 GiB) with `df --block-size=1 /home`.
- nothing removes a tree by itself:
  `cargo-shadow-lean.service` reads loaded,
  disabled and inactive,
  and no cargo-shadow timer is listed.

## Errata, 2026-11-28T09:17:57-05:00

Five sentences of this page are wrong or incomplete,
found by the third gate read of
`doc/planning/cli-git-development-profile-after-the-move-to-the-new-drive.md`
and checked again in this round.
The figures beside them stand;
only what they were said to mean changes.

### What `debug/build` holds

The heading "What a live tree holds now, and what removes it" says
"every one of those trees holds only `debug/build` and `release/build`:
the build scripts' outputs ...
No tree holds `deps` or `.fingerprint`,
so the compiled units a first build makes are not in a build directory at all".

That is wrong, and the survey behind it never looked inside a tree.
Cargo's nightly build-cache reference describes `<build-dir>/debug/build/` as
"Build step results including compilation artifacts and build script output",
and `doc/decision/cargo-build-dir-host-cache.md`
("The build-dir layout is internal") lists
`debug/build/<package>/<meta-hash>/out` and `.../fingerprint`.
This page's own `-vv` logs say the same of a real build:
in the form D round 327 rustc lines write `--out-dir` under `debug/build/`,
the registry crate `itoa`'s among them,
and the test command's `Executable` line is
`<build-dir>/debug/build/monochromatic-git-policy-cli/390354d153c8f2be/out/git_policy_cli-390354d153c8f2be`.

Two further corrections follow in the same list:
the `deps=-1` in every row of the survey is an absence of evidence
(the survey looked for `debug/deps`, which this layout does not use),
and the field that reads `incremental=true` records that the directory exists,
not that it is empty:
22 of the 23 hold one and one does not, and no size was taken of any of them.

### Apparent bytes

The heading "Disk, from this round's trees" says the figures are
"Apparent bytes, which on this filesystem equal the block count the same `du` reports".
That equality is the flag's:
the harness runs `du --summarize --bytes`,
and GNU `du` documents `--bytes` as "equivalent to '--apparent-size --block-size=1'".
No allocated size was measured here,
so nothing on this page says what the ZFS pool compresses or what a snapshot keeps.

### The build-script counts under form C

The heading "Whether a build script runs a second time" says
"In the first command of both `-vv` rounds,
the detector finds 21 and 20 build-script run lines and 33 build-script compile lines"
and that the second command of the form C round has "two build-script run lines,
`monochromatic-catalog-specifier`'s and `forbidden-strings`'s".

The counts this round re-took from the same four logs:

- first command, both rounds:
  33 compile lines (`--crate-name build_script_build`),
  36 run tokens of `debug/build/<package>/<hash>/out/build_script_build`,
  and 33 distinct run paths.
- second command, form C:
  one compile line and one run token,
  both `forbidden-strings` (`CARGO_PKG_NAME=forbidden-strings`,
  run path ending `forbidden-strings/c65731ebdc1d74a6/out/build_script_build`);
  `monochromatic-catalog-specifier` is one of the seven compiled units and has no `build.rs`.
- second command, form D:
  none of either.

So the conclusion stands (under form C the test command compiles the build script again and runs it again,
and under form D it does neither)
and what changes is the count: one build script, not two.
The detector in the arm harness was `/Running .*build-script-build/`,
which never matched Cargo's `build_script_build`,
so `scriptRuns` is 0 in every arm record and the control the earlier text claimed is not in those records.

### Which build folder a round's cargo names

The heading "What the worktrees of this repository share today" says
`cargo-shadow` sets the child Cargo's build directory to
`<base>/toolchain/<toolchain>/<two hex>/<fourteen hex>`.
`apply_keyed_build_dir` (`package/cargo-shadow/cli/src/lean_keyed_folder.rs`)
does that only when `RUSTUP_TOOLCHAIN` or a `+toolchain` argument is present;
otherwise the folder is keyed by workspace, `<base>/<two hex>/<fourteen hex>`,
which is what the 2026-11-28 survey found for all 29 workspaces
(`build-directories.txt`: `toolchain-keyed: 0`).

### What a worktree shares

The same heading says a second worktree's first build
"compiles again every one of the crate's 4,012 Rust files and every dependency:
266 to 271 dependency units in 64.3 to 78.8 s ...
It shares no unit, no `rmeta`, no incremental data and no final artifact."

That is true of a worktree with no prewarmed clone in its keyed folder,
which is what that measurement built.
A worktree the prewarm filled starts with the registry and git dependencies' compiled units,
because the clone carries them and the job's clean removes only the worktree's own members,
and what it compiles is this crate and the six repository path crates outside its workspace,
whose absolute paths are hashed into their unit hashes
(`SourceId::stable_hash`, `src/workspace/source_id.rs`, unless `trim-paths` is set).
What that first build costs was not measured.
