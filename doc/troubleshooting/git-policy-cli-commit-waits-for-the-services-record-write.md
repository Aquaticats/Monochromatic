# A commit waits for the service's own record of a request, on a drive the commit does not touch

Date:
 2026-11-15.
Subject:
 the commit service's landing board published one decision file per request in the journal's `busy` directory
 before its group was landed,
 so a commit's place in the queue waited for a write of the drive the state directory is on,
 120,560 ms and 78,864 ms in the two sightings,
 while the repository's drive answered in 10 ms.
Owner's rules this bears on:
 "a commit takes no longer than the greater of five seconds and twice what real Git takes for the same commit"
 (`SPEC.md`),
 "slow storage is what the wrapper is measured against and removed from",
 and "a wait over the bound names the step it is in,
in the service's log and to the person waiting".

## Symptom

`0.6.1-alpha.37`,
the main checkout of this repository (on `/var/mnt/encrypted`),
2026-11-15 between 00:54 and 00:58 local.
Two commits printed

```text
The commit service has not said where this commit stands yet; it has waited 10 seconds.
```

and then `This commit is next.`,
and both landed (`fd002ed513`,
`5a7c6c0d03`).
Nothing was lost.
The service was alive the whole time:
a position can only be printed from a place the service named
(`service/service_client_waiting_words.rs`,
`landing/service_messages_of_a_wait_for_the_service.rs`).

## What the journal proves

The instance's own lines (`journalctl --user -u 'monochromatic-git-policy-cli@….service'`):

```text
00:56:35 answered request="c38f61f8-…" outcome=landed size=1 position=1 queued_behind=0 read_ms=0 \
         queued_ms=120560 settle_ms=1 lock_wait_ms=0 landing_ms=2747 answer_ms=0 total_ms=123442 \
         syncs_waited=0 steps="git config@0+3;…;git show@0+6;git config@120690+8;git rev-parse@120707+6;…"
00:58:31 answered request="040edde0-…" outcome=landed size=1 position=1 queued_behind=0 read_ms=0 \
         queued_ms=78864 landing_ms=168 answer_ms=0 total_ms=79063 syncs_waited=0
```

- Both waits are `queued_ms`:
  the time from the request line being read to the group's landing beginning.
  Nothing was ahead of either commit in its batch (`queued_behind=0`,
  `position=1`),
  so they did not wait for another commit.
- Neither wait is a sync a landing waited on:
  `syncs_waited=0`,
  and every sync of both landings was queued to the
  background task (`landing/service_durable.rs`).
- The step ledger of the first request lists nine Git steps that began before its own connection was accepted
  (their offsets saturate at 0),
  and then nothing until `git config@120690`.
  So between the request being read and its landing beginning,
  the service started no Git child at all:
  the 120 s was spent inside one step of its own that is not a Git command.
- The journal holds no line between 00:52:47 and 00:56:32,
  so which step of the service held the first commit is not measured by it:
  the service wrote nothing that named the wait.
- The drive readings from a 4 KiB write-and-sync probe begun once a minute
  (`~/temp/agent/disk-ceiling-search-by-agent-count-2026-11-13/keep/probe-readings.jsonl`):
  the reading begun 00:54:43 took 109,241 ms,
  the one begun 00:57:33 took 59,474 ms,
  and the repository's drive took 10 ms and 78 ms in the same minutes.
  Both stalls end within a second of the moment the service's own write returned.

## The steps between a request and its answer

Every step below runs on the landing thread,
the one thread that lands and answers,
between the moment the service reads a request line and the moment the client is answered.
"Waited for" means the commit's own answer could not happen before the step ended.

On the connection task,
before the request reaches the queue:

- the request line is read (`read_ms`),
  in the socket;
- the check of whose the request is (`is_foreign`),
  on the repository's drive,
  and skipped when the request names this instance's own Git directory;
- `service_scratch_trees_of_queued_requests::note`,
  a `Mutex` in memory;
- the job is sent to the landing thread's channel,
  in memory.
  Nothing here writes storage.

On the landing thread,
before the group's landing begins (all of it inside `queued_ms`):

- `receive_for_at_most`,
  a wait on the channel;
- the withdrawals and the grouping of the batch,
  in memory
  (`service/service_server.rs`);
- `service_scratch_release_removal::the_tree_of` and `shadow_drawn::is_a_drawn_tree`,
  path checks in memory;
- `begin_unfinished_work`,
  an atomic count;
- `LandingBoard::begin_group`:
  one decision file per request,
  `<journal>/busy/<id>.decision`,
  created whole and published without replacing,
  in the state directory (`StateDirectory=monochromatic-git-policy-cli/service/%i`),
  which is under `~/.local/state` on the drive the home directory is on.
  It was written for every request,
  and never read back before the landing.
  Waited for.
- `board.begin_group`'s per-request removal at the end of a group,
  `end_group`,
  is the same directory and runs before the loop takes the next job,
  so the next commit's `queued_ms` covers it too.
  Waited for.

Inside the landing (inside `landing_ms`,
after `queued_ms` ends):

- `enter_step` writes `<journal>/busy/step` before every Git child of the landing and removes it after
  (`service/service_busy_step_record_on_this_thread.rs`),
  on the same drive.
- the journal intent of the batch,
  `<journal>/*.open`,
  written and published on the same drive (`journal_ms`).
- the branch move and the index update,
  on the repository's drive
  (`ref_ms`,
  `index_update_ms`).
- every sync of those records is queued to one background task and awaited by nobody
  (`syncs_queued`,
  `syncs_waited=0`).
- the note that names the repository to the next service,
  bounded by `LANDING_WATCH_NOTE_CEILING` (1 s).

Off every waiting path:

- the after-commit work's plan,
  watch and housekeeping records,
  in the same state directory,
  run by the work loop's own task and by the slow lane
  (`landing/service_work_slow_lane.rs`),
  not on the landing thread;
- the background syncs,
  in one task;
- the client's own scratch,
  saved message and timing record,
  in the repository's Git directory,
  on the repository's drive;
- the client's runtime directory,
  `/run`,
  tmpfs.

Steps marked "waited for" and on the home drive are the candidates for the 120 s:
the two of `begin_group` and `end_group`.

## Which step the evidence points at, and how sure

The evidence points at the landing board's decision record,
with two limits stated plainly:

- It is the only storage write the landing thread makes between reading a request and beginning its landing,
  which is exactly the interval `queued_ms` measures,
  and the step ledger shows no Git child in that interval.
  That is a reading of the code and not a measurement of the sighting.
- The journal says which interval the wait was in and that it was not a sync,
  a child or another commit.
  It cannot say whether the write that held the first commit was the arriving group's own decision file or the
  removal of the previous group's,
  because both are the same directory,
  both are on the landing thread,
  and the
  service wrote no line between 00:52:47 and 00:56:32.
- What would have made it certain is a line the service did not write:
  one naming the step the landing thread was inside when a request waited past the bound,
  and one naming it to the person waiting.
  That line is now written:
  `A_QUEUE_WAIT_NAMED_AFTER` (`service/wait_ceilings.rs`) is five seconds,
  the floor of this project's bound on a commit,
  and past it the answer carries
  "a commit waited N seconds for the landing thread to take its group,
  and …;
  the landing thread was last inside
  `<command>`,
  which ended M seconds before this answer",
  or,
  when no write was recorded,
  "the landing thread had recorded no write it was inside,
  so the wait was in the work before its first write".
  The same words go to the waiting client as a `waiting` notice.

## The control, without a slow drive

`service/service_decision_record_hold_control_window.rs` holds the write of a request's decision where a stalled
drive would,
and only the explicit `CLI_GIT_TEST_ONLY_DECISION_RECORD_HOLD` arms it:
the board leaves `<path>.reached` and goes on once `<path>.release` is there,
or at 60 seconds.
`binary/binary_commit_while_a_decision_record_is_held_tests.rs` starts a fixture service with that variable,
lets the first commit through,
holds the record while a second commit arrives,
and asserts the second is answered inside the greater of five seconds and twice what real Git took for the same
commit in the same fixture.

Before the change,
the same control on a wrapper built from the tree before it:

```text
MEASURE held=true commit_ms=6141 bound_ms=5000 git_ms=4 queued_ms=5975
the commit waited 6141 ms while the service was held inside the write of a request's decision record,
and this project's bound on a commit is 5000 ms (twice real Git's 4 ms, or five seconds):
the service answered it with queued_ms=5975.
```

with the fix taken back out of the same tree by putting the write into `begin_group` again:

```text
MEASURE held=true commit_ms=6730 bound_ms=5000 git_ms=6 queued_ms=5977
```

and at the change:

```text
MEASURE held=false commit_ms=46 bound_ms=5000 git_ms=6 queued_ms=0
```

`held=false` is the point:
the landing thread never reaches that write any more,
so a commit is answered while the record stays unwritten.

## The change

The first change after the sighting moved the *write* of the decision off the landing thread and left a *read* of it
there,
and the review of it found the read to be the same dependence:
the control that holds the record reports 6,141 ms while its write was held and 6,156 ms while its read was,
with
`queued_ms` 5,974 in both.

So every reach of that record is memory now,
one `Mutex` over the identities,
the last write a landing left,
and the withdrawals decided,
on the lock inventory with its reason beside it
(`source_guard/service_lock_inventory_tests.rs`):

- `begin_group` claims each identity and looks at the withdrawals under the same hold of the lock,
  which is one decision with no drive and no other thread between the two halves:
  a request a withdrawal took is dropped,
  and the rest are landed.
  Nothing can keep a request out of its group any more,
  so `GroupAdmission` has no `unavailable` arm and the server
  answers no request with a failure for that reason.
- `withdraw_early` leaves a request the board holds to the landing thread,
  decides a withdrawal of any other under the token of its maker,
  takes that decision back when the claim or the journal says otherwise,
  and confirms it in memory.
- `end_group` gives the claims back and forgets the oldest confirmed withdrawals past `DONE_RETAINED`,
  with their
  confirmations in the same record,
  so a confirmation cannot outlive the decision it belongs to.
- The step record (`busy/step`) and the syncs are untouched.
- Two controls whose subject the change removed are gone:
  a staging file of a decision,
  and a confirmation that outlives its decision.
  What replaces the first is `an_admission_and_a_decision_need_no_file_of_the_record`,
  which makes the record's own directory take no new file and shows an admission and a withdrawal still decided;
  the second cannot happen,
  because one record holds a decision and its confirmation together.

### What is durable when a commit is answered, and what is not

- Durable:
  the batch's journal record in the state directory
  (`*.open` written and published during the landing,
  and the `*.done` mark and the refs after it),
  its syncs queued to the background task,
  and the branch itself on the repository's drive.
  A commit that is answered is in the journal,
  so the after-commit work pushes it without a person acting,
  and a restarted service settles the batch from that record.
- Not durable,
  and never was:
  the board's claims and its record of withdrawals.
  `LandingBoard::new` wiped the directory at every service start before this change,
  so nothing in it ever described a thread of a service that had stopped,
  and the claims now die with the landing that made them.
  A crash leaves no admission record and no claim,
  which is the state a fresh service must start from:
  a withdrawal of a request no batch names is either published afresh or left to the landing thread,
  and one whose commit landed is answered `landed` from the journal.
- The record write of a request is gone,
  so a crash between the answer and it leaves nothing to put right.
  The journal is the only record of a landed request,
  as it was for the answer's own durability.

## The cost when no drive is slow

Two arms built from the same source,
the second with the write put back into `begin_group`,
run interleaved:
five rounds,
five commits an arm in each,
the arm that goes first alternating.
Each arm's own wrapper was placed at the path its test binary starts it from,
so the service and the client of each
arm are that arm's build.

- fixed:
  25 commits,
  min 33 ms,
  median 41 ms,
  max 107 ms (the first measured commit of one run).
- write put back:
  25 commits,
  min 37 ms,
  median 40 ms,
  max 53 ms.
- ratio fixed to write put back,
  per interleaved pair:
  median 0.986,
  p10 0.84,
  p90 1.075,
  min 0.717,
  max 2.675 (the same first-commit outlier),
  and without it max 1.158.
- `queued_ms` was 0 for every commit of both arms:
  a write of a few hundred microseconds does not reach the millisecond the field counts,
  which is why the sighting needed a stalled drive to show it.

So the change costs a commit nothing measurable,
and the figure that matters is the one the control shows:
6,141 ms to 46 ms while the write is held.

## What is left, and what this change does not fix

- The landing still writes on the state directory's drive inside its own landing:
  `busy/step` before every Git child,
  the journal intent,
  and the `done` mark.
  A stall there costs `landing_ms` and `journal_ms`,
  not `queued_ms`,
  and a commit's answer still waits for its own journal record,
  which is the commit's and not bookkeeping.
  Taking those off the answer is a bigger change:
  the journal is what makes an answer durable.
- Nothing of the record is read from the state directory any more,
  so the note that a read does not join a filesystem
  transaction needs no weight here.
- The after-commit work of a round still drains its own records on the state directory's drive,
  on the work loop's task and on the slow lane,
  where no commit waits for it.
- The two sightings' precise attribution stays unproven,
  and no run may stall a real drive to prove it.

## What a start does before it reads its queue, and what it cost on 2026-11-15

At 07:58:09 that day the main checkout's service was replaced by release `0.6.1-alpha.41`,
and the first commit after it took 5,577 ms (`answerWaitMs` 4,706),
where the five installs before it took 1,366 to 2,452 ms in all.
The instance's journal carries `at_ms`:
ready at 1794747489566,
the rules warmed at 1794747489575,
"removed the records of 10 old finished batches" at
1794747493203,
and the batch at 1794747494428.
So 3,628 ms passed between the warm line and the removal line,
where the two starts before took 106 ms (47 records) and 40 ms (13 records).
The drive probe read the home drive at 23 ms and 2 ms on either side of that minute,
so a stall of a few seconds is neither shown nor ruled out,
and no line named the step,
which is the same silence the two commits met.

Each step the start took before the landing thread read its queue,
and what a landing needs from it:

- `start_acceptor_or_exit`:
  the greeting.
  Stays first,
  or a waiting client reads a service that stopped.
- `restored_within`:
  the plan of pushes waiting for a new try,
  and the repositories this instance was woken for.
  Only the after-commit work uses them,
  so it moved into that work's task.
- `settle_open_batches`:
  the recovery of a batch a killed service left open.
  Stays:
  a landing that overtook it would
  move the branch on a batch whose own record is still open.
- `prune_done`:
  the removal of the records past `DONE_RETAINED`.
  No landing reads them,
  so it moved beside the loop.
- `landed_commits`:
  the newest finished records,
  read into memory.
  Stays:
  an answer from memory is what stops a repeat
  of a landed request from being landed twice.
- `earlier_commits_landed`:
  what an earlier release's helper finished,
  which is the same answer for its clients.
  Stays.

What stays names itself:
a step past `A_START_STEP_NAMED_AFTER` (one second) writes one line saying which step it was
and how long,
and a commit that waits past its own bound is told the step in the service's notice and in the client's
own words.

## The controls that keep it from coming back

- `binary/binary_commit_while_a_decision_record_is_held_tests.rs::
  a_commit_is_answered_while_the_record_of_a_decision_is_held`
- `service/service_busy_tests.rs::a_wait_over_the_bound_names_the_write_the_landing_thread_was_last_inside`
- `service/service_server_connection_handling_tests.rs::
  a_request_that_waited_past_the_bound_is_told_what_the_wait_is_in`
- `service/service_busy_tests.rs::admission_and_early_withdrawal_never_both_succeed`
- `service/service_busy_tests.rs::an_admission_and_a_decision_need_no_file_of_the_record`
- `service/service_busy_tests.rs::two_connection_threads_never_both_win_a_decision`
- `service/service_busy_tests.rs::a_wait_over_the_bound_names_the_write_it_was_in_when_the_group_began`
- `service/service_server_connection_handling_tests.rs::a_request_the_landing_thread_has_taken_is_never_told_it_was_not`
- `binary/binary_a_first_commit_after_a_start_tests.rs::
  a_first_commit_after_a_start_is_answered_while_the_old_records_are_removed`
- `binary/binary_a_first_commit_after_a_start_tests.rs::
  a_start_step_a_commit_waits_for_names_itself`
- `source_guard/service_lock_inventory_tests.rs`,
  which now names `service/service_busy.rs` and checks the reason beside its lock
