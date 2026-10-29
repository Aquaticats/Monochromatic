# Complete CI accounting and completion audit

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/toml-edit` is now `package/toml-edit/core`.

## Correction and purpose

The earlier Depot trial completion claim was unsupported.
The [trial brief](../planning/depot-ci-trial.md) requires broader repository coverage,
code-quality checks,
consumer validation,
cache-boundary experiments,
and combined provider cost plus elapsed unfinished-CI time.
The representative jobs and their cost replay establish only their named subsets.
Account inspection was deliberately excluded by the user;
that exclusion does not remove execution or quality gates.

This audit defines the accounting contract for the complete execution manifest.
The [complete measurement checklist](../planning/depot-complete-ci-measurement.md)
owns the task inventory and routing.
This document does not choose a smaller representative set,
change production workflows,
or launch new cloud work.

The user clarified that checks run only for affected files or packages.
Complete CI therefore means every applicable check for the actual change,
not an unconditional repository-wide sweep on every push.
For Oxlint,
the user explicitly rejected file-scoped execution because custom rules require project context.
The diff selects affected packages and consumers;
Oxlint checks each selected package/project with its complete configured inputs.

## Evidence missing from the earlier completion claim

- No immutable revision has a reconciled result for every required build,
  code-quality,
  unit,
  browser,
  native,
  platform,
  and consumer check in the intended manifest.
  Results across independently modified trial revisions cannot be combined into one passing revision.
- The root [`lint` task](../../mise.no-env.toml) reaches package lint families,
  including Oxlint,
  TypeScript,
  Clippy,
  detekt,
  and OpenTofu where declared,
  then repository-wide dprint and monochromatic-lint.
  The earlier logger timing included build,
  types,
  and unit tests;
  it does not measure this quality-gate boundary.
- Root `buildAndTest` runs build and test phases but does not run `lint`.
  Root `test` dispatches TypeScript unit,
  browser,
  and end-to-end suites;
  it does not discover every native or Android package test.
  Unit discovery excludes expensive tests unless explicitly requested.
  Root `validate` changes files through `format` and is not an immutable-input verification substitute.
- Existing browser selection is bounded by its configured packages and engines.
  The prior storage run had a failure and a skip;
  Android's original device suite had failures.
  Neither successful setup nor a corrected subset removes those original results.
  The [coverage inventory](../planning/depot-expanded-coverage.md)
  records further native applications,
  infrastructure inspections,
  bounded fuzz/conformance,
  packaged-consumer checks,
  and platform routes without new execution evidence.
- There is no observed end-to-end timeline for the complete required-check graph,
  including queueing,
  preparation,
  artifact transfer,
  downstream consumers,
  and result delivery.
  Summing durations from unrelated jobs gives compute usage under assumptions,
  not the full pipeline's latency or critical dependency path.
- Prior cold/warm/source/dependency/toolchain experiments apply to the selected Alpine build.
  Its initial dependency and linker-change exports verified executable loading,
  not every intended consumer check.
  The sizing matrix's cold state emptied Cargo caches while permitting base-image and page-cache reuse.
  Fresh complete-pipeline cache-boundary evidence remains required.

The detailed source evidence is retained in the
[trial report](depot-trial-2026-10-15.md),
[sizing report](depot-runner-sizing-2026-10-15.md),
and [partial cost replay](depot-resumed-cost-2026-10-16.md).

## Minimum completion predicate

### Development execution evidence

The first collected external gate is
[GitHub run 38840751221](https://github.com/Aquaticats/Monochromatic/actions/runs/38840751221).
Its scanner job passed on the actual TOML change from
`cf9fcb48047d86e0af01ac9c73c1c7bf26fb277f` to `525d2e09f8168a3bdd7e513e26d4c80b363123e5`.
The log verifies the source commits and the single changed path,
`package/module/toml-edit/src/value-encoders.ts`.
The workflow revision is separately recorded as `0b8cc50c62d9b9b6f2dd853aa18a7eff85cc9e40`.
It downloaded and verified scanner `0.5.0` before checking out the historical workload,
whose manifest declares `0.4.1`.
This is a current-tool gate on immutable historical source,
not a historical-tool performance comparison.

The provider created the run at `2026-10-16T02:30:12Z`.
Job `116780030044` ran from `02:30:16Z` to `02:30:47Z`,
giving 31 job seconds and 35 seconds from provider run creation to job completion.
The standard public `ubuntu-latest` allocation has zero published compute charge;
the account ledger is outside this work.
The scan step's second-resolution metadata does not support a subsecond runtime claim.

The Depot preparation attempt `zrnlbjx4xr` in run `1v4l9tglbt` failed after 56.677 provider seconds.
The repaired preparation attempt `5947wjvb9c` in run `tg907s5kfs` succeeded after 102.819 provider seconds.
Both attempts remain setup usage;
neither is evidence that a selected validation check ran.
The [retained observations](depot-complete-ci-accounting-2026-10-16.evidence.json)
preserve their workflow revisions,
provider timestamps,
log hashes,
and scanner identity excerpts.
These separately dispatched development waves do not establish complete-pipeline latency.
The completed adapters still need a coherent graph replay with a frozen manifest.

Scorecard is also selected by pushes to `main`,
in addition to its schedule and branch-protection events.
The historical TOML source predates its rate-limit gate;
the current [workflow](../../.github/workflows/scorecard.yml) includes that gate.
Retain historical event policy separately from the current policy used by a disposable trial.
A measured current-policy denial does not establish that the historical unthrottled analysis ran.
Scorecard's remote repository analysis also observes live repository metadata,
so its observed commit and timestamps remain separate from an immutable historical package workload.

[Scorecard run 38841678032](https://github.com/Aquaticats/Monochromatic/actions/runs/38841678032)
executed the unchanged repaired gate under the intended current policy.
Its gate passed after inspecting 63 historical runs using one GraphQL request and one point,
then denied analysis because four runs had already executed in the last hour.
The gate job used six provider seconds;
provider creation to completion was nine seconds.
The analysis job was policy-skipped with no executed steps.
This completes this development cell's conditional gate disposition,
while leaving the prepared analysis adapter unexercised.
The [context investigation](../troubleshooting/scorecard-trial-context.md)
records the default-branch restriction,
explicit live-main adaptation,
resource bounds,
and separate historical-policy interpretation.

The TOML development run `hgqj2nstwq` emitted 24 package results:
16 passed,
five failed,
and three were deferred.
Its frozen manifest also required two external checks without results in that execution.
Four Oxlint checks and fuzz coverage failed;
these failures remain visible even though other tasks passed.
Every emitted result matched the final summary and frozen manifest identity.
The collector recovered five TypeScript completion records appended to compiler output without a separating newline;
their exact prefixes remain in the reconciliation evidence.
Task exits alone do not verify discovered case identities or consumer reach.

Attempt `811p86php8` consumed 229 provider seconds on 16 vCPUs,
or 3,664 vCPU-seconds.
The 242.564 seconds from request to observation is censored because required checks remained incomplete.
Its 42.555-second sum along recorded task dependencies excludes preparation,
the executor's additional serialization,
and missing checks;
it is not a measured complete-pipeline critical path.
The later manifest added Scorecard,
so this original 26-entry manifest cannot establish completion of that later graph.

The four executor controls in run `g57729x66d` produced the intended provider outcomes:
the positive control succeeded,
the explicit failure propagated,
and both narrowed Oxlint invocations were rejected.
The immutable workflow allocated two vCPUs and 8 GB per control.
Its emitted executor metadata incorrectly claimed 16 vCPUs and 64 GB;
accounting uses the actual workflow allocation.

For the native source revision `cb95cf6d785c3073c192b95f0632cdae7b910894`,
[run 38842242564](https://github.com/Aquaticats/Monochromatic/actions/runs/38842242564)
failed both Windows and macOS jobs while installing tools,
before native validation began.
The independent
[scanner run 38842263437](https://github.com/Aquaticats/Monochromatic/actions/runs/38842263437)
passed against all eight changed paths from base `45795584ff2dcf830dc505e121f0cc92c77860e5`.
It used 45 job seconds and 49 seconds from run creation to job completion.
The failed setup attempts remain usage observations and cannot count as native check execution.
The omitted configured Cargo installer is tracked in
[issue #1059](https://github.com/Aquaticats/Monochromatic/issues/1059).
Both hosts passed the corrected installer,
dependent-tool,
and immutable-source verification steps in
[retry 38842787230](https://github.com/Aquaticats/Monochromatic/actions/runs/38842787230).
That closes the prerequisite issue only;
package validation was still in progress,
with a separate failed Windows Clippy step,
at this observation boundary.

Depot native setup in run `5tbb7qftkw` failed at a separate boundary:
the historical Git wrapper's offline Cargo build could not resolve package `ignore`.
Attempts `91gqxrqdt5` and `8b6t6sndtd` used 142.971 and 142.946 provider seconds respectively,
each on 16 vCPUs and 64 GB RAM.
Neither log contains a started-check event.
These observations remain setup usage;
the missing dependency does not establish any native package test outcome.

TOML remaining-check run `d1jgjp62lg` also terminated without a complete validation result.
Its container job recorded four successful builds,
three failed container/mutation checks,
and 17 explicitly deferred checks.
The generated-file sibling failed during fixture setup and emitted no complete check summary.
The reconciler matched every emitted container-job result to the same frozen 26-entry manifest.
The two provider attempts consumed 1,222.476 summed seconds,
or 19,559.616 vCPU-seconds at 16 vCPUs each.
The 1,153.655-second request-to-observation interval remains censored.
These failures retain their usage and do not supply passing mutation or generated-file coverage.

### Active collection queue

All four source cases remain unfinished.
The retained case ledger names every dispatched development run;
it does not assign a complete-pipeline cost to any case.
The TOML container and mutation wave `d1jgjp62lg` ended with failures;
its corrected follow-up remains pending.
Native collection includes GitHub retry `38842787230`,
Depot core and Android run `j80v9dqfb4`,
and Linux desktop run `xg3znh2j0p`.
Logger scanner `38843463588` has been collected;
its package and consumer graph remains separate work.
Its three scanned paths exactly match the changed-file set from the immutable Git comparison.
Logger host run `38843901438` exercises filesystem and semantic consumers on Linux,
Windows,
and macOS.
Its filesystem checks retain Node `26.5.0` from the original workflow;
semantic checks retain source-lock Node `26.11.1` and pnpm `12.11.2`.
All six host jobs passed.
Their before/after source identities and both lockfile hashes match the immutable Git objects.
The filesystem probes verified preferred stable identifiers and explicitly unstable fallbacks on each host.
Semantic logs retain the bridge,
UTF-16 offset,
and packed external-consumer passing groups on each host.
The authored external-consumer test resolves fresh dependency ranges,
captures pnpm output,
and deletes its fixture;
these logs do not retain the resulting consumer lockfile or exact external dependency versions.
That evidence limitation is separate from the verified frozen workspace inputs.
Desktop collection covers Windows run `38843524267`,
Alpine run `38843525621`,
macOS run `38843527788`,
and scanner run `38843587917`.
The desktop scanner also passed,
and its six scanned paths exactly match the immutable Git comparison.
Both scanner jobs used 28 provider seconds,
with 32 seconds from run creation to completion;
these are individual observations,
not repeated latency measurements.
These independently dispatched observations still need immutable-manifest reconciliation
and a coherent graph replay before complete-pipeline latency or cost can be reported.

Both Alpine jobs passed with the same 1,461 executed test identities,
verified individually against their numbered nextest output and final summaries.
Each summary reports one skipped test without naming it.
The frozen package has one ignored real-library refresh benchmark;
the retained evidence distinguishes that source annotation from the unnamed runtime skip.
Both jobs also reported successful no-audio,
ALSA,
and PipeWire window scenarios,
including the PipeWire stream connection to the null sink.
Their full job durations were 318 seconds on ARM64 and 393 seconds on x86-64.

All three original Windows jobs failed the direct PowerShell file-redirection assertion,
after pipeline capture and the independent bash checks succeeded.
This reproduces the already documented historical result at the selected source;
the [GUI-subsystem decision](../decision/music-player-windows-gui-subsystem.md)
records the accepted limitation and later corrected workflow checks.
[Issue #1060](https://github.com/Aquaticats/Monochromatic/issues/1060)
is closed as that historical assertion finding,
not classified as a new application defect.
The original replay skipped nextest after the failed assertion.
Recovery run `38844610546` keeps the application and assertion unchanged,
while allowing independent tests after successful test compilation.
All three recovery jobs executed exactly the same 1,432 test identities,
with every numbered result passing and one unnamed skipped benchmark in each summary.
Their job durations were 353 seconds for MSVC x64,
456 seconds for GNU x64,
and 461 seconds for MSVC ARM64.
The jobs remain failed because the original PowerShell assertion remains failed.

The macOS desktop replay ran 1,020 of 1,433 selected nonignored tests:
1,017 passed and three watch assertions failed.
Nextest explicitly reports 413 tests left unexecuted after fail-fast.
The rename-target path mismatch is tracked in
[issue #1063](https://github.com/Aquaticats/Monochromatic/issues/1063),
the burst-write path and flag mismatch in
[issue #1064](https://github.com/Aquaticats/Monochromatic/issues/1064),
and the in-place-write mismatch in
[issue #1065](https://github.com/Aquaticats/Monochromatic/issues/1065).
These observations do not establish one shared backend cause.

Native-source replay `38842787230` retained separate earlier failures.
Windows test compilation failed with rustc `E0433` for the Unix-only CPU-time helper;
[issue #1061](https://github.com/Aquaticats/Monochromatic/issues/1061)
records the historical failure and existing fix `2ec128bebf`.
The macOS random change-plan fixture failed on case-colliding names;
[issue #1062](https://github.com/Aquaticats/Monochromatic/issues/1062)
records existing fix `95ec5a9da8` and the exact test passing in the later desktop replay.
Both historical issues are closed with the newer-source evidence,
without changing the failed replay outcomes.
The earlier macOS run executed 55 of 1,382 tests,
with 54 passes and one failure;
1,327 remained unexecuted.
Missing locked-toolchain Clippy components and fail-fast recovery belong to the harness,
separate from these source outcomes.

The [Android consumer evidence](depot-complete-android-2026-10-16.evidence.json)
matches all 15 test identities independently derived from the frozen Kotlin source and its hashes.
The local consumer run took 6.041 seconds,
with 13 passes,
two retained `SessionStoreTest` failures,
and no skips.
Its APK producer is Depot run `j80v9dqfb4`;
local consumer time is not added to provider compute usage.

The native feature-only run `79lgpx91sf` passed every numbered test in its default,
fingerprint,
and service configurations:
58,
67,
and 71 distinct identities respectively,
with no skips.
Each retained check event matches its artifact,
source revision,
manifest hash,
and exact nextest arguments.
Provider stdout and stderr arrived out of execution order;
verification therefore matches complete-suite identities and summaries independently of delivery order.
The separate all-features core test passed in native wave `j80v9dqfb4`.
That wave still contains failed Clippy and Rust-style checks.

Linux desktop waves `lvcwx21cbt` and `ssvlvk57k4` retain their main manifest results separately
from supplemental E2E results.
Both supplemental E2E attempts failed a harness assertion before runtime verification completed,
and both report successful cleanup.
Those results do not establish application-window behavior.
The retained usage ledger includes these waves,
the feature matrix,
WireGuard,
mutation identity,
Pi,
logger unit execution,
scheduler controls,
native attempts,
and the VM bootstrap retry.
Each allocation is verified against its exact workflow source.

Later Linux repeats `nb6ptmwjs5` and `xmvgm200k5` passed all 1,406 and 1,460 nonignored tests respectively,
with one unnamed skip each.
Every numbered identity and the retained per-check log hash was verified.
The later desktop pass does not establish a fix for the earlier watch failures;
the unchanged source produced different outcomes.
The original rescan,
idle-queue,
and extra burst-report failures are tracked separately in
[issue #1079](https://github.com/Aquaticats/Monochromatic/issues/1079),
[issue #1080](https://github.com/Aquaticats/Monochromatic/issues/1080),
and [issue #1081](https://github.com/Aquaticats/Monochromatic/issues/1081).

Both later E2E invocations still timed out during the first session.
Their systemd journals report that session's service exiting with status 101,
but the downloaded artifacts omit the scratch session log that contains the application and compositor output.
The harness copied `target/e2e`,
while the frozen session implementation writes its log under the separate scratch run directory.
The result's `sessions` array lists configured sessions;
it is not evidence that those sessions executed.
Similarly,
`runtimeCompleted: true` records a returned invocation,
not successful user-flow coverage.
The failure needs retained session diagnostics before assigning a product cause.

### Logger static accounting

Depot run `qm7w64tmhj` used workflow revision `0886b2669dc9696b60493870f172d77269b6e854`
and frozen logger source `ef01490f5e960f8ca0b424c12242caeb3c5a2b5e`.
Its manifest contains 737 required checks:
153 passed,
171 failed,
394 were skipped,
and 19 external checks have no result in this wave.
The graph remains incomplete.

The provider artifact retains all 718 emitted result identities.
Every one of the 547 retained log completion events exactly matches its artifact result;
171 completion events and the aggregate summary are absent from the downloaded log.
The artifact supplies those missing records,
without establishing why the log lacks them.
The retained evidence records both artifact and log hashes.
Child output shares the concurrent workers' streams without per-check identifiers,
so a relative diagnostic path cannot always be attributed to one check.

The verified allocation was 16 vCPUs,
64 GiB memory,
four workers,
and a 30-minute job limit.
The provider attempt used 357.317 seconds and 5,717.072 vCPU-seconds;
the request-to-finish observation spans 370.518 seconds.
These measurements describe this development attempt,
not complete CI latency or account billing.

The accounting adapter now distinguishes affected-owner test-file selection from changed-file lint selection.
It verifies explicit test arguments against expected files and affected owners,
retains full-project Oxlint checks,
and ignores unexecuted skip timestamps when evaluating dependency execution order.
Positive and negative fixtures exercise those selection boundaries.

Logger unit wave `xz7cchwbsk` retains 718 emitted checks:
211 passed,
15 failed,
and 492 were skipped.
All emitted check identities match the artifact and downloaded logs;
19 external manifest checks remain outside this wave.
Its provider attempt used 237.566 seconds and 3,801.056 vCPU-seconds.
The manifest predates the corrected build-prerequisite graph,
so failures and skips cannot be treated as a completed source validation.

VM run `99tl7zjfgp` at harness revision
`a0785c49e68573be8a806af33c50ab1fb1d1c7ca`,
using the same frozen logger source,
has been collected with its artifact.
Its attempt used 258.417 seconds and 4,134.672 vCPU-seconds at the verified 16-vCPU allocation.
Source and lockfile hashes match immutable Git objects before and after execution.
The original package task failed on a relative helper path;
the separate private-environment experiment failed on a Unix socket path length before guest runtime.
Domain lists before preparation and after cleanup were empty,
and the lifecycle record reports no cleanup errors.
Bootstrap retry `8wfm7bfgjv` failed before checks because its required `XDG_RUNTIME_DIR` was absent.
Its 148.819 provider seconds and 2,381.104 vCPU-seconds remain in the usage ledger.
Replay `7gxcgml68r` started private libvirt successfully,
then guest preparation failed because the QEMU monitor socket path under `XDG_CONFIG_HOME` was too long.
Its original package task still failed on the relative helper path.
The artifact confirms unchanged source inputs and empty private domain lists after cleanup.
Its provider attempt used 165.155 seconds and 2,642.480 vCPU-seconds.
Replay `rv6hfj76w9` booted the template guest:
its guest agent became ready,
the guest shut down,
and the standalone template image was saved.
Final guest startup then failed when QEMU could not connect to the virtiofs socket.
Cleanup destroyed the defined private guest and the final domain list was empty.
This attempt used 189.342 provider seconds and 3,029.472 vCPU-seconds.
The next diagnostic replay must retain libvirt and virtiofs logs before cleanup.
Native recovery runs `38847322477` and `38847322559` also remain pending collection.
After diagnostic issue filing settles,
the issue-ledger totals must be reconciled against the explicit issue IDs before updating them.

### Acceptance requirements

Freeze one source revision and the intended manifest before each execution cell.
Freeze the comparison base as well,
retain the actual changed-file list,
and record affected packages,
reverse consumers,
shared configuration/toolchain effects,
selected checks,
and explicit exclusions for unrelated packages.
The manifest contains the complete applicable graph for that change.
Positive controls must select necessary owning-package and consumer checks;
negative controls must exclude unrelated packages.
An Oxlint control must also reject narrowed changed-file arguments,
even when the changed package was selected correctly.
Every required manifest entry names its existing task,
platform and architecture,
feature configuration,
dependency checks,
consumer artifact,
selection reason,
and acceptance evidence.
Inventory omissions remain open work;
an unsupported platform is routed to its supported executor rather than silently removed.
Publication itself stays outside the disposable trial.

Reconcile manifest identities against discovered tasks and actual emitted results.
Each required check must execute against the frozen inputs and retain its exit status,
diagnostics,
discovered case identities,
unexpected skips,
and consumer evidence.
Zero cases,
cached reports from another execution,
and provider-level success with missing child results do not establish coverage.
Existing intentional ignored cases retain their explicit identity and rationale.

Report both terminal execution and accepted validation.
A real failed check can finish execution without validating the revision.
Completed execution/accounting means every eligible immutable-source command has a terminal attempt
or a verified failed-prerequisite disposition,
with independent checks still executed and all attempts reconciled.
Historical source failures remain failed outcomes;
they do not require product fixes merely to complete the trial's measurement.
Harness omissions,
deferred waves,
wrong execution environments,
and unverified dependency blocks remain incomplete execution.
The current reconciler's `accepted` field means passing validation,
not completion of a failed experiment.
Its terminal-execution field does not yet award completion credit to prerequisite-blocked records;
those require explicit verified evidence in the final reconciliation.
A skipped,
cancelled,
or absent required check leaves validation incomplete.
Successful-pipeline latency requires a complete passing replay;
a completed failing experiment establishes only its observed failed-pipeline timeline.
Admission policy creates conditional requirements:
a verified gate denial can finish the policy check without selecting its analysis child.
The reconciler requires the approved policy revision,
gate script hash,
successful observed gate result,
decision timestamp,
denial reason,
and matching unexecuted child disposition.
A generic skipped result cannot satisfy that contract.
An allow decision requires actual analysis evidence,
including its observed repository commit and check identities.
Do not allow fail-fast cancellation or ignored child errors to turn missing coverage into a shorter successful pipeline.
Negative controls must demonstrate propagation through the same task,
workflow,
and result-collection path used for the complete run.

## Collect one coherent timeline

Record the request timestamp,
immutable revision,
workflow hash,
manifest hash,
resolved tool versions,
lockfile hashes,
environment-image digest,
cache namespace and provenance,
and resource allocation.
Record artifact hashes at both producer and consumer boundaries.
Source-specific preparation belongs to that revision's dependency graph;
reusable tool/dependency preparation has its own identity and refresh accounting.

For every provider run,
retain the run record and every workflow,
job,
and attempt identity.
Retries and cancelled attempts remain in usage totals.
Fetch logs by attempt identity;
a job-level latest-attempt lookup can hide earlier failed work.
Retain provider timestamps and in-task start/end events for preparation,
download,
cache restoration,
compilation,
quality checks,
behavioral validation,
artifact upload,
and cache saving.
Use monotonic durations within one process and UTC timestamps for the cross-job timeline.
Record clock uncertainty rather than treating cross-host millisecond differences as exact causality.

The [Depot CLI reference][depot-cli]
documents status,
attempt metrics,
log export,
and artifact metadata.
The read-only collector was exercised against completed run `wf9xzbwfph`
at revision `b1d360002d08b38eeddedaec1bd8b235dc296c83`.
It retained both attempts,
their metrics,
and parseable JSONL logs,
and checked the expected revision.
This verifies collection transport;
the old run is not a complete-pipeline measurement.

The GitHub collector likewise reads every run attempt's job pages,
checks the expected revision,
and retains job/step timestamps and completed-job logs.
It was exercised against Windows run `38778131811`,
retaining all three jobs at revision `29175e0d768d679fa9f3ea4d0ea8183cd871fb24`.
Raw log retrieval follows the existing
[GitHub CLI workaround](../troubleshooting/gh-completed-job-logs.md)
and captures escape-containing bytes privately.

On 2026-10-16,
the repository API reported public visibility.
The standard `windows-2025`,
`windows-11-arm`,
`ubuntu-latest`,
and `ubuntu-24.04-arm` labels used by the selected desktop workflows
have four CPUs and 16 GB RAM in public repositories.
GitHub publishes free standard-runner execution for public repositories.
Record those observed labels,
their documented widths,
and zero published compute charge;
do not substitute private-repository or larger-runner pricing.
This does not establish storage charges or an account invoice.
[GitHub runner specifications][github-runners]

The logger host matrix also uses `windows-latest`,
`macos-latest`,
and `macos-15`.
The same official specifications give Windows four CPUs and 16 GB RAM,
and those ARM macOS labels three CPUs and 7 GB RAM.
The separate `macos-15-intel` native jobs have four CPUs and 14 GB RAM.
All these observed standard labels retain zero published compute charge in this public repository.

## Keep usage and elapsed time separate

Compute usage is the sum of distinct provider attempt intervals,
weighted by their actual resource sizes and product rates.
A shared job containing build and test is charged once,
even when both checks reference that attempt.
Parallel jobs are charged separately.
Include failed setup,
retries,
post-task cleanup,
uploads,
and cache-save work in their observed provider intervals.
Record startup and teardown discrepancies between sandbox and job boundaries when available.
These are measured execution proxies unless reconciled with a billing ledger;
account inspection is excluded from this trial.

End-to-end time starts when the immutable revision's required validation is requested,
not when its slowest job begins.
It ends only when the final required result is available.
Queueing,
dependency waits,
artifact handoff,
and delayed dispatch remain visible.
Also report the longest dependency path through measured check execution durations.
That graph calculation explains serialization,
but excludes queueing and cannot replace the observed end-to-end interval.

For the user's elapsed-time valuation,
merge all intervals during which at least one relevant required check remains unfinished.
Overlapping jobs and revisions contribute once to that union.
The provider-attempt union is a separate diagnostic:
it omits queued work and is not the user's complete waiting interval.
A required check that never executes is censored at the observation boundary,
not assigned zero duration.
A failed executed check has a terminal failure time and cannot supply a successful completion baseline.
If a failure prevents downstream checks,
report the missing checks and the censored complete-pipeline result.

## Repeated inputs and cache boundaries

Establish unchanged-input spread for the complete manifest,
not just the dominant compiler command.
Use independent fresh consumers with the same revision,
tools,
resource size,
environment digest,
and declared cache state.
Retain each observation and its minimum,
maximum,
median,
and range before comparing a proposed alternative.
A pair of repetitions exposes observed spread but does not establish a population distribution.
Interleave alternatives where practical and retain allocation order.

The cache experiments distinguish these boundaries:

- Repository-cache cold:
  empty source build outputs,
  incremental artifacts,
  dependency download caches,
  and package stores for the declared experiment.
  Identify any retained OS image,
  installed tools,
  provider image layers,
  or host cache;
  do not describe that state as physically cold hardware.
- Warm unchanged input:
  fresh execution using declared reusable caches and the identical manifest.
  Runtime services still start in each consumer.
- Source change:
  a new immutable revision with a consumer-visible change,
  source/cache-key evidence,
  and a control proving the old artifact cannot satisfy the new assertion.
- Dependency change:
  a new lockfile revision,
  resolved dependency/version evidence,
  and complete affected-consumer results.
- Toolchain change:
  a changed resolved tool or compiler/linker configuration,
  a new environment/cache identity,
  and complete affected-consumer results.

Each changed-input cell has its own immutable revision or input fingerprint.
Its unchanged repetitions share that fingerprint.
Do not average cold and warm states,
source and dependency changes,
or failed and passing coverage into one runtime baseline.
Amortize reusable environment preparation only at an explicitly modeled refresh frequency.

## Cancellation and historical replay

Replay the complete required-check graph,
with observed offsets and dependencies,
against the selected change history.
Do not run every stage from time zero or charge consumers whose prerequisites never finished.
Preserve shared preparation and repeated attempts exactly once in the provider usage model.
State whether overlapping revisions share caches,
preparation,
or concurrency limits;
independent-workload cancellation is not a measured full-pipeline schedule.

A newer relevant revision can supersede an intermediate revision under the agreed policy.
Record when supersession occurs,
when cancellation is requested,
and when each provider attempt actually stops.
Time after the cancellation request is still consumed compute.
The newer revision remains relevant until its required checks finish;
superseding an older revision cannot erase that unresolved interval.
Daily selection also includes the delay from a relevant request to its scheduled validation.
It cannot claim lower waiting cost by starting the accounting clock at the daily run.

Measure cancellation delivery and cleanup on a disposable full graph before using them as observed parameters.
Until then,
zero-lag cancellation is a labeled optimistic scenario,
not a result.
Report selected,
completed,
failed,
cancelled,
and unstarted checks separately.
The monthly forecast must replay the actual affected-package selector,
not charge a full repository sweep for every revision.
It must include every intended trigger family and its explicit routing,
plus scheduled,
release-validation,
retry,
and agent-requested work.

## Resource and collector verification

Root package fanout uses `availableParallelism()` workers.
Each package's task fanout can launch child tasks concurrently;
root `lint` limits Oxlint and TypeScript threads to one per child.
These mechanisms are visible in [`mise.no-env.toml`](../../mise.no-env.toml).
Do not infer a process or memory bound from CPU count alone.
Record effective CPU quota/affinity,
memory limits,
peak memory,
child concurrency,
and OOM events in the complete run.

The earlier sixteen-CPU native-build result supports that allocation for its measured compilation workload.
It does not establish an optimum for full repository lint or mixed nested task fanout.
Use explicit per-job resource bounds,
retain existing task-level thread controls,
and measure the complete graph before recommending a cheaper or larger runner.

The scratch accounting harness separates raw collection from normalized manifest reconciliation.
Its positive control distinguishes 52 summed provider seconds,
30 request-to-result seconds,
29 seconds of active-provider union,
25 seconds on the execution dependency path,
and 150 vCPU-seconds.
Fixtures also verify missing and cancelled checks,
terminal failures,
retry charges,
shared-job deduplication,
and rejection of duplicate attempts,
mixed revisions,
unknown checks,
and dependency cycles.
Selection controls reject an empty manifest,
missing selected checks,
unrelated packages,
and either declared or observed Oxlint changed-file narrowing.
An incomplete canonical manifest also prevents acceptance,
even if every emitted package result passes.
External gates remain required entries rather than disappearing during adapter work.
The selector's discovery algorithm still needs independent positive and negative controls;
the accounting reconciler validates its frozen output rather than inventing the affected set.
The [retained verification evidence](depot-complete-ci-accounting-2026-10-16.evidence.json)
records these checks and the exercised collector.

Scoped Markdown lint passed,
and the report and issue comments were rendered with Pandoc and inspected.
The scope correction is attached to
[issue #1030](https://github.com/Aquaticats/Monochromatic/issues/1030#issuecomment-6206595590).
The unsupported completion claim is attached to
[issue #572](https://github.com/Aquaticats/Monochromatic/issues/572#issuecomment-6206595886),
including a proposed tightening of the existing completion rule.
That rule proposal is not applied.

[depot-cli]: https://depot.dev/docs/cli/reference/depot-ci
[github-runners]: https://docs.github.com/en/actions/reference/runners/github-hosted-runners
