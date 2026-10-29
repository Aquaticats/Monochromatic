# cli-git native landing (slice 7, first half)

Note 2026-10-29: the runner scripts this document names under `package/git-policy` (`cli/bin`,
`cli/native-interop` and `cli.fuzz/bin`) are TypeScript files now. Read every `.mjs` file name here as the same
name ending in `.ts`; the driver `old-format-fixtures` is `old-format-driver`.

## Purpose

This lane ports the landing critical section of the incumbent TypeScript commit path
(`package/git-policy/cli/src/policy-engine/commit-landing.ts`
(moved on 2026-10-13 with the TypeScript wrapper: now `package/git-policy/cli/src/native/commit_landing_loop.rs`),
`commit-landing-index.ts`,
`commit-landing-objects.ts`, `commit-landing-support.ts`, `commit-landing-index-lock.ts`,
`commit-landing-findings.ts`, `commit-transaction-test-phase.ts`) into the native Rust wrapper under
`package/git-policy/cli/src/native/`.
It runs on branch `feat/cli-git-native-landing`, based on `c2f4219acb`
(the transactions branch head with slices 0 to 5, 8 core, hooks, capture, signals, auto-push and worktree-copy).

Paths starting with `src/` are relative to `package/git-policy/cli/`; every other path is repository-relative.

## Scope

In scope:

- the phase markers (`CLI_GIT_TEST_ONLY_PHASE_SIGNAL`), compiled into the release executable;
- one landing attempt without replay: real `index.lock` with its journal record, branch-switch and head-moved checks,
  object migration into a kept pack, the post-index against the then-current real index, `landing-<n>.json`,
  compare-and-swap with the nonce reflog message, `ref-updated.json`, `.keep` removal, owner-preserving install,
  `index-installed`, conclusion cleanup, landed-capture recording;
- the `concurrent-commit` finding for a landing that landed nothing, as a pure function.

Out of scope (owned by other lanes or slices):

- the landing lock and the starvation reservation: the caller takes the landing lock
  (`replay_reservation::acquire_reserved_landing_lock` or `recovery_scan::acquire_registry_lock`)
  and passes it to the attempt as a witness;
- replay and the landing loop's replay branch (slice 8): the attempt returns `HeadMoved` with `replayable`;
- post-landing completion (added-path copies, shadow and transaction removal, maintenance, `post-commit`,
  capture-record pruning), including disposal of the transaction directory;
- slice 6 preparation policies;
- wiring into the commit refusal (`Unported::CommitTransaction` stays).

## Status

Done, gated and committed on this branch.
Head `4fd2aafb59`; the container gate, host Clippy and the per-file mutation campaigns below ran on it
(the mutation images were built from heads that differ from it only in test files).

### Built

- `a3e65b4dea`, `e71143ce51`: `transaction_phase.rs`, the phase markers.
- `2ec23af3fa`: `landing_support.rs` (target comparison, nonce reflog message, compare-and-swap),
  `landing_objects.rs` (shadow objects into a kept pack), `landing_index.rs` (post-index).
- `626382ebca`: `real_index_lock.rs` gains `acquire_landing_index_lock`, `RealIndexLock::install`
  and `RealIndexLock::release_after_landing`.
- `d7f390002c`: lint fixes (implicit return, shadowing) in the modules that the Built section lists.
- `ee925b3955`: `landing_attempt.rs`, `landing_findings.rs`, the acceptance tests
  (`binary_landing_support.rs`, `binary_landing_tests.rs`, `binary_landing_crash_tests.rs`) and five planted controls.
- `613522828c` to `4fd2aafb59`: controls found by mutation (PID file release and creation failures, an amend that
  loses the swap, literal pathspecs, guarded resets, captured-path drops), bounded waits so that a defect fails
  a test instead of hanging it, and a wider bound for the wrapper's recovery status under load.
- `13fe865399`: this document and `package/git-policy/cli/bin/mutate-native-container-noplanted.mjs`
  (see Mutation).

### API for the integrator

`landing_attempt::land_transaction(context, cwd, landing_lock, workspace, request)` runs one attempt.

- `landing_lock: &OwnerLock` is a witness that the caller holds the landing lock; it is not read.
- `workspace: &mut LandingWorkspace` carries the transaction ID, directory, shadow, captured index and commit index.
  Its `preserved` flag is the incumbent's `preserveForRecovery()` / `finishTransaction()`:
  the attempt sets it before writing `landing-<n>.json` and clears it when the compare-and-swap is lost.
  The caller's disposal leaves the directory in place while it is set.
  After `Landed` it stays set until the post-landing step finishes the transaction.
- `request: &LandingRequest` carries the capture, mode, payload (`LandingPayload::Commit` or `NormalizeOnly`),
  committed paths, added and selected-worktree records, the `index.lock` backoff budget and the attempt number.
- The result is `Ok(LandingOutcome)`: `Landed { oid }` (for a normalization the base commit, empty when unborn),
  `HeadMoved { current, replayable }`, or `BranchSwitched { current }`; or `Err(LandingError)`.
- A non-`Landed` outcome becomes the finding through `landing_findings::landing_finding(outcome, capture,
  prepared_oid, replay_plumbing_missing)`, rendered by `policy_events::render_policy_event`,
  and the commit exits with `landing_findings::LANDING_FINDING_EXIT_CODE` (1).
  Nothing renders it yet because the commit path is not wired.
- Lock handling: the real `index.lock` is released on every exit except an install and except a failure after the
  target advanced (or, for a normalization, once its record is durable), where recovery installs through it.

### Tests

Host runs use Git 2.55.0 (`/usr/bin/git`); container runs use Git 2.56.0.

- Unit, host: `real_index_lock` 18, `landing_index` 9, `transaction_phase` 11 (1 ignored child helper),
  `landing_findings` 6, `landing_attempt` 2.
- Binary (`native_binary`, filter `landing`), host:
  26 passed, 1 ignored (the child helper `landing_crash::child_lands`).
  - `landing::*` (17): landed commit and journal bytes, second process's staged path survives, unborn and detached
    targets, conclusion cleanup, branch switch (to a branch and to detached `HEAD`) with the rendered
    `concurrent-commit/branch-switched` line, head-moved with `replayable` for an ordinary commit, amend, merge,
    cherry-pick and revert conclusions, a deleted target and a normalization, normalization landed and on an
    unborn branch, a failed migration, a failed landing record, a foreign lock of a dead owner.
  - `landing_crash::*` (8 plus the helper): a SIGKILL at each of `landing-locked`, `objects-migrated`, `ref-updated`
    and `index-installed` in a child process running the landing API, then the built wrapper's `git status`
    recovers it (exit 0, branch, index tree, status output, nonce once or never in the branch and `HEAD` reflogs,
    no lock, no PID file, no keep, no transaction directory, no shadow, `git fsck --strict` clean);
    a target moved while the child is paused after migration loses the compare-and-swap, for an ordinary commit
    (replayable) and for an amend (not replayable);
    an install failing after the branch advanced leaves the lock for recovery;
    a malformed marker variable fails the landing and releases the lock.
- Container gate (Git 2.56.0), tag `landing`, head `4fd2aafb59`,
  image `59c23cb4060dc9d2efa62f32db6d0d4079b6245b23fa8a9442cf92aa032b5735`:
  1374 library tests passed (5 ignored), 75 binary tests passed (1 ignored), 1 candidate consumer test passed,
  Clippy with `-D warnings` clean.
- Host Clippy (`native:clippy`, all targets, `-D warnings`) clean on the same tree.
- Not run: `native:clippy:windows` (the host lacks the `x86_64-pc-windows-gnu` target).
  The attempt adds no `cfg(not(unix))` code; see the trailing gates.

### Mutation

Each file ran alone through `gate-slot.ts` with `--timeout 600`, after the container gate built the image.
The planted-control phase ran once, with the unmodified `bin/mutate-native-container.mjs`, on head `ee925b3955`
(evidence `package/git-policy/cli/target/verification/native-mutation-Qj2qNr/planted-controls.json`
(scratch output of one run, not kept)):
all 62 controls were noticed, including the five this lane added.
Every later head differs from that one in test files only, which can add failing tests but not remove one,
and the lane's source files are byte-identical; so the per-file runs use
`bin/mutate-native-container-noplanted.mjs`, a copy of the campaign script with phase 1 removed
(each repeat of phase 1 costs about four and a half hours of container runs).
  (Note 2026-10-20: the script `bin/mutate-native-container-noplanted.mjs` was removed; `bin/mutate-native-container.mjs`
  with `NATIVE_MUTATION_SKIP_PLANTED=1` does the same.)
Rerun the unmodified script instead if a source file changes.

Counts are from the final run of each file.

- `landing_support.rs`: 9 mutants, 8 caught, 1 unviable, 0 missed, 0 timeouts.
- `landing_objects.rs`: 2 mutants, 2 caught.
- `landing_findings.rs`: 8 mutants, 7 caught, 1 unviable.
- `landing_index.rs`: 33 mutants, 33 caught.
  The first run caught 24 and missed 9; the survivors were `literal_pathspecs` (two), `changed_since_captured`
  (four), `reset_guarded_paths`, and two inverted filters in `compute_landing_post_index`.
  They are killed by `explicit_paths_are_reset_with_literal_pathspecs_disabled_for_the_magic_prefix`,
  `index_mode_removes_a_captured_path_the_landed_tree_dropped`,
  `index_mode_tolerates_a_captured_index_that_cannot_form_a_tree` and
  `explicit_path_mode_resets_a_guarded_path_that_nobody_restaged`.
- `transaction_phase.rs`: 24 mutants, 23 caught, 1 unviable, 0 missed, 0 timeouts.
  The first run had four timeouts (`==` to `!=` in `parse_phase_signal`, `!` deleted in `await_release`, the
  `reach_with_value` body and its `filter`): the controls waited for a marker that never appeared or for a
  release that never ended.
  They now fail instead of waiting: `await_marker` ends when the child ended, `reach_bounded` and
  `acquire_bounded` fail after a bound, and `pause_waits_for_the_release_file` asserts the pause lasted.
  The second run caught all of them.
- `landing_attempt.rs`: 20 mutants, 15 caught, 4 unviable, 1 missed, 0 timeouts.
  Missed: `landing_attempt.rs:584:48`
  (`==` to `!=` in `detached: capture.layout.symbolic_head == SymbolicHead::Detached`).
  It is equivalent: the flag only adds `--no-deref` to `update-ref`, which changes nothing for the two targets
  that can reach it, a branch ref (not symbolic) or `HEAD` while detached (not symbolic), because
  `land_locked` returns `BranchSwitched` before the swap unless the current `HEAD` equals the captured one,
  and `target_ref` is `HEAD` exactly when the captured one is `Detached`.
  No input can make the flag matter, so no test can observe it; the incumbent passes it as defense in depth.
- `real_index_lock.rs`: 40 mutants, 26 caught, 6 unviable, 7 missed, 1 timeout.
  The seven missed are all inside `#[cfg(not(unix))]` functions that the Linux build does not compile, so the
  mutated source builds the same binary: `same_file_identity` at line 240, `create_exclusive` at lines 278
  (both return values) and 282 (guard `true`, guard `false`, `==` to `!=`), and `created_identity` at line 331.
  A Windows run is a trailing gate.
  The timeout is `create_exclusive` (unix) returning `Ok(false)` at line 255: the acquisition then waits for a
  holder that does not exist, so every test that takes a lock spins and the test binary never ends.
  It is detected, not unverified: `a_free_index_is_locked_without_waiting` bounds the acquisition and fails,
  confirmed by applying the mutant by hand to the source and running that test, which then failed.
  cargo-mutants cannot attribute it because the other tests hang.
  The first run also missed the unix guard `true` at line 260; the new
  `a_lock_that_cannot_be_created_is_a_stop_and_not_a_wait` kills it.

The five planted controls added by this lane (`landing branch-switch check`, `landing head-moved check`,
`landing releases the index lock when it lands nothing`, `landing hands the directory back after a lost
compare-and-swap`, `landing journals the index lock before announcing it`)
were noticed in the Mutation campaign of this lane.

### Git 2.55.0

The host Git is 2.55.0, so every host run above is a 2.55.0 run of the same tests; no test needed a version branch.
A probe with real Git 2.55.0 (a scratch script, not kept) showed that `update-ref -m` of a branch that `HEAD` points
at writes the message once to the branch and `HEAD` reflogs, and that `update-ref --no-deref HEAD` on a detached
`HEAD` writes the `HEAD` reflog, identically for the files and the reftable backends.
The container tests use the files backend only; the reftable behavior of the landing is probed, not tested.

## Choices open to veto

### Landing lock and reservation are the caller's

The attempt takes `&OwnerLock` instead of acquiring the lock, and does no reservation check.
The incumbent takes either `acquireLandingLock` or `acquireReservedLandingLock` inside `landTransaction`.
Slice 8 owns the reservation (`replay_reservation.rs`), and a lock acquired inside the attempt would force it to
reimplement the choice, so the caller chooses and the type witnesses the choice.

### The index lock stays in place after the target advanced

The incumbent's disposal always removes an uninstalled lock.
Recovery installs the recorded post-index through that exact lock and fails closed when it is absent,
so the incumbent turns a failed install after `ref-updated` into an unrecoverable repository.
The attempt keeps the lock from the moment the compare-and-swap succeeded (for a normalization, from the moment
its record is durable) and releases it on every earlier exit.
Cost: a plain Git process waits (or fails on `index.lock`) until the next wrapped command runs recovery.

### Install link name

`RealIndexLock::install` reuses `recovery_files::install_recovered_index`, which links the lock as `install.index`;
the incumbent's landing uses `install-<n>.index` and its recovery uses `install.index`.
`create_owned_file_link` removes an existing link first, so a stale `install.index` from a crash between link and
rename is harmless, and the transaction directory is removed whole afterwards.

### Journal before PID file, and an existing PID file

`acquire_landing_index_lock` creates the lock, writes `index-lock-<n>.json`, syncs the directory, fixes the mode
and only then writes the PID file, the incumbent's order.
An existing `index~pid.lock` is left alone (a debug line), as the incumbent does; `acquire_real_index_lock`
(used by the index-writer coordination) still fails on it and is unchanged.

### Release errors

`release_after_landing` warns for a lock that is gone or another process's, and still removes its own PID file,
as the incumbent's disposal does; any other operating-system error is returned as `LandingError::Release`
unless an earlier error already decided the result.

### Directory mark before the landing record

`preserved` is set before `landing-<n>.json` is written, as in the incumbent.
A failed write therefore leaves the directory marked for recovery, which finds no landing record and discards it.

### Wording kept from the incumbent

The head-moved finding says "an merge conclusion", "an cherry-pick conclusion" and "an revert conclusion",
because the incumbent builds that text from the conclusion name; the native text is identical.

## Trailing gates

Skipped on purpose, as the lane brief allows, and what remains:

- Mixed old-and-new wrapper matrix: the incumbent wrapper recovering a native crash (the plan's "under the incumbent"
  half of the crash test) and the native wrapper recovering an incumbent crash.
  The native side of every phase marker is covered; the incumbent side is not run.
- Windows: `mise run //package/git-policy/cli:native:clippy:windows` needs
  `rustup target add x86_64-pc-windows-gnu`; the attempt adds no `cfg(not(unix))` code,
  and `real_index_lock.rs` reuses the existing ones.
- Hook-point kills ("a kill at each hook point") need the hook lane's commit path and belong to the integration step.
- Fuzz: no target added, because the slice 7 plan names none; the journal record codecs that the landing writes
  (`encode_landing`, `encode_index_lock`, `encode_ref_updated`) are covered by the existing
  `package/git-policy/cli.fuzz/fuzz_targets/transaction_records.rs`.

## Rerun

```sh
# Unit tests of this lane on the host (Git 2.55.0).
mise run //package/git-policy/cli:native:test:host -- landing_ real_index_lock transaction_phase
# Binary acceptance tests on the host.
mise run //package/git-policy/cli:native:test:binary:host -- landing
# Gates, through the host slot limiter.
GIT_POLICY_NATIVE_IMAGE_TAG=landing node "${HOME}/temp/agent/gate-slot.ts" mise run //package/git-policy/cli:native:test:container
node "${HOME}/temp/agent/gate-slot.ts" mise run //package/git-policy/cli:native:clippy
# Mutation of one file after the container gate; the script with the planted controls needs about four and a half hours first.
cd package/git-policy/cli
GIT_POLICY_NATIVE_IMAGE_TAG=landing node "${HOME}/temp/agent/gate-slot.ts" mise run //package/git-policy/cli:native:mutation:scoped -- --timeout 600 --file src/native/landing_attempt.rs
GIT_POLICY_NATIVE_IMAGE_TAG=landing node "${HOME}/temp/agent/gate-slot.ts" env GIT_POLICY_NATIVE_IMAGE_TAG=landing node bin/mutate-native-container-noplanted.mjs --timeout 600 --file src/native/landing_attempt.rs
```

Files to run one at a time: `real_index_lock`, `landing_attempt`, `landing_support`, `landing_objects`,
`landing_index`, `landing_findings`, `transaction_phase`, all under `src/native/` with the `.rs` suffix.
