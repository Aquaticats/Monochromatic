## Step T12, the landing's body and its own storage steps are awaited faces, 2026-11-11T19:20:14-05:00

- Branch and worktree:
  `refactor/git-policy-cli-tokio-landing-batch-body-storage-steps-awaited`
  in `.claude/worktrees/pi-tokio-landing-batch-body-storage-steps-awaited`,
  from `3b94069dbf` on `integrate/git-policy-cli-jam-fixes-after-alpha-9`.
  Not merged into main and not merged into the integration.
- What this run was asked to do,
  in the plan's own order:
  the step the previous section leaves as "the batch body's own storage steps
  (`service_journal::write_intent`,
  `write_start`,
  the scratch-ref marker's records,
  the conclusion copy) and its one index-lock wait
  (`service_land_take_index_lock.rs`,
  whose announcement and holder notes are the reason it is not a leaf)".
  Four agents take this step at once,
  each on its own files;
  this one took the five the coordinator named and the awaited faces they need.
- What it does not do:
  the landing's body is still one synchronous function
  (`landing/service_land_batch_body.rs` was not changed),
  because every step it has not converted belongs to another slice of this step
  (`objects_of_batch`,
  `requests_that_land`,
  `finish_after_move`,
  the branch write and the hold),
  and a body made a task today would call the synchronous Git runner,
  which registers the pipes of a step through the runtime it is already inside and is refused there.
  So the faces below answer a body that becomes a task,
  and each one is marked with the reason no production caller reaches it yet.

### What moved

- `service/service_journal_writes_awaited.rs` (new),
  the two records a landing writes before its branch moves.
  `write_intent_awaiting` and `write_start_awaiting` run the same write as one job of the blocking pool,
  awaited no longer than `JOURNAL_WRITE_CEILING`,
  and the writes themselves stay in `service/service_journal.rs`
  (`write_intent`,
  `write_intent_carrying`,
  `write_start`),
  where the synchronous faces stay for the callers that are not tasks
  (the recovery's settle and the body).
  The file is new because `service_journal.rs` passed the 300-code-line limit of `rust/max-lines`.
- `landing/service_durable.rs`,
  what a landing's syncs became while one piece of code ran.
  A job of the pool cannot read or write the landing's state
  (`landing_state_carried_by_its_task.rs` is a `tokio::task_local!` its threads are outside),
  so `WhatTheSyncsBecame` carries the figures its answer states and the flushes the landing settles,
  `flush_file_carrying` and `flush_directory_carrying` are the one implementation of each flush,
  and the synchronous faces read the landing's deferral and keep the outcome themselves.
  Without that carry the intent's flush would be made instead of queued
  and a person's commit would wait for the disk,
  which is what `service_durable.rs` exists to keep off its path.
- `landing/service_scratch_ref_marker_of_a_batch.rs`,
  the records that name a batch's scratch reference and the sweep that settles them.
  `record_scratch_ref_awaiting` and `forget_scratch_ref_awaiting` write and remove one record as a job of the pool,
  `end_scratch_ref_record_awaiting` settles a failed landing's reference with awaited Git
  (`clear_after_a_failed_landing_awaiting`,
  which the previous section built),
  and `clear_recorded_scratch_refs_awaiting` reads the journal's records as one job,
  asks Git about each reference on the task and removes the settled records as another job.
- `landing/conclusion_state_copy.rs`,
  the copy of the conclusion state a landed commit's cleanup compares.
  `keep_of_intent_awaiting` makes the same copy on the pool,
  and a copy that did not run or gave no answer is answered as one that was not kept,
  which the cleanup reads as "no conclusion state was copied" and removes nothing on,
  so nothing of the person's merge,
  cherry-pick or revert state is taken on a copy nobody proved.
- `landing/service_index_lock.rs`,
  `landing/service_index_attempts.rs`,
  `landing/service_index_prestage.rs`,
  `landing/service_index_refused_lock.rs`,
  `landing/service_index_claim.rs`,
  `landing/service_index_reclaim.rs` and
  `landing/service_index_placement.rs`,
  the landing's own memory of its lock waits,
  which is why the index lock is not a leaf.
  `TheLockWaitsMemory` holds the markers made ahead the take may still use,
  the foreign lock an earlier commit gave up on and the claims a reclaim could not read back;
  `carry_the_lock_waits_memory` takes it out of the landing's state and
  `keep_the_lock_waits_memory` writes it back,
  `acquire_carrying` is the acquisition as a job of the pool reads it,
  and every entry point that is not a job (`acquire_noting_holder`,
  `acquire_on`,
  `acquire_within`)
  carries the memory the same way,
  so a landing that takes the lock twice reads what the first take learned,
  and the synchronous faces (`take_carrying`,
  `recall_in`,
  `remember_in`,
  `take_claim`,
  `reclaim_carrying`,
  `reclaim_if_carrying`) are the one implementation of each step with the memory carried.
  Every synchronous face keeps its name and its behaviour:
  `take`,
  `recall`,
  `remember`,
  `forget`,
  `take_claim`,
  `reclaim` and `reclaim_if` read and write the state as they did,
  and only the include of the memory is new.
- `landing/service_land_take_index_lock.rs`,
  the take itself.
  `take_index_lock_awaiting` runs the acquisition as one job of the blocking pool,
  forwards each notice the job makes to the clients through a channel while it awaits
  (`AnnouncingToTheAcquirer`),
  keeps the timing and the holder notes the job answers,
  and makes the refusal the synchronous face makes:
  the intent is forgotten as a job of the pool,
  the built commit is kept on the side reference the person is told about with awaited Git,
  and the scratch reference is deleted with awaited Git.
  The bound is the one the synchronous face is given (`context.lock_ceiling`,
  `LANDING_LOCK_CEILING` or `SETTLE_LOCK_CEILING`),
  so the wait still ends at it and a holder that outlasted it is refused with `IndexInUse` and named.
- `landing/service_land_git_steps_awaited.rs` (new),
  the two Git steps the refusal needs,
  awaited:
  `run_awaiting` and `run_words_awaiting` (the synchronous step with its child awaited),
  `keep_on_a_side_ref_awaiting`,
  `write_until_the_reference_names_awaiting` and `keep_unlanded_awaiting`.
  The words of both faces come from one place each
  (`the_failure_of_a_step`,
  `the_line_of_an_unlanded_commit`,
  `refused_words_of`),
  so a task and a synchronous caller cannot answer one ending two ways.
  The Git steps themselves stay in `landing/service_land_git_steps_and_import_stream.rs`,
  which is where one place names each request and each answer,
  and the move brought that file back under the 300-code-line limit of `rust/max-lines`.
  A control shows why the awaited face is needed rather than the synchronous one:
  a step that may be ended at its ceiling registers its pipes by entering the runtime
  (`step_output_read_whole_through_pipes.rs`,
  `attach_pipes`),
  which a task may not do,
  so the synchronous face of a quick step answers `Unavailable` inside one.
- `service/wait_ceilings.rs` names `JOURNAL_WRITE_CEILING`:
  the journal write step's longest record (40,188 ms of 1,302 landings,
  99th percentile 3,792 ms),
  five times that in whole minutes,
  which is four minutes,
  and it bounds every record a landing writes into its batch's journal directory.

### What pins it

Eighteen controls were written before the faces were awaited,
and the ones a plant can be made for were shown to fail against the shape they replace.
Two of the quotes below name the files the faces were in when the plant ran
(`service_journal_awaiting_tests.rs` and
`landing/service_land_git_steps_and_import_stream_awaiting_tests.rs`);
they were renamed when the faces moved to `service_journal_writes_awaited.rs` and
`service_land_git_steps_awaited.rs` for `rust/max-lines`,
and each control's own name below says where it lives now.

- `an_intent_write_that_gives_no_answer_does_not_hold_a_worker`
  (`service/service_journal_writes_awaited_tests.rs`) stands a pipe where the intent's temporary file belongs,
  runs the write on a runtime of one worker,
  and requires a task beside it to run.
  Planted,
  by writing the intent on the task as the landing does today,
  it failed after `THE_PATIENCE` of 10 s:

  ```text
  thread '...an_intent_write_that_gives_no_answer_does_not_hold_a_worker' panicked at src/native/service/service_journal_awaiting_tests.rs:123:5:
  the task beside an intent write that gives no answer still runs, so no worker is held by that write
  ```

- `the_awaited_intent_write_keeps_the_flushes_the_landing_defers`
  requires the intent's two flushes to be the landing's to settle and its figures to be recorded,
  inside a landing whose syncs are deferred.
  Planted,
  by having the job write the intent itself with no deferral carried,
  it failed:

  ```text
  assertion `left == right` failed: the flushes of the intent are the landing's to settle, so nothing waited for the disk
    left: ""
   right: "intent-file=queued;intent-dir=queued"
  ```

- `the_awaited_start_record_is_the_one_the_synchronous_face_writes`
  requires both faces to write one record,
  and a job that gave no answer to be answered as a record that could not be written.
- `a_copy_that_gives_no_answer_does_not_hold_a_worker`
  (`landing/conclusion_state_copy_tests.rs`) stands a pipe where a shadow state file belongs,
  runs the copy on a runtime of one worker,
  and requires a task beside it to run,
  the answer to keep nothing and no copy to take the name the cleanup reads.
  The pipe is opened by the copy itself,
  so the read waits in the kernel exactly as a mount that stopped answering does.
- `the_awaited_copy_keeps_what_the_synchronous_one_keeps`
  compares both faces' trees of one preparation,
  and
  `a_copy_no_job_promised_is_answered_as_one_that_removes_nothing`
  states both answers a refused or unanswered job gives.
- `the_awaited_record_names_what_the_synchronous_one_names`
  (`landing/service_scratch_ref_marker_of_a_batch_awaiting_tests.rs`) and
  `the_awaited_sweep_settles_what_the_synchronous_one_settles`
  require both faces to write one record and to settle a dropped batch's reference one way
  (the commit on `refs/cli-git/kept/<seq>`,
  the record and the reference gone),
  and
  `a_failed_landings_record_goes_once_its_reference_did`
  requires the awaited end of a failed landing to keep its commit and remove its record.
- `a_sweep_that_gives_no_answer_holds_no_worker`
  stands a pipe where a record of the journal belongs and requires a task beside the sweep to run,
  and the reference of the batch the record named to stay,
  because a look nobody answered must set no removal in motion.
- `a_quick_git_step_is_answered_inside_a_task`
  (`landing/service_land_git_steps_awaited_tests.rs`)
  asks Git for the head through the awaited face and requires the synchronous face of the same step to be refused inside a task.
  Planted,
  by running the synchronous step in the awaited face,
  it failed:

  ```text
  thread '...a_quick_git_step_is_answered_inside_a_task' panicked at src/native/landing/service_land_git_steps_and_import_stream_awaiting_tests.rs:54:38:
  the awaited face asks Git for the head this repository has: Unavailable
  ```

- `both_faces_keep_one_side_reference_and_say_one_line`
  requires the kept reference and the sentence to be one for both faces.
- `the_holder_of_a_held_lock_is_named_while_the_take_waits`
  (`landing/service_land_take_index_lock_awaiting_tests.rs`) holds an `index.lock` a live process is proven to hold,
  gives the take a ceiling of five seconds,
  and requires the first notice to reach the clients in the first half of the whole take
  (which is its staging and then its ceiling,
  so the notice came while the wait ran),
  the holder to be named and the wait to end at its ceiling with `IndexInUse`.
  Planted,
  by forwarding the job's notices only when the job ended,
  it failed:

  ```text
  thread '...the_holder_of_a_held_lock_is_named_while_the_take_waits' panicked at src/native/landing/service_land_take_index_lock_awaiting_tests.rs:126:5:
  the holder's notice reached the clients while the wait still ran (5.015059187s of 5.041425923s)
  ```

- `the_lock_a_take_gave_up_on_is_remembered_by_the_landing`
  requires the memory of a refused lock to come back to the landing and the next take,
  with a ceiling of a minute,
  to be refused at once.
  Planted,
  by dropping what the job answered of the landing's memory,
  it failed after the second take waited its whole minute:

  ```text
  monochromatic-git-policy-cli-service: waited for the index lock batch=5 result=refused lock_wait_ms=60010 lock_stage_ms=0 lock_stage_kind=cold lock_place_ms=60009 lock_looks=7 lock_found="foreign no-marker size=7 age_ms=316" lock_last="foreign no-marker size=7 age_ms=60325" at_ms=1794441657991

  thread '...the_lock_a_take_gave_up_on_is_remembered_by_the_landing' panicked at src/native/landing/service_land_take_index_lock_awaiting_tests.rs:292:5:
  the lock the take gave up on is the landing's memory, so the next take does not wait for it again
  ```

  The 60,010 ms in that line is the incident the control is named for:
  four commits of one worktree waited 678 to 1,229 s behind one lock on 2026-11-07,
  each paying a full ceiling for the file the one ahead had already given up on.
- `the_awaited_take_holds_the_lock_the_synchronous_one_holds`
  requires both faces to hold a lock that names the batch they took it for and to stage the marker they take it from the same way,
  and
  `the_awaited_take_starts_the_markers_of_the_batches_after_it`
  requires the markers of the two batches after it to be started and to reach the landing that asked,
  which is what keeps the next commit from staging its own.
- `a_refused_take_keeps_the_commit_and_forgets_the_intent`
  requires the refusal of a held lock inside a task to keep the built commit on `refs/cli-git/kept/<seq>`,
  remove the batch's scratch reference,
  forget its intent and name the reference to the person.

Two of these pin less than their names suggest,
and this is said rather than hidden:

- the prestage half of `TheLockWaitsMemory` has no control that can tell a carried slot from one the take starts again,
  because the face starts the two markers after the batch whatever came back,
  so a lost slot costs one more making and no answer a control can read.
  What a control pins is the refused lock's memory (above),
  which is the part of the carry whose loss a person feels.
- the conclusion copy's stall control makes the copy fail as well as wait,
  because `std::fs::copy` refuses a pipe as its source once the pipe is opened,
  so the answer the control reads is "not kept" and not "kept after the wait".

### Checks

- `native:clippy` on every target:
  `Finished` of the `dev` profile in 46.26 s after the last code change,
  with warnings denied.
- Both test targets built with `--no-run`:
  `Finished` of the `test` profile in 3.59 s for the library and in 1 m 57 s for `native_binary`.
- `native:test:host` filtered to
  `landing::`,
  `service::service_journal`,
  `service::wait_ceilings` and
  `service::service_timing`,
  in a `systemd-run --user --scope` at `CPUQuota=700%`,
  `MemoryMax=22G` and `TasksMax=3000`,
  with `CLI_GIT_TEST_ROOT=${HOME}/temp/agent/cli-git-fixtures/root`,
  run five times,
  in the order they were made:
  1. `1006 tests run: 1002 passed (10 slow), 4 failed` in 388.088 s.
     One of the four was this run's own defect:
     `landing::service_index_lock::tests::a_lock_that_outlasted_one_wait_is_refused_at_once_to_the_next_commit`
     failed because the entry points a landing does not reach through a job
     (`acquire_noting_holder`,
     `acquire_on`,
     `acquire_within`)
     carried no memory,
     and they now carry it as `acquire_noting_wait_within` does.
     One is the failure this branch's brief names as already red and owned by another agent,
     `landing::service_work::tests::a_retryable_answer_keeps_the_record_for_a_later_round`,
     which passed in every run after it.
     The other two passed when run again and in every later run.
  2. `1006 tests run: 1004 passed, 2 failed` in 88.347 s:
     the known red one,
     and
     `service::wait_ceilings::tests::a_journal_record_waits_five_times_the_longest_write_measured`,
     which still named the file the awaited journal faces had moved out of.
     It names the new one now.
  3. `1006 tests run: 1006 passed, 5956 skipped` in 166.567 s.
  4. `1006 tests run: 1005 passed (3 slow), 1 failed` in 323.652 s:
     `landing::service_land_take_index_lock::awaiting_tests::the_holder_of_a_held_lock_is_named_while_the_take_waits`,
     whose patience of ten seconds a take outlasted while it waited for a place of the process's blocking budget.
     That patience is sixty seconds now,
     as every other control of that file.
  5. `1006 tests run: 1005 passed (16 slow), 1 failed, 5956 skipped` in 855.250 s,
     with
     `landing::service_land::hung_write_tests::a_client_is_told_it_is_inside_the_write_when_the_write_starts`
     as the one failure,
     whose line is the service's own:
     "cannot start the acceptor of connections:
     the runtime that runs it has no worker thread (the system may refuse threads;
     check `ulimit -u` and the control group's `pids.max`)".
     That module alone then ran
     `7 tests run: 7 passed, 6955 skipped` in 18.780 s.
- The modules this run touched,
  run as one selection
  (`landing::service_index`,
  `landing::service_journal_writes_awaited`,
  `landing::service_land_git_steps_awaited`,
  `landing::service_land_take_index_lock`,
  `landing::conclusion_state_copy`,
  `landing::service_scratch_ref_marker`,
  `landing::service_durable`,
  `landing::service_land::hung_write_tests`,
  `landing::service_handoff::tests::the_wake_message`):
  `233 tests run: 233 passed, 6729 skipped` in 105.399 s.
- The control of the timing line a commit's answer states,
  which this run's `service_durable.rs` change could have emptied
  (`service/service_durable.rs` records the figures a landing's syncs became,
  and a real commit's line still names `lock-marker=queued`):
  `native:test:binary:host -- a_wrapped_commit_leaves_one_record`:
  `1 test run: 1 passed, 1030 skipped` in 13.678 s.
- `//:lint:monochromatic-lint --max-warnings 0` on the whole `landing` and `service` folders and on this record:
  0 errors and 0 warnings,
  after `format:monochromatic-lint` wrote the line breaks the prose rule asked for,
  and after `service_journal.rs` and `service_land_git_steps_and_import_stream.rs` were split into
  `service_journal_writes_awaited.rs` and
  `service_land_git_steps_awaited.rs`,
  because each passed the 300-code-line limit of `rust/max-lines` by this run's faces.
- Not run:
  the container gate,
  `native:perf`
  (nothing of this run is in a forwarded command's path,
  and no synchronous face changed what it does),
  `native:clippy:windows`,
  and any mutation campaign (paused by the owner).

### What is left

- The landing's body (`landing/service_land_batch_body.rs`) is still one synchronous function,
  and it is the only caller the faces above are waiting for:
  `write_intent_awaiting` and `write_start_awaiting` (a new file,
  `service_journal_writes_awaited.rs`) where it writes its records,
  `keep_of_intent_awaiting` where it keeps the conclusion state,
  `take_index_lock_awaiting` where it takes the lock,
  and `run_awaiting` with `keep_unlanded_awaiting` where it moves and clears references.
  What still stands between it and a task is the other slices of this step:
  `objects_of_batch` (which also writes the scratch marker's record through
  `record_scratch_ref`),
  `requests_that_land`,
  `batch_content_missing`,
  `witness_of`,
  `finish_after_move`,
  `what_a_batch_owes`,
  the reference-lock wait,
  the hold of a batch and the branch write itself.
- `record_scratch_ref_awaiting` has no caller yet:
  the record is written inside `service_land_objects_of_batch.rs`,
  which is another slice's file,
  so the face waits for that slice's `record_scratch_ref` call to become the awaited one.
- `clear_recorded_scratch_refs_awaiting` and `end_scratch_ref_record_awaiting` wait for
  `landing/service_land_finish.rs` (the restart's sweep) and `landing/service_land.rs` (the end of one landing),
  which are respectively the recovery path's and the landing loop's.
- A take whose job gave no answer cannot hand the landing's memory back:
  the markers made ahead it carried are dropped with it,
  so the next take stages its own marker (the nine to thirteen seconds a cold staging costs),
  the memory of a refused lock is lost (the next commit waits a full ceiling for the same file) and
  a claim whose placement could not be read back stays beside the lock until a later reclaim judges it.
  No published wait reaches it except a system that refuses every thread,
  and the answer says so in the log.
- An `IndexLock` a job took and nobody read is removed by its own `Drop` on the pool's thread,
  where an ownership question it cannot answer is counted in no landing's state
  (`service_index_hold.rs`,
  `note_unsettled`):
  the lock then stays until a later reclaim or a restart,
  which is the cost of a lock taken by a step whose answer was dropped.
- A take's `lock_wait_ms` now covers the wait for a place of the process's blocking budget as well as the wait for the lock,
  because the clock starts before the job is asked for;
  the synchronous face measures the acquisition alone.
  What a place costs is unmeasured here,
  and the figure the service logs can only grow by it.
- The journal write's ceiling is four minutes,
  five times the longest journal write the service log holds;
  a write that outlasts it ends its landing with a failure,
  which is a commit the person runs again and never a commit that lands half recorded.
- `service_durable.rs`'s two carrying faces are one implementation each with the synchronous ones,
  and the synchronous ones were not changed in what they do:
  the same figures in the same order,
  the same flush queued or made,
  and the same value answered.
