# Comparing replacement pipeline interfaces

## Status

This compares design candidates under corrected requirements in
`doc/planning/translation-repair-pipeline-redesign.md`.
It is a proposal,
not architecture decision.
Measured motivation is recorded in
`doc/audit/translation-repair-carena-stopped-run.md`.

## Shared interface contract

All candidates expose one deep entry module:

```ts
export type TranslationEntryModule = {
  /**
   * @throws ProductionUnavailableError when every finite producer is unusable before first adoption.
   * @throws PublicationUnavailableError when assembly, atomic write, or readback fails.
   */
  readonly produce: (
    input: StartEntry | RestartEntry,
  ) => Promise<CompletedEntry>;
};
```

`CompletedEntry` always carries one good complete document,
sealed audit,
and publication receipt.
It is only normal translation outcome.
There is no `failed-quality`,
`refused`,
`error-without-output`,
`incomplete-terminal`,
or user-visible suspended translation variant.
Caller abort throws exact `signal.reason` rather than converting it to outcome.

Every start operation persists finite work manifest before provider contact.
Every node records canonical model and prompt digest plus durable payload state.
Provider adapter refuses payload not named in manifest.
No reply,
finding,
text change,
nonce,
or round can add node.
Model nodes are preparation-evidence or candidate-producer nodes.
Preparation-evidence node may produce brief or specification before authorship;
its unusable response contributes nothing and cannot withhold producer work.
First adopted candidate producer owns full concrete quality contract:
fidelity,
completeness,
identity,
grammar,
clear references,
consistent tense,
paragraph relations,
and register.
Later producers are targeted improvements,
not required rescue for deficient baseline.
Every post-preparation model node is candidate producer and produces complete candidate or has no effect.
After first complete candidate exists,
node timeout,
refusal,
or unusable output preserves prior candidate byte-for-byte and execution continues.
Before first candidate exists,
manifest names finite fallback producers that may each be tried once.
If all planned producers are exhausted by transport failure or unusable response,
`produce` throws bounded `ProductionUnavailableError` with exhausted node identities.
It does not suspend,
automatically requeue,
or publish unchecked archive fallback.
`ProductionUnavailableError` concerns exhausted candidate producers only;
failed preparation-evidence node never causes it.

Restart requires same manifest digest and checkpoint and executes pending nodes only.
Completed,
failed,
unusable,
aborted,
or indeterminate nodes are spent.
Recorded payload from indeterminate transmission may be reused,
but canonical prompt may never be resent.
Caller abort bypasses fallback immediately and throws exact `signal.reason`.
Checkpoint is internal crash and cancellation evidence,
not user-visible translation state.
Each node digest binds exact source,
archive,
brief,
prior candidate bytes or explicit absence marker,
role,
and response contract.
Assembly,
atomic write,
or readback failure after first adoption throws bounded `PublicationUnavailableError`.
It cannot suspend,
automatically requeue,
publish partial bytes,
or publish archive fallback.
Current self-replenishing path in
`package/module/translation-repair/src/corpus-run/entry-attempt-queue.ts` (moved on 2026-10-18 to
`package/translation-repair/slice-cache/src/corpus-run/entry-attempt-queue.ts`)
must be replaced rather than adapted.

Shared ports:

- `ModelWorkPort`:
  true-external provider work,
  with routed production adapter and scripted test adapter
- `RunJournalPort`:
  local-substitutable plan,
  payload,
  audit,
  checkpoint,
  and artifact journal
- `PublicationPort`:
  local-substitutable atomic write and readback verification

Manifest planning follows provider preflight.
For each supported single-provider mode,
every producer,
fallback,
renderer,
specification author,
brief author,
and editor resolves through that wet provider.
Cross-provider response is never correctness dependency.

Parsing,
source spans,
structure,
coverage,
identity authority,
media evidence,
deduplication,
and deterministic validation stay in-process.

Quality is represented by concrete defects,
never score or historical naturalness label:

```ts
export type EditorialDefect = {
  readonly kind:
    | 'wrong-meaning'
    | 'omission'
    | 'addition'
    | 'identity-change'
    | 'syntax-damage'
    | 'grammar'
    | 'unclear-reference'
    | 'tense-inconsistency'
    | 'paragraph-relation'
    | 'register-mismatch';
  readonly sourceAnchor?: SourceAnchor;
  readonly targetAnchor: TargetAnchor;
  readonly explanation: string;
};
```

Subjective finding is editorial evidence.
It is not numeric measurement and does not become objective because several models repeat it.

## Final disposition, 2026-09-01

Candidates A through M have no production survivor.
Candidate M failed its single pinned-Carena calibration,
and implementation stopped.
Read the [timestamped failure report](../audit/translation-repair-redesign-failure-2026-09-01.md) before using historical candidate details.

## Candidate detail files

- [Candidates A to D](translation-repair-interface-candidates-a-d.md):
  serial producers,
  specification compiler,
  editorial briefs,
  and immutable-shell iterations.
- [Candidates E to G](translation-repair-interface-candidates-e-g.md):
  conditional adoption,
  donor design gate,
  and realization ledger.
- [Candidates H to J](translation-repair-interface-candidates-h-j.md):
  bounded verdicts,
  candidate-scoped ballots,
  and rejected Kimi expansion.
- [Candidates K to M](translation-repair-interface-candidates-k-m.md):
  readable review units,
  lean realization,
  and risk-split challengers.

No detail file is authorization to retry a prompt or implement a successor.
