# The Git wrapper's development profile after the move to the new drive

Proposal,
2026-11-28T06:39:41-05:00.
Status:
unapproved and not final.
Nothing here is decided,
no setting was changed on the branch that carries this file,
and the owner declares completion.

Gate reads:
one same-vendor gate read has read the version of this file at commit `888d3646df`,
on 2026-11-28,
and came back not ready with four High,
ten Medium and seven Low findings.
It was a same-vendor second pass,
not an independent review,
and it stood in scratch that expires a day after it was written.
This revision answers every one of its findings,
and the section "What the gate read found,
and what changed" says where each one landed.
No gate read has read this revision,
and no independent review has read any version.

The measurements behind every number are in `package/git-policy/cli/doc/development-profile-measurements.md`,
under its heading dated 2026-11-28,
which names the command behind each figure.
The corner figures this proposal also uses for the machine's state were read on 2026-11-28,
between 06:15 and 06:40 -05:00,
and are named where they are used.

The setting it re-measures is in `package/git-policy/cli/Cargo.toml`:

```toml
# package/git-policy/cli/Cargo.toml
[profile.dev]
debug = "line-tables-only"
incremental = false
```

## What the owner is asked for

Three decisions,
separate,
each with its own question under "Questions for the owner":

1. Whether incremental compilation is on in this crate's development and test builds,
   for which of the two commands a round runs,
   and whether the repository's own path dependencies are included in that.
   Five forms are on the table:
   keep the setting as it is,
   or one of four manifest lines.
2. Whether a round passes `CARGO_INCREMENTAL` itself.
   This does not compose with decision 1 the way the earlier version of this file said it did:
   the variable overrides the manifest for every path source,
   so a round that passes `1` gets the everywhere form whatever the manifest holds,
   and a round that passes `0` gets the setting as it is.
3. Whether dependency builds are shared across worktrees at all.
   This one is not a profile setting,
   and it shares less than the earlier version claimed.

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
and `df -h /home` read 1.4 TB free at 2026-11-28T05:04:56-05:00.

The crate reached 1,298,623 lines in 4,012 Rust files below `src/native`,
680,703 of those lines in 1,565 `*_tests.rs` files,
counted with `find src/native -type f -name '*.rs' -exec cat {} + | wc -l` and the same command with `-name '*_tests.rs'`.

Memory became the limit,
and what the record of that night says is narrower than the earlier version of this file said.
`doc/handover/cli-git-rust-implementation.md`,
section "Memory",
at 21:48:24 on 2026-11-27:

> At 21:48:24 `MemAvailable` read 1549 MiB,
> swap held 50,404 of 66,219 MB,
> and the memory stall read `full avg10=46.85`,
> with `rustc` holding 28.8 GiB resident and seven guests each holding about 24 MiB of swap.

That sentence does not say one compiler,
and it does not say this crate's test unit:
it says `rustc`,
and the same section counts "eleven `rustc` processes and nineteen rounds" at 22:51:42,
so the writer was counting processes in aggregate.
The measured test units in this proposal peaked at 7,429 and 7,488 MiB (7.25 and 7.31 GiB),
so 28.8 GiB is about four such compilers at once,
which is what a minute of twenty-odd first builds holds;
a process's resident size should also not exceed its scope's cap,
and the night's rounds were capped at 24 GiB each.
So the figure stands as the record wrote it,
as a sum over processes that nobody has been able to itemise,
and it does not identify a single compiler to chase.
What the night does establish is that twenty-three rounds started between 21:11 and 21:42 -05:00,
that their first builds ran together,
and that the machine ran out of memory and swap.

## What was measured

### One arm

An arm is one build in a build directory of its own,
inside a scope of its own,
one build at a time,
with a memory-pressure gate at its start.
The two commands are a round's:

```sh
# run in package/git-policy/cli
cargo build --locked --offline --lib
cargo test --locked --offline --lib --features test-only-switches --no-run
```

Each arm records wall time from `date +%s%N`,
processor time from the scope's `cpu.stat`,
three memory readings,
described under "Three memory readings,
and what the sampler can miss",
and `du --summarize --bytes --apparent-size` of the tree.
Every arm ran on 2026-11-28.
The arms are twenty-one in all:
six first builds,
twelve rebuilds after one-line edits (six cells,
each with two controls),
two repeated incremental rebuilds that put a tree back for the next arm,
and one extra first build that kept a timeline of its dependency phase.

The edits are one line added at the end of one file,
reverted afterwards:

- the leaf file,
  `src/native/candidate/candidate_store_commit_listings.rs`,
  which no other Rust file uses (`rg -l '\bcandidate_store_commit_listings\b' src/native --glob '*.rs' | wc -l` answers 1,
  the parent's `pub mod` line);
- the widely used file,
  `src/native/worktree_clone_mount_protocol/mod.rs`,
  which 273 Rust files name,
  the most of any file in the crate.

A round's own edits sit between those two.

### The arms measured one form; three others are inferred

The harness set `CARGO_INCREMENTAL=0` or `1` for each arm (`harness.ts`,
the `env` object handed to the scope),
and it changed no manifest.
Cargo's own source explains what that means (`src/workspace/profiles.rs`,
fetched from `rust-lang/cargo` on 2026-11-28;
the comment's middle lines are left out here):

```txt
// src/workspace/profiles.rs
// Incremental can be globally overridden.
if let Some(v) = self.incremental {
    profile.incremental = v;
}

// Only enable incremental compilation for sources the user can
// modify (aka path sources).
if !is_local {
    profile.incremental = false;
}
```

So the variable reaches every path source,
this crate and the repository's path dependencies alike,
over any package override the manifest holds.
The twenty-one arms therefore measured the everywhere form (option B),
not the package-scoped forms.

This round ran its own plan-only probe to see the forms the arms did not measure.
`cargo ... --unit-graph -Z unstable-options` prints Cargo's planned units and exits,
and it compiled nothing:

```sh
# in a throwaway crate with one path dependency outside its workspace root
cargo build --lib --offline --unit-graph -Z unstable-options
cargo test --lib --no-run --offline --unit-graph -Z unstable-options
```

Six manifests,
each run with both commands,
answered as follows (the dependency's units and this crate's,
with `incremental` as Cargo resolved it):

- the setting as it is:
  dependency false in both commands,
  this crate false in both.
- `[profile.dev] incremental = true`:
  dependency true,
  this crate true,
  in both.
- `[profile.test] incremental = true` beside dev false:
  build command both false;
  test command dependency **true** and this crate true.
- `[profile.dev.package.monochromatic-git-policy-cli] incremental = true` beside dev false:
  dependency false in both commands;
  this crate **true** in both,
  so the dev override does reach the test profile.
- `[profile.test.package.monochromatic-git-policy-cli] incremental = true` beside dev false:
  build command both false;
  test command dependency false and this crate true.
- `CARGO_INCREMENTAL=1` with dev false:
  dependency true and this crate true in both,
  which is the everywhere form.

Every variant printed `codegen_units=null`,
so no manifest line and no Cargo default in these forms passes `-C codegen-units`,
and the compiler's own default follows the flag (see "The default that moves").

So:
the arms measured the everywhere form,
the test-profile forms are shown by the unit graph and were not built,
and no first build or rebuild with a package-scoped line exists in any record.

### Three memory readings, and what the sampler can miss

Every arm has three:
the largest `rustc`'s own high-water mark (`VmHWM`,
sampled once a second for every `rustc` in the scope),
the scope's `memory.peak` (the kernel's mark for everything in the scope,
page cache included),
and the scope's `memory.stat` `anon` share,
sampled once a second.
They disagree in size and they agree in direction at all six cells,
so this proposal never quotes one alone.

The sampler can miss two things:
a compiler that lives less than a second,
and growth in the last second before a compiler exits.
The second is real in the library arms,
whose compiler's peak was first read one to two seconds before its last read:
+522 MiB over the last five reads in the first control,
+387 and +300 in the wide controls,
+251 in the wide treatment;
and flat in the test arms.
No margin quoted in this file is small enough to be erased by a missed second.
The sampler also sees `rustc` only,
so a build script's own run is invisible to it (see "The dependency phase has a peak the sampler cannot see").

### What these builds can show

Two faithful builds of one unchanged crate differed by up to 20% on this host (`package/git-policy/cli/doc/development-profile-measurements.md`,
the 2026-11-25 section,
"The control's spread"),
and the rule the page draws from that is "a difference smaller than 1.20 is no difference these builds can show".
The 2026-11-28 section's own control pairs mostly agree within 2%,
with one pair 16.5% apart,
and one control that took 2.7 times the processor time of its twin.
So every figure in this file carries three things:
the treatment's ratio to the mean of its two controls,
the range that dividing by each control alone gives,
and the spread between the two controls.
An effect inside its controls' spread is reported as no difference the measurement can show,
and a single-run cell inside 20% is reported the same way.

## What a rebuild costs, wall and processor time kept apart

Wall is what one person waits for;
processor time is what the shared machine pays.
They move differently,
and the earlier version of this file used one word,
"time",
for both.

### First builds

Library build,
incremental on:
wall 0.91 of the controls (89.3 s against 94.0 and 103.2 s),
inside those controls' 9.8% spread,
so no difference in wall is shown;
processor time 0.91 (400.8 s against 435.1 and 446.1 s),
which clears their 2.5% spread.

Library test build,
incremental on:
wall 1.22 (165.7 s against 125.5 and 145.2 s;
range 1.14 to 1.32),
which clears the controls' 15.7% spread;
processor time 1.03 (528.5 s against 481.9 and 547.1 s),
inside their 13.5% spread,
so no difference is shown.
Inside that first test build,
this crate's own compile is 99.1 s against 57.6 s and 66.4 s,
which is where the extra wall sits.

### Rebuilds after one line in the leaf file

Library build:
wall 1.18 (37.4 s against 37.2 and 26.1 s),
inside the controls' 42.5% spread,
so no wall gain is shown;
processor time 0.41 (17.9 s against 45.4 and 42.2 s).
The page explains the wall with Cargo's package cache lock (three `Blocking waiting for file lock on package cache` lines,
7.45 s,
so the build itself took 30.0 s),
and the restore arm contradicts that explanation:
`lib-leaf-on-restore` is the same incremental rebuild in the other direction,
has no lock line in its log,
and took 37.2 s of wall with 18.6 s of processor time.
So this cell's wall result stands as no wall gain shown,
with the lock as context rather than as the reason.
The two library arms also started at the two highest memory-pressure readings of their round,
`full avg60` 8.03 and 8.26 of a gate that opens below 10,
so their walls may carry host load;
processor time is the steadier figure at this cell.

Library test build:
wall 0.50 (32.3 s against 66.6 and 62.2 s;
range 0.48 to 0.52),
processor time 0.26 (31.6 s against 125.0 and 119.3 s),
both clearing their controls' 7.1% and 4.8% spreads.

### Rebuilds after one line in the widely used file

Library build:
wall 0.56 (13.4 s against 22.2 and 25.3 s),
processor time 0.34 (13.4 s against 37.9 and 41.1 s),
both clearing the controls' 14% and 8.4% spreads.

Library test build:
wall 1.00 against the control that ran two minutes before it (72.0 s against 72.3 s),
and 0.35 against the control that ran after it (208.2 s),
which no property of that arm explains;
processor time 0.35 against the first control and 0.13 against the second.
Only the first pairing is worth reading,
and inside it the cell's 188% control spread covers both wall and processor time,
so this cell shows no wall or processor-time difference at all.
Its memory readings do clear the spread,
and they are in the next section.

### What that adds up to for a round

Of the four rebuild cells,
processor time falls in all four (0.41,
0.26,
0.34,
and 0.35 on the readable pairing),
and wall falls in two of the four cells it can be read in (0.50 on the leaf test build,
0.56 on the wide library build),
with the other two showing no wall difference this measurement can see.

How that lands on a round depends on how many rebuilds a round makes after its first build,
a number no record holds.
On the test unit,
with the arms' own means:

- processor seconds,
  first build plus k leaf rebuilds:
  today 515 s at k=0 and 637 s at k=1;
  with incremental data 529 s at k=0 and 560 s at k=1;
  the treatment is the smaller from k=1 (0.88 of today) and reaches 0.71 at k=3.
- wall seconds,
  the same shape:
  today 135.3 s at k=0,
  199.8 s at k=1,
  264.1 s at k=2;
  with incremental data 165.7 s,
  198.0 s,
  230.3 s,
  so 1.22,
  0.99 and 0.87 of today.

So a round that rebuilds the test unit once pays its first build's extra wall back exactly and gains nothing in wall,
and gains processor time from its first rebuild.
A round with no rebuild after its first build pays 1.22 in wall and gains nothing.
Which shape the rounds have is unmeasured;
it is the one input the ranking cannot settle from these arms.

## Peak memory, per phase and per reading

### First builds

Library build:
largest compiler 0.99 (2,825 MiB against 2,853 and 2,855 MiB),
scope peak 0.99 (3,796 against 3,817 and 3,817 MiB),
anonymous 1.00 (3,503 against 3,525 and 3,509 MiB).
The treatment is 1.0% under the lower control on the compiler reading,
which is inside the 20% these builds can show,
and the same on the other two.

Library test build:
largest compiler 0.94 (7,013 MiB against 7,429 and 7,488 MiB),
scope peak 0.95 (7,183 against 7,552 and 7,617 MiB),
anonymous 0.96 (6,808 against 6,990 and 7,259 MiB).
The treatment is 5.6% under the lower control on the compiler reading,
and one run on each side;
under the page's own 1.20 bar none of the three first-build cells shows a difference,
and the 2% agreement of the stable control pairs means a second run could land above the controls.
So a first build's peak is reported as **unchanged as far as this measurement can tell**,
not as lowered.

### Rebuilds

Library test build after the leaf edit:
largest compiler 0.67 (4,969 MiB against 7,348 and 7,499 MiB;
controls 2.1% apart),
scope peak 0.78 (5,737 against 7,330 and 7,420 MiB),
anonymous 0.63 (4,475 against 7,140 and 6,964 MiB).

Library build after the leaf edit:
largest compiler 0.68 (2,083 MiB against 3,275 and 2,812 MiB),
scope peak 0.80 (2,396 against 3,246 and 2,739 MiB),
anonymous 0.63 (1,828 against 3,167 and 2,664 MiB).

Library build after the wide edit:
largest compiler 0.72 (2,016 MiB against 2,798 and 2,813 MiB),
scope peak 0.84 (2,331 against 2,778 and 2,741 MiB),
anonymous 0.69 (1,768 against 2,576 and 2,579 MiB).

Library test build after the wide edit:
largest compiler 0.67 (4,899 MiB against 7,411 and 7,270 MiB),
scope peak 0.77 (5,628 against 7,408 and 7,241 MiB),
anonymous 0.64 (4,491 against 6,964 and 7,046 MiB).

So on a rebuild the compiler's own peak falls to about two thirds,
the scope's peak to about four fifths,
and the scope's anonymous memory to about three fifths,
and all four cells agree on that in all three readings.
The earlier version of this file said "about half again" for the concurrency gain,
which was the middle reading alone;
that sentence is withdrawn.

### The dependency phase has a peak the sampler cannot see

The sampler records `rustc` only,
and the dependency phase of a first build has a peak that is not a compiler.
In the timeline kept beside the measurement,
the first test build's scope climbs from 1,291 MiB at 41 s to 3,756 MiB at 61 s with no `rustc` or one running,
and this crate's unit starts at 67.9 s.
The 2026-11-25 research names the cause,
in the paragraph "A build's peak is one process" of `doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md`:
"the `forbidden-strings` build script's run at 3756 MiB in every worktree where its output is not fresh".
So the dependency phase's peak is about 3.8 GiB of anonymous memory in a fresh worktree,
and the largest dependency compiler's 466 MiB says nothing about it.
For the library build,
whose compiler peaks at 2.8 GiB,
the dependency phase is the scope's peak (the extra first build measured for this:
scope 4,267 MiB against the compiler's 2,830 MiB).

### What that leaves for the builds, now that the guests hold three quarters

The owner's answer of 2026-11-28 (`doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`,
"The unit's cap") puts the guests' unit at three quarters of the machine's memory,
which was set on the running unit at 2026-11-28T05:45:26-05:00 as 49,664,753,664 bytes,
and bounds guests alive at six.
The same answer says what that leaves:
"everything else drops to 15.4 GiB at once".

Against that 15.4 GiB,
per build,
on this crate's test unit:

- largest compiler,
  today 7,429 MiB (7.25 GiB):
  two builds fit;
  with incremental data 4,969 MiB (4.85 GiB):
  three fit.
- scope peak,
  today 7,552 MiB (7.37 GiB):
  two fit;
  with incremental data 5,737 MiB (5.60 GiB):
  two fit.
- anonymous,
  today 6,990 MiB (6.83 GiB):
  two fit;
  with incremental data 4,475 MiB (4.37 GiB):
  three fit.

So the number of test builds the non-guest share holds moves from two to three on two of the three readings,
and stays at two on the third,
and a first build does not move at all.
Swap absorbs whatever overflows,
and the rounds' builds are what the kernel has been killing when swap filled.
That is the whole of the capacity claim this measurement supports;
the earlier version's "about eight compilers" divided the machine's total memory by one reading,
counting memory the guests hold.

## The default that moves with the flag

Incremental compilation does not only write data:
it moves a default the owner put off-limits.

Cargo's reference (`https://doc.rust-lang.org/cargo/reference/profiles.html`,
fetched 2026-11-28) says of `codegen-units`:

> The default is 256 for incremental builds,
> and 16 for non-incremental builds.

and lists the built-in `dev` profile with `incremental = true` and `codegen-units = 256`.

rustc's reference (`https://doc.rust-lang.org/rustc/codegen-options/index.html`,
fetched 2026-11-28) says the same from the compiler's side:

> The default value,
> if not specified,
> is 16 for non-incremental builds.
> For incremental builds the default is 256 which allows caching to be more granular.

This crate sets no `codegen-units` in `[profile.dev]`,
and every variant of this round's unit-graph probe printed `codegen_units=null`,
so Cargo passes no flag and the compiler's default follows the flag.
Turning incremental on therefore moves every affected unit from 16 code generation units to 256,
and the 2026-11-13 change moved the same default the other way for every unit,
which no record of that round mentions either.

The owner's words of 2026-11-04 put that setting off-limits (`doc/planning/lean-build-dirs.md`,
"Owner requirements that bound this"):

> "(debuginfo level,
> incremental off for dependencies - no messing with these."

> "other profile settings (split-debuginfo,
> strip,
> codegen-units) should be off-limits too".

and the same document scopes them:
"So no build profile setting is changed or proposed here".
The 2026-11-13 round changed two of those settings,
`debug` and `incremental`,
after those words,
and the page records the `debug = false` arm as rejected by the owner without recording the owner's answer anywhere else;
`rg` over `doc/decision/` and `doc/handover/` finds no record of either that answer or of an acceptance of the two lines.

Two consequences follow,
and they are the owner's to read:

- the arms changed `incremental` and the effective `codegen-units` together,
  so the measurement cannot say which part of the rebuild's time and peak is the incremental data and which the unit count;
- a yes to any of options B,
  C,
  D or D2 is also a yes to 256 code generation units in this crate's units,
  and in the path dependencies' units for B and C,
  unless the owner pins `codegen-units = 16` beside it,
  which is itself a profile setting and is unmeasured here.

## Where the data lands, and who removes it

A build's tree is not in the worktree.
`cargo-shadow` sets `CARGO_BUILD_BUILD_DIR` for the child Cargo to `<base>/toolchain/<toolchain>/<two hex>/<fourteen hex>`,
where the key is SHA-256 of the canonical workspace root,
and leaves a ledger file `.cargo-shadow-workspace` naming that root (`package/cargo-shadow/cli/src/lean_keyed_folder.rs`,
`workspace_key` and `keyed_build_dir`;
`doc/decision/cargo-build-dir-host-cache.md`,
"How the template resolves here").
Two worktree paths are two keys.

Incremental data lands below that tree,
`debug/incremental/<crate>-<hash>/s-<session>/` (`doc/decision/cargo-build-dir-host-cache.md`,
"The build-dir layout is internal"),
and a worktree copy neither carries nor reclaims it.

What is on disk now,
measured this round:

- 33 folders under `.claude/worktrees/` by name (`ls -d /home/user/Monochromatic/.claude/worktrees/*/ | wc -l`,
  2026-11-28T06:19:56-05:00),
  and `git worktree list` read 30 entries in the same hour,
  which moves while rounds start and end;
- of those 33 worktrees and the main checkout,
  2 hold a keyed folder with a `debug` tree for `package/git-policy/cli`:
  the main checkout and `gate-wrapper-next-alpha`;
  the other 32 hold none,
  which is a reading of one stat per workspace,
  not a walk of the base;
- a real tree measures 2,174,623,752 bytes (2.03 GiB) for the main checkout,
  and 3,228,469,226 bytes (3.01 GiB) for another worktree,
  1.8 and 2.6 times the 1.14 GiB model the earlier version of this file used;
  that model does not survive its own components either (787 + 895 MiB less 572 MiB of dependency units is 1,110 MiB,
  and the page's own two components put the dependency units at 564 MiB).

Trees outlive their worktrees,
and nothing prunes them today:

- the decision record says "Throwaway worktree builds outlive their worktree",
  and "Nothing prunes `~/.cargo/build/` automatically";
- `cargo-shadow lean` removes a tree only once its workspace is gone and it has been idle for `TREE_IDLE_SECONDS`,
  seven days (`package/cargo-shadow/cli/src/lean_policy.rs`);
- the unit that would run that pass continuously,
  `cargo-shadow-lean.service`,
  reads loaded,
  disabled and inactive at 06:20 on 2026-11-28,
  and `systemctl --user list-timers` lists no cargo-shadow timer,
  so no pass runs by itself;
- a pass over every tree costs about 6.6 h at the size the base had on 2026-11-10,
  when it held 33,804 trees (`package/cargo-shadow/cli/README.md`,
  "Keeping the build folders lean").

How fast trees are made,
counting distinct `git-policy` branch names merged per day (`git log --merges --since=<day>T00:00:00-05:00 --until=<day>T23:59:59-05:00 --format='%s'`,
then the quoted names,
deduplicated):
62 on 2026-11-25,
54 on 2026-11-26,
166 on 2026-11-27,
24 on 2026-11-28 to 06:20.
That is a proxy:
a merged branch is not a built tree,
and rounds that never merge leave trees too.

So the disk cost of any option here is not a number of live worktrees times a tree size:
it is a number of trees that stay until somebody runs a pass,
and the only count of that population is the base's own,
from 2026-11-10.

What a person does about it,
whichever option the owner picks:

- the data going stale:
  `cargo clean` inside the package clears that workspace's build dir,
  and deleting `<base>/toolchain/<toolchain>/<two hex>/<fourteen hex>/` for a workspace no longer built removes a whole tree;
- the disk filling:
  the trees of removed worktrees go first,
  then idle trees of live ones;
  a pass bounded to trees is `cargo-shadow lean run --only trees --deadline-minutes <n>`;
- turning the removal into a standing rule is one decision of the owner's,
  and it is asked under "Questions for the owner".

## The options

Each option states the line and the file,
what it changes for good,
what lands on disk and where,
the loop and the first build in the arms' own numbers,
and what a yes would still leave undecided.
The numbers are the everywhere form's (what the arms measured) unless the option says they are inferred.

### A. The setting as it is

`package/git-policy/cli/Cargo.toml`,
`[profile.dev]` holding `debug = "line-tables-only"` and `incremental = false`,
unchanged.

What it does:
no incremental data is written;
every edit compiles the whole crate again;
file and line stay in every backtrace frame,
because line tables are kept.

What lands where:
a first library build's tree is 787 MiB and a first test build's 895 MiB,
and both commands in one directory share the 572 MiB of dependency units the page measures (564 MiB by its own two components),
so the model puts a round's tree at about 1.1 GiB;
real trees measure 2.03 and 3.01 GiB.

What it costs a round:
the test build's compiler peaks at 7,429 to 7,488 MiB every time,
on the first build and on every rebuild,
so two such builds fit the 15.4 GiB the guests leave;
a rebuild after a one-line edit costs 2.4 to 2.5 times the treatment's processor time,
45.4 s and 42.2 s against 17.9 s on the library leaf cell and 125.0 s and 119.3 s against 31.6 s on the test leaf cell,
and 66.6 s and 62.2 s of wall for the test build.

Cleanup:
nothing to remove,
because no incremental data is written.

What a yes leaves undecided:
nothing about the profile,
and everything the other options ask about stays open.

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
so both commands and every path dependency are compiled with incremental data,
and every affected unit's `codegen-units` default moves from 16 to 256.

What lands where:
the implementation's `debug/incremental/<crate>-<hash>/s-<session>/` below the keyed tree,
a first library tree is 1,551 MiB,
of which 808 MiB is incremental data;
a first test tree is 2,705 MiB,
of which 1,824 MiB is data ( of which 1,616 MiB is this crate's own data,
161 MiB the path dependencies',
and 47 MiB unaccounted);
after a rebuild the test tree reads 2,658 MiB with 1,777 MiB of data.
A round's one tree is about 3.5 GiB by the model and above that in reality.

What it gives:
the rebuilds' processor time,
and the peaks of "Peak memory,
per phase and per reading",
on the one form that was actually measured.

What it costs:
a first test build's wall at 1.22 of today and this crate's own compile at 99.1 s against 57.6 s and 66.4 s,
about 1.8 GiB more per tree,
the path dependencies' 161 MiB which no rebuild reads,
and the owner's 2026-11-04 words on dependencies.
The tree grows with no bound of its own and stays until someone removes it.

Cleanup:
`cargo clean` in the package,
or deleting the keyed folder of a workspace no longer built,
as under "Where the data lands,
and who removes it";
nothing removes it by itself,
and a pass over every tree costs about 6.6 h.

What a yes leaves undecided:
the `codegen-units` reading,
who removes stale trees,
and whether the units this proposal never measured (`--all-targets`,
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
and the test profile inherits the `dev` package overrides.
The unit graph shows the path dependencies also become incremental in the test command,
so this form writes their 161 MiB too,
and it separates them from their dev units:
Cargo compares units by a tuple that includes `incremental` (`Profile::comparable()` in `src/workspace/profiles.rs`,
fetched 2026-11-28),
so a round that runs both commands holds the dependencies' library units twice.
The build-script *run* unit is not doubled,
because Cargo strips its profile to fields that carry no `incremental` (`get_profile_run_custom_build`),
and the probe's graph shows one run unit per command.

What lands where:
the test tree carries the incremental data (1,824 MiB at the first build,
1,777 MiB after a rebuild),
and a round's one tree is 2.8 GiB by the model,
plus whatever the second set of dependency units adds,
which is inferred and not sized.

What it gives:
the test build's loop (0.26 processor time,
0.50 wall,
the peaks under "Peak memory,
per phase and per reading"),
which is the build that sets the peak and takes the longest;
the library build keeps today's numbers.

What it costs:
the dependencies' 161 MiB and their second unit set,
so it breaks the dependency part of the owner's words as B does,
while delivering less than B.

Cleanup:
the same as B:
`cargo clean` in the package or the keyed folder of a workspace no longer built,
by hand,
until a pass runs.

What a yes leaves undecided:
the `codegen-units` reading,
the dependency constraint,
and the cleanup.

### D. Incremental compilation for this crate's own units

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
this crate's units are incremental under both commands:
the unit graph shows the dev package override reaching the test profile,
and the path dependencies stay as they are,
so their units are shared between the two commands and stay cold.

What lands where:
only this crate's data:
1,616 MiB on the test tree and 647 MiB on the library tree,
so about 2.21 GiB more per tree that builds after the line,
and nothing for the dependencies.

What it gives:
the everywhere form's loop on both commands,
without the dependencies' 161 MiB,
which is the form that meets the owner's 2026-11-04 words on dependencies exactly.

What it costs:
the first build's wall and this crate's own compile are **not measured for this form**:
the arms measured the everywhere form,
and this form differs from it only in the path dependencies' data,
so the first-build figures under "First builds" are inferred from that and not measured.
Its rebuild figures are inferred too:
a rebuild recompiles one unit,
and this crate's unit is that unit,
so the measured rebuild cells carry over;
a round with this line would confirm it in two interleaved builds.

Cleanup:
the same as B,
and only for this crate's data:
`cargo clean` in the package,
or the keyed folder of a workspace no longer built.

What a yes leaves undecided:
the `codegen-units` reading (the crate's units still move 16 to 256),
who removes stale trees,
the unmeasured units,
and confirmation of the first build.

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
The unit graph shows exactly that,
and this is the form an earlier round's kept probe manifest held.

What lands where:
1,616 MiB of this crate's data on the test tree,
no library data and no dependency data,
the smallest of the four forms that write anything,
and nothing at all for a round that never runs the test build.

What it gives:
the test build's loop,
peaks and data,
and nothing for the library build,
whose rebuild stays at today's cost,
2.4 to 2.5 times the treatment's processor time.

What it costs:
the test build's first-build wall (inferred,
as for D),
and the cleanup.

Cleanup:
the same as D,
for 1,616 MiB of this crate's data on the test tree.

What a yes leaves undecided:
the `codegen-units` reading,
who removes stale trees,
and whether the library build's loop is worth its own 647 MiB later.

### E. Dependency builds shared across worktrees

Not a profile setting,
and not an alternative to any profile form.

What it does:
a shared build directory or a compiler cache,
so a second worktree's first build reuses what the first one compiled.
Nothing of a build is shared today:
the key is the canonical workspace root,
and the registry sources and the toolchain are already shared.

What it can share:
registry crates only.
Cargo hashes a path dependency outside the workspace root by its absolute path (`src/workspace/source_id.rs`,
"Paths outside the workspace are hashed as absolute paths",
read 2026-11-28),
and this crate is the root of its own `[workspace]`,
whose five path dependencies sit outside it (`package/git-policy/cli/Cargo.toml`).
So the 572 MiB of dependency units that repeat in every worktree's first build is partly shareable and partly not,
and nothing kept sizes the shareable part.

What it cannot share:
this crate's own unit,
which is the peak that limits rounds;
any rebuild,
whose one compiled unit is this crate's own;
the `forbidden-strings` build script's run,
whose 3.8 GiB is the dependency phase's peak;
and the 24 s of quiet between the last dependency and this crate's unit in the timeline (the script's own run,
not a compiler).

What it would save:
part of a fresh worktree's dependency phase (266 to 271 units in 64.3 to 78.8 s,
and 185.5 s in the arm that ran on a slow host).

What it costs:
a mechanism nothing owns today.
A shared build directory serialises builds on Cargo's own directory lock;
`sccache` refuses the test unit in the record's inference;
making the path dependencies shareable would mean moving them inside this crate's workspace,
which is a repository layout change and not a profile one.

What a yes leaves undecided:
who owns the mechanism,
what it stores and where,
what happens when its lock is held,
and what a stale entry means for a rebuild.

### F. A round's own `CARGO_INCREMENTAL`, a complement

Not a profile setting.
A round's task passes `CARGO_INCREMENTAL=1` for its builds,
and the data is removed when the round ends.

What it does:
Cargo's source shows the variable overrides the manifest for every path source,
so a round passing `1` gets option B's effect whatever the manifest holds,
including the dependencies' 161 MiB which option D avoids;
a round passing `0` gets option A's effect.
The unit graph confirms both.

What it costs:
a step in every round's task set,
and the removal of a tree that otherwise stays until a pass runs it,
which no record shows happening by itself.
A round that forgets the variable writes no data (for `0`) or all of it (for `1`),
and nothing in the manifest explains either.

What a yes leaves undecided:
which value a round passes,
who removes the data and on what signal (the page already records the obstacle:
"A per-run tree that ends with its run needs the run to say when it ends,
which the driver does not do today"),
and what a round that forgets it gets.

### Combinations that cannot be built

- D or D2 with a round passing `CARGO_INCREMENTAL=1`:
  the variable wins,
  so that round is B in effect,
  and D2's test-only scoping turns into B for both commands.
- C with a round passing `CARGO_INCREMENTAL=1`:
  the same,
  so C becomes B.
- Any form with a round passing `CARGO_INCREMENTAL=0`:
  that round builds as A,
  which is coherent as a per-round escape and is the only "both" this file offers.
- B or C with the owner's dependency words still standing as written:
  the lines write dependency data,
  so they cannot be adopted without the owner saying those words do not reach them.
- E with nothing else needed:
  it can be built beside any of A,
  B,
  C,
  D,
  D2,
  and it changes none of their numbers,
  except that sharing registry crates would shorten a fresh worktree's first build for all of them.

## Ranking

The criterion is what each option delivers for the target workload the records describe:
many agent rounds at once,
each in a worktree of its own,
on a machine whose memory is the limit (`doc/handover/cli-git-rust-implementation.md`,
"Memory":
twenty-three rounds started together on 2026-11-27;
166 `git-policy` branches merged that day).
In that workload every round pays one first build and then k rebuilds,
and the guests hold three quarters of the memory all the while.

Ranking:
D > B > D2 > C > A.

Why each adjacent pair:

- D over B:
  both deliver the same loop,
  since a rebuild recompiles this crate's one unit either way.
  D writes 161 MiB less per tree,
  keeps the dependencies cold as the owner's 2026-11-04 words ask,
  and keeps their units shared between the two commands.
  What D gives up is that its own first build is unmeasured;
  what B gives is that every one of its numbers was measured as the arms ran it.
- B over D2:
  B gives both commands' loops,
  and the library build's loop is 0.41 processor time and a compiler peak at 0.68 of today;
  D2 leaves the library build at today's cost.
  What B pays for that is 647 MiB of this crate's library data and 161 MiB of dependency data,
  808 MiB in all,
  of which 161 MiB is data no rebuild reads and the owner's words put off-limits.
- D2 over C:
  the same test-build loop and the same first-build cost,
  inferred the same way,
  with no dependency data,
  one set of dependency units per tree,
  and no dependency constraint broken.
  C delivers the same loop for 161 MiB more and an inferred second set of dependency units.
- C over A:
  C gives the test build's rebuild at 0.26 of its processor time,
  0.50 of its wall and two thirds of its peak,
  which is the build that sets the peak and takes longest;
  A gives the smallest first builds and no data at all.
  A is the answer where a round makes no rebuild after its first build,
  because there the extra first-build wall is paid and nothing earns it back;
  on the test unit's wall arithmetic the two are level at one rebuild and C leads from two,
  and k is not measured,
  which is the one input that could move A above C.

E and F stand outside this order.
E is not an alternative to any of them and shares only registry crates;
F is not an alternative either,
and passing `1` turns any form into B.
The order in which they can be had is:
pick one of A,
B,
C,
D,
D2,
then add E if a mechanism is wanted,
and treat F as a per-round escape (passing `0`) rather than as a gain.

What the ranking does not settle is the bound on concurrent builds,
which no option sets,
and which the owner's records already answer differently (question 6).

## Questions for the owner

### Question 1. Which form does the dev and test profile take?

Each form adds one line to `package/git-policy/cli/Cargo.toml`,
beside `[profile.dev] debug = "line-tables-only"` and `incremental = false`:

- A,
  nothing added:
  no incremental data anywhere,
  the whole crate recompiled after every edit.
- B,
  `[profile.dev] incremental = true`:
  both commands,
  this crate and its path dependencies,
  and the `codegen-units` default moves from 16 to 256 for all of them.
- C,
  `[profile.test] incremental = true`:
  the test command only,
  this crate and its path dependencies.
- D,
  `[profile.dev.package.monochromatic-git-policy-cli] incremental = true`:
  both commands,
  this crate only,
  dependencies cold.
- D2,
  `[profile.test.package.monochromatic-git-policy-cli] incremental = true`:
  the test command only,
  this crate only,
  dependencies cold.

The recommended one is D:
this crate's units only,
both commands,
about 2.21 GiB more per tree that builds,
the dependencies untouched.

A yes to D leaves undecided:
whether the `codegen-units` default moving 16 to 256 is acceptable (question 3),
who removes stale trees (question 2),
confirmation of D's own first build in two interleaved builds,
and whether the units this file never measured are inside it.

A yes to any form also decides nothing about E or F.

### Question 2. Is the tree growth acceptable, and should something remove it?

Counts:
33 worktrees alive at 2026-11-28T06:19:56-05:00,
2 of them and the main checkout holding a tree for this crate;
a real tree measuring 2.03 and 3.01 GiB;
1.4 TB free at 05:04:56;
trees outliving their worktrees and waiting seven idle days for a pass that nothing runs today;
33,804 trees in the base on 2026-11-10,
and a full pass costing about 6.6 h;
24 to 166 `git-policy` branches merged per day in the four days before this file.

The profile forms add 1.6 to 2.4 GiB to each tree that builds after the line,
against trees already at 2.03 and 3.01 GiB.

A yes to "the growth is acceptable" leaves undecided how many trees there will be,
how long they live,
and who removes them.
A yes to "something should remove them" leaves undecided which mechanism (a hand-run pass,
the disabled service,
or a task in a round's own set),
what bound the pass gets,
and what "idle" means for a tree whose worktree is gone.

### Question 3. Do the 2026-11-04 words still stand, and does the `codegen-units` default sit inside them?

The words are quoted under "The default that moves with the flag",
and they name three things this change touches:
the debuginfo level (moved on 2026-11-13),
incremental compilation off for dependencies (which B and C break,
D and D2 keep),
and `codegen-units` off-limits (which every form moves from 16 to 256 by default,
because the crate sets no such key and Cargo passes none).

No record in `doc/decision/` or `doc/handover/` holds the owner's words for either answer:
neither a rejection of `debug = false`,
nor an acceptance of the two 2026-11-13 lines.
So this file cannot say whether that change was an exception the owner granted.

A yes to "they still stand" leaves two things undecided:
whether a package override that touches only this crate counts as messing with dependencies,
which D and D2 rest on;
and whether `codegen-units` may be pinned to 16 beside incremental,
which is unmeasured and would cost the parallelism 256 buys.

### Question 4. Should a round pass `CARGO_INCREMENTAL`, and with which value?

The variable overrides the manifest for every path source,
so `1` is the everywhere form and `0` is the setting as it is.
It cannot be added beside D or D2 to keep their scoping.

A yes to "rounds pass `1`" leaves undecided who removes the data and when,
and it makes the manifest line moot for every round that passes it.
A yes to "rounds pass `0`" as an escape leaves undecided when a round should use it.

### Question 5. Should dependency builds be shared across worktrees at all?

What such a mechanism can share is registry crates only,
because the path dependencies' unit hashes hold absolute paths;
it saves part of a fresh worktree's first build,
nothing of a rebuild,
and nothing of the peak that limits rounds.
It needs an owner and a design before a yes can mean anything.

A yes leaves undecided who owns it,
what it stores,
how its lock behaves when two worktrees build at once,
and whether the path dependencies should move inside this crate's workspace to become shareable.

### Question 6. Does anything need to bound how many wrapper builds run at once?

The records already hold the owner's answers,
and this question is put against them rather than around them:

- batches are bounded by guests alive,
  and by nothing else the question offered (`doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`,
  "The rounds that were running,
  and what bounds a later batch"),
  six guests since 2026-11-28T05:44-05:00;
- the guests' unit takes three quarters of the machine's memory,
  which leaves 15.4 GiB for the builds,
  the gate and the desktop,
  set on the running unit at 05:45:26;
- no start or merge check waits on memory or on a spacing between starts (the owner,
  2026-11-26,
  quoted in the handover's "Memory":
  "I didn't tell you to apply the 3 minutes apart and memory test rules.");
- the agent-count search is a separate bound,
  starting from twenty and bisecting for the drive (`doc/handover/agents-at-once-on-this-machine.md`,
  "The cap now").

On this crate's numbers the 15.4 GiB share holds two wrapper test builds today,
and three with incremental data on two of the three readings,
and the night's builds' slice peaked at 55,702,958,080 bytes,
so neither share is shown to be the larger at the moment memory ran out.

A yes to "builds need their own bound" leaves undecided which reading it is set from,
whether it counts a first build or a rebuild,
who enforces it,
and what happens when the guests end and the share changes.
A no leaves the guests-alive bound standing alone,
as the owner's answer already has it.

## What is not asked, and why

`debug = false` is not put to the owner.
The 2026-11-13 page records it as rejected for what it costs a backtrace,
no decision record holds those words,
and how much memory it saves is measured neither there nor here:
the 2026-11-25 pass table puts code generation at 2,288 of 6,410 MiB,
"not measured".
Asking to reconsider a rejection without the number invites a yes or a no on a guess;
two interleaved arms would give the number.

## What was not measured

Named so that no reader takes its absence for a result:

- The forms C,
  D and D2 themselves:
  no first build and no rebuild exists with any of those lines in place.
  The unit graph shows what each line resolves to;
  everything else about them here is inference.
- The units this crate builds besides the library test build:
  `--all-targets`,
  the `native_binary` consumer target,
  `--bin`,
  clippy and `cargo check` (the last two at 4.0 to 4.3 GiB of peak in the 2026-11-25 section,
  under a different setting).
- A second worktree's first build at its own path:
  no second worktree of the repository was built for this measurement,
  so what a fresh worktree pays is argued from the key and from the first builds measured.
- The effective `-C codegen-units` in a real invocation:
  the unit graph shows `null` for every form,
  and what `cargo build -v` prints needs a compile.
- What a rebuild writes:
  the arms recorded sizes,
  not bytes written,
  and their "disk" columns equal their apparent sizes,
  so nothing here says what one rebuild writes on the ZFS pool,
  what compression keeps,
  or what a snapshot retains.
- The guests' builds:
  guest arms build this crate with `--release` (`cargo build --release --test native_binary`,
  `cargo test --release --lib --no-run … -j 2`,
  `doc/planning/mvm-guest-memory-held-for-a-rounds-life.md`),
  and one release compiler of this crate's tests was measured at 7,019,956 kB of a guest (6.69 GiB),
  so `[profile.dev]` and `[profile.test]` do not reach those builds at all,
  and the guest unit held 29.9 GB of memory and about 50 GB of swap with ten guests on the night.
- How many rebuilds a round makes after its first build (k),
  which the ranking's A-versus-C margin turns on.
- The dependency phase's shareable part,
  which nothing kept sizes.
- Anything on this host's ZFS pool:
  no command here touched it,
  so the pool's compression,
  its snapshot policy,
  and the wear the new NVMe takes from incremental writes are all unknown.

## What this cannot touch

Whichever form the owner picks,
the recorded pressure of 2026-11-27 is only partly inside its reach:

- the guests' builds are release builds inside the guests,
  under a unit capped at three quarters of the machine's memory;
- the builds' host share is what these options lower,
  and only the dev and test part of it;
- the guests' unit's 50 GB of swap was held whether a guest was in use or not,
  and no profile setting changes that.

## What the gate read found, and what changed

The read's four High,
ten Medium and seven Low findings,
and where each landed in this revision:

- H1,
  the memory claim made on rebuilds while the recorded failure was simultaneous first builds:
  answered by splitting the memory section by phase and by reading,
  withdrawing "about half again",
  and replacing the machine-total arithmetic with the 15.4 GiB the guests leave.
- H2,
  "one compiler of 28.8 GiB":
  the record's own sentence is now quoted,
  the sentence says `rustc` and not one compiler,
  "unexplained" is gone,
  and the concurrency question is put against the owner's recorded answers.
- H3,
  the option ranked first was not measured:
  the arms' variable is named,
  the unit graph shows what each form resolves to,
  C,
  D and D2 are marked as inferred rather than measured,
  and the combinations that cannot be built are listed.
- H4,
  the disk claim had no count:
  the live worktrees are counted with the command and the time,
  the trees that outlive them and the missing pruner are named,
  the real tree sizes are given,
  and the model the earlier version used is corrected.
- M1,
  wall kept apart from processor time:
  every cell now gives both,
  with its controls' spread.
- M2,
  what one treatment run supports:
  each cell gives all three readings and the headroom a second run would have to overturn.
- M3,
  the moving `codegen-units` default:
  quoted from both references fetched today,
  and put to the owner in question 3.
- M4,
  what sharing dependency builds can share:
  cut to registry crates with the reason,
  and the dependency phase's own peak named.
- M5,
  each option's mechanism:
  every option now names its line,
  its file,
  the folder that fills,
  the growth,
  the cleanup and the loss.
- M6,
  complements offered as alternatives:
  E and F are out of the single order,
  and the missing fourth form is option D2.
- M7,
  the ranking's reasons:
  each reason is in delivered terms,
  the target workload is stated,
  B over C's cost is corrected (647 MiB of this crate's data,
  not 161),
  and the inference that C repeats the build script's run is withdrawn,
  because the probe's graph shows the run unit is not doubled.
- M8,
  the questions:
  each is restated in full,
  with the contexts the records hold,
  and with what a yes leaves undecided.
- M9,
  the guests' `--release` builds:
  stated under "What this cannot touch" and "What was not measured".
- M10,
  writes:
  stated under "What was not measured".
- L1,
  unit slips:
  MiB and GiB are now kept apart (7,429 to 7,488 MiB is 7.25 to 7.31 GiB,
  4,969 MiB is 4.85 GiB),
  the 1,824 MiB split names its 47 MiB remainder,
  and the tree model's arithmetic is corrected.
- L2,
  "eight cores":
  this machine has 32 logical processors (`nproc --all`) on a 16-core Ryzen 9 7950X,
  and the arms read 6 because of the 600% quota;
  "two to four builds at once" is replaced by the counts the new share gives.
- L3,
  absolute times as findings:
  every comparative claim is now a ratio to its own controls,
  and absolute seconds appear only as context beside them.
- L4,
  the first-build slowdown:
  stated as this crate's own compile (99.1 s against 57.6 s and 66.4 s) and as 1.22 of wall,
  with the unit-count change named as a possible part of it.
- L5,
  probe evidence and the 572 MiB:
  the probe's six forms are this round's own run and are kept with it,
  and the dependency-units figure is reported with its own two components (564 MiB).
- L6,
  "five of six runs":
  corrected to six of seven,
  because the page's seventh library control (2,830 MiB) is also inside 2,798 to 2,855 MiB.
- L7,
  file ownership:
  every option names `package/git-policy/cli/Cargo.toml`,
  and `file-enforcer.config.jsonc` owns that manifest's release keys and its two build-override keys,
  not `[profile.dev] incremental`,
  so no generated file is edited.
