# The Git wrapper's development profile after the move to the new drive

Proposal,
revised 2026-11-28T09:38:07-05:00.
Status:
unapproved and not final.
Nothing here is decided,
no setting was changed on the branch that carries this file,
and the owner declares completion.

Three same-vendor gate reads have read earlier versions of this file,
all on 2026-11-28,
and this revision answers the third of them finding by finding;
what each finding asked and what settled it is in the round's report,
which stood in scratch that expires a day after its last change and is not kept in this repository.
The commit each read saw and the count of each severity are in "The gate reads this file has had",
below the questions.
All three were same-vendor second passes,
not one of them an independent review,
so every fact taken from the third here is either quoted from it or traced to a record in this repository.
No gate read has seen this revision.

What is measured here,
and what is not:
form D's rebuild and peak cells are measurements of this package's own builds in this round,
beside form A's;
the form B cells are the 2026-11-28 arms' own measurements;
form C's rebuild cells and form D2's cells are inferred from form D's;
form Z's cells are the 2026-11-13 page's,
on the old drive;
no arm measured a round's two commands in one empty build folder for form A,
and no arm built in a worktree born by the prewarm;
the days-to-fill figures are a multiplication of a proxy rate.
Every figure is kept in `package/git-policy/cli/doc/development-profile-measurements.md`,
under its heading dated 2026-11-28 and its heading "The recommended form measured on its own,
2026-11-28",
and each heading names the command behind each figure.
The machine's state figures were read on 2026-11-28,
between 07:30 and 08:10 -05:00 for the build trees and the free space,
and at 09:04 and 09:08 -05:00 for the cleanup service,
the unit files and the free space;
each is named where it is used.

The setting this file is about is in `package/git-policy/cli/Cargo.toml`:

```toml
# package/git-policy/cli/Cargo.toml
[profile.dev]
debug = "line-tables-only"
incremental = false
```

## The first screen

What is asked:
five decisions,
each its own question under "Questions for the owner",
in the order they can be answered.
The first is whether the two `[profile.dev]` lines of 2026-11-13 stand,
because on 2026-11-04,
when the owner's words were said,
the manifest had neither line.
The second is whether this repository's six path crates count as the "dependencies" those words name,
bearing in mind that Cargo keeps every registry and git dependency non-incremental under every form.
The third is whether the compiler's own default of 256 code generation units,
which follows incremental compilation when the manifest names no count,
counts as the change those words put off-limits.
The fourth is which form the dev and test profile takes,
from six,
one of which is Cargo's own default.
The fifth is whether the cleanup service the owner chose,
which is installed and switched off on this machine,
is switched on again,
and with what bound.

What is recommended:
form D,
`[profile.dev.package.monochromatic-git-policy-cli] incremental = true` beside `[profile.dev] incremental = false`,
which writes incremental data for this crate's own units only and leaves the six path crates as they are.
The recommendation stands on two of the owner's answers:
that the two lines of 2026-11-13 may stay,
and that the six path crates are not what the words' dependency clause means.
If the words stand as written,
including their debuginfo clause,
the form that changes nothing is Cargo's own default (form Z),
because the setting as it is (form A) already moved the debuginfo level on 2026-11-13.

What D delivers,
measured in this round:
a rebuild costs 0.44 to 0.64 of today's wall and 0.25 to 0.49 of its processor time,
on both commands and both edits,
and the compiler's peak on a rebuild falls to 0.66 to 0.77,
the scope's peak to 0.75 to 0.88 and anonymous memory to 0.63 to 0.72.

What D costs:
2,325 MiB of this crate's own incremental data in a round's one tree;
a first library test build at 1.24 of the controls' wall
(164.9 s against 136.1 and 129.7 s)
and 1.07 of their processor time,
both measured in an empty build folder;
no wall difference on the first library build,
whose controls sit 24.5% apart,
so that cell shows nothing either way;
and a first build's peak that does not move in either direction.

Which figures are measurements and which are not:
form D's cells in "What a rebuild costs" and "Peak memory" were measured in this round,
beside two form A controls in the same round;
form B's cells are the 2026-11-28 arms' own measurements,
marked "form B" wherever they are used;
form C's rebuild cells are inferred from form D's,
as are all of form D2's,
and each place says so;
form Z has no cell measured on this drive at all.
No form's first build here was measured in a worktree born by the prewarm,
and no arm measured form A's two commands in one empty build folder.

What none of the six forms changes:
a first build's peak memory.
Form D reads 0.94 to 0.96 of the controls and form B read 0.94 to 0.96,
both inside the 20% these builds can show.
The failure on 2026-11-27 was twenty-three first builds running at once,
and no form here lowers that phase.
What D and B add there is wall time:
24% and 22% more on the library test build in an empty build folder,
so a wave of first builds holds the machine longer.

Where a first build is measured,
and where it is not:
every first-build cell in this file is of an empty build folder.
A worktree born by the prewarm does not start empty,
because the commit service clones the source checkout's build directory into it
and cleans only the worktree's own crates out of the copy,
so what such a worktree compiles is this crate and the six repository path crates outside its workspace.
That population was not measured:
no arm built in a worktree born by the prewarm,
and no arm ran form A's two commands in one build folder.
What would settle it is one `-vv` pair of a round's two commands in a worktree just made by the wrapper's add,
beside the same pair in a worktree under form A.

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

Five decisions,
separate,
each with its own question under "Questions for the owner":

1. Do the two `[profile.dev]` lines of 2026-11-13 stand,
   does the profile return to Cargo's own default,
   or are the owner's words of 2026-11-04 lifted for this crate?
   The words name the debuginfo level,
   incremental compilation off for the dependencies,
   and `codegen-units` off-limits,
   and they were written for the planning pass that produced `doc/planning/lean-build-dirs.md`,
   against a manifest that then held neither a `debug` nor an `incremental` line.
2. Do this repository's six path crates count as "dependencies" under the clause about them?
   Cargo marks every registry and git dependency non-incremental under every form,
   so the clause has no other subject left.
3. Does the compiler's default of 256 code generation units count as the change the words put off-limits?
   It follows incremental compilation whenever the manifest names no count,
   which this manifest does not.
4. Which form do the dev and test profiles take,
   from the six below?
5. Is the cleanup service the owner chose on 2026-11-11 switched on again on this machine,
   and with what bound?

Five things the records already answer are told here and not asked:
dependency builds are shared by the prewarm the owner chose on 2026-11-03;
how many builds may run at once is bounded by guests alive and by the memory share the owner set on 2026-11-28;
`debug = false` was rejected and is not reopened;
compiling less of the crate per round is the project's already-decided route to a smaller peak;
and which tool keeps the build folders lean,
which the owner chose on 2026-11-07 and 2026-11-11.
One further thing is a recorded default with the owner's veto open rather than a question,
because two of its three values restate forms the fourth question already offers:
a round passes no `CARGO_INCREMENTAL` of its own,
and a round that wants form A's build for one run types `0`.
Its section says what that leaves open and what the record is.

## What changed since the 2026-11-13 choice

Those two lines were chosen on 2026-11-13,
when a build tree was what limited how many agents worked on this package at once,
and the round that measured them wrote them into the manifest with this comment:

```toml
# package/git-policy/cli/Cargo.toml
# A build tree is what limits how many agents work on this package at once, so development and test builds
# trade debug detail and incremental data for disk.
```

Its first sentence is the premise that every answer except A's falsifies:
a yes that sets a profile line,
in either direction,
leaves "a build tree is what limits how many agents work on this package at once"
standing for a machine whose home is on a 3.5 TB drive.
That comment is in a source file this round does not change,
so whichever answer the owner gives,
the comment needs an edit of its own.

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

What the words of 2026-11-04 were said about,
checked with `git show`:
the crate's manifest at `a5a804df00`,
the last commit that touched it on that day,
held `[profile.release]`,
`[profile.release.build-override]` and `[profile.dev.build-override]`
and no `debug` and no `incremental` line under `[profile.dev]` at all.
Both lines were added on 2026-11-13 by `bf9c397cf2` (2026-11-13T16:43:49-05:00),
which is why the words were said about Cargo's own default profile,
carried here as form Z,
and why the two lines moved two of the four settings the words name:
the debuginfo level and incremental compilation.
The commit that recorded the words,
`37caad8dbb` (2026-11-04T16:52:44-05:00),
says of it "(debuginfo and incremental untouched)".

## What incremental compilation is

Cargo's reference says it plainly:
"Incremental compilation causes rustc to save additional information to disk which will be reused when recompiling the
crate,
improving re-compile times".
That information is a query cache and a dependency graph per crate per profile,
written under `<build dir>/debug/incremental/<crate>-<hash>/s-<session>/`,
with one `s-<session>` directory per compiler run.
The compiler garbage-collects them when it starts in incremental mode
and again when it finalises a session
(`compiler/rustc_incremental/src/persist/load.rs` and `.../persist/fs.rs`,
rust-lang/rust,
read 2026-11-28):
it deletes every session directory that has no lock file,
and of those that have one it deletes each finalized directory whose lock it can take,
skipping a directory it cannot lock (the source's debug line for that branch reads "not collecting,
still in use") and leaving the previous session alone while it is being read.
So one `<crate>-<hash>` directory holds one session's data,
not one per rebuild.
The `<hash>` is what makes that a bound per compiler and not per tree:
`StableCrateId::new` hashes the rustc version string
(`compiler/rustc_span/src/def_id.rs`,
read 2026-11-28),
so a toolchain update opens a new directory beside the old one and Cargo never removes the old ones,
as the lean tool's own header says
(`package/cargo-shadow/cli/src/lean_keyed_folder.rs`,
lines 4 to 8:
"A nightly update dirties every unit of a build folder in place and Cargo never removes the old ones").
This machine keeps dated nightlies and runs `toolchain-daily-update.service`,
so a long-lived tree can hold one session's data for every compiler it has seen.
What removes them is `cargo clean --package <the crate>`,
which removes that crate's own build outputs and leaves every other package's alone:
its fingerprint entries,
its `build/<package>/<hash>` directories,
its compiled units and its `incremental/<crate>-*` directories
(`src/ops/cargo_clean.rs`,
read 2026-11-28),
or deleting the tree.

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
The arms' own `-vv` logs say the same from the other side:
no `-C codegen-units=` appears in any rustc line of the four logs,
so Cargo passed none in any form.
What is not read is rustc's own default in a real invocation;
a count of the `.o` files in one form D session directory would show it.
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

The environment names an arm recorded are `CARGO_BUILD_BUILD_DIR`,
`CARGO_BUILD_JOBS`,
`CARGO_INCREMENTAL`,
`CARGO_PROFILE_DEV_INCREMENTAL`,
`CARGO_PROFILE_TEST_INCREMENTAL`,
`CARGO_TARGET_DIR` and `RUSTFLAGS`.
Four names that change what a round builds are not among them,
and no record here says whether a round's own environment sets them:
`CI` and `TF_BUILD`,
either of which makes Cargo override every manifest's `incremental` line to off,
and `RUSTUP_TOOLCHAIN`,
which decides whether the wrapper keys a build folder by toolchain rather than by workspace.
The Cargo versions in play are three:
`cargo 1.102.0-nightly (ede6cefea 2026-11-11)` in the earlier round's unit-graph probe,
`nightly-2026-11-16` in every arm of this round,
and the `rust-lang/cargo` and `rust-lang/rust` sources as read on 2026-11-28.
Every claim here about Cargo's own behaviour rests on the third of those,
not on a run,
except where a cell says an arm measured it.

Two faithful builds of one unchanged crate differed by up to 20% on this host
(`package/git-policy/cli/doc/development-profile-measurements.md`,
the 2026-11-25 section,
"The control's spread"),
and the page's own rule reads "A difference smaller than 1.20 is no difference these builds can show."
So every figure here carries the treatment's ratio to the mean of its two controls,
the range that dividing by each control alone gives,
and the spread between the two controls;
a figure inside its controls' spread is reported as no difference the measurement can show.
Each treatment is one run,
so no figure carries the treatment's own run-to-run spread:
only the two controls' spread is measured,
and the one place where two runs of one arm disagreed is recorded in the measurement page.

## The arms, and what they measured

Twenty-four arms ran on 2026-11-28.

Six first builds:
form A's library and library test build twice each,
and form D's once each,
in this order so that the treatment sits between its controls.

Twelve rebuilds after one line added at the end of one file:
the line is `pub fn a_rebuild_measured_after_one_edit() {}`,
an item appended to the file,
with no existing body or signature changed and no test file touched,
in the leaf file,
`src/native/candidate/candidate_store_commit_listings.rs`,
which no other Rust file names,
and in the widely used file,
`src/native/worktree_clone_mount_protocol/mod.rs`,
which 273 Rust files name,
the most of any file in the crate.
Each pair is applied in one direction and removed in the other,
so the two controls of a cell are the same work in opposite directions.
That is the cheapest kind of edit for incremental compilation,
and the gain for any other kind is unmeasured:
an edit that changes a signature other files use,
or one that changes a test body,
is in no cell here,
although 688,854 of the crate's 1,311,645 lines sit in `*_tests.rs` files.

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
The two first-build cells are of an empty build folder,
which is not what a worktree born by the prewarm starts with;
"The first screen" says what that leaves unmeasured.

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

The arithmetic below is the library test build's,
and it starts from a first build measured in an empty build folder.
In a worktree born by the prewarm the dependency units are already there,
so both forms lose that phase,
and what stays is this crate's own unit and the six path crates outside its workspace.
The arms' own timelines put that phase at 63.4 and 57.4 s under form A
(this crate's unit starting at 72.7 s of 136.1 s and at 72.3 s of 129.7 s)
and at 90.5 s under form D (starting at 74.4 s of 164.9 s),
so on the crate's own unit D is about 1.5 times A rather than 1.24 times,
which would make the premium a larger share of a smaller first build.
That is an inference from the arms' own-unit start and end times,
not a measurement of a prewarmed worktree,
and no arm ran form A's two commands in one folder,
so nothing here says what a round's pair costs.

Against today's empty-folder build,
form D's first build on the library test build costs 32.0 s more wall and 36.0 s more processor time,
and both edits give that back at about one rebuild.

At k rebuilds after its first build,
the wall is these fractions of today:

- leaf edit:
  1.0 at k=1,
  0.9 at k=2,
  0.8 at k=3,
  0.7 at k=5,
  0.6 at k=10;
- wide edit:
  1.0,
  0.9,
  0.8,
  0.7 and 0.6 at the same k;
- the processor time at the same k is 0.9,
  0.8,
  0.8,
  0.7 and 0.5 (leaf),
  and 0.9,
  0.8,
  0.7,
  0.6 and 0.5 (wide).

The rows carry one decimal because the two edits' wall cells differ by 0.01
while the wide cell's controls sit 18.3% apart,
so a second decimal would say more than the runs can.

The first build's extra wall is paid back after 1.04 rebuilds on the leaf edit and 0.99 on the wide edit;
its extra processor time after 0.45 and 0.42.

A round that makes no rebuild after its first build pays 1.24 in wall and 1.07 in processor time and earns nothing back,
in an empty build folder;
in a prewarmed one the pair was not measured at all.
How many rebuilds a round makes is not recorded anywhere:
no count of `cargo build` or `cargo test` scopes per round exists,
and this round searched `doc/` and the package's own docs for one rather than guessing.
A count of build scopes in the journal,
one per round,
is the read that would settle it.
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
This round's memory sampler cannot name the process,
but this round's own `-vv` rounds narrow it to one unit:
in the form D round the `forbidden-strings` build script's run line is at 38.7 s
and that crate's library compiles at 68.5 s,
with no `rustc` in the scope from 54.1 s and the scope's peak climbing from 2,439 to 3,792 MiB across those samples;
the form C round reads 40.1 s and 71.7 s with the same climb.
So the only unit in flight through the quiet stretch,
by the log,
is that build script's run,
and naming the process from the kernel is what is still not read.

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
  the smallest 506,154,628 bytes (483 MiB) and the largest 3,692,305,893 bytes (3.44 GiB),
  1,637,111,834 bytes (1561 MiB) at the mean.
- what a tree holds is the compiled units and their build scripts,
  not only the scripts.
  Cargo's nightly build-cache reference describes `<build-dir>/debug/build/` as
  "Build step results including compilation artifacts and build script output",
  and the repository's own decision record lists the layout as
  `debug/build/<package>/<meta-hash>/out` and `debug/build/<package>/<meta-hash>/fingerprint`
  (`doc/decision/cargo-build-dir-host-cache.md`,
  "The build-dir layout is internal").
  The arms' own `-vv` logs say the same of a real build:
  in the form D round 327 rustc lines write `--out-dir` under `debug/build/`,
  the registry crate `itoa`'s among them,
  and the test command's `Executable` line is
  `<build-dir>/debug/build/monochromatic-git-policy-cli/<hash>/out/git_policy_cli-<hash>`.
- no tree in the survey was listed,
  so what each holds is the layout above and not a listing of its children.
  What the survey recorded is that 22 of the 23 hold an `incremental` directory and one does not,
  and it recorded no size for any of them:
  the field reading `incremental=true` says the directory exists,
  and nothing more.
  The `deps=-1` in every row is the same kind of absence:
  the survey looked for `debug/deps`,
  which this layout has no use for,
  so it says nothing about compiled units either way.
- none of the 29 reports a `toolchain/<toolchain>/` path.
  The earlier version of this file measured two trees in that layout (2.03 and 3.01 GiB)
  and counted 2 of 34 workspaces as holding a tree;
  that is not the layout the workspace's own cargo names when no `+toolchain` and no `RUSTUP_TOOLCHAIN` are in play,
  and the count that follows from the other layout is 23 of 29.
- free space:
  1,548,279,218,176 bytes (1442 GiB),
  `df --block-size=1 /home` at 2026-11-28T07:36-05:00.

### What the forms add, and what a day of them is

Measured this round in apparent bytes.
The harness's figure is `du --summarize --bytes`,
and GNU `du` documents `--bytes` as "equivalent to '--apparent-size --block-size=1'",
so the byte count and the block count agree because of the flag,
not because of anything this filesystem does.
No allocated size was measured,
so nothing here says what the ZFS pool compresses or what a snapshot keeps.

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

### What removes a tree today, and what the owner chose

Nothing removes a tree by itself on this machine today:

- `cargo-shadow lean` removes a tree once its workspace is gone and the tree has been idle for `TREE_IDLE_SECONDS`,
  seven days (`package/cargo-shadow/cli/src/lean_policy.rs`);
- the unit that runs that pass,
  `cargo-shadow-lean.service`,
  is installed and switched off:
  `systemctl --user is-enabled` answered `disabled`,
  `is-active` answered `inactive`,
  and `list-unit-files` showed state `disabled` with preset `enabled` at 2026-11-28T09:04-05:00;
  `list-timers --all` lists no cargo-shadow timer,
  and the unit file carries `[Install] WantedBy=default.target`,
  so enabling and starting it is one command for whoever keeps the machine;
- the unit's own journal holds no entry at all (`journalctl --user --unit=cargo-shadow-lean.service`),
  and the user journal on this machine holds one boot only (2026-11-28T05:53:37-05:00 to the time of reading),
  so it cannot say what happened on 2026-11-17T12:32-05:00,
  the day the unit file was written;
  no record in `doc/` says why it is off either.
  The last record of anyone switching it on is about the daily timer it replaced
  (`doc/decision/owner-answers-of-2026-11-15-machine-housekeeping-and-tooling.md`,
  "`cargo-shadow-lean.timer` is running again"),
  and the handover's "Timer off until the moved checkouts are checked,
  recorded 2026-11-08T12:43:37-05:00" is about that same older timer and an earlier move.
  So why the service reads disabled is unrecorded,
  which is a finding for whoever keeps this machine and not a question this file can answer.

The owner chose that mechanism,
and the words are in `doc/planning/lean-build-dirs.md`:

- 2026-11-04:
  "The real fix is to make sure every build dir is always lean.
  That's not the wrapper's responsibility,
  but I'm assigning this to you to."
- 2026-11-07,
  in the question tool:
  "A to F,
  B allowed (Recommended)",
  where F is "the service that keeps the cleanups running".
- 2026-11-11:
  "Make the cleanups a systemd service that runs continuously."

The service installed from that last word runs one `cargo-shadow lean run` pass,
kills a pass that outlasts `RuntimeMaxSec=19800`,
starts the next pass 60 s after the last one ended,
and carries `CPUQuota=200%`,
`MemoryMax=4G`,
`TasksMax=512`,
`IOWeight=5`,
`Nice=15` and `IOSchedulingClass=idle`.
Its size caps and graces,
as the owner-question record holds them with the owner's veto open,
are 12 GiB in a checkout and 24 GiB in the shared build folder,
with a 7-day grace for a tree and 21 days for one the tool cannot identify.

What a first pass would remove on this machine is not counted,
because a walk of the base is closed to agent runs.
The records hold two earlier passes of the same tool over other populations:
a dry run of 2026-11-08 read 6,923 trees,
would remove 18 (26.31 GiB) and kept 6,905,
and the pass that ended 2026-11-10T07:37:24-05:00 judged 14,850 of 33,804 trees and removed 278 for 511.49 GiB,
which is 0.71 s per tree including the removals.
Neither is today's base,
and today's base holds at least the 23 trees of 35.1 GiB this round read,
beside whatever the older layout still holds.

What a pass costs a build running beside it is not measured either.
The unit's own caps bound it (two cores of CPU,
4 GiB of memory,
idle I/O class),
a pass that finds nothing costs about a minute on this machine by the plan's own measurement of 2026-11-11,
and a 56-minute pass of this service is one of the load suspects the 2026-11-14 record names,
with no cause established there.
No record holds a build measured beside a running pass.

What a person does,
whichever form the owner picks and whether or not the service runs:

- the data going stale:
  `cargo clean --package <the crate>` removes that crate's own outputs,
  its incremental directories among them,
  `cargo clean` in the package clears that workspace's whole build directory,
  and deleting `<base>/<2 hex>/<14 hex>/` for a workspace no longer built removes a tree;
- the disk filling:
  the trees of removed worktrees go first,
  then idle trees of live ones;
  a pass bounded to trees is `cargo-shadow lean run --only trees --deadline-minutes <n>`.

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
  the compiler keeps one finalized session in each `<crate>-<hash>` directory,
  and what grew there was one session's own size,
  across the crate's several profiles,
  its path dependencies and the compilers it had seen.
- the 33,804 trees and the 10,475 s pass above were measured on 2026-11-10,
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
the build directory the source checkout has,
which holds the compiled units of every registry and git dependency as well as their build scripts.
That is the layout the section above reads out of Cargo's reference and the repository's decision record,
`debug/build/<package>/<meta-hash>/out` and `.../fingerprint`,
and the same `-vv` logs that put the test binary's `Executable` line there put a registry crate's `--out-dir` there too.
The job's clean removes the worktree's own members' outputs and nothing of any other package's
(`src/ops/cargo_clean.rs` removes the named packages' `.fingerprint` entries and `incremental/<crate>-*`),
so what the clone leaves is every other package's units.
Cargo reuses those because a registry or git package's unit hash carries no path,
which is what the owner's words above say,
while a path package's hash does:
`SourceId::stable_hash` removes the workspace prefix and hashes a path outside the workspace absolutely,
unless `trim-paths` is set
(`src/workspace/source_id.rs`,
read 2026-11-28).

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
this crate's own units,
which the job cleans out of the copy,
and the units of the six repository path crates outside this crate's workspace
(`forbidden-regex`,
`monochromatic-jsonc-edit`,
`monochromatic-zfs-request-to-openzfs-kernel-device`,
`monochromatic-catalog-specifier`,
`monochromatic-lint` and `forbidden-strings`),
whose absolute paths are hashed into their own unit hashes,
so a worktree at another path finds them stale and compiles them again.
So a prewarmed worktree compiles seven units:
the six path crates and this crate,
which is the set the second command of the form C `-vv` round compiled,
and the `forbidden-strings` build script is compiled and run again with them rather than found fresh.
A fresh worktree therefore does not compile 266 to 271 dependency units,
and the earlier version of this file was wrong when it said no mechanism here avoids the dependency phase.
What is not measured is what that costs in wall and memory:
no arm built in a worktree born by the prewarm,
and the read that would settle it is one `-vv` pair of a round's two commands there,
counting `Fresh` and `Compiling` lines and looking for the build script's own run line.

What remains an engineering question inside the prewarm is not the units but those six path crates:
their units are carried and then unusable in the new worktree,
so by the hashing rule above every clone carries a set of units no build there reads,
and the lean sweep retires them with the tree.
Changing that would make the job a reader of the build directory's layout,
which `doc/decision/cargo-build-dir-host-cache.md` allows no reader of except the lean sweep;
that is a design of this package's own and not a profile setting,
and it is not put to the owner.

What the earlier version called option E,
"a shared build directory or a compiler cache",
is therefore answered:
the owner chose the mechanism,
it runs,
and what remains is one engineering question inside it,
about the path crates' unusable units,
which a later round can take up without a decision here.

## The options

Each option says what its lines are and where they go,
what it changes for good,
what lands on disk and what removes it,
the loop and the first build in the arms' own numbers,
and what a yes would still leave undecided.

The six forms,
one line each:

- form A,
  the setting as it is:
  `debug = "line-tables-only"` and `incremental = false`,
  so no incremental data is written anywhere and no compiler default moves;
- form Z,
  Cargo's own default:
  no `[profile.dev]` line at all,
  which is the state the 2026-11-04 words were said about;
- form B,
  incremental compilation for everything:
  `incremental = true` under `[profile.dev]`,
  so this crate and the six path crates hold data and their code generation units default to 256;
- form C,
  incremental compilation in the test profile alone:
  `[profile.test] incremental = true` beside `[profile.dev] incremental = false`;
- form D,
  incremental compilation for this crate's own units:
  `[profile.dev.package.monochromatic-git-policy-cli] incremental = true` beside the two lines,
  which is the recommended one;
- form D2,
  incremental compilation for this crate's test build alone:
  the same package override under `[profile.test]`.

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
which are the compiled units and their build scripts;
no incremental data at all,
in any directory.
A tree per worktree stays until the sweep (Question 5) or a person removes it.

What it gives:
the smallest first build of the six forms,
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

### Z. Cargo's own default, which is the state the words were said about

`package/git-policy/cli/Cargo.toml`,
with no `[profile.dev]` table at all:

```toml
# package/git-policy/cli/Cargo.toml
# no [profile.dev] table: Cargo's defaults apply
```

What it does:
this crate and the six path crates are compiled with full debug information and with incremental data,
because a path source is local as far as Cargo is concerned,
while every registry and git dependency stays non-incremental,
as it does under every form;
the compiler's code generation units are its own default,
16 without incremental data and 256 with it,
and the debuginfo level is the default of a full build.

What lands where:
`debug/incremental/<crate>-<hash>/s-<session>/` for this crate and the six path crates,
full debug information in every executable,
and a tree per worktree that the sweep or a person removes.
The only figures are the 2026-11-13 page's,
on the old drive,
after the build and the two test compiles an agent of this package runs:
a tree of 5885 MiB of which 2813 MiB is incremental data,
a build of 124.8 s wall and 512.5 s CPU,
and the library test compile at 105.1 s wall and 144.0 s CPU.
Nothing on this drive was measured for it.

What it gives:
the richest backtraces of the six,
because every executable carries full debug information,
and the incremental loop for this crate and the path crates,
which no arm in this round measured.
It also leaves every setting as it was on 2026-11-04,
which is the state the owner's words were said about;
every other form,
A included,
has moved one of the settings those words name.

What it costs:
4323 MiB (4.2 GiB) more per tree than the same page's 1562 MiB for A over the same three commands,
and the same page's 124.8 s against A's 83.2 s for the build,
which is 1.5 times,
on the old drive.
The 2026-11-13 page's own conclusion was that the tree size was what limited how many agents worked here,
and the home has moved to a 3.5 TB drive since.

What a yes leaves undecided:
whether the 2026-11-13 page's premise still holds on the new drive,
since nothing here measured that,
and everything the arms measured for the loops of the forms that set a line,
which for Z is nothing at all.

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
The tree and its nine directories stay until the sweep (Question 5) or a person removes them,
and `cargo clean --package <the crate>` removes that crate's own outputs and no other package's.

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
and the owner's 2026-11-04 words on the dependencies,
which is the six repository path crates and no registry or git dependency,
since Cargo never gives those incremental data under any form.

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
debug = "line-tables-only"
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
the test command compiled seven units,
the six path crates and this crate,
wrote nine incremental directories,
and compiled and ran the `forbidden-strings` build script again,
against form D's one compiled unit,
two directories and no build-script line at all.
`monochromatic-catalog-specifier` is one of the seven compiled units and has no build script;
its line in the log is a rustc compile,
not a run.
The build-script *run* unit is not doubled by the graph,
because Cargo strips its profile to fields that carry no `incremental` (`get_profile_run_custom_build`),
and the run repeats anyway when the compiled script's unit changes,
which is what the round shows.

What lands where:
the test command's data:
1836 MiB in a round's one tree,
in nine directories,
and none for the library build.
The tree and its nine directories stay until the sweep (Question 5) or a person removes them,
and `cargo clean --package <the crate>` removes that crate's own outputs and no other package's.

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
which is 263.4 s of processor time in the test command against D's 140.7 s in the same `-vv` round (1.87),
and 268.4 s against 133.2 s in the `-v` round (2.02);
the same round's wall is 94.7 s against 75.4 s (1.26) and 103.2 s against 74.2 s (1.39);
the `forbidden-strings` build script compiled and run again per round;
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
debug = "line-tables-only"
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
The tree and its directories stay until the sweep (Question 5) or a person removes them,
and `cargo clean --package monochromatic-git-policy-cli` removes this crate's own outputs.

What it gives:
the loops measured on this form in this round,
on both edits:
0.64 and 0.50 of the controls' wall on the library build,
0.44 and 0.44 on the library test build,
0.49 and 0.36 of the processor time on the library build,
0.25 and 0.25 on the library test build,
and on a rebuild 0.66 to 0.77 of the compiler peak,
0.75 to 0.88 of the scope peak,
0.63 to 0.72 of anonymous memory,
with backtraces that keep file and line in every frame,
as line tables do under A.

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

Two named caveats.
The first is what a new worktree pays:
the prewarm clones the source checkout's build directory and then removes this crate's members,
so a worktree born by the prewarm always starts with no incremental data for this crate
and its first build is a form D first build,
while the source checkout's data for the path crates is copied and then never read there.
The second is a compiler update:
it opens a new `<crate>-<hash>` incremental directory beside the old one and starts a fresh session,
so the data a tree carries is the last session of each compiler it has seen,
and `cargo clean --package <the crate>` is what removes one compiler's set.

The profile table is also keyed by the package name,
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
debug = "line-tables-only"
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
which is the smallest of the forms that write any data:
1674 MiB against D's 2325 MiB,
C's 1836 MiB and B's 1824 MiB,
and against Z's 2813 MiB on the 2026-11-13 page.
The tree and its data stay until the sweep (Question 5) or a person removes them.

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

### The round's own variable, recorded as a default with the veto open

A round's task can pass `CARGO_INCREMENTAL`,
which Cargo applies over any manifest for every path source:
the override is applied before the check that leaves non-path sources non-incremental
(`src/workspace/profiles.rs`,
fetched 2026-11-28).
So `1` is form B whatever the manifest holds,
data for the six path crates included,
and `0` is form A.
Only the literal `1` turns it on;
any other value,
`true` included,
means off.
Nothing in the manifest affects either,
and a round that forgets the variable gets whatever the manifest holds.
Cargo's reference names two more routes to the same flag,
the `build.incremental` config key and the per-profile `CARGO_PROFILE_<name>_INCREMENTAL` variables,
and this repository already sets `CARGO_INCREMENTAL=0` in several images and tasks
(`package/linter/monochromatic-lint/test.Containerfile`,
`package/git-policy/cli/bin/e2e-native-container.ts`,
`package/git-policy/cli.fuzz/bin/fuzz-build.ts`),
so a round's environment is the third place the value can come from.

A fourth route overrides everything the manifest says,
and the earlier versions of this file did not name it:
with neither `CARGO_INCREMENTAL` nor `build.incremental` set,
Cargo turns the global override to off when `CI` or `TF_BUILD` exists in its environment
(`is_ci` in `crates/cargo-util/src/lib.rs` and the override in `src/workspace/profiles.rs`,
both read 2026-11-28).
A global override beats every `incremental` line of every manifest,
so under form D a round whose environment carries `CI` builds as form A without a word,
and the warning about an unmatched package spec does not fire either.
No record here says whether any round's environment sets `CI` or `TF_BUILD`:
this round's arms recorded neither name and wrote incremental data,
which says theirs did not.

Recorded default,
with the owner's veto open:
no round passes the variable,
and a round that wants form A's build for one run types `0`.
It is not one of the questions because two of its three values restate forms the form question offers,
and because `0` stays reachable without a decision.
What that leaves open:
what a round that needs the escape does when it is not allowed one,
and who notices a round's environment carrying the variable,
or `CI`,
or `TF_BUILD`,
from elsewhere.

## Combinations that cannot be built

- Every form but Z while the 2026-11-04 words bind as written:
  A moved the debuginfo level away from the default those words were said about,
  and B,
  C,
  D and D2 move `codegen-units` for the units they touch from 16 to 256 by the compiler's own default,
  which the words put off-limits.
- Z beside the words' dependency clause,
  read as covering this repository's six path crates:
  Z is the state the words were said about,
  and it writes incremental data for those six,
  because Cargo treats a path package as local.
  Read that way no form meets both clauses,
  which is what the second question asks the owner to settle.
- B or C beside that clause:
  both write those six crates' data,
  and C's test command splits their units from the library command's.
- D or D2 with a round passing `CARGO_INCREMENTAL=1`:
  the variable wins,
  so that round is B in effect,
  and D2's test-only scoping turns into B for both commands.
- C with a round passing `CARGO_INCREMENTAL=1`:
  the same,
  so C becomes B.
- Any form with a round passing `CARGO_INCREMENTAL=0`,
  or with `CI` or `TF_BUILD` in its environment:
  that round builds as A,
  whatever the manifest holds.

One combination can be built and is unmeasured here:
pinning `codegen-units = 16` beside an incremental form.
It is the answer to the third question if the owner reads the compiler's default as a change the words forbid,
it would take back part of what the data buys,
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
Every pair below rests on cells that exist,
says which reading of a disagreeing cell it takes,
and names what the higher form gives up.

The order depends on the first two questions,
so here it is under each answer to them.

### If the words stand as written and bind this crate now

Ranking:
Z alone,
and only if the dependency clause does not cover the six path crates;
with that clause covering them,
no form meets both and the second question has to be answered first.
This is not a choice between two loops but a revert:
A is the deviation the words did not authorise,
and the 2026-11-13 page's own figures price the revert at 5885 MiB a tree against A's 1562 MiB
and 124.8 s a build against 83.2 s,
both on the old drive and over the same three commands.
A yes here buys the words back and gives up the disk and the build time A was chosen for.

### If the words bind the dependencies only and this crate's own units may move

Ranking:
D > D2 > A.
Z drops out because it writes the six path crates' data,
and B and C drop out for the same reason.

- D over D2:
  D delivers both commands' loops,
  0.64 and 0.50 of the controls' wall on the library build's rebuild and 0.49 and 0.36 of its processor time,
  which D2 leaves at 20.6 to 23.8 s and 35.3 to 39.5 s for good.
  D pays 651 MiB of this crate's own library data for that.
- D2 over A:
  the same test loop D delivers (0.44 of wall,
  0.25 of processor time,
  0.66 to 0.75 of peak),
  1674 MiB of data a test tree,
  where A gives the smallest first build and no data at all and pays 52.9 to 62.6 s of wall a test rebuild.

### If the words were about that planning pass only and do not bind this crate now

Ranking:
D > B > Z > D2 > C > A.

- D over B:
  on the leaf edit the two forms' measured cells agree
  (B's test rebuild 0.50 of wall and 0.26 of processor time in the 2026-11-28 arms,
  D's 0.44 and 0.25 here);
  on the wide edit the two rounds disagree and both readings stand:
  B's test wall read 1.00 against the control that ran before it and 0.35 against the one after it,
  which the measurement page explains by the second control having run while the machine slowed every build by about
  2.7 times,
  so only the first pairing is worth reading,
  and D's read 0.44 against two controls 18.3% apart.
  So B's wide-edit wall is not established and D's is.
  D also writes 1674 MiB on a first test tree against B's 1824 MiB,
  keeps the six path crates' 161 MiB out,
  and keeps their units shared between the two commands.
  What D gives up is B's second round of arms,
  which nobody has run for D's own wide-edit cell.
- B over Z:
  B gives the same incremental data with line tables only,
  where Z carries full debug information in every executable
  (the 2026-11-13 page's 5885 MiB tree,
  2813 MiB of it incremental),
  and B's cells are measured on this drive while Z's are the old drive's.
  What B gives up is Z's full debug information in a backtrace.
- Z over D2:
  Z gives both commands' loop where D2 gives the test build's alone,
  at the price of the debug information and the six path crates' data.
  That pair rests on what Z delivers and on no cell measured on this drive.
- D2 over C:
  the same library test loop,
  with no path crates' data and one set of their units per tree,
  where C writes 161 MiB of that data,
  splits their units between the two commands
  (seven units compiled in the test command in the `-vv` round,
  263.4 s of processor time against D's 140.7 s in the same round,
  1.87;
  268.4 s against 133.2 s in the `-v` round,
  2.02),
  and compiles and runs the `forbidden-strings` build script a second time each round.
- C over A:
  C gives the library test build's loop for 1836 MiB of data a tree,
  where A gives the smallest first build and no data at all.
  A is the answer where a round makes no rebuild after its first build,
  because there the first build's premium is paid and nothing earns it back;
  on the test build's wall arithmetic the two are level at one rebuild and C leads from two.

Three things the ranking says once,
for all three orders:

- no form lowers a first build's peak,
  which is what failed on 2026-11-27;
  D and B make a first build 24% and 22% longer in wall,
  measured in an empty build folder,
  so a wave of them holds the machine longer,
  and only A leaves that phase where it is;
  Z's first build is longer again for its full debug information,
  by the 2026-11-13 page's 124.8 s against 83.2 s.
- every first-build figure here is of an empty build folder,
  and a worktree born by the prewarm starts with the dependency units in place,
  which was not measured;
  the cost side of each pair above is therefore a cost for a population no round meets.
- what the ranking does not settle is how many rebuilds a round makes,
  which no record holds,
  and the bound on concurrent builds,
  which the records already answer in the section that lists what is not asked.

## Questions for the owner

Each question is one decision,
understandable without the rest of this file,
and each says what a yes leaves undecided.
They come in the order they can be answered,
and the first three are the three decisions an earlier version of this file asked as one.

### Question 1. Do the two profile lines of 2026-11-13 stand?

The two lines are in `package/git-policy/cli/Cargo.toml`:

```toml
# package/git-policy/cli/Cargo.toml
[profile.dev]
debug = "line-tables-only"
incremental = false
```

`debug = "line-tables-only"` cuts debug information down to file and line;
`incremental = false` stops the compiler writing reusable data between builds.
On 2026-11-04 the manifest held neither line,
so both were added on 2026-11-13 by `bf9c397cf2`,
by a round that made them to keep build trees small.

The owner's words of 2026-11-04,
and the page's own conclusion from them,
are recorded in `doc/planning/lean-build-dirs.md`,
"Owner requirements that bound this":

> "(debuginfo level,
> incremental off for dependencies - no messing with these."

> "other profile settings (split-debuginfo,
> strip,
> codegen-units) should be off-limits too"

> So no build profile setting is changed or proposed here,

One of the two lines moved the debuginfo level those words name.
Their first clause names incremental compilation as a subject.
The second quoted passage puts `codegen-units` off-limits,
which is the third question.
Both quotations are carried as standing rules for the wrapper's work by two later records,
`doc/planning/git-policy-cli-full-tokio.md`
(under "Standing rules that bind every step",
last changed 2026-11-25T22:06:53-05:00)
and `doc/research/wrapper-git-free-jobs-and-published-crates.md`
(under "The owner's words that bind this research"),
so the reading that the words bounded that planning pass alone
is weaker than the records this file cited before made it look.

Three answers,
and they are exclusive:

- the two lines stand:
  the debuginfo line is this crate's own choice of detail for its backtraces,
  and the words' debuginfo clause is read as about that planning pass,
  so nothing changes from today;
- the words bind and the settings return to what they were:
  that is form Z,
  no `[profile.dev]` line at all,
  which carries full debug information and incremental data for this crate and the six path crates,
  and the 2026-11-13 page prices it at 5885 MiB a tree and 124.8 s a build against A's 1562 MiB and 83.2 s;
- the words are lifted for this crate from now on,
  recorded as lifted in a record of its own,
  which leaves questions 2 and 3 open rather than answering them.

The recommendation is the first,
because the second pays 4323 MiB a tree and 41.6 s a build for what the 2026-11-13 round bought with the two lines,
on figures measured before the home moved,
and nothing measured since shows the tree size straining the new drive.

What the records also hold,
and it is not an answer:
the old handover at `342d6c5a59` records the 2026-11-13 change as merged and "not released by itself"
under "Adopted while the owner was away",
with "No record settles this one;
it is the coordinating session's judgement",
and no record of `doc/decision/` or `doc/handover/` holds the owner's acceptance of the two lines or a rejection of
`debug = false`.

What a yes to each answer leaves undecided:

- the two lines stand:
  whether the six path crates are "dependencies" (question 2),
  and whether the compiler's own 256 code generation units count as the change the words forbid (question 3);
- the settings return:
  nothing about the form,
  since Z is then the only form that leaves the settings as they were,
  and whether the tree size is acceptable on the new drive,
  which nobody has measured for Z;
- the words are lifted:
  the same two questions as the first answer,
  and what the words still bind elsewhere.

### Question 2. Do the six repository path crates count as the "dependencies" the words name?

Cargo keeps every registry and git dependency non-incremental under every form and even under `CARGO_INCREMENTAL=1`:
the profile code turns incremental off for any unit whose source is not a path source
(`src/workspace/profiles.rs`,
read 2026-11-28),
and Cargo's reference says the same.
So the only dependencies that clause can bite on are path dependencies,
and six of those sit outside this crate's own workspace:
`forbidden-regex`,
`monochromatic-jsonc-edit`,
`monochromatic-zfs-request-to-openzfs-kernel-device`,
`monochromatic-catalog-specifier`,
`monochromatic-lint` and `forbidden-strings`.

Two answers:

- they count:
  then B and C are out,
  because they write those six crates' data,
  and so is Z,
  because the state the words were said about already wrote data for them,
  which leaves no form meeting both clauses and makes this question decide which clause yields;
- they do not count:
  the clause is about third-party crates,
  which Cargo already keeps cold,
  and it constrains nothing here.

This file reads the clause as the second,
because it is what Cargo already does,
because the alternative leaves no form standing,
and because the word was written against a manifest in which those six already held incremental data.
The owner's meaning is his to settle.

What a yes leaves undecided:

- they count:
  that the words' debuginfo clause has to yield instead,
  and which form loses less when it does;
- they do not count:
  what the clause was about if not these six,
  and whether it binds anything at all in this repository.

### Question 3. Does the compiler's own 256 code generation units count as the change the words forbid?

The words put `codegen-units` off-limits,
and this crate's manifest names no count.
With incremental compilation on and no count in the manifest,
Cargo passes no `-C codegen-units` and the compiler takes its own default,
16 without incremental data and 256 with it
(Cargo's profiles page and rustc's codegen options page,
fetched 2026-11-28).
The arms' `-vv` logs show no `-C codegen-units=` in any rustc line,
so that is what happens in a real build here,
and the move from 16 to 256 is what every incremental form does to the units it touches.

Three answers,
and they are exclusive:

- it is a change the words reach,
  so the count is pinned:
  `codegen-units = 16` beside the incremental form keeps the setting and part of the data,
  and nothing measured here says how much of the gain survives,
  because no arm measured a pinned form;
- it is not a change the words reach,
  because they name a manifest setting and this is the compiler's own default,
  which is what the incremental forms measured here do;
- it is a change and no form may make it:
  then A or Z only,
  and the form question loses every measured option.

This file reads it as the second,
because the words name manifest settings and the manifest names no count,
and because the first answer costs a measured form for an unmeasured change.
The owner's meaning is his to settle.

What a yes to each answer leaves undecided:

- pinned at 16:
  what the loop costs then,
  which nobody has measured,
  and whether the pin is wanted on every incremental form or only on D;
- not a change:
  whether a future manifest line that did set a count would need its own answer;
- no form may make it:
  everything the arms measured,
  and what remains for the memory pressure of 2026-11-27 besides the crate split.

### Question 4. Which form do the dev and test profiles take?

Six forms,
each described by what it sets and does in "The options" above,
where each also carries its disk,
its loop,
its cost and what a yes leaves undecided:

- A,
  the setting as it is:
  line-tables-only debug information and no incremental data anywhere,
  unchanged;
- Z,
  Cargo's default:
  no `[profile.dev]` line,
  so full debug information and incremental data for this crate and the six path crates;
- B,
  incremental compilation for everything:
  `incremental = true` under `[profile.dev]`;
- C,
  incremental compilation in the test profile alone:
  `[profile.test] incremental = true` beside the two lines;
- D,
  incremental compilation for this crate's own units under both commands,
  which is the recommended one;
- D2,
  incremental compilation for this crate's test build alone.

The three facts the decision turns on,
with what each rests on:

- a rebuild under D costs 0.44 to 0.64 of today's wall and 0.25 to 0.49 of its processor time,
  and 0.66 to 0.77 of the compiler's peak,
  on both edits and both commands (measured this round);
- a first build under D costs 1.24 of today's wall on the library test build in an empty build folder,
  and moves the peak not at all (measured this round),
  while a worktree born by the prewarm would pay the same premium on a smaller first build (inferred,
  not measured);
- a round's tree under D grows by 2325 MiB,
  and 24 to 166 such trees a day would spend the 1442 GiB free in 3.8 to 26 days
  (a multiplication of a proxy rate,
  not a count of trees).

The ranking under each answer to questions 1 to 3 is in "Ranking",
and the recommendation is D.

A yes to a form leaves undecided:
is the cleanup service switched on (question 5),
what the units this file never measured cost (`--all-targets`,
`--bin`,
clippy,
`cargo check`),
whether `codegen-units` may move as it does (question 3),
and,
for D and D2,
whether the six path crates fall outside the words' dependency clause (question 2).

### Question 5. Is the cleanup service the owner chose switched on again on this machine?

The record:
the owner chose "A to F,
B allowed (Recommended)" on 2026-11-07,
where F is "the service that keeps the cleanups running",
and on 2026-11-11 wrote "Make the cleanups a systemd service that runs continuously."
The service that word produced is `cargo-shadow-lean.service`:
one unit,
a `cargo-shadow lean run` pass starting 60 s after the last one ended,
`RuntimeMaxSec=19800` for one pass,
and its own caps of two cores,
4 GiB of memory and idle I/O.

What is true now,
read on 2026-11-28 at 09:04 -05:00:
the unit is installed and disabled,
inactive,
has not been active since this boot,
and no cargo-shadow timer exists.
No record in `doc/` says why it is off,
the unit's own journal holds no entry at all,
and the user journal on this machine covers one boot (2026-11-28T05:53:37-05:00 onward),
so the reason is a gap in the machine's records and not a decision anyone can point to.
That gap is a finding for whoever keeps the machine,
and it is not what this question asks.

What a first pass would remove is not counted,
because a walk of the base is closed to agent runs.
Two earlier passes of the same tool over other populations are recorded:
a dry run of 2026-11-08 read 6,923 trees and would remove 18 (26.31 GiB),
keeping 6,905,
and the pass that ended 2026-11-10T07:37:24-05:00 judged 14,850 of 33,804 trees
and removed 278 for 511.49 GiB,
0.71 s per tree.
Today's base holds at least the 23 trees of 35.1 GiB this round read.

What a pass costs a build running beside it is not measured:
the unit's own caps bound it,
a pass that finds nothing costs about a minute on this machine,
and a 56-minute pass of this service is one of the load suspects the 2026-11-14 record names,
with no cause established there.

Three answers,
and they are exclusive:

- leave it off:
  nothing runs a pass,
  the trees of removed worktrees stay until a person removes them,
  and the disk holds 1442 GiB free;
- switch it on as installed:
  the owner's 2026-11-11 word is carried out,
  the first pass starts at once and may remove any tree whose workspace is gone and which has been idle seven days,
  and the unit keeps running a pass every 60 s thereafter under its own caps;
- switch it on with a bound:
  the same,
  with the deadline,
  the idle rule and the caps set now rather than left as they are,
  which today are `RuntimeMaxSec=19800`,
  seven days idle,
  12 GiB in a checkout and 24 GiB in the shared build folder.

The recommendation is the second,
because the owner's words of 2026-11-07 and 2026-11-11 are the only recorded decision on this,
because today's base holds 35.1 GiB in the 23 trees this round read and nothing is scheduled to remove any of them,
and because leaving it off continues a state no record asks for.

A yes to each answer leaves undecided:

- leave it off:
  what triggers a pass at all,
  and who notices when the disk fills;
- switch it on:
  whether the first pass is watched,
  and what the pass does to a build that runs beside it,
  which nobody has measured;
- switch it on with a bound:
  which deadline,
  which idle rule and which caps,
  and whether a bound set now has to be measured again on the new drive.

## What is not asked, and why

- Whether dependency builds should be shared across worktrees.
  The owner chose the prewarm on 2026-11-03,
  it runs (27 of 29 live workspaces named,
  23 holding a filled tree),
  and it carries the dependency units,
  so the question is answered.
  What remains inside it is a design question for a later round:
  the clone also carries the six path crates' units at the source checkout's absolute paths,
  which no build in the new worktree can read.
- Whether a round passes `CARGO_INCREMENTAL`.
  Two of its three answers restate forms the form question offers,
  and the third stays reachable without a decision,
  so it is a recorded default with the owner's veto open and not a question.
  Its section says what that leaves open.
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

- A first build in a worktree born by the prewarm:
  every first-build cell here is of an empty build folder,
  and what such a worktree compiles is seven units where these builds compiled 266 to 272.
  Neither the wall nor the memory of that population was measured.
- Form A's two commands in one build folder:
  no arm ran them,
  so what a round's pair costs under A,
  and how much of a pair's wall goes to the dependency phase,
  is unknown.
  Two form D pairs were measured (161.8 s and 168.4 s of wall for the two commands).
- Form Z on this drive:
  the only figures for Cargo's default are the 2026-11-13 page's,
  on the old drive,
  over three commands.
  Nothing of its loop,
  its peak or its tree here has been measured.
- Form B's cells in this round:
  the 2026-11-28 arms measured B,
  and every B figure here is theirs,
  taken in a round of its own;
  they were not repeated beside this round's controls,
  and their wide-edit test wall reads 1.00 in that round and 0.44 for D in this one.
- Form C's rebuild cells and form D2's cells:
  inferred from D's,
  which differ from C in the dependencies' data and from D2 only in the library build's own data.
  What was measured for C is two rounds of both commands,
  and what it writes and runs.
- `codegen-units` pinned to 16 beside an incremental form.
- One build under `CI=1` beside each form's manifest,
  which is the trial that would show the override closing the form.
- The units this crate builds besides the library and its test unit:
  `--all-targets`,
  the `native_binary` consumer target,
  `--bin`,
  clippy and `cargo check`.
- The effective `-C codegen-units` in a real invocation:
  the logs show Cargo passes no such flag,
  and rustc's own default is what no read here counted;
  a count of the `.o` files in one form D session directory would show it.
- What a rebuild writes:
  the arms record sizes,
  not bytes written,
  and the sizes are apparent bytes by the flag that took them,
  so nothing here says what the ZFS pool compresses or what a snapshot retains.
- How many rebuilds a round makes,
  and how many builds by kind run on this host:
  no record holds either,
  and a count of build scopes in the journal was not run.
- The base's population today:
  a walk of it is closed to agent runs,
  so what a first `cargo-shadow lean run` pass would remove now is unknown.
- What a pass costs a build that runs beside it.
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

## The gate reads this file has had

Three same-vendor gate reads read earlier versions of this file,
all on 2026-11-28,
each one a Claude Sonnet pass over a proposal written by Claude Sonnet,
which makes them second passes and not independent reviews.
No independent review has read any version of this file.

- The first read the version at `888d3646df` and returned four High,
  ten Medium and seven Low findings.
- The second read the version at `471d462ccf` and returned three High,
  ten Medium and nine Low findings;
  the revision that answered it is `22e50e6120`,
  merged as `e74da32adf`.
- The third read the version at `e74da32adf` and returned three High,
  eleven Medium and eight Low findings.

This revision answers the third read finding by finding,
and the round's report held each answer with the check that settled it;
that report is not kept in this repository,
so what each finding changed is what this file carries.
The findings' own text is not carried here:
each read stood in scratch that expires a day after its last change,
and what a read found is what this file changed.
