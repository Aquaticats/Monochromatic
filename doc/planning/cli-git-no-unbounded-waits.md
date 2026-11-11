# Every wait of the git wrapper has a bound

Note 2026-11-04T22:23:58-05:00:
the program `cli-git-native` is renamed `monochromatic-git-policy-cli`,
the command word `git cli-git` becomes `git monochromatic-git-policy-cli`,
the settings file `cli-git.config.jsonc` becomes `monochromatic-git-policy-cli.config.jsonc`,
and the folders,
logs and service units that carry `cli-git` carry `monochromatic-git-policy-cli`
(the list is in `doc/planning/git-policy-cli-program-rename.md`).
The new names arrive with the first release after this date;
this repository switches its installed program and its settings file when it installs that release.
This record keeps the names of its time.

This is the inventory the new owner rule asks for,
with the bound proposed for each wait and what happens at the bound.
It was written on `fix/git-policy-cli-every-wait-has-a-bound` from the 0.5.28 base (`14c26dbcc9`) on 2026-11-03.
Paths are relative to `package/git-policy/cli`.

## The rule and the answers it rests on

The owner,
said in the session at about 2026-11-03T12:37-05:00 (quoted):
"nothing should ever wait forever or indefinitely".
It is newer than,
and overrides,
the unbounded waits built under the delegated picks 2 and 6 of
`doc/planning/cli-git-shipped-limits-audit.md`,
the follow-up pick that waits for an owner nothing proves alive with no
bound,
and the owner's earlier "Wait while connected (Recommended)" for pushes.
The coordinator flags that conflict for the owner and treats the newer rule as governing.
0.5.28 does not ship until every wait has a bound.

The owner keeps:
"Commits should never fail.";
"never fail doesn't primarily mean retries!";
"never discard user data";
"You should (almost) never be waiting."

The owner answered through the question tool at about 2026-11-03T12:45-05:00:

- At a wait's bound:
  "Return with an error" (the option read:
  "The command stops waiting and exits non-zero,
  naming what
  blocked it;
  the saved work stays saved.").
- Whether the saved commit should still land by itself later:
  "No,
  keep it for a retry (Recommended)" (the option read:
  "Nothing lands later by itself.
  Your staged changes stay staged,
  and the commit is also kept on a side ref.
  The error names
  the blocker and the command to run again.").
- Pushes:
  "Bound it,
  keep retrying (Recommended)" (the option read:
  "A connected push that stops making progress for a measured
  time is ended;
  the service's retry plan keeps the push and tries again;
  the command returns with a line saying the push is
  queued.").
  This replaces "Wait while connected".

So at the bound of a wait on the commit path the request is withdrawn so it can never land later,
durably,
the person's index
and worktree stay as they were,
the built commit stays on a side ref that the error names,
and the command exits non-zero with
one message that names the blocker and the command to run again.
The withdrawal races correctly with a landing that starts at the bound:
either the commit landed,
and success is reported,
or
it never will.

Everything here is the coordinator's reading under the owner's delegation and open to the owner's veto,
except the three
answers quoted under `The rule and the answers it rests on`.

## Method

The sources are every non-test Rust file of the crate (500 files).
The searches were:
`thread::sleep`,
`recv(`,
`recv_timeout`,
`Condvar`,
`.wait(`,
`wait_with_output`,
`.status()`,
`.output()`,
`accept(`,
`incoming()`,
`read_line`,
`flock`,
`try_wait`,
`set_read_timeout`,
`poll(`,
and every `loop {` and
`while` that is not a parser over a finite input (about 90 loops read,
the rest are parsers and tree walks).
The test helpers were read for waits that could hang a gate;
`.config/nextest.toml` ends any test at five minutes,
which is the
bound of the gate itself.
Status words:
"built" is built on this branch,
"bounded" already had a bound,
"exempt" is an idle wait by design,
"coordinator" is an unclear end behavior with options under the `Options for the coordinator to pick` heading.

## The waits

### Locks

1.  `src/native/service_index_attempts.rs` `wait_and_place` (landing waits for a foreign `index.lock`).
    Waits for the lock to go;
    ended today by release or a dead owner (a PID file naming a gone process),
    nothing else.
    Bound:
    `LOCK_CEILING`.
    At the bound:
    the batch is withdrawn before the branch moved,
    the built commit is kept on a side ref,
    the client exits non-zero naming the holder and the command to run again.
    Built.
2.  `src/native/index_holder_wait.rs:119` `wait_for_index_release` (a merge waits for any `index.lock`).
    Ended by release or a dead owner.
    Bound:
    `LOCK_CEILING`.
    At the bound:
    the merge stops,
    nothing changed,
    non-zero,
    naming
    the holder.
    Built.
3.  `src/native/service_index_wait.rs:162` `wait_for_service_hold` (a forwarded `git add`,
    `rm` or `mv` waits for the service's
    hold).
    Ended by release,
    a mark naming a gone holder,
    or a hold of `UNASKABLE_HOLD_AGE` whose holder cannot be asked about.
    Bound:
    `SERVICE_HOLD_CEILING`.
    At the bound:
    the command runs and reports what Git says.
    Built.
4.  `src/native/service_land_follow.rs:157` `update_index` (index reconciliation of a commit with no change left),
    through
    `wait_for_index_release_here`.
    Bound:
    the bound of entry 2.
    At the bound:
    the command ends,
    non-zero as it always does.
    Built.
5.  `src/native/service_hold_retry.rs:103` `run_again_while_service_holds` (up to `ATTEMPTS` 50 reruns,
    each preceded by entry 3).
    Bounded by count,
    not by time:
    the worst case is 50 times the bound of entry 3.
    Proposed bound:
    one deadline over the whole
    loop,
    `SERVICE_HOLD_CEILING`.
    Built.
6.  `src/native/service_index_lock.rs:97` `try_acquire` and `src/native/index_reset_support.rs:209`:
    one attempt,
    no wait.
    Bounded.
7.  Reference locks:
    `src/native/auto_push.rs:46` `PUSH_LOCK_RETRIES` 3 with `PUSH_LOCK_PAUSE` 250 ms;
    the stale lock judges
    (`stale_ref_lock_*.rs`) read and decide without waiting.
    Bounded.
8.  Lane and owner locks:
    `src/native/worktree_convert_journal.rs:290` uses `flock(LOCK_EX | LOCK_NB)`,
    which never waits.
    The discard audit appends (`discard_audit.rs:335`) loop only on an interrupted write.
    Bounded.

### The commit service, client side

9.  `src/native/service_client.rs:416` `submit_loop` (the client waits for the answer,
    with heartbeats).
    Ended by an answer,
    a silence of `HEARTBEAT_SILENCE`,
    or a close;
    nothing ends it while the service keeps sending.
    Bound:
    `ANSWER_CEILING`.
    At the bound:
    the client withdraws the request (below),
    exits non-zero naming what the service was
    waiting for when it could tell,
    with the side ref and the command to run again;
    if the request had landed,
    success.
    Built.
10. `src/native/service_client.rs:220` `reach` with a manager watch (waits for the service to start).
    Ended by a greeting or two `failed` answers of the manager;
    nothing else.
    Bound:
    `START_CEILING`.
    At the bound:
    "The commit service did not start" with the journal command,
    as for the other start failures.
    Built.
11. `reach` without a watch,
    `reach_current_untimed:330` (retire an older service),
    `ask_to_retire:517`:
    bounded by `START_WAIT`.
12. `reach` after a service was seen:
    the welcome read is bounded by `heartbeat_silence`.
    Bounded.
13. `src/native/service_client.rs:568` `wake`:
    reads until the service closes,
    each read bounded by `heartbeat_silence`.
    Bounded.
14. Reconnect loops (`RECONNECTS` 5) and the pauses between them (`CONNECT_PAUSE` 25 ms):
    bounded by count.
15. `src/native/systemd_dbus.rs:468` and the provisioning waits:
    `EXCHANGE_WAIT` 20 s and `RETIREMENT_WAIT` 10 s.
    Bounded.

### The commit service, server side

16. `src/native/service_reply.rs:69` `write_until_answered`:
    runs until the reply is dropped (the request is answered)
    or the
    client is gone.
    Ended by the landing thread's answer.
    Bounded by the landing:
    entries 1 and 17.
    No change.
17. `src/native/service_server.rs:423` `land_group` and the landing steps:
    the lock wait (entry 1),
    the Git child steps (entries
    21 and 22),
    `LANDING_ATTEMPTS` 3 and `REBUILD_ATTEMPTS` 5.
    Bounded once entries 1,
    21 and 22 are.
18. `src/native/service_server.rs:246` `wait_for_close` (a scratch connection):
    ended when the client closes or dies.
    Bounded by
    the client,
    whose every wait is bounded.
19. `src/native/service_server.rs:577` `serve` with `recv_timeout(idle)` (`IDLE_EXIT` 3 hours).
    Bounded,
    exempt idle.
20. `src/native/service_server.rs:537` `work_loop` (`receiver.recv()` for wakes)
    and the acceptor (`incoming()` at `:603`):
    idle by design;
    the service exits at `IDLE_EXIT` when nothing is busy.
    Exempt.
    The request read timeout (`REQUEST_READ_TIMEOUT` 10 s) and `ACCEPT_BACKOFF` bound the rest.
    `service_claim_renewal.rs` receives on its own threads and ends with its owner.
    Exempt.
    Work items:
    `MAX_TRIES` 3,
    `CLAIM_LEASE` 120 s.
    Bounded.

### Child processes

21. (Built,
    2026-11-03.)
    `src/native/child_watch.rs:396` `watch_loop` (Git steps of a commit):
    a step is ended after the quiet window (60 s quick,
    300 s bulk,
    both labelled unmeasured) of no sign of work;
    a step that shows work is waited for however long it takes.
    Ended today by completion or by quiet.
    No time bound.
    Coordinator:
    option set A.
22. Steps marked `never_stop` (`child_watch.rs:337`:
    reference updates,
    a commit Git builds):
    never ended by the wrapper.
    No bound.
    Coordinator:
    option set A.
23. `src/native/bounded_push.rs:453` and `src/native/push_progress.rs` (pushes,
    fetches and LFS uploads of the service):
    a push with no connection is ended after the stall window (30 s labelled unmeasured);
    a push with an open connection is never
    ended.
    Owner's answer:
    "Bound it,
    keep retrying".
    Bound:
    `CONNECTED_PUSH_QUIET_WINDOW` of no progress.
    At the bound:
    the
    push is ended,
    the service's retry plan keeps it,
    and the notice says the push is queued.
    Built.
24. `src/native/bounded_query.rs` read-only queries (120 s ceiling,
    `doc/planning/cli-git-shipped-limits-audit.md`).
    Bounded.
25. `src/native/forwarding.rs:186` `run_real_git` and `src/native/signal_relay.rs:290` `wait_without_reaping`:
    the wrapper
    waits for the Git command the person typed (editor,
    the person's hooks,
    a long checkout).
    Not a wait the wrapper chose;
    ended
    by the person (interrupt) or by Git.
    Coordinator:
    option set B.
26. `src/native/package_repair_command.rs:205` (600 s),
    `src/native/markdown_linter_guard.rs:142` (60 s),
    `src/native/standards_review_run.rs:210` `run_contained` (bounded by the review's own deadlines),
    `src/native/start_retry.rs`
    (`START_RETRY_BUDGET` 5 s),
    `src/native/commit_preparation.rs:168` (counted retries).
    Bounded.

### Polls, watches and channels

27. `src/native/service_index_watch.rs:157` `Watch::wait`:
    bounded by the timeout its caller passes.
    Bounded.
28. The poll loops of entries 1 to 3 (`POLL_PAUSE` 50 ms):
    bounded by the ceilings of those entries.
29. `src/native/transaction_phase.rs:289` `await_release`:
    a test marker compiled into the release executable,
    armed only by an
    environment variable;
    it waits for a file and gives up only when the directory is gone.
    Proposed bound:
    `PHASE_PATIENCE`.
    Built.
30. `src/native/child_watch.rs:233` and `:241` (`STOP_POLL` 10 ms while a stopped child is reaped):
    bounded by the stop grace
    (`STOP_GRACE` 5 s,
    then a kill).
    Bounded.
31. `src/native/signal_relay.rs:412` and `:476` `child.wait()` after a stop or an exit:
    bounded by the child's end.
    Bounded.

### Retry loops

32. The push retry plan (`src/native/push_retry.rs`):
    by design it keeps trying;
    each try is bounded by entry 23,
    and the plan
    spaces the tries (10 s to 10 minutes).
    This is the owner's "keep retrying" and is not a wait of the command.
33. `LANDING_ATTEMPTS` 3,
    `REBUILD_ATTEMPTS` 5,
    merge `MAX_ROUNDS` 20,
    `MAX_RECONCILE_ATTEMPTS` 10,
    claim take rounds 64,
    `INDEX_ATTEMPTS` 5.
    Bounded by count;
    none loops on a clock.

### Test helpers

34. `src/native/parallel_test_support.rs` (`Rendezvous`,
    patience set) and `src/native/stale_ref_lock_killed_git.rs:135`
    (`wait_for`,
    `HOLD_PATIENCE`):
    bounded.
35. Binary controls that wait on children (`command_test_support.rs:190`,
    `index_reset_support.rs:58`,
    `fixture_executable.rs:50`,
    `umask_child_test_support.rs:30`) have no timeout of their own;
    `.config/nextest.toml` ends a test at five minutes
    (`slow-timeout` 60 s,
    terminate after 5),
    so a hang ends the run with the test named.
    Bounded by the harness.

Built on this branch (2026-11-03):
entries 1 to 5,
9,
10,
23 and 29,
with a red test or a mutation red for each except 5 and 29,
which were written with their fix.
Entries counted as waits:
35 (rows 6 to 8,
11 to 15 and 24 to 31 are bounded already,
rows 19,
20 are idle by design).

## Bound values

The rule `LQM` asks for a measurement or an owner decision beside each value.
The data on this machine,
read on 2026-11-03 (read-only):

- The commit timing log of this repository,
  `.git/cli-git-commit-timing.jsonl`,
  1,329 records.
  It is the only timing log under the home directory (`find` over the home directory found no other).
  `answerWaitMs`:
  median 67 ms,
  99th percentile 50,344 ms,
  longest 308,267 ms.
  `totalMs`:
  median 688 ms,
  99th percentile 168,794 ms,
  longest 750,604 ms.
  `startWaitMs`:
  longest 1,041 ms. `reachMs`:
  longest 1,041 ms.
  `afterLandingMs`:
  longest 43,120 ms. `gateMs`:
  longest 38,551 ms. `prepareMs`:
  longest 250,280 ms.
- The service journal:
  1,335 landings;
  `lock_hold_ms` of 715 of them,
  longest 119,721 ms (all in the disk sync of the new
  index);
  `queued_ms` longest 89,874 ms;
  794 pushes that landed,
  whole-push times p50 3.5 s,
  p99 26.4 s,
  longest 72.4 s.
- No log holds a wait behind a foreign `index.lock` holder (no `lock_looks`,
  no waiting notice),
  and none holds a quiet gap of a
  connected push.

The values,
each with its margin:

- `ANSWER_CEILING` 1200 s:
  3.9 times the longest observed answer wait (308.3 s)
  and 1.6 times the longest whole commit (750.6 s).
- `START_CEILING` 60 s:
  58 times the slowest observed start (1.04 s).
  `START_WAIT` 10 s keeps its earlier citation.
- `SERVICE_HOLD_CEILING` 600 s:
  five times the longest observed service hold (119.7 s).
  It is also the bound of the whole rerun
  loop of entry 5.
- `LOCK_CEILING` 600 s:
  unmeasured when this was written,
  no foreign-holder wait was logged anywhere
  (measured on 2026-11-08 from the commit timing log and the service journal:
  the longest wait on a lock was 11,490 ms,
  so the ten minutes is 52 times it and no wait takes it;
  `doc/planning/cli-git-shipped-limits-audit.md`,
  "The lock ceilings' waits read again from the service journal").
  It is below `ANSWER_CEILING` so that the service
  ends its own wait and answers before the client gives up.
  A person's `git commit` with an editor open longer than this makes
  ours stop with the error;
  that is the price of the rule,
  listed in the options.
- `CONNECTED_PUSH_QUIET_WINDOW`:
  unmeasured,
  labelled as the other quiet windows are (see option set C).
- `WITHDRAW_CEILING` 600 s:
  the client's wait for the service's answer to a withdrawal;
  five times the longest observed landing
  (119.8 s).
  Built.
- `PHASE_PATIENCE` 300 s:
  a test marker,
  the longest any test may run (`.config/nextest.toml`).
  Built.

## Options for the coordinator to pick

### A. Git steps that keep working, and steps that are never stopped

Covers entries 21 and 22.
A step that shows signs of work is waited for however long it takes,
and a `never_stop` step (a
reference update,
a commit Git builds) is never ended:
the owner's "Stop only on no progress" and "End Git,
hold hooks" still
apply,
and "nothing should ever wait forever" now bounds them.

- A.
  A ceiling per class of step,
  with the longest observed step as the base (`prepareMs` 250 s,
  `buildMs` 183 s):
  at the ceiling
  the step is ended,
  the commit is withdrawn as in entry 9,
  and the error names the step.
  Pros:
  every step ends.
  Cons:
  a legitimately slow step on a huge repository fails the commit,
  and a `never_stop` step ended in
  the middle of a write can leave a lock or a half-written reference.
- B.
  A ceiling for steps that can be run twice,
  none for `never_stop` steps,
  which are bounded by the commit's `ANSWER_CEILING`
  (the client withdraws at that bound and the service finishes the step it holds).
  Pros:
  nothing is ended in the middle of a write;
  the person gets the error at a known time.
  Cons:
  the service thread stays
  busy with the step after the client left.
- C.
  A ceiling for every step that is a multiple of the longest measured step,
  with a hard stop for `never_stop` steps after a
  longer one.
  Pros:
  one rule.
  Cons:
  ends a write in the middle,
  the case the owner's "never discard user data" is wary of.

Ranking:
B > A > C,
because B keeps the owner's rule that a write is ended only by the person while still giving the
person an
error at a known time;
B > A because A ends a step that may be mid-write only for a number nobody measured;
A > C because C
extends the same risk to the steps that cannot be run twice.

### B. The person's own Git and hooks

Covers entry 25.
The wrapper waits for the Git command the person typed;
the hooks,
the editor and a long checkout are the
person's.
The wrapper runs none of the person's hooks itself (every Git it starts carries `core.hooksPath` below
`/dev/null`).

- A.
  Exempt:
  the wait is the person's command,
  ended by the person,
  as with native Git.
  Pros:
  the wrapper does not change what `git` does when it forwards.
  Cons:
  it reads as a wait with no bound.
- B.
  A ceiling on forwarded commands that are not interactive (no editor,
  no terminal prompt),
  none on interactive ones.
  Pros:
  bounds scripts.
  Cons:
  ends a long `git checkout` or `git gc` the person asked for;
  "never discard user data".
- C.
  A notice after a long wait,
  no bound.
  Pros:
  tells the person what is running.
  Cons:
  still no bound.

Ranking:
A > C > B,
because the wait is the person's own command;
A > C because C adds output with no change in outcome;
C > B
because B ends the person's work.

### C. The quiet window of a connected push

Covers entry 23.
The owner chose to bound a connected push that stops making progress and keep it in the retry plan.
The number is not measured.

- A.
  The existing stall window (30 s,
  labelled unmeasured) for connected pushes too.
  Pros:
  one number.
  Cons:
  a slow remote that answers after 40 s is ended and retried;
  the whole-push p99 is 26.4 s and the
  longest 72.4 s.
- B.
  Twice the longest whole push in the journal (72.4 s),
  about 150 s.
  Pros:
  above everything observed.
  Cons:
  a stuck push holds the branch's push for two and a half minutes.
- C.
  Ten times the p99 (264 s).
  Pros:
  very safe.
  Cons:
  long.

Ranking:
B > A > C.
B is built as the value,
labelled unmeasured as a quiet window;
B > A because A is below the longest observed
push;
A > C because C holds every later push of the branch for minutes.

### D. A person's `git commit` with an editor open when the bound passes

Covers entry 1 and 2 at their bound.
A foreign `git commit` holds `index.lock` for as long as its editor is open.

- A.
  As built:
  our commit is withdrawn at `LOCK_CEILING` with the error and the side ref.
- B.
  A longer ceiling (30 minutes,
  the earlier unmeasured value).
- C.
  Treat an owner whose process shows work as a different case from one that shows none.

Ranking:
A > B > C,
because A matches the owner's answer;
B only moves the number;
C needs a measure of a foreign process's work.

### E. Idle waits that are not bounded in time

Covers entry 20.
The work loop and the acceptor wait for events with no bound;
the service exits at `IDLE_EXIT` (3 hours) when
nothing is busy.
Exempt by design;
if the owner reads the rule as covering them too,
the service's idle exit already bounds them.

## Picks

Picked by the coordinator under the owner's delegation,
veto open,
2026-11-03:

- A,
  not option B:
  every step class gets a service-side ceiling,
  including steps that keep showing work and steps that are never
  ended for being quiet.
  Under B a `never_stop` step that never returns would keep the service's landing thread for
  good,
  so every
  later commit on that repository would wait out `ANSWER_CEILING` and fail,
  which is a wait forever.
  Ending a reference update or a
  commit Git builds cannot half-write a reference (Git writes a lock file and renames it);
  the residue is a lock the shipped
  stale-lock removal clears once the holder is proven gone ("End Git,
  hold hooks" is the owner's earlier pick).
  Each ceiling sits below
  `ANSWER_CEILING`,
  so the service answers first.
  Built:
  `PLUMBING_STEP_CEILING` 120 s (the longest read step of 1,651 landings
  took 9,663 ms),
  `OBJECT_STEP_CEILING` 600 s (the longest client preparation of 1,400 records took 250,280 ms;
  the service's
  object step at most 1,358 ms),
  `REFERENCE_STEP_CEILING` 60 s (the longest of 1,651 reference updates took 3,438 ms).
  After a
  reference update is ended the branch is read:
  at the request's commit,
  the outcome is `Landed`;
  otherwise the batch is withdrawn
  exactly as at `LOCK_CEILING` (intent removed,
  commit kept on `refs/cli-git/kept/<batch>`,
  staging unchanged,
  lock given up,
  and the
  error names the step).
  Hooks are still never ended by the wrapper.
  Amended 2026-11-04T09:35:19-05:00 by the owner ("Never cut a branch write (Recommended)"):
  the reference update,
  and every other
  write in place that is not safe to run twice,
  has no ceiling;
  `REFERENCE_STEP_CEILING` and the read of the branch after an ended
  update are gone (see "Branch write and tool ceilings,
  2026-11-04T09:35:19-05:00" below).
  The reasoning above,
  that a hung
  write holds the landing thread for good,
  is the con the owner accepted.
- B:
  A,
  exempt:
  the wait for the person's own Git command and hooks is the person's,
  as with native Git.
- C:
  B,
  150 s as built.
- D:
  A,
  the withdrawal at the lock ceiling as built.
- E:
  exempt.
- The owner's push answer ("Bound it,
  keep retrying") said the command says the push is queued:
  a push ended for showing no
  progress now prints "The push is queued:
  the commit service keeps it in its retry plan and tries it again."
  (`PUSH_QUEUED_LINE`),
  and a test reads that line from the output.

## Branch write and tool ceilings, 2026-11-04T09:35:19-05:00

The owner answered two questions through the question tool on 2026-11-04.
They amend the pick of option set A (every step class has a ceiling,
the reference update's being 60 seconds)
and the outcome of pick 9 of `doc/planning/cli-git-shipped-limits-audit.md` (tools and checks end only on no progress).
Paths are relative to `package/git-policy/cli`.

### The owner's answers

"Never cut a branch write (Recommended)".
The option read:
"Steps that can safely run twice keep their ceilings;
the branch write is never ended.
Your command still returns at its bound with "result not yet known" and the `git log -1` and side-ref hint
(your "Short extra wait" answer).
Con:
a truly hung write keeps the service busy,
and later commits fail at their own 20-minute ceiling with an error naming it."

"Silence plus a ceiling (Recommended)".
The option read:
"Keep ending on silence,
and add a generous overall ceiling per tool,
measured from real runs,
after which it's ended with an error naming the tool.
Con:
an extremely slow but working run on a huge repository can hit the ceiling."
It covers the non-Git tools of `git worktree add` and the review:
package repair,
the Markdown linter,
the standards-review call and the whole review.

Also carried out,
from the follow-up of pick 7 of the audit that was recorded and never run:
`FOREIGN_OWNER_PATIENCE` is twice `LOCK_CEILING`,
20 minutes,
derived in code (see the audit).

### What changed for the branch write

`REFERENCE_STEP_CEILING` (60 seconds) is deleted.
`src/native/transaction/transaction_git.rs` `step_bound` gives a step a ceiling only when `may_be_ended` says so (it can be run twice
by `may_be_rerun`,
it only reads or writes unreferenced objects,
or it is one of the exact forms the experiment proved);
which forms those are was decided by experiment (`The cut experiment`),
not by reading,
and every other form has no ceiling.
The first build of this branch (`a8b7473fc7`) had instead a list of subcommands,
`WRITES_IN_PLACE`,
chosen by reading;
the coordinator's follow-up of 2026-11-04 replaced it with the experiment.
The landing's branch move no longer reads the branch after an ended update:
that arm of `service_land.rs` is gone with the ceiling that produced it.

What the clients are told,
because the single landing thread also answers every withdrawal between two landings:

- `src/native/service/service_busy.rs` is the shared record of what the landing thread is doing.
  The Git runner writes into it when a step with no ceiling begins,
  and the landing adds what it knows
  (the branch and commit the update moves,
  the scratch ref that holds the commit until the update ends).
- After `NEVER_ENDED_STEP_SLOW_AFTER` (60 seconds,
  17 times the longest of 2,093 reference updates,
  `ref_ms` 3,438 ms) the thread
  that reads each connection sends a `waiting` message naming the write,
  again whenever the whole-minute count changes.
  The client prints it,
  and keeps it as the blocker its error names (`StillWithService`,
  and now `WithdrawalUnconfirmed`).
  A client whose own request is the landing thread's is told at once,
  when the write starts,
  with the `inside_write` line
  (see "The short extra wait"),
  on the commit connection and on the connection it opens to withdraw.
- A `withdraw` for a request that is not in the group being landed is answered by the connection thread
  (`withdrawn`,
  or `landed` when a finished batch of the journal names it) while the record shows a stuck write;
  a request that a batch still open names is left to the landing thread,
  because that batch may hold its commit on the branch
  already (`a_request_whose_commit_is_on_the_branch_in_an_open_batch_is_not_withdrawn_early`,
  found in review of this branch;
  red before the fix);
  the request is remembered as a tombstone and dropped whenever a copy of it reaches a group (the review of 2026-11-04 found that the
  first build consumed it with the first copy).
  The record holds no lock (the first build used a mutex and the
  gate's lock inventory refused it,
  `source_guard/service_lock_inventory_tests.rs`,
  in keeping with "Use as few locks as
  possible."):
  admission and cancellation are one decision per request,
  the file `<id>.decision` in `busy` under the journal,
  made by `link(2)` of a whole file,
  which exactly one side can create:
  `landing` by the landing thread before it lands the request,
  `withdrawn` by a connection thread before it answers.
  The loser reads the winner's content.
  A `landing` file is removed when its
  group is done,
  after the batch is in the journal;
  a withdrawal that wins after that asks the journal again and answers `landed`
  for a request the journal names.
  Anything that cannot be established (a file that cannot be made or read,
  a journal that cannot be
  read whole) is never evidence of a cancellation:
  the withdrawal is left to the landing thread,
  and a request that cannot be
  admitted is answered with a failure and not landed.
  Tombstones are kept up to the number of finished batches the journal
  keeps (`DONE_RETAINED`),
  oldest first out,
  like the service's memory of normal withdrawals,
  and are cleared when the service
  starts,
  as that memory is.
  So a later commit ends at its own `ANSWER_CEILING` (20 minutes),
  not 10 minutes after it,
  and the error names the write.
- The request inside the stuck write is not withdrawn that way:
  its commit may be made by the write.
  Its client is told that,
  and at its `ANSWER_CEILING` waits `SHORT_EXTRA_WAIT` (7 seconds,
  "The short extra wait" below) and then
  ends with the failure that says it is not known whether the commit was made,
  names the write and the scratch ref,
  and gives
  `git log -1` and `git status`.
  (The first build of this section had the client wait `WITHDRAW_CEILING`,
  10 minutes,
  for the landing;
  that is the older path,
  taken only when the client was not told in time,
  and it is what the later section replaced.)
- Protocol messages and the protocol version are unchanged.

Tests (red first where the behavior existed):
`a_branch_write_that_outlasts_the_step_ceiling_is_not_ended` (red at `afad55b58d`:
`StepTooSlow`,
the old arm ended the write at the
shortened ceiling and withdrew the commit),
`a_write_in_place_has_no_ceiling_and_every_other_step_keeps_one`,
`a_shortened_ceiling_never_reaches_a_write_in_place`,
`a_step_with_no_ceiling_is_waited_for_until_it_ends_by_itself`,
the board's controls in `service_busy_tests.rs`,
and in `landing/service_land_hung_write_tests.rs` two controls against a real `serve` with a Git whose `update-ref --stdin` waits for a
release file:
`a_commit_queued_behind_a_branch_write_that_does_not_return_is_withdrawn_at_its_ceiling_naming_the_write`
(shown to fail with the early answer disabled,
in 49 s,
then restored)
and `the_commit_inside_a_branch_write_that_does_not_return_is_reported_as_not_known_and_lands_when_the_write_returns`.
The two controls that encoded the removed behavior (`a_reference_update_ended_after_it_renamed_the_branch_reports_landed`,
`a_reference_update_ended_before_it_renamed_the_branch_withdraws_the_commit`) are deleted.

### What changed for the tools

Each tool keeps its silence window and gains a ceiling on its whole run.
The windows are unchanged:
the owner chose ceilings,
not new windows.

- `REPAIR_TOOL_CEILING` 18 minutes:
  `run_tool` ends the program with its whole process group
  (`ToolEnd::CeilingReached`);
  the finding names the tool,
  what it was doing,
  the seconds,
  and the command to run by hand.
  Test:
  `a_program_that_keeps_working_is_ended_at_its_ceiling_with_its_whole_process_group`
  (red on the unfixed code,
  observed before the fix and committed with it in `95c46239e7`:
  the tool,
  which printed all the time,
  ran to its end,
  20.2 s;
  the control also checks that a helper of the tool is gone).
- `LINT_CEILING` 4 minutes:
  the Markdown check runs on a thread that cannot be stopped from outside,
  so the ceiling abandons it,
  as the quiet window does,
  and the reason names the tool,
  the file and the ceiling.
  Test:
  `a_job_that_keeps_working_is_abandoned_at_its_ceiling` (red at `af74896d85`:
  the guard never answered while the job worked,
  and the run was ended by the unit-test timeout at 120 s).
- `REVIEW_CALL_CEILING` 1 minute:
  `ureq`'s `global` timeout;
  the sentence names the call,
  the ceiling and
  `SHADOW_REVIEW_SHA=<commit> mise run shadow-review`.
  Test:
  `a_reply_that_keeps_arriving_is_ended_at_the_ceiling` (red at `6f439cb4a1`:
  the reply was read whole).
- `REVIEW_CEILING` 2 minutes:
  `run_contained` waited with `recv()` and no bound at all,
  so the local write of the log entry
  was unjudged;
  it now stops waiting at the ceiling and the item fails with the sentence of `ceiling_reason`.
  Test:
  `the_runner_stops_waiting_for_a_job_at_its_ceiling` (no red run:
  the old runner had no ceiling to test against,
  and the control would have waited for ever).
- The owner's message names a call window of 10 seconds and a review of 30 seconds.
  The shipped values are a 10 second window in the service (`CALL_QUIET_WINDOW`) and 30 seconds as the script's default
  (`SCRIPT_QUIET_WINDOW`),
  and the whole review had no deadline since pick 9,
  so the review's ceiling is new,
  not a replacement.

### Measurements

Read on 2026-11-04 between 09:00 and 09:35 (-05:00),
read-only,
on this host under the load it had
(load average 3.7 on 28 threads at the start of the linter timing).
The scripts are `/var/home/user/temp/agent/ceiling-measure/measure.ts`,
`measure-lint.ts`,
`quiet-names.ts` and `commits.ts`.

- Worktree add timing log,
  `.git/cli-git-worktree-add-timing.jsonl`:
  79 adds,
  2026-11-03T08:55:46Z to 2026-11-04T13:53:17Z,
  releases 0.5.20 to 0.5.29;
  69 records hold the per-tool `work` figures.
  A stage is one `work` entry with its call count,
  so a figure with many calls is a total.
  Longest stage:
  `repairPnpm` 53.1 s (one call),
  `repairToolchain` (`mpm-shadow install`) 30.0 s (one call),
  `repairCargo` 211.2 s (the whole Cargo stage,
  which holds every `cargo metadata` and `cargo clean` of the add,
  34 workspaces at most,
  so it bounds any one of them).
  Longest add 539.7 s.
- Quiet gaps of the add log (`quiet`,
  12 records,
  resolution 500 ms):
  `cargo clean` 30.3 s,
  `mpm-shadow install` 7.6 s,
  `cargo metadata` 0.5 s,
  `pnpm install` 0.5 s.
  The 600 second window is 19.8 times the longest.
  Too small a sample to change the window,
  and unchanged.
- Commit timing logs (`cli-git-commit-timing.jsonl` and `.1`):
  1,838 commits up to 2026-11-04T14:08:47Z (the first record of the
  rotated file has no `time` field,
  so the start of the sample is not known;
  the oldest record that has one is 2026-11-02),
  longest `gateMs` 38,551 ms (99th percentile 943 ms),
  longest `totalMs` 750,604 ms;
  quiet gap of the Markdown linter (`quietMs`,
  116 records) at most 404 ms,
  so the 60 second window is 149 times the longest.
- The Markdown linter program on the four largest Markdown files of the repository (188 kB to 748 kB),
  five runs each,
  through standard input,
  with and without `--fix`:
  8 ms to 43 ms,
  run-to-run spread at most 25 ms (a cold first run).
  This is the command-line program,
  not the thread the wrapper links;
  the gate figure is the one that covers the thread.
- Standards-review log,
  `.git/cli-git/standards-review-shadow.md`:
  1,598 calls to the decisions endpoint
  (2026-10-31T01:38:57Z to 2026-11-04T13:57:13Z):
  median 0.9 s,
  99th percentile 2.6 s,
  longest 9.8 s.
  (20 older entries went through the `pi` route and took 48 s to 251 s;
  they are not this call.)
- User journal,
  `review_ms` of 1,764 completed reviews (2026-11-01T12:19:57-05:00 to 2026-11-04T09:01:52-05:00):
  median 668 ms,
  99th percentile 2,351 ms,
  longest 18,208 ms.
  Reference updates,
  2,093 `answered` lines:
  `ref_ms` median 3 ms,
  99th percentile 47 ms,
  longest 3,438 ms.
- Margin:
  the smallest whole number of minutes that is at least five times the longest figure,
  the factor `SERVICE_HOLD_CEILING` and `WITHDRAW_CEILING` use.
  Repair:
  5 times 211.2 s is 1,056 s,
  so 18 minutes.
  Lint:
  5 times 38.6 s is 192.8 s,
  so 4 minutes.
  Call:
  5 times 9.8 s is 49 s,
  so 1 minute.
  Review:
  5 times 18.2 s is 91 s,
  so 2 minutes.
  The factor is the coordinator's reading of "generous",
  open to the owner's veto.
- Limits of the sample:
  one repository,
  two days,
  warm package stores.
  No `pnpm install` from an empty store was logged,
  so the repair ceiling is the one most likely to be reached by a first install of a
  large dependency graph.
  The linter and review samples hold small files;
  a very large Markdown file with many linked images,
  which the linter hashes,
  is not in them.

### Left to the owner

- Decided by the owner after this section was first written,
  and built (next sections):
  the "Short extra wait" for the commit
  inside a stuck write,
  and the width of the steps without a ceiling.
- Whether a service stuck in a write should ever end itself so the manager restarts it,
  and recovery settles the batch.
  Decided (the coordinator,
  quoting the owner's earlier answers "Never cut a branch write (Recommended)" and "never discard user
  data"):
  never;
  a restart would end the write in the middle.
  The error names the restart command for the person to run when no program uses the repository.
- The readers `RERUNNABLE` does not name are not run again after a quiet window although they are safe to run twice.
  Not changed here.

### The cut experiment, 2026-11-04T10:30:00-05:00

The owner's criterion,
as the coordinator put it:
a step keeps its ceiling exactly when it can run twice.
Decided on a throwaway repository,
per form of each step the wrapper starts that changes the repository in place.
Method (`/var/home/user/temp/agent/ceiling-measure/cut-experiment.ts`,
Git 2.56.0,
run in a capped scope):

- Build a repository from a fixed template with fixed author and committer dates,
  so object names repeat.
- Run the step uncut on a copy and take its end state:
  every file of the Git directory except the object store and the raw index
  (refs,
  packed refs,
  reflogs,
  `HEAD`,
  `ORIG_HEAD`,
  configuration,
  stash reflog),
  the index listing,
  the worktree files with their
  content hashes and modes,
  the stash list.
- Count the mutating system calls of the step under `strace -f` (`rename`,
  `unlink`,
  `link`,
  `symlink`,
  `mkdir`,
  `rmdir`,
  `write`,
  `fsync`,
  `ftruncate`,
  `chmod`,
  `utimensat` and their variants).
  `strace` numbers each call apart,
  so a cut point is a call name and its k-th occurrence.
- For every cut point,
  on a fresh copy,
  end the step there with `SIGKILL` and again with `SIGTERM` (`strace --inject`),
  remove the lock files a killed Git leaves (what the wrapper's stale-lock sweep does once the process is gone),
  run the step a second time,
  and compare the end state with the uncut run's.
- A form that came out identical at every cut,
  and whose second run always exited zero,
  can run twice.

Results (cuts are the cut points times two signals):

- Identical,
  so they keep a ceiling:
  `update-ref REF VALUE` and `update-ref -d REF` on a ref with no reflog (12 and 14 cuts),
  `stash list` (a read,
  2),
  `clean --force -d` (16),
  `read-tree --reset -u` (34),
  `update-index --refresh` (6),
  `--index-info` (6)
  and `--add --cacheinfo` (6),
  `add --update` (30) and `add --all` (32) on the real index,
  `config` local write (10),
  `checkout-index --all --force` (24),
  `restore --staged --worktree` (10),
  `reset COMMIT -- PATH` (6).
- Different at some cut,
  so they have no ceiling:
  `update-ref --stdin` of two refs (32 cuts,
  14 different:
  the scratch ref is left and the second run fails),
  `update-ref REF NEW OLD` (12,
  4 different,
  5 second runs failed),
  `update-ref --stdin` of one update (18,
  4,
  7),
  `update-ref REF NEW` of a branch (12,
  7 different:
  a branch has a reflog and the second run writes another entry),
  `stash store` (8,
  2 different:
  the stash reflog),
  `merge --ff-only` (54,
  40 different,
  11 second runs failed),
  `reset --hard` (80,
  35 different:
  `ORIG_HEAD` and the reflogs),
  `read-tree -u -m OLD NEW` (14,
  11,
  11),
  `checkout BRANCH` (46,
  40, 11) and `switch BRANCH` (46,
  40,
  11).
- Added in round 3 (2026-11-04T14:20:00-05:00 onward,
  same method):
  identical,
  `update-ref --no-deref REF VALUE` with
  `-c core.logAllRefUpdates=false` on the command line while the configuration says `always` (12 cuts),
  the same with REF a symbolic ref
  to a branch (8),
  `update-ref -d --no-deref REF` with the setting (14),
  `symbolic-ref HEAD REF` in a bare repository (4).
  Different:
  `init --bare --template= --ref-format=reftable DIR`,
  run again in place (104 cuts,
  54 identical and 50 different:
  a cut
  leaves reftable tables the second run does not replace;
  every second run exited zero),
  and `fast-import --done --export-marks=FILE` with the wrapper's own stream,
  the setting on the command line and the
  configuration saying `always` (88 cuts,
  2 different,
  both second runs failed on a `marks-1.lock` a cut left in the worktree).
  Output files:
  `cut-experiment-round3.out` and a run of the `fast-import` case.
  Correction of 2026-11-04 (round 4):
  the first run of
  the `init` case gave the step an absolute path inside the template directory,
  so its cuts and second runs acted on one shared
  directory;
  the figures "64 cuts,
  57 second runs failed" that this paragraph and the first round 3 report gave were that artifact and
  are withdrawn.
  The run with the path relative to each copy (`cut-experiment-init-r4.out`,
  `cut-experiment.ts`) is the one above.
  The
  same case with the partial directory discarded before the second run (the fresh shadow of `init_with_git`) came out identical in
  104 of 104 cuts,
  normalizing the random reftable table names and checking the refs through the repository.
- Not run,
  so counted with the different ones because data is lost by guessing wrong and a wait is not:
  every other form,
  such as `reset --soft`,
  `update-ref` of a ref outside `refs/cli-git/`,
  `stash push`.
- Limits:
  one small repository,
  Git 2.56.0,
  local filesystem;
  a step cut part way through a large worktree could differ in ways a
  small one does not;
  `fsync` was off in the template (`core.fsync=none`),
  so a power cut is not modelled,
  only an ended process.
  Reflog and `ORIG_HEAD` differences are counted as differences even where the visible state is the same.
  The experiment ran `checkout` and `switch` although the wrapper never starts them for itself (they are the person's own commands,
  forwarded,
  never watched),
  so they matter only as evidence about the class.

`step_bound` in `src/native/transaction/transaction_git.rs` encodes the table as a whitelist of the exact forms (`may_be_ended`);
`a_step_keeps_its_ceiling_exactly_when_the_cut_experiment_ran_it_twice_to_the_same_state` pins it
(red at the commit before it,
against the wide list).
The experiment shows the first build's list was wider than the criterion:
`clean`,
`read-tree --reset`,
`update-index`,
`add`,
`checkout-index`,
`restore`,
`config` and `stash list` keep their ceilings,
and `stash list` (a read) had lost its by being counted as a `stash`.

### What a command does with a step that has no ceiling

- The commit service's landing thread waits for it as long as it lives (the con the owner accepted),
  and tells the clients.
- Any other command (the person's `git merge`,
  the stash carry,
  the cleanup) stops waiting at the bound of the step's class
  (120 seconds for a quick step,
  10 minutes for a bulk one,
  the same measured numbers the ceilings have) plus `SHORT_EXTRA_WAIT`,
  and returns the error of `child_watch::NotKnown`:
  the step and its command line,
  the seconds,
  that it was left running and not ended,
  and what to check
  (`git status`,
  `git log -1`,
  `ps`,
  the disk).
  The step is left running and a thread reaps it when it ends.
  This is `StepBound::LeaveRunning` in `src/native/push/child_watch.rs`.
  The coordinator wrote that a command returns "at its own bound";
  a person's `git merge` has no bound of its own,
  so the class ceiling of the step is the bound used here.
  The owner chose it:
  "Class ceiling + 7 s (Recommended)" (below).
- The auto-push work thread runs its reconcile merge through the push machinery's own runner (`auto_push_reconcile.rs`,
  `run_query`),
  judged by the push's stall window and not through `run_git`;
  it is not a person's command and is unchanged.

### The short extra wait

Both owner answers are quoted beside `SHORT_EXTRA_WAIT` in `wait_ceilings.rs`:
commit protocol revision 12,
"Short extra wait (Recommended)" ("the command waits a few more seconds,
a constant to be measured,
for the write it started;
if the disk is still stalled it exits non-zero saying the result is not known,
naming `git log -1` and the
side ref"),
and 2026-11-04,
"7 s (Recommended)":
twice the longest measured branch write (3,438 ms of 2,093 reference updates),
so the commit inside a stalled branch write exits not-known 7 seconds after its own bound.
The value is 7 seconds.

How the client learns that its request is inside the write,
and why an older client or service is unharmed:

- The service sends the `waiting` line it already sends,
  with one extra field,
  `"inside_write": true`
  (`ServiceMessage::InsideWrite` in the code).
  Decoders read fields by name,
  so a client that does not know the field reads the line as the plain `waiting` notice it is
  (it prints the text and keeps it as the blocker its error names) and goes on as before;
  a service that does not send it leaves a client that knows it on the older path.
  The protocol version stays 7,
  so no client or service is retired for this.
  Releases never meet on commit traffic anyway:
  the client compares the release in the service's greeting and retires a service of
  another release before it sends a request.
  Test:
  `an_inside_write_notice_is_a_waiting_line_with_one_extra_field`.
- Once a client has heard it,
  at its `ANSWER_CEILING` it does not ask to be withdrawn (the commit may be on the branch already);
  it listens for the landing for `SHORT_EXTRA_WAIT` more,
  and then ends with the not-known failure
  (`ClientFailure::WithdrawalUnconfirmed`,
  the blocker naming the write and the scratch ref,
  and `git log -1` and `git status`).
  The write is left running and the commit lands when it returns.
  Test:
  `the_commit_inside_a_branch_write_that_does_not_return_is_reported_as_not_known_and_lands_when_the_write_returns`
  (red before:
  it waited the 45 seconds of the control's withdrawal ceiling;
  green:
  it returns in about the control's ceiling plus the
  extra wait).
- The service tells the client whose request is the landing thread's when the write starts (the review of 2026-11-04 found that
  telling it only after `NEVER_ENDED_STEP_SLOW_AFTER` missed a write that began within a minute of the ceiling),
  and the connection
  the client opens to withdraw keeps the request's identity,
  so a client that withdrew before it was told hears it there.
  In both
  places the wait ends at the original answer deadline plus `SHORT_EXTRA_WAIT`,
  not at the moment a loop first notices it.
  Only a service that does not send the field leaves a client on the older path (withdraw,
  wait `WITHDRAW_CEILING`).
  Tests:
  `a_client_is_told_it_is_inside_the_write_when_the_write_starts` (the threshold is an hour in the control) and
  `a_notice_that_arrives_on_the_withdrawal_connection_ends_the_wait_at_the_answer_ceiling_plus_the_extra_wait` (red first).

#### When the person is told they are inside that write, 2026-11-11T03:40:00Z

A commit that exited 0 printed "this commit is inside a write of the commit service ... begun 0 seconds ago"
(`commit-exit-zero-prints-no-false-write-notice`, reported from
`event_path_commit::an_untracked_name_that_is_not_utf8_does_not_stop_a_commit`,
`commit-held-behind-a-dead-branch-lock-4` and `process-look-ceiling-under-load-1`).
Measured cause: the service sends the line as the write begins (the fix above),
and the client printed it at once,
while a branch write that crossed a heartbeat took milliseconds.
Of the 6,703 landings this host's service journal recorded from 2026-11-01 to 2026-11-11,
the branch write took 5 ms at the median,
601 ms at the 99th percentile and 4,785 ms at the 99.9th,
and 6 reached 5,000 ms.

What changed: the client holds those words and says them on standard error only once its own wait has passed
`WAITING_NOTICE_AFTER` (10 seconds,
the wait the place of a waiting commit waits for,
set by `CLI_GIT_STANDING_NOTICE_SECONDS`),
and says words that differ from the ones already said at once,
so a write that has stayed stuck a new minute is still told at once.
The words carry no moment of the write,
because the moment they are read is the client's.
The noticing at the write's start, and the not-withdrawing it decides, are unchanged.
Tests:
`a_commit_inside_a_write_says_nothing_of_it_before_its_wait_has_mattered` (at the board, on the client's own connection
and on a held write of a real landing; red first),
`a_commit_inside_a_write_says_it_once_its_wait_has_mattered`,
`the_new_words_of_a_write_that_stayed_stuck_are_said_at_once` and
`the_words_of_the_write_a_request_is_inside_carry_no_moment` (red first).

### The factor of five, and the cold stores

The owner,
2026-11-04,
"5x (Recommended)":
keep five times the longest measured run,
rounded up to whole minutes,
recomputed with the cold-store case if it is longer.
Cold-store timing (`cold-measure.ts`,
a copy of this repository's sources without `.git`,
a capped systemd unit with four CPUs of quota,
eight GiB of memory and an IO weight of 10;
the output is kept in `/var/home/user/temp/agent/ceiling-measure/cold-measure.kept.out`):

- `pnpm install --frozen-lockfile` with an empty store directory and the network:
  20.9 s.
  This is the measured cold-install duration on this host,
  one run,
  and not an upper bound of what a cold install can take.
  (The repair itself runs `pnpm install --offline --frozen-lockfile`,
  so a cold store makes it fail fast and not run long.)
- `cargo metadata --locked --offline --no-deps` in all 34 Cargo workspaces with no target directories:
  1.7 s in all,
  1.3 s the longest.
- `cargo fetch --locked` with an empty `CARGO_HOME` in `package/git-policy/cli`:
  7.6 s.
- `mpm-shadow install --frozen-lockfile` has a cold case,
  because it downloads the tools the worktree's lock names that are not
  installed yet (`SPEC.md`,
  the repair section).
  It was not measured cold:
  running it would install into,
  and could retire copies in,
  the shared tool directories of this host.
  Its warm figure is the 30.0 s longest of the add log,
  and the ceiling it is under is the
  one all repair tools share.
  A cold download of large tool archives is the case the ceiling is most likely to end.
- `cargo fetch` is not `cargo clean` or the clone of a build directory,
  so the Cargo cold case is partial (the registry and the
  workspaces,
  not the build state).
- All three measured runs are shorter than the warm-store maxima (53.1 s and 211.2 s),
  so the longest run is unchanged and the
  ceilings stay 18 minutes,
  4 minutes,
  1 minute and 2 minutes.
- Limits:
  the page cache cannot be dropped without privileges,
  so "cold" is a cold store and a cold registry,
  not a cold disk;
  the clone of real build directories,
  which dominates the Cargo stage,
  needs real target directories and is in the 69 add records
  (`cargoCloneTree`,
  up to 46 s per workspace);
  the host load average was 3.7 to 5.9 of 28.

### Item 3, as the coordinator and owner decided

A service stuck in a write never restarts itself.
The owner's answers behind it:
"Never cut a branch write (Recommended)" and "never discard user data" (a restart ends the write in the
middle,
which the first answer declines).

### A person's command inside a step that cannot run twice

The owner,
2026-11-04,
"Class ceiling + 7 s (Recommended)":
a person's command inside a step that cannot run twice stops waiting at
the step's class bound plus 7 seconds with a not-known error,
and the step is left running.
That is `StepBound::LeaveRunning { after: <class ceiling>, extra: SHORT_EXTRA_WAIT }`,
and it settles the open item of the earlier
section ("a person's `git merge` has no bound of its own").

### Review of 2026-11-04 (sol-1 and sol-2), and what was done about each finding

Both verdicts were "merge after the named fixes".
Files:
`/var/home/user/temp/agent/reviews/branch-write-sol-1.txt` and
`branch-write-sol-2.txt`.

- Withdrawal race (sol-1 P1):
  a landing that completes between the withdrawal's checks was answered `withdrawn`.
  Fixed by making admission and cancellation one decision (the `.decision` file,
  above).
  Tests:
  `admission_and_early_withdrawal_never_both_succeed` (two threads behind a barrier,
  200 rounds),
  `a_request_whose_commit_is_on_the_branch_in_an_open_batch_is_not_withdrawn_early`,
  and the journal-after-the-win check.
- Fail closed (sol-1 P1):
  `a_record_that_cannot_be_written_never_answers_withdrawn` (red first:
  the directory made read-only),
  the journal read is strict (`Naming::Unknown` for anything that cannot be read whole).
- Tombstone (sol-1 P1):
  `every_copy_of_a_withdrawn_request_is_dropped_not_only_the_first` (red first).
- Left running must survive the exit (sol-1 P1):
  a step that is never ended writes into anonymous files (`memfd`,
  `process/output_capture.rs`) and not into pipes the command holds;
  the step is a child of the command,
  and when the command
  exits it is reparented to the system's init or subreaper,
  which reaps it;
  while the command lives a thread reaps it.
  Test:
  `a_step_left_running_survives_the_exit_of_the_command_that_left_it` runs the command as a separate process that exits
  (red first:
  the step died of SIGPIPE at its next write).
  Only Linux has `memfd_create`;
  elsewhere the files cannot be made and the step does not start (macOS is out of scope for now).
- Fail-open classifier (sol-1):
  `may_be_ended` is a whitelist of the exact forms the experiment ran,
  parsed by their options and
  words;
  it rejects `config --add`,
  `stash -m list`,
  `update-ref --create-reflog` and `-m`,
  a ref outside the two numbered side-ref
  shapes,
  `add --intent-to-add`,
  and every step nobody listed.
  Test:
  `a_form_the_cut_experiment_did_not_run_has_no_ceiling` (red first).
  Round 2 left the assumptions that the side refs are plain refs and have no reflog written in a comment;
  round 3 enforces them (see
  "Round 3").
- Seven-second path missed (sol-1):
  see "The short extra wait".
- Tests (sol-1):
  barrier-driven race,
  storage-failure injection,
  replay after consumption,
  late notice,
  and a deadline around the
  joins of the hung-write controls (`Submitting::join` waits at most the longest a control waits).
  The delayed-write control now runs on the landing thread and outlasts the bound plus the extra wait (6 s against 3 s and 1 s).
  Power loss is not modelled (the experiment ran with `core.fsync=none`),
  and hooks and concurrent writers were off or absent;
  the
  experiment's limits are stated in "The cut experiment".
- Output draining (sol-2 A):
  `run_tool` writes the tool's output to anonymous files,
  so the ceiling covers the draining of the
  output too and no reader thread waits for a stream to close.
  A helper that keeps an inherited stream open after the leader has exited,
  or that left the process group while holding it,
  holds
  nothing up;
  at the window or the ceiling the whole process group is still stopped,
  while the leader is not yet reaped,
  so the group
  number is still the leader's.
  A helper still running when the leader has exited is left alone (a tool may start one on purpose).
  Tests:
  `a_helper_that_keeps_the_output_open_after_the_leader_exits_does_not_hold_the_run` and
  `a_helper_outside_the_process_group_that_holds_the_output_does_not_hold_the_ceiling` (red first).
- Review log ownership (sol-2 B):
  `run_contained` counts a review it stopped waiting for as outstanding until its thread ends and
  starts no other review meanwhile (`Contained::Busy`);
  the refused item fails with the command that reviews it again.
  Tests:
  `no_second_job_starts_while_a_job_the_runner_stopped_waiting_for_is_still_running`,
  `a_job_that_ends_as_the_runner_gives_up_leaves_nothing_outstanding`,
  and a service-level control with a stalled log (a named
  pipe nobody reads) in `service_review_tests.rs`,
  which refuses the second item without a thread and reviews the first again after
  the stall ends.
  A mutation that removes the refusal fails both.
- Repair test helper (sol-2):
  the helper process number is recorded with `$!`,
  and a helper with redirected output is added;
  zombies
  are not counted as running.
- Documentation (sol-2):
  the `/proc` paragraph of `SPEC.md` no longer says an unreadable counter means a run is never ended;
  the two
  descriptions of the inside-write client are one;
  the cold-install figure is called a measured duration.

### Foreign-owner patience: what its expiry authorizes (sol-2 C), decided: refuse and name it

The owner chose option B on 2026-11-04.
Verbatim choice:
"Refuse and name it (Recommended)" (the question:
"Foreign-owner patience:
what its expiry authorizes").
Built in round 3:
when `FOREIGN_OWNER_PATIENCE` expires on a record from another PID namespace,
the
command stops and changes nothing.
The error names the record file,
the owner's process number and namespace,
the record's age,
that
the owner may be live in another container,
and the exact `rm -- '<path>'` for when no cli-git command is running on the repository
in any container and no Git process that cli-git left running is either (the second condition because a step that cannot run twice
is now left running,
not ended).
It covers the merge intent recovery,
the conversion claim takeover,
and the worktree-copy journals
and stage claims;
commits stay outside it.
Each consumer has a red-first test,
listed in `SPEC.md`,
"Foreign-owner expiry".
The
analysis that follows is kept as the record of what the expiry authorized before the choice (the text says "today" for that time).

`FOREIGN_OWNER_PATIENCE` (20 minutes,
twice `LOCK_CEILING`) is the age after which a record written by an owner in another PID
namespace stops counting as alive.
The reviewer's point holds:
the number was derived from the longest lock wait,
and
a step that cannot be run twice may now legitimately run past any number,
so an old record is not proof of a dead owner.
What the expiry authorized before round 3,
from the code:

- Merge intent records (`merge_isolation_recovery.rs`,
  through `classify_transaction_owner_record`):
  a record judged dead is
  recovered by the next wrapper command in that worktree:
  `recover_interrupted_merges` decides whether the merge landed and
  completes it or discards it (it moves the branch,
  resets the index and the worktree it finds).
  A live foreign merge past 20
  minutes would be completed or discarded under its own feet.
- Claim files (`single_owner_claim.rs`,
  `live_owner`):
  a claim judged not alive is replaced by a newer generation,
  and the taker
  runs the conversion or migration it guards.
  A live foreign owner past 20 minutes would have a second command run beside it on the
  same item.
- Worktree-add journals and stage claims (`worktree_copy_owner.rs`,
  `token_owner_is_dead`):
  a journal judged dead is taken over by
  renaming it and the copy is finished or undone,
  the temporary files of a dead writer are removed,
  and the claim of a dead owner
  lets the stage be reported (a stage of an add is reported and never removed,
  a stage of a copy may be).
  A live foreign copy past
  20 minutes would be installed twice.
- The transaction owner records of the earlier release (`transaction_owner.rs`) use the same rule;
  this release no longer reads
  them for anything else than the merge intent above.
- Not authorized by it:
  a lock (`index.lock`) is judged by its marker's namespace and boot and by the lock holder
  (`service_index_lock.rs`),
  not by this age;
  commits never wait for it.

Options (B was built),
each with its schedule and what a person sees:

- A.
  Keep the derived value and its behavior.
  Schedule:
  a container's merge holds its record 21 minutes;
  another namespace's wrapper command in the same worktree finds it
  expired,
  recovers it,
  and the branch or worktree is changed under the live merge.
  The person sees a notice that an interrupted merge was settled,
  or a corrupted checkout.
  Needs nothing;
  leaves the hazard.
- B.
  Expiry refuses and names the record.
  Schedule:
  the same record is 21 minutes old;
  the command that finds it stops with an error that names the record file,
  its owner's
  process number and namespace,
  its age,
  that the owner may be a live command in another container,
  and the exact `rm` to run if no
  cli-git command is running on the repository in any container;
  it changes nothing.
  The person sees one error per such command until the record is removed or its owner ends (a live owner removes its own record at
  the end).
  Never reclaims a live owner's record;
  the cost is a manual step after a crashed container,
  and the commands that need
  the item fail until then (a merge,
  a conversion,
  a copy),
  which is a failure the owner has said commits should not suffer,
  so the
  commit path is not among them.
- C.
  Expiry reclaims only with independent liveness.
  Schedule:
  the owner renews its record while it works (a touch every half of a short interval,
  on the same file),
  so the age is
  the time since the last renewal and a live owner is never old;
  a record that has not been renewed for the interval times a margin
  is reclaimed.
  The patience then measures silence,
  not length,
  and its value is a multiple of the renewal interval,
  not of a lock
  wait.
  The person sees nothing in the normal case and the old reclaim after a crashed owner is found silent.
  Costs:
  every writer of the three record kinds needs a renewal (a thread or a call in its wait loops),
  the renewal of a process that
  is stopped but not dead keeps it alive,
  and a filesystem that does not show the touch to the other namespace defeats it.
  Fencing (the owner re-reads its record before each destructive step and stops if it was replaced) would be needed beside it for the
  steps the wrapper does not control,
  a Git command,
  so it does not close the merge case alone.
- D.
  Put the value back to an hour.
  Schedule:
  the same record is 61 minutes old instead of 21.
  Shrinks the window;
  the hazard is unchanged for a write that runs
  longer.
  The person sees what A shows,
  later.

Ranking:
B > C > D > A.
B over C because B never reclaims a live owner's record and needs a change only where the expiry is judged,
while C changes every
writer and still cannot fence a Git command;
C over D because C removes the dependence on a length that no measurement bounds,
where D only moves the number;
D over A because an hour is less likely to be crossed by a write than 20 minutes,
and costs a revert of the derivation the owner asked for.
The owner chose B.

## Round 3, 2026-11-04T14:20:00-05:00 onward: the findings of the second reviews and what was done

Files:
`/var/home/user/temp/agent/reviews/branch-write-r2-sol-1.txt` and `branch-write-r2-sol-2.txt`.
Both verdicts:
correct after
named fixes.
Each finding,
with its disposition and its test (red first unless said):

- Provisional `withdrawn` acknowledged as final (sol-1 P1).
  A decision is provisional until its maker has re-read the journal and
  created `<id>.cancelled`;
  a loser of the creation answers `Withdrawn` only when that file exists,
  else queues its withdrawal;
  a maker that finds the request landed,
  open or unknown removes its own unacknowledged decision.
  Pruning removes the oldest
  `.cancelled` file with its decision.
  Tests:
  `a_decision_another_withdrawer_has_not_confirmed_is_not_acknowledged`,
  `a_journal_that_fails_after_the_win_leaves_no_acknowledged_cancellation`,
  `only_the_retained_number_of_confirmed_withdrawals_is_kept`.
  The reviewer's alternative,
  a short-held coordinated state lock,
  was not taken:
  the owner's "Use as few locks as possible."
- Classifier (sol-1 P1).
  A side-ref write is ended only with `--no-deref` and `-c core.logAllRefUpdates=false` on the argv,
  which the
  wrapper passes at every call site (`keep_unlanded` and the four scratch deletions in `service_land.rs`);
  `fast-import` is never
  ended (its stream can update and checkpoint branches;
  the experiment found a marks lock left by a cut that failed the second run);
  `RERUNNABLE` no longer names `symbolic-ref` or `init`,
  and `may_be_rerun` accepts a `symbolic-ref` read,
  a `symbolic-ref` write on
  a shadow request with no option,
  and `init --bare` on a shadow request.
  Real argv forms found by `rg`:
  `symbolic-ref --quiet HEAD`
  and `--quiet --short HEAD` (reads),
  `symbolic-ref HEAD <ref>` and `symbolic-ref <name> <target>` in `shadow_refs.rs` (shadow
  requests),
  `init --bare --template= --object-format=<f> --ref-format=reftable <shadow>` in `shadow_create.rs`.
  `init` came out
  different in the experiment,
  and it is kept as a rerun form because it creates a private repository that its transaction discards
  when it fails.
  Tests:
  `a_form_the_cut_experiment_did_not_run_has_no_ceiling` (extended),
  `a_step_keeps_its_ceiling_exactly_when_the_cut_experiment_ran_it_twice_to_the_same_state`,
  `symbolic_ref_and_init_are_run_again_only_in_the_forms_the_wrapper_runs`.
  The reflog setting must be the last `-c` of its key (the key is not case-sensitive and `--config-env` of it is refused),
  and the
  `init` form is the exact argv of `shadow_create.rs`;
  those two controls were written after their code and not run red first.
  `core.logAllRefUpdates=false` stops a reflog from being created and does not stop an append to one that exists.
  The first answer to
  that was that the proof rests on the sequence number being new at each use;
  the round 3 review rejected it (an earlier release or
  another writer may have made the log),
  and round 4 replaced it with the `git reflog exists` check.
- Unbounded capture read (sol-1 P1,
  sol-2 A).
  The output is read once as a snapshot of the last `STEP_OUTPUT_LIMIT` bytes of the
  sizes found when the read began,
  with a note for what was left out and for a read that failed;
  the files are emptied after the
  read and when the command stops waiting for a step.
  Tests in `output_capture_tests.rs`
  (`output_past_the_limit_keeps_the_tail_and_says_what_was_left_out`,
  `the_snapshot_reads_the_length_it_found_and_no_more`,
  `a_read_that_fails_is_reported_and_not_taken_for_the_end`) and
  `a_helper_that_keeps_writing_after_the_leader_exits_does_not_hold_the_run`.
  Residual,
  stated in `SPEC.md`:
  a step left running that
  keeps writing is held in memory by the kernel until it ends;
  nothing in the wrapper bounds that.
  This round wrote the snapshot and
  its tests together and did not run them red first.
- `decide` leaked `.tmp` on a failed write (sol-1 P2).
  An RAII `TemporaryFile` removes it on every return.
  Test:
  `a_temporary_file_is_removed_by_an_early_return`.
- A review that panics after its timeout left `OUTSTANDING` positive (sol-2 P1),
  and admission was not atomic (sol-2 P2).
  The runner
  state is a generation and a status in one atomic;
  the release is a guard that runs when the thread unwinds.
  Tests:
  `a_review_that_panics_after_its_timeout_leaves_nothing_outstanding` (timeout,
  panic,
  zero,
  new review) and
  `two_callers_that_arrive_together_are_never_admitted_at_once`.
  The tests that shared the global use runners of their own.
- `run_tool` timeout path (sol-2 A):
  `reap_within` waits for the leader within `STOP_GRACE` and hands a leader that does not end to a
  reaper thread.
  Tests:
  `a_leader_that_does_not_end_is_handed_to_a_reaper_after_the_budget`,
  `a_leader_that_ends_is_reaped_within_the_budget`.
  `has_shadow_log` now runs inside the contained job;
  the FIFO control retries on the path that stalled and has its own runner.
- Tests named by both reviews:
  two withdrawers plus completion and the journal failing after the win (above);
  the CLI-exit test has
  a deadline around its command;
  the inherited-output tests assert the helper is alive before cleanup;
  a continuously writing
  descendant after the leader exits (above).
- Protocol and upgrade (sol-1 section 3).
  No wire change.
  `SPEC.md` ("Output of a step,
  the review runner,
  and releases that meet")
  says what services and clients of 0.5.31 and 0.5.32 keep doing.
  Retirement:
  the polite `retire` of a same-instance upgrade never
  stops a unit;
  the retirement of the single service of the releases before one service per worktree used to ask and stop at once,
  and the manager's stop ends the whole unit with its Git children;
  it now stops the socket,
  waits `retirement_wait` for the
  service to leave by itself,
  and stops the service only after it has left,
  failing the commit with the stop named for the person
  otherwise.
  Tests:
  `an_earlier_service_that_is_still_working_is_asked_to_retire_and_never_stopped_by_the_wrapper`,
  `the_earlier_service_is_stopped_only_after_it_has_left`.
- Foreign-owner expiry (sol-1,
  sol-2 C):
  decided and built,
  above.
- Not changed:
  guard-fixture cleanup in older tests,
  `wait_ceilings.rs` verification (sol-2 round 1 notes),
  and the unrun reviewer
  items about reason-string tests versus end-to-end propagation.

### The output snapshot limit, measured 2026-11-04

`STEP_OUTPUT_LIMIT` (8 MiB per stream) is the most of a step's output the wrapper keeps when it captures it in full:
the steps that are
never ended and the repair tools.
Measured on this host (`measure-output.ts`,
outputs in `measure-output.out`,
`pnpm-out.txt` and
`mpm-out.txt` beside it):
`cargo metadata --locked --offline --no-deps --format-version 1` for every one of the 35 manifests of the
repository,
at most 21,012 bytes (`package/ide/app`);
`pnpm install --offline --frozen-lockfile` in a worktree,
634 bytes;
`mpm install --frozen-lockfile`,
639 bytes;
the diffstat of the three merges with the most changed files in the Monochromatic history
(4096,
1850 and 1393 files),
638,784,
114,892 and 201,802 bytes,
and of the three largest in `/var/home/user/resharp`,
at most 2,381
bytes.
Git's own writers print one line (`HEAD is now at ...`) or nothing when their output is not a terminal.
So the largest was 638,784
bytes,
and 8 MiB is 13.1 times that.
The value stays at 8 MiB with this citation;
what a step writes beyond it is disclosed,
and output
that was cut is never parsed as a whole result.

### Retirement of the single service of the earlier releases: options for an owner question, 2026-11-04

The coordinator asked for these options on 2026-11-04 (round 4).
Decided by the owner on 2026-11-04 through the question tool,
verbatim
choice:
"A:
fail,
retry (Recommended)".
It is built (the stop of the service goes only to a service that has left,
and a service still
there after `RETIREMENT_WAIT` fails the commit with a retryable sentence);
the options in the `Options` list are kept as the record of what was weighed.

What exists today:

- The releases before one service per worktree ran one `cli-git.service` with a `cli-git.socket` for every worktree,
  listening on
  `cli-git/service.sock` under the runtime directory.
  The first release with a service for each worktree is 0.5.7 (the commit
  `4b52f3745c`,
  2026-11-01T16:51:44-05:00,
  is first contained in the tag `monochromatic-git-policy-cli-v0.5.7`),
  so the earlier
  installs are 0.5.0 to 0.5.6.
- On this host none can exist:
  no `cli-git.service` or `cli-git.socket` in `~/.config/systemd/user`,
  in the runtime directory's
  `systemd/user`,
  or anywhere under the home directory to four levels,
  no `cli-git/service.sock` in the runtime directory,
  and
  `systemctl --user list-units --all 'cli-git.service' 'cli-git.socket'` lists nothing.
  The units that exist are `cli-git@<instance>`
  of the current design (their sockets are `<instance>.sock`).
  Other accounts,
  containers and machines were not looked at;
  a machine
  where an install of 0.5.0 to 0.5.6 still runs would meet this the first time the current release provisions its first worktree.
- The retirement is `retire_earlier_service` (`service_provision.rs`).
  Before round 4 it asked the service to retire (a message the
  service handles between batches) and then had the manager stop both units at once;
  the manager's stop sends `SIGTERM` to the unit's
  whole control group,
  the Git children of a landing in progress included,
  and `SIGKILL` after its own timeout.
  Built in round 4:
  the
  socket is stopped,
  the service is given `retirement_wait` (10 seconds,
  `RETIREMENT_WAIT`) to leave by itself,
  the stop of the
  service goes only to a service that has left,
  and a service still there fails the commit.
- Why a service can still be there after the wait:
  it is landing a batch (a Git step that can run for the class ceiling and more),
  or
  it is stuck in one that is never ended,
  or it is an old release that does not read the retire message.

Options:

- A.
  Fail the commit and let it be retried (built).
  Schedule:
  the commit asks the old service to retire and stops its socket;
  10 seconds later the service is still in a landing;
  the
  commit exits non-zero with a sentence that names the service,
  says it may be in the middle of a landing that stopping it would cut,
  and gives `systemctl --user stop cli-git.service cli-git.socket` for when no commit is running.
  The next commit,
  run again,
  finds
  the service gone if its landing finished (it handled the retire between batches) and goes on.
  The person sees one failed commit with a retryable sentence,
  once per upgrade per machine,
  and nothing else changes.
  What can be
  lost:
  nothing of the repository;
  a commit that was built and not made can be made again by running the command again.
  Costs:
  it is the one place where a commit fails for want of a bounded wait,
  against the owner's "Commits should never fail."
- B.
  Land through the worktree's own service while the old one still runs.
  Not safe,
  as the code and the invariant stand.
  The documented invariant (`SPEC.md`,
  the lock section) is that two commit services
  never overlap on one repository:
  an instance that starts recovers the batches open in the journals it finds and claims work items,
  and the old service is still doing that work.
  Two landing threads would race for the index lock marker and the branch write,
  and
  the new instance's recovery would settle an intent that the old service is writing.
  Schedule:
  the old service has an open intent
  for batch 41 and is inside its `update-ref --stdin`;
  the new instance starts,
  reads the open intent,
  finds the branch not yet
  moved,
  drops the intent;
  the old service's write completes and moves the branch,
  and the journal says the batch was dropped.
  What
  can be lost:
  a landed commit recorded as dropped,
  a client told "not landed" for a commit that is on the branch.
  Making it safe
  needs a handshake between the two releases that the earlier ones,
  which cannot be changed,
  would have to take part in.
- C.
  Wait longer,
  with a stated bound.
  Schedule:
  as A,
  but the commit waits for the service to leave for a longer bound (for example `LOCK_CEILING`,
  10 minutes,
  which is
  the longest the wrapper waits for a lock holder,
  or `ANSWER_CEILING`,
  20 minutes),
  polling the manager and saying once a minute
  what it waits for;
  if the service leaves,
  the commit goes on and succeeds;
  if not,
  it fails as in A.
  The person sees a commit that takes as long as the old service's landing,
  with a notice,
  and a failure only after the longer
  bound.
  What can be lost:
  nothing of the repository.
  Costs:
  a commit that waits minutes on a machine that has the old service stuck
  in a write;
  the bound is a new number that needs a measurement (the longest landing is 751 seconds,
  measured) or an owner
  decision.
  Note 2026-11-08T19:19:04-05:00:
  the live lock waits are no longer `LOCK_CEILING`:
  a commit's wait for a foreign `index.lock`,
  a merge's wait and the reconciliation's are one minute each (`LANDING_LOCK_CEILING`,
  `MERGE_LOCK_CEILING`,
  `RECONCILE_LOCK_CEILING`),
  so ten minutes is not the longest the wrapper waits for a lock holder;
  `SERVICE_HOLD_CEILING` is ten minutes,
  and all four are in `package/git-policy/cli/src/native/service/wait_ceilings.rs`.
- D.
  Stop the old service anyway after the wait.
  Schedule:
  as A,
  but after 10 seconds the manager is asked to stop the units (the behavior before round 4).
  The control group
  receives `SIGTERM`;
  a Git child cut in the middle of `update-ref --stdin` or `merge` is exactly what "Never cut a branch write
  (Recommended)" decided against,
  and the service's own recovery (the intent journal) has to complete or undo the batch afterwards
  in the worktree's instance.
  The person sees a commit that succeeds.
  What can be lost:
  a branch write cut at an unlucky moment (the cut experiment shows
  `update-ref --stdin` and `merge` come out different at some cuts),
  the work of the clients queued behind it,
  and an index lock left
  by the killed process (reclaimed by the stale-lock sweep).

Ranking:
A > C > D > B.
A over C because A never makes a commit wait for a landing it cannot see the end of,
fails once,
retryably,
per upgrade per machine
(none exists on this host),
and needs no new number,
where C adds a bound that nothing measures and that a stuck service makes the
person pay in full before the same failure.
C over D because C keeps every landing whole and costs only time,
where D cuts a write that cannot run twice,
which the owner's
decision rules out for branch writes.
D over B because D loses at worst one cut write that the journal recovery is built to settle,
where B can report a landed commit as
dropped with no recovery at all,
and only a change to releases that are already published could make it safe.
The owner chose A:
it puts "never discard user data" before "Commits should never fail."
for this one case.

## Round 4, 2026-11-04: the findings of the cross-vendor review of round 3

Review:
`/var/home/user/temp/agent/reviews/branch-write-r3-sol.txt` ("correct after named fixes").
Each finding,
its fix and its test:

- A,
  P1.
  Withdrawal confirmations were not tied to a decision generation.
  The decision now reads `withdrawn:<token>` and its
  confirmation is `<id>.<token>.cancelled`;
  a withdrawer acknowledges another's decision only when the confirmation with that
  decision's token exists.
  The retention moved to the landing thread (`end_group`),
  so there is one pruner,
  and it removes a decision
  only while it still carries the token of the confirmation it removes,
  so a replacement from a stale listing stays.
  The comment about
  the order of the two removals was wrong (the decision first leaves the confirmation standing alone)
  and is replaced by the rule stated for `end_group`.
  Tests:
  `a_confirmation_that_outlived_its_decision_confirms_nothing_of_the_next_decision` (the reviewer's schedule:
  failed when the
  confirmation was shared by the id again) and `the_retention_never_removes_a_decision_that_replaced_the_one_it_listed`.
- B,
  P1.
  An existing reflog defeats the side-ref proof.
  `step_bound_checked` asks `git reflog exists <ref>` at the start of the write,
  on either ref backend,
  and the write keeps its ceiling only when Git answers that there is none;
  Git that cannot be asked is no
  proof.
  Test:
  `a_side_ref_that_has_a_reflog_keeps_no_ceiling` (files and reftable repositories,
  both side-ref shapes,
  set and
  delete,
  and a missing repository).
- C,
  P1.
  A foreign-owner refusal could follow mutations.
  Every owner is now judged once before anything changes:
  the journals and the
  stage claims and temporaries of a worktree copy (`judge_stage_owners`,
  `reap_judged_stages`),
  and the records of the merge recovery.
  `token_owner_is_dead` reads the item's age once and propagates the expiry;
  it no longer calls `is_alive`,
  which read it again and
  whose refusal was swallowed.
  Tests:
  `an_expired_foreign_journal_after_a_dead_one_stops_recovery_before_the_dead_one_is_touched`,
  `an_expired_foreign_claim_stops_the_sweep_before_any_stage_is_removed`,
  `an_expired_foreign_temporary_stops_the_sweep_before_any_temporary_is_removed`,
  `an_old_record_of_another_namespace_after_a_dead_one_stops_recovery_before_the_dead_one_is_settled`,
  and
  `an_owner_of_another_namespace_is_judged_from_one_reading_of_its_item` (counts the lookups).
  An item that is young when judged and
  expires during the run is acted on as judged;
  it is refused by the next command.
- D,
  P1.
  The review timeout could wait for ever after the worker released the runner and stalled before its send.
  The caller now waits
  for the value at most `SHORT_EXTRA_WAIT` and reports the ceiling otherwise.
  Test:
  `a_worker_that_stalls_between_its_release_and_its_send_does_not_hold_the_caller` (with a deadline of its own;
  the unbounded receive
  fails it after thirty seconds),
  and the controlled `a_runner_with_a_running_job_admits_no_other`.
- E,
  P2.
  The notes of the snapshot reach the person (the failure text of a repair tool) and the result (`ToolRun::notes`);
  the
  `cargo metadata` of a repair refuses cut output;
  `output_of` and the stop path of `run_tool` release the pages.
  Tests:
  `output_past_the_snapshot_is_disclosed_to_the_person`,
  `the_output_of_a_step_is_released_once_it_is_read`,
  and
  `a_stopped_tool_is_reaped_through_the_budgeted_reaper_after_its_output_is_released` (a reaper is given to `run_tool_reaping`).
- `init`.
  The first round 3 report said `init` came out different in 64 of 64 cuts with 57 second runs failing.
  That run gave the step
  an absolute path in the template directory,
  so it measured a shared directory;
  it is withdrawn (see "The cut experiment").
  With the
  path relative to each copy:
  in place,
  50 of 104 cuts differ and every second run exits zero;
  with the partial directory discarded
  and a fresh one made,
  104 of 104 are identical.
  `init_with_git` discards and starts again,
  at most `SHADOW_CREATION_ATTEMPTS` (2)
  times,
  also when the system refused the start,
  and `init` is no longer in the allow-list of steps run again in place.
  Tests:
  `a_shadow_whose_init_was_cut_is_made_again_in_a_fresh_directory`,
  `a_shadow_whose_init_is_cut_every_time_fails_after_the_attempts_and_leaves_nothing`,
  `only_the_shadow_creation_init_keeps_a_ceiling`.
- Tests the reviewer called too weak:
  the snapshot control appends after every chunk the reader takes
  (`the_snapshot_reads_the_length_it_found_and_no_more`);
  the failed-write control fails inside `decide`
  (`a_decision_whose_write_fails_leaves_no_temporary_file`);
  the 10 ms helper control is kept for what it proves (the run does not
  wait for a helper) and the snapshot boundary is proven by the control above,
  since a loop that writes fast enough to outrun a reader
  is a busy loop,
  which the synthetic-load rule forbids.
  Not written:
  a binary test that a busy legacy service and its Git children are
  untouched when the legacy socket is stopped.
  It needs a legacy-socket client and a service held in a landing,
  which the harness has
  no helper for;
  the recorder controls (`an_earlier_service_that_is_still_working_is_asked_to_retire_and_never_stopped_by_the_wrapper`)
  and the fake-bus call order in `binary_service_provision_tests.rs` cover the wrapper's calls.
- Legacy retirement:
  the reviewer calls failing (retryable) the correct safety direction;
  the options for the owner are in
  "Retirement of the single service of the earlier releases".

## Round 5, 2026-11-04: the findings of the cross-vendor review of round 4, and the shipped thread-start panic

Review:
`/var/home/user/temp/agent/reviews/branch-write-r4-sol.txt` ("correct after named fixes";
B "still lacks a valid safety proof").

- B,
  P1.
  The probe of `git reflog exists` was not atomic with the write:
  another Git can create the log between them,
  and a cut write
  then appends twice on its rerun.
  No write of a side ref keeps a ceiling any more (the absence probe,
  the argv classification and the
  `reflog` read form are removed);
  on the landing thread they are waited for like the branch write.
  The call sites keep `--no-deref`
  and the reflog setting as hygiene.
  Red shown:
  `a_step_keeps_its_ceiling_exactly_when_the_cut_experiment_ran_it_twice_to_the_same_state`
  failed with the side-ref forms in its "differs" list while the classifier still ended them ("came out different after a second run
  and has no ceiling").
- `init`,
  direct refusal.
  A start refused before the command ran reaches `init_with_git` as an I/O error of kind `WouldBlock`
  (`EAGAIN`);
  it is now discarded and started again in a fresh directory like a cut or a refused-as-finished start,
  and other I/O
  errors are not retried.
  Test `only_a_cut_or_refused_start_of_init_is_retried` (red with the `WouldBlock` arm removed).
- E,
  the wait error.
  A failed wait for a repair tool returned before the group was stopped,
  the pages released and the leader reaped.
  It is now a stop.
  Test `a_wait_that_fails_stops_the_tool_and_releases_its_output` (red first:
  the reaper was never called).
  The stop
  control now first establishes that the tool had written.
- A,
  regression.
  The record of confirmed withdrawals was no longer bounded while the landing thread was stuck (only it prunes).
  Past `WITHDRAWALS_CEILING` (twice the retained number) the connection threads leave withdrawals to the landing thread.
  Test
  `the_memory_of_withdrawals_stays_bounded_while_the_landing_is_stuck` (red first:
  2058 withdrawn against 2048).
- C and D,
  test strength.
  A cross-category test (dead journal,
  expired foreign stage claim),
  temporaries judged in name order,
  and a
  handoff control that forces the release before the caller gives up.
- The thread-start panic (shipped since 0.5.28,
  found by the gate of 2026-11-04,
  test
  `a_memory_limit_close_to_exhaustion_on_the_service_leaves_a_recoverable_repository`,
  4873 of 4874).
  `landing/service_durable.rs`
  made the syncs of a landing with `Scope::spawn`,
  which panics when the system gives no thread;
  on the service's main thread
  (`force`) that took the service down,
  so the commit could not land.
  The measured cause:
  the test lowers the address space of the
  service until a start fails,
  and the two places that start threads for the syncs were the first to be refused;
  the panic is in the
  program,
  not in the load.
  `foundation/scoped_start.rs` starts a scoped thread and runs the work on the calling thread when the start
  is refused (the closure is handed through a slot,
  because `spawn_scoped` consumes it even when it fails).
  Every production site:
  - `landing/service_durable.rs` `sync_batch` and `force`:
    fixed;
    test `a_landing_whose_syncs_cannot_start_a_thread_makes_them_where_it_is_and_does_not_panic`
    (a child with a process limit of one;
    red against the old code with the exact panic).
  - `foundation/parallel_runner.rs`,
    `repair/package_repair.rs`,
    `repair/package_repair_cargo.rs`,
    `repair/package_repair_tree_clone.rs`,
    `worktree/worktree_clean.rs`,
    `worktree/worktree_snapshot_source.rs`,
    `worktree_copying/worktree_copy_compare.rs`,
    `worktree_copy_install.rs`,
    `worktree_copy_journal.rs`,
    `worktree_copy_stage.rs`:
    fixed the same way.
    Controls:
    a child refused
    every thread for the parallel runner,
    the cargo `map_concurrent`,
    the tree clone,
    the install,
    the final check,
    the removal and the
    stage copy;
    the seam of `scoped_start` for the clean check and the repair,
    whose scenarios start Git.
    The snapshot source has no
    scenario of its own that runs without a btrfs fixture,
    so its two starts are covered by the mechanism's tests only.
  - `push/auto_push_reconcile.rs`:
    the input thread of a second-pass command was a plain `std::thread::spawn`;
    a refusal now stops the
    command and reports,
    since writing the input here could block on a full pipe.
    Test
    `a_second_pass_command_whose_input_thread_cannot_start_is_stopped_and_reported` (the seam;
    no plant is possible against a plain spawn).
  - Already handled,
    unchanged:
    `landing/service_index_prestage.rs` (no marker made ahead),
    `landing/service_claim_renewal.rs` (no
    renewal,
    claims expire),
    `push/bounded_push.rs`,
    `process/thread_quiet.rs`,
    `push/child_watch.rs` watch thread,
    `markdown/markdown_linter_guard.rs`
    (`GuardFailure::Start`),
    `repair/package_repair_command.rs` reaper (the leader is left to init),
    and `push/child_watch.rs` left-running
    waiter,
    which reports that no thread could be started to wait for the step.
    `testing/test_support.rs` watchdog:
    failure ignored.
  Planted proof:
    with the fallback replaced by the panic of `Scope::spawn`,
    8 of 8 child controls and the durable control failed,
    and the
  two seam controls failed.

- Correction after the first round 5 report:
  removing the ceiling of the side-ref writes would have made `keep_unlanded` and the scratch
  deletions wait without a bound on the landing thread,
  which the owner's "nothing should ever wait forever or indefinitely" does not
  allow and "Never cut a branch write (Recommended)" does not cover (a side ref is not the branch).
  They are now left running after the
  class ceiling plus the short extra wait on every thread,
  the landing thread included.
  Test
  `a_side_ref_write_that_hangs_on_the_landing_thread_is_left_running_after_a_bound` (red first:
  `Unbounded`).
  What still waits without a
  bound on the landing thread is the branch write (`update-ref --stdin`),
  `fast-import` and the other steps that cannot run twice,
  which
  the clients behind them are told about after `NEVER_ENDED_STEP_SLOW_AFTER` and which end for each client at its own `ANSWER_CEILING`.

## The watch and the wait for the output on one deadline, 2026-11-09T22:53:51-05:00

The release gate on `cfce0bc84e`
(`~/temp/agent/gate/gpc-gate-wrapper-next-alpha-on-cfce0bc84e-20261109T215524-0500.log`)
failed `push::child_watch::tests::a_step_that_is_never_ended_for_being_quiet_is_ended_at_its_ceiling`
once in 6,909,
asserting `is_ceiling(&error)` over this sentence:

```text
a quiet step ran for 0 seconds, the longest the wrapper lets one step run, and was ended. Its watch could not end
it: nothing proved which process to stop. Hint: check with `ps` what the step waits for, and run the command again.
```

Nothing in that control refused the watch anything,
so the reason it gave was false,
and the error was not a `CeilingReached`.

### The cause, measured

`push/child_watch.rs` gave the watch the step's ceiling (`ChildWatch::within`)
and gave the wait for the step's output the same ceiling as its backstop
(`carried_for`),
measured from the same moment in all but the microseconds
between the begin mark and the read.
Both run on the process's runtime,
the watch as a task,
so whichever was scheduled first won,
and a watch that was a little late was passed by the backstop that exists for it.

The watch writes a due notice before it judges the ceiling,
and that write runs on the blocking pool,
so the notice is one measured place the watch is held back:
`a_step_whose_watch_is_held_back_is_still_ended_at_its_ceiling`
holds it there,
deterministically and with no load on the host,
and the wait then reaches the deadline first.
That control failed on the code before the fix in 0.79 s with the sentence above,
and passes in 19.9 s after it.

What the wrong answer cost,
beyond the control:

- A step that may be run again (`transaction_git.rs`,
  `transaction_git_awaiting.rs`) was run again after a ceiling it had already
  met,
  which doubles the wait for a step that is too slow and tells the person
  it "showed no sign of work",
  which it had not.
- The landing (`landing/service_land_git_steps_and_import_stream.rs`) answered
  `Unavailable` instead of `StepTooSlow`,
  so the commit the landing had built was
  not kept on `refs/cli-git/kept/<n>`,
  the step was not named,
  and the scratch
  ref holding that commit was removed.
  That is the case to look at first:
  a step
  that may not be repeated loses the record of what it built when its ending is
  misread.

### The fix

Three changes,
each with a control:

- The wait's backstop is the ceiling plus `THE_WATCH_HAS_ITS_ENDING`,
  which is
  `WATCHER_END_BOUND`:
  the bound the drop of a watch already gives its task,
  itself the owner's "3x worst case" over the 6.03 s delay measured here plus a
  second,
  24 times the 41.68 ms tail measured after the kill
  (`foundation/worst_delay_seen_on_this_host.rs`,
  `push/child_watch_running_task.rs`).
  A wait that is only the backstop never
  takes the ending from a watch that is still at it.
- Only a watch whose pin proves which process to stop gets that room.
  A watch
  that can signal nothing (`Pinned::Unproven`) and one that never started leave
  the whole ending to the wait,
  at the ceiling itself,
  with the sentences that
  say so.
  `ChildWatch::what_watches_the_step` is the one place that answers
  which of the three it is (`TheWatchOfAStep`).
- The error of a step the wait ends at that ceiling is the ceiling's own when a
  watch was what should have ended it,
  and the wait's sentence when nothing
  could:
  `CarriedCeiling::ending`.
  A watch that proved no process ends nothing,
  so neither of its two readings (`ceiling_hit`,
  `expired`) is taken as the
  step's ending (`ending_of_a_watched_step`).

No wait became unbounded:
the backstop is still a deadline,
later by one
measured bound.
`only_a_watch_that_can_end_the_step_puts_the_backstop_behind_its_ceiling` pins
the three backstops.
