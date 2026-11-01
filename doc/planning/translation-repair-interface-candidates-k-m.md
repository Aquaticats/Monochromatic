# Translation repair interfaces: Candidates K to M

Part of the [interface comparison index](translation-repair-interface-comparison.md).

## Contents

This record is kept as one file per subject or span,
each of at most 1,000 lines and named for what it holds.
New text goes into the file whose subject it belongs to;
a file that would pass the limit gives up a subject to a new file,
named for that subject and listed here.

- [Translation repair interfaces K to M: Candidate K, three-family authors with conjunctive review units](translation-repair-interface-candidates-k-m-candidate-k-three-family-authors-and-review-units.md):
  Candidate K's plan,
  its static graph,
  review-unit compiler,
  ballot algebra,
  alternatives and acceptance controls,
  its structural prototype status,
  and the calibration that rejected it as an author protocol.
- [Translation repair interfaces K to M: Candidate L, lean realization with readable verification](translation-repair-interface-candidates-k-m-candidate-l-lean-realization-and-readable-verification.md):
  Candidate L's plan,
  its lean author contract and readable verification contract,
  alternatives and acceptance controls,
  its prototype result,
  and the calibration that rejected it after an actor reversal in the only admitted candidate.
- [Translation repair interfaces K to M: Candidate M, risk-attested authors and role-split challengers](translation-repair-interface-candidates-k-m-candidate-m-risk-attested-authors-and-role-split-challengers.md):
  Candidate M's plan,
  its author and challenger contracts,
  alternatives and acceptance controls,
  its implementation evidence,
  calibration preflight,
  and the calibration result that ended implementation.

## Current stop condition

Historical design evidence only.
Candidate M failed on 2026-09-01.
No candidate prompt may be retried,
and no successor implementation is authorized by this file.

## Required lifecycle migration

Replacement must remove current terminal-looking outcomes in
`package/module/translation-repair/src/corpus-run/pass-entry-contract.ts` (moved on 2026-10-18 to
`package/translation-repair/slice-cache/src/corpus-run/pass-entry-contract.ts`).
`resumable-failure` and `stopped` cannot remain normal translation results under new run contract.
`package/module/translation-repair/src/corpus-run/pass-entry.ts` (moved on 2026-10-18 to
`package/translation-repair/corpus-pass/src/corpus-run/pass-entry.ts`)
must return published output after finite producer graph,
not `TALLY status=ERROR`,
`INCOMPLETE`,
or visible suspended translation because ancillary model work failed.

`package/module/translation-repair/src/corpus-run/entry-attempt-queue.ts` (moved on 2026-10-18 to
`package/translation-repair/slice-cache/src/corpus-run/entry-attempt-queue.ts`)
must not infer new work from cache growth or append same entry to queue.
Deterministic restart may reuse completed nodes from same manifest after crash or exact caller cancellation;
it cannot create another manifest automatically.
Publication assertions remain candidate-adoption integrity implementation,
not terminal quality refusal.

## Prototype acceptance evidence

Before selecting any replacement for production:

- static manifest proves declared finite payload ceiling
- positive controls cover seeded omission,
wrong meaning,
identity change,
syntax damage,
grammar,
unclear reference,
tense inconsistency,
paragraph relation,
and register mismatch
- negative controls use previously read acceptable text and prove responsibility-specific producers preserve it
- stopped Carena inputs fit each whole-document producer envelope
- either one-provider adapter can supply complete producer and fallback roster
- abort preserves exact identity
- deterministic restart sends no duplicate payload
- each unusable later producer preserves prior complete candidate and execution reaches publication
- first-producer failure exercises statically named fallback producer without adding manifest work
- deterministic publication guards still GFP-fail when removed
- complete actual Carena output from selected candidate is read by human and independent reviewer;
  artifact or tally alone is not comparison evidence
- recurring output defect changes brief,
producer responsibility,
or adoption contract;
it never adds Gate or retry loop
