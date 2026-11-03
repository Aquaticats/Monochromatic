# Commits never fail: removing each environmental exit of `git commit` by construction

Scope: `package/git-policy/cli`, the commit path of the wrapper installed as `git` (`cli-git-native`)
and the resident commit service (`cli-git-service`).

The owner's rule, recorded verbatim in `package/git-policy/cli/SPEC.md`, "Merges and commits never fail":
"Commits should never fail."
The owner's correction (2026-11-03, verbatim): "never fail doesn't primarily mean retries!"
The owner's related rules, quoted: "Stop only on no progress", "You should (almost) never be waiting.",
"never discard user data", "commits vs native git should not take 5s longer.", "Use as few locks as possible.",
"Invoke real git as rarely as possible."

## How this record reads

The first version of this record, and the first code written from it, answered each exit with a longer wait, a bigger count
or a step started again.
The owner's correction says that is the wrong default: a failure that can happen is a design defect, and the design
removes it.
So each item below gives the root cause (why this can fail at all), the change that removes the failure mode by
construction, and, only where something outside the system remains, the residue: what happens then and why it cannot be
designed out.
Nothing past the red controls and the conflicted-path change is built; the coordinator and the owner review the design first.

What exists on the branch from before the correction, kept and not discarded:

- The planning record's first version (commit `6c8c686b2a`).
- A red control for item 1 (`53e40e9fc0`), a red control for item 3 (`7b7d9870f3`, written, not run).
- A retry-based first attempt at item 1 (`26db9e54af`): a Git step with no sign of work for its window (the push
  meter of `push_progress.rs`) is stopped and started again with a doubled window, and the object reader is restarted.
  Under this record it is at most the residue handling of item 1 (below) and is the owner's to keep or drop.

## Scope: what the owner settled

The owner settled which causes are outside the system's control (question tool, 2026-11-03).
This list is binding.

Outside our control: the wrapper may stop with a clear message, or retry where the cause can clear by itself;
work already handed over stays on disk and the next run finishes it.

- network down (no route, DNS, reset, refused);
- the remote refusing (login or auth, a server hook, GitHub's GH009, branch protection);
- disk full or quota;
- disk I/O errors or a read-only remount;
- a memory or process-count cap set by someone else;
- a kill (the OOM killer, a person), power loss or a reboot;
- a corrupt repository (missing objects, a broken index) caused by a disk fault or another tool;
- the person's signer failing (gpg or ssh-agent).

Not outside, so designed out: a slow remote; other programs' locks (an IDE or `/usr/bin/git` holding `index.lock` or a ref
lock); Git's own crashes or bugs.
Everything not listed as outside is designed out too, which includes the residues this record first called "a malformed
request" and "an object format this release cannot read" (see "Residues that are designed out", below).

Refusals that may still end a commit or merge with an error: policy errors at error severity; an empty message; a content
conflict no resolver settles (the merge stops with the conflict recorded).

Three more points were put to the owner and are decided (question tool, 2026-11-03):

- A live foreign program holding the branch's ref lock: "Save on a side ref (Recommended)". The commit is stored under a wrapper
  ref (`refs/cli-git/pending/<id>`), the command exits 0 and says the branch will show it once the other program lets go, the
  service then moves the branch onto it in order, and pending commits are pushed only after they land on the branch.
- The person's hooks and required filters: "Allowed refusals (Recommended)". A failing hook (`pre-commit`, `commit-msg`) or a failing
  required clean filter (`filter.<name>.required`) ends the commit with its own message and changes nothing, like a policy error.
  (Today the service route does not run configured hooks and warns that they never run; this decision covers the hooks and
  filters that do run, and does not change which hooks run.)
- A merge that would overwrite unsaved edits: "Carry edits forward (Recommended)". The merge lands, each edited file gets a
  three-way merge of the person's edit onto the merged result in the worktree, conflict markers hold both sides where they
  collide, nothing is lost, and a line names those files. `merge-isolation/paths-in-use` stops being a refusal.

The conflicted-file rule covers `commit -a` as well: `-a` stages the worktree copy of every tracked file, an unmerged file
included, so a `commit -a` over unmerged files commits the markers and prints the same warning for each of those files as the plain
commit does (owner: "Committing a conflicted path is fine. Warn and commit anyway"). Today `-a` commits them without a warning;
the fix lists the unmerged paths of the real index before the staging and warns for those it commits (built on the plain-commit
branch, `fix/git-policy-cli-plain-commit-finishes-conflicted-merge`, tip `a06af6102c`).

A commit that names a conflicted path is no longer a refusal.
Owner, verbatim: "Committing a conflicted path is fine. Warn and commit anyway, even if conflict markers will be committed."
That reverses `d8bcd95218`; it is built on `fix/git-policy-cli-commit-conflicted-path-warns` (tip `7ec22c34da`, for 0.5.26).

So item 7 of the starting list is now: policy errors, an empty message and an unresolvable conflict.
A named conflicted path commits with a warning, and a failing signer is outside.
`SPEC.md` says so.

Items 1 to 6 are designed out where the design below removes the failure, and listed as waits where it only moves it ("What is guaranteed to complete, and what may wait"); no wait is called designed out.
Item 4's moved-head questions were put to the owner and are decided (below).

## What the code shows about why these fail

Three facts of the present design explain most of the exits.

- A cap is a limit, not a mechanism.
  `START_WAIT`, `COMMIT_WAIT`, the class bounds, `LANDING_ATTEMPTS`, `REBUILD_ATTEMPTS`, the index wait of 1800 s and
  `RECONNECTS` are numbers somebody chose to end a wait; none of them detects a fault.
  A commit that fails on one of them fails although nothing is broken.
- The landing holds `index.lock` across slow Git work.
  `follow_up` in `service_land.rs` takes the lock, moves the branch, then builds the index with Git (`diff-tree`, then
  `reset`, in `service_land_follow.rs`) while holding the lock.
  The journal of 2026-11-03 shows that hold at 119 s (below).
  The commit is already durable when the branch moves; the index update is derived data (`build_index` copies the index
  beside the real one and installs it by the release of the lock) and is already finished later by recovery when it is
  not done.
- Work that Git does the wrapper could do in process.
  The wrapper has in-process readers for objects, trees, refs and the index (`repo_read_*.rs`, `repo_read_diff.rs`), and the
  commit-speed branch already compares the index with its base in process.
  Every Git child on the commit path is a chance to be slow for reasons that are not the commit's.

## Evidence for the 120 s commit

An observed commit on `main` (11:07:54Z on 2026-11-03) took exactly 120 s to prepare and 120 s to submit,
with `sync_ms` up to 119429.
The previous agent found no 120 s constant on the commit or landing path, and this record agrees:
the production 120 s constants are `QUERY_CEILING` (`bounded_query.rs`, auto-push queries only) and `CLAIM_LEASE`
(`service_work.rs`); `WAIT_CAP` in `binary_wrapper_store.rs` is test support.

What the evidence shows:

- The commit's timing record (`.git/cli-git-commit-timing.jsonl`, request `ac73f10b-a0a8-456a-91b7-efa8b8e75a25`)
  reads `totalMs` 299988, `prepareMs` 120013, `buildMs` 59790, `submitMs` 120009.
- The service journal (`journalctl --user -t cli-git-service`, instance process 1186934) answers the same request at
  06:07:54.042 local with `lock_hold_ms=119432` and `sync_ms=119429`: the service held `index.lock` for 119 s
  while it brought the index up to date (`follow_up` in `service_land.rs`).
- In that window the service printed wait notices for exactly two Git children, one after the other:
  `git diff-tree -r --name-only -z --no-renames <old> <new>` (process 1240607, a quick query step) started at
  06:05:54.6 and ended just before 06:06:54.0, which is under its 60 s class bound;
  then `git reset -q <new> --pathspec-from-file=- --pathspec-file-nul` (process 1246804, a bulk step) started at
  06:06:54.0 and ended at 06:07:54.0.
  These are the two Git calls of `build_index` for a commit that Git built (`IndexPaths::Derived`).
  So 120 s is two steps of about 60 s, not one 120 s limit.
- The record of 11:10:06Z shows the same shape on the client: `prepareMs` 119474 and `buildMs` 59785,
  and the record of 11:18:08Z shows `submitMs` 119768.
  Phases of one and of two times about 60 s recur in the slow records of that hour (the slow records are phases of
  50 s to 250 s), while a commit of another worktree of the same repository landed in 85 ms
  of `index.lock` hold at 06:06:01 and 06:08:23 local, so the host was not stalled as a whole.
- A later `git diff-tree --no-commit-id --name-only -r -z <oid>` of the review of that commit also waited
  55 s and more (journal at 06:08:03 to 06:08:53).

What Git 2.56.0's own source says about those two children (tag `v2.56.0`, clone `~/temp/agent/git-2.56.0-20261103`):

- `diff-tree -r --name-only -z --no-renames <old> <new>` takes no lock.
  It reads the whole index first, whatever the two trees are: `builtin/diff-tree.c:133` calls `repo_read_index` before it
  looks at its arguments (this is why the package classed it as reading the index).
  Nothing in it waits on a timer; its cost is the index load plus the tree objects, which on a saturated disk with a
  large index and several packs is read time.
  I could not find in its source a place where it waits about 60 s.
- `reset -q <oid> --pathspec-from-file=- --pathspec-file-nul` takes `index.lock` once and dies at once if it is held
  (`builtin/reset.c:491`, `LOCK_DIE_ON_ERROR`, which is a single try with no timeout; `lockfile.c:215` shows that a timeout of
  zero means one try).
  It then does two things: `read_from_tree` (`builtin/reset.c:197`, a diff of the tree against the index limited to the named
  paths, applied by `update_index_from_diff` at `builtin/reset.c:149`), and, unless `--no-refresh` was given,
  `refresh_index` over the whole index (`builtin/reset.c:500-505`, quiet because of `-q`).
  The refresh makes one `lstat` per tracked file, so its cost grows with the number of tracked files and the state of the
  disk's caches; the package's earlier measurement (`doc/planning/cli-git-commit-timeouts.md`) puts it at seconds on a
  warm disk for twenty thousand files.
  The wrapper never needs that refresh: the index it resets is a private copy it installs itself.
  So the `reset` step has a cause in Git's source for being slow (a full-index refresh) and a Git-provided way to skip it.
- Neither command contains a 60 s timer.
  The only 60 s timeout in the source tree is the fsmonitor daemon's start timeout (`builtin/fsmonitor--daemon.c:40`),
  which the owner's rule keeps out of scope: it was not set, unset, pinned or chased.
- Was the 60.0 s `reset` stopped by the wrapper's 60 s quick-class bound?
  No.
  `reset` is not in `PLUMBING_SUBCOMMANDS` (`child_class.rs`), so it is a bulk step with a 300 s bound, and the notice the
  journal printed for it says "a bulk step ... stopping it after 300 seconds".
  The `diff-tree` is a quick step with a 60 s bound and ended 59.4 s after it started, before its bound.
  Neither was stopped; both ended by themselves, so the coincidence of two near-60 s durations is not the wrapper's clock.

What stays unexplained: why each of the two took about 60 s on the main worktree and not on its sibling worktree.
The source shows what each does and that the wrapper's clocks did not fire, but it does not show a 60 s wait, and the
timing was not reproduced (reproducing it would need synthetic load, which is banned).
What the evidence supports, and nothing more: both steps are Git work proportional to the size of the index and the state of
the disk, both ran inside the lock hold, and the design below removes both from the commit path.
A child that took a few seconds longer than its 60 s class bound would have been stopped, and the stop ends the commit
(before the branch moves) or leaves the index update undone (after).

## Transaction model

Three cross-vendor reviews shaped this section.
The third found that the second version still had a receipt that proved nothing on the files backend, a receipt whose deletion
removed duplicate protection, a fence that left the worktree, the index and cleanup unfenced, a checksum offered as provenance, and
acceptance steps that needed ref writes a foreign lock can block.
Several of those came from features: live takeover, receipt refs, a ref fence, a numbered queue, checksum provenance.
This version removes those features instead of adding protocol, and says which owner decision each narrowing still meets.
It is a design to be checked, not a proof; claims marked "to test" need models or fault-injection controls before code ships.

### One owner per worktree, enforced by the kernel

The lane of a worktree (the process that publishes its commits and merges, and materializes them) is whoever holds an exclusive
advisory lock on `lane/owner` in the worktree's private service directory.
There is no epoch, no fence ref and no takeover of a live owner.

- The lock is held on an open file description that every child the lane starts inherits (not close-on-exec). The kernel keeps it
  until every process holding that description has exited, so when a new owner holds it, no obsolete worker, and no obsolete
  Git child, exists. That is the one fence that also covers the worktree, the index and cleanup, which a ref transaction cannot fence.
- The lock covers the smallest unit (one worktree) and is held by the kernel for exactly the owner's life, so it cannot go
  stale. The reason it is needed is written here as the repository's rule asks (concurrency is lock-free first): no lock-free
  claim can tell a new owner that the previous owner and all its children are gone.
- A wedged owner that is alive is not taken over by another process. Its service unit has a watchdog fed by the landing task's
  progress counter (written after each phase, never by a timer thread); the manager ends the unit's whole control group when the
  counter stops, which releases the lock. A client-owned lane ends with the client's process group.
- A new owner first ends any process group still recorded in `lane/owner` (checked by process start time), then holds the lock.
- Services and clients both can own the lane: a client that finds no owner takes the lock and runs the lane inline until its queue
  is empty; a service that starts later waits for that client's lock and then takes over the rest.

The owner's decision "Use as few locks as possible" is met by one lock per worktree and no other lock added; the alternative that
needs no lock is a service-only lane (no client fallback), which makes the manager the single-instance enforcer; that would remove
the client's ability to land when the service does not start, so it is listed as a question below.

### Durable acceptance needs no ref write

A command is accepted when its immutable request is durable in the worktree's service directory; no Git ref is written and no
foreign lock can stand in the way:
1. read the branch tip `B` (reading refs takes no lock) and read the base content the replay needs from `B` into the spool
   (below); if an object is missing because a foreign rewrite and a prune raced the read, read the tip again; nothing was
   handed over yet, so starting again is allowed;
2. read the worktree and the real index and write the captured bytes into the spool;
3. fsync each spool file, fsync the spool directory;
4. write the request record to a temporary name, fsync it, rename it into `inbox/<id>`, fsync `inbox/` and its parent;
5. only then answer that the request is handed over.
Acceptance is the durable existence of the complete record in `inbox/`. There is no number to allocate and no half-accepted
state: a record is either complete under its final name or absent.

### Order, adoption and the logical head

The lane alone orders requests, by adoption: when it takes a request from `inbox/` it appends `adopt <id>` to `lane/order`
(append, fsync) and moves the record to `queue/<id>`. A crash leaves a request either in `inbox/` (not adopted; adopted again in
the order of its record's capture time and id) or adopted (named in `lane/order`); there is no slot that can stall the queue.
Because there is one lane per worktree and one writer of `lane/order`, an obsolete allocator cannot exist.

Capture against the logical head.
A request records the logical head it was captured against: the newest commit of the queue (a pending one) as published in the lane's
state file, or the branch tip when nothing is pending, together with the delta (the paths, their new bytes, deletions, modes)
relative to that base.
At adoption the lane replays the delta onto the actual logical tail by the three-way replay below. It never replaces the base
under an unchanged result tree: a base substitution would turn another request's change into part of this one (the reviewer's
`x=0`, `y` example), and the replay does not.

Staged changes while commits are pending (the logical-index model).
A plain commit captures what is staged as the difference between the real index and the tree of the physical head, and applies
that delta to the logical tail. The real index is not rewritten by a pending commit, so it equals the physical head plus what
the person staged since; the pending request's own changes are therefore not in the staged difference, and no stale entry is
read as a reversal. The limit, stated: while commits are pending, staging a path back to the physical head's content cannot
express a revert of a pending commit.

Pending requests are immutable versions.
An amendment of a pending request (owner: "Amend the pending commit") is a new immutable request that names the one it
supersedes. The lane is one sequential worker: it adopts the amendment between steps. If the superseded request is unpublished
the lane drops it and keeps the new one in its place; if it was published the amendment is an amend of the tip. No worker prepares
the old version while the new one is adopted, because there is no second worker.

Signed and hooked commits.
A commit Git signed is valid only on its own parent. It publishes only if its parent is the tip; if the branch moved it stays
pending with a message that says it needs signing again, and it is built again when the person next runs a wrapper command (the
signer and its prompt belong to the person, outside our control). This narrows "the service lands it in order when the branch
lets it" for signed commits; it is a question for the owner below.

### Publication

The publication is `update-ref refs/heads/<branch> <new> <old>`, a compare-and-swap, nothing else: no receipt ref, no fence ref
in the transaction (both withdrawn).
The lane writes `publishing <id> <old> <new>` into the request record (fsync) before the swap and `published` after it.

Recovery decides from the branch and the reflog, and never loses a commit:
1. the new commit is the tip or an ancestor of the tip: published;
2. otherwise the new commit is unreachable. If the branch's reflog has an entry whose new value is the commit, it was on the
   branch and was rewritten away deliberately: it is not published again. The lane keeps it under a hold ref, records
   `rewritten-away`, and tells the person where it is saved. A reflog entry written before the ref rename is evidence the swap
   began, not that it finished, so this rule errs toward not resurrecting;
3. otherwise (no reflog evidence, or no reflog) the swap did not happen or the evidence is gone: the lane publishes again, replaying
   the captured delta on the tip. This resurrects the commit only in the case where it was published and then removed by a
   non-descendant branch rewrite before recovery ran and the reflog is missing. The consequence is one commit reappearing,
   visible, with a message; no data is lost.
Reachability alone never closes a request: a request is `published` only on rule 1, never on "not reachable".

### Retirement keeps the deduplication evidence

When all obligations are done the lane retires the request in this order: (1) append a terminal line to `lane/ledger`
(`<id> <outcome> <commit> <time>`, fsync); (2) release the retention (hold refs, spool, record); (3) forget nothing else.
The ledger is append-only and rotated by size, keeping at least the newest entries covering a long horizon (days, measured in
entries, not minutes); a client resending a request id after retirement receives the ledger's outcome and nothing is published.
If a crash falls between (1) and (2) the ledger line exists and recovery completes (2); a request never has the retention released
before its ledger line is durable. A resend of an id found in neither the inbox, the queue nor the ledger is a new request
(clients live seconds; the horizon is far beyond that).

### Obsolete workers cannot exist

The lock above is the proof obligation for the worktree, the index and cleanup, not only the ref: any process that acts for the
lane holds the lock's description, so a new owner starts only when none is left. Cleanup of a spool, a hold or a saved file is
done only by the owner, only for requests whose ledger line is durable. No process outside the lane deletes request files.
To test: kill the owner at every phase, start the next owner immediately, and check that no Git child of the old owner is alive
and that no request is published twice.

### Interrupted wrapper-owned transactions

Before each ref transaction the lane records in `lane/ledger-open` the lock files it expects to create (ref lock paths) and the
content it writes. A new owner sweeps a `.lock` file only when it is listed there, its content equals what was recorded, and the
previous owner's group is gone (it is the owner now). No other lock file is touched.

### Retention lifecycle and the spool closure

The spool is the source of truth for the request until the ledger line exists. It is a self-contained closure for replay: the
captured bytes, the paths and modes, the expected pre-commit index entry of each path, the content at the captured base of each
path (the "before" side) and the trees along each path, and for a conclusion, an amend or a revert the other parent's commit and
what the replay needs from it, the captured resolved tree, the message and the identities. It is written as files and a pack
(`index-pack` style) that can be imported again.
The lane imports the spool with one `fast-import` stream and, when the ref store allows, a hold ref
`refs/cli-git/<instance>/hold/<id>`; the hold is an extra protection and the spool is the guarantee.
Anything a prune removes before retirement is imported again from the spool.

Overlapping prune, checked in Git's source (`builtin/prune.c:36` and `:98`): `git prune` removes an unreachable object whose
modification time is not later than the expiry, and the default expiry when none is given is the largest time
(`builtin/prune.c:172`), so a plain `git prune` deletes unreachable objects written after it started, and `git gc --prune=now`
spares only objects written strictly after the second it began. A prune that took its reachability view before the request's
objects were referenced can therefore delete them, whatever the lane does after that.
The lane's answer is narrowed accordingly:
- after publication and before retirement it checks connectivity (`rev-list --objects --quiet <new> --not <old>`) and imports
  again from the spool if anything is missing;
- the spool is released only after that check passes;
- a prune that is still running across the release and deletes afterwards is Git's documented-unsafe concurrent prune (the manual
  quoted in `service_pruned.rs`), and the resulting missing object is a corrupt repository caused by another tool, which is on the
  owner's outside list. This is the one residue of item 6, stated, and a question for the owner.
To test: prunes that start before, during and after each phase, with and without `--expire=now`.

### Installing into the current index

Only a delta applied to the index read under the lock; never a whole-file install built earlier.

When the lock is free at publication time the lane takes it first (a single try), swaps the ref, applies the delta and releases it
in one critical section, so no index writer can interleave between the publication and its materialization.
When the lock is held by a foreign program the commit is published without the index and the delta is applied later:
1. take `index.lock` with a single try;
2. read the current index (in process);
3. verify the commit is still reachable from the branch tip (else the branch was rewritten: drop the intent, do not restore
   staging from an abandoned commit);
4. for each path apply the entry rule, leaving every other entry untouched;
5. write the lock file and rename it over the index.

The entry rule, with `E_pre` (entry at capture), `E_commit` and `E_now` read under the lock: if `E_now == E_commit`, nothing; if
`E_now == E_pre` and the worktree file still has the committed content, install `E_commit`; otherwise leave `E_now` and report the
path. Equality of entry values is the only evidence the index offers. No checksum provenance is claimed: a checksum identifies
contents, not who wrote them, and an index restored to the same bytes by another program is indistinguishable from none.
The accepted residue, stated: a program that deliberately restores exactly the pre-commit entry while the file still holds the
committed content gets that entry replaced; the file, the history and the index then agree, and no content is lost.

Extensions, per policy (not preserved wholesale): the cache-tree is dropped (Git rebuilds it); the resolve-undo data and the
untracked cache are carried unchanged; the offset tables (`EOIE`, `IEOT`) are not written; any other extension (fsmonitor,
split index, sparse index, an unknown one) makes the index ineligible, and the delta runs through Git on a copy made under the lock
(`reset --no-refresh`), the slower hold the service has today. Index versions the writer does not produce make it ineligible.

The wrapper's own index commands do not wait for the lane's intents behind a foreign lock: they run as Git runs them and the
conditional rule keeps whatever they stage.

### The side ref for a held branch lock, or a branch a foreign program keeps moving

Owner decisions (question tool, 2026-11-03): "Save on a side ref (Recommended)" for a held branch ref lock, and "Save on the side
ref (Recommended)" for a branch a foreign program keeps moving.

The publication is tried once. If it fails the lane reads the actual failure and classifies it: the lock on the branch exists
(`unable_to_lock_message`, `refs/files-backend.c:859-866`, Git waits 100 ms by default, `refs.c:996-997`); the old value did not match;
no space, I/O error or read-only (the owner's outside causes); or other (kept `handed-over`, reported, retried when the repository
changes). An unchanged tip never counts as proof of a lock.
For a held lock, or a lost swap, the request becomes `pending`: the commit is built and in the spool and record, the command
exits 0 and says the branch will show it when the other program lets go, and the lane publishes pending requests in adoption
order when the branch allows it. The side ref `refs/cli-git/<instance>/pending/<id>` is written when the ref store lets the lane
(a loose ref in the files backend takes no branch lock); the record and the spool are the guarantee, so a store-wide lock on
reftable does not stand in the way of acceptance.
Pending requests are not materialized. A request remains pending, intact, for as long as the holder holds the lock.

Pushes.
Refs under `refs/cli-git/` are never pushed. The enforcement is on the effective updates, not on argument strings: the wrapper
runs the push it controls (the auto-push, and a forwarded `git push`) after a `--dry-run --porcelain` that Git expands (explicit
refspecs including `refs/*:refs/*`, configured `remote.<name>.push`, `push.default`, `--mirror`), and adds the negative refspec
`^refs/cli-git/*` for every update the dry run lists in that namespace, so the real push sends everything else; the wrapper prints
a line saying the wrapper refs were left out and why (owner, "Leave out wrapper refs (Recommended)"). Whether a negative refspec
is honored together with `--mirror` is to be shown by a control on the installed Git; if not, a mirror push is run as the
dry-run's list of updates minus the namespace. Controls: `refs/*:refs/*`, `--mirror`, a configured push refspec, the auto-push.

### Replay of the captured change (owner's choice: stack on the new head)

Let `B` be the captured base (the logical head at capture), `T` the captured resulting tree (or resolved tree for a conclusion)
and `C` the tree to publish on. The new tree is the three-way merge with base `B`, ours `C`, theirs `T`
(`git merge-tree --write-tree --merge-base=B C T`, or an in-process equivalent that matches it). A conflict is an allowed outcome:
the replay stops with the conflict recorded and the captured inputs kept. The commit's parents are `C` (and `T`'s other parent for
a conclusion).

## What is guaranteed to complete, and what may wait

No wait below is called designed out.

Guaranteed to complete, with no wait on another program and no cap or count:
- acceptance: local reads and an fsynced write in the worktree's own directory, no ref write; failures are the owner's outside
  causes (a full disk, I/O errors);
- the command's answer: `published`, or `pending` (exit 0, saved) when a foreign program holds the branch lock or keeps moving the
  branch, or `handed over` when the lane is busy; the lane's single attempt is prompt;
- the answer after a crash (a resend returns the record's state or the ledger's outcome);
- retention of everything the request needs until its ledger line is durable.

May wait, off the command (it has exited 0), with a notice that names what is waited for:
- publishing a pending request while a live foreign program holds the branch's ref lock or keeps moving it; if it never lets go
  the commit stays pending, intact;
- the index catching up while a live foreign program holds `index.lock` (owner: "Same as the ref lock (Recommended)"); a merge saves
  its result on a side ref, exits 0 and finishes the checkout when the lock frees;
- a wedged lane owner, until the manager's watchdog ends its unit.

May wait on the command: nothing on a foreign program. The remaining waits are the person's own: an editor, a signer's prompt, a hook.

Hung Git children.
The children on the commit path are few: `fast-import`, `commit-tree`, `merge-tree`, `update-ref`, and the commit Git builds for a
signed or hooked commit. A child that shows no sign of work for its window is stopped and started again when it is idempotent:
objects are content-addressed and the publication is a compare-and-swap that is safe to run twice. Not restarted: a commit Git
builds with a hook the person configured, and an `update-ref` while a `reference-transaction` hook is configured, because the
hook is the person's program and may have an external effect; those wait (a person's hook). Internal transactions on
`refs/cli-git/` run without the person's hooks (`core.hooksPath` set for them only).
A deterministic Git bug that hangs the same input every time cannot be restarted away. The guarantee is narrowed to what holds:
a child that stalls twice on identical input makes the request pending with a message that names the child and the input, the
command exits 0, no work is lost, and the commit is not on the branch until the cause changes (a Git upgrade, or the in-process
writer below, which is not chosen). This is a question for the owner, because "Git's own crashes or bugs" were listed as designed out.

## Item 1: a Git step of the commit crosses its class bound

Root cause: the commit path starts many Git children (the quick class, 60 s, includes `diff-tree`; the bulk class, 300 s)
and a clock ends any child that outlasts its class, whatever it is doing.
A child that is slow on a loaded or saturated host is killed and the commit fails.
Two causes are stacked: the clock kills, and the steps are slow.

Change that removes it:

- Remove the kill.
  The watcher keeps its notices (it names the process and what it waits for) and stops ending children.
  A commit then has no time-based exit at all.
- Remove the steps.
  The slow ones in the evidence are the follow-up's `diff-tree` (old to new commit, to learn which paths changed)
  and `reset` (to put those paths in the index), and the review's `diff-tree`.
  The first is a tree comparison the wrapper already does in process (`repo_read_diff.rs`); the service has the
  request's change list for every commit it builds itself, and for a commit Git built it can compare the two trees
  with the in-process reader.
  The second is an index entry update for known paths, which is a write of the index file the wrapper can do
  itself (the index is copied beside the real one already), so the landing starts no Git for it.
  The goal is a landing that starts Git only for `fast-import`, `update-ref` and, for commits Git must build
  (signed, hooked, editor), the build itself.
- `diff-index --cached --quiet HEAD` over 50 s on main (reported by the coordinator, not measured here): the
  commit-speed branch replaces it by an in-process comparison of the index with its base; this record takes that over.
- Two more client steps measured on main by the coordinator at about 07:35 local on 2026-11-03 (installed 0.5.25, load
  average about 23): `git add --all -- toolchain.lock.json` (bulk class, process 3490505) waited over 50 s, and the commit
  before it waited over 20 s on `git read-tree --reset <head>`.
  From Git's source: `add` takes `index.lock` with a single try (`builtin/add.c:536`, `LOCK_DIE_ON_ERROR`), refreshes the
  index for the pathspec (`builtin/add.c:134`), runs the diff of the worktree against the index for the pathspec
  (`add_files_to_cache`, `read-cache.c:4027`) and writes the index; `read-tree --reset` takes the lock the same way
  (`builtin/read-tree.c:191`), reads the index and rewrites every entry from the tree.
  In both the work that Git does for one named file is proportional to the whole index.
  The change: hash the named path in process (with the repository's clean filters and line-ending rules, ported from Git's
  `convert.c`, and falling back to Git for a path with a filter the port does not cover) and write its entry into the
  private index in process; and skip `read-tree --reset` when the private index already equals its base tree, which the
  commit-speed branch already decides.
  Expected saving per step: `add`, the whole observed wait except the read and write of the index file the in-process
  writer still does once (no process start, no lock try, no full refresh, no untracked walk); `read-tree --reset`, all of
  the observed wait when the index equals its base, which is the common case for a path commit.
  These are expectations, not measurements: the observed waits came from a loaded host and I did not reproduce them.
  Before building, time each step in the fixture with `strace -f -ttt` as `cli-git-commit-timeouts.md` did, so the saving is
  measured and not asserted.
- The follow-up `reset` of the landing: put the named paths' entries into the index copy in process by porting
  `update_index_from_diff` (`builtin/reset.c:149`): for a path missing from the tree remove the entry, otherwise make a
  cache entry from the tree's mode and object (keeping a skip-worktree bit as that function does) and replace the entry,
  which also clears unmerged stages of the path.
  Where the port cannot decide (sparse index, sparse checkout), run Git with `--no-refresh`, which is Git's own switch
  (`builtin/reset.c:352`) and removes the whole-index refresh.
  The follow-up `diff-tree` becomes an in-process tree comparison (`repo_read_diff.rs`), which also never reads the index.

Residue: a Git child that never ends (a dead network file system under the repository, a process stopped by someone else).
No time bound can tell it from a slow one without risking the commit, and killing a writer in the middle of its write
can cost data, so the commit waits with a notice every interval that names the process, and the person (or the process's
owner) ends it.
The retry-based attempt on the branch (stop a read-only step that shows no sign of work, start it again with a doubled
window) can serve this residue for steps that only read; the owner decides whether it stays.
It also serves as the containment of a Git bug in a retained child that is safe to run twice (a hung `fast-import` or `commit-tree`);
`update-ref` is stopped and started again too, because the publication receipt makes the transaction safe to run twice (second review,
accepted); a deterministic Git bug on one input cannot be restarted away, and "Transaction model" names the alternative.

## Item 2: the service does not start within `START_WAIT`

Root cause: the commit depends on a second process that the manager must start, and the client gives it ten seconds.
The commit's own work is complete before the service is asked.

Change: the service stops being a correctness dependency.
The client hands the request over by writing the record (state `handed-over`) and then asks the manager to start the instance.
If no live owner of the lane answers, the client takes the fence of the lane by compare-and-swap and acts as the lane owner for this
worktree, with the same code and the same record, until the service takes it over the same way.
No second coordinator exists: ownership is the fence ref, and the publication transaction carries the fence, so the older owner
cannot publish (review findings 3 and 1 of the second review, accepted; see "Transaction model").

Residue: none that ends a commit.
Failures about the machine's setup (another account's service, a program that is missing, a unit file in the way) become
notices that the service is not running.

## Item 3: the service does not answer within `COMMIT_WAIT`

Root cause: silence is read as a fault, and a read timeout decides.

Change: the commit waits on the record, not on a connection or a clock.
The client reads the request's state (the service pushes it on the open connection and the client re-reads the record if
the connection closes).
The service's progress counter (written by the landing task after each phase, "Transaction model") replaces the heartbeat:
a heartbeat shows that the sender runs, not that the request moves (review finding 7, accepted).
A closed connection is a resend with the same id.
A lane whose counter has not advanced for a lease period and whose process shows no sign of work is taken over by a swap of the fence, so a
wedged service neither fails the commit nor lands it twice.
`WaitedTooLong` and `RECONNECTS` leave the commit path.

May wait: a service that is alive and slow makes the commit slow, and the notice names the phase and the process.

## Item 4: the branch keeps changing; a commit cannot replay onto a moved head

Root cause: the landing reads the tip, builds on it, then moves the ref only if it is still the tip
(`update-ref` with the old value), and counts failed rounds (`LANDING_ATTEMPTS` 3 in the service, 5 builds and 3 prepares
in the client).
A failed compare-and-swap means another writer moved the ref, which means the system as a whole made progress: the
compare-and-swap is lock-free, so some writer always succeeds.
A count turns that guarantee into a failure for the writer that happened to lose three times.
For a commit Git builds elsewhere (signed, hooked) the moved head also invalidates the work, because a signature covers
the parents.

Change that removes it:

- Remove the counts, and say honestly what remains.
  A commit built by the lane from a change list is rebuilt on the new tip with no cost to the person, and each lost
  compare-and-swap is another writer's commit.
  That is system-wide progress, not progress for this request: a compare-and-swap is not starvation-free (review finding 7).
- Make the moves rare.
  The lane of a worktree is the one writer of that worktree's branch: its commits, the push's reconcile merge and the merges
  it completes are items of the same lane, served first in, first out, so the head moves under a landing only for a foreign
  program that writes the ref with real Git directly.
- For a commit Git must build (signed, hooked, editor) the build starts from the logical head and its publication is the fenced
  transaction; a build whose base moved is built again, or, for a signed commit that cannot be replayed, becomes pending.
- A conclusion, an amend or a revert whose head moved while the person worked is built by replaying the captured change on
  the new head (owner's choice, below).

May wait: a foreign program that writes the branch with real Git directly, faster than a commit can be built.
Real Git offers only the compare-and-swap for a ref, so that program cannot be excluded; each lost round is that program's
commit, and the request is built again on it with a notice that names the foreign tip. Not starvation-free for one request.

### Decided: stack the result on the new head

The owner chose (question tool, 2026-11-03): "Stack on the new head (Recommended)".
Options that rewrote the other session's commits (rebuild on the new head), held the commit until the head was quiet, or
kept failing are dropped.

Setting: `main` is at commit `B`; another session lands `C` on `main` while this commit is built or while the person works.

An amend.
The person typed `git commit --amend -m "fix the parser"` on top of `B`, whose message was "parse the file".
Before: `A <- B (parse the file)`; the other session lands `C (add the lexer)`: `A <- B <- C`.
After: `A <- B <- C <- D (fix the parser)`, where `D` is the amend's captured change (`B` to the amend's tree) replayed on `C`,
so the lexer that `C` added is still in `D`'s tree.
`B` and `C` stay as they are, so no commit another session landed is rewritten.
Standard error says why: the branch moved, so the amend was made as a new commit on the new head.

A conclusion.
The person finished a merge of `topic` (tip `T`) into `main` on `B` with `git commit`; the other session lands `C`.
After: `A <- B <- C <- M`, where `M` has parents `C` and `T` and its tree is the person's captured resolution (`B`'s merge with
`T`, with their manual choices) replayed on `C`; the resolution is not recomputed from scratch, so the person's choices are kept.
A content conflict that the replay on `C` cannot settle stops it as any merge does, with the conflict recorded.

A revert.
The person ran `git revert B` and then `git commit`; the other session lands `C`.
After: `A <- B <- C <- R`, where `R` is the captured revert (`B` to the revert's tree, with any edits the person made while
preparing it) replayed on `C`, and stderr names the new head.

What the owner's choice leaves: an amend on a moved head is no longer an amend (`B` stays and `D` is added), so the person sees
two commits where they asked for one, and stderr says so.
This is the one place where the commit does not do what the words of the command mean, chosen by the owner.
The commit is built inside the worktree's lane from the captured inputs, not from the worktree.

## Item 5: `index.lock` is refused after the service's wait

Root cause: the landing needs the real index lock to install the updated index and holds it across Git work; a foreign program can
hold it for as long as it likes, and after 1800 s, or at once when no inotify watch can be set, the commit fails although the
commit itself did not need the index.

Change (owner, 2026-11-03, "Same as the ref lock"): the commit does not wait for `index.lock` at all.
The publication does not take it. The index catches up afterwards by the delta applied to the index read under the lock
("Transaction model, installing into the current index"), with a single try for the lock and a watch on its path set before the
try (re-checked on every wake, with a periodic re-check when the watch fails, so a missed event or a replaced file cannot stall
it).
The command exits 0 with a line saying the index will catch up when the lock is free.
No foreign lock is ever removed: a PID or an open descriptor does not prove the pathname still names an abandoned lock, and a dead
writer's lock can hold a complete index not yet installed.
The earlier plan to build the index off the lock and install the whole file is withdrawn, because it can erase another program's
staging.

May wait, off the command: the index catching up, for as long as a foreign program holds the lock. The commit is on the branch.

## Item 6: `ContentPruned`

Root cause: the objects a commit captured are unreferenced until the branch moves, and `git gc --prune=now` deletes every
unreferenced object at once.

Change: the retention lifecycle of "Transaction model": the hold on the base before any read, a spool that is a self-contained
closure (new bytes, the base content the replay needs, and the conclusion's dependencies), the import with a hold ref in one
stream, the owner of retention being the record until `closed`.
A prune at any point after handover is repaired from the spool; before handover the client reads again, which is legitimate
because nothing was handed over.
The earlier plan to prepare again from the worktree after handover is withdrawn: it would commit a later edit.

Residue: none for pruning.
The spool is on the same disk, so a full disk or an I/O error while writing it is one of the owner's outside causes.

## Merges, by the same lens (design only)

The merge-fixes branch waits for any `index.lock` holder before the merge and computes the merge again on a moved head up to
twenty rounds (`merge_isolation_moved.rs`).
Deferring the checkout past the publication is withdrawn (first review): a checkout after another commit descends from the merge
cannot be rolled back.
The order stays: compute the merge commit (no index), materialize the worktree and index under the lock, then swap the ref.

- Held `index.lock` (owner, "Same as the ref lock"): the merge takes the lock with a single try. If a live foreign program holds
  it, the merge commit is saved on `refs/cli-git/pending/<id>` (and in the spool), the command exits 0 and says the checkout will
  finish when the lock is free, and the lane does the checkout and the swap then. Nothing was published or checked out, so the
  person's worktree is untouched meanwhile; edits made in the meantime are carried forward at the time of the checkout (below).
  If the branch moved meanwhile, the merge is replayed on it as for any moved head.
- Held branch ref lock at the final swap: the checkout already happened; the merge commit is saved on the side ref, the command
  exits 0, and the lane swaps when the lock is free. Until then `git status` shows the merge's changes as staged against the old
  head.
- The twenty rounds: a lost swap is another writer's commit, so the count is artificial; the merge is an item of the worktree's
  lane (first in, first out with commits and the push) and a branch a foreign program keeps moving makes it pending.
- `merge-isolation/paths-in-use` stops being a refusal (owner, "Carry edits forward"). Per file that the person edited and the
  merge changes, under the lock and before the swap:
  1. open the path with `O_NOFOLLOW`, read its bytes `E`, and keep that descriptor (inode `N`); a symlink, a directory or a
     special file is not merged as text (below);
  2. compute `R`, Git's three-way text merge of the old head's content `O`, `E` and the merged result `M` (`git merge-file`
     semantics, with the repository's conflict style and labels naming the person's edit and the merge), and write it to a
     temporary file beside the path;
  3. pin `N` under a saved name by a hard link to the inode itself (`linkat` through `/proc/self/fd/<n>`, same file system),
     so a write through any descriptor still open on `N` lands in the saved file and cannot be lost; where a hard link is not
     possible, make a copy and keep the descriptor until step 5;
  4. exchange the temporary file and the path in one `renameat2(RENAME_EXCHANGE)`: the path now holds `R` and was never absent,
     and the temporary name holds whatever inode was at the path at that instant;
  5. read the exchanged-out inode again: if it is `N` with the bytes `E`, the merge is complete; if its bytes differ, or it is
     another inode (an editor saved a new file between the read and the exchange), run the three-way merge once more with that
     content as the person's file and exchange again. Two rounds at most: writes after the second exchange to an inode that
     is no longer named by the path go to the saved file, which the line names; they are never discarded.
  6. the index receives `M` for the path, so the person's edits are ordinary unstaged changes on the merged result, and a
     conflict leaves markers with both sides and a line naming the file and the saved copy.
  A file that is not text, a mode or type change, a symlink, a directory in the way, or an untracked file colliding with a file the
  merge adds keeps the person's version at its path; the merged version is written beside it under a name the line gives.
  Where the file system does not support the exchange, the weaker protocol is move-aside then no-replace link, with the saved
  hard link, and the line says that a write through an open descriptor after that moment is in the saved file.
  Supported semantics, stated: an edit through a descriptor that is still open is preserved either in `R` or in the saved file;
  an atomic-save editor that renames a new file over the path is treated as a new version of the person's file.
  To test: a write through an open descriptor before the exchange, after it, a truncation, an atomic-save editor, a symlink at the
  path.

## Residues, by class

- A malformed request.
  Structural errors are detected before the record is written (the client builds it from typed values and validates it), and a
  record carries a format version: a worker that does not know a record's version does not process it, and the fence passes to
  a worker that does. A well-formed request that means something the lane cannot do is a policy decision, classified below.
- An object format or layout this release cannot read.
  Every in-process port is gated by eligibility (files or reftable ref backend, SHA-1 or SHA-256, no sparse or split index or
  unknown extension, attributes and filters covered), and every operation has a Git-command implementation complete for all
  layouts: the landing, the reconciliation and the recovery all run through Git when the gate says no. The ports (hashing with the
  repository's conversions, index delta, `update_index_from_diff`) are correctness boundaries, each with differential controls
  against Git on attribute precedence, required filters, encodings, line endings, index flags and stages.
  Failure independence is not claimed for the Git path: it meets the same locks and crashes, and those are classified as above.
- Local hooks and required clean filters: allowed refusals (owner); the commit ends with the hook's or filter's own message and
  changes nothing. The service route does not run configured hooks today and warns; this does not change which hooks run.
- The exit inventory is a gate, not a runtime default: before any code, every reachable `die` and `error` of each retained Git
  child (`fast-import`, `commit-tree`, `merge-tree`, `update-ref`, `reset`, `add`, `read-tree`, the commit Git builds) is listed
  from the 2.56.0 source with its class: outside the owner's list (clear message), designed out, or an allowed refusal.
  An exit that cannot be assigned is a design defect to report to the owner, not a nonzero exit left in place.

## Answers to the first review

Each finding of `~/temp/agent/reviews/commits-never-fail-sol.txt`, with a decision.

- Critical 1, deferred merge checkout overwrites later edits: accepted; the merge keeps Git's order.
- Critical 2, index reconcile erases a foreign program's staged content: accepted; superseded by the second review's finding on
  whole-index installs (below).
- Critical 3, client fallback is a second coordinator: accepted; superseded by the fence in the publication transaction (below).
- Critical 4, reclaiming a "proven dead" lock: accepted; no foreign lock is removed.
- High, stacking replays the captured change: accepted (Transaction model).
- High, hold refs leave a window and end with the connection: accepted; superseded by the retention lifecycle (below).
- High, unbounded waits: accepted in part. Rejected in part: the claim that waiting off the commit's path moves the wait onto the
  next batch; recovery uses a single try for the lock and never blocks a batch (`service_land.rs`, `recover_selected` calls
  `try_acquire` and leaves the intent open).
- High, native fallback is not end to end: accepted; gates and a complete Git implementation, and failure independence is not claimed.
- High, hooks, required filters and `paths-in-use` unclassified: accepted; decided by the owner.
- Git source qualifications: accepted ("one `lstat` per tracked file" is too categorical; a failed `update-ref` is classified by
  its message; the lock watcher is files-backend only; `core.lockfilePid` proves only what it records; the port of
  `update_index_from_diff` needs differential tests).

## Answers to the second review

Each finding of `~/temp/agent/reviews/commits-never-fail-sol-2.txt`.

- Critical, epoch fencing does not fence an obsolete worker: accepted. The reviewer's execution is right: a check that precedes the
  swap proves nothing. The fence is now a ref verified inside the publication transaction, whose refs are all locked in
  `prepare` (Transaction model, "Fencing"). Atomic renames of state files do not fence either; state files are exclusive,
  monotone caches of facts, and only the ref transaction is fenced.
- Critical, carry-forward loses writes through an open descriptor: accepted. A rename does not freeze an inode. The protocol now
  pins the inode by a hard link, exchanges with `RENAME_EXCHANGE`, re-reads the exchanged-out inode, and states the supported
  semantics; a late write goes to the saved file (Merges).
- Critical, installing an index built off the lock erases foreign staging: accepted. The delta is applied to the index read under
  the lock. On provenance: equality is a conflict guard, not provenance; the index checksum recorded at the publication detects
  that a program wrote the index after it; an entry equal to the old one in a changed index is left (the worktree keeps the
  content, so nothing is lost). I do not claim more than that (Transaction model, "Installing into the current index").
- Critical, recovery infers "never published" from reachability: accepted. A receipt ref is created in the publication
  transaction; recovery decides from the receipt; a commit is never resurrected after a deliberate reset. A crash in the
  microsecond window of the files backend on a repository without a branch reflog leaves the commit saved under its receipt ref
  and says so (case 4).
- High, acknowledgement conflated with `closed`: accepted. Acknowledgement is a flag; `closed` means all obligations are
  discharged; recovery has an explicit case per state; the persistence ordering for spool files, records and directories is
  listed.
- High, pending needs a logical head and a durable order: accepted, with the reviewer's `value` and delete examples.
  Capture is against the newest pending commit; order is `queue/<n>` created exclusively under the fence.
- High, the spool must hold the complete replay closure: accepted; the closure is enumerated, the base is held before any read,
  and a prune is tested at every interval.
- High, the side ref can meet a second foreign lock; an unchanged tip does not prove a lock: accepted. The failure is classified
  from the actual message; pending does not depend on a ref write; reftable is stated.
- High, remaining Git hangs: accepted in part. `update-ref` is made safe to run twice by the receipt and is restarted like the
  other children. A deterministic Git bug cannot be restarted away; an in-process transaction writer for the files backend is
  named as the alternative and not chosen. The exit inventory is a gate, as the reviewer asks.
- Medium, pending refs kept from pushes by assertion: accepted. Enforcement is at the wrapper's push boundary for the auto-push and
  explicit refspecs; a `--mirror` push leaves wrapper refs out with a line saying so (owner, below).
- Foreign `index.lock` wait: the owner decided "Same as the ref lock"; the commit and the merge no longer wait on it. The
  carry-forward loop is bounded at two rounds. The reviewer's point that a wait is not a completion guarantee stands: the
  pending request and the lagging index complete only when the foreign program lets go, and the record says so.

## Exits that stay nonzero

- Policy errors at error severity, an empty message, and a content conflict no resolver settles (the merge stops with the
  conflict recorded, and a replay conflict keeps the captured inputs).
- A failing hook or a failing required clean filter (owner): the commit ends with its own message and changes nothing.
- The causes the owner listed as outside our control, with a clear message, and work already handed over finished by the next
  run.
- No longer here: a commit that names a conflicted path, a plain commit with unmerged files and `commit -a` over unmerged files
  (committed with a warning); a live foreign ref lock or `index.lock` (the commit lands or is saved, exit 0); a merge that would
  overwrite unsaved edits (the edits are carried forward).

## Questions for the owner that this revision raised, and the answers

All three were answered on 2026-11-03 (question tool):
- A branch that a foreign program keeps moving: "Save on the side ref (Recommended)" (the request becomes pending, exit 0).
- A `--mirror` push while wrapper refs exist: "Leave out wrapper refs (Recommended)" (everything except `refs/cli-git/*`, with a line).
- An amend while commits are pending: "Amend the pending commit (Recommended)" (the amend replaces the newest pending commit).

## Verification plan

Nothing past the red controls and the conflicted-file changes is built.
Before building: the two-worker model of the fenced publication; the receipt recovery cases under crash injection; the prune
controls at every interval; the index delta against Git on a large index with concurrent foreign staging; the carry-forward
controls with open descriptors; differential controls for each port; the exit inventory; measured times of each step with
`strace`.
Each item then keeps its red control, then the change, then a guard shown red once.
