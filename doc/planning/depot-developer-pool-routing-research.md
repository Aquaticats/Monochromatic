# Depot Developer pool routing research

Research and bounded routing experiments completed on 2026-10-18.
The objective is substantially broader CI coverage using Depot as an additional execution pool,
with GitHub Actions and GitHub-hosted runners remaining heavily used permanently.
Depot-managed GitHub runners and native Depot CI are additional pools.
GitHub's own compute retains ordinary eligible workloads,
not only platform exceptions or emergency fallback.
Staged native promotion applies to selected workflows and does not imply eventual replacement.
This note records repository architecture,
routing mechanisms,
security boundaries,
and rollout evidence for the parent adoption study.
It does not authorize a production workflow or repository-policy change.

## Finding

GitHub Actions can already route an explicitly selected matrix to GitHub and Depot workers,
consume their artifacts,
and preserve a selected worker's failure or cancellation in a stable final gate.
The [routing proof](depot-developer-pool-routing-proof.json) retains terminal runs,
actual runner identities,
source SHAs,
receipt contents,
and verified artifact archive digests.
This establishes a usable additional pool mechanism.
It measures neither application speedup nor comprehensive application coverage.

The immediate architecture should combine workload-specific GitHub runner labels,
explicit experimental pool overrides,
and standalone Depot CI for capabilities such as nested virtualization.
BuildKit is another complementary execution surface.
A new universal scheduler or impact selector is not a prerequisite for useful additional coverage.
The current task inventory and the declared dependency graph contain real constraints that must shape the expansion.

## Measured starting point

At `cf9c9c028bf59c990108ed97a660bf72d64311f3`,
the main checkout has no `.depot/` directory and has 17 GitHub workflow files defining 48 jobs before matrix expansion.
The workflow inventory includes focused fuzzing,
host-contract checks,
native Windows and Alpine consumers,
release pipelines,
CodeQL,
Scorecard,
and Claude execution.
The complete `mise tasks ls --all --hidden --json` inventory contains 2,421 repository tasks;
omitting `--hidden` produces 2,394 and misses part of the graph.
There are 172 `lint:types`,
165 `lint:oxlint`,
19 `lint:clippy`,
130 `test:unit`,
14 `fuzz`,
13 `fuzz:coverage`,
and 153 `build` definitions,
including root aggregators where present.
These are task definitions,
not a claim that all are independent,
currently passing,
safe to invoke with every optional integration,
or appropriate on every platform.
This is the frozen starting inventory.
A later observation at `2026-10-18T05:37:36Z` finds 2,424 tasks after concurrent auto-mode work
adds native source-binding,
review-retirement,
and pool-append capacity checks.
The proof retains that source and task delta separately.

The package inventory contains 180 canonical Node manifests and 785 declared workspace dependency edges,
including development,
peer,
and optional dependencies.
No declared internal package names were unresolved.
Tracked sources include 1,072 TypeScript unit-test files,
8 browser-test files,
294 Kotlin source files,
22 package-root Cargo manifests,
and the additional nested `package/music-player/android-app/rust/Cargo.toml`.
The [proof inventory](depot-developer-pool-routing-proof.json) records the counted paths and graph limitations.

Root task behavior matters when assigning these definitions:

- `vars.fanout_packages` uses bounded package workers;
  watcher fanout deliberately differs.
  Direct-child `vars.fanout` can start its children together.
  A runner size and matrix count must account for nested task parallelism.
- The unit-file worker pool is bounded by available parallelism.
  Ordinary unit execution excludes `*.expensive.*` unless explicitly requested.
  Excluded fixtures include VM lifecycle,
  image-diff,
  TypeScript filter,
  TUI,
  and billed Hetzner provisioning tests.
- Root `prepare` installs Android and browser prerequisites even for jobs that do not need them.
  Expanded CI should install tools for the selected profile.
  Root bootstrap also contains a bare `mise upgrade` and is not a suitable immutable CI bootstrap.
- Root `validate` includes mutating format work.
  Generated-file and format checks need disposable checkout state and a resulting-diff assertion.

An explicit-path search covered all 197 unique Mise task source files and all 17 workflow files.
It found the Mac `hdiutil` workaround,
the root package-start-task TODO,
and the root upstream-workaround section containing `fix:reinstall`.
That repair task removes `node_modules` and reinstalls dependencies;
it is not an immutable CI installation profile.
The search included suppression markers and retained source hashes in the proof.
Its scope is task and CI definitions,
not a whole-codebase suppression or health verdict.

Repository-only GitHub API reads on 2026-10-18 found an empty required-status-check list on `main`,
an active Copilot review ruleset,
SHA-pinned allowed actions,
and read-only default workflow tokens.
The legacy protection object has `strict: true` but empty `contexts` and `checks` arrays.
The active ruleset supplies Copilot review,
not required CI checks or a merge queue.
Selected action patterns include `actions/*`,
`docker/*`,
`jdx/*`,
and `changesets/*`,
with GitHub-owned and verified actions allowed and full action SHA pinning required.
No account-wide settings or secret contents were read or changed.
The checked-in historical [CI provider decision](../decision/ci-provider.md) predates the native trial;
its old platform inventory must not be treated as current capability evidence.
An eventual accepted adoption should reconcile that document in place.

## Observed repository arrivals

The [arrival dataset](depot-developer-pool-arrivals.json) covers
`2026-10-11T05:13:21Z` through `2026-10-18T05:13:21Z`.
It retains 6,917 repository run records with exact head SHAs.
Enumeration made 86 API requests and split the window into 11 disjoint intervals,
each reporting fewer than 1,000 results.
This avoids GitHub's documented [filtered-run result cap][github-run-api].
Query identities,
response hashes,
interval counts,
classification rules,
and individual run records are retained.

- Automatic default-branch workflows:
  5,202 arrivals,
  2,526 distinct head SHAs,
  2,538 distinct event and head-SHA pairs.
  The largest observed hour contains 182 arrivals and the largest minute contains 10.
- PR workflows:
  24 arrivals across 6 head SHAs;
  no `merge_group` arrivals in this window.
- Explicit trial or research markers:
  77 arrivals,
  including workflows reused on trial branches.
- Repository agent events:
  1,526 Claude issue or issue-comment arrivals,
  kept separate from validation demand.
- Other categories:
  36 development-branch automatic arrivals,
  7 manual default-branch arrivals,
  and 45 other manual arrivals.

The automatic-main mix is dominated by Scorecard with 2,538 arrivals
and forbidden-strings with 2,537.
These are real existing workflow arrivals even when a triggering commit message discusses research.
Commit messages and issue titles do not classify a run as an experiment.
Branch and workflow identity markers do;
that heuristic does not establish complete intent classification.

An arrival is not proof that a job executed.
Job conditions may skip work,
particularly on repository agent events.
The API records the latest `run_attempt`,
not separate records for every retry.
This study does not infer billed minutes,
sum overlapping workflow waits,
or extrapolate the desired 2,421-task program from the runtimes of the existing focused workflows.
The arrival rate makes affected scope,
coalescing,
cadence,
and bounded fanout material design choices.

## Affected consumers and existing owners

Changed-package-only routing is insufficient.
Declared reverse reachability,
including the changed package,
selects 157 packages for a logger change,
157 for module-test,
167 for config-typescript,
and 5 for TOML.
The same analysis gives config-oxlint no package consumers,
yet root `oxlint.config.ts:17` imports it and root `package.json:31` declares it.
Workspace configuration influence therefore needs explicit handling in addition to package edges.

Existing ownership should be extended before introducing an independent graph engine:

- `package/config/pnpr/src/registry-read.ts` owns workspace package discovery.
  `publish-plan.ts` owns publication ordering.
  `publish-missing-versions.ts:36` explicitly excludes development edges from publication order
  because those edges create cycles.
  Publication order cannot serve as the complete CI consumer graph.
- Installed Mise 2026.10.0 already exposes `--affected`,
  explicit base and head revisions,
  `--affected-json`,
  and `--affected-explain`.
  Its [workspace graph documentation][mise-graph] covers reverse impact,
  global inputs,
  lockfile attribution,
  and project overrides.
- A disposable positive control verified that Mise selects a changed producer and its development consumer,
  excludes an unrelated package,
  executes nothing for `--affected-json`,
  executes the selected task set for an actual run,
  and rejects a missing Git revision.
- In this repository,
  both `mise tasks graph --json` and `mise run --affected --affected-json` fail on the declared
  `config-rolldown -> module-test -> config-rolldown` cycle.
  The [cycle investigation](../troubleshooting/mise-affected-workspace-cycles.md) traces the validator.
  Removing real development edges would lose impact evidence.
- Incumbent pnpm 12.14.0 successfully lists the logger reverse closure with
  `pnpm --recursive --filter '...@monochromatic-dev/logger-core' --include-workspace-root list --depth -1 --json`.
  The selected set exactly matches all 157 declared affected packages plus repository root.
  This verifies Node metadata reachability through the actual cyclic workspace;
  it does not verify a complete cross-language changed-file selector.

The initial expansion can run bounded,
explicit validation profiles without an affected-only optimization.
Affected filtering should become authoritative only after positive controls cover shared configuration,
deletions,
renames,
lockfile changes,
generated inputs,
Cargo paths,
and nested Gradle inputs.
Use immutable event base and head SHAs,
retain the selected workload manifest and reasons,
and fail a selector error or conservatively execute its full bounded profile.
Never turn a failed diff or missing revision into an empty successful matrix.

Do not add a root Cargo workspace merely to enable provider discovery:
the repository's [Cargo ownership policy](../../.out-of-scope/cargo-workspace.md) preserves package-local profiles.
Mise's Cargo provider discovers a root Cargo workspace;
that is not the current standalone-crate layout.

The existing TOML scope implementation already uses explicit revisions and fail-closed tests.
The logger merge-group shell scope is different:
it tolerates diff failure with `|| true`,
excludes deleted files,
and compares moving `origin/main`.
Those parallel implementations must not be treated as equivalent evidence.

## Coverage expansion anchored in actual consumers

### Static checks and source tests

Expand from focused workflows to package type checks,
Oxlint,
Markdown and generated-file validation,
Rust formatting and Clippy,
Kotlin checks,
Slint,
and applicable infrastructure validation.
Run ordinary source unit tests and Rust nextest through their owning tasks.
Capture discovered and executed test counts so an empty discovery cannot become coverage evidence.
Optional tool prerequisites and extra feature sets need named profiles;
compiling one default feature set does not establish other feature combinations.

Mutation,
property,
fuzz,
coverage,
conformance,
interop,
and benchmark tasks already have package or sidecar precedents.
Use bounded corpora and seeds for ordinary validation,
with separately budgeted campaigns for longer searches and mutation work.
Retain corpus identity,
seed,
crash artifacts,
and a positive control proving each guard or coverage assertion can fail.
Performance changes need unchanged-input spread before a provider ratio or regression threshold is credible.

### Built artifacts and browser consumers

Test the produced Node package,
CLI,
browser bundle,
and native library from a separate consumer,
including install,
import,
and representative calls.
Source tests do not prove exports,
bundling,
permissions,
native loading,
or installed dependency resolution.
Bind consumed bytes to source SHA,
toolchain,
target,
feature set,
and digest.
Cached producer steps must not substitute for freshly executing the selected consumer assertion.

Extend browser coverage to the engines actually configured by the browser and end-to-end task families.
Include real UI actions,
accessibility,
navigation,
storage,
error paths,
and representative visual states.
Keep configured retries and all attempt results visible;
a retry is not an independent clean first-attempt pass.
Build-only or empty test discovery is not browser interaction evidence.

### Native systems and GUI applications

Separate native execution from cross-compilation.
GNU and musl,
Alpine behavior,
Windows MSVC and GNU,
Linux ARM,
Windows ARM,
and Apple architectures need their own build and runtime evidence where supported.
A cross-built executable is useful coverage,
but does not demonstrate launch,
filesystem,
process,
DLL,
GUI,
or platform integration behavior on the target system.

Desktop IDE,
terminal,
file-manager,
music-player,
language-server,
and native launcher consumers need disposable display sessions and homes.
Cover installed resources,
software rendering where supported,
process cleanup,
confinement,
and representative GUI interactions.
Record the compositor,
display,
GPU or software renderer,
and application startup diagnostics.
An executable's `--help` output is a CLI smoke check,
not GUI lifecycle coverage.

The Mac release workflow already records an intermittent `hdiutil` retry and its unconfirmed cause.
Preserve retry diagnostics and final failure semantics when routing that workload;
do not count a successful retry as a proven platform fix.

### Android and VM consumers

Android needs distinct Kotlin and JVM tests,
native ABI builds,
APK consumer checks,
and instrumented application behavior across selected API,
ABI,
permission,
storage,
and lifecycle cases.
The repository requires a headful emulator on the host GPU in its own systemd user unit,
with a private AVD and adb port.
Its [Android emulator skill](../../.agents/skills/android-emulator/SKILL.md) prohibits headless flags,
Xvfb,
and container launches.
Cloud feasibility therefore requires a real display and GPU launch proof in addition to KVM.
KVM availability alone does not complete this lane.
Physical audio,
device hardware,
and interaction-specific evidence remain distinct from emulator evidence.

Depot's [current troubleshooting documentation][depot-troubleshooting] says its GitHub runners lack `/dev/kvm`,
while standalone Depot CI sandboxes expose it.
The [nested-virtualization announcement][depot-kvm] gives the standalone surface its concrete purpose here.
Use an actual device or VM lifecycle,
not merely a device-file check,
before adopting it for a required workload.

`package/mvm/cli` already supports Linux-hosted QEMU/KVM guests including Windows evaluation images;
`package/vmsync/cli` has separate KVM and Hyper-V lifecycle paths.
Desired tests should exercise boot,
guest-agent commands,
transfer,
incremental image synchronization,
failure and cancellation,
and cleanup in disposable state.
A Linux KVM guest does not cover the Windows-host Hyper-V path.
The latter needs its own measured host capability and runtime lane.
The existing Hetzner provisioning test creates billed resources on another provider;
permission to use Depot and GitHub does not authorize that additional provider.

### Releases and agent-requested experiments

Release coverage should verify complete target asset sets,
checksums,
licenses and notices,
archive structure,
installed launch behavior,
native dependency loading,
and the bytes actually published.
Add signing,
SBOM,
and provenance assertions where those artifacts are produced.
GitHub's [artifact attestations][github-attestations] are a separate identity and permission surface,
not an implication of a successful artifact upload.

npm's [trusted-publishing documentation][npm-trusted] currently requires GitHub-hosted runners
for its GitHub provider and excludes self-hosted runners.
Keep that publication boundary on an eligible host until supported evidence changes.
The repository's custom pnpr publisher is a different boundary:
its generated configuration binds issuer,
audience,
main-branch subject,
repository identities,
and the exact `pnpr-publish.yml` workflow reference.
Standalone Depot's [OIDC identity][depot-oidc] differs from GitHub's [OIDC identity][github-oidc].
Do not move that trust relationship by copying a workflow.

Cloud tasks requested by an agent can share the workload profiles and receipt format.
Each request needs immutable source or patch identity,
an allowlisted profile,
resource and time bounds,
trust domain,
and an artifact consumer assertion.
Use disposable homes and configured host applications for plugin or hook integration tests.
Keep recursion guards when a test itself launches agents.
Arbitrary untrusted runner labels,
commands,
or privileged publication must not be accepted as dispatch inputs.

## Routing mechanisms evaluated

### Static workload routing in GitHub Actions

Assign known jobs explicit runner labels and keep GitHub's dependency graph,
PR events,
artifacts,
logs,
and final checks.
This is directly compatible with the successful two-pool probe.
It is the first implementation candidate for ordinary package validation and native workers.
Its limit is workload capability:
changing a label does not add standalone Depot's nested virtualization or change publishing eligibility.

Developer supports Linux and Windows runners;
Depot Mac runners require another plan according to the current [plan matrix][depot-github].
GitHub-hosted Mac coverage remains available alongside the Developer pool.
This is plan-specific routing,
not a claim that Depot lacks Mac support.

### Explicit experimental pool override

A trusted `workflow_dispatch` accepts `github`,
`depot`,
or `compare`,
then maps that choice to fixed labels.
The probe verifies both cardinalities and the comparison case.
This supports agent-requested diagnostic work and same-source comparisons without altering defaults.
It does not enforce PR status:
[GitHub explicitly excludes manual workflow-dispatch checks from PR required-check association][github-required].
A required PR gate needs an eligible automatic run for any pool change;
a successful manual comparison cannot replace it.

### Trusted plan and bounded matrix

A small automatic planning job emits workload ID,
source identity,
provider,
runner label,
size,
selection reason,
and expected artifact names.
The final gate requires the entire selected set.
Use the existing task and workspace owners;
the real Mise cycle and global-input gaps must be resolved
or conservatively bypassed before affected scope is authoritative.
Do not expose arbitrary `runs-on` expressions to untrusted inputs.

GitHub permits at most 256 matrix jobs per workflow run;
the full inventory therefore needs task grouping,
shards,
or separate cadences rather than one task per unbounded matrix row.
Its current [limits][github-limits] and measured task parallelism belong in the plan receipt.

### Standalone Depot CI and remote builds

Standalone `.depot/` workflows can supply Linux sandboxes,
VM snapshots,
and KVM-dependent work in addition to Depot-managed GitHub runners.
The current [compatibility reference][depot-compatibility] documents a Linux execution surface,
unsupported fork execution and environments,
GitHub Packages token differences,
and ignored unsupported permission keys.
These are concrete migration checks,
not reasons to prohibit requested work.
The live permission names differ from older bundled reference material;
validate current schema and behavior before copying configuration.
The documented keys are `actions`,
`checks`,
`code-quality`,
`contents`,
`id-token`,
`issues`,
`packages`,
`pull-requests`,
`security-events`,
and `statuses`.

Depot remote BuildKit can build containerized application or toolchain artifacts while GitHub retains orchestration.
Account for context transfer,
image retrieval,
and the actual consumer run.
A cached image build is not proof that an application assertion executed during the current trial.

A GitHub wrapper around `depot ci run` could bridge final status for a standalone workload,
but requires explicit run-ID binding,
polling,
cancellation propagation,
artifact transfer,
and result mapping.
It does not automatically share GitHub's artifact namespace or concurrency lock.
Prefer Depot's native status integration when it demonstrably supplies the required current-source check.
The routing probe does not establish that standalone integration.

### Implementation ranking

For the next ordinary validation increment,
static workload routing ranks ahead of a dynamic planner because the workload-to-runner map is visible
and its worker and artifact failure semantics were exercised.
A bounded planner ranks ahead of a custom cross-orchestrator scheduler because existing task ownership remains usable
without introducing another global queue and cancellation model.
Explicit experimental overrides,
standalone KVM lanes,
and remote BuildKit are complements to that sequence.
Their respective workload advantages survive independently of the static-versus-dynamic routing choice.

### Concrete candidate assignments

These assignments illustrate the mechanism without changing production defaults:

- Trusted Node validation:
  a GitHub planning job selects explicit type,
  lint,
  and unit profiles;
  Depot-managed Linux workers execute them;
  the stable GitHub gate consumes their exact receipts.
  Untrusted source remains on a secret-free GitHub-hosted profile until Depot cache isolation is proven.
- Native Rust and desktop consumers:
  select native Linux x64 or ARM and Windows workers by target capability,
  with separate Alpine consumers and cross-compilation artifacts.
  GitHub-hosted Mac workers complement the Developer plan.
  Native Windows ARM runtime coverage needs a verified available host;
  it is not implied by a cross-built artifact.
- Android:
  compile and package on the applicable ordinary Linux profile;
  treat headful instrumentation as a separate standalone-Depot candidate that must pass the display,
  GPU,
  and actual emulator lifecycle checks.
- VM workloads:
  use standalone Depot's KVM surface for a disposable MVM guest lifecycle candidate;
  retain a distinct unresolved host-capability requirement for VMSync's Windows Hyper-V path.
- Containerized builds:
  keep GitHub orchestration while Depot BuildKit produces source-bound images,
  then run a separate application consumer assertion.
- Publishing:
  keep npm trusted publishing on an eligible GitHub-hosted worker;
  consume verified build artifacts without changing pnpr's issuer and workflow trust bindings.
- Agent experiments:
  dispatch an allowlisted workload and explicit `github`,
  `depot`,
  or `compare` selection with source or patch identity and bounded resources.
  Preserve its diagnostic status and artifacts without representing it as required-PR enforcement.

## Gates, trust, and failure semantics

Required validation must use eligible automatic events and stable check names.
Add `merge_group` only with an actual merge-queue adoption and test its synthetic head separately.
The [required-check reference][github-required] distinguishes skipped workflows,
skipped jobs,
manual runs,
and app-bound check identities.
Changing a required check's name or provider without a concrete policy transition can leave a PR waiting.
This repository currently has no required CI contexts to preserve,
but that does not remove the need to design the intended gate before making it required.

Use an `always()` final job with explicit `needs` result assertions and the exact artifact set.
The [workflow syntax reference][github-syntax] supplies the expression and permission semantics;
the experiment supplies actual failure and cancellation evidence.
An application failure must remain a failure even when another provider passes.
Infrastructure fallback must be an explicitly recorded new attempt for the same source,
with a reason and resource bound.
An expression-based runner label does not automatically fail over after a queue or allocation error.

Superseded validation may cancel to save work;
publication needs a policy that retains required releases.
Current [concurrency documentation][github-concurrency] supports a larger pending queue with `queue: max`,
but default concurrency retains only one pending run,
and the queue mode is incompatible with `cancel-in-progress: true`.
`cancel-in-progress: false` alone is not a durable release queue.
GitHub and standalone Depot concurrency groups do not establish a shared cross-service lock.

Treat fork and other untrusted source execution as a distinct trust domain.
The existing Windows workflow's repository-level `fork` condition does not identify a fork PR into the base repository.
Use actual event and PR head repository identity,
with immutable trusted source rules for paid execution,
secrets,
and write permissions.
Do not run untrusted checkout code under a privileged `pull_request_target` or artifact-consuming `workflow_run` bridge;
GitHub's [PR-target guidance][github-pr-target] and [secure-use guidance][github-secure] explain those boundaries.

Depot Cache is repository-scoped and [does not isolate branches][depot-github].
A cache-key prefix is not an access boundary.
Before untrusted execution shares that service,
prove its actual read and write isolation or give it a separate trusted-cache domain.
Snapshots also need a synthetic-secret-marker test before claims that checkout credentials or home-directory tokens
are excluded.
Provider ephemeral-VM isolation and cache isolation are different claims;
the current [Depot security reference][depot-security] distinguishes resource scopes.
The existing [CI caching policy](../decision/ci-caching.md) also prohibits caching what a workflow ships
and restricts cache saving by event.
An accepted Depot cache design must explicitly reconcile that policy and its GitHub branch-isolation assumptions
before changing a production publication path.

## Routing experiment results and limits

The research branch replaces one registered workflow only on `research/depot-pool-routing-20261018`.
It does not check out application code or read project secrets.
It uses pinned GitHub upload and download actions,
read-only repository contents,
5-minute job limits,
and at most 2 matrix workers.
Both worker classes assert 4 logical CPUs on Ubuntu 24.04 x64.
Actual receipts identify GitHub-hosted and Depot self-hosted execution.

At `3ea5cab1cba6ca42b4eb9ad8b63f91e21ec04ea9`:

- Run `39099643878`:
  GitHub and Depot workers pass;
  the stable artifact consumer gate passes.
- Run `39099645918`:
  the deliberately failing Depot worker fails;
  the GitHub worker passes;
  the final gate fails.
- Run `39099648238`:
  GitHub-only selection and the final gate pass.
- Run `39099650257`:
  Depot-only selection and the final gate pass.

At `0cbfa748cbd3219502b8ae8c62172da08ba292c9`,
the same-source two-pool no-hold baseline `39099976529` passes.
In `39099978536`,
the actual Depot worker `depot-fsz0gcf4vw` enters a bounded 90-second idle step at `05:16:31Z`.
Cancellation is requested at `05:16:32.164Z`.
Terminal readback reports the run `cancelled`,
the Depot worker `cancelled`,
and the stable gate `failure`.
That establishes job and workflow failure propagation after cancellation,
not the VM destruction timestamp or billing stop.

The initial harness's single-pool consumers failed after successful workers
because download-artifact directly extracts a single pattern match.
That documented behavior is recorded in the
[artifact consumer investigation](../troubleshooting/github-artifact-download-cardinality.md).
The corrected producer uses distinct filenames and explicit merged extraction.
The old failures remain in the proof and are classified as authored harness failures,
not provider incidents or source-test failures.

All 15 receipt artifact ZIPs from the 10 retained runs were downloaded and matched against GitHub's SHA-256 digests.
Their decoded receipts match source,
run,
attempt,
selected pool,
and actual job runner identity.
Workflow log archives are retained with hashes.
The probe does not measure application correctness,
speedup,
fork isolation,
OIDC publication,
merge-queue enforcement,
automatic supersession,
cross-orchestrator status bridging,
or non-Linux runner cancellation.

## Research and experiment boundary

Current tasks are an inventory starting point,
not the ceiling of desired coverage.
Research includes affected consumers,
static checks,
unit and property tests,
fuzzing and mutation,
browser engines,
native architectures,
GUI and VM consumers,
Android,
release provenance,
and bounded agent-requested cloud experiments.

The research changed only documentation on `main` and isolated experimental workflow source on the research branch.
Repository rules,
subscriptions,
secrets,
and production workflow files were not changed.
Third-party source clones were read without modification.
The disposable research worktree was removed after checking tracked status,
ignored files,
and possible root Git-directory artifacts.
Its only ignored file was the generated forbidden-strings rule cache;
the research branch and provider run artifacts remain available.

## Adoption conditions and remaining evidence

Before a production increment,
freeze each workload profile and its known source failures,
record the full selected consumer set,
and run representative positive and intentionally failing consumers.
Do not hide pre-existing application failures behind infrastructure fallback or cache reuse.
Publish source-bound receipts for duration,
queue,
cache state,
resource size,
provider errors,
and actual assertions executed.

Use measured expanded profiles and the observed source-arrival pattern for budget estimates.
Keep per-run work bounded,
coalesce superseded validation where appropriate,
and choose explicit longer-running cadences.
The current arrival dataset cannot price tasks that were never in the existing graph.
After a pilot establishes behavior,
adopt stable required checks deliberately and test their real PR association.
Retain a concrete routing override and a reversible GitHub-hosted default for supported workloads.

Remaining evidence includes complete affected scope for this cyclic mixed-language workspace;
cloud headful Android feasibility;
actual VM and Hyper-V consumers;
cache and snapshot trust isolation;
standalone Depot status and cancellation bridging;
and representative expanded-workload cost and reliability distributions.
Desired feedback cadence,
spending and concurrency limits,
and which new consumers become required remain user decisions for the later grilling session.

[github-run-api]: https://docs.github.com/en/rest/actions/workflow-runs#list-workflow-runs-for-a-repository
[mise-graph]: https://mise.jdx.dev/tasks/workspace-graph.html
[depot-troubleshooting]: https://depot.dev/docs/github-actions/troubleshooting
[depot-kvm]: https://depot.dev/changelog/2026-05-20-nested-virtualization-on-depot-ci
[github-attestations]: https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations
[npm-trusted]: https://docs.npmjs.com/trusted-publishers/
[depot-oidc]: https://depot.dev/docs/ci/oidc
[github-oidc]: https://docs.github.com/en/actions/reference/security/oidc
[depot-github]: https://depot.dev/docs/github-actions/overview
[github-required]: https://docs.github.com/en/pull-requests/how-tos/merge-and-close-pull-requests/troubleshooting-required-status-checks
[github-limits]: https://docs.github.com/en/actions/reference/limits
[depot-compatibility]: https://depot.dev/docs/ci/compatibility
[github-syntax]: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax
[github-concurrency]: https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency
[github-pr-target]: https://docs.github.com/en/actions/reference/security/securely-using-pull_request_target
[github-secure]: https://docs.github.com/en/actions/reference/security/secure-use
[depot-security]: https://depot.dev/docs/security
