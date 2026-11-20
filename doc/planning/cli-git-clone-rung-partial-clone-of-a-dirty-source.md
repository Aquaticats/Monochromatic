# cli-git clone rung: cloning the unchanged files of a dirty source

## Status

Proposal,
written 2026-11-20T07:35:00-05:00.
**Unapproved and not final**:
only the owner declares work complete.
Nothing here is in `main`;
the design is proposed,
a prototype measures it,
and the last section asks for a decision.
The question it answers is the one the owner answered "Yes,
design and measure it (Recommended)" on 2026-11-20,
recorded in `doc/decision/cli-git-zfs-clone-rung-dirty-source-waiver.md`,
which also names when the waiver it granted ends.

The work behind this proposal:

- Prototype on `docs/planning-cli-git-clone-rung-partial-clone-of-a-dirty-source`,
  marked as a prototype for measurement in every commit,
  not for merging as it is.
- Measurements on this host,
  under the load there is,
  against step 3's branch (`73908f4fd7`) and `main` (`392042de70`),
  each built the same way,
  arms alternating with a same-binary control.
- The untouched-files rule,
  the refusal list,
  the index and the proof are controlled by 24 tests,
  three of which were shown red against a planted defect each.

What was not done is in "What this leaves owed".

## What the rung does today

Rung 2 clones the tracked files of a source checkout and installs the source's index.
Because the index is installed as it is,
the rung first proves the whole source clean,
and a source with any uncommitted change ends in Git's own checkout,
having paid the proof.

`package/git-policy/cli/SPEC.md`,
under "Rung 2 needs a clean source",
says:

> Rung 2 needs a clean source on btrfs,
> XFS,
> overlayfs or ZFS,
> with the source and the destination in one clone domain.

and,
later in the same section:

> The same `--no-checkout` run is followed by `FICLONE` of every tracked file (mode and nanosecond times kept;
> links recreated),
> a second comparison of the source with its own index,
> the installation of the index,
> and the same tree difference.

The accepted design says the same.
`doc/planning/cli-git-btrfs-worktree.md`,
step 2's rung list:

> - Rung 2,
> reflink:
> needs a filesystem with `FICLONE` (btrfs,
> XFS with reflink,
> overlayfs over either) and a clean
> source worktree on the same filesystem.
> The measurement says it saves disk,
> not time,
> so it is optional.

and,
in its checks:

> - The source must be clean,
> checked without spawning Git where possible:
> no `index.lock`,
> `MERGE_HEAD`,
> `CHERRY_PICK_HEAD`,
> `REVERT_HEAD`,
> `BISECT_LOG`,
> `rebase-merge`,
> `rebase-apply` or
> `sequencer` in its administrative directory;
> its index equals the tree of its `HEAD` (in-process `parse_index` and
> `flatten_tree`);
> every entry's size,
> modification time and inode equal a fresh `lstat`;
> and no untracked file that is
>   not ignored.

What replaces each of those is in "Where the design text changes".

## What a partial clone does

The new worktree holds what Git's own `git worktree add` makes:
the tree of the new `HEAD`,
written into an empty directory.
A tracked path whose file still equals the entry that names it holds exactly the bytes Git would write there,
so it can be cloned;
every other path is written by Git.
The rung therefore keeps its job,
which is making the files by cloning them,
and changes only which files it takes.

### The paths a clone takes

A tree path is cloned when all of these hold:

- the index records the path at stage 0,
  with no extended flags and no assume-unchanged bit;
- the index entry's mode and object name equal the tree entry's,
  so nothing is staged there;
- the entry's mode is a regular file or a link;
- the file's own `lstat` equals the entry's recorded size,
  type,
  executable bit,
  inode,
  owner and both times to the nanosecond,
  with no recorded count read as a wildcard;
- the file's modification time is older,
  in whole seconds,
  than the index file's own,
  which is Git's racy rule.

Those are Git's own fast-path conditions,
taken in process.
The clone then keeps the source's mode and times,
and a link's target is recreated.

### The paths Git writes

Git writes,
with one command (below),
every tree path the clone does not take:

- a path the index records with another object name or mode than the tree names,
  which is a change staged in the source;
- a path the index lacks,
  which is a deletion staged in the source;
- a path whose file is missing from the source,
  which is a deletion not staged;
- a path whose file differs from its entry by size,
  inode,
  owner or either time,
  including a file only touched;
- a path whose type or executable bit changed;
- a path whose entry is racy,
  or records a zero nanosecond count a Git without nanosecond support wrote.

Writing a path that was merely touched costs one write and never a wrong byte;
cloning one that really changed would put the person's bytes in a worktree that must hold the commit's.

A path only the index names,
a staged addition or an intent-to-add entry,
is in no tree and reaches no new worktree:
the index is built from the tree,
so it is not named there either.

### States that still refuse the whole rung

Each of these ends the rung before anything is made,
and Git's own checkout makes the worktree:

- an operation in progress in the source's Git directory:
  `index.lock`,
  `MERGE_HEAD`,
  `CHERRY_PICK_HEAD`,
  `REVERT_HEAD`,
  `BISECT_START`,
  `BISECT_LOG`,
  `rebase-merge`,
  `rebase-apply` or `sequencer`;
- an index or object database the in-process readers cannot read:
  a split index,
  a sparse index,
  a layout they do not know,
  a checksum that does not verify,
  an unreadable file;
- a source with a sparse checkout,
  which the source check already refuses;
- an unmerged entry,
  an intent-to-add entry,
  a skip-worktree entry and an assume-unchanged entry;
- a submodule,
  in the index or in the tree;
- a tree entry that is neither a file nor a link;
- a `.gitattributes` file that is changed or untracked in the source,
  because its rules decide how Git converts the bytes of other paths,
  and the source's converted bytes and the new worktree's rules would have to agree;
- a path whose file cannot be reached through plain directories below the source's root,
  which a link where a directory was is.

A source a reset could not undo,
and a source whose conversion settings differ between the two worktrees,
are refused by checks the rung already had,
and the partial clone inherits them unchanged.

### Untracked files

Today's proof refuses a source with an untracked file that no ignore rule covers.
The reason is in the module that owns the check,
`package/git-policy/cli/src/native/worktree/worktree_clean.rs`:

> A snapshot or a clone carries the bytes of the source's files,
> so any change the source has not committed would reach the new worktree.

`git log -S` on that sentence and on the removal path shows it entered with `95ce7f01be`
(2026-11-02T03:40:16-05:00),
whose message names both callers:

> A cleanliness check built on the in-process index reader (no operation in progress,
> index equal to HEAD,
> stat fields
> equal to lstat,
> no unignored untracked file) decides,
> and every refusal falls to Git with a log line.

and whose module doc gives the two reasons:

> A snapshot or a clone carries the bytes of the source's files,
> so any change the source has not committed would reach the new worktree,
> and a removal by the subvolume ioctl cannot be undone.

So the rule exists for two callers:
the snapshot rung,
which copies the whole directory and would carry untracked files into the new worktree,
and the removal path,
which must never destroy a file nobody committed.
A partial clone carries no untracked file:
its paths come from the tree,
and the new worktree's ignored state arrives through the ladder's own ignored-state copy,
as it does after Git's checkout.
**A partial clone does not need the untracked rule.**
The rule stays for the snapshot rung and the removal path,
and the clone rung asks its own question instead of inheriting it.
This proposal does not split the check yet,
so the prototype still pays the scan and the trash question;
what the split would save is measured under "What the proof costs".

### The other states the question names

- `.gitattributes` changed or untracked in the source:
  refused,
  as above,
  because the source's bytes may be a conversion of the blob under rules the new worktree does not have.
  Unchanged `.gitattributes` files are covered by the checks the rung already makes:
  the two trees' attribute blobs must be equal,
  and the conversion settings of the two worktrees must agree in order and value.
- A filter or text conversion:
  the cloned bytes are the source's already-converted worktree bytes,
  and the written paths are converted by Git itself inside `git checkout-index`,
  which runs the same filters Git's checkout runs.
  A filter other than the standard Git LFS definition is refused by the existing check,
  as in the whole-rung clone.
- A submodule:
  the whole rung is refused.
  A clone makes files and links;
  a gitlink is neither,
  and the rung cannot prove what Git would put at that path.
- A symbolic link:
  cloned as a link with its target,
  at the source's times.
  Its index entry keeps the fields the source's own index recorded,
  never the new link's,
  so a name another process replaced between the clone and the index write is not certified;
  Git compares a link by target and by the length the entry records.
- A skip-worktree entry:
  the whole rung is refused.
  Git was told the file is outside the checkout,
  so neither the clones nor the writer can say what the worktree should hold there.

### The new worktree's index

The index cannot be the source's:
a staged path records another object,
a staged deletion is missing from it,
and a staged addition names a path no tree has.
So the rung builds one from the tree of the new `HEAD`,
in `package/git-policy/cli/src/native/worktree/worktree_clone_partial_index.rs`:

- one entry per tree path,
  in path order,
  with the tree's mode and object name and no flags;
- the stat fields of the files the clones made,
  taken from their descriptors after their last change;
- a link's entry keeps the fields the source's index recorded;
- zero stat fields for the paths Git writes,
  which is what makes Git check those files by content before it trusts them;
- the file written as version 2 with the repository's own checksum,
  SHA-1 or SHA-256.

It is published by the rung's existing install step,
`subvolume_index_install_into_new_worktree.rs`:
`index.lock` created exclusively,
the absence of an index checked under it,
the finished bytes written into the lock,
and the open file linked under the name `index` without replacing anything.

### Which Git command writes the changed files

One command:

```sh
printf 'a.txt\0bin/tool.sh\0' | git checkout-index --stdin -z --index
```

run in the destination with `GIT_DIR` and `GIT_WORK_TREE` naming the new worktree,
and the paths NUL-terminated on standard input,
in the order the tree lists them.
It is Git's own scripted checkout:
it writes each entry's object through the checkout's conversion filters,
and `--index` records the new stat fields so the first `git status` of the worktree is cheap.
Naming the paths on standard input is what keeps a source with many changed paths out of the argument list.
A run that fails ends the rung,
and Git's own checkout then writes the same bytes.

### How the result is proven

The rung proves the worktree after the tree difference,
with two read-only Git runs in the new worktree:

- `git --no-optional-locks status --porcelain=v1 -z --untracked-files=all --no-renames` prints nothing,
  so no worktree file differs from its entry by Git's own judgement and nothing is untracked;
- `git --no-optional-locks diff-index --cached --quiet <new HEAD>` answers zero,
  so the index equals the new `HEAD`'s tree exactly.

`--no-optional-locks` keeps the proof from writing an index it judges.
A failed proof ends the rung and the fallback checkout makes the worktree,
so nothing unproven is ever published as the add's result.

`git diff-files --quiet` is deliberately **not** the worktree check:
it reports a stat difference without comparing content,
and a link's entry keeps the source's fields on purpose,
so it reports every cloned link as modified.
The prototype found exactly that,
and the controls assert the `git status` answer instead.

Modes and link targets are the clone's own business and are asserted by the controls:
a cloned executable keeps 0755,
and a cloned link keeps its target.

## The prototype

Branch `docs/planning-cli-git-clone-rung-partial-clone-of-a-dirty-source`,
commits marked `PROTOTYPE FOR MEASUREMENT AND NOT FOR MERGING`.

- `worktree_clone_partial_split.rs`:
  which tree paths a clone takes and which Git writes,
  and the refusals above,
  in process,
  with no Git run.
- `worktree_clone_partial_index.rs`:
  the index built from the tree.
- `worktree_clone_partial.rs`:
  the step,
  the write command and the proof,
  and two size rules that exist only to be measured,
  `CLI_GIT_PROTOTYPE_CLONE_MIN_BYTES` and `CLI_GIT_PROTOTYPE_DECLINE_BELOW_TOTAL_BYTES`.
- `worktree_ladder_finish_clone_rung.rs`:
  the clone rung's steps after real Git,
  moved out of `worktree_ladder_finish.rs` to stay under the file-size limit.
- `worktree_clone.rs`:
  the clone engine answers the stat fields it made and installs an index only when the caller asks.
- `worktree_ladder_run.rs`:
  a source with changes is no longer a refusal of the clone rung,
  and the prototype's whole-tree rule,
  when set,
  declines the rung before any proof is paid.

24 tests pass on this host,
run as `cargo test --lib worktree_clone_partial` with `CLI_GIT_BTRFS_ABSENT_ACKNOWLEDGED=1`,
the btrfs-gated controls printing `BTRFS CONTROL NOT RUN` as everywhere on ZFS.
Three of them were shown red,
each against one planted defect,
with the logs kept:

- the stat verdict accepting every present file:
  the end-to-end control fails with
  `the source changed while it was cloned (edited.txt differs from its index entry by size, times or inode)`.
- the object-name comparison dropped:
  the staged path is cloned,
  4 clones instead of 3.
- the source's index installed instead of one built from the tree:
  the proof fails with
  `the new worktree is not what Git's checkout makes: staged.txt differs`.

## The measurements

### Method

- Host:
  this machine,
  its 8 CPUs and its own load,
  under a `systemd-run --user --scope` capped at 6 CPUs,
  20 GiB and 512 tasks for the harness;
  the adds are the real work,
  not synthetic load.
- Filesystems:
  every fixture repository and every destination under the home dataset,
  on ZFS,
  source and destination in one dataset,
  so rung 2 applies to the prototype and to step 3.
- Fixtures:
  throwaway repositories built from `git archive HEAD` of this worktree,
  24,389 tracked files of about 780 MB,
  and synthetic trees of 10,001 files of about 2.6 KB,
  8.8 KB and 26 KB in ten flat directories,
  one commit each.
  Each fixture's source was put in one of two states before every add:
  clean,
  or the state this machine's main checkout is usually in,
  three tracked files rewritten and three files nobody tracks.
  The index was refreshed after the state was written,
  so no entry was racy.
- Builds:
  release,
  the same flags for every arm,
  `mise run //package/git-policy/cli:native:build:release`.
  Step 3's build is the branch base `73908f4fd7`;
  `main`'s is `392042de70` from a `git archive`.
- Arms:
  the prototype,
  the prototype again under another name as the same-binary control,
  step 3,
  and `main`.
  The order rotates every round;
  every figure is a ratio to `main` in the same round,
  as `TVC` asks.
  Rounds:
  five per state on this repository's tree,
  three per fixture for the size rules.
- Fixture setup ran through `main`'s build,
  with its bulk-staging policy bypassed and its commits made from plumbing:
  the wrapper's commit path waits on the account's own commit service,
  which a throwaway repository must not reach.
- Verified,
  every round,
  for every worktree the arms made:
  `git status --porcelain` empty,
  `git diff --name-only HEAD` empty,
  and `diff --recursive --exclude=.git` against `main`'s worktree of the same round empty.
  Every arm passed all three in every round.
- Every prototype round took the clone route on a clean source and the partial clone on a dirty one,
  and every step 3 and `main` round took Git's checkout on a dirty source,
  so the arms are what they say.

### The dirty source, this repository's tree

Medians over five rounds,
in ms,
and the ratio to `main` per round:

- `main`:
  1256 ms (1139 to 1319),
  the reference.
- step 3:
  1412 ms,
  **1.14** of `main` (1.07 to 1.16).
  This is the cost the waiver records:
  the clean proof is paid and the add then takes Git's checkout.
- prototype:
  915 ms,
  **0.74** of `main` (0.67 to 0.80).
- prototype,
  same-binary control:
  1021 ms,
  0.82 (0.77 to 0.92).
  The two identical arms differ by up to 12 per cent within a round,
  so the prototype's 0.74 is a real gain and not the noise,
  and its distance from step 3's 1.14 is far outside that spread.

### The clean source, this repository's tree

Medians over five rounds,
ratio to `main` per round:

- `main`:
  1304 ms (1185 to 1596).
- step 3:
  997 ms,
  0.81 (0.51 to 0.84).
- prototype:
  856 ms,
  0.62 (0.55 to 0.78).
- prototype,
  same-binary control:
  1058 ms,
  0.85 (0.59 to 0.88).

The clean path is the rung it already was,
unchanged by this proposal;
the three clone arms are the same code and the same work,
and their ratios span 0.51 to 0.88 in this run,
so no difference between them is claimed here.

### What the proof costs, and what it could cost

Step split of the dirty arm,
medians over the five rounds on this repository's tree,
in ms:

- `git ls-files`,
  the untracked scan:
  82.3
- `git check-ignore`,
  the trash question,
  which only the snapshot rung uses:
  35.0
- `git status`,
  the changed-path listing and the proof's own status summed under one name:
  113.9
- the in-process stat walk inside the clean check,
  `checkSource`:
  35.1
- reading the index and the tree for the rung,
  `partRead`:
  26.9
- the split's own walk of every tracked file,
  `partClassify`:
  79.7
- building and installing the index,
  `partIndex`:
  55.2
- `git diff-index --cached --quiet`,
  the proof's index side:
  24.7

That is about 453 ms of proof and setup,
49 per cent of the 915 ms add.
The clones themselves are `cloneEntries` 333.7 ms;
the rest of the command is `gitAdd` 4.8,
`copyIgnored` 34.1,
`finishRung` 18.1 and Git's own worktree bookkeeping.

For comparison,
step 3's dirty arm spends about 214 ms on its proof,
its `git ls-files` 105.0,
`git check-ignore` 34.6,
`git status` 40.8 and `checkSource` 32.9,
and then 1085 ms on Git's checkout;
`main` spends nothing on a proof and 1087 ms on Git's checkout.

The proof can be made cheaper,
and the parts are measured even though the combination is not:

- One `git status --porcelain=v2 -z --untracked-files=all --ignored=no --no-optional-locks` answers the untracked
  scan,
  the changed-path listing,
  and the per-path verdict a length-based proof takes from its own walk.
  The runs it would replace cost 82.3,
  113.9,
  26.9 and 79.7 ms here,
  and the one run measured 90 to 114 ms in the same rounds,
  so the saving is between about 150 and 190 ms,
  roughly a fifth of the add.
  What it gives up is the strict nanosecond proof:
  Git's status settles a racy entry by content and then reports nothing,
  which is a stronger answer,
  not a weaker one.
- `git check-ignore` is asked for the snapshot rung's trash place.
  A clone rung makes no trash,
  so the clone rung need not pay its 35 ms.
  Splitting the source check is a change of its own and is not in the prototype.
- The in-process walk inside the clean check,
  35.1 ms,
  is paid before anything knows whether the rung will be taken;
  a rung that asks its own question once would not pay it twice.

### The size rules

The evidence from step 3's integration pass is that a clean add of many small files is slower than `main`'s on ZFS.
It is finding `H1` in
`~/temp/agent/integration-pass-zfs-step-3-with-the-dirty-source-waiver-2026-11-20/keep/review.md`:
10,001 files of about 2.6 KB took a median 1.33 times `main`,
8.8 KB 1.18,
26 KB parity,
and this repository's tree 0.64.
A partial clone inherits that shape unless something stops it,
and two rules were built into the prototype and measured against it.

All figures below are the dirty source,
three rounds per fixture,
medians of the ratio to `main` in the same round,
with the range in brackets.

The per-file rule clones only the unchanged files at or above a size,
and lets Git write the smaller ones too:

- this repository's tree,
  cutoff 4 KB:
  **1.05** (0.96 to 1.10);
  cutoff 32 KB:
  **1.90** (1.43 to 2.11);
  no cutoff at all,
  so every settled path is cloned:
  **0.77** (0.71 to 0.83);
  the same-binary control:
  0.76 (0.73 to 0.78).
- writing every unchanged path through the rung's write step,
  which is what a cutoff above every file means:
  2.6 KB **2.80**,
  8.8 KB **3.59**,
  26 KB **4.30**,
  this tree **2.83**.

The per-file rule loses at every size measured,
because the rung's write step costs several times what cloning the same file costs,
and several times what Git's own checkout of the whole tree costs:
10,001 small files written by `git checkout-index` cost 534 ms on a 190 ms `main` add,
and the same 10,001 files cloned cost 211 ms.
So moving an unchanged path from the clones to the write step is a loss even at 2.6 KB,
and the size at which cloning wins is above every size measured here.
That is the opposite of what the clean-source evidence suggested,
and it is a property of this rung's write step rather than of the clone.

The whole-tree rule declines the rung when the tracked bytes are too few for clones to win.
Declined before the proof,
from the index the rung would read anyway:

- 2.6 KB,
  26 MB:
  **0.89** (0.72 to 1.01)
- 8.8 KB,
  88 MB:
  **1.00** (0.93 to 1.52)
- 26 KB,
  260 MB:
  **1.03** (0.70 to 1.20)
- this repository's tree,
  780 MB:
  **0.96** (0.94 to 1.03),
  declined on purpose,
  so it is `main` plus one read of the index

and cloning everything on the same fixtures:

- 2.6 KB:
  **1.14** (1.09 to 1.33)
- 8.8 KB:
  **1.57** (1.12 to 1.64)
- 26 KB:
  **1.03** (1.02 to 1.57)
- this repository's tree:
  **0.77** (0.71 to 0.83)

So the crossover sits between a tree of 260 MB,
where cloning is a loss,
and this repository's 780 MB,
where it is a gain of about a quarter.
A threshold of about 400 MB of tracked bytes,
a mean of about 30 KB over 10,001 to 24,000 files,
is where these points put it;
pinning it needs the same measurement at sizes between them,
which this round did not take.

Declined after the proof,
which is how a rule placed in the source check would behave,
the same fixtures cost 1.38,
1.30 and 1.23 of `main` at 2.6,
8.8 and 26 KB,
and 1.33 on this repository's tree:
the proof's own cost is then the loss,
which is why the rule belongs at the rung's choice.

## Where the design text changes

`package/git-policy/cli/SPEC.md`,
under "Rung 2 needs a clean source",
first sentence,
from:

> Rung 2 needs a clean source on btrfs,
> XFS,
> overlayfs or ZFS,
> with the source and the destination in one clone domain.

to:

> Rung 2 needs a source whose tracked files it can prove and whose clone domain holds the destination,
> on btrfs,
> XFS,
> overlayfs or ZFS,
> with the source and the destination in one clone domain.
> A source whose every tracked file equals its index and whose index equals the tree of `HEAD` is cloned whole,
> and a source with uncommitted changes clones every tracked path the proof still settles while Git writes the rest;
> see "Rung 2 on a source with changes".

and,
after the paragraph that ends "and the same tree difference",
a new subsection:

> Rung 2 on a source with changes.
> A path is cloned when its index entry is plain,
> at stage 0 with no extended flags and no assume-unchanged bit,
> names the tree's mode and object,
> is a file or a link,
> and its file equals the entry by type,
> size,
> inode,
> owner and both times to the nanosecond,
> with a modification time older than the index file.
> Every other tree path is written by one `git checkout-index --stdin -z --index` in the new worktree,
> which runs the checkout's conversion filters.
> The new worktree's index is built from the tree of its `HEAD`,
> with the stat fields of the files the clones made,
> the fields the source's index recorded for a link,
> and none for the paths Git writes.
> The rung is refused,
> and Git's checkout makes the worktree,
> for an operation in progress,
> an index or object database the readers cannot read,
> a sparse checkout,
> an unmerged,
> intent-to-add,
> skip-worktree or assume-unchanged entry,
> a submodule,
> a tree entry that is neither a file nor a link,
> a changed or untracked `.gitattributes`,
> and a path reached through a link.
> An untracked file of the source is no refusal:
> the new worktree takes only the tree's paths.
> The worktree is proven after the tree difference with `git --no-optional-locks status --porcelain` printing nothing,
> and `git --no-optional-locks diff-index --cached --quiet <new HEAD>` answering zero.

`doc/planning/cli-git-btrfs-worktree.md`,
step 2's rung list,
from:

> - Rung 2,
> reflink:
> needs a filesystem with `FICLONE` (btrfs,
> XFS with reflink,
> overlayfs over either) and a clean
> source worktree on the same filesystem.
> The measurement says it saves disk,
> not time,
> so it is optional.

to:

> - Rung 2,
> reflink:
> needs a filesystem with `FICLONE` (btrfs,
> XFS with reflink,
> overlayfs over either) and a source
> worktree on the same filesystem whose tracked files the rung can prove;
> a source with uncommitted changes clones
> the paths the proof still settles and lets Git write the rest.
> The measurement says it saves disk,
> not time,
> so it
>   is optional.

and,
in its checks,
the clean-source bullet,
from:

> - The source must be clean,
> checked without spawning Git where possible:
> ...
> and no untracked file that is not
>   ignored.

to:

> - The source must be provable,
> checked without spawning Git where possible:
> ...
> and,
> for rung 1 only,
> no untracked
> file that is not ignored,
> because a snapshot carries the whole directory and a partial clone takes none of it.

The waiver record `doc/decision/cli-git-zfs-clone-rung-dirty-source-waiver.md` ends with the proposal it names;
if this one is accepted,
that record says how.

## Options for the owner, with their measured cost

Every figure is the dirty source on this repository's tree unless it says otherwise,
as a ratio to `main` in the same round over five rounds,
and the clean-path figure is the same tree over five rounds.
The small-tree figures are the three synthetic fixtures,
three rounds each.

- **A.
  Partial clone for every source with changes,
  no size rule.**
  Dirty path 0.74 (0.67 to 0.80);
  clean path unchanged,
  0.62 (0.55 to 0.78) against a same-binary control of 0.85.
  It removes the waived cost and turns it into a gain on this tree,
  and the rung stops refusing a dirty source.
  It keeps the loss `H1` found on trees of small files:
  1.14 at 2.6 KB,
  1.57 at 8.8 KB,
  1.03 at 26 KB,
  dirty;
  1.33 and 1.18 clean by the integration pass's numbers.
  Nothing about the design is left open except the write step's ceiling and the split of the source check.
- **B.
  Partial clone with the whole-tree size rule,
  declined before the proof.**
  Dirty path 0.74 on this tree and 0.89,
  1.00,
  1.03 on the three small fixtures;
  clean path unchanged on this tree,
  and on the small trees the rung is cheaply declined,
  so the add is `main`'s.
  It costs one read of the index and one sum,
  about 25 ms at this tree's 24,389 entries,
  measured as the 0.96 of the declined arm.
  It needs a threshold;
  these points put it near 400 MB of tracked bytes,
  and the crossover is bracketed by 260 MB and 780 MB rather than pinned.
  It is the only option measured that is never slower than `main` on any fixture.
- **C.
  Partial clone with the per-file size rule.**
  Dirty path 1.05 at a 4 KB cutoff and 1.90 at 32 KB on this tree,
  and 2.80 to 4.30 on the small fixtures when every unchanged path is written;
  the clean path would pay the same write step.
  It is worse than A everywhere measured,
  and it does not reach `main` anywhere,
  because the rung's write step costs three to four times Git's own checkout for the same files.
  It could only pay with a write step as cheap as Git's checkout,
  which this round did not build.
- **D.
  No partial clone:
  keep step 3 as it is under the waiver.**
  Dirty path 1.14 (1.07 to 1.16) on this tree,
  and by the same shape on the small trees,
  which is `main` plus the proof;
  clean path 0.81 here and,
  by the integration pass,
  1.33 and 1.18 on the 2.6 KB and 8.8 KB trees.
  It costs nothing new and leaves the waiver,
  and the loss `H1` found,
  in place.

Ranking:
**B > A > C > D.**

- B over A:
  B is the only option measured that is never above `main`,
  and it pays for that with one index read and a threshold;
  A leaves a measured loss of 1.14 to 1.57 on trees of small files,
  which is the shape `H1` already blocked the branch for.
- A over C:
  on this tree A is 0.74 and C is 1.05,
  and on every small fixture A is faster than C,
  because the write step is the expensive part of the rung,
  so a rule that moves work into it loses.
- C over D:
  C at least removes the dirty-source cost on this tree,
  1.05 against 1.14,
  and keeps every correctness property A has;
  D keeps a cost the project has already recorded as a waiver.

## What this leaves owed

- The gate's whole suite in the gate image ran at this branch's tip:
  9,291 tests,
  9,287 passed,
  4 failed and 28 skipped,
  and the four failures are none of this rung's:
  two `repair::package_repair_prewarm_job` controls whose fixture cannot resolve `serde` offline in that image,
  and two commit-service controls that find a `cli-git-lock-holders` entry a control does not expect.
  The gate is what a merge runs again,
  so these figures name what this round saw and not a green gate.
- The standing notice's control over the steps an add enters failed on the first gate run,
  because the partial clone's five steps had no sentence;
  the sentences are written and the control passes.
- `worktree_clone.rs` was already over the package's 300-code-line limit before this change,
  and the four functions this prototype added to it belong in a sibling when the rung is written for a merge.
- A ladder ceiling for `git checkout-index`:
  the prototype runs it through the captured runner,
  which bounds it,
  but `ladder_step_ceilings.rs` does not classify it,
  so a merge must give it `LADDER_CHECKOUT_CEILING` or a class of its own.
- The split of the source check,
  so the clone rung stops paying the untracked scan and the trash question:
  measured above at about 117 ms together,
  and not built.
- The one-`git status` classification:
  its parts are measured,
  the combination is not.
- A cheaper write step,
  if the per-file rule is ever wanted again.
- A pin on the threshold between 260 MB and 780 MB,
  and the same measurement on XFS,
  overlayfs and btrfs,
  which this host cannot take.
- The size rules are environment variables of the prototype.
  A shipped rule is a design decision with a recorded threshold,
  not a variable.
