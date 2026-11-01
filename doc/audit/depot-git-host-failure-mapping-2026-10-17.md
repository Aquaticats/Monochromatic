# Git host native failure mapping

Run `10zmqch10h` executed unchanged source `ef01490f5e960f8ca0b424c12242caeb3c5a2b5e`.
The actual Git build/install/process gate passed.
That gate does not satisfy the separate real-linter prerequisite.
Final coherent dispatch remains held.

## Parent suite outcomes

The binary host suite ended with 136 passed,
one failed and one ignored.
The library host suite ended with 1805 passed,
seven failed and four ignored.
The retained log hashes match the provider artifact evidence.
The final parent failure sections exclude expected negative subprocess controls.

## Missing real-linter prerequisite

Both `markdown::the_real_linter_rewrites_through_add_check_and_fix`
and `markdown_linter::tests::the_real_linter_meets_the_contract` stop at:

`GIT_POLICY_NATIVE_TEST_LINTER is unset`.

The real-linter assertion has not executed in these cases.
The source diagnostic requires building `package/linter/monochromatic-lint`
and naming its executable in that environment variable.
The logger owner is preparing this prerequisite and affected consumer replay.
No authentic source-matching linter ELF was previously verified in the native/TOML-owned routes.

## Distinct non-Markdown boundaries

- `real_index_lock::tests::a_pid_file_that_cannot_be_created_fails_the_acquisition`:
  `real_index_lock_tests.rs:336` expects a diagnostic containing `~pid.lock`.
  The actual value is `Stop("File name too long (os error 36)")`.
- `real_index_lock::tests::a_replaced_lock_is_never_released`:
  `real_index_lock_tests.rs:56` expects refusal;
  release returns `Ok(())`.
- `real_index_lock::tests::an_install_through_a_replaced_lock_is_refused`:
  `real_index_lock_tests.rs:240` expects refusal;
  installation returns `Ok(())`.
- `real_index_lock::tests::releasing_after_a_landing_leaves_a_replaced_lock`:
  `real_index_lock_tests.rs:274` cannot read the replacement after release.
  The actual error is `ENOENT`.
- `transaction_git::tests::input_files_leave_nothing_behind`:
  `transaction_git_tests.rs:190` observes `nlink() == 1`;
  it expects zero.
- `worktree_completion::tests::an_error_that_is_not_a_conflict_fails_the_inspection`:
  `worktree_completion_tests.rs:321` returns `Conflict`;
  it expects a long-name error.

Every named source file is under `package/git-policy/cli/src/native`.
The machine evidence retains immutable source fragments and all actual panic messages.
The replacement-lock cases use related device/inode checks through different release/install paths.
No common cause has been verified.
Generic lock names do not justify merging their observed boundaries.

## Issue comparison

Exact and broader all-state searches found no matching incident.
The evidence retains the focused query responses;
no listed result reached the 100-issue limit.
A separate recent-issue page is explicitly bounded.
Related `#516` concerns a truncated dependency error and malformed-report attribution.
Related `#627` concerns different helper divergences.
Closed `#571` and `#352` are feature acceptance records.
None substitutes for a matching observed failure.
The logger owner retains new-incident and replay ownership.

## Evidence and verification

[Machine evidence](depot-git-host-failure-mapping-2026-10-17.evidence.json)
retains the provider attempt,
source identity,
log digests,
final parent failure blocks,
source fragments and duplicate searches.
This mapping adds no application source changes.
Full TOML retry `23dk3nr90f` remains independently owned and under its existing terminal wait.
