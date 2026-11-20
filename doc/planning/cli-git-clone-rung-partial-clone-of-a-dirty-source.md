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

**A gate review found it not ready.**
This round answers its findings.
Claude Sonnet's gate review of 2026-11-20
(`~/temp/agent/gate-review-of-the-partial-clone-proposal-on-sonnet-2026-11-20/keep/review.md`)
found the design sound where it could attack it and the central claim wrong where a text conversion applies (its
finding H1),
the proof blind to the bytes of a cloned path (M1),
a source it did not say the rung still refuses (M2),
measurement statements its own data does not support (M3,
M4),
a shape nobody measured where the rung loses (M5),
a ranking step its numbers contradict (M6),
and incomplete design text (M7).
Its M1 to M7 and L1 to L5 are answered here,
each in "The gate review's findings,
answered".
H1 stays an open limit:
it reaches the accepted clone rung too,
so its remedy is a project decision being prepared separately,
and this round builds no fence for it.

`NQH` holds for every figure here:
no quiet host was waited for,
the control shares the measured binary,
and the arm orders are stated so the control is not always the arm after the prototype.

The work behind this proposal:

- Prototype on `docs/planning-cli-git-clone-rung-partial-clone-of-a-dirty-source`,
  marked as a prototype for measurement in every commit,
  not for merging as it is.
- Measurements on this host,
  under the load there is:
  the author's rounds against step 3's branch (`73908f4fd7`) and `main` (`392042de70`),
  and this round's against `main` at `2df9b1689b` (the same add path as `392042de70`),
  each built the same way,
  with a same-binary control beside every figure,
  and the gate review's independent rerun beside them.
- The untouched-files rule,
  the refusal list,
  the index and the proof are controlled by 26 tests,
  four of which were shown red against a planted defect each.

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
A tracked path whose file still equals the entry that names it is cloned;
every other path is written by Git.
The rung therefore keeps its job,
which is making the files by cloning them,
and changes only which files it takes.

What a clone holds is the source's own bytes for that path,
and those are the bytes Git's checkout writes **only where the source file is the conversion of its blob** under the
rules the new worktree applies:
a `text` or `eol` attribute,
or `core.autocrlf`,
makes Git's checkout write other bytes than the blob's,
and neither the index's stat data nor Git's own `status` can tell,
because the clean filter maps both forms to one blob.
This is the gate review's finding `H1`.
Rung 2 has had the limit since it existed:
a clean source whose LF files sit under an `eol=crlf` rule is cloned whole and differs from Git's checkout (`s04b`),
and the dirty path reaches the same state (`s04`);
`core.autocrlf=true` is the third form (`s04c`).
The remedy changes the accepted clone rung too,
so it waits on the project's decision,
and this proposal builds no fence for it:
a fence on the presence of `text`,
`eol` or `autocrlf` would refuse this repository,
whose `.gitattributes` begins
`* text=auto eol=lf`,
and a per-path fence needs the bytes,
which defeats cloning.

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
- a path reached through a link where a directory was,
  which the clone cannot open without following the link,
  so Git writes the tree's bytes there;
  `a_path_below_a_link_is_written` controls it and the review's `s07` equals Git's checkout.
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
- a source that is a linked worktree with uncommitted changes:
  the snapshot's `restorable_here` refuses it,
  because a linked worktree's index lies outside the directory a snapshot carries,
  and the clone rung shares that refusal today
  (`worktree_snapshot_source.rs:586` and `643` to `645`);
  the review's `s11` shows the add paying the proof and ending in Git's own checkout with `uncommitted-changes`.
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
  because its rules decide how Git converts the bytes of other paths,
  and the source's converted bytes and the new worktree's rules would have to agree;
- nothing for a path reached through a link:
  the split writes such a path instead of refusing the rung,
  which the review's `s07` shows equals Git's checkout;

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
whose message says:

> A cleanliness check built on the in-process index reader (no operation in progress,
> index equal to HEAD,
> stat fields
> equal to lstat,
> no unignored untracked file) decides,
> and every refusal falls to Git with a log line.

The message names the subvolume destination and the removal,
not the callers;
it is the module doc that names them and gives the two reasons:

> A snapshot or a clone carries the bytes of the source's files,
> so any change the source has not committed would reach the new worktree,
> and a removal by the subvolume ioctl cannot be undone.

So the rule exists for two callers:
the snapshot rung,
which copies the whole directory and would carry untracked files into the new worktree,
and the removal path,
which must never destroy a file nobody committed.
It is not a clone rung's own reason at all:
the clone rung took the refusal over when it arrived in `faede9969b`
(2026-11-02T04:08:28-05:00),
whose message says

> The refusals of the two rungs share one set of checks,
> so the clean check starts one Git run for both.

That sharing,
not a reason of the clone's own,
is why a clone of a dirty source still pays the untracked scan today.
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
  the cloned bytes are the source's worktree bytes,
  which are Git's checkout bytes only under the condition stated in "What a partial clone does";
  the written paths are converted by Git itself inside `git checkout-index`,
  which runs the same filters Git's checkout runs.
  A filter other than the standard Git LFS definition is refused by the existing check,
  as in the whole-rung clone.
  A text or end-of-line conversion is not refused:
  where the source file is not the conversion of its blob under the new worktree's rules,
  the clone holds bytes Git's checkout would not write
  (`H1`,
the review's `s04`,
`s04b` and `s04c`).
- An ignored,
untracked `.gitattributes`:
  invisible to the refusal,
  because the untracked listing uses `ls-files --others --exclude-standard`,
which hides it,
  while Git still reads its rules;
  a source whose files were made under it differs from a checkout without it,
  the same class as `H1`.
  This is inherited from the source check (the review's `L3`) and this round did not run it.
- A file cloned within the wrapper's pool transaction timeout of being written waits for the pool
  (`README.md`,
the ladder's item 2).
  The dirty arm writes only the files it does not clone,
  so the timing could not show it;
  a source that was just pulled can,
  and the whole-rung clone has the same limit (the review's `L4`).
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
So the rung builds one from the **source's** `HEAD` tree
(`worktree_clone_partial.rs`,
`source.clean.head_tree`),
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

A new `HEAD` that differs from the source's is brought in after the clones and before the proof by the ladder's
`git read-tree -m -u <the source's HEAD> <the new HEAD>`
(`worktree_ladder_target.rs`,
`apply_difference`),
which is what makes the worktree Git's checkout of the new commit;
the review's `s10` and the control `a_new_head_that_differs_is_brought_in_after_the_clones` show the result is right.

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

The proof proves completeness,
the entry kinds,
executable bits,
link targets,
the absence of extra or untracked files,
and that the index equals the new `HEAD`'s tree.
It does **not** prove the content of a path the clones made:
the new index records the stat fields of the file each clone made,
so Git's status trusts it,
and a clone carrying the person's bytes passes it.
Two answers guard that class:
the split's comparison of each entry's object name with the tree's,
and a second,
differently built one,
`git --no-optional-locks diff-index --cached --name-only -z <the source's HEAD>` run in the source,
whose staged paths must not be in the clone set;
the rung refuses when one is
(`worktree_clone_partial.rs`,
`staged_paths` and `a_staged_path_among_the_clones`).
The staged-path red below shows the second answer firing alone,
after the split's comparison was planted away.
What neither guards is a file whose stat fields an edit kept;
that is Git's own trust,
and the clone engine compares each held descriptor before and after it clones.

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
  the second listing of the source's staged paths (`staged_paths` and `a_staged_path_among_the_clones`),
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

26 tests pass on this host,
run as `cargo test --locked --offline --lib worktree_clone_partial` with `CLI_GIT_BTRFS_ABSENT_ACKNOWLEDGED=1`
(`cargo nextest` is not installed here,
so the gate image is the authority on the nextest run),
the btrfs-gated controls printing `BTRFS CONTROL NOT RUN` as everywhere on ZFS,
and the reflink-needing ones really ran.
Four of them were shown red,
each against one planted defect,
with the logs kept:

- the stat verdict accepting every present file:
  the end-to-end control fails with
  `the source changed while it was cloned (edited.txt differs from its index entry by size, times or inode)`.
- the object-name comparison dropped:
  the second answer refuses the staged path that the split would have cloned,
  `staged.txt is staged in the source and would have been cloned`
  (`keep/red-second-answer-staged.log`).
  The same defect,
before that answer existed,
cloned the staged path (the earlier round's red),
which is the review's
  `M1`.
- the source's index installed instead of one built from the tree:
  the proof fails with
  `the new worktree is not what Git's checkout makes: staged.txt differs`.

Two controls answer the review's `L2`:
`a_new_head_that_differs_is_brought_in_after_the_clones`
(a later `HEAD`,
cloned from the source's tree,
the ladder's `read-tree` applied,
proven against the new commit)
and `a_dirty_source_gets_the_clone_rung_where_files_can_be_cloned`
(the ladder's routing for a dirty source,
skipping with its own line where the fixture cannot reflink).

## The measurements

### Method

- Host:
  this machine on 2026-11-20,
  `nproc` 8 and 32 processors in `/proc/cpuinfo`,
  under other sessions' load:
  the load average was 20 and 22 at the runs' starts and never below 9.
  No quiet host was waited for (`NQH`).
- Scope:
  one `systemd-run --user --scope` per run:
  CPUQuota 600%,
  MemoryMax 20G,
  TasksMax 512,
  IOWeight 1,
  so this session gets the least IO weight beside the others.
  A 6-CPU quota makes the process's parallelism 6,
  and the engine leaves an eighth of the CPUs free (`parallel_budget.rs`),
  so a clone step's record says "24,387 entries in 5 slices,
up to 5 workers",
  and so did the author's;
  the engine's own ceiling is 8.
  The adds are the real work,
not synthetic load.
- Filesystems:
  every fixture repository and every destination under the home dataset,
  on ZFS,
  source and destination in one dataset,
  so rung 2 applies to the prototype and to step 3.
- Fixtures:
  one throwaway repository of this repository's tracked files at `33a8d065ff`:
  24,390 files,
  724 MiB of file bytes (780 MB as an archive),
  with the `filter=lfs` lines of `.gitattributes` stripped as earlier rounds did,
  committed by plumbing.
  The synthetic trees of 10,001 files of about 2.6 KB,
8.8 KB and 26 KB in ten flat directories
  are the author's,
not rebuilt this round.
  A hard reset and a clean before each run (with the wrapper's `--no-enforce-worktree`),
  then the run's state:
  usual (three tracked files rewritten and three files nobody tracks),
  clean,
  many:N (N tracked files rewritten and the three untracked),
  or decline.
  `sync --file-system`,
1.7 s and `update-index --really-refresh` follow,
  and a run whose source does not show exactly the state's paths stops before any round.
- Builds:
  release,
  each from a `git archive` copy of its commit with
  `cargo build --locked --offline --release --bin monochromatic-git-policy-cli`:
  the prototype at `33a8d065ff`
  (sha256 `82723396c2d09cae0c098877196224e0baee5620b33faa89f4cc28dc6510e233`),
  `main` at `2df9b1689b`
  (sha256 `5404afd4be6a9082a2844cdad773cf8b36fafe7f1693d65b104fda7e18cbb194`),
  and the harness asserts both digests before a round runs.
  `main`'s add path is the same code the author measured at `392042de70`:
  no file under `worktree/` or `subvolume/` differs between them,
  and the 32 changed files under `src` are the clone-mount handler and its tests,
  `lib.rs` and `source_guard`.
  The author's timed binary was built at 07:19:16 from the working tree,
  before the commits that hold its size rules (07:35:31) and its notice sentences (07:50:32),
  so it cannot be tied to a commit;
  this round's can.
- Arms:
  the prototype,
  the same binary under another name (the control),
  and `main`,
  each reached as plain `git` through a PATH of its own,
  with its own `HOME` and `XDG_RUNTIME_DIR`.
  The order follows six permutations of the three,
  so the control ran first in half of the pairs:
  `prototype, control, main`;
  `control, prototype, main`;
  `main, control, prototype`;
  `prototype, main, control`;
  `control, main, prototype`;
  `main, prototype, control`.
  `sync --file-system` and 300 ms separate the arms of a round.
  Every figure is a ratio to `main` in the same round,
  as `TVC` asks.
  Rounds:
  six counted and one warm-up for the usual state,
twice;
  four counted and one warm-up for each many-path state and the declined arm.
- Fixture setup ran through the prototype's own wrapper,
  which knows its bulk-add escape hatch,
  and its commits are made from plumbing:
  the wrapper's commit path waits on the account's own commit service,
  which a throwaway repository must not reach.
- Verified,
  every round,
  for every worktree the arms made:
  `git status --porcelain` empty,
  and `diff --recursive --no-dereference --exclude=.git` against `main`'s worktree of the same round empty.
  Every arm of every round passed both.
- Every prototype and control round took the clone route (a partial clone on a dirty source),
  and every `main` round took Git's checkout,
  so the arms are what they say.
- Records:
  each run's round record and log are in
  `~/temp/agent/clone-rung-partial-clone-proposal-gate-review-findings-fixed-2026-11-20/scratch/keep/`,
  the one run where a harness bug left the decline rule unset is kept there too and marked invalid,
  and the fixture repository and the worktrees are removed once their numbers are taken.

### The author's kept rounds, recomputed

From the author's kept JSON (`measure-partial-clone-2026-11-20T12-26-41-617Z.json`),
medians over five rounds,
in ms,
and the ratio to `main` per round:

- dirty:
  prototype 915 ms,
  **0.74** (0.67 to 0.80);
  its identical control 1021 ms,
  0.82 (0.77 to 0.92);
  step 3 1412 ms,
  1.14 (1.07 to 1.16);
  `main` 1256 ms (1139 to 1319).
- clean:
  prototype 856 ms,
  0.62 (0.55 to 0.78);
  its identical control 1058 ms,
  0.85 (0.59 to 0.88);
  step 3 997 ms,
  0.81 (0.51 to 0.84);
  `main` 1304 ms (1185 to 1596).

The control is not the narrow spread the earlier text claimed.
It followed the prototype in 8 of the 10 rounds,
and in all 8 it was the slower,
by 11 to 44 per cent:
control over prototype 1.15,
1.15,
1.26 and 1.11 dirty,
and 1.22,
1.37,
1.44 and 1.22 clean;
in the 2 rounds it ran first it was 1.02 and 0.75.
That is a systematic sign,
so the author's prototype arm is the favorable member of an identical pair,
and the pair's control is the fairer figure for that run:
0.82 dirty,
0.85 clean.
The review's rerun (0.84 to 0.86 pooled over 20 identical-binary values,
4 of them above 1.0)
and this round's (below) agree with the control,
not with 0.74.

### The dirty source, this repository's tree, this round

Two runs of six counted rounds each after the gate review,
under load averages 20 and 22,
in ms and as the ratio to `main` in the same round:

- run A:
  `main` 2946 ms (1184 to 3970);
  prototype 1305 ms,
  **0.63** (0.28 to 0.90);
  control 1204 ms,
  0.54 (0.29 to 1.05);
  control over prototype per round 1.66,
1.36,
0.60,
0.98,
0.58,
0.89 (median 0.93).
- run B:
  `main` 3427 ms (1728 to 4716);
  prototype 1389 ms,
  **0.54** (0.30 to 0.66);
  control 1786 ms,
  0.54 (0.31 to 0.72);
  control over prototype 1.04,
1.61,
0.59,
1.67,
0.87,
0.94 (median 0.99).
- pooled over the twelve counted rounds:
  prototype 0.63 (0.28 to 0.90);
  control 0.54 (0.29 to 1.05);
  the 24 identical-binary values range 0.28 to 1.05,
  median 0.61,
  one above 1.0.

On this host both identical arms sit far below `main`:
`main`'s 724 MiB checkout took 1.2 to 4.7 s,
where the review's rounds took about 1.0 s and the author's 1256 ms,
while the clones share blocks and the write step writes three files,
and the harness's `IOWeight=1` gives this session the least IO weight.
So the ratio to `main` is not comparable across sessions;
the identical pair is.
The pair itself differs by up to 67 per cent within a round,
which is wider than any gain claimed here,
so this round confirms the direction and cannot pin the size.
That is also the answer to the review's `M4`:
it measured 0.84 to 0.86 on a host under load 6 to 11,
this round measures 0.54 to 0.63 under load 9 to 22,
the author measured 0.74 with a control of 0.82,
and no run reproduces 0.74 without its control beside it.
The gain exists;
its size is the host's,
from about 0.85 on a quieter host to about 0.6 here.

### The clean source, the author's rounds

Unchanged code,
not rerun this round;
from the same recomputation:

- `main` 1304 ms (1185 to 1596);
  step 3 0.81 (0.51 to 0.84);
  prototype 0.62 (0.55 to 0.78),
  and its identical control 0.85 (0.59 to 0.88).

The three clone arms are the same code and the same work,
so the pair's control is again the fairer member,
and the clean gain to quote is the pair's 0.85,
not 0.62.

### What the proof costs, and what it could cost

Step split of the dirty prototype arm,
medians over the twelve counted rounds of the two runs above,
in ms
(the author's five-round figures are the second number,
for the quieter host):

- `git ls-files`,
  the untracked scan:
  108.8 (82.3),
  in four calls
- `git check-ignore`,
  the trash question,
  which only the snapshot rung uses:
  58.5 (35.0)
- `git status`,
  the changed-path listing and the proof's own status summed under one name:
  145.9 (113.9),
  in two calls
- the in-process stat walk inside the clean check,
  `checkSource`:
  39.3 (35.1)
- reading the index and the tree for the rung,
  `partRead`:
  31.3 (26.9)
- the split's own walk of every tracked file,
  `partClassify`:
  88.9 (79.7)
- building and installing the index,
  `partIndex`:
  53.4 (55.2)
- `git diff-index`,
  the proof's index side and the second staged-path listing together:
  30.8 (24.7),
  in two calls;
  the listing is a few milliseconds of that,
  and its own step,
`partStaged`,
measures 0.02 ms

The proof and setup together are about 557 ms here,
40 per cent of the 1389 ms add
(453 ms of 915 ms on the author's host).
The clones themselves are `cloneEntries` 590.2 ms;
the rest of the command is `gitAdd` 4.8,
`copyIgnored` 37.9,
`git checkout-index` 15.5,
`finishRung` 24.0 and Git's own worktree bookkeeping.
For comparison,
`main` spends nothing on a proof and its checkout is `gitAdd` 2850.9 ms here,
against 1087 ms in the author's rounds:
that difference is the host's load,
and it is why the ratios above are not comparable across sessions.

The proof can be made cheaper,
and the parts are measured even though the combination is not:

- One `git status --porcelain=v2 -z --untracked-files=all --ignored=no --no-optional-locks` answers the untracked
  scan,
  the changed-path listing,
  and the per-path verdict a length-based proof takes from its own walk.
  The runs it would replace cost 108.8,
145.9,
31.3 and 88.9 ms here (82.3,
113.9,
26.9 and 79.7 on the author's
  host),
  and the one run measured 90 to 114 ms in the author's rounds,
  so the saving is around 150 to 190 ms on the author's host and more here,
  a fifth of the add or better.
  What it gives up is the strict nanosecond proof:
  Git's status settles a racy entry by content and then reports nothing,
  which is a stronger answer,
  not a weaker one.
- `git check-ignore` is asked for the snapshot rung's trash place.
  A clone rung makes no trash,
  so the clone rung need not pay its 58.5 ms here (35.0 there).
  Splitting the source check is a change of its own and is not in the prototype.
- The in-process walk inside the clean check,
  39.3 ms,
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
with the range in brackets,
and they are the author's kept rounds recomputed
(`measure-size-cutoff-2026-11-20T12-06-17-568Z.json` and `...T12-20-24-304Z.json`),
not new rounds.
Only this tree's run had an identical-binary arm;
the small fixtures' runs had none,
so their figures carry no control of their own,
which is one reason the threshold below stays unpinned.

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

The per-file rule loses at every size measured with this write step,
because the rung's write step costs several times what cloning the same file costs,
and several times what Git's own checkout of the whole tree costs:
10,001 small files written by `git checkout-index` cost 534 ms on a 190 ms `main` add,
and the same 10,001 files cloned cost 211 ms.
So moving an unchanged path from the clones to the write step is a loss even at 2.6 KB,
and the size at which cloning wins is above every size measured here.
That is the opposite of what the clean-source evidence suggested,
and it is a property of this rung's write step rather than of the clone.
The answer to the review's `M6` here is therefore narrow,
and it is the important half:
`this write step loses`,
not `no per-file rule can win`;
a write step as cheap as Git's checkout was not built,
and with it the rule could win.

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

The unit may not be bytes.
All four fixtures hold 10,001 or 24,389 files,
so file count and mean size were never varied apart:
a tree of 100,000 small files and 400 MB would clone under a bytes rule,
and small files are the shape that loses.
The whole-tree rule's scope and placement are therefore part of what B asks the owner to decide:
it belongs at the clone rung's choice,
judged only after the snapshot rung has been declined,
and on the filesystem it was measured on,
ZFS here.
The prototype evaluates it in `choose_rung` before the snapshot is considered
(`worktree_ladder_run.rs`),
so on a btrfs host with a subvolume source it would also turn off the snapshot rung,
and it applies to clean sources as well,
while every figure above is ZFS.
That placement is a measurement artifact,
not the design.

A rule placed after the proof would spend the proof before declining.
The measured cost of a dirty add that pays the proof and then takes Git's checkout is step 3's,
1.14 of `main` on this tree (1.07 to 1.16),
and the same shape on the small trees;
this round did not measure that placement,
and the earlier reconstruction (1.38,
1.30,
1.23 and 1.33)
could not be reproduced from the kept JSON and is withdrawn.
The declined-before-proof arm this round did take is in "The declined arm".

### A source with many changed paths

Same fixture and arms as the usual state,
with 300 and then 3,000 tracked files rewritten (a line appended to each) and the three untracked ones,
four counted rounds and one warm-up each,
under load averages 9 to 14.
This is the review's `M5`,
and its script's `many:<count>` state.

- 300 changed paths:
  prototype 885 ms,
  **0.80** (0.74 to 0.89);
  its identical control 1047 ms,
  0.94 (0.81 to 1.06);
  `main` 1124 ms (1051 to 1195);
  the pair 0.95,
1.32,
1.29 and 1.08 (median 1.19).
- 3,000 changed paths:
  prototype 1254 ms,
  **0.92** (0.49 to 1.12);
  control 1203 ms,
  0.76 (0.50 to 1.02);
  `main` 1804 ms (1184 to 2901);
  the pair 0.65,
1.01,
0.91 and 1.02 (median 0.96).
- the write step's own step,
  `git checkout-index`:
  15.5 ms at three paths,
  31.8 ms at 300 and 218.1 ms at 3,000,
  about 55 and 68 microseconds a path,
  agreeing with the review's 60;
  the changed-path listing,
in the `git status` step,
is 124.7 ms at 300 and 317.7 ms at 3,000.

The bound is what `M5` asks for,
and it is a shape,
not a size:
at 300 changed paths the add is a gain in both members of the pair (0.80 and 0.94);
at 3,000 it straddles `main` (0.92 and 0.76,
one counted round above it at 1.12);
the review measured 1.13 with a control of 1.01 at 1,500 and 1.54 with 1.44 at 6,000.
So the gain is gone somewhere above a few hundred changed paths,
and by 6,000 the rung is plainly a loss;
the crossing itself is not pinned,
between 3,000 and 6,000 here and lower on the review's host.
Above it the rung does **not** meet the waiver's "no slower than `main`'s".
Two ways keep the option honest,
and the choice is the owner's:
decline the rung above a measured count of changed paths,
which costs the proof already paid (step 3's 1.14,
the cost the waiver records) and keeps the add at or below that;
or state the bound and accept the loss for a bulk rewrite (a codemod or a reformat in the main checkout).
Step 3 would pay the same enlarged changed-path listing at that shape,
so this does not reorder B against D;
that comparison is untested.

### The declined arm

Same fixture and state as the usual one,
with the prototype's whole-tree rule set above the tree's bytes for both clone arms,
four counted rounds and one warm-up,
load average 9.

- prototype 1205 ms,
  **1.00** (0.95 to 1.13);
  its identical control 1227 ms,
  1.01 (0.86 to 1.06);
  `main` 1228 ms (1151 to 1259);
  the pair 1.04,
1.03,
1.06 and 0.77 (median 1.03).
- Both clone arms took Git's checkout (the route says so),
  and their ledger is `main`'s:
  `gitAdd` 1084.7 ms against `main`'s 1060.2,
  `copyIgnored` 35.6 against 49.7,
  `git ls-files` 52.8 against 51.8.
- The rule's own work,
one read of the index and a sum,
is not a named step;
  the whole declined arm is at parity with `main` inside the pair's spread,
  so the rule's cost cannot be separated from noise at this tree's size.
  The earlier text's "about 25 ms,
measured as the 0.96 of the declined arm" was wrong twice:
  0.96 is below 1,
so that arm measured no cost,
  and 25 ms was never a measurement.

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
> A clone holds the source file's bytes,
> and those are the bytes Git's checkout writes only where the source file is the conversion of its blob under the
> new worktree's rules:
> neither the index's stat data nor Git's `status` can tell,
> so a source whose text or end-of-line conversion differs from its blob is cloned with bytes Git's checkout would
> not write.
> Rung 2 has had that limit since it existed.

and,
after the paragraph that ends "The ignored-state copy then runs as for Git's checkout.",
a new subsection (the sentence "and the same tree difference" the review found named here is mid-paragraph):

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
> which runs the checkout's conversion filters,
> and a path reached through a link is written the same way.
> The new worktree's index is built from the tree of the source's `HEAD`,
> with the stat fields of the files the clones made,
> the fields the source's index recorded for a link,
> and none for the paths Git writes;
> the ladder's `git read-tree -m -u` brings a different new `HEAD` in after the clones.
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
> and a source that is a linked worktree with uncommitted changes.
> An untracked file of the source is no refusal:
> the new worktree takes only the tree's paths.
> The worktree is proven after the tree difference with `git --no-optional-locks status --porcelain` printing nothing,
> and `git --no-optional-locks diff-index --cached --quiet <new HEAD>` answering zero,
> which proves completeness,
> types,
> modes,
> link targets and the index's tree,
> and not the bytes of a path a clone made:
> the index records that clone's own stat fields,
> so a staged path among the clones is caught by the rung's second listing of the source's staged paths instead.

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
> and time where the proof settles most files,
> so it
>   is optional,
> and a source whose text or end-of-line conversion differs from its blob is
> cloned with bytes Git's checkout would not write (rung 2's own limit since it existed).

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

Four other places state the clean-source rule and change with these.

`package/git-policy/cli/SPEC.md:1250` to `1253`,
among the snapshot's refusals,
from:

> and the clone rung,
> which is only for clean sources.

to:

> and,
>  for rung 1 only,
> an untracked file that no ignore rule covers,
> which a partial clone takes none of.

`package/git-policy/cli/README.md:819`,
the ladder's item 2,
from:

> 2. Clones (`FICLONE`) of every tracked file of a clean worktree,
>    with its index,

to:

> 2. Clones (`FICLONE`) of every tracked file the source's proof settles,
>    with the index the rung builds from the source's `HEAD`,
>    while Git writes the paths a change reaches,

`package/git-policy/cli/src/native/worktree/worktree_snapshot_source.rs:349` to `354`,
the doc comment of `source_or_dirty`,
from its sentence:

> the clone rung and the other
>  callers of the clean answer never see it.

to:

> the clone rung takes the same source and clones the paths its proof settles while Git writes the rest,
> and the callers of the clean answer see clean sources only.

`doc/planning/cli-git-zfs-worktree-implementation-plan.md:605` to `617`,
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
"This round" is the two usual runs and the many-path,
declined-arm and size runs above,
on this host under load 9 to 22;
"the author's" and "the review's" name the earlier rounds.
Where a pair is quoted,
the first figure is the prototype and the second its identical control.

- **A.
  Partial clone for every source with changes,
  no size rule.**
  Dirty path on this tree:
  this round 0.63 (0.28 to 0.90) with its control 0.54 (0.29 to 1.05);
  the review's 0.84 to 0.86 pooled over 20 identical-binary values,
4 of them above 1.0;
  the author's 0.74 with a control of 0.82.
  At 300 changed paths this round:
  0.80 (0.74 to 0.89) with a control of 0.94 (0.81 to 1.06);
  at 3,000:
  0.92 (0.49 to 1.12) with a control of 0.76 (0.50 to 1.02),
one counted round above `main`.
  Clean path unchanged:
  0.62 with a control of 0.85 (the author's,
not rerun).
  It removes the waived cost on this tree's usual shape and turns it into a gain,
  and a dirty source no longer ends in Git's checkout.
  It keeps the small-file loss:
  the author's cloning arm cost 1.14,
1.57 and 1.03 at 2.6,
8.8 and 26 KB dirty,
  and the integration pass measured 1.33 and 1.18 clean on the first two.
  It ends the waiver's dirty half on this tree's usual shape in the median,
not in every round;
  it does not end the widened small-file half;
  and above a few hundred changed paths it does not meet "no slower than `main`".
- **B.
  Partial clone with the whole-tree size rule,
  declined before the proof.**
  On this tree it clones,
so A's dirty figures are its;
  on the author's small trees it declined at 0.89,
1.00 and 1.03 (26 MB,
88 MB and 260 MB),
  and that run had no identical-binary arm.
  Declined on this tree this round:
  1.00 (0.95 to 1.13) with a control of 1.01 (0.86 to 1.06),
  both arms taking Git's checkout;
  the rule's index read is not a named step and its cost is inside the pair's spread
  (`0.96` as "25 ms" was the earlier text's error,
corrected in "The declined arm").
  It needs a threshold,
  these points put it near 400 MB of tracked bytes,
  the crossover is bracketed by 260 MB and 780 MB rather than pinned,
  and the unit may not be bytes:
all four fixtures hold 10,001 or 24,389 files.
  Its scope and placement are part of the decision:
  at the clone rung's choice,
after the snapshot rung is declined,
on ZFS as measured;
  the prototype's earlier placement would also turn off the snapshot rung on btrfs.
  And above a few thousand changed paths it is slower than `main` like A;
  the bound is in "A source with many changed paths".
  It is the only option measured at or below `main` on both clean small-file shapes (it declines there),
  and it ends both halves of the waiver only where the threshold is right and the changed-path count is small.
- **C.
  Partial clone with the per-file size rule.**
  Dirty path 1.05 (0.96 to 1.10) at a 4 KB cutoff and 1.90 (1.43 to 2.11) at 32 KB on this tree,
  against the same run's clone-everything arm at 0.77 (0.71 to 0.83) with its control 0.76 (0.73 to 0.78);
  writing every unchanged path,
which a cutoff above every file means,
  cost 2.80,
3.59 and 4.30 on the small fixtures and 2.83 here.
  Its clean path would pay the same write step;
  that was not measured,
and on the small trees the write-all figures are what it would cost.
  It does not reach `main` anywhere measured,
  and its only measured edge over D is inside the identical pair's spread.
  It was measured only with a write step the proposal itself calls the wrong tool,
  so its finding is "this write step loses",
  not "no per-file rule can win".
  It ends neither half of the waiver.
- **D.
  No partial clone:
  keep step 3 as it is under the waiver.**
  Dirty path 1.14 (1.07 to 1.16) on this tree,
the waiver's measured cost;
  clean path 0.81 here and 1.33 and 1.18 on the 2.6 KB and 8.8 KB trees by the integration pass.
  It costs nothing new,
leaves the waiver in place,
  and leaves `H1`'s reach and the widened small-file cost as they are.

Ranking:
**B > A > D > C.**

- B over A:
  on the small trees B declines to `main`'s own checkout (0.89 to 1.03 plus one index read)
  where A clones them at 1.14 to 1.57 dirty and 1.33 and 1.18 clean by the integration pass;
  on this tree both clone,
so B's gain is A's.
  B pays a threshold whose unit is not yet pinned;
  A needs no rule but leaves that loss.
- A over D:
  A removes the dirty cost on this tree
  (0.63 this round with a control of 0.54,
0.84 to 0.86 on the review's host,
0.74 with a control of 0.82 by the author)
  where D keeps 1.14 and `H1`'s reach.
  A's small-tree loss is why B is ahead of it,
not why D is.
- D over C:
  C's dirty edge over D on this tree,
1.05 against 1.14,
is inside the identical pair's spread
  (the author's control 0.76 to prototype 0.83,
the pair's range 0.71 to 1.10),
  while C's clean path pays the per-file write step for every file below its cutoff,
  which on the small trees is 2.80 to 4.30 against D's 1.33 and 1.18.
  C therefore loses more than it gains,
and it ends no waiver.

Which waiver each ends:

- The record `doc/decision/cli-git-zfs-clone-rung-dirty-source-waiver.md` ends when a dirty add is no slower than
  `main`'s,
and its widened part covers the clean add of a tree of small files.
- A ends the dirty half on this tree's usual shape (medians;
one round of the review's 20,
4 of 20 in fact,
was above
  `main`),
and neither the widened part nor the many-changed-paths shape.
- B ends both parts on the measured shapes,
  under a threshold and a changed-path bound it still needs;
  declined,
its add is `main`'s plus one index read.
- C ends neither.
- D ends nothing.

## The gate review's findings, answered

The review is `~/temp/agent/gate-review-of-the-partial-clone-proposal-on-sonnet-2026-11-20/keep/review.md`,
with its logs and scripts beside it.
Each finding is fixed here,
recorded as a limit,
or dismissed with the reason.

- **H1,
the equality claim where a text or end-of-line conversion applies:
  recorded as an open limit,
and no fence built.**
  "What a partial clone does" states the condition under which a clone equals Git's checkout,
  that rung 2 has had the limit since it existed (`s04b`),
  the dirty forms (`s04`,
`s04c`),
  and that the remedy waits on the project's decision.
  The conversion bullet under "The other states the question names",
"How the result is proven" and the new `SPEC.md`
  text carry the same condition.
  A fence on the presence of `text`,
`eol` or `autocrlf` would refuse this repository,
  and a per-path fence needs the bytes.
- **M1,
the proof cannot see wrong bytes in a cloned path,
and one comparison guards the staged case:
  fixed in the text and the prototype.**
  "How the result is proven" now says what the proof proves and what it does not,
  and the rung asks Git in the source for the staged paths and refuses one among the clones
  (`staged_paths`,
`a_staged_path_among_the_clones`).
  The red shows that answer firing alone after the split's comparison was planted away.
  The remaining classes (a stat-trusted edit,
a racy entry) are as the review describes them:
  Git's own trust and the engine's descriptors,
not the proof.
- **M2,
a dirty linked worktree is still refused:
  fixed in the text,
the lift owed.**
  "States that still refuse the whole rung" and the new `SPEC.md` text name it,
  with `restorable_here` and the review's `s11` as the evidence;
  letting the clone rung take one needs a flag of its own and a measurement of its add.
- **M3,
measurement statements the data does not support:
  fixed,
five corrections.**
  First,
the control spread:
the author's control followed the prototype in 8 of 10 rounds and was slower in all 8,
  by 11 to 44 per cent,
so the pair's control is the fairer figure.
  Second,
the host:
`nproc` 8 and 32 processors in `/proc/cpuinfo` are now said,
  and the 6-CPU quota's 5 workers is what the records say,
not an inference.
  Third,
the "declined after the proof" figures 1.38,
1.30,
1.23 and 1.33 could not be reproduced from the kept JSON
  and are withdrawn;
the placement's cost is argued from step 3's measured 1.14 instead.
  Fourth,
"about 25 ms,
measured as the 0.96 of the declined arm" is corrected:
  0.96 is below 1,
so that arm measured no cost,
and the declined arm's parity is what the new round shows.
  Fifth,
the author's binary was built from the working tree before its size-rule and notice commits,
  so it cannot be tied to a commit;
  this round's binaries are named by commit and sha256,
and the harness asserts both digests before a round.
- **M4,
the gain's size is not reproduced:
  fixed by restating it with its control.**
  This round measures 0.63 with its control 0.54 on a host under load 20 to 22,
  the review measures 0.84 to 0.86 under 6 to 11,
  and the author's 0.74 comes with a control of 0.82.
  "Never slower" is gone from A and B:
the identical pair itself spans 0.58 to 1.67 in a round.
- **M5,
no measurement of many changed paths:
  fixed by measuring 300 and 3,000 and stating the bound.**
  At 300 the pair is 0.80 and 0.94;
  at 3,000 it is 0.92 and 0.76 with one round above `main` at 1.12;
  the review's 6,000 is 1.54 with 1.44.
  "A source with many changed paths" states the bound,
the two ways to hold it,
and that above it the waiver's
  "no slower than `main`'s" is not met.
- **M6,
the ranking and the options:
  fixed.**
  The ranking is B > A > D > C,
  with D over C because C's dirty edge over D is inside the identical pair's spread while C's clean path pays the
  per-file write step (2.80 to 4.30 on the small trees against D's 1.33 and 1.18).
  B's scope and placement are written:
the clone rung's choice,
after the snapshot,
on ZFS as measured;
  the threshold's unit may not be bytes,
and all four fixtures held one file count;
  the declined arm's cost is at parity,
not 25 ms.
  Each option now says which half of the waiver it ends,
or that it ends neither.
- **M7,
incomplete and inaccurate design text:
  fixed.**
  The index is built from the source's `HEAD` tree,
with `read-tree -m -u` bringing a different `HEAD` in;
  a path below a link is written,
not refused (`s07` equals Git's checkout);
  the new subsection is anchored to the end of the paragraph it follows,
not to a sentence in its middle.
  The stale "saves disk,
not time" is gone from the new text.
  The four other places are listed with their new text:
  `SPEC.md:1250` to `1253`,
`README.md:819`,
`worktree_snapshot_source.rs:349` to `354`,
  and `cli-git-zfs-worktree-implementation-plan.md:605` to `617`.
- **L1:
fixed.**
  `95ce7f01be`'s message is quoted as what it says (the subvolume destination and the removal),
  the module doc is what names the callers,
  and `faede9969b`'s message is quoted for the sharing that made the clone rung inherit the untracked refusal.
- **L2:
fixed with two controls.**
  `a_new_head_that_differs_is_brought_in_after_the_clones` and
  `a_dirty_source_gets_the_clone_rung_where_files_can_be_cloned`;
  both passed on this host,
the second after its own skip check ran the reflink probe.
- **L3:
recorded as an inherited limit.**
  An ignored,
untracked `.gitattributes` is invisible to the untracked listing the source check uses,
  and it is the same class as `H1`;
  this round did not run it,
and the new `SPEC.md` text does not claim it is covered.
- **L4:
recorded as an inherited limit.**
  A file cloned within the pool's transaction timeout waits for the pool;
  the dirty arm writes only the files it does not clone,
so the measurement could not show it,
  and a source that was just pulled can.
- **L5:
partly fixed,
partly owed.**
  The commit message is not amended (it is named in "What this leaves owed" with the corrective comment owed under
  `GCA`).
  The fixture's stripped `filter=lfs` lines are now stated as a limit,
  with `git-lfs` 3.8.1 on this host and the round owed;
  the small trees' ten flat directories are named where they are described.

The review withdrew its draft finding about the racy rule's name;
nothing here answers it.

## What this leaves owed

- The gate's whole suite in the gate image ran at `dba1276a06`,
the tip before this round's prototype commit:
  9,291 tests,
  9,287 passed,
  4 failed and 28 skipped,
  and the four failures are none of this rung's:
  two `repair::package_repair_prewarm_job` controls whose fixture cannot resolve `serde` offline in that image,
  and two commit-service controls that find a `cli-git-lock-holders` entry a control does not expect.
  The code this round added after that tip (the second answer and two controls) passed the module's tests on this host;
  the gate is what a merge runs again,
  so these figures name what the earlier round saw,
not a green gate.
- The standing notice's control over the steps an add enters failed on the first gate run,
  because the partial clone's five steps had no sentence;
  the sentences are written,
the sixth step (`partStaged`) has its own,
and the control passes.
- This round did not rerun the clean arm,
build step 3,
or rebuild the author's three small-file fixtures;
  their figures are the author's kept rounds,
recomputed,
with the control notes given beside them,
  and its own rounds ran under load 9 to 22,
which is why the ratios to `main` differ from the review's.
- A round on a tree where the source is a linked worktree,
once the refusal is lifted,
and a measured changed-path
  bound if the owner wants one rather than a stated bound.
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
- `H1`:
a clone keeps the source's bytes where a text or end-of-line conversion would make Git's checkout write others,
  and neither stat data nor `git status` can tell.
  The remedy reaches the accepted clone rung,
so it is a project decision being prepared separately,
  and this proposal only records the limit.
- A dirty linked worktree is still refused (`restorable_here`);
  letting the clone rung take one needs a check of its own and a measurement of its add.
- A round on a tree whose `.gitattributes` keeps its `filter=lfs` lines.
  This host has `git-lfs` 3.8.1,
  but every fixture here strips those lines as the earlier rounds did,
  so no round exercised an LFS-attributed tree (the review's `L5`).
- The ignored,
untracked `.gitattributes` above (the review's `L3`),
  and the pool wait for a file cloned within its transaction timeout (the review's `L4`):
  both are inherited limits stated,
not measured.
- A bound on the number of unchanged paths a clone may write,
or a decline above it:
  the measured break-even is in "A source with many changed paths".
- The ladder's full routing control,
with an `AddPlan` and both rungs on a clone-capable source,
  beyond `a_dirty_source_gets_the_clone_rung_where_files_can_be_cloned`
  and the timing harness.
- The commit `dba1276a06` repeats the subject of `4f0175786c` and does not say it amends;
  it is not amended,
and a corrective comment on it is owed under `GCA`.
- The Rust controls ran on this host as `cargo test --locked --offline --lib` with
  `CLI_GIT_BTRFS_ABSENT_ACKNOWLEDGED=1`,
  because `cargo nextest` is not installed here;
  the gate image's nextest run is the authority and a merge runs it again.
