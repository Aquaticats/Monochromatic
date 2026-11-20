# The clone and snapshot rungs under Git's checkout conversions: a fence, its cost, and the options

**Unapproved and non-final.** Nothing here is a project decision and nothing is declared complete; the owner declares
completion. This is a proposal with a prototype and measurements on this host, under the load it had.

Issue: GitHub issue 6323,
"git-policy-cli: a cloned worktree keeps the source bytes where Git checkout would convert line endings or ident".
Prototype branch: `docs/planning-cli-git-clone-rung-conversion-fence`, made from ZFS plan step 3's integration tip
`74caa69e7a`.
Related records: `doc/decision/cli-git-zfs-clone-rung-dirty-source-waiver.md` (the merge bar and its one waiver),
`doc/planning/cli-git-btrfs-worktree.md` (the ladder and its conversion rule, the bullet "Both rungs require the new
worktree's effective conversion settings"), `package/git-policy/cli/SPEC.md` (what rung 1 and rung 2 promise).

## What is wrong, in one paragraph

Both fast rungs carry the source checkout's working bytes: rung 1 snapshots the whole directory, rung 2 clones every
tracked file and installs the source's index. Git's own checkout does not copy those bytes: it writes the conversion of
each blob (a line-ending rule, `core.autocrlf`, `ident`, a `working-tree-encoding`, a filter's smudge). Where such a
rule applies, the source file need not be the conversion of its blob, and nothing tells: the clean filter maps both
forms to the same blob, and the installed index carries the clone's own stat data, so `git status` is clean. The new
worktree then holds other bytes than `git worktree add` writes, and every check the add runs stays green.

Measured before this proposal, on clean and dirty sources, with controls equal:

- ZFS, `74caa69e7a` against `main` at `2df9b1689b`, rung 2, clean sources: `*.txt text eol=crlf` over LF files,
  `core.autocrlf=true` set after the files were written, `notes.txt ident` over an unexpanded `$Id$`, and
  `* text=auto eol=lf` over a CRLF file that was then `git add`ed all differ from Git's checkout
  (`~/temp/agent/clone-rung-line-ending-reproduction-on-step-3-and-main-2026-11-20/keep/`).
- btrfs in a guest, `main` at `2df9b1689b`: rung 1 with a clean source, rung 1 with a dirty source, and rung 2 with a
  clean source all differ from Git's checkout in those same four cases, with the controls equal
  (`~/temp/agent/snapshot-and-clone-rungs-under-conversion-in-a-btrfs-guest-2026-11-20/keep/findings.md`).

The second measured set matters for the merge bar: on btrfs `main` serves both rungs, so this defect is older than step
3. On ZFS `main` adds through Git's checkout and gets these bytes right, so step 3 does not merge with it
(`doc/decision/cli-git-zfs-clone-rung-dirty-source-waiver.md`: the waiver covers time costs only).

### Why the existing checks cannot see it

Git's own fast path is the stat: `read-cache.c`, `ie_modified` returns 0 as soon as `ie_match_stat` says the entry's
recorded size, times, inode and owner are the file's. The wrapper's clean check makes the same comparison and only
reads a file whose stat data cannot vouch for it (`worktree_clean_tree_walk.rs:248`, `matches_stored_exactly`). The
content judge in `worktree_clean_content.rs` does refuse a path where a conversion could change the bytes, but only
entries it reads reach it: in every case above, all entries are stat-exact, so it reads none (the guest report's
"judge read 2 entries" counts only the genuinely dirty files). `serve_clones` takes no `ConversionEnvironment` at all.

## What Git's checkout actually does, from its own source

These are the rules the fence implements, read from Git 2.56.1's `convert.c` (the gate image's Git is 2.56; the guest's
was 2.43 and behaved the same in the four cases).

- `convert_attrs` makes one action per path from the `text`, `crlf`, `eol`, `filter`, `ident` and
  `working-tree-encoding` attributes and then from `core.autocrlf` and `core.eol`: an attribute wins over the
  configuration, `eol` refines any action but the binary one, and `core.autocrlf` (default false) decides only where no
  attribute names one.
- `crlf_to_worktree` converts only when `output_eol(crlf_action)` is `EOL_CRLF`, that is for `text eol=crlf`,
  `text=auto eol=crlf` and `core.autocrlf=true`, and then only when `will_convert_lf_to_crlf` holds: the blob has a
  lone line feed, and for the `auto` actions it also has no carriage return and is not binary. A blob with no line feed
  at all is never converted, which the fence does not need: it gives those paths to Git as well.
- `ident_to_worktree` expands `$Id$` and `$Id: ...$` with the blob's object name.
- the encoding and the smudge filter convert whole bytes, in both directions.
- `crlf_to_git`, the clean side, only ever *removes* carriage returns, and only for a non-binary action. That is the
  one property the fence's cheap proof rests on: if the rule that wrote the blob can only remove carriage returns, then
  a file whose size equals its blob's size has had nothing removed, so the file is the blob.

## The fence: which paths a clone may carry

The fence decides one tracked path at a time, in the source, before the clone. It may carry a path only when both
sides are proven: the checkout's conversion of the blob is the blob, and the source's file is the blob.

- the checkout's conversion is the blob when the action's output ending is not the carriage return, no `ident` and no
  converting `working-tree-encoding` applies, the `filter` attribute names no program, and the `text` value is one Git
  names;
- the source's file is the blob when no filter, `ident` or encoding applies and either the action is binary (no clean
  conversion at all) or the index entry's recorded size equals the blob's.

Everything else is a path Git writes. The order of the checks is: link, `text` value Git does not name, `filter`,
`working-tree-encoding`, `ident`, then the ending, then the size.

### The classes, and what each answers

- **No rule at all** (`text` unset, `core.autocrlf` unset or false, no `eol`, no filter, no `ident`, no encoding): the
  checkout and the clean side are both the identity, so the clone carries the file with no further read. This is the
  common case and it costs nothing.
- **A text rule whose output ending is the line feed** (`text` with `core.eol` unset or `lf` on a Unix host,
  `text eol=lf`, `text=auto eol=lf`, `text=input`, `core.autocrlf=input`): the checkout writes the blob unchanged, and
  the clean side can only drop carriage returns, so the path is carried exactly when the recorded size equals the
  blob's size. This repository's own first attribute line, `* text=auto eol=lf`, is this class, so every one of its
  tracked files takes the size comparison.
- **A text rule whose output ending is the carriage return** (`eol=crlf`, `text eol=crlf`, `text=auto eol=crlf`,
  `core.autocrlf=true`, `text` with `core.eol=crlf`): Git's checkout may add a carriage return, so Git writes the path.
  With `core.autocrlf=true` and no attributes this is every text path, and the rung degrades to Git's checkout of the
  whole tree (measured below).
- **`text` unset or `!text`**: binary, nothing converts on either side, so the file is carried even under
  `core.autocrlf=true`. This is what makes the fence usable: a repository can exempt its binaries.
- **`ident`**: Git's checkout expands the keyword in the blob, so Git writes the path. A blob with no `$Id...$` in it
  would be carried safely, but the fence does not read blobs to find that out.
- **`working-tree-encoding`**: both sides convert, and the source's file is the working-tree form, which is what Git's
  checkout writes back; a stat-vouched file is the same bytes. The prototype nevertheless gives these paths to Git,
  because it cannot prove that the encode and decode pair is the inverse. The control that measures this class
  (`a_working_tree_encoding_carries_the_sources_bytes_as_gits_checkout_does`) passes on step 3 as well, so the fence's
  choice here costs time and buys nothing measured.
- **A `filter` attribute**: the rung already declines every filter but the standard Git LFS definition
  (`external_conversion_refusal`, `worktree_ladder_target.rs`), so the fence sees at most `filter=lfs`. The fence gives
  those paths to Git: the smudge writes content, and a missing `git-lfs` then fails the same way in both arms. The
  option of carrying them on the design's own argument (a filter that converts a file by its content alone) is below.
- **A link**: Git writes the target and converts nothing, so a link is carried.
- **A `text` value Git does not name** (`text=foo`) or a quoted attributes line: the fence reads Git's own answers
  (`git check-attr`), so a quoted line is not a modelling problem for it; an unknown value is given to Git.
- **A submodule, a split index, a skip-worktree entry**: refused by the clean check before the fence runs.

### The attribute sources, and what each means

- **Tracked `.gitattributes` files**: read by both worktrees. `prepare_target` already refuses a new `HEAD` whose
  attribute files differ from the source's (`attributes_agree`), so one answer stands for both.
- **`$GIT_DIR/info/attributes`, `core.attributesFile` and the default global file, and the system file**: shared by both
  worktrees, and `core.attributesFile` must be absolute (`external_conversion_refusal`).
- **An untracked `.gitattributes` anywhere above a tracked path, ignored or not**: the source alone has it, so Git read
  it when the files were added and the new worktree will not have it. The fence refuses the rung: it walks the
  ancestors of every tracked path in process, compares each `.gitattributes` against the index, and answers
  `<path> is in the source and not in its index, so the rules Git read when its tracked files were added cannot be
  known`. The clean check lists untracked files with `--exclude-standard` (`worktree_clean_untracked_scan.rs`), which
  hides an ignored one, so without this the gate review's finding L3 stays open. The in-process walk costs no Git run
  and no walk of the ignored directories.
- **`GIT_ATTR_SOURCE`, `--attr-source`, `attr.tree`**: the clean check already refuses `GIT_ATTR_SOURCE`; `attr.tree`
  must be refused for the same reason, because the destination would read attributes from a tree the fence did not
  read.
- **An attributes file that was removed after the files were added**: invisible to any fence, in the same way that a
  file rewritten with its own stat data restored is invisible to Git. Recorded as a limit, not fixed.

## How the fence is computed, and what it reuses

The prototype adds `worktree_clone_conversion_rules.rs` (Git's `convert_attrs`, `output_eol` and
`will_convert_lf_to_crlf` as a pure decision, with its own controls) and `worktree_clone_conversion_fence.rs` (the
reads and the one Git run that writes the rest), and calls them from `serve_clones`:

1. `first_untracked_attributes`: the index entries' ancestor directories, in process, one `lstat` of `.gitattributes`
   each; a hit refuses the rung.
2. `git check-attr --all -z --stdin` over every tracked path, in the source: one run, every attribute source, macros
   expanded, precedence resolved by Git. This replaces the prototype's earlier idea of matching patterns again with
   `worktree_attributes.rs`'s parser, which the clean check uses where it must answer without a Git process.
3. `git ls-tree -r -l -z <HEAD>`: one run, only when some path's ending rule can strip carriage returns (class two
   above), for the blob sizes the size proof needs.
4. the decision per entry, from `stored_stats(source.clean.index_bytes, …)`: the entry's recorded size, its mode and
   the attributes.
5. `git checkout-index --force -z --stdin`, after the tree difference and only when the fence left paths to Git, in the
   destination with `GIT_DIR`/`GIT_WORK_TREE` as the ladder's other runs set them, so the destination's own conversion
   writes them.

What it reuses rather than reimplements: `SnapshotSource` and its proven `CleanSource` (the index bytes, the tree and
the hash length), `stored_stats` and `StatEntry` from the clean check, `metadata_verdict`'s stat reading where the
entry's own fields must be judged, `run_git`/`GitRequest` (which records each run as a step of the add's timing
record), `checkout_environment`, `add_timing`'s steps, and `attributes_agree`, `conversion_agrees` and
`external_conversion_refusal` in `prepare_target`, which already narrow what the fence must consider.

What it does *not* use, and why: `worktree_clean_content.rs`'s `conversion_is_the_identity` answers only for the four
tokens `filter`, `ident`, `working-tree-encoding` and a carriage return in the file (`CONVERTING_ATTRIBUTES`); it has no
line-ending action, so it cannot tell `text=auto eol=lf` (carry, with the size proof) from `eol=crlf` (Git writes), and
it reads file bytes to find a carriage return, which a per-path fence over 24,000 files cannot afford. Its
`ContentJudge` is still the right tool for the entries the clean check reads by content, and the fence leaves those
paths to Git.

## What it costs

Measured on this host, on a throwaway repository of this repository's 24,313 tracked files, three arms per round in a
rotating order (the prototype, the same prototype binary under another name as the control, and step 3 at
`74caa69e7a`), with `sync` and 300 ms between arms, and one reference add per round outside the timed window made by
forcing Git's own checkout:

<<MEASUREMENTS>>

What was run, and what it shows: a throwaway repository of this repository's 24,313 tracked files on this host's ZFS
home dataset, three arms per round in a rotating order (the prototype, the same prototype binary under another name as
the control, step 3 at `74caa69e7a`), `sync --file-system` and 300 ms between arms, one round of warm-up, and one
reference add per round outside the timed window (the same source plus one untracked file, so the clean check refuses
and Git's own checkout writes the worktree). The binaries: prototype `fe20d722c3` (sha256
`5598d080d22a4d3e48d4aef7fa700f2359d6e5f67641550b812af02fe48cd95a` before the two debug lines below were added),
step 3 `74caa69e7a` (sha256 `8077e6f9bc1209d76c6a0b67ef3b8a51a1e5011f48677052079863e1b6e69aaf`), both built with
`cargo build --release --locked --offline --bins` because `mise run //package/git-policy/cli:native:build:release`
inside the step 3 worktree built the main checkout's package instead (its binary carried main's refusal text; see
"What went wrong in this run").

Clean source, three counted rounds (the run was cut short at round 4, and its JSON was never written):

- prototype: 1020, 844, 857 ms, median 857
- prototype-control (identical binary): 839, 819, 850 ms, median 839
- step 3: 511, 652, 677 ms, median 652
- every arm's route was `clones`; every worktree was clean on its first status
- the fence's own steps in the prototype's records: `git check-attr` and `git ls-tree` present in each round, and
  `git checkout-index` absent, because this repository's own `* text=auto eol=lf` leaves every path's checkout the
  blob itself and every recorded size equals its blob's size.

So the fence costs about 200 ms on a clean source of this repository's size (857 against 652 ms, +31%), and the
control arm differs from the prototype by up to 18% in one round, which is the noise floor of three rounds on this host.
The step split, the dirty arm, the `crlf` arm and the reference add's own time were not recorded, because the run was
stopped before it wrote its JSON and before it printed its summary. Nothing here is a median over enough rounds to
separate a 20% difference from the host's noise (`AGENTS.md`, `NQB`).

## How the fence applies to rung 1, the snapshot

The snapshot rung carries the same bytes, and the guest measured it wrong for a clean and a dirty source. The fence
applies at one place, in the stage and before the exchange, and needs no new decision: `serve_snapshot`
(`worktree_ladder_finish.rs:131`) runs `restore_dirty` or `apply_difference` on the stage and then swaps it in, so a
call after those two steps and before `put_in_place` (`:180`) would compute the same fence over the stage's own
`.gitattributes` files and hand the paths to `git checkout-index --force --stdin` in the stage. Three details differ
from rung 2:

- The stage's files are the source's, byte for byte, so the fence's clean-side reading is the same; but the stage is a
  copy made by the snapshot, not the source, so the fence's ancestor walk must look at the stage's directories, which
  hold the same tracked and ignored state.
- A dirty source's stage has had `restore_dirty` replace the files the index could not vouch for; those are already
  Git's bytes, and the fence may leave them out, or include them harmlessly.
- The exchange happens after the fence, so a fence that fails ends the rung exactly where `restore_dirty` failing ends
  it: the stage is destroyed and Git's own checkout completes the worktree.

This is a design application, not a measured one: this host has no btrfs and no ZFS snapshot rung (plan steps 10 and
11), so rung 1's fence is written nowhere in the prototype and tested nowhere. What the guest's report establishes is
that rung 1 needs it: its clean path runs no content judge at all, and its dirty path's `restore_dirty` believes every
stat-exact entry.

## The options, with their measured cost

<<OPTIONS>>

Every option below is a way to keep the clone rung (or give it up) while never writing bytes Git would not; each is
judged by the fence's measured cost above and by what it does to a dirty source, which this host's main checkout usually
is (`AGENTS.md`, `NQH`).

**A. The per-path fence before the clone, the prototype.** One `git check-attr` over every tracked path, one
`git ls-tree` when a size can decide, the decision in process, and one `git checkout-index` for the paths left to Git.
Measured cost: about 200 ms on a clean source of 24,313 files (+31% over step 3), and every class it flags is written by
Git. Pros: nothing is refused, so a clean source keeps its clone's disk saving and its time; the proof is Git's own
rules, and an unknown state is a path for Git, never a guess. Cons: it costs a Git run on every clean add even where no
attribute exists, it reads the whole index and every path, and its write step is not yet proven (below).

**B. The same fence beside the clone.** The two Git reads and the decision start before the tree difference and run
concurrently with the clone, since the clone is a blocking-pool job and the reads are child processes; the add would pay
the larger of the two instead of their sum. Its cost is not measured: from the measurement above the fence is about
200 ms and the clone of this repository's files is most of step 3's 652 ms, so the saving would be up to about 200 ms,
computed from measured steps and not measured as a whole. Pros: the cheapest of the per-path options. Cons: one more
concurrent job inside an add, and the fence must finish before the index is installed, which keeps a wait on the add's
path anyway; where the clone is short (a small repository) the fence is the whole cost.

**C. A whole-source refusal, no per-path fence.** Decline the clone rung whenever any conversion source could apply: a
tracked `.gitattributes` anywhere in the tree, `info/attributes`, `core.attributesFile`, a global or system file,
`core.autocrlf` that is not false, or a `filter.` setting. Cost: no Git run at all, one in-process read of the
configuration and a few small files. But this repository has `* text=auto eol=lf` as its first attribute line, so every
clean add here would fall to Git's checkout: the cost is the difference between step 3's 652 ms and Git's own checkout of
the same tree, which this run did not time (the reference add is outside the window and unrecorded). Pros: trivial to
prove, no new Git run, no per-path work. Cons: gives up the clone rung for the repository it is written in, which is the
disk and time saving the ladder exists for.

**D. Git writes every tracked path after the clone.** No attribute read at all: clone (or not) and then let Git write
every path, as the `crlf` arm of the fence would. Cost: one `git checkout-index` over every path with the conversion,
which is Git's checkout plus the clone that was wasted; on this host that is at least Git's own checkout, so the rung
would be slower than rung 3 and save disk only. Pros: no attribute model, no untracked-attribute rule, nothing to get
wrong. Cons: it is the clone rung without its time benefit, and a source with no conversions pays it too.

**E. Keep step 3 as it is and accept the wrong bytes.** Rejected: `main` on ZFS gets these bytes right, so this fails
the merge bar with no waiver that covers it, and `SPEC.md:1223` promises the snapshot's worktree is what Git's checkout
makes.

Ranking: **A best, B next, C next, D next, E last.**

- A over B: A is measured end to end, its steps are separate entries in the add's record, and it needs no concurrency
  inside an add; B's saving is computed, not measured, and its shape puts a second job on the add's path. The ranking
  flips only if a later measurement shows the fence's ~200 ms is the difference between the clone rung and Git's
  checkout on the smallest trees the ladder serves.
- B over C: B keeps the clone rung for this repository and pays ~200 ms; C gives the rung up on every clean add here,
  which costs the whole difference between the clone and Git's checkout, measured for the rung but not for the
  reference.
- C over D: both give up the rung on conversion-bearing sources, but C gives it up only where a conversion could apply,
  so a repository with no attributes keeps its clones; D never keeps the time benefit.
- D over E: D is at least right, and E is wrong on `main`'s own behaviour.

The prototype's write step is the one thing that must be fixed before A can be recommended: measured by hand,
`git checkout-index --prefix=<destination>/ --force -z --stdin` writes the converted bytes over a file whose stat data
matches its index entry, and inside the add the same run exits 0 and writes nothing, so the fence flags the right paths
and the worktree keeps the clone's bytes. The control that compares the two worktrees catches this (the four conversion
controls stay red under the prototype), which is the controls doing their job.

## Where the design and `SPEC.md` change

### `package/git-policy/cli/SPEC.md`, rung 2

The specification says, of rung 2 (`SPEC.md:2651` to `2656`):

> The same `--no-checkout` run is followed by `FICLONE` of every tracked file (mode and nanosecond times kept; links
> recreated), a second comparison of the source with its own index, the installation of the index, and the same tree
> difference.

and the clean check's content judge says of itself (`worktree_clean_content.rs`, the module doc):

> The hash of the raw bytes is conclusive only when Git's own conversion on the way into the index (`filter`, `ident`,
> `working-tree-encoding`, or a line-ending conversion that strips a carriage return) leaves the bytes alone, so a path
> such an attribute applies to, and a file with a carriage return, are refused with that reason instead of guessed.

New text, after the sentence about the clean-source requirement:

> The clone rung carries a tracked path only where Git's own checkout of that path's blob is the blob itself, and the
> source's file is proven to hold that blob. The proof is Git's own conversion rules, read from the path's attributes,
> `core.autocrlf` and `core.eol`: a path whose checkout could add a carriage return, expand an `ident` keyword, convert
> a `working-tree-encoding` or run a filter's smudge is written by one `git checkout-index --force` in the new worktree;
> a path whose rule can only drop carriage returns on the way into the index is carried when its recorded size equals
> the size of its blob; every other path is carried as it is. A source with an untracked or ignored `.gitattributes`
> anywhere above a tracked path refuses the rung, because Git read it when the files were added and the new worktree
> will not have it.

### `package/git-policy/cli/SPEC.md`, rung 1

The promise at `SPEC.md:1223` stands as written ("the new worktree is restored to what `git worktree add` makes from a
clean checkout"); what changes is that it now holds for conversions too. New text, after the restore's steps:

> Then, in the stage and before it is swapped in, the snapshot rung runs the same conversion fence as the clone rung,
> over the stage's own files and the new `HEAD`'s blobs, and the paths it leaves to Git are written by one
> `git checkout-index --force` in the stage. A fence that cannot read its inputs ends the rung and Git's own checkout
> completes the worktree.

### `doc/planning/cli-git-btrfs-worktree.md`, the conversion bullet (about lines 1187 to 1197)

The bullet says:

> Both rungs require the new worktree's effective conversion settings (`filter.*`, `extensions.*`, `core.autocrlf` and
> its relatives), read in the new worktree's own context, to be the same sequence as the source's, in the same order. A
> filter selected by an attribute (tracked, `info/attributes` or a global file) declines the fast rungs, except the
> standard Git LFS definition; …

New text for the end of that bullet:

> Equal settings are necessary and not sufficient: the source's files are the source's bytes, not the conversion of
> their blobs, so both rungs then run the conversion fence described in
> `doc/planning/cli-git-clone-rung-conversion-fence.md` and let Git write the paths it names. A source with an
> untracked or ignored `.gitattributes` above a tracked path declines both rungs, because Git read it when the files
> were added and the new worktree will not have it.

### `doc/decision/cli-git-zfs-clone-rung-dirty-source-waiver.md`

No change. Its waiver covers time costs, and the fence is a time cost; the wrong bytes are a merge bar failure it does
not cover, which is why this proposal exists.

## What went wrong in this run, and what that means for the numbers

Three things are stated so the records can be read without trusting them further than they go.

- The step 3 binary built by `mise run //package/git-policy/cli:native:build:release` inside
  `.claude/worktrees/clone-rung-conversion-fence-step3` was the main checkout's package: it refused the clone rung on
  ZFS with main's words ("the filesystem does not share data blocks between files"), which step 3 does not say. The
  measurement's step 3 arm is therefore the binary this run built with `cargo build --release --locked --offline
  --bins` in that worktree, tied to `74caa69e7a`; the first attempt's numbers were thrown away, not reported.
- The prototype's `git checkout-index` write does not take effect inside the add (the section above). The four
  conversion controls stay red with the fence running, so the prototype today proves the decision and the reads, not the
  bytes.
- The timing run was stopped after four rounds by the coordinator's own timeout, so its JSON and its summary were never
  written; the three counted rounds above are read from its console lines.

Nothing in this run was measured for rung 1, for a dirty source, for the `crlf` arm, or in the container gate, and
`cargo nextest` is not installed on this host, so the unit controls of the two new modules have not been run either.

## What this proposal does not do

- It does not run rung 1's fence, and it does not measure rung 1's cost on btrfs or ZFS. The guest recipe
  (`~/temp/agent/snapshot-and-clone-rungs-under-conversion-in-a-btrfs-guest-2026-11-20/keep/run-guest.ts`) is the way,
  and a fence built with this module needs no change to it beyond the hook.
- It does not measure the fence beside the clone (option B below): the saving is computed from measured steps, not
  measured as a whole.
- It does not measure a repository whose `.gitattributes` sends most paths to Git through `ident`, an encoding or a
  filter; the `crlf` mode measures the case where *every* text path is written by Git, which is the worst case.
- It does not make the content judge read stat-exact entries. That is a different fix (a per-entry conversion refusal
  in the clean check), and it would cost a file read per path, which the fence avoids.
- It does not measure XFS or overlayfs, the other two filesystems `main` serves with rung 2. The fence's decision does
  not read the filesystem.
- It does not touch `doc/handover/`.
