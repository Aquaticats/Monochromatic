<!--
status: historical initial cohort; expanded native evaluation recorded separately
lifecycle-phase: validation incomplete; no validated finalist
subject: Depot complete CI trial
scope: Frozen four-case complete affected CI graph; no migration or adoption
started: 2026-10-17
last-updated: 2026-10-18
governing-skill: .agents/skills/choosing-technology/SKILL.md
governing-skill-commit: 37117e36397e30233062828581a5e38217da9458
governing-skill-sha256: 552ac9955299b65a9f921a4e836b60a3fabc15d3477eeb8ec2bfb3f400647f9b
compatibility-fingerprint: 9c4a914216d017284ed9db714fe5df27b848be7f7445388e32ef57a0d4727a5c
active-audit-owner: /root
prior-compatible-report: none; new frozen complete-graph evaluation subject
unresolved-preferences: none required for this no-recommendation result
criteria-and-weights: not scored; no fully validated finalist
hard-constraints:
  - Complete applicable gate and real artifact-consumer accounting
  - No account-wide access
  - No new cohort or application source fixes
  - Preserve four historical source cases and six frozen harnesses
base-components:
  - Inspectable open-source local technology
  - Managed service or SaaS
overlays:
  - CI high-trust execution
  - Incumbent replacement parity
  - Multi-platform and browser claims
  - Native and prebuilt artifact boundaries
-->
# Depot complete CI trial evaluation

The [expanded native runner and hybrid routing evaluation](tech-depot-complete-ci-trial-vet-2026-10-18.md)
records the authorized follow-up and current scoped verdict.
This document retains the original frozen cohort and its acceptance limits.

## Decision and acceptance limits

This trial does not yet establish the intended GitHub/Depot routing under the user's budget and waiting-time objective.
The reconciled registry contains 770 primary gates:
654 pass,
115 fail,
and one is blocked by a failed prerequisite.
The 33 external gates have explicit execution or non-execution dispositions.
Task completion does not establish complete assertions,
paired provenance,
or artifact-consumer acceptance
([terminal registry](depot-final-gates-and-intervals-2026-10-17.evidence.json)).

The evidence supports a narrower conclusion:
Depot executed the Linux producer and consumer workloads represented by the retained runs,
including containerized mutation tests,
browser matrices and software-composited desktop consumers.
The complete measurement also depends on GitHub-hosted native platforms and a local Android consumer.
It is therefore a mixed-provider trial,
not a measurement of an entirely Depot-hosted graph
([dispatch evidence](depot-complete-dispatch-2026-10-17.evidence.json),
[consumer evidence](depot-coherent-android-2026-10-17.evidence.json)).

No adoption or vendor ranking follows from this report.
Required validation is incomplete,
and neither a controlled GitHub baseline nor a fully reconciled trial invoice is available.
A service reliability verdict cannot be inferred from source assertion failures or an unexplained worker termination.
The primary evidence consists of retained provider responses,
hashed outputs and source-bound inventories;
it is stronger for observed execution than for root causes,
repeatability or billing
([accounting limits](depot-repository-accounting-2026-10-17.evidence.json),
[completion incidents](../troubleshooting/depot-logger-final-completion-gaps.md)).

## Immutable workloads and scope

The shared cohort is `complete-20261017T111555806Z`.
Its common manifest SHA-256 is
`ec26a2edd105dc17fb7f27bbafbe3bd92e9d0f479901e9a5f8938a98b963fc50`.
The request clock starts at `2026-10-17T11:17:09.003Z`.
The published manifest binds the source cases and frozen harnesses;
the terminal registry independently retains each case's check definitions and exclusions
([published cohort](depot-complete-cohort-complete-20261017T111555806Z.json),
[terminal registry](depot-final-gates-and-intervals-2026-10-17.evidence.json)).

- TOML:
  source `525d2e09f8168a3bdd7e513e26d4c80b363123e5`;
  24 primary gates and three external gates.
- Logger:
  source `ef01490f5e960f8ca0b424c12242caeb3c5a2b5e`;
  719 primary gates and 19 external gates.
- Native core:
  source `cb95cf6d785c3073c192b95f0632cdae7b910894`;
  21 primary gates and five external gates.
- Desktop:
  source `3434eaa455d569c18b52cf15100f77f9db251d21`;
  six primary gates and six external gates.

These are the frozen selection counts,
not a claim that every gate passes or every authored assertion executes
([static registration](depot-complete-static-registration-2026-10-17.evidence.json)).

The selected graph follows changed owners and affected consumers.
It includes package-level semantic linting,
types,
builds,
tests,
fuzzing,
mutation,
host and artifact boundaries.
An out-of-wave skip is reconciled against its owning execution;
an unaffected exclusion is distinct from a failed prerequisite or missing observation.
The measurement contract requires the complete applicable graph and rejects file-scoped Oxlint as a substitute
([measurement plan](../planning/depot-complete-ci-measurement.md),
[frozen definitions and exclusions](depot-final-gates-and-intervals-2026-10-17.evidence.json)).

Twenty original routes use six frozen harness revisions:
seven Depot routes and 13 GitHub Actions routes.
The native macOS repeat retains its own allocation;
the carried-forward Windows job is an alias of its original execution.
The source cases and harnesses were preserved during this reconciliation.
This evaluation authorizes no production migration,
source fixes,
new cohort or account-wide access
([dispatch evidence](depot-complete-dispatch-2026-10-17.evidence.json),
[retry and alias evidence](depot-coherent-final-boundaries-2026-10-17.evidence.json),
[trial scope](../handover/depot-trial-grilling.md)).

## Gate and artifact-consumer results

The status counts come from the frozen check registry joined to retained execution records.
Detailed inventories provide stronger evidence than provider success summaries:
they preserve original identities,
authored skips and unexecuted assertions.
The terminal registry keeps gate status separate from final acceptance
([registry and methodology](depot-final-gates-and-intervals-2026-10-17.evidence.json)).

### TOML

Twenty primary gates pass and four fail.
The failures are file-enforcer Oxlint,
the fuzz coverage-value check,
the file-enforcer container test and its mutation task.
The external generated-consistency check also fails:
an exit-zero generator changes tracked `mise.toml`.
The native TOML mutation task passes,
but its final provider source/cohort guard exits one without a retained diagnostic.
That task receives execution credit without a complete final provenance pair
([TOML execution and outputs](depot-coherent-toml-2026-10-17.evidence.json),
[final guard investigation](depot-coherent-final-boundaries-2026-10-17.evidence.json)).

The native report reconciles 3062 selected mutation identities,
463 physical shard reports and source-return records.
The selected full-suite inventory contains 37 files.
The retained run records 1269 metadata operations and 2538 forwarding events;
the longest operation is 8926.082681 milliseconds against a 60000-millisecond budget.
This proves those retained boundaries for this run,
not every mutation assertion or future execution
([physical shard and operation receipts](depot-coherent-toml-2026-10-17.evidence.json)).

### Logger

The logger graph records 613 primary passes,
105 failures and one prerequisite-blocked skip.
Only 199 logical primary checks have an observation with both source and common-cohort completion pairs.
Static and unit jobs rebuild tracked plugin outputs;
their final guards reject those differences.
The native TOML mutation job has a local source pair,
but its later common-manifest lookup reports `gh: Bad credentials (HTTP 401)`.
The credential rejection cause remains unknown
([logger reconciliation](depot-coherent-logger-2026-10-17.evidence.json),
[completion incident boundaries](../troubleshooting/depot-logger-final-completion-gaps.md)).

The browser and DOM partitions account for all 118 authored cases:
116 pass,
the WebKit OPFS assertion fails,
and one authored WebKit fs-path case skips.
All 32 cells have both provenance pairs.
Configured retries and actual retry indices are zero;
removing a selected identity fails the independent inventory comparison.
The DOM gate passes while the browser gate fails
([browser inventories and results](depot-coherent-logger-2026-10-17.evidence.json),
[gate dispositions](depot-final-gates-and-intervals-2026-10-17.evidence.json),
[tracked WebKit failure](https://github.com/Aquaticats/Monochromatic/issues/1052#issuecomment-6235901728)).

Nine TypeScript native mutation reports verify 5542 selected identities and their source spans across 117 files.
Their 77 ignored entries remain separate.
They declare 863 shards,
but the archives do not retain the real physical shard manifests and reports.
The aggregate JSON recovery closes `#1160`;
it does not close the separate physical retention gap `#1434`
([aggregate evidence](depot-coherent-logger-2026-10-17.evidence.json),
[aggregate recovery](https://github.com/Aquaticats/Monochromatic/issues/1160#issuecomment-6235899048),
[physical retention issue](https://github.com/Aquaticats/Monochromatic/issues/1434)).

The Rust mutation task stops at its original parent test gate:
1806 pass,
six fail and four are ignored.
Its later mutation phase does not execute.
JSONC and file-enforcer mutation tasks fail without native JSON.
The current conf mutation task passes,
but its earlier red baseline remains an unresolved historical observation.
Deliberately failing child controls and aggregate killed statuses are not extra parent failures or assertion proof
([native report boundaries](depot-coherent-logger-2026-10-17.evidence.json),
[preserved conf baseline](https://github.com/Aquaticats/Monochromatic/issues/1156#issuecomment-6235900834)).

Both Electron Wayland consumers,
WireGuard route integration,
terminal answer/cancellation,
notification delivery and offline tmux pass.
WireGuard bypass and the configured-extension serializer assertion fail.
The Pi fixture actually loads all 22 configured extensions.
The full root Oxlint command fails.
The original VM lifecycle task fails before guest execution;
source-free bundle help/list succeeds without establishing Linux or Windows guest lifecycle coverage
([configured consumers](depot-coherent-logger-2026-10-17.evidence.json),
[VM artifact consumer](depot-vm-artifact-consumer-2026-10-17.evidence.json)).

The logger desktop artifact has 118 authored assertions:
110 original passes,
one original failure,
one authored skip and six unexecuted originals.
A synthetic wide-session abort is separate.
All 17 retained session logs match archived hashes.
The provider samples record no OOM or PID-limit rejection for this consumer;
that observation does not explain the abort
([desktop inventory and logs](depot-coherent-logger-2026-10-17.evidence.json)).

### Native and desktop

The native case records 17 primary passes and four failures:
core Clippy,
changed Rust style,
desktop end-to-end and Android instrumentation.
The desktop case records four primary passes and two failures:
changed Rust style and desktop end-to-end.
The native desktop artifact has 114 authored cases,
108 observed originals and six unexecuted originals.
Its 106 original passes,
one failure and one authored skip are separate from its synthetic abort
([native/desktop evidence](depot-coherent-native-desktop-2026-10-17.evidence.json),
[reconciled primary counts](depot-final-gates-and-intervals-2026-10-17.evidence.json)).

All eight Android producer gates pass.
The real local consumer installs the bound APKs and executes all 15 authored instrumentation cases:
13 pass and two fail with the known null-context assertion boundary.
The device reports API 37,
x86_64,
16384-byte pages and font scale 1.0.
The consumer and cleanup evidence are retained;
producer success does not replace these failed assertions
([Android consumer evidence](depot-coherent-android-2026-10-17.evidence.json),
[tracked assertions](https://github.com/Aquaticats/Monochromatic/issues/1002)).

Both Alpine architectures pass their original tests and smoke consumers.
The desktop Windows variants execute their suites,
but the original PowerShell file-redirection probe fails;
independent merged-stream and uncaptured-console successes do not replace it.
Native Windows fails compilation.
macOS executes with source test and Clippy failures.
The unchanged native macOS repeat passes the previously failed decode-cache identity,
without establishing its cause or a fix
([native host consumers](depot-coherent-github-gates-2026-10-17.evidence.json),
[macOS repeat](depot-coherent-final-boundaries-2026-10-17.evidence.json)).

The isolated native feature rows pass,
but the row named `default` actually uses `--no-default-features`.
It receives no claim of Cargo default-feature coverage.
Changed Cargo formatting passes for the native case and fails for the desktop case
([feature and formatter commands](depot-coherent-native-desktop-2026-10-17.evidence.json)).

The four changed-file scanners pass.
The four Scorecard admission gates execute successfully and deny analysis at the configured rolling-day cap.
Their analysis jobs are policy skips,
not executed analyses and not provider quota failures
([GitHub gate evidence](depot-coherent-github-gates-2026-10-17.evidence.json)).

## Reliability and source failures

Source quality differs by incident.
Retained assertion messages establish failed checks;
they do not establish a provider cause.
The 105 failed logger primary outputs are retained with independently checked original hashes.
Public text redacts personal home paths and has separate retained-text hashes.
Issue mapping is incomplete,
so neither these failures nor a package-name match establishes 105 distinct new bugs
([failed primary outputs](depot-logger-primary-failures-2026-10-17.evidence.json)).

Existing source issues include the built-artifact test-import migration,
extension API stub drift,
unused-export API drift and the Doodle `canvg` resolution boundary.
These explain specific matching diagnostics,
not every failure in the same package.
The broad built-artifact import migration is already tracked as `#1651`;
duplicating its finding per package would obscure that shared source rule
([retained diagnostics](depot-logger-primary-failures-2026-10-17.evidence.json),
[test-import migration](https://github.com/Aquaticats/Monochromatic/issues/1651),
[extension stubs](https://github.com/Aquaticats/Monochromatic/issues/621),
[unused-export API](https://github.com/Aquaticats/Monochromatic/issues/594),
[Doodle build](https://github.com/Aquaticats/Monochromatic/issues/1269)).

The logger generated-consistency worker terminates without an archive.
Its log records a sidecar `SIGKILL`,
then exit 137;
the provider reports `Sandbox terminated before worker reported completion`.
Its `16x64` allocation has a peak memory-utilization sample of `0.9975022728697344`.
This is a resource correlation,
not proof of an OOM cause.
It remains separate from rejected tracked-output guards,
the common-manifest credential failure and the TOML guard's missing diagnostic
([separate completion incidents](../troubleshooting/depot-logger-final-completion-gaps.md),
[worker issue](https://github.com/Aquaticats/Monochromatic/issues/1182),
[TOML guard issue](https://github.com/Aquaticats/Monochromatic/issues/1179)).

The macOS transport incidents remain open even though a later unchanged-source repeat succeeds.
Similarly,
GitHub CLI `2.102.0` initially rejects reviewed issue comments with
`GraphQL: was submitted too quickly (addComment)`.
A later paced retry publishes all four comments with exact body readback.
That verifies operational recovery,
not the deciding server quota or pacing as the cause of recovery
([macOS retry evidence](depot-coherent-final-boundaries-2026-10-17.evidence.json),
[comment incident and source trace](../troubleshooting/github-issue-creation-concurrency.md)).

These observations are not a controlled provider reliability sample.
The runs use different workloads,
resource sizes and failure boundaries;
the evaluation lacks repeated unchanged-input distributions against an equivalent incumbent graph.
No provider success-rate,
speedup or availability percentage is justified
([execution definitions](depot-final-gates-and-intervals-2026-10-17.evidence.json),
[measurement contract](../planning/depot-complete-ci-measurement.md)).

## Provider allocation, cost, and critical paths

The strongest cost evidence is the frozen cohort's explicit resource declarations joined to precise Depot metrics.
Historical YAML matches are weaker:
their exact uploaded bytes are not established.
Current published prices establish a gross proration method,
not the account's historical invoice or net allowance consumption
([repository accounting evidence](depot-repository-accounting-2026-10-17.evidence.json),
[endpoint precision investigation](../troubleshooting/depot-ci-workflow-metadata-accounting.md)).

The coherent Depot cohort has 108 allocated attempt intervals,
34103393 summed worker milliseconds and 291275.192 allocated vCPU-seconds.
At the published `$0.00005` per vCPU-second,
that prorates to **$14.5637596 gross Depot compute**.
This is allocation,
not measured CPU work,
and excludes subscription,
storage and other providers
([exact resource ledger](depot-coherent-logger-2026-10-17.evidence.json),
[retained rate calculation](depot-repository-accounting-2026-10-17.evidence.json),
[Depot pricing](https://depot.dev/pricing)).

Depot's current documentation bills sandbox usage by the second without a one-minute minimum.
Included minutes use the smallest sandbox,
two CPUs and eight GB.
The coherent allocation equals 2427.2932666666666 such minute equivalents.
No allowance credit is deducted:
the account's other consumption and billing terms are outside the collected scope
([Depot CI overview](https://depot.dev/docs/ci/overview),
[retained primary text and calculation](depot-repository-accounting-2026-10-17.evidence.json)).

The repository-filtered historical inventory contains 276 terminal run records,
122 workflows,
394 jobs and 391 attempt records.
Of those attempts,
389 have observed allocation intervals;
two successful `no_sandbox` records have no start timestamp.
The latter receive no invented interval or assumed zero invoice charge.
There are 263 historical intervals with provisional resource candidates,
prorating to a separate **$47.2234074 provisional compute amount**,
and 18 allocated intervals without a resource binding.
The provisional amount is not verified historical spend and must not be combined into a verified trial total
([complete scoped inventory and candidates](depot-repository-accounting-2026-10-17.evidence.json)).

The coherent GitHub records contain 21 executed job intervals and four skipped analysis jobs.
Executed intervals sum to 11000000 milliseconds;
their overlap union is 4595000 milliseconds.
The native macOS repeat contributes its own 1849000-millisecond interval.
Its Windows alias contributes no second execution.
These are whole-second job start/end measurements,
not invoice quantities
([job and alias ledger](depot-final-gates-and-intervals-2026-10-17.evidence.json)).

GitHub's current policy makes standard hosted-runner execution free for public repositories.
That policy does not establish historical account charges,
artifact storage charges or the cost of uncollected preparation work.
The local Android lifecycle takes 59240 milliseconds,
including a 6854-millisecond instrumentation interval.
The local record does not establish an exclusive CPU/RAM allocation or incremental electricity cost
([GitHub billing policy](https://docs.github.com/en/billing/concepts/product-billing/github-actions),
[local and GitHub interval limits](depot-final-gates-and-intervals-2026-10-17.evidence.json)).

From the common request to the last provider finish,
the observed frontier is **5851263 milliseconds**,
or 97 minutes 31.263 seconds.
The last attempt is logger `cnhrgd3dct`,
ending at `2026-10-17T12:54:40.266Z` on `4x16`.
The combined coherent provider interval union is 5837644 milliseconds.
Summing overlapping jobs would misstate elapsed time
([provider intervals](depot-final-gates-and-intervals-2026-10-17.evidence.json)).

This frontier includes failed and incomplete results.
It is neither successful complete-graph latency nor a reconstructed command-level dependency critical path.
The historical Depot interval union is 28061005 milliseconds with peak concurrency of 108 attempts,
but that union also does not identify dependency causes or savings.
Missing final provenance,
unexecuted assertions and uncollected historical GitHub/local allocation prevent a complete cost-and-critical-path claim
([historical intervals](depot-repository-accounting-2026-10-17.evidence.json),
[explicit reconstruction limits](depot-final-gates-and-intervals-2026-10-17.evidence.json)).

## Adoption conditions and unresolved evidence

The immediate disposition is **no adoption recommendation**.
The complete applicable graph has not met its acceptance contract.
Reopening that decision requires explicit resolution or accepted disposition of failing gates,
paired completion evidence,
actual artifact-consumer assertions and the missing allocation boundaries
([acceptance contract](../planning/depot-complete-ci-measurement.md),
[terminal acceptance flags](depot-final-gates-and-intervals-2026-10-17.evidence.json)).

### Candidate ledger and vetting boundary

#### Depot CI

Depot CI is the explicitly requested managed-service trial candidate.
Its open-source CLI is a separate local component;
inspecting that client does not make the hosted control plane open source.
The trial demonstrates Linux execution and inspectable client behavior at the collected boundaries.
It does not complete service terms,
retention,
security,
support,
business-continuity or full client maintenance/provenance audits
([client source trace](../troubleshooting/depot-ci-workflow-metadata-accounting.md),
[execution evidence](depot-final-gates-and-intervals-2026-10-17.evidence.json)).

#### GitHub Actions

Retaining GitHub Actions is the measured incumbent path for native platforms and repository gates.
It preserves those demonstrated boundaries,
but this evaluation does not establish a complete equivalent GitHub-only graph or its comparative latency and spend
([actual GitHub consumers](depot-coherent-github-gates-2026-10-17.evidence.json)).

#### Local execution

Local execution is an existing consumer path,
demonstrated here by Android instrumentation.
It adds real device-boundary evidence,
but has no complete-graph capacity,
isolation or operating-cost comparison in this trial
([Android execution](depot-coherent-android-2026-10-17.evidence.json)).

#### Depot GitHub Actions runners

The standalone Depot CI service provides Linux x86/Arm sandboxes.
Its native OS restriction does not apply to Depot's separate GitHub Actions runner product,
which offers Windows Server and Apple Silicon macOS runners.
GitHub Actions orchestrates those jobs while Depot supplies the compute.
The original native jobs in this report used GitHub-hosted machines;
that placement is not evidence that another compute provider is required.
The user has instructed execution of the missing Depot native-runner evaluation
([Depot CI sandbox scope](https://depot.dev/docs/ci/overview),
[Depot GitHub runner types](https://depot.dev/docs/github-actions/runner-types),
[correction and follow-up](../troubleshooting/depot-ci-versus-github-runner-platforms.md)).

This is a named-candidate workload evaluation,
not saturated vendor discovery.
The additional technology-selection gates remain pending wherever the trial lacks evidence.
Default and non-default workload validation,
consumer completeness and replacement parity do not pass as a whole.
A broad market ranking,
weighted score and sensitivity analysis are not applicable to a set with no fully validated finalist.
No failed gate is offset by a soft score.
No decision record or migration is authorized by this report
([technology-vetting requirements](../../.agents/skills/choosing-technology/SKILL.md),
[trial authorization](../planning/depot-ci-trial.md)).

### Evidence required for acceptance

The unresolved boundaries are concrete:
retain real logger mutation shard evidence;
complete or explicitly dispose of missing desktop and VM assertions;
recover final source/common bindings;
resolve the generator termination and tracked-output differences;
finish source-issue reconciliation;
bind historical resource sizes and uploaded definitions where recoverable;
and reconcile remaining provider/local allocation and billing.
A latency comparison additionally requires equivalent immutable graphs and repeated unchanged-input measurements.
These are acceptance conditions,
not authorization to alter the frozen cohort or rerun resource-consuming jobs
([completion gaps](../troubleshooting/depot-logger-final-completion-gaps.md),
[physical shard gap](https://github.com/Aquaticats/Monochromatic/issues/1434),
[accounting limitations](depot-repository-accounting-2026-10-17.evidence.json),
[measurement scope](../planning/depot-complete-ci-measurement.md)).
