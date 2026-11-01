# DeepSeek V4.1 Flash admission

The owner states:
“DeepSeek V4.1 Flash is now up and approved on both OpenRouter and Hyper.
Do with this information whatever is appropriate.”

Task 30 verifies and integrates serving,
identity and catalog data.
Task 31 has verified judge admission under the existing rule.
Tasks 35 and 36 separately measure writing and image reading.
Task 27's archive-evidence implementation is paused,
not complete.
No full-entry pass is running.

## Measured availability

Authenticated catalog reads on 2026-09-11 returned HTTP 200 on both providers.
`~/temp/agent/deepseek-v41-flash-discovery-20260911.json`
retains the metadata without credentials.

- OpenRouter:
  `deepseek/deepseek-v4.1-flash`.
  Canonical slug `deepseek/deepseek-v4.1-flash-20260910`.
  Reported context 1048576 tokens,
  maximum completion 384000 tokens,
  text and image input.
  Base listed USD rates are 0.3 input and 1.2 completion per million,
  with time-dependent discounted overrides.
  Reported wire cost remains authoritative over a fixed catalog estimate.
- Hyper:
  `deepseek-v4.1-flash`.
  Reported context 1048576 tokens,
  maximum output 26214 tokens,
  vision capability.
  Its API pricing fields were initially kept unpriced until the credit unit was verified.
  The completed conversion is recorded in “Credit quote verification.”

The raw-fetch protocol probe uses existing compiled builders and stream parsers,
not provider SDKs.
It asks for an invented object containing `animal: cat` and `count: 7`.
It supplies the existing unmeasured-model pooled p99 cap of 13082,
not V4 Flash 0731's cap,
and no thinking,
budget or reasoning-effort parameter.

- Hyper:
  streamed forced-tool request succeeded,
  `tool_use` finish,
  exact object,
  708 prompt and 59 completion tokens.
- OpenRouter:
  first request returned HTTP 429 with no reported usage retained.
  One such response is not treated as a stable availability limit.
  The same-input recheck succeeded through DeepInfra:
  `stop` finish,
  exact object,
  52 prompt and 120 completion tokens,
  0.0000824 USD reported cost.

Reports:
`deepseek-v41-flash-protocol-20260911.json`
and `deepseek-v41-flash-protocol-r2-20260911.json`
under `~/temp/agent/`.
Raw traffic is transient;
reports retain decoded answers,
usage,
status and serving identity only.

These calls establish text protocol compatibility,
not translation quality or image-reading performance.

## Integration boundaries

- Use one new roster identity shared by Hyper and OpenRouter.
  Neither provider creates a second voter for the same model.
- Preserve V4 Flash 0731 as a distinct existing version unless separate evidence or owner instruction retires it.
- Do not inherit that version's writer exclusion,
  calibration,
  completion distribution or endpoint exclusions.
- Use the existing pooled unmeasured cap until sufficient own-model usage is measured.
- Preserve provider order and all current self-vote,
  author-defense and quorum rules.
- Do not infer endpoint exclusion from the initial HTTP 429.
- Keep judge,
  writer and reader eligibility separate.
  Gemma 4 31B is an existing example of a measured reader without a judge seat;
  deriving readers from the judge roster would incorrectly remove it.
- Catalog approval is not measured role admission.
  New-model calibration holds must prevent accidental seating through catalog-derived arrays.

## Implemented serving boundary

`dc1b1d943` registers the model and independent admission holds.
Both actual clients from frozen `.frozen-dist-dc1b1d943` returned the exact object,
without transport interception:

- Hyper:
  708 prompt and 56 completion tokens.
- OpenRouter through DeepInfra:
  348 prompt and 28 completion tokens,
  0.0000864 USD reported cost.

`deepseek-v41-clients-20260911.json`
and `deepseek-v41-clients-20260911.log`
retain the result and spend lines.
The raw-protocol calls' ledger was reconstructed separately from their retained reported outcomes.
The initial HTTP 429 remains an attempt with unreported usage,
not an invented zero charge.
Spend and daily helpers ran.

`cc7ccfc00` renames the historical Hyper-only API vocabulary to its actual meaning:
Hyper-origin identities and Synthetic-counterpart projections.
Canonical model IDs are unchanged.
The new API names cannot be read as a claim that OpenRouter lacks a route.

The routing fixture uses actual compiled provider clients with disposable transports.
It verifies Hyper preference,
OpenRouter fallback spelling,
unreachable versus all-dry refusals,
one dispatch,
the 13082-token body cap and absence of reasoning-control keys.

`deepseek-v41-role-boundary-20260911.json`
compares every exported production role array and its order,
`RUN_MODELS`,
and `judgeSeatsFor` across all 16 boolean budget states
against frozen `b9d3b2ea0`.
All are identical.
Its positive control requires that only the newly approved catalog identity was added.
Direct `ROSTER_MODEL_IDS` consumers were inspected:
production admission is confined to the run-roster and reader projections;
other consumers validate artifact identities or calibration candidate arguments.

The cap basis is not inherited from the predecessor:
`cap-measure-20260909.txt` records 142437 completed samples with pooled p99 13082.
The abandoned-stream fallback 137 is the median of the recorded model medians
132,
137,
137,
297 and 386,
not a pooled-stream percentile or a V4.1 measurement.
`deepseek-v41-fallback-basis-20260911.out`
records that distinction.

## Credit quote verification

Hyper's [model documentation][hyper-models]
and [FAQ][hyper-faq]
define one hypercredit as USD 0.05.
Its [model-list API][hyper-list]
defines per-million-token pricing.
The live 0.3/1.2 USD quote therefore yields 6/24 credits per million,
with cache-create 0 and cache-hit 0.6.
These are quoted-rate estimates,
not measured account debits.

Every prior table row remained in the live catalog.
`027f59dc3` refreshes the whole 34-model price snapshot to 2026-09-11,
rather than assigning an old date to a newly introduced model.
`hyper-price-snapshot-verification-20260911.json`
checks every rate field against the retained API capture
and proves that pricing rows for Inkling and Kimi K2 Thinking do not approve their serving identities.
Unknown models still report `unpriced`.
The [pricing troubleshooting record][pricing-record]
contains the source trace and rejected interpretations.

Completed OpenRouter calls retain their reported costs or explicit absence.
Fixed catalog rates are used only for abandoned-call estimates.
The difference between a serving endpoint's bill and the aggregate catalog quote
is not attributed to scheduled discounts without endpoint evidence.

## Serving implementation checkpoint

The serving checkpoint used frozen build:
`package/module/translation-repair/node_modules/.frozen-dist-24b1cbdb7`.
Build and type checking pass.
`deepseek-v41-lint-r3-20260911.out`
reports zero warnings/errors.
`deepseek-v41-full-unit-r3-20260911.out`
ends `unit exit 0` at line 9164.

A native `judge-fidelity-probe --cap 0 --candidates deepseek-v4.1-flash --candidates-alone`
invocation in disposable runs directory `v41-preflight.gJPEy1Ao`
prints the candidate identity and persists zero rows without model generation.
This proves selection reachability,
not quality.
The producer CLI uses the same candidate resolver;
its count parser intentionally rejects zero,
so it was not misrepresented as supporting that preflight mode.

The budget reading at 2026-09-11T01:35:58Z reports all providers wet:
Hyper balance 145,
Bedrock 185.38 USD,
OpenRouter 267.73 USD.
Refresh before extended paid calibration.

## Provider identity caveat

DeepSeek's [release announcement][release]
states that its own undated V4 Flash aliases now route to V4.1 Flash,
and that its undated V4 Pro API name switches at 04:00 UTC on 2026-09-14.
This does not establish a remap of our gateways' dated `0731` and `0813` IDs.
No such gateway remap was verified,
and no old identity was silently retired or merged.
Provider identity must remain part of the calibration evidence.

## Verified judge admission

The paid source-reviewed comparison on frozen `993583ad5` returned all 140 model/question cells.
All fourteen distinct questions heard every one of the nine peers.
V4.1 selected the reviewed reference fourteen times,
with no damaged pick,
decline or missing ballot.
Peer median clean count was fourteen;
maximum peer damaged count was one.
All actual ballot reasons were read,
and an independent review confirmed admission under the unchanged criterion.

`83e632127` removes only the judge/general text-roster hold.
The writer and reader holds remain.
Hyper served all fourteen V4.1 calls and returned that version's identity.
The predecessor's responses retained its distinct 0731 identity.
No gateway remap is inferred.

Post-change build,
types,
zero-warning lint and the full unit suite pass.
`v41-judge-seat-final-unit-20260911.out` ends `unit exit 0` at line 9164.
Checked runtime is frozen at `node_modules/.frozen-dist-45e64e411` inside the package.
`v41-judge-seat-final-boundary-20260911.json`
compares every exported role array,
the nested repair configuration and all sixteen budget states with frozen `993583ad5`.
It proves one added judge identity,
unchanged existing order,
unchanged writer/reader arrays,
and unchanged provider order,
completion caps and quorum formulas.

Full reading and cost provenance are in
[the seating decision](../decision/translation-repair-roster-seating-2026-09-01.md).
The run is `~/temp/agent/v41-reviewed-run-20260911-vbOoMQ`.
No full-entry readiness follows from this result.

## Remaining role calibration

Task 35 runs the existing 40-round writer calibration before releasing its writing hold.
Task 36 exercises image input and corroboration before releasing its reader hold.
Both must record actual serving context and participation.
Keep the source corpus pinned and do not launch a whole-corpus development run.

[hyper-models]: https://hyper.charm.land/docs/models.html
[hyper-faq]: https://hyper.charm.land/faq
[hyper-list]: https://hyper.charm.land/docs/api/list-models.html
[pricing-record]: ../troubleshooting/charm-hyper-credit-price-snapshots.md
[release]: https://api-docs.deepseek.com/news/news260910
