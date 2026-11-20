# The clone and snapshot rungs under Git's checkout conversions: a fence, its cost, and the options

**Unapproved and non-final.**
Nothing here is a project decision and nothing is declared complete;
the owner declares
completion.
This is a proposal with a prototype and measurements on this host,
under the load it had.

Issue:
GitHub issue 6323,
"git-policy-cli:
a cloned worktree keeps the source bytes where Git checkout would convert line endings or ident".
Prototype branch:
`docs/planning-cli-git-clone-rung-conversion-fence`,
made from ZFS plan step 3's integration tip
`74caa69e7a`.
Related records:
`doc/decision/cli-git-zfs-clone-rung-dirty-source-waiver.md` (the merge bar and its one waiver),
`doc/planning/cli-git-btrfs-worktree.md` (the ladder and its conversion rule,
the bullet "Both rungs require the new
worktree's effective conversion settings"),
`package/git-policy/cli/SPEC.md` (what rung 1 and rung 2 promise).

## What is wrong, in one paragraph

Both fast rungs carry the source checkout's working bytes:
rung 1 snapshots the whole directory,
rung 2 clones every
tracked file and installs the source's index.
Git's own checkout does not copy those bytes:
it writes the conversion of
each blob (a line-ending rule,
`core.autocrlf`,
`ident`,
a `working-tree-encoding`,
a filter's smudge).
Where such a
rule applies,
the source file need not be the conversion of its blob,
and nothing tells:
the clean filter maps both
forms to the same blob,
and the installed index carries the clone's own stat data,
so `git status` is clean.
The new
worktree then holds other bytes than `git worktree add` writes,
and every check the add runs stays green.

Measured before this proposal,
on clean and dirty sources,
with controls equal:

- ZFS,
  `74caa69e7a` against `main` at `2df9b1689b`,
  rung 2,
  clean sources:
  `*.txt text eol=crlf` over LF files,
  `core.autocrlf=true` set after the files were written,
  `notes.txt ident` over an unexpanded `$Id$`,
  and
  `* text=auto eol=lf` over a CRLF file that was then `git add`ed all differ from Git's checkout
  (`~/temp/agent/clone-rung-line-ending-reproduction-on-step-3-and-main-2026-11-20/keep/`).
- btrfs in a guest,
  `main` at `2df9b1689b`:
  rung 1 with a clean source,
  rung 1 with a dirty source,
  and rung 2 with a
  clean source all differ from Git's checkout in those same four cases,
  with the controls equal
  (`~/temp/agent/snapshot-and-clone-rungs-under-conversion-in-a-btrfs-guest-2026-11-20/keep/findings.md`).

The second measured set matters for the merge bar:
on btrfs `main` serves both rungs,
so this defect is older than step
3.
On ZFS `main` adds through Git's checkout and gets these bytes right,
so step 3 does not merge with it
(`doc/decision/cli-git-zfs-clone-rung-dirty-source-waiver.md`:
the waiver covers time costs only).

### Why the existing checks cannot see it

Git's own fast path is the stat:
`read-cache.c`,
`ie_modified` returns 0 as soon as `ie_match_stat` says the entry's
recorded size,
times,
inode and owner are the file's.
The wrapper's clean check makes the same comparison and only
reads a file whose stat data cannot vouch for it (`worktree_clean_tree_walk.rs:248`,
`matches_stored_exactly`).
The
content judge in `worktree_clean_content.rs` does refuse a path where a conversion could change the bytes,
but only
entries it reads reach it:
in every case above,
all entries are stat-exact,
so it reads none (the guest report's
"judge read 2 entries" counts only the genuinely dirty files).
`serve_clones` takes no `ConversionEnvironment` at all.

## What Git's checkout actually does, from its own source

These are the rules the fence implements,
read from Git 2.56.1's `convert.c` (the gate image's Git is 2.56;
the guest's
was 2.43 and behaved the same in the four cases).

- `convert_attrs` makes one action per path from the `text`,
  `crlf`,
  `eol`,
  `filter`,
  `ident` and
  `working-tree-encoding` attributes and then from `core.autocrlf` and `core.eol`:
  an attribute wins over the
  configuration,
  `eol` refines any action but the binary one,
  and `core.autocrlf` (default false) decides only where no
  attribute names one.
- `crlf_to_worktree` converts only when `output_eol(crlf_action)` is `EOL_CRLF`,
  that is for `text eol=crlf`,
  `text=auto eol=crlf` and `core.autocrlf=true`,
  and then only when `will_convert_lf_to_crlf` holds:
  the blob has a
  lone line feed,
  and for the `auto` actions it also has no carriage return and is not binary.
  A blob with no line feed
  at all is never converted,
  which the fence does not need:
  it gives those paths to Git as well.
- `ident_to_worktree` expands `$Id$` and `$Id: ...$` with the blob's object name.
- the encoding and the smudge filter convert whole bytes,
  in both directions.
- `crlf_to_git`,
  the clean side,
  only ever *removes* carriage returns,
  and only for a non-binary action.
  That is the
  one property the fence's cheap proof rests on:
  if the rule that wrote the blob can only remove carriage returns,
  then
  a file whose size equals its blob's size has had nothing removed,
  so the file is the blob.

## The fence: which paths a clone may carry

The fence decides one tracked path at a time,
in the source,
before the clone.
It may carry a path only when both
sides are proven:
the checkout's conversion of the blob is the blob,
and the source's file is the blob.

- the checkout's conversion is the blob when the action's output ending is not the carriage return,
  no `ident` and no
  converting `working-tree-encoding` applies,
  the `filter` attribute names no program,
  and the `text` value is one Git
  names;
- the source's file is the blob when no filter,
  `ident` or encoding applies and either the action is binary (no clean
  conversion at all) or the index entry's recorded size equals the blob's.

Everything else is a path Git writes.
The order of the checks is:
link,
`text` value Git does not name,
`filter`,
`working-tree-encoding`,
`ident`,
then the ending,
then the size.

### The classes, and what each answers

- **No rule at all** (`text` unset,
  `core.autocrlf` unset or false,
  no `eol`,
  no filter,
  no `ident`,
  no encoding):
  the
  checkout and the clean side are both the identity,
  so the clone carries the file with no further read.
  This is the
  common case and it costs nothing.
- **A text rule whose output ending is the line feed** (`text` with `core.eol` unset or `lf` on a Unix host,
  `text eol=lf`,
  `text=auto eol=lf`,
  `text=input`,
  `core.autocrlf=input`):
  the checkout writes the blob unchanged,
  and
  the clean side can only drop carriage returns,
  so the path is carried exactly when the recorded size equals the
  blob's size.
  This repository's own first attribute line,
  `* text=auto eol=lf`,
  is this class,
  so every one of its
  tracked files takes the size comparison.
- **A text rule whose output ending is the carriage return** (`eol=crlf`,
  `text eol=crlf`,
  `text=auto eol=crlf`,
  `core.autocrlf=true`,
  `text` with `core.eol=crlf`):
  Git's checkout may add a carriage return,
  so Git writes the path.
  With `core.autocrlf=true` and no attributes this is every text path,
  and the rung degrades to Git's checkout of the
  whole tree (measured below).
- **`text` unset or `!text`**:
  binary,
  nothing converts on either side,
  so the file is carried even under
  `core.autocrlf=true`.
  This is what makes the fence usable:
  a repository can exempt its binaries.
- **`ident`**:
  Git's checkout expands the keyword in the blob,
  so Git writes the path.
  A blob with no `$Id...$` in it
  would be carried safely,
  but the fence does not read blobs to find that out.
- **`working-tree-encoding`**:
  both sides convert,
  and the source's file is the working-tree form,
  which is what Git's
  checkout writes back;
  a stat-vouched file is the same bytes.
  The prototype nevertheless gives these paths to Git,
  because it cannot prove that the encode and decode pair is the inverse.
  The control that measures this class
  (`a_working_tree_encoding_carries_the_sources_bytes_as_gits_checkout_does`) passes on step 3 as well,
  so the fence's
  choice here costs time and buys nothing measured.
- **A `filter` attribute**:
  the rung already declines every filter but the standard Git LFS definition
  (`external_conversion_refusal`,
  `worktree_ladder_target.rs`),
  so the fence sees at most `filter=lfs`.
  The fence gives
  those paths to Git:
  the smudge writes content,
  and a missing `git-lfs` then fails the same way in both arms.
  The
  option of carrying them on the design's own argument (a filter that converts a file by its content alone) is below.
- **A link**:
  Git writes the target and converts nothing,
  so a link is carried.
- **A `text` value Git does not name** (`text=foo`) or a quoted attributes line:
  the fence reads Git's own answers
  (`git check-attr`),
  so a quoted line is not a modelling problem for it;
  an unknown value is given to Git.
- **A submodule,
  a split index,
  a skip-worktree entry**:
  refused by the clean check before the fence runs.

### The attribute sources, and what each means

- **Tracked `.gitattributes` files**:
  read by both worktrees.
  `prepare_target` already refuses a new `HEAD` whose
  attribute files differ from the source's (`attributes_agree`),
  so one answer stands for both.
- **`$GIT_DIR/info/attributes`,
  `core.attributesFile` and the default global file,
  and the system file**:
  shared by both
  worktrees,
  and `core.attributesFile` must be absolute (`external_conversion_refusal`).
- **An untracked `.gitattributes` anywhere above a tracked path,
  ignored or not**:
  the source alone has it,
  so Git read
  it when the files were added and the new worktree will not have it.
  The fence refuses the rung:
  it walks the
  ancestors of every tracked path in process,
  compares each `.gitattributes` against the index,
  and answers
  `<path> is in the source and not in its index, so the rules Git read when its tracked files were added cannot be
  known`.
  The clean check lists untracked files with `--exclude-standard` (`worktree_clean_untracked_scan.rs`),
  which
  hides an ignored one,
  so without this the gate review's finding L3 stays open.
  The in-process walk costs no Git run
  and no walk of the ignored directories.
- **`GIT_ATTR_SOURCE`,
  `--attr-source`,
  `attr.tree`**:
  the clean check already refuses `GIT_ATTR_SOURCE`;
  `attr.tree`
  must be refused for the same reason,
  because the destination would read attributes from a tree the fence did not
  read.
- **An attributes file that was removed after the files were added**:
  invisible to any fence,
  in the same way that a
  file rewritten with its own stat data restored is invisible to Git.
  Recorded as a limit,
  not fixed.

## How the fence is computed, and what it reuses

The prototype adds `worktree_clone_conversion_rules.rs` (Git's `convert_attrs`,
`output_eol` and
`will_convert_lf_to_crlf` as a pure decision,
with its own controls) and `worktree_clone_conversion_fence.rs` (the
reads and the one Git run that writes the rest),
and calls them from `serve_clones`:

1. `first_untracked_attributes`:
   the index entries' ancestor directories,
   in process,
   one `lstat` of `.gitattributes`
   each;
   a hit refuses the rung.
2. `git check-attr --all -z --stdin` over every tracked path,
   in the source:
   one run,
   every attribute source,
   macros
   expanded,
   precedence resolved by Git.
   This replaces the prototype's earlier idea of matching patterns again with
   `worktree_attributes.rs`'s parser,
   which the clean check uses where it must answer without a Git process.
3. `git ls-tree -r -l -z <HEAD>`:
   one run,
   only when some path's ending rule can strip carriage returns (class two
   above),
   for the blob sizes the size proof needs.
4. the decision per entry,
   from `stored_stats(source.clean.index_bytes, …)`:
   the entry's recorded size,
   its mode and
   the attributes.
5. `git checkout-index --force -z --stdin`,
   after the tree difference and only when the fence left paths to Git,
   in the
   destination with `GIT_DIR`/`GIT_WORK_TREE` as the ladder's other runs set them,
   so the destination's own conversion
   writes them.

What it reuses rather than reimplements:
`SnapshotSource` and its proven `CleanSource` (the index bytes,
the tree and
the hash length),
`stored_stats` and `StatEntry` from the clean check,
`metadata_verdict`'s stat reading where the
entry's own fields must be judged,
`run_git`/`GitRequest` (which records each run as a step of the add's timing
record),
`checkout_environment`,
`add_timing`'s steps,
and `attributes_agree`,
`conversion_agrees` and
`external_conversion_refusal` in `prepare_target`,
which already narrow what the fence must consider.

What it does *not* use,
and why:
`worktree_clean_content.rs`'s `conversion_is_the_identity` answers only for the four
tokens `filter`,
`ident`,
`working-tree-encoding` and a carriage return in the file (`CONVERTING_ATTRIBUTES`);
it has no
line-ending action,
so it cannot tell `text=auto eol=lf` (carry,
with the size proof) from `eol=crlf` (Git writes),
and
it reads file bytes to find a carriage return,
which a per-path fence over 24,000 files cannot afford.
Its
`ContentJudge` is still the right tool for the entries the clean check reads by content,
and the fence leaves those
paths to Git.

## What it costs

Measured on this host,
on a throwaway repository of this repository's 24,313 tracked files,
three arms per round in a
rotating order (the prototype,
the same prototype binary under another name as the control,
and step 3 at
`74caa69e7a`),
with `sync` and 300 ms between arms,
and one reference add per round outside the timed window made by
forcing Git's own checkout:

Measured in this round,
on this host,
with rule `QNB`'s spread taken first:
a throwaway repository of this
repository's 24,389 tracked files on the ZFS home dataset,
six arms per round in a rotating order (the prototype's
per-path fence,
the same binary with the option C switch,
the same binary with the option D switch,
the same binary
again as the control,
step 3 at `74caa69e7a`,
and `main` at `4997e2a4cb`),
`sync --file-system` and 300 ms between arms,
one warm-up round,
and every arm's worktree compared with `main`'s outside the timed window.
Binaries,
each tied to its
commit and kept under `~/temp/agent/clone-rung-conversion-fence-design-and-measurement-2026-11-20/keep/`:
prototype
`11fac3f625`,
step 3 `74caa69e7a`,
main `4997e2a4cb`.
Every round's JSON and step split is kept beside this record as
`measure-r4-*.json` and `measure-r4-*.summary.txt`.

Clean source,
six counted rounds (spread first:
six rounds of one arm on unchanged input,
median 4939 ms,
range
4549 to 15129 ms,
so the host's own noise reached three times the median before any difference was claimed):

- step 3:
  1019 ms (982 to 1085)
- the per-path fence,
  option A:
  1501 ms (1324 to 1581);
  its identical-binary control 1399 ms (1288 to 1603),
  a
  control-to-prototype ratio of 0.95 in the median (0.85 to 1.16 per round),
  so the fence's 1.42 times step 3 is
  outside the noise
- `main`'s Git checkout:
  2057 ms (1971 to 2753)
- option C,
  the whole-source refusal:
  2185 ms (2120 to 2289),
  which is `main`'s checkout,
  because that is what a
  declined add does
- option D,
  Git writes every path:
  4274 ms (4111 to 4737)

The fence's own steps in the clean arm's record:
`git check-attr` 41 ms,
`git ls-tree` 230 ms,
and no
`git update-index` or `git checkout-index` at all,
because this repository's `* text=auto eol=lf` leaves every path's
checkout the blob itself and every recorded size equals its blob's size.
The clone itself is 851 ms of the arm.

A source whose attributes convert,
four counted rounds with `*.md` and `*.ts` under `eol=crlf` added to the fixture's
own file:

- step 3:
  1012 ms,
  and its worktree differs from `main`'s by 7,106,268 lines of `diff`:
  the defect at this scale
- option A:
  2567 ms (2406 to 2622),
  equal to `main` on every path;
  control 2553 ms (2415 to 2602),
  a control-to-
  prototype ratio of 0.99,
  so 1.28 times `main` is measured and not noise
- option D:
  4170 ms (1.28 to 2.15 per round against `main`)

Dirty source,
four counted rounds (three modified tracked files,
two untracked ones):
every arm took `Git's checkout`,
the route of the clean check's refusal,
and every arm's worktree equalled `main`'s on every path.
The wall times
(prototype 2501 ms,
control 2349,
step 3 4261,
main 3327) are inside that mode's noise:
the control-to-prototype ratio
was 0.90 in the median and one round held a 19.9 s outlier,
so no difference between builds is claimed there.
The reason
is structural:
a dirty source never reaches either rung,
so the fence costs nothing on that path.

In the btrfs guest,
through the recipe of
`~/temp/agent/snapshot-and-clone-rungs-under-conversion-in-a-btrfs-guest-2026-11-20/keep/` (one `mvm` guest over
libvirt,
created and destroyed by this run's own calls,
two loop-backed btrfs filesystems,
the image's Git 2.43,
90
tracked files per repository):

- the same build with the fence:
  15 of 15 scenarios equal to Git's own checkout (both rungs,
  clean and dirty sources,
  the four conversion classes and the positive controls)
- the same build without it (`MONOCHROMATIC_CLONE_FENCE=off`):
  3 of 15 equal,
  the twelve others different:
  the four
  classes on rung 1 with a clean source,
  on rung 1 with a dirty source,
  and on rung 2 with a clean source
- cost,
  one add per arm per scenario:
  rung 1's median 61 ms without the fence and 78 ms with it (per-scenario deltas
  +2,
  +4,
  +13,
  +17,
  +16,
  +25,
  +3,
  -14,
  +32 and -8 ms),
  and rung 2's 41 ms and 58 ms. Single adds on a 90-file
  repository put these deltas at the size of the run-to-run spread,
  so they say the fence is cheap there and not how
  cheap.

What this says about the cost:
the fence is a third of `main`'s checkout on a clean source of this repository's size,
and it is the only arm that is right on a source whose attributes convert,
where step 3 is wrong on 24,389 files.
Its
price is the two Git runs over every tracked path (`git check-attr` and `git ls-tree`),
which is 271 ms of the 482 ms
it adds over step 3 on the clean arm.

## How the fence applies to rung 1, the snapshot

The snapshot rung carries the same bytes,
and the guest measured it wrong for a clean and a dirty source.
The fence
applies at one place,
in the stage and before the exchange,
and needs no new decision:
`serve_snapshot`
(`worktree_ladder_finish.rs:131`) runs `restore_dirty` or `apply_difference` on the stage and then swaps it in,
so a
call after those two steps and before `put_in_place` (`:180`) would compute the same fence over the stage's own
`.gitattributes` files and hand the paths to `git checkout-index --force --stdin` in the stage.
Three details differ
from rung 2:

- The stage's files are the source's,
  byte for byte,
  so the fence's clean-side reading is the same;
  but the stage is a
  copy made by the snapshot,
  not the source,
  so the fence's ancestor walk must look at the stage's directories,
  which
  hold the same tracked and ignored state.
- A dirty source's stage has had `restore_dirty` replace the files the index could not vouch for;
  those are already
  Git's bytes,
  and the fence may leave them out,
  or include them harmlessly.
- The exchange happens after the fence,
  so a fence that fails ends the rung exactly where `restore_dirty` failing ends
  it:
  the stage is destroyed and Git's own checkout completes the worktree.

This is written and measured in the guest:
with the fence,
rung 1 equalled Git's own checkout in the four conversion classes with a clean source and with a dirty
one,
and the same build without it was different in all eight.
The first guest run of this round established why it needs one:
its clean path runs no content judge at all,
and its dirty path's `restore_dirty` believes every stat-exact entry.

## The options, with their measured cost

Every option below is a way to keep the clone rung (or give it up) while never writing bytes Git would not.
Each cost
is measured on this round's fixture,
24,389 tracked files,
as the median of the counted rounds named above;
`main` is
2057 ms on the clean source and 2018 ms on the converting one.

**A.
The per-path fence before the clone,
the prototype.**
One `git check-attr` over every tracked path,
one
`git ls-tree` when a size can decide,
the decision in process,
then one `git update-index` and one `git checkout-index`
for the paths left to Git.
Costs:
1501 ms on the clean source (1.42 times step 3,
0.70 of `main`) and 2567 ms on the
converting one (1.28 times `main`),
and it is the only arm that was right on both.
Pros:
nothing is refused,
so a clean
source keeps its clone's disk saving;
the decision is Git's own rules,
and an unknown state is a path for Git rather
than a guess;
measured to be the cheapest arm that is right (`decline` is 2185 ms on the clean source,
`all` 4274 ms).
Cons:
two Git runs over every tracked path cost 271 ms of its 482 ms over step 3,
which grows with the path count;
and
its write step can only be as good as the entries it zeroes,
which is why the guest had to be run.

**B.
The same fence beside the clone.**
Not built in this round,
and the reason is measured:
the fence's own cost is
271 ms of Git runs plus 55 ms of decision work,
against a clone of 851 ms on this fixture,
so overlapping the two would
save at most the smaller of them and needs the clone,
the fence and the index install to be reordered around a
concurrency the add does not have today.
Its saving is therefore bounded by about 250 ms here,
less than the 482 ms A
adds over step 3,
and it would not change the ranking.
The option stays open for a repository whose clone is much longer
than its fence.

**C.
A whole-source refusal,
no per-path fence.**
Measured at 2185 ms on the clean source,
which is `main`'s checkout,
because a declined add is Git's checkout:
it gives up the clone rung for every source with a conversion setting or an
attributes file,
and this repository's first attribute line is one.
Pros:
no Git run,
0.3 ms of configuration reading,
nothing to model.
Cons:
1.07 times `main` and 1.46 times A on the clean source,
where step 3 is 0.49 of `main`:
the
cheapest correct answer only for a repository without conversion rules.

**D.
Git writes every tracked path after the clone.**
Measured at 4274 ms on the clean source (2.07 times `main`) and
4170 ms on the converting one:
the clone is wasted work and the checkout is not.
Pros:
no attribute model,
no untracked
-attribute rule.
Cons:
twice `main`,
so the rung keeps only its disk saving;
A is 2.8 times faster.

**E.
Keep step 3 as it is and accept the wrong bytes.**
Measured wrong:
7,106,268 lines of `diff` against `main` on the
converting source.
Rejected:
`main` on ZFS gets these bytes right,
so this fails the merge bar with no waiver that
covers it,
and `SPEC.md:1223` promises the snapshot's worktree is what Git's checkout makes.

Ranking:
**A best,
C next,
B next,
D next,
E last.**

- A over C:
  A is 1501 ms against C's 2185 ms on the clean source and the only one of the two that keeps the clone rung
  where a conversion rule exists;
  both are right.
  The 482 ms A adds over step 3 is the price of keeping the rung here.
- C over B:
  C is measured and needs no new concurrency;
  B's saving is bounded by about 250 ms,
  which would put it near
  C's cost on this fixture while adding a reordering of the add's steps,
  so it is worth building only for a repository
  whose clone is much longer than 851 ms.
- B over D:
  B keeps the clone's disk saving on a conversion-bearing source and D throws it away,
  at 4170 ms against at
  most 2325 ms for B on the converting source.
- D over E:
  D is at least right,
  and E is wrong on `main`'s own behaviour.

Two things must happen before A can be recommended to the project:
the fence's two Git runs have to be replaced by the
in-process reads the clean check already has (`worktree_attributes.rs` and the object database),
which removes the 271 ms
that grows with the path count,
and the guest's evidence has to be repeated with counted rounds rather than one add per
scenario.
Neither is done here.
## Where the design and `SPEC.md` change

### `package/git-policy/cli/SPEC.md`, rung 2

The specification says,
of rung 2 (`SPEC.md:2651` to `2656`):

> The same `--no-checkout` run is followed by `FICLONE` of every tracked file (mode and nanosecond times kept;
> links
> recreated),
> a second comparison of the source with its own index,
> the installation of the index,
> and the same tree
> difference.

and the clean check's content judge says of itself (`worktree_clean_content.rs`,
the module doc):

> The hash of the raw bytes is conclusive only when Git's own conversion on the way into the index (`filter`,
> `ident`,
> `working-tree-encoding`,
> or a line-ending conversion that strips a carriage return) leaves the bytes alone,
> so a path
> such an attribute applies to,
> and a file with a carriage return,
> are refused with that reason instead of guessed.

New text,
after the sentence about the clean-source requirement:

> The clone rung carries a tracked path only where Git's own checkout of that path's blob is the blob itself,
> and the
> source's file is proven to hold that blob.
> The proof is Git's own conversion rules,
> read from the path's attributes,
> `core.autocrlf` and `core.eol`:
> a path whose checkout could add a carriage return,
> expand an `ident` keyword,
> convert
> a `working-tree-encoding` or run a filter's smudge is written by one `git checkout-index --force` in the new worktree;
> a path whose rule can only drop carriage returns on the way into the index is carried when its recorded size equals
> the size of its blob;
> every other path is carried as it is.
> A source with an untracked or ignored `.gitattributes`
> anywhere above a tracked path refuses the rung,
> because Git read it when the files were added and the new worktree
> will not have it.

### `package/git-policy/cli/SPEC.md`, rung 1

The promise at `SPEC.md:1223` stands as written ("the new worktree is restored to what `git worktree add` makes from a
clean checkout");
what changes is that it now holds for conversions too.
New text,
after the restore's steps:

> Then,
> in the stage and before it is swapped in,
> the snapshot rung runs the same conversion fence as the clone rung,
> over the stage's own files and the new `HEAD`'s blobs,
> and the paths it leaves to Git are written by one
> `git checkout-index --force` in the stage.
> A fence that cannot read its inputs ends the rung and Git's own checkout
> completes the worktree.

### `doc/planning/cli-git-btrfs-worktree.md`, the conversion bullet (about lines 1187 to 1197)

The bullet says:

> Both rungs require the new worktree's effective conversion settings (`filter.*`,
> `extensions.*`,
> `core.autocrlf` and
> its relatives),
> read in the new worktree's own context,
> to be the same sequence as the source's,
> in the same order.
> A
> filter selected by an attribute (tracked,
> `info/attributes` or a global file) declines the fast rungs,
> except the
> standard Git LFS definition;
> …

New text for the end of that bullet:

> Equal settings are necessary and not sufficient:
> the source's files are the source's bytes,
> not the conversion of
> their blobs,
> so both rungs then run the conversion fence described in
> `doc/planning/cli-git-clone-rung-conversion-fence.md` and let Git write the paths it names.
> A source with an
> untracked or ignored `.gitattributes` above a tracked path declines both rungs,
> because Git read it when the files
> were added and the new worktree will not have it.

### `doc/decision/cli-git-zfs-clone-rung-dirty-source-waiver.md`

No change.
Its waiver covers time costs,
and the fence is a time cost;
the wrong bytes are a merge bar failure it does
not cover,
which is why this proposal exists.

## What this round closed, and what went wrong in it

Closed:
the write step,
twice.
`git checkout-index --force` in place writes a path only when its entry's stat data no
longer matches the file (`entry.c`,
`checkout_entry_ca`,
`if (!changed) return 0;`),
and neither rung can promise that:
the clone rung installs the source's index over a copy,
and a btrfs snapshot keeps the source's inode and times,
so the
copied file *is* the file the source's index recorded (measured in the guest:
the write ran and the stage kept the
source's bytes).
`write_through_git` now zeroes exactly the fenced entries with one `git update-index -z --index-info`
and then lets Git write them,
which is the state `read-tree` leaves an entry in.
The guest's fifteen scenarios then came
out equal,
and the host's eight class controls with them.

Also closed:
the size proof.
`git ls-tree -r -l` right-aligns its size column,
so splitting it on single spaces found no
size at all and gave every path of the line-ending class to Git:
a clean add of this repository spent 3012 ms of its
4668 ms in `git checkout-index` rewriting a tree whose bytes were already right,
and the first clean measurement of this
round measured that.
The split is on runs of spaces now,
and the clean arm's fence steps are `git check-attr` 41 ms and
`git ls-tree` 230 ms with no write at all.

What went wrong and is recorded so the numbers are not read for more than they say:

- The first clean measurement of this round (`measure-r3-*`) was taken with that defect,
  and its prototype arm (4668 ms)
  is reported here only as the defect's cost,
  never as option A's.
- The gate image's full run at `39e12fdbc3`-era code failed three tests:
  the standing notice's
  `every_step_an_add_enters_has_its_own_sentence`,
  which the two new steps caused and which `6a87d8bafe` fixes,
  and two
  commit and service tests (`repository_naming_commit::...`,
  `service_no_manager::...`) whose failures name the host,
  not this change.
  All eighteen unit controls of the two new modules passed there.
  The full gate was **not** re-run at
  the final commit:
  its vendored dependency snapshot lives in the gate image,
  and running nextest in the image against
  the worktree mounted as it is cannot resolve crates offline.
- The guest's cost numbers are one add per arm per scenario,
  so they are a size,
  not a median.

## What this proposal does not do

- It does not measure option B:
  it is not built,
  and the reason and the bound are in the options.
- It does not run the class controls in the gate image.
  The controls are gated on a clone-capable filesystem (`ZFS` or
  `btrfs`) and the gate image's fixture root is a tmpfs,
  so there they print "CONTROL NOT RUN";
  the host has ZFS but no
  `cargo nextest`.
  They ran on the host through this round's own harness,
  against the built binaries,
  and in the guest.
- It does not repeat the guest's adds:
  one per arm per scenario.
- It does not make the fence cheaper.
  Replacing the two Git runs with the in-process attribute and object reads the
  clean check already has is the next step,
  and the 271 ms they cost here is why.
- It does not measure XFS or overlayfs,
  the other two filesystems `main` serves with rung 2.
  The decision does not read
  the filesystem.
- It does not touch `doc/handover/`.
