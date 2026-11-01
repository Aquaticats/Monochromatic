# Depot CI trial brief

## Purpose and status

Confirmed on 2026-10-15 through the [trial interview](../handover/depot-trial-grilling.md),
including its correction for current GitHub organization ownership.
The user wants more comprehensive CI with less repeated setup,
rebuilding,
and scheduling overhead.
They delegated the technical recommendation and set a USD $200 monthly provider-spending ceiling.
During execution they clarified that saving USD $180 per month matters:
prefer the USD $20 Developer plan if the workload fits without excessive inconvenience.
Evaluate Developer with intentional GitHub routing first,
then Developer with selective overage,
and compare Startup only when its added allowances justify the price.
The USD $200 figure is a ceiling,
not a target subscription cost.
The user wants intentional routing between GitHub and Depot based on measured benefit and budget,
not a complete migration or a fallback used only after exhausting Depot allowances.
They accept deeper Depot dependence when evidence justifies it;
portability is not a hard requirement.
This document records the agreed evidence to collect;
it does not establish vendor adoption or authorize migration.

## What success means

Compare provider charges plus the user's value of elapsed time while relevant CI is unfinished.
The user chose half [Ontario's general minimum wage][ontario-wage]:
CAD $8.975 per hour at the October 1,
2026 rate.
Overlapping jobs count once,
even when other useful work continues.
One dated exchange rate will put provider charges and time value in the same currency.

The user clarified that this is not an apples-to-apples migration.
Equivalent-check comparisons isolate performance effects;
they do not define the migration's value or restrict its coverage target.
Evaluate the expanded workflow's coverage,
reusable environments,
agent-requested experiments,
convenience,
and total cost against the user's budget.
Today's GitHub workload is a reference measurement,
not the target feature set.
Performance comparisons use equivalent checks and the same stream of changes.
Missing tests,
failed setup,
and skipped required checks cannot count as performance improvements.
Early commits,
including broken intermediate states,
remain part of the development workflow.
Agent-requested cloud checks are part of the desired workflow.
The user accepts stopping superseded intermediate checks to prioritize the newest version.
Existing explicit-release handling stays unchanged during the proposed trial.

## Technical coverage target

The proposed coverage target follows changed components and their consumers across supported platforms.
It distinguishes whether code builds,
whether behavior works,
and whether packaged releases actually run.
The coverage inventory also records important behavior for which tests do not exist yet.

Existing checks provide concrete starting points:

- [Android tasks](../../package/music-player/android-app/mise.toml):
  unit tests,
  device tests,
  Android lint,
  Kotlin checks,
  and native Rust checks.
- [Desktop interaction tests](../../package/music-player/desktop-e2e/README.md):
  real keyboard input and observable application behavior.
- [Shared audio-core tasks](../../package/music-player/truepeak-core/mise.toml):
  the core's own tests,
  in addition to tests of applications consuming it.
- [Repository task families](../../mise.toml):
  build,
  code-quality checks,
  unit tests,
  browser tests,
  and end-to-end tests.

The initial evaluation includes music-player components and representative repository checks.
The coverage target spans the repository;
the first workloads are not a limit on that target.
Check frequency remains an engineering recommendation to make from runtime,
failure-detection,
and cost evidence.

## Independently verifiable trial areas

### Reduce repeated setup and compilation

Compare the Alpine x64 release compilation on the same source revision and with equivalent output checks.
The observed GitHub run spent 18 minutes 46 seconds compiling and 28 seconds creating container images.
This makes compilation and dependency reuse a separate measurement from tool installation reuse.

Measure unchanged repetitions before comparing providers,
so ordinary runtime variation is visible.
Package installation is part of the reuse target:
distinguish packages already present in the runner image,
cached download archives,
and a prepared filesystem containing installed dependencies.
Measure provisioning separately from the services that must start in each fresh execution.
The user reiterated this requirement after the desktop runner repeated its apt installation.
They chose to retain it in the trial documents and leave `AGENTS.md` rule `EVL` unchanged.
Then exercise cold state,
warm state,
a source change,
a dependency change,
and a toolchain change.
Record setup,
downloads,
cache restoration,
compilation,
validation,
upload,
and cache-save time separately.
Successful output must pass the existing consumer checks after every relevant change.

### Reduce scheduling overhead

Use the existing [Scorecard budgets](../../.github/workflows/scorecard.yml):
2 analyses per minute,
4 per hour,
and 8 per day.
The present policy skips rejected runs rather than scheduling them for later.
Its counting step was still running more than three minutes after setup in an observed run.

Measure the decision separately from the analysis.
Replay allowed,
rejected,
and overlapping requests against disposable history before any live integration.
Check the selected behavior when a time window expires and when a run is cancelled or fails.
Depot's documented concurrency waiting does not establish support for these rolling budgets.
A native capability or an existing compatible mechanism needs evidence before proposing custom scheduling code.

### Add currently unwired checks

Run existing Android,
desktop-interaction,
and audio-core checks through an isolated CI path.
The resulting inventory links each task to triggers,
required platforms,
outputs,
and a completion result.
A deliberately failing fixture must produce a visible failure,
so a successful empty run cannot masquerade as coverage.
Prefer reusing task definitions locally and through agent-requested cloud execution;
provider-specific checks remain acceptable when measured benefits justify them.

### Evaluate Depot Test

The user explicitly added Depot's Test feature to the execution scope.
Run real repository tests through its reporting integration,
including a deliberate failure and an explicit skip where supported.
Verify reported identities,
counts,
failure details,
result retrieval,
and history or retry behavior against the original runner outputs.
Evaluate timing-based test distribution if the current product supports it.
Record setup effort and test-result usage separately from execution compute.
Reporting a fabricated result is not evidence that a repository test executed.

The [executed Test trial](../audit/depot-test-feature-2026-10-16.md)
verifies those reporting paths and timing-based splitting.
The final split contains 375 unique identities:
374 passing and one existing ignored test.
Repeated same-key uploads duplicate rows,
so exactly-once report submission remains an integration requirement.

### Preserve release and platform behavior

The trial produces disposable validation artifacts without publishing releases or importing release credentials.
Existing Windows and macOS paths remain the reference while Linux execution is evaluated.
[Depot CI][depot-ci] does not offer Windows or macOS sandboxes.
The separate [Depot GitHub runner product][depot-runners] requires an organization-owned repository;
the repository was personally owned when the interview began.
On 2026-10-15,
GitHub reported its move from `Aquaticat/Monochromatic` to `Aquaticats/Monochromatic`,
and a fresh API query confirmed organization ownership.
Both Depot CI and Depot GitHub Actions runners are therefore candidates where their platform support applies.
Cross-platform migration therefore needs its own design and validation before replacing existing jobs.

### Measure the complete monthly cost

Replay a representative change history and record elapsed CI intervals,
machine sizes,
storage,
and billable usage.
The broader coverage target also needs measured costs for checks currently absent from GitHub workflows.
Historical provider minutes alone cannot estimate that future workload.
Compare hybrid allocations of workloads between GitHub and Depot,
including the effect of stopping superseded intermediate checks.
Account for each product's separate allowances and the costs of broader coverage.

[Depot Startup pricing][depot-pricing] has a USD $200 base plus additional usage charges.
Developer has a USD $20 base with 500 container build minutes,
2,000 CI minutes,
2,000 GitHub runner minutes,
25 GB combined cache and registry storage,
and 1,000,000 passing test results.
Both plans publish unlimited concurrency.
The cheaper plan's smaller allowances change placement and retention decisions;
they do not by themselves imply serial execution.
Startup includes 250 GB combined cache/registry storage,
20,000 CI minutes,
20,000 GitHub runner minutes,
5,000 container build minutes,
and 1,000,000 passing test results.
Larger CI machines consume the allowance faster.

Before recurring paid execution,
account plan,
current usage,
and applicable cap semantics
must be inspectable so the proposed monthly allocation can fit the user's ceiling.
Published per-product limits do not yet prove an aggregate billing cap.
The current browser connection could not inspect account settings;
CLI authentication itself is verified.
The documented usage endpoint rejected the available user token because it requires an organization token.
No token was created;
the user subsequently reported the USD $200/month plan,
a seven-day trial,
and no usage so far.
Exact trial expiry and configured spending controls remain unverified.
The user subsequently authorized immediate execution during the unused Startup trial
and clarified that its available resources should be used for the evaluation.
Developer allowances constrain the future recommendation,
not the scope of these trial measurements.
The user subsequently clarified that further account access was deliberately rejected.
Account inspection is excluded from trial completion;
unverified invoice and spending-control claims remain limits on the recommendation.

## Result and adoption boundary

Completion requires an executed complete intended CI graph,
including Oxlint and applicable TypeScript,
formatting,
Rust,
Kotlin,
and Android quality gates alongside builds and tests.
Only checks applicable to affected files or packages and their affected consumers run.
Shared configuration and lockfile changes widen selection according to their demonstrated impact.
Oxlint always uses the selected package's complete configured project context;
the custom rules prohibit changed-file-only Oxlint invocation.
Selection controls must demonstrate both included affected checks and excluded unrelated checks.
The user explicitly corrected the earlier selected-job completion claim.
One immutable source revision must identify every required check,
its outcome,
resource use,
and contribution to the total unfinished-CI interval.
Account access is deliberately excluded;
that exclusion does not waive pipeline measurement.

The [trial report](../audit/depot-trial-2026-10-15.md) distinguishes measured savings,
newly exercised checks,
remaining gaps,
and unverified claims.
It recommends check frequency and an execution arrangement using the user's combined cost objective.
Selected execution experiments and the representative change-stream replay have finished.
Those selected results do not complete the full intended CI measurement.
The resumed batch includes combined prepared desktop execution,
representative browser/native checks,
Depot Tests,
and a controlled standalone linker workload.
The [broader cost replay](../audit/depot-resumed-cost-2026-10-16.md)
charges source-specific snapshot rebuilding and records additional usage sensitivities.
The report explicitly retains unpriced workloads,
storage reconciliation,
and account-control gaps;
it does not establish a complete monthly bill.
Unchanged-input variation and coverage parity must support any claimed improvement.
Adoption,
repository ownership changes,
purchases,
and production workflow changes require a separate action request.

[ontario-wage]: https://www.ontario.ca/document/your-guide-employment-standards-act-0/minimum-wage
[depot-ci]: https://depot.dev/docs/ci/overview
[depot-runners]: https://depot.dev/docs/github-actions/overview
[depot-pricing]: https://depot.dev/pricing
