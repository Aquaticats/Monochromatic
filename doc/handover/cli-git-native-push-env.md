# Native push environment fix (evidence and resume notes)

Note 2026-10-29: the runner scripts this document names under `package/git-policy` (`cli/bin`,
`cli/native-interop` and `cli.fuzz/bin`) are TypeScript files now. Read every `.mjs` file name here as the same
name ending in `.ts`; the driver `old-format-fixtures` is `old-format-driver`.

Worktree: `.claude/worktrees/cli-git-lane-push-env`.
Branch: `feat/cli-git-native-push-env`, based on `main` at `7f63ef5ed6`.
Image tag: `push-env`.
Code head when this was written: `116615ad41` (the doc commit that follows it changes no code).
The work is paused part way through the mutation gate; the section "Not done yet" lists what remains.

## The defect

The published `monochromatic-git-policy-cli` 0.1.1 auto-pushed a real commit and printed
`fatal: could not read Username for https://github.com: No such device or address`, then
`commit saved locally, but auto-push to origin failed`.
A manual `git push origin main` through the native wrapper ended with exit 2 and an engine-failure
`content-unavailable` ("Git could not determine push updates:
fatal: could not read Username ...", trigger `manual-push`).
A plain `/usr/bin/git push` from the same shell worked because the global Git configuration names a credential helper
that needs `HOME` and `PATH`.

## Cause

`git_command` in `package/git-policy/cli/src/native/auto_push_single_flight.rs` starts every coordination query,
the manual-push probe and the push with `.env_clear().envs(git.environment)`.
That is the intended contract of `PushGit.environment` (a complete environment), but the two places that built a
`PushGit` passed only the child-environment overlay (lock PID injection and forward-target marker):

- `post_commit` in `commit_command.rs` passed `command.context.overlay`.
- `prepare_manual_push` in `wrapped_command.rs` passed `context.overlay`.

So those Git children had no `HOME`, `PATH`, `SSH_AUTH_SOCK`, `XDG_*` or the user's `GIT_*` variables.

## Incumbent contract

The TypeScript wrapper calls `nanoSpawn(gitPath, args, { cwd })` for the push, the coordination queries, `ls-remote`
and the probe (`src/auto-push.ts`, `src/auto-push-single-flight.ts`, `src/policy-engine/manual-push-probe.ts`,
`src/policy-engine/manual-push-hook.ts`).
`bin.ts` first applies the lock PID overlay to `process.env` (`installGitChildEnvironment`),
and `nano-spawn` 2.1.0 builds the child environment as `{ ...process.env, ...options.env }` (`source/options.js`).
The probe adds only `CAPTURE_DIRECTORY_ENV`.
A search of `src` (outside tests) for `delete process.env` and assignments to `process.env[...]` found nothing,
so the incumbent strips no variable: the child gets the full process environment plus the overlay.
The push never re-enters the wrapper because the child is the absolute `real_git` path and the overlay carries the
forward-target marker; both are unchanged.

## The fix

- `environment_with_overlay(environment, overlay)` in `child_environment.rs`: the caller's variables in order,
  without any name the overlay sets, followed by the overlay.
- `commit_command.rs` and `wrapped_command.rs` build `PushGit.environment` with it from the invocation's environment
  (`prepare_manual_push` now receives `environment`).
- `git_command` and its `env_clear` stay, so every `PushGit` unit test keeps its hermetic environment.
  The docs of `PushGit` say the environment is the complete one.

## Tests

All green on the final code (host and container).

- `push_environment::an_auto_push_after_a_landed_commit_runs_git_with_the_invocation_environment` in
  `src/native/binary_push_environment_tests.rs`, registered in `binary_tests.rs`.
  A repository whose `origin` is an `ext::` transport runs a shim that records `HOME`, `PATH`, a marker variable
  set only in the wrapper's environment, the forward-target marker and `GIT_CONFIG_COUNT`, and refuses the push
  without the marker.
  The control commits through the wrapper and requires exit 0, no "auto-push to origin failed", the commit on the
  bare origin, and every recorded start to carry all five values.
- `push_environment::a_manual_push_probes_and_pushes_with_the_invocation_environment`, same file.
  A manual `git push origin main` through the wrapper; the shim must record exactly receive-pack (probe dry run),
  upload-pack (`ls-remote` confirmation), receive-pack (the push), each with the full environment.
- Three unit tests in `child_environment_tests.rs` for `environment_with_overlay` (order kept and overlay appended,
  overlay replaces every same-named entry, an empty side yields the other side).

Red before the fix (commit `108759bdc0`, host run): both binary controls failed with the shim message
"the push environment lost the marker"; the manual one showed the `content-unavailable` engine failure with exit 2.
Each site is needed on its own: reverting only the `commit_command.rs` site fails only the auto-push control,
and reverting only the `wrapped_command.rs` site fails only the manual-push control.
Logs are under `${HOME}/temp/agent/push-env/` (`red1.log`, `revert-commit-site.log`, `revert-wrapped-site.log`).

## Commits

- `108759bdc0` the two binary controls (red).
- `7f5b938234` the fix, the helper and its unit tests.
- `5d909962e2` two planted controls in `bin/native-planted-controls.mjs`
  ("auto-push Git children drop the invocation environment" and
  "manual-push probe Git children drop the invocation environment").
- `116615ad41` crate version 0.1.2 in `Cargo.toml`, `Cargo.lock` (via `native:lock:members`) and the `README.md`
  install example. The mise pin is untouched.

## Other `env_clear` uses

Production code:

- `auto_push_single_flight.rs` `git_command`: the contract is right, the callers were the defect;
  fixed as the section The fix records.
- `bounded_child.rs` `run_bounded_child`: intentional. Its only caller, `markdown_linter.rs`, passes the invocation's
  whole environment (`std::env::vars_os()` from `main.rs`), so clearing then setting is an explicit copy.
- `hook_dispatch.rs` `run_git_hook`: intentional. `dispatch_environment` rebuilds the caller's whole environment with
  three documented overrides, and `run_executable` receives the whole environment from `main.rs`.

Test and support files (hermetic fixtures by design, no change): `auto_push_concurrency_tests.rs`,
`binary_hook_tests.rs`, `binary_forwarding_tests.rs`, `candidate_consumer_tests.rs`, `binary_support.rs`,
`binary_resolution_tests.rs`, `child_environment_tests.rs`, `child_environment_count_tests.rs`,
`command_test_support.rs`, `manual_push_probe_tests.rs`, `test_support.rs`.

## Gates

On code head `116615ad41`:

- Container gate (`GIT_POLICY_NATIVE_IMAGE_TAG=push-env`, through `gate-slot.ts`): image
  `0e377893f747abfd8499f4545f7354b735f0baa0ce13387b12ac38ab08ae4f1c`.
  Library tests: 1660 passed, 0 failed, 6 ignored.
  Binary tests (`native_binary`): 118 passed, 0 failed, 1 ignored; both new controls ran and passed.
  Clippy in the container finished with no warning or error. Log: `${HOME}/temp/agent/push-env/gate-container.log`.
- Host `native:clippy` through `gate-slot.ts`: finished clean. Log: `clippy.log`.
- Scoped mutation with `NATIVE_MUTATION_SKIP_PLANTED=1` (the planted phase was skipped on purpose):
  `src/native/child_environment.rs`: 50 mutants, 49 caught, 1 unviable, 0 missed, 0 timeouts.
  `src/native/commit_command.rs`: 14 mutants, 10 caught, 4 unviable, 0 missed, 0 timeouts.
  Logs: `mut-child-environment.log`, `mut-commit-command.log`.
- Lint of `bin/native-planted-controls.mjs` (oxlint with `OXLINT_THREADS=2`): three errors, all present before this
  work (`max-lines` for a 631-line file, two `no-useless-escape` at the `hook_dispatch_plan.rs` control).
  The Rust code is not rustfmt-clean in the repository today, so `cargo fmt` was not applied.

## Not done yet

- Scoped mutation of `src/native/wrapped_command.rs`.
- Scoped mutation of `src/native/auto_push_single_flight.rs` (only doc comments changed there; optional).
- The two new planted controls have never been executed. Run them with the planted phase on
  (see the Rerun commands section).
- Nothing was published and nothing was installed on the live PATH.

## Rerun commands

Run from `.claude/worktrees/cli-git-lane-push-env`,
each through the slot limiter (`gate-slot.ts` in `${HOME}/temp/agent`):

```sh
# doc/handover/cli-git-native-push-env.md
GIT_POLICY_NATIVE_IMAGE_TAG=push-env node "${HOME}/temp/agent/gate-slot.ts" mise run //package/git-policy/cli:native:test:container
node "${HOME}/temp/agent/gate-slot.ts" mise run //package/git-policy/cli:native:clippy
NATIVE_MUTATION_SKIP_PLANTED=1 GIT_POLICY_NATIVE_IMAGE_TAG=push-env node "${HOME}/temp/agent/gate-slot.ts" mise run //package/git-policy/cli:native:mutation:scoped -- --file src/native/wrapped_command.rs --timeout 900
GIT_POLICY_NATIVE_IMAGE_TAG=push-env node "${HOME}/temp/agent/gate-slot.ts" mise run //package/git-policy/cli:native:mutation:scoped -- --file src/native/wrapped_command.rs --timeout 900
```

The last command runs the planted phase too; both new planted controls must be reported as noticed.
cargo-mutants exits 3 on a timeout: rerun that file once alone and record the result.
Host iteration: `mise run //package/git-policy/cli:native:test:binary:host -- push_environment` and
`mise run //package/git-policy/cli:native:test:host -- child_environment` (host Git is 2.55.0, the gate's is 2.56.0).
Heavy commands need the CPU hold prefix `systemd-run --user --scope --quiet --collect --property=CPUQuota=200% --property=CPUWeight=10 nice --adjustment=19`
unless they go through `gate-slot.ts`.

## Facts for the orchestrator

- 0.1.2 is released (crates.io and GitHub release `monochromatic-git-policy-cli-v0.1.2`, 2026-10-13); the mise pin
  is still 0.1.1 and is re-pinned by the orchestrator,
  see the resume steps in `doc/handover/cli-git-rust-implementation.md`.
- Git 2.55.0 (host) ran both new binary controls green; no difference from the 2.56.0 container run.
