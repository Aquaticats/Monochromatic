# Complete applicable CI measurement

## Purpose and accepted scope

Measure the complete validation graph selected by an actual repository change.
The earlier Depot trials measured representative commands and platform execution,
not this complete graph.
Their timings remain component evidence and cannot establish complete CI cost or latency.

The user clarified that intended checks include Oxlint and other linters,
that checks must run only for affected files or packages,
and that file-scoped Oxlint is invalid for this repository's custom rules.
Account access remains explicitly outside this work.

The execution owner must retain the actual base and head commits,
the changed paths,
the selected packages and consumers,
every required check,
and every exclusion with its reason.
An unaffected package is an intentional exclusion.
A selected package whose check did not run is unfinished coverage.

This document is the canonical checklist for the measurement.
It defines the completion boundary;
it does not claim that the checks described here have passed.

## Inventory evidence and its limits

The initial discovery used `mise tasks --all --hidden --json` in the repository at
`bae6ef28a183039a6c4ba509179dc4baaedeac2e`.
It returned 2,415 tasks.
The separate worktree discovery returned 2,434 because it also inherited 19 unqualified scratch tasks
from its parent directory's `mise.toml`.
Comparing names found every original task and no added package task.
Discovery for execution must retain only tasks owned by the immutable checkout.

The initial inventory contains:

- 189 package `lint` aggregators,
  164 `lint:oxlint` tasks,
  and 172 `lint:types` tasks.
- 19 `lint:clippy`,
  2 `lint:detekt`,
  3 `lint:slint`,
  2 `lint:rustfmt`,
  and one `lint:tofu` task.
- 152 package `build` aggregators,
  129 `test:unit` tasks,
  and 26 package `test` aggregators.
- 14 `fuzz`,
  13 `fuzz:coverage`,
  12 `test:mutation`,
  and 2 `test:conformance` tasks,
  plus specialized host,
  integration,
  browser,
  device,
  and container tasks.

These counts describe the discovery universe,
not the number of checks to run for every change.
Each historical case needs its own task catalog from its own head commit.
Task presence also does not establish independent coverage:
an aggregator can repeat its children,
an example task can require a human interaction,
and a successful no-op is not a validation result.

## Existing ownership and selector audit

The existing root owner is `mise.no-env.toml`;
`mise.toml` is generated through file-enforcer.
Its `fanout_packages` enumerates task suffixes across active package task roots.
It does not compute an affected-package graph.
Its `fanout` enumerates immediate child tasks,
which is different from selecting consumers of changed source.

The existing logger and TOML workflows contain path selectors:
`.github/workflows/logger-fuzz.yml` and `.github/workflows/toml-edit-fuzz.yml`.
Their selectors describe their own suites,
not all repository consumers.
The TOML owner also has `package/toml-edit/core.fuzz/src/ci-scope.ts`:
it validates immutable event commits,
checks checkout identity and ancestry,
and preserves deleted paths through a NUL-delimited diff with rename detection disabled.
`package/monorepo-task/core-exec/src/depends.ts` owns source/output freshness checks,
not Git-diff consumer selection.

The retained historical analysis in
[the change-stream audit](../audit/depot-change-stream-2026-10-15.md)
already computes reverse consumers from dependency,
development dependency,
peer dependency,
and optional dependency declarations.
Its explicitly documented limitations include using a current manifest snapshot
and missing undeclared imports and historical renames.
Reuse that algorithm's ownership model,
but read manifests at the actual measured head and retain the graph evidence.
The publishing order in `package/config/pnpr/src/publish-missing-versions.ts`
deliberately excludes development dependencies because its boundary is installation order.
That narrower publishing graph is not a CI impact graph.

## Change selection and dependency closure

- [ ] Resolve and retain immutable base and head commit IDs.
  Enumerate changed paths with a NUL-delimited Git diff that retains deletions.
  Treat a rename as both removed and added ownership when establishing affected packages.
- [ ] Read package owners,
  manifests,
  task definitions,
  and configuration from the case head.
  Use the base graph as well when an owner or dependency was removed.
- [ ] Select changed package owners and their transitive consumers.
  Include runtime,
  development,
  peer,
  optional,
  Cargo path,
  and explicit build/configuration dependencies.
  Preserve a dependency path explaining each indirectly selected package.
  Distinguish implementation inputs from test-only fixture inputs:
  changing a test's fixture selects that test,
  but does not itself change the tested tool's implementation or all its consumers.
- [ ] Audit source imports and configuration ownership that manifests do not express.
  A root tool configuration consuming a changed package is a dependency edge,
  not an excuse to omit its consumers.
  Record a broader affected ecosystem only when that shared input actually affects it.
- [ ] Select package/project checks for semantic tools.
  Use changed-file lists only for checks whose behavior is valid at file scope.
  Do not pass changed-file arguments to Oxlint.
- [ ] Deduplicate equivalent checks and shared prerequisite builds,
  while retaining every platform,
  feature,
  browser,
  and consumer boundary that supplies distinct evidence.
- [ ] Record unaffected packages and account-dependent operations as explicit exclusions.
  Do not interpret missing prerequisites,
  a failed selector,
  or an empty discovery result as an unaffected package.

### Oxlint project boundary

Run each selected package's `lint:oxlint` task unchanged,
with no file arguments.
Include relevant consumer packages when a dependency's semantic behavior changes.

`package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types/semantic-bridge-cache.ts`
requires the nearest configured project and explains why substituting the root project can omit callers.
`effect-summary-index.ts` describes backwards caller traversal,
and `effect-fixed-point-propagation.ts` propagates callee and callback effects.
These sources substantiate the user's project-context constraint.
A root sweep is not evidence that every selected package used its own complete project.

`package/config/oxlint/src/index.ts` loads the shared custom plugins.
Changes to those plugins or their shared configuration select the projects using them.
Root-level TypeScript/configuration inputs also require their own applicable validation.
There is no root `lint:types` task in the initial task catalog;
an implementation must retain this gap rather than invent an invocation that does not exist.

## Required validation families

For each selected package,
expand the task graph into concrete required leaves and compare it with the package's source and tests.
Keep specialized tasks whose behavior is not reached by the ordinary aggregator.
Keep family-specific selection reasons so build-only changes do not automatically run unrelated device interaction.

### Static checks

- [ ] Run full package `lint:oxlint` and `lint:types` tasks where declared.
  Include additional configuration type checks such as file-enforcer's `lint:config:types`
  when their configuration inputs are affected.
- [ ] Run declared Rust `lint:clippy`,
  formatting checks,
  Slint checks,
  Kotlin detekt,
  Android lint,
  and OpenTofu validation on the selected owners.
  An ordinary Rust `lint` task often runs only `cargo check`.
  Kotlin's ordinary `lint` compiles Kotlin and tests;
  its detekt task is separate.
- [ ] Run root `lint:dprint` and `lint:monochromatic-lint` on affected eligible files
  through a verified file-selection adapter.
  Configuration changes select the files governed by that configuration.
  Respect each tool's configured exclusions and retain those exclusions in the evidence.
- [ ] Run the forbidden-strings gate against the actual change,
  using the effective deny-list and retained workflow semantics.
  A changed deny-list requires checking the files governed by that list.

Root `lint` is not the execution manifest:
it first dispatches package aggregators and then repository-wide checks.
Dispatch independent gates independently so an earlier failure cannot hide their results.
Direct package Oxlint/type invocations must retain the root concurrency controls where applicable:
`OXLINT_THREADS=1` and `TSC_SINGLE_THREADED=1`.
Their implementation is in the task-util wrappers.

`dprint.json` extends `package/config/dprint/index.json`.
That configuration disables its TypeScript and Markdown plugins;
Oxlint and monochromatic-lint supply those separate responsibilities.
`monochromatic-lint.config.jsonc` excludes paused and deprecated package trees
and contains documented warning/exemption policy.
Preserve that policy;
do not silently strengthen or weaken it for measurement.

### Builds and artifact consumers

- [ ] Build every required artifact for the selected source and consumer checks.
  Record the task,
  platform,
  feature set,
  input hash,
  and artifact hash.
- [ ] Include native and Android ABI builds when the changed dependency reaches those applications.
  Android's `build:native` produces both `arm64-v8a` and `x86_64` libraries;
  its Gradle build depends on that task.
- [ ] Run tests against the artifact the consumer uses,
  including external-consumer or extension-host tasks where those boundaries are affected.
  An existing bundle from another revision cannot satisfy this requirement.
- [ ] Record root/shared prerequisite builds separately from consumer execution.
  Reuse them only when their complete inputs match.

### Unit, property, conformance, and mutation checks

- [ ] Enumerate the selected packages' real unit-test files,
  including separately declared native and integration suites.
  Compare expected files/cases with executed files/cases.
- [ ] Classify expensive tests individually.
  Root `test:unit --all` includes account provisioning in the mvm package,
  so it is not an authorized blanket invocation here.
  Retain non-account expensive checks when their source is affected.
- [ ] Run bounded fuzz and coverage gates already attached to the selected families.
  The logger and TOML workflows use `fuzz --budget 3000`
  and `fuzz:coverage` without `--write`.
  Preserve the required scope and conformance checks of the selected sidecar.
- [ ] Classify each declared mutation task in the workload manifest.
  If it is part of the intended gate,
  include its bounded policy and report;
  if it is a separate maintenance campaign,
  record that exclusion rather than pretending its task does not exist.
- [ ] Retain Rust feature matrices and native test execution.
  Do not substitute compilation for nextest results.
  Existing tasks that use a different Rust test command remain visible as task-policy gaps.

The root `test` task dispatches unit,
browser,
and end-to-end families.
It does not dispatch every native package `test` task.
The shared unit runner returns successfully for an empty file list,
so exit zero alone is not proof of coverage.

### Browsers, desktop, mobile, and platform hosts

- [ ] Select browser test files belonging to affected packages and consumers,
  preserving complete suites within those selected owners.
  `playwright.browser.config.ts` defines Chromium,
  Firefox,
  and WebKit projects with explicit package patterns.
  `playwright.e2e.config.ts` defines Chromium and Firefox.
  Compare tracked browser/e2e files with those configuration patterns.
- [ ] Run affected desktop integration through its declared compositor/container/host boundary.
  Preserve relevant Alpine architectures and Windows targets from the existing workflows.
- [ ] Run affected Android lint,
  JVM unit tests,
  native ABI checks,
  APK builds,
  and instrumented/device consumers when selected by the change.
  Device prerequisites remain requirements until exercised;
  missing hardware is not a passing check.
- [ ] Preserve host-specific filesystem identity and semantic bridge checks when affected.
  `.github/workflows/fs-id.yml` and `.github/workflows/readonly-semantic-bridge.yml`
  establish distinct host boundaries not replaced by a Linux unit suite.
- [ ] Retain all retries,
  skipped cases,
  expected failures,
  and their reasons.
  Configured Playwright retries are part of execution and cost.

### Repository consistency and security

- [ ] Validate affected generated files against their owners using a disposable checkout.
  `package/file-enforcer/core/src/cli.ts` imports and executes its config;
  it has a watch option but no implemented read-only check option.
  Regenerate in the disposable fixture and compare its resulting tracked changes.
- [ ] Include affected CodeQL language/workflow scans and their existing selection policy.
  Distinguish source analysis from publication of its report.
- [ ] Retain the actual event type when applying workflow policy.
  These source cases replay push changes.
  At the TOML head,
  CodeQL runs on pull requests,
  schedules,
  or manual dispatch;
  final-newline runs on pull requests,
  merge groups,
  or manual dispatch.
  Both are explicit exclusions for that push case.
- [ ] Preserve Scorecard's push-on-main,
  branch-protection,
  schedule,
  and rate-limit policy as a separate repository-security workload.
  The TOML head predates its rate gate;
  the logger and native heads contain one.
  State the policy revision used by the trial and account for gate/analysis attempts accordingly.
- [ ] Exclude publishing,
  deployment,
  account login,
  secret editing,
  production provisioning,
  watch servers,
  automatic upgrades,
  and source-formatting mutations from validation gates.

Root `validate` formats,
builds,
and tests;
it omits lint and mutates source.
It therefore cannot stand in for this checklist.

## Actual change cases

Execute the complete applicable graph for each selected positive case.
The smaller case establishes pipeline plumbing;
it does not replace broader consumer or native cases.

- TOML source change:
  base `cf9fcb48047d86e0af01ac9c73c1c7bf26fb277f`,
  head `525d2e09f8168a3bdd7e513e26d4c80b363123e5`.
  The actual diff changes `package/toml-edit/core/src/value-encoders.ts`.
- Logger source and tests:
  base `20dbaca6c2bb39d5d7e51fac447d76393c21a32e`,
  head `ef01490f5e960f8ca0b424c12242caeb3c5a2b5e`.
  This case exercises the logger's real consumer closure.
- Shared native audio core:
  base `45795584ff2dcf830dc505e121f0cc92c77860e5`,
  head `cb95cf6d785c3073c192b95f0632cdae7b910894`.
  Its Cargo path consumers include desktop,
  Android,
  and the benchmark sidecar.
- Desktop platform change:
  base `a9b4a681d21907ecc5eb208a754e7b9a0f6e9168`,
  head `3434eaa455d569c18b52cf15100f77f9db251d21`.
  This actual Windows console/subsystem change retains both platform workflows
  and the Alpine container source at its own head.
  It complements the earlier native-core case,
  whose head predates those platform workflows.
- Documentation selection control:
  head `bae6ef28a183039a6c4ba509179dc4baaedeac2e` versus its first parent.
  Assert that its documentation-only diff does not select Oxlint or unrelated application checks.

Also verify selection for a real shared-configuration change.
That selection control does not require running unrelated global work merely to manufacture a whole-repository scenario.

### TOML case task manifest

The exact-head manifest closure selects `toml-edit/core`,
`toml-edit/core.fuzz`,
`file-enforcer/core`,
`vm-builder/cli`,
and `test-fixture/file-enforcer-perf` under `package/`.
The source audit adds a test-only consumer in `oxlint-plugin/prefer-readonly-parameter-type`:
`src/workspace-source-effect.unit.test.ts` reads file-enforcer's live Cargo enforcement source
and follows its TOML calls.
This edge selects that unit test and its plugin build prerequisite,
not every consumer of the plugin implementation.

The package manifest has 24 checks:

- `toml-edit/core`:
  `build`,
  `lint:oxlint`,
  `lint:types`,
  `test:unit --all`,
  `test:conformance`,
  and `test:mutation`.
- `toml-edit/core.fuzz`:
  `lint:oxlint`,
  `lint:types`,
  `test:unit --all`,
  `fuzz --budget 3000`,
  and `fuzz:coverage`.
- `file-enforcer/core`:
  `build`,
  `lint:oxlint`,
  `lint:types`,
  `test:unit --all`,
  `fuzz --budget 3000`,
  `test:container`,
  and `test:mutation`.
- `vm-builder/cli`:
  `build`,
  `lint:oxlint`,
  and `lint:types`.
  Its declared unit task has no test files at this head and cannot supply coverage.
- `test-fixture/file-enforcer-perf`:
  `lint:types`.
  Its task file deliberately omits Oxlint;
  its host benchmarking and capability-changing tasks are separate maintenance work.
- `oxlint-plugin/prefer-readonly-parameter-type`:
  `build`,
  then `test:unit` selecting only `src/workspace-source-effect.unit.test.ts`.

The exact source contains 55 file-enforcer unit files,
37 TOML unit files,
7 sidecar unit files,
and the additional semantic-plugin consumer file.
None of those selected package trees contains an expensive unit-test filename.
The full sidecar unit suite already includes `ci-scope.unit.test.ts`;
do not run `test:scope` again for separate coverage credit.
The file-scoped `lint:scope` is superseded by full package Oxlint.

Required external dispositions are the push forbidden-strings gate,
generated-output consistency through root `sync:files` in a separate disposable fixture,
and Scorecard's repository-event accounting under the chosen policy revision.
The actual changed path is TypeScript,
which this head's dprint configuration does not format;
no changed Markdown or Rust file selects their file linters.
No browser/device/application consumer appears in this case's audited closure.

The mutation runner's source sets two containers,
each bounded to `2g` memory,
2 CPUs,
512 processes,
and a 1,800-second session timeout.
Per-mutant timeout is three times the baseline with a 5,000-millisecond minimum,
and a failing single-mutant shard has one retry.
Retain survivor and timeout findings from the report:
the CLI exit alone does not encode a mutation-score threshold.

The first execution manifest still marks completion false until the root scanner,
generated-output,
and Scorecard accounting dispositions are connected and all required checks report results.

### Broader case reach

The exact logger-head manifest closure contains 158 packages.
This measured breadth follows real development/configuration dependency cycles;
it must not be replaced by just the logger package to obtain a smaller timing.
Source-import and task-boundary review supplies the conservative execution manifest.
The revised task expansion contains 719 package checks and 19 external check groups,
including 155 full-package Oxlint tasks,
157 type checks,
1,057 account-independent unit files,
eight browser files,
and four end-to-end files.
These are candidate requirements,
not passing or completed execution counts.
Live Hetzner provisioning,
paid image-embedding calls,
and live OpenAI verification are explicitly excluded.
Offline Pi verification,
desktop terminals,
disposable VM lifecycle tests,
and network-namespace tests remain required affected consumer boundaries.

The task-name audit also includes credential-free Worker deployment dry runs,
Git policy Windows cross-target Clippy,
old-format interoperability fixtures,
native mutation testing,
the Git policy sidecar's bounded AddressSanitizer smoke task,
and the deepmerge sidecar's compiler and Bun/Deno runtime checks.
The browser property suite needs the separately named `bundle:browser-properties` preparation task.
Deepmerge campaigns requiring a separately selected external checkout or historical-release ledger are excluded;
the locked dependency's ordinary,
coverage,
declared-type,
compiler,
and runtime checks remain selected.
Intentional-invalid lint fixture helpers are diagnostic displays;
the selected plugin unit suites assert their results.

A Rolldown AST parse resolves package export maps and relative `dist` imports.
It parsed the tracked JavaScript and TypeScript inputs without parser errors
and found 3,555 real imports.
Positive and negative controls cover static imports,
literal dynamic imports,
`require` calls,
comments,
string contents,
and regular expressions.
The AST graph has no build cycle or prerequisite owner outside the affected closure.
The runtime-path audit resolved 596 artifact strings.
The unmatched cross-package candidates are Git self-shim markers and test stack-frame filters,
which compare text rather than load those artifacts.
Computed Pi paths remain owned by the native preparation task;
extension verifiers retain their package builds.
The first static wave exposed missing owned-artifact prerequisites:
type checks carried an unused `artifactPrerequisites` field,
while the earlier scanner discarded self imports and missed relative `dist` imports.
Commit `4544076ec3` supplied these as executable `prerequisites` and proposed pruning lexical false matches.
Independent review identified unresolved `require`,
computed-import,
URL,
and spawned-artifact paths.
Follow-up `ed1d31ae11` retains all prior barriers,
folds the declared owned-artifact metadata into executable prerequisites,
and keeps the newly found literal dependencies.
The AST audit adds no prerequisites beyond that conservative graph.
It retains 772 barriers absent from the narrower AST projection;
no prerequisite is removed for this measurement.
The frozen manifest has SHA-256
`f4cbd47f210f5b031d76c5fc1c28fa704b349621f91eefa3f4f3ad575acfedc3`.
Adapter commit `d4a082cd47` partitions the remaining checks into container,
fuzz,
mutation,
native mutation,
Pi regression,
regular,
and Rust groups.
Its partition identity and prerequisite closure checks cover every selected check exactly once
between the prior static/unit assignment and the remainder assignments.
The fuzz group executes inside a network-free Podman overlay limited to 2 GiB and 2 CPUs.
Task parsing,
JavaScript syntax,
partition coverage,
and manifest digest checks pass locally;
the cloud consumer execution remains required.
The source diagnostics need replay with this corrected ordering before source-failure classification.
Outside-closure active matches are root configurations,
the Playwright declaration file,
an already-covered plugin fixture,
and a documentation benchmark excluded from correctness execution.
Paused and deprecated package matches remain separately recorded and excluded from the active workspace.
Build and test/type prerequisites now name the relevant artifact owners;
full-package Oxlint keeps its authored configuration bootstrap.

The Git policy native container checks require locally identified base images,
not just `podman` on `PATH`.
The authored native gate uses image
`6ec87f6d290a2f59bda5b3ffd4197058fe0749d02b4978c877e8edf6dc38802a`.
Its Git image builder requires Git revision `a018953688f1b10bddf91bff8747068f5f4746a4`
and compiler image `84f24e75017a7d8afa1c69e51c52f37e7d8d644cd2597a01f4732ad0b386dc20`.
The execution adapter preserves that Git image and Rust image
`62ba2f7ce22ba9bc501110d3452c7ae814fba367c46f7eea3629a79286353884`.
The locally exported Docker archive measures 2,329,816,576 bytes with SHA-256
`c377f23ca6284f3d8020cb8697cf7654eaa7549a25c05150c6478ac139f50fb4`.
Its custom layers contain Rust sources,
upstream Git sources,
and compiled Git outputs;
the inspected image configuration and custom-layer paths contain no user credentials.
Transfer uses authenticated Depot sandbox stdin streaming into a disposable runner.
The importer checks the archive digest and exact loaded image IDs before creating the private CI snapshot.
Successful transfer and snapshot creation remain pending.

The exact native-core-head Cargo path closure contains four packages:
the core,
its benchmark sidecar,
the desktop application,
and the Android application.
At that head both application manifests select the `service` feature;
the later fingerprint consumer changes must not be projected backwards into this case.
The documentation control has no selected package owner.

The native candidate manifest retains 21 exact-head task checks and five external groups.
Cargo path dependencies select affected consumers but do not impose separately built artifact edges.
Desktop,
Android native,
and benchmark builds compile their own core dependency through their authored Cargo invocation.
They do not wait for the standalone core build;
that build remains an independently selected core check.
The desktop `test:e2e` consumes the built application,
and Android instrumentation consumes the built APKs.
Remaining adapters cover root file checks,
the core feature matrix,
native host platforms,
device prerequisites,
and repository-security accounting.

The desktop console case selects only `package/music-player/desktop-app`.
Its manifest has six local checks and six external groups.
The exact-head Windows workflow covers MSVC x64,
MSVC arm64,
and GNU UCRT64 x64.
It asserts executable headers,
redirected output,
missing-standard-handle behavior,
launch survival,
and nextest results.
The exact-head Alpine workflow covers x86_64 and aarch64,
each with no-server,
ALSA,
and PipeWire smoke scenarios inside the repository's nested compositor.
A separate native macOS adapter exercises package build,
Clippy,
nextest,
and the release CLI.
The desktop `test:e2e` task already builds a fresh release executable before driving it.
An empty `E2E_ONLY` selects every session.
The authored harness requires a parent Wayland display,
PipeWire,
and a systemd user manager;
it is not a headless command-only suite.
Its repository fallback library removes the need for a private external generator.
Unchanged core and compositor sources are build prerequisites;
they do not select unrelated core or Android lint/test gates.

Native trial setup must record the actual selected tool binaries,
not only declarations from `mise ls`.
The root agent observed Rust selection drifting from the historical lockfile without an explicit version override.
The source tasks retain auto-install disabled and tracked-source identity checks before and after execution.
Tool installation belongs to a separate harness context,
and current Scorecard gate policy is recorded separately from historical source policy.
The host mismatch was reproduced on mise `2026.10.0` and traced to its Rustup install-link classification.
The same source and actual compiler probe passed with downloaded mise `2026.10.9`.
Upstream fixed the defect in `2026.10.1`;
see [the reproduction and source trace](../troubleshooting/mise-rust-lockfile-linked-install.md).
That host result does not diagnose a separate cloud failure using a fixed mise release.

## Evidence and completion predicate

### Shared cohort identity

The authoritative common manifest is the main-branch file
`doc/audit/depot-complete-cohort-<cohort_id>.json`.
Freeze and publish its exact bytes before creating the shared request timestamp.
The file must be smaller than 1 MiB,
use `schemaVersion: 1`,
and carry these bindings:

- `cohortId` identifies the same immutable experiment across provider routes.
- `cases.<caseId>` records `sourceRevision`,
  `baseRevision`,
  and the exact per-case `manifestSha256`.
- `routes.<routeId>` records `caseId`,
  the immutable `harnessRevision`,
  and the repo-relative `workflow` path.

Every final workflow requires `cohort_id`,
`manifest_sha256`,
and `requested_at` inputs.
The cohort ID must match `^[A-Za-z0-9._-]{1,128}$`.
The manifest digest must be lowercase hexadecimal SHA-256 of the common manifest's exact bytes;
it is distinct from each case-manifest digest.
The timestamp must be canonical UTC `Date.toISOString()` output,
shared across all routes and retained identically before and after each route.
It does not belong in the frozen manifest.

Retrieve the file through the authenticated Contents API object response,
decode its base64 `content`,
and hash the decoded bytes before parsing the JSON.
The actual transport control preserved all 1,043,537 bytes of the historical audit fixture;
a mismatched expected digest was rejected.
Raw JSON CLI output transformed escaped control characters and is not the selected transport.

Each route validates the common cohort ID,
its case source/base/digest,
its workflow path,
its actual harness `HEAD`,
and provider `GITHUB_SHA` against that same `HEAD`.
Before and after source guards retain clean tracked state,
reject index exclusion flags,
compare configured metadata directly with committed bytes,
and prove source identity remains unchanged.
After-work validation may revalidate the cached exact manifest bytes.
Guard rejection remains failed evidence;
it cannot produce a successful after-work receipt.

The case manifests still have `completeGraph: false`.
Regenerate their complete graph registrations and dependent partition headers before final cohort acceptance.
This field describes graph registration;
it does not prove passing execution.
Disposable API response fixtures establish adapter rejection behavior only.
They are not published as real cohorts and cannot establish provider execution.

### Required evidence

Every case manifest records `baseRevision`,
`headRevision`,
`sourceRevision`,
`changedFiles`,
`selectedPackages`,
`selectionReasons`,
`excludedChecks`,
and ordered `checks`.
Each check records its ID,
task,
arguments,
family,
scope,
package owners,
and prerequisite check IDs.
Oxlint checks require package or project scope and empty file arguments.

- [ ] Prove the selector with a positive affected-package case,
  an affected consumer,
  and an unrelated negative package.
  Preserve the actual outputs and the expected membership.
- [ ] Collect each required check's command,
  exit status,
  platform,
  expected and observed test counts,
  skip/retry details,
  artifacts,
  logs,
  and provider run/job/attempt identity.
- [ ] Include environment preparation,
  cache restore/save,
  cancelled attempts,
  failures,
  retries,
  and final verification in cost accounting.
  Separate additive resource consumption from elapsed wall-clock unions.
- [ ] Reconcile every eligible immutable-source command with a terminal attempt
  or a verified failed-prerequisite disposition.
  Preserve historical source failures as failed outcomes,
  and execute independent checks despite those failures.
  Harness omissions,
  deferred waves,
  wrong working directories or toolchains,
  and unverified dependency blocks remain incomplete execution.
  A skipped record does not establish a verified prerequisite failure by itself.
- [ ] Report passing CI separately from completed execution/accounting.
  Passing CI requires every required gate to succeed
  or its existing policy to permit nonexecution.
  A failed,
  cancelled,
  or unobserved required gate cannot establish passing CI.
  Successful-pipeline latency requires a complete passing replay;
  a diagnosed failure supplies only the observed failed-pipeline timeline.
- [ ] Report unexercised validation families and unresolved selector edges explicitly.
  No weighted monthly projection can turn missing execution into measured coverage.

The active queue is to finish tool and image preparation for the reviewed logger artifact graph,
connect the retained external check groups,
and execute every selected gate.
The offline Pi host adapter is prepared on `trial/depot-pi-host-20261016`,
through commit `60a0142d87`,
and awaits live execution by the environment agent.
Its manifest retains the historical configured catalog's 22 extension entries,
using exact-head repository artifacts and recorded public package versions.
Preparation records independent build failures;
only consumers needing those artifacts are blocked.
The isolated runtime checks configured extension loading,
real D-Bus notification payloads after an observer positive control,
actual detached-terminal answer and cancellation,
and all offline Pi TUI cases.
Its network namespace has no external interface,
and its container has two CPUs,
2 GiB of memory,
and a bounded lifetime.
Task parsing and catalog membership checks pass;
these are not live consumer success evidence.
The filesystem and semantic host adapter review preserves historical task entrypoints;
its bootstrap must disable mise-action environment export,
and its filesystem macOS runner must retain or explicitly identify the original `macos-latest` selection.
The first TOML and logger static-check waves may run while those independent reviews proceed.
Their results must reconcile against the final frozen manifest before completion is claimed.
