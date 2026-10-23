# No-loop production remediation proposal

## Status

Proposal,
2026-09-01,
implementing the
[post-redesign direction decision](../decision/translation-repair-post-redesign-direction.md)
and its no-loops refinement
("loops are discouraged altogether...
Discouraged,
not a ban.
You can overrule on this if you provide enough evidence and get an approval").
Grounded in
[`translation-repair-redesign-insights.md`](translation-repair-redesign-insights.md).
Supersedes the loop-bound shape in
[`translation-repair-bounded-quality-termination.md`](translation-repair-bounded-quality-termination.md).
Decision points answered 2026-09-01
(recorded in
[`translation-repair-roster-blocklist.md`](../decision/translation-repair-roster-blocklist.md)):
the shape is neither approved nor disapproved,
with prototyping and measurement delegated
("You're free to prototype and measure and do anything"),
so this document is a working design,
not a ratified one;
the roster decision is R3,
calibrate then seat,
under the owner's blocklist.

## Why the loops exist, and why they go

The blocking review and its correction loops are all takeover additions,
landed 2026-08-27 through 2026-08-29 (`git log --follow` on each module):
overlapped consolidation on 08-27 (`e8f6e78e9`),
the blocking absolute-naturalness review on 08-28 (`6fadd3be0`),
correction first bounded the same day (`4a60a660e`,
"allow bounded follow-up correction"),
then continuous on 08-29 (`1d16d89c4`,
`97fda9f95`),
with the translate and archive-block continuations the same day (`ed756993b`,
`ccaad1f53`).
None of this machinery predates the takeover.
The owner's stated rationale for removing it:
they were built "under an assumption that models are bad,
which isn't really true as of today".
The loops compensate for weak model output by re-asking until a panel is satisfied;
with current-generation models the compensation is the cost center,
not the safety net,
and the stopped Carena run measured it buying churn rather than convergence.

## Design rule

Every model stage is a producer or a selector inside a fixed-depth graph.
No stage may reject without replacing,
no response creates work,
and reviews are recorded evidence,
never withholding authority.
This is the owner's own corrected-requirements language applied at slice scale,
where the insights show responses fit completion budgets and the wide OCR-bridged roster stays usable.

Where a bounded second round exists it is a statically named graph node,
not a loop:
depth is fixed before the first provider call.

## The six loops and their replacements

### Consolidation naturalness correction, removed

`settleNaturalnessCorrections` and the absolute-naturalness review's blocking authority go away.
Consolidation becomes:
produce slate once,
judged selection once,
deterministic gates
(preservation,
structure,
declared names),
then one fixed polish round:
refiners propose,
judges select between proposal and standing text,
and the deterministic guard applies to whichever was selected.
A declined proposal,
and equally a selected proposal the guard rejects,
leaves the standing text shipping with the outcome recorded as a finding;
this node has no re-ask under any outcome.
Absolute-naturalness findings are recorded as evidence on the settlement.

As a landed consequence (`4eabf53cf`),
the persistence completeness guard accepts a rejected or quorumless final
review as recorded evidence:
the artifact parser's repeated-acceptance invariant now binds only an
acceptable final verdict,
because `confirmAbsoluteNaturalness` legitimately returns zero acceptance
confirmations when the decisive reading is not acceptable,
and the unpatched parser refused exactly the record this remediation ships.

Evidence:
86 percent of the stopped Carena run's consolidation round time was post-quorum grace;
acceptance demanded near-unanimity of eight seats and passed 5 of 46 times;
22 of 40 blocks came from a single seat;
serial correction without located evidence measurably degrades text (D1.3);
and the judged-contest form is the one that produced the page a reading accepted.

### Consolidation recovery, single attempt

`buyConsolidationSlice` makes one attempt.
As built (`consolidate-slice-buy.ts`,
commit `1ba8f713a`),
this deviates from the proposal's deterministic best-produced selection:
when standing lacks contest endorsement and the single attempt kept it,
the standing text ships with a
`consolidation-standing-unendorsed` finding recorded as evidence,
and `consolidationWorthResuming` refuses to persist the settlement,
so a later run buys a fresh attempt instead of resuming the non-endorsement.
The deterministic pick was rejected during implementation because every
produced candidate at that point was one the gate or the judges declined,
so selecting among them would ship text that never passed the fidelity-first
gate while the standing text is the one wording with a pedigree;
overriding a deciding stage's recorded verdict with a non-model tiebreak
inverts the guarantee worth keeping.
Zero produced candidates remain the bounded provider error.

### Translate follow-up, fixed depth two

Initial production plus at most one statically named follow-up round carrying the judge's located rejection evidence.
After that the slice settles as unfilled or incumbent-kept,
never as a thrown entry.

This is the one rejection-driven second round the proposal retains,
so it is where "discouraged,
not a ban" is being exercised.
Its evidence is adjacent,
not direct:
the measured three-of-four recovery (`#230`) converted schema-mismatch re-asks,
an infrastructure class,
not judge rejections;
and the redesign showed located-evidence-carrying second rounds are the one safe form
(E1 double-prime) while evidence-free serial rounds degrade text (D1.3).
Direct conversion of a quality follow-up round is unmeasured;
the four-entry pass logs it,
and if it converts nothing this round drops to depth one.
Striking it now instead is a coherent stricter choice.

### Archive-block review, single round

Unresolved blocks are retained with findings;
reviewer indecision cannot withhold the entry.

### Preparation revision, linear two-step chain

One preparation,
selected corrections applied once,
one re-preparation over the corrected archive.
Remaining unclaimed blocks become findings.
This is not a rejection-driven re-ask:
the second preparation is the structural consequence of having edited the archive,
a producer chain,
so it does not exercise the loop allowance.

### Insertion placement, single round

One coverage round.
As built (`pass-insertion-admission.ts`,
commit `7e702e075`),
a passage still unresolved after that round is not admitted:
the pass records an
`insertion-unresolved-after-single-round` finding naming the slice and the
verdict,
and the page ships without the passage,
because an insertion is recovered supplementary content whose absence is a
recorded gap,
not a missing required page.
This sentence is about the insertion path only:
a passage that is not admitted has no slice, so the artifact records no `sourceText` for it
and the page stays silent about it.
A slice that exists and is left unfilled ships its source text under a note
(owner ruling of 2026-10-23, ledger B827 in
`package/translation-repair/doc/audit-ledger-page-assembly-and-corpus-run-driver.md`).
The corroboration guard keeps its admission authority unchanged,
since it is a deterministic-integrity check rather than a quality review.
`UnfilledPageError` remains the one bounded no-page terminal,
reached only when required content produced no admissible candidate.
A placement-repair second round was considered and dropped:
no measurement shows what such a round converts,
and retaining it on symmetry alone would exercise the loop allowance without evidence.
If the four-entry pass actually hits unfilled terminals here,
that run supplies the evidence to propose the round then.

## Entry terminals after the change

The three-bucket classification from the bounded-termination plan carries over unchanged:
quality-withholding `INCOMPLETE` classes retire,
deterministic-integrity classes stay,
infrastructure and model-output classes stay.
The two-seat located-agreement floor from the zero-spend replay becomes moot for stages that no longer block;
it stays available if any blocking review survives design review.

## Portable guards landing regardless

- Truncation-as-unusable in production's `readJsonOutcome`:
  today content that parses wins even at `finish_reason=max_tokens`,
  the exact defect `doc/troubleshooting/charm-hyper-max-tokens-tool-json.md` records;
  the rejection feeds the existing bounded schema-mismatch recovery.
- Corpus-pin environment override
  (clone dir and commit),
  replacing the throwaway edits the Carena fixture runs used.

## Roster refresh options

Production's writer seats
(Qwen3.8-27B,
Kimi-K3,
gemma-4-26b)
were measured before the current Hyper top end existed.
The live Hyper catalog now carries qwen3.8-flash and qwen3.8-max (vision,
128,000 and 65,536 output),
qwen3.8-2.4t-a95b,
full glm-5.3 (262,144 output),
deepseek-v4-pro and -flash (384,000 output),
minimax-m3 (512,000 output),
and glm-5.3-flash whose own ceiling is 131,072 where the project vetted 32,000.
Synthetic's live listing today
(11 rows fetched 2026-09-01 from its models endpoint)
adds GLM-5.2 and Nemotron-3-Super-120B beyond what is seated,
so the advancement is concentrated on the Hyper side.

- R1,
  keep the measured roster:
  cleanest attribution,
  but leaves the advancement unused against the owner's stated premise.
- R2,
  seat obvious upgrades unmeasured and let the four-entry pass read the combination:
  fastest leverage;
  the owner has accepted exactly this bundling before
  ("Bundle all the improvements that could be made,
  in"),
  at the recorded cost that no delta attributes to one change.
- R3,
  run the existing producer-calibrate and editor-calibrate instruments over the refreshed candidate set first,
  seat winners on measurement,
  then pass:
  matches the repo's seat-on-measurement practice and vets each new route's real ceiling.
  Measured instrument cost
  (`doc/decision/translation-repair-calibration-overlap.md`,
  `doc/decision/translation-repair-multi-provider.md`):
  editor-calibrate arms ran 24.18 to 58.95 minutes wall each over four bench slices,
  and the twelve-round producer pass finished in 3,637 seconds
  before the forty-round pass that seated the writers;
  a refreshed calibration is that order of spend,
  well under a pass-scale run.

Under R3 the order is:
loop-removal code lands and is suite-,
lint-,
and GFP-verified;
calibration runs on that build and seats the roster;
then the four-entry pass;
then the reading.
The calibration doubles as a live smoke of the changed build before the pass spends anything,
since `producer-calibrate` drives `runTranslateStage`
(the import is in `src/corpus-run/producer-calibrate.ts`),
the lane whose follow-up depth this proposal changes.

Ranking:
R3 over R2,
because disinterested-ballot calibration has already overturned a headline ordering once
(the twelve-round table read gemma and qwen3.8-max as second and third,
and both confidence intervals contained the null;
seating on that ordering "would be reading a ranking as a result"),
and the calibration also vets routes and ceilings the pass will depend on.
R2 over R1,
because R1 spends the whole pass without testing the premise the owner named.

## What stays

Slice-scale two-lane shape,
lane contest,
deterministic guards,
slice caches,
the overlap dial,
the OCR reading lane,
the four-entry completion set,
and the pass-then-reading gate.

## Decision points

1.  Approve the loop-free shape above,
    including the one retained rejection-driven follow-up round
    (translate,
    fixed depth two,
    the only place the loop allowance is exercised)?
    Vetoing that round to depth one is a coherent stricter variant.
2.  Roster:
    R1,
    R2,
    or R3?
