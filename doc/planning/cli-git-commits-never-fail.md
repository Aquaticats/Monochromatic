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
Nothing here is built yet; the owner chooses first.

What exists on the branch from before the correction, kept and not discarded:

- The planning record's first version (commit `6c8c686b2a`).
- A red control for item 1 (`53e40e9fc0`), a red control for item 3 (`7b7d9870f3`, written, not run).
- A retry-based first attempt at item 1 (`26db9e54af`): a Git step with no sign of work for its window (the push
  meter of `push_progress.rs`) is stopped and started again with a doubled window, and the object reader is restarted.
  Under this record it is at most the residue handling of item 1 (below) and is the owner's to keep or drop.

## Scope

Item 7 of the starting list (a policy finding at error severity, a named path in conflict, an empty message,
a failing signer) is out of scope.
It is Git's own refusal, or the policy's, of the person's input; native Git refuses an empty message
and a failing signer too.
`SPEC.md` says so.

Items 1 to 6 are environmental.
Item 4 has a part that is a question of meaning, not of racing: a conclusion, an amend or a revert commit that cannot
be replayed onto a head that another commit moved.
That part lists options for the owner and builds none.

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

What stays unexplained: why each Git child of the main worktree took about 60 s.
Nothing in the wrapper waits that long on a timer, and the child bound did not fire (the first step ended at
59.4 s, the second at 60.0 s, the class bound being 60 s for the first and 300 s for the second).
The owner's rule forbids chasing Git's own fsmonitor as a cause, so it was not chased, and no other cause is shown
by the evidence.
This record therefore claims only what the evidence shows: two consecutive Git children of about 60 s each,
both inside the lock hold, both inside the existing bounds.
It also shows why item 1 matters: a child that took a few seconds longer than its 60 s class bound would have
been stopped, and the stop ends the commit (before the branch moves) or leaves the index update undone (after).

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
  landed) is a question of meaning: see the options below.

Residue: a foreign program that writes the branch with real Git directly, faster than a commit can be built.
Real Git offers only the compare-and-swap for a ref, so that program's progress cannot be excluded; each lost round
is that program's commit, and the commit is built again on it with a notice.

### Options for a commit that cannot replay

These are not built.
The owner decides, because each touches "never discard user data" differently.
Setting: `main` is at commit `B`; another session lands `C` on `main` while this commit is built.

Example 1, an amend.
The person typed `git commit --amend -m "fix the parser"` on top of `B`, whose message was "parse the file".
Before: `A <- B (parse the file)`, `main` at `B`.
Another session landed `C (add the lexer)` on `main` while the amend was built: `A <- B <- C`.
Today the amend ends nonzero and nothing changes.

Example 2, a conclusion.
The person finished a merge with `git commit` (the merge of `topic` into `main` on `B`; `MERGE_HEAD` is `T`).
The merge commit `M` has parents `B` and `T`.
Another session landed `C` on `main`: `A <- B <- C`.
Today the conclusion ends nonzero and the merge state stays.

Example 3, a revert.
The person ran `git revert B` and then `git commit` (the revert commit `R` has parent `B`).
Another session landed `C`.
Today the commit ends nonzero.

Option A, rebuild on the new head for every kind (a merge commit with parents `C` and `T`, a revert of `B` on `C`,
an amend that replaces `B` and keeps `C` on top: `A <- B' <- C'`).
Pros: no commit ever fails; the person's message and intent are kept.
Cons: an amend or a conclusion would be computed against a head the person did not see; an amend that rewrites `B`
while `C` sits on it must also rewrite `C`, which discards another session's commit identity (`C` becomes `C'`),
and a conclusion on `C` may conflict, which needs the person.
What it leaves for example 1: `A <- B' <- C'` where `C'` is a new object with `C`'s content; for example 2:
`M` on `C` and `T`; for example 3: `R` on `C`.

Option B, stack the result as a new commit on the new head and keep the person's work whole
(an amend becomes an ordinary commit `D (fix the parser)` on `C` with the amend's tree change; a conclusion lands as the
merge of `T` into `C`; a revert lands as the revert of `B` on `C`).
Pros: nothing another session landed is rewritten; the person's content lands; no commit fails.
Cons: an amend is no longer an amend (`B` stays, `D` is added), so the person sees two commits where they asked for one;
the person's `git commit --amend` intent of replacing history is not honored.
What it leaves for example 1: `A <- B (parse the file) <- C (add the lexer) <- D (fix the parser)`;
example 2: `A <- B <- C <- M'` with parents `C` and `T`; example 3: `A <- B <- C <- R`.

Option C, wait until the head has not moved for a quiet interval and build then, with the person's staged state kept
(the commit is held, with a line naming the branch and the interval, until two lookups an interval apart see the same tip;
then it is built and lands on that tip).
Pros: the commit is built once, on the head it lands on, amends and conclusions included; no history of another session
is rewritten; nothing is discarded.
Cons: a branch that moves without pause never lets it land (a wait, not a failure; "you should (almost) never be
waiting" is bent); the person waits while others commit.
What it leaves for example 2: `A <- B <- C <- M'`, built once on `C`; for example 3: `A <- B <- C <- R`.
For example 1 it leaves nothing new: an amend replaces the tip it saw, and `C` already sits on `B` when the hold ends,
so the amend of example 1 still needs option A or B at the end.

Option D, the current behavior made kind: the commit ends nonzero and says exactly what moved,
keeping the message in a file (`git commit -F`).
Pros: nothing is rewritten or reordered; the person decides.
Cons: it is the failure the owner's rule forbids.

Ranking: B > A > C > D.
B ranks over A because it never rewrites a commit that another session landed (`C` stays `C`), which is the data the
rule "never discard user data" protects, while A must rewrite `C`.
A ranks over C because A always ends in a landing and C can wait for ever on a busy branch, and A's cost (new objects
for `C`) is bounded, whereas C's cost is the person's time.
C ranks over D because C ends in a landing for two of the three kinds and keeps the person's input, while D never does.
The cost B asks of the owner is the meaning of an amend: B turns an amend into an extra commit when the head moved,
and says so on standard error.

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

## Exits that stay nonzero

- Item 7: policy findings at error severity, a named path in conflict, an empty message, a failing signer.
- An amend, a conclusion or a revert on a head that other commits moved while the person worked, pending the owner's choice.
- A repository that is full, read-only or unreadable, a malformed request, an object format this release cannot read.
  These are the repository's state, not a race or a wait.

## Verification plan

Each item keeps its red control, then the change, then a guard shown red once.
Nothing past the red controls is built until the owner chooses.
