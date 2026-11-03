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

A commit that names a conflicted path is no longer a refusal.
Owner, verbatim: "Committing a conflicted path is fine. Warn and commit anyway, even if conflict markers will be committed."
That reverses `d8bcd95218`; it is built on `fix/git-policy-cli-commit-conflicted-path-warns` (tip `7ec22c34da`, for 0.5.26).

So item 7 of the starting list is now: policy errors, an empty message and an unresolvable conflict.
A named conflicted path commits with a warning, and a failing signer is outside.
`SPEC.md` says so.

Items 1 to 6 are designed out.
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

## Item 2: the service does not start within `START_WAIT`

Root cause: the commit depends on a second process that the manager must start, and the client gives it ten seconds.
The commit's own work is complete before the service is asked: the client has built the commit, or the change list,
and the objects.

Change that removes it: the client does not need the service to land.
The landing is a function of the library (`land_batch`), and it is correct without a service because its atomic steps are
Git's own (a compare-and-swap of the ref, Git's index lock protocol) and the journal records it.
When no service greets within the first window, the client lands the commit itself with the same function and the same
journal, instead of failing.
The service keeps its role as the single batcher and the owner of after-commit work; it is a speed-up, never a
dependency.
The client still asks the manager to start the instance (the socket is the manager's) so the next commit finds it.

Needed for this to be safe, to settle in the build: a landing by the client and a landing by a service that starts
a moment later must not both write the same batch number or the same request.
The request identity already makes a resend idempotent (`already` in `Landed`), and the ref compare-and-swap decides any
race.

Residue: a platform with no service manager and no way to land in process.
None exists on Linux; the in-process landing covers Windows-like hosts the crate does not build for.
Failures about the machine's setup (another account's service, a program that is missing, a unit file in the way) stop
being commit failures too, because the client no longer needs the service: they become notices that the service is not
running.

## Item 3: the service does not answer within `COMMIT_WAIT`

Root cause: the client treats silence as a fault and a read timeout of ten minutes decides.
A service that is landing a slow commit says nothing, and a queue behind slow commits is progress made by others.
Liveness is not observable from the client, so the client guessed with a clock.

Change that removes it: make liveness observable and drop the clock.
The service sends a heartbeat line at a fixed short interval while it holds a request (it already sends `Position` and
`Waiting` lines), and the client reads with a timeout of a few heartbeats.
Heartbeats arriving means alive and working: the client waits, prints a notice naming the service process and the queue
position, and no cap applies.
A closed connection means the service died: the client resends the same request (identity makes it idempotent) or,
with item 2, lands it itself.
Heartbeats stopping while the connection stays open means a wedged service: the client lands it itself as in item 2,
the ref compare-and-swap keeping a late service from doing it twice.
`WaitedTooLong` and `RECONNECTS` leave the commit path because a dead or wedged service is never a reason to fail
when the client can land.

Residue: none that ends a commit.
A service that is alive and slow makes the commit slow, and the notice says why.

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

- Remove the counts.
  A commit built by the service from a change list is rebuilt on the new tip inside the serialized step with no cost to
  the person (`fast-import` from the current tip; only a path that changed on the branch goes back to be prepared again, as
  now), and the rounds end because each lost round is another commit landing.
- Make the moves rare.
  The service of a worktree is the one writer of that worktree's branch: the commits, the push's reconcile merge and the
  merges the wrapper completes all become items of the same lane (the wrapper already hands them over), so the head moves
  under a landing only for a foreign program that writes the ref with real Git directly.
- For a commit Git must build (signed, hooked, editor) the client asks the service for the branch's turn first
  (a reservation: while it is held, the service lands nothing else on that branch), builds, and lands in that turn.
  A reservation held while a person types in an editor would make everyone wait, so it is taken after the message is
  fixed and held only for the build and the landing; a build that was begun before the turn is built again in the turn.
- A conclusion, an amend or a revert whose head moved while the person worked (their editor was open, other commits
  landed) is a question of meaning, which the owner has decided: the result is stacked on the new head (below).

Residue: a foreign program that writes the branch with real Git directly, faster than a commit can be built.
Real Git offers only the compare-and-swap for a ref, so that program's progress cannot be excluded; each lost round
is that program's commit, and the commit is built again on it with a notice.

### Decided: stack the result on the new head

The owner chose (question tool, 2026-11-03): "Stack on the new head (Recommended)".
Options that rewrote the other session's commits (rebuild on the new head), held the commit until the head was quiet, or
kept failing are dropped.

Setting: `main` is at commit `B`; another session lands `C` on `main` while this commit is built or while the person works.

An amend.
The person typed `git commit --amend -m "fix the parser"` on top of `B`, whose message was "parse the file".
Before: `A <- B (parse the file)`; the other session lands `C (add the lexer)`: `A <- B <- C`.
After: `A <- B <- C <- D (fix the parser)`, where `D` carries the amend's tree change and message on `C`.
`B` and `C` stay as they are, so no commit another session landed is rewritten.
Standard error says why: the branch moved, so the amend was made as a new commit on the new head.

A conclusion.
The person finished a merge of `topic` (tip `T`) into `main` on `B` with `git commit`; the other session lands `C`.
After: `A <- B <- C <- M`, where `M` has parents `C` and `T` and the merge result computed on `C`.
A content conflict that the merge on `C` cannot settle stops it as any merge does, with the conflict recorded.

A revert.
The person ran `git revert B` and then `git commit`; the other session lands `C`.
After: `A <- B <- C <- R`, where `R` reverts `B` on `C`.
The result is stacked on `C`, and stderr names the new head.

What the owner's choice leaves: an amend on a moved head is no longer an amend (`B` stays and `D` is added), so the person sees
two commits where they asked for one, and stderr says so.
This is the one place where the commit does not do what the words of the command mean, chosen by the owner.
The commit is built inside the worktree's lane, from the amend's captured tree and message, so no extra round trips are
needed.

## Item 5: `index.lock` is refused after the service's wait

Root cause: the landing needs the real index lock to install the updated index, and a foreign program (a person's
`git add`, an editor commit, a script) can hold it for as long as it likes; after 1800 s, or at once when no inotify
watch can be set, the commit fails although the commit itself did not need the index at all.
The commit is durable when the ref moves.

Change that removes it: move the ref without the lock, and finish the index afterwards.
The landing does not take `index.lock` before the move.
It writes the intent to the journal, moves the ref by compare-and-swap, answers `Landed`, and then reconciles the index
as a separate step: it builds the new index beside the real one (no lock), takes the lock when it is free (watching
for its release, not polling), checks that the real index is the one it copied, installs, and marks the intent done.
A foreign holder delays the reconciliation, never the commit.
The wrapper's own index commands already wait for an open intent before they run (`service_hold_retry.rs`, the index writers
wait for the service), so a person using the wrapper sees a consistent index; recovery before each batch and at start
already finishes open intents.
This also removes the 119 s hold: Git work no longer runs under the lock.

Other programs' locks are not outside our control, so the reconciliation does not stop at a foreign lock either:
- A lock whose holder is proven dead (Git's own lock PID file, when `core.lockfilePid` is on, `lockfile.c:90,184`, or the
  evidence the wrapper already gathers in `index_lock_evidence.rs`) holds only a partial index of a process that is gone, so the
  reconciler removes it, logs that it did, and goes on.
  The real index file is untouched by Git until the rename, so nothing the person had is lost.
- A lock that cannot be attributed is waited for by the reconciler, off the commit's path, with a notice that names the file
  once per interval.
  The commit is already on the branch.
- The same holds for a ref lock: `update-ref` waits for a held `refs/heads/<name>.lock` only 100 ms by default
  (`refs.c:996-997`, `core.filesRefLockTimeout`) and then fails (`refs/files-backend.c:846-860`), which the landing
  today turns into a failed round.
  The landing waits for the release of the ref lock by watching the file, as it does for `index.lock`, and treats a proven-dead
  holder's lock the same way.

Residue: while a foreign program holds the lock, the index lags the branch for the paths of the commit (a raw
`git status` shows them as staged changes against the new head).
That is the same state Git leaves between its own ref update and index write, extended in time; it ends when the foreign
holder releases the lock, and the service finishes the intent at once.
If the foreign holder is dead and left a stale lock, the index never catches up until the person removes the lock, which
Git itself requires for every index write; the commit is on the branch and nothing is lost.
Nothing foreign is ever removed.
The alternative the owner may prefer, writing a new index and renaming it over the old one with no lock, can lose a
foreign writer's update, which Git's lock exists to prevent, so it is not proposed.

## Item 6: `ContentPruned`

Root cause: the objects a commit captured are unreferenced until the branch moves, and `git gc --prune=now` deletes every
unreferenced object at once (the Git manual says this risks failures when another process writes at the same moment).
The commit holds nothing that keeps them.

Change that removes it: keep them reachable from the moment they exist.
The preparation writes the captured blobs and trees, and the same step makes a hold ref, `refs/worktree/cli-git-hold/<id>`,
that points at a commit whose tree contains every captured blob.
The ref is per worktree, so it is not pushed, and it is removed when the landing is done or the command ends; the service
also removes holds whose client is gone, as it does with scratch directories (the connection is the signal).
`gc`, `prune` and `repack -A` keep what a ref reaches, `--prune=now` included.
The existing checks stay as assertions that now cannot fire.

Residue: the interval between a blob being written and the hold ref being created.
Writing the blobs through one `fast-import` stream that ends by updating the hold ref makes that interval the stream's
last instant, and nothing can be designed tighter, because Git creates objects before it can reference them;
Git's own manual calls `--prune=now` unsafe for exactly this window.
If a prune lands in it anyway, the commit is prepared again from the worktree (nothing is lost, the person's files are
untouched), with a notice naming the cleanup.

## Merges, by the same lens (design only)

The merge-fixes branch waits for any `index.lock` holder before the merge, and computes the merge again on a moved head up to
twenty rounds (`merge_isolation_moved.rs`).

- The wait for `index.lock`.
  The merge tree is computed with `merge-tree --write-tree` and the commit with `commit-tree`, none of which uses the
  index; only the checkout (`merge --ff-only`) does.
  By construction there is nothing to wait for if the merge, like the commit, is durable when the ref moves: move the ref
  by compare-and-swap first, and let the service bring the worktree and index to the new head afterwards
  (the files to change are known: the diff between the old and new head).
  The paths the person has unsaved changes in stay in the existing refusal (`merge-isolation/paths-in-use`): that one is
  never data loss, because the merge is not applied there and the person's files stay as they are; it is a question of
  meaning, not of racing.
- The twenty rounds.
  A lost compare-and-swap is another writer making progress, so the count is as artificial as the landing count of item 4.
  Run the merge as an item in the worktree's lane (the same serialized owner as commits and the push), so the head moves
  under it only for a foreign real-Git writer, and remove the count.
- The residue is the same as for commits: a foreign writer of the ref or of the index lock, both delaying and never failing.

## Residues that are designed out

- A malformed request.
  The client and the service share one protocol library, and a request is built from typed values, so a request the
  service calls malformed means a client and a service of different protocol versions.
  The welcome line already names the version.
  With item 2, a client that meets a service it cannot talk to lands the commit itself with its own release's code, so
  the commit does not depend on the service's reading of it, and the version mismatch becomes a notice.
- An object format this release cannot read.
  The in-process readers cover SHA-1 and SHA-256 repositories; a repository whose layout the readers do not cover
  (a reftable ref store, an extension) is already routed to Git, which builds the commit (`needs_native`).
  The design makes that the rule: an unreadable layout means the commit is made by Git in the scratch repository and landed
  by the lane, never refused by the wrapper.

## Exits that stay nonzero

- Policy errors at error severity, an empty message, and a content conflict no resolver settles (the merge stops with the
  conflict recorded).
- A commit or merge that names a conflicted path does not stay here: it commits with a warning.
- The causes the owner listed as outside our control, with a clear message, and work already handed over finished by the
  next run: network down, the remote refusing, a full disk or quota, disk I/O errors, a cap set by someone else, a kill or
  power loss, a corrupt repository caused by a disk fault or another tool, the person's signer failing.

## Verification plan

Each item keeps its red control, then the change, then a guard shown red once.
Nothing past the red controls is built until the owner chooses.
