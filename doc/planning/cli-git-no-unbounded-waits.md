# Every wait of the git wrapper has a bound

This is the inventory the new owner rule asks for, with the bound proposed for each wait and what happens at the bound.
It was written on `fix/git-policy-cli-every-wait-has-a-bound` from the 0.5.28 base (`14c26dbcc9`) on 2026-11-03.
Paths are relative to `package/git-policy/cli`.

## The rule and the answers it rests on

The owner, said in the session at about 2026-11-03T12:37-05:00 (quoted):
"nothing should ever wait forever or indefinitely".
It is newer than, and overrides, the unbounded waits built under the delegated picks 2 and 6 of
`doc/planning/cli-git-shipped-limits-audit.md`, the follow-up pick that waits for an owner nothing proves alive with no
bound, and the owner's earlier "Wait while connected (Recommended)" for pushes.
The coordinator flags that conflict for the owner and treats the newer rule as governing.
0.5.28 does not ship until every wait has a bound.

The owner keeps: "Commits should never fail."; "never fail doesn't primarily mean retries!"; "never discard user data";
"You should (almost) never be waiting."

The owner answered through the question tool at about 2026-11-03T12:45-05:00:

- At a wait's bound: "Return with an error" (the option read: "The command stops waiting and exits non-zero, naming what
  blocked it; the saved work stays saved.").
- Whether the saved commit should still land by itself later: "No, keep it for a retry (Recommended)" (the option read:
  "Nothing lands later by itself. Your staged changes stay staged, and the commit is also kept on a side ref.
  The error names
  the blocker and the command to run again.").
- Pushes: "Bound it, keep retrying (Recommended)" (the option read:
  "A connected push that stops making progress for a measured
  time is ended; the service's retry plan keeps the push and tries again;
  the command returns with a line saying the push is
  queued."). This replaces "Wait while connected".

So at the bound of a wait on the commit path the request is withdrawn so it can never land later, durably,
the person's index
and worktree stay as they were, the built commit stays on a side ref that the error names,
and the command exits non-zero with
one message that names the blocker and the command to run again.
The withdrawal races correctly with a landing that starts at the bound: either the commit landed,
and success is reported, or
it never will.

Everything here is the coordinator's reading under the owner's delegation and open to the owner's veto, except the three
answers quoted above.

## Method

The sources are every non-test Rust file of the crate (500 files).
The searches were: `thread::sleep`, `recv(`, `recv_timeout`, `Condvar`, `.wait(`, `wait_with_output`, `.status()`,
`.output()`, `accept(`, `incoming()`, `read_line`, `flock`, `try_wait`, `set_read_timeout`, `poll(`,
and every `loop {` and
`while` that is not a parser over a finite input (about 90 loops read, the rest are parsers and tree walks).
The test helpers were read for waits that could hang a gate; `.config/nextest.toml` ends any test at five minutes,
which is the
bound of the gate itself.
Status words: "built" is built on this branch, "bounded" already had a bound, "exempt" is an idle wait by design,
"coordinator" is an unclear end behavior with options below.

## The waits

### Locks

1. `src/native/service_index_attempts.rs` `wait_and_place` (landing waits for a foreign `index.lock`).
   Waits for the lock to go; ended today by release or a dead owner (a PID file naming a gone process), nothing else.
   Bound: `LOCK_CEILING`. At the bound: the batch is withdrawn before the branch moved,
   the built commit is kept on a side ref,
   the client exits non-zero naming the holder and the command to run again. Built.
2. `src/native/index_holder_wait.rs:119` `wait_for_index_release` (a merge waits for any `index.lock`).
   Ended by release or a dead owner. Bound: `LOCK_CEILING`. At the bound: the merge stops, nothing changed, non-zero,
   naming
   the holder. Built.
3. `src/native/service_index_wait.rs:162` `wait_for_service_hold` (a forwarded `git add`,
   `rm` or `mv` waits for the service's
   hold). Ended by release, a mark naming a gone holder,
   or a hold of `UNASKABLE_HOLD_AGE` whose holder cannot be asked about.
   Bound: `SERVICE_HOLD_CEILING`. At the bound: the command runs and reports what Git says. Built.
4. `src/native/service_land_follow.rs:157` `update_index` (index reconciliation of a commit with no change left),
   through
   `wait_for_index_release_here`. Bound: the bound of entry 2. At the bound: the command ends,
   non-zero as it always does. Built.
5. `src/native/service_hold_retry.rs:103` `run_again_while_service_holds` (up to `ATTEMPTS` 50 reruns,
   each preceded by entry 3).
   Bounded by count, not by time: the worst case is 50 times the bound of entry 3. Proposed bound:
   one deadline over the whole
   loop, `SERVICE_HOLD_CEILING`. Built.
6. `src/native/service_index_lock.rs:97` `try_acquire` and `src/native/index_reset_support.rs:209`: one attempt,
   no wait. Bounded.
7. Reference locks: `src/native/auto_push.rs:46` `PUSH_LOCK_RETRIES` 3 with `PUSH_LOCK_PAUSE` 250 ms;
   the stale lock judges
   (`stale_ref_lock_*.rs`) read and decide without waiting. Bounded.
8. Lane and owner locks: `src/native/worktree_convert_journal.rs:290` uses `flock(LOCK_EX | LOCK_NB)`,
   which never waits.
   The discard audit appends (`discard_audit.rs:335`) loop only on an interrupted write. Bounded.

### The commit service, client side

9. `src/native/service_client.rs:416` `submit_loop` (the client waits for the answer, with heartbeats).
   Ended by an answer, a silence of `HEARTBEAT_SILENCE`, or a close; nothing ends it while the service keeps sending.
   Bound: `ANSWER_CEILING`. At the bound: the client withdraws the request (below),
   exits non-zero naming what the service was
   waiting for when it could tell, with the side ref and the command to run again; if the request had landed, success.
   Built.
10. `src/native/service_client.rs:220` `reach` with a manager watch (waits for the service to start).
    Ended by a greeting or two `failed` answers of the manager; nothing else. Bound: `START_CEILING`. At the bound:
    "The commit service did not start" with the journal command, as for the other start failures. Built.
11. `reach` without a watch, `reach_current_untimed:330` (retire an older service), `ask_to_retire:517`:
    bounded by `START_WAIT`.
12. `reach` after a service was seen: the welcome read is bounded by `heartbeat_silence`. Bounded.
13. `src/native/service_client.rs:568` `wake`: reads until the service closes,
    each read bounded by `heartbeat_silence`. Bounded.
14. Reconnect loops (`RECONNECTS` 5) and the pauses between them (`CONNECT_PAUSE` 25 ms): bounded by count.
15. `src/native/systemd_dbus.rs:468` and the provisioning waits: `EXCHANGE_WAIT` 20 s and `RETIREMENT_WAIT` 10 s.
    Bounded.

### The commit service, server side

16. `src/native/service_reply.rs:69` `write_until_answered`: runs until the reply is dropped (the request is answered)
    or the
    client is gone. Ended by the landing thread's answer. Bounded by the landing: entries 1 and 17. No change.
17. `src/native/service_server.rs:423` `land_group` and the landing steps: the lock wait (entry 1),
    the Git child steps (entries
    21 and 22), `LANDING_ATTEMPTS` 3 and `REBUILD_ATTEMPTS` 5. Bounded once entries 1, 21 and 22 are.
18. `src/native/service_server.rs:246` `wait_for_close` (a scratch connection): ended when the client closes or dies.
    Bounded by
    the client, whose every wait is bounded.
19. `src/native/service_server.rs:577` `serve` with `recv_timeout(idle)` (`IDLE_EXIT` 3 hours). Bounded, exempt idle.
20. `src/native/service_server.rs:537` `work_loop` (`receiver.recv()` for wakes)
    and the acceptor (`incoming()` at `:603`):
    idle by design; the service exits at `IDLE_EXIT` when nothing is busy. Exempt.
    The request read timeout (`REQUEST_READ_TIMEOUT` 10 s) and `ACCEPT_BACKOFF` bound the rest.
    `service_claim_renewal.rs` receives on its own threads and ends with its owner. Exempt.
    Work items: `MAX_TRIES` 3, `CLAIM_LEASE` 120 s. Bounded.

### Child processes

21. (Built, 2026-11-03.) `src/native/child_watch.rs:396` `watch_loop` (Git steps of a commit):
    a step is ended after the quiet window (60 s quick,
    300 s bulk, both labelled unmeasured) of no sign of work;
    a step that shows work is waited for however long it takes.
    Ended today by completion or by quiet. No time bound. Coordinator: option set A below.
22. Steps marked `never_stop` (`child_watch.rs:337`: reference updates, a commit Git builds):
    never ended by the wrapper.
    No bound. Coordinator: option set A below.
23. `src/native/bounded_push.rs:453` and `src/native/push_progress.rs` (pushes, fetches and LFS uploads of the service):
    a push with no connection is ended after the stall window (30 s labelled unmeasured);
    a push with an open connection is never
    ended. Owner's answer: "Bound it, keep retrying". Bound: `CONNECTED_PUSH_QUIET_WINDOW` of no progress.
    At the bound: the
    push is ended, the service's retry plan keeps it, and the notice says the push is queued. Built.
24. `src/native/bounded_query.rs` read-only queries (120 s ceiling, `doc/planning/cli-git-shipped-limits-audit.md`).
    Bounded.
25. `src/native/forwarding.rs:186` `run_real_git` and `src/native/signal_relay.rs:290` `wait_without_reaping`:
    the wrapper
    waits for the Git command the person typed (editor, the person's hooks, a long checkout).
    Not a wait the wrapper chose; ended
    by the person (interrupt) or by Git. Coordinator: option set B below.
26. `src/native/package_repair_command.rs:205` (600 s), `src/native/markdown_linter_guard.rs:142` (60 s),
    `src/native/standards_review_run.rs:210` `run_contained` (bounded by the review's own deadlines),
    `src/native/start_retry.rs`
    (`START_RETRY_BUDGET` 5 s), `src/native/commit_preparation.rs:168` (counted retries). Bounded.

### Polls, watches and channels

27. `src/native/service_index_watch.rs:157` `Watch::wait`: bounded by the timeout its caller passes. Bounded.
28. The poll loops of entries 1 to 3 (`POLL_PAUSE` 50 ms): bounded by the ceilings of those entries.
29. `src/native/transaction_phase.rs:289` `await_release`: a test marker compiled into the release executable,
    armed only by an
    environment variable; it waits for a file and gives up only when the directory is gone. Proposed bound:
    `PHASE_PATIENCE`.
    Built.
30. `src/native/child_watch.rs:233` and `:241` (`STOP_POLL` 10 ms while a stopped child is reaped):
    bounded by the stop grace
    (`STOP_GRACE` 5 s, then a kill). Bounded.
31. `src/native/signal_relay.rs:412` and `:476` `child.wait()` after a stop or an exit: bounded by the child's end.
    Bounded.

### Retry loops

32. The push retry plan (`src/native/push_retry.rs`): by design it keeps trying; each try is bounded by entry 23,
    and the plan
    spaces the tries (10 s to 10 minutes). This is the owner's "keep retrying" and is not a wait of the command.
33. `LANDING_ATTEMPTS` 3, `REBUILD_ATTEMPTS` 5, merge `MAX_ROUNDS` 20, `MAX_RECONCILE_ATTEMPTS` 10,
    claim take rounds 64,
    `INDEX_ATTEMPTS` 5. Bounded by count; none loops on a clock.

### Test helpers

34. `src/native/parallel_test_support.rs` (`Rendezvous`, patience set) and `src/native/stale_ref_lock_killed_git.rs:135`
    (`wait_for`, `HOLD_PATIENCE`): bounded.
35. Binary controls that wait on children (`command_test_support.rs:190`, `index_reset_support.rs:58`,
    `fixture_executable.rs:50`,
    `umask_child_test_support.rs:30`) have no timeout of their own; `.config/nextest.toml` ends a test at five minutes
    (`slow-timeout` 60 s, terminate after 5), so a hang ends the run with the test named. Bounded by the harness.

Built on this branch (2026-11-03): entries 1 to 5, 9, 10, 23 and 29,
with a red test or a mutation red for each except 5 and 29,
which were written with their fix. Entries counted as waits: 35 (rows 6 to 8,
11 to 15 and 24 to 31 are bounded already, rows 19, 20 are idle by design).

## Bound values

The rule `LQM` asks for a measurement or an owner decision beside each value.
The data on this machine, read on 2026-11-03 (read-only):

- The commit timing log of this repository, `.git/cli-git-commit-timing.jsonl`, 1,329 records.
  It is the only timing log under the home directory (`find` over the home directory found no other).
  `answerWaitMs`: median 67 ms, 99th percentile 50,344 ms, longest 308,267 ms.
  `totalMs`: median 688 ms, 99th percentile 168,794 ms, longest 750,604 ms.
  `startWaitMs`: longest 1,041 ms. `reachMs`: longest 1,041 ms.
  `afterLandingMs`: longest 43,120 ms. `gateMs`: longest 38,551 ms. `prepareMs`: longest 250,280 ms.
- The service journal: 1,335 landings; `lock_hold_ms` of 715 of them,
  longest 119,721 ms (all in the disk sync of the new
  index); `queued_ms` longest 89,874 ms; 794 pushes that landed, whole-push times p50 3.5 s, p99 26.4 s, longest 72.4 s.
- No log holds a wait behind a foreign `index.lock` holder (no `lock_looks`, no waiting notice),
  and none holds a quiet gap of a
  connected push.

The values, each with its margin:

- `ANSWER_CEILING` 1200 s: 3.9 times the longest observed answer wait (308.3 s)
  and 1.6 times the longest whole commit (750.6 s).
- `START_CEILING` 60 s: 58 times the slowest observed start (1.04 s). `START_WAIT` 10 s keeps its earlier citation.
- `SERVICE_HOLD_CEILING` 600 s: five times the longest observed service hold (119.7 s).
  It is also the bound of the whole rerun
  loop of entry 5.
- `LOCK_CEILING` 600 s: unmeasured, no foreign-holder wait is logged anywhere.
  It is below `ANSWER_CEILING` so that the service
  ends its own wait and answers before the client gives up.
  A person's `git commit` with an editor open longer than this makes
  ours stop with the error; that is the price of the rule, listed in the options.
- `CONNECTED_PUSH_QUIET_WINDOW`: unmeasured, labelled as the other quiet windows are (see option set C).
- `WITHDRAW_CEILING` 600 s: the client's wait for the service's answer to a withdrawal;
  five times the longest observed landing
  (119.8 s). Built.
- `PHASE_PATIENCE` 300 s: a test marker, the longest any test may run (`.config/nextest.toml`). Built.

## Options for the coordinator to pick

### A. Git steps that keep working, and steps that are never stopped

Covers entries 21 and 22. A step that shows signs of work is waited for however long it takes,
and a `never_stop` step (a
reference update, a commit Git builds) is never ended: the owner's "Stop only on no progress" and "End Git,
hold hooks" still
apply, and "nothing should ever wait forever" now bounds them.

- A. A ceiling per class of step, with the longest observed step as the base (`prepareMs` 250 s, `buildMs` 183 s):
  at the ceiling
  the step is ended, the commit is withdrawn as in entry 9, and the error names the step.
  Pros: every step ends. Cons: a legitimately slow step on a huge repository fails the commit,
  and a `never_stop` step ended in
  the middle of a write can leave a lock or a half-written reference.
- B. A ceiling for steps that can be run twice, none for `never_stop` steps,
  which are bounded by the commit's `ANSWER_CEILING`
  (the client withdraws at that bound and the service finishes the step it holds).
  Pros: nothing is ended in the middle of a write; the person gets the error at a known time. Cons:
  the service thread stays
  busy with the step after the client left.
- C. A ceiling for every step that is a multiple of the longest measured step,
  with a hard stop for `never_stop` steps after a
  longer one.
  Pros: one rule. Cons: ends a write in the middle, the case the owner's "never discard user data" is wary of.

Ranking: B > A > C, because B keeps the owner's rule that a write is ended only by the person while still giving the
person an
error at a known time; B > A because A ends a step that may be mid-write only for a number nobody measured;
A > C because C
extends the same risk to the steps that cannot be run twice.

### B. The person's own Git and hooks

Covers entry 25. The wrapper waits for the Git command the person typed; the hooks,
the editor and a long checkout are the
person's. The wrapper runs none of the person's hooks itself (every Git it starts carries `core.hooksPath` below
`/dev/null`).

- A. Exempt: the wait is the person's command, ended by the person, as with native Git.
  Pros: the wrapper does not change what `git` does when it forwards. Cons: it reads as a wait with no bound.
- B. A ceiling on forwarded commands that are not interactive (no editor, no terminal prompt), none on interactive ones.
  Pros: bounds scripts. Cons: ends a long `git checkout` or `git gc` the person asked for; "never discard user data".
- C. A notice after a long wait, no bound.
  Pros: tells the person what is running. Cons: still no bound.

Ranking: A > C > B, because the wait is the person's own command;
A > C because C adds output with no change in outcome; C > B
because B ends the person's work.

### C. The quiet window of a connected push

Covers entry 23. The owner chose to bound a connected push that stops making progress and keep it in the retry plan.
The number is not measured.

- A. The existing stall window (30 s, labelled unmeasured) for connected pushes too.
  Pros: one number. Cons: a slow remote that answers after 40 s is ended and retried;
  the whole-push p99 is 26.4 s and the
  longest 72.4 s.
- B. Twice the longest whole push in the journal (72.4 s), about 150 s.
  Pros: above everything observed. Cons: a stuck push holds the branch's push for two and a half minutes.
- C. Ten times the p99 (264 s).
  Pros: very safe. Cons: long.

Ranking: B > A > C. B is built as the value, labelled unmeasured as a quiet window;
B > A because A is below the longest observed
push; A > C because C holds every later push of the branch for minutes.

### D. A person's `git commit` with an editor open when the bound passes

Covers entry 1 and 2 at their bound. A foreign `git commit` holds `index.lock` for as long as its editor is open.

- A. As built: our commit is withdrawn at `LOCK_CEILING` with the error and the side ref.
- B. A longer ceiling (30 minutes, the earlier unmeasured value).
- C. Treat an owner whose process shows work as a different case from one that shows none.

Ranking: A > B > C, because A matches the owner's answer; B only moves the number;
C needs a measure of a foreign process's work.

### E. Idle waits that are not bounded in time

Covers entry 20. The work loop and the acceptor wait for events with no bound;
the service exits at `IDLE_EXIT` (3 hours) when
nothing is busy. Exempt by design; if the owner reads the rule as covering them too,
the service's idle exit already bounds them.

## Picks

Picked by the coordinator under the owner's delegation, veto open, 2026-11-03:

- A, not option B: every step class gets a service-side ceiling,
  including steps that keep showing work and steps that are never
  ended for being quiet. Under B a `never_stop` step that never returns would keep the service's landing thread for
  good, so every
  later commit on that repository would wait out `ANSWER_CEILING` and fail, which is a wait forever.
  Ending a reference update or a
  commit Git builds cannot half-write a reference (Git writes a lock file and renames it);
  the residue is a lock the shipped
  stale-lock removal clears once the holder is proven gone ("End Git, hold hooks" is the owner's earlier pick).
  Each ceiling sits below
  `ANSWER_CEILING`, so the service answers first. Built:
  `PLUMBING_STEP_CEILING` 120 s (the longest read step of 1,651 landings
  took 9,663 ms), `OBJECT_STEP_CEILING` 600 s (the longest client preparation of 1,400 records took 250,280 ms;
  the service's
  object step at most 1,358 ms), `REFERENCE_STEP_CEILING` 60 s (the longest of 1,651 reference updates took 3,438 ms).
  After a
  reference update is ended the branch is read: at the request's commit, the outcome is `Landed`;
  otherwise the batch is withdrawn
  exactly as at `LOCK_CEILING` (intent removed, commit kept on `refs/cli-git/kept/<batch>`, staging unchanged,
  lock given up, and the
  error names the step). Hooks are still never ended by the wrapper.
- B: A, exempt: the wait for the person's own Git command and hooks is the person's, as with native Git.
- C: B, 150 s as built.
- D: A, the withdrawal at the lock ceiling as built.
- E: exempt.
- The owner's push answer ("Bound it, keep retrying") said the command says the push is queued:
  a push ended for showing no
  progress now prints "The push is queued: the commit service keeps it in its retry plan and tries it again."
  (`PUSH_QUEUED_LINE`),
  and a test reads that line from the output.
