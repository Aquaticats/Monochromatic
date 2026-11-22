# cli-git clone rung: cloning the unchanged files of a dirty source

## Status

Accepted 2026-11-21 in the question tool as the option the decision record calls "clone with limits",
recorded in `doc/decision/cli-git-clone-rung-partial-clone-of-a-dirty-source.md`,
which is the answer this proposal went to the owner for.
**Unapproved and not final**:
only the owner declares work complete.
The build that lands it is on
`feat/git-policy-cli-clone-rung-partial-clone-of-a-dirty-source`,
with both limits pinned and the round of 2026-11-21 recorded in "The build that landed".

The design was proposed and a prototype measured it on `main`'s fenced clone rung,
and "The decision asked for" records what was asked.
This is the answering round of the gate re-check
(`~/temp/agent/gate-recheck-partial-clone-proposal-on-the-fenced-rung-2026-11-21/keep/review.md`,
not final,
Claude Sonnet 5.5;
the review file's modification time is 2026-11-21T20:38-05:00).
The round merges `main` at `9b07ba55bd` (merge `e86faecf31`,
2026-11-21T20:41:43-05:00),
reruns the shapes the decision rests on in a position-balanced order,
adds the class control the re-check's sixth red plant showed missing,
and answers every finding in "Answers to the gate re-check".

This document is written on the absorbed `main`,
not patched around it,
and this branch is the branch measured.
The owner answered "Yes,
design and measure it (Recommended)" on 2026-11-20,
recorded in `doc/decision/cli-git-zfs-clone-rung-dirty-source-waiver.md`,
which also names when the waiver it granted ends;
"Which comparison the waiver means" reads that sentence against the record.

Four things changed since the version this branch carried on 2026-11-20,
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
  and every number in "The measurements" is measured with both changes compiled together.
- `main` moved under the proposal.
  The up-front skip of a differing fenced tree,
  the second follow-up ZFS plan step 10 records,
  merged at `5d2a6c58b3` on 2026-11-21T20:16:22-05:00;
  this round merges `main` at `9b07ba55bd`,
  so the skip is in the prototype and in the measured pair.
- A dirty source has a different opponent than it had.
  On `main` as it is,
  a clean add takes the fenced clone rung on ZFS,
  and a dirty add still takes Git's own checkout.
  The two are compared separately in "The measurements",
  and which one the waiver's sentence means is said in "Which comparison the waiver means".

`NQH` holds for every figure here:
no quiet host was waited for,
the control shares the measured binary,
and every round's arms are adjacent in time.

## This round

This round answers the gate re-check,
and it is the round the figures in "The measurements" come from.
It merged `main` at `9b07ba55bd` (merge `e86faecf31`,
2026-11-21T20:41:43-05:00),
resolving the one test conflict by keeping both sides' controls.
It reran five shapes in a position-balanced order:
the usual three-path state,
300,
6,000 and 20,000 changed paths,
and the 2.6 KB tree with three changed paths,
eight or ten counted rounds each,
with the identical-binary control and Git's own checkout.
It added the class control for the second staged-path listing's wiring,
and showed it red with that answer ignored.
It reran the seeded differential sample unchanged:
seed 11,
300 of 300.
It answers H1,
M1 to M7 and L1 to L7 in "Answers to the gate re-check".
The records are in `~/temp/agent/clone-rung-partial-clone-on-the-fenced-rung-2026-11-21/scratch/keep/`
(`measure-balanced-*.json`,
`log-*-this-round.txt`,
`sample-seed11-this-round.log`,
`red-plant-second-listing-ignored.log`,
`class-tests-this-round.log` and `summary-this-round.txt`).
The work is unapproved and non-final.

## The build that landed

The design is built on
`feat/git-policy-cli-clone-rung-partial-clone-of-a-dirty-source`,
in production shape.
The prototype's measurement switches are gone.
The same code is split under the file-size limit:
`worktree_clone_partial.rs` for the run itself,
`worktree_clone_partial_tracked.rs` for the source's index,
the tree and the stat fields the new index records,
and `worktree_clone_partial_staged.rs` for the live staged-path listing.
A source's changed-path count comes from the status read the source check already makes,
so an untracked file is no changed file.

### The two limits as pinned

`package/git-policy/cli/src/native/worktree/worktree_clone_rung_limits.rs` holds both limits,
each with its measurement beside it.
Both are stated as provisional and open to the project's veto.

The whole-tree size limit is 762,000,000 tracked bytes,
pinned at the winning edge of the bracket.
A dirty source lost at every size measured below it:
1.30 of Git's own checkout at 10,001 files of 2,600 bytes (about 26 MB),
and 1.32 at 10,001 files of 26,000 bytes (260,000,028 bytes,
1.03 to 1.71,
identical control 1.40,
8 counted rounds,
measured for this pin on 2026-11-21).
This repository's tree of 24,621 files and 763,289,228 bytes won at 0.73 to 0.81 of it.
The rung's fence reads cost about the same whatever the tree holds:
275 ms of `git ls-tree` at 24,621 paths,
110 ms at 10,001.
Git's own checkout grows with the bytes,
so the rung loses wherever a checkout is shorter than those reads.
The pin sits at the top of the bracket,
so no size nobody measured is cloned.
The 260 MB shape measured 1.04 of Git's own checkout (0.76 to 1.35),
and 1.00 of `main` (0.77 to 1.21),
once the pin declined it,
which is the control for the pin's placement.

The changed-file cut is one quarter of the tracked paths,
which is the fence's own decline share.
It is 6,155 of this repository's 24,621.
Three rounds measured 6,000 changed paths at 1.00 to 1.05 of Git's own checkout,
which is parity,
and 20,000 at 1.68 to 1.74,
which is the loss the cut declines.

### The controls this build added

Each limit has a ladder control that goes red when the limit is removed.
`a_source_below_the_size_limit_gets_gits_own_checkout_and_its_bytes` and
`a_source_above_the_changed_file_cut_gets_gits_own_checkout_and_its_bytes` each assert:
the rung the ladder chose,
the numbers its sentence names,
that nothing was claimed,
that nothing was copied,
and that the worktree the add leaves is byte for byte the one Git's own checkout makes,
with the source's change in neither.
Their fixtures derive their sizes from the pinned constant,
and both name their own skip where the fixture root cannot reflink a file.

`the_size_limit_declines_one_byte_below_the_pinned_bytes_and_clones_it` and
`the_cut_declines_one_changed_path_more_than_its_share_and_ignores_untracked_ones` hold each value's own edge in
`worktree_clone_rung_limits_tests.rs`,
including the untracked file the cut must not count,
and `the_cut_is_the_fence_own_share_of_the_tracked_paths` ties the cut to the fence's constant,
so a change to that share moves both.

The ladder controls that have to reach the clone rung now build fixtures above the size limit,
and the btrfs control that relied on a dirty source ending the rung uses a state no rung serves,
because a source with changes no longer does.

### The six guard re-plants

Each was planted alone in a scratch copy of the tip,
and run once against the class controls
(`keep/plant-*.log` in the round's folder).
Every one went red:

- the union drops the fence's paths:
  `a_path_a_conversion_rule_reaches_is_written_by_git`.
- the new index is the source's index bytes:
  four controls.
- written paths keep their clones' stat data:
  `a_path_a_conversion_rule_reaches_is_written_by_git`.
- the object-name comparison dropped:
  five controls,
  `a_staged_path_is_written_not_cloned` among them.
- the stat verdict accepts every file:
  nine controls.
- the second staged-path listing computed and ignored:
  `a_path_staged_after_the_source_check_is_refused`.
  This is the plant that stayed green before that control existed,
  so no control had to be added for it.

### This build's own rounds

The shapes the decision rests on were rerun against `main` and Git's own checkout,
in a position-balanced order.
That order is a Williams square of four arms,
two blocks after one warm-up,
with the second block reversed.
The identical-binary control is the same build under another name.
The records are `keep/measure-balanced-*.json` and `keep/log-*.txt` in the round's folder.

- The usual dirty state (3 changed and 3 untracked paths),
  8 counted rounds:
  the partial clone is 0.63 of `main` (0.29 to 0.87,
  below in 8 of 8),
  and 0.81 of Git's own checkout (0.68 to 1.17).
  The identical control is 0.65 of `main`,
  and the pair's median ratio is 0.99.
- 6,000 changed paths,
  12 counted rounds pooled from three blocks:
  0.85 of `main` (0.74 to 1.16,
  below in 9 of 12),
  and 1.00 of Git's own checkout (0.90 to 1.31,
  below in 6 of 12),
  which is parity.
  The identical control is 0.91 of `main` and 1.03 of Git's own checkout,
  and the pair's median is 1.06.
- 20,000 changed paths,
  8 counted rounds:
  every arm took Git's own checkout,
  because the cut declines the shape.
  The add is 1.02 of `main` (0.39 to 2.11),
  where it was 1.74 of Git's own checkout before the cut.
- The 2.6 KB dirty tree,
  8 counted rounds:
  every arm took Git's own checkout,
  because the size limit declines it.
  The add is 0.91 of `main` (0.79 to 1.05),
  and 0.98 of Git's own checkout (0.77 to 1.21),
  which is parity,
  where the clone took 1.30 of Git's own checkout.
- The 260 MB dirty tree (10,001 files of 26,000 bytes),
  8 counted rounds before the pin and 8 after it.
  Before,
  the clone took 1.32 of Git's own checkout (1.03 to 1.71),
  with its control at 1.40.
  After,
  the limit declines it,
  and the add is 1.04 of Git's own checkout (0.76 to 1.35).

Every arm of every counted round exited 0,
took the route its name and the record say,
left a worktree clean by `git status`,
and was equal to Git's own checkout by `diff --recursive --no-dereference`.

### The correctness checks

- The class controls:
  28 in the three sidecar files,
  and the ladder's routing control,
  all passing on this host with the two new limit controls beside them.
- The seeded differential sample (seed 11,
  150 combinations,
  300 checks):
  300 of 300 equal to Git's checkout by bytes,
  mode,
  link target and staged index,
  and clean by `git status` with an index equal to its `HEAD`'s tree.
  Its fixtures carry a sparse anchor above the size limit,
  so the rung is reached rather than declined:
  70 combinations went through the partial clone,
  and 80 through Git's own checkout,
  which are the fence's refusals and the cut on the combinations with more changed paths than a quarter of a small
  fixture's.
- The container gate on this host:
  9,770 tests run,
  9,768 passed,
  with the two standing cause-H failures
  `doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md` names,
  and nothing else.

### The guest arm

One throwaway `mvm` guest of this round's own ran the crate's btrfs controls.
It is Ubuntu 24.04 on kernel 6.8.0-147-generic,
with a 12 GiB btrfs filesystem on a loop file as `CLI_GIT_TEST_ROOT` and no acknowledgement set,
so every btrfs control ran rather than skipping.
Result:
186 tests run,
174 passed,
12 failed.
`btrfs_a_clean_ordinary_source_gets_the_clone_rung_and_a_subvolume` and
`btrfs_a_refused_source_gets_a_subvolume_that_a_failed_git_leaves_no_trace_of`,
the two this round rewrote,
passed,
and so did the snapshot rung's own btrfs controls.
Six failures are this limit's:
`worktree_ladder_finish`'s controls say "the source is eligible" and see `Person` where they expect
`WithoutCheckout(2)`,
because the size limit now declines their small fixtures.
They needed the sparse fixture the `worktree_ladder_run` controls got,
which the rerun below gave them.
The other six are outside this rung,
two `fs_btrfs_ioctl` snapshot and destroy controls,
and one each in removal,
trash,
stage claim and landed removal,
and that round did not diagnose them;
their words name a snapshot whose record proves no object and a boundary set the sweep did not expect,
which is what a loop-mounted btrfs may explain and which no control here settles.

The work is **unapproved and non-final**.
Only the owner declares it complete.

### The rerun of 2026-11-22, both arms in one guest

One throwaway `mvm` guest of the fixtures round's own ran every control that needs a btrfs fixture,
on this branch and on `main` in the same guest.
It is Ubuntu 24.04.5 on kernel 6.8.0-147-generic,
with Git 2.43.0,
Rust 1.100.0,
`cargo-nextest` 0.9.152,
8 GiB of memory,
four CPUs,
and a 12 GiB btrfs filesystem on a loop file as `CLI_GIT_TEST_ROOT`,
with no acknowledgement set,
so every btrfs control ran.
The filterset is the 37 test modules that call `btrfs_fixture`,
which is 580 tests here and 577 on `main`,
and they ran one at a time so a heavy control's fixture never competed for the loop file.

- This branch at `e283ebf298`:
  580 run,
  562 passed,
  17 failed,
  1 timed out,
  7,934 filtered out.
- `main` at `e5c856071b`:
  577 run,
  559 passed,
  17 failed,
  1 timed out,
  7,926 filtered out.
- The two failure sets are the same 18 names,
  no test fails on one arm only,
  and each pair prints the same message
  (`~/temp/agent/clone-rung-partial-clone-btrfs-fixtures-2026-11-22/keep/suite-comparison.txt`).

So none of the 18 is this rung's,
and the six controls this build owed pass in it:
`btrfs_a_refused_clone_rung_leaves_a_file_it_did_not_create`,
`btrfs_a_failing_checkout_after_a_refused_clone_rung_leaves_the_worktree_and_its_registration`,
`btrfs_each_rung_hands_the_adds_record_over_before_it_prints_and_marks_nothing`,
`btrfs_an_add_whose_records_cannot_be_written_still_succeeds_and_keeps_everything`,
`btrfs_a_fallback_never_resets_a_replacement_at_the_destination` and
`btrfs_a_checkout_that_ran_against_a_replacement_is_not_taken_for_the_adds`,
each taking 3 to 6 seconds,
which is a rung reached rather than a control that stopped at Git's own checkout.

The 18 that fail on both arms,
by what their messages show:

- Eight are one misread ioctl,
  diagnosed here with a direct call in the same guest
  (`doc/troubleshooting/btrfs-get-subvol-info-returns-one-on-success.md`):
  `BTRFS_IOC_GET_SUBVOL_INFO` answers 1 for every subvolume that has a root backref,
  and `subvolume_info` treats any nonzero result as an error,
  so the message names a `PrimitiveError` with a stale errno,
  or says a subvolume names no origin.
  They are `fs_btrfs_ioctl`'s snapshot control,
  the four `worktree_copy_stage_snapshot_origin` controls,
  and the stage-claim controls
  `a_different_snapshot_at_the_stage_name_is_not_taken_for_the_one_a_kill_left`,
  `a_snapshot_holding_a_newer_entry_is_kept` and
  `btrfs_the_snapshot_created_before_its_record_is_removed`.
  `main` carries the same check at the same line,
  so neither arm can pass these in a btrfs guest.
- Seven assert the branch a caller without `CAP_SYS_ADMIN` meets,
  and the guest runs every control as root:
  a destroy by id succeeds where the control expects `EPERM`,
  a worktree holding a nested subvolume is removed around it where the control expects a refusal,
  and five controls need a permission a root caller has anyway
  (a store this account may rewrite,
  a trash that cannot be emptied,
  a handler that cannot remove it,
  a subtree the sweep cannot examine,
  and a pointer that cannot be examined).
  An earlier guest recorded the store control and four stage-claim controls as failing,
  and attributed them to its root and btrfs environment;
  three of those four are the misread ioctl above,
  and the fourth is the pointer a root caller can examine.
- One hands a landed worktree's pool to the resident service instead of removing it,
  and this guest settles nothing about why.
- Two are the guest's own limits:
  one control hits the guest's descriptor limit while removing its fixture,
  and one waits past nextest's 300-second timeout.

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

### The two follow-ups ZFS plan step 10 records

Step 10's notes (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
"Step 10")
record two follow-ups the project accepted with the fence,
and both are the clone rung's own costs,
not this proposal's:

- the faster size reader,
  which would replace the fence's `git ls-tree -r -l` with sizes read from the object database,
  still owed;
- the up-front skip of a differing fenced tree,
  which merged at `5d2a6c58b3` on 2026-11-21T20:16:22-05:00.
  It adds `skip_reason_for_the_target_tree` to `choose_rung` (`worktree_ladder_run.rs`)
  and `worktree_ladder_target_tree_skip.rs`,
  and it ends an add at Git's checkout when the commit the add names changes a path the fence hands to Git.

This proposal merges `main` at `9b07ba55bd`,
so the skip is in its prototype and in the measured pair.
The skip returns at once when the add names no commit (`worktree_ladder_target_tree_skip.rs`,
`plan.target.as_deref()?`),
so every same-tree add measured here is untouched by it,
and the figures in "The measurements" stand as measured.
Where the add does name a differing commit,
the clone-rung block takes the skip with no dirty-source guard
(the snapshot's block has one),
so a dirty add at a tree that changes a fenced path is declined before any proof is paid,
which is what the follow-up intended.
Without it,
the re-check's variant sample saw 22 of 43 attempts at a differing new `HEAD`
end after the clone rung with `error: Entry 'beta.txt' not uptodate. Cannot merge.`
and finish in Git's own checkout,
equal in bytes and paying the copies the follow-up removes.

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
with one command ("Which Git command writes the changed files"),
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
  by the conversion rules the fence lists.

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
and the gate review of 2026-11-20 had already shown it cannot see the bytes of a path the clones made,
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
  `a_staged_path_among_the_clones`).
  Two controls cover it:
  `a_staged_path_among_the_clones_is_refused` covers the comparison itself with hand-built entries,
  and `a_path_staged_after_the_source_check_is_refused` covers its wiring,
  by moving the source's index after the source check so the snapshot's proof still carries the path
  and the live listing is the only answer that can refuse it,
  which is how it guards that listing's wiring rather than a hazard the listing removes;
  that control goes red when the listing's answer is computed and ignored ("Correctness");
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
  and the refusals in "States that still refuse the whole rung",
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

The controls are 28 tests in three sidecar files
(`worktree_clone_partial_tests.rs`,
`worktree_clone_partial_split_tests.rs`,
`worktree_clone_partial_index_tests.rs`),
run as `cargo test --locked --offline --lib -- worktree_clone_partial` with `CLI_GIT_BTRFS_ABSENT_ACKNOWLEDGED=1`
(`cargo nextest` is not installed on this host;
the gate image's nextest run is the authority).
All 28 pass on this host,
and the reflink-needing ones really ran,
because this host's fixture root is on ZFS and can clone:
ten tests in `worktree_clone_partial` alone,
including `the_unchanged_paths_are_cloned_and_the_changed_ones_written`,
`the_index_records_the_files_the_clones_made`,
`a_path_a_conversion_rule_reaches_is_written_by_git`,
`a_path_staged_after_the_source_check_is_refused`,
`a_new_head_that_differs_is_brought_in_after_the_clones` and
`the_flow_runs_where_no_file_can_be_cloned`.
With `a_dirty_source_gets_the_clone_rung_where_files_can_be_cloned`
(`worktree_ladder_run_tests.rs`) the class controls are 29.

The four red proofs of the round before the fence
(the stat verdict accepting every file,
the object-name comparison dropped,
the source's index installed,
and the count of written paths)
were taken against the pre-fence code.
The gate re-check re-planted them against this tip,
one removal at a time:
the guards still stand and the catchers moved.
The stat verdict's red is caught at run time by the clone engine's descriptor comparison in `worktree_clone.rs`,
which the merge replaced with `main`'s version,
extended with `clone_these_bytes_beside`;
the source's-index red by the post-clone `git status` proof,
which is `#[cfg(test)]`
(`worktree_clone_partial.rs`) and so a control's answer,
not the rung's;
the written-count red by an assertion in a controls file this round rewrote for the fence;
the object-name comparison by five controls,
the split's own among them.
The code this round added (the union of the written paths,
the waiting index,
`the_recorded_stats`)
had no red proof,
and the re-check's sixth removal,
the second staged-path listing computed and ignored,
left all 28 then-existing controls green.
That gap is closed and measured here:
with the listing's answer ignored,
the new control fails and the other 27 pass
(`red-plant-second-listing-ignored.log`).
The six re-plants are owed again before a merge,
because a merge rewrites the catchers once more.

## The measurements

### Method

This round reran the shapes the decision rests on with the author's harness
(`clone-rung-partial-clone-on-the-fenced-rung-2026-11-21/scratch/keep/measure-balanced.ts`),
whose fixture,
source states,
arms and verification are the author's,
with one change:
the order.
The author's rounds stay in the record and are quoted as the author's,
marked where they differ.

#### Host and scope

The host is this machine,
under other sessions' load;
the one-minute load average was 4.2 to 11.3 at the end of each shape,
and 9.9 to 14.4 over the fifteen minutes before,
across the shapes,
and the runs never waited for it to fall (`NQH`).
`nproc` reports 12 in this session;
`/proc/cpuinfo` lists 32 processors.
The processor count is the session's,
not the machine's:
the gate re-check's session reported 30.
Each shape ran in one `systemd-run --user --scope` with
`CPUQuota` 600%,
`MemoryMax` 20G,
`TasksMax` 512 and
`IOWeight` 1,
so this session had the least IO weight beside the others.
A 6-CPU quota makes the clone engine's slices 5 to 8 on this tree,
as the add's own record says.

#### Filesystem

Every fixture repository and every destination sit under the home dataset,
on ZFS,
source and destination in one dataset,
so rung 2 applies to every clone arm.
`zfs_bclone_enabled 1`,
`zfs_bclone_wait_dirty 1`,
`zfs_txg_timeout 5`.

#### The fixture tree

This repository's tracked working files as this round found them:
24,620 files and 763,309,482 bytes of regular files,
which is the merge commit `e86faecf31`'s tree with the new control's text in its test file
(no shipped code differs),
with the `filter=lfs` lines of `.gitattributes` stripped,
as the rounds before the fence stripped them,
and with the images kept as their own blobs.
The fixture is built once and copied per shape with `cp -a --reflink=always`.

#### The synthetic trees

10,001 files of 2,600,
8,800 and 26,000 bytes in ten flat directories,
the shapes the waiver names,
with one file's bytes committed each.
This round reran only the 2.6 KB tree,
because it is the one the choice bites on.

#### The source states

Held fixed over all rounds of a shape.
Usual is three tracked files rewritten and three files nobody tracks;
many:N is N tracked text files each with a line appended plus the three untracked;
clean is neither;
small:<size>:dirty is three tracked files rewritten plus the three untracked.
`sync --file-system`,
1.7 s and `update-index --really-refresh` follow the state,
and a run whose source does not show exactly the state's paths stops before any round.

#### Builds and arms

Release builds;
the prototype and `main` are built from this repository at `e86faecf31` and `9b07ba55bd` with
`cargo build --locked --offline --release --bin monochromatic-git-policy-cli`:
the prototype (sha256 `a1aa619a57219940b9c2da2e6f6c283a2d90770e98c52d30a1cbd50fe58c8706`),
`main` (sha256 `93f989f42f94ce851bb6b3727c1b481965147e43f8672ee2c27ad02213935b34`).
`main` is the tip this round merged,
not the `ad35d82537` the author's round and the re-check paired,
so the pair differs by the prototype only on top of current `main`,
and every `main`-side change since that pair is in both arms.
The prototype binary was built from the merge commit;
the only later change to the crate is a test file,
so the binary's behavior is the tip's.

The four arms are the prototype (P),
the same binary under another name (the control,
C),
`main` (M),
and Git's own checkout (G),
which is `main`'s own binary under a form the ladder does not read,
`git --no-optional-locks worktree add`,
forwarded as the person wrote it and recorded as `route: plain`.
Each arm runs as plain `git` through a PATH of its own,
with its own `HOME` and `XDG_RUNTIME_DIR`.
On the 2.6 KB dirty tree a fifth arm runs the prototype with the whole-tree rule set to 400,000,000 tracked bytes,
which is option B (arm B),
and its record names `route: Git's checkout`.

#### The round orders and the stopping rule

The four-arm shapes ran a Williams square:
each arm takes each position once per block of four rounds,
and each arm follows each other arm once;
the second block runs the rows reversed.
The five-arm shape ran a cyclic Latin square:
each arm takes each position once per block of five rounds;
the second block runs the rows reversed.
One warm-up round precedes each shape,
and 300 ms and `sync --file-system` after the state separate the arms of a round.

The stopping rule was fixed before the first round:
two blocks after one warm-up,
and one more block only if the identical arms' median ratios to `main` differ by more than 0.10.
The largest difference was 0.10,
at 20,000 changed paths,
and 0.04 or less on the other shapes,
so no shape took a third block.
Counted rounds are therefore eight for each four-arm shape and ten for the five-arm shape.
The identical pair shares a position once per block,
which the author's order never did;
the author's order gave the prototype and `main` positions 1 and 3 only,
and the control and Git's own checkout positions 2 and 4 only.
The author's wide ranges are reported as the author's,
and the balanced figures stand beside them.

#### Verification

Outside the timed window,
for every worktree every arm made:
`git status --porcelain` empty,
and `diff --recursive --no-dereference --exclude=.git` against Git's own checkout's worktree of the same round empty.
Every arm of every counted round passed both,
and every arm took the route it is named for:
"a partial clone" for P,
C and B where it clones,
"clones" for the whole rung,
"Git's checkout" for `main` on a dirty source and for B where it declines,
and "plain" for Git's own checkout.

#### Records

Each shape's round record and log are in the author's scratch folder beside this round's,
`~/temp/agent/clone-rung-partial-clone-on-the-fenced-rung-2026-11-21/scratch/keep/`,
named `measure-balanced-*.json` and `log-*-this-round.txt`;
the author's are `measure-fenced-*.json` and `log-*.txt`.
Two batches kept beside them are marked invalid:
one ran before the fixture's LFS attributes were handled,
the other synced the whole filesystem before every arm and its spread was the sync's.

### This repository's tree, three changed paths

The usual dirty state,
ratio to `main` in the same round,
median with the range over the eight counted rounds,
and wall times in brackets:

- `main` 1,567 ms (1,420 to 2,577),
  which is Git's own checkout after the clean proof;
- prototype 1,450 ms,
  **0.80** (0.61 to 3.35),
  at or below `main` in 6 of 8 rounds;
- its identical control 1,193 ms,
  0.73 (0.61 to 2.55),
  so the identical arms differ by 0.07 in the median;
- Git's own checkout 2,172 ms (1,332 to 3,204),
  and the prototype is 0.73 of it (0.45 to 2.05),
  below it in 7 of 8 rounds.

The identical pair's round-by-round ratio spans 0.18 to 3.18:
one round resolves nothing,
and the median of eight resolves about 0.1.
Stalls of 2.5 to 4.9 seconds hit different arms in different rounds (rounds 4,
5 and 7 of this shape),
so no single arm carries the range.

The author's round and the re-check's round of the same shape measured the same direction:
the author's round 0.64 of `main` (0.23 to 2.84) with a control of 0.67,
and the re-check's balanced round 0.69 (0.62 to 0.78) with a control of 0.74.
This round's median is 0.80 under a heavier hour (load 11.3 at the shape's end);
the gain over `main` is 0.20 of `main` in this round and 0.31 to 0.36 in the other two,
and its size is the hour's host,
not a property of the design.

Against Git's own checkout the same add is 0.73 in this round,
0.77 in the re-check's and 0.75 in the author's,
and `main` over Git's own checkout is 0.90 in this round,
1.10 in the re-check's and 1.14 in the author's:
the ratio varies by shape and hour,
which is why "Which comparison the waiver means" reads each shape's own figures.

### This repository's tree, clean

Not rerun this round.
The author's round measured `main` 861 ms (753 to 997),
which is the fenced clone rung;
prototype 824 ms,
0.95 (0.83 to 4.89);
its control 814 ms,
0.97 (0.76 to 1.10);
Git's own checkout 1,317 ms (1,177 to 1,993),
so the fenced whole rung is 0.66 of Git's own checkout there
(0.38 to 0.79).
The prototype and `main` run the same code on a clean source,
because the partial clone is the dirty source's path and nothing else,
so this line is a control and not a result:
the two agree inside the pair's spread,
and no clean add changes when this proposal lands.
Skipping it this round costs nothing the decision rests on,
because both arms are `main`'s code there.

### This repository's tree, many changed paths

#### 300 changed paths

`main` 1,482 ms (1,365 to 2,582);
prototype 1,050 ms,
**0.73** (0.36 to 2.49),
at or below `main` in 6 of 8 rounds;
control 1,003 ms,
0.69 (0.52 to 1.90);
Git's own checkout 1,365 ms,
and the prototype is 0.73 of it (0.43 to 1.80),
below it in 7 of 8.
The author's round measured 0.74 (0.64 to 1.28) with a control of 0.79 and 0.80 of Git's own checkout,
which this round reproduces.

#### 6,000 changed paths

`main` 1,465 ms (1,413 to 3,782);
prototype 1,255 ms,
**0.86** (0.35 to 0.90),
at or below `main` in 8 of 8 rounds;
control 1,230 ms,
0.82 (0.43 to 1.00);
Git's own checkout 1,254 ms,
and the prototype is **1.00 of it** (0.94 to 1.10),
below it in 4 of 8 rounds.

Three rounds have measured this shape:
this one at 0.86 of `main` with 8 of 8 rounds at or below,
the author's at 0.86 (0.57 to 1.01) with 0.98 of Git's own checkout,
and the re-check's at 0.93 (0.82 to 1.07) with 1.05 (0.92 to 1.28) of Git's own checkout.
Against `main` the shape is at 0.86 to 0.93 and not distinguishable from it across rounds;
against Git's own checkout it is **parity** in all three (0.98,
1.00 and 1.05),
never shown to be below it.
"At or below `main` in every counted round" is true of this round's `main` comparison and false of Git's own checkout,
and it is not used as a reason anywhere the clone is over Git's checkout.

#### 20,000 changed paths

`main` 1,569 ms (1,363 to 3,830);
prototype 2,341 ms,
**1.44** (0.63 to 1.72),
at or below `main` in 2 of 8 rounds;
control 2,217 ms,
1.53 (0.70 to 2.97);
Git's own checkout 1,322 ms,
and the prototype is **1.74 of it** (0.96 to 2.14),
below it in 1 of 8 rounds.
20,000 paths are 81 per cent of this tree,
which is the shape a bulk reformat or a licence-header sweep makes.
The re-check's one block of four rounds measured the same direction:
1.44 of `main` (0.43 to 2.99) and 1.68 of Git's own checkout (1.56 to 1.90).
This is the shape option F's changed-path bound would decline.

#### The write step's own cost

`git checkout-index`,
from the prototype's own step records:
14.8 ms at three paths,
33.5 ms at 300,
373.4 ms at 6,000 and
1,357.1 ms at 20,000,
which is 112 microseconds a path at 300,
62 at 6,000 and
68 at 20,000.
The author's round measured 13.5,
32.8 and 379.9 ms at the same first three points,
so the price per path is the same order in both rounds.

### The small-file trees the waiver names

#### The clean small trees, from the author's round

Not rerun this round,
because a clean source never takes the partial clone:
both arms are `main`'s code there.
Round-matched ratios to Git's own checkout at 2.6 KB,
8.8 KB and 26 KB:
`main` 1.19 (1.00 to 1.40),
1.04 (1.00 to 1.52) and 0.99 (0.89 to 1.56).
Ratios of medians (1.17,
1.04 and 1.08) are not used here,
because the method's ratios are round-matched (`TVC`).
At 26 KB the half is not distinguishable from zero.
The declined arm,
which is the whole-tree rule,
is Git's own checkout:
0.78 (0.69 to 1.09),
0.87 (0.85 to 1.31) and 0.93 (0.62 to 1.16) of Git's own checkout,
and 0.65,
0.86 and 0.81 of `main`.

#### The 2.6 KB tree with three changed paths

This round's balanced run,
ten counted rounds:

- `main` 171 ms (145 to 204),
  which is Git's own checkout on a dirty source;
- prototype 251 ms,
  **1.44** (1.23 to 2.28) of `main`,
  at or below `main` in 0 of 10 rounds;
- its identical control 246 ms,
  1.44 (1.19 to 4.55);
- Git's own checkout 200 ms,
  and the prototype is **1.30 of it** (1.13 to 1.98),
  below it in 0 of 10 rounds;
- the declined arm (option B) 201 ms,
  **1.10 of `main`** (0.73 to 1.45) and **1.04 of Git's own checkout** (0.58 to 1.12),
  which is parity with both.

The author's round measured the same direction:
A 1.44 of `main` (0.83 to 1.69) and 1.54 of Git's own checkout
(1.27 to 2.22),
B 0.91 of `main` (0.67 to 1.37) and 1.11 of Git's own checkout (0.92 to 1.38).
The re-check's round measured A 1.19 of `main` (0.93 to 1.97) and 1.31 of Git's own checkout (1.05 to 1.70),
B 1.03 of `main` (0.64 to 1.25) and 0.98 of Git's own checkout (0.69 to 1.40).

Across the three rounds A is 19 to 44 per cent above `main` and 30 to 54 per cent above Git's own checkout,
and B is parity with both.
The direction holds in all three:
cloning a dirty tree of many small files loses,
and the whole-tree rule gives it back to Git's checkout at parity,
not at a gain.

### The step split

Medians of the dirty prototype add's own record,
this repository's tree at the usual state,
from this round's balanced run,
in ms,
with `main`'s beside it where it has the step.

The ladder's own checks and reads:
`checkSource` 36.6 (`main` 35.9),
`git ls-files` 111.0 (105.1),
`git check-ignore` 50.2 (43.1),
`git status` 52.0 (46.3),
and `git diff-index` 4.4,
which is the second staged-path listing.

The fence:
`git check-attr` 39.2,
`git ls-tree` 260.6,
`conversionFence` 26.6,
and `conversionBesideTheClone` 31.4,
whose wait overlaps the copies.
`targetTreeDifference` is 0.0 because the add names no commit.

The partial clone's own work:
`partRead` 29.0,
`partClassify` 89.1,
`partStaged` 0.0,
`partIndex` 8.7.

The copies:
`cloneEntries` 261.4,
beside the fence's reads.

The write:
`writeConversions` 7.8 and `git checkout-index` 14.8.

The rest:
`cloneIndex` 46.4 (the waiting index,
its own step since the fence),
`copyIgnored` 47.7,
`finishRung` 15.7,
`gitAdd` 4.0.
`main`,
at the same state,
spends `gitAdd` 1,215.6 (Git's own checkout after its checks),
and Git's own checkout without the checks spends `gitAdd` 1,827.0.

The fence's `git ls-tree` is the largest single item in a partial clone's add,
and it is the fence's cost,
not the partial clone's:
`main` pays it on every clean add,
and the faster size reader the project accepted as a follow-up
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
step 10's notes)
would remove most of it.
The split's own work is 127 ms,
the copies 261 ms with the fence's reads beside them,
and the write 23 ms.
These are one round's medians under other sessions' load,
so they place the steps against each other,
not against another hour's host.

### Correctness

The class controls are 29:
the 28 partial-clone tests in three sidecar files,
and `a_dirty_source_gets_the_clone_rung_where_files_can_be_cloned`.
All 29 pass on this host,
and the reflink-needing ones really ran,
because the fixture root is on ZFS and can clone.

The seeded differential sample ran unchanged with this round's binaries:
seed 11,
150 combinations,
two checks per combination,
300 of 300.
Each combination is a fresh repository of twelve tracked paths with a link,
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
Of the 150,
74 took the partial clone and 76 ended in Git's checkout,
which is the refusal list firing
(skip-worktree,
assume-unchanged and intent-to-add are three states of the sample,
and any combination holding one ends the rung).
The log is `sample-seed11-this-round.log`.

The red proof of the new control was taken here:
with the second staged-path listing's answer ignored in `clone_the_unchanged`,
`a_path_staged_after_the_source_check_is_refused` fails and the other 27 pass
(`red-plant-second-listing-ignored.log`),
so the wiring the re-check's sixth plant left green is guarded now.

Three limits of the sample are stated rather than fixed.
After applying the states it waits 1.1 s and runs `update-index --really-refresh`,
which rewrites a touched file's entry,
so a path touched only is cloned with the refresh and written without it
(the re-check demonstrated both,
`keep/refresh-collapse-demo.log` in its folder);
"touched only" is therefore not the state its name says in this sample.
The `racy-entry` state stages new content,
so the whole-second racy rule is exercised end to end only by the unit control `a_racy_entry_is_written`.
And the sample never draws a different new `HEAD`;
the re-check's variant sample did,
found no byte difference,
and saw 22 of 43 attempts at a differing `HEAD` end after the clone rung and finish in Git's checkout,
all of them adds whose differing tree changes a path a conversion rule reaches,
which the up-front skip now covers:
on the merged binaries none of the 43 ends after the clone rung.
What the skip does not cover is the path the size proof hands to Git,
which no sample draws ("What this leaves owed").

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
after the paragraph that ends "The ignored-state copy then runs as for Git's checkout."
(`SPEC.md:2820`),

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
> and the conversion fence this section describes carries it.
> Every other tree path is written through the one `git checkout-index` this section describes,
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
> That cost is the check that rung 2 requires and no snapshot can carry out here;
> the measurements,
> the options and
> their costs are with the integration pass that found it.

to:

> A source with uncommitted changes clones every tracked path its proof settles and lets Git write the rest,
> so the waived cost ends where rung 2 serves the add;
> a linked worktree with uncommitted changes still pays the source checks and then ends in Git's own checkout,
> and step 11's verified dirty restore is still the snapshot's.
> The measurements that place the clone path are the partial-clone proposal's.

The waiver record `doc/decision/cli-git-zfs-clone-rung-dirty-source-waiver.md` ends with the proposal it names;
if this one is accepted,
that record says how.

`doc/planning/cli-git-btrfs-worktree.md`,
the rung 2 paragraph under "Reading 2,
overlayfs as the mechanism of a cheap worktree",
which still states the clean-source rule as the design:

from:

> Rung 2 (reflink from a clean source) applies and is the useful rung there because the checkout is slow.

to the same design the `SPEC.md` first sentence states:
on btrfs,
XFS,
overlayfs and ZFS a source whose tracked paths the proof settles is cloned,
and a source with changes clones the paths the proof settles while Git writes the rest,
so the word "clean" no longer describes rung 2's source.
The snapshot rung's own clean-source requirement (the rung list's rung 1) is unchanged,
and the btrfs round's measurement entries that say "from a clean source"
(`B0` and the option B line) keep their text:
they record what that round measured,
and the note in this list says what rung 2's source is now.

`doc/planning/cli-git-clone-rung-conversion-fence.md`,
the paragraph under "What these costs imply for the partial-clone proposal" that says the partial clone
"cannot be put to the project with this fence as it stands":

from:

> The partial-clone proposal
> (`docs/planning-cli-git-clone-rung-partial-clone-of-a-dirty-source`) waits on this fence,
> and an earlier estimate put a fenced dirty partial clone at 0.97 to 1.08 times `main`
> from the fence's 0.12 to 0.23 of `main` against the partial clone's gain of 0.14.

to:

> The partial-clone proposal went to the project on this fence,
> and every figure it puts to the owner is measured with the fence compiled in,
> so the waits-on sentence this paragraph replaced no longer applies;
> the fence's own costs it names are still the fence's,
> and the proposal reports them beside its gains.

That paragraph's own cost entries are left as they are:
they are the fence's measurements,
not this proposal's claims.

## Options for the owner, with their measured cost

Every figure is a ratio to `main` in the same round unless it names Git's own checkout.
This round's figures are the balanced runs in this document;
the author's round and the re-check's round are named where their figures are used.
The two rules are separable from each other and from the clone,
so the options are grouped:
the clone,
the whole-tree rule,
the changed-path bound and the per-file rule.

### A: the partial clone for every dirty source, no rule

The usual shape 0.80 of `main` (0.61 to 3.35),
with its control at 0.73,
at or below `main` in 6 of 8 rounds;
300 changed paths 0.73 (0.36 to 2.49),
6 of 8;
6,000 changed paths 0.86 (0.35 to 0.90),
8 of 8.
Against Git's own checkout:
0.73,
0.73 and 1.00,
the last at parity.
The clean path is unchanged,
because the partial clone is the dirty source's path only
(the author's control 0.95 with its identical pair at 0.97).
Its losses:
the 2.6 KB dirty tree at 1.44 of `main` and 1.30 of Git's own checkout,
20,000 changed paths at 1.44 of `main` and 1.74 of Git's own checkout,
and the fence's own costs on top of both.
It ends the waiver's dirty half on the few-paths shapes,
holds parity at 6,000,
and does not touch the widened small-file half,
which is `main`'s own cost now.

### B: A with the whole-tree size rule

On this tree it clones,
so A's numbers are its.
On the 2.6 KB dirty tree it declines and takes Git's checkout:
1.10 of `main` and 1.04 of Git's own checkout,
which is parity.
On the clean small trees it declines at 0.65,
0.86 and 0.81 of `main`,
and at 0.78,
0.87 and 0.93 of Git's own checkout.
It needs a threshold,
and for dirty adds the bracket is 26 MB to this tree's own 763 MB;
the quantity is the index entries' recorded stat sizes the rule sums
(`prototype_tree_too_small` in `worktree_ladder_run.rs`),
not the blobs' sizes.
Its scope and placement are part of the decision:
at the clone rung's choice,
after the snapshot rung is declined,
on the filesystems rung 2 serves.
The prototype evaluates it in `choose_rung` before the snapshot is considered,
so on a btrfs host with a subvolume source it would also turn off the snapshot rung,
and it applies to clean sources as well,
while every figure in this section is ZFS.
That placement is a measurement artifact,
not the design.

### E: the whole-tree size rule alone, with no partial clone

This is a composition of measured arms,
not a new measurement.
Where the rule declines,
the add is Git's own checkout:
1.04 of Git's own checkout on the dirty 2.6 KB tree,
and 0.78,
0.87 and 0.93 on the clean small trees.
Where it does not decline,
the add is `main`'s:
on this tree (763 MB) that is 1.00 by definition.
So E ends the clean small-file half and the dirty small tree at parity,
and leaves the dirty half under the waiver,
where A removes 0.20 to 0.36 of `main` on the usual shape and 0.26 at 300 paths.
Its cost is one count and the threshold,
unit and placement question stated in this section.

### F: A with a changed-path bound

A's figures on every shape with fewer changed paths than the cut,
which here is 300 (1 per cent of the tree) and 6,000 (24 per cent),
and Git's own checkout above the cut.
The one measured loss the bound answers is 20,000 changed paths (81 per cent):
A is 1.44 of `main` and 1.74 of Git's own checkout there,
where a decline makes the add Git's checkout.
No round ran a cut,
so the cut,
and the count it costs at the clone rung's choice,
are owed.

### C: the per-file size rule

Not rebuilt this round.
The round before the fence measured it at 1.05 (0.96 to 1.10) with a 4 KB cutoff and 1.90 (1.43 to 2.11) at 32 KB on this
tree,
against that run's clone-everything arm at 0.77 with a control of 0.76,
and writing every unchanged path cost 2.80,
3.59 and 4.30 on the small trees.
This round gives the reason to keep that verdict:
the write step costs 62 microseconds a path at 6,000 and 68 at 20,000,
against a clone of the same file,
so moving an unchanged path from the clones to the write step loses at every size measured.
Its finding is "this write step loses",
not "no per-file rule can win":
a write step as cheap as Git's checkout was not built.

### D: no partial clone, keeping step 3 under the waiver

A dirty add is `main`'s,
which is Git's own checkout plus the clean proof:
1.00 by definition,
0.90 of Git's own checkout on this tree in this round and 1.10 to 1.14 in the two other rounds,
1.01 at 300 changed paths,
1.18 at 6,000 and 0.84 on the 2.6 KB tree in this round.
It costs nothing new,
leaves the waiver in place,
and leaves the dirty source's 0.20 to 0.36 of `main` that A removes.
It ends nothing.

### Ranking

**B > F > A > E > D > C.**

- B over F:
  B's decline answers a loss measured in three rounds
  (the 2.6 KB dirty tree at 1.19 to 1.44 of `main` and 1.30 to 1.54 of Git's own checkout),
  and it also ends the clean small-file half;
  F's bound answers a loss measured in two rounds at 20,000 changed paths,
  which a bulk rewrite makes.
  B's threshold has measured points bracketing the crossing;
  F's cut has one measured point above it and none below,
  so F's evidence is thinner.
- F over A:
  F costs one count and bounds the one shape where the clone is measured well above Git's own checkout at many changed
  paths (1.74 there,
  1.68 in the re-check's block);
  A has nothing there.
- A over E:
  A removes 0.20 to 0.36 of `main` on the daily shape and 0.26 at 300 paths;
  E leaves the dirty half under the waiver and ends only the small-file half.
- E over D:
  E ends the small-file half at parity with Git's own checkout where it declines,
  and the dirty small tree at the same parity,
  for one count and a threshold;
  D ends nothing.
- D over C:
  C's write step costs 62 to 68 microseconds a path,
  and its clean path would pay that for every file under its cutoff,
  which on the small trees is 2.80 to 4.30 against D's parity.
  C therefore loses more than it gains,
  and it ends no waiver.

### The orders the figures support

A now,
then the rule once its threshold is measured (which makes it B),
then the bound once its cut is measured (which adds
F);
or B and F together when both thresholds are measured first.
The two rules are independent:
the whole-tree rule is a size test before the proof,
and the changed-path bound is a count after the tree is read,
so either can land with or without the other.
E is the whole-tree rule without the clone.

## Which comparison the waiver means

The waiver record's sentence is "If it is accepted and lands with a dirty add no slower than `main`'s,
this waiver ends
with it".
It was written when `main` did not take the clone rung on ZFS,
so "`main`'s" is the add the wrapper made then:
Git's own checkout plus the checks of that day.
Today's `main` is that add plus the clean proof step 3 added,
which is the cost the record waives,
so today's `main` cannot be the baseline:
an add that still carried the waived cost would satisfy a sentence meant to end the waiver for it.
The record says the same in its own words:
"On `main` the wrapper does not take rung 2 on ZFS,
so it never runs that proof on this host."
The criterion here is therefore **Git's own checkout**,
the stricter of the two,
and today's `main` is reported beside it as the second figure.
This is the reading the gate re-check gave it.

### The dirty half, shape by shape

- Usual,
  three changed paths:
  A is 0.73 of Git's own checkout (0.45 to 2.05),
  below it in 7 of 8 rounds.
  Met.
  B clones on this tree,
  so B's figure is A's.
- 300 changed paths:
  A is 0.73 (0.43 to 1.80),
  below in 7 of 8.
  Met.
- 6,000 changed paths:
  A is 1.00 (0.94 to 1.10),
  below in 4 of 8,
  and the other two rounds measured 0.98 and 1.05.
  Parity,
  so "no slower" holds as parity and is not shown as a gain.
- The 2.6 KB dirty tree:
  A is 1.30 (1.13 to 1.98),
  below in 0 of 10.
  Not met.
  B,
  which declines there,
  is 1.04 (0.58 to 1.12),
  which is parity:
  what the sentence asks,
  met at parity.
- 20,000 changed paths:
  A is 1.74 (0.96 to 2.14),
  below in 1 of 8,
  and the re-check's block measured 1.68 (1.56 to 1.90).
  Not met.
  F's bound would decline it.

### The small-file half, option by option

The half is a clean add of many small files,
and its cost is already inside `main`:
`main` is 1.19 (1.00 to 1.40),
1.04 (1.00 to 1.52) and 0.99 (0.89 to 1.56) of Git's own checkout on the 2.6 KB,
8.8 KB
and 26 KB trees,
round-matched from the author's records.

- A and D leave it:
  a clean source never takes the partial clone,
  and D changes nothing,
  so the half stays `main`'s own cost.
  A sentence that ends the whole waiver with the dirty half therefore leaves the half without cover,
  and "The decision asked for" puts that to the owner.
- C was measured not to end it:
  its write step loses on every unchanged path it moves.
- B ends it where the rule declines,
  at 0.78 (0.69 to 1.09),
  0.87 (0.85 to 1.31) and 0.93 (0.62 to 1.16) of Git's own checkout,
  which is parity to a gain.
  E does the same without the clone.
- Where B or E do not decline,
  the half stays as it is.

### What the sentence therefore supports

The sentence is met by A on the few-paths dirty shapes (the usual one and 300),
holds at parity on 6,000,
and is not met on the small dirty tree or at 20,000 changed paths;
B meets it at parity on the small dirty tree by declining,
and F meets it at 20,000 by declining there.
So the waiver's dirty half ends with A on a source with a handful of changed paths,
and only with B or F on the two shapes where A is over Git's own checkout.
That is the shape this repository's main checkout is usually in as an assumption,
not a measurement:
the one observation taken for this round,
2026-11-21T22:20:51-05:00,
read `git --no-optional-locks status --porcelain` in
`/home/user/Monochromatic` as four modified tracked paths and no untracked path.
The small-file half ends only where a size rule declines,
which A and D do not do.

## Answers to the gate re-check

The review is
`~/temp/agent/gate-recheck-partial-clone-proposal-on-the-fenced-rung-2026-11-21/keep/review.md`
(Claude Sonnet 5.5,
unapproved and non-final),
with its scripts,
logs and round records beside it.
Every finding is fixed in the text,
and the ones that needed measurement were rerun in this round.

### H1: the waiver's baseline, and the small-file half

Fixed in "Which comparison the waiver means".
The baseline the record names is Git's own checkout with the checks the pre-step-3 wrapper paid,
not today's `main`,
which carries the waived proof;
Git's own checkout is the stricter figure and the criterion,
and today's `main` is reported as the second figure.
That section gives,
per shape and per option,
whether the sentence is met,
with Git's own checkout as the denominator and today's `main` as the second figure.
The false sentence about the small-file half is replaced by what each option does to it:
A and D leave it,
C was measured not to end it,
and B or B's rule alone ends it where the rule declines.
"The decision asked for" gained the question the finding asked for:
what the small-file half becomes if A or D is chosen.

### M1: the 6,000-path result and "every counted round"

Partly reproduced,
and the unsupported parts are dropped.
This round reran the shape at 0.86 of `main` (0.35 to 0.90) with 8 of 8 rounds at or below,
matching the author's 0.86 and inside the re-check's 0.93 (0.82 to 1.07),
so 0.86 of `main` stands where the re-check could not reproduce it.
Against Git's own checkout the three rounds are 0.98,
1.00 and 1.05,
which is parity,
not a gain,
so the shape is no longer said to be below Git's own checkout.
"Every counted round" is kept only for the `main` comparison this round measured,
and it is no longer a reason for A over D where the clone is over Git's checkout.
"Ends the dirty half" is said only for the few-paths shapes.
The changed-path bound the finding points to is option F.

### M2: the stated rotation

Fixed by rerunning the shapes the decision rests on in a position-balanced order,
and by replacing the method sentence with what ran.
The author's order rotates one place a round and reverses on even rounds,
which gives four orders over four arms;
the prototype and `main` took positions 1 and 3 only,
and the control and Git's own checkout positions 2 and 4 only,
so the identical pair never shared a position.
This round's design is a Williams square for four arms (each arm in each position once per block,
each arm following each other arm once;
two blocks,
the second reversed)
and a cyclic Latin square for five arms (position-balanced only;
two blocks,
the second reversed).
The one-stall reading of the author's range top is dropped;
the wide range is reported as the author's own round's,
and the balanced figures stand beside it.

### M3: the small dirty tree, and option B there

Every measurement is reported instead of one:
the author's 1.44 (0.83 to 1.69) of `main`,
the re-check's 1.19 (0.93 to 1.97),
and this round's 1.44 (1.23 to 2.28),
so the text says "19 to 44 per cent" rather than "about 40 per cent".
Against Git's own checkout the three are 1.54 (1.27 to 2.22),
1.31 (1.05 to 1.70) and 1.30 (1.13 to 1.98),
so it says "30 to 54 per cent" there.
B's decline on that tree is parity with `main` and with Git's own checkout,
not a gain:
1.10 (0.73 to 1.45) of `main` and 1.04 (0.58 to 1.12) of Git's own checkout in this round,
0.91 and 0.98 in the author's,
and 1.03 and 0.98 in the re-check's.
0.91 is dropped from the verdict text and quoted only as the author's measurement.
The direction all three rounds measure (A loses there,
B is at parity) is unchanged.

### M4: option B's bracket and the summed quantity

Fixed.
The bracket for a rule on dirty adds is 26 MB to this tree's own size,
the two points this round measured;
260 MB and 780 MB belong to clean points and a tree of the round before the fence.
The quantity the rule sums is named:
`prototype_tree_too_small` in `worktree_ladder_run.rs` sums the index entries' recorded stat sizes,
and this tree's figure for that quantity is given beside the bracket.

### M5: the red proofs

Corrected,
recorded and closed where it could be.
The section now says which guards stand and which catchers moved,
records the re-check's six re-plants and the one that stayed green,
and requires the re-plants again before a merge.
The one that stayed green was the second staged-path listing computed and ignored:
the layer looked redundant while the split's first answer is right.
It is an independent second answer on the split's own comparison,
and in the control's scenario the refusal prevents no wrong result:
the file is untouched,
so the clone holds the commit's bytes either way,
and a file edited and staged after the check is already caught by the clone engine's descriptor comparison.
What the control guards is that listing's wiring,
by moving the source's index after the source check so the two answers disagree through time.
The new control `a_path_staged_after_the_source_check_is_refused` moves the index after the check,
and with the listing's answer ignored it fails while the other 27 pass ("Correctness").

### M6: `main` moved, and the second follow-up

Fixed.
Both of step 10's accepted follow-ups are named in "What the rung does today",
with the skip's merge,
its no-dirty-guard shape and what it removes.
This round merged `main` at `9b07ba55bd`,
and the same-tree adds measured here are untouched by the skip because it returns when the add names no commit.
"What this leaves owed" carries the differing-tree interaction as a limit of the measured rounds,
not of the design.

### M7: the option set

Unbundled.
The whole-tree size rule is its own option (B's rule alone,
option E),
the partial clone with it is B,
and the changed-path bound is option F,
measured at 20,000 changed paths.
Every order is reachable:
"A now,
then B's rule once its threshold is measured" is written as an order in "Options for the owner",
and so is "B and F together".
The ranking is restated with a reason for every adjacent pair.

### L1: the text changes

Fixed.
The `SPEC.md` anchor is now the paragraph's own last sentence,
verbatim ("The ignored-state copy then runs as for Git's checkout.").
The plan replacement covers the orphaned sentence too.
The btrfs design doc is relisted,
with the one line that states rung 2's source as current design,
and the note that its measurement entries keep their text.
The fence doc's stale paragraph is in the list with its replacement.

### L2: the prose rules

Fixed by this rewrite.
Relative pointers are replaced by the heading,
tag or path they meant,
including inside the new `SPEC.md` text,
which names the fence's own subsection rather than pointing at it by position.
Bold-label bullets are gone:
every labeled point is a heading with prose under it.
The Status names the merge commit and its time instead of a bare date.

### L3: ratios of medians

The waiver section and the small-tree paragraphs quote round-matched medians with their ranges.
The clean small trees are the author's records re-summarized that way:
`main` is 1.19 (1.00 to 1.40),
1.04 (1.00 to 1.52) and 0.99 (0.89 to 1.56) of Git's own checkout,
and at 26 KB the half is not distinguishable from zero.

### L4: the measurement build's unguarded arm

Recorded as a limit,
not changed.
`CloneFence::BeforeTheClone` exists only under the `measurement-routes` feature,
which no shipped build enables,
and a build with it must not be used on a source with changes;
"Limitations" says so.

### L5: the host line

Fixed.
The host line says the processor count is the session's,
not the machine's,
and the pending whole-suite figure stays with the gate image,
as the proposal already said.

### L6: what the sample can and cannot show

The three limits are stated beside the sample's counts:
the refresh after the states rewrites a touched file's entry,
so "touched only" is not the state its name says in this sample;
`racy-entry` stages new content,
so the whole-second racy rule is exercised end to end only by its unit control;
and the sample never draws a different new `HEAD`,
which the re-check's variant did,
with no byte difference found.

### L7: a source dirty only by untracked files

Stated,
not measured.
That source now takes the partial clone with no changed path;
no shape measures it,
and "Limitations" says what it would add over a whole clone.

## The first gate review's findings, answered

The review is `~/temp/agent/gate-review-of-the-partial-clone-proposal-on-sonnet-2026-11-20/keep/review.md`,
with its logs and scripts beside it.
Each finding is fixed,
recorded as a limit,
or superseded by a change `main` has since taken.
"Answered" refers to the state the author's round left,
and this round's corrections to those answers are in "Answers to the gate re-check".

### H1: a clone keeps the source's bytes where a checkout conversion would write others

Answered by the fence,
not by this proposal.
The project chose the fence's option B on 2026-11-20,
`c423010a4e` merged it into `main` on 2026-11-21,
and the fence now hands every path whose checkout would convert it to Git.
A partial clone reaches the fence through the same beside-the-copies shape,
so a path it clones is one both proofs carry,
and the control `a_path_a_conversion_rule_reaches_is_written_by_git` shows the case end to end:
a `*.txt text eol=crlf` rule over LF files,
the path written by Git,
the served bytes `\r\n`.
What remains of the class is the fence's own limits,
an ignored untracked `.gitattributes` among them,
recorded in the fence's proposal and in "Limitations".

### M1: the proof cannot see wrong bytes in a cloned path

Fixed by dropping the proof and by the second answer.
The rung no longer claims a `git status` proof it does not have;
what it has is the split's per-path proof,
the second staged-path listing,
the index built from the tree,
and the fence's write.
The staged class has its own comparison and,
since this round,
a control of its wiring.

### M2: a dirty linked worktree is still refused

Fixed in the text,
the lift owed.
"States that still refuse the whole rung" names it with `restorable_here`,
and the new `SPEC.md` text names it too.
Letting the clone rung take one needs a check of its own and a measurement of its add.

### M3: measurement statements the data does not support

Superseded.
The author's round's numbers are its own,
every figure has its identical-binary control beside it,
the host and the quota are stated,
and the binaries are named by commit and sha256.
The author's 0.74 and the review's 0.84 to 0.86 are kept only as the round before the fence's,
and the comparison they belong to is named.

### M4: the gain's size is not reproduced

Answered by measuring the fenced rung,
and this round reran it.
The author's round measured 0.64 of `main` with a control of 0.67 on this tree;
the re-check's balanced round measured 0.69 (0.62 to 0.78) with a control of 0.74;
this round's balanced round measures the figures in "This repository's tree,
three changed paths".
The gain is real in all three and its size is the host's;
"never slower" is not claimed,
and every ratio here carries its range.

### M5: a source with many changed paths

Measured to 6,000,
and the loss the first review found is gone at 300 paths.
The write step is 63 microseconds a path at 6,000,
and the first review's 1.54 came from a rung that wrote the paths through a slower shape and then proved the worktree;
the fenced rung writes them through the waiting index and proves nothing after.
This round's rerun covers 300,
6,000 and 20,000 changed paths,
and at 20,000 the clone is over Git's own checkout,
which is why the changed-path bound is an option.

### M6: the ranking and the options

Redone on the author's round's figures,
then unbundled and reranked in this round:
the size rule and the changed-path bound are separate options,
and every option says what it does to each half of the waiver.

### M7: incomplete and inaccurate design text

Rewritten.
The index is built from the source's `HEAD` tree,
a path under a link is written,
the new `SPEC.md` subsection follows the fence's paragraph,
the stale "saves disk,
not time" is gone,
and the places listed in "Where the design text changes" are the ones `main` now has.

### L1: the untracked refusal's origin

Fixed by citing `faede9969b` and the module doc separately.

### L2: no control through the ladder

Fixed with two controls,
which pass on this host.

### L3: an ignored untracked `.gitattributes`

An inherited fence limit,
not measured in either round.

### L4: the pool wait

An inherited limit,
not measured in either round.

### L5: the commit message and the fixture

Partly closed.
The corrective comment on `dba1276a06` is on the commit
(`gh api repos/Aquaticats/Monochromatic/commits/dba1276a06ff4deaee2ca2b83ae2acc1edb97d0e/comments`).
The fixture keeps this repository's own `.gitattributes` except for the `filter=lfs` lines,
as the rounds before the fence did,
so an LFS-attributed tree's round is still owed;
the round established why the fixture cannot keep those lines without an LFS object store:
with the standard LFS filter configured,
a repository whose blobs are the images themselves reads every image as modified,
and without the filter configured,
the fence refuses the source it cannot prove
("the lfs filter is not the standard Git LFS definition").

## Limitations

### The fence's `git ls-tree` is the largest single step of a dirty add on this tree

260.6 ms of the 1,450 ms usual add in this round's prototype record,
and 219 to 250 ms of the author's 1,012 ms one;
71.8 ms of a 251 ms small-tree add.
It is the fence's cost,
`main` pays it on every clean add,
and the faster size reader the project accepted would remove most of it.

### A dirty source of many small files is slower than `main`

1.44 (1.23 to 2.28) with a control of 1.44 on 10,001 files of 2.6 KB,
and 1.30 (1.13 to 1.98) of Git's own checkout.
Across three rounds the figure is 1.19 to 1.44 of `main` and 1.30 to 1.54 of Git's own checkout.
The whole-tree rule takes it to parity:
1.10 of `main` and 1.04 of Git's own checkout.
No design here makes cloning a small file cheaper than Git's write of it.

### A dirty source with many changed paths is slower than `main`

1.44 of `main` (0.63 to 1.72) and 1.74 of Git's own checkout (0.96 to 2.14) at 20,000 changed paths,
81 per cent of this tree,
with the re-check's one block at 1.44 and 1.68.
A changed-path bound (option F) would decline it;
no round ran a cut.

### A linked worktree with uncommitted changes is still refused

A session working in `.claude/worktrees/<name>` gets Git's checkout,
not the partial clone.

### A whole-tree rewrite was not measured

The closest point is 20,000 changed paths of 24,620,
where the write step costs 68 microseconds a path and the add is 1.44 of `main`;
the remaining 4,620 paths were not exercised,
and the estimate for all of them is about 1.7 s of Git's write on top of the clones and the fence.

### The threshold is not pinned, and its unit may not be bytes

The bracket is 26 MB to 763 MB of the quantity the rule sums (index stat sizes),
on trees of 10,001 and 24,620 files,
and file count and mean size were never varied apart in one disk's worth of measurement.

### This repository's own tree sits about 1.3 MB above the pinned size limit

Its tracked files summed to 763,289,228 bytes when the limit was pinned
(the build's measurement over 24,621 paths),
and they read 763,409,836 bytes over 24,626 paths on 2026-11-22,
so the tree clears the 762,000,000-byte limit by about 1.3 MB and its own dirty adds clone on that margin alone.
That margin is where the measured bracket's top fell,
not a value the fence chose.
If the tree shrinks below the limit its own dirty adds take Git's own checkout and print the reason,
until a measurement below this tree's size moves the limit.

### The size rules are environment variables of the prototype

A shipped rule is a design decision with a recorded threshold,
not a variable.

### A build with the fence's measurement routes must not take a dirty source

`CloneFence::BeforeTheClone` exists only under the `measurement-routes` feature,
which no shipped build enables (`Cargo.toml`),
and `clone_with_the_fence` has no `source.dirty` guard on that arm,
so such a build would clone a dirty source whole and install its index.
Not a shipped path,
and a measurement build must not be pointed at a source with changes.

### A source dirty only by untracked files was not measured

It now takes the partial clone with no changed path,
where before this proposal it ended in Git's checkout;
the split's 127 ms and the fence's reads are what it adds over a whole clone,
and no shape measures it.

### An ignored untracked `.gitattributes` is still invisible to the fence's listing

An inherited limit,
not measured in either round.

### A file cloned within the pool's transaction timeout waits for the pool

An inherited limit,
not measured in either round.

### An LFS-attributed source was not measured

The fixture keeps this repository's own `.gitattributes` except for the `filter=lfs` lines,
and the round established why neither alternative is a measurement:
with the standard LFS filter configured,
a repository whose blobs are the images themselves reads every image as modified,
and without the filter configured,
the fence refuses the source it cannot prove
("the lfs filter is not the standard Git LFS definition").
The round is owed.

### One round resolves about 0.1, and this round ran under other sessions' load

The identical arms' median ratios differ by 0.00 to 0.10 across the five shapes,
and one round of the pair spans 0.18 to 5.33,
so the ratios here are medians of eight or ten rounds and a shape's ratio moving by less than 0.1 between rounds is
not a finding.
The one-minute load average was 4.2 to 11.3 at the end of the shapes and 9.9 to 14.4 over the fifteen minutes before.

### The sample's three limits

The refresh after the states rewrites a touched file's entry;
`racy-entry` stages new content,
so the whole-second racy rule is exercised only by its unit control;
and the sample never draws a different new `HEAD`.
They are stated in "Correctness" beside the sample's counts.

### The check the source passes no longer reads the diff to content

The split's proof is Git's stat trust taken in process,
so an edit that keeps size and times is caught by the change time,
the inode or the owner,
and an engine that cannot see one of those would clone the edit.
This is Git's own trust,
not a weaker one.

### `worktree_clone.rs` is over the package's line limit

The entry point this round added belongs in a sibling when the rung is written for a merge.

## What this leaves owed

- The gate image's whole suite at this branch's tip,
  which is what a merge runs again.
  On this host 913 of the 930 `worktree::` tests pass;
  the 17 that fail are this host's and not this rung's:
  eight create a name that is not UTF-8 and this ZFS dataset refuses it with `EILSEQ`,
  and nine are the parallelism controls that pass alone and fail in a batch under load.
  The figure is the area run of the round before the fence;
  this round ran the class controls and the sample,
  not the area.
- The six guard removals re-planted against this tip before a merge,
  with the new control among the reds this time
  (this round planted the sixth only,
  and it is red).
- A `git checkout-index` ceiling for the ladder:
  the run goes through the captured runner,
  which bounds it,
  but `ladder_step_ceilings.rs` does not classify it.
- A round on a tree whose `.gitattributes` keeps its `filter=lfs` lines and whose blobs are LFS pointers.
- A round on a source that is a linked worktree,
  once the refusal is lifted.
- A measured threshold for the whole-tree rule,
  its unit,
  and its placement,
  and the same measurement on XFS,
  overlayfs and btrfs.
- A measured cut for the changed-path bound,
  and the count it costs at the clone rung's choice,
  which no round here priced.
- The size rule on its own (option E):
  where it is placed if the partial clone is not taken,
  and what it costs on a clean add above its threshold.
- A differing new `HEAD` whose differing tree changes a path the size proof hands to Git:
  that class still ends after the clone rung and pays the copies,
  and the up-front skip does not cover it.
  The class the skip does cover,
  the differing tree that changes a path a conversion rule reaches,
  is what the re-check's variant sample measured 22 of 43 attempts ending after the clone rung on the pre-merge tip;
  on the merged binaries none of the 43 ends there,
  and an add whose differing tree changes no fenced path does not end at all.
  No sample draws the size-proof class.
- The split of the source check,
  so the clone rung stops paying the untracked scan and the trash question,
  measured at about 145 ms of a dirty add here.
- The one-`git status` classification of the source's changes,
  whose parts are measured and whose combination is not.
- A cheaper write step,
  if the per-file rule is ever wanted again.
- The ladder's full routing control,
  with an `AddPlan` and both rungs on a clone-capable source,
  beyond `a_dirty_source_gets_the_clone_rung_where_files_can_be_cloned`.
- The `subvolume_info` misread of what `BTRFS_IOC_GET_SUBVOL_INFO` answers,
  which keeps eight controls red in any btrfs guest
  and is `main`'s source as much as this branch's
  (`doc/troubleshooting/btrfs-get-subvol-info-returns-one-on-success.md`).
- The root-only btrfs controls' guest story:
  seven controls assert the branch a caller without `CAP_SYS_ADMIN` meets,
  and the guest that runs the btrfs controls runs them as root,
  so either that guest gets an ordinary account or each control states both branches.
- What handed a landed worktree's pool to the resident service in that guest,
  and the two guest limits a control met there,
  a descriptor cap and a 300-second timeout,
  which no round has diagnosed.

## The decision asked for

Answered on 2026-11-21:
the project chose the option its record calls "clone with limits",
which is the partial clone with the whole-tree size rule and the changed-path cut,
and the answer is `doc/decision/cli-git-clone-rung-partial-clone-of-a-dirty-source.md`.
What is below is the record of what was asked and of the figures it rested on.

Two questions were asked,
not one list of options,
because the rule is separable from the clone:
the clone,
which is A without a bound,
F with the changed-path cut,
or D with no clone;
and the whole-tree size rule,
yes or no.
Their products are A,
B (A with the rule),
F,
F with the rule,
D and E (D with the rule),
and F with the rule is the seventh,
which with the cut pinned is the one the decision takes.

### The partial clone: A, F or D

A is the clone as it is measured.
F adds the changed-path bound,
which answers the one shape measured over Git's own checkout at many changed paths,
and costs one count and an unpinned cut.
D keeps step 3 under the waiver and leaves the usual shape's 0.20 to 0.36 of `main` in place.

### The whole-tree size rule: B, E or neither

B is the rule with the clone.
E is the rule alone,
with no partial clone,
which ends the clean small-file half and the dirty small tree
and leaves the dirty half under the waiver.
Neither leaves the small-file half where it is:
it stays `main`'s own cost.
Either needs a threshold,
its unit and its placement decided;
both are options for the owner,
not a recommendation.

### The per-file rule: C

C is the per-file rule,
which this round's numbers put last:
the write step costs 62 to 63 microseconds a path at 6,000,
so moving an unchanged path from the clones to the write step loses at every size measured.

### The small-file half under A or D

If A or D is chosen,
nothing in this proposal ends the clean small-file half,
which is `main`'s own cost now;
the owner is asked what becomes of it:
a recorded permanent limit,
the accepted faster size reader,
or the size rule later (which turns A into B,
or lands E beside D).

### The order the figures support

The recommended order is A now,
then the rule once its threshold is measured,
which makes it B,
and the bound after its cut is measured,
which makes it F;
each order is reachable among the options in this section and none is bundled with another.
Whichever is chosen,
what lands is the prototype's design on the fenced rung,
with the `SPEC.md`,
`README.md`,
plan and source-comment changes quoted in "Where the design text changes",
and with the gate image's suite green.
