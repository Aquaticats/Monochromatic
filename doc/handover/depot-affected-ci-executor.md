# Depot affected CI executor

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/nested-wayland-session` is now `package/nested-wayland-session/cli`.

## Purpose and boundary

Run the complete applicable CI surface for the historical TOML, native, logger,
and desktop changes, preserving each source revision and dependency lockfile.
Package Oxlint always receives the unchanged package task with no file arguments.
Partial waves remain partial evidence, never a complete-graph timing claim.

The isolated harness is branch `trial/depot-complete-ci-20261016`.
Its latest shared executor harness commit is `203bef4198`.
Manifests live under `.depot/manifests/` on that branch.
The executor records selected and deferred checks, per-check exit status,
source and manifest identity, expected unit files, and input hashes.

## Cases and ownership

- TOML: `525d2e09f8168a3bdd7e513e26d4c80b363123e5`, six affected owners.
  This executor owns package checks, container and mutation checks,
  and generated-file consistency.
- Native: `cb95cf6d785c3073c192b95f0632cdae7b910894`, four affected owners.
  This executor owns Linux core, Android APK producers, Linux desktop,
  and feature combinations. The main agent owns the disposable Android consumer
  and Windows/macOS dispatch.
- Logger: the manifest records the historical source revision and 158 owners.
  This executor owns package checks and bounded scheduling.
- Desktop: `3434eaa455d569c18b52cf15100f77f9db251d21`.
  The main agent owns its Windows and Alpine workflow replay.
- The main agent owns changed-file scanner workflows.
  The scheduling agent owns Scorecard and provider accounting.

## Run ledger

- Preparation `1v4l9tglbt` failed because its shallow checkout lacked the requested ancestor.
  Preparation `tg907s5kfs` passed after full history was fetched.
  Issue #1057 tracks the observed ancestry failure and verified remedy.
- TOML `54sv5gl5l7` was cancelled after review found unnecessary prerequisite edges
  that could suppress independent checks. The corrected manifest removes those edges.
- TOML `zn4q03mh59` failed before checks because the fresh historical checkout's
  Git policy executable had not been built. The harness now invokes its existing build task.
- TOML `zgsc58pmfc` and remaining checks `zxdc83cpbp` failed during setup:
  mise changed historical lock metadata, and a generated-file fixture needed policy trust.
  The next harness uses `MISE_LOCKED=1` for source commands and explicitly trusts
  the reviewed historical repository policy after building its executable.
- TOML core `hgqj2nstwq`, attempt `811p86php8`, completed every selected core check:
  16 passed, 5 failed, and 3 container/mutation checks were explicitly deferred.
  All four full-package Oxlint checks failed.
  The `tsgolint` diagnostic includes `panic: unknown rule: no-generated-empty-object-type`.
  The coverage gate reports `src/values.ts: 332 -> 304 covered lines`.
  Builds, declared unit files, fuzz campaigns, conformance, and the live-source plugin test passed.
- TOML remaining `d1jgjp62lg` failed.
  Container tests failed after 88.057 seconds.
  TOML mutation failed after 957.015 seconds with manifest-read `EACCES` errors
  under the rootful container bridge; it reported 3062 runtime errors.
  File-enforcer mutation failed after 0.323 seconds because its package argument
  was resolved beneath its package working directory a second time.
  Generated consistency failed before generation because Git LFS tried an obsolete endpoint.
  Retain these failures separately from product-check failures.
- Executor controls `g57729x66d` ended with the passing fixture successful
  and the failure, Oxlint-argument, and Oxlint-scope fixtures failed as expected.
  Each terminal log confirms the intended assertion boundary:
  passing task exit 0, deliberate task exit 1, forbidden Oxlint arguments,
  and forbidden Oxlint file scope.

The control result metadata incorrectly reported 16 CPUs and 64 GiB;
its immutable workflow allocates 2 CPUs and 8 GiB.
Use workflow allocation for accounting. Commit `e84911e320` corrects future control records.
Some TypeScript output omitted a final newline, joining an executor event to a `TSFILE` path.
Commit `a65a41cd08` starts every structured executor event with a newline.
The original aggregate case record remains the reconciliation source for that attempt.

- Native setup `5tbb7qftkw` failed because the older JavaScript bootstrap was copied
  to a revision using a published native Git wrapper. Its unrelated offline source build
  lacked the `ignore` registry package. The published-tool bootstrap replaces that step.
- Native `j80v9dqfb4` reached the selected core and Android checks.
  Core build, default build, all-feature tests, benchmark build, and benchmark Clippy passed.
  Core Clippy rejected `clippy::assertions_on_constants` at `cache_tests.rs:300`;
  changed-Rust style also failed.
  Android passed all eight selected producer checks, including both native ABIs,
  debug/release APKs, lint, Clippy, Detekt, JVM tests, and instrumented APK assembly.
  Its artifact is `01a142b1-8fab-7b97-a8fe-257a04d5c35d`,
  SHA-256 `0021a6cda4b4848ec4fc96280894800b3899329f05bfe94e5bfaf58c8cf33d51`.
- Desktop `xg3znh2j0p` failed before checks at an overly strict existing-Clang-directory assertion,
  followed by a harness-relative import of an uninstalled workspace dependency.
  Commit `43c0fb4e8f` verifies the existing headers using Clang
  and resolves the utility from its installed source consumer.
- Scheduler controls `7vqccn47nx` passed their intended boundaries:
  independent build ran despite another check failing,
  the failed dependency suppressed its consumer,
  and the passing consumer read the build marker.
- Virtualization probe `1dg9t9znbj` passed its read-only probe.
  `/dev/kvm` exists and is readable/writable; `virsh` and guest images are absent.
  The `mvm` executable resolves to an unbuilt historical workspace entry point.
  No guest was created. The main agent owns disposable VM fixture preparation and execution.
- Logger static `qm7w64tmhj` reached terminal failure after completing its selected wave.
  The full aggregate record is retained for independent reconciliation.
- Native desktop `w6ln4j09fz` passed build, Clippy and 1406 nextest tests;
  Slint lacked `libinput.so.10`, and the E2E control pipe lacked standard input.
  Native replay `lvcwx21cbt` and desktop-case replay `ssvlvk57k4` verify the stdin repair,
  but a second harness defect rejected the compositor's valid `ok wayland-1` response.
  The source protocol at `package/cli/nested-wayland-session/src/protocol.rs:557`
  prefixes successful data responses with `ok `.
  Replays `nb6ptmwjs5` and `xmvgm200k5` use the corrected parser.
- Desktop-case `ssvlvk57k4` executed 1067 of 1460 nextest cases:
  1064 passed, three failed, one skipped, and 393 were not executed after fail-fast.
  Failures are `watcher_drives_rescan_update_through_engine`,
  `idle_root_emits_no_extra_queue_updates`, and
  `a_burst_of_writes_is_one_report_naming_each_file_once`.
  The main agent's verified in-memory no-fail-fast adapter is available for replay.
- Expanded controls `dxk4v3s0w3` reached their expected provider statuses:
  passing and deferred-prerequisite succeeded; failure, Oxlint-argument,
  and Oxlint-scope controls failed. Terminal assertion logs remain to be reconciled.

The Android archive was downloaded and its provider SHA-256 verified.
The main agent received debug, release, and instrumentation APK paths plus their individual hashes
for the disposable headed emulator consumer check.
Issue comments now record the ancestry remedy on #1057,
graph and metadata corrections on #1030, and sidecar-ignore evidence on #1012.

The executor now closes every selected wave over its prerequisites.
Deferred checks cannot satisfy selected consumers.
The fifth control specifically selects a type consumer whose build falls outside
the initial static wave and requires that build's actual marker.
The independently reviewed native manifest removes false core-build barriers:
the app and benchmark Cargo tasks compile their path dependencies themselves.
Logger Oxlint depends on the shared configuration build inside the graph;
configuration failure does not suppress independent type checks.

All attempts, including setup failures, remain part of provider cost accounting.

## Corrected graph and consumer evidence

The authoritative logger static artifact has 718 records:
153 passed, 171 failed and 394 skipped.
Logger unit `xz7cchwbsk` has 718 records:
211 passed, 15 failed and 492 skipped.
Both used the old manifest, which declared some build requirements outside its scheduling edges.
Those results do not establish source defects for missing build artifacts.

Conservative graph commits `172f8e6ec0` and `3573bdfe2e` retain all existing prerequisites,
consume the declared artifact prerequisites, and add the bounded Rust fuzz smoke task.
Independent verification found 719 unique checks, no cycles and no removed prior prerequisite.
The import/runtime audit remains open before final graph freezing.
The static and unit closures now cover 564 checks; the remaining partitions cover 155.

Each executed check now retains a dedicated output file, byte count and SHA-256.
The executor rejects unused artifact-prerequisite metadata.
Explicit `DEPOT_CI_CHECK_IDS` selection preserves prerequisite closure;
control run `m00504dz4c` exercises that boundary and retained output.

Native feature run `79lgpx91sf` passed all selected suites:
default 58 tests, fingerprint 67 tests, and service 71 tests, with none skipped.

WireGuard `1pfkxwlwbd` passed endpoint routing but failed the watcher process cleanup assertion.
Replay `cwkngc2w7g` adds an init reaper and reaches a later source failure:
the test rewrites `ExemptMark = 9999`, then `down` rejects its reserved `0x2000` bits.
Endpoint routing passes again, and both private containers pass cleanup checks.
The init replay stops without the prior SIGKILL fallback warning.

Pi `4z014g290p` passed terminal answers/cancellation, actual notification delivery,
and the offline tmux tests; its configured-extension prerequisite lacked clone authentication.
Replay `04pjg1sw9j` builds that prerequisite and loads 22 configured extensions,
with no fixture extensions, in the offline container.
The authored serializer assertion then fails:
`The normal native serializer must retain its original system input derivation`.
The other consumer checks pass again; artifact ownership and container cleanup checks pass.

Mutation controls preserve the original denied private-directory probe.
`g1nngvfsrg` rejects the user-namespace approach with
`cannot unmount /sys/kernel/debug: device or resource busy`.
`70p38v12d7` rejects unsupported tmpfs `uid=` options.
Run `0snqhv0rhl` tests explicit host UID/GID execution with a private writable tmpfs,
preserving the bind directory's restrictive permissions.

Private image job `wqz9h633wj` receives only the audited historical image archive,
verifies its SHA-256 and both original image IDs, then prepares the private snapshot
`monochromatic-trial-native-bases:20261016-v1`.
The archive is 2329816576 bytes with SHA-256
`c377f23ca6284f3d8020cb8697cf7654eaa7549a25c05150c6478ac139f50fb4`.
Restoration is pending; the images cannot yet satisfy an execution prerequisite.

Tracked failures are #1066 for the full-package lint panic,
#1067 for TOML coverage, #1068 for the duplicated mutation package path,
#1069 for the constant Clippy assertion, #1070 for container termination,
and #1071 for the obsolete LFS endpoint.

## Queue

1.  Retain terminal TOML diagnostics and check records in the audit and incident trackers.
2.  Collect the running logger, desktop, virtualization, and control results.
3.  Collect the main agent's headed Android consumer result.
4.  Execute the native feature matrix and full desktop runtime suite.
5.  Complete bounded logger waves, including declared browser and integration surfaces.
    The coverage agent owns the offline configured-Pi extension,
    notification, terminal, and tmux consumer adapters.
6.  Complete generated-file and container/mutation TOML evidence.
7.  Collect terminal scheduler-control assertions. Cycle evidence is recorded on #339.
    Deduplicate and document bootstrap, lock metadata, trust, tsgolint panic,
    coverage, rootful mutation permissions, duplicated mutation package path,
    obsolete LFS endpoint, desktop setup, configured-Pi assertion,
    WireGuard fixture, and Linux nextest failures separately.
    Add resource metadata and event delimiter
    corrections to the existing execution-evidence issue.
8.  Render-check this handover and all incident documentation.
9.  Remove disposable catalog worktrees only after inspecting ignored files and Git-directory hazards.

## Constraints and verification

The source `mise.lock` resolves Rust `nightly-2026-10-13`, but mise's installed-tool
selection probe chose `nightly-2026-10-15` without an explicit override.
The main agent verified `MISE_RUST_VERSION=nightly-2026-10-13` selects the required toolchain.
Install exact versions outside source, then verify actual executable versions inside source.
The native workflow installs Rust and `cargo-binstall` before dependent Cargo tools.
The mise action runs in the harness directory because its environment activation
must not alter historical source lock metadata.

The `cb95cf6` desktop E2E source selects six sessions when `E2E_ONLY` is empty:
wide, narrow, dialog, quit, states, and visual.
Its authored host build overrides `LIBCLANG_PATH` to `/usr/lib64` and chooses
Clang resources from `/usr/lib/clang/*`.
Ubuntu preparation must bridge those filesystem paths after probing installed libraries.
The runtime requires a disposable systemd user manager, parent Wayland compositor,
PipeWire services, ImageMagick, and the repository compositor artifact.
The private runtime uses 2 CPUs, 2 GiB, and a 512-process limit.
Successful suite exit is insufficient: audio sinks, user units, processes,
container removal, and overlay isolation must also pass before its record reports success.

The main worktree's modified `mise.lock` belongs to concurrent work and must remain untouched.
Cloud jobs are bounded to the declared `16x64` runner with a 30-minute job limit.
Container wrappers retain the reviewed 2 GiB memory and 2 CPU limits per container.
No cloud Android emulator is authorized by this task arrangement.

The final Android consumer is prepared separately at
`~/temp/agent/depot-coherent-android-consumer-20261017/mise.toml`.
The [consumer preparation](../audit/depot-coherent-android-consumer-2026-10-17.md)
and [input evidence](../audit/depot-coherent-android-consumer-2026-10-17.evidence.json)
retain the immutable native 15-test catalog,
reused APK-hash/full-status capture,
private headful launch bindings,
and required producer/cohort/source/harness joins.
The historical 13-pass/two-failure record remains separate.
Preparation verification caught and corrected a stale launch-unit literal before any launch;
[#1115](https://github.com/Aquaticats/Monochromatic/issues/1115) is closed with parsed-command controls.
The current staged harness hash is recorded in the preparation evidence,
and every parsed runtime task rejects the legacy unit name.
The held input validator now binds `cases.native.sourceRevision`,
the separate native case digest,
and `native.producers` in the settled outer manifest.
It requires `.depot/workflows/depot-complete-native.yml` and the `native-android` matrix wave,
including the wave in the producer receipt.
Its thirteen offline controls pass,
including self-consistent wrong-route/workflow receipts and rejection of `native-core`;
the original five and settled-schema nine controls remain separate historical evidence.
The input/receipt uses `requestedAt` and distinguishes `cohortManifestSha256` from `caseManifestSha256`.
No emulator,
adb server,
or device operation ran during this preparation.
Root must supply the final producer run/job/attempt,
provider receipt/archive/APK hashes,
and the common cohort request/manifest binding before input validation and live consumer execution.
The final consumer must retain actual settings/resources,
all unfiltered test statuses,
graceful cleanup,
and local phase timing separately from provider compute.

Inspect manifest reach, complete check records, original diagnostics,
and source/input hashes when assessing the result. Report missing checks as missing evidence.

## Logger full replay on 2026-10-17

The logger execution agent owns package request `4x77lb56cs`
and external request `j4m7gw8mwg` through terminal reconciliation.
The frozen source is `ef01490f5e960f8ca0b424c12242caeb3c5a2b5e`;
development manifest SHA-256 is `f4cbd47f210f5b031d76c5fc1c28fa704b349621f91eefa3f4f3ad575acfedc3`.
On 2026-10-17, the candidate manifest became
`4f9bcf0a94e09ecda8eae84c8e5ff9d89f15f0bd398df8136f0771011d3931f7`
after correcting the Vmsync external route to authored `test:e2e` and its build prerequisite.
That correction retained all 719 package specifications and the other 18 external specifications.
The current candidate is
`d8cf02e46bafd03ab4f410ccf2a259a29c68c757af1cddf81735258372025af9`,
after adding the missing DOM and WC build prerequisites to their external checks.
Its package specifications remain identical to the original frozen inventory.
Independent closure verification confirms 719 package checks and 19 external specifications.
The graph remains acyclic and preserves full package Oxlint arguments.

The current host request `39003180992` passed all six Linux/macOS/Windows jobs,
with immutable source and dependency input hashes independently verified before and after each job.
The root agent owns current scanners and the Scorecard gate.
The held VM route is clean and remotely verified at
`8719ad3db0167574a5a8324132fa9a0bb74fd098`
on `trial/depot-complete-vm-20261016`.
It carries current logger manifest digest
`d8cf02e46bafd03ab4f410ccf2a259a29c68c757af1cddf81735258372025af9`,
requires the common cohort strings,
verifies the exact base64-decoded manifest and `logger.vm` route binding,
and retains reviewed source identity guards before/after execution.
The earlier 28-control preparation at `b2dcdc40d8ae2cfb1f7e13001db28ad8f17626b3`
remains separate historical evidence.
The same cohort/source controls were repeated against the current route.
The unchanged authored `test:e2e` command retains its natural build dependency and failed required-check semantics.
Both original lifecycle identities remain unexecuted until the actual source task reaches them.
An additional standalone bundle consumer is now registered after that original command.
It requires actual natural-build and source/cohort receipts,
then passes only the produced bundle,
its receipt,
Node,
and required ELF libraries to a no-network,
read-only,
two-CPU/2-GiB container.
It can pass after the source task fails without changing the failed source gate.
The actual 19-control artifact suite and two default-rootful controls passed locally,
including the real #1076 failure,
standalone `--help`/empty `list`,
exact artifact/provenance joins,
rejected altered handoffs,
and removed container/rootfs.
The initial mapped-owned rootfs cleanup defect is retained separately as #1134;
the verified `:O` overlay repair leaves copied lower inputs unchanged.
Neither lifecycle body,
guest preparation,
nor an emulator is exercised by those smoke checks.
See the [VM route contract](../troubleshooting/depot-vm-runtime-paths.md#common-cohort-binding)
and [artifact evidence](../audit/depot-vm-artifact-consumer-2026-10-17.evidence.json).
No final dispatch or guest preparation occurred.

Both requests are terminal and collected with hashed artifact downloads.
Package reconciliation verifies 685 identities with terminal records,
three started checks without terminal records and 31 unstarted checks.
That preserved snapshot covers `4x77lb56cs` only.
The independently hash-verified global development union includes external executor records
and has 686 terminal package identities and 33 unresolved identities;
the browser-properties bundle supplies the additional terminal identity.
Development prerequisite recovery `dhxhzzvmc3` is terminal
at pushed revision `2676cb365a206434b525bf032a5875cafde827e9`.
All 12 selected package checks and their 24-check closure passed;
the sidecar generator commands passed without output drift.
The browser group reached the engines but exceeded its 1,200,000 ms adapter budget;
DOM setup failed on missing artifacts and browser thread creation.
Their incomplete case coverage requires explicit file/engine recovery with case JSON.
Provider URL verification matches each artifact to its actual workflow and attempt,
with request IDs kept distinct from workflow IDs.
Invalid request `tf9ddwgbw3` used the wrong revision and receives no historical check credit.
Continuation routing at `7f9fb86ba7` selects the 34 unresolved checks
and three proved rsync ownership infrastructure failures in isolated jobs.
Continuation `1wzr86fssk` failed at clean pushed revision
`09941541e8ba43d59b03969248f8484a2d2c4269`, independently verified by the provider.
All 37 collected logs prove the same user-namespace `/sys/kernel/debug` preflight failure,
with no source execution; all 37 artifacts are hashed.
Owned-volume pilot `85fv28fjkl` passed its cloud boundary controls at pushed `bc6889aaaa8d`,
then exposed unreadable baked pnpm metadata in the unchanged CLI integration.
That log proves the sender's permission denial without measuring UID, GID or mode.
All 49 shard volumes were verified removed.
Request `360d91ht6w` at pushed `d098532a5b69` passed the unchanged CLI integration and its two builds.
All three logs and unchanged source inputs are independently verified.
Six readonly metadata binds preserve original image bytes and identity;
actual metadata measured root `0:0`, mode `0600`, and 33,427 bytes.
All six shard volumes and six inspection containers were verified removed.
Issues `#1116` and `#1117` retain the distinct incidents and actual consumer closure proofs.
Executor metadata now distinguishes inventory, requested IDs, prerequisite closure,
live progress and actual case termination; its five disposable controls passed.
Continuation `1pgpb8qhn6` started at `2026-10-17T06:53:35Z`
from clean pushed `51d5a2fd69507d9346d7fe6eb6345a4ee9929383`, matching the provider's source and head revision.
Its 35 selectors are exactly the 33 unresolved identities plus two remaining ownership repairs,
with frozen prerequisite closures verified.
The passed browser-properties bundle and mutation CLI integration are excluded from development recovery.
Retain its actual `16x64` allocation when accounting for compute.
All 35 jobs/artifacts are collected: 26 provider successes and nine failures.
Six failures expose source diagnostics; conf fuzz, TOML mutation and conf mutation remain incomplete
after the 24-minute adapter budget, with explicit `started-without-terminal-record` receipts.
The two mutation campaigns still completed assertions and shards at cutoff.
Long recovery `lt7qwlz6h1`, requested at `2026-10-17T07:35:35Z`, selects only those three unfinished tasks
with a bounded two-hour allowance at clean pushed `f69a78a3b31f81cbfe47496bcdc6e2ac75549777`,
which matches both provider revisions,
using `2x8` for the single fuzz container and `4x16` for the authored two mutation containers.
The browser routes now declare 32 disjoint file/project cells,
with generated-program positive,
source-negative and invalid-inventory controls passed.
Two supported `2x8` pilot cells ran in `x0rb16tjzb`,
with provider source and head revision matching clean pushed `c0eefbea382ef41c4667e90242010757eb5f759f`.
They are now independently verified successful, with five passing Playwright cases,
all three logger backend properties passed,
full 84/34-case inventories and exact partitions,
and actual two-CPU/two-GiB/512-PID bounds without PID or OOM events.
The remaining 30 cells ran in `jps875l9qs` at pushed `51d5a2fd69507d9346d7fe6eb6345a4ee9929383`;
independent reconciliation of both requests verifies all 118 configured cases:
116 passed, one failed and one authored skip, with none unexecuted.
WebKit's logger OPFS availability assertion received `false` where it expected `true`;
the other WebKit OPFS consumers explicitly skip an unavailable backend.
Every prerequisite/external log hash and actual attempt URL matches,
and all resource samples have zero PID rejection and OOM events.
The 32 actual `2x8` jobs consumed 9,363.94 allocated CPU-seconds,
or $0.468197 published compute before credits.
The separately verified 35-leaf request consumed 177,819.248 allocated CPU-seconds,
or $8.8909624 published compute before credits, including incomplete attempts.
Git host recovery `10zmqch10h` verifies exact Git `2.56.0`,
its real installed ELF and linked-library hashes,
an actual Git commit and the literal `/proc/PID/exe` boundary in bounded private worker userland.
Both unchanged host tasks execute with source input verification and cleanup:
binary-host reports 136 passed, one failed and one ignored;
library-host reports 1,805 passed, seven failed and four ignored.
Two Markdown failures are unset `GIT_POLICY_NATIVE_TEST_LINTER` prerequisites,
not exercised real-linter assertions;
the previous blanket source-failure characterization is retracted.
Build and gate the authentic immutable-source linter before replaying both original failed host checks.
The native accounting owner is matching the remaining six actual assertions to existing incidents.
The separate prior Git `2.55.0` prerequisite gap is repaired for this adapter.
The provider allocation consumed 1,144.408 allocated CPU-seconds or $0.0572204 published compute before credits.
Current development evidence has terminal observations for 717 package identities,
with two never-terminal tasks and three latest campaigns still incomplete.
Keep the original `685/34` and `686/33` snapshots as scoped historical evidence.
Collect only the three unfinished campaigns,
complete remaining source-issue mapping and wire the common routes.
The root agent must align GHA, VM and shared helper routes before final combined dispatch.
The prepared combined workflow remains incomplete until that integration and bounded check coverage are verified.
The settled common manifest lives at `doc/audit/depot-complete-cohort-${cohort_id}.json` on `main`.
Authenticate through GitHub's base64 object response and hash the decoded bytes.
Bind the three common inputs before and after execution,
distinguishing the outer cohort SHA-256 from `cases.logger.manifestSha256`.
Validate the logger source/base and selected route's actual harness revision.
The logger cohort receipt task at `040a4814a63e27e392a68c83bceaaff78441c962` is implemented and verified:
paired receipts, a 1,047,160-byte escaped-text payload, ten rejected invalid inputs,
and a digest-guard removal control.
Its checked fixture transport and the root's actual authenticated transport are separate proofs.
Workflow wiring and the final manifest freeze remain pending.
The prepared 57-selector candidate covers all 155 remainder checks;
Git host two are split from the Rust six, leaving Rust four and regular 16.
Both primary and candidate manifests keep `completeGraph: false`.
The premature `freeze` wording in candidate commit `3ac9616096` is corrected by commit comment `205400693`.
Current OPFS evidence updates `#1052`; the verified CI/retry environment boundary is `#1131`.
The generated-consistency adapter must run its original task in a separate disposable consumer checkout.
Its own frozen dependency links must resolve within that consumer,
with bound initial HEAD/tree/input bytes and retained command, generated diff, output hashes and cleanup.
The authoritative checkout retains strict unchanged before/after guards.
Prove rejection of pre-task consumer drift and authoritative drift before final route registration.
The adapter already used separate detached fixtures and preserved the authoritative checkout;
the hypothesis that generated drift necessarily dirties authoritative source is retracted.
Its diff counters measure bytes.
The stronger adapter now isolates each authored invocation,
verifies tracked bytes/modes and workspace links before execution,
and retains its exact command, patch, output hashes, process outcome and cleanup.
Miniature positive, drift, link-leak, hidden-index and three guard-removal controls pass.
Real-source pilot `vt8vzj154t` executed no original source check:
its incorrect `DEPOT_LOGGER_EXTERNAL_GROUP` binding was rejected by the adapter's actual
`DEPOT_LOGGER_EXTERNAL` guard.
Commit `673ab92e49885dd61fcf08e168ed9c59bf29c6be` fixes that pilot binding;
actual-program prefix controls verify the correct selector and the rejecting guard.
Corrected bounded pilot `drzwk5q8mc` is now requested from the clean explicitly pushed ref.
Collect its actual consumer identity, generated output, source cleanliness and cleanup proofs before final registration.
The combined workflow already has the correct selector.
Independent candidate controls verify its 97 jobs, 57 disjoint selectors and all 719 package identities,
full-package Oxlint context, 19 external routes, 32 browser cells and paired common/source guards.
Root integration of canonical helpers, GHA and VM routes remains pending; `completeGraph` remains false.
Authentic linter recovery `q10910cxfm` is independently verified at clean pushed
`c789a14548fec9df1cee01facb9577d83ddfd42c`.
The real source-built linter fixes the expected stdin Markdown and is idempotent;
both original Markdown assertions now pass.
Original binary-host is 137 passed, zero failed and one ignored;
library-host is 1,806 passed, six failed and four ignored.
The six actual failures equal the independent native mapping at `8661765628`;
their distinct, remotely verified incidents are `#1135` through `#1140`.
Source inputs, log hashes, actual attempt URL and cleanup match.
The linter sample retains 9,663 memory-limit events with zero OOM and PID rejections.
Its actual `2x8` provider usage is 1,374.014 allocated CPU-seconds, or $0.0687007 before credits.
Pilot `drzwk5q8mc` is terminal without source generator execution:
the guard incorrectly required local links despite frozen pnpm direct-dependency deduplication.
The correction at `8b64f79013` has seven passing fixture controls and four guard-removal acceptances.
Actual pilot `ctg9148f4t` verifies three independent consumers,
all 13,715 frozen tracked inputs, 809 workspace links and strict equal authoritative before/after snapshots.
All consumers and the independent patch-reconstruction fixture are removed.
Its root generator exits zero but changes `mise.toml` PATH entries with a retained 2,037-byte patch;
the exact generated bytes are reconstructed and hash-verified as authored-generated-output drift.
The first deepmerge type generator passes without drift.
The declared-type generator's process is invoked but Podman rejects rootless `newuidmap` before its corpus consumer.
The existing verified rootful forwarder is now activated only for that invocation.
Recovery `62nbb3srhk`, provider-matched to clean pushed `6a17c8fc61cf7613432c9f481416fa9ef7cc7bfe`,
selects only that unfinished invocation and explicitly reports partial external-check coverage.
Its original corpus consumer now passes with 439 kept/61 deliberately dropped draws and no generated drift;
all frozen inputs, strict authoritative after identity, five log hashes and actual attempt URL match.
The job separately fails deleting the disposable checkout with `Permission denied`.
The predecessor has no output ownership stat, so its UID/GID remains unrecorded.
The confined cleanup helper passes one ordinary, eight rejection and one guard-removal controls.
Cloud cleanup-only control `qsg8tx4bvb` is requested from clean pushed
`bdd4144b991a98b1204621f8d3e304e37ac8abf9` on `2x8`, without repeating the completed generator.
Three valid/five rejected selector controls, a guard-removal control and two prerequisite controls pass.
Root has integrated the unchanged generated identity helper and owns its strengthened cohort guard;
preserve that guard while grafting the generated branch's new forwarding/selection logic.
Cleanup control `qsg8tx4bvb` measures root `0:0` output and reproduces ordinary deletion failure;
Git already removes registration, so the privileged Git retry is rejected.
The orphan-only correction at `53514e17d6` passes `p3jlkgvl99`,
with strict source identity, image config, metadata,
confined removal and no remaining containers independently verified.
Ten negative fixtures, the ordinary positive fixture and the guard-removal control pass, all removed.
Long-three is independently reconciled: conf fuzz and TOML mutation pass;
conf mutation exits one from a red `src/conf.cache.unit.test.ts` baseline.
All three have actual source terminal records and nine verified logs, with no wrapper timeouts.
Native mutation-report JSON was omitted from these historical artifacts;
per-mutant reconciliation is unavailable, and CLI totals are explicitly summaries.
Development union v3 has all 719 package identities terminal, with no latest unfinished task;
it remains separate from coherent replay and does not infer eligibility from terminal state.
The native-report collector has four valid, seven rejected and one guard-removal control.
The emitted program's syntax/environment controls pass;
package type checking stops at this worktree's absent `nano-spawn` dependency before checking types.
Control `r1mn88ccqt` terminates before mutation invocation:
its diagnostic-only workflow omits frozen pnpm installation and caught-value cannot resolve config-rolldown.
The exact failed prerequisite log and strict source guards are retained.
Recovery `knsghczl71` at clean-pushed `392a3ea04128547d9b452dd3bea7a4df1235acd1` is independently verified.
Its original bounded TOML CLI retains a fresh 650-byte native report:
`9cd1056a095eaea1` is `compileError`, zero unrun IDs, zero mutant assertion/whole-check credit.
Both original build prerequisites, strict source identity, 33,427-byte readonly metadata,
original/copied/archive hashes and actual volume/container cleanup verify.
The actual `2x8` allocation is 360.034 CPU-seconds, or $0.0180017 before credits.
Parent commits `d30d88df0d` and `ce052bc29c` integrate the verified external/cleanup and remainder/report blocks,
including the combined native-verification archive path.
Its canonical guards/native/desktop helpers remain authoritative; diagnostic-only pilot code is unnecessary.
Async/matcher/p-map fuzz tasks source-fail in their first differential property file;
their later fork/limits property file is explicitly unexecuted under the authored fail-fast loop.
Distinct source boundaries are filed as `#1145` to `#1156`;
verified scoped trial repairs are closed as `#1157` to `#1159` and `#1161` to `#1163`.
Existing `#335`/`#1047` have additive consumer proofs; `#1160` stays open for final per-ID artifact reconciliation.
The initial `#1162` field-name direction is explicitly corrected from immutable caller/receiver definitions.
Classifier `#1142` has byte-identical deciding modules in both relevant source revisions;
native `killed` totals do not imply literal assertion credit or prove a caught-spawn occurrence here.
Current-ref candidate controls pass at `392a3ea041`:
719 package identities, 19 external routes, 57 disjoint remainder selectors and 97 combined jobs.
The combined route includes both browser checks in all 32 disjoint cells;
the separate browser workflow duplicates those cells and must not also run.
The original Podman boundary omits `CI`, so actual authored retries remain zero;
the intended two-retry CI branch is unexercised and tracked by `#1131`.
The generated invocation labels are `root.generated-consistency.0`,
`sidecar.generated-type-consistency.0` and `sidecar.generated-type-consistency.1`.
The VM route retains original `test:e2e` without arguments and its Node build prerequisite;
both original lifecycle cases remain source-blocked before registration by `#1076`.
The companion evidence's `freezeReadiness` binds the exact parent receipt,
current clean remote-matched ref and fresh controls.
Root owns the remaining queue: complete-manifest/auxiliary digest regeneration,
current GHA/VM bindings, coherent authenticated dispatch and final source/prerequisite/per-ID reconciliation.
No generated outputs will be restored to manufacture a clean receipt.
Host-scope correction `#1168` preserves the original four-label union, including semantic-only `macos-15`.
The source-only transfer is `5db0268856`, `dc5423d361` and artifact follow-up `face4f6199`, based on `6aefa5d651`.
The follow-up keeps all four artifact names disjoint; equal simulated macOS architecture aliases the legacy names.
No actual provider artifact collision is claimed.
Four positive, 70 rejected and a true guard-removal result control pass; protected helpers remain byte-equal.
Primary manifest correction and actual final `macos-15` provider proof remain parent-owned and unexecuted.
See [the host scope audit](../audit/depot-logger-host-scope-2026-10-17.md).
Only harness execution failures receive recovery edits;
historical source failures remain diagnostics.
The requests were launched separately and do not establish a single complete passing pipeline latency.
See [the logger replay audit](../audit/depot-logger-complete-2026-10-17.md)
and its companion evidence for current outcomes and limitations.
