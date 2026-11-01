# Depot trial results, October 15, 2026

## Status and decision being tested

The compilation measurements and runner-size follow-up have finished.
The resumed execution now covers Depot Tests,
combined prepared desktop execution,
representative browser suites and native feature checks,
and a controlled standalone linker workload.
The [completion record](../handover/depot-trial-grilling.md#completed-trial-execution-and-remaining-acceptance-gaps)
links each executed result and distinguishes production integration from evaluation.
The overall evaluation is not complete:
the intended CI pipeline,
including Oxlint and the other quality gates,
has not yet been executed and measured as a whole.
The earlier completion claim was incorrect.
Further account access is deliberately excluded by the user's decision;
that decision is separate from the missing pipeline measurement.
All ten corrected runner configurations passed,
as did the default-container-builder concurrency repetitions.
The [sizing report](depot-runner-sizing-2026-10-15.md) records the measured results and allocation recommendation.
Full desktop interaction and Android instrumentation have now executed.
The Android test-context experiment passed all 16 cases;
the recovered desktop run passed 117 checks with no failures and one uninjectable check.
Two fresh Depot CI consumers verified installed-package snapshot reuse;
the chooser recovered after correcting the private portal's deprecated `Read` response.
The user supplied the account's usage and selected hard-start limits.
Storage views still need their different scopes reconciled.
This report follows the [confirmed trial brief](../planning/depot-ci-trial.md).
The user prefers the USD $20 Developer plan if intentional GitHub routing keeps it useful and convenient.
USD $200 is the monthly ceiling,
not a target spend.
The current Startup trial remains available for evaluating options.
No subscription change or production migration has been made.

The user's clarification is explicit:
this is not an apples-to-apples migration.
The objective is broader CI coverage,
reusable environments,
agent-requested experiments,
and a convenient workflow within the spending ceiling.
Existing-job timings isolate performance effects;
they do not price the value of additional capabilities or define the target workload.
The [confirmed brief](../planning/depot-ci-trial.md) preserves that distinction.

The [Depot Test trial](depot-test-feature-2026-10-16.md) verifies reporting,
failure and skip details,
same-commit retry history,
analytics,
and timing-based splitting with real nextest output.
Its final split returns 374 passes and one existing ignored case,
with every expected identity present exactly once.
Repeated same-key upload duplicates rows and needs integration care.
The [expanded native/browser runs](depot-expanded-execution-2026-10-16.md)
pass native feature checks,
Clippy,
and DOM interactions;
the logger/filesystem browser run retains one WebKit failure and one skip.
The [standalone linker experiment](depot-linker-experiment-2026-10-16.md)
completes with retained inputs and verified outputs,
but its timing difference is smaller than baseline variation.

Developer with selective routing is the recommended arrangement for that expanded workload.
This is a provisional allocation recommendation,
not a complete technology-adoption verdict.
The container build,
Depot CI,
and standalone sandbox routes have executed real work.
The Depot GitHub runner retry also completed the same build and runtime check.

## Compilation and cache behavior

The later package snapshot trial verified all 23 requested package versions and ImageMagick
in two fresh consumers without apt installation or downloads.
The snapshot preserves files;
both consumers still lacked a working user session bus and PipeWire service.
That filesystem-only experiment established reusable installation.
The subsequent [prepared desktop trial](depot-prepared-desktop-2026-10-16.md)
combines reused packages and built artifacts with fresh real user services.
Both consumers pass 117 checks,
retain one uninjectable check,
and verify cleanup.
[Package reuse evidence](depot-expanded-cost-2026-10-15.md#reusing-installed-desktop-dependencies)

The later runner-sizing experiment exposed a correctness gap in the original cache design:
replaying changed source through an older BuildKit layer can return an unchanged Cargo executable.
The runtime-name guard rejected this output in run `38819692881`.
The individual successful observations remain valid,
but they do not establish safe cache reuse across repeated source snapshots.
Trial commit `4648e5ad3d` adds source-content invalidation of the local packages;
the full corrected reruns passed all 100 runtime assertions,
including both source changes in every configuration.
[Main rerun](https://github.com/Aquaticats/Monochromatic/actions/runs/38822116503),
[large-runner rerun](https://github.com/Aquaticats/Monochromatic/actions/runs/38822116510)
The [cache investigation](../troubleshooting/cargo-buildkit-source-timestamps.md)
preserves the minimal failing case and its verified package-clean remedy.

These are observed executions,
not vendor benchmark claims.
The disposable branch is `trial/depot-20261015`,
based on application revision `46effff65ab4193b4311d7675e301c97007176d1`.
Its trial files live under `package/music-player/desktop-app/depot-trial/`.
The same Alpine toolchain and runtime stages feed the GitHub and Depot builds.
The initial provider comparison uses four Cargo build jobs in each.
Depot's default builder has 16 CPUs and 32 GB RAM;
this is not a comparison of identical hardware.

- Depot container build `ws40v1c9xt` finished in 468 seconds,
  including export to the local machine.
  Cargo reported 7 minutes 13 seconds.
  Unchanged Cargo-input builds `t66qp9xk3l` and `5cfbvsmxz9` finished in 21 and 22 seconds.
  The first warm Cargo invocation itself took 0.31 seconds.
  [Depot build](https://depot.dev/orgs/l4zncjjbj5/projects/72lp3rlfvw/builds/ws40v1c9xt)
- GitHub's reference cold build took 869.508 seconds,
  with Cargo reporting 14 minutes 14 seconds.
  Same-job warm repeats took 1.975 and 1.716 seconds.
  Their artifacts stayed on the runner;
  these export times are not comparable to downloading the Depot artifact locally.
  [GitHub reference run](https://github.com/Aquaticats/Monochromatic/actions/runs/38813764705/attempts/1)
- Depot's four-CPU GitHub runner retry took 757.394 seconds for the cold build,
  with Cargo reporting 12 minutes 24 seconds.
  Same-job warm repeats took 1.173 and 1.093 seconds.
  The cold difference from GitHub was 112.114 seconds in these individual observations.
  Both runner labels request four CPUs;
  that does not establish identical processors or identical infrastructure.
  The original Depot job failed with GitHub's lost-communication annotation before recording steps.
  The successful retry does not explain or fix that failure.
  [Runner retry](https://github.com/Aquaticats/Monochromatic/actions/runs/38813764705/attempts/2),
  [reliability record](../troubleshooting/depot-github-runner-communication.md)
- Source-change build `jbld9m545r` took 226 seconds overall,
  with Cargo reporting 3 minutes 17 seconds.
  The exported executable displayed the deliberately changed CLI name.
  The marker was then restored in commit `d5ab91d795`.
  [Source-change build](https://depot.dev/orgs/l4zncjjbj5/projects/72lp3rlfvw/builds/jbld9m545r)
- Dependency-change build `41xd3lwh8d` finished in 382 seconds overall.
  Cargo regenerated its disposable lockfile from `log` version `0.4.34` to `0.4.33`
  and compiled in 5 minutes 23 seconds.
  The exported lockfile confirmed the changed dependency and the executable passed its runtime check.
  The checkout's lockfile stayed unchanged.
  [Dependency-change build](https://depot.dev/orgs/l4zncjjbj5/projects/72lp3rlfvw/builds/41xd3lwh8d)
- Toolchain-change build `14trfrrxg6` finished in 473 seconds overall.
  Cargo compiled in 7 minutes 12 seconds with the added `mold` linker flag.
  The executable's ELF comment identified `mold 2.39.1`,
  its runtime version check passed,
  and the exported lockfile returned to `log` version `0.4.34`.
  [Toolchain-change build](https://depot.dev/orgs/l4zncjjbj5/projects/72lp3rlfvw/builds/14trfrrxg6)
- Failure-control build `wk649mjc2s` returned nonzero when the post-verification stage required failure.
  This verifies propagation of a failed build step.
  It does not prove that every application defect is tested.
  [Failure control](https://depot.dev/orgs/l4zncjjbj5/projects/72lp3rlfvw/builds/wk649mjc2s)

Each successful export ran `music-player --version` inside the production Alpine runtime stage.
That proves basic executable loading,
not graphical interaction or audio correctness.
The warm repetitions establish observed spread of one second for the Depot local-export route
and 0.258 seconds for the GitHub same-job route.
The Depot GitHub runner's warm spread was 0.081 seconds.
There is only one successful cold sample per route;
the cold difference is promising but does not establish repeatability.

The linker probe changes compiler flags and recompiles dependencies;
it is not a controlled `lld` versus `mold` benchmark.
The final build measured 5,719,616 KiB in its Cargo target cache
and 700,660 KiB in its Cargo download cache.
These are filesystem measurements inside that build,
not account-wide billable storage.

## Runner sizing follow-up

The corrected matrix averaged 526.7 seconds cold at four CPUs/four jobs,
320.4 at eight/eight,
246.6 at sixteen/sixteen,
212.0 at thirty-two/thirty-two,
and 323.4 at sixty-four/sixty-four.
Sixteen CPUs with only four Cargo jobs averaged 476.6 seconds.
The measured benefit therefore requires increasing compilation concurrency as well as allocating CPUs.
The sixteen/sixteen configuration has the lowest cold-build combined cost at the user's time valuation.
Source rebuilds put eight/eight and sixteen/sixteen within one US cent of combined cost,
an unresolved economic tie given the observed spread.
See the [sizing report](depot-runner-sizing-2026-10-15.md)
for every configuration,
raw evidence,
cost method,
and processor-model limitations.

The separate default container builder already has sixteen CPUs.
With sixteen Cargo jobs,
its two cold observations took 268.558 and 265.417 seconds,
including artifact download;
all unchanged repetitions and runtime assertions passed.
The original four-job cold observation was 468 seconds,
but had no repeated cold baseline and predates the cache fix.
These results support using more concurrency on the existing builder before paying for a larger allocation.
[Container evidence](depot-container-concurrency-2026-10-15.evidence.json)

## Existing checks newly exercised through Depot CI

The existing Alpine task also passed on a four-CPU Depot GitHub runner in
[run 38820372915](https://github.com/Aquaticats/Monochromatic/actions/runs/38820372915).
Nextest reported 1,461 passing tests and one skipped test.
The graphical smoke task drew the application window for no-audio,
ALSA,
and PipeWire scenarios,
then confirmed the PipeWire stream was linked to its null sink.
It finished with `SMOKE OK`.
This exercises the existing smoke checks,
not every desktop interaction or Android instrumentation.

Run `wjr4cmpjvs` passed the logger's build,
type check,
and unit suite,
plus all 80 audio-core tests with every feature enabled.
The run took 152 seconds from creation to completion.
Audio's job interval was 136.452 seconds on four CPUs;
logger's was 45.918 seconds on two CPUs.
These include setup and compilation rather than test execution alone.
[Coverage run](https://depot.dev/orgs/l4zncjjbj5/workflows/wjr4cmpjvs)

Negative-control run `2clfb5npj3` changed one audio test's expected value inside its disposable sandbox.
It reported 79 passing tests and one failing test,
named `window::tests::window_frames_floors_and_clamps`,
with actual value `192000` and expected value `192001`.
The complete CI run failed as intended.
The fixture restored the remote file in cleanup and never changed the checkout's test source.
[Real-test failure control](https://depot.dev/orgs/l4zncjjbj5/workflows/2clfb5npj3)

Android run `3tvq2wvskh` passed JVM tests and Android lint in 218 seconds overall.
The follow-up inspected 127 XML suites containing 1,289 test cases.
Their timestamps identify the original execution;
the repeated Gradle task restored those results from cache.
The repeat's Android job took 100.149 seconds on four CPUs.
No instrumented device coverage is claimed.
[Original Android run](https://depot.dev/orgs/l4zncjjbj5/workflows/3tvq2wvskh),
[counted repeat](https://depot.dev/orgs/l4zncjjbj5/workflows/jp9w8gw62r)

The cloud prerequisite probe found KVM but no display session,
GPU device,
running systemd user manager,
PipeWire tools,
or Weston executable.
The desktop suite's documented prerequisites include a graphical session,
user services,
and isolated audio routing.
The Android emulator workflow also requires a headed host-GPU launch under the repository's emulator skill.
That initial probe did not run desktop interaction or Android instrumentation.
The separate Alpine graphical smoke scenarios did pass as recorded in this section.
This is a gap in the tested environment,
not proof that Depot cannot host a suitable environment.
[Prerequisite run](https://depot.dev/orgs/l4zncjjbj5/workflows/jp9w8gw62r),
[desktop suite requirements](../../package/music-player/desktop-e2e/README.md)

The later Depot GitHub runner probe `38826610100` found a running systemd user manager and sudo access.
Run `38826827023` then provisioned PipeWire,
ImageMagick,
and graphics dependencies,
built the existing app and compositor,
and ran the keyboard suite through software rendering.
Keyboard navigation,
playback,
clipboard,
and private-sink audio checks passed;
pointer/text checks failed and the wide session aborted before later scenarios ran.
This is executed but failing coverage,
not a completed passing desktop suite.
[Desktop interaction run](https://github.com/Aquaticats/Monochromatic/actions/runs/38826827023)

Subsequent trial run `38831098294` ran each session independently with direct software rendering.
Keyboard selection of the thirty-track fixture recovered the wide session.
The terminal results were 116 passes,
one folder-dialog failure,
and one held-modifier case that cannot be injected.
The wide,
narrow,
auto-play,
quit,
state,
and visual sessions passed;
the chooser session passed its focus-return check but failed to show a dialog.
Zenity was installed in this run,
so package absence is not a confirmed explanation for the remaining failure.
The full sixteen-CPU job took 441 seconds,
including provisioning and builds;
the interaction step took 199 seconds.
These are single-run observations and do not establish timing variance.
[Full-session run](https://github.com/Aquaticats/Monochromatic/actions/runs/38831098294)

Transparent subprocess capture in `38832874391` then recorded Zenity crashing with `SIGSEGV` after 163 ms.
GLib reported a variant-format mismatch:
`v` was expected but `u` was received.
The private appearance portal's deprecated `Read` method currently returns the same shape as `ReadOne`,
while the XDG contract documents an additional variant layer for `Read`.
Exact signature assertions failed in four of 270 tests before the correction.
The corrected implementation passed all 270 tests and Clippy.
Cloud run `38833901553` then passed all seven desktop sessions:
117 checks passed,
none failed,
and one held-modifier check remained uninjectable.
The chooser opened and returned keyboard focus after cancellation;
Zenity exited with cancellation status `1` and no signal.
The full job took 410 seconds and its interaction step took 185 seconds.
These changed inputs do not establish a speedup or unchanged-input timing spread.
The original failures remain retained separately,
and this compositor correction remains on the trial branch.
[Chooser issue and diagnostic](https://github.com/Aquaticats/Monochromatic/issues/1004#issuecomment-6205699992),
[Settings contract](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Settings.html)

The [successful recovery evidence](depot-desktop-recovery-2026-10-15.evidence.json)
retains every check and the complete job interval.
The passing chooser still emitted accessibility-bus,
dconf,
and transient-parent warnings;
passing interaction assertions do not establish those integrations.

Android artifact run `38826684750` built both native ABIs and the app/test APKs successfully.
Those APKs installed on an isolated local headed Android 37 x86_64 emulator with 16 KiB pages.
The unfiltered instrumentation invocation reported 16 cases and two failures in `SessionStoreTest`,
both dereferencing a null application context in `SessionStore.prefs`.
The raw initial result was nine passes,
five assumption skips,
and two failures.
After granting audio permission and indexing synthetic ten-second FLAC,
Opus,
and MP3 tracks,
all sixteen cases executed:
fourteen passed,
two failed,
and none skipped.
The native decoder exercised all three indexed formats;
playback advanced,
paused at a stable position,
sought from zero to the middle of the track,
and advanced exactly once at natural end.
The two storage tests still failed with their original context.
The isolated test-context experiment changed both tests to use instrumentation `targetContext`.
Cloud run `38828426717` built those app/test APKs in a 195-second job;
the build task reported 139 seconds.
All 16 instrumented cases then passed with no skips in 5.925 seconds at the client boundary
(5.115 seconds reported by JUnit).
This establishes the context problem for the observed failures,
without changing the application or replacing the original failing result.
The experiment clears preferences in the disposable application,
so it is not a proposed test change for a real user's installed player.
Both APK sets and all three case-level results are retained by digest in the evidence file.
The emulator shut down gracefully and its owning service is inactive.
[Device evidence](depot-android-device-2026-10-15.evidence.json)
[Cloud APK build](https://github.com/Aquaticats/Monochromatic/actions/runs/38826684750),
[instrumented storage tests](../../package/music-player/android-app/app/src/androidTest/kotlin/dev/monochromatic/musicplayer/SessionStoreTest.kt)

## Prepared environments and non-CI work

A Node/dependency snapshot was created and reused successfully.
The first consumer spent 252.433 seconds in an extra mise cache lookup.
Disabling that redundant action cache preserved passing checks in the subsequent run.
The snapshot preparation job then reused its existing tag without another sandbox execution.
The [workaround record](../troubleshooting/depot-snapshot-mise-cache.md)
separates this observed wait from unverified service-side causes.
[Prepared-image validation](https://depot.dev/orgs/l4zncjjbj5/workflows/jp9w8gw62r)

The standalone SDK also created,
executed a command in,
and stopped a sandbox using the existing login.
The successful sequence took 4.396 seconds.
Two further standalone sandboxes ran commands with 9.882 seconds of overlap,
then both stopped.
This verifies a non-CI execution route and concurrency of two;
it does not measure saturation or validate unlimited capacity.
Bare command lookup failed in the first probe;
an absolute executable path worked.
See the [probe and workaround record](../troubleshooting/depot-sandbox-command-path.md)
and [official SDK reference](https://depot.dev/docs/api/sandbox-sdk-reference).

For future linker experiments,
use identical object files,
fixed CPU and memory allocations,
and repeated measurements before comparing linkers.
Measure individual latency separately from aggregate parallel throughput.
Container builds in the same project and architecture share a builder by default;
launching competing benchmarks there would introduce contention.
[Depot build parallelism](https://depot.dev/docs/container-builds/build-parallelism)

## Rolling analysis budgets

The current Scorecard policy is two analyses per minute,
four per hour,
and eight per day.
Fixture replay executed the existing workflow script with fixed time and disposable GitHub responses.
Allowed,
rejected,
expiry,
failed,
cancelled,
skipped,
and pending-analysis cases matched its current implementation.
A fixture of 926 skipped runs caused 926 sequential job-history queries.
The fixture demonstrates query amplification;
it does not measure GitHub API latency.
[Current gate](../../.github/workflows/scorecard.yml)

Two concurrent fixture evaluations both admitted when prior analysis jobs were not visible.
The gate creates no reservation in that modeled interval.
This is not evidence that a live budget violation occurred.
Moving the gate unchanged would preserve its history-query cost
and make its GitHub accounting inappropriate for Depot runs.
A rolling-budget admission mechanism remains separate from unlimited execution concurrency.
No native Depot rolling-window configuration was verified,
and no production controller was deployed.
The [completed scheduling evaluation](../planning/depot-rolling-budget-evaluation.md)
also records the live gate's final result:
331 seconds of counting,
922 fetched runs,
and 790 job-history results before rejecting the day budget.
The 926-run fixture remains separate evidence.

## Account evidence supplied by the user

The user copied the account's October usage and limit settings on October 15 local time.
This is account-specific user-supplied evidence,
not an independently queried invoice.
The USD $200 plan shows 36 container minutes used against a 5,000-minute cap,
3,804 GitHub Actions minutes against 20,000,
and 29 Depot CI minutes against 20,000.
The selected "Limit usage" setting states that new builds or workflows fail to start at the limit.
These are separate product minute limits,
not evidence of an aggregate dollar cap covering storage or other products.

The same copy reports 25.1 GB in an Intel cache view,
2 GB cache in current usage,
and 1 GB average in build history.
Keep these observations separate until their scope and time aggregation are established.
The account estimates 74 container minutes,
7,859 GitHub Actions minutes,
and 59 CI minutes on the next bill;
this projection includes an exceptional runner-sizing experiment and is not a target-workload forecast.
The trial expiry is stated as seven days,
without an exact timestamp.
No account setting was changed.

## Monthly plan scenarios

The [resumed execution replay](depot-resumed-cost-2026-10-16.md)
is the latest measured-workload allocation.
It replaces the earlier desktop routes with source-specific preparation plus one fresh consumer,
replaces the core baseline with its feature matrix,
and adds representative browser jobs.
Developer subtotals are USD 133.33 for every affected revision,
USD 125.01 with immediate superseded cancellation,
or USD 20 with daily Depot selection and GitHub push validation retained.
Startup remains USD 200 for those measured compute totals.
Storage,
other packages,
extra retries,
and account-wide usage remain outside these subtotals.
Standalone experiment frequency and image refreshes have separate sensitivities.

The retained scenarios in this section document the preceding execution routes.

The successful 410-second desktop run provides an additional scenario input.
With the separate container validation retained,
the measured-subset Developer totals are USD 160.28 for every affected revision,
USD 149.33 with superseded cancellation,
and USD 23.38 for daily-latest Depot allocation with routine GitHub validation retained.
The corresponding Startup totals are USD 205.71,
USD 200,
and USD 200 if overage is permitted.
These supersede the failed-run duration only for the new scenario;
they do not establish lower typical runtime or a complete monthly bill.
[Successful-run replay](depot-desktop-recovery-2026-10-15.evidence.json)

The following retained scenarios document the earlier measured inputs.

The [expanded change-stream replay](depot-expanded-cost-2026-10-15.md)
and [desktop interaction overlay](depot-desktop-cost-2026-10-15.md)
record the preceding cost assessment.
Applying five measured workloads to the pretrial change stream gives Developer monthly subtotals of
USD 77.35 for every affected revision,
USD 73.06 with cancellation of unfinished superseded work,
and USD 20 for daily-latest Depot allocation with affected-push validation retained on GitHub.
Startup costs USD 200 before excluded charges.
These subtotals exclude unmeasured browser and other-package work,
full desktop interaction,
releases,
agent experiments,
and retention.
Adding the full measured desktop interaction job while retaining container validation
raises Developer to USD 166.56 for every affected revision,
USD 154.54 with superseded cancellation,
or USD 24.13 for daily-latest Depot allocation.
The corresponding Startup scenarios are USD 211.98,
USD 202.74,
and USD 200 when overage is allowed.
The first two exceed its runner allowance;
hard-start limits would instead leave some requested work unstarted.
These remain partial-workload scenarios and price the observed chooser failure honestly.
The every-affected desktop-inclusive subtotal leaves USD 33.44 before the ceiling,
or USD 18.44 at 100 GB average storage.
The initial bundle scenarios in this section remain illustrative observations,
not the expanded workload forecast.

Prices are published provider rates checked on October 15,
not an account invoice.
Developer includes 500 container minutes,
2,000 CI minutes,
2,000 GitHub runner minutes,
and 25 GB combined cache and registry storage for USD $20.
Startup includes ten times those compute and storage allowances for USD $200.
Both include one million passing test results and unlimited concurrency.
[Depot pricing](https://depot.dev/pricing)

CI allowances use two-CPU minutes;
a four-CPU minute consumes two included minutes.
Overage is USD $0.006 per base CI minute.
The measured audio,
logger,
and cached Android job intervals total 8.652 base CI minutes per combined bundle.
That bundle combines separate observations,
not a measured recurring workload.
[CI billing rules](https://depot.dev/docs/ci/overview)

These scenarios cost a subset of the expanded target coverage,
not an apples-to-apples replacement of current GitHub jobs.
Additional tests and experiments are intended benefits with additional resource needs;
their value is not captured by current-build speedup alone.
The scenarios retain the initial four-job build observation and are not a completed forecast for the target workflow.

The following scenarios assume 30 days,
25 GB storage,
one 226-second source-change container build per bundle,
and the 8.652-minute CI bundle.
They exclude runner-product usage,
standalone experiments,
additional platforms,
test-result overage,
taxes,
and storage overage.

- Five bundles per day:
  Developer USD $22.60;
  Startup USD $200.
- Ten bundles per day:
  Developer USD $48.77;
  Startup USD $200.
- Twenty-one bundles per day:
  Developer USD $115.62;
  Startup USD $200.
- Thirty-three bundles per day:
  Developer USD $188.55;
  Startup USD $200.

The 21 and 33 cases have a concrete scale reference:
the October 15 Windows workflow history contained 21 push runs and 12 manual runs.
That single day is not a monthly forecast,
and those Windows runs are not the proposed Linux CI bundle.
Cold-cache behavior materially changes the model:
using the observed 468-second cold build at 21 bundles per day gives Developer USD $217.26,
which exceeds the user's ceiling,
versus Startup USD $200 before excluded costs.
[Workflow history](https://github.com/Aquaticats/Monochromatic/actions/workflows/music-player-windows.yml)

The Developer container allowance fits 132 builds at the observed 226-second duration.
Its CI allowance fits 231 of the modeled bundles.
Those are separate allowances;
unused runner minutes cannot be assumed transferable.
Staying near USD $20 therefore requires routing work to GitHub or requesting fewer cloud runs,
not merely selecting the cheaper subscription.

The Bank of Canada's October 15 rate is CAD $1.4196 per USD $1.
The user's CAD $8.975/hour time value becomes approximately USD $6.3222/hour.
An extra USD $180 would require about 28.47 hours of saved valued elapsed time per month
if time savings were its only benefit.
The calculation does not assume Startup has faster default hardware than Developer.
[Bank of Canada observation](https://www.bankofcanada.ca/valet/observations/FXUSDCAD/json?start_date=2026-10-15&end_date=2026-10-15),
[time-value requirement](../handover/depot-trial-grilling.md)

The user's accounting applies to the union of relevant unfinished CI intervals.
A faster Linux job saves none of that elapsed time when a longer required Windows job still determines completion.
The individual build differences therefore cannot be multiplied by commit count to forecast monthly value.
This trial has not replayed the expanded target coverage over a complete cross-platform change stream.

A follow-up inspected every October 15 run returned for the Alpine and Windows workflows:
13 Alpine runs and 33 Windows runs at the query time.
The API total counts matched the returned counts.
Matching completed push runs by exact commit produced four pairs,
of which three succeeded on both platforms.
Windows finished after the final Linux job in all three successful pairs,
by 104,
199,
and 123 seconds.
For those pairs,
even an instantaneous Linux build would not advance the combined completion time while Windows stays unchanged.
The fourth pair had a Linux failure and is excluded from successful-completion comparisons.
This is observed history,
not a forecast for all revisions or a reason to omit Linux coverage.
It also does not value newly enabled checks or experiments that were absent from those historical runs.
The upper-bound calculation was checked with a positive control where Linux finishes 60 seconds later
and correctly reports 60 seconds of possible earlier completion.
[Paired job timestamps](depot-platform-overlap-2026-10-15.evidence.json),
[first paired Linux run](https://github.com/Aquaticats/Monochromatic/actions/runs/38765172148),
[first paired Windows run](https://github.com/Aquaticats/Monochromatic/actions/runs/38765171887)

## Recommended allocation and check frequency

These are proposed operating arrangements for the expanded target workflow,
not deployed routing rules or a complete migration-cost forecast.
The Developer preference follows the user's budget priorities;
the trial has not yet demonstrated that all desired graphical tests and experiments fit its allowances conveniently.

- Developer with intentional GitHub routing is the first choice.
  It preserves the USD $180 base-price saving and the measured Depot execution routes.
  Send the newest requested Linux release validation and selected dependency/toolchain checks to Depot.
  Keep Windows and macOS on their existing paths,
  and use GitHub for routine overflow and checks whose completion is already hidden behind another required job.
  The disadvantage is allocation and usage tracking;
  132 source-change builds per month is only the measured container allowance example,
  not a sufficient automatic policy.
- Developer with selective overage is the second choice.
  It allows more relevant checks without immediately paying USD $200.
  The current prepared-desktop and expanded-browser daily-latest scenario costs USD $20
  before excluded work and storage.
  Every affected revision costs USD $133.33 in that measured subset.
  The disadvantage is variable charges and dependence on a verified spending safeguard.
- Startup is the third choice for the currently measured workload.
  Its added allowances reduce allocation pressure for frequent cloud work and parallel experiments.
  The disadvantage is the USD $200 base price,
  leaving no room beneath the user's ceiling for overage.
  The current measured compute subset fits its included allowances.
  The trial has not demonstrated enough additional benefit to justify that base price.

Ranking:
Developer with routing > Developer with selective overage > Startup.
Routing ranks first because the user explicitly values the USD $180 saving and accepts a hybrid arrangement.
Selective overage ranks ahead of Startup because the measured moderate-use scenarios cost less than USD $200;
this ranking must be revisited if graphical tests,
storage,
or non-CI experiments materially increase usage.

For coverage,
run affected unit tests and type/lint checks for the newest relevant revision.
Use the existing audio-core suite for changes to shared DSP or its consumers,
Android host tests and lint for Android changes,
and a packaged Linux build for release candidates or native dependency/toolchain changes.
Keep the proposed superseded-intermediate cancellation policy separate from explicit releases.
Do not gate every early commit on a new release build.
Run the complete desktop suite before accepting UI,
input,
playback,
or compositor changes and before releases;
use named subsets during iteration.
Run fixture-backed Android instrumentation for device-facing changes and before Android releases.
The demonstrated device route uses an isolated local headed emulator;
its availability and additional workloads still constrain recurring automation.
Keep ordinary affected-revision checks on GitHub when Depot allocation is exhausted,
and use the daily-latest model as a cost scenario rather than a silently adopted cadence.
The passing host suites do not replace those checks.

## Unresolved acceptance gaps and retained artifacts

- Desktop interaction recovered with 117 passes,
  no failures,
  and one uninjectable check after the isolated portal correction.
  Integrating the trial corrections remains separate work.
  Android instrumentation passed all 16 cases in the isolated context experiment;
  the original test source still has the two documented failures.
- Account usage and per-product hard-start limits are user-reported in Account evidence supplied by the user.
  Storage-view aggregation,
  exact trial expiry,
  and an aggregate dollar cap remain unverified.
  The direct usage endpoint still requires an organization credential unavailable to the CLI session.
  The user deliberately excluded further account access;
  this verification is not pending trial work.
- The monthly figures are explicit workload scenarios,
  not a complete monthly forecast or measured cross-platform latency saving.
- Rolling-budget fixture replay found query amplification and a missing reservation interval.
  The query repair is deployed and production-verified:
  run `38831778986` used 10 GraphQL requests to inspect 972 runs without REST history requests.
  The separate admission-reservation design remains a proposal.
- The controlled standalone linker experiment completed with retained artifacts and consumer checks.
  Its timing difference is smaller than unchanged-input spread.
  Two concurrent sandboxes executed successfully;
  no capacity benchmark was performed.

The [machine-readable evidence](depot-trial-2026-10-15.evidence.json)
records build IDs,
job intervals,
fixture outcomes,
and cost calculations.
The trial branch retains the build definitions and runnable tasks for review.
The [issue inventory](../handover/depot-trial-issues.md) tracks discovered defects,
repaired incidents,
and remaining acceptance gaps.
All standalone SDK sandboxes were stopped.
The dedicated project cache and reusable CI snapshot remain available during the trial;
no recurring jobs were installed.

Production routing,
paid plan changes,
and new recurring automation remain outside this trial's adoption boundary.
