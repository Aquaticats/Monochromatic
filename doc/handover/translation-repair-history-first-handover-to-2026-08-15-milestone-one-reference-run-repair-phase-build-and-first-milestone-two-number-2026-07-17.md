# Translation repair history, first handover document: milestone one reference run, the repair-phase build and the first milestone two number with the retry-to-quorum directive, 2026-07-17

The 93-minute reference run that closed milestone one at an ensemble recall of 0.981,
the notes on deadlines,
concurrency and the driver environment,
the repair stages built and checked live on DarlinChit,
and the first milestone two number with the directive to retry lost voices up to quorum.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Milestone one reference run, per-call deadlines and pack-scaled concurrency

1. MILESTONE ONE NUMBER IS IN (2026-07-17,
   93-minute reference run,
   pre-budget code):
   `ensembleRecall` 0.981 (53 of 54 seeds) over 18
   entries spanning 714 to 5_826 chars,
   all seven models,
   one stream per
   model.
   Every one of the 126 calls ended `ok` (schemaOkRate 1.0 for
   every model,
   zero refusals);
   the retry layer recovered all four
   deadline forfeits (GLM-5.2,
   Flash,
   Qwen,
   MiniMax each once).
   Per-model seeded recall:
   GLM-5.2 0.889,
   Qwen 0.889,
   Kimi 0.815,
   gpt-oss 0.741,
   Nemotron 0.722,
   Flash 0.426,
   MiniMax 0.407;
   the ensemble union is the design working as intended.
   The single ensemble miss,
   analyzed at repair-phase kickoff
   (2026-07-17):
   wangzihao980 `seed/omission-1`,
   found by nobody.
   Sharper than first recorded:
   on that entry six models found ONLY
   `omission-2` and gpt-oss-120b found ONLY `omission-0` (its sole hit).
   Structural facts (no content):
   the missed needle is a 73-char
   interior sentence deleted from MID-paragraph (64 chars into its
   line);
   both found seeds were list items whose deletion left a
   countable zh/en structure asymmetry.
   Two consequences adopted:
   interior-sentence omissions are the critic's weak class (future
   prompt polish),
   and a real defect can arrive with exactly ONE
   proposer,
   so adjudication judges claims on evidence and never gates
   on corroboration count (confirms the settled panel design).
   Unresolved-reason distribution across the run (233 rejected claims):
   quote-outside-blocks source 90,
   quote-not-found source 39 / target
   33,
   ambiguous-quote 39,
   empty-quote 27,
   two model category typos;
   source-side anchoring is future critic polish,
   not a repair blocker.
   Quota after the whole run:
   2747.5/2750
   (regeneration outpaced consumption).
   MiniMax quirk:
   on 7 of 18 entries it returned near-empty reports
   (5-6 completion tokens,
   zero claims) yet valid JSON;
   on others it
   produced 10-19 resolved claims.
   Ensemble absorbs it;
   noted for
   scheduler weighting later.
   Future runs are 25-minute-budget runs via the updated
   `broadened-benchmark.ts` (shuffled samples,
   coverage-reporting
   scorecard).
2. Per-call deadlines are DONE (commit `18a8e95ca`):
   `armCallDeadline` in
   `benchmark.ts` arms a plain-timer-driven `AbortController` per call and
   forwards caller aborts through a listener;
   disposal (`using`) clears both.
   Never compose `AbortSignal.any` with an `AbortSignal.timeout` source on
   Node 26.5.0:
   the dependent signal never aborts (isolated repro confirmed;
   single-source `AbortSignal.any([signal,],)` works fine,
   verified by probe).
3. Pack-scaled concurrency is DONE:
   `createSyntheticClient` takes
   `perModelConcurrency` (default 1;
   provider grants one concurrent request
   per model per subscribed pack),
   and benchmark entries run in parallel.
   The user bought 4 more packs (joining a 1.5-pack-equivalent founder's
   pack);
   live quota ceiling is 2750 (5.5 pack-equivalents),
   the driver floors to `perModelConcurrency: 5`.
4. Driver env:
   the API key resolves only through mise sops,
   so run the
   scratchpad driver as
   `cd <worktree> && mise exec -- node <scratchpad>/run-benchmark.ts`;
   a bare `node` invocation dies on the missing env var.
5. The `prefer-readonly-parameter-types` idiom learned for opaque DOM calls:
   the `@mutates` contract must sit on the function DIRECTLY containing the
   calls,
   name every flagged boundary verbatim on an unbroken line
   (`signal.addEventListener`,
   `signal.removeEventListener`,
   `DOM commit 5796f716 AbortController abort steps retain reason`),
   and the parameter must be `ForeignBorrowed`-marked (the
   `fetchTransport`/`chatJson` pattern);
   callers inherit the documented
   uncertainty without re-documenting.

## Repair-phase stages built and checked live

- REPAIR PHASE PLAN (tasks 11 to 17,
  dependency-ordered;
  user confirmed
  repair phase as the next scope 2026-07-17):
  11 claim aggregation (cross-model dedupe proposing clusters,
  deterministic span-overlap plus category-family compatibility,
  merges only PROPOSED),
  12 adjudication panel (provenance-blind,
  vote states
  supported/unsupported/ambiguous/source-defect/abstain,
  quorum,
  evidence-based never corroboration-gated,
  disposes proposed merges),
  DONE (commit `ebfe447a2`) with live boundary check on seeded
  DarlinChit (2026-07-17):
  7 critics -> 58 validated claims -> 15
  clusters (one 28-member hot-region mega-cluster,
  a refinement
  candidate) -> 5 of 7 panelists returned complete 58-verdict ballots
  with ZERO wire findings (integer-reference ballots work;
  Nemotron
  and gpt-oss forfeited the 5-minute deadline,
  driver had no retry)
  -> tally 42 issues (39 accepted,
  2 rejected,
  1 needs-human),
  BOTH
  planted seeds inside accepted issues,
  net quota cost ~0.
  chatJson success discriminant is `kind === 'ok'` (a first driver
  run wasted 5.6 quota units checking `'value'`),
  13 patch-operation model plus editable envelopes plus deterministic
  apply guards,
  DONE (commit `15450a437`:
  envelopes merge overlapping
  and touching accepted target spans,
  editors replace whole envelope
  contents against echoed base hashes,
  gate rejects unknown/duplicate/
  stale/drifted/unchanged as data,
  overlap throws),
  14 editor stage,
  DONE (commit `249dafad1`:
  numbered regions with
  current text and context,
  integer-referenced wire,
  resolver binds
  numbers to envelopes so models never echo hashes).
  Live editor check
  on the saved DarlinChit adjudication (2026-07-17):
  39 accepted
  issues merged to 10 envelopes,
  GLM-5.2 answered in 7 s,
  10/10 edits
  resolved and applied with zero findings and zero rejections;
  both
  deleted needles restored at about two-thirds content-word overlap
  (editor re-translates from zh,
  so byte-exact restoration is not
  expected).
  DESIGN FACT:
  the quote-based critic wire anchors
  omissions via the adjacent sentence,
  so omission repairs flow
  through replacement envelopes,
  not zero-width insertions;
  zero-width
  machinery stays for other producers.
  15 resolution check plus no-regression gate plus lexicographic
  candidate selection,
  DONE (commit `015b67b89`:
  checkers vote
  fixed/not-fixed/worse per issue,
  strict fixed majority resolves,
  worse majority counts as regression;
  selection order integrity >
  high-severity resolution > regressions > total resolution >
  preservation,
  unchanged candidate always competes and wins perfect
  ties).
  Live check (2026-07-17):
  3 checkers,
  39/39 verdicts each,
  zero findings,
  25 s;
  38/39 resolved (13 high-severity),
  0
  regressed,
  patched candidate parsed clean,
  selection picked the
  repaired candidate over unchanged.
  Note checker leniency mirrors
  panel leniency;
  canary calibration remains the designed control,
  16 `repairTranslation` end-to-end,
  DONE (commit `a4d384dcb`:
  `repair-translation.ts` batch driver over `repair-chunk.ts` and the
  stage runners in `repair-stages.ts`/`repair-edit-stages.ts`;
  `stage-call.ts` turns lost voices into ensemble degradation,
  caller
  aborts always propagate;
  two wire-level critical non-translation
  votes block repair;
  changed chunks splice back descending;
  unit-tested end to end over a stub client scripted per
  response-format schema name).
  LIVE end-to-end on seeded DarlinChit
  (2026-07-17,
  523 s,
  ~8.5 quota units):
  status `repaired`,
  critics
  5/7 heard (Nemotron and gpt-oss forfeited 4-minute deadlines,
  gracefully absorbed),
  41 claims -> 31 issues (29 accepted,
  2
  rejected),
  editor 10/10 applied zero rejections,
  checkers 3/3 with
  29/29 accepted issues resolved,
  BOTH deleted needles restored
  (11/18 and 10/15 distinctive words returned),
  17 milestone-two benchmark,
  DONE (commit `765e3059c`,
  `repair-benchmark.ts`):
  restoration grades on the distinctive
  vocabulary the deletion removed (words surviving elsewhere in the
  seeded text prove nothing;
  half returning marks restored,
  `RESTORATION_WORD_THRESHOLD`);
  `runRepairBenchmark` budget-gates
  entries sequentially with an injectable repair seam for tests;
  `computeRepairScorecard` reports `seededRepairRate` over measurable
  seeds plus coverage.

## First milestone two number and the retry-to-quorum directive

FIRST MILESTONE-TWO NUMBER (2026-07-17,
25-minute budget,
1502 s
wall,
driver `repair-benchmark-run.ts` in the session scratchpad):
seededRepairRate 0.75 (3 of 4 measurable seeds restored),
2 of 4
sampled entries dispatched (coverage 0.5,
budget cut the rest
honestly),
both dispatched entries status `repaired`
(a2581911655 25/29 issues resolved,
homoyamakaze 33/34).
The one missed seed (a2581911655 omission-0,
5/18 words returned)
is the first repair-quality calibration datum.
One chunk
(a2581911655 chunk 1) lost SIX of seven critics to 4-minute
deadlines while GLM-5.2 completed and chunk 0 of the same entry had
completed 7/7 four minutes earlier;
the pipeline finished the chunk
on the surviving voice.
PROBE VERDICT (task 19,
2026-07-17):
the
exact seeded chunk re-run in a fresh window completed on ALL SEVEN
models (44 to 205 s,
six substantive reports plus MiniMax's known
near-empty quirk),
so the forfeit was provider weather,
not
content-hard input;
retry-to-quorum is the right remedy.
gpt-oss
and Nemotron ran 200+ s even in the calm window:
they are the
systematic slow tail on this plan,
not occasionally unlucky.
Net quota over the whole run POSITIVE
(2740.5 -> 2745.6 of 2750).
The repair-phase exports live in `pipeline-barrel.ts` (root barrel
hit max-lines).
MILESTONE TWO IS NOT DONE (user directive 2026-07-17):
a stage that
loses voices must retry the lost ones until over half its roster is
heard,
DONE (task 18,
commit `46d716ffa`:
`stage-quorum.ts`
`gatherStageVoices`,
retries stop at quorum,
unmet quorum proceeds
with a finding;
wired through critic,
panel,
checker,
and the
one-model editor roster).
Budgeted runs now accumulate the real
seededRepairRate (task 20);
per-run accumulator JSONL sits in the
session scratchpad.
