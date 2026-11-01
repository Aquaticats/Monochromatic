# Writer calibration unit scope: prepared-parent integration seams and the normalized pool prototype

The extraction of block pairing preparation for a bounded parent pool,
the pinned prepared-scope control on the windward parent,
the parity checkpoint,
and the normalized pool prototype with its qualification contract for the planner.

Part of [Writer calibration unit scope](translation-repair-writer-unit-scope-2026-09-11.md),
which lists every file of this record.

## Integration seams still to build

`prepareDocumentPairWithRoster`
in `package/module/translation-repair/src/prepare-with-pairing.ts` (moved on 2026-10-18 to
`package/translation-repair/archive/src/prepare-with-pairing.ts`) owns the block-round loop,
media-adjacent target normalization,
footnote-definition separation and pure `prepareDocumentPair` handoff.
A bounded parent-pool consumer needs the same block-round machinery without buying unrelated section rounds.
That operation is now extracted as `prepareBlockPairing`,
with `PreparedBlockPairing` distinguishing implicit,
empty,
paired and fallback states.
Only questioned states carry cached or queried acquisition evidence.
`prepareDocumentPairWithRoster` now delegates each indexed parent to it,
retaining the pure preparation map and existing aggregate result shape.
At `97599f638`,
build,
manual type checking,
zero-warning lint and the full unit suite pass.
`writer-parent-preparation-final-unit-20260911.out` ends at line 9237 with `unit exit 0`.
The frozen runtime is `.frozen-dist-97599f638` under the package's `node_modules`,
digest `sha256-tree-v1:4a9ba31bddbc9cbf190830cf17625212f10d12a9c9c40263cb95d14a165e620d`.
`readBlockPairingOutcomes` is shared between live pairing and recipe replay.
At `6f496fd0c`,
its build,
types,
zero-warning lint and full unit suite pass;
`writer-pairing-reader-final-unit-20260911.out` ends at line 9187 with `unit exit 0`.
Independent review then identified the separate identity preconditions recorded in
`Pairing identity verification`.

Production also changes the actual writer surface after subdivision.
`settleTranslateSlice` in `package/module/translation-repair/src/translate-slice.ts` (moved on 2026-10-18 to
`package/translation-repair/translate-lane/src/translate-slice.ts`):

- Uses `splitTargetOnlyRun` before the whole writing/judging stage.
- Derives absent/present mode from the target chunk variant,
  not its text length.
- Forwards production identity context,
  neighboring context,
  picture context,
  syntax and child-level line-structure governance.
- Returns `stageResult`,
  including slate and ballots,
  beside publication disposition.

The benchmark needs the same pre-stage surface,
not a manually reproduced subset.
A later independent review correctly separated that requirement from changing the benchmark's operation:
keep the native `runTranslateStage` call and existing scoring,
and extract a shared pure pre-stage input projection from `settleTranslateSlice` instead.
Do not add its publication-disposition guards to calibration.
Those guards do not replace input review.
Picture evidence and footnote relabeling must remain explicit in the preparation plan;
a prepared range alone does not supply unseen image text or reconcile differing footnote labels.
Do not substitute the entire `preparePassEntry` shell without further design:
it also invokes archive prose repair.
Allowing a benchmark peer to rewrite the incumbent during preparation would hide authorship
behind the incumbent producer label and undermine self-vote exclusions.
Preparation here buys correspondence evidence and may apply the existing pure footnote relabel;
it must not buy uncredited benchmark-baseline prose.
Native `producer-calibrate.ts` is still unchanged and unsafe to launch on the old sample.

## Pinned prepared-scope control

The provider-free control on frozen `06862482e` supplies a manually source-reviewed pairing recipe
for the known windward parent,
then invokes actual `prepareDocumentPair` and `validateTranslatedSlice`.
This is an isolated mechanism control,
not automatic acquisition or admission gold.

- Heading source 2069 to 2079 pairs with target 11101 to 11126 and passes the unchanged validator.
- Body source 2081 to 2419 pairs with target 11128 to 12232,
  including all of its incumbent body paragraphs,
  and passes the unchanged validator.
- Every source and target parent node appears exactly once across the prepared outputs.
- The old heading task still rejects the heading-only candidate.
- The independent `lintong#0` heading-only control passes.

The retained incumbent body is only a structural-contract witness,
not a claim of grammatical or factual perfection.
`prepared-scope-control.json` contains the exact outcome and membership arrays.
The bounded container used no network and made no model calls.
Its log is `writer-prepared-scope-control-20260911.out`.

Frozen runtime:
`node_modules/.frozen-dist-06862482e` under the translation package,
digest `sha256-tree-v1:a31c12b618bb4fd76a6f9c00af0ccab720364648a39907b6de62e0f32653c1e7`.
This snapshot adds pairing-seat evidence only;
it does not contain a safe new writer sampler.

## Current implementation checkpoint

`prepare-block-pairing.ts`,
`prepare-block-pairing-finish.ts` and `prepare-block-pairing-model.ts`
live under `package/module/translation-repair/src/` (moved on 2026-10-18 to `package/translation-repair/*/src/`).
The before/after comparison verifies the old cache key,
cold/warm media normalization,
cache eligibility,
definition separation and scorer fallback across fourteen invented fixture cases.
Frozen `06862482e` and `97599f638` match complete cold/warm results,
including Sets,
exact request bodies and serialized cache bytes.
Positive controls detect changed wire bytes,
cache bytes and a line-structure Set.
`~/temp/agent/parent-preparation-parity-20260911.json` records the comparison.
Only mocked transports ran;
the abort case records transport callback invocations,
not real provider traffic.
New tests cover explicit acquisition,
cache reuse without invented outcomes,
empty and singleton fast paths,
queried and cached fallback,
unclaimed-target cache refusal,
definition crossing,
persistence failure and cancellation.

The first test run exposed two incorrect fixture expectations:
`declinedTargetBlocks` in `package/module/translation-repair/src/declined-target-runs.ts` (moved on 2026-10-18 to
`package/translation-repair/slice-reading/src/declined-target-runs.ts`)
deliberately declines nothing while an original remains unplaced,
and a usable empty agreement can be cached even though its execution state is fallback.
The tests now preserve those existing policies and use a fully placed source plus an extra target block
for the actual unclaimed-target cache control.
This is why cache eligibility alone must never qualify a calibration parent.
No model transport beyond test mocks is used.
The native producer CLI and bounded preparation planner remain unchanged.

## Normalized pool prototype and next qualification contract

The provider-free pool prototype now uses `readCorpusFile` followed by `passArchiveText`.
It contains 283 deterministic parent pairs,
not the raw census's 275.
The proposed forty-parent pool spans 32 entries,
with 20591 source and 55002 incumbent characters.
Its ordered population digest is
`dd26c62568689c68c2320aaaa96dd43f8f49a138acd0079a010cfa50a95dea71`.
The private `preparation-pool-prototype.json` and `preparation-pool-reading/` directory
retain full entry context.
They remain a prototype,
not approval for paid calls.

Parent scope reading has begun in `preparation-pool-scope-reading.json`.
The selected `ArtsEpiphany` parent is identical empty layout markup.
Such a task must be declared a preservation control,
not claimed as positive writing-quality evidence.
Do not drop it after seeing model outcomes.
`writer-preservation-standing-20260911.json` verifies the conditional accounting:
when every roster writer matches the incumbent,
all twelve receive candidate exposure but zero disinterested ballots and votes.
The sole-stake positive control receives eleven disinterested votes.
This does not predict actual candidate provenance in the future run.
The final prepared-unit draw still has to be constructed and reviewed.

Planner qualification remains separate from production pairing behavior:

- Use current queried seat evidence,
  not historical cache records lacking ballots.
- Require the configured quorum number of usable parsed voices for calibration input acceptance.
  This is a planner gate;
  the production stage still gathers according to its existing heard-voice policy.
- Preserve the two-distinct-identity per-relation threshold.
- Reject fallback instead of dropping its parent after the fact.
- Trust target declines only under the existing all-originals-placed condition.
  Preserve legitimate source insertions rather than globally forbidding unplaced originals.
- Trace every target block entering a task to an endorsed relation or an identified deterministic media claim.
- Derive parent nodes and target containers from the same normalized parse.
- Reparse and rebuild after deterministic footnote relabeling changes target bytes.
- Persist current outcomes in a separate evidence journal;
  the old pairing cache does not bind the electorate or retain its ballots.

Mixed-parent aggregate parity now passes,
as do durable no-network acquisition-through-slicing,
media-after and exact cache-byte controls.
`translateSliceInput` is implemented and used by production settlement without changing the native benchmark operation.
At `895843508`,
build,
types,
zero-warning lint and the full suite pass at line 9244.
Eight before/after settlement cases preserve exact stage inputs and complete results,
with positive controls.
The native benchmark still has no reviewed-plan gate.

Task 39's [archive footnote remedy](translation-repair-archive-footnote-collisions-2026-09-11.md)
is verified at `086fd9f6e`.
The current pinned Y1 consumer retains the unmatched note under a fresh label,
keeps all references/definitions and front matter,
and rebuilds current slice metadata.
Strict syntax,
stale-map and range checks,
protected-original vetoes,
real pass/cache lifecycle tests,
full unit/lint and the updated mutation proof all pass.
This does not qualify any model correspondence or approve the writer plan.

The reading journal covers eighteen of the initial prototype's thirty-two entries.
Task 38 must first rebuild the policy-eligible population,
applying whole-page original decline and sealed-span rules,
then revalidate those reading anchors and finish the required readings.
Finish current-outcome qualification,
actual prompt/transport bounds and the distinct reviewed-plan execution gate before any paid launch.
No paid preparation,
writer or V4.1 image-reading call has occurred.
