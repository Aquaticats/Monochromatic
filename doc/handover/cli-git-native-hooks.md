# cli-git native hook dispatch and hook lock (lane hooks)

Note 2026-10-29: the runner scripts this document names under `package/git-policy` (`cli/bin`,
`cli/native-interop` and `cli.fuzz/bin`) are TypeScript files now. Read every `.mjs` file name here as the same
name ending in `.ts`; the driver `old-format-fixtures` is `old-format-driver`.

## Purpose

This lane ports slice 4 of `doc/handover/cli-git-native-transactions.md` into the native wrapper:
the dispatch plan (`plan.json`), hook entries, the dispatcher entered by a start name inside a
transaction's `hooks` directory, the hook lock at `<git-common-dir>/cli-git/hook.lock`,
and the preparation lease checks used by nested invocations.
Branch `feat/cli-git-native-hooks`, worktree `.claude/worktrees/cli-git-lane-hooks`.
Paths starting with `src/`, `bin/`, `native-interop/` or `e2e/` are relative to `package/git-policy/cli/`.

## Probe

Question: does Git 2.56.0 start a `core.hooksPath` hook that is a symbolic link with the link's path as its start name?

Script: `${HOME}/temp/agent/hooks-probe/probe.sh`, a shell script that links `hooks/pre-commit` and
`hooks/commit-msg` to a printer script, sets `core.hooksPath`, and commits.
Run as `podman run --rm --init --network=none --memory=2g --cpus=2 --security-opt label=disable`
with image `6ec87f6d290a2f59bda5b3ffd4197058fe0749d02b4978c877e8edf6dc38802a`
(the gate's audited Git 2.56.0 base) and the script mounted read-only.

Output:

- `git version 2.56.0`
- `argv0=/work/hooks/pre-commit args=` (the commit's `pre-commit` hook, through the link)
- `argv0=/work/hooks/commit-msg args=.git/COMMIT_EDITMSG`
- `argv0=/work/hooks/pre-commit args=foo` (direct `git hook run --ignore-missing pre-commit -- foo`)

Conclusion: Git starts the hook with the path of the link (the `hooks` directory joined with the event name)
as argv[0], so the start name is the link path, not the resolved target.
The `Choices` and binary tests rely on this.

Git 2.55.0 (the host's `/usr/bin/git`, run with `PATH=/usr/bin:/bin` so the installed cli-git wrapper stays out)
gives the same three lines, with the probe directory under the scratch directory in place of `/work`:
`git version 2.55.0`, then `argv0=<hooks>/pre-commit args=`, `argv0=<hooks>/commit-msg args=.git/COMMIT_EDITMSG`,
and `argv0=<hooks>/pre-commit args=foo`.
No difference between 2.55.0 and 2.56.0 was found for the start name.
The plain host `git` on PATH is the installed cli-git wrapper, so a probe run through it proves nothing about Git;
the first host attempt was discarded for that reason.

## Built

Modules, all under `package/git-policy/cli/src/native/`:

- `hook_dispatch_plan.rs` (with `hook_dispatch_plan_tests.rs`):
  the `plan.json` record in the incumbent's field order, `quote_config_parameter`, `global_config_overrides`,
  `combine_config_parameters`, `normalized_path` (Node `path` semantics), `hook_lock_directory`,
  `compute_hook_dispatch_plan` (real Git reads through a `GitContext`), `parse_hook_dispatch_plan`,
  `new_preparation_lease`.
- `hook_dispatch_entries.rs` (with `hook_dispatch_entries_tests.rs`):
  `write_hook_dispatch` writes `plan.json` and one entry per preparation event
  (symbolic links on Unix, hard links or copies named `<event>.exe` on Windows).
- `hook_dispatch.rs` (with `hook_dispatch_tests.rs`):
  `dispatch_target_for_start_name`, `dispatch_environment`, `run_hook_dispatch`, `run_executable`,
  and the ignored `interop_probe` test used by `native-interop/hook-dispatch.mjs`.
  Note (2026-10-13): deleted at the user's request and readable at commit `5d61d7b442`:
  The ignored `interop_probe` test and `native-interop/hook-dispatch.mjs`.
  The results are in `doc/handover/cli-git-native-interop.md`.
- `main.rs` passes `argv[0]` to `hook_dispatch::run_executable`, which runs before any wrapper logic.
- `binary_hook_tests.rs`, registered in `binary_tests.rs` as the `hook` module (six controls).

The public API for the capture and preparation lane and the integrator is:
`compute_hook_dispatch_plan(&HookPlanInputs)`, `write_hook_dispatch(hooks, &plan, executable)`,
`new_preparation_lease(transaction_directory)`, `dispatch_target_for_start_name(path)`,
`run_hook_dispatch(&target, args, environment)`, and `run_executable(start_name, args, environment)`.
`transaction_lease::has_valid_preparation_lease` (slice 2) is the validator the nested invocations use.

## Commits

- `36ab6b5004` the start-name probe on Git 2.56.0 and its evidence.
- `92f2482a33` the same probe on Git 2.55.0.
- `379c60bb08` the plan, entries, dispatcher and `main.rs`, with eight planted controls.
- `89e379e805` the binary controls, the interop driver and the `native:test:binary:host` task.
- `d50d53aad5` explicit `return` in the dispatch closures (Clippy).
- `b2895c7b83` explicit `return` in the binary control closures (Clippy).
- `e1e79ae682` the wrong-root control strengthened, and the planted anchors corrected.
- `36832ac1cf` and `c5a0057250` the planted status-clamp registry entry.
  The second commit is empty and has the same subject as the first, because the installed wrapper retried
  after a landing race; it was not amended.

## Gates

Container gate (`GIT_POLICY_NATIVE_IMAGE_TAG=hooks mise run //package/git-policy/cli:native:test:container`)
at `e1e79ae682`, image `localhost/git-policy-native-test:hooks` (Git 2.56.0):
880 library tests passed with 3 ignored, 45 binary-level tests passed (39 existing and six hook controls),
and the candidate consumer passed.
Clippy failed with four findings, none in this lane's files:
`entry_tests.rs:114` (`forward_after` unused),
`index_lock_wait_tests.rs:190` and `:201` (missing `return`),
and `index_writer_coordination_tests.rs:192` (`match` replaceable with `?`).
All four come from `0faec78d77`, the base of this branch.
The gate's own test run is green; the Clippy failure predates this lane.

Image id of the planted-control run: `6fac1b052593960071bf9ac8fb102d07be8fc265b68019fdde6305c1df6c39a2`.

Host runs, Git 2.55.0 (`/usr/bin/git`, host `cargo` through the slot limiter):
the hook unit filter passed 24 with one ignored (the probe), and the six binary hook controls passed.

Windows: `native:clippy:windows` stopped at `src/native/action.rs:60` (`large size difference between variants`),
a finding in a file this lane did not change, so the cfg-gated Windows branches of the entries and the
dispatcher were not reported on.

## Planted controls

Eight planted guard removals were added to `bin/native-planted-controls.mjs` for this lane;
all eight are noticed (the phase ran the full 18-control registry, and every control was noticed):

- hook entry recognized outside a transaction
  (noticed by `hook_dispatch::tests::every_other_start_name_is_not_a_dispatch`)
- disabled events still start Git (noticed by `hook_dispatch::tests::a_disabled_event_succeeds_without_starting_git`)
- hook lock skip flag ignored (noticed by `hook::two_dispatches_overlap_with_concurrent_commits`)
- hook status not propagated (noticed by `hook::a_failing_hook_propagates_its_status`)
- stale config parameters kept (noticed by `hook_dispatch::tests::the_dispatched_environment_carries_the_plan`)
- concurrent commits do not skip the hook lock
  (noticed by `hook_dispatch_plan::tests::plans_read_real_git_configuration`)
- repeated dash c pairing skipped
  (noticed by `hook_dispatch_plan::tests::global_overrides_pair_each_dash_c_with_its_next_token`)
- existing hooks directory reused (noticed by `hook_dispatch_entries::tests::an_existing_hooks_directory_is_refused`)

Two of the first-run controls were not noticed and were fixed rather than accepted:
the wrong-root control needed a plan beside the entry, and the status control changed only the signal path,
so it now clamps the normal exit path.

## Interoperability

`native-interop/hook-dispatch.mjs` is written and syntax-checked, but it has not been run.
Note (2026-10-13): deleted at the user's request and readable at commit `5d61d7b442`:
The driver `native-interop/hook-dispatch.mjs`.
The results are in `doc/handover/cli-git-native-interop.md`.
See `Trailing gates`.

## Fuzz

Slice 4 names no fuzz target, so none was added.

## Choices open to veto

- A relative start name is recognized by the same lexical rule as an absolute one, and the file reads resolve
  against the current directory. Git always starts hooks by their absolute path, so this only affects direct runs.
- The hook lock's parent directory (`<git-common-dir>/cli-git`) must already exist, as the incumbent's
  non-recursive `mkdir` requires. The dispatcher does not create it.
- Dispatcher failures print the incumbent's message text and exit 1, which matches an uncaught error in Node.
- Non-UTF-8 paths and global `-c` values are refused rather than encoded lossily, because the plan is JSON text.
- Only the three preparation events get entries. `post-commit` has no entry, and the dispatcher never recognizes it.
- The writer creates no `dispatch.mjs` or `package.json`, because the native entries are the dispatcher.
- Nested invocations: `recover_for_command` does not consult `has_valid_preparation_lease`.
  Guarded commands still refuse under any inherited lease (`Unported::InheritedLease`), and read-only commands
  never refuse on a live transaction.
  The commit path (slice 9) must consult the validator before lifting that refusal.

## Facts for the integrator

- The four Clippy findings listed under Gates, and the Windows finding in `action.rs`, come from the base branch.
- This lane edited shared files only additively: `lib.rs` (three module entries), `binary_tests.rs`
  (one registration), `mise.toml` (one task `native:test:binary:host`), and `bin/native-planted-controls.mjs`
  (eight registry entries, one corrected).
- Clippy in this repository rejects implicit returns, including in closures, so closure bodies need `return`.
- The commit path must still call `write_hook_dispatch` and `compute_hook_dispatch_plan` with the lease it creates;
  nothing in this lane wires them into `Unported::CommitTransaction`.

## Trailing gates

The user moved these gates after the cutover, so they are not part of this lane's result:

- The mixed-lock interoperability matrix against the incumbent's dispatcher.
  The driver `native-interop/hook-dispatch.mjs` covers four scenarios with positive controls (native holds and
  incumbent waits, the reverse, incumbent nested in native with a tampered-lease control, and native validator
  nested in incumbent with a tampered-lease control). It has not been run.
  Note (2026-10-13): deleted at the user's request and readable at commit `5d61d7b442`:
  The driver `native-interop/hook-dispatch.mjs`.
  The results are in `doc/handover/cli-git-native-interop.md`.
- The cargo-mutants campaign (`native:mutation:scoped`) over `hook_dispatch*.rs`, `hook_dispatch_plan.rs`,
  `hook_dispatch_entries.rs` and `binary_hook_tests.rs`. Not run.
  Its planted phase is the eight controls in the Planted controls section.
- Long fuzz campaigns: none named for slice 4.
