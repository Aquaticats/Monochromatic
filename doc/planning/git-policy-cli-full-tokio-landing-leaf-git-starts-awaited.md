# Step T12, the landing's leaf Git starts are awaited faces, 2026-11-11T17:24:36-05:00

- Branch and worktree:
  `refactor/git-policy-cli-tokio-landing-leaf-git-starts-awaited` in
  `.claude/worktrees/pi-tokio-landing-leaf-git-starts-awaited`,
  from the integration branch `integrate/git-policy-cli-jam-fixes-after-alpha-9` at `3b94069dbf`,
  which is that branch after its twelfth merge pass.
  Commit `4a11a9c7f1` is the change and the commit this record takes its checks from;
  this record is its own commit.
  Not merged into the integration branch or main;
  the coordinator merges the branch.
- What this run was asked to do,
  in the plan's own order:
  the next leaf-first piece of step T12,
  which the section
  "the landing's own state and its side-reference questions are awaited faces" leaves as
  "the landing's body (`landing/service_land_batch_body.rs`) and everything it calls,
  then the loop that lands (`serve_in_a_landing`),
  then the commit path".
  Four runs take that step at the same time,
  each over its own files;
  this one took the eight leaf files of `package/git-policy/cli/src/native/landing`
  that name one or two synchronous Git starts:
  `landed_summary.rs`,
  `service_branch_shows.rs`,
  `service_amend.rs`,
  `service_replay_capture_order.rs`,
  `service_route.rs`,
  `service_land_follow.rs`,
  `service_land_finish.rs` and
  `service_land_git_steps_and_import_stream.rs`.
  Every synchronous face keeps its name,
  its signature and its behaviour,
  because the landing's body,
  the loop and the commit path still call them;
  what this run adds is the awaited face beside each,
  so the slice that makes the landing a task can await the same step.
- The owner's decision this follows is
  "Full on tokio.
  Multi-thread.",
  and the binding bound is
  `doc/decision/cli-git-concurrent-commits.md` lines 220 to 222:
  neither wait is unbounded any more,
  and the bounds are the ones the "Locks" list names,
  quoting the owner's "nothing should ever wait forever or indefinitely".

## What moved

Measured on this branch in `package/git-policy/cli/src/native/landing`,
with the controls excluded,
the folder holds 24 production files that name a synchronous Git start,
and 50 of them:
36 `run_git(` and 14 `run_git_checked(`.
The count is the same before and after this run,
because every synchronous face stays for its callers outside the landing;
what the run adds is 9 calls of the awaited seam
(`run_git_awaiting` and `run_git_checked_awaiting`)
and one awaited face beside each synchronous step that starts Git.

- `landing/landed_summary.rs`:
  `short_name` and its Git fallback had one face.
  `short_name_awaiting` is the awaited one,
  and it reads the object store as a job of the blocking pool
  (`abbreviate`,
  `abbreviation_within`,
  `abbreviated_from_the_files`)
  awaited no longer than `LANDING_STATE_CEILING`,
  then awaits Git's own answer through `asked_of_git_awaiting`.
  Both faces ask one request (`asked_request`),
  read one answer (`asked_name`) and take the setting from one function (`setting_for`),
  so neither can print a name the other would not.
  A reading the ceiling or a refused thread leaves without an answer answers nothing,
  which is what the files answer when they cannot describe the name,
  so the caller asks Git rather than taking the seven characters of the last resort.
- `landing/service_branch_shows.rs`:
  `what_the_branch_shows` and `the_silent_service_after_a_prepare` gained
  `what_the_branch_shows_awaiting` and `the_silent_service_after_a_prepare_awaiting`.
  Both faces ask one question (`branch_shows_request`),
  read it through one function (`shows_answered_by`)
  and word a silence once (`the_silence_means`),
  so "made",
  "not made" and "not known" stay three answers on both.
- `landing/service_amend.rs`:
  `read_amend_source` gained `read_amend_source_awaiting`,
  over `query_awaiting` and one request builder (`query_request`).
  Both faces read Git's answer through `query_answered`,
  so a Git that cannot be started is refused in the same sentence,
  which is the stop the amending command prints.
- `landing/service_replay_capture_order.rs`:
  `capture_order_of_each` gained `capture_order_of_each_awaiting`.
  The plan both faces walk comes from one function (`plan_of`),
  and the answer Git gave it is folded by the unchanged `note_history` through one reading
  (`walk_answered`),
  so neither face can ask about another range than the other or turn an unreadable
  commit name into another failure.
- `landing/service_route.rs`:
  `worktree_git_dir` gained `worktree_git_dir_awaiting`,
  which reads the repository's layout as a job of the blocking pool
  (`layout_within`,
  over `read_layout_in`) awaited no longer than `ROUTE_STATE_CEILING`,
  and asks Git otherwise.
  Both faces ask one question (`git_directory_request`)
  and read one answer (`git_directory_answered`),
  so the pathname Git printed keeps its bytes on both faces.
  A reading that gives no answer is not "outside a repository":
  it falls through to Git,
  and a commit the wrapper could serve is not refused with the words of a repository that is not there.
- `landing/service_land_follow.rs`,
  with its reset family in the file of its own the next bullet names:
  `reset_entries` gained `reset_entries_awaiting`,
  and `derived_names` gained `derived_names_awaiting`.
  Both faces build one request (`request_of`),
  read a run through one function (`run_answered`),
  read a reset through one (`reset_answered`)
  and name the paths a base changed through one (`derived_names_arguments`).
  A list that holds no name resets nothing,
  and the faces answer that arm with `None` rather than with a reset Git never ran.
- `landing/service_land_finish.rs`,
  with its branch questions in the file of its own the next bullet names:
  `decision_for` gained `decision_for_awaiting`,
  over `current_value_awaiting` and `landed_below_awaiting`.
  Each question has one request (`current_value_request`,
  `landed_below_request`)
  and one reading of Git's answer (`value_answered`,
  `below_answered`),
  so "a reference that is not there",
  "a reference Git could not read" and "a Git that could not be started"
  stay three answers for both faces,
  and a batch is not dropped on an answer Git never gave.
- `landing/service_land_git_steps_and_import_stream.rs`,
  with the batch's text in the file of its own the next bullet names:
  `run`,
  `run_words`,
  `checked`,
  `checked_bytes` and `requests_that_land` gained
  `run_awaiting`,
  `run_words_awaiting`,
  `checked_awaiting`,
  `checked_bytes_awaiting` and
  `requests_that_land_awaiting`,
  and the writes of the wrapper's own side references gained
  `write_until_the_reference_names_awaiting`,
  `keep_on_the_scratch_reference_awaiting`,
  `keep_on_a_side_ref_awaiting` and `keep_unlanded_awaiting`.
  One request builder (`words_request`),
  one reading of a run (`run_words_answered`),
  one failure (`failure_of`,
  which keeps a step's ceiling sentence)
  and one reading of a listing (`checked_text`,
  `checked_stdout`) serve both faces.
  The fold that decides which requests of a batch land is one function (`landing_of`) over the paths
  each base changed (`names_of`),
  and the keeplines are worded once per arm,
  so a commit one face keeps is kept at the same reference by the other.
- Three files went over the 300-code-line limit with the new faces
  (`rust/max-lines`),
  so each keeps one responsibility in a sibling module it declares with `#[path]`
  (`AGENTS.md`,
  `MXL`):
  - `landing/service_land_finish_branch_questions.rs` holds the branch questions and their answers,
    re-exported from `service_land_finish.rs` (271 and 57 code lines).
  - `landing/service_land_follow_index_reset.rs` holds `IndexPaths`,
    the derived names and the reset,
    re-exported from `service_land_follow.rs` (174 and 183 code lines).
  - `landing/service_land_batch_import_stream_and_overlaps.rs` holds the `fast-import` stream,
    the marks,
    the quoting and the comparisons of the paths two requests touch,
    re-exported from `service_land_git_steps_and_import_stream.rs` (259 and 120 code lines).
  Every crate path callers use is unchanged,
  because each sibling's names are re-exported where they were declared.
- `service/wait_ceilings.rs` gains `ROUTE_STATE_CEILING`:
  the wait the route decision's walk of the file system is granted,
  `START_CEILING`,
  the value `LANDING_STATE_CEILING` already has for a landing's own state step,
  with the owner's "nothing should ever wait forever or indefinitely" beside it and no measurement,
  because the value bounds a wait rather than serving a workload.

## What pins it

Twenty controls,
in eight `*_awaiting_tests.rs` files beside the faces they pin,
each written before its face was awaited and each shown to fail against a shape it replaces.
A face with no production caller is pinned at two things:
that it answers what the synchronous face answers,
and that it answers it inside the runtime,
where the synchronous face cannot start a bounded child at all
(`lazy_runtime::run_on_runtime` refuses a second entry,
and a step the wrapper ends at its ceiling registers its pipes by entering the runtime).

Every control was first shown failing against the shape the file replaces,
which was the awaited face delegating to the synchronous one:
20 of the 20 failed against that shape or against a later plant of their own,
and the eight that stayed green under the first plant are listed with the plant that moved them.
The reason some stayed green is itself worth keeping:
a step the wrapper never ends (`update-ref`)
runs on a task through the synchronous runner with no runtime entry,
and a command line an in-process reader answers
(`diff-tree` of a batch's paths)
starts no child at all,
so only a bounded child tells the two runners apart.

- `the_awaited_branch_read_answers_what_the_synchronous_one_answers`
  (`landing/service_branch_shows_awaiting_tests.rs`)
  requires one answer about a commit the branch holds,
  one it does not hold and one no object store has,
  from both faces.
  Planted,
  by delegating to the synchronous face,
  it failed:

  ```text
  thread '...the_awaited_branch_read_answers_what_the_synchronous_one_answers' panicked at src/native/landing/service_branch_shows_awaiting_tests.rs:64:9:
  assertion `left == right` failed: a commit the branch holds: the two faces answer one thing about the branch
    left: Unread
   right: Made
  ```

- `the_awaited_silence_reports_what_the_synchronous_one_reports`
  (`landing/service_branch_shows_awaiting_tests.rs`)
  requires the failure a silent service is reported with,
  for the same three states.
  Planted,
  by delegating,
  it failed:

  ```text
  assertion `left == right` failed: a branch that holds the commit: the two faces report one silence the same way
    left: Some(ServiceSilentAfterPreparing { shows: Unread })
   right: None
  ```

- `the_awaited_amend_source_is_what_the_synchronous_one_reads`
  (`landing/service_amend_awaiting_tests.rs`)
  requires the same reading for a merge,
  a commit with a parent and a root commit.
  Planted,
  by delegating,
  it failed:

  ```text
  assertion `left == right` failed: a merge: the two faces read one commit
    left: None
   right: Some(Some(AmendSource { base: "69b628f96531da9fc1eaf75de5e71f69475e925e", parents: ["b0c3c032773992b2137a2877166139633951df88", "84ec7ddcf06e81783bb396ab6343fec389b71094"], parent_tree: Some("20e50a07feffafe7699bf38ff4027a606f406eaa"), author: "Fixture <fixture@example.invalid> 1794431067 -0500", message: "Merge branch 'side'\n" }))
  ```

- `a_git_that_cannot_run_is_refused_in_the_words_of_the_synchronous_face`
  (`landing/service_amend_awaiting_tests.rs`)
  requires one sentence for a Git that cannot be started.
  Planted,
  by delegating,
  it failed:

  ```text
  assertion `left == right` failed: the two faces refuse a Git that cannot be run in one sentence
    left: "Err(Failed { code: TransactionFailed, message: \"reading the commit to amend failed: Custom { kind: Other, error: \\\"the runtime that runs this command's waits was asked for from inside itself, where waiting for it would never end; nothing was run. This is a defect of monochromatic-git-policy-cli: report it with the command that was run\\\" }\" })"
   right: "Err(Failed { code: TransactionFailed, message: \"reading the commit to amend failed: Os { code: 2, kind: NotFound, message: \\\"No such file or directory\\\" }\" })"
  ```

- `the_awaited_name_answers_what_the_synchronous_one_answers_inside_the_runtime`
  (`landing/landed_summary_awaiting_tests.rs`)
  requires one name with the files answering and with Git answering.
  Planted,
  by delegating,
  it failed on the arm Git answers:

  ```text
  assertion `left == right` failed: Git answers: the two faces answer one name for the same commit
    left: "c0f5a76"
   right: "c0f5a76cc1248"
  ```

- `a_reading_of_the_object_store_that_gives_no_answer_asks_git`
  (`landing/landed_summary_awaiting_tests.rs`)
  requires the no-answer arm of the blocking job to ask Git,
  holding the snapshot the files would have answered from at twelve characters while Git's own
  `core.abbrev` says thirteen.
  Planted,
  by answering the last resort when the reading gave nothing in time,
  it failed:

  ```text
  assertion `left == right` failed: a reading of the object store that gave no answer is answered by Git's own name, not by the last resort
    left: "183aeba"
   right: "183aeba7ee430"
  ```

- `the_awaited_walk_decides_what_the_synchronous_one_decides`
  (`landing/service_replay_capture_order_awaiting_tests.rs`)
  requires the four decisions of the walk (prepared,
  landed,
  unordered by an unknown commit,
  unordered for a path history never changed) from both faces.
  Planted,
  by delegating,
  it failed:

  ```text
  the awaited walk decides: Message("git log -m -r --raw -z --no-renames --no-abbrev --format=%H 64cc5b378713155ca4a1a2bab5077e26e430e0bb ^d490d47b1ea3752225bc2e81140d58d2068db870 could not start: the runtime that runs this command's waits was asked for from inside itself, where waiting for it would never end; nothing was run. This is a defect of monochromatic-git-policy-cli: report it with the command that was run")
  ```

- `a_capture_with_no_stamp_is_decided_the_same_way_by_both_faces`
  (`landing/service_replay_capture_order_awaiting_tests.rs`)
  requires the undecided set both faces answer when the order decides nothing.
  It stayed green under the delegate plant,
  because no Git is asked on that arm at all;
  planted with the awaited face answering no path at all,
  it failed:

  ```text
  assertion `left == right` failed: a capture with no stamp of its own is decided the same way by both faces
    left: {}
   right: {[112, 97, 116, 104, 45, 48, 48, 48, 48, 46, 116, 120, 116]: Unordered}
  ```

- `the_awaited_git_directory_is_the_one_the_synchronous_face_reads`
  (`landing/service_route_awaiting_tests.rs`)
  requires one Git directory from the files of a plain worktree and from Git,
  where a global option names the repository and the reader leaves the answer to Git.
  Planted,
  by delegating,
  it failed:

  ```text
  assertion `left == right` failed: the two faces name one Git directory for the worktree they are asked about
    left: None
   right: Some("/var/mnt/encrypted/cargo-build/toolchain/nightly-2026-11-11/8c/4c698255dcf13f/debug/build/monochromatic-git-policy-cli/test-fixtures/process-2806358/native-unit-2806358-route-awaiting-agrees/repo/.git")
  ```

- `a_layout_reading_that_gives_no_answer_asks_git`
  (`landing/service_route_awaiting_tests.rs`)
  requires a layout reading that gave nothing in time to fall through to Git.
  It stayed green under the delegate plant,
  because a plain worktree's layout is answered by the files without a child;
  planted with the awaited face reading the files as the whole answer,
  it failed:

  ```text
  assertion `left == right` failed: a layout reading that gave no answer is answered by Git's own Git directory, not by an absence
    left: None
   right: Some(".../native-unit-2954327-route-awaiting-no-answer/repo/.git")
  ```

- `the_awaited_reset_installs_what_the_synchronous_one_installs`
  (`landing/service_land_follow_awaiting_tests.rs`)
  requires the entries of a commit whose paths include one that is not text,
  installed by both faces.
  Planted,
  by delegating,
  it failed:

  ```text
  assertion `left == right` failed: both faces answer one thing about the entries of a commit
    left: Err(Unavailable)
   right: Ok(())
  ```

- `the_awaited_derived_names_are_the_ones_the_synchronous_face_reads`
  (`landing/service_land_follow_awaiting_tests.rs`)
  requires one list of names for a branch that moved and for a branch that did not exist.
  Planted,
  by delegating,
  it failed:

  ```text
  the names the batch changed: Unavailable
  ```

- `a_refused_reset_is_classified_as_the_synchronous_face_classifies_it`
  (`landing/service_land_follow_awaiting_tests.rs`)
  requires one failure code for a step that could not run.
  It stayed green under the delegate plant,
  because both faces answered `Unavailable`;
  planted with the awaited face answering the index lock's code,
  it failed:

  ```text
  assertion `left == right` failed: both faces classify a step that could not run the same way
    left: Err(IndexInUse)
   right: Err(Unavailable)
  ```

- `the_awaited_decision_is_the_one_the_synchronous_face_makes`
  (`landing/service_land_finish_awaiting_tests.rs`)
  requires one decision for a branch at the batch's commit,
  past it,
  at the old value,
  gone and elsewhere.
  Planted,
  by delegating,
  it failed:

  ```text
  assertion `left == right` failed: both faces finish a batch the branch shows
    left: (Some(Finish), None)
   right: (Some(Finish), Some(Finish))
  ```

- `a_git_that_gives_no_answer_decides_nothing_through_either_face`
  (`landing/service_land_finish_awaiting_tests.rs`)
  requires no decision when Git cannot be asked.
  It stayed green under the delegate plant,
  because both faces answered nothing;
  planted with the awaited face reading no answer as a branch that never moved,
  it failed:

  ```text
  assertion `left == right` failed: a Git that cannot be asked leaves the batch open on both faces
    left: (None, Some(Drop))
   right: (None, None)
  ```

- `the_awaited_steps_run_what_the_synchronous_ones_run`
  (`landing/service_land_git_steps_awaiting_tests.rs`)
  requires one output,
  one text and one listing from both faces.
  Planted,
  by delegating,
  it failed:

  ```text
  the awaited step runs: Unavailable
  ```

- `a_step_git_cannot_run_is_the_answer_both_faces_give`
  (`landing/service_land_git_steps_awaiting_tests.rs`)
  requires one failure code for a step that could not run.
  It stayed green under the delegate plant;
  planted with the awaited face answering a step that was ended at its ceiling,
  it failed:

  ```text
  assertion `left == right` failed: both faces answer a step that could not run the same way
    left: Err(StepTooSlow)
   right: Err(Unavailable)
  ```

- `the_awaited_requests_that_land_are_the_ones_the_synchronous_face_chooses`
  (`landing/service_land_git_steps_awaiting_tests.rs`)
  requires one set of requests for a request whose paths the branch changed since,
  and two of one path where the later capture holds the earlier one's edit.
  It stayed green under the delegate plant,
  because that listing is answered in process without a child;
  planted with the awaited face reading no history,
  it failed:

  ```text
  assertion `left == right` failed: a request whose path the branch changed since is left out, and one that captured later lands
    left: [0, 1, 2]
   right: [1, 2]
  ```

- `the_awaited_side_reference_write_keeps_what_the_synchronous_one_keeps`
  (`landing/service_land_git_steps_awaiting_tests.rs`)
  requires the write of `refs/worktree/cli-git-service/<batch>` to name the batch's commit,
  with the reference removed between the two faces so the awaited write cannot read the one the
  synchronous face made.
  It stayed green under the delegate plant,
  because `update-ref` is a step the wrapper never ends and runs on a task through the synchronous
  runner;
  planted with the awaited face answering the write made and writing nothing,
  it failed:

  ```text
  thread '...the_awaited_side_reference_write_keeps_what_the_synchronous_one_keeps' panicked at src/native/testing/fixture_git_commands.rs:46:5:
  git ["rev-parse", "--verify", "--quiet", "refs/worktree/cli-git-service/1"] failed:
  ```

- `the_awaited_kept_commit_is_the_one_the_synchronous_line_names`
  (`landing/service_land_git_steps_awaiting_tests.rs`)
  requires the line that tells the person where their commit was kept,
  and the reference it names,
  from both faces.
  It stayed green under the delegate plant,
  for the reason the side-reference control stayed green;
  planted with the awaited face wording a line without keeping the commit,
  it failed:

  ```text
  assertion `left == right` failed: both faces tell the person one thing about where their commit is kept
    left: "monochromatic-git-policy-cli: the commit built for you is kept at refs/cli-git/kept/7\n"
   right: "monochromatic-git-policy-cli: the commit built for you is kept at refs/cli-git/kept/7 (see it with `git log refs/cli-git/kept/7`); nothing lands later by itself, and your staged changes and your worktree are as they were. Run the commit again when the program above is done.\n"
  ```

## Checks

- `native:test:host -- --no-run` on the committed tree:
  `Finished` of the `test` profile in 2.27 s,
  the target having been built by the runs below.
- `native:test:binary:host -- --no-run` at the tip:
  `Finished` of the `test` profile in 47.13 s
  (1m 27s the first time,
  on the same tree).
- `native:clippy` on every target,
  warnings denied:
  `Finished` of the `dev` profile in 39.47s.
- The twenty controls of this run,
  one run on the committed tree:
  `20 tests run: 20 passed, 6944 skipped` in 0.540 s.
- The eight modules' own controls at the tip,
  their new ones included,
  filtered to `landing::landed_summary`,
  `landing::service_branch_shows`,
  `landing::service_amend`,
  `landing::service_replay_capture_order`,
  `landing::service_route`,
  `landing::service_land_follow`,
  `landing::service_land_finish` and
  `landing::service_land_git_steps_and_import_stream`:
  `80 tests run: 80 passed, 6884 skipped` in 30.938 s.
- The same eight modules at the tip,
  with the restart's own controls under `landing::service_land::`,
  which exercise `service_land_finish.rs` through `recover`,
  `recover_instance` and the replay of an open batch,
  and with `service::wait_ceilings` where the new ceiling is:
  `307 tests run: 307 passed (2 slow), 6657 skipped` in 151.606 s.
- `source_guard`,
  which holds the step T13 rules this run's awaited faces must pass
  (`blocking_code_off_the_runtime_*`):
  `67 tests run: 67 passed, 6897 skipped` in 5.165 s,
  run after the three files were split and before the last unused-import corrections;
  the runs above are at the tip.
- `//:lint:monochromatic-lint --max-warnings 0` on the twenty files this run touched:
  0 findings,
  `Finished in 23.80s`.
  It first refused one line of `service_branch_shows_awaiting_tests.rs`
  (`markdown/semantic-line-breaks`,
  a break after `"made",`)
  and the three split files for being over 300 code lines,
  and `//:format:monochromatic-lint` made the break;
  the splits are this run's own change,
  not a formatting fix.
- Not run:
  the container gate,
  `native:perf`
  (nothing of this run is in a forwarded command's path,
  and every synchronous face is unchanged),
  `native:clippy:windows`
  (nothing added here is behind `#[cfg(not(unix))]`;
  only `short_name_awaiting` has that arm,
  and it mirrors the synchronous one),
  and any mutation campaign (paused by the owner).

## What is left

- The landing's body (`landing/service_land_batch_body.rs`) and everything it calls,
  then the loop that lands (`service/service_server.rs`,
  `serve_in_a_landing`),
  then the commit path,
  in the plan's own order.
  The callers that should now use this run's awaited faces,
  each in a file another slice owns:
  - `landing/service_land_batch_body.rs`:
    `checked_awaiting`,
    `requests_that_land_awaiting`,
    `run_awaiting` and `run_words_awaiting`
    replace `checked`,
    `requests_that_land`,
    `run` and `run_words`.
  - `landing/service_land_objects_of_batch.rs`:
    `keep_on_the_scratch_reference_awaiting` replaces `keep_on_the_scratch_reference`,
    and `run_awaiting` replaces `run`.
  - `landing/service_land.rs`:
    the awaited face of `keep_what_an_ended_step_built` is still to be written
    (it announces through a `&dyn Announcer` and calls `checked`,
    `keep_unlanded` and `run`);
    `keep_unlanded_awaiting` and `run_awaiting` are what it needs.
  - `landing/service_land_kept_commit.rs`:
    `keep_on_a_side_ref_awaiting` replaces `keep_on_a_side_ref`.
  - `landing/service_commit_requests_and_landing_report.rs`,
    `landing/service_landed_aftermath.rs` and `landing/service_land_hold_a_batch.rs`:
    `short_name_awaiting` replaces `short_name`.
  - `landing/service_commit_looking_for_what_landed.rs` and
    `landing/service_landed_aftermath.rs`:
    `what_the_branch_shows_awaiting` and `the_silent_service_after_a_prepare_awaiting`
    replace their synchronous faces.
  - `landing/service_commit_input_notes_and_embedded_commit.rs`:
    `read_amend_source_awaiting` and `worktree_git_dir_awaiting` replace
    `read_amend_source` and `worktree_git_dir`.
  - `landing/service_replay.rs`:
    `capture_order_of_each_awaiting` replaces `capture_order_of_each`.
  - `landing/service_land_finish.rs` itself,
    when its recovery becomes a task:
    `decision_for_awaiting` replaces `decision_for`,
    and the faces of `follow_up`,
    `settle`,
    `recover_selected`,
    `cleanup_done` and `build_index`
    are still to be written.
- What those faces wait for,
  in files this slice does not own:
  - `landing/service_land_follow.rs`:
    `build_index` needs `PrivateCopy::make`
    (`landing/service_land_copy.rs`) and `refresh_stat_data_when_stale`
    (`commit/commit_index_stat_refresh.rs`) as awaited faces,
    and `cleanup_done` needs `carry_stash` (`conflict/autostash_carry.rs`) and
    `reproduce_conclusion_cleanup` (`commit/conclusion_cleanup.rs`).
  - the same file's `update_index_within` also needs the index lock wait
    (`lock/index_holder_wait.rs`) on the runtime's timer,
    so its pause and its poll leave the worker.
  - `landing/service_land_finish.rs`:
    `follow_up` and `settle` need
    `carry_stages` (`conflict/autostash_carry.rs`) awaited,
    and `service_land_kept_commit.rs` and `held_commit_settle.rs` need their Git awaited.
  - `landing/service_land_git_steps_and_import_stream.rs`:
    `keep_what_an_ended_step_built` needs its announce and its Git awaited in
    `landing/service_land.rs`.
- The synchronous `run_git(` and `run_git_checked(` starts left in `landing/` are 50 in 24 files,
  the same as before this run,
  because every caller of these eight files still calls the synchronous face.
  The next slice is what turns the caller,
  and the count of synchronous starts falls when it does,
  not with this run.
- The blocking work this run moved is the two storage readings
  (the object store's abbreviation at `LANDING_STATE_CEILING`,
  the repository's layout at `ROUTE_STATE_CEILING`);
  a reading that outlasts its ceiling keeps its place of the process's bound,
  as `foundation/blocking_jobs_within_the_budget.rs` holds every place.
  The other awaited faces of this slice do no storage work of their own.
- The source guards still cannot see a synchronous Git start reached through a call,
  so a file whose awaited face called a synchronous helper would pass the scan;
  that is why the controls compare the two faces rather than trusting the scan,
  and why the eight controls that stayed green under the delegate plant were each shown failing
  with a plant of their own.
- This run did not edit `doc/planning/git-policy-cli-full-tokio.md`,
  on purpose:
  four slices append to it at once,
  and the coordinator links the four records from it.
