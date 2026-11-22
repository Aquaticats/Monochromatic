# cli-git clone rung: cloning the unchanged files of a dirty source

## Status

Proposal,
rewritten on 2026-11-21 on measurements taken with the conversion fence in place.
**Unapproved and not final**:
only the owner declares work complete.
Nothing here is in `main`;
the design is proposed,
a prototype measures it on `main`'s fenced clone rung,
and the last section asks for a decision.

The owner answered "Yes,
design and measure it (Recommended)" on 2026-11-20,
recorded in `doc/decision/cli-git-zfs-clone-rung-dirty-source-waiver.md`,
which also names when the waiver it granted ends.

Three things changed since the version this branch carried on 2026-11-20,
and this document is written on them rather than patched around them:

- The conversion fence is in `main`.
  Its proposal is `doc/planning/cli-git-clone-rung-conversion-fence.md`,
  the project chose its option B on 2026-11-20T23:27:33-05:00,
  and `c423010a4e` merged it with ZFS plan step 3 on 2026-11-21.
  So a clone no longer carries a path whose checkout would convert it:
  the fence hands that path to Git.
  The gate review's finding `H1` is answered by that,
  and this proposal builds no fence of its own.
- The prototype is on that rung,
  not beside it.
  `ec4dd92ae2` merges `main` into this branch and resolves the prototype onto the fenced steps,
  and every number below is measured with both changes compiled together.
- A dirty source has a different opponent than it had.
  On `main` as it is,
  a clean add takes the fenced clone rung on ZFS,
  and a dirty add still takes Git's own checkout.
  The two are compared separately below,
  and which one the waiver's sentence means is said in "Which comparison the waiver means".

`NQH` holds for every figure here:
no quiet host was waited for,
the control shares the measured binary,
and every round's arms are adjacent in time.

## What the rung does today

Rung 2 clones the tracked files of a source and installs an index.
On `main` as it is,
that rung has four parts:

- the source must pass the clean proof,
  so a source with any uncommitted change ends in Git's own checkout having paid it;
- the conversion fence reads the tracked paths' attributes (`git check-attr`),
  decides its decline before anything is copied,
  and decides each path beside the copies with one `git ls-tree -r -l` and Git's own conversion rules:
  a path whose checkout could write other bytes than its blob's is written by Git;
- the index is written under a name of its own,
  `index.pending`,
  with no stat data for a path Git still has to write;
- Git writes those paths through that name with one `git checkout-index --force -u -z --stdin`,
  and only then does the file take the worktree's name `index`.

`package/git-policy/cli/SPEC.md`,
"Rung 2 needs a clean source",
states the fence and the waiting name at length;
`doc/planning/cli-git-clone-rung-conversion-fence.md` records why each step is safe when the other side fails or is
killed.
The parts this proposal changes are the first and the third:
a dirty source no longer ends in Git's checkout,
and the waiting index is built from the tree instead of from the source's index.

## What a partial clone does

The new worktree holds what Git's own `git worktree add` makes:
the tree of the new `HEAD`,
written into an empty directory.
A tracked path whose file still equals the entry that names it is cloned;
every other path is written by Git.
The rung therefore keeps its job,
which is making the files,
and changes only which files it takes.

Two decisions decide one path,
and a clone carries it only when both say so:

- the split asks whether the file is still the one its index entry recorded,
  which is the question the whole rung's clean proof asks of every path before it clones any;
- the fence asks whether Git's checkout would write other bytes there,
  which is the question the clean path already answers for a whole clean source.

A path the split carries and the fence refuses is copied and then overwritten by Git's write,
which is what the clean rung does with such a path today;
the write is the fence's own one run,
so a partial clone adds no second answer and no second conversion path.

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
taken in process,
and they are the whole rung's own (`worktree_clone_partial_split.rs`,
`the_file_is_the_entry`).
The fence then decides the same path as it decides one of a clean source:
a `filter`,
an `ident`,
a `working-tree-encoding`,
a `text` value Git does not name,
or a rule whose checkout writes carriage returns sends it to Git,
and so does a recorded size that differs from the blob's.
A cloned path's file equals its entry exactly,
so the size the fence compares is the file's own,
not a stale record:
the split's proof is what makes the fence's size argument exact on a dirty source.

### The paths Git writes

Git writes,
with one command (below),
every tree path the clone does not take,
and every path the fence hands it:

- a path the index records with another object name or mode than the tree names,
  which is a change staged in the source;
- a path the index lacks,
  which is a deletion staged in the source;
- a path whose file is missing from the source,
  which is a deletion not staged;
- a path reached through a link where a directory was,
  which the clone cannot open without following the link,
  so Git writes the tree's bytes there;
- a path whose file differs from its entry by size,
  inode,
  owner or either time,
  including a file only touched;
- a path whose type or executable bit changed;
- a path whose entry is racy,
  or records a zero nanosecond count a Git without nanosecond support wrote;
- a path the fence converts,
  by the rules listed above.

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
- a source that is a linked worktree with uncommitted changes:
  the snapshot's `restorable_here` refuses it,
  because a linked worktree's index lies outside the directory a snapshot carries,
  and the clone rung shares that refusal today
  (`worktree_snapshot_source.rs:694`).
  A partial clone reads `git_dir/index`,
  which a linked source has,
  so lifting this needs a check of its own and is owed,
  not designed here;
- an unmerged entry,
  an intent-to-add entry,
  a skip-worktree entry and an assume-unchanged entry;
- a submodule,
  in the index or in the tree;
- a tree entry that is neither a file nor a link;
- a `.gitattributes` file that is changed or untracked in the source,
  which the fence refuses before any copy;
- the fence's decline:
  more than a quarter of the tracked paths and more than a thousand of them converting,
  which ends the rung before anything is copied.

A source a reset could not undo,
and a source whose conversion settings differ between the two worktrees,
are refused by checks the rung already had,
and a partial clone inherits them unchanged.

### Untracked files

Today's clean proof refuses a source with an untracked file that no ignore rule covers,
and a dirty source reaches the clone rung without that refusal because the clean proof is not the one a dirty source
takes.
The rule's reason is not a clone's:
`worktree_clean.rs`'s module doc says a snapshot or a clone carries the bytes of the source's files,
and it names the removal path too,
which cannot undo what a subvolume ioctl removed.
The clone rung took the refusal over when it arrived in `faede9969b`
(2026-11-02T04:08:28-05:00),
whose message says the two rungs' refusals share one set of checks so the clean check starts one Git run for both.
A partial clone carries no untracked file:
its paths come from the tree,
and the new worktree's ignored state arrives through the ladder's own ignored-state copy,
as it does after Git's checkout.
**A partial clone does not need the untracked rule.**
What it still pays is the scan,
because the ladder reads it for every add,
and what that costs is in "The step split".

### The other states the question names

- `.gitattributes` changed or untracked in the source:
  refused by the fence before any copy,
  as it is for a clean source.
- A filter or text conversion:
  the fence answers,
  and the paths it takes are written by Git inside the one `git checkout-index`,
  which runs the checkout's filters exactly as Git's own checkout does.
  A cloned path is one whose checkout writes its blob,
  so no conversion class is left to the clone.
- `core.autocrlf` and `core.eol`:
  read by the fence from the source's configuration,
  and the two worktrees' conversion settings are compared before the copies.
- An ignored,
  untracked `.gitattributes`:
  invisible to the fence's untracked listing,
  because it uses `ls-files --others --exclude-standard`,
  which hides it,
  while Git still reads its rules.
  This is inherited from the clean rung and is the fence's own limit
  (`doc/planning/cli-git-clone-rung-conversion-fence.md`,
  "Limitations"),
  not a partial clone's.
- A file cloned within the wrapper's pool transaction timeout of being written waits for the pool
  (`README.md`,
  the ladder's item 2).
  The whole-rung clone has the same limit;
  the partial clone writes only the files it does not clone,
  so a source that was just pulled can show it here too.
- A submodule:
  the whole rung is refused.
- A symbolic link:
  cloned as a link with its target,
  at the source's times,
  and its index entry keeps the fields the source's own index recorded.
  A link the fence takes is written by Git like any other path.
- A skip-worktree entry:
  the whole rung is refused.

### The new worktree's index

The index cannot be the source's:
a staged path records another object,
a staged deletion is missing from it,
and a staged addition names a path no tree has.
So the rung builds one from the **source's** `HEAD` tree
(`worktree_clone_partial.rs`,
`source.clean.head_tree`),
in `worktree_clone_partial_index.rs`:

- one entry per tree path,
  in path order,
  with the tree's mode and object name and no flags;
- the stat fields of the files the clones made,
  taken from their descriptors after their last change;
- a link's entry keeps the fields the source's index recorded;
- zero stat fields for the paths Git writes,
  which is what makes Git check those files by content before it trusts them,
  and what keeps the waiting index from vouching for a file Git has not written;
- the file written as version 2 with the repository's own checksum,
  SHA-1 or SHA-256.

It is written under the fence's own waiting name by
`worktree_clone_index::install_the_index_beside_the_write`,
so the order the clean rung already has is kept:
`index.pending` holds the index,
Git's tree difference and write are pointed at it with `GIT_INDEX_FILE`,
and `PendingIndex::publish` gives it the worktree's name only after that write.
A kill between the copies and the publication leaves no index at all,
which is the unfinished shape the recovery already names.

A new `HEAD` that differs from the source's is brought in after the clones and before the write by the ladder's
`git read-tree -m -u <the source's HEAD> <the new HEAD>`
(`worktree_ladder_target.rs`,
`apply_difference_with_index`),
against the waiting index.

### Which Git command writes the changed files

The fence's own command,
once:

```sh
printf 'a.txt\0bin/tool.sh\0' | git checkout-index --force -u -z --stdin
```

run in the destination with `GIT_INDEX_FILE` naming the waiting index and `GIT_WORK_TREE` naming the new worktree,
with the paths NUL-terminated on standard input.
The rung sends the union of the split's paths and the fence's,
in path order,
and drops the ones the index no longer holds after the tree difference
(`checkout_index_through_the_index_at`),
because `git checkout-index` fails the whole run on a path its index lacks.
`--force` is what makes Git write a path the source's own bytes still sit at,
which is exactly a path the fence took from a clone;
`-u` records the stat data of what it wrote in the waiting index,
so the worktree's first `git status` compares rather than rehashes.
Naming the paths on standard input keeps a source with many changed paths out of the argument list.
A run that fails ends the rung,
and the ladder's fallback completes the worktree with Git's own checkout against the waiting index.

### What proves the result

The rung no longer runs a `git status` proof of its own.
The prototype carried one,
and this round dropped it when the prototype moved onto the fenced rung:
the shipped rung's guarantee is the index it writes and the fence's write,
the proof cost 44 ms of every add,
and the earlier review had already shown it cannot see the bytes of a path the clones made,
because the index records those clones' own stat data.

What stands in its place is what the whole rung already relies on:

- the split's per-path proof,
  which is Git's own fast-path comparison taken in process;
- a second,
  differently built answer for the class where a mistake is worst:
  the rung asks Git in the source for the paths staged against the source's `HEAD`
  (`git diff-index --cached --name-only -z <the source's HEAD>`),
  and refuses when one of them is among the clones
  (`staged_paths`,
  `a_staged_path_among_the_clones`),
  which the unit control `a_staged_path_among_the_clones_is_refused` covers;
- the index built from the tree,
  not from the source's index,
  so a staged path cannot reach the new worktree through the index either;
- the tree difference,
  Git's own write,
  and the publication order the fence already had.

The end-to-end evidence is in "Correctness",
where every worktree of every measured round and of the seeded sample is compared byte for byte with Git's own
checkout.

## The prototype

Branch `docs/planning-cli-git-clone-rung-partial-clone-of-a-dirty-source`,
commits marked `PROTOTYPE FOR MEASUREMENT, NOT FOR MERGING`,
and `ec4dd92ae2` is the merge that put it on the fenced rung.

- `worktree_clone_partial_split.rs`:
  which tree paths a clone takes and which Git writes,
  and the refusals above,
  in process,
  with no Git run.
- `worktree_clone_partial_index.rs`:
  the index built from the tree.
- `worktree_clone_partial.rs`:
  the step,
  the fence's answer beside the copies,
  the union of the written paths,
  the index under the waiting name,
  and the second listing of the source's staged paths,
  plus two size rules that exist only to be measured,
  `CLI_GIT_PROTOTYPE_CLONE_MIN_BYTES` and `CLI_GIT_PROTOTYPE_DECLINE_BELOW_TOTAL_BYTES`.
- `worktree_clone.rs`:
  an entry point that clones a chosen entry set beside the fence
  (`clone_these_bytes_beside`),
  and the one-turn path takes entries rather than re-reading the source's index
  (`worktree_clone_entries::clone_with_entries`).
- `worktree_ladder_finish_clones.rs`:
  a dirty source takes the partial clone in the fence's beside shape;
  a clean source takes the whole rung's steps unchanged.
- `worktree_ladder_run.rs`:
  a source with changes is no longer a refusal of the clone rung,
  and the prototype's whole-tree rule,
  when set,
  declines the rung before any proof is paid.

The controls are 27 tests in three sidecar files
(`worktree_clone_partial_tests.rs`,
`worktree_clone_partial_split_tests.rs`,
`worktree_clone_partial_index_tests.rs`),
run as `cargo test --locked --offline --lib -- worktree_clone_partial` with `CLI_GIT_BTRFS_ABSENT_ACKNOWLEDGED=1`
(`cargo nextest` is not installed on this host;
the gate image's nextest run is the authority).
All 27 pass on this host,
and the reflink-needing ones really ran,
because this host's fixture root is on ZFS and can clone:
nine tests in `worktree_clone_partial` alone,
including `the_unchanged_paths_are_cloned_and_the_changed_ones_written`,
`the_index_records_the_files_the_clones_made`,
`a_path_a_conversion_rule_reaches_is_written_by_git`,
`a_new_head_that_differs_is_brought_in_after_the_clones` and
`the_flow_runs_where_no_file_can_be_cloned`.

The earlier round's four red proofs
(the stat verdict accepting every file,
the object-name comparison dropped,
the source's index installed,
and the count of written paths)
were taken against the pre-fence code;
this round did not re-plant them,
and the guards they proved are the split's own,
which this round did not change.

## The measurements

### Method

- Host:
  this machine on 2026-11-21 and 2026-11-22,
  `nproc` 12 and 32 processors in `/proc/cpuinfo`,
  under other sessions' load:
  the one-minute load average was 6 to 15 across the shapes,
  and the runs never waited for it to fall (`NQH`).
- Scope:
  one `systemd-run --user --scope` per batch:
  CPUQuota 600%,
  MemoryMax 20G,
  TasksMax 512,
  IOWeight 1,
  so this session has the least IO weight beside the others.
  A 6-CPU quota makes the clone engine's slices 5 to 8 on this tree,
  as the add's own record says.
- Filesystem:
  every fixture repository and every destination under the home dataset,
  on ZFS,
  source and destination in one dataset,
  so rung 2 applies to every clone arm.
  `zfs_bclone_enabled 1`,
  `zfs_bclone_wait_dirty 1`,
  `zfs_txg_timeout 5`.
- This repository's tree:
  a throwaway repository of this repository's tracked files at `ec4dd92ae2`:
  24,561 files,
  735,213,873 tracked bytes,
  with the `filter=lfs` lines of `.gitattributes` stripped,
  as the earlier rounds stripped them,
  and with the images kept as their own blobs.
  The fixture is built once and copied per shape with `cp -a --reflink=always`.
- The synthetic trees:
  10,001 files of 2,600,
  8,800 and 26,000 bytes in ten flat directories,
  the shapes the waiver names,
  with one file's bytes committed each.
- Source state held fixed over all rounds of a shape:
  usual is three tracked files rewritten and three files nobody tracks,
  many:N is N tracked text files each with a line appended plus the three untracked,
  clean is neither,
  and `small:<size>:dirty` is three tracked files rewritten plus the three untracked.
  `sync --file-system`,
  1.7 s and `update-index --really-refresh` follow the state,
  and a run whose source does not show exactly the state's paths stops before any round.
- Builds:
  release,
  each from a `git archive` copy of its commit with
  `cargo build --locked --offline --release --bin monochromatic-git-policy-cli`:
  the prototype at `ec4dd92ae2`
  (sha256 `96329e6b89109cea9d452d73f6943c1a533019b4ee4ccf20efad3d839f30b105`),
  `main` at `ad35d82537`
  (sha256 `ff2dff2b95c34fcd3ccd7c0e950a19be048296ada5df12dbbfabe4d9af6d49ea`).
- Arms:
  the prototype,
  the same binary under another name (the control),
  `main`,
  and Git's own checkout.
  Each is reached as plain `git` through a PATH of its own,
  with its own `HOME` and `XDG_RUNTIME_DIR`.
  Git's own checkout is `main`'s own binary under a form the ladder does not read,
  `git --no-optional-locks worktree add`,
  which the wrapper forwards as the person wrote it and which its record names `route: plain`.
  On the small shapes a fifth arm runs the prototype with the whole-tree rule set to 400,000,000 tracked bytes,
  which is option B,
  and its record names `route: Git's checkout`.
- Rounds:
  the order rotates one place per round and reverses on even rounds,
  so each arm takes each position,
  and the identical pair is adjacent in half the rounds and separated in the other half;
  300 ms and `sync --file-system` after the state separate the arms of a round,
  and nothing else runs beside them.
  Counted rounds:
  eight for the usual state,
  six for the clean state,
  six at 300 and six at 3,000 changed paths,
  six for the 2.6 KB tree dirty,
  and four for each clean small tree,
  after one warm-up round each.
- Verified,
  outside the timed window,
  for every worktree every arm made:
  `git status --porcelain` empty,
  and `diff --recursive --no-dereference --exclude=.git` against Git's own checkout's worktree of the same round
  empty.
  Every arm of every round passed both.
- Every prototype round took "a partial clone" on the dirty shapes,
  every whole-rung round took "clones",
  every `main` round on a dirty source took "Git's checkout",
  and every Git's-checkout round took "plain",
  so the arms are what they say.
- Records:
  each shape's round record and log are in
  `~/temp/agent/clone-rung-partial-clone-on-the-fenced-rung-2026-11-21/scratch/keep/`,
  and the two earlier batches kept there are marked invalid:
  one ran before the fixture's LFS attributes were handled,
  the other synced the whole filesystem before every arm and its spread was the sync's.

### This repository's tree, three changed paths

The usual dirty state,
ratio to `main` in the same round,
median with the range over the counted rounds,
and `main`'s wall time in brackets:

- `main` 1,533 ms (1,421 to 4,149);
- prototype 1,012 ms,
  **0.64** (0.23 to 2.84);
- its identical control 1,077 ms,
  0.67 (0.41 to 0.84);
- Git's own checkout 1,454 ms (1,253 to 2,176),
  and the prototype is 0.75 of it (0.45 to 2.02).
- The identical pair differs by 1.04 in the median and 0.22 to 1.79 round by round.

The range's top is one stall:
the prototype took 4,163 ms in round 6 while its control took 920,
and its 2,213 ms round is not one,
because the control took 3,424 in the same round.
The identical pair agrees at 1.04 in the median with a round-to-round range of 0.22 to 1.79,
so one round resolves nothing and the median of eight is what these ratios rest on,
to one significant figure.

Against Git's own checkout the same add is 0.75,
and `main` is 1.14 of Git's own checkout (0.71 to 3.19),
which is its clean proof paid and then the checkout.

### This repository's tree, clean

- `main` 861 ms (753 to 997),
  which is the fenced clone rung;
- prototype 824 ms,
  0.95 (0.83 to 4.89);
- its identical control 814 ms,
  0.97 (0.76 to 1.10).
- Git's own checkout 1,317 ms (1,177 to 1,993),
  so the fenced whole rung is 0.66 of Git's own checkout here
  (0.38 to 0.79).

The prototype and `main` run the same code on a clean source,
because the partial clone is the dirty source's path and nothing else,
so this line is a control and not a result:
the two agree inside the pair's spread,
and no clean add changes when this proposal lands.

### This repository's tree, many changed paths

- 300 changed paths:
  `main` 1,450 ms (1,326 to 1,664);
  prototype 1,035 ms,
  **0.74** (0.64 to 1.28);
  control 1,134 ms,
  0.79 (0.67 to 1.01);
  Git's own checkout 1,265 ms (1,227 to 1,475);
  the pair 1.07.
- 6,000 changed paths,
  the shape the gate review found the earlier design losing:
  `main` 1,411 ms (1,367 to 2,069);
  prototype 1,208 ms,
  **0.86** (0.57 to 1.01);
  control 1,341 ms,
  0.90 (0.65 to 1.04);
  Git's own checkout 1,231 ms (1,120 to 1,485);
  the pair 1.12.
- The write step's own cost,
  `git checkout-index`:
  13.5 ms at three paths,
  32.8 ms at 300 and 379.9 ms at 6,000,
  which is 109 microseconds a path at 300 and 63 at 6,000.

The earlier round measured this shape at 1.54 of `main` with a control of 1.44,
and this round measures 0.86 with a control of 0.90,
in every counted round at or below 1.01 of `main`.
The difference is not the write step's price per path,
which is the same order in both rounds,
but two things the fenced rung changed:
the write is one `git checkout-index` through the waiting index instead of a proof and a write around it,
and the 44 ms `git status` proof of the worktree is gone.

### The small-file trees the waiver names

Clean,
10,001 files,
one directory level of ten parts,
ratio to `main` in the same round:

- 2.6 KB,
  26,000,000 tracked bytes:
  `main` 210 ms,
  prototype 243 ms,
  **1.17** (0.81 to 1.38),
  its control 196 ms,
  0.98 (0.87 to 1.03),
  Git's own checkout 179 ms,
  and the declined arm 140 ms,
  0.65 (0.59 to 0.96).
- 8.8 KB,
  88,000,000 bytes:
  `main` 201 ms,
  prototype 209 ms,
  **1.01** (0.89 to 1.28),
  control 211 ms,
  1.03 (0.98 to 1.13),
  Git's own checkout 193 ms,
  declined 173 ms,
  0.86 (0.79 to 0.88).
- 26 KB,
  260,000,000 bytes:
  `main` 224 ms,
  prototype 254 ms,
  **1.10** (0.97 to 1.17),
  control 234 ms,
  1.02 (0.88 to 1.15),
  Git's own checkout 208 ms,
  declined 198 ms,
  0.81 (0.70 to 1.01).

Two things are read from this:

- `main` is already the clone rung here,
  so the prototype and `main` are the same code and their ratios are the pair's spread,
  not a gain or a loss:
  the widened small-file cost is inside `main` now,
  and no part of this proposal touches it.
- Where the whole-tree rule declines,
  the add is Git's own checkout and pays none of the rung:
  0.65 to 0.86 of `main`,
  below `main` because the decline happens before the source check that `main` still pays.

The 2.6 KB tree with three changed paths is where the choice bites:

- `main` 246 ms (170 to 313),
  which is Git's own checkout;
- prototype 316 ms,
  **1.44** (0.83 to 1.69);
- its identical control 318 ms,
  1.45 (0.83 to 1.69);
- Git's own checkout 198 ms;
- the declined arm 222 ms,
  0.91 (0.67 to 1.37);
- the pair 1.02.

So a dirty source of many small files is where cloning loses,
by about 40 per cent of `main`'s add,
and where option B declines it the add is 0.91 of `main`.

### The step split

Medians of the dirty prototype add's own record,
this repository's tree,
in ms,
with `main`'s beside it where it has the step:

- the ladder's own checks and reads:
  `checkSource` 37.4 (main 38.3),
  `git ls-files` 94.2 (116.8),
  `git check-ignore` 50.1 (49.6),
  `git status` 43.6 (46.3);
- the fence:
  `git check-attr` 37.8,
  `git ls-tree` 239.8,
  `conversionFence` 22.9 and `conversionBesideTheClone` 29.7,
  whose wait overlaps the copies;
- the partial clone's own work:
  `partRead` 29.5,
  `partClassify` 85.2,
  `partStaged` 0.02,
  `partIndex` 7.1;
- the copies:
  `cloneEntries` 144.0;
- the write:
  `writeConversions` 7.3 and `git checkout-index` 13.5;
- the rest:
  `copyIgnored` 37.0,
  `finishRung` 13.7,
  `gitAdd` 3.6.
- `main`,
  the same shape:
  `gitAdd` 1,212.7.

The fence's `git ls-tree` is the largest single item in a partial clone's add,
about a quarter of it,
and it is the fence's cost,
not the partial clone's:
`main` pays it on every clean add,
and the faster size reader the project accepted as a follow-up
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
step 10's notes)
would remove most of it.
The split's own work is 115 ms,
the copies 144 ms with the fence's reads beside them,
and the write 21 ms.

### Correctness

- The 27 class tests pass,
  and all of them really ran their reflink paths here.
- A seeded differential sample,
  seed 11,
  150 combinations,
  two checks per combination,
  300 of 300:
  each combination is a fresh repository of twelve tracked paths with a link,
  an executable,
  an empty file and a file whose name is not ASCII,
  with or without a `*.txt text eol=crlf` rule,
  and two to four states drawn from nineteen
  (unchanged,
  edited with another size,
  edited with the same size,
  edited with its times restored,
  touched only,
  mode changed unstaged and staged,
  staged content,
  staged deletion,
  unstaged deletion,
  staged addition,
  intent-to-add,
  skip-worktree,
  assume-unchanged,
  link retargeted,
  directory replaced by a link,
  racy entry,
  untracked,
  ignored untracked).
  Each is added twice,
  once through the prototype and once as Git's own checkout,
  and the two worktrees are compared with `diff --recursive --no-dereference` and by `git ls-files --stage`,
  while the prototype's worktree must also be clean by `git status` and its index must equal its `HEAD`'s tree.
- Of the 150,
  74 took the partial clone and 76 ended in Git's checkout,
  which is the refusal list above firing
  (skip-worktree,
  assume-unchanged and intent-to-add are three states of the sample,
  and any combination holding one ends the rung):
  every one of the 76 is Git's own checkout and so equal to its reference by construction,
  and the 74 are the rung's own result.
- The earlier round's four red proofs stand for the split's guards,
  and this round did not re-plant them.

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

> Rung 2 needs a source whose tracked paths it can prove and whose clone domain holds the destination,
> on btrfs,
> XFS,
> overlayfs or ZFS,
> with the source and the destination in one clone domain.
> A source whose every tracked file equals its index and whose index equals the tree of `HEAD` is cloned whole,
> and a source with uncommitted changes clones every tracked path the proof still settles while Git writes the rest;
> see "Rung 2 on a source with changes".

and,
after the paragraph that ends "a `git -C <path> status` for any other",

> Rung 2 on a source with changes.
> A path is cloned when its index entry is plain,
> at stage 0 with no extended flags and no assume-unchanged bit,
> names the tree's mode and object,
> is a file or a link,
> its file equals the entry by type,
> size,
> inode,
> owner and both times to the nanosecond,
> and its modification time is older than the index file,
> and the conversion fence above carries it.
> Every other tree path is written through the one `git checkout-index` above,
> whose paths are the union of the changes' paths and the fence's.
> The index the new worktree takes is built from the tree of the source's `HEAD`,
> with the stat data of the files the clones made,
> a link's entry keeping the source's fields,
> and no stat data for the paths Git writes;
> the ladder's `git read-tree -m -u` brings a different new `HEAD` in against that index,
> and the file takes the worktree's name only after Git's write.
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
> a source that is a linked worktree with uncommitted changes,
> and the fence's own decline and refusals.
> An untracked file of the source is no refusal:
> the new worktree takes only the tree's paths.
> After Git's write the worktree holds what Git's checkout holds:
> a path the clones carried is one whose file its entry proves and whose checkout writes its blob,
> and the paths a change or the fence reaches are Git's own bytes.

`package/git-policy/cli/SPEC.md:1282`,
among the snapshot's refusals,
from:

> and the clone rung,
> which is only for clean sources.

to:

> and the clone rung only where its proof cannot settle the source.

`package/git-policy/cli/README.md`,
the ladder's item 2,
from:

> 2. Clones (`FICLONE`) of every tracked file of a clean worktree,
>    with its index,

to:

> 2. Clones (`FICLONE`) of every tracked file the source's proof settles,
>    with the index the rung builds from the source's `HEAD`,
>    while Git writes the paths a change or the conversion fence reaches,

`package/git-policy/cli/src/native/worktree/worktree_snapshot_source.rs`,
the doc comment of `source_or_dirty`,
from its sentence:

> the clone rung and the other
> callers of the clean answer never see it.

to:

> the clone rung takes the same source and clones the paths its proof settles while Git writes the rest,
> and the callers of the clean answer see clean sources only.

`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
the dirty cost,
from:

> A source with uncommitted changes still pays the source checks and then ends in Git's own checkout:
> 0.80 to 1.59 times the same build of `main`,
> medians of 1.25,
> 0.93,
> 1.24 and 1.17 in its four runs,
> about 0.2 seconds.

to:

> A source with uncommitted changes clones every tracked path its proof settles and lets Git write the rest,
> so the waived cost ends where rung 2 serves the add;
> a linked worktree with uncommitted changes still pays the source checks and then ends in Git's own checkout,
> and step 11's verified dirty restore is still the snapshot's.
> The measurements that place the clone path are the partial-clone proposal's.

The waiver record `doc/decision/cli-git-zfs-clone-rung-dirty-source-waiver.md` ends with the proposal it names;
if this one is accepted,
that record says how.

## Options for the owner, with their measured cost

Every figure is a ratio to `main` in the same round.
"This round" is the runs above;
"the earlier round" is the author's and the gate review's,
taken before the fence was in `main`.

- **A.
  Partial clone for every source with changes,
  no size rule.**
  Dirty path on this tree:
  0.64 (0.23 to 2.84) with its control 0.67 (0.41 to 0.84);
  at 300 changed paths 0.74 (0.64 to 1.28) with a control 0.79;
  at 6,000,
  0.86 (0.57 to 1.01) with a control 0.90.
  Against Git's own checkout:
  0.75,
  0.80 and 1.00 on those three shapes.
  Clean path unchanged,
  because the partial clone is the dirty source's path:
  0.95 with a control 0.97,
  and `main` is 0.66 of Git's own checkout there.
  Its losses:
  a dirty tree of many small files at 1.44 with a control 1.45,
  and the fence's own costs on top of it.
  It ends the waiver's dirty half on the three shapes measured,
  in the median and,
  at 6,000,
  in every counted round;
  it does not touch the widened small-file half,
  which is now `main`'s own cost;
  and a dirty source of many small files is slower than `main`.
- **B.
  Partial clone with the whole-tree size rule,
  declined before the proof.**
  On this tree it clones,
  so A's numbers are its.
  On the 2.6 KB dirty tree it declines and takes Git's checkout:
  0.91 (0.67 to 1.37) of `main`,
  where A is 1.44.
  On the clean small trees it declines at 0.65,
  0.86 and 0.81 of `main`.
  It needs a threshold,
  these points put the crossing between 260 MB and 780 MB of tracked bytes,
  and the unit may not be bytes:
  every fixture here holds 10,001 or 24,561 files.
  Its scope and placement are part of the decision:
  at the clone rung's choice,
  after the snapshot rung is declined,
  on the filesystems rung 2 serves.
  The prototype evaluates it in `choose_rung` before the snapshot is considered,
  so on a btrfs host with a subvolume source it would also turn off the snapshot rung,
  and it applies to clean sources as well,
  while every figure above is ZFS.
  That placement is a measurement artifact,
  not the design.
- **C.
  Partial clone with the per-file size rule.**
  Not rebuilt this round.
  The earlier round measured it at 1.05 (0.96 to 1.10) with a 4 KB cutoff and 1.90 (1.43 to 2.11) at 32 KB on this
  tree,
  against the same run's clone-everything arm at 0.77 with a control 0.76,
  and writing every unchanged path cost 2.80,
  3.59 and 4.30 on the small trees.
  This round gives the reason to keep that verdict:
  the write step costs 63 microseconds a path at 6,000 paths,
  against a clone of the same file,
  so moving an unchanged path from the clones to the write step loses at every size measured.
  Its finding is "this write step loses",
  not "no per-file rule can win":
  a write step as cheap as Git's checkout was not built.
- **D.
  No partial clone:
  keep step 3 under the waiver.**
  A dirty add is `main`'s,
  which is Git's own checkout plus the clean proof:
  1.00 by definition and 1.14 of Git's own checkout (0.71 to 3.19) on this tree,
  1.20 (0.92 to 1.65) at 6,000 changed paths and 1.27 (0.88 to 1.54) on the 2.6 KB dirty tree.
  It costs nothing new,
  leaves the waiver in place,
  and leaves the dirty source's 0.36 of `main` that A removes.
  It ends nothing.

Ranking:
**B > A > D > C.**

- B over A:
  B keeps A's gain where cloning wins and gives the dirty small tree back to Git's checkout at 0.91 of `main`,
  where A pays 1.44;
  on this repository's tree the two clone and their numbers are the same.
  B pays a threshold whose unit is not yet pinned;
  A needs no rule but leaves that loss.
- A over D:
  A removes 0.36 of `main` on this tree's usual shape,
  0.26 at 300 changed paths and 0.14 at 6,000,
  and stays at or below `main` on every counted round of those shapes,
  where D keeps `main`'s dirty add.
- D over C:
  C was measured only with a write step this round shows costs 63 microseconds a path,
  and its clean path would pay the same write step for every file below its cutoff,
  which on the small trees is 2.80 to 4.30 against D's parity.
  C therefore loses more than it gains,
  and it ends no waiver.

## Which comparison the waiver means

The waiver record's sentence is "If it is accepted and lands with a dirty add no slower than `main`'s,
this waiver ends with it".
It was written when `main` did not take the clone rung on ZFS,
so "`main`'s" meant Git's own checkout plus the wrapper's checks.
On `main` as it is,
a dirty add is still Git's own checkout,
because a dirty source fails the clean proof,
so the two readings differ only by the checks `main` pays before the checkout:
`main` is 1.14 of Git's own checkout on this tree (0.71 to 3.19),
1.20 at 6,000 changed paths (0.92 to 1.65),
and 1.27 on the 2.6 KB dirty tree (0.88 to 1.54).
The waiver's sentence is therefore read as `main`'s own add,
and Git's own checkout is reported beside it so a reader can take the stricter one.

- The dirty half:
  A ends it on this tree's usual shape (0.64 of `main`,
  0.75 of Git's own checkout),
  at 300 changed paths (0.74,
  0.80) and at 6,000 (0.86,
  1.00).
  B ends it on those shapes and on the dirty small tree where it declines (0.91).
- The widened small-file half:
  neither A nor B changes a clean add at all,
  because a clean source never takes the partial clone.
  `main` now takes the fenced clone rung for a clean source,
  so the widened cost is `main`'s own:
  it is 1.17,
  1.04 and 1.08 of Git's own checkout on the three small trees,
  and this proposal cannot end it.
  A record that ends that half has to be about the clean rung's own price,
  which is the fence's size read and the clone's per-file cost.

## The gate review's findings, answered

The review is `~/temp/agent/gate-review-of-the-partial-clone-proposal-on-sonnet-2026-11-20/keep/review.md`,
with its logs and scripts beside it.
Each finding is fixed,
recorded as a limit,
or superseded by a change `main` has since taken.

- **H1,
  a clone keeps the source's bytes where a checkout conversion would write others:
  answered by the fence,
  not by this proposal.**
  The project chose the fence's option B on 2026-11-20,
  `c423010a4e` merged it into `main` on 2026-11-21,
  and the fence now hands every path whose checkout would convert it to Git.
  A partial clone reaches the fence through the same beside-the-copies shape,
  so a path it clones is one both proofs carry,
  and the unit control `a_path_a_conversion_rule_reaches_is_written_by_git` shows the case end to end:
  a `*.txt text eol=crlf` rule over LF files,
  the path written by Git,
  the served bytes `\r\n`.
  What remains of the class is the fence's own limits,
  an ignored untracked `.gitattributes` among them,
  and they are recorded in the fence's proposal and in "Limitations" here.
- **M1,
  the proof cannot see wrong bytes in a cloned path:
  fixed by dropping the proof and by the second answer.**
  The rung no longer claims a `git status` proof it does not have;
  what it has is the split's per-path proof,
  the second staged-path listing,
  the index built from the tree,
  and the fence's write.
  The staged class has its own control and its own second computation.
- **M2,
  a dirty linked worktree is still refused:
  fixed in the text,
  the lift owed.**
  "States that still refuse the whole rung" names it with `restorable_here`,
  and the new `SPEC.md` text names it too.
  Letting the clone rung take one needs a check of its own and a measurement of its add.
- **M3,
  measurement statements the data does not support:
  superseded.**
  This round's numbers are its own,
  every figure has its identical-binary control beside it,
  the host and the quota are stated,
  and the binaries are named by commit and sha256.
  The author's 0.74 and the review's 0.84 to 0.86 are kept only as the earlier round's,
  and the comparison they belong to is named.
- **M4,
  the gain's size is not reproduced:
  answered by measuring the fenced rung.**
  This round measures 0.64 with a control of 0.67 on this tree,
  and the pair's median spread is 1.04:
  the gain is real,
  about a third,
  and its size is the host's.
  "Never slower" is not claimed;
  every ratio here carries its range.
- **M5,
  a source with many changed paths:
  measured to 6,000,
  and the loss the review found is gone.**
  At 300 paths the add is 0.74 of `main` with a control of 0.79,
  and at 6,000 it is 0.86 with a control of 0.90,
  at or below `main` in every counted round.
  The review's 1.54 came from a rung that wrote the paths through a slower shape and then proved the worktree;
  the fenced rung writes them through the waiting index and proves nothing after.
  The write step is 63 microseconds a path at 6,000,
  and no round measured a whole-tree rewrite.
- **M6,
  the ranking and the options:
  redone on this round's figures.**
  The ranking is B > A > D > C with the reasons given,
  every option says which half of the waiver it ends,
  B's scope and placement are written,
  and the threshold's unit may not be bytes.
- **M7,
  incomplete and inaccurate design text:
  rewritten.**
  The index is built from the source's `HEAD` tree,
  a path below a link is written,
  the new `SPEC.md` subsection follows the fence's paragraph,
  the stale "saves disk,
  not time" is gone,
  and the places listed are the ones `main` now has.

The review's lows:

- **L1:
  fixed** by citing `faede9969b` and the module doc separately.
- **L2:
  fixed with two controls**,
  which pass on this host.
- **L3:
  an inherited fence limit**,
  not measured this round.
- **L4:
  an inherited limit** for the pool wait,
  not measured this round.
- **L5:
  partly closed.**
  The fixture keeps this repository's own `.gitattributes` except for the `filter=lfs` lines,
  as the earlier rounds did,
  so an LFS-attributed tree's round is still owed;
  this round did establish why the fixture cannot keep those lines without an LFS object store:
  with the standard LFS filter configured,
  a repository whose blobs are the images themselves reads every image as modified,
  and without the filter configured,
  the fence refuses the source it cannot prove
  ("the lfs filter is not the standard Git LFS definition").

## Limitations

- **The fence's `git ls-tree` is a quarter of a dirty add on this tree.**
  It is 219 to 250 ms of the 1,012 ms usual add,
  and 71 to 84 ms of a 200 to 250 ms small-tree add.
  It is the fence's cost,
  `main` pays it on every clean add,
  and the faster size reader the project accepted would remove most of it.
- **A dirty source of many small files is slower than `main`.**
  1.44 (0.83 to 1.69) with a control of 1.45 on 10,001 files of 2.6 KB,
  and the whole-tree rule takes it to 0.91.
  No design here makes cloning a small file cheaper than Git's write of it.
- **A linked worktree with uncommitted changes is still refused**,
  so a session working in `.claude/worktrees/<name>` gets Git's checkout,
  not the partial clone.
- **A whole-tree rewrite was not measured.**
  The write step costs 63 microseconds a path at 6,000,
  so a source whose every path changed would write 24,561 paths through Git and pay about 1.5 s for it,
  on top of the clones and the fence;
  no round exercised it.
- **The threshold is not pinned,
  and its unit may not be bytes.**
  The crossing lies between 260 MB and 780 MB of tracked bytes on trees of 10,001 and 24,561 files,
  and file count and mean size were never varied apart in the same disk's worth of measurement.
- **The size rules are environment variables of the prototype.**
  A shipped rule is a design decision with a recorded threshold,
  not a variable.
- **An ignored untracked `.gitattributes` is still invisible to the fence's listing**,
  an inherited limit,
  not measured this round.
- **A file cloned within the pool's transaction timeout waits for the pool**,
  an inherited limit,
  not measured this round.
- **An LFS-attributed source was not measured.**
  This round established the two shapes that fixture takes and why neither is a measurement;
  the round is owed.
- **The check the source passes no longer reads the diff to content.**
  The split's proof is Git's stat trust taken in process,
  so an edit that keeps size and times is caught by the change time,
  the inode or the owner,
  and an engine that cannot see one of those would clone the edit.
  This is Git's own trust,
  not a weaker one.
- **`worktree_clone.rs` is over the package's line limit**,
  and the entry point this round added belongs in a sibling when the rung is written for a merge.

## What this leaves owed

- The gate image's whole suite at this branch's tip,
  which is what a merge runs again.
  On this host 913 of the 930 `worktree::` tests pass;
  the 17 that fail are this host's and not this rung's:
  eight create a name that is not UTF-8 and this ZFS dataset refuses it with `EILSEQ`,
  and nine are the parallelism controls that pass alone and fail in a batch under load.
- A `git checkout-index` ceiling for the ladder:
  the run goes through the captured runner,
  which bounds it,
  but `ladder_step_ceilings.rs` does not classify it.
- A round on a tree whose `.gitattributes` keeps its `filter=lfs` lines and whose blobs are LFS pointers.
- A round on a source that is a linked worktree,
  once the refusal is lifted.
- A measured threshold for the whole-tree rule,
  and the same measurement on XFS,
  overlayfs and btrfs.
- The split of the source check,
  so the clone rung stops paying the untracked scan and the trash question,
  measured at about 145 ms of a dirty add here.
- The one-`git status` classification of the source's changes,
  whose parts are measured and whose combination is not.
- A cheaper write step,
  if the per-file rule is ever wanted again.
- The four red proofs re-planted against the fenced rung,
  which this round did not do.
- The ladder's full routing control,
  with an `AddPlan` and both rungs on a clone-capable source,
  beyond `a_dirty_source_gets_the_clone_rung_where_files_can_be_cloned`.

## The decision asked for

The owner is asked to choose among A,
B,
C and D as ranked,
and the ranking's own reasons say what each buys:

- A lands the partial clone as it is measured,
  and accepts the dirty small-file loss;
- B lands it with the whole-tree rule,
  and needs the threshold,
  its unit and its placement decided;
- C is the per-file rule,
  which this round's numbers put last;
- D keeps step 3 as it is under the waiver.

Whichever is chosen,
what lands is the prototype's design on the fenced rung,
with the `SPEC.md`,
`README.md`,
plan and source-comment changes quoted above,
and with the gate image's suite green.
