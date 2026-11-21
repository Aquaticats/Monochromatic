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
  which the configuration's `core.eol`,
  or `core.autocrlf` where `core.eol` is unset on a Unix host,
  turns into the input or the carriage-return ending;
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
  nothing cheaper proves it,
  because a pointer and its content are different lengths and a content file has no marker.
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
3. the decline rule,
   from the attributes alone and before any blob is read:
   the number of paths whose own rule converts them (`checkout_converts`)
   is compared with a quarter of the tracked paths;
   past that share and past a thousand paths,
   the rung ends with the count and Git's checkout writes the worktree,
   which is the one shape that keeps this fence from ever being slower than `main`.
   Nothing is cloned before this answer,
   so a declined add costs the attribute read and Git's checkout.
4. `git ls-tree -r -l -z <HEAD>`:
   one run,
   for the blob sizes the size proof needs,
   which is every path the fence may carry (see the no-rule class above).
5. the decision per entry,
   from `stored_stats(source.clean.index_bytes, …)`:
   the entry's recorded size,
   its mode and
   the attributes.
6. `git update-index -z --index-info` over the fenced paths the destination's index holds,
   to zero their instance data,
   then `git checkout-index --force -u -z --stdin`,
   after the tree difference and only when the fence left paths to Git,
   in the
   destination with `GIT_DIR`/`GIT_WORK_TREE` as the ladder's other runs set them,
   so the destination's own conversion
   writes them and their stat data is recorded (`-u`).
   The intersection with the destination's index is a correctness rule:
   `--index-info` adds an entry a path lacks,
   and a fenced path the new `HEAD` does not carry would otherwise be injected and fail the run.

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

Two runs of this round,
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
24,389 tracked files,
14,153 of them `.md` or `.ts`.
The two modes differ in one thing:
the "converting" one appends `*.md text eol=crlf` and `*.ts text eol=crlf` to the attributes,
and sets no `core.autocrlf`,
so the difference is the attributes alone
(the earlier round's converting fixture also set `core.autocrlf=true`,
which the repository's own `* text=auto eol=lf` overrode for every path;
the gate review's L4).

Rule `QNB`'s spread came first in each run:
one arm alone on unchanged input,
before any arm was compared with another.
Every run went in its own capped `systemd-run` scope
(`CPUQuota=200%`,
`MemoryMax=4G`,
`TasksMax=400`,
`IOWeight=1`),
on this host as other sessions left it:
load average 8 to 13 at the end of each run,
which is why every per-round ratio is printed beside the identical-binary control's own.
Arms rotated one position per round,
`sync --file-system` and 300 ms between arms,
one warm-up round,
and every arm's worktree compared with `main`'s **after** the counted rounds,
outside the timed window,
because the LFS content makes each comparison a read of the whole fixture.

Binaries,
each tied to its commit and kept under
`~/temp/agent/clone-rung-conversion-fence-gate-review-findings-fixed-2026-11-20/scratch/keep/`:
the fence at `527b890d3e` (sha256 `b7ddcb21…`,
the shipped build),
its measurement build with the same code plus `measurement-routes` (sha256 `96587a9e…`),
step 3 at `74caa69e7a` (sha256 `8077e6f9…`),
`main` at `4997e2a4cb` (sha256 `20a3e00b…`).
Every round's JSON and step split:
`measure-r5b-real-*` and `measure-r5-crlf-*` in that folder.

### This repository as it is, conversion rules and LFS content together

One warm-up and five counted rounds,
four arms,
the fence here **does not decline**
(1,578 of 24,389 paths convert,
0.065,
and 1,578 is the LFS count):

- spread,
  one arm on unchanged input:
  1,999,
  2,200,
  3,561 ms,
  median 2,200
  (the first round is the cold one,
  straight after the fixture was written)
- the fence,
  option A:
  **2,395 ms** (1,835 to 6,714),
  1.11 of `main` per round [0.80 1.11 2.16 2.90 1.04]
- the same binary as its control:
  2,323 ms (1,952 to 4,948),
  1.03 of `main` [0.85 1.03 2.22 1.00 1.71];
  control over fence per round 1.06,
  0.93,
  1.03,
  0.35,
  1.64,
  **median 1.03**
- step 3:
  1,356 ms (1,076 to 1,466),
  0.59 of `main`
- `main`'s Git checkout:
  2,286 ms (1,981 to 2,314)
- every arm's worktree:
  `git status` clean and zero lines of `diff` against `main`'s.

The fence's own steps (medians):
`git check-attr` 36.5 ms,
`git ls-tree` 202.8,
`git update-index` 16.7,
`git checkout-index` 905.5,
the clone 847.5,
the clean check 69.6.
The identical-binary control's `git checkout-index`,
the same 1,578 paths,
was 642.0 ms.

What this says:
on this repository as it is,
the fence is **inside its own control's spread of `main`**.
Its median is 0.08 of `main` above `main`,
and its control's per-round ratio to `main` ranged 0.35 to 1.64 on the same rounds,
so no difference between the fence and `main` is claimed here.
What is measured and resolvable:
the fence is 1.04 to 1.11 times `main` on this workload,
step 3 is 0.59 of `main` and carries 1,578 LFS paths unproven,
and the write of those 1,578 paths through Git's smudge costs 0.64 to 0.91 s
(0.41 to 0.57 ms per path),
which is the price of not being able to prove a smudged file.

### A source whose attributes convert most of its paths

One warm-up and five counted rounds,
six arms,
15,731 of 24,389 paths convert here (0.645:
the LFS 1,578 plus the 14,153 `.md` and `.ts`),
so the decline rule fires:

- spread,
  one arm on unchanged input:
  2,218,
  2,302,
  4,215 ms,
  median 2,302
- the fence with its decline rule,
  option A′ (the shipped shape):
  **2,468 ms** (2,155 to 2,602),
  **1.09 of `main`** [1.09 1.16 1.05 1.17 1.05],
  route "Git's checkout after the clone rung ended"
- its identical-binary control:
  2,439 ms (2,207 to 2,654),
  1.04 of `main`;
  control over fence 1.13,
  0.89,
  0.97,
  1.02,
  0.99,
  median 0.99
- the fence **without** the decline rule,
  option A (`measurement-routes` only,
  route `nodecline`):
  **3,192 ms** (3,013 to 3,725),
  **1.50 of `main`** [1.58 1.41 1.50 1.68 1.35]
- the whole-source refusal,
  option C:
  2,282 ms (2,136 to 2,776),
  1.06 of `main`
- step 3:
  1,097 ms (1,000 to 1,221),
  0.50 of `main`,
  and **wrong:
  7,097,444 lines of `diff` against `main`'s worktree**
- `main`'s Git checkout:
  2,222 ms (1,972 to 2,361)
- every arm but step 3:
  `git status` clean,
  zero lines of `diff` against `main`'s.

The steps:
`git check-attr` 36.2,
`git ls-tree` 193.9,
`git checkout-index` 1,788.7 for option A (15,731 written paths,
0.114 ms each),
`finishRung` 2,036.3 for the declining arms (Git's `reset --hard` writes the tree),
`main`'s `gitAdd` 2,023.2.

What this says:
the shape that was 1.28 times `main` in the earlier round is 1.50 times `main` here,
and the decline rule,
on the same fixture and in the same rounds,
brings it to 1.09 with a control of 0.99,
so **no shipped shape is slower than `main` any more**.
The decline costs 36 ms of `git check-attr` plus the decision loop over the attributes,
which is why A′ is 0.09 of `main` above C:
C refuses on the configuration and the attribute files alone and never asks Git what the per-path attributes are.

### The decline rule's threshold, from these numbers

Option A's total is its work without writes plus 0.114 ms for every written path:
3,192 − 1,788.7 = 1,403 ms of everything else,
against `main` 2,222.
So A stays below `main` while Git writes fewer than (2,222 − 1,403) / 0.114 ≈ **7,175 paths**,
which is 0.294 of 24,389.
The rule declines past a quarter of the tracked paths and past 1,000 of them:
a quarter is under the 0.294 the arithmetic gives,
and 1,000 paths (114 ms of writing) is inside the run-to-run spread,
so a small repository with a converting rule keeps its clone rung
and this round's own class controls,
four paths each,
still reach the fence.

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
and the reading that follows is why it is not the small change the draft assumed:
this repository's largest pack holds 499,083 objects of which 394,112 are deltified,
with chains up to 50 deep (`git verify-pack -v` on its index),
so one size per path means reading the pack index,
following the delta header,
and resolving the chain,
and `repo_read_object.rs` today reads loose objects and zlib streams only,
with no pack index and no delta code.
For scale,
the measured in-process walk of the same 24,389 paths that does one `lstat` each
(the clean check's own `checkSource` step) is 69.6 ms in the same records.
The claim that in-process reads "remove the 236 ms" is therefore **removed**,
and the honest statement is:
the two runs cost 239 ms here,
and what an in-process reader would cost is unmeasured
until someone builds one with pack and delta support.

### What this round did not measure

- A repository with no conversion rules at all:
  not built this round.
  What differs from the "as it is" run is that no path is written through Git,
  so its write step (905 ms here) and nothing else would go,
  and the reads (239 ms) and the clone are the same.
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
  not run this round (below).

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
`.gitattributes` files and hand the paths to `git update-index --index-info` and `git checkout-index --force -u --stdin`
in the stage.
The prototype's call is in `serve_snapshot` and does exactly that.
Three details differ
from rung 2:

- The stage's files are the source's,
  byte for byte,
  so the fence's clean-side reading is the same;
  but the stage is a
  copy made by the snapshot,
  not the source,
  so the fence's ancestor walk must look at the stage's directories.
  The prototype still runs the walk,
  and `git check-attr`,
  against `source.root`
  (the fence's call in `worktree_ladder_finish.rs`),
  which is right where the snapshot is a copy of the source's own tree and the source's untracked attributes files
  are copied with it,
  and wrong in one case the gate review named (L3):
  a source with an edited tracked `.gitattributes` whose worktree differs from its `HEAD`,
  where the fence would read the worktree's file while the stage gets `HEAD`'s,
  and `attributes_agree` compares the two commits' attribute blobs without seeing the edited file.
  The fix travels with the fence's rung-1 call:
  run the walk and `git check-attr` in the stage,
  whose files and attributes are the ones Git's checkout of the new `HEAD` reads.
- A dirty source's stage has had `restore_dirty` replace the files the index could not vouch for;
  those are already
  Git's bytes,
  and the fence may leave them out,
  or include them harmlessly.
- The exchange happens after the fence,
  so a fence that fails ends the rung exactly where `restore_dirty` failing ends
  it:
  the stage is destroyed and Git's own checkout completes the worktree.

This was written and measured in the guest by the earlier round of this branch,
at 90 tracked files:
with the fence,
rung 1 equalled Git's own checkout in the four conversion classes with a clean source and with a dirty
one,
and the same build without it was different in all eight.
That run's costs are single adds per arm per scenario (rung 1 median 61 ms without the fence and 78 ms with it),
so they say the fence is cheap at that size and not how cheap,
and this round did **not** repeat the guest run at this repository's size (the gate review's M4):
what stands is the decision's equality and the shape of the add,
not a cost at 24,389 paths.
At this size the fence's floor on rung 1 is the same two Git runs the clone rung pays (239 ms here)
on top of a snapshot add `main` already serves on btrfs,
plus one `git checkout-index` for each path it must write
(905 ms for this repository's 1,578 LFS paths in the clone rung's rounds).
A decline on rung 1 is decided **after** the snapshot,
so a declined snapshot pays the snapshot and then Git's checkout:
the clone rung's decline path measured 2,468 ms against `main` 2,222 on the converting source,
2,036 of it Git's own checkout in `finishRung`.
That is why rung 1's cost,
and the decline rule's answer there,
are the weakest part of this proposal:
recorded as open work,
not as a measured cost.
A sentence on the merge bar the gate review asked for:
on btrfs `main` serves both rungs,
so rung 1's fence does not fix a byte defect `main` avoids there;
it pays the 239 ms of reads,
and any write step,
to keep the fast rung's bytes right,
and the merge bar's time test is `main`'s own btrfs add,
which this round did not measure at this size.
The shape to measure first is a default that runs the fence on rung 1 only where the stage would otherwise be carried
unchecked.
The first guest run of the earlier round established why rung 1 needs a fence at all:
its clean path runs no content judge,
and its dirty path's `restore_dirty` believes every stat-exact entry.

## The options, with their measured cost

Every option below is a way to keep the clone rung (or give it up) while never writing bytes Git would not.
Only options built and measured this round are ranked;
one unbuilt option is listed after them,
marked unmeasured,
and ranked nowhere
(the gate review's M2).
The numbers are this round's medians on the two fixtures above,
24,389 tracked files and 1,578 LFS paths,
`main` at 2,286 ms on the one and 2,222 ms on the other,
with the identical-binary control beside every arm.

**A.
The per-path fence before the clone,
without a decline rule.**
One `git check-attr` over every tracked path,
one `git ls-tree` for the sizes,
the decision in process,
then one `git update-index` and one `git checkout-index -u` for the paths left to Git.
Measured:
2,395 ms on this repository as it is (1.11 of `main`,
control 1.03),
and **3,192 ms on the converting source,
1.50 of `main`** [1.35 to 1.68 per round],
control 0.99.
Right on every path in both.
Pros:
nothing is refused,
so any source keeps the clone's disk saving;
the decision is Git's own rules and an unknown state is a path for Git.
Cons:
where many paths convert it is slower than the thing it replaces,
which is why it does not ship in this shape (H4).

**A′.
A plus the decline rule,
the shipped shape of this proposal.**
The count of paths whose own rule converts them is known from the attributes alone,
before any blob is read and before anything is cloned;
past a quarter of the tracked paths and past a thousand of them,
the rung ends and Git's checkout writes the worktree.
Measured:
2,395 ms on this repository as it is (the rule does not fire:
1,578 of 24,389,
0.065),
and **2,468 ms on the converting source,
1.09 of `main`** [1.05 to 1.17],
control 0.99.
Right on every path in both.
Pros:
no shape of it is slower than `main` on either workload:
where it fires,
its cost is `main` plus one `git check-attr` (36 ms) and the wrapper's own checks;
where it does not,
it keeps the clone rung and is inside its control's spread of `main`.
Cons:
its threshold is arithmetic on this fixture's per-path write cost (0.114 ms),
so a repository whose writes are much more expensive per path,
or whose clone is much cheaper,
would need the share lowered;
and on rung 1 the rule is decided after the snapshot,
which is the cost this proposal does not yet have.
This is the option the ranking puts first.

**B.
The same fence beside the clone.**
**Not built,
not measured,
not ranked.**
The earlier draft placed it between measured options on a bound computed from measured steps;
that bound is still only a bound,
and the option needs the clone,
the fence and the index install reordered around a concurrency the add does not have.
It stays open for a repository whose clone is much longer than its fence,
and building it is the first thing to do if the fence's reads are to come off the add's waiting path.

**C.
A whole-source refusal,
no per-path fence.**
Measured:
2,282 ms on the converting source (1.06 of `main`),
and 2,583 ms on this repository as it is in the first of this round's runs
(5 rounds,
2,168 to 4,956,
control's own spread the same order),
which is `main`'s checkout plus the wrapper's checks,
because a declined add is Git's checkout.
It gives up the clone rung for every source with a conversion setting or an attributes file,
and this repository's first attribute line is one.
Pros:
no `git check-attr`,
no `git ls-tree`,
no per-path model,
0.3 ms of configuration reading.
Cons:
it is 0.09 of `main` cheaper than A′ where A′ declines and keeps nothing else,
and on this repository it throws away the clone rung (A′ 2,395 against C 2,583)
even though only 6.5% of the paths convert.

**D.
Git writes every tracked path after the clone.**
Measured in the earlier round's `r4` on its clean fixture,
with its own control arm:
4,274 ms against `main` 2,057 (2.07 times `main`),
and 4,170 on the converting one.
This round's "as it is" run measured it once,
at 4,735 ms median (4,091 to 9,801) against `main` 4,225 in that run,
whose control spread was 1.44,
so the 2.07 is the number to use.
Pros:
no attribute model,
no untracked-attribute rule.
Cons:
twice `main`;
the rung keeps only its disk saving.

**E.
Keep step 3 as it is and accept the wrong bytes.**
Measured wrong in this round's converting run:
7,097,444 lines of `diff` against `main`'s worktree,
with every stat check green.
Rejected:
`main` on ZFS gets these bytes right,
so this fails the merge bar with no waiver that covers it,
and `SPEC.md:1223` promises the snapshot's worktree is what Git's checkout makes.

**A waiver for a slower-than-`main` fence is no longer an option to offer.**
With A′ measured at 1.09 of `main` where it declines and inside its control's spread where it does not,
there is no shape left of this fence that is slower than `main`,
so the project is not asked to waive one.

Ranking:
**A′ first,
C second,
D third,
E last;
B unranked until it is measured.**

- A′ over C:
  where the share of converting paths is small,
  A′ keeps the clone rung and C gives it up
  (2,395 against 2,583 ms on this repository as it is);
  where the share is large,
  A′ declines and pays one `git check-attr` for the privilege,
  2,468 against C's 2,282 ms,
  which is 0.08 of `main` more and inside the control's 0.99 spread.
  A′ therefore costs nothing measurable on either workload and wins the one where the rung survives.
- C over D:
  on this repository as it is,
  C is 2,583 ms and D 4,274 ms,
  because D's clone is wasted work on top of a checkout
  that writes everything;
  both are right,
  and C is the cheapest correct answer for a repository with no conversion
  rules at all.
- D over E:
  D is right and E is wrong by 7,097,444 lines of `diff` with every check green.

Why B is not between them:
an option that was not built cannot be ranked against options that were,
and the earlier draft's B over C reason ("C is measured and needs no new concurrency") was a reason to prefer the
built one,
not a cost.

## What these costs imply for the partial-clone proposal

The partial-clone proposal
(`docs/planning-cli-git-clone-rung-partial-clone-of-a-dirty-source`) waits on this fence,
and the gate review of this proposal estimated a fenced dirty partial clone at 0.97 to 1.08 times `main`
from the fence's 0.12 to 0.23 of `main` against the partial clone's gain of 0.14.
This round's numbers change three things in that estimate:

- the reads are 239 ms,
  0.10 of `main` on this fixture,
  and they do not shrink by being put beside the clone
  until B is built;
- the write step is real and large where a filter applies:
  1,578 LFS paths cost 0.64 to 0.91 s
  (0.41 to 0.57 ms each) in this round's "as it is" run,
  and that alone is 0.28 to 0.40 of `main`,
  more than the partial clone's entire 0.14 gain;
- so a fenced partial clone of a source with this repository's own attribute set would be **1.3 to 1.5 times `main`**
  on the dirty add,
  not 0.97 to 1.08,
  unless the fence either proves the LFS case or is given a rule that keeps it off sources whose writes cost more than
  the clone saves.
  On a source with no LFS content and no converting rule the review's 0.97 to 1.08 stands as the ceiling.

The partial clone therefore cannot be put to the project with this fence as it stands,
and the LFS question above is the one to settle first:
either Git's smudge stays the proof and the partial clone pays for it,
or someone finds a cheaper proof that a working file is the smudge of its pointer.

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
> through one `git update-index --index-info` and one `git checkout-index --force -u`;
> every other path is carried when its recorded size equals the size of its blob,
> which the one `git ls-tree` reads,
> because a rule can only shrink a file it touches on the way into the index and a file with no rule today can still
> hold a form an earlier rule wrote.
> Where more than a quarter of the tracked paths,
> and more than a thousand,
> have a rule of their own that converts them,
> the rung declines before it clones anything and Git's checkout writes the worktree,
> so the rung is never slower than Git's own checkout.
> A source with an untracked or ignored `.gitattributes` anywhere above a tracked path refuses the rung,
> because Git read it when the files were added and the new worktree will not have it.
> A source that sets `attr.tree` or `GIT_ATTR_SOURCE` refuses the rung,
> because the destination would read attributes from a tree the fence did not read.

### `package/git-policy/cli/SPEC.md`, rung 1

The promise at `SPEC.md:1223` stands as written ("the new worktree is restored to what `git worktree add` makes from a
clean checkout");
what changes is that it now holds for conversions too.
New text,
after the restore's steps:

> Then,
> in the stage and before it is swapped in,
> the snapshot rung runs the same conversion fence as the clone rung,
> reading the stage's own files and attributes and the new `HEAD`'s blobs,
> and the paths it leaves to Git are written by one `git update-index --index-info` and one
> `git checkout-index --force -u` in the stage.
> A fence that cannot read its inputs ends the rung and Git's own checkout completes the worktree,
> and so does the fence's decline rule,
> after the snapshot rather than before it.

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
> and their cost is measured at 0.41 to 0.57 ms per path.
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
No new waiver is needed because no shape of the fence is slower than `main` any more:
option A′ measured 1.09 of `main` where it declines and inside its identical-binary control's spread where it does not.
The wrong bytes are a merge bar failure the waiver does not cover,
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
  (0.41 to 0.57 ms per path,
  0.64 to 0.91 s for this repository's 1,578).
  The earlier promise of a section about carrying them is replaced by that reasoning.
- **H4,
  no shape slower than `main`,
  and a decline rule priced.**
  The rule declines past a quarter of the tracked paths and past a thousand;
  measured on the converting source:
  2,468 ms with the rule against 3,192 without,
  `main` 2,222,
  control 0.99.
  Option A without the rule is listed and measured,
  and no waiver is asked for.
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
  the two runs are 239 ms here,
  and this repository's largest pack has 394,112 deltified objects out of 499,083 with
  chains 50 deep,
  which the repository's object reader (loose objects and zlib only) cannot walk today.
  The measured in-process walk of the same paths that does one `lstat` each is 69.6 ms,
  for scale,
  not as a price.
- **M5,
  the controls and the gate.**
  The six new binary controls are in the tree,
  the ignored-attributes control's pattern is corrected to one that matches
  (`c06b`),
  and the full gate was run at this round's final commit as its own capped unit:
  its verdict is recorded in
  the section below.
  The class controls still cannot run in the gate image,
  whose fixture root is a tmpfs,
  so they print "CONTROL NOT RUN"
  there and run on the host.
- **M6,
  no measurement switch in shipped code.**
  The three routes are behind an off-by-default `measurement-routes` feature which no shipped build enables:
  `strings` on the shipped release binary finds no `MONOCHROMATIC_CLONE_FENCE`,
  and the `off` route,
  which carried every
  path,
  is deleted.
  The proposal says the feature is for measurement only and does not ship enabled.
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

### The gate at this round's final commit

Run as its own capped unit through `mise run //package/git-policy/cli:native:gate`
(the image build,
the compile,
the suite and Clippy of `native:test:container`),
at the proposal's final tip,
with its verdict read through `native:gate:verdict`.
The class controls do not run in that image (a tmpfs fixture root,
so "CONTROL NOT RUN"),
which is the gap the gate review's M5 named and this round states rather than closes:
the image can run the unit controls (18 before this round's six additions),
not the six binary ones.

## What this proposal does not do

- It does not build option B:
  measured now,
  it is bounded by the fence's reads (239 ms) and the clone (847 ms) against `main`'s whole checkout,
  and it needs the add's steps reordered;
  it is ranked nowhere until it is built.
- It does not build the in-process attribute and object reads.
  The claim that they would remove the fence's 239 ms is removed from this proposal,
  and the reason they are not a small change is measured (deltified objects and chains in this repository's pack).
- It does not run the class controls in the gate image,
  for the tmpfs reason above.
- It does not measure rung 1 at this repository's size,
  in a guest or anywhere else:
  the earlier round's 15 of 15 equality at 90 files stands,
  and this round's rung-1 costs are the clone rung's
  components,
  not a measurement.
- It does not repeat the guest's adds:
  one per arm per scenario,
  from the earlier round.
- It does not measure a repository with no conversion rules at all,
  nor `core.autocrlf=true` with no attributes
  (this round refreshed neither fixture),
  nor XFS or overlayfs.
- It does not prove the LFS case cheaply:
  it pays Git's smudge,
  and the partial-clone section says what that means.
- It does not touch `doc/handover/`.
