# Depot expanded CI evaluation working report

## Decision and acceptance limits

Use Depot as an additional execution and build pool,
with Developer as the requested planning baseline.
GitHub Actions and GitHub-hosted runners remain heavily used in the permanent design.
Depot adds capacity and capabilities through managed GitHub runners,
selected native CI workflows and its other services.
Heavy use of GitHub's own compute is a requirement,
not merely retention of GitHub orchestration around Depot workers.
The expanded evaluation covers application owners,
artifact consumers,
caching,
containers,
VMs and test infrastructure;
the [execution handover][depot-source-1]
records the accepted scope and remaining work.
The requested interview is settled;
the independent whole-report sufficiency review is now recorded in
[the review record](depot-expanded-ci-evaluation-independent-review.md).

Correction (2026-10-18):
the earlier text said the interview was in progress and the
independent whole-report review remained pending.
The interview has since settled the
operating model and the review has run;
the record is cited above.
The original sentence is
preserved in this note.

The evidence supports a selective hybrid rather than a blanket whole-CI speed multiplier.
Native Depot CI and Depot-managed GitHub Actions are distinct products:
the [native CI overview](https://depot.dev/docs/ci/overview)
limits native sandboxes to Linux,
while the [GitHub runner catalog](https://depot.dev/docs/github-actions/runner-types)
lists Windows and Apple Silicon Mac compute.
Developer makes Depot Windows available;
Mac eligibility starts with Startup
([current eligibility and allowance sources](../audit/depot-included-allowance-2026-10-18.evidence.json);
[public pricing feature rows](../audit/depot-live-public-pricing-2026-10-18.evidence.json.gz)).

Correction (2026-10-18):
the earlier citation pointed only to the allowance JSON,
which
records Windows rates but no Windows plan-eligibility flag;
its only eligibility booleans
are macOS `DeveloperEligible` and `StartupEligible` at lines 163-166.
The public pricing
feature rows show Windows Runners for all plans,
and the execution handover records at
line 1794 that Developer includes Depot Windows runners.
The original claim and citation
are preserved in this note.

The historical native follow-up's price comparison covers native CI usage alone.
It does not establish that Developer is always cheaper across separate product allowances.
For example,
the explicitly hypothetical monthly input of 20000 native two-CPU base minutes and
20000 Linux GitHub-runner two-CPU base minutes costs USD 236 on Developer versus USD 200 on Startup,
before unspecified services or charges
([executed combined model][depot-source-2]).
The USD 200 ceiling therefore requires an explicit workload allocation.

GitHub remains the forge,
check consumer and a heavily used execution provider.
Ordinary Linux work remains part of its workload allocation,
alongside platform-specific and publishing work.
Selected Linux workflows can qualify for native Depot CI;
Depot-managed GitHub runners provide an additional Linux AMD64/ARM64 and Windows pool.
This corrects the former wording that described native Depot as the Linux orchestration pool.
Neither native Depot nor Depot-managed runners become a blanket replacement for GitHub-hosted execution.
GitHub scheduling also retains release permissions and the release workflow bindings that native
compatibility does not cover.
The repository currently has no protected-environment approval or reusable-workflow boundary.

Correction (2026-10-18):
the earlier text said GitHub scheduling retains
protected-environment approvals and reusable-workflow references.
At the pinned revision,
no
workflow uses `environment:`,
no workflow defines `workflow_call`,
and the repository has no
reusable-workflow references;
the release boundary is push-triggered automatic publishing
(`npm-release`,
`pnpr-publish`,
`cargo-publish`,
`music-player-release`).
The original
sentence is preserved in this note.
Mac work stays on public GitHub under Developer.
The exercised ARM owners establish selected portability,
not every ARM workload
([native contracts][depot-source-3],
[actual ARM consumers][depot-source-4]).

## Immutable workloads and scope

The expanded repository source is `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`.
Application source,
research harness revisions,
generated producers and consumer receipts retain separate identities in the
[coverage research record][depot-source-5],
[Rust record][depot-source-6]
and [runtime record](depot-runtime-orchestration-research.md).
Later main changes do not silently enter that cohort.

Documentation,
inspected source and runtime-qualified consumers are separate evidence levels.
Native filesystem snapshots,
ordinary OCI job/service images,
compiled target archives and SDK custom runtimes retain different formats and identities.
The SDK custom thin-image success does not qualify ordinary OCI or named runtimes;
planned SDK snapshot and secret APIs are not delivered-feature evidence
([platform ledger][depot-source-7],
[prepared filesystem and archive controls][depot-source-8]).

The original trial's four historical source cases and native follow-up cohorts retain their own manifests.
Their 770 primary gates are a historical inventory,
not the denominator of the expanded repository evaluation
([historical evaluation](../audit/tech-depot-complete-ci-trial-vet-2026-10-18.md)).
Counts of source files,
owning task invocations,
enumerated tests,
executed bodies and result-upload events remain separate.
The current-source tree has 223 package directories:
208 active,
13 paused and two archived.
The initial 227-candidate census included four documentation blobs;
the corrected directory count does not change task or execution counts.
It has 2,425 source-filtered repository Mise tasks across 13,860 tracked files,
excluding 19 inherited scratch-ancestor tasks.

Correction (2026-10-18):
the earlier sentence said "2,425 source-owned Mise tasks across
13,860 tracked paths".
The manifest records `repositoryTaskCount: 2425`,
`sourceTreeFiles: 13860` and `excludedAncestorTaskCount: 19`;
the cited research says
"2,425 repository Mise tasks and 13,860 tracked files".
The original sentence is preserved
in this note;
the counts are unchanged.
Its source-language inventory includes JavaScript/TypeScript,
Kotlin,
Rust,
Slint and C/C++;
the absence of Go or Python owners is a measured scope result,
not a reason to install unrelated task systems
([source inventory and applicability controls][depot-source-5]).
The platform source ledger now reconciles all 120 captured deployed documentation pages,
with source/hash-bound findings and explicit applicability limits.
The ledger's top-level status remains `Research reopened; no readiness claim`;
the 120-page reconciliation is a bounded source-review result,
not a readiness claim.
This does not mean every public API method or documented example was runtime-exercised
([platform source ledger][depot-source-7]).

Correction (2026-10-18):
the earlier sentence did not record the ledger's top-level status.
The ledger at line 5 says "Research reopened;
no readiness claim";
its nested
`publicDocumentationReconciliation` status at line 16699 supports the 120-page
reconciliation.
The original sentence is preserved in this note;
the 120-page count is
unchanged.

The evidence is principally first-party execution data and inspected implementation source.
Primary receipts bind original ZIP/member hashes,
source revisions and actual consumers.
Documentation-only capabilities remain distinct from exercised repository workloads;
capturing a source page does not establish substantive applicability review.

## Gate and artifact-consumer results

### Actual producer and consumer boundaries

The fresh native artifact consumer verified all 1036 declared compiled hashes,
including the hidden manifest,
and restored 33,802 ZIP members.
Its first actual domain import failed because `runner.js` did not expose `nativeAssertActive`;
a separate flattened-artifact failure is retained.
Restoring source-owned activation and emitted dependency links from the same immutable artifact made
permission and prohibition pass with zero HTTP/provider requests;
the remaining `none` case reached the preserved `unresolved` versus `unspecified` assertion.
The consumer did not rebuild the application or rerun the unit suite
([source and retained controls][depot-source-5]).
An artifact-membership pass alone cannot qualify that runtime boundary.

Correction (2026-10-18):
the earlier text said "Flattened symlinks still broke its first
real import."
The cited research records that the fresh consumer's first actual domain
import failed because `runner.js` did not expose `nativeAssertActive`;
the flattened
artifact failure is a separate retained observation.
The same research records 33,802 ZIP
members restored alongside 1,036 declared compiled file hashes.
The original sentence is
preserved in this note;
1,036 is a file-hash count,
not an artifact-member count.

The packed-install profile activates the actual package manager and imports 25 installed runtime packages.
It separately compiles a positive and a rejecting TypeScript consumer for the type-only ownership-marker package.
Its unexpected runtime import of that type-only package is retained as a harness expectation error.
All five functional consumers pass,
including the actual mutation CLI's dry-run discovery of two mutants and selection of
`src/value.unit.test.ts`.
Discovery does not mean the mutants or fixture tests were executed
([packed consumer source and original receipts][depot-source-5]).

Correction (2026-10-18):
the earlier sentence said the dry-run covered two mutants "and
its tests" and that discovery did not mean the mutants were executed.
The cited research
records mutant discovery and test-file selection and says the dry-run executes neither
mutants nor fixture tests.
The original sentence is preserved in this note;
the five
passing functional controls are unchanged.

The browser profile exercised all eight browser specs in each engine:
Chromium passed 28 cases,
Firefox passed 28,
and WebKit passed 26 with one OPFS assertion failure and one authored skip.
The separate active DOM E2E profile exercised all four specs,
passing 17 cases each in Chromium and Firefox.
The original root E2E discovery failure remains retained separately from that explicit active-spec profile
([browser/E2E evidence and original receipts][depot-source-5]).

The current-source Rust work completed bounded ASan execution of all 35 frozen fuzz targets.
Truepeak's all-features Nextest owner passed 89 cases with two expected skips.
The actual Codex host returned forty real successful shell results through its unchanged signed binary,
using a deterministic local fixture rather than paid provider calls
([Rust primary receipt](../audit/depot-rust-coverage-2026-10-18.evidence.json)).
Planted-fault,
compiler-consumer,
corpus-provenance and privileged-WG evidence remain separate result families in that record.
The privileged WG replay passed all eleven ignored cases after explicit Node-PID-one and Tini reaping controls;
the fifteen ordinary cases were filtered from that targeted replay.
The missing namespace init is the supported explanation for that fixture's six preceding
process-identity teardown failures,
after the Node-PID-one and Tini reaping controls,
without changing frozen product source
([Rust execution record][depot-source-6]).

Correction (2026-10-18):
the earlier sentence said the missing namespace init "explains"
the six failures.
The cited research first calls unreaped orphan state an explicit
hypothesis at lines 820-826;
the later Tini control and eleven-pass replay at lines 846-848
support the explanation.
The original sentence is preserved in this note;
the six failures
and unchanged product source are unchanged.

The positive Truepeak search fixture completed the unchanged grid twice,
selected a winning candidate and emitted matching 24-row feature files of 8704 bytes.
That closes synthetic search and output emission;
it does not measure real audio collection or decoding
([benchmark boundary][depot-source-6]).

The native ARM64 repository profile passed the same Truepeak all-features owner:
89 passes and two intentional skips,
with all 91 JUnit identity/status tuples matching AMD64.
The built `p-limit-fork` owner also passed the same eleven runtime describe groups.
This is source-bound functional portability evidence,
without repeated timings or a whole-repository ARM qualification
([ARM64 workload and original receipt][depot-source-4]).

The exact cloud-produced Android APK pair has a headful local consumer:
all sixteen source-derived instrumentation cases were reported,
fourteen passed and two retained the known session-store context-fixture failure,
with no skips.
The receipt binds APK identities,
device settings and emulator/ADB cleanup
([Android consumer][depot-source-9]).
This functional device pass is separate from APK layout,
signing,
JNI and producer JVM checks.

Desktop execution now covers the original wide session and all six remaining authored sessions,
using the real application and nested compositor.
The wide session reports 69 passing assertions and one authored modifier-repeat skip,
then a separate track-loading harness error.
The remaining structured session summaries report 41 passes and one dialog assertion failure,
with no skips.
The failing assertion is `Ctrl+Shift+O opens the system folder dialog`.
Repeated assertions across layouts and duplicated raw log lines are not unique case identities.
The continuation retains ninety PNG artifact identities,
with cleanup verifying no app/compositor processes,
units or container and restored audio state
([desktop sessions and original artifacts][depot-source-5]).

The current-source native-host recovery executes 1495 Rust cases on the selected Mac contexts,
with 1468 passes and 27 failures,
and 1494 on the selected Windows contexts,
with 1493 passes and one failure.
The recorded binary/case identities and statuses match exactly for public Mac 15 versus Depot Mac,
and public Windows x64 versus Depot Windows x64.
The same comparator detects changed identities,
missing cases and changed statuses;
authored skips remain separate from this executed-case comparison.
The pass/fail split is recorded in the execution handover;
the comparator evidence records the 1495 and 1494 totals and the exact status equality
([host comparison and controls][depot-source-10];
[public OS recovery counts][depot-source-1]).

Correction (2026-10-18):
the earlier paragraph cited only [host comparison and controls]
[depot-source-10][] for the 1468/27 and 1493/1 pass/fail counts.
The comparator evidence
records 1495/1495 and 1494/1494 recorded cases and empty status-change arrays,
not
pass/fail counts.
The pass/fail split is recorded in the execution handover at lines
536-540.
The original citation and counts are preserved in this note.
These standalone-workspace owners do not exercise IDE,
terminal or file-manager tests through their Truepeak dependency.
The shared builds,
actual Mach-O/PE version consumers,
filesystem identity and semantic-plugin host/external consumers pass;
logger failures retain their child-versus-aggregate boundaries
([current owner execution][depot-source-5]).

The requested/catalog Depot Mac profile is eight CPUs and 24 GB,
but its actual Node probe reports seven available CPUs and 18253611008 memory bytes (17 GiB).
Public Mac reports three available CPUs and 7516192768 bytes (seven GiB).
Both Windows probes report four available CPUs:
17174360064 memory bytes on public GitHub and 16827592704 on Depot.
Depot's actual OS build and CPU model were not captured;
the catalog's Apple M2 is not an observed chip identity.
These measurements do not establish why the requested and observed Mac resources differ,
or a fair native-platform performance ratio
([retained environment observations][depot-source-10];
[catalog shape in the execution handover][depot-source-1]).

Correction (2026-10-18):
the earlier paragraph cited only the comparator evidence for the
eight-CPU/24-GB catalog shape.
The comparator JSON records `requestedLabel: depot-macos-15`
and the observed seven-CPU/17-GiB values,
not the catalog 8/24;
the catalog shape is
recorded in the execution handover at line 740.
The original paragraph and citation are
preserved in this note;
the measured values are unchanged.

The corrected public Windows GNU owner runs 1494 distinct binary/case identities:
1492 pass and two fail,
with one separate summary-only skip.
The failing cases are
`music-player::session::tests::an_unreadable_session_path_loads_as_defaults_and_is_logged_with_the_read_error`
and `watch::tests::a_burst_of_changes_requests_one_rescan`.
Repeated final-failure lines are not extra executed cases.
The original duplicate-argument adapter ran zero bodies and retains no test credit;
the corrected result preserves unchanged source assertions.
The extra GNU watch failure's cause and repeat rate remain unmeasured
([retained GNU identities and original-member hashes](../audit/depot-current-host-gnu-owner-2026-10-18.evidence.json.gz)).
The matched Depot Windows GNU run executes the same 1494 binary/case identities:
1493 pass and the unreadable-session-path/logging case fails,
with one separate summary-only skip.
The watch case passes in this sample;
that status difference does not establish a provider cause or failure rate.
Its actual probe records Windows Server 2025 Datacenter,
release `10.0.26100`,
four available/logical CPUs,
16827592704 bytes of memory and CPU model `AMD EPYC 9R45`
([matched GNU and design terminal receipt](../audit/depot-gnu-design-terminal-2026-10-18.evidence.json)).

### Execution infrastructure

Native orchestration needs explicit event and check ownership,
with content-bound artifact names and supported reusable-workflow references.
Native retries reuse filesystem/environment state,
and native parallel branches share mutable directories;
they do not create isolated repetitions.
The measured single-job graph does not qualify every native parallel-step merge rule.
Tests reporting should retain original JUnit and validated identity transformations,
use attempt-specific report keys and split only admitted runnable identities.
The 34-case Nextest adapter qualifies its demonstrated single-binary shape.
Full-framework reporting,
dashboard flakiness and GitHub/native analytics parity retain narrower evidence limits
([orchestration contracts][depot-source-3],
[physical report adapter](depot-tests-identity-adapter-research.md)).

Native Depot,
Depot-managed GitHub Actions and standard GitHub now each have a passing actual frozen `mvm` guest lifecycle.
The standard-GitHub replay preserves UID/GID and adds the actual device group only to the disposable child.
All three passing consumers verify guest stdout,
exit-two stderr,
a 51-byte shared-directory CLI roundtrip and private-domain/daemon cleanup
([all eight original archives and outcomes](../audit/depot-mvm-kvm-2026-10-18.evidence.json)).
That roundtrip did not separately assert guest payload visibility or a guest write.
An independent source review identified this test gap;
the follow-up now proves that boundary on all three pools.
Each guest verifies its own boot identity and exact 51-byte input,
creates an initially absent distinct output,
transforms it and returns the exact 57-byte result to the host.
All three retain private-domain and daemon cleanup
([new original archives and guest proofs](../audit/depot-mvm-guest-files-2026-10-18.evidence.json)).
The first follow-up GitHub dispatch remained queued because of a corrupted generated runner expression;
it is retained as a harness failure,
separately from the three actual consumers.
The earlier failed probes remain historical observations;
the named-ACL mutation's writer is not identified.

The service/container profile passed four native and six GitHub host/job-container consumers.
It verifies actual locked Node 26.12.0,
its executable identity and loaded `libatomic`,
separately from the image's Node 24 runtime
([service primary receipt](../audit/depot-service-containers-2026-10-18.evidence.json)).
That image's Git 2.39.5 qualifies its exercised checkout consumer,
not the Git-policy suite's exact Git 2.56 requirement.

The per-test identity adapter passed timed physical splitting:
both shard commands found historical timings for all 34 candidates,
executed disjoint 17-case partitions and passed the exact-union consumer.
The original JUnit bytes,
validated classname adaptation,
stored result rows and committed guard-removal failure are retained
([Tests adapter receipt](../audit/depot-tests-identity-adapter-2026-10-18.evidence.json)).
Physical sandbox identities are not a claim about distinct physical hosts or a speed ratio.

The multi-architecture BuildKit campaign produced actual AMD64 and ARM64 image consumers,
including locked Node 26 execution,
and verified registry read-only access against a rejected write.
SPDX and SLSA subjects matched their runtime-manifest digests,
which are not necessarily the image-index digest;
the separate OCI archive's empty subject retained its own disposition.
The four-target Bake run exercised overlapping target work,
secret delivery and keyless SSH listing,
but did not perform an SSH signature or prove project-level autoscaling
([BuildKit primary receipt][depot-source-11]).
Truepeak's image build and cache checks reported zero tests;
the separately executed 89-case owner supplies its test evidence.

Native jobs also completed actual BuildKit and registry consumers using only their injected job token.
The local-export correction `pxx75mxnw2` imports and executes the frozen queue source from its exported image.
The separate registry run `38z4wlq9d3` rejects anonymous and invented-token access,
then exchanges the injected token,
reads and rehashes the OCI graph and executes the extracted frozen-source consumer.
The original `dzsf27t0xx` run reached the builder but failed because the fixture mixed incompatible exporters.
The temporary image and solely owned project were deleted;
the exact manifest changed from HTTP 200 to 404 and the project query returned `not_found`.
Those are logical cleanup observations,
not physical-erasure or invoice claims
([native authentication receipt and original ZIPs](../audit/depot-native-buildkit-job-auth-2026-10-18.evidence.json)).

The custom-runtime SDK campaign created and stopped three isolated VMs.
Its final consumer used the absolute required Git executable,
read a bounded manifest and completed a nonce write/read/delete cycle.
The earlier Node-path failure and operation with an unresolved read timeout remain separate failed probes.
The command-log query's empty response does not prove complete absence of execution logs.
Fresh terminal VM reads prove the observed stop state,
without establishing a billing-release instant
([SDK and platform evidence][depot-source-12]).

## Reliability and source failures

The original same-name artifact rerun selected a stale archive alongside current producer outputs;
the hash guard rejected it.
Numeric-ID sorting is source-proven,
but the failing Action's raw pre-filter response was not captured,
so the specific incident cannot distinguish sorting from a visibility race.
Content-bound artifact names passed both a fresh consumer and complete rerun
([source trace and controls](../troubleshooting/depot-rerun-artifact-id-order.md),
[original evidence](../audit/depot-native-orchestration-2026-10-18.evidence.json)).

An isolated job-level allowed failure produced a failing native aggregate and GitHub App suite,
while the paired GitHub workflow succeeded.
Default downstream eligibility passed separately;
the isolated step-level continuation retained failure outcome,
success conclusion and successful native job/workflow status
([policy/history receipt](../audit/depot-job-policy-history-2026-10-18.evidence.json)).
Continuation semantics need the exercised granularity.

The original job-container image selected checkout's REST fallback;
the independent image control found no Git executable on its default AMD64 `PATH`.
These facts do not identify the original REST download stall's cause.
The Git-capable successor then exposed the locked Node 26 loader's missing `libatomic.so.1`;
the final derived image passed all ten actual service consumers
([prerequisite source and run controls](../audit/depot-container-prerequisites-2026-10-18.evidence.json),
[retained executable image delta](../troubleshooting/github-actions-container-node-runtime-prerequisites.md)).

The built-in Semgrep 1.34.1 scan missed the controlled missing-`USER` case,
and its `CMD $...VARS` pattern did not match the exercised JSON-form `CMD`.
A separate Semgrep 1.181.0 control using the exact fixed rule pack detected both,
while the Hadolint root/non-root controls behaved as expected.
This qualifies the replacement scan execution;
it does not establish a repaired Depot built-in scanner
([scanner and rule-source evidence][depot-source-12]).

Correction (2026-10-18):
the earlier text said "The same retained rule bytes executed by
Semgrep 1.181.0 detected both."
The cited evidence records old-engine fetched and retained
rule bytes and a current-engine "exact fixed rule pack";
it does not bind the 1.181.0 run
to the identical retained bytes.
The original sentence is preserved in this note;
the
replacement claims the fixed rule pack,
not byte identity.

Every measured provider allowed the finite seventy-second writer past a sixty-second step deadline.
Only native Depot's observation proves completion after the executor advanced to its next step;
both GitHub-orchestrated observers started after their writers finished.
The source-backed note separates that observation from unknown descendant signal routing,
VM release and billing
([process receipt](../audit/depot-process-timeouts-2026-10-18.evidence.json),
[deciding-source note](../troubleshooting/depot-step-timeout-process-boundary.md)).

Product assertions,
harness errors,
missing producers and prerequisite failures retain different dispositions.
The recorded browser OPFS assertion,
native `none` assertion,
property/conformance counterexamples and Android fixture failures are not converted to passing gates
by successful setup or successful unrelated siblings
([coverage record][depot-source-5],
[Android receipt][depot-source-9]).

The Linux static campaign ran 465 scheduled command invocations,
including 42 shared prerequisite invocations per shard across 297 owning target tasks
(109 owning type-check tasks,
106 owning Oxlint tasks and 82 build entry points),
with 418 invocations passing and 47 failing;
these are task outcomes rather than assertion counts.

Correction (2026-10-18):
the earlier text said "The Linux static campaign ran 465 owning
commands".
The 465 total is the scheduled command-invocation count:
297 owning target
tasks (109 type-check,
106 Oxlint,
82 build entry points) plus 42 shared prerequisite
invocations per shard across four shards (168).
The original claim is preserved in this
note;
the 465 total and the 418/47 split are unchanged.
The default Node owner campaign reports 126 passing commands and one failing unused-export owner.
The bounded property profile reports 64 passing cases and five retained counterexamples.
The separate mutation profile records eleven killed mutants,
six surviving mutants and two compile-error outcomes.
The published semantic map now covers all 223 directory owners and 1818 exact command receipts,
with every original command log rehashed;
121 failed command outcomes remain failures.
Its source-export,
assertion-import and generator projections distinguish static candidates from executed bodies
([semantic publication](../audit/depot-semantic-package-publication-2026-10-18.evidence.json)).

The source audit found eleven authored unit files across six owners that had no owning test execution
in that original command projection.
Their local consumers now produce 24 Figma passes,
including the three actual private exports,
and 151 other source-unit passes.
Four import-attributes integration cases fail because they request `src/fixtures`
while the frozen source contains `src/fixture`.
With logging held constant,
an untracked alias alone makes those same four cases pass.
The named no-with case still uses an attributed import and only checks nonempty output;
its negative behavior remains unqualified.
These are source-import assertions,
not compiled product-package acceptance
([source consumers and exact qualifications][depot-source-5],
[fixture controls](../audit/depot-local-node-counterfactual-2026-10-18.evidence.json)).
The separately rebuilt Node harness passes its existing nine-case built-export self-test.
The initial local wrapper discarded successful-child stderr;
its empty fields remain unobserved placeholders.
The corrected capture first passes a real stdout/stderr positive control,
then retains both streams from all twelve files.
On the derived fixture topology,
all 179 product source-case labels and the nine separate harness-control labels
join actual named debug `PASS` records.
The 92794 captured stderr bytes contain no logger-internal diagnostic;
the harness control's eight deliberate failure records remain separate from product outcomes
([corrected source capture](../audit/depot-local-node-capture-correction-2026-10-18.evidence.json)).
Repeated cases do not add new distinct coverage,
and the original missing-fixture failures remain recorded.

The first repeated-graph dispatch `39135886118` failed tool setup in all four jobs,
before any of its planned 222 owner commands ran.
It omitted the configured `cargo-binstall` prerequisite.
The corrected driver also closes staged-source mutation and resistant descendant cleanup gaps found by
independent preflight.
Its twelve local controls pass on exact source `234ccc92dabb81e998e0a0bb5e838713924dd83f`;
those stub controls earn no application execution credit
([setup failure](../audit/depot-expanded-graph-setup-failure-2026-10-18.evidence.json),
[independent source and cancellation controls](../audit/depot-expanded-graph-preflight-2026-10-18.evidence.json)).

The first native graph repetitions complete their shared-build and Rust owners,
but browser preparation reaches the documented `newuidmap` primary-group rejection.
The isolated correction routes native Podman probes,
builds and runs through the previously verified noninteractive rootful boundary.
It preserves the absolute executable,
application inputs,
command plan and container resource limits.
Both real corrected native repetitions pass preparation,
Chromium and Firefox owner tasks;
WebKit still reaches the source OPFS verification assertion,
and root E2E discovers the unresolved paused-owner dependency before any test executes.
All 222 command logs are retained per corrected sample,
without command-budget termination or unstarted work.
Passing setup does not convert these source gates into a passing pipeline
([original and corrected native consumers][depot-source-13]).

The independent review of all eight original graph samples verifies 1720 retained command-log hashes.
It separates strict lint warnings,
property counterexamples,
WireGuard diagnostic expectations,
logger Promise/JSON assertions and intermittent watch waits.
The watch assertion fails in three samples across multiple provider contexts;
the samples do not establish a provider-specific failure rate or cause.
WireGuard's HTTP 500,
`IPINFO_TOKEN` and `ENOTFOUND` lines belong passing injected negative fixtures,
so those lines do not establish provider network failures.
Reached WebKit reports retain 26 expected cases,
one skip and one unexpected OPFS assertion;
root E2E discovery reports contain zero executed cases
([source and exact reporter classifications][depot-source-14]).

Two public GitHub WebKit logs contain `npm notice` text interleaved with reporter JSON.
The research collector merges command stdout and stderr into one file.
The reviewer retains original hashes and explicitly derived views that remove only those identified notice lines.
Those views support assertion classification;
they do not qualify a strict machine consumer of the original mixed streams.
The observation does not establish invalid JSON on Playwright's own stdout
([collector source and derived-view boundary][depot-source-14]).

The native-CI documentation review covers parallel shared-filesystem semantics,
retry state,
identity,
environments,
secret selection and automatic log-based AI summaries.
Not issuing an explicit paid model request does not mean the vendor performs no AI processing of job logs.
These documented behaviors remain separate from executed repository consumers
([substantive native capability review][depot-source-3]).

Native live/paused-step SSH,
per-job logs and resource metrics are separately documented debugging capabilities.
Their operational admission must retain exact attempt identity and private connection details.
Owned native run `sqrz4sml30` executes matching source/boot/challenge responses through direct OpenSSH
and the native CLI PTY path.
Their local exits are 255 and zero,
so the direct response alone is not a successful-client qualification.
Neither remote context inherits the synthetic workflow environment;
direct SSH runs as UID 0 and native CLI RPC as UID 1001.
The paused-action target and final workflow readback remain unqualified.
Dependency installation is the last recorded phase before watchdog cancellation;
the last APT line does not identify the blocking instruction or cause.
Its exact source has a real completed/cancelled GitHub check,
`native-ssh-acceptance / ssh-proof`,
whose details identify the owned native job.
Fresh native job metadata joins that job to the run and attempt.
The app is Depot Code Access (`219785`);
that identity does not mean the optional Depot Code product was enabled.
The known GitHub graph source provides nonempty check controls,
while both queried sources have zero legacy commit statuses.
This qualifies the observed native check publication,
not branch-protection configuration or a stable production check-name mapping
([SSH and forge-consumer receipt](../audit/depot-native-ssh-runtime-2026-10-18.evidence.json)).
Developer community support,
beta terms and the incomplete incident-history denominator provide no repository availability guarantee.
The proposal retains exact-attempt diagnostics and GitHub execution as a fallback;
both products still depend on their configured forge and identity paths
([operating terms and status boundaries][depot-source-12]).

## Provider allocation, cost, and critical paths

Developer includes 500 build minutes,
2000 native CI base minutes,
2000 GitHub runner base minutes and 25 GB combined cache/registry storage.
Startup includes 5000,
20000,
20000 and 250 GB respectively.
Both include one million passing Tests results;
extra passing results cost USD 0.000001 each.
Current primary wording and unit limits are retained in the
[allowance reconciliation][depot-source-15].
Mac allowance normalization and generic SDK eligibility for included CI credits remain unresolved;
the model does not grant those forecast credits.
The [current GHA troubleshooting guide](https://depot.dev/docs/github-actions/troubleshooting)
documents a separate GitHub Actions minute limit,
while container builds have their own usage control.
Those published controls are not a demonstrated combined USD 200 limit across native CI,
storage,
Tests,
SDK and other products.
Configured account limits and their live effects were not inspected.
The [container-build overview](https://depot.dev/docs/container-builds/overview)
also describes cache-size policy as soft because required parent layers can be retained.
A 25 GB project cache policy is therefore not a strict storage spending cap.

The [combined pricing model][depot-source-2]
prices independent named product entitlements,
combined storage and passing results.
Its scenario inputs are hypothetical monthly quantities,
not observed future event rates.
The [cost/critical-path work record](depot-expanded-ci-cost-model.md)
owns allocation-row normalization and the observed dependency-path calculation.
Standard GitHub public-repository compute remains a zero-compute-cost option;
artifact/storage rules are separate
([GitHub billing](https://docs.github.com/en/billing/concepts/product-billing/github-actions)).

The frozen owned-allocation baseline contains 223 distinct records.
Independent coordinator checks rehash all 411 retained source bodies and exactly recompute all 195 priced intervals:
native CI USD 4.8546,
Depot GitHub runners USD 1.3372 and SDK intervals USD 0.0081959,
with public GitHub compute zero.
Their known partial nominal subtotal is USD 6.1999959 before allowances.
BuildKit's USD 0.538584 request-window proxy remains separate from unknown allocated-builder compute.
Admission reservations are not added as observed charges
([normalized baseline](depot-owned-allocation-ledger.md),
[independent exact arithmetic](../audit/depot-owned-allocation-independent-arithmetic-2026-10-18.evidence.json)).

The independently checked follow-on snapshot adds 30 distinct terminal allocation rows,
with no baseline duplicates and 35 rehashed source bodies.
Its nominal provider-window proxy is USD 0.6041:
Depot GitHub USD 0.1797 and native CI USD 0.4244,
with public GitHub compute zero.
The desktop row uses its exact 263-second failed attempt,
not the superseded 279-second run envelope.
The separate BuildKit request-window proxy is USD 0.015897333333;
actual allocated-builder billing remains unknown.
Later OS,
owner and debug controls are excluded from this frozen snapshot until separately normalized
([follow-on ledger](depot-owned-allocation-follow-on.md),
[independent frozen source and arithmetic](../audit/depot-owned-allocation-follow-on-independent-2026-10-18.evidence.json)).

The separately verified successor has 37 allocation rows and 41 retained source bodies,
including the public recovery and managed-host windows.
Its nominal known-provider proxy is USD 1.2625:
Depot GitHub USD 0.8381 and native CI USD 0.4244.
The managed Mac's 359 seconds and Windows' 674 seconds add USD 0.6584.
Public compute remains zero,
the Root graph subtotal remains USD 0.487,
and the separate BuildKit request-window proxy remains USD 0.015897333333.
These are interval-derived proxies before allowances,
with invoice and allocated-builder compute unmeasured.
([37-row successor verification](../audit/depot-owned-allocation-follow-on-hosts-verification-2026-10-18.evidence.json)).

The final follow-on snapshot preserves those historical inputs and has 49 distinct terminal rows:
23 public GitHub jobs,
13 Depot GitHub jobs,
10 native jobs and the same three BuildKit requests.
All 53 retained source bodies and 49 exact timestamp-pointer joins verify.
Its known provider-window list proxy is USD 1.685366666667:
Depot GitHub USD 1.233566666667 and native USD 0.4518,
with public nominal compute zero.
The final append adds USD 0.422866666667:
material Depot jobs USD 0.2704,
matched GNU USD 0.125066666667 and both SSH attempts USD 0.0274.
BuildKit's USD 0.015897333333 request-window proxy remains separate.
The frozen baseline,
historical follow-on snapshots and full-window reservations are not added again
([49-row terminal ledger](../audit/depot-owned-allocation-final-2026-10-18.evidence.json),
[source/hash/BigInt verification](../audit/depot-owned-allocation-final-source-verification-2026-10-18.evidence.json)).

The current-source four-CPU Linux comparison has two completed repetitions on each GitHub runner pool.
All 42 shared Node build commands pass in every sample.
GitHub takes 206.729 and 213.691 seconds;
Depot's GitHub runners take 73.986 and 72.591 seconds.
The means are 210.210 and 73.2885 seconds,
with observed spreads of 6.962 and 1.395 seconds.
The ratio of means is 2.87.
The two Rust owner commands average 720.5195 seconds on GitHub versus 325.8625 seconds on Depot,
a 2.21 ratio,
with observed spreads of 10.631 and 28.513 seconds.
Both original Rust command logs contain the same 89 Truepeak and 237 forbidden-strings passing identities
in every sample;
the Truepeak summary also records two skips.
These compare complete owner programs,
including Rust compilation and test execution,
under observed four-CPU,
approximately 16 GB Linux x64 allocations
([original samples, sources and exact arithmetic][depot-source-16]).

Neither ratio describes complete CI.
Each public GitHub repetition leaves 28 commands unstarted and terminates six commands at their budgets,
while both Depot GitHub repetitions retain all 222 scheduled command logs.
Property-test seeds also differ across repetitions.
The graph is one job with parallel dependency branches,
not a distributed artifact pipeline or every repository owner.
It restores no explicit package/compiler cache and uses a fresh Cargo target;
provider image,
tool and registry caches are not assumed empty.
The ranges are observed spreads from two repetitions,
not statistical confidence intervals
([source-bound limits][depot-source-16]).

Corrected native Depot repeats the same completed shared-build commands in 62.589 and 62.011 seconds,
with a 0.578-second observed spread.
Its 62.300-second mean gives a 3.37 ratio against GitHub's shared-build mean.
The native Rust commands take 322.278 and 331.160 seconds,
with an 8.882-second observed spread;
their 326.719-second mean gives a 2.21 ratio against GitHub.
Native uses the explicit rootful Podman prerequisite adapter and retains separate source/test failures,
so these also remain completed-phase comparisons
([source and original artifact joins][depot-source-13]).

Independent native parity checks match the same 89 Truepeak and 237 forbidden-strings passing identities
against the GitHub reference,
including changed-case and changed-status negative controls
([original-log parity](../audit/depot-expanded-graph-native-parity-review-2026-10-18.evidence.json)).
The observed longest dependency path runs from shared builds into a Node consumer branch in all six profiles;
the independent Rust branch is shorter.
The model therefore does not add overlapping Rust and Node time reductions as human waiting savings
([source-bound path arithmetic](../audit/depot-expanded-graph-critical-path-2026-10-18.evidence.json)).

Cache conclusions name the workload and mechanism.
Both fresh pnpm profiles use pinned `actions/cache/restore` and `actions/cache/save`
at `27d5ce7f107fe9357f9df03efb73ab90386fccae` for the isolated pnpm content store.
The `pnpm store github` jobs use GitHub Actions cache;
the `pnpm store depot` jobs use Depot's Actions cache integration.
The fresh GitHub profile's two cache-disabled restore-plus-install observations were nine to ten seconds;
the two warm observations were eleven to twelve seconds.
The corresponding Depot profile was three seconds without caching and four to five seconds warm.
These are the isolated pnpm-store profiles,
not a result for every job inheriting a cache step
([exact jobs, phases, original ZIP/member receipts](../audit/depot-pnpm-fresh-jobs-2026-10-18.evidence.json)).
Depot's checkout also changed between those phases,
so its whole-job decrease does not isolate pnpm caching.

The Kotlin linter plugin's repeated Gradle `test --rerun jar` comparison exercises six fresh JUnit cases per job.
Depot's generic HTTP remote task cache restores compilation outputs;
the tests still execute.
The two warm commands took 17.550 and 15.836 seconds,
versus 21.435 and 20.262 seconds with both remote and local task-output caching disabled.
The within-condition spreads were 1.714 and 1.172 seconds.
The observed mean reduction is about 19.9 percent for that command boundary,
which includes JVM startup,
compilation or restoration,
tests and JAR production.
Whole warm jobs took 162 and 177 seconds versus 146 and 153 seconds without task caching;
variable pnpm import/link time prevents a whole-job benefit claim
([unchanged-input controls and exact receipts](depot-reuse-strategy-research.md#repeated-gradle-task-output-comparison)).

The forbidden-strings release CLI's compiled-target archive also has a real fresh consumer:
all 237 owned Nextest cases passed in producer and reader.
The reader's CLI build took 0.070 seconds versus the seed's 52.228 seconds,
but archive extraction alone adds 0.334 seconds,
and the reader's full Nextest command took 45.200 seconds versus 39.233 seconds in the seed.
Those commands include 22.506 and 21.449 seconds of actual test execution respectively.
This single reader proves compatible target reuse;
it does not establish a job or provider speed multiplier
([target-archive controls][depot-source-8]).

Prepared dependencies,
Gradle remote task outputs,
sccache,
compiled artifacts,
BuildKit layers and Depot Disk have separate admission,
invalidation and consumer evidence in the
[reuse research][depot-source-8].
The finished allocation must choose these mechanisms per owner and retained consumer boundary.
It cannot treat a cache hit,
restored bytes or a shorter compile phase as a measured whole-CI speedup.
The complete integration review also separates applicable Gradle/Rust mechanisms from unused Bazel,
Go module,
Maven,
Moon,
Nx,
Pants,
Turbo and Xcode owners.

Correction (2026-10-18):
the earlier list said "Go" among unused owners while line 96 says
there are no Go package owners.
The repository has a Go-built `toml-test` conformance tool,
not a Go package owner;
"Go module" names the mechanism.
The original list is preserved in
this note.
Depot's Actions cache guide explicitly documents no branch isolation;
the actual six-job probe confirms sibling-branch read reuse on Depot and a sibling miss on GitHub,
after both same-branch positive controls.
This is qualified to the observed defaults and nondefault sibling seed,
not every access-mode or fork/PR write scenario
([cache applicability and trust review][depot-source-17]).

## Adoption conditions and unresolved evidence

### Proposed workflow allocation

Native Depot capability runs in one bounded weekly matrix covering the admitted native capability
categories:
static checks,
Node and Rust test consumers,
browser and E2E consumers,
packaged artifacts,
OS,
architecture and application consumers,
container producers,
Depot Tests,
cache and Disk,
and SDK VM profiles.
Ordinary Node static,
unit and process validation defaults to GitHub-hosted runners and is not moved by this matrix.
Build shared producers once and validate their real consumers.
Keep GitHub permission and approval boundaries on GitHub scheduling,
with Depot runners available where their tested OS,
resource shape and plan eligibility fit.
The advisory-only model and the single bounded weekly matrix are accepted;
the concrete matrix membership,
per-profile evidence and weekly bounds remain a pilot work item;
the retained source failures require per-product dispositions before their results are reported,
and no result is a required gate
([owner results][depot-source-5],
[Rust execution contexts][depot-source-6]).

The rollout should expose independently verifiable profiles,
using the frozen owner map as its explicit starting scope.
The commands named here are examples of existing owners,
not a claim that selecting those examples covers the entire package inventory.
The final selector must retain every admitted task from the
[coverage manifest](depot-complete-coverage-manifest.json),
including configuration,
generated-file and producer influences.
Its static import inventory is not yet a validated production change selector.
The existing [Mise selector investigation](../troubleshooting/mise-affected-workspace-cycles.md)
qualifies the workspace-cycle rejection and pnpm's working Node reverse closure.
The actual locked Mise 2026.10.9 binary also rejects the same frozen workspace cycle,
after acyclic changed-producer/development-consumer and successful empty-selection controls.
The offline probe preserves all 419 authored metadata files;
it does not execute application owners or qualify the full locked CI bootstrap
([locked-version source and runtime receipt](../audit/depot-mise-locked-affected-2026-10-18.evidence.json)).
That Node metadata result still lacks Cargo/Gradle,
global configuration,
deleted-package membership and artifact-consumer ownership.

#### Static checks and shared producers

Select each admitted package's declared lint/type owners,
including `//package/logger/core:lint:types`,
then retain task-level diagnostics and the source-bound selected task list.
Build `//package/test-harness/core:build` and the other actual shared producers once per admitted graph.
Preserve Mise dependency ownership and repository packing/generator owners;
do not introduce a second package inventory as workflow YAML.
The existing task utility's timestamp checks are incremental-build aids,
not content authentication for restored artifacts
(`package/monorepo-task/core-exec/src/depends.ts`).
Warnings denied by the frozen configuration remain failures,
with their source dispositions visible.

#### Node unit, process and nondefault tests

Run each declared owner,
including `//package/logger/core:test:unit` and `//package/upstream-fork/p-map:test:unit`,
against its real built prerequisites.
Retain runtime case reports separately from command exit counts.
Process owners keep their subprocess and cleanup assertions;
property,
conformance and mutation owners keep their bounded inputs,
seeds and native classifications.
Keep those nondefault profiles independently schedulable,
including `//package/upstream-fork/p-map:test:mutation`.
The retained logger,
matcher and p-map failures need explicit dispositions before required-check adoption.

#### Rust, Kotlin and native prerequisites

Use package Nextest owners,
including `//package/forbidden-strings/cli:test`
and `//package/music-player/truepeak-core:test`,
with default build/test concurrency and the owner-declared feature/profile flags.
Bind case identity to its binary and package context.
Keep privileged WG,
ASan fuzzing,
benchmarks and installed-host controls as separate profiles with their actual prerequisites.
The Kotlin profile invokes `//package/linter/kotlin:test`
and retains its six-case JVM report plus the separate emitted-JAR consumer.
The [finite material-owner campaign](depot-material-owner-completion-proposal.md)
now has a [parent admission](../audit/depot-material-owner-parent-admission-2026-10-18.evidence.json)
for IDE,
terminal,
file-manager variants and the actual design Node/ImageMagick owners.
Its seven primary profiles and two conditional corrections share a USD 5 nominal guard.
All seven primary workflows are now terminal,
and the compiled Nextest inventories match their actual executed identities.
IDE full passes 783 cases with one ignored;
its separately executed library subset passes 316 cases with the same ignored case.
Terminal passes 16,
GTK passes 13,
sticky passes eight with one ignored,
and Qt passes its authored QString/default-state consumer.
The 1137 execution observations contain 821 unique binary/case identities:
the IDE subset overlaps full and is not additive unique coverage.
IDE's real runtime producer stages 2500 files,
and final source checks preserve the authored assertions.
These are library/process/model/runtime consumers,
without a new GUI or compositor launch.
The initial design profile enumerates its 13 actual Node programs but executes zero:
the copied locked Node binary exits 127 because the tool image lacks `libatomic.so.1`.
Its separately admitted public-runner correction adds only the runtime library,
and all 13 actual owning programs now pass with ImageMagick.
The source consumer verifies 997 selected asset/HTML inputs totaling 269996175 materialized bytes;
these are verified input bytes,
not published media.
The actual bootstrap preserves owning commands and assertions
([primary case and producer receipt](../audit/depot-material-owner-primary-2026-10-18.evidence.json),
[design correction admission](../audit/depot-design-libatomic-correction-admission-2026-10-18.evidence.json),
[actual GNU/design outcomes](../audit/depot-gnu-design-terminal-2026-10-18.evidence.json)).
Static test attributes,
successful compilation and an import-only fixture do not substitute for those runtime boundaries.

#### Browser and active E2E tests

Expose the existing `test:browser:chromium`,
`test:browser:firefox` and `test:browser:webkit` owners independently,
with their real Podman/Playwright prerequisites and bounded container resources.
Use the existing argument-bearing `test:e2e` owner with the explicit active DOM spec list
qualified by the retained campaign.
Keep unfiltered root discovery as a distinct unresolved owner;
its paused-package import failure does not erase the active E2E result.
Retain original stdout and stderr separately in the permanent collector,
so consumers receive valid original JSON reports.
The retained WebKit OPFS assertion failure still requires disposition.

#### Packed artifacts and host consumers

Produce package archives with the existing packing and manifest-transformation owners,
then install/import/launch those exact artifacts in fresh consumers.
Retain hidden SDK resources,
native executable formats,
source/tool hashes and the consumer's actual ABI.
Type-only packages need their type consumer;
runtime import expectations must follow actual exports.
Reuse the exercised semantic-plugin owners
`//package/oxlint-plugin/prefer-readonly-parameter-type:test:semantic-bridge-host`
and `//package/oxlint-plugin/prefer-readonly-parameter-type:test:external-consumer-host`.
The source-bound artifact checks and actual host activation remain separate acceptance results.

#### OS, architecture and application consumers

Keep Linux x64/ARM64,
Windows x64/ARM64/GNU and macOS contexts explicit rather than merging their case totals.
Use public GitHub and eligible Depot-managed GitHub runners for the demonstrated OS boundaries;
native Linux CI remains a separate scheduler.
Retain machine shape and OS version in comparisons.
VM,
desktop and Android profiles keep their real runtime,
display/device and cleanup consumers alongside build outputs.
Their completed research does not authorize recurring device sessions or production releases.
The current task does not relaunch the completed desktop or emulator campaigns.

#### Control plane, artifacts and diagnostics

Use content-bound cross-job artifacts and exact attempt IDs for producer/consumer handoff,
with negative reader controls and actual rerun behavior retained.
Tests uploads keep original JUnit identities,
attempt IDs and shard boundaries;
the existing passing physical-shard probe does not establish a universal splitting speedup.
Keep retries,
cancellation,
timeout and remote-debug controls separately observable.
Every paid profile needs finite resource/time admission,
logical cleanup and a cost record;
an API terminal state is not a measured billing or deallocation boundary.
Every profile result is advisory;
each native capability category runs in the single bounded weekly matrix;
per-profile admission evidence and per-product dispositions remain pilot work items.

### Measured reuse and container producers

Allocate reuse by measured owner:
Gradle remote task outputs for the demonstrated Kotlin linter,
direct sccache for qualified Rust compiler consumers,
and source/tool/ABI-bound dependency or target archives with passing readers.
The Kotlin owner join independently matches all six authored JVM methods to four original passing JUnit reports
and the exact frozen package subtree;
it closes a missing evidence projection without another allocation
([source/task/case join](../audit/depot-kotlin-gradle-owner-join-2026-10-18.evidence.json)).
The measured pnpm content-store profiles do not justify universal store caching.
Native tool snapshots require current checkout and executable/PATH admission.
Organization-global Disk mounts need a single writer,
partitioning or immutable content;
concurrent mounts do not provide atomic mutation.
An expected digest supplied by the same mutable untrusted cache does not independently authenticate its contents
([reuse controls][depot-source-8],
[cache capabilities and trust][depot-source-17]).

Use BuildKit/Bake for admitted container contexts and immutable multiarchitecture CI/service producers,
preserving the exercised SBOM/provenance-to-runtime-digest checks.
A matching attestation subject does not establish signing trust.
Autoscaling has no demonstrated total-builder bound in this evaluation;
keep it outside automatic paid admission until that bound is supplied.
Registry pull-through,
Fast CDN and server-side promotion are reviewed options with no measured speedup here.
Tagged retention is opt-in,
saved project images default to unlimited retention,
and logical deletion can precede blob reclamation.
Standard transfer is documented free;
Fast transfer costs USD 0.10/GB
([executed BuildKit graph][depot-source-11],
[registry lifecycle sources][depot-source-12]).
Credentialed Containerfiles also need their own context and cleanup controls:
the synthetic Git rewrite probe retained its invented-token key,
without supplying a real credential or proving exported-layer retention
([source-owned cleanup boundary](../troubleshooting/git-url-rewrite-cleanup-key.md)).

### Trust, budget and optional products

Trusted branches and untrusted PRs need explicit cache-write and secret authority.
Depot Actions cache,
project BuildKit cache and organization-global Disk have different scopes.
Linux egress filtering is root-modifiable and incompatible with Tailscale;
it is not a hostile-root exfiltration boundary or a Windows/Mac policy.
Private connectivity needs its documented credentials and eligibility.
Dependabot has a separate runner setting and lacks ordinary GitHub OIDC,
so its authentication cannot inherit the regular-job assumption.
No network policy,
tailnet credential or Dependabot secret was activated
([cache trust][depot-source-17],
[network and bot source review][depot-source-7]).

Paid scheduling needs finite workload reservations and separately priced storage,
Tests,
SDK and optional-product quantities.
The research excludes account-wide usage and invoice inspection,
so it cannot verify an aggregate USD 200 invoice guarantee.
Production admission needs current authorized headroom and expiry,
with public GitHub compute available when paid headroom is unknown.
Retention and deletion must name exported artifacts,
registry manifests/blobs,
snapshots,
Disk and logs separately
([allowances and limits][depot-source-15],
[actual logical registry cleanup](depot-native-buildkit-auth-research.md)).

Managed/customer-AWS,
GPU and custom GitHub runner images need their separate eligibility and administrative delegation.
Code is a public-beta Git transport/mirror with push-only native events;
it has no demonstrated independent GitHub recovery path here.
Depot Agents describes Claude Code,
Anthropic authentication and separately billed agent minutes without included usage.
The repository's Codex fixture does not qualify that product.
The Dagger engine integration is deprecated and adds a distinct engine charge.
These reviewed adjacent products remain unactivated,
as do absent-owner cache/language recipes and external observability sinks
([optional-product boundaries][depot-source-12],
[current Dagger guide](https://depot.dev/docs/github-actions/integrations/dagger)).

### Remaining research and interview boundary

The semantic owner projection,
remaining authored source consumers and final allocation/cost normalization are now sealed.
The coverage manifest still marks `coverageReviewComplete: false` and
`executionComplete: false`;
sealing these artifacts is not a complete-coverage or execution-complete claim.

Correction (2026-10-18):
the earlier text said these artifacts were "now sealed" without
recording the manifest's false flags.
The cited manifest retains
`coverageReviewComplete: false` and `executionComplete: false` at lines 420-421,
and the
cited research says the manifest explicitly marks semantic coverage review and execution
incomplete at lines 374-375.
The original sentence is preserved in this note;
"sealed"
refers to the artifact state,
not complete coverage.
The [execution handover][depot-source-1] retains the remaining publication checks and explicit limits.
The capability-allocation source review and concrete rollout profiles are written;
the primary IDE/file-manager/terminal case identities and real producer joins are sealed.
The two-attempt native debug campaign is complete with partial transport qualification:
both direct OpenSSH challenges execute with client exit 255,
both native CLI PTY challenges execute with exit zero,
but no paused-session target or final workflow readback qualifies.
The corrected attempt verifies the exact admitted tmate binary before its watchdog cancels the run;
this does not establish a tmate failure cause or lack of support
([final bounded SSH result](../audit/depot-native-ssh-correction-runtime-2026-10-18.evidence.json)).
All 120 captured documentation pages now have substantive applicability findings in the source ledger.
The ledger's top-level status remains `Research reopened; no readiness claim`;
the 120-page reconciliation is a bounded source-review result,
not a readiness claim.

Correction (2026-10-18):
the earlier sentence did not record the ledger's top-level status.
The ledger at line 5 says "Research reopened;
no readiness claim";
its nested
`publicDocumentationReconciliation` status at line 16699 supports the 120-page
reconciliation.
The original sentence is preserved in this note;
the 120-page count is
unchanged.
The repeated GitHub and corrected native graph phases,
case parity and independent failure classifications are sealed;
their failed full-graph outcomes remain qualified rather than converted into an overall speed claim.
The desktop,
packed-install and native automatic BuildKit/registry consumer queues are complete for the named results,
with the qualifications in Gate and artifact-consumer results.
The coverage manifest still marks `coverageReviewComplete: false` and
`executionComplete: false`;
these queue completions are not an execution-complete claim.

Correction (2026-10-18):
the earlier text said the queues "are complete" without scoping
the completion to the named results or recording the manifest flags.
The cited manifest
retains `coverageReviewComplete: false` and `executionComplete: false` at lines 420-421.
The original sentence is preserved in this note;
the named queue results are unchanged.
A fresh independent whole-report sufficiency review has now run and is recorded in
[the review record](depot-expanded-ci-evaluation-independent-review.md);
it found the report sufficient with corrections for a bounded pilot and not sufficient for a
production rollout.

Correction (2026-10-18):
the earlier text said the review had not run because the agent
service quota was exhausted.
The review has since run;
the record is cited above.
The original
sentence is preserved in this note.
Source/hash/pointer/arithmetic checks are concrete verification,
not a substitute for that review.
Every source branch,
every promised product feature and the retained failures remain outside any complete-coverage claim.

After the evidence closed,
the current interview settled the advisory-only model,
the Developer no-overage baseline,
the single bounded weekly native matrix and the per-product disposition rule.
The remaining open items are the concrete weekly matrix membership and bounds,
the measured baselines for the relative success targets,
the recorded independent review,
and the Q7 confirmation of the revised rollout plan.
The first posted round asks about failure enforcement,
workflow cadence and Linux orchestration order.
The user accepted baseline-before-advisory discipline:
existing source failures require exact reproducible baselines,
and setup,
discovery and artifact-integrity failures are never converted to passes.
Under the advisory-only model those failures are reported,
not blocking.
The current interview selected advisory-only CI and one bounded weekly native matrix covering every
admitted native capability category,
replacing per-push paid execution and the proposed nightly schedule.
The current interview replaced staged per-workflow native promotion with one bounded weekly native
matrix covering every admitted native capability category.
This rollout stays within a permanent hybrid with heavy GitHub-hosted runner use.
Promotion does not mean progressively draining eligible workloads from GitHub's own compute.
Production routing follows confirmation of shared understanding of the concrete rollout plan.
The user disabled overage on every product and set Developer as the operating baseline;
the current interview supersedes the standing overage authority of Q4.
One reserved automatic recovery attempt for classified infrastructure failures remains.
The original failure remains visible;
source failures,
discovery failures and artifact-integrity failures are reported as failures and never converted to
passes.
The scope correction stays in CI documents;
`AUT` remains unchanged.
The [concrete rollout plan](depot-expanded-ci-rollout.md) must be revised to consolidate the
Developer no-overage baseline,
the advisory-only model,
the single bounded weekly native matrix,
the sliding-window default-branch advisory cadence,
the trust boundaries and the cache write policy.
The plan remains a proposal until Q7 is answered and `doc/decision/depot-ci-adoption.md` is
accepted.
The recommendation must retain its source cohorts,
documented plan eligibility,
exact consumer limits and the USD 200 capacity ceiling with overage disabled.

[depot-source-1]: ../handover/depot-trial-grilling.md
[depot-source-2]: ../audit/depot-combined-product-pricing-2026-10-18.evidence.json
[depot-source-3]: depot-native-ci-capability-review.md
[depot-source-4]: depot-arm64-workload-research.md
[depot-source-5]: depot-complete-coverage-research.md
[depot-source-6]: depot-rust-coverage-research.md
[depot-source-7]: depot-platform-surface-ledger.json
[depot-source-8]: depot-reuse-strategy-research.md
[depot-source-9]: ../audit/depot-android-consumer-2026-10-18.evidence.json
[depot-source-10]: ../audit/depot-current-host-case-comparator-controls-2026-10-18.evidence.json
[depot-source-11]: ../audit/depot-buildkit-platform-campaign-2026-10-18.evidence.json
[depot-source-12]: depot-platform-surface-audit.md
[depot-source-13]: ../audit/depot-expanded-graph-native-correction-2026-10-18.evidence.json
[depot-source-14]: depot-expanded-graph-independent-review.md
[depot-source-15]: depot-included-allowance-reconciliation.md
[depot-source-16]: ../audit/depot-expanded-graph-github-repetition-2026-10-18.evidence.json
[depot-source-17]: depot-cache-capability-review.md
