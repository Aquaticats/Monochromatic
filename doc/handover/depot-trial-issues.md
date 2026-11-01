# Depot trial issue inventory

## Purpose and filing boundary

The user requested a GitHub issue for every problem discovered during this trial.
This inventory links each distinct finding to its current-repository issue.
Existing open and closed issues were searched before filing.
Observed failures are distinguished from unknown causes,
coverage gaps,
and unverified policy boundaries.
Trial-only workarounds remain open where integration or diagnosis is outstanding.
No upstream issue or support message was sent.

The first completed-batch repository query returned 50 newly filed issues:
41 open and nine closed with recovery evidence.
Existing issues were reused for six additional discovery or process findings.
These counts describe the filing result,
not completed integration of the trial-only corrections.
The completed resumed experiments add 12 issues,
including four closed after verified trial corrections.
The aggregate repository query now returns 62 newly filed issues:
49 open and 13 closed,
with the six reused issues remaining separate.

## Application, harness, and runner findings

### Final logger completion incidents

- [#1434](https://github.com/Aquaticats/Monochromatic/issues/1434):
  aggregate mutation JSON is retained,
  but real physical shard manifests and reports are omitted.
  Independent verification covers 5542 selected IDs;
  the 863 declared shards receive no physical reconciliation credit.
- [#1180](https://github.com/Aquaticats/Monochromatic/issues/1180):
  final common-manifest lookup returns HTTP 401 after successful native TOML mutation.
  Credential rejection cause remains unknown.
- [#1181](https://github.com/Aquaticats/Monochromatic/issues/1181):
  static and unit tasks rebuild tracked distribution outputs;
  their clean-tree guards reject the differences.
- [#1182](https://github.com/Aquaticats/Monochromatic/issues/1182):
  generated-consistency worker terminates without an artifact.
  Partial command outcomes and `SIGKILL` observations survive in the provider log.
  Neither OOM nor another termination cause is established.

These additions do not change the historical completed-batch counts.
Their evidence and acceptance boundaries are recorded in
[the logger completion troubleshooting doc](../troubleshooting/depot-logger-final-completion-gaps.md).

### Original application and runner findings

- [#1002](https://github.com/Aquaticats/Monochromatic/issues/1002):
  Android instrumentation:
  SessionStoreTest uses a context with null applicationContext.
- [#1003](https://github.com/Aquaticats/Monochromatic/issues/1003):
  Android instrumentation can skip decoder and playback coverage when indexed audio fixtures are absent.
- [#1004](https://github.com/Aquaticats/Monochromatic/issues/1004):
  Desktop E2E:
  folder chooser never appears on the prepared Depot software-rendered runner.
- [#1005](https://github.com/Aquaticats/Monochromatic/issues/1005):
  Desktop E2E:
  fixed-coordinate folder selection aborts the thirty-track fallback fixture.
- [#1006](https://github.com/Aquaticats/Monochromatic/issues/1006):
  Desktop E2E:
  an aborted session prevents later independent sessions from reporting coverage.
- [#1007](https://github.com/Aquaticats/Monochromatic/issues/1007):
  Desktop E2E:
  nested parent compositor route fails pointer and text checks in the cloud trial.
- [#1008](https://github.com/Aquaticats/Monochromatic/issues/1008):
  Depot trial Cargo caches can return stale binaries after older source layers are restored.
- [#1009](https://github.com/Aquaticats/Monochromatic/issues/1009):
  Depot runner loses communication before recording any build steps.
- [#1010](https://github.com/Aquaticats/Monochromatic/issues/1010):
  Desktop chooser errors are hidden by the unbridged log facade and discarded Zenity stderr.
- [#1011](https://github.com/Aquaticats/Monochromatic/issues/1011):
  Trial mise setup warns that the cmake vfox plugin metadata module is missing.
- [#1012](https://github.com/Aquaticats/Monochromatic/issues/1012):
  Trial runtime installation reports the pipx-slopo lock sidecar missing.

## Package reuse and account observability

- [#995](https://github.com/Aquaticats/Monochromatic/issues/995):
  snapshot checkout collides with the CLI's injected local patch.
- [#996](https://github.com/Aquaticats/Monochromatic/issues/996):
  redundant mise cache restoration delays a prepared-image consumer.
- [#997](https://github.com/Aquaticats/Monochromatic/issues/997):
  fresh Depot CI consumers lack the user session required by desktop tests.
- [#998](https://github.com/Aquaticats/Monochromatic/issues/998):
  integrate verified installed-package reuse into an adopted desktop route.
- [#999](https://github.com/Aquaticats/Monochromatic/issues/999):
  the existing CLI credential cannot query the account usage endpoint.
- [#1000](https://github.com/Aquaticats/Monochromatic/issues/1000):
  verify aggregate spending control beyond separate product start limits.
- [#1001](https://github.com/Aquaticats/Monochromatic/issues/1001):
  reconcile storage scopes and documented cache-policy limits.

## Release transfer and scheduling

- [#1013](https://github.com/Aquaticats/Monochromatic/issues/1013):
  public release download depended on exhausted authenticated REST quota;
  closed with the verified consumer repair.
- [#1014](https://github.com/Aquaticats/Monochromatic/issues/1014):
  GitHub CLI masks the quota error as a missing release.
- [#1015](https://github.com/Aquaticats/Monochromatic/issues/1015):
  transferred release provenance is missing from the repository endpoint;
  closed with the verified original-identity bundle consumer repair.
- [#1016](https://github.com/Aquaticats/Monochromatic/issues/1016):
  Cargo metadata used the previous repository owner;
  closed with the generator and managed-manifest correction.
- [#1017](https://github.com/Aquaticats/Monochromatic/issues/1017):
  Scorecard history amplified REST usage;
  closed after production GraphQL verification.
- [#1018](https://github.com/Aquaticats/Monochromatic/issues/1018):
  Scorecard admission has no reservation before analysis starts.
- [#1019](https://github.com/Aquaticats/Monochromatic/issues/1019):
  Scorecard budget accounting uses workflow creation time.
- [#1020](https://github.com/Aquaticats/Monochromatic/issues/1020):
  cancellation can refund already-started analysis in the existing accounting.
- [#1021](https://github.com/Aquaticats/Monochromatic/issues/1021):
  latest-attempt history does not retain all attempted analysis work.
- [#1022](https://github.com/Aquaticats/Monochromatic/issues/1022):
  the GitHub CLI run-log route rejects completed-job logs while siblings run.
- [#1031](https://github.com/Aquaticats/Monochromatic/issues/1031):
  published scanner version 0.5.0 omits the binstall release-tag template;
  explicit download-layout dry runs succeed while registry-default dry runs fail.

The accounting issues describe source and fixture evidence;
they do not assert an observed live budget overrun.

## Repository publication

- [#1032](https://github.com/Aquaticats/Monochromatic/issues/1032):
  GitHub rejects an ordinary signed-commit push with `GH006` and no named requirement.
  Concurrent publication delivered the commit without a protection or history change;
  the server-side cause remains unknown.

## Expanded coverage and routing

- [#1023](https://github.com/Aquaticats/Monochromatic/issues/1023):
  native verification routes are absent from routine orchestration.
- [#1024](https://github.com/Aquaticats/Monochromatic/issues/1024):
  Android host,
  lint,
  and ABI checks need explicit routing.
- [#1025](https://github.com/Aquaticats/Monochromatic/issues/1025):
  existing browser suites need CI execution.
- [#1026](https://github.com/Aquaticats/Monochromatic/issues/1026):
  the `done` application lacks selected browser user-flow coverage.
- [#1027](https://github.com/Aquaticats/Monochromatic/issues/1027):
  affected-consumer selection needs transitive workspace dependencies.
- [#1028](https://github.com/Aquaticats/Monochromatic/issues/1028):
  a workflow filter still names the removed ownership package.
- [#1029](https://github.com/Aquaticats/Monochromatic/issues/1029):
  shared lockfile and toolchain changes are missing from selected workflow triggers.
- [#1030](https://github.com/Aquaticats/Monochromatic/issues/1030):
  remaining expanded workloads lack measured costs and an allocation policy.

Existing discovery issues #305,
#555,
#310,
and #603 were reused instead of duplicated.
The [coverage issue handover](depot-coverage-issues-2026-10-15.md) records those matches.

## Historical setup and access incidents

- [#1033](https://github.com/Aquaticats/Monochromatic/issues/1033):
  organization action policy rejected workflow startup;
  closed after the user's policy choice and verified startup recovery.
- [#1034](https://github.com/Aquaticats/Monochromatic/issues/1034):
  mise release discovery failed with HTTP 403 during setup.
- [#1035](https://github.com/Aquaticats/Monochromatic/issues/1035):
  the local worker task ceiling interrupted trial collection;
  closed with the user's recovery and measured replacement limit.
- [#1036](https://github.com/Aquaticats/Monochromatic/issues/1036):
  a local helper/kernel update mismatch prevented execution;
  closed with verified recovery.
- [#1037](https://github.com/Aquaticats/Monochromatic/issues/1037):
  the trial task was outside mise discovery roots;
  closed after relocating it and running the discovered task.
- [#1038](https://github.com/Aquaticats/Monochromatic/issues/1038):
  the direct desktop trial omitted the compositor path;
  closed with the corrected execution.
- [#1039](https://github.com/Aquaticats/Monochromatic/issues/1039):
  standalone SDK command execution failed to resolve bare command names.
- [#1040](https://github.com/Aquaticats/Monochromatic/issues/1040):
  Browser discovery returned no available browser for account inspection.

Existing process issues #425 and #572 cover asking measurable interview questions
and under-scoped completion or verification.
The final audit reused those instead of creating duplicates.

## Warnings exposed by successful chooser verification

- [#1041](https://github.com/Aquaticats/Monochromatic/issues/1041):
  the private desktop session does not provide the accessibility bus requested by GTK.
- [#1042](https://github.com/Aquaticats/Monochromatic/issues/1042):
  GTK cannot commit dconf changes in the private desktop session.
- [#1043](https://github.com/Aquaticats/Monochromatic/issues/1043):
  the folder dialog maps without a transient parent.
- [#1044](https://github.com/Aquaticats/Monochromatic/issues/1044):
  held-modifier repeat remains outside interactive coverage;
  pure detector tests and ordinary held-key repeat controls do not exercise that input boundary.

These warnings were visible after the chooser recovered in run `38833901553`.
They did not fail that run's interaction assertions.
Issue #1004 retains the verified recovery and stays open for integration of the trial-only correction.

## Resumed execution and accounting

- [#1045](https://github.com/Aquaticats/Monochromatic/issues/1045):
  the Playwright container version differs from the locked test runner.
- [#1046](https://github.com/Aquaticats/Monochromatic/issues/1046):
  repeated same-key Depot Test uploads duplicate stored result rows.
- [#1047](https://github.com/Aquaticats/Monochromatic/issues/1047):
  browser execution needs a working Podman user mapping in Depot CI.
- [#1048](https://github.com/Aquaticats/Monochromatic/issues/1048):
  the initial skip fixture matched no compiled Rust test;
  closed after the corrected fixture reported the intended skip.
- [#1049](https://github.com/Aquaticats/Monochromatic/issues/1049):
  the split probe passed a run-only flag;
  closed after invoking the supported split interface successfully.
- [#1050](https://github.com/Aquaticats/Monochromatic/issues/1050):
  the Rust timing adapter omitted an integration-test binary identity;
  closed after every expected identity appeared exactly once across the final shards.
- [#1051](https://github.com/Aquaticats/Monochromatic/issues/1051):
  the Sandbox SDK client intermittently remains alive after completed remote commands.
  The bounded reproduction,
  later passing control,
  and unresolved source ownership are retained in the
  [client-lifecycle record](../troubleshooting/depot-sandbox-client-exit.md).
- [#1052](https://github.com/Aquaticats/Monochromatic/issues/1052):
  the browser suite needs to distinguish unavailable OPFS from a logger regression.
- [#1053](https://github.com/Aquaticats/Monochromatic/issues/1053):
  prepared desktop execution needs verified decode-priority requests and rtkit fallback.
- [#1054](https://github.com/Aquaticats/Monochromatic/issues/1054):
  the prepared desktop session lacks verified analysis notifications through the built player.
- [#1055](https://github.com/Aquaticats/Monochromatic/issues/1055):
  compositor EGL diagnostics do not identify the affected operation.
- [#1056](https://github.com/Aquaticats/Monochromatic/issues/1056):
  the first expanded-cost draft used the wrong Linux runner overage rate;
  closed after correcting the rate and independently verifying the revised replay.

The [Test report](../audit/depot-test-feature-2026-10-16.md),
[expanded execution report](../audit/depot-expanded-execution-2026-10-16.md),
and [prepared desktop report](../audit/depot-prepared-desktop-2026-10-16.md)
retain the corresponding probes and recovery boundaries.
Existing issues #997 and #998 retain the successful combined desktop evidence;
they remain open for production integration.

## Complete-graph replay findings

- [#1057](https://github.com/Aquaticats/Monochromatic/issues/1057):
  immutable-base ancestry assertions need the base object fetched into the trial checkout.
- [#1058](https://github.com/Aquaticats/Monochromatic/issues/1058):
  Scorecard image inspection collided with mise template rendering;
  closed after the corrected adapter and fixture controls passed.
- [#1059](https://github.com/Aquaticats/Monochromatic/issues/1059):
  native trial setup omitted the source-configured cargo-binstall prerequisite;
  closed after Windows and macOS setup recovery.
- [#1060](https://github.com/Aquaticats/Monochromatic/issues/1060):
  the historical Windows workflow required unsupported GUI PowerShell redirection and skipped nextest;
  closed with existing corrected-workflow evidence and the independent test replay.
- [#1061](https://github.com/Aquaticats/Monochromatic/issues/1061):
  historical Windows tests referenced a Unix-only libc CPU helper;
  closed with the existing source fix and later passing compilation.
- [#1062](https://github.com/Aquaticats/Monochromatic/issues/1062):
  historical macOS change-plan fixtures generated case-colliding names;
  closed with the existing source fix and later passing test.
- [#1063](https://github.com/Aquaticats/Monochromatic/issues/1063):
  macOS rename notifications use a canonical target path that differs from the fixture expectation.
- [#1064](https://github.com/Aquaticats/Monochromatic/issues/1064):
  macOS burst-write notifications differ in reported paths and structural flags.
- [#1065](https://github.com/Aquaticats/Monochromatic/issues/1065):
  macOS in-place-write notifications differ in reported path and structural flag.
- [#1066](https://github.com/Aquaticats/Monochromatic/issues/1066):
  full-package Oxlint panics in `no-generated-empty-object-type` at the selected TOML revision.
- [#1067](https://github.com/Aquaticats/Monochromatic/issues/1067):
  the TOML fuzz coverage gate reports fewer covered lines in `values.ts` than its retained baseline.
- [#1068](https://github.com/Aquaticats/Monochromatic/issues/1068):
  the file-enforcer mutation task resolves its repository-relative package path twice.
- [#1069](https://github.com/Aquaticats/Monochromatic/issues/1069):
  truepeak-core Clippy rejects the schema-version constant assertion.
- [#1070](https://github.com/Aquaticats/Monochromatic/issues/1070):
  file-enforcer container tests receive `SIGKILL` after a logger sink verification timeout.
- [#1071](https://github.com/Aquaticats/Monochromatic/issues/1071):
  historical generated-file worktree checkout references an unresolvable Git LFS endpoint.
- [#1072](https://github.com/Aquaticats/Monochromatic/issues/1072):
  native trial setup needs Clippy installed for the exact locked Rust toolchain.
- [#1073](https://github.com/Aquaticats/Monochromatic/issues/1073):
  fail-fast native replay leaves independent selected tests unexecuted after a source failure.
- [#1074](https://github.com/Aquaticats/Monochromatic/issues/1074):
  isolated VM preparation needs bounded libvirt and QEMU Unix socket paths.
- [#1075](https://github.com/Aquaticats/Monochromatic/issues/1075):
  the VM bootstrap assumed `XDG_RUNTIME_DIR` was already present.
- [#1076](https://github.com/Aquaticats/Monochromatic/issues/1076):
  the vmsync lifecycle package task resolves its mvm helper from the package working directory.
- [#1079](https://github.com/Aquaticats/Monochromatic/issues/1079):
  the Linux rescan watcher test does not receive its expected initial queue before the rescan assertion.
- [#1080](https://github.com/Aquaticats/Monochromatic/issues/1080):
  the Linux idle watcher test does not receive its expected initial queue before the idle assertion.
- [#1081](https://github.com/Aquaticats/Monochromatic/issues/1081):
  the Linux burst-write test receives an extra report after the first correct report.

These findings preserve source,
harness,
and environment boundaries.
The macOS watch symptoms remain separate until a shared cause is verified.

## Source identity helper incident

- [#1114](https://github.com/Aquaticats/Monochromatic/issues/1114):
  the original trial source-identity helper omitted root Cargo config and nested Android Cargo metadata,
  and lacked an independent check for index exclusions hiding tracked paths.
  Closed as fixed for the isolated helper after actual metadata/path and index-invariant controls passed.

The original `4c4063e78b` metadata omissions,
the `faf388e838` metadata/path repair,
and the `a07d84a129` independent index invariant remain distinct in the issue body.
The [original 15 controls](../audit/depot-source-identity-controls-2026-10-17.evidence.json),
[metadata/path 19 controls](../audit/depot-source-identity-repaired-controls-2026-10-17.evidence.json),
and [index-invariant 12 controls](../audit/depot-source-identity-index-controls-2026-10-17.evidence.json)
remain separate unchanged evidence records.
Both historical before/after source pairs passed;
ordinary source outside the metadata subset rejected unchanged flags and hidden edits for both index exclusions.
The acceptance tradeoff rejects indexed exclusions even when their file bytes equal HEAD.
Shared historical source bytes remained unchanged.

All-state searches for source identity,
immutable checkout,
and index-masking terms found no matching incident before filing.
The created body and `bug` label were fresh-read before closure.
A final read verified the matching body,
[closure evidence comment](https://github.com/Aquaticats/Monochromatic/issues/1114#issuecomment-6230481503),
label,
and closed state at `2026-10-17T06:04:17Z`.
This helper-specific closure does not establish production workflow integration,
passing application checks,
or a coherent provider replay.
The [source identity diagnosis](../troubleshooting/depot-historical-checkout-bootstrap.md#source-identity-input-coverage)
retains the tested revisions and input boundaries.

## Android consumer staging incident

- [#1115](https://github.com/Aquaticats/Monochromatic/issues/1115):
  the first prepared consumer launch retained the old systemd unit literal while boot checked the unique unit.
  Closed after the actual parsed pre-fix control showed the mismatch,
  repaired launch/boot bindings agreed,
  and all parsed task bodies rejected the legacy unit.

The [Android preparation](../audit/depot-coherent-android-consumer-2026-10-17.md)
retains the retracted claim,
corrected task hash,
and remaining final-producer dependency.
The bug was caught before any emulator or device operation.
Fresh reads verified the issue body,
`bug` label,
[closure comment](https://github.com/Aquaticats/Monochromatic/issues/1115#issuecomment-6230699668),
and closed state at `2026-10-17T06:30:13Z`.
This closes only the local staging defect,
not the pending final APK consumer or an Android Emulator defect.

## Pending final reconciliation

The totals in this handover predate the latest complete-graph replays.
After diagnostic filing settles,
query every explicit issue ID and reconcile the new issue records,
reused existing issues,
open states,
and closures before replacing the aggregate totals.
The native-source findings are recorded in issues #1061 and #1062 and closed as already fixed;
the separate macOS watch findings remain open in #1063,
#1064,
and #1065.
Root also added replay evidence to existing issue #1002;
that is not a newly created issue.

Accounting collection is complete for terminal Depot runs `79lgpx91sf`,
`lvcwx21cbt`,
`ssvlvk57k4`,
`1pfkxwlwbd`,
`g1nngvfsrg`,
`xz7cchwbsk`,
`4z014g290p`,
and `dxk4v3s0w3`.
Their retained records include allocation verification and every provider attempt.
Final source-case reconciliation still requires the corrected manifests and later recovery runs.
VM replay `rv6hfj76w9` is collected with proof of template boot and a separate final-guest virtiofs failure.
Its next diagnostic retry remains queued for collection after dispatch.
Native recovery runs `38847322477` and `38847322559` are collected and independently reconciled.
Historical macOS ran every eligible case:
1356 passed and 26 failed.
Later macOS ran every eligible case:
1406 passed and 27 failed.
Historical Windows reached the known `E0433` missing-`libc` test compile boundary.
Linux desktop nextest recovery `pbg4h8ww6k` failed during `slint-viewer --version` discovery before tests.
Root dispatched corrected Linux recovery `xf4hhngdqg`;
its terminal artifact is collected and every eligible test identity is independently reconciled.
All 1460 passed,
with one separate ignored benchmark.
Matching before/after source hashes and immutable Git-object hashes verify the source stayed unchanged.
This completes the nextest coverage recovery `#1073`
and verifies the missing Slint viewer wave repair `#1108`.
Historical Windows still has its genuine source test-compilation blocker;
complete coverage accounting does not mean passing cross-platform CI.
The next terminal batch has been collected:
mutation attempts `70p38v12d7` and `0snqhv0rhl`,
scheduler controls `m00504dz4c`,
desktop E2E attempts `nb6ptmwjs5` and `xmvgm200k5`,
WireGuard `cwkngc2w7g`,
Pi `04pjg1sw9j`,
and prepared-image import `wqz9h633wj`.
Artifact inspection found that the desktop attempts omit scratch session logs;
their next diagnostic replay must retain those logs before classifying the startup timeout.
The [accounting audit](../audit/depot-complete-ci-accounting-2026-10-16.md)
owns the measurements and distinctions between execution completion and passing CI.

## Verification

The application and harness issue bodies were rendered before submission.
Each created issue was fetched again and its title,
body,
and open state matched the prepared content.
The package-reuse and account issues were independently fetched and checked for their bodies and triage labels.
The [trial report](../audit/depot-trial-2026-10-15.md)
and [coverage proposal](../planning/depot-expanded-coverage.md)
retain the measurement and adoption boundaries.

## TOML reconciliation queue

The [TOML audit](../audit/depot-toml-reconciliation-2026-10-17.md)
retains original run `p9r4xl6n12` and terminal development run `p6vm5s00w6`.
The latter emitted 3062 distinct mutant IDs across 9037 attempts,
all infrastructure-derived `runtimeError` before assertions.
The full native report,
shard identities,
artifacts,
provider usage,
and cleanup are retained.
File-enforcer mutation repeats `#1068`;
the completed container failure `#1070` remains separate.
Timeout/independent-check coverage issue `#1113` remains open.
Source lint issues `#1111` and `#1112` remain distinct from historical panic `#1066`.

Actual bounded pilot `0j8zhbn9n0` completed the unchanged source command with exit zero,
a green baseline,
and one source-terminal `compileError` mutant.
No mutant assertion execution or full-campaign credit is claimed.
Its provider job failed metadata collection with `null !== 0`;
the old harness did not retain the underlying operation details.
The complete owned work-volume and generated-metadata adapter crossed both consumer boundaries.
Exact receipts,
artifact hashes,
the 167222-millisecond provider interval,
and verified cleanup are retained.

Actual bounded receipt repeat `mj128hqwv0` finished on
`0f8ad795aa92d3beb83e3bfa4e4aa54f1bea06f8`.
Terminal collection and all 20 artifact member hashes are verified.
The exact observer list operation exceeded eight seconds with `SIGKILL` and `ETIMEDOUT`,
after three stdout bytes and zero stderr bytes.
Source command and eight-second metadata budgets are unchanged;
the harness now retains exact command,
signal,
error code,
and output bytes before assertions.
The subsequent metadata-only forwarding trial retains the failure and its rejection controls.
Issue `#1118` records the independent,
source-consumer-proved inherited-method replacement defect;
it does not explain the infrastructure failures.
No application source fix is authorized or applied.

The [canonical guard audit](../audit/depot-common-manifest-route-guards-2026-10-17.md)
records current pushed route references,
20 accepted and 110 rejected disposable invocations,
and separate actual authenticated transport proof.
Only absent canonical API responses are stubbed in local controls.
Provisional case manifests remain `completeGraph: false` and correctly rejected.
Root must supply complete frozen case bytes,
publish the real canonical manifest with final harness hashes,
and verify actual canonical reads before final coherent dispatch.
Final dispatch remains held.

Bounded metadata forwarding trial `vlrxjltfkq` succeeded on `316e1a8a309e71e456a741eb63e5ff3b94b75646`.
The observer deadline is 60000 milliseconds;
original source adapter bytes and source arguments are preserved.
A copied observer forwarder retains sudo lifecycle results,
and source Podman correlation is now recorded.
Terminal collection verifies success,
all ten metadata results,
source correlation and scoped cleanup.
The longest metadata call was 14100.441549 milliseconds;
its forwarded sudo call returned zero.
Issue `#1130` retains this observer deadline boundary,
original failures and rejection controls.
Its complete actual result permitted the full TOML-only immutable mutation retry.

Full TOML-only development retry `23dk3nr90f` is dispatched at `0f1ef041a0de2ff6d2db6dd55210b1160c592648`.
The exact request,
provider workflow bytes,
source defaults and budget controls are retained in the TOML audit.
Its source wrapper budget is five hours;
provider job budget is 360 minutes.
These are bounds,
not a runtime prediction.
Terminal wait session `96306` was active at dispatch
and is terminal and reaped as of `2026-10-17`.
The next action is artifact collection and reconciliation against all 3062 prior mutant identities.
The immutable runner stops at the first failing selected test file,
so the full-suite list does not prove 37 executions per mutant.
Its spawn-error-to-`killed` classification is being checked with an independent disposable source control
on `trial/depot-toml-spawn-control-20261017`;
the active campaign and accepted final refs remain unchanged.
Issue `#1130` is closed after actual bounded source,
observer and cleanup recovery;
`#1113` and `#1118` remain open.
Final coherent dispatch remains held.

Independent actual-source spawn classification control `m018vgrtcv` is dispatched
on clean harness `b78123c7bd8e20d4319cddd63c34c70fa7638d72`;
its terminal evidence and any demonstrated distinct issue remain required.

Control `m018vgrtcv` failed its fixture typecheck before mutation assertions.
Its compiler detail was not retained;
no false-kill result is claimed.
Independent observation repeat `j03l179f3x` is dispatched with incremental receipts
and original PATH commands preserved except the removable Node launcher.
Terminal wait session `83226` ended;
observed-repeat terminal wait session `51269` is active.

Observation repeat `j03l179f3x` ended at the control eval-carrier `EEXIST` boundary.
NanoSpawn inherited its parent eval flags;
actual local file/eval controls verify this behavior.
File-based cloud control `699f9lg5zm` is dispatched on
`f636b13af902ed6abe5cd8758da4cb8ba99f2175`.
Observed-repeat wait session `51269` ended;
file-carrier terminal wait session `32240` was active at dispatch
and is terminal and reaped as of `2026-10-17`.
No false-kill reproduction is claimed until this actual source control completes.

Current TOML checkpoint:
full development run `23dk3nr90f` is successful and reconciled.
All `3062` identities have source-native terminal dispositions backed by physical green-baseline reports:
`921 killed`,
`256 survived`,
`3 timeout`,
`1882 compileError`,
and zero `runtimeError`.
Keep native `killed` as a source classification without inferred literal assertion credit.
Four auxiliary Podman `inspect` status-`125` receipts remain separate from zero final infrastructure errors.
The durable primary ZIP and per-identity compressed reconciliation are linked in the owned TOML audit.
Historical full wait `96306` and file-control wait `32240` are terminal and reaped.
File control `699f9lg5zm` failed its ancillary dependency metadata lookup before classifier assertions.
Independent corrected control `rsbr0wrdt0` finished successfully at
`f650e3e2f923bea21a7baf9565bb1ec3efb4fbde`;
terminal wait session `43521` is reaped with exit zero.
Next:
retain that actual classifier and emitter result,
file a distinct source incident only if reproduced after duplicate comparison,
and reconcile the original timeout issue's required coverage.
Only the TOML mutation check ran in the full retry;
this is not final cohort latency.
Final common publication and dispatch remain held.

Actual-source classifier and emitter controls passed.
A genuine assertion killed the positive fixture;
removing the test launcher after the real compiler passed also returned native `killed`,
with no test-entry marker and exact `spawn node ENOENT`.
The source emitted both expected caught-spawn diagnostics.
Keep this controlled source defect separate from the full campaign's zero matching log diagnostics.
Next:
publish the distinct source report after duplicate comparison,
retain its receipt,
and close `#1113` only against the fully retained independent check outcomes.
Final common publication and dispatch remain held.

Source classifier issue `#1142` is published and verified open with `needs-triage`.
All `24` original selected task commands now have retained terminal receipts:
`20 passed` and `4 failed`,
with the historical canceled run preserved as incomplete.
Issue `#1113` is closed against verified independent coverage;
its exact comment `6232394931` and closed state were re-fetched.
Source failures remain failed;
this historical union is not final coherent coverage or latency.
Final common publication and dispatch remain held.

Current owned TOML recovery accounting is complete.
Source classifier `#1142` and source prototype lookup `#1118` remain open.
No application source changed.
The final TOML route stays at `0f1ef041a0de2ff6d2db6dd55210b1160c592648`.
Native/Windows/Alpine original guard templates resolve to their intended harnesses
under actual nested Mise metadata calls;
no guard rewrite was made.
Next owned step depends on root supplying complete frozen case bytes and the published canonical manifest.
Then update the copied case bytes and route hashes,
verify actual canonical retrieval,
and return the final route receipts to the common orchestrator.
Final coherent dispatch remains held.

## Native recovery assertion tracking

The coherent replay routes are prepared and pushed,
pending the common complete-graph orchestrator.
The [frozen route report](../audit/depot-coherent-native-routes-2026-10-17.md)
records native macOS/Windows,
original desktop Windows variants,
and both Alpine architectures.
Every original consumer command body is preserved.
Trial-only provenance binds each job to a shared manifest,
request instant,
clean source,
and exact harness.
Windows merged-stream and uncaptured-console probes continue independently after the original file assertion.
Uncaptured completion also requires its exact expected version in the provider step log.
Alpine app/compositor builds continue independently after source test assertions.
Local positive and rejection controls pass;
the new cloud steps have not been dispatched.
No application source fixes are included.
Independent development runs cannot establish combined latency.

The native recovery evidence retains every eligible test identity,
exact failing output block,
source hash proof,
provider interval,
and allocation in
[the native evidence](../audit/depot-native-recovery-2026-10-17.evidence.json).
The 29 distinct failed identities include the existing issues `#1062` through `#1065`.
The remaining assertions have separate issues because a shared cause has not been established.
The existing fleet-wide Clippy issue `#372` now has the exact native source diagnostics.
The missing Clippy component incident `#1072` remains closed.
Coverage issue `#1073` is closed with complete native and Linux execution evidence,
with the historical Windows source compile blocker retained as such.
The missing Slint viewer wave issue `#1108` is closed after the corrected consumer passed.
Linux watcher issues `#1079`,
`#1080`,
and `#1081` retain their original failures plus a separately observed successful repeat;
their causes are not resolved by that later pass.

- [#1082](https://github.com/Aquaticats/Monochromatic/issues/1082):
  a file that is not listed lands where the walk would put it.
- [#1083](https://github.com/Aquaticats/Monochromatic/issues/1083):
  a new folder and a removed file are reported as entries that appeared and went.
- [#1084](https://github.com/Aquaticats/Monochromatic/issues/1084):
  the selected settings row ring follows the 21 px pill in the two pane form.
- [#1085](https://github.com/Aquaticats/Monochromatic/issues/1085):
  the settings back ring is a circle and a row ring follows the group corners in the single form.
- [#1086](https://github.com/Aquaticats/Monochromatic/issues/1086):
  the template editor page rings follow the back circle and the reset pill.
- [#1087](https://github.com/Aquaticats/Monochromatic/issues/1087):
  the template text field ring follows its 4 px outline and leaves the outline alone.
- [#1089](https://github.com/Aquaticats/Monochromatic/issues/1089):
  the menu is wide enough for its longest shortcut.
- [#1090](https://github.com/Aquaticats/Monochromatic/issues/1090):
  the settings page keeps tab inside it and a template line does not swallow tab.
- [#1091](https://github.com/Aquaticats/Monochromatic/issues/1091):
  a key between the shift taps cancels the double tap.
- [#1092](https://github.com/Aquaticats/Monochromatic/issues/1092):
  badges appear after 300 ms of held control and vanish on release.
- [#1093](https://github.com/Aquaticats/Monochromatic/issues/1093):
  bare keys do nothing while the template line has focus.
- [#1094](https://github.com/Aquaticats/Monochromatic/issues/1094):
  ctrl f does nothing in settings and in the editor page.
- [#1095](https://github.com/Aquaticats/Monochromatic/issues/1095):
  ctrl f ctrl alt s and ctrl o open search settings and the picker.
- [#1096](https://github.com/Aquaticats/Monochromatic/issues/1096):
  ctrl left and ctrl right go to the previous and next track.
- [#1097](https://github.com/Aquaticats/Monochromatic/issues/1097):
  ctrl m cycles the mode and a held ctrl m cycles once.
- [#1098](https://github.com/Aquaticats/Monochromatic/issues/1098):
  ctrl shift backspace resets the template on the editor page.
- [#1099](https://github.com/Aquaticats/Monochromatic/issues/1099):
  ctrl shift g and ctrl g show the playing tracks folder.
- [#1100](https://github.com/Aquaticats/Monochromatic/issues/1100):
  double shift does nothing in settings and in the editor page.
- [#1101](https://github.com/Aquaticats/Monochromatic/issues/1101):
  escape on the editor page returns to the settings list and ctrl enter does too.
- [#1102](https://github.com/Aquaticats/Monochromatic/issues/1102):
  every menu row shows its shortcut right aligned in the row.
- [#1103](https://github.com/Aquaticats/Monochromatic/issues/1103):
  the tooltips name the shortcut of every button that has one.
- [#1104](https://github.com/Aquaticats/Monochromatic/issues/1104):
  the settings editor chords apply and reset in the two pane page.
- [#1105](https://github.com/Aquaticats/Monochromatic/issues/1105):
  in the dark scheme the off track is pure black with an outline border.
- [#1106](https://github.com/Aquaticats/Monochromatic/issues/1106):
  the keyboard ring hugs the switch row with the group corners.
- [#1107](https://github.com/Aquaticats/Monochromatic/issues/1107):
  the switch shows its state by thumb position track fill and mark.
- [#1108](https://github.com/Aquaticats/Monochromatic/issues/1108):
  the Linux trial omitted the source-selected Slint viewer tool wave;
  the corrected tool gate and every eligible Linux test now pass.

## Coherent replay preparer closures on October 17

The [verified closure receipts](../audit/depot-coherent-incident-closures-2026-10-17.evidence.json)
retain actual final provider proof and exact filed issue bodies.

- `#1168` is closed:
 all original logger host runner labels and applicable commands passed,
  including semantic-only `macos-15`.
- `#1169` is closed:
 the private dispatcher now passes the route through its child environment;
  all 20 routes were accepted after the pre-provider argument failure.
- `#1170` is closed:
 the Android collector reads the actual case result filename;
  the bound APK pair reached all 15 original device assertions.
- `#1171` is closed:
 the private cold-boot copy excludes unused NOCOW snapshot/transient state;
  retained entries still require reflinks,
  and the real device lifecycle and cleanup passed.

The Android source failures remain open in `#1002`;
13 device assertions passed and two failed,
with no skipped or missing assertions.
Source/tool compatibility diagnostics from the remaining cohort routes are still being reconciled.

## Executed Cargo formatter boundary on October 17

Issues `#1166` and `#1167` are closed after actual common-bound formatter execution.
The native selected Cargo file passed;
the desktop selected file failed the documented legacy-parser grammar gap.
Scoped source/configuration follow-up `#1172` remains open under `#562`.
The [verified incident receipts](../audit/depot-coherent-incident-closures-2026-10-17.evidence.json)
retain exact bodies and closing comments.
The native macOS retry remains active after separate API-connect and artifact-service timeout boundaries.
The separate transport and changed-Rust source-style incidents are tracked in the October 17 source/transport section.

## Source style and separate macOS transport incidents on October 17

- `#1173`:
  native changed-Rust prose-style errors.
- `#1174`:
  desktop changed-Rust prose-style and missing-rustdoc errors.
- `#1175`:
  native macOS after-provenance connection timeout at the Contents API.
- `#1176`:
  native macOS artifact-service CreateArtifact retry exhaustion.
- `#1177` is closed:
  the private verifier now distinguishes Alpine smoke/provenance archives
  and actual Scorecard output from echoed task source.

The [host consumer evidence](../audit/depot-coherent-host-consumers-2026-10-17.evidence.json)
retains exact filed bodies and source/platform consumer outcomes.
Native macOS decode-count assertion issue `#1178` is open.
The actual `tone.wav` count was two instead of one;
its cause remains unestablished.
The API and artifact failures have no established shared cause;
the unchanged-harness macOS retry remains under verification.

## Final TOML guard and native macOS repeat

Issue `#1179` is open for the TOML final source/cohort guard exit without a retained diagnostic.
The source mutation passed,
but the missing after record receives no final paired-provenance credit.
Native macOS attempt two has independently verified source/common records and artifact publication.
Its 1382 tests recorded 1356 passes and 26 known failures.
The decode-cache case in `#1178` passed on this repeat;
that issue and the separate transport issues `#1175` and `#1176` remain open.
The Windows row carried into attempt two is an alias of its original execution,
not another allocation.
The [final boundary receipts](../audit/depot-coherent-final-boundaries-2026-10-17.evidence.json)
retain those distinctions and the private evidence-construction findings still awaiting filing.
