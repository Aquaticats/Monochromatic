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

The second cross-vendor review (below, "Answers to the second review") found that the first transaction model still allowed a
duplicate publication, lost staging, lost edits in the carry-forward, and a resurrected commit.
This section is rewritten in the reviewer's order: fencing, publication and its receipt, the retention lifecycle, installing
into the current index, then the rest.
It is a design to be checked, not a proof; every claim marked "to test" needs a model or fault-injection control before code ships.

### Principle

Only one operation needs fencing: the ref transaction that publishes.
Everything else is made harmless for an obsolete worker by being idempotent, monotone and verified from an authoritative source:
objects are content-addressed, index installs are conditional, and a record state is a cache of a fact that was true when it was
written and never becomes false.

### Fencing: the fence rides in the publication transaction

Ownership of a worktree's lane is a value of a ref, `refs/cli-git/lane/fence`, that points to a small blob naming the owner and a
number (the epoch).
A worker takes over by a compare-and-swap of that ref from the epoch it saw to the next one.
The publication is one `update-ref --stdin` transaction that contains, together:

```
start
verify refs/cli-git/lane/fence <epoch blob of this worker>
create refs/cli-git/receipt/<id> <published commit>
update refs/heads/<branch> <published commit> <old tip>
prepare
commit
```

`verify` is a command of `update-ref --stdin`.
Every ref in the transaction is locked in `prepare` before any is changed, so a successor's swap of the fence ref cannot interleave:
either it comes after the transaction (the successor then finds the receipt and the new tip) or the transaction fails at
`verify`.
This closes the execution the reviewer gave: worker A passes any earlier check, B takes over, A resumes and swaps; with the fence
inside the transaction A's swap is refused if B changed the fence first, and if A's transaction came first B finds A's receipt and
does not publish the request again.
No check precedes the swap that the swap does not repeat atomically.
The old three-step order (read tip, check epoch, swap) is withdrawn.

Takeover happens only when the owner's progress counter has not advanced for a lease period and the owner process shows no sign
of work (the push meter); the counter is written by the landing task after each phase, never by a timer thread.
A worker's other side effects after losing the fence are harmless by the principle above, and its Git children are ended with
it (a service is a unit; the manager ends the unit's whole control group; a client-owned lane ends with the client's process
group).

To test: a model of two workers with takeover at every point of build, `verify`, `prepare`, `commit`, and a fault-injection
control that holds A inside `prepare`.

### Publication receipt and recovery

The receipt is `refs/cli-git/receipt/<id>`, created in the same transaction as the branch update, naming the published commit.
It is the durable record that publication happened, so recovery never infers it from reachability.

- Reftable repositories commit the transaction atomically.
- The files backend locks every ref in `prepare` and then renames them in the order they were queued in the transaction
  (`refs/files-backend.c:3348` loops over the updates in queue order; to verify with a control). The receipt is queued first, so
  a crash in the instant between the renames leaves the receipt without the branch move, never the branch move without the
  receipt.

Recovery, per request, in this order:
1. The receipt exists and the tip equals the published commit, or the commit is reachable from the tip: published.
2. The receipt exists and the tip is the old tip and the branch's reflog has no entry for the commit: the branch update did not
   happen. Complete it by one fenced swap.
3. The receipt exists and the tip is otherwise: the commit was published and the branch was later rewritten (a deliberate reset, a
   force update). Published; it is not published again.
4. The receipt exists, the tip is the old tip and the reflog is missing or disabled: the two cases cannot be told apart. The lane
   does not publish again; it keeps the commit under its receipt ref, and says plainly that the commit is saved there and the
   branch does not show it (the commit is never lost and never resurrected).
5. No receipt: not published. The lane publishes it under the fence, or, if the tip moved, replays it on the tip.

A commit is never resurrected after a deliberate reset by recovery, because a receipt, not reachability, decides.
The cost is the case 4, for a crash in a microsecond window on a repository with no branch reflog.

### Request record, states, durability

A request is a durable record with a state, one directory per request id under the worktree's service directory.
The states are: `captured` (client-owned), `handed-over` (lane-owned from here), `pending` (below), `published`, `materialized`,
`closed`.
`closed` means only that every obligation of the request is discharged: published, materialized, and the retention (spool, hold
and receipt refs) released.
Client acknowledgement is orthogonal: it is a flag on the record and never changes the state or releases retention.
A client that has exited, and a command that has answered `pending`, leave the record open.
Cleanup runs from `closed` only, and recovery has an explicit case for each state: `captured` whose client process is gone is
removed (its client never answered success); `handed-over`, `pending` and `published` resume at their next obligation.

State files are created exclusively (one file per state name), so a stale worker can only create a state whose fact was true when
it created it; readers take the furthest state.

Persistence ordering for handover, which the guarantee depends on (an atomic rename is atomic, not durable):
1. write each spool file, fsync it;
2. fsync the spool directory;
3. write the record's state file to a temporary name in the record directory, fsync it, rename it into place;
4. fsync the record directory, then its parent directory;
5. only then answer the client that the request is handed over.
The ref transaction of publication uses Git's own ref durability (`core.fsync` for references, set by the lane for the commands it
runs itself); the receipt is as durable as the branch update.

### Retention lifecycle and the spool closure

At capture, in this order, so a prune cannot take anything the request needs before it is handed over:
1. read the branch tip `B` and create `refs/cli-git/hold/<id>/base` pointing at `B`; check that `B` is still readable (if not,
   read the tip again; nothing was handed over yet, so starting again is allowed);
2. read the worktree and the real index;
3. write the spool and fsync it (above).

The spool is a self-contained closure for replay, not only the new bytes:
- the bytes of every captured file, the paths, modes, and the expected pre-commit index entry (stage, mode, object) of every path;
- for each path, its content at the captured base (the "before" side of the three-way replay) and the trees along the path from
  the base commit;
- for a conclusion, an amend or a revert: the captured base commit, the other parent's commit and the trees and blobs the replay
  needs, the captured resolved tree, the message and the identities.
It is written as a pack of those objects plus the files, so a reimport restores them even if the repository dropped them.
The lane imports the spool into the object store with one `fast-import` stream that also writes the hold ref
`refs/cli-git/hold/<id>/commit`; anything a prune removes before `closed` is imported again from the spool.
Retention is owned by the record, not by the connection: it ends at `closed`, so a disconnect does not release it.
To test: a prune of every kind between capture, import, hold creation, publication, materialization and closure.

### Installing into the current index

The first revision built a new index beside the real one with no lock and installed it later; that can erase staging written by
another program in between (the reviewer's `x=C, y=A` over `y=B`). That is withdrawn.

The index is changed only as a delta applied to the index read while the lock is held:
1. take `index.lock` with a single try;
2. read the current index (in process, into entries, keeping the version and every extension byte for byte, and dropping only the
   cache-tree entries of the changed directories);
3. for each path of the request, apply the entry rule below to the current entry, and leave every other entry untouched;
4. write the lock file and rename it over the index, which also releases the lock.
In process this is milliseconds for an index of tens of thousands of entries. No Git child runs under the lock. Where the port is
not eligible (split index, sparse index, an extension it does not carry, an index version it does not write), the lane runs Git on
a copy made under the lock (`reset` with `--no-refresh`), which is the slower hold the service has today, and this is the only
case where a long hold remains.

The entry rule, per path, with `E_pre` (the entry at capture), `E_commit` (the entry the commit gives) and `E_now` read under the lock:
- `E_now == E_commit`: nothing to do;
- `E_now == E_pre` and the index file's checksum is the one recorded at the publication: install `E_commit` (nobody wrote the index
  between the publication and now);
- `E_now == E_pre` and the checksum changed since the publication: leave the entry. A program wrote the index after the commit was
  published, and an entry equal to the old one may be a deliberate re-stage; the path is reported as left as staged by another
  program (the worktree still holds the person's content, so nothing is lost);
- anything else: leave `E_now`, and report the path.
Equality of entries is a conflict guard and not provenance of who staged it; the checksum comparison is the provenance the file
format offers, and the case left is the one where it cannot be told, resolved in favor of the other program's staging.
Open intents are applied in publication order. The next batch never waits for them: recovery uses a single try for the lock
(`service_land.rs`, `recover_selected`) and leaves the intent open.

The wrapper's own index commands do not wait for the lane's intents behind a foreign lock: they run as Git runs them, and the
conditional rule keeps whatever they stage. They wait only for the lane's own hold of the lock, as today (a hold that is short
now).

### The side ref for a held branch lock, with the logical head

Decided by the owner for a live foreign program that holds the branch's ref lock.

What the lane does.
The publication transaction above is tried once. If it fails, the lane reads the actual failure and classifies it:
- the lock on `refs/heads/<branch>` exists (`unable_to_lock_message`, `refs/files-backend.c:859-866`; Git waits only 100 ms by
  default, `refs.c:996-997`): a held lock;
- the old value did not match (`cannot lock ref ... is at X but expected Y`): the branch moved;
- no space, I/O error, read-only: the causes the owner listed as outside our control;
- any other message: unclassified; the lane keeps the request `handed-over`, reports the message, and retries at the next
  change of the repository; it never calls an unchanged tip a held lock (review finding).
For a held lock, or a branch that moves faster than the replay can follow, the request becomes `pending`: the commit is built and
imported, the record names it, and the command answers success: "saved; the branch `<name>` will show it when the other program
lets go".
Decided by the owner (question tool, 2026-11-03) for a branch that a foreign program keeps moving: "Save on the side ref
(Recommended)". After the swap is lost to a foreign writer, the commit is saved under `refs/cli-git/pending/<id>`, the command
exits 0 saying so, and the service lands it in order when the branch lets it.

Pending does not depend on a ref write.
The pending commit is held by the spool and the record (the guarantee), and by `refs/cli-git/pending/<id>` when the ref store
lets the lane write it (the files backend creates a new loose ref without taking the branch's lock or the packed-refs lock).
On reftable, where the store has one lock for every ref and a foreign holder blocks any ref write, the side ref is not written
and the spool and record carry the commit; the lane writes the ref when the store is free.
A foreign lock on the side ref or the hold ref is not expected (the namespace is the lane's own) and, if it occurs, changes
nothing: the request is pending by its record.

Logical head and order.
While a request is pending, the next commit of the worktree is captured against the logical head, which is the newest pending
commit (or the branch tip when none is pending), not against the physical tip.
The private index starts from the logical head, so adds, edits and deletions are computed against what the person last committed:
- R1 adds `f` and is pending; R2 deletes `f`: R2 is captured against R1's commit, where `f` exists, so it records the deletion and
  replay keeps it;
- R1 sets `value=1`, then the person sets `value=2` and commits R2: R2's base is R1's commit, so the change is `1` to `2`, not
  `0` to `2`, and no false conflict appears.
Order is durable: a request takes the next number by creating `queue/<n>` exclusively under the fence; the number is its place,
the number is recorded in the record, and the lane publishes strictly in that order. A request that finds its captured logical
base is no longer the newest is captured again against the newest before it takes a number (its bytes are the same; only the
base changes). A branch can be checked out in one worktree only, so the order is per worktree.
An amend, a conclusion or a revert while commits are pending acts on the logical head, because that is the commit the person just
made; the command says so. Decided by the owner (question tool, 2026-11-03) for an amend: "Amend the pending commit
(Recommended)": the amend replaces the newest pending commit before it lands.

Publishing a pending request.
When the lock is free the lane publishes the oldest pending request: if the tip equals the captured logical base's published
commit, the pending commit itself is the new tip; if the tip moved, the lane replays the captured change on the tip (below). Both
use the fenced transaction.

Pushes.
Pending, hold and receipt refs are never pushed: the auto-push passes an explicit branch refspec, and the wrapper's push boundary
(`bounded_push.rs`, the forwarded `git push`) rejects, with a message, an explicit refspec whose source or destination names
`refs/cli-git/`.
A `--mirror` push mirrors every ref under `refs/` by definition, so left alone it would send them. Decided by the owner (question
tool, 2026-11-03): "Leave out wrapper refs (Recommended)": the mirror push sends everything except `refs/cli-git/*`, with a line
saying that those refs were left out and why. The enforcement is at the wrapper's push boundary, which excludes the namespace
(for example by adding a negative refspec for `refs/cli-git/*` to the push it runs) and prints the line; a control must show that
a mirror push, an explicit refspec and the auto-push send none of them.

### Replay of the captured change on a moved head (owner's choice: stack on the new head)

The result is built from the captured change, not from the captured tree.
Let `B` be the captured base (the logical head when it was captured), `T` the captured resulting tree (or resolved tree for a
conclusion) and `C` the head to publish on. The new tree is the result of a three-way merge with base `B`, ours `C`, theirs `T`
(`git merge-tree --write-tree --merge-base=B C T`, or the in-process equivalent that matches it), so whatever `C` added stays
and the person's change is applied on top. A conflict in this replay is an allowed outcome: the replay stops with the conflict
recorded and the captured inputs kept. The commit's parents are `C` (and `T`'s other parent for a conclusion).

## What is guaranteed to complete, and what may wait

No wait below is called designed out.

Guaranteed to complete, with no wait on another program and no cap or count:
- capture and handover (local reads, one fsynced spool write; failures are the owner's outside causes: a full disk, I/O errors);
- the command's answer: `published` or `pending`, after a single prompt attempt (a Git child that stalls is restarted; the receipt
  makes the publication transaction safe to run twice);
- starting without the service (the client can own the lane under the fence);
- the answer after a crash (asking again returns the state, decided by the receipt);
- retention of everything the request needs until `closed`;
- the index catching up after a commit when the lock is free (milliseconds, in process, no Git child under the lock).

May wait, off the command (the command has already exited 0), with a notice that names what is waited for:
- publishing a pending request, for as long as a live foreign program holds the branch's ref lock, or keeps moving the branch;
  if the holder never lets go the commit stays pending, intact, and the branch never shows it;
- the index catching up, for as long as a live foreign program holds `index.lock`; owner (question tool, 2026-11-03):
  "Same as the ref lock (Recommended)": the commit lands on the branch and the command exits 0, saying the index will catch up
  when the lock is free; a merge saves its result on a side ref, exits 0 and finishes the checkout when the lock frees.

May wait on the command: nothing on a foreign program.
The remaining waits are the person's own: an editor, a signer's prompt, a hook.
A compare-and-swap is not starvation-free, which is why a branch that a foreign writer keeps moving makes the request pending
(decided by the owner, above) instead of looping in the command.

Hung Git children.
The children that remain on the commit path are few: `fast-import`, `commit-tree`, `merge-tree`, `update-ref`, and the commit
Git builds for signed or hooked commits.
All but the last are idempotent: the objects are content-addressed, and `update-ref` is safe to run again because the receipt
makes the publication run at most once.
So a child that shows no sign of work for its window is stopped and started again (the retry-based attempt on this branch,
kept for this purpose), including `update-ref`.
A Git bug that hangs the same input every time cannot be restarted away; for the files backend the alternative is an in-process
writer of the transaction (the lock-file protocol, the receipt, the reflog line), with differential controls against Git, which
this record names as the fallback and does not choose.

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
