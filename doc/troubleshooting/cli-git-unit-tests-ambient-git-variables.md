# Eight cli-git unit tests failed whenever `GIT_POLICY_NATIVE_IMAGE_TAG` was exported, which looked like a fixture location problem

`package/git-policy/cli` runs its unit tests with `mise run //package/git-policy/cli:native:test:host`.
Eight tests failed in a run whose fixtures lived on `/var/mnt/encrypted` (a second btrfs file system,
reached through
the `~/temp` link) and passed with the default fixture root on the system disk.

## Symptom

With `CLI_GIT_TEST_ROOT=/var/home/user/temp/agent/svc056/f`,
2780 of 2788 tests passed.
The failing ones were:

- three `lfs_object_plan` tests (`a_push_that_adds_no_pointer_uploads_nothing`,
  `a_push_that_adds_one_pointer_names_that_object`,
  `an_untrusted_destination_scans_the_whole_history`);
- four `repo_read_hook` tests (`known_queries_start_no_process_and_the_switch_restores_it`,
  `a_request_reads_its_private_index`,
  `a_caller_that_names_only_its_git_still_gets_in_process_answers`,
  `batch_tests::batch_requests_equal_git`);
- `repository_facts::queries_are_lazy_and_the_location_is_asked_once`.

Each asserts that the in-process readers answered:
no Git start was counted,
or a plan was narrowed.
Each saw Git started (or the plan left whole) instead.

## Root cause

The location was never the cause.
The runs that failed were started with `--setenv=GIT_POLICY_NATIVE_IMAGE_TAG=svc056`
(the variable `README.md` tells a developer to export so parallel container runs keep their images apart).
`environment_is_plain` in `src/native/repo_read_layout.rs` makes the layout reader decline when the environment Git would
see holds any variable that starts with `GIT_` and is not on its list of names Git reads without moving a path or a key.
`GIT_POLICY_NATIVE_IMAGE_TAG` starts with `GIT_`,
so every in-process answer was declined and Git was started,
which is
correct but is exactly what those eight tests rule out.
The readers take the process environment (`std::env::vars_os`),
a test overlay can only add variables,
and
`std::env::remove_var` is `unsafe` in edition 2024,
so a test cannot clear the variable for itself.

## Evidence

The same filtered run (`lfs_object_plan`,
`repo_read_hook`,
`repository_facts`;
20 tests) on one commit,
with
`--no-fail-fast`:

- Fixture root under `~/temp/agent` (the second btrfs file system):
  20 passed without the variable,
  8 failed with it.
- Fixture root under `~/.cache` (the system disk,
  the `/home` subvolume):
  20 passed without the variable,
  the same 8 failed with it.
- The default fixture root (`target/test-fixtures` in the worktree):
  six of the same tests failed with it before the
  run stopped at its first failure,
  so the result does not depend on any root.
- The earlier failing runs used a transient service (`systemd-run --user --unit`);
  a scope fails the same way.

## Change

`GIT_POLICY_` is a namespace this package owns and Git does not define,
so
`WRAPPER_ENVIRONMENT_PREFIX` in `src/native/repo_read_layout.rs` joins the tolerated prefixes.
A lookalike such as `GIT_POLICYX` still declines,
because Git could define such a name.
The control `the_packages_own_variables_do_not_move_discovery` in `src/native/repo_read_layout_tests.rs` failed before the
change and passes after it.

The container gate was not affected:
`bin/test-native-container.ts` passes `PATH`,
`RUSTC` and `RUSTDOC` to the
container and nothing else,
so the tag never reaches the tests there.

## Still open

Any other variable that starts with `GIT_` and is not on the list (for example a developer's own `GIT_FOO`) still
switches the in-process answers off for the process,
in tests and in production.
That is the intended safe fallback
(Git answers),
and these eight tests assume an environment without such a variable.
