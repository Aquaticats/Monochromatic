# Native cli-git signals and automatic maintenance lane

Note 2026-10-29: the runner scripts this document names under `package/git-policy` (`cli/bin`,
`cli/native-interop` and `cli.fuzz/bin`) are TypeScript files now. Read every `.mjs` file name here as the same
name ending in `.ts`; the driver `old-format-fixtures` is `old-format-driver`.

## Purpose and how to respond

This lane ports two parts of the commit path into the native Rust wrapper under `package/git-policy/cli/src/native/`:
signal handling while the wrapper waits for Git (slice 10 of `doc/handover/cli-git-native-transactions.md`),
and automatic maintenance after a landing, with the hk Git-config cleanup found under `src/maintenance/`.

Branch `feat/cli-git-native-signals`, worktree `.claude/worktrees/cli-git-lane-signals`,
based on `0faec78d77`, the committed head of `feat/cli-git-native-transactions`.
Paths starting with `src/` are relative to `package/git-policy/cli/`.

To respond, the integrator merges this branch into `feat/cli-git-native-transactions`,
checks the open facts below, and decides the items under `Choices open to veto`.
Neither module is wired into the commit refusal (`Unported::CommitTransaction` is unchanged).

## Commits

- `20cdf70e3b` `feat(git-policy-cli): port automatic maintenance decision and run to native modules`
- `1595a4ed0c` `feat(git-policy-cli): relay process signals to the waiting Git child and port the hk config cleanup`
- `385ab5b80b` `fix(git-policy-cli): satisfy clippy in the signal guard, hk parser and signal tests`
- `1262241dc7` `test(git-policy-cli): prove the signal sender check and correct the hk prefix control`
- `5d5ca284c1` `fix(git-policy-cli): make the early-signal test's child a direct sleep and add a pre-spawn relay regression test`
  (on `feat/cli-git-native-signals-fix`, not on this lane's branch; see `Early-signal test flake` below)
- `4a525a744c` `test(git-policy-cli): gate the relay statics lock to unix, where the tests that use it are compiled`
  (same branch, follows `5d5ca284c1`)

## What was built

### Signals (`src/native/signal_relay.rs`)

- `SignalGuard::install()` installs handlers for `SIGINT`, `SIGTERM`, `SIGHUP` and `SIGQUIT`
  with `libc::sigaction`, using `SA_SIGINFO | SA_RESTART`.
  It saves the previous dispositions and restores them on drop.
  A signal the parent ignores stays ignored, so Git does not inherit a changed ignore.
- `SignalGuard::run_child(&mut Command)` spawns the child, records its PID, relays any pending signal,
  waits with `waitid(P_PID, WEXITED | WNOWAIT)` (Linux), clears the PID, reaps the child,
  and returns a `ChildOutcome` from `forwarding::outcome_of`.
  The caller maps the outcome with `forwarding::exit_code`, which gives `128 + signal` for a signaled child.
- The handler relays a signal with `kill` only when `si_code` is `SI_USER`, `SI_QUEUE`,
  or `SI_TKILL` on Linux.
  A terminal interrupt carries `SI_KERNEL` and already reached Git through the foreground process group,
  so the handler does not relay it.
- The handler touches only atomics and `kill`, so it is async-signal-safe.
  Relaying from the handler also wakes a blocked wait without an EINTR retry loop.
- The guard does not exit the process.
  The caller releases its locks, removes the transaction, and exits after the guard is dropped.
  Keep the guard alive through cleanup, so a signal during cleanup is caught rather than killing the wrapper.
- One guard may be active per process.

### Automatic maintenance (`src/native/maintenance_decision.rs`, `src/native/maintenance_run.rs`)

- `parse_config_records` reads the NUL-separated output of `git config --type=bool-or-int --null --get-regexp`.
  The last value of a key wins; a key without a newline reads as `true`; output that is not UTF-8 is refused.
- `auto_maintenance_arguments` mirrors `autoMaintenanceArgs`.
  `maintenance.auto` decides when set; otherwise an unset or positive `gc.auto` enables maintenance.
  `maintenance.autodetach`, then `gc.autodetach`, selects `--detach` (the default) or `--no-detach`.
  Values follow JavaScript `Number` semantics: text that is not a number is `NaN`,
  so `gc.auto=true` disables maintenance and `maintenance.auto=maybe` enables it.
- `run_auto_maintenance(context, cwd)` reads the config through `run_git` with the caller's global prefix.
  A read that exits with neither 0 nor 1 logs the warning and returns that status.
  It then runs `git maintenance run --auto --quiet --detach` (or `--no-detach`) and retries once without the flag on
  status 129.
  A status ended by a signal counts as `1`, as `exitCode ?? 1` does.
  The status is returned for the caller to ignore, as native Git does.
- Diagnostics use `diagnostic_log` with the tag `runAutoMaintenance`, as the incumbent's logger prints them.
- Automatic maintenance writes no JSONL event and no on-disk record; its only output is diagnostics on standard error.

### hk Git-config cleanup (`src/native/maintenance_hk_config.rs`)

- `hk_keys_in_scope` pairs the fields of `git config --null --show-scope --name-only --list`.
  It drops only a trailing empty field, reports `MalformedListing` for an unpaired field,
  keeps keys of the requested scope that start with `hook.hk-`, and sorts them by UTF-16 code units.
- `cleanup_hk_git_config` runs `git config --<scope> --unset-all KEY` for each key, in order.

## Tests

- `maintenance_decision_tests.rs`: 14 tests, covering the decision and parser rules, including the `Number` edge cases.
- `maintenance_run_tests.rs`: 10 tests.
  Stand-in Git scripts cover the config read, the 129 retry, a signaled and a failing config read,
  non-UTF-8 output, and the disabled path.
  Real Git covers disabled, enabled, and damaged configurations.
- `maintenance_hk_config_tests.rs`: 12 tests.
  Parser cases; real Git runs for the local and global scopes and for idempotence;
  a damaged configuration; a missing working directory.
- `signal_relay_tests.rs`: 11 tests.
  Handlers are process-wide, so the signal tests re-run the test executable as a helper (`helper_entry`)
  with a scenario in the environment.
  - `handler_marks_a_signal_only_for_a_process_sender`: calls the handler directly with a terminal and a process
    `si_code` and reads the pending slot.
    This is the deterministic control for the sender check.
  - `process_signal_is_relayed_and_the_child_outcome_is_reported`: `SIGTERM` from a process reaches the child;
    the wrapper survives and reports `signaled 15`.
  - `signal_during_start_is_relayed_after_the_child_starts`: a signal sent between installation and spawn is relayed.
    The helper's child is a direct `sleep 30`; see `Early-signal test flake`.
  - `a_signal_pending_before_spawn_is_relayed_when_the_pid_is_published`: the relay path itself, without a helper.
    It sets a pending `SIGTERM`, spawns `sleep 30` through `run_relayed`, and requires the child to end by `SIGTERM`
    within 10 seconds.
  - `terminal_interrupt_is_not_relayed_a_second_time`: the helper runs its child on a pseudo-terminal.
    The test writes `^C` to the master side; the child's trap counts the interrupt, and the count stays at one after
    300 ms.
    A later `SIGTERM` from a process ends the child.
    Standard signals can coalesce, so this test alone cannot prove the absence of a second relay; the handler test
    does.
  - `term_during_a_held_pre_commit_ends_git_and_reports_143`: the slice-10 acceptance case in miniature,
    with real Git and a `pre-commit` hook held by `sleep 5`.
    `SIGTERM` to the helper ends Git; the helper reports `signaled 15`, which `forwarding::exit_code` maps to 143;
    the code after the guard then runs.
  - `a_second_guard_is_refused_until_the_first_is_dropped`, `only_process_senders_are_relayed`,
    `thread_directed_kill_is_a_process_sender_on_linux`, `guard_catches_the_four_signals`.

Test counts by filter, run with `mise run //package/git-policy/cli:native:test:host` on host Git 2.55.0:
the `maintenance` filter passed 36 (decision 14, run 10, hk 12) on `385ab5b80b`;
the `signal_relay` filter passed 9 on `1595a4ed0c` and the new `handler_marks...` test passed on `1262241dc7`;
on `5d5ca284c1` the `signal_relay` filter passed 11, including the new pre-spawn test.

## Early-signal test flake

Branch `feat/cli-git-native-signals-fix`, worktree `.claude/worktrees/cli-git-lane-signals-fix`,
cut from `c2f4219acb`, the head of `feat/cli-git-native-transactions` when the work started.
Fix commit `5d5ca284c1`.

Symptom: `signal_relay::tests::signal_during_start_is_relayed_after_the_child_starts` failed with
`helper did not exit` after the 20 second bound.
It failed on the host in 3 of 6 earlier full-filter runs and in 3 of 20 and 5 of 20 runs of the unfixed code
in this investigation; it passed in the container gate.

Cause: the helper's child was `/bin/sh -c 'touch <dir>/started; exec sleep 30'`.
The relay reached the shell, not the `sleep`.
A shell that waits on a foreground command, here `touch`, absorbs `SIGINT` until that command ends, then `exec`s
`sleep` with the signal already consumed, so the `sleep` runs its full 30 seconds.
The relay lands during the `touch` only when the timing falls there, so the failure is intermittent.
Why the container gate passed was not measured; the timing of the `touch` is the likely reason.
The wrapper's relay itself was correct.

Evidence:
- Trace bytes written by the helper in a failing run read `A B R C S P`:
  the spawn finished, the PID was published, the pending drain found nothing, the handler ran,
  the sender check accepted the signal, and `kill` was sent.
  At the timeout the child showed no pending signal, default dispositions, and the `sleep 30` still running.
  The trace code was temporary and is not in the commit.
- A probe from a Node parent, so that `SIGINT` is not inherited as ignored, sent `SIGINT` 400 ms after start:
  a foreground wait in `/bin/sh` survived and exited 0; `sh -c 'sleep 1; exec sleep 0.3'` survived and exited 0;
  `exec sleep 1` ended by `SIGINT`.

Fix:
- For the early-signal case the helper runs `sleep 30` directly (`helper_command` mode `early`).
- The new test `a_signal_pending_before_spawn_is_relayed_when_the_pid_is_published` covers the path the early test
  meant to cover: it sets `PENDING[1]` to `SIGTERM`, runs `run_relayed` with `sleep 30`, and requires
  `Signaled(15)` within 10 seconds.
- `run_child` now delegates to `run_relayed`, a free function with the same body, so the test can drive the
  spawn path without a `SignalGuard`.
  Behavior is unchanged.
- The in-process tests that touch `PENDING` or `CHILD_PID` take a shared lock, `RELAY_STATICS`, which is compiled
  only on unix with the tests that use it (`4a525a744c`).

Proof:
- The new test is the guard for the relay after publication.
  With that relay removed, the test fails: `the pending SIGTERM reaches the child`, after `finished in 30.04s`.
  The fixed file was restored byte for byte.
- Host: 50 consecutive runs of the `signal_relay` filter, driven by a scratch `.ts` script through the slot
  limiter, all passed (0 failed).
- Host, final source: the `signal_relay` filter passed 11 of 11.
- Container gate: `GIT_POLICY_NATIVE_IMAGE_TAG=signals-fix mise run //package/git-policy/cli:native:test:container`
  through the slot limiter exited 0.
  Its image is `9e96ae6ff3fe464b4fc2e3ffa6b00ff8906d899d25523b30a028f3b698c3d242` on base
  `6ec87f6d290a2f59bda5b3ffd4197058fe0749d02b4978c877e8edf6dc38802a`.
  All test result lines passed; the library run reported 1321 passed and 4 ignored.
  The Clippy step passed in the same run.
  The image was built before a comment-only edit above `run_relayed`; the host run above covers that edit.

Windows: `native:clippy:windows` on `4a525a744c` exits 101 before it reaches the signal files.
It reports six errors in `src/native/auto_push_single_flight.rs` and `src/native/manual_push_probe.rs`, which call
unix-only `std::os::unix` APIs without a `cfg`.
Both files are unchanged from `c2f4219acb`, so the failure is on the base branch and is not this fix.
No reported error names `signal_relay.rs` or `signal_relay_tests.rs`, and the `#[cfg(not(unix))]` guard branch
was not type-checked to a clean result because the crate-level failure stops the run.

Not claimed: the production hand-off was not changed, and no lost-relay race was reproduced in it.
The wrapper signals the child it spawned, so this flake was a property of the test helper's `sh -c` child.

## Gate results

Container test gate, `GIT_POLICY_NATIVE_IMAGE_TAG=signals mise run //package/git-policy/cli:native:test:container`.
At commit `385ab5b80b`, image `fc8fd07481526bc1fc09873e1e72f2994dd38b900ba3ba23a6b0fc1ca3627035`.
At commit `1262241dc7` (the final test change), image
`dbcc6a616f0afdb253c79292dc136bb33adea5cd7fecabbd16de20ce70414bb2`.
Both images use the base `6ec87f6d290a2f59bda5b3ffd4197058fe0749d02b4978c877e8edf6dc38802a` (Git 2.56.0).
The test step passed at both commits.
At `385ab5b80b` it reported 942 tests passed, 2 ignored, 0 failed across five test result lines
(the library run alone was 901 passed and 2 ignored).
At `1262241dc7` the library run reported 902 passed and 2 ignored, including the new handler test,
and the other test result lines were all passing.
The Clippy step failed at `385ab5b80b` on the base-file findings that the Gate results section lists.
The Clippy step at `1262241dc7` was still running when this lane's report was written;
its log is `~/temp/agent/signals-gate-final.log` (the line `container-exit=` ends it).
The only change since `385ab5b80b` is test code in `signal_relay_tests.rs` and two registry entries,
so the same base-file findings are expected there.

Clippy with `-D warnings` on the host (`native:clippy`) and in the container gives the same findings:
- Library tests: `forward_after` is never used (`src/native/entry_tests.rs:114`);
  two `missing return statement` (`src/native/index_lock_wait_tests.rs:190` and `:201`);
  one `this match expression can be replaced with ?` (`src/native/index_writer_coordination_tests.rs:192`).
- Library: one `large size difference between variants` (`src/native/action.rs:60`).
- This lane's files produce no findings on the host or in the container.

Windows GNU Clippy (`native:clippy:windows`, `--lib`) reports only the `src/native/action.rs:60` finding.
The `#[cfg(not(unix))]` branch of the signal guard and the maintenance modules type-check on that target.

## Planted-defect controls

Seven controls were added to `bin/native-planted-controls.mjs`, additively, before the array's closing bracket.
Each one replaces a guard in a source file and names the test that must fail.
A scratch driver applied each replacement on the host, ran the named test with `native:test:host`,
checked for `test <name> ... FAILED`, and restored the file.
All seven are noticed:

- `signal relay sender check` (`if !is_process_sender(code) {` becomes `if false {`):
  noticed by `signal_relay::tests::handler_marks_a_signal_only_for_a_process_sender`.
- `signal relay in the handler` (the handler's `PENDING` store removed):
  noticed by `signal_relay::tests::process_signal_is_relayed_and_the_child_outcome_is_reported`.
- `automatic maintenance detach retry` (the 129 check becomes `if true {`):
  noticed by `maintenance_run::tests::rejected_detach_retries_without_the_flag`.
- `gc.auto positive threshold` (`> 0.0` becomes `!= 0.0`):
  noticed by `maintenance_decision::tests::non_positive_gc_auto_disables`.
- `gc.autodetach fallback` (the fallback removed):
  noticed by `maintenance_decision::tests::gc_autodetach_false_runs_in_the_foreground`.
- `hk listing field pairing` (the odd-field check becomes `if false {`):
  noticed by `maintenance_hk_config::tests::scope_without_key_is_malformed`.
- `hk exact prefix` (`key.starts_with(HK_HOOK_PREFIX)` becomes `key.starts_with("hook.")`):
  noticed by `maintenance_hk_config::tests::prefix_match_is_exact`.

The first run of the driver found two problems that were then fixed in `1262241dc7`:
the sender-check control was not noticed (the old terminal test could not see a double relay),
and the hk prefix control's `from` string was stale after the clippy fix,
so the planted-control harness would have refused to start.

These controls run in the container only as phase 1 of `native:mutation`, which trails the cutover
(see the Trailing gates section).
The host proof above is the evidence that the guard tests fail; the container noticing is not yet run.

## Trailing gates

Per the lane brief's Trailing gates decision, these gates were not run before the cutover:

- Mutation campaigns (`native:mutation`, cargo-mutants) over `src/native/signal_relay.rs`,
  `src/native/maintenance_decision.rs`, `src/native/maintenance_run.rs` and `src/native/maintenance_hk_config.rs`.
  Signals and automatic maintenance are non-core modules.
  Remaining: run the scoped mutation task over these files through the slot limiter and record every survivor with a
  proof.
  The phase-1 planted-control run in the container also belongs to this campaign and remains to be run there.
- Fuzz campaigns: the plan names no fuzz target for these modules, so none was added.
- The old-and-new interop matrix for maintenance.
  The real-Git tests read the same `git config` output the incumbent reads, but no incumbent run was compared against
  the native run.
  Remaining: a one-direction fixture against the incumbent's output and a differential run of the same repository
  state.
- Windows execution: `native:clippy:windows` checks compilation only.
  The signal guard's Windows branch is compiled but not run.
- The release-artifact timing gate.
- The non-UTF-8 path field in events: this lane emits no events, so nothing here changes that trailing item.

## Choices open to veto

- The hk cleanup is ported as a library module with no runnable entry.
  The root task `cleanup:hk-git-config` (in the root `mise.toml`) ran the TypeScript file;
  it was retired on 2026-10-13,
  and the module has no consumer.
  A runnable entry needs a new `[[bin]]` in the native crate or a subcommand, and a root `mise.toml` edit;
  both are shared files outside this lane, so the integrator decides the entry.
- The hk cleanup prints no `info` lines, which the TypeScript prints with `rl.info`.
  The native diagnostic module has only `debug` and `warn`, and the cleanup has no entry to print from.
- Signaled exit mapping: a Git ended by a signal exits `128 + signal`, as the 2026-10-05 user decision says.
  The TypeScript incumbent exits `1` (ledger section `Forwarding, standard streams, signals, and exit codes`);
  the incumbent has no signal handler at all.
- Relay scope: only the Git child is signaled, not its process group.
  A hook that Git started is a grandchild; the wrapper does not signal it, and it keeps running until it exits by
  itself.
- Platform scope: the `SI_KERNEL` classification and the pre-reap wait are verified only on Linux.
  On other Unix systems the wait does not hold the PID until reaping,
  so a signal in the instant between the child's exit and its reaping could reach a recycled PID.
- Guard lifetime: the caller decides when the guard drops.
  A signal that arrives after the child exits and before the drop is not relayed and is not re-raised.

## Facts for the integrator

- Commit path: install the guard before the wait for Git, run the child with `run_child`,
  release the locks and remove the transaction, drop the guard, then exit with `forwarding::exit_code(outcome)`.
- Automatic maintenance: call `run_auto_maintenance` after the landing and before `post-commit`, as the TypeScript
  does.
  Its `Result` error (`MaintenanceError::ConfigNotUtf8` or `Start`) is a decision for the caller:
  the incumbent throws out of the landing there, so the native caller must choose between failing the commit and
  logging.
- `forwarding.rs` changed one line: `outcome_of` is now `pub(crate)` so the guard reuses it.
- `lib.rs` gained four `pub mod` declarations at its end (`maintenance_decision`, `maintenance_run`,
  `maintenance_hk_config`, `signal_relay`), each with a doc comment.
- `bin/native-planted-controls.mjs` gained seven control objects before the array's closing bracket.
- `src/native/maintenance_hk_config.rs` does not run under the native binary;
  the root `cleanup:hk-git-config` task ran TypeScript and was retired on 2026-10-13.
- Clippy: the five findings listed under `Gate results` are in base files this lane did not touch.
  They make the crate-level Clippy gate fail on the base branch; the integrator should fix them there.
- Git versions: the maintenance and signal tests passed against host Git 2.55.0 and, in the container, against Git
  2.56.0.
  No differential comparison of the two versions' output was made.
