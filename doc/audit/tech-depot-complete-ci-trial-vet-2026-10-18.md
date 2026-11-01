<!--
status: final native follow-up evaluation; production acceptance limits retained
lifecycle-phase: native execution complete; no broad vendor ranking
subject: Depot complete CI trial
scope: Evaluate frozen source cases and authorized native runner follow-up for intentional GitHub/Depot routing.
started: 2026-10-18
last-updated: 2026-10-18
governing-skill: .agents/skills/choosing-technology/SKILL.md
governing-skill-commit: 37117e36397e30233062828581a5e38217da9458
governing-skill-sha256: 552ac9955299b65a9f921a4e836b60a3fabc15d3477eeb8ec2bfb3f400647f9b
compatibility-fingerprint: 8024f142c314b7384a015376ce121c553e1ad83f32072b4f6b5eb3898a1f754a
active-audit-owner: /root
prior-compatible-report: none; expanded routing scope differs from the initial frozen cohort
unresolved-preferences: none required for the scoped trial verdict
criteria-and-weights: no weighted vendor score; capability, budget arithmetic and waiting evidence remain separate
hard-constraints:
  - Future all-provider monthly ceiling USD 200; prefer Developer when practical
  - No account-wide access or subscription changes
  - No application source fixes
  - Preserve four historical source cases and six original harnesses
  - Record source, common manifest and original artifact-consumer evidence
  - Value unfinished waiting once at CAD 8.975 per hour
base-components:
  - Inspectable open-source local technology
  - Managed service or SaaS
overlays:
  - CI high-trust execution
  - Multi-platform and browser claims
  - Native and prebuilt artifact boundaries
  - Selective provider routing
-->
# Depot complete CI trial and native runner evaluation

## Decision and acceptance limits

Use an intentional GitHub/Depot hybrid with Developer as the planning baseline,
plus selective overage for paid checks whose coverage or waiting-time benefit justifies the cost.
Keep routine standard GitHub-hosted work where its free public-repository compute is sufficient.
Depot can supply Windows and Apple Silicon macOS compute through GitHub Actions.
Developer includes Depot Windows runners;
Depot Mac runners require Startup.
Routine native work can remain on free GitHub-hosted runners within the chosen budget
([pricing][pricing],
[runner eligibility][runner-overview],
[public GitHub compute][github-billing],
[executed native consumers][consumers]).

For native Depot CI compute considered alone,
the financial ranking is Developer plus overage > Startup:
Startup's additional USD 180 monthly fee buys USD 108 more native CI allowance.
The executed model checks the pricing breakpoints and shows Developer remains cheaper at every usage level.
This calculation does not combine the separate GitHub runner and build entitlements.
The [expanded combined-product model](depot-combined-product-pricing-2026-10-18.evidence.json)
includes explicit inputs where Startup costs less;
do not generalize this single-product result to all CI or all Depot usage.
Developer's tradeoffs are lower allowances,
variable overage and no Depot Mac access.
Startup provides Mac access and larger separate product pools,
but its USD 200 base fee consumes the user's entire monthly provider ceiling before extra charges.
Usage across other pools or a measured need for paid Mac capacity can change that comparison
([routing model][model],
[plan terms][pricing],
[native runner plans][runner-overview]).

This is a scoped placement and budget verdict.
Production acceptance still has the original source failures and unexecuted surfaces described in the
[initial evaluation][initial].
The original registry records 770 primary gates:
654 passed,
115 failed and one blocked by a failed prerequisite,
plus 33 external gates with explicit dispositions.
The native follow-up adds execution observations without changing that frozen gate inventory
([original registry][registry],
[native consumer reconciliation][consumers]).

No subscription or production workflow routing was changed.
The authorized Startup trial was used for the Mac evaluation.
Account-wide invoices and caps remain outside the chosen access scope;
their absence limits billing certainty and does not prevent reporting this trial's result
([authorization][authorization],
[routing model][model]).

## Immutable workloads and scope

The original cohort remains `complete-20261017T111555806Z`.
Its four historical source cases and six original harnesses remain frozen.
The source revisions,
case hashes,
770 primary definitions and original execution results remain in the
[initial evaluation][initial] and [original registry][registry].

The supplemental native cohorts are
`native-runners-20261018T034256508Z`
and `native-recovery-20261018T035939491Z`.
Their published manifests bind the unchanged source cases,
new harness revisions and shared request clocks.
The recovery changes the Windows runner bootstrap while preserving the original consumer step values.
Native repetitions use the same harness revision
([first manifest][manifest-first],
[recovery manifest][manifest-second],
[first execution][first-wave],
[recovery execution][second-wave]).

All six workflow runs and 12 allocated job rows are terminal.
Ten job rows ran on actual `depot-*` runners;
two preserved Windows ARM rows ran on GitHub.
The selected labels were `depot-windows-2025-4`,
`depot-macos-15` and the unchanged `windows-11-arm`.
The current Depot Windows catalog advertises Intel runners;
the trial preserves actual native ARM coverage instead of inventing a Depot ARM label
([consumer reconciliation][consumers],
[runner catalog][runner-types]).

The report distinguishes the source case from the execution profile:
the frozen harness's `caseName` field is `native` or `windows`,
while `commonManifest.caseId` binds `native` or `desktop`.
The validator independently checks both fields,
source revision,
clean tracked status,
Git tree,
every retained input digest,
published common bytes and provider artifact digests
([first execution][first-wave],
[recovery execution][second-wave]).

## Gate and artifact-consumer results

### macOS

Both native Mac repetitions execute the same 1382 test identities:
1356 pass and 26 fail,
with one additional summary skip whose identity is not printed by nextest.
Those outcomes match the last GitHub native Mac retry.
The decode case that failed in the first GitHub attempt passes in both Depot attempts and the GitHub retry;
that is repeat evidence,
not an established root-cause fix
([first execution][first-wave],
[recovery execution][second-wave],
[original consumer reconciliation][original-consumers]).

Both desktop Mac repetitions execute the same 1433 identities:
1406 pass and 27 fail,
with one additional summary skip.
The individual identities and outcomes match the GitHub reference.
The two existing Clippy missing-return diagnostics remain.
Original release builds and CLI consumers pass.
The verifier's positive control removes an actual identity and confirms the comparison rejects the omission
([first execution][first-wave],
[recovery execution][second-wave],
[consumer reconciliation][consumers]).

### Windows

The native source case builds its release binary and passes its original CLI consumer on Depot.
Test compilation reproduces the frozen source's `E0433` diagnostics for unresolved `libc`;
it executes no nextest identities in that blocked test step
([first execution][first-wave],
[recovery execution][second-wave]).

After the checkout recovery,
both Depot desktop variants (`msvc-x64` and `gnu-x64-ucrt64`)
and the GitHub `msvc-arm64` row each pass all 1432 executed nextest identities,
with one additional summary skip.
Their original binary build,
linker reporting,
PE inspection,
Bash runtime and launch checks pass.
Source/common before-and-after pairs and all provider artifact digests verify
([recovery execution][second-wave],
[consumer reconciliation][consumers]).

The original PowerShell file-capture assertion remains failed in every desktop Windows row.
The independent merged-stream and uncaptured-console consumers both verify `music-player 0.1.3`.
These are separate outcomes:
the probes establish actual runtime output while the original assertion remains failed
([consumer reconciliation][consumers]).

## Reliability and source failures

The first Depot desktop Windows attempts stop before consumers because Git emits `Filename too long`.
The same fixture path occupies 260 characters in the longer Depot harness checkout,
247 on GitHub ARM and 243 with the shorter Depot native harness directory.
The recovery records an unset `core.longpaths` value,
enables and verifies the supported option,
then passes the original full checkout at the same 260-character path.
The failed attempts remain included in the cost estimates
([path measurements][paths],
[verified recovery][long-paths]).

Local `gh run watch` processes also encounter `net/http: TLS handshake timeout`.
Run metadata still shows active workflows after those monitors exit.
Read-only watching resumes against the same run IDs,
and all runs subsequently reach terminal states.
No workflow is redispatched for this monitor transport failure
([consumer reconciliation][consumers]).

All observed nextest failure identities in the follow-up match retained source-failure records.
The repeated Mac outcomes agree,
and the Windows test-compilation diagnostics remain tied to the historical source.
These observations do not establish a service-wide failure rate,
an OOM cause for the earlier logger worker,
or resolution of the initial evaluation's separate source incidents
([first execution][first-wave],
[recovery execution][second-wave],
[initial failure boundaries][initial]).

## Provider allocation, cost, and critical paths

The native follow-up's published-label proration totals USD 3.23 using GitHub job start/end timestamps,
or USD 3.09 using first-step-to-terminal timestamps.
Both include the failed initial Windows checkouts and the repetitions.
These are observable timestamp proxies for resource cost,
not independently retrieved Depot billing intervals or an invoice
([consumer reconciliation][consumers],
[runner prices][runner-types]).

The Mac catalog lists eight CPUs and 24 GB for `depot-macos-15`.
The four real probes report seven,
seven,
seven and nine CPUs,
all with 17 GiB of memory (`18253611008` bytes).
The label's published price is retained for pricing;
the observed guest hardware is retained for interpretation
([runner catalog][runner-types],
[hardware and variation][consumers]).

Native Mac job durations are 8 minutes 16 seconds and 7 minutes 1 second.
Desktop Mac durations are 9 minutes 26 seconds and 5 minutes 55 seconds.
All native artifact-cache lookups miss,
and failed jobs skip cache saving.
The desktop pair reports different CPU counts.
The observations therefore establish neither a warm-cache speedup nor a provider-only performance effect.
Two samples do not establish a p50/p95 distribution
([variation evidence][consumers],
[cache observations][paths]).

The user's waiting-time value is CAD 8.975 per hour,
counting overlapping unfinished work once.
The dated Bank of Canada observation is CAD 1.4154 per USD on 2026-10-16,
giving USD 6.340963685177335 per hour.
The USD 180 Startup fee premium alone corresponds to about 28.39 hours of valued waiting.
Paid acceleration needs to reduce the relevant unfinished-work interval to earn waiting-time credit;
an isolated faster step is insufficient evidence of a graph-level improvement
([routing model][model],
[Bank of Canada observation][fx]).

The original 108 coherent Depot attempts have USD 14.5637596 of gross compute proration.
As a fixed-usage illustration,
four repetitions of that recorded attempt set cost USD 66.26 on Developer with CI overage,
versus USD 200 on Startup.
That illustration excludes other product pools,
storage,
GitHub runner usage and additional or unpriced work.
It is not a monthly workload forecast or an accepted complete-graph price
([original accounting][accounting],
[routing model][model]).

The initial shared-request frontier remains 97 minutes 31.263 seconds.
It is the last observed provider completion in that cohort,
not an accepted complete-graph latency or reconstructed command dependency critical path.
The native follow-up does not resolve that distinction
([original registry][registry]).

## Adoption conditions and unresolved evidence

The native capability correction is now backed by real consumers:
Depot supplies Windows and Apple Silicon Mac compute through GitHub Actions.
Standalone Depot CI retains its separate Linux-only sandbox scope
([consumer reconciliation][consumers],
[Depot CI scope][ci-overview],
[Depot runner scope][runner-overview]).

Production routing should preserve applicable source checks and platform coverage,
route paid work only within the all-provider budget,
and evaluate the relevant unfinished-work interval using repeated measurements.
The present evidence favors Developer as the base-plan candidate,
with routine free GitHub work retained and selective paid execution evaluated per workload
([authorization][authorization],
[routing model][model]).

The retained limits remain explicit:

- The original primary gate set has source failures and a prerequisite-blocked gate.
- The initial desktop and VM consumer inventories include unexecuted assertions.
- Physical logger TypeScript shard evidence remains incomplete despite aggregate reports.
- Historical resource bindings and the full trial invoice remain unreconciled.
- The monthly arrival mix,
  storage charges,
  mixed runner allowance allocation and complete-graph waiting benefit are not established by this follow-up.
- Mac guest hardware varies under one label,
  and the physical CPU model was not probed.

These limits constrain production acceptance and savings claims.
They do not turn the documented and executed native runner products into unsupported operating systems
([initial evaluation][initial],
[original accounting][accounting],
[native evidence][consumers]).

[initial]: tech-depot-complete-ci-trial-vet-2026-10-17.md
[registry]: depot-final-gates-and-intervals-2026-10-17.evidence.json
[accounting]: depot-repository-accounting-2026-10-17.evidence.json
[original-consumers]: depot-coherent-host-consumers-2026-10-17.evidence.json
[manifest-first]: depot-complete-cohort-native-runners-20261018T034256508Z.json
[manifest-second]: depot-complete-cohort-native-recovery-20261018T035939491Z.json
[first-wave]: depot-native-runner-first-wave-2026-10-18.evidence.json
[second-wave]: depot-native-recovery-wave-2026-10-18.evidence.json
[consumers]: depot-native-consumers-and-variation-2026-10-18.evidence.json
[paths]: depot-native-checkout-paths-2026-10-18.evidence.json
[model]: depot-native-hybrid-routing-model-2026-10-18.evidence.json
[long-paths]: ../troubleshooting/depot-windows-long-path-checkout.md
[authorization]: ../planning/depot-ci-trial.md
[pricing]: https://depot.dev/pricing
[runner-overview]: https://depot.dev/docs/github-actions/overview
[runner-types]: https://depot.dev/docs/github-actions/runner-types
[ci-overview]: https://depot.dev/docs/ci/overview
[github-billing]: https://docs.github.com/en/billing/concepts/product-billing/github-actions
[fx]: https://www.bankofcanada.ca/valet/observations/FXUSDCAD/json?start_date=2026-10-16&end_date=2026-10-16
