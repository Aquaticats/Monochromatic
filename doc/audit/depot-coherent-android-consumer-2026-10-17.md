# Android consumer preparation for the coherent producer

## Prepared boundary

The local consumer is staged without launching an emulator,
starting an adb server,
installing APKs,
or changing a device.
The final producer has not been supplied.
The [preparation evidence](depot-coherent-android-consumer-2026-10-17.evidence.json)
retains the exact catalog,
source hashes,
legacy/adapted task hashes,
fixture hashes,
launch bindings,
and offline input controls.

The immutable source is `cb95cf6d785c3073c192b95f0632cdae7b910894`
at `~/temp/agent/depot-ci-source-truepeak-20261016`.
Its tracked status remains clean.
Fresh catalog extraction matches every identity and test-file hash in the historical evidence:
15 instrumented tests across four Kotlin classes.
The later 16-test catalog is not substituted.
A separate read-only control evaluated the current Mise-parsed `complete-instrument` catalog prefix
and returned all 15 exact source identities.
It excluded the device-command portion;
no `adb`,
emulator,
or systemd command ran.
The `catalogPrefixControl` evidence retains the parsed task-body hash and returned identities.

The existing harness is `~/temp/agent/depot-android-device/mise.toml`.
Its `complete-install` verifies APK hashes before installation,
and `complete-instrument` records the complete unfiltered identity/status stream before asserting success.
The preparation reuses those bodies with private runtime bindings.
The legacy `instrument` task selects the later 16-test expectation and is excluded.
The legacy `complete-record` hardcodes the historical 13-pass result and is also excluded.
Future accounting must retain actual results,
including failures,
skips,
unknown statuses,
and unexecuted identities.

The historical consumer used run `j80v9dqfb4`,
job `sk8lkf9xfp`,
attempt `s57prgfpx8`.
It executed all 15 identities:
13 passed,
two failed,
and none skipped.
Both `SessionStoreTest` failures remain source failures in issue `#1002`.
Its 6.041-second instrumentation interval is historical functional evidence,
not a prediction or performance baseline for the final consumer.
The [historical evidence](depot-complete-android-2026-10-16.evidence.json) remains unchanged.

## Scratch tasks and frozen inputs

The preparation task file is
`~/temp/agent/depot-coherent-android-consumer-20261017/mise.toml`.
Its `stage` task reads and stages inputs without device operations;
`validate` checks a supplied producer contract and local bytes;
`controls` exercises only offline input checks.
The staged runtime task file is
`~/temp/agent/depot-coherent-android-consumer-20261017/runtime/mise.toml`.
Its SHA-256 is `f7450a475640ff27bfad6bef131591d7420e3f9793d392019af11a89b227174c`.
It contains private variants of `prepare`,
`launch`,
`boot`,
`complete-install`,
`fixtures`,
`complete-instrument`,
and `shutdown`.
The device-operation tasks require the separate validated input record;
shutdown remains available for cleanup if provenance validation later fails.

The staged fixture files are copied from the retained synthetic 440 Hz,
nominal ten-second fixture set.
Their byte hashes are retained and checked before final input validation succeeds.
The future fixture task pushes the prepared FLAC,
Opus,
and MP3 files and the private `bench.flac`/`bench.opus` inputs;
it does not import a changing main-worktree tone generator or re-encode them.
The final pass must verify MediaStore reach and synthetic fixture identities before instrumentation.
An unrelated library in the copied AVD is a preparation problem to report,
not permission to claim synthetic-only coverage.

The original five offline controls passed historical real APK hashes with explicitly synthetic cohort,
receipt,
and archive metadata.
They rejected missing producer input,
a wrong APK hash,
a mismatched cohort join,
and wrong source revision.
Those controls and their task hash remain separate historical evidence.
After the common manifest schema settled,
the settled-schema controls passed,
including wrong native-case digest,
producer route,
producer harness,
and outer bytes.
The final producer binding rejects self-consistent receipts for another route,
another workflow,
or the `native-core` wave,
and rejects a receipt whose wave differs from the accepted input.
All thirteen current controls pass.
The five original and nine settled-schema controls remain separate historical records.
Fixture authorization is revoked in `finally`,
including when a control fails.
Those fixtures are not final provider proof.
Their validated output was replaced with an explicitly invalid fixture marker after the controls,
so it cannot authorize the prepared device tasks.
The adapted task bodies passed Node syntax checks.
Every generated task was also parsed through the actual `mise tasks info` boundary,
and its parsed body hash matched the checked module.
Their live device behavior remains unexercised in this preparation.

The initial preparation in commit `5d5544da6a` published task-file hash
`d229f7af50416d54b2cca276d638f6d61f3a9b9bef81e81922e7807c182aef91`.
Parsed-command verification then exposed an unrewritten `--unit=depot-trial-fold` launch argument,
while boot checked the unique prepared unit.
The claim that this initial launch already used the unique unit was retracted.
The exact launch argument was corrected on 2026-10-17 before any launch.
The preparation evidence retains the original/repaired parsed-body hashes and unit mismatch control.
Both the actual parsed launch and boot tasks now name `depot-coherent-android-a038cc3c`,
and every parsed runtime body rejects residual legacy-unit references.
The staging defect is tracked in [#1115](https://github.com/Aquaticats/Monochromatic/issues/1115),
closed after the actual parsed before/after control and repeated offline input checks passed.
Fresh reads verified its body,
`bug` label,
closure evidence comment,
and closed state.

## Final producer input contract

`input-template.json` in the preparation directory names the required fields with pending values.
The caller supplies a completed JSON path through `DEPOT_ANDROID_CONSUMER_INPUT` to `mise run validate`
from that directory.
Validation changes only private staging records.

The required joins are:

- `cohort.id`,
  `cohort.requestedAt`,
  `cohort.manifestSha256`,
  and `cohort.manifestPath` identify the common coherent request and exact manifest bytes.
- `sourceRevision` must equal the immutable native source and `cases.native.sourceRevision`.
  The outer manifest must have schema version 1 and matching `cohortId`,
  remain below 1 MiB,
  and hash byte-for-byte to `cohort.manifestSha256`.
  `caseManifestSha256` separately matches `cases.native.manifestSha256`.
- `harness.producerRevision` names the exact producer harness commit;
  `harness.consumerHarnessSha256` pins the staged local task file.
  The preparation evidence retains both original and adapted task body hashes.
  `producer.routeId` must be `native.producers`,
  with workflow `.depot/workflows/depot-complete-native.yml` and `producer.wave` equal to `native-android`.
  That outer `routes` entry must belong to case `native`;
  its harness revision and workflow must match the producer input.
- `producer.runId`,
  `producer.jobId`,
  `producer.attemptId`,
  and `producer.workflowPath` identify the actual APK-producing attempt.
  `producer.receiptPath` and `producer.receiptSha256` bind the retained producer receipt bytes.
- The receipt must repeat `providerRun`,
  `providerJob`,
  `providerAttempt`,
  `sourceRevision`,
  `harnessRevision`,
  `routeId`,
  `wave`,
  `caseManifestSha256`,
  `cohortId`,
  `requestedAt`,
  and `cohortManifestSha256`,
  and provide the individual APK hashes.
- `archive.path` and `archive.sha256` retain the actual downloaded artifact archive identity.
  Provider collection must also retain its artifact ID/name and producing run/job/attempt relationship.
- `artifacts` provides exactly `app-debug.apk` and `app-debug-androidTest.apk`,
  with absolute local paths and individual SHA-256 values matching both receipt and actual bytes.
  A release APK remains a separate producer artifact unless a separate release consumer is executed.

The root producer owner must verify receipt fields against collected provider metadata,
the frozen workflow/harness ref,
and the common cohort manifest before supplying this contract.
The authoritative path is `doc/audit/depot-complete-cohort-${cohort_id}.json` on `main`.
Collection uses the contents API object's base64 data and checks the decoded bytes;
the local validator receives those exact retained bytes.
The local validator checks consistent joins and byte integrity;
it does not authenticate self-supplied metadata against the provider API.
An overall source-case failure does not erase a successfully produced independent APK artifact.
Its original failed checks remain failed in the cohort record.

## Headful environment and actual-device evidence

The required launch form is the one in
[Working agent launch: headful, host GPU, no container](../troubleshooting/android-emulator-37-software-renderer-sigsegv.md#working-agent-launch-headful-host-gpu-no-container-2026-10-12),
also required by [the emulator skill](../../.agents/skills/android-emulator/SKILL.md).
It is a separate `systemd-run --user` unit with `--collect`,
`--property=MemoryMax=12G`,
actual display/session variables,
and `-gpu host`.
There is no headless,
Xvfb,
container,
or renderer-fallback route.
Launch failure stops execution and retains the diagnostic.

The prepared unique unit is `depot-coherent-android-a038cc3c`.
The private adb server port is `5097`,
and emulator console/transport ports are `5802`/`5803`.
These are planned bindings,
not reserved ports;
their availability and the display sockets must be checked again before launch.
`HOME`,
`ANDROID_USER_HOME`,
and `ANDROID_AVD_HOME` all point inside the new private runtime.
Every adb call,
including shutdown,
uses that same private HOME and port.
The AVD is a required reflink copy of the owned `Pixel_9_Pro_Fold`;
the owned AVD is never driven or edited.
Only lock records copied into a new,
not-yet-launched private destination may be removed there.

Read-only inspection found emulator `37.2.12.0`,
adb `37.0.1-15733141`,
and systemd `259.9-1.fc44`.
The owned config targets the API 37 x86_64 16 KiB-page image,
with configured display `2076` by `2152` at density `390` and configured guest RAM `8192` MiB.
The staged launch requests four cores and `4096` MiB guest RAM under the separate 12 GiB unit cap.
The config values and launch requests are not observations of a new guest.
The prior unit was absent/inactive during inspection;
no guest was launched for a new settings probe.

After boot,
the actual consumer must retain SDK,
ABI,
page size,
font scale,
forced-density and forced-size settings,
guest CPU/memory information,
and actual renderer identity from the unit journal.
It must retain effective unit limits,
invocation ID,
peak/current memory,
task counters,
and CPU accounting as observed,
separately from requested limits.
No `wm density` or `wm size` command is permitted.
Read display settings before and after the pass;
restore any Settings UI changes if a later pass explicitly makes them.

## Consumer result and cleanup contract

The final result envelope must retain the input contract,
producer receipt/archive/APK hashes,
source/head/tree/test-source hashes before and after,
and the actual device/resource observations.
The unfiltered command remains
`am instrument -w -r dev.monochromatic.musicplayer.test/androidx.test.runner.AndroidJUnitRunner`.
No class or test filter may replace the complete 15-identity catalog.
Retain raw stdout/stderr,
runner completion status,
command exit/signal/error,
every terminal test status,
duplicate or unknown identities,
and any unexecuted identities.
An adb exit zero is not a passing-suite result.
The historical context failure is not repaired in either source or tests.

The driver must run cleanup in a `finally` path after successful launch,
including when installation or instrumentation fails.
The graceful order is guest `sync`,
an eight-second wait,
guest `sync`,
then `adb emu kill` with the private HOME and port.
Never stop the unit while the guest writes.
Retain cleanup command results,
unit terminal state,
process exit evidence,
and private adb-server termination.
A cleanup failure stays visible and must not be recorded as successful cleanup.

Retain local phase timestamps for artifact validation,
AVD copy,
launch,
boot,
installation,
fixture readiness,
instrumentation,
and cleanup.
Keep that local consumer interval and local resource accounting separate from provider compute.
The shared request timestamp and producer-attempt identity bind the dependency into the coherent timeline;
the consumer starts only after its actual producer artifacts are available.

## Remaining dependency

Root must supply the final producer run/job/attempt,
frozen workflow/harness refs,
common cohort/request/manifest binding,
provider receipt,
artifact archive,
and individual APK bytes/hashes.
Only then can live reflink preparation,
port/display checks,
headful launch,
actual-device observations,
unfiltered instrumentation,
and cleanup verification run.
The final phase/timing/resource envelope must be completed around those actual task invocations.
This preparation makes no account changes and dispatches no provider work.
