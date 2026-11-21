# The clone and snapshot rungs under Git's checkout conversions: a fence, its cost, and the options

**Unapproved and non-final.**
Nothing here is declared complete;
the owner declares
completion.
This is a proposal with a prototype and measurements on this host,
under the load it had.
**The project has chosen option B as the remedy**
(2026-11-20T23:27:00-05:00,
"Yeah,
do B.",
`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
step 10's notes),
so the shape this proposal ships is B,
the options section is the record of what it was chosen over,
and the "Limitations" section lists what B still is not right about.

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

## The trade, first

On this repository as it is,
step 3 without the fence was **correct there**:
every arm's worktree was clean and had `diffVsMain=0` against `main`'s checkout,
and step 3 took a median **0.59 of `main`** (1,356 against 2,286 ms).
The fence takes that add to a median **1.11 of `main`** (2,395 ms,
its identical-binary control 1.03).
That is the trade the project weighs:
the clone rung's disk saving at 0.59,
against the fence's correctness at 1.11,
and the second is the price of the bytes being right where a conversion rule applies.
Those four numbers are the last round's kept run
(`~/temp/agent/clone-rung-conversion-fence-gate-review-findings-fixed-2026-11-20/scratch/keep/`,
`measure-r5b-real-*.summary.txt` and `.json`).
This round re-measured the same shape with the current code (`measure-recheck2-real-*`):
step 3 0.49 of `main` (1,028 against 2,119 ms) and the fence 0.98 (control 0.90).
The two runs differ by more than the fence-to-`main` difference in either,
and both keep every arm's worktree equal to `main`'s,
so what the project weighs is step 3's **0.49 to 0.59** (resolved in both) against the fence's **0.98 to 1.11**
(not resolved more tightly than that;
see "This repository as it is" below).
Those numbers are option A′'s,
the fence decided before the clone.
The last round built option B,
which reads what the decline needs before the copies and the rest beside them,
and this round moved the decline ahead of that,
so that a declined add pays no copy:
on the same three fixtures option B is 0.96 of `main` on this repository
and 1.01 to 1.04 on the 0.199 source,
both inside the identical pair's disagreement,
and 1.10 on the 0.644 source,
just past its pair's 0.95 to 1.05.
The numbers,
and each option's cost above `main`,
are in this round's runs section and in the options section below.

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

- `convert_attrs` makes one action per path from the attributes and then from the configuration,
  in one order that the fence now follows exactly (`convert.c` at 2.56.1,
  `convert_attrs` and `git_path_check_crlf`):
  `text` is read first (`git_path_check_crlf`),
  and only while it names no action does the legacy `crlf` attribute decide,
  read by the same function;
  then `eol` refines any action but the binary one,
  **including an action nothing has named yet**,
  so a bare `eol=lf` makes the clean side strip carriage returns (`CRLF_TEXT_INPUT`)
  and a bare `eol=crlf` makes the checkout write them (`CRLF_TEXT_CRLF`);
  `text` alone becomes `CRLF_TEXT`,
  which `text_eol_is_crlf` turns into the input or the carriage-return ending from `core.autocrlf` first
  (true is the carriage return,
  `input` is the line feed)
  and only from `core.eol` when `core.autocrlf` is false or unset on a Unix host;
  and only an action still unnamed falls to `core.autocrlf`
  (false is binary,
  `input` is `CRLF_AUTO_INPUT`,
  true is `CRLF_AUTO_CRLF`).
  The prototype's first version read `text` and `eol` only and had no arm for `crlf`,
  which the gate review caught (`m01` to `m03b`).
- `crlf_to_worktree` converts only when `output_eol(crlf_action)` is `EOL_CRLF`,
  that is for `text eol=crlf`,
  `crlf eol=crlf`,
  `eol=crlf` with nothing else,
  `text=auto eol=crlf`,
  `text` with `core.eol=crlf`,
  and `core.autocrlf=true`,
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
  where the checkout writes the blob unchanged,
  a rule can only have shrunk a file it touched on the way in,
  so a file whose size equals its blob's size has had nothing removed,
  and a path with no rule at all is the same case with no rule to shrink it.
  Size equality is necessary in both and sufficient in both:
  the proof is not "no conversion happened" but "nothing can have changed the length".

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
  conversion at all)
  or the index entry's recorded size equals the blob's,
  **including the path where no rule applies at all**:
  a file with no rule today can still hold a form an earlier rule wrote,
  and its size is the only thing that says so
  (the gate review's `m04` and `m06`),
  so the blob size is read for every path the fence may carry,
  not only for the line-feed class.

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
  no `crlf`,
  `core.autocrlf` unset or false,
  no `eol`,
  no filter,
  no `ident`,
  no encoding):
  the
  checkout and the clean side are both the identity,
  so the clone carries the file when its recorded size equals its blob's.
  The size read is the fence's one `git ls-tree`,
  which a repository whose paths are all in the line-feed class pays anyway,
  and which a repository with no attributes at all pays for the first time here:
  measured at 200 ms at this repository's 24,389 paths.
  The prototype's first version carried this class after reading only the attributes,
  and the gate review found two ways a file then differs from its blob (`m04`,
  `m06`).
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
- **A text rule whose output ending is the carriage return** (`eol=crlf` alone,
  `text eol=crlf`,
  `text=auto eol=crlf`,
  `crlf` with `core.eol=crlf`,
  `core.autocrlf=true`,
  `text` with `core.eol=crlf`):
  Git's checkout may add a carriage return,
  so Git writes the path.
  With `core.autocrlf=true` and no attributes this is every text path,
  and the fence's decline rule then ends the rung before it clones anything,
  because these paths alone are more than a quarter of the tracked ones
  (measured below).
- **A printed `unset`** (`-text`,
  or the literal value `text=unset`):
  Git reads the two differently,
  and `git check-attr` prints both as `unset`,
  so the fence takes Git's undefined reading,
  which is exact for `text=unset` and only more careful for `-text`:
  `-text` is binary and nothing converts on either side;
  the value `unset` is a string `git_path_check_crlf` does not know,
  so `convert_attrs` gives it to `eol`
  and then to `core.autocrlf`.
  Under `core.autocrlf=true` the token is therefore written by Git,
  and it counts toward the decline share there.
  Only without a converting configuration does it still take the size proof,
  so a repository exempts its binaries with `-text` at the price of that one read and no wrong bytes
  **where nothing converts**;
  under `core.autocrlf=true` a `-text` path is written by Git like any other
  (the diff check's `c07` shows the add writing it through Git and the bytes equal),
  and what that costs at a repository's own share of binaries is not priced.
  `!text` is a different spelling:
  it leaves the attribute unspecified (`ATTR_UNSET`),
  so `eol` and
  `core.autocrlf` decide exactly as they do for a path with no `text` line at all.
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
  The fence gives those paths to Git,
  and this is this repository's own second-largest class after the line-feed one:
  `git ls-files` counts 1,578 of its 24,389 tracked files under the eight `filter=lfs` patterns,
  and the configuration holds the standard definition
  (`filter.lfs.clean`,
  `smudge`,
  `process`,
  `required`),
  which the rung accepts.
  What proves the worktree's file is the smudge of its pointer is that Git wrote it:
  one `git checkout-index` run in the destination,
  with the destination's own filter settings,
  and the entry zeroed first so Git does not skip it.
  Carrying them instead would need the smudge's output to compare with the file,
  which costs the same filter run and a read of both;
  no cheaper complete proof exists,
  because a pointer and its content are different lengths and a content file has no marker.
  A cheaper partial one does:
  the pointer carries the content's `size` and `oid sha256:`,
  so a stat-exact file whose size is the pointer's size is very likely its content,
  but "very likely" is not a proof of bytes and this proposal does not ship it (the suggestion is weighed in its own
  section).
  The cost of writing them is measured below,
  and a missing `git-lfs` then fails the same way in both arms.
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
  `attr.tree` is **not** refused by the prototype,
  and this proposal does not claim it is (the gate review's L3):
  the one refusal of a fixed attribute source is `load_fixed` in
  `worktree_clean_content_attributes.rs`,
  reached only from the content judge's per-path read,
  which a stat-exact source never reaches.
  `git check-attr` does honour `attr.tree`,
  so both the fence and Git's own checkout in the destination read the same tree,
  and the answer is consistent;
  the gap is that nothing refuses the setting,
  and the fence's refusal of an untracked `.gitattributes` assumes the source's worktree is the attribute source.
  The proposal's new text below therefore requires the refusal in `prepare_target`,
  where the other external conversion settings are refused.
- **An attributes file that was removed after the files were added**:
  the fence cannot know the rule that wrote the blobs,
  and it does not have to:
  such a file leaves a path with no rule or with a different one,
  and every path whose checkout writes the blob unchanged takes the size proof,
  so a CRLF file with an LF blob is written by Git
  (the gate review's `m06`,
  and its `m04` for a configuration rule that was unset after the add).
  Recorded as the reason the size read is unconditional,
  not as a limit.

## How the fence is computed, and what it reuses

The fence is `worktree_clone_conversion_rules.rs` (Git's `convert_attrs`,
`output_eol` and
`will_convert_lf_to_crlf` as a pure decision,
with its own controls),
`worktree_clone_conversion_fence.rs` (the reads and the two write steps)
and,
for the shipped order,
`worktree_clone_conversion_beside_the_clone.rs` (the reads as a job beside the copies and the seed the index is
installed from).
Its steps are the same in both orders this round built:

1. `first_untracked_attributes`:
   the index entries' ancestor directories,
   in process,
   one `lstat` of `.gitattributes`
   each;
   a hit refuses the rung.
   It costs no Git run and no copy,
   so both orders take it in `choose_rung`,
   before the add's record is claimed.
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
3. the decline rule,
   from the attributes alone and before any blob is read:
   the number of paths whose own rule converts them (`checkout_converts`)
   is compared with a quarter of the tracked paths;
   past that share and past a thousand paths,
   the rung ends with the count and Git's own single checkout writes the worktree.
4. `git ls-tree -r -l -z <HEAD>`:
   one run,
   for the blob sizes the size proof needs,
   which is every path the fence may carry (see the no-rule class above).
5. the decision per entry,
   from `stored_stats(source.clean.index_bytes, …)`:
   the entry's recorded size,
   its mode and
   the attributes,
   and the paths Git must write.
6. the write step,
   after the tree difference and only when the fence left paths to Git:
   one `git checkout-index --force -u -z --stdin`,
   in the destination with `GIT_DIR`/`GIT_WORK_TREE` as the ladder's other runs set them,
   so the destination's own conversion writes them and their stat data is recorded (`-u`),
   and only for the paths the destination's index holds,
   because one it lacks would fail the run ("is not in the cache").

The two orders differ in when steps 2 to 5 run and what the index holds when step 6 runs:

- **A′,
  the fence before the clone** (this branch's earlier shape,
  kept behind the off-by-default
  `measurement-routes` feature because it is the arm the report times):
  steps 2 to 5 run in `choose_rung`,
  before the add's record is claimed and before real Git runs;
  nothing is cloned when step 3 declines.
  The clone then copies every tracked file and installs the source's index with the stat fields of every copy;
  step 6 first zeroes the written paths' entries with one `git update-index -z --index-info`
  (the intersection with the index is also why that run sends only paths the index holds),
  then asks for them.
- **B,
  the fence beside the clone** (shipped;
  the order and its safety argument are the next section):
  steps 2 to 5 run concurrently with the copies,
  and the index is installed only after both have answered,
  from a seed that holds no stat data for the written paths,
  so step 6 needs no zeroing run of its own.
  A decline in step 3 then happens after the clone,
  which is the cost that section states.

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
`add_timing`'s steps and its cross-thread `begin_work`/`end_work`,
`subvolume_index::rewrite_index_stat_with` and the index install the rung already used,
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

## Option B: the order beside the clone, and why each step is safe

The previous round built option B,
which the project asked for first
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
step 10's notes,
the project's 2026-11-20T22:37:00-05:00 answer),
and this round changed one thing about it:
the decline is decided before the first copy.
In the previous round the decline arrived after the copies,
so a declined add paid the copy and then Git's whole checkout over it
(measured on the 0.644 source at 1.67 to 1.75 of `main`,
where the fence decided before the copy measured 1.06 to 1.09).
The decline needs nothing but the attributes,
so it costs one `git check-attr` and never a copied byte;
only what remains (`git ls-tree` and the per-path decision) runs beside the copies.
The order below is what keeps both of those honest.

### The order

Before step 1 the ladder chooses the rung:
the cheap refusals of the source,
and the empty subvolume the rung prepares at the destination when it can.
Step 1 runs **before** real Git's `git worktree add --no-checkout`;
that run is the seam between step 1 and step 2,
and it is unchanged:
Git validates the branch and the registration and copies the configuration,
and it checks nothing out.
Steps 2 to 9 run after it returned.
The one check that reads the new `HEAD`,
`prepare_target`
(the destination still holds only Git's `.git` file,
the new `HEAD` is a plain commit,
`core.autocrlf` and its relatives agree with the source's,
the tracked attribute files of the two trees agree),
runs at the head of step 2,
before the copies.

1. **The fence's attributes and its decline are decided first,
   before the add's record is claimed and before anything is copied**
   (`the_fence_of_this_build`,
   `fence_the_attributes`):
   the untracked-attributes refusal,
   one `git check-attr --all -z --stdin` over every tracked path,
   the count of the paths a rule of their own would convert,
   and the quarter rule.
   The count is read from the attributes alone,
   before any blob and before any copy,
   so a source above the quarter ends the rung here:
   a declined add is Git's own checkout plus that one query,
   and it pays no copy.
   The rung returns to Git's checkout with the fence's own words in the person's sentence
   (`explain_decline`).
   For rung 1 this step decides the whole fence
   (`fence_the_clone`),
   because the snapshot installs its index itself:
   see the section "How the fence applies to rung 1".
2. **The copies start as one task of the process's runtime,
   and the rest of the fence runs on the add's own thread**
   (`clone_tracked_bytes_beside`).
   The task awaits the slices' jobs of the blocking pool;
   the fence's Git runs cannot run inside the runtime at all,
   because the synchronous face enters it around the pipes it reads and is refused from within it
   (`RuntimeError::Nested`),
   and a blocking-pool thread carries the runtime's handle.
   So the add's own thread,
   which is otherwise waiting,
   runs what the attributes left:
   one `git ls-tree` for the blob sizes,
   and the per-path decision.
   It reads the frozen commit,
   and nothing else.
   A repository below the clone's break-even has one thread for the copies,
   so that job runs before them there instead of beside them.
3. **The clones copy every tracked file**
   (`clone_tracked_bytes_beside`):
   each source file is opened below a held handle of the source root,
   its descriptor's fields are compared with the frozen index entry before and after the copy,
   and the destination entry is created below a held handle of the destination.
   No index exists yet,
   so nothing can vouch for a copied path.
4. **The join is taken whatever each side answered**
   (`run_on_runtime` awaits the copies' task once the fence has returned),
   so no task or job is left reading the source while Git's own checkout writes the destination,
   and a task that ended without an answer is the rung's reason.
   The copies' jobs take their places in the process's one bound on blocking jobs,
   so the fence adds no thread to the fan-out and the slices keep their own.
5. **A fence that ends the rung ends it here**,
   before any index exists:
   the copies stay (Git's checkout overwrites the tracked ones),
   the reason is logged and,
   for a failure of the fence's remaining reads and every other clone-rung ending,
   told to the person in the clone rung's words
   (`explain_clone_failure`;
   a decline or a refusal decided in step 1 never reaches this step,
   and is told as `explain_decline`).
6. **The seed is built in memory**
   (`seed_without_stat`):
   the proven source index bytes with the stat data of every written path zeroed,
   and the checksum made good.
   The entries keep their object names and modes,
   so Git writes each path from its own blob;
   no stat field of a written path survives,
   and the install's own rewrite skips an entry whose recorded size is zero,
   so the published index cannot vouch for one.
7. **The index is installed**
   (`install_the_index`):
   one `index.lock` created exclusively,
   the absence of an index checked under it,
   the finished index written into the lock and linked under its name,
   the lock released only if it still holds that file.
   The carried paths get the stat fields the copies recorded;
   the written paths keep their zeroes.
8. **The tree difference runs** (`git read-tree -m -u`) when the two trees differ,
   as it did before:
   Git writes the paths that differ and updates the index.
9. **The written paths go through Git**
   (`checkout_index_through_git`):
   the fence's answer intersected with the index as step 8 left it,
   one `git checkout-index --force -u -z --stdin`,
   which applies the destination's own conversion and records each written path's stat data.

### Why each step is safe if the other side fails or is killed

A killed process at any instruction is normal here,
so each step is judged by what it leaves behind,
not by what it would have cleaned up.

- **Step 1 writes nothing and claims nothing.**
  Its reads are the source's untracked-attributes walk,
  one `git check-attr` and the count,
  all over files the source's own index describes;
  the add's record is claimed in step 2 only after it answered,
  so a process killed in step 1 leaves the add exactly as Git's own `worktree add` left it,
  with no record that would make the next command remove anything.
  A decline or a refusal here is not a failure:
  it ends the rung before Git's run,
  and Git's own checkout writes the worktree.
- **Steps 2 and 3 cannot disturb each other.**
  The remaining fence runs Git reads over the frozen commit and the attributes it already read;
  the clones only write below the destination and only read below a held source handle.
  Neither takes a lock the other wants,
  so no interleaving of them changes an answer:
  a fence killed leaves the copies as they were,
  and a clone killed leaves the fence's reads to finish or fail on their own.
  The fence's remaining work records its duration as work (`fenceBesideTheClone`),
  because a timing step belongs to the thread that runs it and this one runs beside the copies' task.
- **A kill before step 7 leaves no index at all.**
  The destination holds copies of tracked files and Git's `.git` file;
  no entry vouches for any path,
  so Git's own checkout (`git reset --hard`,
  the rung's fallback) writes every tracked path,
  and the add's claim,
  taken at the head of step 2,
  is what the next command uses to remove or finish the add.
- **A kill inside step 7 leaves `index.lock` and no index** (the index is linked under its name only once it is
  complete),
  which is the state above.
- **A kill between steps 7 and 9 leaves an index that cannot vouch for the written paths.**
  Their entries hold a zero size,
  zero times and a zero inode,
  which no real file has,
  so a later `git status` reads those files and reports them as changed instead of trusting them;
  the carried paths are recorded clean only where the fence proved their bytes,
  and the fallback rewrites the rest from `HEAD`.
- **A kill inside step 9 leaves each written path either Git's own bytes with the stat data `-u` recorded,
  or its zeroes from step 6**:
  no path is ever recorded clean while its bytes are not Git's checkout's,
  which is the wrong ordering this option exists to avoid
  (the red-first controls named under "The round that built option B" and
  "The round that decided the decline before the copying" show it).
- **A failure of the clone after the fence answered** is the clone's reason,
  with the entries it made counted,
  and the fence's answer is dropped:
  no index is installed,
  and Git's checkout writes the worktree.
- **A failure of step 8** (the tree difference) ends the rung after the index is published:
  the carried paths' entries match their files,
  the written paths' entries do not,
  and the fallback's `reset --hard` writes the whole worktree.
  One case is new and measured:
  `read-tree -m -u` verifies the entries of the paths it is about to change,
  and a written path whose clean form is not its blob
  (a rule that changed or was removed after the add) makes it refuse where option A′'s fresh stat data let it pass.
  The harness's `m05` (an add to a commit whose tree lacks a fenced path) shows it:
  `error: Entry 'extra.txt' not uptodate. Cannot merge.`,
  the rung ends,
  and the bytes are still equal to Git's checkout because the fallback writes them.
  That failure is safe but costs the clone,
  and it is option B's one known roteness regression against A′:
  an add to a differing tree whose written path's clean form is not its blob ends the clone rung.
  The three measured fixtures check out one commit in both trees,
  so no timed arm reaches it;
  the fix (run the tree difference against a copy of the index that still has its stat data,
  then install the zeroed
  seed from that result) is recorded as open work,
  not shipped.
  Rung 1 has the same exposure for the same reason,
  and no fixture reaches it there either.
- **A failure of step 9** is a Git failure with Git's own words,
  and it ends the rung the same way.

### What the decline now costs

The rule's answer is the same and its price changed with this round's order.
Decided in step 1,
before the add's record and before any copy,
a decline pays one `git check-attr` (35.5 ms on this repository,
47.5 on the 0.199 source)
and then Git's own checkout,
which is what option A′ paid.
The previous round's order,
which decided the same rule after the copies,
paid the copy as well:
on the 0.644 source that was 1.67 to 1.75 of `main`,
against A′'s 1.06 to 1.09.
This round's runs measure the decline in step 1 against A′ and `main`
("What it costs",
"Option B beside the clone,
this round's runs").
The rule still means what the threshold section says it means:
it keeps the clone rung for sources whose converting share is small,
and past a quarter it makes Git's own checkout write the worktree,
because writing the paths one at a time inside the rung costs more than one checkout.

## What it costs

Three runs of this round,
each on a throwaway repository of this repository's tracked files built from its working tree,
**with its real attribute set:
all eight `filter=lfs` lines kept**,
their 1,578 files added through the standard Git LFS filter
(`filter.lfs.clean`,
`smudge`,
`process`,
`required`,
written where `git lfs install --local` writes them,
which is the definition the rung accepts).
24,426 tracked files,
14,165 of them `.md` or `.ts`.
The runs differ in the attribute lines alone:
"as it is" appends nothing,
the 0.199 source appends `*.md text eol=crlf`,
and the converting one appends `*.md text eol=crlf` and `*.ts text eol=crlf`;
none sets `core.autocrlf`
(the earlier round's converting fixture did,
behind an attribute line that overrode it for every path;
the gate review's L4).

Rule `QNB`'s spread came first in each run:
one arm alone on unchanged input,
before any arm was compared with another.
Every timing arm ran in its own capped `systemd-run --user --scope`
(`CPUQuota=200%`,
`MemoryMax=4G`,
`TasksMax=400`,
`IOWeight=1`,
the arm's environment through `env --ignore-environment`),
on this host as other sessions left it:
which is why every per-round ratio is printed beside the identical-binary control's own.
Arms rotated one position per round,
`sync --file-system` and 300 ms between arms,
one warm-up round,
and every arm's worktree compared with `main`'s **after** the counted rounds,
outside the timed window,
because the LFS content makes each comparison a read of the whole fixture.

Binaries,
each tied to its commit:
the fence at `527b890d3e` (sha256 `b7ddcb21…`,
the shipped build) and its measurement build with the same code plus `measurement-routes` (sha256 `96587a9e…`)
are in the previous round's
`~/temp/agent/clone-rung-conversion-fence-gate-review-findings-fixed-2026-11-20/scratch/keep/`
(the files `fence-h1h2-fixed` and `fence-h1h2-fixed-measurement`);
step 3 at `74caa69e7a` (sha256 `8077e6f9…`) and `main` at `4997e2a4cb` (sha256 `20a3e00b…`)
are in the earlier round's `~/temp/agent/clone-rung-conversion-fence-design-and-measurement-2026-11-20/keep/binaries/`
(`step3`,
`main`,
with `sha256.txt` and `commits.txt` beside them).
This round's fence binaries are in this round's
`~/temp/agent/clone-rung-conversion-fence-recheck-findings-fixed-2026-11-20/scratch/keep/binaries/`:
`fence` at `ef0f6c91f7` (sha256 `e1505e76…`) and `fence-measurement`,
the same code plus `measurement-routes`
(sha256 `771d167d…`),
with `sha256.txt` and `commits.txt` beside them;
every round's JSON and step split are in that `keep/` (`measure-recheck2-*`).

### This repository as it is, conversion rules and LFS content together

This round's kept run (`measure-recheck2-real-*`):
24,426 tracked files,
the repository's own attributes with all eight `filter=lfs` lines,
1,578 files added through the standard LFS filter,
14,165 `.md` or `.ts`,
1,578 of 24,426 paths converting (0.065,
the LFS count),
one warm-up and five counted rounds,
every arm in its own capped `systemd-run --user --scope`
(`CPUQuota=200%`,
`MemoryMax=4G`,
`TasksMax=400`,
`IOWeight=1`),
arms rotating one position per round,
`sync --file-system` and 300 ms between adds,
and every worktree compared with `main`'s after the rounds.

- spread,
  one arm on unchanged input:
  3 rounds,
  median 2,070 ms,
  range 1,876 to 6,598
- the fence (A′):
  **2,021 ms** (1,878 to 2,183),
  **0.98 of `main`** per round [1.02 0.98 0.89 0.94 1.01]
- its identical-binary control:
  1,936 ms (1,893 to 1,992),
  0.90 of `main`;
  control over fence per round 0.88,
  0.93,
  1.06,
  0.95,
  0.96,
  **median 0.95**
- step 3:
  **1,028 ms** (1,019 to 1,120),
  **0.49 of `main`** [0.48 0.48 0.49 0.53 0.51]
- `main`:
  2,119 ms (1,992 to 2,155)
- every arm:
  `git status` clean and `diffVsMain=0`.

The fence's steps (medians):
`git check-attr` 36.9 ms,
`git ls-tree` 199.4,
`git update-index` 14.7,
`git checkout-index` **551.7** for 1,578 LFS paths (0.35 ms each;
the control's,
on identical work,
561.0),
the clone 894.2,
the clean check 75.8,
the fence's own decision 31.5.

What this says:
on this repository as it is,
the fence and its identical-binary control are not separably different from `main` in this run
(fence 0.98,
control 0.90,
and the control's own per-round ratio to `main` spans 0.89 to 0.97).
The previous round's kept run measured the same shape at fence 1.11 and control 1.03
(`measure-r5b-real-*`),
so the fence's cost here is between about 0.98 and 1.11 of `main` across two kept runs and is not resolved
more tightly than that;
step 3's saving **is** resolved in both (0.49 and 0.59).

### A source where a fifth of its paths convert

The review's source (`measure-recheck2-md-*`):
this repository's 24,426 tracked files with one appended line `*.md text eol=crlf`,
so 3,288 `.md` plus 1,578 LFS paths,
4,866 of 24,426 (0.199) written by Git,
below the quarter,
so the decline rule does **not** decline;
one warm-up and seven counted rounds,
five arms.

- path (A′):
  2,477 ms (2,241 to 6,289),
  **1.13 of `main`** [1.07 2.65 0.85 1.13 1.13 1.19 0.98]
- control:
  2,502 ms (2,299 to 5,908),
  1.13
- option C:
  2,292 ms (2,037 to 2,376),
  **0.98**
- option A:
  2,703 ms (2,402 to 7,815),
  **1.18** [1.05 1.17 2.95 1.25 1.20 1.18 1.17]
- `main`:
  2,287 ms (2,102 to 2,650)
- control over path per round:
  2.16,
  0.94,
  1.15,
  0.93,
  1.01,
  0.96,
  1.08,
  **median 1.01**
- every arm:
  clean,
  `diffVsMain=0`.

The write step:
`git checkout-index` **930.4 ms** for 4,866 paths (A′;
control 999.8;
A 1,066.1),
and the same step costs 551.7 ms for 1,578 LFS paths alone in the "as it is" run,
so the two classes cost about **0.35 ms per LFS path** and **0.12 ms per text path** in this round's runs.
The previous round's two arms priced the same classes at 0.41 to 0.57 and 0.06 to 0.08 ms (its two medians of identical
work),
and the diff check's own run of the "as it is" shape priced the LFS class at 0.31;
all are kept measurements of the same classes under different host states,
and none is a constant.

What this says:
at 0.199,
below the quarter,
the fence costs **1.13 of `main`** and its identical-binary control also 1.13,
so this run does not separate the fence from `main` there either,
and C costs 0.98 with no control arm of its own in this run.
A′ against C is **not separable** from that spread:
A′ is slower in five of the seven rounds,
by a median of 0.09 of `main`,
which is the size of the identical pair's own disagreement in those same rounds (0.01 to 0.13).
So the cost the rule leaves is the band below the quarter that the rule does not decline,
stated as a band and not as a resolved difference (see the ranking).

### A source whose attributes convert most of its paths

The converting source (`measure-recheck2-crlf-*`):
the same 24,426 files with `*.md text eol=crlf` and `*.ts text eol=crlf` appended,
so 15,743 of 24,426 (0.644) convert and the rule declines;
one warm-up and seven counted rounds,
four arms.

- path (A′,
  declining):
  **2,089 ms** (2,030 to 3,570),
  **1.08 of `main`** [0.98 1.08 1.09 1.08 1.07 1.51 0.90]
- control:
  2,133 ms (2,035 to 2,784),
  1.09;
  control over path per round 1.01,
  1.01,
  1.03,
  0.99,
  1.02,
  0.78,
  1.00,
  **median 1.01**
- option C:
  2,037 ms (1,968 to 3,912),
  1.07
- `main`:
  1,952 ms (1,854 to 2,372)
- every arm:
  clean,
  `diffVsMain=0`.

The declining add's steps:
`checkSource` 72.7,
`git check-attr` 32.8,
the fence's own decision 19.1,
`copyIgnored` 39.4,
`git ls-files` 48.0,
then Git's `gitAdd` 1,730.1,
one checkout rather than a `--no-checkout` run followed by a `reset --hard`;
`main`'s `gitAdd` is 1,792.3.

What this says:
the declining shape is `main` plus the fence's reads,
about **0.07 of `main`** (137 ms median) in this run,
and about 0.11 (246 ms) in the previous round's kept `measure-r5-crlf-*`
(A′ 2,468 against `main` 2,222).
It is slower than `main` by construction;
the early decision took it from two Git checkouts to one,
and what remains is a cost the project must accept or refuse,
not an absent one.

### Option B beside the clone, this round's runs

Three runs of this round,
each on a throwaway repository of this repository's tracked files built from its working tree,
**with its real attribute set:
all eight `filter=lfs` lines kept**,
their 1,578 files added through the standard Git LFS filter,
24,435 tracked files,
14,166 of them `.md` or `.ts`.
The runs differ in the attribute lines alone:
"as it is" appends nothing,
the 0.199 source appends `*.md text eol=crlf`,
and the 0.644 source appends `*.md text eol=crlf` and `*.ts text eol=crlf`;
none sets `core.autocrlf`.
Five arms rotate one position per round:
option B (the shipped build),
its identical-binary control,
option A′ (`MONOCHROMATIC_CLONE_FENCE=beforeclone` on the measurement build),
option C (`decline`),
and `main`.
Every add ran in its own capped `systemd-run --user --scope`
(`CPUQuota=200%`,
`MemoryMax=4G`,
`TasksMax=400`,
`IOWeight=1`,
its environment through `env --ignore-environment`),
with `sync --file-system` and 300 ms between adds,
one warm-up round,
and every worktree compared with `main`'s **after** the counted rounds,
outside the timed window.

Rule `QNB`'s spread came first in each run,
one arm alone on unchanged input:
1,823 to 3,365 ms on this repository,
2,138 to 2,212 on the 0.199 source,
and 3,353 to 3,589 on the 0.644 source.
The host carried other sessions' work through all three runs
(load averages 8.9,
7.2 and 5.7 at their ends),
which is why each arm is quoted with its identical-binary control's own per-round disagreement beside it.

Binaries,
each tied to its commit:
option B at `a95b3e2eb3` (sha256 `c97aa0de…`),
the same code plus `measurement-routes` for A′ and C (sha256 `dc5de205…`),
and `main` at `f055bdc5c4`,
the tip this round merged (sha256 `ccd37cc5…`;
the two commits after it are handover documents and touch no module the add runs),
all under
`~/temp/agent/clone-rung-conversion-fence-beside-the-clone-2026-11-20/scratch/keep/binaries/`
with `sha256.txt` and `commits.txt` beside them.
The shipped build holds no `MONOCHROMATIC_CLONE_FENCE` string (`strings` finds none) and the measurement build does.
The commits after `57c6bae080` change no shipped code:
they add tests,
a comment,
and one move of two declarations between the clone rung's files
(`Created` and `Directories` now live in `worktree_clone_entry.rs` and are re-exported where they were),
so the measured behaviour is the shipped behaviour.
The two commits after the measured one (`0fe2cece9e`,
which moves the clone rung's serving step to its own file,
moves the index seed's function into the index-install module and takes the repository fixer's line breaks)
change no behaviour the runs touch;
the measured binary is therefore the shipped code's own,
byte for byte before that move.

The step split,
medians in ms:

- option B on this repository:
  the clean check 76,
  the fence 451 on the add's thread (check-attr 35.8,
  ls-tree 299.8,
  decision 12.7,
  the window's tail 73.1),
  the copies' wait 477,
  the install 45,
  `git checkout-index` 512 for 1,578 LFS paths,
  the whole record 1,875;
- option A′ on this repository:
  the clean check 77,
  the fence 276 on the waiting path (35.5,
  197.7 and 43.9),
  the copies 701,
  the install 46,
  `git update-index` 14,
  `git checkout-index` 538,
  the whole record 1,905;
- option B on the 0.199 source:
  the fence 463 (47.5,
  345.2,
  12.8 and 54.4),
  the copies 448,
  the install 44,
  `git checkout-index` 876 for 4,866 paths,
  the record 2,177;
- option A′ on the 0.199 source:
  the fence 266 (33.3,
  188.5 and 43.7),
  the copies 733,
  the install 46,
  `git update-index` 15,
  `git checkout-index` 890,
  the record 2,321;
- option B on the 0.644 source:
  the copies 731,
  then `finishRung` 2,522,
  which is the fallback `git reset --hard` writing all 24,435 paths over the copies,
  against `main`'s `gitAdd` 1,971;
- option C and `main` never clone or write through the fence:
  C is `main`'s checkout (`gitAdd` 1,845 to 1,926) plus the reads (26 to 27 ms).

Per round,
the option B arm against its identical control and against the other arms
(wall ms):

- this repository,
  five counted rounds:
  B 1,729,
  1,992,
  1,907,
  1,899,
  1,774;
  control 1,814,
  1,844,
  1,813,
  1,834,
  1,648;
  A′ 1,979,
  1,914,
  1,928,
  1,985,
  1,737;
  C 2,337,
  2,183,
  2,215,
  2,166,
  2,038;
  `main` 2,154,
  2,240,
  2,225,
  2,050,
  2,218;
- the 0.199 source,
  seven counted rounds:
  B 2,054,
  2,149,
  2,129,
  2,338,
  2,198,
  2,288,
  2,248;
  control 2,236,
  2,084,
  2,078,
  2,251,
  2,309,
  2,131,
  2,288;
  A′ 2,349,
  2,158,
  2,200,
  2,446,
  2,383,
  2,352,
  2,294;
  C 2,258,
  2,133,
  2,115,
  2,239,
  2,207,
  2,178,
  2,266;
  `main` 2,108,
  1,972,
  2,091,
  2,180,
  2,254,
  2,259,
  2,212;
- the 0.644 source,
  seven counted rounds:
  B 3,756,
  3,787,
  4,048,
  3,839,
  3,543,
  3,430,
  3,780;
  control 3,451,
  3,623,
  3,443,
  3,584,
  3,602,
  3,606,
  3,658;
  A′ 2,165,
  3,783,
  2,267,
  2,574,
  2,534,
  2,289,
  2,159;
  C 2,094,
  2,282,
  2,201,
  2,410,
  2,424,
  2,239,
  2,403;
  `main` 2,027,
  2,344,
  2,080,
  2,163,
  2,198,
  2,142,
  2,267.

What these say:

- **B against A′ is not resolved on wall time where the fence serves the rung.**
  The identical pair disagrees by 0.93 to 1.05 on this repository and 0.93 to 1.09 on the 0.199 source,
  and A′ against B is 1.01 and 1.03 across those rounds (rule `QNB`).
  What the records show instead is where the time went:
  B's copies overlap the fence,
  so the copies' wait is 477 against 701 ms and 448 against 733,
  while the fence's own steps are longer for the same overlap (451 against 276,
  463 against 266).
- **B's decline is resolved as a cost.**
  On the 0.644 source B is slower than A′ in six of the seven rounds,
  by 1,009 to 1,781 ms (the seventh is 4 ms apart),
  and slower than `main` in all seven by 1,288 to 1,968;
  its identical pair agrees at 0.96 (0.85 to 1.05),
  so 1.67 to 1.75 of `main` is a difference and not noise.
- **Every arm's bytes are right in all three runs:**
  `git status` clean and `diffVsMain=0` for all five arms of all three,
  after the rounds.

### This round's runs, with the decline decided before the copies

Three runs of this round,
each on a throwaway repository of this repository's tracked files built from its working tree,
**with its real attribute set:
all eight `filter=lfs` lines kept**,
their 1,578 files added through the standard Git LFS filter,
24,483 tracked files,
14,167 of them `.md` or `.ts`.
The runs differ in the attribute lines alone:
"as it is" appends nothing,
the 0.199 source appends `*.md text eol=crlf`,
and the 0.644 source appends `*.md text eol=crlf` and `*.ts text eol=crlf`;
none sets `core.autocrlf`.
Five arms rotate one position per round:
this round's option B (the shipped build,
decline decided before the copies),
its identical-binary control (the same shipped build),
option A′ (the whole fence before the copies,
`MONOCHROMATIC_CLONE_FENCE=beforeclone` on the measurement build),
option C (`decline`),
and `main`.
Every add ran in its own capped `systemd-run --user --scope`
(`CPUQuota=200%`,
`MemoryMax=4G`,
`TasksMax=400`,
`IOWeight=1`,
`RuntimeMaxSec=600`,
its environment through `env --ignore-environment`),
with `sync --file-system` and 300 ms between adds,
one warm-up round,
and every worktree compared with `main`'s **after** the counted rounds,
outside the timed window.
Five counted rounds on this repository,
seven on the 0.199 source and seven on the 0.644 source.

Rule `QNB`'s spread came first in each run,
one arm alone on unchanged input:
1,791 to 2,112 ms on this repository,
2,128 to 2,168 on the 0.199 source,
and 2,233 to 2,332 on the 0.644 source.
The host was otherwise idle during these runs
(load average 3.2,
3.9 and 3.7 at the three ends),
which is why the identical pair's own disagreement is quoted beside every number below:
it is the spread any difference has to clear.

Binaries,
each tied to its commit:
option B at `57c6bae080` (sha256 `3227035e…`) and the same code plus `measurement-routes` for A′ and C
(sha256 `c54c99d4…`),
both in this round's
`~/temp/agent/clone-rung-conversion-fence-b-decides-before-copying-2026-11-21/scratch/keep/binaries/`,
and `main` at `04ecd9e6d3`,
the tip `main` pointed at when the binary was built (sha256 `a7ac1fad…`),
with `sha256.txt` and `commits.txt` beside them.
The shipped build holds no `MONOCHROMATIC_CLONE_FENCE` string (`strings` finds none) and the measurement build does.

Each arm's wall time,
as median ms (range) and its ratio of medians to `main`:

- **this repository**:
  B 2,088 (1,824–2,099) 0.96;
  control 1,844 (1,764–2,868) 0.85;
  A′ 2,070 (1,689–5,876) 0.95;
  C 2,135 (2,118–2,238) 0.98;
  `main` 2,175 (2,124–4,385) 1.00.
- **the 0.199 source**:
  B 2,207 (2,158–2,460) 1.01;
  control 2,125 (2,059–2,301) 0.97;
  A′ 2,210 (2,141–2,336) 1.01;
  C 2,159 (2,096–2,284) 0.98;
  `main` 2,191 (2,109–2,324) 1.00.
- **the 0.644 source**:
  B 2,337 (2,247–2,539) 1.10;
  control 2,325 (2,226–2,541) 1.10;
  A′ 2,323 (2,160–2,465) 1.10;
  C 2,187 (2,064–2,289) 1.03;
  `main` 2,117 (2,061–2,301) 1.00.

Per-round ratios,
which is what each arm's own spread has to be compared with.
Each round's own values are in the kept `.summary.txt` files;
what they say:

- **B against A′**:
  1.01 on this repository
  (0.98 to 1.08 across five rounds;
  one round stalled at 0.36),
  1.01 on the 0.199 source (0.92 to 1.07 across seven rounds),
  and **0.99 on the 0.644 source** (0.92 to 1.18 across seven rounds).
  The identical pair disagrees by 0.88 to 1.37 on this repository
  (one stalled control round),
  0.86 to 1.00 on the 0.199 source,
  and 0.95 to 1.05 on the 0.644 source,
  so none of these differences is resolved:
  B is not slower than A′ anywhere,
  and the 0.644 source no longer separates them at all.
- **B against `main`**:
  0.95 on this repository
  (0.44 to 0.96;
  the 0.44 is one round in which `main` alone stalled,
  at 4,385 ms against B's 1,824),
  1.04 on the 0.199 source (0.98 to 1.08),
  and **1.10 on the 0.644 source** (1.02 to 1.20).
  The last is resolved against the identical pair's 0.95 to 1.05,
  and it is the price of declining at all:
  the declined add's own steps are the ladder's clean check (73 ms),
  one `git check-attr` (33 ms) and the fence's decision (21 ms),
  and then Git's own `gitAdd` (1,971 ms against `main`'s 1,909 in the same run).
- Every arm of all three runs has `git status` clean and `diffVsMain=0`.

The host was otherwise idle during these runs;
its load average was 3.2 at the first run's end,
3.9 at the second's,
and 3.7 at the third's.

The step split,
medians in ms,
with every arm's numbers in the kept `.summary.txt` files:

- B on this repository:
  the clean check 77,
  the fence's attributes and decline 23.5 (of which `git check-attr` 37.9 overlapped),
  the remaining fence beside the copies 23.1 (`git ls-tree` 267.8),
  the copies 524,
  the install 49,
  `git checkout-index` 774 for 1,578 LFS paths,
  the record 2,062;
- A′ on this repository:
  the whole fence 37.3 on the waiting path (`git ls-tree` 203.3),
  the copies 752,
  `git update-index` 15,
  `git checkout-index` 613,
  the record 2,046;
- B on the 0.199 source:
  the fence's attributes 21.7 (`git check-attr` 33.4),
  the rest beside the copies 23.2 (`git ls-tree` 267.4),
  the copies 656,
  `git checkout-index` 921,
  the record 2,182;
- B on the 0.644 source:
  no clone step at all;
  the fence declines at 21.3 plus `git check-attr` 32.9,
  and the record is `gitAdd` 1,971 with the ladder's own checks before it;
  A′'s record is the same shape (21.4 plus 34.3 and `gitAdd` 1,960),
  which is why the two are one arm in that run to within the pair's spread.

Three things the runs say about the order:

- The decline is now the cheap one:
  on the 0.644 source B is 1.10 of `main` and its own control is 1.10 too,
  against the previous round's 1.67 to 1.75,
  and A′'s 1.13 in this run is the same shape as B's.
- Beside the copies the remaining fence is cheap work rather than the whole read:
  its step is 23 ms and its `git ls-tree` runs under `cloneEntries` (267 ms inside 524 to 656 of copying),
  where A′'s whole fence sits on the waiting path before the copies start.
  The copies' wait is 524 to 656 beside 752 in A′'s run on this repository and 673 on the 0.199 source.
- The totals are still inside the pair's disagreement where the rung serves the add,
  so this round claims no wall-time win for B over A′:
  it claims that the decline no longer costs a copy,
  and that the rest of the fence no longer waits in front of the
  copies.

### The decline rule's threshold, from these runs

No fitted model,
these runs' own per-path prices:
0.35 ms for an LFS path and 0.12 ms for a text path,
and the fence's non-write work is its total minus its write step:
2,451 − 930.4 = 1,520.6 ms on the 0.199 source,
against `main`'s 2,260 in the same run.
The fence stays under `main` while its written paths cost less than 2,260 − 1,520.6 = 739 ms.
1,578 LFS paths cost 552 ms,
which is under;
4,866 mixed paths cost 930 ms,
which is over.
The break-even therefore lies between those shares,
**0.065 and 0.199**,
and the quarter the rule declines past is **above** the break-even,
which is why 0.199 is measured at 1.13 of `main`.
The previous round's derivation (a 0.114 ms mean over both classes and one `checkout-index`) is replaced;
0.114 ms stands for no path class.

The rule is therefore not a break-even rule,
and this proposal no longer says it is.
Its threshold prices option A′'s decision,
and option B changes what a decline costs:
measured on the 0.644 source,
A′'s decline is 1.06 to 1.09 of `main` and B's is 1.67 to 1.75,
because B pays the copies first.
A threshold that minimized B's cost would be lower,
or the decline would be decided from the attributes before the copies;
neither is measured,
and the quarter is kept as the threshold A′'s runs set.
What it does:
it keeps the clone rung for sources whose converting share is small
(this repository's 0.065 is measured at 0.98 to 1.11 of `main`)
and declines past a quarter,
where writing the paths would cost more than Git's checkout.
What it leaves:
a source whose share lies between the break-even and a quarter pays up to about 1.2 of `main`
(1.13 measured at 0.199;
the review measured 1.22 on its own run).
The project can keep the quarter and accept that band,
or lower the share
(an eighth would decline at 0.199 and still keep this repository's 0.065)
and pay for one more measurement at the new boundary before shipping it.
This proposal keeps the quarter because no other boundary was measured,
and states the band as the project's to accept or refuse.

### The two Git runs, and what replacing them would have to do

The fence's reads over every tracked path are one `git check-attr` (36.5 ms here)
and one `git ls-tree -r -l` (202.8 ms),
**239 ms together,
0.10 of `main` on this fixture**,
and `git ls-tree` is now unconditional (the size proof for every path the fence may carry),
which this repository already paid before this round because all its paths are `text=auto eol=lf`,
and a repository with no attributes at all pays it for the first time
(the gate review's H2).

The in-process replacement this proposal's earlier draft promised to remove them with
**was not built this round**,
and the earlier draft's ground for calling it a large change was wrong:
the repository already has a pack reader.
`package/git-policy/cli/src/native/repo_read/repo_read_pack.rs` is the version 2 index lookup by object name and
the entry read,
with `OFS_DELTA` and `REF_DELTA` resolution (`:1` to `:14`),
a delta depth limit of 100 (`:30`,
above this repository's 50-deep chains),
`apply_delta` (`:127`),
`Pack::find` (`:255`) and `Pack::read` (`:321`).
What is missing is a size-only method,
and that is small in principle:
a whole object's size is in its entry header in the pack format,
and a deltified blob's size does not need the chain resolved:
Git's own `get_size_from_delta` (`packfile.c:901` to `:953` at 2.56.1)
inflates the first 20 bytes of the delta stream and returns the result-size varint.
So a `git ls-tree` replacement is a new method on an existing reader,
not "a reader with pack and delta support",
and the same holds for the attributes:
the tracked `.gitattributes` blobs could be read in process instead of one `git check-attr`.
Neither was built or measured.
For scale,
the measured in-process walk of the same 24,389 paths that does one `lstat` each
(the clean check's own `checkSource` step) is 69.6 ms in the same records.
The honest statement is:
the two runs cost 239 ms here,
a size-only pack read is new code on an existing reader,
and what it and an in-process attribute read would cost is unmeasured until someone builds them.

### The LFS pointer as a cheaper proof, weighed and not built

A suggestion worth stating because LFS is most of the fence's cost here:
a pointer carries the content's `size` and `oid sha256:`,
so a clean source's LFS path might be provable from its stat size against the pointer's size,
or its SHA-256 against the oid,
without running the filter.
It was reasoned against Git's and git-lfs's source and **not built**,
for two reasons.

The cheap variant is not a proof.
The text paths' size proof works because their clean side can only remove bytes:
equal sizes leave nothing removed,
so the file is the blob.
git-lfs's clean is a content hash,
not a length edit;
`git-lfs clean -- %f` reads the file and prints a pointer whose `size` is the file's byte length and whose `oid` is the
SHA-256 of its bytes,
and `git-lfs smudge -- %f` writes the object at `.git/lfs/objects/<oid[0..2]>/<oid[2..4]>/<oid>`,
or fails when it is absent.
A pointer's `size` is text in a blob that nothing on the carrying path re-derives from the object,
so a hand-written or rewritten pointer can name a size that is not the object's length,
and a working file of that length is not thereby the object's bytes.
Size equality proves nothing for a hash-based clean.

The complete variant is not measurably cheaper.
It would hash every LFS working file and compare with the oid,
which reads the whole content:
this repository's 1,578 LFS paths hold 244,553,284 bytes in the worktree (`lfs-bytes.ts` over `git ls-files`),
where the smudge that this round measured copies the same bytes and took 0.64 to 0.91 s
(642.0 to 905.5 ms of `git checkout-index` in the two arms' medians,
541 to 4,049 ms per round).
Reading 244 MB and hashing it costs at least the read half of that,
and it needs `.git/lfs`'s object store to be present and trusted,
which the smudge already assumes.
No measurement of a built hash path exists,
so the claim this proposal makes is only that the cheap proof is unsound and the complete one buys at most the write
half of a cost that is already inside the fence's own spread.

When a stat-exact LFS path could hold bytes other than its smudge,
then:
when the stat data that vouches for it is not the object's (the same trust every carried path already extends to its
stat),
when the pointer names a `size` the object does not have,
and when Git's own line-ending conversion would rewrite the smudge on the way out
(the fence already sends those paths to Git,
because their `text` action applies before the size is looked at).

### What this round did not measure

- An add whose new `HEAD` differs in a path Git must write:
  the timed fixtures check out one commit in both trees,
  so no measured arm pays the clone-and-checkout the tree difference can force on option B;
  the class harness's `m05` reaches it at three files and shows the route,
  not a price.
- A clone rung below the decline rule with a share between this repository's 0.065 and the measured 0.199:
  the band is priced by the two endpoints and no run sits inside it.
- A repository with no conversion rules at all:
  not built.
  What differs from the "as it is" run is that no path is written through Git,
  so its write step (551.7 ms here) and nothing else would go,
  and the reads (about 240 ms in the previous round's records) and the clone are the same.
  That is arithmetic on measured steps,
  not a measurement.
- `core.autocrlf=true` with no attribute override:
  the earlier round measured option D at 2.07 times `main` on it
  (`measure-r4-*`,
  4,274 ms against 2,057),
  and this round did not repeat it.
- XFS and overlayfs:
  not measured.
- The btrfs guest at this repository's size:
  not run.
  The guest this round ran the ladder module's 66 controls on a btrfs image (§ "The twelve class controls,
  in a
  guest"),
  which is not the same as an add of this repository's 24,426 paths.
- A share between the measured 0.065 and 0.199 to locate the break-even more tightly,
  and a run at another decline boundary than the quarter (the threshold section says what this leaves).
- An in-process size read or attribute read (unbuilt;
  see the two Git runs above).

## How the fence applies to rung 1, the snapshot

The snapshot rung carries the same bytes,
and the guest measured it wrong for a clean and a dirty source.
This round gives it the same fence and the same ordering rule as the clone rung:
**no path is recorded clean before its bytes are Git's checkout's**,
and the rule fixes where the fence is decided as well as what the write does.

The snapshot installs the new worktree's index itself
(`worktree_snapshot_index_step::install_index`),
so the fence must be decided **before** the snapshot:
decided after it,
as the previous round's rung 1 did,
the installed index carries the source's proven stat fields for the paths Git must write,
and a process killed before the fence's zeroing leaves an index that vouches for the source's bytes there.
`choose_rung` now decides the whole fence for rung 1 (`fence_the_clone`)
before it claims the stage,
and its answer travels in the rung.
What that changes:

- Each written path's entry is installed with its stat data **zeroed**
  (the same `with_zeroed_stat` seed the clone rung uses),
  and the walk's proven fields are never written over it,
  so the published index cannot vouch for a path whose bytes are not Git's yet.
- The write is the clone rung's own step (`checkout_index_through_git`):
  the fence's answer intersected with the destination's index,
  and one `git checkout-index --force -u -z --stdin` in the stage.
  The previous round's `write_through_git` (one `git update-index --index-info` to zero the entries,
  then the checkout) is no longer needed here,
  because the install already carries the zeroes.
- A decline or a refusal is decided **before** the snapshot,
  so a declined btrfs add pays one `git check-attr` and Git's own checkout,
  where the previous round's rung 1 paid the whole snapshot and then the checkout.
- The fence's reads run on the add's waiting path in this rung:
  a snapshot copies everything anyway,
  so there is no second thread for them to run beside,
  and this round does not claim one.

Three details the previous round's text got wrong,
and what is true now:

- **The fence reads the source,
  not the stage,
  and that is the stage's own answer.**
  A dirty source whose changes include any `.gitattributes` file
  (untracked,
  modified,
  or staged) is refused by the snapshot rung's own check
  (`worktree_snapshot_source.rs`,
  `changed.attributes_changed`),
  so for every source the rung serves,
  the worktree's attribute files are its index's and its `HEAD`'s.
  The stage keeps them,
  the fence reads the same bytes either way,
  and the gate review's L3 case cannot reach the rung.
- **A dirty source's stage has had `restore_dirty` replace the files the index could not vouch for;**
  those are already Git's bytes,
  and the fence writes them a second time,
  harmlessly.
- **The exchange still happens after the fence**,
  so a fence or a write that fails ends the rung where `restore_dirty` failing ends it:
  the stage is destroyed and Git's own checkout completes the worktree.

What this round measured on rung 1:

- A new btrfs-gated control runs the whole rung on a subvolume whose `*.txt` rule converts,
  asserts the fence's `checkout-index` step ran,
  and compares the served path with a real Git checkout of the same commit:
  `btrfs_a_converting_path_is_written_by_git_and_the_snapshot_rung_is_clean`
  (`worktree_ladder_finish_tests.rs`),
  run in the guest's btrfs image.
- A control over the index step asserts the ordering rule itself:
  `the_index_the_snapshot_installs_vouches_for_no_path_the_fence_leaves_to_git`
  (`worktree_snapshot_index_step_tests.rs`),
  red first with the previous order planted
  (`it records 18 bytes ... Git's checkout writes 20 bytes`).
- The costs the rung pays are not measured at this repository's size.
  The guest's btrfs image serves fixtures of a few paths,
  and no round has timed a snapshot add of 24,435 files on btrfs:
  what such an add pays is the fence's two reads (35 to 48 ms for the attributes and 190 to 345 ms for the sizes,
  measured on the clone rung's runs),
  the fence's per-path write step for its converting paths,
  and the snapshot's own cost,
  which `main` already pays on btrfs.
  A decline there is now the cheap one:
  the same reads and Git's own checkout.

## The options, with their measured cost

The project chose option B on 2026-11-20T23:27:00-05:00
("Yeah,
do B.",
`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
step 10's notes),
after the previous round put A′,
B,
C,
D and E to it.
This section is therefore the record of what B was chosen over,
not a ranking to answer again:
**B is the remedy**,
and each option below carries the cost it was measured at,
so the project can see what it accepted.

Every option keeps the clone rung (or gives it up) while writing no byte Git's own checkout would not.
The numbers are kept runs'
medians of the whole add,
24,435 tracked files and 1,578 LFS paths in this round's fixtures,
with the identical-binary control beside every arm that this round ran,
and every arm's worktree clean with `diffVsMain=0` against `main`'s.

**B.
The fence beside the clone (the project's remedy).**
The fence's reads (`git check-attr`,
the decline count,
`git ls-tree` and the decision) run on their own thread while the clones copy the tracked files;
the index is installed only after both have answered,
from the source's proven index with the written paths' stat data zeroed;
one `git checkout-index` then writes them.
Measured this round (`measure-beside-real-*`,
`-md-*`,
`-crlf-*`,
five and seven counted rounds):

- this repository as it is (1,578 of 24,435 paths convert,
  0.065):
  **1,899 ms,
  0.86 of `main` per round (median of per-round ratios;
  the ratio of medians is 0.86)**,
  its identical-binary control 1,814 ms (0.82 per round,
  0.82 of medians),
  `main` 2,218;
  the identical pair agrees at 0.95 (0.93 to 1.05),
  so this run separates neither B from `main` nor B from A′
  (A′ 1,928,
  0.87;
  the pair's per-round ratio of A′ to B is 1.01,
  0.96 to 1.15).
- the 0.199 source (4,866 of 24,435 paths written by Git):
  **2,198 ms,
  1.02 of `main` per round (1.01 as a ratio of medians)**,
  control 2,236 (1.03 per round),
  A′ 2,349 (1.06),
  C 2,207 (1.02),
  `main` 2,180;
  the pair agrees at 0.98 (0.93 to 1.09) and A′ against B is 1.03 (1.00 to 1.14),
  so again not separable.
- the 0.644 source (15,743 of 24,435 paths,
  above the quarter,
  so B declines after its copies):
  **3,780 ms,
  1.67 of `main` per round** (1.75 as a ratio of medians),
  control 3,602 (1.66 per round;
  the identical pair agrees at 0.96,
  0.85 to 1.05),
  A′ 2,289 (1.09),
  C 2,282 (1.06),
  `main` 2,163.
  Here the difference is resolved:
  B is slower than A′ in six of the seven rounds,
  by 1,009 to 1,781 ms (the seventh is 4 ms apart),
  and than `main` in all seven by 1,288 to 1,968.

What B buys,
and what it does not,
is in the record of the same runs:
the copies start as a task of the runtime and their slices' jobs run on the blocking pool while the add's own thread
runs the fence,
so the copies' wait overlaps the fence's reads.
The fence's steps on the add's own thread are **451 ms on this repository** (`git check-attr` 35.8,
`git ls-tree` 299.8,
the decision 12.7 and the window's own tail 73.1)
against A′'s **276 ms** (35.5,
197.7 and 43.9),
and on the 0.199 source **463 against 266**:
B's `git ls-tree` is slower because during its pipe reads the add's thread also drives the copies' task.
What the overlap saves is the copies' wait:
the clones' own step is 477 against 701 ms on this repository and 448 against 733 on the 0.199 source,
because in B part of that wait is spent inside the fence's steps.
The two totals then are 1,899 against 1,928 and 2,198 against 2,349 ms,
inside the identical pair's disagreement in both runs,
so what B removes from the waiting path is the rest of the copies' wait,
and this round did not resolve its wall-time size:
the readings that would (the fence's own work,
421 to 443 ms,
against the copies' 448 to 477) are the same order.
Right on every path in all three:
18 of 18 class controls in the guest,
26 of 26 scenarios in the class harness (one of which ends the rung at the tree difference,
`m05`),
and 300 of 300 repositories of the 150-combination sample.
Its limits are the "Limitations" section's.

**A′.
The fence before the clone with its decline rule**
(the shape the previous round put first,
kept as the measurement arm behind `measurement-routes`).
One `git check-attr` over every tracked path,
one `git ls-tree`,
the decision in process,
then one `git update-index` and one `git checkout-index` for the paths left to Git;
past a quarter of the converting paths and past a thousand of them,
the rung ends before anything is copied.
Measured this round (`beforeclone`):
**1,928 ms on this repository (0.87 of `main` per round,
0.87 of medians)**,
**2,349 on the 0.199 source (1.06,
1.08)**,
**2,289 on the 0.644 source (1.09,
1.06)**;
its fence puts 266 to 276 ms on the waiting path,
and its copies are 701 to 733 ms because they start after that read.
Right on every path in all three.
Its costs above `main`:
a declining add is `main` plus the reads (1.06 to 1.09 measured here,
0.07 to 0.11 in the previous round),
and the 0.199 band is up to about 1.1.

**A.
A′ without its decline rule.**
Measured in the previous round on the 0.199 source at 2,703 ms,
1.18 of `main` [1.05 to 2.95 per round],
control 1.01,
and in the round before on the converting source at 3,192 ms (1.50).
Not measured this round.
Its cost above `main` is the whole write step where many paths convert,
which is why the rule exists.

**C.
A whole-source refusal,
no per-path fence.**
Measured this round (`decline`):
**2,183 ms on this repository (1.00 of `main` per round,
0.98 of medians)**,
**2,207 on the 0.199 source (1.02,
1.01)**,
**2,282 on the 0.644 source (1.06,
1.06)**,
which is `main`'s checkout plus the wrapper's checks,
because a refused add is Git's checkout.
Its cost above `main` is those checks,
and it gives up the clone rung for every source with a conversion setting or an attributes file,
which includes this repository's first attribute line:
on this repository that is the disk saving at 0.49 to 0.59 of `main`.

**D.
Git writes every tracked path after the clone.**
Measured in the earlier round on its clean fixture:
4,274 ms against `main` 2,057 (2.07 times `main`),
and 4,170 on the converting one.
Not measured this round.
Its cost above `main` is about one whole checkout,
because the clone is wasted work under it.

**E.
Keep step 3 as it is and accept the wrong bytes.**
Wrong by 7,097,444 lines of `diff` against `main`'s worktree in the earlier round's converting run,
with every stat check green.
Rejected:
`main` on ZFS gets these bytes right,
so it fails the merge bar with no waiver that covers it,
and `SPEC.md:1223` promises the snapshot's worktree is what Git's checkout makes.

**The costs above `main` the project accepted with B.**
One paragraph,
because they are the price of the choice and not an absence of one:
where the fence writes few paths B measures 0.86 of `main` on this repository and 1.02 at 0.199,
with its identical pair agreeing at 0.95 and 0.98,
so neither distance is resolved and the costs there are not shown to be above `main` at all;
above the quarter a decline now pays the copies as well as Git's checkout,
measured at 1.67 to 1.75 of `main` on the 0.644 source against A′'s 1.06 to 1.09 and C's 1.06.
The third is the one the choice buys and the project should have in front of it:
a repository whose attributes convert most of its paths gets a correct worktree at about 1.7 times `main`'s time,
where the same repository under A′ would get it at about `main`'s.
Deciding the decline from the attributes before the copies (33 to 35 ms of `git check-attr`) is the change that would
remove that cost,
and it is not built or measured.

## What these costs imply for the partial-clone proposal

The partial-clone proposal
(`docs/planning-cli-git-clone-rung-partial-clone-of-a-dirty-source`) waits on this fence,
and an earlier estimate put a fenced dirty partial clone at 0.97 to 1.08 times `main`
from the fence's 0.12 to 0.23 of `main` against the partial clone's gain of 0.14.
This round's numbers support only the parts of that estimate that compare like with like:

- the reads are 239 ms,
  0.10 of `main` on this fixture,
  and they do not shrink by being put beside the clone
  until B is built;
- the write step is real and large where a filter applies:
  1,578 LFS paths cost 0.64 to 0.91 s
  (0.41 to 0.57 ms each) in the previous round's two arms,
  0.55 s (0.35 ms each) in the round before this one,
  and 0.31 ms each in the diff check's own run of the same shape.
  **That write is not an increment over `main`**:
  `main` on the same LFS-bearing tree writes the same 1,578 paths through the same smudge,
  so a fenced partial clone cannot be priced by adding the fence's LFS write to a `main` measured without it.
  The earlier draft did exactly that and its 1.3 to 1.5 times `main` figure is **removed**.
- on the fence's own as-it-is numbers the fence and `main` are not separable (see the cost section),
  so a fenced dirty partial clone of a source with this repository's own attribute set is expected near `main`,
  with the partial clone's gain erased by the fence's reads and its LFS handling,
  not multiplied.
  A source with no LFS content and no converting rule was named as the case where the review's earlier 0.97 to 1.08
  stands as a ceiling,
  and that figure is **withdrawn**:
  it came from the estimate this section removes,
  and no kept run measured that case.

No fenced partial clone has been measured,
so the partial clone cannot be put to the project with this fence as it stands,
and the LFS question above is the one to settle first:
either Git's smudge stays the proof and the partial clone pays for it,
or someone finds a cheaper complete proof that a working file is the smudge of its pointer.

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
> and the source's file is proven to hold that blob.
> The proof is Git's own conversion rules,
> read from the path's attributes (`text`,
> the legacy `crlf`,
> `eol`,
> `filter`,
> `ident`,
> `working-tree-encoding`),
> `core.autocrlf` and `core.eol`,
> in the order `convert.c` reads them:
> a path whose checkout could add a carriage return,
> expand an `ident` keyword,
> convert a `working-tree-encoding` or run a filter's smudge is written by Git in the new worktree,
> through one `git checkout-index --force -u`;
> every other path is carried when its recorded size equals the size of its blob,
> which the one `git ls-tree` reads,
> because a rule can only shrink a file it touches on the way into the index and a file with no rule today can still
> hold a form an earlier rule wrote.
> The attributes and the decline are decided before anything is copied
> (`git check-attr` and a count over its answer),
> so a source above the quarter falls to Git's checkout and pays no copy;
> the rest of the fence (`git ls-tree` and the decision) runs on a thread of their own while the clones copy the tracked
> files,
> and the index is installed only after both have answered,
> from the source's proven index with the stat data of every path Git must write zeroed,
> so no step between a copy and Git's write records such a path clean;
> one `git read-tree -m -u` then applies the tree difference and one `git checkout-index --force -u` writes the fenced
> paths the destination's index still holds,
> which is where their own stat data is recorded.
> A process killed between the install and that write leaves an index in which the paths Git had not written hold no
> stat data,
> so the first `git status` reports them as changed instead of trusting them.
> Where more than a quarter of the tracked paths,
> and more than a thousand,
> have a rule of their own that converts them,
> the rung declines before any copy and Git's checkout writes the worktree,
> so a declined add is Git's own checkout plus one `git check-attr` and the count over it
> (measured on this repository at 1.06 to 1.09 of `main`,
> where the same rule decided after the copies measured 1.67 to 1.75).
> An add whose new `HEAD` differs in a path Git must write can end the rung at the tree difference,
> because its zeroed entry cannot be verified as up to date;
> Git's own checkout then writes the worktree and the bytes are equal.
> A source with an untracked or ignored `.gitattributes` anywhere above a tracked path refuses the rung,
> because Git read it when the files were added and the new worktree will not have it,
> and the refusal is taken before anything is copied.
> A source that sets `attr.tree` or `GIT_ATTR_SOURCE` refuses the rung,
> because the destination would read attributes from a tree the fence did not read.

### `package/git-policy/cli/SPEC.md`, rung 1

The promise at `SPEC.md:1223` stands as written ("the new worktree is restored to what `git worktree add` makes from a
clean checkout");
what changes is that it now holds for conversions too.
New text,
after the restore's steps:

> The same conversion fence decides the snapshot rung,
> before the snapshot and before its stage name is claimed:
> the tracked paths whose checkout would differ are known then,
> and the index the snapshot installs holds them with no stat data,
> so no path is recorded clean before Git's own write records the bytes it checked out.
> In the stage,
> and before it is swapped in,
> the paths the fence leaves to Git are written by one `git checkout-index --force -u` in the stage,
> intersected with the index the tree difference left.
> A fence that cannot read its inputs ends the rung and Git's own checkout completes the worktree,
> and so does the fence's decline rule,
> now before the snapshot rather than after it.

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
> not the conversion of their blobs,
> so both rungs then run the conversion fence described in
> `doc/planning/cli-git-clone-rung-conversion-fence.md` and let Git write the paths it names.
> The LFS definition this bullet accepts is not exempt from that:
> a working file whose content came from a smudge cannot be proven to be that smudge without running it,
> so the fence writes LFS paths through Git's own filter,
> and their cost is measured at 0.31 to 0.57 ms per path across kept runs.
> A source with an untracked or ignored `.gitattributes` above a tracked path declines both rungs,
> because Git read it when the files were added and the new worktree will not have it,
> and a source where more than a quarter of the tracked paths,
> and more than a thousand,
> have a rule that converts them declines both rungs,
> because Git's own checkout is cheaper than writing that many paths inside the rung.

### `doc/decision/cli-git-zfs-clone-rung-dirty-source-waiver.md`

No change,
and no waiver is asked for this fence.
The waiver record names two time costs it covers (a dirty-source add about 0.2 s slower,
and the small-file clean add),
and neither is this fence;
the earlier draft read the waiver as covering "a time cost",
which the gate review corrected (H4).
The costs above `main` this shape has are **costs**,
not an absence of them:
a declining add is `main` plus the fence's reads (0.07 of `main` in the previous round's run and 0.11 in the one before
it),
and an add below the decline rule that writes many paths is up to about 1.2 of `main` (1.13 measured at 0.199).
Option B changes the first of those:
a decline after its clone pays the copy and then Git's checkout,
which is this round's to state as a cost (see the options).
Whether the project asks for a waiver for them or refuses the shape is the project's answer to give;
this proposal does not decide it,
and it no longer says the costs are absent.
The wrong bytes are a merge bar failure neither waiver covers,
which is why this proposal exists.

## What this round closed, and what went wrong in it

This is the round after the gate review of this proposal on Sonnet
(`~/temp/agent/gate-review-of-the-clone-conversion-fence-on-sonnet-2026-11-20/keep/review.md`).
Its six findings from H1 to M6 and the lows it named are each fixed or dismissed below.

Closed,
with the evidence kept beside this round's records:

- **H1,
  the decision is Git's `convert_attrs`.**
  `action_of` reads `text` first,
  falls back to the legacy `crlf` attribute,
  lets `eol` refine an action nothing has
  named,
  and only then consults `core.autocrlf`,
  which is `convert_attrs` line for line at 2.56.1.
  Unit controls added for `eol=crlf` alone,
  `eol=lf` alone,
  `crlf` alone with `core.eol=crlf`,
  `crlf` over an added CRLF file,
  and `text` winning over `crlf`;
  binary controls for the same five forms plus the two no-rule cases below.
  The old prototype binary differs from Git's checkout on `m01`,
  `m02`,
  `m03`,
  `m03b`,
  `m04` and `m06` and equals it
  everywhere else (16 scenarios,
  `class-records-prototype-old.json`);
  the fixed binary equals it on all sixteen (`class-records-new.json`);
  step 3 at `74caa69e7a` differs on `c01`,
  `c02`,
  `c03`,
  `c04`,
  `c07`,
  `c06b` and `m01` to `m06`
  (`class-records-step3.json`).
- **H2,
  a path with no rule can hold bytes Git's checkout would not write.**
  The blob's size now decides for every path whose checkout writes the blob unchanged,
  no rule included,
  so `git ls-tree` is unconditional;
  the sentence "invisible to any fence" is gone,
  and the price is stated (202.8 ms here,
  already paid by this
  repository,
  new for a repository with no attributes).
  Binary controls for an unset `core.autocrlf` after the add (`m04`) and a tracked attributes file a later commit
  deleted (`m06`),
  both red on the old prototype and on step 3,
  green now.
- **H3,
  the cost is measured on this repository as it is,
  LFS lines kept.**
  Both runs of this round kept the eight `filter=lfs` lines and added 1,578 files through the standard LFS filter,
  and the design now says what proves an LFS path is the smudge of its pointer (Git wrote it) and what it costs
  (0.31 to 0.57 ms per path across kept runs,
  and 0.55 to 0.91 s for this repository's 1,578).
  The earlier promise of a section about carrying them is replaced by that reasoning.
- **H4,
  the decline rule priced,
  and the shapes that are slower than `main` stated as costs.**
  The rule declines past a quarter of the tracked paths and past a thousand;
  measured on the converting source:
  2,468 ms with the rule against 3,192 without,
  `main` 2,222,
  control 0.99.
  That claim in this bullet once read "no shape slower than `main`",
  which this round withdraws:
  the declining add is 1.07 to 1.11 of `main` across kept runs,
  the 0.199 band is 1.13,
  and option B's decline pays the clone as well (the options section).
  Option A without the rule is listed and measured,
  and the waiver question is the project's.
- **M1,
  every number re-quoted from the runs it comes from.**
  The `r3` spread line,
  the "271 of 482 ms" decomposition,
  the `41 ms` and `230 ms` steps,
  the "55 ms of decision
  work",
  option B's three different savings,
  the "wrong on 24,389 files" sentence and the stale first introduction are
  gone;
  this proposal's numbers now come from `measure-r5b-real-*`,
  `measure-r5-crlf-*` and,
  where marked,
  `measure-r4-*`,
  with each arm's per-round ratio to `main` and the identical-binary control's spread beside it.
- **M2,
  the ranking follows the measurements.**
  Only built and measured options are ranked,
  per workload class;
  B is listed as unmeasured and ranked nowhere;
  the break-even between A and `main` (about 7,175 paths,
  0.294 of 24,389) is shown,
  which is what the decline rule
  is set from.
- **M3,
  the in-process reads.**
  Not built this round,
  and the claim that they would remove the reads is removed instead:
  the two runs are 239 ms here.
  The earlier draft's ground for calling them a large change was wrong and is retracted:
  the repository has a pack reader (`repo_read/repo_read_pack.rs`,
  version 2 index lookup by name with `OFS_DELTA` and `REF_DELTA` resolution),
  a size-only method on it is new code and not "a reader with pack and delta support",
  and the tracked `.gitattributes` blobs could be read in process instead of one `git check-attr`.
  Neither was built or measured.
  The measured in-process walk of the same paths that does one `lstat` each is 69.6 ms,
  for scale,
  not as a price.
- **M5,
  the controls and the gate.**
  The six new binary controls are in the tree,
  the ignored-attributes control's pattern is corrected to one that matches
  (`c06b`),
  and the full gate was run at `a690379aa5` as its own capped unit:
  9,303 tests,
  9,301 passed,
  two failed.
  What that gate did **not** establish,
  and the previous round's text wrongly said it did:
  the twelve binary class controls of `worktree_clone_conversion` printed `ZFS CONTROL NOT RUN` there and returned
  before any add,
  because the image's fixture root is a tmpfs and the helper refuses without a guest named by
  `CLI_GIT_ZFS_FIXTURE_GUEST`
  (`binary_support_zfs_controls.rs:82` to `:107`).
  Their nextest `PASS` lines were vacuous (rule `TRV`).
  This round ran them in a throwaway guest (§ "The twelve class controls,
  in a guest"),
  and ran the whole gate again at this round's final commit.
- **M6,
  no measurement switch in shipped code.**
  The three routes are behind an off-by-default `measurement-routes` feature which no shipped build enables:
  `strings` on the shipped release binary finds no `MONOCHROMATIC_CLONE_FENCE`,
  and the `off` route,
  which carried every
  path,
  is deleted.
  The proposal says the feature is for measurement only and does not ship enabled.
  Only two `mise` tasks name it (`native:build:measurement`,
  `native:clippy:measurement`),
  their descriptions say the
  feature is off by default and that they exist only beside it,
  and `native:test:host:cargo`,
  a second test runner
  beside the nextest one rule `RNQ` names,
  is removed:
  `cargo-nextest` 0.9.152 is installed on this host.
- **L1,
  a fenced path the new `HEAD` lacks.**
  The write step sends only the paths the destination's index holds,
  so the run no longer fails and no entry is injected,
  and the record's route now says "Git's checkout after the clone rung ended" (or the snapshot's) when a rung ends
  after its work began.
  Measured:
  `m05` no longer ends the rung,
  and the route is truthful in `m05`,
  `c06`,
  `c06b` and every declined arm.
- **L2,
  the zero stat data.**
  `git checkout-index -u` records the stat data of every path it writes.
  The first-status cost this removes is not re-timed this round;
  it sits outside every timed window either way.
- **L3,
  the three doc and code disagreements.**
  The `attr.tree` sentence now says the prototype does not refuse it and the proposal's new text requires the refusal;
  the rung-1 ancestor walk is corrected in the text to run in the stage;
  and the declined-add count is a class the
  decline rule now names.
- **L4,
  the fixture's `core.autocrlf=true`.**
  This round's converting fixture sets no `core.autocrlf`;
  the earlier one did,
  behind an attribute line that overrode it,
  and the text now says so.

What went wrong and is recorded so the numbers are not read for more than they say:

- The first of this round's "as it is" runs (`measure-r5-real-*`) verified the worktrees inside the counted loop,
  and the 8 GB of comparisons polluted the next arm's timing:
  its identical-binary control ranged 0.35 to 2.71 of the fence.
  The second run verified after the rounds,
  and its control ranged 0.35 to 1.64 with a median of 1.03;
  only that run's numbers are quoted for option A.
- On this host as other sessions load it,
  the fence and `main` are not separable on this repository's own attribute set:
  the fence's 1.11 of `main` sits inside its control's spread.
  The claim this proposal makes is therefore "not measurably different from `main`" there,
  not "faster" or "slower".
- The conversion counts on the two fixtures are the fixture's,
  not a prediction for another repository:
  1,578 of 24,389 (0.065) on the repository as it is and 15,731 (0.645) on the converting one.
- The earlier rounds' numbers that are still quoted are marked with the run they come from (`measure-r4-*`),
  and the `r3` run appears only as the record of a defect,
  never as an option's cost.

### The previous round's gate, at its final commit

Run as its own capped unit at `a690379aa5`
through `mise run //package/git-policy/cli:native:gate`
(the image build,
the compile,
the suite and Clippy of `native:test:container`),
with its verdict read through `native:gate:verdict`,
and its log kept as
`~/temp/agent/clone-rung-conversion-fence-gate-review-findings-fixed-2026-11-20/scratch/gate-logs/gpc-gate-fence-final-a690379aa5-20261120T192719-0500.log`:

- **9,303 tests run across 4 binaries:
  9,301 passed (5 slow),
  2 failed,
  28 skipped**,
  in 785 s.
- Passed there:
  the six new unit controls of the decision module.
  The fence's own module had four unit controls at that commit and has six at this branch's tip;
  an earlier version of this sentence claimed two new ones had passed there,
  which was false for that gate (the diff check's N3).
- Did **not** run there:
  the twelve binary controls of `worktree_clone_conversion`.
  Each printed `ZFS CONTROL NOT RUN` and returned before its add
  (`rg 'ZFS CONTROL NOT RUN'` finds 21 lines in that log,
  twelve of them these controls),
  because the image's fixture root is a tmpfs and the helper refuses without a guest named by
  `CLI_GIT_ZFS_FIXTURE_GUEST`.
  Their `PASS` lines say nothing about an add (rule `TRV`).
  This round ran them where they run (§ "The twelve class controls,
  in a guest").
- Failed there,
  both in `native_binary`,
  and neither in a module this round touched:
  `repository_naming_commit::a_commit_that_names_another_repository_lands_there_and_leaves_this_one_alone`
  (the fixture repository's `.git` held a `cli-git-lock-holders` directory the control did not expect:
  "the repository the command ran in gained nothing",
  left `("... ", [... "cli-git-lock-holders" ...])`,
  right `("...", [...])`),
  and
  `service_no_manager::a_commit_with_no_service_manager_stops_plainly_and_changes_nothing`.
  The earlier round failed the same two at `39e12fdbc3`-era code and read them as naming the host;
  this round did not re-run the gate at `74caa69e7a` to compare,
  so the honest statement is "the same two fail before and after this change,
  and neither is in the fence",
  not "they are pre-existing" as a proof.
- Clippy of the image did not run:
  the gate stops at the first failing step,
  and the suite failed first.
  The host's Clippy is clean for both configurations (`native:clippy` and `native:clippy:measurement`).
- The clone-capable controls do not run in that image (fixture root `Ok(Other)`,
  so "CONTROL NOT RUN" for every btrfs
  control,
  each naming `native:test:host:btrfs`),
  which is the gap the gate review's M5 named and this round states rather than closes.

### This round's gate

Run as its own capped `systemd-run --user --scope` unit
(`CPUQuota=800%`,
`MemoryMax=14G`,
`TasksMax=800`,
`IOWeight=10`)
through `node bin/test-native-container.ts` in `package/git-policy/cli`,
at the code of `21ad9d8ef8` plus the comment reflow committed after it,
with the image `localhost/git-policy-native-test:development`:

- **9,358 tests run across 4 binaries:
  9,356 passed (6 slow),
  2 failed,
  35 skipped**,
  in 801.055 s.
- The two failures are the repository's documented standing pair:
  `repository_naming_commit::a_commit_that_names_another_repository_lands_there_and_leaves_this_one_alone` and
  `service_no_manager::a_commit_with_no_service_manager_stops_plainly_and_changes_nothing`
  (`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`,
  "Cause H:
  the two standing failures").
  They fail the same way at `a690379aa5`-era code and before this round's code,
  and neither is in a module this round touched.
- The `source_guard` set ran in the same image on its own:
  **80 of 80 passed**,
  log kept as
  `~/temp/agent/clone-rung-conversion-fence-recheck-findings-fixed-2026-11-20/scratch/keep/source-guard-in-the-gate-image-80-tests.log`.
- The image did **not** run the twelve binary class controls (they print `ZFS CONTROL NOT RUN`,
  § above);
  their evidence is the guest run below.
- The image's Clippy step did not run because the suite failed first;
  the host's Clippy is clean for both configurations this round
  (`native:clippy` and `native:clippy:measurement`).

### The twelve class controls, in a guest

One throwaway `libvirt` guest through `mvm` (`crf-class-controls`),
made by `package/zfs/request-to-openzfs-kernel-device`'s recipe and destroyed by the run that made it:
Ubuntu 24.04 on kernel 6.8.0-147-generic,
Git 2.43.0 as the reference checkout,
OpenZFS 2.4.4 built against that kernel,
4 cores and 7 GiB.
The recipe's step 3 pool stood on a 1 GiB encrypted file image
(`aes-256-gcm`,
raw key,
`recordsize=32K`,
`compression=zstd-3`),
its `checkout` dataset the fixture root,
so every control's add was served by the clone rung on a real ZFS dataset.
The crate was built inside the guest (`cargo build --release --tests --locked --jobs 2`,
9 min 27 s)
because the host's glibc is newer than the guest's.

**`worktree_clone_conversion`:
18 of 18 passed**,
run one at a time
(`--test native_binary --test-threads 1`),
the log kept as
`~/temp/agent/clone-rung-conversion-fence-recheck-findings-fixed-2026-11-20/scratch/keep/guest-zfs-conversion-suite-18-of-18.log`:

`a_config_rule_gone_after_the_add_is_written_by_git`,
`a_converting_count_at_the_floor_keeps_the_clone_rung`,
`a_converting_count_one_past_the_floor_declines_the_rung`,
`a_converting_share_of_exactly_a_quarter_keeps_the_clone_rung`,
`a_converting_share_one_path_past_a_quarter_declines_the_rung`,
`a_core_autocrlf_key_written_without_a_value_is_written_by_git`,
`a_crlf_file_added_under_text_auto_eol_lf_is_written_by_git`,
`a_legacy_crlf_attribute_beside_core_eol_crlf_is_written_by_git`,
`a_legacy_crlf_attribute_over_an_added_crlf_file_is_written_by_git`,
`a_literal_text_unset_value_under_core_autocrlf_true_is_written_by_git`,
`a_text_attribute_asking_for_crlf_over_lf_files_is_written_by_git`,
`a_tracked_attributes_file_a_later_commit_deleted_is_written_by_git`,
`a_working_tree_encoding_carries_the_sources_bytes_as_gits_checkout_does`,
`an_eol_crlf_attribute_without_text_is_written_by_git`,
`an_eol_lf_attribute_without_text_is_written_by_git`,
`an_ident_attribute_over_an_unexpanded_keyword_is_written_by_git`,
`an_ignored_untracked_attributes_file_sends_the_add_to_gits_checkout`,
`core_autocrlf_true_set_after_the_files_were_written_is_written_by_git`.

Six of them could not have passed anywhere before this run:
they passed `--no-enforce-bulk-add` to native Git,
which only the wrapper knows,
and their `git commit` after a converted add had nothing to commit and no `--allow-empty`.
Both were test defects,
fixed before this run,
and the two the review named (`p01`,
`p02`) are among the eighteen.

The same guest then made a 6 GiB btrfs image,
mounted it at `/mnt/bt`,
and ran this round's ladder module on it,
because no host here and no gate image is btrfs:
**66 of 66 passed**,
including
`worktree_ladder_run::tests::btrfs_a_clean_ordinary_source_gets_the_clone_rung_and_a_subvolume`,
`btrfs_a_dirty_subvolume_source_is_snapshotted_and_a_merging_one_keeps_its_reason` and
`worktree_ladder_finish::tests::btrfs_the_clone_rung_writes_nothing_when_the_registration_was_replaced`,
which are the clone rung's own controls for the decision this round moved into `choose_rung`;
the log is
`~/temp/agent/clone-rung-conversion-fence-recheck-findings-fixed-2026-11-20/scratch/keep/guest-btrfs-ladder-66-of-66.log`.
A control that could not reach its fixture would have printed `BTRFS CONTROL NOT RUN`,
and the run refuses that line,
so the 66 are 66 that ran.

## The round that built option B

This round is the one after the diff check of the previous round on Sonnet
(`~/temp/agent/gate-diff-check-of-the-clone-conversion-fence-costs-on-sonnet-2026-11-20/keep/review.md`).
It built option B,
which the project asked for first,
and it fixed the sentences that diff check found wrong.
Everything it measured is kept under
`~/temp/agent/clone-rung-conversion-fence-beside-the-clone-2026-11-20/scratch/keep/`.

### What the diff check found, and what this round did

The diff check's verdict was "not fit to put to the project as it stands;
what is left is a few sentences",
and every finding it raised is closed here:

- **N1 (high),
  a section that said no shape is slower than `main`.**
  The waiver section and the retrospective bullet now state the costs above `main` as costs:
  the declining add at `main` plus the fence's reads (0.07 to 0.11 of `main`),
  the 0.199 band at 1.13,
  and option B's decline paying the copy as well.
  Whether the project asks for a waiver is left to the project.
- **N2/M3 (medium),
  the false reason for not building the in-process reads.**
  Both places now say what the body said:
  the repository has a pack reader,
  a size-only method on it is new code,
  and neither read was built or measured.
  The claim that the object reader "cannot walk" this repository's packs is retracted.
- **N3 (low),
  two fence unit controls that did not exist at that gate.**
  The section is retitled as the previous round's gate,
  and it says the module had four unit controls at that commit and has six at the tip.
- **N4 (low),
  prices quoted from the wrong run.**
  The shipped constant's comment,
  the btrfs proposal's proposed text and the retrospective now give the range the kept runs give
  (0.31 to 0.57 ms per LFS path),
  and the comment no longer calls the quarter a break-even point.
- **N5 (low),
  the earlier fixture's converting counts.**
  The counts are this fixture's:
  3,288 `.md` plus 1,578 LFS paths is 4,866 of 24,426,
  and the converting fixture's 15,743 of 24,426 is 0.644.
- **N6 (low),
  two sentences about A′ against C.**
  A′ against C is "not separable" in both places,
  the control's whole span is printed (0.93 to 2.16 and 0.78 to 1.03),
  and the mixed "0.97 to 1.08" is gone.
- **N8 (low),
  the person is not told why a decline happened,
  and a dead route note.**
  Fixed in code:
  a clone rung that ends now sets the slow route from its own reason
  (`explain_clone_failure`,
  and `explain_decline` for a decline decided before any copy),
  so the person reads why the add took Git's checkout,
  in the clone rung's words and never a snapshot's;
  the dead `note_timing` line is deleted.
- **N9 (low),
  one partial-clone figure.**
  The 0.97 to 1.08 ceiling is withdrawn,
  because no kept run measured a no-LFS no-rule source.
- **N10 (low),
  no kept log for the whole container gate.**
  This round's gate log is kept and named under "The runs,
  and this round's gate".
- **N11 (low),
  a `-text` exemption the text claimed under `core.autocrlf=true`.**
  The bullet now says the exemption holds only where nothing converts,
  that under `core.autocrlf=true` a `-text` path is written by Git and counts toward the decline share,
  and that what that costs is not priced.

### The ordering, red first

The wrong ordering the option exists to avoid is a path recorded clean whose bytes Git would write differently.
The control `the_installed_index_vouches_for_no_path_git_still_has_to_write`
(`worktree_clone_conversion_beside_the_clone_tests.rs`) builds a repository whose `*.txt` rule makes Git's checkout
write carriage returns,
copies the source's files into a `--no-checkout` worktree with the source's times,
installs the index the way the shipped order does,
and asserts two things about the written path before Git's write:
its entry holds no stat data,
and the bytes on disk are not the ones Git's checkout writes for the same commit
(so a stat that vouched for them would have been a lie).
It then runs the write step and asserts the bytes equal Git's and the status is clean.

Run with the seed's zeroing planted out
(`seed_without_stat` passing an empty set,
which is the fence-before-the-clone order's index),
it fails and names the harm:
`it records 18 and Git writes 20 bytes`
(`red-planted-ordering-control.log`).
With the seed as shipped it passes
(`green-ordering-control.log`),
and the plant patch is kept as `red-plant-seed-unzeroed.patch`.

Three more unit controls stand beside it:
`the_seed_zeroes_the_stat_data_of_the_path_the_fence_leaves_to_git` reads the published seed entry by entry,
`the_fence_beside_the_clone_answers_what_the_fence_before_it_answers` runs the same fence as the work beside the
copies and compares its answer,
and the seed built from it,
with the caller's,
and `the_slices_answer_beside_a_job_that_runs_with_the_copies` drives the sliced copy's start-and-wait with a job
beside it and reads the index installed afterwards.
The class controls' fixtures are below the clone's break-even,
so they take the one-thread path;
this control and the whole-repository runs are what reach the concurrent order.

### The class controls

The eighteen committed binary controls of `worktree_clone_conversion` (the twelve classes,
the two spellings fixed last
round,
and the four decline boundaries) ran in a throwaway `libvirt` guest through `mvm`
(their fixtures have three paths each,
so they exercise the one-thread order,
the same fence and the same seed the concurrent order uses,
not the concurrent start)
(`crf-b-controls`,
created by the recipe of `package/zfs/request-to-openzfs-kernel-device`,
Ubuntu 24.04 on 6.8.0-147-generic,
Git 2.43.0,
OpenZFS 2.4.4 on a 1 GiB encrypted file-backed pool with block cloning on),
where the crate was built inside the guest and the controls ran one at a time:
**18 of 18 passed**
(`guest-zfs-conversion-suite-18-of-18.log`).
The same guest then made a 6 GiB btrfs image and ran the ladder's and the clone module's own controls on it:
**62 of 62 passed**,
including `btrfs_the_slices_clone_every_entry_of_a_large_run`,
`the_slices_answer_beside_a_job_that_runs_with_the_copies` and the other beside-the-clone unit controls
(`guest-btrfs-ladder-and-clone-suite-62-of-62.log`).
The first btrfs run there named every module whose name begins `worktree_clone`,
189 tests,
and 187 passed:
the two failures are the clone-mount store controls for which root may rewrite a store its owner may not.
The 62 are the ladder's and the clone's own modules by name;
the 189-test run also carried `worktree_clone_mount`,
whose two store controls fail as root,
and the earlier guest's run of the pre-fix code named the same two the same way
(`btrfs-suite.log` in the round's `guest-logs/`).
The diff check's own harness (`classes.ts`,
its copy kept here) ran twenty-six scenarios against the built binary on this host:
**26 EQUAL**,
23 served by the clone rung,
2 refused before any copy by an untracked `.gitattributes` (`c06`,
`c06b`),
and one (`m05`) ended at the tree difference as the option's known regression describes
(`class-records-beside-346ec0bec0.json`,
`class-stderr-beside-346ec0bec0-*.log`,
run against the final code;
the earlier run's records are kept beside them).

The re-check's 150-combination sample (seed 11) ran here too,
from the diff check's kept harness (`matrix.ts`) against the same built binary:
**300 repositories,
300 EQUAL,
300 served by the clone rung**
(150 combinations,
each in the two states the harness uses:
the rules in place when the files were added and the rules set after the add),
in `matrix-beside-346ec0bec0-seed11-n150.json`
(the earlier run's `matrix-beside-seed11-n150.json` is kept beside it).

### What went wrong in this round, and what caught it

- The first build of option B started the fence on a thread of its own.
  The container gate's `source_guard` set refused it by name
  (`worktree/worktree_clone_conversion_beside_the_clone.rs:94 [thread-start] starts a thread; work the runtime can carry is
  a task, and work it cannot carry is a job of the blocking pool`),
  and the same gate run found that the standing notice's table had no sentence for the new step
  (`a step an add enters has no sentence of its own: ["conversionBesideTheClone"]`).
  The gate's first log is kept as `container-gate-beside-db5bc6818b.log`.
- The first fix moved the fence onto the blocking pool,
  and the measurement run that followed showed every served add
  ending at the tree difference:
  `a Git run of the fence could not start (the runtime that runs this command's waits was asked for from inside itself…)`.
  The synchronous Git face enters the runtime around the pipes it reads,
  and a blocking-pool thread carries the runtime's handle,
  so it is refused there as it is inside a task.
  The shipped shape therefore starts the copies as one task of the runtime and runs the fence on the add's own thread,
  which is otherwise waiting;
  that run's numbers are kept and superseded (`measure-beside-real-real-…T05-27-11` and `-md-md-…T05-34-38`).
- The first btrfs guest run failed `btrfs_the_slices_clone_every_entry_of_a_large_run`,
  whose assertion counted the `MANY_FILES` constant while the fixture tracks that many files plus a `.gitignore`.
  The control now counts the source's index (41 paths at that fixture).
  The same run's two clone-mount store controls fail as root,
  which is the account the guest suite runs as,
  and they were excluded by naming only the modules this round touches.
- - Three timing batches were run.
  The first (`measure-beside-real-real-…T04-26-36`,
    `-md-md-…T04-32-09`,
    `-crlf-crlf-…T04-39-00`)
  measured the thread version under another session's load,
  with the control's disagreement reaching 44 percent on this repository;
  the second (`…T05-27-11`,
    `…T05-34-38`,
    and the `-crlf` run this round stopped)
  measured the pool version,
    whose served adds all ended at the tree difference;
  the third (`…T05-45-56`,
    `…T05-51-05`,
    `…T05-56-56`) measures the shipped code and is what this proposal quotes.
- The first driver run's guest build named no `package/zfs/request-to-openzfs-kernel-device`,
  which the wrapper depends on by path,
  and failed before compiling;
  the second's guest had no `HOME`,
  which the socket-folder fixture needs;
  both are recorded in the round's `guest-logs/` and fixed in the driver.
- Two shell slips of mine are recorded rather than hidden:
  one command chained with `;` where `AGENTS.md` asks for one script,
  and one long command was not put in a scope of its own.

### The runs, and this round's gate

The three timing runs are "Option B beside the clone,
this round's runs":
five counted rounds on this repository,
seven on the 0.199 source and seven on the 0.644 source,
with the identical-binary control beside B in every one,
and their numbers are in that section and in the options'.
Each run's JSON and summary are kept as
`measure-beside-<label>-<mode>-<stamp>.json` and `.summary.txt` in this round's `keep/`,
beside `run-measurements.ts`,
`measure-beside-the-clone.ts` and `stats-beside.ts`,
so every number above can be recomputed.

The whole container gate ran at the code of `d4963cfb16` as its own capped `systemd-run --user --scope` unit
(`CPUQuota=800%`,
`MemoryMax=14G`,
`TasksMax=800`,
`IOWeight=10`),
through `node bin/test-native-container.ts` in `package/git-policy/cli`,
with the image `localhost/git-policy-native-test:development`,
and its log is kept as `container-gate-beside-d4963cfb16.log` in this round's `keep/`:

- **9,409 tests run:
  9,407 passed (6 slow),
  2 failed,
  35 skipped**,
  in 813.349 s.
  The same gate ran at `a95b3e2eb3` before the clone rung's serving step moved,
  with the same pair failing and the same counts (`container-gate-beside-a95b3e2eb3.log`).
- The two failures are the repository's documented standing pair,
  `repository_naming_commit::a_commit_that_names_another_repository_lands_there_and_leaves_this_one_alone` and
  `service_no_manager::a_commit_with_no_service_manager_stops_plainly_and_changes_nothing`
  (`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`,
  "Cause H:
  the two standing failures").
- The whole `source_guard` set ran in the same image and passed there:
  80 distinct controls,
  among them `the_release_code_keeps_blocking_work_off_the_runtime` and
  `every_step_an_add_enters_has_its_own_sentence`,
  which the first gate of this round refused.
  It passed on the host too,
  80 of 80,
  in `source-guard-host-346ec0bec0.log`.
- The image's `Clippy` step did not run,
  because the suite failed first;
  the host's Clippy is clean for both configurations (`native:clippy` and `native:clippy:measurement`).
- The image did not run the eighteen binary class controls (their fixture root is a tmpfs and they print
  `ZFS CONTROL NOT RUN` there);
  their evidence is the guest run below,
  on the final code.

## The round that decided the decline before the copying

This round is the one after the report on option B
(`~/temp/agent/clone-rung-conversion-fence-beside-the-clone-2026-11-20/keep/report-from-summary.md`).
That report found B right on every case it checked
(18 of 18 guest conversion controls,
62 of 62 btrfs controls,
300 of 300 sampled combinations)
and wrong about one thing where it resolved:
**where the fence declines,
B cost 1.67 to 1.75 of `main`,
because the decline was decided after the copies,
so a declined add paid the copy and then Git's whole checkout over it.**
A′ decided the same decline from `git check-attr` alone,
before anything was copied,
at 1.06 to 1.09.
This round did three things:
decided the decline in `choose_rung` before the copies,
gave rung 1 the same fence and the same ordering rule before its index install,
and split `worktree_clone.rs`,
which was over `rust/max-lines`.
Everything it measured and every log it kept is under
`~/temp/agent/clone-rung-conversion-fence-b-decides-before-copying-2026-11-21/scratch/keep/`.

### The new first step

`the_fence_of_this_build` splits the fence in two
(`worktree_clone_conversion_beside_the_clone.rs`):

- `fence_the_attributes` reads the source's attributes
  (the untracked-attributes refusal,
  one `git check-attr --all -z --stdin`,
  the conversion rules,
  the count of the paths a rule of their own would convert)
  and decides the quarter rule;
  a decline or a refusal is the caller's signal to take Git's checkout,
  and it now comes before the add's record is claimed and before real Git runs.
- `decide_the_paths` runs what the attributes left beside the copies:
  one `git ls-tree -r -l` and the per-path decision.
- Option A′ and the measurement-only options C and D are the routes that decide the whole fence before the copies,
  unchanged in what they read.
  The shipped build holds none of them (`strings` finds no `MONOCHROMATIC_CLONE_FENCE`).

The safety argument is the one in "Option B:
the order beside the clone",
with its first step replaced:
step 1 writes nothing and claims nothing,
so a kill there leaves the add as Git's own `worktree add` left it;
the claim,
the copies and the seed follow it.

### Rung 1's fence, and its ordering rule

`choose_rung` now decides rung 1's whole fence before the snapshot,
and `install_index` installs a seed whose written entries hold no stat data,
so the ordering rule ("no path recorded clean before its bytes are Git's") holds at the moment the index appears.
The write is `checkout_index_through_git`,
the clone rung's own step.
See "How the fence applies to rung 1,
the snapshot" for the argument and what it costs.

Two reds,
each with the previous order planted:

- `the_index_the_snapshot_installs_vouches_for_no_path_the_fence_leaves_to_git`
  (`worktree_snapshot_index_step_tests.rs`),
  run with the install ignoring the fence's paths:
  red,
  `the path the fence leaves to Git records 18 bytes, so a status would skip it while the file holds 18 and Git's
  checkout writes 20 bytes`
  (`red-snapshot-index-control.log`,
  plant `red-plant-snapshot-index-unzeroed.patch`);
  with the shipped install,
  green (`green-snapshot-index-control.log`).
- `a_converting_source_is_declined_before_anything_is_claimed_or_copied`
  (`worktree_ladder_run_tests.rs`),
  run with `declines` planted to answer false:
  red,
  the rung becomes `WithoutCheckout(2)` and the control names the person's own run it expected
  (`red-decline-before-the-copy-control.log`,
  plant `red-plant-no-decline.patch`);
  with the rule as shipped,
  green (`green-decline-before-the-copy-control.log`).

The clone rung's own ordering control
(`the_installed_index_vouches_for_no_path_git_still_has_to_write`)
still passes with this round's shape,
because the seed and the write step are the ones it was built for.

### The controls, by name

In one throwaway `libvirt` guest through `mvm`
(`crf-b-decides-2`,
created by the recipe of `package/zfs/request-to-openzfs-kernel-device`,
Ubuntu 24.04 on 6.8.0-147-generic,
Git 2.43.0,
OpenZFS 2.4.4 on a 1 GiB encrypted file-backed pool with block cloning on),
with the crate built inside the guest:

- **rung 2 on ZFS**:
  the eighteen committed binary controls of `worktree_clone_conversion`
  (the twelve classes,
  the two spellings fixed in the previous round,
  `a_core_autocrlf_key_written_without_a_value_is_written_by_git` and
  `a_literal_text_unset_value_under_core_autocrlf_true_is_written_by_git`,
  and the four decline boundaries),
  run one at a time:
  **18 of 18 passed** (`conversion-suite.log`).
- **rung 1 and rung 2 on a 6 GiB btrfs image**:
  the ladder's,
  the clone's,
  the snapshot rung's and the dirty restore's own controls,
  176 tests,
  of which the ones this round changed are
  `btrfs_a_converting_path_is_written_by_git_and_the_snapshot_rung_is_clean`,
  `btrfs_a_configuration_rule_writes_gits_bytes_in_the_snapshot_rung`,
  `a_converting_source_is_declined_before_anything_is_claimed_or_copied` and
  `the_fence_beside_the_clone_answers_what_the_fence_before_it_answers`:
  **176 of 176 passed**,
  with no `BTRFS CONTROL NOT RUN` line
  (`btrfs-suite-rerun.log`).
  The first run of that suite was 265 tests with four failures:
  the two controls this round wrote for the new behaviour
  (both wrong about how the rung shows what it did,
  fixed in `1e4789b8c7`) and the two clone-mount store controls that cannot fail as root
  (the standing limitation the previous round named).

On this host,
against the built binary `b`:

- the class harness's twenty-six scenarios,
  the author's eight classes plus the review's `m` and `p` cases:
  **26 EQUAL**,
  23 served by the clone rung,
  two refused before any copy (`c06`,
  `c06b`),
  and `m05` ended at the tree difference
  (`class-records-decides-57c6bae080.json`).
- the re-check's 150-combination sample,
  seed 11,
  150 of 1,440 attribute-and-setting combinations:
  **300 repositories,
  300 EQUAL,
  300 served by the clone rung**
  (`matrix-decides-57c6bae080.json`).

### The runs, and this round's gate

The three timing runs are this round's runs section,
with their JSON and summary files kept as
`measure-decides-<label>-<mode>-<stamp>.json` and `.summary.txt`,
beside `measure-fence.ts` and `run-measurements.ts`,
so every number above can be recomputed.

The whole container gate ran twice.
The first,
at `7900f47c5c`,
**failed one control**,
`a_converting_source_is_declined_before_anything_is_claimed_or_copied`,
and its message was lost because the gate's own output was piped through `tail`
(a mistake of this round's):
the gate image's fixture root is a tmpfs whose filesystem shares no data blocks,
so the clone rung refuses the rung *before* the fence runs,
and the control's decline assertion could never hold there.
The control now names its own skip on such a filesystem,
the way the btrfs controls do,
and the second gate passes it,
printing the reason it skipped inside the image:
`/work/package/git-policy/cli/target/gate-fixtures/process-152295/… would receive the clones and its filesystem does
not share data blocks between files`
(`container-gate-decides-737c28846f.log`).
The control's real runs are the host's ZFS run and the guest's btrfs image.

The second gate ran at `737c28846f` as its own capped `systemd-run --user --scope` unit
(`CPUQuota=800%`,
`MemoryMax=14G`,
`TasksMax=800`,
`IOWeight=10`,
`RuntimeMaxSec=3600`),
through `node bin/test-native-container.ts` in `package/git-policy/cli`,
and its log is kept as `container-gate-decides-737c28846f.log`:

- **9,472 tests run:
  9,470 passed (5 slow),
  2 failed,
  35 skipped**,
  in 761.774 s.
- The two failures are the repository's documented standing pair,
  `repository_naming_commit::a_commit_that_names_another_repository_lands_there_and_leaves_this_one_alone` and
  `service_no_manager::a_commit_with_no_service_manager_stops_plainly_and_changes_nothing`
  (`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`,
  "Cause H:
  the two standing failures").
- The whole `source_guard` set ran in the image:
  80 distinct controls passed there,
  among them `the_release_code_keeps_blocking_work_off_the_runtime`
  (the copies still start as a task and the fence's Git face still runs outside the runtime)
  and `every_step_an_add_enters_has_its_own_sentence`.
- The whole container gate is the authority for Clippy;
  its Clippy step runs the locked toolchain and did not report a finding,
  and the host's own `native:clippy` and `native:clippy:measurement` are clean.

## Limitations

Each line is a case where option B,
which the project has chosen
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
step 10's notes,
the project's 2026-11-20T23:27:00-05:00
word "Yeah,
do B."),
is still not completely right,
with the evidence behind it.

- **A declining add is about 10 percent slower than `main` on a source that declines.**
  The decline now costs one `git check-attr` and the fence's decision,
  but the ladder's own clean check runs before them,
  and the add's own `gitAdd` is `main`'s:
  on the 0.644 source B measured 1.10 of `main`
  (1.02 to 1.20 across seven rounds;
  its identical control 1.10,
  and the pair agreeing at 0.95 to 1.05),
  of which 73 ms is the clean check,
  33 ms the attribute read and 21 ms the decision.
  A′ measures the same 1.10 in this run,
  so this is the price of declining at all,
  not of the order.
- **A source between this repository's converting share and the quarter is not declined.**
  At 0.199 the fence writes 4,866 paths through Git,
  and the add costs 1.01 to 1.04 of `main`:
  B against `main` is 0.98 to 1.08 across seven rounds,
  and its identical pair agrees at 0.86 to 1.00,
  so the add is at best `main`'s time and this is the band the quarter leaves.
- **An add whose new `HEAD` differs in a path Git must write can end the clone rung.**
  `git read-tree -m -u` refuses a zeroed entry whose clean form is not its blob
  (`error: Entry 'extra.txt' not uptodate. Cannot merge.`,
  the harness's `m05`,
  which ends the rung and leaves bytes equal to Git's),
  so the bytes are right but the add pays the copy and Git's checkout,
  where option A′ would finish the rung.
- **Rung 1's fence reads sit on the add's waiting path and are not timed at this repository's size.**
  A snapshot copies everything anyway,
  so there is no second thread for the fence's reads to run beside;
  and the guest's btrfs fixtures are a few paths,
  so no kept run prices a 24,435-file snapshot add with the fence,
  nor a decline there.
  The two conversions the guest does prove are `btrfs_a_converting_path_is_written_by_git_and_the_snapshot_rung_is_clean`
  and `btrfs_a_configuration_rule_writes_gits_bytes_in_the_snapshot_rung`,
  and the ordering rule behind them is
  `the_index_the_snapshot_installs_vouches_for_no_path_the_fence_leaves_to_git`.
- **An add killed between the index install and the write step leaves a worktree whose written paths report as changed.**
  That is the safe direction (nothing is vouched for and `git status` reads the bytes),
  but it is not a clean worktree,
  and the recovery is the one the add's record already names.
  Rung 1 is in the same state between its install and its checkout step.
- **A path with the `-text` attribute under `core.autocrlf=true` is written by Git.**
  The exemption the class table describes holds only where nothing converts,
  so such a repository's binaries count toward
  the decline share,
  and that cost is not priced.
- **The LFS paths are copied and then overwritten.**
  The clone copies this repository's 1,578 LFS files and Git's smudge writes them again;
  option A′ copies them too,
  so it is not a difference between the options,
  but it is a cost neither avoids.
- **The fence's remaining read still runs on the add's thread in B.**
  Only the copies' wait is overlapped.
  Inside the beside window the add's thread runs `git ls-tree`,
  267 ms on this repository under the copies' own load,
  and then the per-path decision,
  while the copies' wait is 524 to 656 ms beside A′'s 752,
  and the totals are inside the identical pair's disagreement (1.01 against A′).
  What this round removed from that thread is the attribute read and the decline,
  which now run before anything is copied and before Git's own run.
  No in-process replacement is built or measured.
- **The `attr.tree` setting is still not refused by the shipped code.**
  `git check-attr` honours it and so would the destination,
  so the two agree,
  but the refusal the proposed `SPEC.md` text
  requires is not in `prepare_target`.
- **A repository with no conversion rules,
  XFS and overlayfs are unmeasured for B.**
  The no-rule case is arithmetic on measured steps,
  and no B run exists on either filesystem.
- **A clone rung that ends after its copies tells the person in the clone rung's words.**
  That is better than the snapshot sentence it replaced,
  and a decline decided before the copies is told as `explain_decline`,
  but a failure of the fence's remaining reads still reads like a decline in one line,
  and the difference is only in the journal's detail.

## What this proposal does not do

- It does not time rung 1 at this repository's size.
  The ordering rule is built and controlled
  (the btrfs control named in "How the fence applies to rung 1",
  and the index control behind it),
  but the guest's btrfs fixtures are a few paths,
  so no kept run prices a 24,435-file snapshot add with the fence,
  nor a decline there.
  The rung's reads are the clone rung's reads (35 to 48 ms and 190 to 345 ms),
  and that is arithmetic on another rung's measurement,
  not this rung's.
- It does not build the in-process attribute and object reads.
  The claim that they would remove the fence's 239 ms is removed from this proposal,
  and the reason the earlier draft gave for their being a large change is retracted:
  the repository has a pack reader,
  a size-only method on it is new code,
  and neither was built or measured.
- It does not build the "run the tree difference against a copy of the index that still has its stat data" fix
  for the differing-tree case,
  which is option B's one known roteness regression against A′
  (the `m05` case in "Why each step is safe").
  Rung 1 shares the exposure.
- It does not run the clone-capable controls in the gate image,
  whose fixture root is neither btrfs nor ZFS:
  the image runs the unit controls and the binary controls that do not need a snapshot-capable filesystem,
  and the btrfs and ZFS-gated controls print `CONTROL NOT RUN`.
  This round ran the conversion class controls in a guest instead
  (§ "The round that decided the decline before the copying").
- It does not repeat the guest's adds:
  one per arm per scenario.
- It does not measure a repository with no conversion rules at all,
  nor `core.autocrlf=true` with no attributes,
  nor XFS or overlayfs.
- It does not prove the LFS case cheaply:
  it pays Git's smudge,
  and the partial-clone section says what that means.
- It does not touch `doc/handover/`.
