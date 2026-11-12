# Step T12, the commit's message and the preparation's own Git steps are awaited faces, 2026-11-11T18:47:54-05:00

- Branch and worktree:
  `refactor/git-policy-cli-tokio-landing-message-and-prepare-awaited` in
  `.claude/worktrees/pi-tokio-landing-message-and-prepare-awaited`,
  from the integration `integrate/git-policy-cli-jam-fixes-after-alpha-9` at `3b94069dbf`.
  Commit `a443c41877` is the message and its identities,
  `23ddacc067` is the preparation's two steps,
  `81ab197344` is the two Clippy findings those two files still carried,
  `6a18f86d5c` is the split the line budget asked for,
  `1bc21f1066` is this record,
  and `8b8ac24b96` with `6f691c0b9b` are its own checks at the tip and one doc sentence.
  Not merged into the integration or `main`:
  the coordinator merges this branch.
- What this run was asked to do,
  in the plan's own order after the last "What is left" list
  ("Step T12,
  the landing's own state and its side-reference questions are awaited faces"):
  two of the leaf files that list names as still starting Git synchronously,
  `landing/service_message.rs` and `landing/service_prepare.rs`,
  each given the face a task of the runtime asks and a named bound for every storage wait it does.
  Three other agents took the other slices of this step at the same time,
  so nothing here waits on their files,
  and no file outside this slice's own was changed.

## What moved

- `landing/service_message.rs`,
  the message and the two identities a commit the service builds is made with.
  Each step now has the face a task asks:
  - `build_message_awaiting` reads the `-F` file as a job of the blocking pool,
    awaited no longer than `PREPARATION_READ_CEILING`,
    and awaits every Git child of the step on the runtime
    (`rev-parse` and `cat-file` of a reused commit,
    `log -1 --format=%s` of a fixup,
    `interpret-trailers` of the sign-off and the `--trailer` lines).
    A read the ceiling or a refused thread leaves without an answer is the answer an unreadable file gives,
    which is `Ok(None)`:
    native `git commit` builds the line and the person reads Git's own words,
    so no message waits on a file without end and none is committed empty.
  - `identity_awaiting` awaits the `git var` child of `GIT_AUTHOR_IDENT` or `GIT_COMMITTER_IDENT` on the runtime,
    with the overrides the synchronous face passes.
  - Both faces ask one request and read one answer each,
    so neither can answer where the other fails:
    `identity_request` with `identity_answered`,
    `query` and `query_awaiting` with `text_answered_by`,
    `the_commit_named_by` and `the_raw_commit` with `reused_of` for a reused commit,
    `the_subject_of` and `subject_answered_by` for a fixup,
    `message_of_sources` for the cleaning an empty message and `--cleanup` decide,
    `trailer_lines` with `trailers_request` and `trailed_answered_by` for the trailer interpreter,
    and `read_the_message_file` for the one file read.
    The two bodies therefore differ only in their awaits,
    as the head questions' two faces do (`landing/service_head.rs`).
  - The synchronous face keeps its name,
    its signature and its behaviour:
    `build_message` still takes the invocation and reads the `-F` file on the calling thread,
    which is what a caller that is a thread may do.
- `landing/service_prepare.rs`,
  the preparation of a commit that goes through the service.
  Its two own Git starts are now named steps with an awaited face,
  in `landing/service_prepare_own_git_steps.rs`:
  - `stage_the_unmerged_files` and `stage_the_unmerged_files_awaiting` stage every unmerged file's working copy
    into the commit's own index,
    which is the owner's "Warn and finish the merge" (2026-11-03) taken as `git add` takes it.
  - `base_tree_of` and `base_tree_of_awaiting` read the tree of the branch the commit was prepared on,
    the comparison that decides "nothing to commit" in Git's own words.
  - Both faces ask one request each
    (`unmerged_staging_request` and `base_tree_request`)
    and read Git's answer through the seam's `checked_from`,
    so a refusal keeps Git's words and the status `TransactionStop::Message` gives,
    and a step an empty list leaves no work for starts no child through either face.
  - The preparation's body calls the synchronous faces and nothing else changed there:
    the two steps are extractions,
    so every refusal the preparation ends with keeps its words and its place in the order,
    and no person's commit behaves differently after this run.
- `service/wait_ceilings.rs` names `PREPARATION_READ_CEILING`:
  the wait a preparation's own file read is granted,
  `LANDING_STATE_CEILING`,
  the value `AFTER_COMMIT_STATE_CEILING` and `START_CEILING` already have,
  with the owner's "nothing should ever wait forever or indefinitely" beside it and no measurement,
  because the value bounds a wait rather than serving a workload.
- `landing/mod.rs` declares `service_prepare_own_git_steps`:
  the file the 300-code-line budget asked for once the two steps and their faces sat in
  `service_prepare.rs`,
  which then held 329 code lines and was 29 over that budget.

## What pins it

Eight controls,
each shown failing against the shape it replaces before it passed:

- `a_message_file_that_gives_no_answer_does_not_hold_a_worker`
  (`landing/service_message_awaiting_tests.rs`) stands a pipe where the `-F` file belongs,
  runs the read on a runtime of one worker,
  and requires a task beside it to run and the read to end at its ceiling as an unreadable file does.
  Planted,
  by reading the file on the task as the synchronous face does,
  it failed after `THE_PATIENCE` of 10 s:

  ```text
  thread '...a_message_file_that_gives_no_answer_does_not_hold_a_worker' panicked at src/native/landing/service_message_awaiting_tests.rs:183:5:
  the task beside a message file that gives no answer still runs, so no worker is held by that read
  ```

  and it passes in 0.2 s.
- `the_awaited_message_answers_what_the_synchronous_one_answers`
  compares both faces over ten shapes of command line:
  one `-m` and two,
  a `-F` file,
  a `-C` reuse,
  a `--fixup`,
  an amend that keeps the replaced message,
  a sign-off with a `--trailer`,
  `--cleanup=strip` over a comment line,
  an empty message with `--allow-empty-message`,
  and `--cleanup=verbatim`.
  Planted,
  by asking the reused commit's Git through the synchronous runner
  (`query_awaiting` calling `run_git`),
  it failed:

  ```text
  thread '...the_awaited_message_answers_what_the_synchronous_one_answers' panicked at src/native/landing/service_message_awaiting_tests.rs:235:9:
  assertion `left == right` failed: both faces make one message of ["-C", "HEAD"]
    left: "Ok(Some(Built { message: \"An earlier subject\\n\", reused_author: Some(\"Fixture <fixture@example.invalid> 1794436765 -0500\") }))"
   right: "Err(Failed { code: TransactionFailed, message: \"reading the repository failed: Custom { kind: Other, error: \\\"the runtime that runs this command's waits was asked for from inside itself, where waiting for it would never end; nothing was run. ...\\\" }\" })"
  ```
- `a_message_whose_git_must_be_asked_is_built_inside_a_task`
  builds a `-C` message inside a task and requires the reused commit's own message and author,
  then requires the synchronous face to be refused there.
  Under the same plant it failed:

  ```text
  thread '...a_message_whose_git_must_be_asked_is_built_inside_a_task' panicked at src/native/landing/service_message_awaiting_tests.rs:280:32:
  the awaited face builds the reused message inside a task: Failed { code: TransactionFailed, message: "reading the repository failed: Custom { kind: Other, error: \"the runtime that runs this command's waits was asked for from inside itself, where waiting for it would never end; nothing was run. ...\" }" }
  ```
- `a_message_file_no_look_answered_is_answered_as_an_unreadable_file`
  states the two answers a wait can give without a value
  (`foundation/the_answer_a_control_states.rs`,
  the mechanism the earlier step built for answers no fixture can arrange)
  and requires the message to be `Ok(None)` for both of them,
  so the line goes to native `git commit` and is never committed empty.
  Planted,
  by reading the file on the task,
  it failed:

  ```text
  thread '...a_message_file_no_look_answered_is_answered_as_an_unreadable_file' panicked at src/native/landing/service_message_awaiting_tests.rs:322:9:
  a message file no look answered is the answer an unreadable file gives, so native Git builds the line
  ```
- `the_awaited_identity_answers_what_the_synchronous_one_answers`
  compares both faces over the committer identity,
  the author identity with an `--author` override and that with a date Git does not read,
  and reads one identity inside a task.
  Planted,
  by asking the awaited face for the identity without the overrides the synchronous one passes,
  it failed:

  ```text
  thread '...the_awaited_identity_answers_what_the_synchronous_one_answers' panicked at src/native/landing/service_message_awaiting_tests.rs:355:9:
  assertion `left == right` failed: both faces read one identity for GIT_AUTHOR_IDENT
    left: "Ok(None)"
   right: "Ok(Some(\"Fixture <fixture@example.invalid> 1794441178 -0500\"))"
  ```
- `the_awaited_base_tree_answers_what_the_synchronous_one_answers`
  (`landing/service_prepare_own_git_steps.rs`,
  measured before the split that moved these two controls out of `service_prepare.rs`) requires both faces to read the base's own tree,
  requires a task to be able to ask the awaited one while the synchronous one is refused there,
  and requires a base that names no commit to be refused in Git's words by both.
  Planted,
  by awaiting the synchronous runner in the awaited face,
  it failed:

  ```text
  thread '...the_awaited_base_tree_answers_what_the_synchronous_one_answers' panicked at src/native/landing/service_prepare_awaiting_tests.rs:91:142:
  the base tree: Message("git rev-parse --verify HEAD^{tree} could not start: the runtime that runs this command's waits was asked for from inside itself, where waiting for it would never end; nothing was run. ...")
  ```
- `the_awaited_staging_stages_what_the_synchronous_one_stages`
  starts from a real conflicted index,
  runs each face over its own fresh copy of that index,
  and requires the same staged entries from both,
  one entry per file with no conflict stage left and the work tree's own bytes in it.
  Under the same plant it failed:

  ```text
  thread '...the_awaited_staging_stages_what_the_synchronous_one_stages' panicked at src/native/landing/service_prepare_awaiting_tests.rs:142:12:
  the awaited face stages the unmerged files inside a task: Message("git add --all -- :(top,literal)a.txt could not start: the runtime that runs this command's waits was asked for from inside itself, where waiting for it would never end; nothing was run. ...")
  ```
- `a_commit_with_no_unmerged_file_starts_no_staging`
  hands both faces a Git context whose program does not exist and an empty list of unmerged files,
  so a face that started a child at all would fail instead of answering.
  Planted,
  by taking away the empty-list arm of the awaited face,
  it failed:

  ```text
  thread '...a_commit_with_no_unmerged_file_starts_no_staging' panicked at src/native/landing/service_prepare_awaiting_tests.rs:181:13:
  the awaited face starts nothing for a commit with no unmerged file: Message("git add --all -- could not start: No such file or directory (os error 2)")
  ```

## Checks

- The two test targets build with the task's own flags:
  `native:test:host -- --no-run` answered
  `Finished test profile [unoptimized + debuginfo] target(s) in 1m 09s`,
  and `native:test:binary:host -- --no-run` answered
  `Finished test profile [unoptimized + debuginfo] target(s) in 1m 41s`,
  both in a `systemd-run --user --scope` at `CPUQuota=600%`,
  `MemoryMax=16G` and `TasksMax=4000`.
  At the tip,
  after the split,
  the same two answered
  `Finished test profile [unoptimized + debuginfo] target(s) in 2.64s`
  and
  `Finished test profile [unoptimized + debuginfo] target(s) in 1m 49s`.
- `native:clippy` on every target:
  `Finished dev profile [unoptimized + debuginfo] target(s) in 49.19s` at the tip,
  with warnings denied,
  after it refused two things this run then fixed
  (`clippy::question_mark` in `reused_of`,
  and `clippy::shadow-reuse` in two of the controls).
- The modules this run touched,
  through `native:test:host -- 'landing::service_message::' 'landing::service_prepare' 'service::wait_ceilings'`,
  in a scope at `CPUQuota=600%`,
  `MemoryMax=16G` and `TasksMax=4000`,
  with `CLI_GIT_TEST_ROOT=${HOME}/temp/agent/cli-git-fixtures/root`:
  `32 tests run: 32 passed, 6920 skipped` in 0.381 s,
  and after the split that moved the preparation's steps into their own file,
  `20 tests run: 20 passed, 6932 skipped` in 0.194 s over the same two folders.
- The whole selection this slice lives in,
  `native:test:host -- 'landing::' 'service::' --no-fail-fast`,
  in a scope at `CPUQuota=700%`,
  `MemoryMax=24G` and `TasksMax=4000`:
  `1311 tests run: 1311 passed, 5641 skipped` in 241.770 s.
  Of the three controls another agent owns,
  `landing::service_work::tests::a_retryable_answer_keeps_the_record_for_a_later_round`
  is in this selection and passed here as well as alone
  (`1 test run: 1 passed, 6951 skipped`),
  so this branch no longer carries that failure.
  The other two are binary-target controls
  (`binary::binary_service_stop_ends_its_gits`),
  which this run built and did not run,
  so this run says nothing about them and changed nothing near them.
- `source_guard::blocking_code_off_the_runtime` and `source_guard::hook_self_invocation`,
  the guards step T13 landed,
  because this run adds a storage read inside a step of a task and a new file the self-invocation scan reads:
  `23 tests run: 23 passed, 6929 skipped` in 3.771 s.
- `//:lint:monochromatic-lint --max-warnings 0` over the seven files this run touched:
  `Finished in 42.88s` with no finding,
  after it refused three lines the prose rules asked for
  (`rust/max-lines` on `service_prepare.rs`,
  which the split below it answers,
  and `markdown/semantic-line-breaks` twice).
- Not run:
  the container gate,
  `native:perf`
  (nothing of this run is in a forwarded command's path),
  `native:clippy:windows`,
  and any mutation campaign (paused by the owner).

## What is left

- The awaited preparation itself,
  which is the step this slice's faces wait for.
  `prepare` cannot become a task until every step its body names has an awaited face of its own,
  and the ones that still start Git synchronously are,
  measured on this branch with the controls excluded,
  at least 27 starts across ten modules:
  `landing/owed_index_repair.rs` (6),
  `landing/service_replay.rs` (4),
  `commit/commit_index.rs` (4),
  `landing/owed_index_entries.rs` (3),
  `commit/commit_selection_staging.rs` (3),
  `commit/commit_worktree_records.rs` (2),
  `commit/capture_invocation.rs` (2),
  `landing/service_prepare_changes_config_and_pathspec_steps.rs` (1),
  `commit/private_index_stage.rs` (1) and
  `commit/conclusion_copy.rs` (1),
  and the modules those call in turn.
  The storage steps of the body need their pool jobs at the same time:
  `Scratch::make`,
  `the_pathspec_file`,
  `witness_of`,
  the look at every conclusion state file,
  `deliver_notice` and
  `reach_service_phase`.
  Both awaited faces of this run are marked
  `expect(dead_code, reason = "no production caller yet ...")`,
  so the build refuses a stale expectation once the body calls them.
- The two callers this run's message faces should replace,
  in files other slices of this step own:
  `embedded_commit` in `landing/service_commit_input_notes_and_embedded_commit.rs`
  should call `build_message_awaiting(invocation, &parsed, committer.as_str(), previous).await` where it calls
  `build_message`,
  and the amend path in `landing/service_amend.rs` should call
  `identity_awaiting(invocation.context, invocation.cwd, …)` where it calls `identity`.
  Both keep their signatures,
  so the change is the call and its `.await`.
- The plan's own order after this slice:
  the loop that lands (`service/service_server.rs`,
  `serve_in_a_landing`),
  then the commit path.
- Nothing of this run changed what a person's commit does:
  one thread prepares and lands it,
  the synchronous faces are unchanged,
  and the two new faces have no production caller yet.
- The wrapper printed one note this run did not act on:
  an interrupted removal of another agent's worktree
  (`pi-clippy-passes-after-the-daily-update-of-2026-11-11`)
  left a nested subvolume in its Git directory,
  and the note suggests `git worktree prune`.
  This run touched no worktree but its own,
  so the note stays for whoever owns that worktree.
