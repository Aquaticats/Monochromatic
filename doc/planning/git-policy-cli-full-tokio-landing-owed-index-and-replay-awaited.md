## Step T12, the landing's owed index and its replay are awaited faces, 2026-11-11T22:27:44-05:00

- Branch and worktree:
  `refactor/git-policy-cli-tokio-landing-owed-index-and-replay-awaited` in
  `.claude/worktrees/pi-tokio-landing-owed-index-and-replay-awaited`,
  from `integrate/git-policy-cli-jam-fixes-after-alpha-9` at `3b94069dbf`
  (the integration after its twelfth merge pass).
  Not merged into the integration or into main;
  the coordinator merges this branch.
- What this run was asked to do,
  in the plan's own order:
  the slice of step T12 that the previous section leaves as
  "the owed index update and repair (`owed_index_entries.rs`,
  `owed_index_repair.rs`),
  the replay and amend replay (`service_replay.rs`,
  `service_amend_replayed.rs`,
  `held_commit_replay.rs`,
  `held_commit_settle.rs`)".
  Four agents take this step at the same time with disjoint file sets;
  this one took the six files above and the faces of the helpers they call.
  The landing's body still calls the synchronous faces,
  so no person's commit behaves differently after this run.
- The two choices the previous sections recorded as the coordinator's pick under
  the owner's delegation are unchanged here:
  the loop's three state steps keep their own state rather than running in turn,
  and the hand-back of a job's value through a store the caller awaits was not
  built.

### What moved

- `landing/owed_index_entries.rs`,
  the five Git questions the repair reads a tree and an index with.
  Each has an awaited face a task asks
  (`tree_entries_awaiting`,
  `index_entries_awaiting`,
  `whole_listing_awaiting`,
  `head_entries_awaiting`,
  `moved_since_listing_awaiting`),
  and both faces of every question ask one request
  (`specs_request`,
  `whole_listing_request`,
  `unborn_request`)
  and read Git's answer through one function
  (`answered`,
  `tree_entries_of`,
  `index_entries_of`,
  `unborn_answered`,
  `moved_between`),
  so no face can read another listing than another.
- `landing/owed_index_repair.rs`,
  the whole repair of what a landed batch still owes an index
  (`exclude_what_a_landed_commit_left_awaiting`,
  `exclude_with_note_awaiting`,
  `exclude_with_note_within`).
  Three storage steps are jobs of the blocking pool awaited no longer than
  `COMMIT_OWN_STATE_CEILING`:
  the look at the owed records (`records_within`),
  the witness of the live index (`witness_within`),
  and the look at every listed deletion and every compared worktree path
  (`present_in_the_worktree`,
  `worktree_file_within`).
  The copy of the commit's own index aside and the put back are jobs of the pool
  with no ceiling,
  because a copy abandoned half-written is a file nothing waits for.
  Each of the three reads keeps its three answers apart:
  a set of records that cannot be read is the answer it already had,
  a witness that gives no answer is compared with nothing,
  and a look that gave nothing in time stops the commit with its own sentence
  rather than letting it record what it never read.
- `landing/service_land_follow.rs`,
  the two steps of the repair that live there:
  `derived_names_awaiting`,
  `reset_entries_awaiting` and the `run_awaiting` they share,
  each asking the one question its synchronous face asks
  (`derived_name_arguments`,
  `names_answered`,
  `reset_once_awaiting`).
  This file is not one of the files the other agents were named as changing,
  but the landing's body step is likely to touch it:
  the faces are additive and no synchronous face changed.
- `landing/service_amend_replayed.rs`,
  the commit an amend makes again:
  `rebuilt_onto_awaiting` (the message a job of the pool with no ceiling,
  then the object awaited),
  `whole_commit_awaiting` and `signed_commit_awaiting` over the shared
  `whole_commit_bytes`,
  `whole_commit_request`,
  `signed_commit_request` and `say_which_headers_are_dropped`,
  and `amended_fields_awaiting`.
- `landing/held_commit_replay.rs`,
  the replay of one held batch:
  `onto_awaiting` (the scratch directory a job of the pool with no ceiling),
  `current_value_awaiting`/`value_now_awaiting`,
  `parents_of_awaiting`,
  `merged_tree_awaiting` and `raw_commit_awaiting`,
  over the shared `current_value_request`,
  `parents_request`,
  `merge_tree_request`,
  `raw_commit_request` and the four readings of Git's answers.
  `merged_tree_awaiting` asks the landed tip's attributes through
  `attribute_tree_variables`, awaited, instead of entering the runtime for them.
- `landing/held_commit_settle.rs`,
  the settlement's own two Git starts and its two state reads:
  `landed_below_awaiting`,
  `move_branch_awaiting`,
  `intent_within`/`intent_awaiting` and
  `finished_within`/`finished_awaiting`
  (both jobs of the pool at `COMMIT_OWN_STATE_CEILING`),
  over the shared `landed_below_request`,
  `move_branch_request`,
  `landed_below_answered` and `move_branch_answered`.
- `landing/service_replay.rs`,
  the replay's two leaf reads:
  `tree_of_commit_awaiting` and `captured_tree_awaiting`,
  the second with its look at the private index a job of the pool at
  `COMMIT_OWN_STATE_CEILING` (`captured_index_look_within`),
  which keeps "the file is there", "it is gone" and "the look gave no answer"
  apart.
- `service/wait_ceilings.rs` gains `COMMIT_OWN_STATE_CEILING`:
  the wait a commit's own storage step is granted,
  `START_CEILING`,
  the value `LANDING_STATE_CEILING` already has,
  with the owner's "nothing should ever wait forever or indefinitely" beside it
  and no measurement,
  because the value bounds a wait rather than serving a workload.
- What did not change:
  every synchronous face keeps its name,
  its signature and its behaviour.
  The repair's, the replay's, the amend's and the settlement's own seven
  controls that read those faces through `git()`, `status` and `cat-file`
  pass unchanged,
  which is what pins that no refusal's words and no bound of them moved.

### What pins it

Fourteen controls,
written before and beside the faces,
each shown to fail against the shape it replaces.
Every quote below is a run of `native:test:host` filtered to the control named.

`landing/owed_index_entries_awaiting_tests.rs`:

- `the_awaited_reads_answer_what_the_synchronous_ones_answer`
  compares both faces over one tree,
  one index,
  one whole listing,
  one head and one copy the index moved past.
  Planted,
  by reading the awaited index listing through the tree parser,
  it failed:

  ```text
  assertion `left == right` failed: the two faces answer one thing about the entries of an index
    left: Ok({[65, 46, 116, 120, 116]: ("100644", "5225f47da9b3a2d2529c70329d56424b573726cb"), ...})
   right: Ok({[65, 46, 116, 120, 116]: ("100644", "0"), ...})
  ```

- `the_awaited_reads_answer_inside_a_task_where_the_synchronous_ones_are_refused`
  asks all five through the awaited faces in a task,
  and the synchronous face of the same question there.
  Planted,
  by starting the awaited face's Git through `run_git`,
  it failed:

  ```text
  assertion `left == right` failed: the awaited face answers the tree of the fixture inside a task
    left: Err(Unavailable)
   right: Ok({[65, 46, 116, 120, 116]: ("100644", "5225f47da9b3a2d2529c70329d56424b573726cb"), ...})
  ```

`landing/owed_index_repair_awaiting_tests.rs`:

- `the_awaited_repair_answers_what_the_synchronous_one_answers`
  runs both faces over one worktree whose index still holds the entry its
  landing replaced,
  each on its own copy of the index,
  and requires the same paths left out,
  the same sentence about the records and the same index afterwards,
  with the live index untouched.
  Planted,
  by asking the tree the landing replaced instead of the head,
  it failed:

  ```text
  assertion `left == right` failed: the two faces leave the same paths out of the commit
    left: ["one.txt"]
   right: []
  ```

- `both_faces_answer_one_thing_about_an_index_that_moved_after_the_copy`
  is the arm where the live index moved after the commit copied it,
  which reaches the tree of the live index and the fallback listing.
  Planted,
  by starting both of those steps through the synchronous runner,
  it failed:

  ```text
  the repair of an index that moved: "the entries the index moved since this commit copied it cannot be read (Unavailable)"
  ```

- `a_record_that_gives_no_answer_holds_no_worker_and_stops_the_commit`
  stands a pipe where the worktree's owed record belongs,
  runs the repair on a runtime of one worker and requires a task beside it to run,
  the commit to stop and the live index to stay.
  Planted,
  by reading the records on the task itself,
  it failed after `THE_PATIENCE` of 10 s:

  ```text
  the task beside a record that gives no answer still runs, so no worker is held by that read
  ```

  and planted,
  by reading a look that gave nothing in time as "nothing is owed",
  it failed:

  ```text
  a look that gave no answer stops the commit rather than answering that nothing is owed: []
  ```

`landing/service_amend_replayed_awaiting_tests.rs`:

- `the_awaited_commit_answers_what_the_synchronous_one_answers`
  requires both faces to write one commit object from one amended commit,
  which a commit's content addressing makes one name.
  Planted,
  by dropping the header lines the amended commit carried on the awaited path,
  it failed:

  ```text
  assertion `left == right` failed: the two faces write one commit object
    left: "51507e9a5bda250384874ae72926b5de39548d50"
   right: "317b8c9fe1e8e6a241b31d28d8897303f1e45d29"
  ```

- `the_awaited_fields_of_an_amend_answer_what_the_synchronous_ones_answer`
  reads one commit back into its authorship,
  committer,
  message and header lines,
  out of a repository whose tip is another commit.
  Planted,
  by reading the tip instead of the commit named,
  it failed:

  ```text
  assertion `left == right` failed: the two faces read one commit back into one set of fields
    left: AmendedCommit { ... message: [105, 110, 105, 116, 105, 97, 108, 10], ... }
   right: AmendedCommit { ... message: [97, 32, 108, 97, 116, 101, 114, 32, 99, 111, 109, 109, 105, 116, 10], ... }
  ```

- `the_commit_is_written_inside_a_task_where_the_synchronous_one_is_refused`
  writes the commit in a task and refuses the synchronous face there.
  Planted,
  by starting the awaited face's Git through the synchronous runner,
  it failed:

  ```text
  the awaited face writes the commit inside a task: Message("git hash-object -t commit -w --stdin could not start: the runtime that runs this command's waits was asked for from inside itself, where waiting for it would never end; nothing was run. ...")
  ```

`landing/held_commit_replay_awaiting_tests.rs`:

- `the_awaited_replay_carries_the_batch_what_the_synchronous_one_does`
  carries one held batch onto a branch that did not move and onto one that moved
  on another path,
  and requires one commit from both faces in each case.
  Planted,
  by parenting the commit the awaited face writes on the preparation base
  instead of the tip,
  it failed:

  ```text
  assertion `left == right` failed: the two faces carry the batch onto a branch that moved the same way
    left: Replayed { commits: ["be4a3a10fe1dd73a5efee9eecaefd77ae6706cb9"] }
   right: Replayed { commits: ["71b4b8ce67da8d530d222ec417bfcd5bb6af1e3a"] }
  ```

- `the_replay_is_made_inside_a_task_where_the_synchronous_one_is_refused`
  replays in a task,
  requires the same commit,
  and requires the synchronous replay to answer that the commit cannot be read,
  which is how its refused Git shows.
  Planted,
  by asking for the commit's parents through the synchronous runner,
  it failed:

  ```text
  assertion `left == right` failed: the awaited face carries the batch the synchronous one carried, inside a task
    left: Cannot { because: "the commit 6ff3198b83f4c1ec24b74dcc7f6c358250b1734a cannot be read" }
   right: Replayed { commits: ["6ff3198b83f4c1ec24b74dcc7f6c358250b1734a"] }
  ```

`landing/held_commit_settle_tests.rs`:

- `the_awaited_landing_question_answers_what_the_synchronous_one_answers`
  asks both faces about a branch the batch is on,
  one it is not on,
  and a name Git cannot answer for.
  Planted,
  by starting the awaited face's Git through the synchronous runner,
  it failed:

  ```text
  assertion `left == right` failed: the awaited face answers the same about a branch the batch is on
    left: true
   right: false
  ```

- `the_awaited_branch_write_answers_what_the_synchronous_one_answers`
  requires both faces to move a branch from the value it was read at and to refuse
  a stale expectation with the same words.
  Planted,
  by swapping from the batch's own commit instead of the value read,
  it failed:

  ```text
  assertion `left == right` failed: and they refuse it with the same words
    left: Some("fatal: prepare: cannot lock ref 'refs/heads/main': reference already exists")
   right: Some("fatal: prepare: cannot lock ref 'refs/heads/main': is at be93eb318080f56b782401204c7249db53dc8bbe but expected 80435a4dfdc8084be3b504cbce8d9f2b5b185316")
  ```

- `a_record_that_gives_no_answer_holds_no_worker_and_is_not_an_absent_one`
  stands a pipe where the batch's open record belongs,
  requires a task beside the read to run,
  the answer to be one no record can give,
  a damaged record to be told from an absent one,
  and a readable one to be answered as the batch it names.
  Planted,
  by answering a look that gave nothing in time as "no record is there",
  it failed:

  ```text
  a look that gave nothing in time is not the answer that no record is there
  ```

`landing/service_replay_awaiting_tests.rs`:

- `the_awaited_reads_of_the_replay_answer_what_the_synchronous_ones_answer`
  answers one tree from one commit and one tree from one private index through
  both faces.
  Planted,
  by starting the awaited face's Git through the synchronous runner,
  it failed:

  ```text
  the commit's tree reads: Message("git rev-parse --verify b12c83a51fee85c44f0d8e3bd5839f735a92e6b6^{tree} could not start: the runtime that runs this command's waits was asked for from inside itself, ...")
  ```

- `a_private_index_that_is_gone_or_gives_no_answer_stops_the_commit`
  requires the "is not there" sentence for an index that is gone through both
  faces,
  and its own sentence for a look that gave no answer.
  No fixture on this host makes a look wait in the kernel,
  so that answer is stated for the job
  (`foundation/the_answer_a_control_states.rs`,
  the mechanism the previous step built for exactly these answers).
  Planted,
  by answering a look that gave nothing in time as an index that is gone,
  it failed:

  ```text
  the sentence says what happened rather than that the index is gone: Failed { code: TransactionFailed, message: "The change this commit captured cannot be read back: the private index of the attempt that captured it, .../captured.index, is not there, so this commit was not made. ..." }
  ```

- `the_reads_answer_inside_a_task_where_the_synchronous_ones_are_refused`
  reads both in a task and refuses the synchronous face there.

`service/wait_ceilings_tests.rs`:

- `a_commits_own_storage_step_waits_as_long_as_a_landings_state_step`
  requires the new ceiling to keep the value the landing's state step has,
  and requires the repair to name it and to read the owed records through it.

One control of this run was wrong and is corrected in the commit that carries it:
`the_awaited_repair_answers_what_the_synchronous_one_answers` first ran the
synchronous face and then the awaited one on the same held record,
which settled the batch,
so the awaited face answered `AlreadyThere`.
Each face now runs over its own repository,
and the two are compared by what each answered and by what each branch names,
never by a commit the two could not share
(the correction the previous step's record describes).

### Checks

- `native:test:host -- --no-run`:
  `Finished` of the `test` profile in 2.71 s after the last code change.
- `native:test:binary:host -- --no-run`:
  see the line this section's commit carries.
- `native:clippy` on every target:
  `Finished` of the `dev` profile in 27.15 s at the tip,
  with warnings denied.
- The modules this run touched,
  each read from its own summary line:
  `owed_index_entries` and `owed_index_repair` and `service_replay::awaiting`
  and `wait_ceilings` together 28 of 28,
  `service_amend_replayed` 9 of 9,
  `held_commit_replay` 2 of 2,
  and `held_commit_settle` 25 of 25.
- The families whose synchronous faces this run refactored under,
  read from their own summary lines:
  `owed_index` 70 of 70 (the repair's own controls over real Git and the
  ceilings),
  and `held_commit` with `service_replay`, `service_amend_replayed`,
  `service_land_follow` and the finish's held-settlement controls 96 of 96.
- One control of this run was wrong and is corrected in the commit that carries
  it:
  `a_record_that_gives_no_answer_holds_no_worker_and_is_not_an_absent_one`
  measured its ceiling against the whole control's time,
  which on a host whose load average was 129 of 8 threads reached 35,327 ms and
  failed though the read itself ended at its ceiling:

  ```text
  the read ended at its ceiling and never waited on the file without end: 35327 ms
  ```

  It now times the read from its own start to its answer,
- `//:lint:monochromatic-lint --max-warnings 0` on the fifteen files this run
  touched,
  after `//:format:monochromatic-lint` applied its whitespace fixes:
  every line-break finding went,
  and five findings stay,
  all of them the one rule about the size of a file:

  ```text
  file has 376 code lines, limit is 300 (blank and comment lines excluded) -- landing/held_commit_settle.rs
  file has 632 code lines, limit is 300 (blank and comment lines excluded) -- landing/owed_index_repair.rs
  file has 329 code lines, limit is 300 (blank and comment lines excluded) -- landing/service_amend_replayed.rs
  file has 339 code lines, limit is 300 (blank and comment lines excluded) -- landing/service_land_follow.rs
  file has 323 code lines, limit is 300 (blank and comment lines excluded) -- landing/service_replay.rs
  ```

  Each of those five files was under the limit before this run
  (298,
  277,
  and their three neighbours likewise),
  so this run's faces pushed them over.
  The lint of the package does not pass at this base either:
  the same run names untouched files,
  `worktree_copying/worktree_copy_owed_copy.rs` at 398 code lines and
  `worktree_copying/worktree_copy_stage_claim_reaping.rs` at 309,
  together with line-break findings in a dozen `worktree_copying` files
  this run never opened.
  Each of the five is split the same way,
  which is what is left of this run's own tidying:
  a sibling `<file>_awaiting.rs` holding the awaited faces,
  the job structures of the pool and the answer readers,
  re-exported from the file itself,
  and a sibling `<file>_requests.rs` holding the builders and readers both
  faces ask through
  (the shape `service_work_drains_awaiting.rs` and
  `service_work_storage_awaited.rs` already have).
- Not run:
  the container gate,
  `native:test:binary:host` beyond its build,
  `native:perf`
  (nothing of this run is in a forwarded command's path,
  and no synchronous face changed behaviour),
  `native:clippy:windows`,
  and any mutation campaign (paused by the owner).

### What is left

- The landing's body (`landing/service_land_batch_body.rs`) and everything it calls,
  then the loop that lands (`serve_in_a_landing`),
  then the commit path,
  in the plan's own order.
  Measured on this branch in `package/git-policy/cli/src/native/landing`,
  with the controls and the awaiting test files excluded,
  the six files of this run hold these synchronous and awaited Git starts,
  one of each per face that keeps the caller it has:
  `owed_index_entries.rs` three and three,
  `owed_index_repair.rs` six and six,
  `service_amend_replayed.rs` four and three,
  `held_commit_replay.rs` four and four,
  `held_commit_settle.rs` two and two,
  and `service_replay.rs` four and two.
- The faces this run did not build,
  each with the owner of what it waits for:
  - the settlement's own face (`one_awaiting`) waits for the finish's awaited
    faces.
    Measured,
    not assumed:
    `service_land_finish::follow_up` starts Git through the synchronous runner,
    so it cannot run on a task;
    and a job of the blocking pool cannot bridge it either,
    because tokio sets the runtime handle on the threads it spawns for the pool,
    so `lazy_runtime::run_on_runtime` answers `Nested` there as well
    (this run's diagnostic of that measurement is quoted in the commit that
    drops the bridge).
    The faces that need it are `service_land_finish.rs` and
    `service_land_follow.rs`,
    which the landing's body step owns.
  - the amend's tree and its landing as a new commit
    (`the_amends_tree_awaiting`,
    `the_amendment_lands_as_new_awaiting`).
    Both pictures are a preparation (`Ready`,
    `NativeInputs`,
    `CommitInvocation`),
    which no test builds by hand;
    the commit path owns those fixtures,
    and its own controls will pin these two faces.
  - the merge chain of `landing/service_replay.rs` (`merge_captured`),
    which waits for the awaited faces of
    `shadow/replay_merge.rs` (`merge_replay_tree`),
    `shadow/replay_shared_paths.rs` (`raw_changes`,
    `list_shared_paths`),
    `shadow/replay_subsumption.rs` (`write_subsumed_base`,
    `subsume_landed_changes`),
    `shadow/replay_subsumption_text.rs` (`subsumed_text_paths` and the three
    steps it is made of),
    `shadow/replay_shadow.rs` (`run_shadow_checked`)
    and `landing/service_replay_capture_order.rs` (`capture_order_of_each`),
    plus the blob batch of `candidate/blob_batch.rs`.
    Each is a mirror of one function over the one request and the one reading,
    which is the work this run did for the rest of the slice.
- The callers that should use these faces once their own step is a task,
  named so nobody has to search:
  `landing/service_land_batch_body.rs` and `landing/service_land_finish.rs`
  settle a held batch,
  so they should call `held_commit_settle::one_awaiting` when it exists and
  `held_commit_replay::onto_awaiting` and `value_now_awaiting` now;
  `landing/owed_index_update.rs` (`exclude_or_stop`) should ask
  `owed_index_repair::exclude_what_a_landed_commit_left_awaiting`;
  the client's replay (`landing/service_commit_requests_and_landing_report.rs`,
  `landing/service_prepare.rs`) should ask
  `service_replay::{retry_of,captured_tree_awaiting,tree_of_commit_awaiting}`
  and the amend's `amended_fields_awaiting`;
  and `landing/service_amend_replayed.rs`'s own two remaining steps should be
  built where their preparation fixtures are.
- The bounds this run did not add stay as the previous section states them:
  `HostResources::read()` and `ClaimTime::read()` run on the task that asks,
  `reap_abandoned_children` is a `waitpid` with no storage behind it,
  and a `spawn_blocking` job that outlasts its ceiling keeps its place of the
  process's bound.
