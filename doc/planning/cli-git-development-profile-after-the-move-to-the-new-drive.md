# The Git wrapper's development profile after the move to the new drive

Proposal,
2026-11-28T08:20:00-05:00.
Status:
unapproved and not final.
Nothing here is decided,
no setting was changed on the branch that carries this file,
and the owner declares completion.

Gate reads:
two same-vendor gate reads have read earlier versions of this file,
both on 2026-11-28.
The first read the version at commit `888d3646df` and came back with four High,
ten Medium and seven Low findings;
the second read the version at commit `471d462ccf` and came back with three High,
ten Medium and nine Low findings.
Both were same-vendor second passes,
neither an independent review,
and both stood in scratch that expires a day after it was written.
This revision answers every finding of the second read,
one by one,
in the round's own report;
no gate read and no independent review has read this revision.

Every number here is a measurement of this package's own builds,
kept in `package/git-policy/cli/doc/development-profile-measurements.md`,
under its heading dated 2026-11-28 and under its heading "The recommended form measured on its own,
2026-11-28".
Each heading names the command behind each figure.
The machine's state figures were read on 2026-11-28 between 07:30 and 08:10 -05:00,
and each is named where it is used.

The setting this file is about is in `package/git-policy/cli/Cargo.toml`:

```toml
# package/git-policy/cli/Cargo.toml
[profile.dev]
debug = "line-tables-only"
incremental = false
```

## The first screen

What is asked:
four decisions,
each its own question under "Questions for the owner",
in the order they can be answered.
The first is what the owner's words of 2026-11-04 mean for this crate now,
because two of the four forms rest on the answer.
The second is which form the dev and test profile takes.
The third is whether a round passes `CARGO_INCREMENTAL` itself.
The fourth is whether anything removes build trees,
and when.

What is recommended:
form D,
`[profile.dev.package.monochromatic-git-policy-cli] incremental = true` beside `[profile.dev] incremental = false`,
which writes incremental data for this crate's own units only.
The recommendation stands if the owner's 2026-11-04 words allow this crate's own `codegen-units` to move and leave the
path dependencies alone;
if the words stand as written,
the answer is form A,
the setting as it is.

Which figures are measurements of the recommended form:
every cell of "What a rebuild costs" and "Peak memory" below was measured on form D in this round,
beside two form A controls in the same round:
the first build of the library and of the library test build,
and the rebuild of each after one line in a leaf file and after one line in a widely used file.
Form D's own first build is among them (wall 1.24 of the controls on the test build,
peak unchanged).

Which figures belong to another form:
the 2026-11-28 arms measured the everywhere form (form B),
and those figures are marked "form B" where they are used,
because B is the only form whose rebuild of the path dependencies was measured.
Form C has two rounds measured (both commands in one build directory,
at `-v` and at `-vv`);
its rebuild cells are inferred,
and the file says so.
Form D2's cells are inferred from D's,
and the file says so.

What none of the five forms changes:
a first build's peak memory.
Form D reads 0.94 to 0.96 of the controls and form B read 0.94 to 0.96,
both inside the 20% these builds can show.
The failure on 2026-11-27 was twenty-three first builds running at once,
and no form here lowers that phase.
What D and B add there is wall time:
24% and 22% more on the library test build,
so a wave of first builds holds the machine longer.

Which builds these manifest lines reach:
the host development and test builds of a round's worktree,
and nothing else.
The gate's image sets `CARGO_PROFILE_DEV_DEBUG=0` and `CARGO_INCREMENTAL=0` in its own environment,
which overrides the manifest;
the guests build with `--release`,
which the `dev` and `test` profiles do not reach;
and the container route the owner reopened on 2026-11-28 would bring its own environment and its own question.
A count of host builds by kind does not exist,
so the share of the machine's build pressure these lines reach is not known either.

## What the owner is asked for

Four decisions,
separate,
each with its own question under "Questions for the owner":

1. What the owner's words of 2026-11-04 mean for this crate now.
   They name the debuginfo level,
   incremental compilation off for the dependencies,
   and `codegen-units` off-limits,
   and they were written for the planning pass that produced `doc/planning/lean-build-dirs.md`,
   before the 2026-11-13 change moved two of those settings.
2. Which form the dev and test profile takes,
   from the five below.
3. Whether a round passes `CARGO_INCREMENTAL` itself.
   The variable overrides the manifest for every path source,
   so `1` is the everywhere form whatever the manifest holds,
   and `0` is the setting as it is.
4. Whether anything removes build trees,
   and when.

Four things the records already answer are told here and not asked:
dependency builds are shared by the prewarm the owner chose on 2026-11-03;
how many builds may run at once is bounded by guests alive and by the memory share the owner set on 2026-11-28;
`debug = false` was rejected and is not reopened;
and compiling less of the crate per round is the project's already-decided route to a smaller peak.

## What changed since the 2026-11-13 choice

Those two lines were chosen on 2026-11-13,
when a build tree was what limited how many agents worked on this package at once,
and the round that measured them wrote them into the manifest with this comment:

```toml
# package/git-policy/cli/Cargo.toml
# A build tree is what limits how many agents work on this package at once, so development and test builds
# trade debug detail and incremental data for disk.
```

Three things have changed since.

The home moved to the 3.5 TB drive on 2026-11-17 (`doc/handover/agents-at-once-on-this-machine.md`,
"The cap now"),
and `df --block-size=1 /home` read 1,548,279,218,176 bytes (1442 GiB) free at 2026-11-28T07:36-05:00.

The crate is 1,311,645 lines in 4,049 Rust files below `src/native`,
688,854 of those lines in 1,582 `*_tests.rs` files,
counted at 2026-11-28T08:22-05:00 with `find src/native -type f -name '*.rs' -exec cat {} + | wc -l`,
the same command with `-name '*_tests.rs'`,
and the file counts with `-name '*.rs' | wc -l` and `-name '*_tests.rs' | wc -l`.
The earlier round counted 1,298,623 lines in 4,012 files the same day,
so the crate grew while these rounds ran,
which is why the count is given with its time.

Memory became the limit,
and what the record of that night says is narrower than "one compiler".
`doc/handover/cli-git-rust-implementation.md`,
section "Memory",
at 21:48:24 on 2026-11-27:

> At 21:48:24 `MemAvailable` read 1549 MiB,
> swap held 50,404 of 66,219 MB,
> and the memory stall read `full avg10=46.85`,
> with `rustc` holding 28.8 GiB resident and seven guests each holding about 24 MiB of swap.

That sentence names `rustc` in aggregate,
and the same section counts "eleven `rustc` processes and nineteen rounds" at 22:51:42,
so it identifies no single compiler and no single unit.
The test units measured here peaked at 7,059 to 7,532 MiB (6.9 to 7.4 GiB),
so 28.8 GiB is about four such compilers at once,
which is what a minute of twenty-odd first builds holds;
a process's resident size should also not exceed its scope's cap,
and the night's rounds were capped at 24 GiB each.
What the night does establish is that twenty-three rounds started between 21:11 and 21:42 -05:00,
that their first builds ran together,
and that the machine ran out of memory and swap.

## What incremental compilation is

Cargo's reference says it plainly:
"Incremental compilation causes rustc to save additional information to disk which will be reused when recompiling the
crate,
improving re-compile times".
That information is a query cache and a dependency graph per crate per profile,
written under `<build dir>/debug/incremental/<crate>-<hash>/s-<session>/`,
with one `s-<session>` directory per compiler run;
the compiler garbage-collects them each time it starts in incremental mode,
deleting every session directory it cannot lock and every finalized session except the most recent one
(`compiler/rustc_incremental/src/persist/fs.rs`,
rust-lang/rust,
read 2026-11-28),
so a crate's incremental directory is bounded by that one session's size and not by how many rebuilds a tree has seen.

What it moves besides the data:
with `incremental = true` and no `codegen-units` in the manifest,
Cargo leaves the flag off and the compiler's own default follows,
which is 256 code generation units for incremental builds and 16 otherwise
(the same reference,
and rustc's codegen options page,
both fetched 2026-11-28).
So switching incremental on moves a compiler default the owner put off-limits.

### The default that moves with the flag

Cargo's reference says of `codegen-units`
(`https://doc.rust-lang.org/cargo/reference/profiles.html`,
fetched 2026-11-28):

> The default is 256 for incremental builds,
> and 16 for non-incremental builds.

and rustc's reference says the same from the compiler's side
(`https://doc.rust-lang.org/rustc/codegen-options/index.html`,
fetched 2026-11-28):

> The default value,
> if not specified,
> is 16 for non-incremental builds.
> For incremental builds the default is 256 which allows caching to be more granular.

This crate sets no `codegen-units` in `[profile.dev]`,
and the earlier round's unit-graph probe printed `codegen_units=null` for every variant it ran,
so Cargo passes no flag and the compiler's default follows the flag.
The 2026-11-13 change moved the same default the other way for every unit it touched,
which no record of that round mentions.

## How these figures were made

An arm is one build in a build directory of its own,
inside a `systemd-run --user --scope --slice=agent-work.slice` with `MemoryMax=40G`,
`CPUQuota=600%`,
`TasksMax=1024`,
`IOWeight=1` and `RuntimeMaxSec=5400`,
one build at a time,
behind a gate that opens only when the `full` line of `/proc/pressure/memory` reads `avg60` below 10.
`cargo` inside such a scope reads `nproc` as 6,
so Cargo's job count is six in every arm.

The two commands are a round's:

```sh
# run in package/git-policy/cli
cargo build --locked --offline --lib
cargo test --locked --offline --lib --features test-only-switches --no-run
```

Each arm records wall time from `date +%s%N`,
processor time from the scope's `cpu.stat`,
the largest `rustc`'s own high-water mark (`VmHWM`,
sampled once a second for every `rustc` in the scope),
the scope's `memory.peak` (the kernel's mark for everything in the scope,
page cache included),
the scope's `memory.stat` `anon` share,
and `du --summarize --bytes --apparent-size` of the tree.

This round wrote the manifest text of the arm's own form into the manifest before the arm and put the checked-out bytes
back after it,
and it set no `CARGO_INCREMENTAL`;
each arm's record holds the text it added and the five environment variables that would have changed what it measured,
all five recorded as unset.
So the arms measure the form the owner would get,
not the everywhere form,
and the earlier measurements of form B are marked wherever they are used.

Two faithful builds of one unchanged crate differed by up to 20% on this host
(`package/git-policy/cli/doc/development-profile-measurements.md`,
the 2026-11-25 section,
"The control's spread"),
and the page's own rule reads "A difference smaller than 1.20 is no difference these builds can show."
So every figure here carries the treatment's ratio to the mean of its two controls,
the range that dividing by each control alone gives,
and the spread between the two controls;
a figure inside its controls' spread is reported as no difference the measurement can show.

## The arms, and what they measured

Twenty-four arms ran on 2026-11-28.

Six first builds:
form A's library and library test build twice each,
and form D's once each,
in this order so that the treatment sits between its controls.

Twelve rebuilds after one line added at the end of one file:
the leaf file,
`src/native/candidate/candidate_store_commit_listings.rs`,
which no other Rust file names,
and the widely used file,
`src/native/worktree_clone_mount_protocol/mod.rs`,
which 273 Rust files name,
the most of any file in the crate.
Each pair is applied in one direction and removed in the other,
so the two controls of a cell are the same work in opposite directions.

Four rounds of both commands in one build directory:
two under form D and two under form C,
the first pair at `-v` and the second at `-vv`,
because Cargo prints a build script's own run only at `-vv`.

Two arms only put a tree back to its original bytes before its wide-edit arm,
and are not part of any cell.

## What a rebuild costs

Wall is what one person waits for;
processor time is what the shared machine pays.
They move differently and are kept apart here.
Every cell below is form D against form A,
measured in this round,
and each carries its controls' spread.

### Library build

First build:
wall 104.5 s against 111.0 and 89.2 s,
1.04,
range 0.94 to 1.17,
controls 24.5% apart,
so this cell shows no wall difference and its own controls differ by more than the 20% bar;
processor time 1.03 (446.5 s against 459.3 and 410.4 s),
controls 11.9% apart,
no difference shown.

Rebuild after the leaf file's line:
wall 13.6 s against 20.6 and 21.8 s,
0.64,
range 0.63 to 0.66,
controls 5.7% apart;
processor time 18.1 s against 35.3 and 37.9 s,
0.49,
range 0.48 to 0.51,
controls 7.3% apart.

Rebuild after the widely used file's line:
wall 11.3 s against 21.5 and 23.8 s,
0.50,
range 0.48 to 0.53,
controls 10.7% apart;
processor time 14.0 s against 37.9 and 39.5 s,
0.36,
range 0.35 to 0.37,
controls 4.0% apart.

### Library test build

First build:
wall 164.9 s against 136.1 and 129.7 s,
1.24,
range 1.21 to 1.27,
controls 4.9% apart;
processor time 552.0 s against 518.5 and 513.5 s,
1.07,
range 1.06 to 1.07,
controls 1.0% apart.
This is the build that takes the longest and sets the peak.

Rebuild after the leaf file's line:
wall 23.6 s against 54.0 and 54.5 s,
0.44,
range 0.43 to 0.44,
controls 0.9% apart;
processor time 27.1 s against 107.1 and 107.9 s,
0.25,
range 0.25 to 0.25,
controls 0.7% apart.

Rebuild after the widely used file's line:
wall 25.4 s against 62.6 and 52.9 s,
0.44,
range 0.41 to 0.48,
controls 18.3% apart;
processor time 27.5 s against 117.8 and 106.6 s,
0.25,
range 0.23 to 0.26,
controls 10.5% apart.

### What that adds up to for a round

On the library test build,
form D's first build costs 32.0 s more wall and 36.0 s more processor time,
and both edits give that back at about one rebuild.

At k rebuilds after its first build,
the wall is these fractions of today:

- leaf edit:
  1.01 at k=1,
  0.88 at k=2,
  0.80 at k=3,
  0.70 at k=5,
  0.59 at k=10;
- wide edit:
  1.00,
  0.87,
  0.79,
  0.69 and 0.59 at the same k;
- the processor time at the same k is 0.93,
  0.83,
  0.76,
  0.65 and 0.52 (leaf),
  and 0.92,
  0.82,
  0.74,
  0.64 and 0.51 (wide).

The first build's extra wall is paid back after 1.04 rebuilds on the leaf edit and 0.99 on the wide edit;
its extra processor time after 0.45 and 0.42.

A round that makes no rebuild after its first build pays 1.24 in wall and 1.07 in processor time and earns nothing back.
How many rebuilds a round makes is not recorded anywhere:
no count of `cargo build` or `cargo test` scopes per round exists,
and this round searched `doc/` and the package's own docs for one rather than guessing.
It is the one input the ranking cannot settle from these arms.

## Peak memory, per phase and per reading

Three readings per arm,
taken together:
the largest compiler's own high-water mark,
the scope's `memory.peak`,
and the scope's anonymous share.
They disagree in size and agree in direction at every cell of this round.

### First builds

Library build:
largest compiler 2835 MiB against 2837 and 2848 MiB,
1.00,
controls 0.4% apart;
scope peak 1.00 (3780 against 3807 and 3753 MiB);
anonymous 1.01 (3530 against 3523 and 3473 MiB).

Library test build:
largest compiler 7059 MiB against 7445 and 7532 MiB,
0.94,
range 0.94 to 0.95,
controls 1.2% apart;
scope peak 0.94 (7179 against 7578 and 7659 MiB);
anonymous 0.96 (6741 against 7054 and 7020 MiB).

Under the page's 20% bar none of these shows a difference,
so **a first build's peak is unchanged as far as this measurement can tell**,
not lowered.
The earlier round read form B at 0.94 to 0.96 on the same three readings.

### Rebuilds

Library build after the leaf edit:
largest compiler 2192 MiB against 2853 and 2813 MiB,
0.77,
controls 1.4% apart;
anonymous 0.72;
scope peak 0.88,
inside the bar.

Library test build after the leaf edit:
largest compiler 5057 MiB against 7525 and 7519 MiB,
0.67,
controls 0.1% apart;
scope peak 0.77,
anonymous 0.63.

Library build after the wide edit:
largest compiler 2071 MiB against 2860 and 2815 MiB,
0.73,
controls 1.6% apart;
anonymous 0.68;
scope peak 0.83,
inside the bar.

Library test build after the wide edit:
largest compiler 4961 MiB against 7544 and 7539 MiB,
0.66,
controls 0.1% apart;
scope peak 0.75,
anonymous 0.63.

So on a rebuild the compiler's own peak falls to about two thirds,
the scope's peak to about three quarters,
and anonymous memory to about three fifths,
and all four cells agree in all three readings.

### The dependency phase has a peak the sampler cannot name

The sampler counts processes named `rustc`,
once a second,
and the peak it reports for the scope is the kernel's `memory.peak`,
page cache included.
A peak reached while no `rustc` is in the scope is therefore not a compiler's.

In this round's two form A library first builds,
the scope's peak (3807 and 3753 MiB) was reached during stretches with no `rustc` in the scope,
13 one-second samples each,
while the largest compiler peaked at 2837 and 2848 MiB.
In the two form A library test builds,
the samples with no `rustc` in the scope stood at 3761 and 3748 MiB,
and this crate's unit started at 72.7 s of 136.1 s and 72.3 s of 129.7 s.
The 2026-11-25 research names the level and the cause,
in the paragraph "A build's peak is one process" of
`doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md`:
"the `forbidden-strings` build script's run at 3756 MiB in every worktree where its output is not fresh".
This round measured the level and cannot name the process:
the attribution to that build script is the earlier record's,
and the build script's own runs were seen only in the `-vv` rounds,
which report wall and processor time and not memory.

### What that leaves for the builds, now that the guests hold three quarters

The owner's answer of 2026-11-28
(`doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`,
"The unit's cap")
puts the guests' unit at three quarters of the machine's memory,
set on the running unit at 2026-11-28T05:45:26-05:00 as 49,664,753,664 bytes,
and bounds guests alive at six.
The same answer says what that leaves:
"host builds,
the gate (its unit may take 20 GiB) and the desktop share 15.4 GiB,
so they swap instead".

That 15.4 GiB is therefore a ceiling for the builds and not a share of their own:
the gate's own allowance is larger than the whole of it,
and the desktop lives in it too.
Dividing it by one build gives an upper bound on how many builds fit,
never a promise:

- largest compiler,
  today 7445 MiB:
  two fit;
  on a rebuild under form D,
  5057 MiB:
  three fit;
- scope peak,
  today 7578 MiB:
  two;
  on a rebuild under form D,
  5737 MiB:
  two;
- anonymous,
  today 7054 MiB:
  two;
  on a rebuild under form D,
  4510 MiB:
  three.

So incremental compilation moves the count from two to three on two of the three readings,
for a round that is already in its edit loop,
and it moves nothing at all for a first build.
That is the whole of the capacity claim these arms support.
The earlier version's "about eight compilers" divided the machine's total memory by one reading and counted memory the
guests hold.

## What the data costs and who removes it

### What a tree holds now

Read on 2026-11-28 between 07:34 and 07:36,
with one `cargo metadata` and one stat per workspace,
never a walk of the base:

- of the 29 workspaces this repository has checked out (the main checkout and 28 worktrees),
  24 report a workspace-keyed build directory (`<base>/<2 hex>/<14 hex>`),
  and 23 hold a debug tree there;
  the other five answer with no workspace at that path or a manifest a dependency cannot load.
- those 23 trees total 37,653,572,191 bytes (35.1 GiB),
  from 506 MiB to 3.7 GiB,
  1561 MiB at the mean.
- every one of them holds only `debug/build` and `release/build`,
  the build scripts' outputs,
  and an `incremental` directory that is empty (0 bytes) in all 23;
  none holds `deps` or `.fingerprint`.
  So a worktree's build directory carries the dependency phase's build-script outputs and none of the compiled units,
  and the incremental data the 2026-11-13 setting would write is simply absent from all 23.
- none of the 29 reports a `toolchain/<toolchain>/` path.
  The earlier version of this file measured two trees in that layout (2.03 and 3.01 GiB)
  and counted 2 of 34 workspaces as holding a tree;
  that layout is not the one the workspace's own cargo names today,
  and the count that follows from the right layout is 23 of 29.
- free space:
  1,548,279,218,176 bytes (1442 GiB),
  `df --block-size=1 /home`.

### What the forms add, and what a day of them is

Measured this round,
apparent bytes,
which equal the block count on this filesystem:

- a first library build:
  1410 MiB for form D,
  of which 651 MiB incremental;
- a first library test build:
  2572 MiB for form D,
  of which 1674 MiB incremental;
- a round's one tree with both commands:
  3410 MiB for form D,
  of which 2325 MiB (2.27 GiB) incremental,
  and 2998 MiB for form C,
  of which 1836 MiB;
- form A adds nothing;
- a rebuild does not grow the incremental directory:
  638 and 635 MiB on the library tree,
  1627 MiB twice on the test tree.

Multiplying the round's figure by the rate this file already carries,
distinct `git-policy` branch names merged per day
(`git log --merges --since=<day>T00:00:00-05:00 --until=<day>T23:59:59-05:00 --format='%s'`,
then the quoted names,
deduplicated):
62 on 2026-11-25,
54 on 2026-11-26,
166 on 2026-11-27,
24 on 2026-11-28 to 06:20,
which is 24 to 166 a day:

- 2.27 GiB a tree is 54 to 377 GiB a day,
  which spends the 1442 GiB free in 3.8 days at the high rate and in 26 days at the low one;
- on a first build alone (1674 MiB a tree) it is 39 to 271 GiB a day,
  which is 5.3 days at the high rate and 37 days at the low one.

That multiplication is a ceiling in one direction and a guess in the other:
the rate is merged branches,
which is not the number of built trees,
and rounds that never merge leave trees too.
The count of trees that stay is the base's own,
and it is not counted today.

### What removes a tree today

Nothing removes a tree by itself:

- `cargo-shadow lean` removes a tree once its workspace is gone and the tree has been idle for `TREE_IDLE_SECONDS`,
  seven days (`package/cargo-shadow/cli/src/lean_policy.rs`);
- the unit that would run that pass,
  `cargo-shadow-lean.service`,
  reads loaded,
  disabled and inactive at 2026-11-28T07:37-05:00,
  and `systemctl --user list-timers --all` lists no cargo-shadow timer;
- so a tree stays until a person runs a pass or removes it.

What a person does,
whichever form the owner picks:

- the data going stale:
  `cargo clean` in the package clears that workspace's whole build directory,
  and deleting `<base>/<2 hex>/<14 hex>/` for a workspace no longer built removes a tree;
  the compiler's own garbage collection already bounds each crate's incremental sessions to one;
- the disk filling:
  the trees of removed worktrees go first,
  then idle trees of live ones;
  a pass bounded to trees is `cargo-shadow lean run --only trees --deadline-minutes <n>`;
- a pass over every tree cost about 6.6 h at the size the base had on 2026-11-10,
  when a run judged 14,850 of 33,804 trees in 10,475 s (`package/cargo-shadow/cli/README.md`,
  "Keeping the build folders lean").
  That count is the base as it stood on the old drive before the home moved on 2026-11-17;
  this round did not count the base as it stands now,
  because a walk of it is closed to agent runs,
  so it says what the population was then and nothing about today's.

Two records the second gate read asked for,
and what each still shows:

- `doc/handover/cli-git-rust-implementation.md` at commit `342d6c5a59` (deleted on 2026-11-23 by `f0f870e305`),
  lines 9473 to 9478,
  in the section "The Tokio step's four Kimi K3.1 Pro findings:
  the review's own controls reproduced,
  and all four mended (2026-11-11T03:53:21-05:00)":

  > and one stale incremental session of this crate's own target directory had grown until a single `rustc`
  > had read 10.8 GB from the volume at 98 percent full;
  > removing that crate's stale sessions in this worktree's target directory freed 23 GB and the next build
  > ran in the times above.

  That is 2026-11-11,
  two days before the `incremental = false` line,
  so it is the default profile on the old drive,
  and it shows that one crate's incremental sessions can reach tens of gigabytes when the crate is this large.
  It does not show a tree growing without bound across rebuilds:
  the compiler keeps one finalized session per crate,
  and what grew there was one session's own size,
  across the crate's several profiles and its path dependencies.
- the 33,804 trees and the 6.6 h pass above were measured on 2026-11-10,
  before the home moved;
  they show the base's population and the cost of a full pass at that size,
  and not today's.

## What is already shared, and what it does not share

Dependency builds are shared across worktrees today,
by the prewarm the owner chose,
and that answers the question an earlier version of this file asked as "should dependency builds be shared at all".

The owner's words of 2026-11-03 are kept in `doc/planning/cli-git-worktree-add-package-repair.md`,
"What the repair exists to guarantee":

> The Cargo repair exists because Cargo bakes absolute paths into what it builds,
> so artifacts built at the source checkout's path are wrong for the worktree's own crates,
> while registry and git dependencies hold nothing path-specific and can be reused.
> The owner chose that on 2026-11-03 as "Reuse deps,
> rebuild own"

What the mechanism does,
from the job's own header (`package/git-policy/cli/src/native/repair/package_repair_prewarm_job.rs`)
and the plan that decides when it runs (`package_repair_cargo_build_state_plan.rs`):
for each workspace whose build directory the source checkout has and the new worktree lacks,
the commit service clones the source's build directory into a staging directory beside the destination,
runs `cargo clean --package <the worktree's own members>` against the copy with `CARGO_BUILD_BUILD_DIR` and
`CARGO_TARGET_DIR` pointing at directories of the job's own,
and publishes it by a rename that never replaces.

What it carries:
the build directory,
which on this machine holds the build scripts' outputs and nothing else
(measured above:
`debug/build` 314 MiB to 3.2 GiB and `release/build` 506 MiB per tree,
no `deps`,
no `.fingerprint`).
So a new worktree starts with every dependency's build script already run,
including the `forbidden-strings` one whose run is the dependency phase's own peak,
and compiles every dependency's units itself.

What it reaches today,
read from the commit service's work records and one `cargo metadata` per workspace on 2026-11-28:

- 2056 prewarm records for this repository:
  2037 done and 19 failed,
  the newest written at 2026-11-28T12:28:39Z;
- 27 of the 29 live workspaces are named by a done prewarm record,
  and each record's entry for this crate names a destination under `/home/user/.cargo/build/<2 hex>/<14 hex>`;
- 23 of the 29 hold a debug tree at exactly the build directory their own cargo names,
  and several prewarmed ones hold a tree of the same size as each other (820,056,567 bytes),
  which is what clones of one source tree look like.

So the count the earlier version of this file gave (2 of 34) was a count of the wrong layout,
and the mechanism it said does not exist is the one that made 23 of today's trees.

What is still not shared:
the compiled dependency units.
They are not in the build directory at all,
so a fresh worktree compiles 266 to 271 dependency units in 64.3 to 78.8 s of the first builds measured here,
and no mechanism on this machine avoids that.
Extending the prewarm to carry them would make it a reader of the build directory's layout,
which `doc/decision/cargo-build-dir-host-cache.md` allows no reader of except the lean sweep;
that is a design of this package's own and not a profile setting,
and it is not put to the owner.

What the earlier version called option E,
"a shared build directory or a compiler cache",
is therefore answered:
the owner chose the mechanism,
it runs,
and what remains are two engineering questions inside it,
which a later round can take up without a decision here.

## The options

Each option names its line and its file,
what it changes for good,
what lands on disk,
the loop and the first build in the arms' own numbers,
and what a yes would still leave undecided.

Two things are not offered again.
Compiling less of the crate per round is the project's already-decided route to a smaller peak
(`doc/planning/cli-git-source-layout.md`,
"Splitting into crates (owner,
2026-11-04)"),
and `debug = false` is rejected;
neither is measured here.

### A. The setting as it is

`package/git-policy/cli/Cargo.toml`,
`[profile.dev]` holding `debug = "line-tables-only"` and `incremental = false`,
unchanged.

What it does:
the compiler saves no incremental information,
so every edit compiles the whole crate again;
file and line stay in every backtrace frame,
because line tables are kept,
and the compiler's 16 code generation units stay 16.

What lands where:
a first library build's tree is 788 MiB and a first library test build's 897 MiB,
both the build scripts' outputs;
no incremental data at all.

What it gives:
the smallest first build of the five forms,
and the only one whose first build is today's (wall 136.1 and 129.7 s on the library test build,
processor time 518.5 and 513.5 s).

What it costs:
the loop.
A rebuild after one line costs 20.6 to 23.8 s of wall and 35.3 to 39.5 s of processor time on the library build,
and 52.9 to 62.6 s and 106.6 to 117.8 s on the library test build,
and the compiler peaks at 7,445 to 7,544 MiB every time,
on the first build and on every rebuild.

What a yes leaves undecided:
nothing about this form,
and everything the other forms ask stays as it is.

### B. Incremental compilation for everything

`package/git-policy/cli/Cargo.toml`,
one line added beside the two:

```toml
# package/git-policy/cli/Cargo.toml
[profile.dev]
debug = "line-tables-only"
incremental = true
```

What it does:
the test profile inherits `dev`,
so both commands compile this crate and every path dependency with incremental data,
and every affected unit's `codegen-units` default moves from 16 to 256.

What lands where:
`debug/incremental/<crate>-<hash>/s-<session>/` below the keyed build directory,
for this crate and for its path dependencies,
their own path dependencies and their build scripts.
The 2026-11-28 arms on form B wrote nine such directories,
and this round's form C round wrote nine too.
Measured on form B on 2026-11-28:
a first library tree 1551 MiB of which 808 MiB incremental,
a first library test tree 2705 MiB of which 1824 MiB incremental,
1,616 MiB of that this crate's own and 161 MiB the path dependencies'.

What it gives:
form B's measured cells,
which are the loops the 2026-11-28 arms measured:
the library test rebuild at 0.26 of the controls' processor time and 0.50 of their wall,
the library rebuild at 0.41 and 1.18 (with the package cache lock named there),
and the peaks at 0.67 to 0.72 of today.

What it costs:
a first library test build at 1.22 of today's wall,
1.8 GiB more per tree,
the path dependencies' 161 MiB that no rebuild of this crate reads,
the dependency's units compiled twice when a round runs both commands under the test profile only,
and the owner's 2026-11-04 words on the dependencies.

What a yes leaves undecided:
the `codegen-units` reading,
who removes stale trees,
and whether the units this file never measured (`--all-targets`,
the `native_binary` consumer target) are inside it.

### C. Incremental compilation in the test profile alone

`package/git-policy/cli/Cargo.toml`,
one line added beside the two:

```toml
# package/git-policy/cli/Cargo.toml
[profile.dev]
incremental = false

[profile.test]
incremental = true
```

What it does:
`cargo test` builds with incremental data and `cargo build --lib` does not,
and the test profile inherits the `dev` package overrides,
so the path dependencies become incremental in the test command too.
Cargo compares units by a tuple that includes `incremental` (`Profile::comparable()` in `src/workspace/profiles.rs`,
fetched 2026-11-28),
so a round that runs both commands holds the dependencies' units twice.

This round measured that:
in the `-vv` round,
the test command compiled seven units (the seven path dependencies and this crate),
wrote nine incremental directories,
and ran two build scripts again
(`monochromatic-catalog-specifier`'s and `forbidden-strings`'s),
against form D's one compiled unit,
two directories and no build-script run.
The build-script *run* unit is not doubled by the graph,
because Cargo strips its profile to fields that carry no `incremental` (`get_profile_run_custom_build`),
and the run repeats anyway when the compiled script's unit changes,
which is what the round shows.

What lands where:
the test command's data:
1836 MiB in a round's one tree,
in nine directories,
and none for the library build.

What it gives:
the library test build's loop,
inferred from form D's cells because D and C differ only in the data the dependencies hold,
which no rebuild of this crate reads:
0.44 of the controls' wall,
0.25 of their processor time,
0.66 to 0.75 of their peak.
The library build keeps today's numbers.

What it costs:
1836 MiB of data a tree against D's 2325 MiB,
which is less only because nothing is written for the library build;
the dependencies' units compiled twice when a round runs both commands,
which is 268.4 s of processor time in the test command against D's 140.7 s;
two build scripts run again per round;
and the dependency part of the owner's words,
as B breaks it.

What a yes leaves undecided:
the `codegen-units` reading,
the dependency constraint,
who removes stale trees,
and the two command's units.

### D. Incremental compilation for this crate's own units (recommended)

`package/git-policy/cli/Cargo.toml`,
one line added beside the two:

```toml
# package/git-policy/cli/Cargo.toml
[profile.dev]
incremental = false

[profile.dev.package.monochromatic-git-policy-cli]
incremental = true
```

What it does:
this crate's units are incremental under both commands,
because the dev package override reaches the test profile,
and the path dependencies stay as they are,
so their units are shared between the two commands and hold no incremental data.

What lands where:
only this crate's data:
651 MiB on a first library tree,
1674 MiB on a first library test tree,
2325 MiB on a round's one tree that runs both commands,
in one directory for one command and two for both,
each named `git_policy_cli-<hash>` and no other crate's.
Nothing for the dependencies,
which this round confirmed by listing the directories.

What it gives:
the loops measured on this form in this round,
on both edits:
0.64 and 0.50 of the controls' wall on the library build,
0.44 and 0.44 on the library test build,
0.49 and 0.36 of the processor time on the library build,
0.25 and 0.25 on the library test build,
and on a rebuild 0.66 to 0.77 of the compiler peak,
0.75 to 0.88 of the scope peak,
0.63 to 0.72 of anonymous memory.

What it costs:
a first library test build at 1.24 of the controls' wall (164.9 s against 136.1 and 129.7 s)
and 1.07 of their processor time,
a first library build whose controls disagree by 24.5% so that no difference can be shown there,
2325 MiB more on a round's tree,
and the owner's 2026-11-04 words on `codegen-units`,
which every form moves for the units it touches.

What a yes leaves undecided:
the `codegen-units` reading,
who removes stale trees,
whether a round passes `CARGO_INCREMENTAL`,
and whether the units this file never measured are inside it.

A named caveat:
the profile table is keyed by the package name,
so renaming the package without renaming this line stops the line from applying.
Cargo warns about that,
profile package spec `{}` in profile `{}` did not match any packages
(`src/workspace/profiles.rs`,
fetched 2026-11-28),
so the line announces its own death.

### D2. Incremental compilation for this crate's test build alone

`package/git-policy/cli/Cargo.toml`,
one line added beside the two:

```toml
# package/git-policy/cli/Cargo.toml
[profile.dev]
incremental = false

[profile.test.package.monochromatic-git-policy-cli]
incremental = true
```

What it does:
the test command compiles this crate's own unit with incremental data,
`cargo build --lib` does not,
and the dependencies stay cold and shared.

What lands where:
1674 MiB of this crate's data on a first library test tree,
nothing for the library build and nothing for the dependencies,
which is the smallest of the four forms that write anything.

What it gives:
the library test build's loop and peaks,
inferred from D's cells,
because the two forms differ only in the library build's own data,
which no test rebuild reads;
and nothing at all for the library build.

What it costs:
the same first-build wall as D on the test build (inferred),
and it leaves the library build's loop at today's cost for good:
20.6 to 23.8 s of wall and 35.3 to 39.5 s of processor time a rebuild,
which D brings to 13.6 and 11.3 s.

What a yes leaves undecided:
the `codegen-units` reading,
who removes stale trees,
whether the library build's loop is worth its own 651 MiB later,
and confirmation of the test build's first build on this form.

### The round's own variable, a complement and not an option here

A round's task can pass `CARGO_INCREMENTAL`,
which Cargo applies over any manifest for every path source:
the override is applied before the check that leaves non-path sources non-incremental
(`src/workspace/profiles.rs`,
fetched 2026-11-28).
So `1` is form B whatever the manifest holds,
data for the dependencies included,
and `0` is form A.
Nothing in the manifest affects either,
and a round that forgets the variable gets whatever the manifest holds.
Cargo's reference names two more routes to the same flag,
the `build.incremental` config key and the per-profile `CARGO_PROFILE_<name>_INCREMENTAL` variables,
and this repository already sets `CARGO_INCREMENTAL=0` in several images and tasks
(`package/linter/monochromatic-lint/test.Containerfile`,
`package/git-policy/cli/bin/e2e-native-container.ts`,
`package/git-policy/cli.fuzz/bin/fuzz-build.ts`),
so a round's environment is the third place the value can come from.

This is the third question and not a sixth option.

## Combinations that cannot be built

- D,
  D2,
  C or B while the owner's 2026-11-04 words stand as written:
  each of them moves `codegen-units` for the units it touches from 16 to 256 by the compiler's default,
  and the words put that setting off-limits.
  A is the only form that leaves it alone.
- B or C while the words' dependency clause stands:
  both write the path dependencies' incremental data,
  and C's test command splits their units from the library command's.
  D and D2 leave the dependencies alone.
- D or D2 with a round passing `CARGO_INCREMENTAL=1`:
  the variable wins,
  so that round is B in effect and D2's test-only scoping turns into B for both commands.
- C with a round passing `CARGO_INCREMENTAL=1`:
  the same,
  so C becomes B.
- Any form with a round passing `CARGO_INCREMENTAL=0`:
  that round builds as A,
  which is coherent as a per-round escape and the only "both" this file offers.

One combination can be built and is unmeasured here:
pinning `codegen-units = 16` beside an incremental form.
It would take back part of what the data buys,
because Cargo's reference says 256 is what allows the caching across a crate to be more granular,
and nothing measured here says how much.

## Ranking

The criterion is what each form delivers for the workload the records describe:
many rounds at once,
each in a worktree of its own,
each paying one first build and then k rebuilds,
on a machine whose memory is the limit
(`doc/handover/cli-git-rust-implementation.md`,
"Memory":
twenty-three rounds started together on 2026-11-27;
166 `git-policy` branches merged that day).
Effort to build a form is a cost named beside it and never a reason to rank it lower.

The order depends on the first question,
so here it is under each of its answers.

### If the owner's words stand as written and bind this crate now

Ranking:
A alone.
Every other form moves `codegen-units` for the units it touches,
which the words put off-limits,
and B and C also write dependency data.
So the answer is A,
and the two lines of 2026-11-13 stand.

### If the words stand for the dependencies and this crate's own units may move

Ranking:
D > D2 > A.
D and D2 leave the dependencies' settings alone and write no dependency data;
B and C cannot be built here,
because their lines move the dependencies' `codegen-units` and write their data.

- D over D2:
  D delivers both commands' loops,
  0.64 and 0.50 of the controls' wall on the library build's rebuild and 0.49 and 0.36 of its processor time,
  which D2 leaves at 20.6 to 23.8 s and 35.3 to 39.5 s for good.
  D pays 651 MiB of this crate's own library data for that.
- D2 over A:
  the same test loop D delivers (0.44 of wall,
  0.25 of processor time,
  0.66 to 0.75 of peak)
  and 1674 MiB of data a test tree,
  where A gives the smallest first build and no data at all and pays 52.9 to 62.6 s of wall a test rebuild.

### If the words were about that planning pass only and do not bind this crate now

Ranking:
D > B > D2 > C > A.

- D over B:
  both deliver the measured loop on both commands,
  and D now has its own measured cells for it (0.64 and 0.50 of the controls' wall on the library build's rebuild,
  0.44 and 0.44 on the library test build's,
  0.49,
  0.36,
  0.25 and 0.25 of the processor time);
  D writes 1674 MiB on a first test tree against B's 1824 MiB,
  keeps the dependencies' 161 MiB out,
  and keeps their units shared between the two commands.
  What D gives up is B's longer measurement record:
  B's cells were measured on 2026-11-28 in a round of their own.
- B over D2:
  B gives the library build's loop as well,
  and D2 leaves it at today's cost for good.
  B pays 651 MiB of this crate's library data and 161 MiB of dependency data for that.
- D2 over C:
  the same library test loop,
  with no dependency data and one set of dependency units per tree,
  where C writes 161 MiB of dependency data,
  splits the dependencies' units between the two commands
  (seven units compiled in the test command,
  268.4 s of processor time against D's 140.7 s),
  and runs two build scripts a second time each round.
- C over A:
  C gives the library test build's loop for 1836 MiB of data a tree,
  where A gives the smallest first build and no data at all.
  A is the answer where a round makes no rebuild after its first build,
  because there the 1.24 of wall is paid and nothing earns it back;
  on the test build's wall arithmetic the two are level at one rebuild and C leads from two.

Two things the ranking says once,
for all three orders:

- no form lowers a first build's peak,
  which is what failed on 2026-11-27;
  D and B make a first build 24% and 22% longer in wall,
  so a wave of them holds the machine longer,
  and only A leaves that phase as it is.
- what it does not settle is how many rebuilds a round makes,
  which no record holds,
  and the bound on concurrent builds,
  which the records already answer in the section that lists what is not asked.

## Questions for the owner

Each question is one decision.
The first is asked first because the second's answer depends on it.

### Question 1. What do the words of 2026-11-04 mean for this crate now?

The words are quoted below,
and they come from `doc/planning/lean-build-dirs.md`,
"Owner requirements that bound this" (lines 28 to 48) and "What the repository already owns" (lines 975 to 981):

> "(debuginfo level,
> incremental off for dependencies - no messing with these."

> "other profile settings (split-debuginfo,
> strip,
> codegen-units) should be off-limits too"

> So no build profile setting is changed or proposed here,

Three readings of them are in the records,
and this file cannot choose between them:

- as written,
  binding this crate now:
  the records of that day say the same thing in their own words,
  and the commit that recorded them says "(debuginfo and incremental untouched)"
  (`37caad8dbb`,
  2026-11-04T16:52:44-05:00).
  Under this reading the 2026-11-13 change was a deviation,
  and the settings return to what they were,
  or the words are lifted for it.
- for the dependencies only:
  the words name "incremental off for dependencies" as their subject,
  so a package override that moves nothing but this crate's own units is not what they forbid,
  while `codegen-units` moving for this crate alone is a question here.
- for that planning pass only:
  they bounded what `doc/planning/lean-build-dirs.md` could propose,
  the pass is over,
  and the 2026-11-13 round already changed two of the four settings without recording an exception.

What the records also hold,
and it is not an answer:
the old handover at `342d6c5a59` records the 2026-11-13 change as merged and "not released by itself"
under "Adopted while the owner was away",
with "No record settles this one;
it is the coordinating session's judgement",
and no record of `doc/decision/` or `doc/handover/` holds the owner's acceptance of the two lines or a rejection of
`debug = false`.

What a yes to each reading leaves undecided:

- as written:
  which form the profile takes is answered for now (A),
  and the 2026-11-13 change's status is still unrecorded,
  so a later round would ask again.
- for the dependencies only:
  whether `codegen-units` may move for this crate's own units,
  which is the other half of every incremental form,
  and whether pinning it at 16 beside incremental is wanted,
  which is unmeasured.
- for that pass only:
  nothing about the forms,
  and the 2026-11-13 change stays recorded as it is,
  with no owner's word behind it.

### Question 2. Which form do the dev and test profile take?

The five forms are A,
B,
C,
D and D2 above,
each with its line,
its file,
its data,
its measured or inferred figures and what it leaves undecided.
The recommended one is D,
and the ranking under each answer to Question 1 is in "Ranking".

The three facts the decision turns on,
each measured this round:

- a rebuild under D costs 0.44 to 0.64 of today's wall and 0.25 to 0.49 of its processor time,
  and 0.66 to 0.77 of the compiler's peak,
  on both edits and on both commands;
- a first build under D costs 1.24 of today's wall on the library test build and moves the peak not at all,
  and a first build is what twenty-three rounds did at once on 2026-11-27;
- a round's tree under D grows by 2325 MiB,
  and 24 to 166 such trees a day spends the free space in 3.8 days at the high rate and 26 days at the low one.

A yes to a form leaves undecided:
who removes stale trees (Question 4),
whether a round passes the variable (Question 3),
what the unmeasured units cost (`--all-targets`,
the `native_binary` consumer target),
and,
for D and D2,
whether `codegen-units` may move for this crate's units (Question 1).

### Question 3. Should a round pass `CARGO_INCREMENTAL` itself?

Three answers,
and they are exclusive:

- no round passes it:
  the manifest decides,
  and a person who wants A's build for one round types `0`;
- rounds pass `0`:
  every round builds as A whatever the manifest holds,
  which makes Question 2's answer moot for the rounds that do it and keeps the data from being written;
- rounds pass `1`:
  every round is form B,
  the dependencies' 161 MiB included,
  whatever the manifest holds.

The recommendation is the first,
because it is the only one that leaves one place to look and because `0` remains reachable without a decision.
What the arms show about the value itself:
`1` was the form B arms' own environment,
and `0` is form A's,
so both values are measured,
but no arm measured a round that sets the variable while a different form's manifest is in place.

A yes to each answer leaves undecided:

- no round passes it:
  what a round that needs the escape does,
  and who notices a round's environment carrying the variable from elsewhere,
  which is the third place the value can come from;
- rounds pass `0`:
  when a round is allowed to use it,
  and that the manifest's answer then buys nothing;
- rounds pass `1`:
  who removes the data,
  and that Question 2's answer is moot for every round.

### Question 4. Should anything remove build trees, and when?

Three answers,
and they are exclusive:

- nothing,
  as today:
  23 trees are 35.1 GiB of 1442 GiB free,
  no pass runs by itself,
  and a tree stays until a person removes it.
  This is what happens now and costs nothing to keep.
- a person runs the pass when the disk fills:
  `cargo-shadow lean run --only trees --deadline-minutes <n>`,
  which at the base's 2026-11-10 size cost about 6.6 h for the whole base and can be bounded to a deadline;
  nothing starts by itself and nothing has to be maintained.
- a standing pass:
  enabling the disabled `cargo-shadow-lean.service` or adding a timer,
  which removes trees once their workspace is gone and they have been idle seven days,
  with the same 6.6 h cost per full pass and nobody deciding when it runs.

The recommendation is the second,
because the multiplication above says 3.8 to 26 days of free space at today's rate and no tree has filled the disk,
and because a standing pass is a change to the machine that no measurement here asks for.

A yes to each answer leaves undecided:

- nothing:
  what happens when the disk does fill,
  which is the question this answer declines;
- a person runs it:
  what bound the pass gets and when a person decides to run it;
- a standing pass:
  which mechanism (the disabled unit or a timer),
  what deadline and what idle rule,
  and whether "idle" means the worktree is gone or the tree has not been touched.

## What is not asked, and why

- Whether dependency builds should be shared across worktrees.
  The owner chose the prewarm on 2026-11-03,
  it runs (27 of 29 live workspaces named,
  23 holding a filled tree),
  and what it does not carry is a design question inside it,
  not a decision for the owner.
- Whether anything should bound how many wrapper builds run at once.
  The records already answer it:
  guests alive is the bound the owner chose on 2026-11-28 (six),
  the guests' unit takes three quarters of the memory and leaves 15.4 GiB for the builds,
  the gate and the desktop,
  no start or merge check waits on memory or on a spacing between starts
  ("I didn't tell you to apply the 3 minutes apart and memory test rules.",
  the owner on 2026-11-26,
  quoted in the handover's "Memory"),
  and the agent-count search is a separate bound.
  The 15.4 GiB holds two wrapper test builds today and three on a rebuild with incremental data,
  which is a consequence of the form and not a new bound.
- `debug = false`.
  The 2026-11-13 page records it as rejected for what it costs a backtrace,
  and how much memory it saves is measured neither there nor here,
  so asking again would invite a yes or a no on a guess.
- The crate split.
  It is the decided route to a smaller peak,
  and it changes every figure here.
- The container route for building the test binaries outside the guest.
  The owner reopened that question on 2026-11-28,
  and it would bring its own environment and its own measurement.

## What was not measured

Named so that no reader takes its absence for a result:

- Form B's cells in this round:
  the 2026-11-28 arms measured B,
  and every B figure here is theirs,
  taken in a round of its own;
  they were not repeated beside this round's controls.
- Form C's rebuild cells and form D2's cells:
  inferred from D's,
  which differ from C in the dependencies' data and from D2 only in the library build's own data.
  What was measured for C is two rounds of both commands,
  and what it writes and runs.
- `codegen-units` pinned to 16 beside an incremental form.
- The units this crate builds besides the library and its test unit:
  `--all-targets`,
  the `native_binary` consumer target,
  `--bin`,
  clippy and `cargo check`.
- A second worktree's first build at its own path:
  this round built in build directories of its own,
  not at a worktree's key,
  so what a fresh worktree pays is argued from the first builds and from the prewarm's behaviour.
- The effective `-C codegen-units` in a real invocation:
  the earlier round's unit graph printed `null` for every form,
  and the flag itself was not read off a compile here.
- What a rebuild writes:
  the arms record sizes,
  not bytes written,
  and apparent sizes equal block counts on this filesystem,
  so nothing here says what the ZFS pool compresses or what a snapshot retains.
- How many rebuilds a round makes,
  and how many builds by kind run on this host:
  no record holds either.
- Anything on this host's ZFS pool:
  no command here touched it.

## What this cannot touch

Whichever form the owner picks,
the recorded pressure of 2026-11-27 is only partly inside its reach:

- the gate's image sets `CARGO_PROFILE_DEV_DEBUG=0` and `CARGO_INCREMENTAL=0` in its own environment
  (`package/git-policy/cli/bin/native-test-containerfile.ts`),
  so it ignores every form here;
- the guests' builds are release builds inside the guests
  (`cargo build --release --test native_binary`,
  `cargo test --release --lib --no-run`,
  `doc/planning/mvm-guest-memory-held-for-a-rounds-life.md`),
  and the `dev` and `test` profiles do not reach them:
  one release compiler of this crate's tests was measured at 7,019,956 kB of a guest (6.69 GiB);
- the builds' host share is what these options lower,
  and only the host development and test part of it.
