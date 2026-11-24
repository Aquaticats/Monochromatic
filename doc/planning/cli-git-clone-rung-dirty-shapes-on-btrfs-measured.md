# The clone rung's dirty shapes, measured on Btrfs

## Status

Measured 2026-11-24 in one throwaway `mvm` guest,
which this round created,
used and destroyed through `mvm`;
no domain,
disk or process of it remains
(the destroy is logged at 2026-11-24T03:16:31Z in `keep/mvm-destroy.log`).
**Unapproved and non-final**:
only the owner declares work complete.
The round ran no whole container gate,
by `doc/decision/wrapper-merges-and-publishes-do-not-wait-for-reviews-or-mutation-runs.md`,
and changed no wrapper code
and wrote no decision record (`doc/decision/` and `doc/handover/` untouched).
It answers the measurement that answer owes
(`doc/decision/cli-git-clone-rung-four-answers-after-the-alpha-45-release-pass.md`,
"The dirty-tree limit on Btrfs (D2)", answered "Yes, after one Btrfs round (Recommended)").

The branch is `docs/planning-cli-git-clone-rung-dirty-shapes-on-btrfs`,
made from `main` and merged with it at `6092eaf5be`
(`main`'s tip `2149a02e14` at the merge),
and the guest built the wrapper from that merge's source.

## The question

The owner's answer, as recorded:

> one guest round measures the dirty shapes on Btrfs,
> and the limit is dropped there if the clone is no slower than Git's checkout in that round;
> until the round has run,
> nothing changes on Btrfs.

The limit is `THE_FEWEST_TRACKED_BYTES_THAT_KEEP_THE_CLONE_RUNG = 762_000_000`
(`package/git-policy/cli/src/native/worktree/worktree_clone_rung_limits.rs`).
Its pin was measured on ZFS only,
on three dirty shapes:
a tree of 10,001 files of 2,600 bytes (26,000,028 bytes of tracked files)
cloned at 1.30 of Git's own checkout,
a tree of 10,001 files of 26,000 bytes (260,000,028 bytes) at 1.32 (1.03 to 1.71,
identical control 1.40,
8 counted rounds),
and this repository's own tree of 24,621 files and 763,289,228 bytes,
the tree that round measured,
won at 0.73 to 0.81 of Git's own checkout
(`doc/planning/cli-git-clone-rung-partial-clone-of-a-dirty-source.md`,
"The limit rules as pinned",
and `~/temp/agent/clone-rung-partial-clone-on-the-fenced-rung-2026-11-21/scratch/keep/`,
whose `measure-balanced.ts` this round's instrument is a copy of).

So this round reran those three dirty shapes on Btrfs,
added the shape the alpha.45 release pass found costly on ZFS,
twenty random files of 8 MB with one untracked file,
and added that tree's clean twin,
because the clean rule on Btrfs has no size edge and is what the owner's other answer
("Judge it as clean (Recommended)", D1) would give that source once it is built.

## The shapes

- This repository's tracked working files at the merge commit,
  24,973 paths and 767,946,509 bytes recorded by the index,
  with the three tracked files rewritten and three files nobody tracks (`usual`).
  The fixture is built the way the pin's fixture was:
  the `filter=lfs` lines of `.gitattributes` are removed before it is staged,
  so every byte is a byte the clone copies and the tree sits 5,946,509 bytes above the limit.
- 10,000 files of 2,600 bytes in ten flat directories plus a 28-byte anchor,
  26,000,028 bytes, three rewritten and three untracked (`small:2600:dirty`).
- 10,000 files of 26,000 bytes plus the same anchor,
  260,000,028 bytes, three rewritten and three untracked (`small:26000:dirty`).
- 20 random files of 8,000,000 bytes, 160,000,000 bytes,
  with one file nobody tracks and no tracked change (`soak`).
- the same 20 files with no untracked file (`soak:20:8000000:clean`).

The three tracked rewrites are the same generator the ZFS round used:
the first, the middle and the last third of the tree's text files,
each with a line appended.
The untracked files are `untracked-one.txt`, `untracked-two.txt` and `notes/untracked-three.txt`,
or `untracked-one.txt` alone on the soak shape.
The state is held fixed over all rounds of a shape,
followed by `sync --file-system`, 1.7 s and `git update-index --really-refresh`,
and a run whose source shows any other number of changed paths stops before its first round.

## Method

### Host and guest

The host is this machine,
under other sessions' load as it was found (`NQH`).
Its one-minute load average was between 17.1 and 202.5 over the measurement window,
with a median of 31.2,
read every 30 s into `keep/host-load.log`.
The runs never waited for it to fall.

The guest is Ubuntu 24.04 on kernel 6.8.0-147-generic,
4 vCPU and 8 GiB of memory,
glibc 2.39,
git 2.43.0,
btrfs-progs 6.6.3,
node v24.22.3,
rustc 1.102.0-nightly and cargo 1.102.0-nightly.
Each shape ran inside that one guest;
no shape's rounds spanned two guests.

### The file system

A 12 GiB file on the guest's own disk, `/root/dirty-shapes-btrfs.img`,
formatted with `mkfs.btrfs` and mounted with `mount --options loop` at `/mnt/dirty-shapes`:

- the mount reads `rw,relatime,discard=async,space_cache=v2,subvolid=5,subvol=/`;
- `btrfs filesystem df` reported `Data, single` and `Metadata, DUP`,
  1.45 GiB of data used when the largest shape finished,
  and 1.7 GiB used with 9.6 GiB available before the last three shapes;
- every fixture repository,
  its copies,
  every arm's destination worktree and the harness's own scratch sit on this one file system,
  so a clone's reflink and a worktree's subvolume snapshot stay inside it.

### Builds and arms

Both binaries are release builds of the merge commit's source,
made in the guest with `cargo build --release --locked --jobs 4 --bin monochromatic-git-policy-cli`:

- the shipped wrapper, `main` at the merge with no file changed,
  sha256 `94b5b8721099c50c474edceab3171ef9275774176c5b732c1d33bad756a70976`;
- a measurement copy whose only difference is the one constant,
  `THE_FEWEST_TRACKED_BYTES_THAT_KEEP_THE_CLONE_RUNG` set from `762_000_000` to `0`,
  sha256 `e87d56ea65e2f6ca87b9bc72a0cff2bf46efb7b8c56f2ebe86738243069a8085`.

The clone arm needs that copy because the shipped wrapper declines every dirty tree below the
limit to Git's own checkout,
and the question asks how the clone performs on exactly those trees.
The measurement copy is not shipped and no file of the worktree changes:
the constant is edited in the guest's extracted copy of the pushed source,
and `keep/guest-build.log` prints the edited line.
Every other line of the crate is the merge commit's.

The four arms of every round:

- `wrapper`: the shipped wrapper, an add through the ladder as a person runs it.
  It clones the shapes above the limit and takes Git's own checkout on the shapes below it,
  which the add's own record names.
- `clone`: the measurement copy, so the clone rung runs on every shape the limit would decline.
- `clone-control`: the same bytes as `clone` under another name,
  the identical-binary control the ZFS round used.
- `plain`: the guest's own `/usr/bin/git`,
  the same words (`git worktree add --quiet -b <branch> <destination>`)
  through a PATH that holds no wrapper.

The ZFS round's `plain` arm was `main`'s own binary under `--no-optional-locks`,
a form the ladder does not read.
This round used the guest's Git instead,
because the passthrough costs about 4 ms on this host
(6.0 ms against 2.2 ms for `git version`, 20 runs each, host, 2026-11-24),
which would flatter the clone on the tens-of-milliseconds shapes.
The difference is named here so a reader comparing the two rounds does not read it as the same arm.

### The order and the stopping rule

A Williams square of four arms:
each arm takes each position once per block of four rounds and each arm follows each other arm once;
the second block runs the rows reversed.
One warm-up round precedes each shape.
The stopping rule was fixed before the first round:
two blocks,
and one more block only when the identical pair's median wall ratio falls outside 0.90 to 1.10.
Only the `soak` shape took the third block,
whose pair median read 0.89;
the other first runs' pair medians over their counted rounds were 0.97 to 1.01,
so they stopped after two blocks.
300 ms and a fresh branch name separate the arms of a round.

`usual` and `small:2600:dirty` were run twice,
the first runs being the round above and the second runs a rerun after the first run's largest stall
(the first `usual` run's control arm read 98,614 ms in one round while the host's load reached 202).
Both runs' counted rounds are pooled in the figures below,
and each run's own median is named beside them.

### Verification

Outside the timed window,
for every worktree every arm made in every counted round:
`git status --porcelain` was empty,
and `diff --recursive --no-dereference --exclude=.git` against the Git arm's worktree of the same round was empty.
Every arm of every counted round exited 0,
and each wrapper arm's add record names the route its arm should take:
`a partial clone` for the clone arm and the shipped wrapper on the trees above the limit,
`Git's checkout` for the shipped wrapper on the trees below it,
and `clones` for the clean twin's whole rung.
The Git arm writes no record, so its route is unrecorded by design.

### Records

In `keep/records/`, beside this round's own folder:

- `measure-btrfs-<shape>-<timestamp>.json`, each counted round's arm, position, wall time, route,
  total, steps and verification, for the five shapes and the two reruns;
- `log-<shape>.txt`, the harness's own console output for each run,
  including its summary lines and the file system's facts.

The instruments themselves are kept so the round can be repeated:
`keep/measure-btrfs.ts` (the harness),
`keep/guest-drive.ts` (the guest's steps),
`keep/pool.ts` and `keep/analyze.ts` (the figures below).

Elsewhere in the round folder's `keep/`:
`guest-drive.ts`'s step logs (`guest-prepare.log`, `guest-node.log`, `guest-btrfs.log`,
`guest-build.log`, `guest-run-<shape>.log`, `mvm-destroy.log`),
`host-load.log`,
and `state-so-far.md`.

## The shapes, ratio by ratio

Every ratio is the arm's wall time over Git's own checkout's wall time in the same round,
median with the range over the pooled counted rounds,
and the count of rounds at or below 1.0.

### This repository's tree, three changed paths (`usual`)

24,973 tracked paths, 767,946,509 bytes,
16 counted rounds from two runs (8 each):

- `clone` 2,672 ms,
  **0.41 of Git's own checkout** (0.32 to 0.70),
  at or below 1.0 in 16 of 16 rounds,
  route `a partial clone` in every round;
- the shipped `wrapper` 2,581 ms,
  **0.41** (0.34 to 0.63), at or below 1.0 in 16 of 16,
  route `a partial clone` in every round,
  so the clone above the limit is as quick as the measurement copy;
- its identical control 2,277 ms,
  0.39 (0.33 to 8.91),
  15 of 16,
  the one round above being the 98,614 ms stall;
- Git's own checkout 5,716 ms (4,623 to 11,065).

The identical pair's median is 0.96 (0.73 to 23.24),
the 23.24 being that same stall.
The two runs' own medians of the clone arm are 2,716 ms and 2,470 ms,
and of Git's own checkout 6,201 ms and 5,613 ms.

On ZFS the same tree won at 0.73 to 0.81 of Git's own checkout;
on Btrfs it is 0.41.

### The step split on this repository's tree

The clone arm's and the shipped wrapper's own add records,
medians over the pooled rounds,
on this tree:
`git ls-tree` 613.7 ms and 635.0 ms,
`cloneEntries` 516.6 ms and 502.9 ms,
`cloneIndex` 345.1 ms and 255.5 ms,
`partClassify` 153.0 ms and 158.9 ms,
`git ls-files` 144.2 ms and 141.3 ms,
`git status` 95.1 ms and 96.1 ms,
`git check-attr` 78.4 ms and 84.9 ms,
`copyIgnored` 60.8 ms and 66.7 ms,
`partRead` 43.6 ms and 46.3 ms,
against Git's own checkout's whole 5,716 ms.
The first number of each pair is the shipped wrapper's lane and the second the clone copy's,
both from the same harness record, which holds the steps per arm.

### 10,001 files of 2,600 bytes, three changed paths (`small:2600:dirty`)

26,000,028 bytes, 16 counted rounds from two runs:

- `clone` 661 ms,
  **0.73 of Git's own checkout** (0.60 to 1.13),
  at or below 1.0 in 14 of 16 rounds,
  route `a partial clone` in every round;
- the shipped `wrapper` 894 ms,
  **1.05** (0.77 to 1.48),
  at or below 1.0 in 5 of 16,
  route `Git's checkout` in every round,
  the source check and then Git's own checkout;
- its identical control 692 ms,
  0.76 (0.52 to 1.00), 15 of 16;
- Git's own checkout 889 ms.

The identical pair's median is 1.01 (0.86 to 1.26).
The two runs' own clone medians are 730 ms and 613 ms,
both pooling to the 0.73 above.

On ZFS this tree cloned at 1.30 of Git's own checkout.
On Btrfs the clone is quicker than Git's own checkout on the median,
and two of the sixteen rounds read above 1.0 (1.06 and 1.13),
inside the identical pair's own 0.86 to 1.26 spread.

### 10,001 files of 26,000 bytes, three changed paths (`small:26000:dirty`)

260,000,028 bytes, 8 counted rounds:

- `clone` 614 ms,
  **0.55 of Git's own checkout** (0.49 to 0.98),
  at or below 1.0 in 8 of 8 rounds,
  route `a partial clone` in every round;
- the shipped `wrapper` 1,132 ms,
  **1.00** (0.90 to 1.33),
  at or below 1.0 in 4 of 8,
  route `Git's checkout` in every round;
- its identical control 652 ms,
  0.56 (0.47 to 0.82), 8 of 8;
- Git's own checkout 1,106 ms.

The identical pair's median is 0.97 (0.58 to 1.49).
On ZFS this tree cloned at 1.32 (1.03 to 1.71);
on Btrfs at 0.55.

### Twenty files of 8,000,000 bytes with one untracked file (`soak`)

160,000,000 bytes, no tracked change, 12 counted rounds:

- `clone` 170 ms,
  **0.54 of Git's own checkout** (0.34 to 0.96),
  at or below 1.0 in 12 of 12 rounds,
  route `a partial clone` in every round,
  which is the rung's dirty path and not the clean rule the owner's answer gives this source;
- the shipped `wrapper` 380 ms,
  **1.31** (0.83 to 2.00),
  at or below 1.0 in 1 of 12,
  route `Git's checkout` in every round,
  the source check and then Git's own checkout;
- its identical control 145 ms,
  0.51 (0.33 to 0.89), 12 of 12;
- Git's own checkout 286 ms.

The identical pair's median is 0.94 (0.59 to 1.76).
The alpha.45 pass measured the same shape on ZFS at 380 to 630 ms through Git's own checkout
against 34 to 53 ms through the clone of the same tree when it was clean
(`~/temp/agent/release-pass-wrapper-alpha-45-2026-11-23/clone-rung/keep/review.md`,
M1).
On Btrfs the declined add is 380 ms against Git's own checkout's 286 ms,
and the clone is 170 ms.

### The same 20 files, clean (`soak:20:8000000:clean`)

160,000,000 bytes, no change of any kind, 8 counted rounds:

- the shipped `wrapper` 47 ms,
  **0.24 of Git's own checkout** (0.19 to 0.28),
  at or below 1.0 in 8 of 8 rounds,
  route `clones` in every round,
  which is the whole rung's subvolume snapshot and the rule the owner's answer gives this tree;
- `clone` 51 ms,
  0.27 (0.23 to 0.32),
  8 of 8;
- its identical control 50 ms,
  0.26 (0.19 to 0.31),
  8 of 8;
- Git's own checkout 184 ms.

The identical pair's median is 1.00 (0.61 to 1.14).
The clean clone is 0.24 of Git's own checkout on Btrfs,
about four times quicker,
and this is the arm the owner's D1 answer would route the untracked-only source to.

## Was the clone no slower than Git's own checkout on Btrfs?

For every shape this round measured,
yes:

- this repository's tree,
  0.41 of Git's own checkout (0.32 to 0.70), 16 of 16 rounds at or below;
- 10,001 files of 26,000 bytes,
  0.55 (0.49 to 0.98), 8 of 8;
- 10,001 files of 2,600 bytes,
  0.73 (0.60 to 1.13), 14 of 16,
  and the two rounds above 1.0 lie inside the identical pair's own 0.86 to 1.26 spread;
- 20 files of 8,000,000 bytes with one untracked file,
  0.54 (0.34 to 0.96), 12 of 12 as the rung's dirty path;
- the same 20 files clean,
  0.24 (0.19 to 0.28), 8 of 8 as the whole rung's snapshot.

So the record's condition for dropping the limit on Btrfs
("the clone is no slower than Git's checkout in that round")
holds on every shape measured here,
including the two small trees where the clone lost on ZFS at 1.30 and 1.32.
The limit is not dropped by this round:
this round only measures and records,
and the code change that would drop it,
with the record that would answer for it,
belongs to the round that makes it.

Two qualifications belong beside that verdict:

- the clone arm on the four dirty shapes is the measurement copy with the limit set to zero,
  because the shipped code declines those trees before it clones them.
  The shipped wrapper's own arm is measured beside it,
  and on this repository's tree,
  where the limit lets the shipped code clone,
  the shipped wrapper and the measurement copy read 0.41 and 0.41 (0.34 to 0.63 and 0.32 to 0.70),
  which is what says the copy's only difference changed no timing;
- the untracked-only shape's clone arm is the rung's dirty path (a partial clone of the 20 unchanged files),
  while the owner's D1 answer would judge that source clean and clone the whole rung.
  The clean twin measures that arm on the same tree and the same file system at 0.24,
  so either route is quicker than Git's own checkout there.

## What the limit costs on Btrfs

Keeping the limit leaves a person with the shipped wrapper's add,
which on Btrfs is:

- 0.41 of Git's own checkout on this repository's tree,
  where the wrapper clones;
- 1.00 and 1.05 on the 260 MB and 26 MB dirty trees,
  where the wrapper takes the source check and then Git's own checkout;
- 1.31 on the 20-file tree with one untracked file,
  where the wrapper takes the source check and then Git's own checkout,
  against 0.54 for the clone and 0.24 for the clean twin.

So on Btrfs the limit buys nothing on the shapes it was pinned for:
the trees it declines clone quicker than Git's own checkout,
and the declines themselves pay the source check on top.

## What this round does not measure

- The `many:<N>` changed-path shapes.
  They belong to the changed-file cut, which this round left as it is,
  and they were not run.
- A source whose `filter=lfs` lines are present.
  The fixture removes them, as the pin's fixture did,
  so every byte of the tree is a byte the clone copies.
  This repository as a person has it records 767.9 MB of tracked files with 244 MB behind `filter=lfs`,
  so its cloneable share is about 526.6 MB, below the limit.
  The owner's other answer (D3, "Sum only what it clones (Recommended)") is not built in `main`,
  and the round that builds it changes which of these trees a dirty checkout of this repository lands on.
- A dirty tree between 260,000,028 and 767,946,509 bytes,
  or below 26,000,028 bytes.
  The three points are the pin's own.
- Any second Btrfs configuration.
  One loop-backed file system with its default options was measured,
  with no compression and no qgroup change.
- The same shapes on ZFS in this round's arms.
  The ZFS figures quoted here are the earlier round's,
  with its wrapper-passthrough Git arm,
  and they are context for the Btrfs ratios, not a control of them.

The round's own noise floor has two parts.
The identical pair's per-round spread says how far one round's ratio can move between two identical builds:
0.86 to 1.26 on the 26 MB tree,
0.58 to 1.49 on the 260 MB tree,
0.73 to 23.24 on this repository's tree (the 98 s stall),
0.59 to 1.76 on the 20-file tree with one untracked file,
and 0.61 to 1.14 on the clean twin.
The pair's own medians say how little that movement shifts a median:
0.94 to 1.01 on every shape without the stall.
So the clone's medians (0.24 on the clean twin, 0.41, 0.54, 0.55 and 0.73) are below parity by more than
the medians' own agreement,
while a single round can still read above 1.0, as two of the 26 MB tree's sixteen do.

## What this leaves owed

- The owner's answer to whether the limit is dropped on Btrfs,
  given that the record's condition holds on every shape measured.
  The drop and the decision record are the next round's,
  not this one's.
- The D1, D3 and D4 answers are recorded in
  `doc/decision/cli-git-clone-rung-four-answers-after-the-alpha-45-release-pass.md`
  and not built in `main` at the merge.
  The untracked-only shape's 0.54 and its clean twin's 0.24 are the figures the D1 build will meet;
  D3's sum changes which trees the limit declines.
- A second Btrfs configuration (compression, a subvolume layout other than the default,
  a file system on a real device) if the owner wants the edge pinned tighter than one guest's.
