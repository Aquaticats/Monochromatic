# cli-git native post-landing lane

## Purpose and scope

This lane ports the post-landing completion half of slice 7 in `doc/handover/cli-git-native-transactions.md`
(the module entry starting `post-landing completion:`)
from the incumbent TypeScript wrapper into the native Rust wrapper.
Branch `feat/cli-git-native-post-landing`, worktree `.claude/worktrees/cli-git-lane-post-landing`,
based on `feat/cli-git-native-transactions` at `c2f4219acb`.

The lane covers:
added-path worktree copies,
shadow and transaction removal,
capture-record pruning,
automatic maintenance,
the `post-commit` hook under the hook lock,
and the orchestration that runs them in the incumbent's order after a landing.

The landing loop itself belongs to the landing lane.
The orchestration takes a `LandedTransaction` value that the integrator fills from landing's outcome.
The commit refusal (`Unported::CommitTransaction`) is not wired by this lane.

Incumbent sources:
`package/git-policy/cli/src/policy-engine/commit-transaction-conclusion.ts`
(moved on 2026-10-13 with the TypeScript wrapper: now `package/git-policy/cli/src/native/commit_conclusion.rs`)
(the order of steps),
`commit-transaction-workspace.ts` (shadow, directory, then pruning on disposal),
`commit-transaction-added-paths.ts` (`installAddedWorktreeFiles`),
`commit-landing-auto-maintenance.ts`,
`commit-landing-post-commit-hook.ts`.

## Status

The three modules are complete, registered in `src/native/lib.rs`, and green on the host and in the container gate.
The scoped mutation campaign over the three files finished with no missed mutant and no timeout.

## Progress

### Modules and commits

`landed_disposal` removes the shadow repository, then the transaction directory, then prunes landed-capture records.
`post_commit_hook` checks the disabled event, reads the author environment, takes the hook lock, and runs
`hook run --ignore-missing post-commit` with inherited streams; the hook's exit status is returned and ignored.
`post_landing` runs added-path and worktree-record copies (from the shadow's `objects`), disposal,
`run_auto_maintenance`, then `post-commit`, and maps failures to `PostLandingError`.

Commits on `feat/cli-git-native-post-landing` after the base `c2f4219acb`:
`65b79bf3b6` evidence doc skeleton,
`136cf6d0fd` the three modules and their tests,
`89d0fc7a0c` planted guard removals,
`8ce41e201c` the first doc record,
`517a6896cb` the failure and ordering controls described in the Mutation section.

### Behavior against the incumbent

The order matches `commit-transaction-conclusion.ts`:
copies of `[...addedPaths, ...worktreeRecords]`, disposal (shadow, directory, prune), maintenance, `post-commit`.
Controls (host test names in `*_tests.rs`):

- A copy failure keeps the shadow, the directory and the capture records, and neither maintenance nor the hook starts.
- Both record lists are copied from objects only the shadow holds, and the shadow goes afterwards.
- A copy changed by the user in the meantime is kept and reported; the commit still completes.
- A shadow removal failure stops before the directory, the records, maintenance and the hook.
- A directory removal failure keeps the landed-capture records (pruning runs last).
- A maintenance non-zero exit, and a maintenance config read that cannot be decoded, never fail the commit;
  the hook still runs.
- A hook that exits non-zero is reported in the outcome and does not fail the commit.
- A hook lock that cannot be taken, and an author lookup Git refuses, fail after disposal and (for the author lookup)
  after maintenance, as the incumbent's thrown error does after its cleanup; the hook never starts.
- The hook waits while another live owner holds the hook lock and then runs under it.
- `PostLandingError` prints its message alone.

### Gates

Host: `landed_disposal`, `post_commit_hook` and `post_landing` tests, 27 in total, pass with the host Git 2.55.0
(`/usr/bin/git`, which `REAL_GIT` in `src/native/test_support.rs` names), so the Git 2.55.0 evidence is the host run.
Clippy (`native:clippy`) exits 0 at `517a6896cb`.
Container gate at `517a6896cb`, image tag `post-landing`, image id
`867e6cbcf2514d6d0fa883fcc863c0798774e8fb19791d166ca534ecc7667250`, Git 2.56.0:
1347 library tests passed (4 ignored), 50 binary tests passed.

### Mutation

Scoped campaign over `src/native/landed_disposal.rs`, `src/native/post_commit_hook.rs` and `src/native/post_landing.rs`
against the container image `867e6cbcf2514d6d0fa883fcc863c0798774e8fb19791d166ca534ecc7667250` (source at `517a6896cb`),
one invocation with `--timeout 1200`, evidence directory `target/verification/native-mutation-jA77aw` (not committed):
all 61 planted guard removals noticed, unmutated baseline passed, 19 mutants tested, 16 caught, 0 missed, 0 timeouts,
3 unviable.

The three unviable mutants replace a function body with `Ok(Default::default())` and do not compile, because
`PostCommitHookOutcome`, `OwnerLock` and `PostLandingOutcome` do not implement `Default`.
Nothing survives, so no survivor needs a justification.

An earlier invocation with the default 300 second timeout noticed every planted control and then stopped at its
unmutated baseline with exit 4: the baseline hit the 300 second limit under a host load average near 90, and
`auto_push::concurrency_tests::a_joined_push_failure_is_reported_by_every_commit_it_covered` failed in that run.
That test belongs to the auto-push lane; it passed in the container gate and in the baseline of the final campaign.
Each invocation replays all planted controls first (hours under heavy host load), so run the three files together.

### Trailing gates

Skipped as the lane brief trails them: the old-and-new wrapper interop matrix, long fuzz campaigns (this lane adds no
fuzz target), Windows checks, and the release-artifact performance gates.
Mutation of maintenance and hook plumbing trails in the brief; the campaign above was run anyway, so nothing remains.

### Rerun commands

Host tests (through the slot limiter):
`node ${HOME}/temp/agent/gate-slot.ts mise run //package/git-policy/cli:native:test:host -- landed_disposal post_commit_hook post_landing`.
Clippy: `node ${HOME}/temp/agent/gate-slot.ts mise run //package/git-policy/cli:native:clippy`.
Container gate:
`GIT_POLICY_NATIVE_IMAGE_TAG=post-landing node ${HOME}/temp/agent/gate-slot.ts mise run //package/git-policy/cli:native:test:container`.
Mutation:
`GIT_POLICY_NATIVE_IMAGE_TAG=post-landing node ${HOME}/temp/agent/gate-slot.ts mise run //package/git-policy/cli:native:mutation:scoped -- --timeout 1200 --file src/native/landed_disposal.rs --file src/native/post_commit_hook.rs --file src/native/post_landing.rs`.
Run `cargo fmt` never on the whole tree: it rewrites about a hundred files outside this lane.

## Choices open to veto

The orchestration does not call `run_post_commit` (the post-commit policy gate and auto-push).
That gate belongs to slice 9, needs the policy configuration the integrator holds, and exits 2 when blocked.
The integrator runs it after `complete_landed_transaction` returns, which keeps the incumbent order.

The post-commit child inherits stdio and is not wrapped in `SignalGuard::run_child`.
The integrator wraps the whole command, as the signal relay contract describes.

After a landing, a copy, disposal or hook failure exits 1 with its message, as the incumbent prints a thrown error.
A maintenance failure is logged and never fails the commit.

A maintenance failure that surfaces as an error (config output that is not UTF-8, or Git failing to start) is also
logged and skipped, although the incumbent's `runAutoMaintenance` throws in those two cases and would fail the command.
The lane keeps the stated rule that maintenance never fails a landed commit; the integrator can tighten it in
`complete_landed_transaction` if the incumbent behavior is wanted.
