# Expanded repository coverage and execution routing

## Purpose and recommendation

This is an engineering proposal for the broader workflow authorized for the Depot trial.
It does not adopt a provider or change production workflows.
The [trial brief](depot-ci-trial.md) establishes repository-wide coverage,
reusable environments,
agent-requested experiments,
intentional GitHub routing,
and a USD $200 monthly ceiling with a preference for the USD $20 Developer plan.
Music-player and representative TypeScript checks are the initial measured workloads,
not the boundary of the intended system.

Recommend affected-package checks on the newest relevant revision,
consumer checks for shared changes,
broader platform checks before release,
and a scheduled sweep that detects omissions in change selection.
Use GitHub and Depot deliberately from the start.
Prepared Linux environments and selected native builds are Depot candidates;
existing Windows/macOS checks and routine work without a measured Depot benefit retain GitHub placement.
This expands the checks performed rather than merely reproducing current jobs faster.

## Evidence and scope

The inventory reads the active `package/` task definitions,
root task templates,
Playwright configuration,
music-player dependency manifests,
and every current `.github/workflows/` file.
The [trial results](../audit/depot-trial-2026-10-15.md) own execution evidence and cost observations.
Task presence establishes runnable intent,
not a passing result or complete behavioral coverage.
An existing suite with no production workflow is an unwired check.
A desired behavior absent from the selected suite is a test gap.
An environment that cannot yet run a suite is an execution gap.

Commands in this inventory name existing task suffixes.
Package tasks use `mise run //package/<category>/<name>:<task>`.
The proposal introduces no new task names or production controller.
Counts and durations cited from the trial apply only to the named runs.

## Current orchestration boundaries

The root [`mise.toml`](../../mise.toml) has reusable build,
lint,
TypeScript unit,
browser,
and end-to-end task definitions.
Its root `test` dispatches TypeScript unit tests and Playwright suites;
it does not fan out every package's Rust `test` or Android Gradle task.
The unit template selects active `*.unit.test.ts` files and excludes `*.expensive.*` unless requested.
Archived and paused trees are excluded.
Consequently,
a green root test result is not repository-wide native or mobile evidence.

Root package fanout discovers tasks by name and bounds parallel workers.
It does not calculate reverse dependencies from a changed revision.
The existing [`task-depends` implementation](../../package/monorepo-task/core-exec/src/depends.ts)
compares source/output staleness;
that is a different responsibility from identifying affected consumers.
Any eventual change-selection implementation should reuse existing task execution
and preserve a human-readable list of selected packages and selection reasons.

[`playwright.browser.config.ts`](../../playwright.browser.config.ts) has an explicit package allowlist
and Chromium,
Firefox,
and WebKit projects.
[`playwright.e2e.config.ts`](../../playwright.e2e.config.ts) selects `*.e2e.test.ts`
and Chromium/Firefox projects.
Adding a browser test outside the browser allowlist does not add it to that suite automatically.
The discovery check must compare matching test files with the selected configuration.

## Workload inventory

### TypeScript libraries and shared tooling

- Tasks:
  each affected package's existing `build`,
  `lint:types`,
  `lint:oxlint`,
  and `test:unit`,
  where declared.
  [`logger/core`](../../package/logger/core/mise.toml) is the initial representative.
- Trigger:
  newest relevant source change,
  shared package change,
  configuration change,
  dependency update,
  or an agent request naming a revision and task.
- Platform and route:
  prepared Linux Node/dependency environment on Depot for requested iterations;
  GitHub for routine checks according to measured usage.
  Preserve dedicated host-platform tests when package behavior depends on the operating system.
- Output:
  task exit status,
  discovered test files,
  test results,
  compiler/linter diagnostics,
  and the source revision.
  Build outputs alone do not prove the consumer surface.
- Completion:
  logger build,
  type check,
  and unit suite passed in Depot run `wjr4cmpjvs`.
  Its job interval was 45.918 seconds on two CPUs,
  including setup.
  That result does not establish success for every TypeScript package or logger browser sink.
  [Measured evidence](../audit/depot-trial-2026-10-15.md#existing-checks-newly-exercised-through-depot-ci)

### Shared audio core and native consumers

- Tasks:
  [`truepeak-core`](../../package/music-player/truepeak-core/mise.toml)
  `build:default`,
  `lint:clippy`,
  and `test`;
  the latter runs nextest with all features.
  Add desktop tests and Android native build/device checks when this shared crate changes.
- Trigger:
  core source,
  Cargo dependencies/toolchain,
  or a consumer integration change.
  The [desktop manifest](../../package/music-player/desktop-app/Cargo.toml)
  and [Android native manifest](../../package/music-player/android-app/rust/Cargo.toml)
  both depend on this crate with `service` and `fingerprint` features.
- Platform and route:
  Linux core tests in Depot CI or GitHub;
  consuming application checks on their own platform routes.
- Output:
  nextest totals and named failures,
  default-feature build result,
  Clippy diagnostics,
  and separate consumer results.
- Completion:
  all 80 core tests passed in `wjr4cmpjvs`;
  its audio job interval was 136.452 seconds on four CPUs.
  Negative control `2clfb5npj3` made a real assertion fail and the run failed visibly.
  The expanded native job then passed default,
  fingerprint-only,
  service-only,
  and all-feature suites with 58,
  67,
  71,
  and 80 cases respectively,
  plus the existing all-targets Clippy task.
  Its whole-job duration was 157.920 seconds on four CPUs.
  This replaces the earlier core-only timing baseline when modeling the expanded job;
  it does not substitute for consumer execution.
  [Expanded execution evidence](../audit/depot-expanded-execution-2026-10-16.md#native-results)

### Android host and native checks

- Tasks:
  [`android-app`](../../package/music-player/android-app/mise.toml)
  `test:unit`,
  `lint`,
  `lint:detekt`,
  `lint:clippy`,
  `build:native`,
  and `test:instrumented:build`.
  Native builds target arm64-v8a and x86_64.
- Trigger:
  Kotlin,
  Gradle,
  manifest/resources,
  native Rust,
  shared audio core,
  or Android toolchain changes.
- Platform and route:
  Linux JDK/SDK/NDK prepared environment on Depot;
  GitHub is an intentional alternate allocation for routine runs.
  No emulator is necessary to compile the test APKs or run host JVM tests.
- Output:
  JVM XML reports,
  Android lint report,
  Kotlin/Clippy diagnostics,
  native libraries for both ABIs,
  application APK,
  and test APK.
- Completion:
  host JVM tests and Android lint passed in `3tvq2wvskh`.
  The counted follow-up inspected 127 XML suites and 1,289 cases;
  cache-restored reports retain their original execution timestamps.
  Host success does not claim `lint:detekt` or native Clippy success.
  [Measured evidence](../audit/depot-trial-2026-10-15.md#existing-checks-newly-exercised-through-depot-ci)
  Instrumented APK build [38826684750][android-apk] passed;
  the build task took 149.07 seconds in the trial's sixteen-CPU job.
  Device execution completed on a disposable headed x86_64 Android 37 emulator with 16 KiB pages.
  With audio fixtures,
  the original tests passed 14 cases and failed two storage-context cases without skips.
  A test-only context experiment passed all 16 cases;
  the original failure remains recorded and no production test change is implied.
  [Device evidence](../audit/depot-android-device-2026-10-15.evidence.json)

### Android device behavior

- Tasks:
  `test:instrumented` selects the connected suite;
  `test:instrumented:device` is a separate device helper whose default class is only `PeakSweepWorkerTest`.
  A default helper invocation must not be labeled the full suite.
  [Task definitions](../../package/music-player/android-app/mise.toml)
- Trigger:
  device-facing behavior,
  JNI/native changes,
  shared audio changes,
  release candidates,
  and explicit agent requests.
- Platform and route:
  cloud-built APKs consumed on a disposable local emulator or authorized test device.
  The [emulator skill](../../.agents/skills/android-emulator/SKILL.md)
  requires a headed host-GPU launch in a systemd user unit and prohibits headless/container launches.
  A cloud KVM probe alone does not satisfy that requirement.
- Output:
  installed artifact hashes,
  Android/API/ABI and page-size evidence,
  selected test classes,
  instrumentation result totals,
  skips,
  logcat failure context,
  and cleanup evidence.
- Completion:
  the cloud-built artifacts executed on the disposable local emulator.
  With indexed audio fixtures the original suite had 14 passes and two storage-context failures;
  a test-only context experiment passed all 16 cases without skips.
  The original failures and artifact hashes remain in the
  [device evidence](../audit/depot-android-device-2026-10-15.evidence.json).
  Existing classes are `NativeBridgeTest`,
  `SessionStoreTest`,
  `PeakSweepWorkerTest`,
  and `RustEngineTest` in
  [`app/src/androidTest`](../../package/music-player/android-app/app/src/androidTest/kotlin/dev/monochromatic/musicplayer).
  `RustEngineTest` uses assumptions when indexed music is unavailable.
  A successful run with skipped library-dependent cases
  must state that boundary and retain fixture preparation as open work.
  Compiling the APKs is not behavioral completion.

### Desktop native and Alpine smoke checks

- Tasks:
  [`desktop-app`](../../package/music-player/desktop-app/mise.toml)
  `test`,
  `lint:clippy`,
  `lint:slint`,
  `test:alpine`,
  and platform build tasks.
- Trigger:
  desktop source/UI,
  shared audio core,
  native dependency/toolchain,
  or compositor changes affecting the harness.
- Platform and route:
  Linux compilation with Depot prepared environments or container builds;
  keep platform-native execution for Windows and macOS.
  The production [Alpine workflow](../../.github/workflows/music-player-alpine.yml)
  (moved into the `test-alpine` job of `.github/workflows/music-player-release.yml` on 2026-10-22,
  where it runs on a release or a manual run and no longer on pull requests)
  already tests x86_64 and aarch64 on relevant pull requests,
  main pushes,
  and manual requests.
- Output:
  nextest results,
  linter diagnostics,
  executable runtime checks,
  and no-audio/ALSA/PipeWire smoke evidence.
- Completion:
  [38820372915][alpine-run] passed 1,461 tests with one skipped
  and all Alpine smoke scenarios on a Depot GitHub runner.
  This trial result does not prove both Alpine architectures or the full interactive suite.
  Compilation sizing recommends sixteen CPUs with sixteen Cargo jobs for the measured cold workload,
  with eight/eight and sixteen/sixteen economically unresolved for source rebuilds.
  [Sizing evidence](../audit/depot-runner-sizing-2026-10-15.md)

### Desktop interaction and graphics environment

- Tasks:
  desktop-app `test:e2e` builds then calls
  [`desktop-e2e:run`](../../package/music-player/desktop-e2e/mise.toml).
  Its own `test:unit` and `lint:types` verify harness logic separately.
- Trigger:
  UI,
  input,
  playback integration,
  compositor controls,
  or a release candidate;
  use named subsets during agent iteration and the complete suite before accepting that revision.
- Platform and route:
  prepared Linux graphical session with isolated PipeWire routing and user services.
  The [suite requirements](../../package/music-player/desktop-e2e/README.md)
  require a parent Wayland session and the named utilities.
  Physical GPU hardware is not inherently required for desktop cloud execution:
  the repository [compositor](../../package/nested-wayland-session/cli/README.md#choosing-the-gpu)
  supports software rendering.
  This differs from the Android emulator's current launch constraint.
- Output:
  per-check `PASS`/`FAIL`/`SKIP`,
  `e2e-summary.json`,
  screenshots,
  application logs,
  and proof that audio stayed on the private null sink.
- Completion:
  the first full desktop cloud run [38826827023][desktop-run] failed pointer/text checks
  and aborted the wide session.
  Direct software rendering and keyboard fixture selection allowed every session to finish in run `38831098294`:
  116 checks passed and the folder-dialog check failed.
  Installing Zenity did not resolve that failure.
  Correcting the deprecated portal `Read` response recovered the chooser in `38833901553`:
  117 checks passed with no failures across all seven sessions.
  The full job took 410 seconds,
  including setup and builds.
  The correction is trial-only;
  [recovery evidence](../audit/depot-desktop-recovery-2026-10-15.evidence.json) preserves the original boundary.
  The suite explicitly records held-modifier repeat as not injectable.
  Neither a screenshot nor passing harness unit tests proves the real interaction suite ran.

### Browser applications and browser-dependent libraries

- Tasks:
  root `test:browser` and `test:e2e` with explicit selection,
  plus each affected package's build,
  type check,
  and unit tasks.
  Existing selected browser cases include logger storage sinks,
  fs-path OPFS discovery,
  the test harness,
  and the wc page.
  Existing end-to-end files are under [`dom/core`](../../package/dom/core/src).
- Trigger:
  selected browser package,
  DOM/shared helper,
  browser tooling/configuration,
  or lockfile changes;
  broader browser project coverage before release and in the scheduled sweep.
- Platform and route:
  the existing Podman/Playwright environment on Linux,
  first on GitHub unless a prepared Depot environment yields measured benefit.
  Browser-engine projects do not replace host operating-system checks.
- Output:
  discovered test names,
  project names,
  assertion results,
  HTML report,
  and retry traces.
- Completion:
  the prepared Depot browser image executed logger and fs-path suites in three engines:
  67 passes,
  one WebKit OPFS-availability assertion failure,
  and one WebKit storage-fixture skip.
  Chromium and Firefox each passed 23 cases.
  The existing DOM suites passed 34 cases across Chromium and Firefox.
  A deliberately wrong browser assertion failed at its changed source line.
  Whole-job durations were 151.586 seconds for the failing storage suite
  and 117.823 seconds for the passing DOM suite.
  The three-engine storage suite is not fully green;
  [issue #1052](https://github.com/Aquaticats/Monochromatic/issues/1052) tracks its test assumption.
  Other selected browser packages remain unmeasured,
  and production routing remains unwired.
  [Expanded execution evidence](../audit/depot-expanded-execution-2026-10-16.md#browser-results)
  The [`done` task file](../../package/done/app/mise.toml)
  declares build/type/lint/unit tasks,
  and its discovered test file covers database migrations.
  No `done` browser or end-to-end test is selected by the current Playwright configuration.
  Desired user-flow coverage for that application is therefore a test-selection/authoring gap,
  not something moving existing jobs can supply.

### Other native applications and desktop infrastructure

- Tasks:
  [`terminal`](../../package/terminal/app/mise.toml)
  `build`,
  `test`,
  `lint:clippy`,
  and `lint:slint`;
  [`ide`](../../package/ide/app/mise.toml)
  `test`,
  `test:native`,
  `inspect:bundle`,
  `inspect:bundle-guards`,
  and the relevant language-lifecycle/confinement inspections.
  The [nested compositor](../../package/nested-wayland-session/cli/mise.toml)
  has native tests and explicit input,
  clipboard,
  rendering,
  and desktop-service inspection tasks.
- Trigger:
  the owning component,
  shared native libraries,
  Slint/runtime assets,
  or a compositor change affecting its consumers.
- Platform and route:
  bounded Linux build/test environments;
  graphical and desktop-service probes require their documented environment rather than an assumed generic sandbox.
- Output:
  nextest results,
  packaged-binary consumer assertions,
  guard-control failures,
  interaction evidence,
  and clean child-process shutdown.
- Completion:
  inspected task definitions only in this inventory;
  these are existing checks not measured in the Depot trial.
  The music-player pilot cannot establish their execution cost or behavioral completeness.

### Fuzzing, conformance, and expensive checks

- Tasks:
  existing sidecar `test:unit`,
  `fuzz`,
  `fuzz:coverage`,
  and conformance tasks where declared.
  [Logger workflow](../../.github/workflows/logger-fuzz.yml)
  and [TOML workflow](../../.github/workflows/toml-edit-fuzz.yml)
  already run bounded campaigns and coverage gates;
  TOML also runs conformance.
- Trigger:
  relevant runtime/sidecar/fixture changes for bounded smoke checks;
  dependency/toolchain changes and a weekly unchanged-input baseline for broader campaigns;
  explicit agent requests for experiments.
- Platform and route:
  keep existing GitHub smoke routes;
  use bounded Depot jobs for requested larger campaigns only after recording their cost and resource limits.
- Output:
  seed,
  budget,
  corpus/coverage reach,
  reproducer,
  and any failing input.
  Expensive tests involving real infrastructure require their own disposable-resource conditions.
- Completion:
  logger's ordinary unit result does not establish fuzz or browser-fuzz success.
  These workloads were inventoried but not executed for the Depot comparison.
  Root unit discovery deliberately excludes expensive tests;
  that exclusion must remain visible in completion summaries.

### Cross-platform and package-consumer contracts

- Tasks:
  retain the Windows music-player workflow's nextest,
  PE/linker,
  console-redirection,
  and launch checks;
  retain [`fs-id`](../../.github/workflows/fs-id.yml) host evidence
  and the semantic bridge's
  [`test:semantic-bridge-host` and `test:external-consumer-host`](../../.github/workflows/readonly-semantic-bridge.yml).
  Those host matrices cover Linux,
  macOS,
  and Windows where configured.
- Trigger:
  owning package,
  native bridge,
  packaging/export configuration,
  relevant dependency,
  and release candidate.
- Platform and route:
  preserve existing GitHub platform jobs initially.
  Depot runner placement requires separate workload evidence before changing these routes.
- Output:
  consumer import/execution from the built artifact,
  correct native binary loading,
  lifecycle diagnostics,
  and platform-specific assertions.
- Completion:
  the trial measured Linux routes and historical Windows overlap,
  not new Windows/macOS consumer runs for every package.
  A source-level test import is not evidence that a published artifact installs and works.

### Releases and repository policy checks

- Tasks:
  preserve the existing music-player,
  npm,
  Cargo,
  Kotlin,
  and pnpr publication workflows and their explicit release conditions.
  Add disposable packaged-consumer validation before any proposed routing change.
  Existing [music-player release workflow](../../.github/workflows/music-player-release.yml)
  distinguishes dry runs and release publication,
  and serializes releases without cancelling the running release.
- Trigger:
  explicit release request/version rules for publication;
  affected packaging changes for disposable validation.
  Keep [CodeQL](../../.github/workflows/codeql.yml),
  [Scorecard](../../.github/workflows/scorecard.yml),
  and [forbidden strings](../../.github/workflows/forbidden-strings.yml)
  as independently reported policy checks.
- Platform and route:
  existing GitHub release and policy routes.
  Trial builds contain no production publication credentials.
- Output:
  artifact hashes,
  target/architecture,
  consumer result,
  signing/metadata checks where applicable,
  and separate policy reports.
- Completion:
  Linux executable loading passed in trial builds;
  this is not complete release validation across platforms.
  Scorecard fixture replay exposed sequential history-query amplification and an unreserved admission interval.
  Moving that gate unchanged does not solve rolling-budget admission.
  [Trial evidence](../audit/depot-trial-2026-10-15.md#rolling-analysis-budgets)

## Change selection and proposed frequency

This is engineering judgment to validate against runtime and billing observations.
No recurring schedule is installed by this document.

- On the newest relevant push or pull-request revision,
  run the affected package's unit/type/lint checks and bounded existing fuzz/conformance checks.
  Select reverse consumers of shared libraries and shared configuration.
  Cancel superseded intermediate validation runs while preserving explicit release runs.
- On shared audio changes,
  select the core suite,
  desktop native/smoke checks,
  Android native build and JVM checks,
  and the applicable desktop/device integration suite.
  Neither application's tests replace the shared crate's own suite.
- On shared TypeScript libraries,
  lint plugins,
  bundler configuration,
  task utilities,
  or runtime changes,
  select their consuming packages and built-artifact checks.
  Lockfile changes must follow changed resolved dependencies;
  until reliable selection exists,
  choose the broader affected ecosystem rather than silently selecting no package.
- Once daily when relevant code changed,
  run a broader active-package sweep,
  including independently enumerated Rust and Gradle tasks,
  browser projects,
  and full integration suites whose environments are ready.
  Record missing environments separately from failures and passes.
  Use GitHub for this routine allocation unless measurement justifies Depot.
- Before release,
  run all supported target/architecture checks and consume the packaged artifacts.
  Run applicable full desktop and device suites against the release candidate.
  A nightly result from another revision does not certify a release artifact.
- Weekly and on demand,
  run bounded longer fuzzing,
  expensive fixtures,
  cold-cache/environment rebuilds,
  and selected dependency/toolchain experiments.
  Benchmark comparisons require unchanged-input repetitions and resource isolation.
- For agent-requested checks,
  accept an explicit source revision,
  existing task,
  selected platform,
  resource bound,
  and requested artifacts.
  Return logs and a terminal result using the same task definition as recurring CI.

The current filters already demonstrate some consumer selection:
Alpine includes `truepeak-core` and the compositor;
Windows includes `truepeak-core`.
They do not generally include root toolchain changes.
Logger/TOML workflow filters select package-specific files but omit the root lockfile.
The semantic-bridge filter includes `pnpm-lock.yaml` but still names
`package/ownership-markers/foreign-borrowed/**`,
while the active package is [`package/ownership-marker/foreign-borrowed`](../../package/ownership-marker/foreign-borrowed).
These are concrete selection gaps for a proposed routing design,
not claims that every affected historical change escaped testing.

## Reusable environments and costs

Separate prepared tool/dependency environments from revision-specific outputs.
Key reusable environments by OS/architecture,
toolchain and system-library inputs,
lockfiles,
and their preparation definition.
Restore source at the requested revision,
verify output identity,
and retain a cold reconstruction path.
Prepared environments must include the actual graphical/user-service prerequisites when their tasks need them.
The trial's successful Node snapshot and standalone sandbox execution establish concrete reuse routes.
Two fresh snapshot consumers also verified all 23 installed package versions and ImageMagick without installation.
Their absent user-session services remain a separate runtime preparation gap.
The Cargo source-timestamp failure demonstrates why a successful cache restore is not enough.
[Prepared-environment evidence](../audit/depot-trial-2026-10-15.md#prepared-environments-and-non-ci-work),
[cache correctness investigation](../troubleshooting/cargo-buildkit-source-timestamps.md)

Recommend Developer with intentional GitHub routing first,
Developer with selective overage second,
and Startup only when measured expanded demand justifies it.
Routing saves base subscription cost and retains requested cloud execution,
but requires allocation and usage tracking.
Selective overage offers more execution without immediately paying USD $200,
but charges vary and spending controls remain unverified.
Startup reduces allowance pressure,
but its USD $200 base leaves no room under the ceiling for overage.
Ranking:
routed Developer > selective-overage Developer > Startup,
because the user values the base-price saving and accepts hybrid routing;
selective overage outranks Startup while measured expanded usage remains cheaper.
[Cost observations and limitations](../audit/depot-trial-2026-10-15.md#monthly-plan-scenarios)

Do not treat current GitHub job counts as the target workload or the new suite's cost forecast.
Price each inventory area separately,
including standalone experiments,
storage/retention,
host-platform runs,
and reruns.
Track the union of unfinished relevant CI intervals for the user's elapsed-time valuation.
A faster Linux build may add no earlier overall completion when Windows finishes last;
new behavioral coverage and convenient experiments remain separate benefits.
The user supplied account usage and separate hard-start minute limits for all three compute products.
The current plan is Startup at USD $200,
with limits matching its included 5,000 container minutes and 20,000 minutes for each CI product.
The displayed storage values use different scopes and averaging;
an exact expiry timestamp and aggregate dollar cap remain unverified.

## Completion ledger and remaining evidence

- Measured passing:
  representative logger host checks,
  shared audio feature-matrix tests and Clippy,
  existing DOM browser interactions in Chromium and Firefox,
  Android JVM tests/lint,
  instrumented APK construction,
  Linux build/runtime checks,
  Alpine native/smoke checks,
  prepared Node environment,
  combined prepared desktop execution in two fresh consumers,
  Depot Tests reporting and timing-based splitting,
  and a controlled standalone linker workload with retained artifacts.
- Executed behavioral evidence:
  desktop cloud interaction in [38826827023][desktop-run] failed;
  a direct software-compositor run also reached the wide-session fixture timeout.
  Independent sessions subsequently exposed the chooser crash.
  The portal correction recovered all injectable checks in `38833901553`:
  117 passes,
  no failures,
  and one uninjectable check.
  Local device execution consumed [38826684750][android-apk],
  and the isolated test-context experiment subsequently passed all cases.
  The expanded logger/fs-path browser run executed 69 cases across three engines,
  with one WebKit assertion failure and one storage-fixture skip retained.
  The [prepared desktop follow-up](../audit/depot-prepared-desktop-2026-10-16.md)
  combines snapshot reuse with fresh user services;
  both consumers pass 117 checks and verify cleanup.
- Existing but unmeasured/unwired trial checks:
  remaining browser packages and production browser routing,
  native checks outside the measured shared audio core,
  other desktop applications,
  fuzz/conformance campaigns,
  expensive tests,
  and broader packaged-consumer/platform checks.
- Test coverage gaps requiring authoring or selection work:
  application user flows absent from selected browser suites,
  Android behavior beyond the existing fixture-backed instrumented cases,
  and package-consumer contracts not represented by a dedicated test.
  This inventory is not a branch-by-branch coverage audit of every package.
- Orchestration gaps:
  reverse-consumer selection,
  explicit discovery/skip reporting,
  corrected stale path filters,
  independently enumerated Rust/Gradle coverage,
  and measured daily/release workload costs.
- Operational gaps:
  aggregate billing visibility and spending controls,
  retention policy validated against actual storage,
  and rolling-budget admission if Scorecard execution moves.

Review the proposed triggers,
platform boundaries,
and completion criteria as the engineering recommendation.
The selected trial execution and cost replay are finished,
but the complete intended CI pipeline remains unmeasured.
Oxlint and the other applicable quality gates belong in that pipeline.
the [resumed cost report](../audit/depot-resumed-cost-2026-10-16.md)
includes revision-specific preparation,
expanded browser/native execution,
and test/standalone usage sensitivities.
The named unmeasured checks are tracked in the
[issue inventory](../handover/depot-trial-issues.md).
Further account inspection is intentionally excluded by the user's decision,
not unfinished trial work.
Production implementation and provider adoption remain a separate action request.

[android-apk]: https://github.com/Aquaticats/Monochromatic/actions/runs/38826684750
[desktop-run]: https://github.com/Aquaticats/Monochromatic/actions/runs/38826827023
[alpine-run]: https://github.com/Aquaticats/Monochromatic/actions/runs/38820372915
