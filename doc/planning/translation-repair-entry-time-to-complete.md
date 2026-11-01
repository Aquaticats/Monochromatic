# Translation-repair entry time to complete

## Decision state

Both generation-12 runs were stopped after correctness blockers were identified.
They are phase evidence,
not completion samples.
Before another `Carena0442` launch,
completion path must target fresh-run median below two hours without weakening quality.
Quality gates,
continuous correction,
distinct confirmation responsibility,
durable prompt payload replay,
and one-provider ordinary operation remain fixed measurement inputs.
Measured arms require both providers with `--require-providers synthetic,hyper`.

This work measures **entry time to complete**:
wall time from entry start through terminal `TALLY`,
including repair,
translate,
lane contest,
consolidation,
confirmation,
persistence,
and publication checks.
Only verified artifact and page count as successful completion.
`status=INCOMPLETE`,
quality rejection,
and operational error are censored unfinished outcomes rather than completion samples.

## Stopped measurements

Snapshot taken 2026-08-29 at 12:14 UTC.

### `Weideriche_`

- Process:
  `proc_a259`.
- Pipeline commit:
  `68c37da59c43529386cad78f3f8078d180d57f35`.
- Pipeline digest:
  `sha256-tree-v1:dc788a666af6bce37b37c55d4d674ac99185656795327d120413c3e1304aec9f58`.
- Run root:
  `~/temp/agent/validation-Weideriche-schema9-half-quorum-v12-20260829`.
- Log:
  `~/temp/agent/validation-Weideriche-schema9-half-quorum-v12-20260829.log`.
- Expected artifact:
  run root plus `artifacts/Weideriche_.json`.
- Expected page:
  run root plus `fixed/people/Weideriche_/page.en.md`.
- Verification log to create:
  `~/temp/agent/verify-Weideriche-schema9-half-quorum-v12-20260829.log`.
- State at snapshot:
  repair and translate complete;
  lane contest active;
  no terminal tally.
- First-attempt result after snapshot:
  `status=ERROR ms=4840305 aborted=false error=slice 1 did not meet absolute naturalness floor`.
- First attempt wrote no artifact or page and queued reattempt with 13 additional cache records.
- Process was stopped during non-conforming whole-entry second attempt.
- Repair slice costs recorded:
  0.001,
  927.787,
  883.347,
  and 17.230 seconds.
- Translate slice costs recorded:
  427.660,
  250.694,
  264.892,
  and 8.567 seconds.

### Pull request 386, `Carena0442`

- Process:
  `proc_3a1c`.
- Pull-request head:
  `a80634a674f94861ea3b7056fba054ca9eab1a2c`.
- Pipeline commit:
  `68c37da59c43529386cad78f3f8078d180d57f35`.
- Fixture pipeline digest:
  `sha256-tree-v1:2231798fcc453ccb9fe7ff688f4690ec7662e22ae70b833f91d70c7ec5cc9f58`.
- Run root:
  `~/temp/agent/validation-pr386-Carena0442-schema9-half-quorum-v12-20260829`.
- Log:
  `~/temp/agent/validation-pr386-Carena0442-schema9-half-quorum-v12-20260829.log`.
- Provenance:
  `~/temp/agent/pr386-Carena0442-run-provenance-20260829.md`.
- Expected artifact:
  run root plus `artifacts/Carena0442.json`.
- Expected page:
  run root plus `fixed/people/Carena0442/page.en.md`.
- Verification log to create:
  `~/temp/agent/verify-pr386-Carena0442-schema9-half-quorum-v12-20260829.log`.
- State at snapshot:
  22 slices prepared;
  first non-metadata repair slice active;
  no terminal tally.
- Process was stopped after same-prompt repetition was rejected as independence mechanism.
- No artifact or page exists.

The pull-request run uses exact pull-request files in minimal Git fixture.
Throwaway pipeline source changes only supply pull-request commit and corpus location.
Its different pipeline digest makes it unsuitable as matched runtime arm against production-pinned run.

## Carena stopped-trace phase analysis

The stopped Carena trace ran for 3,586 seconds and never left repair lane.
Pre-lanes preparation interval was 27 seconds.
Its block-pairing round consumed 26.156 seconds,
leaving 0.844 seconds of uninstrumented pre-lanes work;
repair then consumed 3,559 seconds through log end.
This is liveness and phase-shape evidence only:
Carena and Weideriche overlapped and shared provider capacity,
and no terminal tally exists.

Repair completed metadata slice 0 and content slices 1 to 5.
Slice 6 was still active;
slices 7 to 21 had not started.
Completed content-slice costs were 406.356,
479.550,
556.523,
1,047.759,
and 303.092 seconds.
Their sum was 2,793.280 seconds and arithmetic mean was 558.656 seconds.
Active slice 6 had already consumed at least 761.653 seconds at final logged round.
These values do not predict completion under isolated providers,
but they prove serial repair is completion-path bottleneck in this trace.

Repair emitted 66 completed round records totaling 3,554.810 seconds.
Post-quorum grace totaled 2,994.680 seconds:
84.24 percent of round time and 84.14 percent of repair phase wall time.
Four rounds reached at least 175 seconds of grace and four calls were abandoned at 180-second grace.
Round families by count were critic 10,
panel 9,
editor 6,
selection 31,
checker 5,
and introduced-defect probe 5.
Critic,
panel,
and selection together occupied 86.89 percent of repair wall time.
Selection alone occupied 1,306.585 seconds;
panel occupied 1,043.602 seconds;
critic occupied 742.160 seconds.

Completed streams do not show uniform provider slowdown.
Their medians ranged from 2.034 seconds for DeepSeek Pro to 39.184 seconds for GLM-5.3-Flash.
Qwen,
Minimax,
and GLM had long tails,
but abandoned calls are excluded from completed-stream percentiles.
The trace therefore supports scheduling work that overlaps independent slices;
it does not support reducing grace,
quorum,
review responsibilities,
or model eligibility.

## Completion objective and measurement protocol

The objective is median fresh `Carena0442` entry time below 7,200,000 milliseconds.
At least three isolated successful runs on one unchanged optimized pipeline digest are required.
The unchanged-run band is maximum minus minimum wall time across those first three successes.
No optimization delta smaller than that band is credited.

A successful sample requires all of:

- `TALLY Carena0442 status=SETTLED`
- process exit zero
- exactly one schema-9 artifact
- exactly one published page
- `verify-published` success against same artifact and preparation
- complete source,
  archive,
  and published-page reading with no quality blocker
- `REQUIRED-PROVIDERS synthetic,hyper status=wet` before first model call

`INCOMPLETE`,
`ERROR`,
operator stop,
missing artifact,
missing page,
verification refusal,
or quality finding is censored unfinished work.
None enters median.
Every result reports censored attempt count and reasons beside successful-sample median,
so median is not misread as unconditional completion probability.

Every arm uses exact PR head `a80634a674f94861ea3b7056fba054ca9eab1a2c`,
one built pipeline tree,
one pipeline digest,
one model roster,
one prompt inventory,
one provider requirement,
and a fresh isolated run root.
No slice cache,
consolidation cache,
lane cache,
reading cache,
or prompt-payload file crosses roots.
Only one provider-consuming arm runs at a time.
Process logs,
artifact,
page,
verification,
phase report,
slice-cost report,
and provenance stay under that root or its named companion logs.

Positive controls run before provider measurement:

1.  Phase parser must recover 3,586 seconds,
    66 repair rounds,
    2,994.680 seconds of repair grace,
    and no completion sample from stopped Carena log.
2.  Overlap driver tests must show peak one under overlap 1,
    peak greater than one under candidate overlap,
    and output order unchanged.
3.  Publication verifier must accept intact disposable artifact and page,
    then refuse deliberately altered disposable page.
4.  Required-provider gate must refuse dry fixture before model dispatch and accept wet fixture.

For each successful arm,
partition entry wall time into preparation,
repair,
translate,
lane contest,
consolidation,
publication,
and exact uninstrumented remainder.
Round time to quorum and post-quorum grace are reported separately.
Completed,
unusable,
retried,
abandoned,
and payload-replayed calls are counted separately by model and stage.
The sum of named phases plus remainder must equal entry wall time.

Stop and reset measurement when any of these occurs:

- pipeline source,
prompt,
cache generation,
roster,
provider requirement,
or PR fixture changes
- another provider-consuming process overlaps arm
- required provider is dry before dispatch
- quality or defensive invariant stops entry
- output fails publication verification or complete reading
- phase accounting cannot reconcile to wall time

A transient operational failure does not lower global timeout or quality policy.
Investigate it,
then start a fresh isolated root only after cause is understood.
A reproducible quality blocker stops all performance arms until code fix is verified.

The production overlap fallback remains 1 during objective definition.
Task 31 may compare a scheduling-only candidate against overlap 1 on disposable same-digest arms.
A candidate may ship only if it preserves ordering,
cache eligibility,
exact abort identity,
strict quality,
and warm-run-equivalent twin reuse.

## Objective positive controls

Objective-definition controls passed without provider spend:

- `~/temp/agent/Carena0442-stopped-phase-analysis-20260829.json` records 66 repair rounds and 2,994.680 seconds of grace.
- `~/temp/agent/Carena0442-stopped-latency-analysis-20260829.json` records 3,586 stopped-trace seconds and no terminal completion.
- `~/temp/agent/test-Carena-objective-positive-controls-20260829.log` records seven runner `PASS` lines and zero `FAIL` lines for ordered overlap,
  pass overlap wiring,
  artifact preparation agreement,
  and required-provider dry/wet fixture controls.
- `~/temp/agent/verify-positive-control-Carena-objective-20260829.log` accepts one intact historical artifact/page pair with no missing wording and exact expected length.
- `~/temp/agent/verify-negative-control-Carena-objective-20260829.log` refuses a disposable page altered by 17 characters with `WRONG LENGTH` and zero verified pages.

These controls prove instruments can show expected state transitions.
They do not predict current Carena runtime or output quality.

## Selected completion-path candidate

Task 31 selects explicit slice overlap 4 under built-in 180,000-millisecond grace for Carena measurement.
This is a run-arm setting,
not a production fallback change;
`readPassOverlap` fallback remains 1.
No source implementation is required because all five per-slice drivers already receive one entry-level overlap value,
preserve ordered aggregation,
stop starting work after lowest-position failure,
normalize caller abort,
and share only cache-eligible twins.

The scheduling candidate is evidence-supported across every non-null matched corpus pair already run:

- `keyword233`:
  overlap 4 reduced wall time 19.2 percent and normalized wall time 23.9 percent.
- `Toka_ls`:
  overlap 4 reduced wall time 66.7 percent and normalized wall time 64.9 percent with same 61 unheard voices.
- `Zha_Ke`:
  overlap 4 reduced wall time 71.9 percent and normalized wall time 53.7 percent.
- `Weideriche_`:
  overlap 4 reduced wall time 41.7 percent and normalized wall time 46.6 percent.
- One-slice `ArtsEpiphany` was expected null control because no second slice could overlap.

Older output blockers in those runs do not prove readiness,
but paired timing effect is a scheduling result:
both arms in each pair ran same old digest and same quality mechanism,
and defects were not created by overlap setting.
Current full suite and GFP preserve overlap ordering,
cache eligibility,
exact abort identity,
and strict publication on newer pipeline.

Carena stopped trace makes this candidate relevant:
repair serialized independent work across 22 slices,
spent 84.14 percent of observed repair wall time in post-quorum grace,
and had only five content slices complete after 59 minutes.
Overlap fills one slice's grace wait with useful rounds from other slices without shortening review window.

Task 31 rejects these unmeasured or quality-weakening alternatives:

- reducing 180-second grace
- reducing exact-half participation or corroboration
- removing reviewer,
checker,
confirmation,
or fidelity responsibilities
- lowering model eligibility or using one provider for measured arm
- overlap greater than 4,
which has no matched corpus evidence
- sharing run root,
cache,
or provider capacity between timing arms

Task 32 was canceled after first current-digest arm exposed structural settlement failure.
No additional timing arm is authorized under this architecture.
`TRANSLATION_REPAIR_SLICE_OVERLAP=4` remains result of older matched scheduling pairs,
but is withdrawn as Carena completion-path candidate.

Current-digest run 1 started 18 contested consolidation slices with four-slice overlap.
After 8,735 seconds in consolidation,
only one of first four slices had terminated,
with `exit=failed`;
remaining three were still buying correction,
selection,
review,
and polish rounds.
Run was stopped by user after 16,659 seconds of retained log time,
with no `TALLY`,
artifact,
or fixed page.
Both required providers remained wet.

This censored run proves overlap cannot meet objective while settlement can generate open-ended quality work.
It does not measure completion time or median.
Active work moves to
`doc/planning/translation-repair-pipeline-redesign.md`.
No final quality Gate,
shorter grace,
looser quorum,
or naturalness threshold substitutes for root redesign.

## Evidence to collect before and during fresh current-digest runs

1.  Record terminal `TALLY`,
    process exit,
    entry wall time,
    artifact count,
    page count,
    and publication verification.
2.  Run `slice-cost-report` and `run-timing-report` against each complete log.
3.  Partition wall time into repair,
    translate,
    lane contest,
    consolidation,
    absolute review,
    confirmation,
    and persistence.
4.  Sum post-quorum grace separately from time to quorum.
5.  Count completed,
    unusable,
    retried,
    and abandoned calls per model without treating abandoned-call latency as completed latency.
6.  Record source characters,
    slice count,
    changed-slice count,
    correction count,
    and confirmation count.
7.  Read complete published page when one exists;
    terminal success without acceptable output is not performance success.
8.  Compare only runs with same pipeline digest,
    stage inventory,
    cache warmth,
    corpus input,
    and provider conditions.
9.  Require log line `REQUIRED-PROVIDERS synthetic,hyper status=wet`
    before model traffic for every timing arm.
10. Record prompt-payload replay count separately from provider calls;
    resumed replay is not cold-run generation time.

Completion criterion:
every recorded millisecond belongs to named phase or named uninstrumented remainder,
publication quality outcome sits beside timing result,
and median of at least three isolated fresh same-digest successful runs is below two hours.
Measure unchanged-run band before crediting optimization smaller than that band.

## Comparison rules

Stopped runs overlapped in wall clock and shared provider capacity.
They are partial operational traces,
not clean concurrency experiment or completion sample.
Do not infer speedup,
slowdown,
or provider capacity from their elapsed-time ratio.

Before attributing a change smaller than ordinary variation,
measure repeated unchanged-build runs and establish run-to-run band.
A single run does not characterize model latency,
straggler grace,
or correction demand.

Cache reuse changes question from cold entry completion to resumed entry completion.
Report cold and resumed runs separately.
A command exit code of zero remains insufficient;
`TALLY`,
artifact and page counts,
and `verify-published` decide terminal state.

## Questions for completion-path design

- Which phase owns largest measured share of each entry wall time?
- How much post-quorum grace bought usable evidence that affected outcome?
- How much grace ended with abandoned voice or duplicated decision evidence?
- Does serial slice ordering dominate entry time,
or do consolidation corrections dominate?
- Which observability gaps leave wall time unattributed?
- Can scheduling change reduce wall time without changing question,
quorum,
fidelity,
naturalness,
or failure semantics?
- Which proposed change can be tested as matched disposable arm before production adoption?

No new validation launches before partial Carena log is phase-attributed,
correctness changes are committed,
and matched optimization plan preserves strict quality.
