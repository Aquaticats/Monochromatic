# What the thirteen failed redesign candidates teach

## Status

Written 2026-09-01 after reading every candidate detail file
(`translation-repair-interface-candidates-a-d.md` through `-k-m.md`),
the comparison index,
the failure report,
and the raw Candidate L and Candidate M complete-page reviews,
at the owner's instruction to read all failed attempts before any design commitment.
Sections separate what the record shows from my own analysis.

## What was tried, one line each

- A1:
  draft plus finding-only auditors;
  a non-producing auditor withheld a complete draft;
  suspended.
- A2:
  four serial complete-document producers;
  all four failed deterministic admission;
  no page.
- B:
  typed specification then renderer plus three specialists;
  schema and envelope failures;
  no page.
- C:
  three briefs then two whole-document editors;
  both editors failed admission;
  no page.
- D1 to D1.3:
  immutable shell plus slot authors,
  then reviser,
  then copy editor;
  four published private pages,
  every one rejected by complete reading;
  D1.3 measured serial editing as non-monotonic.
- E1 prime:
  three authors,
  three auditors,
  located-finding quorum;
  whole-ballot strictness discarded 72 of 80 valid located findings.
- E1 double-prime:
  per-finding admission;
  replay reproduced human comparisons;
  live Hyper-only run falsely passed a one-candidate floor;
  page rejected.
- E1 triple-prime:
  diversity floors corrected;
  roster exhausted;
  five Qwen-family expansion authors all failed complete reading.
- F:
  donor-slot assembly;
  rejected at design gate;
  absent findings are not clean evidence.
- G:
  author-attested realization ledger over 134 obligations;
  unmeasured verifier envelope;
  no spend.
- H:
  closed-world compact status matrices;
  zero admissible verifier ballots from three models;
  page rejected.
- I:
  candidate-scoped compact ballots;
  both authors admitted,
  one clean nonself family each;
  both pages rejected.
- J:
  Kimi third author;
  rejected at design gate on measured truncation.
- K:
  authors given the readable review plan;
  zero admitted authors;
  verifiers never dispatched.
- L:
  lean values-only authors;
  one admitted page with an actor inversion;
  all three verifier ballots unusable.
- M:
  risk-attested authors plus role-split first-defect challengers;
  one admitted page,
  floor unmet,
  page rejected with an actor-attribution defect
  despite an actor-attribution attestation and clean nonself challenges.

## The strongest cross-candidate signal: the defects repeat

The same semantic defect classes survived every architecture,
prompt rewrite,
and interface change on this roster:

- actor attribution and event ownership:
  D1.2's final regret passage,
  Candidate L lines 19 and 25 (narrator claims the actions the source assigns to Carena),
  Candidate M body paragraph 15;
- omission with substituted emphasis:
  D1.1's recognition-and-support omission,
  Candidate M's final paragraph (youth image dropped,
  autonomy gloss added);
- technical-term mistranslation:
  laboratory staining rendered as ordinary dyeing in D1.1,
  D1.2,
  and D1.3;
- source-language calques,
  register drift,
  and tense drift on nearly every page.

Defects are sparse,
a handful per page,
but publication-blocking,
and they are exactly the classes the corrected requirements name.
Candidate M even put actor attribution into the author's attested risk register and received an attestation of `checked`
over a page carrying an actor-attribution defect.

My analysis:
this is a capability ceiling of one-shot whole-page authorship on this roster
(Qwen3.8-27B,
MiniMax M3,
GLM 5.3 Flash,
Kimi K3),
not a prompt or interface problem.
Six verifier-interface redesigns changed which failures were visible,
never the author defect profile.

## Structural findings

### Whole-page work collides with the output envelope; slice-scale work does not

Complete-page forced-tool responses repeatedly died at completion ceilings with default provider reasoning:
GLM emitted thinking only to 32,000 tokens four times across K,
L,
and M,
and was cut at the 360-second call deadline twice in I;
MiniMax authors truncated at 32,000 twice;
Kimi truncated at its 16,000 model cap;
Qwen completed a 59,438-token author response that was admitted in M,
while its K author and its 47,553-token M verifier returned unparseable tool JSON.
Context windows were never the problem;
completion budgets under mandatory default thinking were.
The owner forbids reasoning knobs,
so this constraint is fixed.
Slice-scale responses are one to two orders of magnitude smaller and did not exhibit this failure mode in the
legacy pipeline's thousands of measured rounds.

### The every-node-vision requirement starved the roster

The redesign required every node to carry the page image,
restricting it to three or four Hyper vision routes.
The legacy pipeline solved the same need with deterministic OCR plus a dedicated reading lane
(register items 111 and 123),
which keeps roughly ten seats across two providers usable.
The redesign's family-diversity crisis was partly self-inflicted by this requirement.

### Verifier verdict form decides everything

Across candidates,
four verdict forms were measured:

- unlocated holistic rejection (the legacy naturalness review's form):
  drives unbounded correction churn;
  40 of 46 rejections on the stopped Carena run with wording that varies every round;
- exhaustive per-obligation matrices (G,
  H):
  models could not return them;
  zero admissible ballots;
- whole-role clean assertions (M):
  returnable but under-discriminating;
  clean challenges over a page with recorded defects;
- located,
  class-closed,
  anchor-bound findings with per-finding admission (E1 double-prime):
  the only form that reproduced human page comparisons,
  in a 48-millisecond zero-spend replay that picked the same page complete reading picked.

Corollaries proven along the way:
an absent finding is never clean evidence (F's rejection reason);
whole-ballot strictness throws away valid located evidence (E1 prime's 72 of 80);
self-clean never qualifies and self-defect still vetoes;
a truncated response is unusable regardless of parseability.

### Serial improvement is non-monotonic without located evidence

D1.3 measured a later usable copy editor preserving existing defects and adding new ones.
Unconditional adoption of later output is unsafe.
Conditional adoption needs located strict-subset evidence,
and at page scale models could not supply it reliably
(the E1 double-prime resolver changed 13 slots when seven were authorized).
My analysis:
at slice scale the legacy pipeline already has the safer form,
judged contests between concrete candidate wordings,
and its measured guard (`checkPreservation`) is deterministic.

### Deterministic guards worked and several are portable to production

- Truncation-as-unusable at the shared provider boundary:
  commit `44ed76c59` rejects Anthropic `max_tokens` and OpenAI `length` before parsing,
  with the defect documented in `doc/troubleshooting/charm-hyper-max-tokens-tool-json.md`.
  That commit exists only on `prototype/translation-repair-finite-pipelines`;
  production's `chat-json-outcome` path needs the equivalent check verified or ported.
- Source-echo refusal (Han ideographs in an English slot),
  which caught Qwen echoing Chinese as a "complete" answer.
- Presentation-artifact refusal (visible return markers,
  control pictures,
  replacement characters).
- Runtime-owned boundary separators,
  which fixed Candidate H's shared footnote-spacing defect without touching prose.
- Raw duplicate-member refusal before JSON parsing.

### Front matter was a common-mode gap on both sides

The redesign hit it as a common-mode veto (Candidate K's hard gate),
then moved four front-matter strings into author-owned slots with per-path identity contracts (Candidate L).
Production never translates front matter at all (register item 269).
Candidate L's per-path contract
(name from alias members,
ordered alias grammar,
protected Latin identity,
location and description as reviewed prose)
is a ready-made design for closing 269.

### Process finding: the cross-candidate spent-prompt doctrine amplified flukes

The owner's contract forbids retry and prompt resending within one invocation's manifest.
The takeover extended this to a permanent cross-candidate doctrine:
any failed exchange poisons that model-plus-prompt pair forever.
Under it,
one missing closing brace at 32,000 tokens or one deadline cut became permanent architecture-level evidence,
forcing a new candidate letter instead of a second sample.
Nothing in the recorded owner decisions requires the cross-run half of that doctrine.
Whether it binds future work is an open owner question;
it also means several candidate rejections rest on single unrepeated observations
(FLK:
one capacity failure is not a stable limit).

### The comparison the redesign never ran

No legacy-pipeline Carena page was ever produced or read,
because the legacy consolidation never terminated on that entry.
Every quality comparison in the redesign is whole-page candidate against complete reading;
none is legacy output against whole-page output.
The one legacy page that was read under the current two-lane shape,
Toka_ls at 300-second grace,
was accepted by strict and independent readings.
The redesign's premise that per-slice competition fragments voice was asserted,
never measured against an actual settled page.

## Zero-spend replay over the stopped Carena run

Following the E1 double-prime precedent of validating an admission change offline,
the 46 retained naturalness-review log lines were re-parsed
(the first parse silently dropped most seats because model ids contain colons;
the corrected parse accounts for all 368 seats,
eight per review):

- per-seat verdicts:
  281 `acceptable`,
  71 `unacceptable`,
  16 `unusable`;
- every one of the 71 rejecting seats carried located findings (paragraph plus problem),
  so the legacy review's rejections are already located,
  which corrects the verdict-form framing in the section
  "Verifier verdict form decides everything":
  the legacy form is located single-seat decisive rejection,
  not unlocated rejection;
- the churn engine is the aggregation rule:
  any single usable rejection among eight seats blocks,
  so acceptance demands near-unanimity every round,
  and with seats individually accepting 76 percent of the time only 5 of 46 reviews accepted.

Replaying a two-seat located-agreement floor
(a rejection blocks only when at least two rejecting seats share a finding paragraph)
over the 40 blocking reviews:

- 22 were blocked by exactly one rejecting seat and become non-blocking;
- 18 had two or more rejecting seats,
  and in all 18 at least two shared a finding paragraph,
  so every corroborated rejection keeps its authority;
- acceptance moves from 5 of 46 to 27 of 46 reviews,
  cutting correction demand by more than half without any provider call.

This overturns the rejected-alternatives reasoning in the provisional bounded-termination plan,
which had dismissed a rejection quorum as the larger change:
the floor is offline-measurable,
surgically effective on the measured wedge,
and loses no corroborated rejection on this run.

## Implications I draw (not yet a decision)

1.  The evidence argues for staying at slice scale on this roster:
    slice outputs fit the envelope,
    the wide OCR-bridged roster stays usable,
    and the one read page from the slice pipeline passed.
2.  The legacy pipeline's measured non-termination is a verdict-form problem as much as a loop-bound problem:
    its naturalness review uses exactly the verdict form the redesign proved churn-generating
    (unlocated holistic rejection with decisive authority).
    Bounding the loops without changing the verdict form caps the churn but keeps its engine.
3.  The redesign's one operationally validated positive result,
    located class-closed anchor-bound findings with per-finding admission and self-exclusion,
    is importable into the legacy reviews at slice scale,
    where responses are small enough to return reliably.
4.  Portable deterministic guards (truncation rejection above all) should be verified or ported to production regardless
    of any other choice.
5.  Reopening whole-document authorship on this roster is contraindicated by every measured attempt.

The bounded-termination plan in
[`translation-repair-bounded-quality-termination.md`](translation-repair-bounded-quality-termination.md)
is provisional and covers implication 2's loop-bound half only;
the owner decides whether the verdict-form change joins it before implementation.
