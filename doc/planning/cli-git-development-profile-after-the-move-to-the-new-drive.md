# The Git wrapper's development profile after the move to the new drive

Proposal,
2026-11-28T05:10:00-05:00.
Status:
unapproved and not final.
Nothing here is decided,
no setting was changed on the branch that carries this file,
and the owner declares completion.
The measurements behind every number are in
`package/git-policy/cli/doc/development-profile-measurements.md`,
under its heading dated 2026-11-28,
which names the command behind each figure.
The setting it re-measures is in `package/git-policy/cli/Cargo.toml`:

```toml
[profile.dev]
debug = "line-tables-only"
incremental = false
```

That choice was made on 2026-11-13,
when a build tree was what limited how many agents worked on this package at once.
Three things have changed since:
the home moved to the 3.5 TB drive on 2026-11-17
(`doc/handover/agents-at-once-on-this-machine.md`,
"The cap now"),
the crate reached 1,298,623 lines in 4,012 files below `src/native`
(680,703 of those lines in 1,565 `*_tests.rs` files),
and memory became the limit:
one compiler of this crate's test build was seen at 28.8 GiB resident on 2026-11-27,
and the machine ran out of memory with twenty-three rounds starting at once
(`doc/handover/cli-git-rust-implementation.md`,
the paragraphs naming that night;
`doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md`).

## What a round pays today, measured

A round's loop is edits and rebuilds,
so the rebuild is the number that matters:

- `cargo build --lib` after a one-line edit,
  the setting as it is:
  26.1 to 37.2 s of wall,
  42.2 to 45.4 s of processor time,
  and the compiler's own peak 2812 to 3275 MiB
  (five of six runs of that unit sat between 2798 and 2855 MiB).
- `cargo test --lib --features test-only-switches --no-run` after a one-line edit,
  the setting as it is:
  62.2 to 66.6 s of wall,
  119.3 to 125.0 s of processor time,
  and the compiler's peak 7348 to 7499 MiB.
- The same test build with incremental compilation on:
  32.3 s of wall,
  31.6 s of processor time,
  and 4969 MiB of peak.
- Disk is no longer the limit:
  one worktree's tree is about 1.14 GiB as the setting is,
  and `df -h /home` read 1.4 TB free at 2026-11-28T05:04:56-05:00.

So the machine's per-round ceiling is the test unit's compiler,
7.3 to 7.5 GiB as the setting is,
and a rebuild reaches that ceiling exactly as a first build does.

## The options

Each option says what it gives a round that edits one file and rebuilds,
what it costs in peak memory and on disk,
and what it looks like for one concrete edit:
a line added to `src/native/candidate/candidate_store_commit_listings.rs`,
a file no other file of the crate uses,
followed by `cargo test --lib --features test-only-switches --no-run`
and then `cargo build --lib`.

### A. The setting as it is

`debug = "line-tables-only"`,
`incremental = false`.

What it gives:
the smallest first-build trees measured (787 MiB for the library build,
895 MiB for the test build),
the shortest first test build of the two settings (125.5 and 145.2 s of wall,
481.9 and 547.1 s of processor time),
and file and line in every backtrace frame.

What it costs:
every edit compiles the whole crate again,
so an edit to a file nothing uses costs what an edit to the most-used file costs
(37.2 and 26.1 s against 22.2 and 25.3 s of wall for the library build;
66.6 and 62.2 s against 72.3 s for the test build),
and the test unit's compiler peaks at 7.3 to 7.5 GiB every time.

The concrete edit:
wall 62.2 to 66.6 s and 119.3 to 125.0 s of processor time for the test build,
7348 to 7499 MiB of compiler peak,
then 26.1 to 37.2 s and 42.2 to 45.4 s of processor time for the library build,
2812 to 3275 MiB of peak.
The tree ends at 895 MiB.

### B. Incremental compilation on for everything

`[profile.dev] incremental = true`,
which also reaches the test profile,
since the test profile inherits the dev one.

What it gives:
the rebuild after an edit costs
0.26 times the processor time and 0.50 of the wall for the test build,
0.41 times the processor time for the library build,
and the compiler's peak falls to 0.67 of the control on both rebuilds
(4969 MiB instead of 7348 to 7499 MiB for the test build;
2083 MiB instead of 2812 to 3275 MiB for the library build).
That peak is what sets how many rounds the machine carries at once,
so this option raises that number by about half again:
the 63,152 MiB of memory (`free -m`) holds about eight compilers at 7.4 GiB
and about twelve at 4.9 GiB.

What it costs:
a first build of the test unit is slower,
99.2 s of its own compile against 57.6 and 66.3 s,
because the incremental data is written then;
and every live tree holds it:
1824 MiB of the test tree's 2705 MiB and 808 MiB of the library tree's 1551 MiB,
so a round's one tree (both commands in one directory,
dependencies counted once)
is about 3.5 GiB instead of about 1.14 GiB.
161 MiB of that is incremental data for the repository's own path dependencies,
which is the part the owner's constraint below addresses.

The concrete edit:
wall 32.3 s and 31.6 s of processor time for the test build,
4969 MiB of peak,
then 37.4 s of wall (7.45 s of it Cargo's package cache lock) and 17.9 s of processor time
for the library build,
2083 MiB of peak.
The tree ends at 2658 MiB,
1777 MiB of it incremental data.

### C. Incremental compilation for the library's test build alone

`[profile.test] incremental = true` beside `[profile.dev] incremental = false`.

Cargo can say this,
and the round proved it on a throwaway crate in the round's own folder:
with those two lines,
`cargo build --lib` left the `build/debug/incremental` directory empty,
and `cargo test --lib --no-run` left one real entry in it.

What it gives:
the loop's dominant build gets option B's numbers
(0.26 processor time,
0.50 wall,
4969 MiB of peak),
while `cargo build --lib` keeps the setting as it is:
787 MiB trees,
the fastest first build,
and no 2083 MiB peak on that build.

What it costs:
1824 MiB of incremental data on the test tree (161 MiB of it the path dependencies'),
and nothing for the library build.
A round's one tree is about 2.8 GiB instead of about 1.1 GiB.
The library loop keeps paying a whole-crate rebuild after every edit.

The concrete edit:
wall 32.3 s and 31.6 s of processor time for the test build,
4969 MiB of peak,
then 26.1 to 37.2 s and 42.2 to 45.4 s of processor time for the library build,
2812 to 3275 MiB of peak.

### D. Incremental compilation for this crate's own units alone

`[profile.dev.package.monochromatic-git-policy-cli] incremental = true`
beside `[profile.dev] incremental = false`.

Cargo accepts this key in a package override and the test profile inherits it;
the round proved both on the same throwaway crate
(the build with the override left one incremental entry,
and so did the test build,
with the override written under `[profile.dev.package]`).

What it gives:
option B's loop on both commands,
and dependencies stay cold,
so the repository's 161 MiB of path-dependency incremental data is not written
and the dependencies' first build is unchanged.
This is the form that meets the owner's constraint of 2026-11-04 exactly
(`doc/planning/lean-build-dirs.md`,
"Owner requirements that bound this":
"incremental off for dependencies - no messing with these").

What it costs:
option B's numbers minus 161 MiB:
about 3.3 GiB per round's tree,
with 647 MiB for the library unit and 1616 MiB for the test unit.

The concrete edit:
the same numbers as option B,
with 161 MiB less incremental data in the tree.

### E. Dependency builds shared across worktrees

Nothing of a build is shared today:
`cargo-shadow` keys `CARGO_BUILD_BUILD_DIR` by SHA-256 of the canonical workspace root,
so each worktree has its own tree,
and a second worktree's first build compiles
266 to 271 dependency units in 64.3 to 78.8 s
before it compiles this crate's own unit.

What it gives:
572 MiB of disk and that dependency phase per worktree's first build,
and nothing else:
no rebuild benefit (a rebuild compiles only this crate's unit),
and no benefit to the peak that limits rounds,
because that peak is this crate's unit,
not a dependency's
(the largest dependency compiler peaked at 466 MiB,
with six running at once).

What it costs:
a mechanism nothing owns today.
Cargo keys its build directory by workspace path and offers no per-dependency directory,
so it needs either a build directory shared with a lock that serializes worktrees
(two builds in one directory serialize on Cargo's build directory lock),
or a compiler cache such as `sccache`,
which the 2026-11-25 section already recorded as refusing the unit that sets the peak.

The concrete edit:
58 to 66 s instead of 125 to 145 s for the first test build of a fresh worktree,
and the same 62 to 67 s as before for the rebuild after the edit.

### F. Incremental compilation during a round's loop, removed when the round ends

No profile change:
the round's own tasks pass `CARGO_INCREMENTAL=1` for the loop,
and the tree is removed,
or its incremental directories are,
when the round ends.
`cargo-shadow lean` already removes a tree whose workspace is gone,
so a dead worktree's tree does not stay behind.

What it gives:
option B's loop while a round works,
and the disk back once it ends.

What it costs:
a step in every round's task set and a bound on how long a tree may live,
and a full first build for the next round in that worktree.
Whether the removal happens is a matter of the round's own discipline,
which is the property the setting as it is does not need.

The concrete edit:
option B's numbers,
with the tree's 2658 MiB deleted when the round ends.

## Ranking

Ranking:
D > B > C > F > A > E.

- D over B:
  both deliver the same loop,
  the test build at 0.26 of its processor time and 0.50 of its wall,
  the library build at 0.41,
  and a rebuild's compiler peak at 0.67 of what it is.
  B pays for that with 161 MiB of path-dependency incremental data no rebuild reads,
  and with the owner's constraint of 2026-11-04 on dependencies' incremental data,
  which D meets and B breaks.
- B over C:
  C improves only the test build,
  and the library build is what a round runs before its consumer checks,
  so C leaves rebuilds at 0.41-of-the-time unclaimed.
  What B costs over C is that 161 MiB,
  which a round's tree carries once and a 1.4 TB drive makes cheap.
- C over F:
  F delivers the same loop while a round keeps its own rule,
  and it is a per-round rule where C is one line in the manifest;
  a round that forgets the rule holds the same incremental data
  with nothing in the manifest that explains it.
  F is the option to add beside a profile setting,
  never instead of one.
- F over A:
  A delivers no loop gain at all,
  and F delivers all of B's while a round works.
  A needs nothing,
  which is why it outranks E but not F:
  a task's environment is one line,
  and the disk bound F would protect is not the bound this machine meets.
- A over E:
  A changes nothing,
  and E needs a mechanism nothing owns,
  while E buys only a worktree's first build
  and nothing of the loop or of the peak that limits rounds.

What the ranking leaves open is the peak-cap question the 2026-11-25 section left unset:
a cap on concurrent rounds read from 7.4 GiB is wrong by 2.5 GiB per round
once incremental data is on,
and the choice of cap belongs with the choice above.

## What this machine carries at once, in these numbers

- Memory:
  63,152 MiB of RAM (`free -m`) against 7.3 to 7.5 GiB per concurrent test build as the setting is,
  so about eight such compilers fit;
  at 4.9 GiB,
  about twelve.
  The 2026-11-27 night saw one compiler at 28.8 GiB,
  which is not this crate's test unit as measured here (its peak was 7.3 to 7.5 GiB)
  and is worth identifying before the number is used for a cap.
- Processor:
  each build asked for 600% of the eight cores;
  the first builds used 400 to 547 s of processor time over 89 to 166 s of wall,
  so two to four builds make progress at once
  and the rest wait on the host,
  as this round's own 42% and 188% control spreads show.
- Disk:
  1.4 TB free,
  about 1,200 worktree trees as the setting is and about 400 with option B,
  which no round count reaches today.

## Questions for the owner

1. Which option does the profile take:
   D (this crate's own units),
   C (the test profile alone),
   B (everything),
   or A (keep the setting as it is)?
   The measurements make D the recommendation and A the safe answer if the drive's
   space is wanted for something else.
2. Is the extra tree space acceptable:
   about 2.2 GiB more per live worktree with D,
   1.7 GiB with C?
3. Do the constraints of 2026-11-04 still stand as written
   (`doc/planning/lean-build-dirs.md`:
   "debuginfo level,
   incremental off for dependencies - no messing with these",
   and split-debuginfo,
   strip and codegen-units off-limits)?
   They bar B's path-dependency data and they bar `debug = false`,
   which the 2026-11-13 round recorded as rejected for its backtraces.
   Under them,
   D is the only form of B that fits.
4. Should a round's tasks pass `CARGO_INCREMENTAL` themselves (option F)
   instead of,
   or beside,
   the profile?
5. Should dependency builds be shared across worktrees at all (option E)?
   It needs a mechanism nothing owns,
   and it saves only a worktree's first build.
6. What cap on concurrent rounds should follow from a per-round peak of
   7.4 GiB or 4.9 GiB?
   The 2026-11-27 night's 28.8 GiB compiler is unexplained by this crate's own builds
   and should be identified before any cap is set from it.
7. Should `debug = false` be reconsidered now that memory,
   not disk,
   is the limit?
   The 2026-11-13 rejection was about backtraces,
   not about memory.

## What was not measured

- The `--all-targets` and `native_binary` units,
  which are larger than the library test
  build measured here,
  and whose peaks are therefore higher than 7.4 GiB.
- `debug = false`,
  `-j`,
  `codegen-units` and `split-debuginfo`:
  the 2026-11-13 and 2026-11-25 numbers above stand for them.
- A second worktree's first build at its own path,
  which would need another worktree of the repository;
  what it pays is argued from the build directory's key and from the first builds measured.
- Clippy and `cargo check`,
  which the 2026-11-25 section covers at 4.0 to 4.3 GiB of peak.
