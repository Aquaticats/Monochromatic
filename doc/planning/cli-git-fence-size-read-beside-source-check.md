# The conversion fence: running the size read beside the source check

**Unapproved and non-final.**
The project accepted the shape below on 2026-11-22
(`doc/decision/cli-git-fence-size-read-keeps-git-ls-tree.md`,
its section "The overlap,
answered 2026-11-22"),
and the build this branch carries is that shape;
only the owner declares the work done.

The first round measured one overlap and proposed what to do with it.
It changes no shipped behaviour,
and its prototype is committed with "PROTOTYPE FOR MEASUREMENT,
NOT FOR MERGING" in its subject.
Every number in "Rung 1,
the btrfs snapshot rung",
"Rung 2,
the clone rung on ZFS" and "What this round does not do" is that round's runs on this machine on 2026-11-21 and 2026-11-22.
This branch then ports the idea into its production shape and times it in one btrfs guest of its own,
and those runs are in "The shipped build,
timed" below.

## What the project asked

`doc/decision/cli-git-fence-size-read-keeps-git-ls-tree.md` answered "A:
measure it" on 2026-11-21:
keep one `git ls-tree` as the fence's size read,
and measure running that read at the same time as the source check,
so the read stops adding to the add's wait.
This round does that on rung 1 (the btrfs snapshot) and on rung 2 (the clone rung),
on loose and on packed sources,
with an identical-binary control and a position-balanced order.

## Where the read runs today, per rung

The fence's read is one `git ls-tree -r -l -z <commit>` process,
and `worktree_clone_conversion_fence::blob_sizes` is its only caller
(`package/git-policy/cli/src/native/worktree/worktree_clone_conversion_fence.rs`).

On **rung 1**,
`worktree_ladder_run.rs`,
`choose_rung`,
runs the add's steps in this order:

1.  `refusals`:
    whether a snapshot can apply at all and whether clones can,
    before any read of the source.
2.  `checkSource`:
    the joined configuration read,
    the trash-ignore read and the cleanliness check,
    which is where `source.clean.head_commit`,
    the index bytes and the walk of the tracked files come from.
3.  `conversionFence`,
    which is `fence_the_clone`:
    `fence_the_attributes` first (`git check-attr` and the decline count),
    then `decide_the_paths`,
    where `blob_sizes` runs.
4.  `targetTreeDifference`,
    `claimStage`,
    then the snapshot.

So on `main` rung 1's read is on the add's waiting path,
after the check and after the attribute read,
and the snapshot's constant-time copy leaves nothing for it to overlap.
This branch moves that one read,
and only in this rung:
it starts between step 1 and step 2,
from what the ref files answer in process,
and it is joined inside step 3 before `decide_the_paths` asks for the map
("What shipped" below).
The attribute read stays where it was,
and rung 2 keeps the position above.

On **rung 2**,
the same `choose_rung` calls `the_fence_of_this_build`,
which reads the attributes and decides the decline only;
the rest of the fence travels in the rung and runs in
`worktree_ladder_finish_clones::clone_with_the_fence`'s `conversionBesideTheClone` step,
where `fence_beside_the_clone` runs `decide_the_paths` on the add's own thread
while the tracked files are cloned.
The read there is already off the waiting path:
its cost is the larger of the read and the copies,
not their sum.

## What the read needs, and what the check proves

The read needs three things:
the source's worktree root,
the commit whose recorded sizes it lists,
and the repository's object format to spell that commit.
It reads committed objects through one `git ls-tree` of that commit,
and on a partial clone that run would fetch every blob it is missing
to learn its size (finding High 1 of the gate review of 2026-11-22,
and "The shipped shape" below says what the build does about it).

The check produces the commit the fence may use.
On a clean source that commit is the tree of the index and of every tracked file,
proven by the clean check.
On a main checkout with only ordinary uncommitted changes the check keeps the source
(`source_or_dirty`,
`SPEC.md` "Snapshot of a main checkout with uncommitted changes"),
snapshots it and restores the copy to the new worktree's `HEAD`,
and there the index is not that commit's tree:
`source.clean.head_commit` is then only the commit `read_state` read.
The guard is right in both cases for the same reason the overlap needs it:
the map is a function of the commit and of nothing else,
so the fence compares two commits and never rests on the proof of a tree.
The read may start beside the check,
and its answer may be used only while the commit it listed is `source.clean.head_commit`.
That is the whole of what the overlap needs:

- a commit resolved before the check (`worktree_clean::head_commit`,
  over the plain ref files,
  with `repository_hash_length` for the format) and a read started on it;
- a thread or a runtime join,
  so the child runs beside the check rather than after it;
- an equality guard,
  which drops the map and lets the fence read it again when the two commits differ;
- the read's own failure still ending the rung where it ends today,
  because the map is the proof and "could not read" must never become "carry everything".

Nothing the read touches is unproven with one exception the gate review measured:
the objects a commit fixes are the same before and after the check,
and the only thing the check adds is which commit is the fence's,
which the guard enforces,
but a read of a partial clone fetches what it is missing into the source,
which is a write.

## What the overlap costs a refused or a declined add

**When the check refuses**,
the ladder falls to Git's own checkout before any fence runs,
so on `main` the read never starts.
With the overlap the read starts anyway and its answer is never taken,
and the add ends the read's Git child where it learns it will not use it (`abandon`).
The abandon often comes before the read's thread has reached its spawn:
the same recheck measured `checkSource` at 0.009 to 0.017 ms on a source that refuses at once,
so the cell takes `ABANDONED_CHILD` and the runner ends the child the moment it publishes it.
An add that refuses at once therefore pays the read's start and about a millisecond of a child,
not its whole length beside Git's checkout.

**When the fence declines**,
the read ran beside `git check-attr` for its whole length
(66 to 68 ms of work in the integration recheck's declined rows),
and the answered map of 24,634 entries was built by the read thread and freed on the adding thread,
which the recheck measured at 1.8 to 3.2 ms in a control and
4 to 5 ms in the declined add's `chooseRung`.
This round has the read hand its `ls-tree` output over unparsed,
so a declined add drops one buffer (about 0.3 ms) and the map is never built for it,
and the fence parses the output only when it uses it;
"The declined add's step cost,
before and after" below states the split.

The same holds for the fence's own cheap refusal (an untracked `.gitattributes`),
for a source the decline rule refuses,
and for a fence with no path to size or whose read listed another commit,
all of which end the read at the fence.

**A partial clone is where that cost could have been a write.**
`git ls-tree -r -l` needs every blob's size,
and on a partial clone a missing blob is fetched from the promisor remote to answer it.
The gate review measured it on a sparse partial clone of a local `file://` origin,
each plain `git` run with its own `HOME`
(its `keep/lazy-fetch-probe-output.json` and `keep/standalone-read-probe-output.json`):

- before any add:
  100 blobs missing,
  3 packs;
- the prototype's add with the switch unset:
  refused ("the source has a sparse checkout"),
  route `Git's checkout`,
  still 100 missing and 3 packs;
- the same add with the switch set:
  the same route and sentence,
  now 98 missing and 5 packs,
  in two separate runs;
- the read alone with lazy fetch left on,
  in a fresh sparse partial clone:
  100 missing and 2 packs before,
  0 missing and 102 packs after,
  one pack per blob.

So on such a source the read writes,
and the sentence "nothing is written and no state is left" was false for the prototype.
`GIT_NO_LAZY_FETCH=1` on every early read is the condition,
and this branch sets it on every read with no test of the source:
the recheck made a read fetch where the old test missed a promisor remote
(a remote whose flags arrive through an included file,
a worktree config or `GIT_CONFIG_COUNT`,
and a `--reference` clone whose flags are in an include),
so a test of the source is not a condition the decision can keep.
There is no second switch:
the global `--no-lazy-fetch` is Git 2.45.0's,
and a Git before that refuses it before reading anything (exit 129),
which sent a plain source from the snapshot to Git's whole checkout (865.3 ms against 3509.2 on the recheck's tree).
The variable is honoured by the May 2024 maintenance releases
(`v2.39.4`,
`v2.40.2`,
`v2.41.1`,
`v2.42.2`,
`v2.43.4`,
`v2.44.1`;
the commit `7b70e9efb`,
the fix for CVE-2024-32465),
by `v2.45.0` and later
(which set `fetch_if_missing = 0`,
and from `v2.45.1` also read it in `promisor-remote.c`),
and by Ubuntu 24.04's `2.43.0-1ubuntu7.1` and later,
which backport the fix.
On a missing blob the maintenance releases and Ubuntu's package refuse the lazy fetch
and the read exits 128
("lazy fetching disabled ... could not fetch ... from promisor remote"),
so the rung declines to Git's checkout;
`v2.45.0` and later print `BAD` in the size column,
the run exits 0,
and the clone is unchanged.
Probed with Git 2.56.1 on this host,
`git ls-tree -r -l -z HEAD` under the variable exits 0 with the same output size,
prints `BAD` in the size column of a blob the clone does not hold,
and leaves 40 missing and 2 packs unchanged;
without the variable the same read fetches (0 missing,
42 packs).
A Git older than those releases that does not carry the fix ignores the variable and can fetch,
and its fetch child can outlive the command:
that is the recorded limit of this read on such a Git,
which is older than the wrapper's tested minimum
(`MINIMUM_GIT_VERSION`,
2.56.0).
`parse_sizes` skips a size it cannot parse,
and a missing size only ever sends a path to Git's writing,
never to a carry.
The control this branch ships,
a sparse partial clone the check refuses,
counts the missing objects and the packs before and after an add and finds them equal,
with the same read without the variable as its positive control.

**When the process is killed** during the overlap,
the difference from today is only that a read-only Git child may be running:
no lock is taken and no index is written,
and both rungs claim their stage name after the fence,
so a kill in the overlap window leaves the state a kill inside `checkSource` leaves today,
with `GIT_NO_LAZY_FETCH=1` making sure no fetch chain outlives it on a Git that honours the variable
(a Git older than the May 2024 maintenance releases can still fetch,
as above).
Rung 2's rule is unchanged:
the worktree holds no index until Git's write is done,
and the read adds nothing to that window.

A replacement ref created inside the window is the other thing the read would see and the commit would not,
because the read honours `refs/replace` as Git does.
The window is the check's own duration (about 150 ms loose,
from the step records below),
and the fence's guard still compares commits;
the prototype's one probe of a replacement made the check itself refuse the source
("the object database has a layout the reader does not know") before any fence.
The replace refs of a reftable store are open in the decision's "Left open" section,
and this branch does not touch them.

## The prototype that was measured

`package/git-policy/cli/src/native/worktree/worktree_clone_conversion_size_read_beside_the_check.rs`,
reachable only from the `measurement-routes` feature:

- `start_beside_the_check` runs in `choose_rung` just before `checkSource`,
  resolves `HEAD` in process and starts `blob_sizes` on a thread of its own,
  noting its time as the work `fenceSizesBesideTheCheck`;
- `decide_the_paths` goes through `sizes_of_the_paths`,
  which takes a started answer when its commit is the fence's and otherwise reads the map itself;
- the wait for a started read is the step `fenceSizesWaitingForTheRead`,
  so every arm's record says how much of the read the check and the attribute read hid.

The switch is `MONOCHROMATIC_CLONE_SIZE_READ_BESIDE=1`,
read only by a measurement build;
a shipped build compiles neither the module nor the switch.
The prototype keeps its answer in a process-wide slot,
which is a prototype's shortcut and not a shape to ship,
and its thread's join has no deadline:
a thread that died left the add waiting.

The prototype's commit message spells the switch `MONOCROMATIC_CLONE_SIZE_READ_BESIDE`
(a misspelled `MONOCHROMATIC`).
The code reads the correct name,
the commit was not amended,
and the message is wrong as committed.

## What shipped

`package/git-policy/cli/src/native/worktree/worktree_clone_conversion_size_read.rs`,
in every build of this branch:

- `start_beside_the_check` is called in `choose_rung` between the two cheap refusals and `checkSource`,
  when and only when `snapshot_refusal` is `None`
  (finding Low 3's predicate,
  so rung 2 and an add no rung serves never start it).
  It resolves `HEAD` in process (`worktree_clean::head_commit`,
  with `repository_hash_length` for the object format) and starts the read on a thread of its own.
  One fall-through pays it for nothing:
  a linked-worktree source whose snapshot is blocked after the check (a nested ordinary worktree)
  takes rung 2 with the read already started,
  and that add's end settles it unused.
  Rung 2's own fence is unchanged and never starts it.
- The read is one `git ls-tree -r -l -z <commit>` with `GIT_NO_LAZY_FETCH=1` in its environment on every source,
  and `GitRequest::beside_another_step` set,
  so it cannot fetch into the source on a Git that honours the variable
  and it neither clears nor writes the process-wide failed-step note
  (`git_step_failure::LAST`,
  finding Low 2).
  The global `--no-lazy-fetch` is not used:
  it is Git 2.45.0's,
  and an older Git refuses it (exit 129),
  which sent a plain source from the snapshot to Git's whole checkout.
- The started read is a value the add owns:
  a `Ladder` field,
  passed into the snapshot rung's fence as `&mut Option<SizeReadBesideTheCheck>`.
  There is no process-wide slot,
  so no command's read or failure can reach another command or another record.
- The fence's sizes step is `sizes_of_the_paths`:
  it takes the started answer when its commit equals `source.clean.head_commit`,
  and otherwise ends the read and reads that commit with the fence's own `git ls-tree`.
  A fence with no path to size ends the read too,
  so a read nobody will use is not left running to the process's end.
- The join is the fence's own `sizes_of_the_paths` when it took the read,
  under one deadline of the plumbing step's ceiling plus the wait the runner gives an ended child
  (`PLUMBING_STEP_CEILING + SHORT_EXTRA_WAIT`,
  127 s);
  an add that ends before the fence states the read in its record and never waits (`Ladder::finish`'s `settle`).
  A read that sends nothing by then,
  or whose thread died,
  refuses the rung with the sentence a failed run of the fence gives
  (`a Git run of the fence could not start (...)`);
  it never holds the add.

**Why a thread and not a task of the runtime the check uses.**
The read is a Git run,
and every Git run of this crate registers its child's pipes with the process's runtime
(`step_output_read_whole_through_pipes::attach_pipes`,
which refuses `RuntimeError::Nested` from inside it).
A task of that runtime is inside it,
and a job of its blocking pool is too,
because the pool's threads carry the runtime's context.
This branch was built the blocking-pool way first,
and four controls failed with exactly that refusal
("the runtime that runs this command's waits was asked for from inside itself").
A plain thread has no runtime context,
so `spawn_named` (`foundation/scoped_start.rs`) is where the read runs,
and a system that refuses the thread loses only the overlap:
the fence reads where it stands.
The thread is named `clone-size-read-beside-the-check`.

**The abandon ends the child wherever it comes.**
`abandon` takes the read's answer if it already came
and states it as the record's work,
otherwise it states `started and not joined`;
then it swaps the published-child cell for `ABANDONED_CHILD` (`-1`).
A child already published is signalled there;
a child not yet published is ended by the runner the moment it is published,
so a check that refuses at once (before the read's thread reached its spawn)
no longer leaves the child to run its whole length beside Git's checkout.
The runner clears the cell within the time it takes to finish reading the child's pipes,
before anything is parsed,
so the parse window the first recheck found is closed;
between the kernel's reap and that clear,
and between the abandon's swap and its signal,
an abandon still signals an id no process has,
which is the integration recheck's Low 3 of 2026-11-22,
and that signal reaches nothing unless the id is reused in that window.

Rung 2 is untouched,
and the two harnesses that proved rung 2's shape still pass against this build:

- the class harness's twenty-six scenarios:
  **26 EQUAL**,
  23 served by the clone rung,
  and `m05` ended at the tree difference
  (`keep/harness/class-records-fence-size-read-shape-559a50966db8.json`);
- the 150-combination sample,
  seed 11,
  150 of 1,440 combinations:
  **300 repositories,
  300 EQUAL,
  300 served by the clone rung**
  (`keep/harness/matrix-fence-size-read-shape-559a50966db8-seed11-n150.json`).

## The controls

All of them are in the default build,
in `worktree_clone_conversion_size_read_tests.rs`.
Every control this round added or changed was shown red once with the guard it tests removed and then restored
(`AGENTS.md`,
rule `GFP`);
the first round's own red runs are in its section,
and the recheck showed that the fetch control stayed green with both switches removed,
because it counted after a settle that no longer waited,
which is why this round joins the read before it counts.
The red runs of the controls this round changed:

- the fetch control,
  with `GIT_NO_LAZY_FETCH` dropped from the request:
  `left == right` failed on "the read fetched nothing into the source the check refused",
  and the three shapes the recheck made fetch failed on their own assertions;
- the five stale-read controls,
  with the guard's commit comparison inverted:
  all five failed on "the fence ended the read of the earlier commit where it decided not to use it",
  because the fence left the read in the add's hand;
- the deadline control,
  with the deadline replaced by a plain receive:
  the held read answered after 30 s and the fence decided,
  so the refusal the control expects never came;
- the settle control,
  with `Ladder::finish` no longer settling:
  the record was `{"steps":{},"work":{}}`;
- the note control,
  with `beside_another_step` dropped:
  the note the control set was gone when it looked.
They call the same functions the ladder calls,
because on this host the ladder cannot take rung 1 at all:
`snapshot_refusal` needs a btrfs subvolume root,
and rung 1's own source is the guest's,
so the start is exercised where the ladder makes it,
not through the whole add.

**A sparse partial clone the check refuses fetches nothing.**
Two clones of a local `file://` origin (`uploadpack.allowfilter` set on it),
each with `--sparse --filter=blob:none` and the cone on `dir0`,
so every blob of `dir1` is missing.
The control counts the objects `git rev-list --objects --all --missing=print` reports missing
and the `*.pack` files,
before and after:
the arm under test starts the read as `choose_rung` does,
runs the check,
which refuses ("the source has a sparse checkout"),
**joins** the read
(the old control settled without waiting,
so a fetch could land after it looked),
and finds both counts unchanged.
The positive control is the same read without `GIT_NO_LAZY_FETCH` on a second clone,
which fetches and moves both counts,
so the pair is one that can move at all.

**The shapes the recheck made fetch fetch nothing either.**
Three more controls build the same sparse partial clone and move the promisor remote's flags to a place the old
test of the source missed,
with no `.promisor` marker in the pack directory:

- the flags in `remote-flags.cfg`,
  which the common config includes,
  and no word `promisor` in the common config;
- the remote set only through `GIT_CONFIG_COUNT` and its `GIT_CONFIG_KEY_n`/`GIT_CONFIG_VALUE_n` pairs,
  with the clone's own promisor settings unset;
- a `--reference` clone of a partial clone (both sparse and filtered),
  with its flags in an include and its own pack directory empty.

Each starts the read,
joins it,
and finds the missing objects and the packs unchanged;
probed on Git 2.56.1 with the same commands,
the three shapes fetch without the variable
(missing to zero,
packs to three to seven)
and fetch nothing with it.

**The abandon ends the child wherever it comes.**
- A read held before its Git run,
  abandoned,
  and then released:
  the child it starts is ended at publication,
  so the fake Git's 30 s sleep is not run to its end.
- A read whose fake Git ends at once,
  joined and held after its wait:
  the published-child cell is zero,
  so an abandon there signals no reaped process id.
- A fence with no path to size,
  holding a reading fake Git:
  the fence ends the child where it decides it has no use for it.
- A read the recheck abandoned before its child existed:
  the child ends at publication.
- A refused thread start:
  no read starts,
  the fence reads where it stands,
  and the record has a `git ls-tree` step and no wait.
- The record a killed read leaves:
  the child start is a line in the repository's per-process record,
  and the sweep every command runs removes it once every holder it names is gone.

**A `HEAD` that moves drops the stale answer.**
Five shapes,
one control each:
a symbolic ref moved to another commit of the same tree,
a branch that lives only in `packed-refs` before a loose ref appears over it,
a `HEAD` that is detached before it moves,
`HEAD` moving to a commit of another tree,
and a linked worktree's own `HEAD` moving.
Each puts its reference in that shape first,
then starts the read on the commit before the move,
moves `HEAD`,
runs the check on the source it then finds,
and fences:
the record states a `git ls-tree` step of the add and no wait step of the fence,
the read is still held by the add,
and the fence's answer equals the serial fence's answer for the commit the check proved
(`fence_the_clone`),
which is the set of paths Git writes and the clones carry.
The destination bytes are not compared here:
the measurement round's decision control did that on a settled source (26 of 26 files equal),
and what this pair of answers proves is that the guard decides with the proved commit's map.
The settled `HEAD` is the positive control:
the fence takes the read,
the record states `fenceSizesBesideTheCheck` work and no `git ls-tree` step,
and the answer equals the serial one.
The linked worktree gets a settled control of its own,
with the main checkout's `HEAD` on a commit the linked worktree's is not:
there the fence takes the read,
so a read that had resolved the main checkout's `HEAD` would have been dropped.

**A read that fails or outlives its deadline gives today's sentence.**
- A read that cannot start (a `GitContext` whose real Git is not there)
  answers exactly the string `blob_sizes` answers for the same broken context.
- A read held past a shortened deadline answers `a Git run of the fence could not start (...)`
  with `` `git ls-tree` did not answer within 50ms ``.
- A read whose thread dies answers the same prefix.

**An add that never reaches the fence states the read's cost and writes nothing at the destination.**
A `Ladder` in the shape a check refusal or a decline leaves it
(`Rung::Git`,
the read still held) settles the read in `finish`:
the record states `fenceSizesBesideTheCheck`,
and nothing is written at the destination.
On a partial clone the first control is the one that proves the read fetches nothing either.

**The read takes no other step's note.**
A failed-step note is set by hand,
the read is started and joined,
and the note is still there to be drained:
the read's Git run is `GitRequest::beside_another_step`.

**The early read's request is the only difference.**
Its `GitRequest` carries `GIT_NO_LAZY_FETCH=1`,
is declared `beside_another_step`,
and is the same `ls-tree -r -l -z <commit>` the fence's own read makes;
both answer the same map for the same commit on a settled source.

## The shipped build, timed

One throwaway `libvirt` guest of this branch's own through `mvm`,
`fence-size-read-build-guest`:
Ubuntu 24.04,
kernel 6.8.0-147,
four vCPUs,
7.9 GiB,
a 6 GiB btrfs image mounted as a loop device with the source as a subvolume on it,
and one Python process inside the guest driving every add,
so no host round trip sits between arms.
`sync --file-system` runs before every add.
The guest was destroyed by name when the runs ended.

The fixture is the first round's tree,
`git archive` of this repository at `40783c23ea`
(24,634 tracked paths),
its `filter=lfs` lines removed and its policy file emptied,
committed in the guest as `694c4eb4b7`:
25,034 loose objects (443 MB),
then the same objects packed in place by one `git gc`.

The arms:

- `branch`:
  this branch's release build (`559a50966db8`),
  the shipped shape;
- `main`:
  `main`'s release build at `7ae5d3ffab` (`033be78592c4`),
  the same tree without the overlap;
- `branch-again`:
  `branch`'s own binary and environment again,
  the identical-binary control.

Each layout runs one uncounted warm-up add per arm,
then rounds that rotate the arm list one place per round and reverse it on every second pass,
so each arm takes each of the three slots twice in six rounds.
A timing is read only as a ratio to another arm of the same round,
with the control pair's own range beside it.
Medians are the upper middle element of an even count.
Every add of every arm took `a snapshot`,
and none failed.

### Loose objects

Six rounds,
18 counted adds.

- `main`:
  median **1023.9 ms** (862.2 to 1283.9);
  its `git ls-tree` step is **384.7 ms**.
- `branch`:
  **726.6 ms** (665.4 to 795.8);
  the read is 380.7 ms of work beside the check,
  and the fence waits **138.1 ms** for it.
- `branch-again`:
  **754.5 ms** (678.2 to 762.6);
  the read is 371.4 ms of work,
  the wait 136.7 ms.

Per-round ratios,
`branch-again` to `branch`:
1.00,
1.05,
1.07,
0.95,
0.96,
1.02
(range 0.95 to 1.07).
`main` to `branch`:
1.38,
1.36,
1.54,
1.27,
1.82,
1.29
(range 1.27 to 1.82).
`main` is above `branch` in every round,
and by more than the control's whole range in every round.
The medians differ by **297.3 ms**,
**29 percent** of the add,
and of the read's 380.7 ms,
242.6 ran between its start and the fence's join.

The steps either side stay put:
`checkSource` 112.7 on `main` and 106.0 on `branch`,
`git check-attr` 52.1 and 55.9,
`walkSnapshot` 227.8 and 209.6,
`prune` 51.1 and 45.1.

### Packed objects

Two sets of six rounds,
18 counted adds each,
the second set after the first showed one round of the control pair wider than the effect.

- `main`:
  median **534.6 ms** (525.2 to 538.2);
  its read step is **29.0 ms**.
- `branch`:
  **501.2 ms** (487.8 to 543.4);
  the read is 43.3 ms of work beside the check,
  and the fence waits **0.0 ms**.
- `branch-again`:
  **504.7 ms** (492.7 to 676.6);
  the read is 44.7 ms of work.

Set one,
per-round,
control `branch-again` to `branch`:
1.00,
1.35,
1.03,
1.01,
0.92,
0.99
(range 0.92 to 1.35,
the 1.35 being a round where the control's own `git check-attr` took 119.2 ms and its fence 88.6).
`main` to `branch`:
1.02,
1.07,
1.09,
1.09,
0.99,
1.07.
So five rounds put `main` 7 to 9 percent above `branch` and one puts it 1 percent below,
while the control's median is **+0.7 percent**.
The medians differ by **33.4 ms**,
**6.2 percent** of the add.

Set two,
the same three arms six rounds later:

- `main`:
  median **557.7 ms** (549.3 to 565.3);
  its read step is **31.2 ms**.
- `branch`:
  **523.1 ms** (507.1 to 537.1);
  the read is 45.4 ms of work beside the check,
  the fence waits **0.0 ms**.
- `branch-again`:
  **530.2 ms** (520.4 to 540.7);
  the read is 47.9 ms of work.

Per-round,
control `branch-again` to `branch`:
1.07,
1.01,
1.01,
1.01,
0.97,
1.02
(range 0.97 to 1.07).
`main` to `branch`:
1.09,
1.07,
1.06,
1.08,
1.03,
1.07
(range 1.03 to 1.09).
Every round of set two puts `main` 3 to 9 percent above `branch`,
the medians differ by **34.6 ms (6.2 percent)**,
and the control's median difference is **+1.4 percent**.

Both sets agree on the size and the direction:
**33.4 and 34.6 ms**,
**6.2 percent** of the add in each,
against a control pair whose median difference is **0.7 and 1.4 percent**.
No single round resolves it,
because the control's spread in both sets (0.92 to 1.35 and 0.97 to 1.07)
is as wide as the effect,
and set two's every-round range (1.03 to 1.09) overlaps the control's 1.07.
So what this document claims for the packed layout is the medians of two sets,
and the direction they agree on,
never one round.
It never waits at the fence,
because the read's 43 to 48 ms beside the check end before the fence reaches it.

The read costs more beside the check on a packed source,
as the first round measured:
29.0 ms alone and 43.3 ms beside it,
**49 percent** slower,
which is what the fence's 0.0 ms wait hides.
The saving is then the serial read's 29.0 ms minus that excess plus the fence's own shorter step
(`conversionFence` 35.4 on `main` and 28.1 on `branch`),
which is the 33.4 ms the medians show.

### A refused add and a declined add

Both sets are four rounds of the two arms that differ only by the read's own start:
`branch` and `branch-again`.

**Refused.**
The source gets an empty `.git/info/sparse-checkout`,
which no rung serves,
so every add took `Git's checkout`:

- `branch`:
  median **2237.9 ms** (2074.3 to 2435.7),
  with 40.1 ms of `fenceSizesBesideTheCheck` work and the fence's wait **0.0 ms**;
- `branch-again`:
  **2422.9 ms** (2192.6 to 2545.2),
  46.6 ms of work,
  wait 0.0 ms.

**Declined.**
The source gets a committed `.gitattributes` line `* ident`,
which converts every tracked path without changing a byte,
so the fence's decline rule ends every add before the snapshot,
and every add took `Git's checkout`:

- `branch`:
  median **6709.5 ms** (6485.4 to 6851.4),
  with 62.3 ms of read work and the fence's wait **0.0 ms**;
- `branch-again`:
  **6570.5 ms** (5843.0 to 6819.0),
  70.5 ms of work,
  wait 0.0 ms.

So the price of the overlap on such an add is the read's own work,
40 to 73 ms on this packed source,
paid while Git's own checkout runs:
the settle at the add's end waits 0.002 to 0.007 ms,
because the read finished long before Git did.
The refused add's record states it,
which is what `SPEC.md` now says and what the control
`an_add_that_never_reaches_the_fence_states_the_reads_cost` proves.

## The measurement

### The tree
One fixture of this repository's tree at `40783c23ea`:
24,634 tracked paths,
the `filter=lfs` lines removed from the copied `.gitattributes` so every path takes the size proof,
and the fixture's own policy file listing no policy.
The guest's commit is `c78ddb96b8` and the host's is `fbe4f3fe5c`;
the two differ only in the commit object's time.

### The layouts
All 25,034 objects loose first (541 MB),
then the same objects packed in place by one `git gc` (one pack,
no new commit).

### The arms

- `main`:
  the release binary of `main` at `a2c1c9ccea`,
  no feature and no switch.
- `branch-serial`:
  this branch's measurement build (`measurement-routes` compiled in),
  switch unset.
- `branch-overlap`:
  the same binary with `MONOCHROMATIC_CLONE_SIZE_READ_BESIDE=1`.
- `branch-serial-again`:
  the same binary and environment as `branch-serial`,
  run twice,
  which is the identical-binary control that prices the harness's own spread.

### The order
Each layout runs one uncounted warm-up add per arm,
then rounds that rotate the arm list one place per round and reverse it on every second pass through the list,
so each arm takes each of the four slots once per pass and follows each other arm.
Rung 1 ran 8 rounds on loose and 16 on packed;
rung 2 ran 16 rounds on each layout.

### The hosts

- Rung 1 in one throwaway `libvirt` guest made with `mvm`,
  `fence-size-read-beside-guest`:
  Ubuntu 24.04,
  kernel 6.8.0-147,
  four vCPUs,
  7.9 GiB,
  a 6 GiB btrfs image mounted as a loop device with the source as a subvolume on it,
  and one Python process inside the guest driving every add,
  so no host round trip sits between arms.
  `sync --file-system` runs before every add.
- Rung 2 on this host's ZFS (`zpcachyos/ROOT/cos/home`),
  source and destinations in one dataset,
  driven from the host with the same rotation.
  The host was loaded by other sessions throughout:
  load average 12 to 23 on ten cores,
  which is why every ratio here is quoted with the control pair's own range beside it.

### How a timing is read
Only as a ratio to another arm's add in the same round,
with the identical pair's range beside it,
and the step records as the account of where the time went.
Medians are the upper middle element of an even count.

### The decision control

One small control runs beside the timing sets,
on this host's ZFS,
because the change is where the sizes are read and a map of the wrong commit is what it could break:

- a 26-path fixture whose `.gitattributes` gives `*.txt text eol=lf`,
  so the size proof leaves 4 converting files to Git and carries 22;
- one add through `branch-serial` and one through `branch-overlap`,
  same binary,
  same source,
  same commit;
- both take the `clones` rung,
  both print `the fence carries 22 paths and leaves 4 to Git`,
  both leave `git status --porcelain` empty,
  and the two destinations are byte-for-byte equal over all 26 files.

This shows the two arms decide the same thing on a settled source.
It does not exercise the guard's own case,
a `HEAD` that moves between the read's start and the check,
which this round did not build.
That control is in the shipped build's own suite now,
under "The controls" below,
and the gate review probed the same case against the prototype
in its own guest and on this host
(the gate review's own guard probe,
`keep/review.md`).

## Rung 1, the btrfs snapshot rung

### Loose objects

The source's 25,034 objects all loose,
8 rounds and 32 counted adds.

- `main`:
  median **946.4 ms**,
  adds 869.1 to 962.4.
- `branch-serial`:
  **909.1**,
  858.0 to 955.6;
  its `git ls-tree` step is **366.6 ms**,
  **40 percent** of the add.
- `branch-overlap`:
  **683.2**,
  665.7 to 724.2;
  the read runs beside the check as 373.4 ms of work,
  the fence waits **140.6 ms** for it,
  and the `conversionFence` step falls from 40.6 to 32.3 ms.
- `branch-serial-again` (the control):
  **923.6**,
  867.6 to 966.3.

The identical pair's per-round ratio is **0.95 to 1.06**.
The overlap's is **0.72 to 0.80**,
below the pair's whole range in every round.
`main` sits at 0.97 to 1.06 of the serial arm.

So the overlap takes **226 ms** off a 909 ms loose snapshot add,
which is **25 percent**.
Of the read's 373.4 ms,
232.8 ran between its start and the fence's join and the other 140.6 was waited for.

The other steps stay put:
`checkSource` 105.1 serial and 105.7 overlap,
`git check-attr` 52.1 and 54.2,
`walkSnapshot` 204.2 and 202.9,
`prune` 44.4 and 44.6,
`snapshotIoctl` 5.1 and 5.4.
The read itself is 366.6 ms alone and 373.4 beside the check,
**1.9 percent** slower.

### Packed objects

The same source packed in place by one `git gc`,
16 rounds and 64 counted adds.

- `main`:
  **624.5 ms**,
  578.9 to 761.5.
- `branch-serial`:
  **611.1**,
  590.9 to 690.0;
  its read step is **34.4 ms**,
  **5.6 percent** of the add.
- `branch-overlap`:
  **582.3**,
  551.2 to 719.5;
  the read is 50.4 ms of work beside the check,
  the fence waits **0.0 ms** for it,
  and `conversionFence` falls from 41.4 to 37.1 ms.
- `branch-serial-again`:
  **616.2**,
  581.9 to 743.3.

The identical pair's ratio is **0.93 to 1.23**;
the overlap's is **0.88 to 1.19**,
below 1.00 in 12 of the 16 rounds and above it in 4.
The medians differ by 29 ms,
**4.7 percent**.

So on the common packed layout the read is wholly hidden,
but it costs 16 ms more while it runs beside the check (34.4 to 50.4 ms),
and the add that remains is about the read's own cost minus that excess.
The 29 ms is **inside** the control pair's range,
so this set shows the direction and not the size.

The gate review's quieter guest resolved that size in six rounds of three arms of one binary
(`keep/evidence/packed-6rounds-1795325006-adds.jsonl`):

- serial **523.2 ms** (511.7 to 523.8),
  overlap **501.2** (490.7 to 512.2),
  serial-again **524.2** (519.8 to 528.5);
- overlap to serial **0.94 to 0.98** in every round,
  pair **0.99 to 1.02**;
- the saving is **22.0 ms of 523.2 ms (4.2 percent)**,
  below the pair's range in every round;
- the read is 28.7 ms alone and 41.0 ms beside the check,
  **43 percent** slower,
  and the fence waits 0.0 ms.

So B's resolved gain is large only while the source is loose,
and about **4 percent** on the layout most sources have:
98.1 percent of this repository's `HEAD` blobs are packed
(`doc/decision/cli-git-fence-size-read-keeps-git-ls-tree.md`,
"What it rests on").

## Rung 2, the clone rung on ZFS

### Loose objects

16 rounds and 64 counted adds,
every add of every arm on the `clones` route.

- `main`:
  median **1171.5 ms**,
  adds 933.3 to 11,292.0.
- `branch-serial`:
  **1243.0**,
  967.0 to 2891.0;
  its `git ls-tree` runs inside the clone window and is 219.7 ms.
- `branch-overlap`:
  **1186.0**,
  964.1 to 5857.2;
  the read is 244.5 ms of work beside the check,
  the fence waits 19.7 ms,
  and `fenceBesideTheClone` falls from 237.6 to 28.3 ms of work.
- `branch-serial-again`:
  **1182.6**,
  951.4 to 4096.9.

The identical pair's ratio is **0.37 to 3.77** and the overlap's **0.35 to 4.13**:
this set resolves nothing about the wall,
because other sessions stalled single adds for seconds.
What it does show is where the read sits:
the fence's own window loses the read's 220 ms and the add does not.

### Packed objects

16 rounds and 64 counted adds.

- `main`:
  **1096.4 ms**,
  995.8 to 4014.3.
- `branch-serial`:
  **1087.5**,
  993.2 to 1664.3;
  read 27.3 ms.
- `branch-overlap`:
  **1067.8**,
  975.4 to 1330.0;
  read 40.0 ms beside the check,
  wait 0.0 ms,
  `fenceBesideTheClone` 45.0 to 9.2 ms.
- `branch-serial-again`:
  **1113.8**,
  935.0 to 1611.2.

The identical pair's ratio is **0.70 to 1.35** and the overlap's **0.68 to 1.12**.
The medians differ by 20 ms and say nothing:
the read already fitted inside the clone window,
so moving it earlier changes which step holds it and not the add.

### A source the check refuses

Two rounds on the packed source with one staged path,
which no rung serves on ZFS,
so every add took the `Git's checkout` route:

- `branch-serial`:
  1753.0 and 1911.0 ms,
  no read at all.
- `branch-overlap`:
  1804.3 and 2053.0 ms,
  with 51.6 and 47.6 ms of read work that the fence never took.
- `branch-serial-again`:
  1798.9 and 2063.8 ms.
- `main`:
  3291.9 and 2115.5 ms.

Two rounds resolve nothing,
but they show the shape of the cost the first round measured:
its prototype let a refused rung-1 or rung-2 add pay the read on a thread of its own while Git's checkout ran.
The shipped fix ends that read at the refusal or the decline,
so the shipped record states it as started and not joined and the add pays no read work.

## Do the two steps compete for the disk

Measured as the read's own cost,
and the check's own step,
in the arm where they overlap against the arm where they do not:

- Rung 1 loose:
  read 366.6 to 373.4 ms (**+1.9 percent**),
  `checkSource` 105.1 to 105.7.
- Rung 1 packed:
  read 34.4 to 50.4 ms (**+47 percent**),
  `checkSource` 100.0 to 105.5.
- Rung 2 loose:
  read 219.7 to 244.5 ms (**+11 percent**),
  `checkSource` 80.9 to 87.4.
- Rung 2 packed:
  read 27.3 to 40.0 ms (**+46 percent**),
  `checkSource` 76.6 to 80.2.

They compete where both read the same pages:
on a packed source the read wants the pack and its index,
which the check has just walked,
and it costs 12 to 16 ms more while the check is walking the working tree.
On a loose source the read touches 25,034 object files that the check never opens,
and it costs 2 percent more.
Neither step slows the other by more than about 6 ms of the check's own time.

## What shipping needed, and what the gate review corrected

The first draft of this section asked for the read as a fourth arm of `the_checks_of_the_source`,
joined with the configuration read,
the trash-ignore read and the cleanliness check.
That is not the shape that was measured
(finding Medium 1):
`the_checks_of_the_source` is `tokio::join!`,
so a fourth arm would put the whole read on `checkSource`'s own critical path and hide only the check,
about 140 to 150 ms of a loose add where the measured shape saves 217 ms.
The shipped shape is what "What shipped" says:
a read started before the check and joined at the fence.
A task of the runtime cannot carry it,
as that section says,
because every Git run refuses a caller already inside the runtime.

The four corrections the gate review asked for are in this document now:

1. the write-free claim,
   replaced by the partial-clone measurement and `GIT_NO_LAZY_FETCH=1`
   ("What the overlap costs a refused or a declined add");
2. the shipped shape,
   named and built,
   with the thread and its conditions ("What shipped").
   `tasks_can_run` is not consulted:
   the read is no task,
   so the stand-in runtime that finding named cannot hold it;
3. the packed figure,
   22.0 ms of 523.2 ms,
   4.2 percent,
   beside the loose 25 percent ("Packed objects");
4. the two rules the overlap amends,
   and the paragraph that said the check proves the index equals the tree
   ("What the read needs,
   and what the check proves",
   and the two rules below).

The two rules it amends,
which are stated where their code is:

- `choose_rung`'s own doc
  (`worktree_ladder_run.rs`):
  "Every check that can be made before Git runs is made now,
  cheapest first,
  so a refused rung costs no extra Git run".
  Rung 1's size read is the one Git run a refused or declined add may start,
  and the fix round ends it at the refusal or the decline rather than paying it beside Git's checkout.
- the decline's price
  (`SPEC.md`,
  rung 2's decline rule,
  and `doc/planning/cli-git-clone-rung-conversion-fence.md`,
  "What the decline now costs"):
  "a declined add is Git's own checkout plus one `git check-attr` and the count over it".
  On rung 1 it is also plus the size read the rung starts before the check and ends at the decline,
  and `SPEC.md` and `README.md` say so.

**On a dirty btrfs main checkout the read is paid and used.**
Such a source is snapshotted and restored rather than refused,
so a rung-1 add with ordinary uncommitted changes runs the same fence and takes the same read.
A dirty source that is a linked worktree is refused instead
(`restorable_here`),
and that add starts the read and ends it at the refusal,
exactly like a refused check.

## The options, and the ranking

### A: keep the serial order everywhere

Pros:
no new order,
no thread,
no guard,
and no read on an add that ends before the fence,
which covers a refused check,
a declined source and an untracked `.gitattributes`.

Cons:
a loose rung-1 add keeps paying 366.6 ms of 909 (**40 percent**) on its waiting path,
and a packed one 34.4 ms of 611 (**5.6 percent**),
for as long as the layout exists.

### B: start the read beside the check on rung 1 only

Pros:
the measured saving lands exactly where the read is on the waiting path:
**226 ms of 909 on loose (25 percent)**,
outside the control's range in every round,
and about 29 ms of 611 on packed (4.7 percent),
a direction only.
Rung 2 keeps the shape whose own runs measured no change.

Cons:
two orders to keep correct instead of one;
every rung-1 add that ends before the fence pays a read it never uses;
the commit guard and the read's failure path are new code on the add's path.

### C: start the read beside the check on both rungs

Pros:
one order everywhere,
and rung 2's fence window drops from 237.6 to 28.3 ms of work on loose
and from 45.0 to 9.2 on packed.

Cons:
rung 2's wall is unchanged within a control spread wider than any difference measured here
(0.37 to 3.77 loose,
0.70 to 1.35 packed),
while every refused rung-2 add on ZFS,
which the dirty-source waiver calls a common case,
paid the read in the background in the first round's prototype;
the shipped fix starts no read there.

### Ranking: B > A > C

B over A:
the rung-1 loose saving is 25 percent,
  below the control's whole range in every round,
  and A gives that up without buying anything B does not already have,
  since B keeps the read's failure path and its sentence exactly as they are today.
A over C:
C changes rung 2,
  where this round's runs show no wall difference at all and where
  a refused add pays a read it never uses;
  A keeps that rung as it is,
  so C carries more surface for no measured gain over A's own rung-1 cost.

The owner may also read C over A if one order everywhere is worth more than a wasted read on a refused
clone-rung add;
nothing measured here says that a changed rung 2 pays for itself.

## The first fix round's timings, on Git 2.43.0

The settle fix was first timed in one throwaway `libvirt` guest
(`fence-size-read-settle-2026-11-22`,
Ubuntu 24.04.5,
kernel 6.8.0-147,
four vCPUs,
7.9 GiB,
Git 2.43.0 from the package,
a 6 GiB btrfs image with the source a subvolume on it)
on the same tree,
one commit,
25,034 loose objects,
with `main` at `e3c9b7e62d`,
this branch and an identical `branch-again`,
one uncounted warm-up per arm,
`sync --file-system` before every add,
and one Python process in the guest driving everything.
Its rows are kept in `perf/` under `fence-size-read-settle-never-waits-2026-11-22-*`.
The recheck then showed that the fail-fast and cone rows were taken while the abandon came before the read's child
in 80 of 80 cone adds,
and that every guest measurement of this branch ran on a Git older than the wrapper's tested minimum,
so those figures are history and the re-time below is what this record states.

## The re-time on Git 2.56.1, 2026-11-22

One throwaway `libvirt` guest through mvm,
`never-fetches-guest`,
Ubuntu 24.04.5,
kernel 6.8.0-147,
four vCPUs,
7.9 GiB,
and a 6 GiB btrfs loop image whose source is a subvolume.
**Git 2.56.1 was built from the kernel.org tarball in the guest**
(`NO_RUST=1`,
because the guest has no Rust toolchain),
and it is the Git behind the wrapper in every row and the plain Git of the fixture:
`/usr/bin` and `/bin` are off the arm's path so the wrapper's `candidate_sequence` cannot promote the guest's 2.43.0.
The fixture is the repository's tree at `40783c23ea`,
`filter=lfs` lines removed,
one commit,
24,634 tracked paths,
25,034 loose objects,
then `git gc` for the packed layouts.
The arms are `main` (`ee0cf2268b`,
the tip at the time,
whose crate is the merge's),
this branch (`f9d8beec3c`;
only test and doc commits followed),
and `branch-again` (this branch's identical binary,
sha256 `f987ef5b...` against `main`'s `49c352af...`).
Every round rotates the three one place and reverses every second pass,
one uncounted warm-up runs per arm,
and `sync --file-system` runs before every add.
The host carried other sessions' load throughout,
so absolute times are context and only the ratios are read.

The reading rule was fixed before any row:
the branch is **resolved slower** only when every round's branch-to-`main` ratio lies above the largest identical-twin
ratio,
**resolved faster** only when every round lies below the smallest,
otherwise **not resolved**.
Every branch record states the read (`fenceSizesBesideTheCheck`):
the served rows state its work and the wait,
the refused,
fail-fast and cone rows state `started and not joined`,
and the declined rows state its work when it answered before the decline.

- served,
  loose,
  8 rounds:
  `main` 1692.9 ms,
  branch 1282.8 (twin 1306.5),
  read work 661.3 ms with a 267.8 ms wait;
  ratios 2.36,
  0.68,
  0.78,
  0.68,
  0.72,
  0.77,
  0.71,
  0.75,
  control 0.41 to 1.10;
  **not resolved**
  (the 2.36 round is a host stall;
  seven of eight are below one).
- served,
  packed,
  8 rounds:
  831.5 against 752.1 (twin 771.0),
  read work 69.7 ms with a 0.008 ms wait;
  ratios 0.90,
  0.99,
  0.94,
  0.92,
  0.88,
  0.90,
  1.00,
  0.83,
  control 0.94 to 1.15;
  **not resolved** (six of eight lie below the control's low).
- refused with Git's whole checkout,
  loose,
  6 rounds:
  4945.9 against 4540.6 (twin 4471.4);
  ratios 0.89,
  0.76,
  0.92,
  0.78,
  0.87,
  0.83,
  control 0.90 to 1.19;
  **not resolved**,
  five of six below the control's low.
- the same,
  packed,
  6 rounds:
  2951.1 against 3393.3 (twin 3163.8);
  ratios 1.00,
  1.28,
  0.90,
  1.09,
  1.17,
  0.73,
  control 0.85 to 1.08;
  **not resolved**,
  three rounds above and one below the control's range.
- refused where Git fails at once,
  loose,
  40 rounds:
  31.8 against 32.2 (twin 35.3);
  ratios 0.25 to 2.54,
  median 0.99,
  control 0.59 to 1.85;
  **not resolved**,
  one round above and three below.
- the same,
  packed,
  40 rounds:
  46.7 against 57.2 (twin 52.0);
  ratios 0.66 to 3.59,
  median 1.03,
  control 0.25 to 3.50;
  **not resolved**,
  one above and none below.
- cone sparse refused,
  loose,
  20 rounds:
  203.2 against 190.6 (twin 196.8);
  ratios 0.75 to 1.27,
  median 0.96,
  control 0.82 to 1.26;
  **not resolved**,
  one above and three below.
- the same,
  packed,
  20 rounds:
  169.3 against 159.2 (twin 171.5);
  ratios 0.62 to 2.24,
  median 1.00,
  control 0.60 to 2.80;
  **not resolved**,
  none above and none below.
- declined,
  loose,
  5 rounds:
  8634.6 against 9358.1 (twin 8394.8),
  read work 111.6 ms;
  ratios 1.08,
  1.29,
  1.04,
  1.08,
  0.96,
  control 0.80 to 1.01;
  **not resolved**,
  four above the control's range.
- declined,
  packed,
  5 rounds:
  8651.8 against 8882.5 (twin 8630.0),
  read work 93.6 ms;
  ratios 1.05,
  1.25,
  0.99,
  1.01,
  1.01,
  control 0.89 to 1.14;
  **not resolved**,
  one above and none below.

The integration recheck's 576 counted adds,
12 to 40 rounds a shape,
put the two rows this round could not resolve at 0.992 (0.952 to 1.032) for declined loose
and 0.968 (0.927 to 1.078) for refused whole checkout packed,
so those lopsided counts were noise;
the recheck's table is in its own review,
and this round's step-level run is below.

**The cost a refused or declined add still pays.**
No shape is resolved slower than `main`.
The refused shape,
the fail-fast shape and the cone rows record no read work at all:
the abandon came before the read's child in every verbose probe
(5 of 5 in each shape,
loose and packed),
and the child was ended at publication or never started.
A declined add pays the read when it answered before the decline:
111.6 ms (loose) and 93.6 ms (packed) of work in an 8.6 to 9.4 s add,
which the reduced rounds do not resolve.
The packed refused shape's median ratio is 1.09 with three rounds above the control's range,
which six rounds cannot resolve;
it is stated,
not claimed as absent.

The verbose probes print the new abandon line in 5 of 5 adds for the fail-fast shape and 5 of 5 for the cone shape:
"the size read started beside the check is not used before its child exists,
so the runner ends that child the moment it publishes it".
No `ls-tree` process was alive after any counted add in any row.

## The declined add's step cost, before and after

The integration recheck measured `chooseRung` on a declined add at 0.67 ms on `main`
and 5.91 ms on this branch
(packed 0.6 and 4.8),
positive in 24 of 24 rounds,
about half of it the adding thread freeing the answered map of 24,634 entries.
This round split the step into the pieces the record can name
(`startBesideTheCheck`,
  the abandon's own `abandonRead`,
`abandonTakeTheAnswer` and `abandonEndTheChild`,
and the `chooseRung` remainder)
and timed the declined shape again in the same throwaway guest,
on the packed source,
with `main`,
the build under test and an identical twin,
6 and then 6 rounds,
rotated and reversed as every other shape.

- **Before**:
  the map-drop build,
  over 12 rounds on packed objects:
  `chooseRung` whole 5.42 ms against `main`'s 0.70;
  `abandonTakeTheAnswer` 1.64 ms (the map's free),
  `chooseRung`'s own remainder 2.14 ms,
  `startBesideTheCheck` 0.13 ms.
  Loose:
  whole 4.41 against 0.87,
  the take 1.73,
  the remainder 1.52.
- **The first attempt**:
  forgetting the map,
  over 12 rounds on packed objects:
  the take fell to 0.00 ms,
  but `chooseRung`'s remainder rose to 3.73 ms and the whole was 3.82:
  the cost moved into the add's later allocations rather than going away.
  That build is not this round's fix;
  its rows are in the perf folder and show why.
- **After**:
  the output handed over unparsed,
  over 12 rounds on packed objects:
  `chooseRung` whole 0.88 ms against `main`'s 0.76,
  the take 0.00 (the buffer is left to the process's end),
  `chooseRung`'s own remainder 0.74 ms,
  the same as `main`'s 0.76,
  `startBesideTheCheck` 0.13 ms.
  The paired difference is +0.12 ms
  (median;
  range -1.81 to 0.74;
  9 of 12 positive),
  against the twin's -0.04 (range -0.64 to 0.27),
  and the per-round step ratio is 1.17 (0.39 to 1.99) against the twin's 0.51 to 1.36:
  the remaining 0.12 ms is the read's start,
  which the served shapes pay too,
  and it is inside the identical twin's own spread.
- **No add's total is slower.**
  Declined packed,
  the fixed build against `main` in the same rounds:
  median 1.014 (range 0.915 to 1.152),
  twin 0.867 to 1.154;
  the map-drop build's own run read 0.983 (0.942 to 1.039),
  and the raw comparison's `after` and `raw` arms read 1.040 and 1.015 against `main`
  with twins 0.644 to 1.147 and 0.809 to 1.201.
  Served packed with the fixed build:
  `main` 1507.5 ms,
  fixed 1472.9,
  per-round ratio median 0.977 (0.902 to 1.161),
  twin 0.77 to 1.168;
  the parse the fence now pays is inside the served add's spread.
  So the step's 4 to 5 ms is gone,
  and no add's total is resolved slower.

## What the first round did not do

- The first round changed none of the shipped code:
  its prototype is behind `measurement-routes`.
- It added no dependency,
  because the overlap needs none.
- It did not build the control for a `HEAD` that moves inside the window,
  so the guard there was read,
  not measured;
  the review measured it against the prototype,
  and this branch's own suite measures it against the shipped shape.
- It did not measure a declined source in its own runs;
  the shipped build's guest run does ("The re-time on Git 2.56.1,
  2026-11-22").
- It did not measure a cold cache,
  and it did not resolve rung 2's wall,
  because the host was loaded by other sessions throughout;
  only rung 2's step records are quoted as evidence.
- It did not measure the shipped shape,
  only the prototype's own thread.

The first fix round of 2026-11-22 added the settle that never waits,
the child's end at a refusal or a decline,
and the no-fetch switches only on a promisor source.
The second fix round (the integration recheck's findings) sets the no-fetch variable on every early read,
removes the option and the test of the source,
ends a read the fence will not use,
ends a child published after an abandon,
clears the child's cell when it is reaped,
and re-times every shape on Git 2.56.1 in the guest below.

## Where the numbers are

The harnesses and every row they wrote are committed beside this proposal,
under the package's `perf/` folder (finding Low 4):

- `fence-size-read-beside-the-check-2026-11-21-guest-measure.py`,
  the rung-1 driver that ran inside the guest;
- `fence-size-read-beside-the-check-2026-11-21-host-measure.ts`,
  the rung-2 driver and the refused-source rounds;
- `fence-size-read-beside-the-check-2026-11-21-control-identical-fence.ts`,
  the decision control;
- `fence-size-read-beside-the-check-2026-11-21-guest-loose-adds.jsonl`,
  `-guest-packed-adds.jsonl`,
  `-host-loose-adds.jsonl`,
  `-host-packed-adds.jsonl` and
  `-host-refused-adds.jsonl`,
  the five row files this document's numbers come from.

The earlier eight-round sets are not kept:
every number this document states is recomputable from the five,
as the gate review recomputed them.
This branch's own rows and harness are there too,
named `-2026-11-22-`:

- `-guest-measure.py`,
  the guest driver the shipped runs used,
  and `-ratios.py`,
  which makes the per-round ratios and the medians from a row file;
- `-guest-loose-adds.jsonl`,
  `-guest-packed-adds-set-one.jsonl`,
  `-guest-packed-adds-set-two.jsonl`,
  `-guest-refused-adds.jsonl` and
  `-guest-declined-adds.jsonl`,
  the shipped build's rows;
- `-host-classes.json` and
  `-host-matrix-seed11-n150.json`,
  the two rung-2 harnesses' records against the shipped build.

The first fix round's rows are there too,
named `fence-size-read-settle-never-waits-2026-11-22-...`;
they are history now.
The re-time's rows are named `fence-size-read-never-fetches-and-abandon-sticks-2026-11-22-...`:

- `-guest-loose-served-adds.jsonl`,
  `-guest-packed-served-adds.jsonl`,
  `-guest-loose-refused-adds.jsonl`,
  `-guest-packed-refused-adds.jsonl`,
  `-guest-loose-failfast-refused-adds.jsonl`,
  `-guest-packed-failfast-refused-adds.jsonl`,
  `-guest-loose-cone-sparse-adds.jsonl`,
  `-guest-packed-cone-sparse-adds.jsonl` and
  `-guest-packed-declined-adds.jsonl`,
  the two binaries' rows for the nine shapes;
- `-host-lazy-fetch-probe.json`,
  the host's `git ls-tree` under both no-fetch switches and without either.

The re-time's rows are there under
`fence-size-read-never-fetches-and-abandon-sticks-2026-11-22-`:

- `-guest-served-loose-adds.jsonl`,
  `-guest-served-packed-adds.jsonl`,
  `-guest-refused-loose-adds.jsonl`,
  `-guest-refused-packed-adds.jsonl`,
  `-guest-refused-failfast-loose-adds.jsonl`,
  `-guest-refused-failfast-packed-adds.jsonl`,
  `-guest-cone-sparse-loose-adds.jsonl`,
  `-guest-cone-sparse-packed-adds.jsonl`,
  `-guest-declined-loose-adds.jsonl` and
  `-guest-declined-packed-adds.jsonl`,
  with `-guest-driver.py` (the guest driver)
  and `-ratios.py` (the per-round ratios and the verdicts).

The section "The shipped build,
timed" says which row file is which run,
"The first fix round's timings,
on Git 2.43.0" says which of that round's rows is which shape,
and "The re-time on Git 2.56.1,
2026-11-22" says which re-time row is which shape.
