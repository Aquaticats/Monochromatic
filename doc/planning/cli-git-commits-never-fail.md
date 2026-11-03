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
the fix lists the unmerged paths of the real index before the staging and warns for those it commits.

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

The cross-vendor review (below, "Answers to the review") found the central gap: the first version removed failure returns one
step at a time and never defined the whole transaction.
This section is that definition.
It is a design to be checked, not a proof: the claims marked "to test" need fault-injection controls before any code ships.

### States of a request

A request is a durable record with a state, written by atomic rename, one file per request under the worktree's service
directory (the journal's directory today).
The states, in order:

- `captured`: the client has read the worktree and index and written the immutable inputs (below).
  The client owns it. A client that dies in this state (its process identity, pid plus start time, is gone) abandons it, and the
  lane removes the record and its retention.
- `handed-over`: the client has marked the record complete; from here the record belongs to the lane and survives the death
  of the client and of the connection (the owner: work already handed over stays on disk and the next run finishes it).
- `pending`: the commit exists under the side ref `refs/cli-git/pending/<id>` because a live foreign program holds the
  branch's ref lock (below); the command has already answered success. It becomes `published` when the branch is moved onto it.
- `published`: the commit exists on the branch. The record names the commit and the ref value it replaced.
- `materialized`: the index entries (and, for merges, the worktree files) are brought to the new head by the rules below.
- `closed`: the client acknowledged, or no client waits any more; retention is released.

### Immutable captured inputs

At capture the client writes, outside the object store's collectible storage, a spool directory named by the request id:
the bytes of every captured file (not only their object names), the paths, modes, the expected pre-commit index entry of
every path (stage, mode, object), the captured base, the message, the identities, the options that decide the commit, and for
a merge, a revert or an amend the captured base, the captured resolved tree and the original parents.
Nothing after capture reads the worktree again for this request: the worktree is not the request's source of truth, the spool is.
The lane imports the spool into the object store with one `fast-import` stream that also creates the hold ref
`refs/worktree/cli-git-hold/<id>`; if a prune removes any of those objects at any time before `closed`, the lane imports them
again from the spool, so the bytes committed are the captured ones and never a later edit.

### Request identity, ownership and fencing

- The request id is a random 128-bit value made by the client and written in the record; a resend carries it, and every
  answer is a function of the record's state, so a lost answer is recovered by asking again.
- One lane per worktree publishes requests. Ownership is an epoch: a worker (the service, or a client that finds no live
  service) becomes the owner by creating `lane/epoch-<n+1>` exclusively (`O_EXCL`), where `<n>` is the highest epoch present;
  the file records the worker's process identity. The highest epoch is the owner. No lock is held.
- A worker takes over from an owner only when the owner's progress counter has not advanced for a lease period and the owner
  process shows no sign of work (the push meter of `push_progress.rs`). The counter is written by the worker's landing task
  after each phase of the work (objects imported, intent written, ref published, entries installed), never by a timer
  thread, so a live heartbeat with a wedged landing is not mistaken for progress.
- Every publication follows one order, which excludes the same request landing twice at successive heads:
  1. read the branch tip `h`;
  2. check that this worker's epoch is the highest and that the record is not `published`;
  3. compare-and-swap the ref from `h` to the new commit.
  If another worker published the request before step 1, its epoch claim came before its swap, so step 2 fails. If it
  publishes after step 1, the tip is no longer `h` and step 3 fails. A worker whose step 2 or 3 fails never rebuilds on the new
  head without checking step 2 again. (To test: a model of two workers with every interleaving of the three steps, and a
  fault-injection control that stops the older worker between each pair of steps.)
- The journal's numbering is per request (`<id>.<epoch>`), so no shared counter is contended.

### Publication

The compare-and-swap of the ref is the publication, and nothing else is: it is the one atomic point at which the commit
exists.
The record is marked `published` immediately after; recovery decides `published` from the ref (the commit named by the record is
the tip or an ancestor of it), so a crash between the swap and the mark loses nothing.
A commit Git must build (signed, hooked, editor) is built after the person's message is fixed, inside a branch turn: the
client asks the lane for the turn (an epoch-fenced lease on the branch, held for the build and the swap only), and a build begun
before the turn is built again in it.

### A branch whose ref lock a live foreign program holds: the pending side ref

Decided by the owner. When the publication's `update-ref` fails because `refs/heads/<name>.lock` exists
(`refs/files-backend.c:846-860`; Git waits only 100 ms by default, `refs.c:996-997`), the lane does not wait.
It tells a held lock from a moved head by re-reading the ref: tip equal to the expected one means the lock is the cause.
Then:

1. The commit object is already built on the tip `h`. The lane stores it with a compare-and-swap on a ref of its own,
   `refs/cli-git/pending/<id>` (a different lock path, so the foreign holder does not block it), and marks the record `pending`.
   That ref also serves as the hold of the captured objects.
2. The command exits 0 and says: the commit is saved, the branch `<name>` does not show it yet because another program holds the
   branch's ref lock, and the branch will show it when that program lets go.
3. The lane watches the branch's lock path (files backend; reftable repositories retry with Git's own lock handling) and, when
   the lock is free, publishes pending commits strictly in request order, one at a time, each by the publication order above.
   If the tip is still the commit's parent, the pending commit itself becomes the tip; if the tip moved, the lane builds the
   final commit by replaying the captured change on the tip, exactly as for any moved head. A later request that arrives while
   an earlier one is pending joins the queue behind it, so the branch never gets a later commit before an earlier one.
   A second request that changes the same file as a pending one captured the first one's edits too, and replaying it on a tip that
   already holds the first is a clean three-way merge (both sides made identical changes).
4. The index and worktree are not materialized for a pending commit: nothing was published, so the person's index and files stay as
   they were, and the conditional install of the next section runs when the branch has been moved.
5. Pending commits are never pushed. The auto-push pushes only the branch it was handed, and the wrapper's push refspecs name no
   `refs/cli-git/` ref; a control must check that a push with an explicit or mirror refspec does not send them.
   A commit is pushed only after it landed on the branch.
6. If the holder never lets go, the commit stays on the side ref, intact and recoverable; the record says so, and the wrapper's
   status notices name the pending commits. Nothing is lost and nothing waits.

This is what the foreign lock costs: the command's answer is "saved", not "on the branch", until the other program is done.
It replaces the wait that was open before.

### Materialization with provenance

Commits.
The index entries are brought to the new head by a conditional install per path, never by snapshot equality.
For each path the record holds the expected pre-commit entry `E_pre` (from capture) and the commit's entry `E_commit`.
Under the index lock, for each path with the current entry `E_now`:
- `E_now == E_pre` (or the entry an earlier request of this lane installed): install `E_commit`;
- `E_now == E_commit`: nothing to do;
- anything else (another program staged different content, or changed stages): keep `E_now`, and report the path.
So a foreign program's staged content is never replaced by ours, and the check is per entry, made while the lock is held.
Unmerged stages follow the same rule. Open intents are applied in publication order. The reconciler never blocks the next
batch: it already uses a single try for the lock (`try_acquire`) and leaves the intent open for the next round.
The worktree files of a plain path commit are not touched by the commit; files a policy rewrote are installed after the commit
by the existing checked install.

Merges.
The checkout cannot be deferred past the check that protects the person's edits, and a checkout after publication cannot be
rolled back safely once another commit descends from the merge.
So a merge keeps Git's order: the worktree and index are materialized first, by Git's own two-way tree update under
`index.lock` (which checks each path against the index immediately before it writes it, the check Git itself makes), and the
ref is swapped last; a swap that loses is followed by the existing restore of the merge's paths, which restores only paths still
holding the old, merged or moved-head content. The merge does not move the ref first. (This answers review finding 1.)

### Acknowledgement

The lane answers `Landed` after `published` is durable; a client that did not get the answer asks again and receives the same
answer.
The wrapper's own index commands wait for open intents of this worktree before they run (`service_hold_retry.rs`), so a person
using the wrapper sees a consistent index.

### Recovery

At start, at each takeover, and before each batch the lane scans the records:
- `captured` whose client is gone: remove;
- `handed-over`, not published: decide from the ref; if the owner is gone, import from the spool and publish under the order
  above, parents set to the current tip by replay of the captured change (below);
- `published`, not `materialized`: apply the conditional install;
- `closed`: remove spool and hold.
Every step is idempotent, and every record file is replaced by atomic rename.

### Replay of the captured change on a moved head (owner's choice: stack on the new head)

The result is built from the captured change, not from the captured tree.
Let `B` be the captured base, `T` the captured resulting tree (or resolved tree for a conclusion), and `C` the new head. The new
tree is the result of a three-way merge with base `B`, ours `C` and theirs `T`
(`git merge-tree --write-tree --merge-base=B C T`, or the in-process equivalent that matches it), so whatever `C` added
(the lexer) stays and the person's change (the amend, the manual resolution, the revert's edits) is applied on top.
A conflict in this replay is an allowed outcome: the replay stops with the conflict recorded and the captured inputs kept.
The commit's parents are `C` (and `T`'s other parent for a conclusion).

## What is guaranteed to complete, and what may wait

No wait below is called designed out.
A wait that can last for ever when a live foreign program never lets go is listed as such.

Guaranteed to complete (no wait on another program, no cap, no retry count):
- capture and handover: local reads and one spool write; they fail only for the causes the owner listed as outside
  (a full disk, I/O errors);
- starting without the service: the client can become the lane owner;
- a request's answer after a crash: asking again returns the state;
- retention of captured bytes until `closed`;
- the answer of a commit: `published` when the ref lock is free, `pending` (saved, exit 0) when a live foreign program holds it;
  the publication itself is a compare-and-swap that a foreign writer who keeps moving the branch can delay (below).

May wait (with a notice naming what is waited for and the holder):
- the branch moving onto a pending commit while a live foreign program holds the branch's ref lock: the commit itself does not
  wait (it is saved on the side ref and the command exits 0); the lane waits for the release, off the commit's path. If that
  program never lets go, the commit stays on the side ref and the branch never shows it. Decided by the owner.
- publication of one request against a foreign writer that wins immediately before every swap: lane-routed requests are served
  first in, first out, and the foreign writer's commits are each progress for the repository, but a compare-and-swap is not
  starvation-free for one request. The notice names the foreign tip each round.
- materialization of the index while a foreign program holds `index.lock` (commits: off the commit's path; merges: before the
  swap). The record stays open and the wrapper's own index commands wait for it.
- the remaining Git children (`fast-import`, `commit-tree`, `update-ref`, `merge-tree`) if one hangs: the idempotent ones are
  stopped and started again after a stall (the retry-based attempt on this branch, kept for exactly this containment of a Git
  bug); `update-ref` is never stopped, so a hung `update-ref` waits.
- a merge that has unsaved edits in files it changes: it does not wait or refuse; the edits are carried forward (below).
- local hooks and required clean filters: when one fails the commit ends with its message and changes nothing; this is an
  allowed refusal, decided by the owner, not a wait.

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
a hung `update-ref` is never stopped and waits (review finding 7, accepted).

## Item 2: the service does not start within `START_WAIT`

Root cause: the commit depends on a second process that the manager must start, and the client gives it ten seconds.
The commit's own work is complete before the service is asked.

Change: the service stops being a correctness dependency.
The client hands the request over by writing the record (state `handed-over`) and then asks the manager to start the instance.
If no live owner of the lane answers, the client claims the next epoch and acts as the lane owner for this worktree under the
ownership order of "Transaction model", with the same code and the same record, until the service takes over by claiming a
higher epoch.
No second coordinator exists: ownership is the epoch, and the three-step publication order fences the older owner (review
finding 3, accepted).

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
A lane whose counter has not advanced for a lease period and whose process shows no sign of work is taken over by a higher
epoch, so a wedged service neither fails the commit nor lands it twice.
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
- For a commit Git must build (signed, hooked, editor) the client takes a branch turn (an epoch-fenced lease, "Transaction
  model") after the message is fixed and holds it for the build and the swap only.
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

Root cause: the landing needs the real index lock to install the updated index, and a foreign program can hold it for as long as
it likes; after 1800 s, or at once when no inotify watch can be set, the commit fails although the commit itself did not need
the index.

Change: the commit's publication does not take `index.lock`.
The ref is swapped (the publication), and the index is brought to the new head afterwards by the conditional per-entry install
of "Transaction model, materialization": an entry is replaced only if it still holds the entry captured before the commit or
the commit's own entry, so another program's staged content is never erased (review finding 2, accepted).
The new index is built beside the real one with no lock, the lock is taken with a single try when it is free (the wait is a
watch on the lock path set before the try, re-checked on every wake, with a periodic re-check when the watch fails, so a
missed event or a replaced file cannot stall it), and the installed file is checked against the entry rule above.
The reconciler never blocks the next batch.

No foreign lock is ever removed (review finding 4, accepted; the earlier text that removed a lock whose holder was "proven dead"
is withdrawn).
A PID or an open descriptor does not prove that the pathname still names the abandoned lock (it may have been replaced by a live
writer's lock since), and a dead writer's lock can hold a complete index not yet installed, which is the person's data.
`core.lockfilePid` (`lockfile.c:90,184`) records an owner but does not prove the pathname is unchanged.
A lock that nobody releases is waited for with a notice naming the file and what is known of its holder.

May wait: the reconciliation of a commit's index, for as long as a foreign program holds the lock, and the commands of the
wrapper that need a settled index.
The commit is on the branch and acknowledged; the index lags for the paths of the commit (a raw `git status` shows them as
staged changes against the new head) until the lock is released.
If the holder never releases it the lag lasts as long as the holder holds the lock; the owner's decision on a held lock
covers the branch's ref lock (saved on a side ref) and not `index.lock`, so this wait remains a wait and is listed as one.

## Item 6: `ContentPruned`

Root cause: the objects a commit captured are unreferenced until the branch moves, and `git gc --prune=now` deletes every
unreferenced object at once; the commit holds nothing that keeps them.

Change: the bytes are not in collectible storage until the commit is published (review finding 6, accepted).
The spool holds the captured bytes (immutable inputs); the lane imports them into the object store with a hold ref made in the same
`fast-import` stream; any prune before `closed` is repaired by importing again from the spool, so the committed bytes are the
captured ones.
The earlier plan to prepare again from the worktree is withdrawn: it would commit a later edit.
The hold is owned by the record, not by the connection: it lasts until the record is `closed`, so a client that disconnects after
handover does not release it.
To verify before relying on it: that `refs/worktree/*` holds are kept by `gc` in every linked worktree, and that a push with an
explicit or mirror refspec does not send them (Git's source, `refs.c` per-worktree ref handling, and a control).

Residue: none for pruning.
The spool itself is on the same disk, so a full disk or an I/O error while writing it is one of the owner's outside causes.

## Merges, by the same lens (design only)

The merge-fixes branch waits for any `index.lock` holder before the merge and computes the merge again on a moved head up to
twenty rounds (`merge_isolation_moved.rs`).
The first version proposed publishing the merge first and checking out afterwards.
The review showed that a deferred checkout can overwrite an edit made after the check, and that a rollback after another commit
descends from the merge would rewrite someone's work (finding 1, accepted).
So the order stays Git's: materialize under the index lock with Git's own per-path check, then swap the ref.

- The wait for `index.lock` before the checkout stays, and is a wait that can last as long as a live foreign program holds the
  lock. The owner's decision on a held lock covers the branch's ref lock only, so this one is still a wait and is not called
  designed out.
- A merge whose final swap finds the branch's ref lock held by a live foreign program follows the owner's decision: the merge
  commit is saved under `refs/cli-git/pending/<id>`, the command exits 0 and says the branch will show it when the other program
  lets go, and the lane moves the branch onto it in order. The worktree and index already hold the merged result (the checkout came
  first), so until then `git status` shows the merge's changes as staged against the old head; nothing is lost.
- The twenty rounds: a lost swap is another writer's commit, so the count is artificial; run the merge as an item of the
  worktree's lane (first in, first out with commits and the push), so the head moves under it only for a foreign real-Git
  writer, and remove the count. Not starvation-free against a foreign writer.
- `merge-isolation/paths-in-use` stops being a refusal. Decided by the owner ("Carry edits forward"): the merge lands, and each
  file the person edited and the merge changes gets a three-way merge of the person's edit onto the merged result.
  The carry-forward, per such path, under the index lock and before the ref swap (the order above), keeps the review's
  safety condition that a file edited after the check is not overwritten:
  1. the base is the old head's content `O`, "ours" is the person's file `E`, "theirs" is the merged result `M`; the result `R`
     is Git's own three-way text merge (`git merge-file`, or an in-process port that matches it, with the repository's conflict
     style and labels naming the person's edit and the merge);
  2. the person's file is moved aside by one atomic rename (so the inode that holds the latest bytes is the one examined,
     whatever an editor wrote a moment ago), its bytes are compared with the `E` that was merged, and, if they differ, the merge
     is made again from the moved file; a copy of the person's file is also kept in the wrapper's saved-copies store
     (`discarded_store.rs`) before `R` is installed, so nothing can be lost;
  3. `R` is installed at the path with a no-replace link; if a new file has appeared at the path meanwhile, it is merged in the
     same way, and the round repeats until the path is quiet (each round is local and cheap);
  4. the index receives `M` for that path, so the person's edits show as ordinary unstaged changes on the merged result;
  5. where `E` and `M` collide, `R` holds conflict markers with both sides, and a line names each such file and says the markers
     are in the worktree and not committed.
  A file that cannot be merged as text (binary, a mode or type change, a symlink, a directory in the way, an untracked file that
  collides with a file the merge adds) keeps the person's version at its path, the merged version is written beside it under a
  name the line gives, and nothing is overwritten; the exact naming is to be settled with the controls when this is built.
  The merge itself does not change worktree files it does not change in the merge.

## Residues, by class

- A malformed request.
  Structural errors in a request are detected before the record is written (the client builds it from typed values and validates
  it), and a record written by a different release carries a format version: a worker that does not know it does not process it
  and yields ownership to a worker that does (the newer release wins the epoch). A request that is well-formed but means
  something the lane cannot do is a policy decision, classified below.
- An object format or layout this release cannot read.
  Every in-process port is gated by eligibility (files ref backend, SHA-1 or SHA-256, no sparse index, no split index or unknown
  index extension, attributes and filters covered by the port), and every operation has a Git-command implementation that is
  complete for all layouts: the landing, the reconciliation and the recovery all run through Git when the gate says no.
  Reftable repositories land through `update-ref --stdin` (backend-agnostic) with no lock watcher, using Git's own lock handling.
  The ports (hashing with the repository's conversions, index entry writing, `update_index_from_diff`) are correctness
  boundaries, not optimizations: each needs differential controls against Git on attribute precedence, required filters,
  encodings, line endings, index flags and stage handling, and an unsupported case must run Git, never produce different bytes.
- Local hooks, required clean filters, remaining native Git failures.
  A failing hook or a failing required clean filter is an allowed refusal (owner): the commit ends with the hook's or filter's own
  message and changes nothing, the same way a policy error does; falling back to Git runs the same hook or filter and fails the
  same way, which is the intended result. Each retained Git child's other reachable `die` and `error` exits are to be enumerated
  from the 2.56.0 source and assigned to a class: outside list (clear message), designed out, or a refusal. Any exit not assigned
  stays nonzero and is reported to the owner.

## Answers to the review

Each finding of `~/temp/agent/reviews/commits-never-fail-sol.txt`, with a decision.

- Critical 1, deferred merge checkout overwrites later edits: accepted. The merge keeps Git's order, materialize then swap
  (Merges).
- Critical 2, index reconcile erases a foreign program's staged content: accepted. Per-entry conditional install with the
  expected pre-commit entry as provenance (Item 5, Transaction model).
- Critical 3, client fallback is a second coordinator: accepted. One lane, ownership by epoch, a three-step publication order
  that excludes the same request landing twice, a progress counter in place of a heartbeat, and per-request journal names
  (Transaction model, Items 2 and 3). To test with a two-worker interleaving model and fault injection.
- Critical 4, reclaiming a "proven dead" lock: accepted. Reclamation is withdrawn entirely; no foreign lock is removed
  (Item 5). The contradiction in the text is gone.
- High, stacking must replay the captured change: accepted. Three-way replay with the captured base (Transaction model,
  Decided).
- High, hold refs leave a window and end with the connection: accepted. The spool is the source of truth; the hold lasts until
  `closed`; the original bytes, not the worktree's, are re-imported (Item 6).
- High, unbounded waits: accepted in part. A heartbeat is replaced by a phase-written progress counter; compare-and-swap
  rounds are not starvation-free and the record says so; a live foreign lock may never release and the record does not call it
  designed out; the section "What is guaranteed to complete" lists each. Rejected in part: the claim that waiting off the
  commit's path "moves the wait" onto the next batch; recovery uses a single try for the lock and never blocks a batch
  (`service_land.rs`, `recover_selected` calls `try_acquire` and leaves the intent open), which is evidence in the code.
- High, the native fallback is not end to end: accepted. Every operation has a Git implementation complete for all layouts; ports
  are gated (Residues).
- High, hooks, required filters and `paths-in-use` unclassified: accepted, and the owner has since decided them: hooks and required
  filters are allowed refusals; `paths-in-use` becomes a carry-forward of the person's edits; the exit inventory is a design task.
- Git source qualifications: accepted. "One `lstat` per tracked file" is too categorical (`refresh_index` skips entries by
  flags); a failed `update-ref` must be classified by its error and by re-reading the ref before it is called a moved head;
  the lock watcher is files-backend only; the `core.lockfilePid` claim is limited to what it records; the hold ref's
  non-transmission and linked-worktree retention are to be verified; the port of `update_index_from_diff` needs differential
  tests for stages, flags, stat invalidation and extensions; the sibling worktree's fast landing does not isolate the cause of
  the slow ones.

## Exits that stay nonzero

- Policy errors at error severity, an empty message, and a content conflict no resolver settles (the merge stops with the
  conflict recorded, and a replay conflict keeps the captured inputs).
- A commit or merge that names a conflicted path, or a plain commit with unmerged files, does not stay here: it commits with a
  warning (owner).
- The causes the owner listed as outside our control, with a clear message, and work already handed over finished by the next
  run.
- A failing hook or a failing required clean filter (owner): the commit ends with its own message and changes nothing.
- No longer here: a commit that names a conflicted path, a plain commit with unmerged files and `commit -a` over unmerged files
  (committed with a warning); a live foreign ref lock (saved on a side ref, exit 0); a merge that would overwrite unsaved edits
  (the edits are carried forward).

## Verification plan

Nothing past the red controls and the two conflicted-file changes is built.
Before building: the two-worker interleaving model of the publication order; differential controls for each port; the
enumeration of Git's exits per retained child; measured times of each step with `strace`.
Each item then keeps its red control, then the change, then a guard shown red once.
