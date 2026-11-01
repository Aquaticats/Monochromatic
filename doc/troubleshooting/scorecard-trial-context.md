# Scorecard action 2.4.1 requires default-branch context even when result publication is disabled

## Symptom and scope

An isolated manual-dispatch trial cannot assume that `publish_results: false`
makes `ossf/scorecard-action` accept a non-default branch.
The pinned action source emits `only default branch is supported` for that context.
This restriction was found by reading the source before dispatch;
the failing action context was not executed against the live repository.

The trial also must not describe remote Scorecard analysis as a check of its local checkout.
For a non-pull-request event,
the action selects the remote repository and default commit.
Historical package source and live repository metadata are distinct inputs.

## Source trace

Inspected a shallow clone of `ossf/scorecard-action` at tag `v2.4.1`,
commit `f49aabe0b5af0936a0987cfb85d86b75731b0186`.
The [action metadata][action-metadata] selects image `ghcr.io/ossf/scorecard-action:v2.4.1`.

In [`options/options.go:122`][action-options],
`Validate` tests the branch independently of publication:

```go
// options/options.go:122
if !o.isPullRequestEvent() &&
    !o.isDefaultBranch() {
    return errOnlyDefaultBranchSupported
}
```

The error-handling excerpt omits diagnostic printing between the condition and return.
`isDefaultBranch` at line 270 compares `GITHUB_REF` with the repository's default branch.
The same file selects remote input at line 173 and resets the commit at line 197:

```go
// options/options.go:173 and 197, separate statements
if !o.isPullRequestEvent() {
    o.ScorecardOpts.Repo = o.GithubRepository
} else {
    o.ScorecardOpts.Local = "."
}
o.ScorecardOpts.Commit = scopts.DefaultCommit
```

Publication is a later branch in [`main.go:54`][action-main]:

```go
// main.go:54
if os.Getenv(options.EnvInputPublishResults) == "true" && triggerEventName != "pull_request" {
```

Disabling that branch cannot bypass the separate validation condition.
The initial assumption that publication alone governed default-branch support was rejected before launch.

## Trial adaptation and verification

The disposable branch `trial/depot-complete-scorecard-20261016`
at `e5869e93d837922d62be470be974d94c39177619`
keeps the production history-query script unchanged.
It adds manual dispatch,
a separate trial concurrency group,
and a five-minute gate bound.
The reviewed analysis adapter invokes the same upstream image with explicit live-main context,
publication disabled,
and a read-only repository token.
It records the resolved image digest,
main commit before and after analysis,
the result's repository identity,
and emitted check names.
The container bounds are two CPUs,
`2g` memory,
256 processes,
and a twelve-minute execution timeout within a fifteen-minute job.

The action source prints its token in `options/options.go:108`.
The wrapper captures stdout and stderr and replaces the token before persisting or printing either stream.
This records a defensive boundary;
it is not evidence that a token leaked from GitHub's masked logs.

[Run 38841678032][trial-run] verified the current-policy gate.
Job `116782975984` completed successfully after reading 63 historical runs
with one GraphQL request and one point.
Its actual decision was:

```text
# Gate log, 2026-10-16T02:41:10.5881133Z
Skipping Scorecard analysis: 4 runs already executed in the last hour.
```

The analysis job was skipped by that decision.
This is a valid current-policy conditional disposition,
not a successful analysis execution.
The prepared analysis context adaptation remains unexercised.
No override or extra run was used to defeat the budget.
The gate occupied six provider job seconds;
provider run creation to final job completion was nine seconds.
These are single observations,
not a runtime baseline.

## What does not work

- Setting only `publish_results: false` does not remove the source-level default-branch check.
- Checking out historical source does not make the non-pull-request action analyze that checkout.
- Treating the policy-skipped analysis as executed coverage invents evidence.
  The gate's disposition and the analysis execution must remain separate.

## Upstream filing decision

No upstream issue is drafted or sent.
The `.out-of-scope/` search found no Scorecard-specific exemption.
The source establishes an intentional context restriction;
this investigation has not established an upstream defect.

1.   Upstream fault:
     not established.
     The adapter needs explicit default-branch semantics.
2.   Fix feasibility:
     branch selection is configurable in principle,
     but changing its supported contract is outside this trial.
3.   Supported use case:
     the README labels manual dispatch experimental;
     it does not promise arbitrary non-default-branch remote analysis.
4.   Contribution policy:
     not evaluated because no upstream defect is proposed.
5.   Maintainer disposition:
     not evaluated for the same reason.
6.   Upstream prototype:
     none.
     The consumer adaptation is isolated and has not yet executed its conditional analysis path.

## Local task-template collision

The runtime prerequisite campaign reproduced this boundary on `2026-10-18`.
Harness `763b2861d6` embedded Docker's `{{json .Server.Version}}` in its inline Mise task.
Mise rejected the undefined `json` template variable before any cloud allocation.
Harness `15ee06968996212b97111accc2013c2780ed1944` captures plain `docker version` output instead.
The local observation and fresh GitHub,
Depot GitHub Actions and native Depot CI consumers then completed.
Their receipts are recorded in
[the runtime research note](../planning/depot-runtime-orchestration-research.md).
This recurrence has the same nested-template boundary;
it is separate from the platform capabilities those jobs later observed.

An earlier adapter revision passed Docker's `{{json .RepoDigests}}` format string inside a `mise` task.
Local `mise 2026.10.0` failed before Docker ran:

```text
# mise run --skip-tools --dry-run analysis
Failed to render task script
Variable `json` is not defined.
```

Inspected `jdx/mise` source at `ac91d06fa6100935b7a07b08bebf03dd6dd9a0dd`.
`src/task/task_script_parser.rs:99` passes the task text through its template renderer:

```rust
// src/task/task_script_parser.rs:104
render_str(tera, script.trim(), ctx)
```

The verified local fix in `54a823d288` removes Docker's format expression:
read `docker image inspect` JSON and select `RepoDigests` in the task's JavaScript.
This preserves the selected value while avoiding nested template syntax.
The complete task's real `mise` dry run passes.
A disposable fixture also reproduces the failing literal and verifies both plain-JSON and parsed-JSON controls.
No Docker command executes in those template fixtures.
This is a consumer syntax error,
not an upstream `mise` defect;
no upstream change or filing is proposed.
The repository tracks the corrected adapter error in
[issue #1058](https://github.com/Aquaticats/Monochromatic/issues/1058).

### GitHub expressions emitted by a task

The workflow generator encounters the same template boundary.
A literal `${{ always() && steps.cohort_before.outcome == 'success' }}` inside its JavaScript string
is parsed by `mise` before Node can write the destination YAML.
A disposable fixture with local `mise 2026.10.0` prints:

```text
# mise run --skip-tools probe
mise ERROR failed to validate task probe
mise ERROR invalid task script template for task probe
mise ERROR Unexpected character
```

The verified generator constructs the opening delimiters from separate string parts in Node
and joins them only when emitting the destination expression.
The literal fixture exits with status 1 before producing Node output.
The generated fixture exits successfully and prints the exact GitHub expression.
A positive control replacing `always()` with `failure()` changes that output.

The private logger incident-body preparer repeats this syntax boundary with a literal Actions token expression.
`mise` reports this diagnostic before Node executes or any issue is created:

```text
# Private incident-body preparer's mise.toml
Variable `github` is not defined
```

The repaired preparer describes the token context in plain text and successfully renders the issue bodies.
An attempted string-replacement repair also used JavaScript's special dollar-single-quote replacement token,
which expanded the original suffix and corrupted only the private task file.
A separate private `mise` recovery task reconstructs the affected block and retains the failed file's hash.
Literal replacement data must use a replacement callback rather than replacement-string interpolation.
The historical source,
frozen CI harnesses and provider runs were unchanged.
The corrected generator produced the scanner and Scorecard workflows;
parsed comparisons preserve the selected scanner command bodies and Scorecard history script exactly.
This is another consumer syntax boundary covered by `#1058`.

[action-metadata]: https://github.com/ossf/scorecard-action/blob/f49aabe0b5af0936a0987cfb85d86b75731b0186/action.yaml
[action-options]: https://github.com/ossf/scorecard-action/blob/f49aabe0b5af0936a0987cfb85d86b75731b0186/options/options.go
[action-main]: https://github.com/ossf/scorecard-action/blob/f49aabe0b5af0936a0987cfb85d86b75731b0186/main.go
[trial-run]: https://github.com/Aquaticats/Monochromatic/actions/runs/38841678032
