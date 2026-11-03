# Commits never fail: the environmental exits of `git commit` in the git wrapper

Scope: `package/git-policy/cli`, the commit path of the wrapper installed as `git` (`cli-git-native`)
and the resident commit service (`cli-git-service`).

The owner's rule, recorded verbatim in `package/git-policy/cli/SPEC.md`, "Merges and commits never fail":
"Commits should never fail."
The owner's related rules, quoted: "Stop only on no progress", "You should (almost) never be waiting.",
"never discard user data", "commits vs native git should not take 5s longer.", "Use as few locks as possible.",
"Invoke real git as rarely as possible."

The coordinator's direction (adopted from the owner's rules; revisable in how, not in whether):

- A bound ends work only on no progress, never on elapsed time alone.
  Where a bound is reached while work progresses, keep going.
- Where something died (the service, a Git child), start it again and carry on.
- Where the head moved, rebuild on the new head and try again, for as long as each round makes progress.
- Every wait prints a line naming what it waits for and the holder, as the existing waits do.

## Scope

Item 7 of the starting list (a policy finding at error severity, a named path in conflict, an empty message,
a failing signer) is out of scope.
It is Git's own refusal, or the policy's, of the person's input; native Git refuses an empty message
and a failing signer too.
`SPEC.md` says so.

Items 1 to 6 are environmental and must not end a commit with a nonzero status.
Item 4 has a part that touches "never discard user data" (a conclusion, an amend or a revert commit that cannot
replay onto a moved head); this record lists options for it and builds none of them.

## The progress meter

The package already measures progress for pushes (`src/native/push_progress.rs`, from the "No-progress push limit"
work): bytes read and written (`rchar` plus `wchar` of `/proc/<pid>/io`), processor time, and a task able to run
or waiting for the disk (state `R` or `D`), summed over the process and everything it started, sampled every
half second.
A tree that shows none of the three for the stall window is stalled.
This record reuses `Clock` and `Meter` and writes no second meter.

Its documented limits carry over, and are accepted here:

- A child that spins for ever never stalls.
- A child that waits in state `S` for something that will arrive (a hook that waits for a person) looks the same
  as a dead one; only steps that are safe to run again are ever stopped on a stall (see item 1).

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

Cause: `ChildWatch` (`child_watch.rs`) ends any Git step the wrapper starts for itself when it has run for the
class bound (60 s for the quick class, 300 s for the bulk class), whatever the step does.
`output_watched` then returns a `TimedOut` error, and `checked_from` or `land_batch` turn it into a failed commit.
The object reader (`candidate_reader.rs`, one long-lived `git cat-file --batch`) is ended the same way
and reports `ReaderEnded`.
Elapsed time alone ends the work, which the owner's rule forbids; and the observed children took about 60 s.

Change:

- The class constants and `CLI_GIT_CHILD_WAIT_SECONDS` stay, and their meaning becomes the stall window:
  how long a step may show no sign of work before it is stopped.
  `watch_loop` stops using `waited >= limit`; it samples a `push_progress::Clock` started on the child
  with `Patience::Progress { stall: limit, ceiling: Duration::MAX }` and stops the child only on `StopCause::Stalled`.
- A step that is stopped for a stall is started again when running it twice is harmless
  (`may_be_rerun`, widened to the read-only plumbing the commit path uses and to `fast-import` and `commit-tree`,
  which write content-addressed objects).
  Each start gets a stall window twice as long as the one before, so a Git that really needs a long quiet stretch
  (the observed 60 s waits) is waited for on the next round, not killed again.
- A step that is not safe to run twice (`update-ref`, a commit Git builds) is never stopped for a stall:
  the wait goes on, announced at the usual interval with the process named.
  Stopping a writer in the middle of its write is the one choice here that could cost data.
- The object reader is started again after a stall stop and the request is sent again, with the same doubling.
- Every notice names the process and says what happens at the end of the window (stopped and started again),
  instead of "stopping it after N seconds".

Test (red first): through the real wrapper, `binary_child_bound_tests.rs` starts a commit whose Git children start
slowly under `strace` and a window shorter than the delay.
Today the commit exits 1 naming the bound; the red control asserts exit 0 and the commit on the branch.
A fixture without `strace` uses a fake `git` that stops itself with `SIGSTOP` on its first run of a step
(no bytes, no processor time, state `T`) and runs normally on the second.
A second control makes a step that writes a byte every half second for longer than the window,
and asserts it is not stopped (progress).
Guard shown red: the stall check in `watch_loop` is replaced by the old elapsed check and the progress control fails.

## Item 2: the service does not start within `START_WAIT`

Cause: `reach` (`service_client.rs`) stops trying after `START_WAIT` (10 s) and returns `DidNotStart`,
whatever the manager is doing.
A host under load can need longer to start a service, and a service that failed (its start counter full,
the socket in failed state) is never asked to start again by the client that needs it.

Change:

- `START_WAIT` becomes the first window, not a bound.
  When a window passes with no greeting, the client asks the manager to start the instance again
  (reset the failed state of the service and socket units, start the socket: the calls `provision` already makes,
  through a hook on `ClientConfig` that `service_commit` fills from the route's provisioning host),
  doubles the window, prints a line saying what it waits for ("the commit service of this worktree to start") and
  that it asked again, and carries on.
- What still ends the command, because no waiting helps: a platform that cannot start the service on demand
  (`NoActivation`), a program that is missing, a unit file that something else wrote, another account's service.
  These are the machine's setup, not the machine's load, and they stay.

Test (red first): `binary_service_provision_tests.rs` or `binary_client_service_tests.rs` starts the fixture service
behind a start that takes longer than `START_WAIT`; today the commit exits 1 with "did not start";
the red control asserts it lands.
A unit control of `reach` with a hook counts the restarts.
Guard shown red: the loop's old exit on `start_wait` is restored and the late-start control fails.

## Item 3: the service does not answer within `COMMIT_WAIT`

Cause: the client reads the socket with a read timeout of `COMMIT_WAIT` (10 min) and `submit_loop` returns
`WaitedTooLong` when the elapsed time passes it.
A queue behind slow commits is progress made by others, and a read timeout is not a dead service
(a dead service closes the socket, which is the existing reconnect path).

Change: the client waits for as long as the connection stays open.
The read timeout becomes the notice interval; on a timeout the client prints "still waiting for the commit service
(process N, position P of Q)", naming the service process read from the socket's peer credentials, and reads again.
A closed connection is the existing reconnect path and sends the same identity again.
`WaitedTooLong` and `COMMIT_WAIT` as a bound are removed; the constant stays as the notice-independent default of
`ClientConfig.commit_wait` only if a caller still reads it (the trash disposal and auto-push callers pass it today).

Test (red first): a unit control in `binary_service_tests.rs` runs `submit` against a stand-in service that greets,
sends a queue position every tenth of `commit_wait`, and lands after a multiple of `commit_wait`.
Today the client returns `WaitedTooLong` or `MayNotHaveLanded`; the red control asserts `Landed`.
Guard shown red: the elapsed check is restored.

## Item 4: the branch keeps changing; a commit cannot replay onto a moved head

Cause: `LANDING_ATTEMPTS` (3) in `land_batch` and `REBUILD_ATTEMPTS` (5) and `LANDING_ATTEMPTS` in `service_commit`
count rounds, not progress.
A busy branch (the auto-push's own reconcile merge, other worktrees, a script) can take more than three or five
landings in a row from one commit, and then the commit fails although every lost round was another commit landing.

Change:

- The landing loop and the client's prepare loop continue for as long as each round sees a new tip.
  Progress is `old` (the branch tip) differing from the tip of the previous round.
  A round that loses its compare-and-swap with the same tip as before is not progress by the branch:
  something else holds the ref (a ref lock); the loop pauses with a doubling pause from 50 ms to 2 s,
  prints a line naming the ref, and goes on.
- Every round after the first prints one line: "the branch <name> moved to <short oid>; making the commit again".
- A rebuilt commit that Git builds (a signed commit with no editor, or an editor commit whose message is saved
  in a file) is built again on the new tip, as today, without the count.
- A conclusion, an amend or a revert commit that cannot replay onto a moved head (`concurrent-commit/head-moved`)
  is not built here; see "Options for a commit that cannot replay" below.

Test (red first): `service_land_tests.rs` moves the branch under the batch more times than `LANDING_ATTEMPTS`
(the seam that exists today) and asserts `Landed`; today it returns `BranchKeepsChanging`.
`binary_service_native_tests.rs` makes a signed commit race more times than `REBUILD_ATTEMPTS` and asserts success.
Guards shown red: the counter in `land_batch`, and the counter in `service_commit`.

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

Cause: `acquire_placing` (`service_index_attempts.rs`) returns `IndexInUse` when the wait passes
`CLI_GIT_LOCK_WAIT_SECONDS` (1800 s), or at once when the inotify watch cannot be set.
A holder that is alive and working (a person's long `git commit` with an editor, a large `git add`) is not a failure,
and an exhausted watch is a machine limit, not an index in use.

Change:

- The wait ends only when the lock is placed.
  While a holder is proven alive, the notice names it (the existing proven-holder lines) and the wait goes on;
  the variable now sets when the notice changes from "waiting for process N" to the longer line that also says
  how to free the index, and no longer ends the wait.
- A lock the wrapper cannot attribute (no process, no marker) is waited for the same way, with the line
  that names the file, as the existing lines do; nothing foreign is ever removed, because the lock may hold a
  person's partial index.
- Without an inotify watch the wait polls every 50 ms instead of ending.
- The alternative, landing the branch without the lock and finishing the index later from the journal, is not built:
  it leaves `git status` showing the commit's paths reversed until the next commit or restart.
  If the owner prefers it, the journal and `recover_instance` already support it.

Test (red first): `binary_service_landing_lock_tests.rs` holds `index.lock` with a live helper process (a
`spawn_owned` guard) for longer than the wait limit the suite sets, then releases it, and asserts the commit lands;
today it exits 1 with "is using this repository's index".
Guard shown red: the `exhausted` check is restored.

## Item 6: `ContentPruned`

Cause: the service, or the client, finds that an object the commit captured was removed from the object store before
the branch moved (`git gc --prune=now` deletes unreferenced objects at once), and the commit ends with
`ClientFailure::ContentPruned`.
The failure was chosen so that the worktree is not read again, because reading it again commits whatever it holds
by then.

Change: nothing is lost by preparing again, because the worktree still holds the person's files.
On content that was removed the client prepares the commit again from the worktree as the command line named it
(a fresh capture, with a line saying what happened and what the second capture reads),
and sends it with the same identity; a repeated removal prepares again, with a doubling pause from 100 ms to 2 s.
A removal that keeps happening for every round is the cleanup's loop, not the commit's.
The content of a path the person changed between the two captures is the content the commit takes; native Git has
the same window between reading the worktree and writing the commit.

Test (red first): `binary_service_pruned_tests.rs` removes the blob while the wrapper is paused at the named phase,
as today; the red control asserts the commit lands with the worktree's content; today it exits 1 with the
`ContentPruned` text.
Guard shown red: the prepare-again arm is replaced by the old failure.

## Exits that stay nonzero

- Item 7: policy findings at error severity, a named path in conflict, an empty message, a failing signer.
- A platform with no user service manager, a missing service program, another account's service, a unit file in the way.
- The service died while handling this commit more than the reconnect count in a row (`RECONNECTS`).
  The request identity makes each send safe, so the bound only keeps a service that dies on this very request from
  looping for ever; it is a defect of the service, not of the machine.
- An amend, a conclusion or a revert on a moved head, pending the owner's choice above.
- A repository that is full, read-only or unreadable, a malformed request, and an object format or protocol this
  release cannot speak.

## Verification

Each item has a red control committed first, its fix, and a guard shown red once (the guard removed, the control run,
the guard restored; the guard is committed before it is removed).
The suites named in the task run once at the end: `native:clippy`, `native:test:host`, `native:test:binary:host`.
