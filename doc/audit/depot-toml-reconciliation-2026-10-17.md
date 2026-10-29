# TOML development-run reconciliation

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/mutation-test` is now `package/mutation-test/cli`.

Run `p9r4xl6n12` is terminal with incomplete selected execution.
The [evidence](depot-toml-reconciliation-2026-10-17.evidence.json)
reconciles the frozen 24-check manifest against actual emitted identities and downloaded artifacts.
This development run is not a complete-graph latency measurement.

## Actual check coverage

Of the 24 package checks,
19 passed and three failed.
TOML mutation started without a terminal result.
File-enforcer mutation never executed.
A `check-started` marker for a deferred check is not proof that its command ran;
the original executor emits scheduling markers even for skipped work.

The core attempt `lzb1kz3c1b` executed 21 checks:
19 passed,
and full-package file-enforcer Oxlint plus TOML fuzz coverage failed.
The three container/mutation checks were explicitly deferred in that attempt.
The container attempt `ltfw0kr3cd` passed its first three builds,
failed file-enforcer container tests,
and then remained inside TOML mutation until cancellation.
An independent readonly-parameter plugin build was also never reached in that attempt;
the core attempt separately built and tested that selected plugin successfully.

## Timeout boundary

File-enforcer container tests completed with failure after `79620.70287899999` milliseconds.
Their retained diagnostic repeats the logger sink verification timeout followed by `SIGKILL` from issue `#1070`.
The current evidence does not prove an out-of-memory kill or a causal link between those observations.
TOML mutation started at `2026-10-17T04:36:53.885Z`.
The provider canceled the job at `2026-10-17T05:04:22.162Z` with:

```text
# Depot provider cancellation diagnostic
The job was canceled because it exceeded the maximum execution time of 30 minutes.
```

The original executor waits for each child exit and buffers its output to a file.
It has no per-check deadline.
The job timeout prevented the artifact upload step from retaining the active mutation log.
The mutation task's internal cause remains unknown.
The canceled attempt also lacks an end-of-job source status/hash proof.
The replay command `mise run reconcile-toml -- --assert-complete` fails on this exact missing-terminal boundary.

## Source failures and artifact proof

The separate generated-file check `root.generated-consistency` ran `//:sync:files` and exited successfully.
It failed the clean-output contract because generated `mise.toml` removed package-bin PATH entries.
The retained patch matches the existing ambient-node_modules generation boundary in issue `#335`.
The historical LFS failure in issue `#1071` is a separate incident.
The full-package Oxlint task reported 84 errors across 137 files:

- `test-import(require-eventual-artifact)`: 62.
- `eslint(no-irregular-whitespace)`: 22.

No plugin panic appears in this observation;
issue `#1066` remains a separate historical incident.
Fuzz coverage remains issue `#1067`.
The unexecuted file-enforcer mutation task has a separate historical path-resolution failure in issue `#1068`.
No source fix is included.

- Generated artifact `01a14824-cf6a-792d-be62-536d80d971cb`:
  SHA-256 `97e1b4b8dd1087ebf4021ddeafe535551f017e20278e111ec768b9b3f7242460`.
- Core artifact `01a14826-ddbd-7b11-ac8f-2a571c7373a9`:
  SHA-256 `473f946fa516ebf1a1d41a31f594b126f0a908e77554d79d25f8d2a60a44b457`.

Every artifact member has a retained byte count and digest.
Downloaded core result records match streamed records,
and every check-log hash matches its emitted result.
Extraction validates relative names and writes fresh regular files,
with bounded expanded output.
The canceled container attempt has no listed artifact.

## Provider usage and continuation queue

- Attempt `ltfw0kr3cd`:
  `1816904` provider milliseconds at `16x64`.
- Attempt `lfx15ztb6k`:
  `89042` provider milliseconds at `16x64`.
- Attempt `lzb1kz3c1b`:
  `224138` provider milliseconds at `16x64`.

The summed provider intervals are `2130084` milliseconds,
or `34081344` allocated vCPU-milliseconds.
These values retain failed and canceled usage;
no monetary total is inferred.

This audit records the isolated bounded continuation and its controls.

## Bounded continuation route

The pushed trial branch `trial/depot-toml-continuation-20261017`
freezes harness `640a576ade81c4edfee6a603543a7fbd18faa46b`
(current prepared route as of 2026-10-17:
`0f1ef041a0de2ff6d2db6dd55210b1160c592648`).
Its workflow is `.depot/workflows/depot-complete-toml-bounded.yml`,
with required inputs
`cohort_id`,
`manifest_sha256`,
`requested_at`,
and `scope`.
`scope=complete` runs `toml`,
`generated`,
`container`,
and the two-member `mutations` matrix.
`scope=mutations-only` runs only the two missing mutation gates.
The independent jobs preserve the exact source tasks and their original flags.
Each bounded gate has a `3600000`-millisecond consumer budget and a `90`-minute job limit.
Separate cleanup operations have `60000`-millisecond budgets.
The adapter retains incremental stdout,
scoped container states/events,
raw mutant shard JSON,
per-file hashes,
and source/cohort proofs before and after consumers.
The private `TMPDIR` and label-only Podman wrapper are recorded as separate harness overlays.

Exact-source controls pass before and after consumers,
reject a tracked source mutation,
and pass again after restoring the disposable fixture.
Both identity and command-fixture modes reject CI use.
A real container control observed `2147483648` memory bytes,
`2` CPU quota,
and `64` PID capacity.
Its SIGTERM-resistant Node child and grandchild were killed,
and the correlated container was removed.
The `15000`-millisecond consumer timeout control finished in `48354.209200000005` milliseconds,
including acknowledged shutdown and evidence retention.
A normal exit `0` passes,
and an exit `7` remains failed with its output retained.
Synthetic shard manifest/report files are hashed and retained as fixture evidence.
The initial cleanup controls failed honestly with containers still present after their short shutdown budgets;
their original results remain in the evidence alongside the successful control.
Scoped dprint and `git diff --check` pass.

Issues [#1111](https://github.com/Aquaticats/Monochromatic/issues/1111)
and [#1112](https://github.com/Aquaticats/Monochromatic/issues/1112)
retain the two actual lint diagnostic boundaries.
Issue [#1113](https://github.com/Aquaticats/Monochromatic/issues/1113)
tracks the missing-terminal scheduling and artifact-retention boundary.
All three bodies,
open states,
and triage labels were re-fetched and verified.

Development dispatch `p6vm5s00w6` requests only the two missing mutation gates.
Final coherent dispatch remains held.

## Development mutation outcomes

Run `p6vm5s00w6` is a separate development continuation on harness `274ec49595d60f663249d61ab35a0806fe29aff0`.
The provider confirms branch `refs/heads/trial/depot-toml-continuation-20261017` and trigger `workflow_dispatch`.
Actual consumer proofs retain the exact request identifier,
manifest hash,
and request time.
Core,
generated,
and completed container jobs are skipped by `scope=mutations-only`.

File-enforcer mutation attempt `3r4zr1jp80` reached an actual failed terminal outcome,
repeating the doubled package-path `ENOENT` from issue `#1068`.
Its selected source task ran for `298.0142400000004` milliseconds.
The wrapper finished in `2738.735309` milliseconds with cleanup verified.
The provider interval is `95451` milliseconds at `16x64`,
or `1527216` allocated vCPU-milliseconds.
The failure occurred before mutant discovery;
no mutant identities or successful campaign are claimed.
Downloaded artifact `01a1487e-a04e-758b-b2cb-910beb55c0ff` has SHA-256
`490aa2cf69719c8894670599d0914720238b4d654905af0f56fd5f60328734bc`.
All check-log hashes and matching clean before/after source proofs are verified.

TOML mutation attempt `q3xbl70q20` independently selected 54 source files,
excluded 37 test files,
and began building its runtime image.
Its terminal outcome is now retained in the development terminal reconciliation.
These development observations do not form a complete-graph latency sample.
Final coherent dispatch remains held.

## Native final report retention

Reading the complete CLI emitter also found its default `cwd()/mutation-report.json` output.
The prepared harness now copies that full native report outside source,
retains its bytes and hash,
and distinguishes fresh reports from pre-existing stale reports.
The positive fixture preserves all native report identities;
the stale-report control is explicitly marked false.

The development run used `274ec49595d60f663249d61ab35a0806fe29aff0`.
Its inputs are unchanged.
A read-only `depot sandbox exec` observer retained the complete native report when orchestration finished.
The actual `/bin/pwd` probe succeeded against its VM.
An initial reader syntax error is retained;
the corrected reader uses a separate output and request record.
If orchestration times out before writing the report,
no completed native report is claimed.
The [instruction proposal](../planning/toml-report-emitter-verification.md)
makes checking report writers and default paths explicit without changing `AGENTS.md`.
Final coherent dispatch remains held.

## Final source guard recovery

The prepared route now uses `640a576ade81c4edfee6a603543a7fbd18faa46b` (current prepared route as of 2026-10-17:
`0f1ef041a0de2ff6d2db6dd55210b1160c592648`).
Its previous native-report recovery revision remains `c159dd63021f9304ab3e4996b1d054e5f19ea0c9`.
The [source guard audit](depot-route-source-guards-2026-10-17.md)
retains direct committed-byte checks for root configuration,
all tracked Cargo metadata,
and selected package configuration,
plus rejection of both Git index exclusions.
The terminal development run retains its original harness identity.

## Terminal TOML infrastructure outcome

Run `p6vm5s00w6` finished with both independent consumer commands failed.
TOML attempt `q3xbl70q20` ran its selected source task for `2640500.405168` milliseconds.
It did not reach the one-hour wrapper deadline;
cleanup verifies no remaining owned process group or labeled containers.
Matching clean before/after source records and every check-log hash are retained.
The combined provider allocation interval is `2834043` milliseconds at `16x64`,
or `45344688` allocated vCPU-milliseconds.

The full native report is retained byte-for-byte as
[`depot-toml-mutation-native-2026-10-17.evidence.json.gz`](depot-toml-mutation-native-2026-10-17.evidence.json.gz).
Its uncompressed SHA-256 is `e3b1023242fb838e30ad22ce1aa0c593f412401be141901594af34c484177bce`.
All `3062` distinct mutant identities reconcile with `9037` shard manifests and attempts.
All final statuses are `runtimeError`,
with no source mutation assertion completion.
Every shard failed `rsync --archive` with exit `23` and:

```text
# Source container copy diagnostic
rsync: [generator] chgrp "/work/." failed: Operation not permitted (1)
```

This repeats the diagnosed root-owned `/work` tmpfs boundary,
separate from user-namespace unmount issue `#1116` and baked-metadata issue `#1117`.
The final TOML route must adopt the complete proven named-volume and baked-metadata adapter
and pass a bounded actual TOML pilot before the full campaign.
The source task body and assertions remain unchanged.

Artifact `01a148a6-f383-7601-97bb-5e2f44ea27ff` is `9453423` bytes,
with SHA-256 `8480e110dfd7c09a672aa6786fa9469b482e08e0df8c8c336e1b55264ce5059d`.
The archive has `9047` members and `57437642` expanded bytes.
Its `32240214`-byte check log exceeded the first collector's `16` MiB member bound;
a fresh extraction used measured `64` MiB member and `128` MiB total bounds,
retaining the first partial extraction.
One metadata snapshot assertion is retained without attributing a narrower cause.
Issue `#1113` remains open while the actual mutation campaign boundary is unresolved.
Final coherent dispatch remains held.

## Repaired adapter consumer pilot

Development pilot `0j8zhbn9n0` uses harness `a6dcb98aaa415c49633474e46acc2a3c366e31df`
and the complete containers adapter from `d098532a5b6946231daf3c4f43e38444359a307e`.
The original source mutation task selects `src/toml-get-comments.ts`,
which has one mutant in the retained full inventory.
It first emits an actual dry-run enumeration,
then runs the full unit suite with one container and shard size one.
The consumer budget is `900000` milliseconds in a `25`-minute job.
Its terminal evidence is now reconciled in the bounded consumer outcome section.
The exact request,
inputs,
receipt,
controls,
and additive observed-proof comment on issue `#1047` are retained.
Final coherent dispatch remains held.

## Bounded consumer outcome and metadata receipt repeat

Pilot `0j8zhbn9n0`,
workflow `qrmzdmwf4q`,
job `kvg78klbhn`,
attempt `4m0vbp3309`,
completed its original source command with exit zero.
The actual baseline was green over the retained 37-file full-suite selection.
The only mutant,
`9cd1056a095eaea1`,
reached source status `compileError` at `src/toml-get-comments.ts:28:26`.
This is a terminal compilation result with no mutant assertion execution credit.
The native report has no infrastructure errors or unrun identities.
Its 650 bytes have SHA-256 `692e5dd934f799518de76762ee34a0f22558a66a2832188cf5b074dfc37d7825`.
The retained shard baseline records tests at `4237.252068` milliseconds and type checking at `271.928494` milliseconds.
The mutant records `499.24852199999987` milliseconds.

Provider status remains failed because the wrapper's metadata operation asserted `null !== 0`.
The old wrapper did not retain the underlying command,
signal,
error code,
or output byte counts;
its exact cause remains unproved.
Scoped cleanup passed with no owned process group or labeled container remaining.
The wrapper observed `59945.775575` milliseconds.
The provider interval was `167222` milliseconds at `16x64`,
or `2675552` allocated vCPU-milliseconds.
Artifact `01a148b3-bf92-7122-bdc0-926d81d5263d` is 44242 bytes,
SHA-256 `fc3178d5e8528b08884098f65f88db305aa41bd202b08369325769135fee6fb9`.
All 19 members are individually hashed.
The six periodic metadata samples missed the live source container;
declared inner resource flags are not presented as runtime-inspected limits.

The actual owned metadata bind preserved 29934 source bytes,
SHA-256 `e82c7a5e4404440e121fffb742f4cde0c444843a3f69dd74707e55789cab66a7`,
changing ownership from root to source UID/GID `1001:1001` with mode `0600` and a read-only bind.
The source image remained unchanged.
This green baseline crosses the original source-copy and baked-metadata boundaries;
it does not complete the full mutation campaign or erase the historical infrastructure results.

Harness `0f8ad795aa92d3beb83e3bfa4e4aa54f1bea06f8` now writes each exact Podman operation
to an incremental JSONL receipt before assertions,
including signal,
error code,
complete arguments,
timings,
output byte counts,
and hashes.
A disposable eight-second timeout control observed `SIGKILL`,
`ETIMEDOUT`,
and 24 bytes in each output stream.
That control proves receipt coverage and does not diagnose the historical cloud operation.
Actual repeat `mj128hqwv0` uses the same source command and eight-second metadata budget.
Its exact request and inputs are retained;
terminal collection must precede any remedy or full retry.

## Independent mutation producer failure

Issue [#1118](https://github.com/Aquaticats/Monochromatic/issues/1118)
records a separate source operator lookup defect.
The unchanged source consumer emits string `endsWith` for mapped `startsWith`,
nothing for an unmapped name,
and function-valued replacements for inherited `toString`,
`constructor`,
and `hasOwnProperty`.
The plain-object lookup at `package/cli/mutation-test/src/engine/operator/chain-method.ts:173`
accepts inherited Object functions.
Three original cloud candidates consequently omit `replacement` in JSON.
Their identities,
direct source probe,
duplicate searches,
and re-fetched issue body are retained.
This producer defect did not cause the earlier source-copy failures.
No application source change is included.

## Canonical final route binding

The [common-manifest guard audit](depot-common-manifest-route-guards-2026-10-17.md)
retains the current clean pushed route references and 20 accepted plus 110 rejected controls.
The case manifests remain provisional and correctly rejected for final acceptance.
Final dispatch waits for root's complete frozen case bytes and the actual canonical API read.

## Exact metadata repeat failure

Repeat `mj128hqwv0`,
workflow `t8wq69b56g`,
job `h4c49btlhq`,
attempt `3wnf82gzb8`,
again completed the source command with exit zero and the same `compileError` identity.
Its retained green baseline has no unrun mutant or infrastructure error.
The wrapper remained failed because this exact metadata command exceeded its unchanged budget:

`container-bin/podman ps --all --filter label=io.monochromatic.toml-attempt=7185dde1-5802-43c2-b296-e257b9f313cf --format json`.

The operation ran from `2026-10-17T07:25:23.786Z` to `2026-10-17T07:25:31.793Z`,
observed `8006.465648000001` milliseconds against an `8000`-millisecond budget,
and ended with null status,
`SIGKILL`,
`ETIMEDOUT`,
and errno `-110`.
Stdout contained three bytes,
`[]\n`;
stderr contained zero bytes.
Printing an empty list does not prove the forwarding process completed.
The generated Node adapter synchronously forwards to `sudo --non-interactive`
and the resolved actual Podman executable,
then propagates the returned status after its conditional cleanup.
The underlying Podman,
sudo,
and Node forwarding lifecycles are not yet isolated.
Seven other list operations and the final event query returned zero.
All operation arguments,
statuses,
timings,
signals,
error fields,
output bytes,
and hashes are retained.

Scoped cleanup passed.
The provider interval is `165116` milliseconds at `16x64`,
or `2641856` allocated vCPU-milliseconds.
Artifact `01a148c0-c29e-7644-a0ec-f546f5caf3e8` is 46389 bytes,
SHA-256 `3e12339c854c28f69ef9ecdc29e03b2f1f0efe2959ad66bfd23347e1d75df2bb`,
with 20 individually verified members.
The authorized next trial may increase only the metadata deadline with explicit forwarding receipts,
retain rejection controls,
then repeat the bounded actual consumer before the full campaign.

## Bounded forwarding trial request

Actual trial `vlrxjltfkq` uses harness `316e1a8a309e71e456a741eb63e5ff3b94b75646`.
Its metadata-only deadline is 60000 milliseconds.
The unchanged source consumer retains the complete proven container adapter byte-for-byte.
An observer-only copy adds before/after sudo forwarding receipts and preserves actual Podman arguments.
The source correlation wrapper now retains each invocation without changing its forwarded argument policy.
Disposable forwarding,
timeout rejection,
and removed-timeout detection controls pass.
Original eight-second failures remain retained.
Only complete actual source,
metadata,
correlation,
and cleanup evidence permits the full immutable-source retry.

## Verified bounded source and observer recovery

Actual trial `vlrxjltfkq`,
workflow `fkzv9kxx7k`,
job `gz17n0n8q6`,
attempt `tfzhj6cztj`,
finished successfully on immutable harness `316e1a8a309e71e456a741eb63e5ff3b94b75646`.
The source command again exited zero with the same green baseline and one `compileError` identity.
No mutant assertion execution or full-campaign pass is claimed.
All ten metadata operations and all forwarded sudo calls returned zero.
The longest list operation took `14100.441549` milliseconds;
the Node observer recorded `14077.079173` milliseconds inside its synchronous
`sudo --non-interactive /usr/local/bin/podman` call.
That call then returned zero.
This localizes most observed delay to the forwarded call;
the division between sudo and actual Podman remains unresolved.
Every operation duration is retained.
The original eight-second failures remain failed historical observations.

The actual source correlation receipts retain image lookup,
build,
and run calls.
Run forwarded the original source argv with only the existing attempt label inserted,
and returned zero.
Before/after source identity and scoped cleanup passed.
The original generated consumer forwarder and observer copy are both retained as artifact bytes.
Removing only the two observer lifecycle lines reconstructs the original file exactly.
The periodic metadata samples still missed the live source container;
inner resource limits remain declared argv values for this bounded pilot.

The provider interval was `165081` milliseconds at `16x64`,
or `2641296` allocated vCPU-milliseconds.
Artifact `01a148cd-37bb-750c-969a-1d040159af2e` has 55933 bytes,
SHA-256 `35ab60a4bbd6adb146676b810e3123d3d0fe08a2174873a73736480ee596be19`.
All 24 members are individually hashed.
Issue [#1130](https://github.com/Aquaticats/Monochromatic/issues/1130)
records the distinct observer deadline boundary and retained rejection controls.
The next authorized action is the full TOML-only immutable-source mutation retry.

## Full immutable-source mutation retry request

Development run `23dk3nr90f` dispatches only `package.module.toml-edit.test.mutation`.
Request time is `2026-10-17T07:50:29.848Z`;
the development manifest digest is `4b18266890cb521dfc87256ecd13f8869f39ee35b747186ca16cd53998fe84a0`.
The clean pushed branch is `trial/depot-toml-continuation-20261017`,
harness `0f1ef041a0de2ff6d2db6dd55210b1160c592648`,
workflow `.depot/workflows/depot-complete-toml-full-mutation.yml`.
Its 4931 authenticated provider bytes match SHA-256
`ce2c907be8b7e2bc449a866964869ea87e3ee0a5140c4453faf07ed0c110e954`.

The original source command and defaults remain unchanged:
two containers,
16 mutants per shard,
1800-second sessions,
5000-millisecond test floor,
baseline timeout factor three,
and one single-container retry.
The full selector is enumerated again before execution.
The retained prior inventory has 3062 distinct mutant IDs.
The wrapper budget is 18000000 milliseconds and the provider job budget is 360 minutes.
These are execution bounds,
not a duration forecast.
The bounded compilation-only pilot does not establish full-suite mutant throughput.
Source shard progress,
all actual outcomes,
metadata durations,
and scoped cleanup remain required for terminal reconciliation.
The allowed budget and rejected over-limit fixture are exercised;
removing the exact upper-budget guard makes the rejected fixture execute.

Single terminal wait session `96306` was active at dispatch
and is terminal and reaped as of `2026-10-17`.
Issue `#1130` is closed with re-fetched observed-recovery comment `6231392374`.
Issues `#1113` and `#1118` remain open.
This independent development run is separate from the held final coherent cohort.

## Mutation runner execution boundary

Immutable source `525d2e09f8168a3bdd7e513e26d4c80b363123e5`
implements `runTests` in `package/cli/mutation-test/src/container/test-run.ts`.
Its selected files execute in order,
stopping at the first failure or timeout.
The full-suite selection contains 37 possible test files;
it does not prove that every file executes for every mutant.
Compiler rejection reaches none of these mutant test invocations.

The same source maps a Node spawn exception to `kind: 'failed'`.
`package/cli/mutation-test/src/container/mutant-loop.ts` maps that kind to `killed`.
The reconciliation checker therefore reports source-reported test-runner terminals separately
from actual emitted assertion evidence.
A status label alone does not prove that a test assertion executed.
Complete cloud logs must also be checked for spawn diagnostics.
No such cloud spawn failure is claimed from reading this path.

A separate development control is being prepared on
`trial/depot-toml-spawn-control-20261017`.
It will exercise the unchanged baked source runner with a disposable typed fixture,
an actual compiler and positive assertion control,
then remove only the disposable Node launcher after a successful typecheck.
The active full retry and accepted final route refs remain unchanged.

The independent classification control is dispatched as `m018vgrtcv`,
using clean harness `b78123c7bd8e20d4319cddd63c34c70fa7638d72`.
Its exact development request and authenticated provider workflow/helper bytes
are retained in the machine evidence.

The first control finished failed before mutation assertions:
its fixture typecheck returned `clean: false`,
but the initial observer lost the returned detail.
The unchanged bounded source prerequisite still completed with its original single compiler outcome.
Source before/after identities and scoped container/volume cleanup are retained.
No false mutation credit is claimed from this attempt.
Its provider interval was `153785` milliseconds at `16x64`,
or `2460560` allocated vCPU-milliseconds.
Repeat `j03l179f3x` adds incremental phase and real-compiler stdout/stderr receipts,
and preserves original PATH commands except the deliberately removable Node launcher.
Its exact request is retained separately from the unchanged full campaign.

Observation repeat `j03l179f3x` retained the actual `EEXIST` detail.
NanoSpawn 2.1.0 inherited the control eval body into its compiler invocation,
so the control re-entered itself before any mutation assertion.
Its five actual source module hashes match immutable source.
The provider interval was `182992` milliseconds at `16x64`,
or `2927872` allocated vCPU-milliseconds.
Actual local positive and guarded negative controls verify file-versus-eval carrier behavior.
File-based cloud repeat `699f9lg5zm` is dispatched separately,
on clean harness `f636b13af902ed6abe5cd8758da4cb8ba99f2175`.
The original full campaign retains its file-based source dispatcher and unchanged inputs.

## Full mutation terminal reconciliation

Development run `23dk3nr90f` completed successfully at unchanged source
`525d2e09f8168a3bdd7e513e26d4c80b363123e5` and harness
`0f1ef041a0de2ff6d2db6dd55210b1160c592648`.
The historical wait session `96306` has been reaped with exit zero.
All `3062` selected identities have native terminal outcomes:
`921 killed`,
`256 survived`,
`3 timeout`,
and `1882 compileError`.
Native `runtimeError` and final infrastructure-error counts are zero.
These are native runner classifications;
`killed` does not independently prove an assertion ran.
Compiler outcomes receive no mutant assertion credit.
The source CLI reports raw totals without a mutation score or threshold.

All `463` physical manifests and reports reconcile against `3333` assignments.
The `32` unrun prefix assignments have subsequent final dispositions.
Every selected identity matches its frozen source span,
recomputed native ID,
and a matching result in a physical report with a passing baseline.
The candidate full suite contains `37` files;
the unchanged runner stops after its first failed or timed-out file.
It does not execute every candidate file for every mutant.

The source task took `4392514.76837` milliseconds;
the wrapper took `4394121.711807` milliseconds.
The provider interval was `4498869` milliseconds at `16x64`,
or `71981904` allocated vCPU-milliseconds.
This independently requested development job is not complete-cohort latency.
Its result envelope names the broader case inventory,
but its actual check overlay and result execute only TOML mutation.

The source log is `41142330` bytes with SHA-256
`cb31ec5955c4d97ba6ce6f0b97a461dba37a8cc16536273f4a471f8c45b667c3`.
It has no matching Node-spawn diagnostics.
That observation is limited to `runOneTest`'s caught-spawn-error branch and `rl.error` emitter;
the actual-source emitter control subsequently passed in `rsbr0wrdt0`.
It does not upgrade native `killed` into literal assertion credit.

Of `1302` metadata operations,
four auxiliary inspections returned Podman status `125`
and `Error: no such object`,
with exact arguments,
times,
byte counts,
and stderr retained.
They remain separate from the zero final native infrastructure errors.
The events command succeeded with empty output;
this does not prove complete event retention or establish an inspection failure's cause.
Periodic sampling recorded `858` observations of `460` distinct containers across `441` samples.
All available bounds were `2 GiB`,
`2 CPU`,
and `512 PIDs`;
the four unavailable inspections remain in the sample record.
All `463` original source run commands retain their declared bounds.
All `463` named work volumes have paired successful cleanup and absence receipts,
and all `463` stopped metadata containers have successful removal and absence receipts.
Owned metadata was byte-identical and read-only;
source image identity remained unchanged.
Final scoped containers and process groups were empty.

The complete provider artifact is retained in
[the primary ZIP](depot-toml-full-mutation-2026-10-17.evidence.zip),
`12057855` bytes with SHA-256
`31e45cf2ad99de72d9253c70e1057f050f985c617c1d69a2ee3402f5ec1fa5e3`.
Its `2337` member hashes are retained in the structured audit.
[The compressed reconciliation](depot-toml-full-reconciliation-2026-10-17.evidence.json.gz)
retains every identity,
assignment,
report mapping,
source span,
metadata duration,
resource sample,
cleanup receipt,
exact provider usage,
and the reconciliation code.

The independent file-carrier classifier control `699f9lg5zm` failed an ancillary
hardcoded NanoSpawn metadata lookup before mutation assertions.
Its source prerequisite,
exact `ENOENT`,
before/after identity,
cleanup,
and provider usage are retained.
No source classifier result is claimed from that failure.
Repeat `rsbr0wrdt0` uses actual package resolution at harness
`f650e3e2f923bea21a7baf9565bb1ec3efb4fbde`,
with historical terminal wait session `43521`,
now reaped with exit zero.
The final common cohort remains held.

## Actual source spawn classification control

Development control `rsbr0wrdt0` completed successfully at harness
`f650e3e2f923bea21a7baf9565bb1ec3efb4fbde`.
The unchanged source modules match frozen `525d2e09f8168a3bdd7e513e26d4c80b363123e5` byte-for-byte.
The real compiler passed before and after the disposable PATH Node launcher was removed.
The genuine assertion control executed the fixture with value `2` and returned `killed`.
After Node removal,
the unchanged mutant loop again returned `killed` with no anomaly,
but its test-entry marker did not change.
Direct `runTests` returned `kind: failed` and
`spawn error: Error: spawn node ENOENT`.
This demonstrates false mutation kill credit in the source classifier.
It is independent of whether the completed cloud campaign encountered that failure.

The actual source `runOneTest` catch emitted both expected `rl.error` diagnostics:
`spawn failed for subject.unit.test.mjs: Error: spawn node ENOENT`.
The completed campaign's hash-verified full source log contains zero matching diagnostics at that branch.
This observation does not prove that every native `killed` reached an assertion,
or exclude other failures before an assertion.
The `921` campaign outcomes therefore remain native runner classifications.
The source's early-stop positive and negative controls and its timeout control passed.
Source restoration,
unchanged image identity,
before/after source identity,
and scoped cleanup all passed.

The actual fixture interval was `1331` milliseconds inside `2 GiB`,
`2 CPU`,
and `512 PIDs`.
The entire provider job took `165855` milliseconds at `16x64`,
or `2653680` allocated vCPU-milliseconds.
The unchanged bounded source prerequisite remains a single compiler outcome with no mutant assertion credit.
The first three unsuccessful control attempts and their provider usage remain retained.
Terminal wait `43521` is reaped with exit zero.
The actual-source control artifact is retained in
[the primary control ZIP](depot-toml-spawn-control-2026-10-17.evidence.zip).
No application source was changed.

## Original selected-check terminal accounting

All `24` originally selected task commands now have retained actual terminal outcomes:
`20 passed` and `4 failed`.
This joins historical independent development evidence for coverage accounting only.
The original canceled run remains incomplete in its original record.
The final common cohort remains held.
File-enforcer mutation ran and failed before enumeration with its original `ENOENT` (`#1068`);
it receives no mutant assertion credit.
The original completed file-enforcer container failure remains failed (`#1070`).
The independent actual-source classifier defect is now
[issue `#1142`](https://github.com/Aquaticats/Monochromatic/issues/1142),
verified open with `needs-triage`.
No application source fix was made.
The structured `historicalIndependentCoverage` retains each selected identity,
its exact source task,
its original disposition,
and its actual terminal receipt.
Issue `#1113` is closed against that retained coverage,
with its exact closing comment and closed state re-fetched.
The source failures and native outcome limitations remain explicit.

### Instruction proposal

Tighten existing `AGENTS.md` rule `QIV` to cover runner labels and actual classifier controls.
The proposed wording is retained in the structured audit;
it requires success and failure controls before deriving execution claims from a native label.
This is a proposal only;
`AGENTS.md` and application source were not changed.

The final closure receipt is
[comment `6232394931`](https://github.com/Aquaticats/Monochromatic/issues/1113#issuecomment-6232394931).
Source classifier issue `#1142` remains open.
Final route preparation remains unchanged;
complete frozen case bytes and actual canonical retrieval are still required before coherent dispatch.
