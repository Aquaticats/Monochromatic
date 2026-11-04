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
  Amended 2026-11-04T09:35:19-05:00 by the owner ("Never cut a branch write (Recommended)"): the reference update, and every other
  write in place that is not safe to run twice, has no ceiling; `REFERENCE_STEP_CEILING` and the read of the branch after an ended
  update are gone (see "Branch write and tool ceilings, 2026-11-04T09:35:19-05:00" below). The reasoning above, that a hung
  write holds the landing thread for good, is the con the owner accepted.
- B: A, exempt: the wait for the person's own Git command and hooks is the person's, as with native Git.
- C: B, 150 s as built.
- D: A, the withdrawal at the lock ceiling as built.
- E: exempt.
- The owner's push answer ("Bound it, keep retrying") said the command says the push is queued:
  a push ended for showing no
  progress now prints "The push is queued: the commit service keeps it in its retry plan and tries it again."
  (`PUSH_QUEUED_LINE`),
  and a test reads that line from the output.

## Branch write and tool ceilings, 2026-11-04T09:35:19-05:00

The owner answered two questions through the question tool on 2026-11-04.
They amend the pick of option set A above (every step class has a ceiling, the reference update's being 60 seconds)
and the outcome of pick 9 of `doc/planning/cli-git-shipped-limits-audit.md` (tools and checks end only on no progress).
Paths are relative to `package/git-policy/cli`.

### The owner's answers

"Never cut a branch write (Recommended)".
The option read:
"Steps that can safely run twice keep their ceilings; the branch write is never ended.
Your command still returns at its bound with "result not yet known" and the `git log -1` and side-ref hint
(your "Short extra wait" answer).
Con: a truly hung write keeps the service busy,
and later commits fail at their own 20-minute ceiling with an error naming it."

"Silence plus a ceiling (Recommended)".
The option read:
"Keep ending on silence, and add a generous overall ceiling per tool, measured from real runs,
after which it's ended with an error naming the tool.
Con: an extremely slow but working run on a huge repository can hit the ceiling."
It covers the non-Git tools of `git worktree add` and the review: package repair, the Markdown linter,
the standards-review call and the whole review.

Also carried out, from the follow-up of pick 7 of the audit that was recorded and never run:
`FOREIGN_OWNER_PATIENCE` is twice `LOCK_CEILING`, 20 minutes, derived in code (see the audit).

### What changed for the branch write

`REFERENCE_STEP_CEILING` (60 seconds) is deleted.
`src/native/transaction/transaction_git.rs` `step_bound` gives a step no ceiling exactly when `cannot_run_twice` says so;
which forms those are was decided by experiment (next section), not by reading.
The first build of this branch (`a8b7473fc7`) had instead a list of subcommands, `WRITES_IN_PLACE`, chosen by reading;
the coordinator's follow-up of 2026-11-04 replaced it with the experiment.
The landing's branch move no longer reads the branch after an ended update:
that arm of `service_land.rs` is gone with the ceiling that produced it.

What the clients are told, because the single landing thread also answers every withdrawal between two landings:

- `src/native/service/service_busy.rs` is the shared record of what the landing thread is doing.
  The Git runner writes into it when a step with no ceiling begins, and the landing adds what it knows
  (the branch and commit the update moves, the scratch ref that holds the commit until the update ends).
- After `NEVER_ENDED_STEP_SLOW_AFTER` (60 seconds, 17 times the longest of 2,093 reference updates, `ref_ms` 3,438 ms) the thread
  that reads each connection sends a `waiting` message naming the write, again whenever the whole-minute count changes.
  The client prints it, and keeps it as the blocker its error names (`StillWithService`, and now `WithdrawalUnconfirmed`).
- A `withdraw` for a request that is not in the group being landed is answered by the connection thread
  (`withdrawn`, or `landed` when a finished batch of the journal names it) while the record shows a stuck write;
  a request that a batch still open names is left to the landing thread, because that batch may hold its commit on the branch
  already (`a_request_whose_commit_is_on_the_branch_in_an_open_batch_is_not_withdrawn_early`, found in review of this branch;
  red before the fix);
  the request is remembered and dropped when its group would begin. The record holds no lock (the first build used a mutex and the
  gate's lock inventory refused it, `source_guard/service_lock_inventory_tests.rs`, in keeping with "Use as few locks as
  possible."): exclusive creation of a `.flight` marker by the landing thread and a `.withdrawn` marker by a connection thread,
  each created before the other's is looked for, in `busy` under the journal.
  So a later commit ends at its own `ANSWER_CEILING` (20 minutes), not 10 minutes after it, and the error names the write.
- The request inside the stuck write is not withdrawn that way:
  its commit may be made by the write.
  Its client waits `WITHDRAW_CEILING` (10 minutes) for the landing, then ends with the failure that says it is not known
  whether the commit was made, names the write and the scratch ref, and gives `git log -1` and `git status`.
- Protocol messages and the protocol version are unchanged.

Tests (red first where the behavior existed):
`a_branch_write_that_outlasts_the_step_ceiling_is_not_ended` (red at `afad55b58d`: `StepTooSlow`, the old arm ended the write at the
shortened ceiling and withdrew the commit),
`a_write_in_place_has_no_ceiling_and_every_other_step_keeps_one`, `a_shortened_ceiling_never_reaches_a_write_in_place`,
`a_step_with_no_ceiling_is_waited_for_until_it_ends_by_itself`,
the board's controls in `service_busy_tests.rs`,
and in `landing/service_land_hung_write_tests.rs` two controls against a real `serve` with a Git whose `update-ref --stdin` waits for a
release file: `a_commit_queued_behind_a_branch_write_that_does_not_return_is_withdrawn_at_its_ceiling_naming_the_write`
(shown to fail with the early answer disabled, in 49 s, then restored)
and `the_commit_inside_a_branch_write_that_does_not_return_is_reported_as_not_known_and_lands_when_the_write_returns`.
The two controls that encoded the removed behavior (`a_reference_update_ended_after_it_renamed_the_branch_reports_landed`,
`a_reference_update_ended_before_it_renamed_the_branch_withdraws_the_commit`) are deleted.

### What changed for the tools

Each tool keeps its silence window and gains a ceiling on its whole run.
The windows are unchanged: the owner chose ceilings, not new windows.

- `REPAIR_TOOL_CEILING` 18 minutes: `run_tool` ends the program with its whole process group
  (`ToolEnd::CeilingReached`); the finding names the tool, what it was doing, the seconds, and the command to run by hand.
  Test: `a_program_that_keeps_working_is_ended_at_its_ceiling_with_its_whole_process_group`
  (red on the unfixed code, observed before the fix and committed with it in `95c46239e7`: the tool, which printed all the time, ran to its end, 20.2 s; the control also checks that a helper of the tool is gone).
- `LINT_CEILING` 4 minutes: the Markdown check runs on a thread that cannot be stopped from outside,
  so the ceiling abandons it, as the quiet window does, and the reason names the tool, the file and the ceiling.
  Test: `a_job_that_keeps_working_is_abandoned_at_its_ceiling` (red at `af74896d85`: the guard never answered while the job worked, and the run was ended by the unit-test timeout at 120 s).
- `REVIEW_CALL_CEILING` 1 minute: `ureq`'s `global` timeout; the sentence names the call, the ceiling and
  `SHADOW_REVIEW_SHA=<commit> mise run shadow-review`.
  Test: `a_reply_that_keeps_arriving_is_ended_at_the_ceiling` (red at `6f439cb4a1`: the reply was read whole).
- `REVIEW_CEILING` 2 minutes: `run_contained` waited with `recv()` and no bound at all, so the local write of the log entry
  was unjudged; it now stops waiting at the ceiling and the item fails with the sentence of `ceiling_reason`.
  Test: `the_runner_stops_waiting_for_a_job_at_its_ceiling` (no red run: the old runner had no ceiling to test against,
  and the control would have waited for ever).
- The owner's message names a call window of 10 seconds and a review of 30 seconds.
  The shipped values are a 10 second window in the service (`CALL_QUIET_WINDOW`) and 30 seconds as the script's default
  (`SCRIPT_QUIET_WINDOW`), and the whole review had no deadline since pick 9, so the review's ceiling is new, not a replacement.

### Measurements

Read on 2026-11-04 between 09:00 and 09:35 (-05:00), read-only, on this host under the load it had
(load average 3.7 on 28 threads at the start of the linter timing).
The scripts are `/var/home/user/temp/agent/ceiling-measure/measure.ts`, `measure-lint.ts`, `quiet-names.ts` and `commits.ts`.

- Worktree add timing log, `.git/cli-git-worktree-add-timing.jsonl`: 79 adds,
  2026-11-03T08:55:46Z to 2026-11-04T13:53:17Z, releases 0.5.20 to 0.5.29; 69 records hold the per-tool `work` figures.
  A stage is one `work` entry with its call count, so a figure with many calls is a total.
  Longest stage: `repairPnpm` 53.1 s (one call), `repairToolchain` (`mpm-shadow install`) 30.0 s (one call),
  `repairCargo` 211.2 s (the whole Cargo stage, which holds every `cargo metadata` and `cargo clean` of the add,
  34 workspaces at most, so it bounds any one of them).
  Longest add 539.7 s.
- Quiet gaps of the add log (`quiet`, 12 records, resolution 500 ms): `cargo clean` 30.3 s, `mpm-shadow install` 7.6 s,
  `cargo metadata` 0.5 s, `pnpm install` 0.5 s.
  The 600 second window is 19.8 times the longest.
  Too small a sample to change the window, and unchanged.
- Commit timing logs (`cli-git-commit-timing.jsonl` and `.1`): 1,838 commits up to 2026-11-04T14:08:47Z (the first record of the
  rotated file has no `time` field, so the start of the sample is not known; the oldest record that has one is 2026-11-02),
  longest `gateMs` 38,551 ms (99th percentile 943 ms),
  longest `totalMs` 750,604 ms; quiet gap of the Markdown linter (`quietMs`, 116 records) at most 404 ms,
  so the 60 second window is 149 times the longest.
- The Markdown linter program on the four largest Markdown files of the repository (188 kB to 748 kB), five runs each,
  through standard input, with and without `--fix`: 8 ms to 43 ms, run-to-run spread at most 25 ms (a cold first run).
  This is the command-line program, not the thread the wrapper links; the gate figure is the one that covers the thread.
- Standards-review log, `.git/cli-git/standards-review-shadow.md`: 1,598 calls to the decisions endpoint
  (2026-10-31T01:38:57Z to 2026-11-04T13:57:13Z): median 0.9 s, 99th percentile 2.6 s, longest 9.8 s.
  (20 older entries went through the `pi` route and took 48 s to 251 s; they are not this call.)
- User journal, `review_ms` of 1,764 completed reviews (2026-11-01T12:19:57-05:00 to 2026-11-04T09:01:52-05:00):
  median 668 ms, 99th percentile 2,351 ms, longest 18,208 ms.
  Reference updates, 2,093 `answered` lines: `ref_ms` median 3 ms, 99th percentile 47 ms, longest 3,438 ms.
- Margin: the smallest whole number of minutes that is at least five times the longest figure,
  the factor `SERVICE_HOLD_CEILING` and `WITHDRAW_CEILING` use.
  Repair: 5 times 211.2 s is 1,056 s, so 18 minutes.
  Lint: 5 times 38.6 s is 192.8 s, so 4 minutes.
  Call: 5 times 9.8 s is 49 s, so 1 minute.
  Review: 5 times 18.2 s is 91 s, so 2 minutes.
  The factor is the coordinator's reading of "generous", open to the owner's veto.
- Limits of the sample: one repository, two days, warm package stores.
  No `pnpm install` from an empty store was logged, so the repair ceiling is the one most likely to be reached by a first install of a
  large dependency graph.
  The linter and review samples hold small files; a very large Markdown file with many linked images, which the linter hashes, is not in them.

### Left to the owner

- Decided by the owner after this section was first written, and built (next sections): the "Short extra wait" for the commit
  inside a stuck write, and the width of the steps without a ceiling.
- Whether a service stuck in a write should ever end itself so the manager restarts it, and recovery settles the batch.
  Decided (the coordinator, quoting the owner's earlier answers "Never cut a branch write (Recommended)" and "never discard user
  data"): never; a restart would end the write in the middle.
  The error names the restart command for the person to run when no program uses the repository.
- The readers `RERUNNABLE` does not name are not run again after a quiet window although they are safe to run twice.
  Not changed here.

### The cut experiment, 2026-11-04T10:30:00-05:00

The owner's criterion, as the coordinator put it: a step keeps its ceiling exactly when it can run twice.
Decided on a throwaway repository, per form of each step the wrapper starts that changes the repository in place.
Method (`/var/home/user/temp/agent/ceiling-measure/cut-experiment.ts`, Git 2.56.0, run in a capped scope):

- Build a repository from a fixed template with fixed author and committer dates, so object names repeat.
- Run the step uncut on a copy and take its end state: every file of the Git directory except the object store and the raw index
  (refs, packed refs, reflogs, `HEAD`, `ORIG_HEAD`, configuration, stash reflog), the index listing, the worktree files with their
  content hashes and modes, the stash list.
- Count the mutating system calls of the step under `strace -f` (`rename`, `unlink`, `link`, `symlink`, `mkdir`, `rmdir`, `write`,
  `fsync`, `ftruncate`, `chmod`, `utimensat` and their variants).
  `strace` numbers each call apart, so a cut point is a call name and its k-th occurrence.
- For every cut point, on a fresh copy, end the step there with `SIGKILL` and again with `SIGTERM` (`strace --inject`),
  remove the lock files a killed Git leaves (what the wrapper's stale-lock sweep does once the process is gone),
  run the step a second time, and compare the end state with the uncut run's.
- A form that came out identical at every cut, and whose second run always exited zero, can run twice.

Results (cuts are the cut points times two signals):

- Identical, so they keep a ceiling: `update-ref REF VALUE` and `update-ref -d REF` on a ref with no reflog (12 and 14 cuts),
  `stash list` (a read, 2), `clean --force -d` (16), `read-tree --reset -u` (34), `update-index --refresh` (6), `--index-info` (6)
  and `--add --cacheinfo` (6), `add --update` (30) and `add --all` (32) on the real index, `config` local write (10),
  `checkout-index --all --force` (24), `restore --staged --worktree` (10), `reset COMMIT -- PATH` (6).
- Different at some cut, so they have no ceiling:
  `update-ref --stdin` of two refs (32 cuts, 14 different: the scratch ref is left and the second run fails),
  `update-ref REF NEW OLD` (12, 4 different, 5 second runs failed), `update-ref --stdin` of one update (18, 4, 7),
  `update-ref REF NEW` of a branch (12, 7 different: a branch has a reflog and the second run writes another entry),
  `stash store` (8, 2 different: the stash reflog), `merge --ff-only` (54, 40 different, 11 second runs failed),
  `reset --hard` (80, 35 different: `ORIG_HEAD` and the reflogs), `read-tree -u -m OLD NEW` (14, 11, 11),
  `checkout BRANCH` (46, 40, 11) and `switch BRANCH` (46, 40, 11).
- Not run, so counted with the different ones because data is lost by guessing wrong and a wait is not:
  every other form, such as `reset --soft`, `update-ref` of a ref outside `refs/cli-git/`, `stash push`.
- Limits: one small repository, Git 2.56.0, local filesystem; a step cut part way through a large worktree could differ in ways a
  small one does not; `fsync` was off in the template (`core.fsync=none`), so a power cut is not modelled, only an ended process.
  Reflog and `ORIG_HEAD` differences are counted as differences even where the visible state is the same.
  The experiment ran `checkout` and `switch` although the wrapper never starts them for itself (they are the person's own commands,
  forwarded, never watched), so they matter only as evidence about the class.

`step_bound` in `src/native/transaction/transaction_git.rs` encodes the table;
`a_step_keeps_its_ceiling_exactly_when_the_cut_experiment_ran_it_twice_to_the_same_state` pins it
(red at the commit before it, against the wide list).
The experiment shows the first build's list was wider than the criterion:
`clean`, `read-tree --reset`, `update-index`, `add`, `checkout-index`, `restore`, `config` and `stash list` keep their ceilings,
and `stash list` (a read) had lost its by being counted as a `stash`.

### What a command does with a step that has no ceiling

- The commit service's landing thread waits for it as long as it lives (the con the owner accepted), and tells the clients.
- Any other command (the person's `git merge`, the stash carry, the cleanup) stops waiting at the bound of the step's class
  (120 seconds for a quick step, 10 minutes for a bulk one, the same measured numbers the ceilings have) plus `SHORT_EXTRA_WAIT`,
  and returns the error of `child_watch::NotKnown`:
  the step and its command line, the seconds, that it was left running and not ended, and what to check
  (`git status`, `git log -1`, `ps`, the disk).
  The step is left running and a thread reaps it when it ends.
  This is `StepBound::LeaveRunning` in `src/native/push/child_watch.rs`.
  The coordinator wrote that a command returns "at its own bound"; a person's `git merge` has no bound of its own,
  so the class ceiling of the step is the bound used here, and the choice is open to the owner's veto.
- The auto-push work thread runs its reconcile merge through the push machinery's own runner (`auto_push_reconcile.rs`,
  `run_query`), judged by the push's stall window and not through `run_git`; it is not a person's command and is unchanged.

### The short extra wait

Both owner answers are quoted beside `SHORT_EXTRA_WAIT` in `wait_ceilings.rs`:
commit protocol revision 12, "Short extra wait (Recommended)" ("the command waits a few more seconds, a constant to be measured,
for the write it started; if the disk is still stalled it exits non-zero saying the result is not known, naming `git log -1` and the
side ref"), and 2026-11-04, "7 s (Recommended)": twice the longest measured branch write (3,438 ms of 2,093 reference updates),
so the commit inside a stalled branch write exits not-known 7 seconds after its own bound.
The value is 7 seconds.

How the client learns that its request is inside the write, and why an older client or service is unharmed:

- The service sends the `waiting` line it already sends, with one extra field, `"inside_write": true`
  (`ServiceMessage::InsideWrite` in the code).
  Decoders read fields by name, so a client that does not know the field reads the line as the plain `waiting` notice it is
  (it prints the text and keeps it as the blocker its error names) and goes on as before;
  a service that does not send it leaves a client that knows it on the older path.
  The protocol version stays 7, so no client or service is retired for this.
  Releases never meet on commit traffic anyway: the client compares the release in the service's greeting and retires a service of
  another release before it sends a request.
  Test: `an_inside_write_notice_is_a_waiting_line_with_one_extra_field`.
- Once a client has heard it, at its `ANSWER_CEILING` it does not ask to be withdrawn (the commit may be on the branch already);
  it listens for the landing for `SHORT_EXTRA_WAIT` more, and then ends with the not-known failure
  (`ClientFailure::WithdrawalUnconfirmed`, the blocker naming the write and the scratch ref, and `git log -1` and `git status`).
  The write is left running and the commit lands when it returns.
  Test: `the_commit_inside_a_branch_write_that_does_not_return_is_reported_as_not_known_and_lands_when_the_write_returns`
  (red before: it waited the 45 seconds of the control's withdrawal ceiling; green: it returns in about the control's ceiling plus the
  extra wait).
- The service tells a client only after the write has run `NEVER_ENDED_STEP_SLOW_AFTER` (60 seconds);
  a client whose own bound falls inside that minute takes the older path (withdraw, wait `WITHDRAW_CEILING`).

### The factor of five, and the cold stores

The owner, 2026-11-04, "5x (Recommended)": keep five times the longest measured run, rounded up to whole minutes,
recomputed with the cold-store case if it is longer.
Cold-store timing (`cold-measure.ts`, a copy of this repository's sources without `.git`, a capped systemd unit with four CPUs of quota,
eight GiB of memory and an IO weight of 10; the output is kept in `/var/home/user/temp/agent/ceiling-measure/cold-measure.kept.out`):

- `pnpm install --frozen-lockfile` with an empty store directory and the network: 20.9 s.
  (The repair itself runs `pnpm install --offline --frozen-lockfile`, so a cold store makes it fail fast and not run long;
  the online figure is the longest a first install of this repository can take.)
- `cargo metadata --locked --offline --no-deps` in all 34 Cargo workspaces with no target directories: 1.7 s in all, 1.3 s the longest.
- `cargo fetch --locked` with an empty `CARGO_HOME` in `package/git-policy/cli`: 7.6 s.
- All three are shorter than the warm-store maxima (53.1 s and 211.2 s), so the longest run is unchanged and the ceilings stay
  18 minutes, 4 minutes, 1 minute and 2 minutes.
- Limits: the page cache cannot be dropped without privileges, so "cold" is a cold store and a cold registry, not a cold disk;
  the clone of real build directories, which dominates the Cargo stage, needs real target directories and is in the 69 add records
  (`cargoCloneTree`, up to 46 s per workspace); the host load average was 3.7 to 5.9 of 28.

### Item 3, as the coordinator and owner decided

A service stuck in a write never restarts itself.
The owner's answers behind it: "Never cut a branch write (Recommended)" and "never discard user data" (a restart ends the write in the
middle, which the first answer declines).
