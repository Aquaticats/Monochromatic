# The conversion fence: running the size read beside the source check

**Unapproved and non-final.**
This round measures one overlap and proposes what to do with it;
it changes no shipped behaviour,
merges nothing,
and only the owner declares it done.
The prototype it measures is committed with "PROTOTYPE FOR MEASUREMENT,
NOT FOR MERGING" in its subject,
and every number below is this round's runs on this machine on 2026-11-21 and 2026-11-22.

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

So on rung 1 the read is on the add's waiting path,
after the check and after the attribute read,
and the snapshot's constant-time copy leaves nothing for it to overlap.

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
It reads committed objects only:
no index byte,
no attribute,
no file of the working tree.

The check produces the commit the fence may use,
and produces it only after proving that the source's index equals the tree of that commit
and that every tracked file equals its index entry.
So the read may start beside the check,
but its answer may be used only while the commit it listed is `source.clean.head_commit`.
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

Nothing the read touches is unproven:
the objects a commit fixes are the same before and after the check,
and the only thing the check adds is which commit is the fence's,
which the guard enforces.

## What the overlap changes when the check refuses, or the add is killed

**When the check refuses**,
the ladder falls to Git's own checkout before any fence runs,
so today the read never starts.
With the overlap the read starts anyway and its answer is never taken:
the map is dropped with the process or when the add ends.
Nothing is written and no state is left,
but the refused add pays a read it did not need.
The same holds for the fence's own cheap refusal (an untracked `.gitattributes`)
and for a source the decline rule refuses,
both of which are decided inside `fence_the_attributes` and therefore after the read has started.

**When the process is killed** during the overlap,
the difference from today is only that a read-only Git child may be running:
no lock is taken and no index is written,
and both rungs claim their stage name after the fence,
so a kill in the overlap window leaves the state a kill inside `checkSource` leaves today.
Rung 2's rule is unchanged:
the worktree holds no index until Git's write is done,
and the read adds nothing to that window.

## The prototype

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
which is a prototype's shortcut and not a shape to ship:
one command is one process,
and a shipped overlap would carry the read through the ladder instead.

The prototype's commit message spells the switch `MONOCROMATIC_CLONE_SIZE_READ_BESIDE`
(a misspelled `MONOCHROMATIC`).
The code reads the correct name,
the commit was not amended,
and the message is wrong as committed.

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
but they show the shape of the cost:
a refused rung-1 or rung-2 add pays the read on a thread of its own while Git's checkout runs,
and its only trace is the `fenceSizesBesideTheCheck` work in the record.

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

## What it would need to ship

A shipped overlap would need:

- the read started where the check's own reads start,
  on the process's runtime (`the_checks_of_the_source` in `worktree_snapshot_source.rs`),
  as a fourth read joined with the configuration read,
  the trash-ignore read and the cleanliness check,
  rather than on a thread of its own as the prototype does;
- the commit resolved before the check and compared with `clean.head_commit` at the fence,
  in the shape the prototype already has;
- the started answer carried through the rung,
  not through a process-wide slot;
- the read's failure still refusing the rung,
  with the same sentence,
  and a read the system could not start falling back to the fence's own read;
- a control this round did not build:
  a `HEAD` that moves between the start and the check,
  proving the guard drops the map and the fence reads its own.
- the cost of a refused or declined add stated in `SPEC.md` next to the rung's other costs.

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
pays the read in the background.

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

## What this round does not do

- It changes none of the shipped code:
  the prototype is behind `measurement-routes`,
  so a shipped build of this branch behaves exactly as `main` on rung 1 and rung 2.
- It adds no dependency,
  because the overlap needs none.
- It does not build the control for a `HEAD` that moves inside the window,
  so the guard is read,
  not measured.
- It does not measure a declined source (more than a quarter of its paths converting),
  where the read is paid and dropped;
  the refused-source runs price the shape of that cost only.
- It does not measure a cold cache,
  and it does not resolve rung 2's wall,
  because the host was loaded by
  other sessions throughout;
  only rung 2's step records are quoted as evidence.
- It does not measure the shipped shape,
  which would join the read with the check on the runtime,
  only the prototype's own thread.

## Where the numbers are

The harnesses and every row they wrote are in the round's scratch folder,
outside the repository:
`/home/user/temp/agent/fence-size-read-beside-source-check-2026-11-21/scratch/`,
with `guest-measure.py` (rung 1,
inside the guest),
`host-measure.ts` (rung 2,
on the host,
and the refused-source rounds),
and the six row files `loose-adds.jsonl`,
`packed-adds.jsonl`,
`loose-host-adds.jsonl`,
`packed-host-adds.jsonl`,
`refused-host-adds.jsonl`,
plus the earlier eight-round sets `loose-host-adds-8rounds.jsonl` and `packed-adds-8rounds.jsonl`.
They are not committed:
this round has no package beside which they would live,
and the proposal that keeps them is the one that builds the overlap.
