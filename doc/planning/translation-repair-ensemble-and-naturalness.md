# Ensemble editing and the naturalness refinement lane

Proposal,
not a ratified decision.
Written after the round-two gate failed at 0.740 to 0.800 against a 0.9 bar.

## Why these two are one design

They look like separate asks and share a mechanism.

The editor is the only stage where a single model decides alone.
Critics,
panel,
and checkers are all ensembles;
`RepairModels.editorModelId` WAS one `SyntheticModelId`,
and `runEditorStage` passed `modelIds: [editorModelId,]` to `gatherStageVoices`.
User directive:
no single model should control any part of the pipeline.
That half is now BUILT;
this paragraph describes the state it replaced.

The naturalness lane needs candidate generation with independent judging,
because a rewrite has no defect to anchor to and the generator must not approve its own work.

Both therefore need:
propose N candidates,
judge them independently of whoever produced them,
select one or fall back.
Build that once.

## What the round-two grading actually asks for

Precision cannot see repair quality.
Four of the 37 true positives carry notes saying detection was right and the repair was poor:
"the claim doesn't make sense for English grammar.
If there is a better way,
that should be proposed.",
"but is there a better way?"
twice,
and "I suspect this one is hard to fix".
Those score as successes today.

So the editor is a measured weak point,
and spending the strongest model plus an ensemble there is aimed at the right target.

## Shape

Three immutable versions,
each with its own provenance:

```text
T0  original translation
T1  accepted issue repairs        diff(T0,T1) confined to accepted issue envelopes
T2  naturalness refinement        diff(T1,T2) confined to one recorded paragraph
```

Every change in the shipped text traces to exactly one of:
an accepted issue repair,
or an accepted paragraph refinement with recorded gate results.
RETRACTED,
see "Still open before the lane can be built":
that guarantee is unachievable once `T2` rewrites a paragraph holding a `T1` insertion,
and replayable stage operations replace it.

Paragraph containment is provable deterministically.
Semantic preservation is not provable by an LLM panel,
so it is an acceptance policy,
never called a proof.

## Selection component, shared

Inputs:
a set of candidates,
each tagged with the model that produced it;
the evidence the judges need;
a fallback.

Rules:
a model never judges its own candidate;
judges vote through the existing `gatherStageVoices` quorum machinery;
a tie or a failed quorum returns the fallback unchanged.

SUPERSEDED 2026-08-14 on the first of those rules.
Producers now judge,
and a ballot for their own work counts half.
`doc/decision/translation-repair-ensemble-voting.md` is canonical for that and
for the removal of the full-roster retry target this document's quorum machinery
once offered.

For the editor the fallback is the unedited chunk.
For the refinement lane the fallback is that paragraph's exact `T1` text.

This means a stage can always decline.
Declining is the conservative outcome and must never read as failure.

## Editor ensemble

`editorModelId: SyntheticModelId` becomes `editorModelIds: readonly SyntheticModelId[]`.
Each editor sees the identical sheet and proposes operations.
Judges score candidate patched chunks on:
did it apply the accepted issues,
is it faithful to the source,
is the English grammatical and natural.

RESOLVED by user decision,
and this paragraph's recommendation was overridden:
judge per envelope AND per chunk,
both.
The original open question read:
Per envelope is finer and lets the best fix of each issue win independently,
but a chunk assembled from several models' operations is a text no model wrote or checked as a whole.
Per chunk keeps coherence and wastes good individual fixes.
Recommendation:
per chunk first,
because coherence is the thing the naturalness work exists to protect,
and revisit only with evidence that per-envelope wins.

## Naturalness lane

Runs AFTER the accuracy repair,
never in parallel,
and only on `T1`.
Per paragraph,
a rewriter may return an improved candidate or decline.

Gates,
all of which must pass:

-    structural invariants:
     Markdown structure,
     links,
     URLs,
     footnote markers,
     block type,
     names,
     handles,
     numbers,
     dates,
     and foreign-language phrases survive unchanged
-    three-way semantic check against source,
     `T1`,
     and candidate:
     propositions,
     negation,
     modality,
     chronology,
     and emotional stance preserved
-    content ledger:
     built before rewriting,
     and especially load-bearing for repaired omissions,
     where fluency rewriting silently compresses inserted clauses
-    independent judging,
     excluding the generator
-    final regression check on the text that will actually ship

Poetry,
quotations,
tables,
and code are routed away or abstained on,
which is also the fix for round-two false positive 26.
See "Still open before the lane can be built":
this is a conservative ELIGIBILITY FILTER and must not be called verse detection.

## Contract breakages this creates, all of which must be handled

-    `RepairIssueRecord.resolved` promises checkers confirmed the fix in the shipped text.
     False if checking happens before refinement.
     Recheck against the shipped text.
-    `changedOutcomes` and `anyChanged` in `repair-translation.ts` miss refinement-only changes
     if the lane is bolted on after they are computed.
-    `SliceCache` entries predate the lane,
     so a cached slice would bypass it.
     SUPERSEDED by "Settled before building the lane":
     two separate caches,
     not one versioned key,
     so a lane edit never invalidates expensive `T1` work.
-    The `blocked-non-translation` path must keep returning the original untouched.
-    A refinement-only change has no representation in `issues` and needs its own record type.

## Measurement

Keep defect precision as it is,
with refinements excluded,
and publish a separate scorecard:
refinement precision,
naturalness win/tie/loss against `T1` by blinded pairwise comparison,
semantic regression rate,
issue-retention rate,
refinement yield,
and repair acceptability.

Excluding refinements from defect precision must not leave them unmeasured.

BUILT since this was written,
covering the semantic-regression half of that
list:
the lane runs (task 46),
and an accepted refinement is now audited for damage it
caused,
by the introduced-defect probe under a second framing,
recorded as
`refinementDefects` and reported on `score-probe`'s `REFINEMENT` line (task 58).
`retainsResolvedIssues` covers issue retention.
Still unbuilt from this list:
refinement precision,
blinded pairwise naturalness win/tie/loss,
refinement
yield,
and repair acceptability.

Use the round-two graded items as regression fixtures:
the poetry,
`总是`,
and conjunction false positives are hard negatives for detection,
and the four "is there a better way?"
items are repair-quality fixtures.

## Settled before building the lane

The editor ensemble half of this proposal is BUILT
(commits `7cce752d4`,
`1527e4929`,
`688b96122`,
task 45).
It judges per envelope AND per chunk by user decision,
overriding this document's "per chunk first" recommendation.
These five are settled for the lane itself.

The lane runs as a SECOND PER-SLICE PHASE in `repairTranslation`,
after every T1 outcome settles and non-translation dominance is decided,
and before `changedOutcomes`,
the issue records,
the status,
and final assembly.

An earlier version of this section said the opposite,
that the lane belongs inside `repairChunk`.
That was WRONG and is retracted.
`repairChunk` returns early when the non-translation votes stand,
when `screening.claims.length === 0`,
when `envelopes.length === 0`,
and when no operation survives the gate.
A lane appended to the bottom of it would never run on text that carries no
accuracy defect,
and text that is merely awkward rather than wrong is precisely what this lane
exists for.
The placement would have missed its own primary target.

The price of the correct placement is real and is accepted:
T1 and final outcomes need separate records,
each final slice must map back to its source slice and T1 text,
and the lane needs its own cache rather than inheriting the accuracy one.

Keep TWO caches rather than versioning the one key.
The accuracy cache stays keyed to T0 inputs and accuracy-stage configuration;
the naturalness cache keys on the source slice,
the exact T1 slice hash,
the global definitions the paragraph depends on,
and a lane schema version.
Two caches mean a naturalness change never invalidates expensive T1 work,
which one shared version constant would have done on every lane edit.

A failed recheck falls back to `T1` for the WHOLE slice,
not per paragraph.
Checkers report per ISSUE while refinement happens per paragraph,
and an issue can span paragraphs,
so per-paragraph attribution is not derivable from what the checker returns.
Log which issue regressed
so a later session can judge whether finer attribution is worth building.

The lane does NOT run through `selectRepairCandidate`,
and an earlier version of
this section saying it should is retracted.
That function is deterministic measurement ranking with no producer exclusion,
and its comparison would defeat the lane outright.
`compareCandidates` in `select-candidate.ts` is lexicographic,
and once
integrity and the resolution counts tie,
it reaches `l.changedCharCount - r.changedCharCount`,
which prefers the candidate that changed FEWER characters.
A naturalness refinement resolves no additional issue by construction,
so `T1` and `T2` tie on every earlier key and `T1` then wins on that one,
every time.
Naturalness selection therefore uses `selectBestCandidate` instead,
which is the producer-excluding judge component.

The lane must NOT reuse `selectChunkPatch`.
That wrapper ships its strongest repair on `indecision`,
which is right for accuracy because a later gate still makes the repair
beat the untouched text on measurements.
Naturalness has no such later gate,
and nobody claimed the text was wrong in the first place,
so BOTH decline dispositions map to exact `T1`.

First cut uses ONE rewriter,
called once per slice rather than per paragraph.
That call returns zero or more paragraph operations,
each carrying a paragraph identity,
a `T1` base hash,
and a replacement;
each operation is gated independently;
the survivors apply to immutable `T1` to form one full-slice candidate.
This keeps paragraph-level containment
while judging coherence at slice level,
which is the same problem whole-chunk judging solves for the editor:
independently selected paragraph rewrites otherwise form a slice
no model ever assessed as a whole.
One producer still cannot make text ship alone,
because `selectBestCandidate` judges even a lone candidate
and requires `MIN_SELECTION_VOTES`.

Two roster invariants the current asserts do NOT cover,
and must:
several rewriters could consume the whole judge roster,
so the lane needs its own assertion analogous to
`assertJudgeableEditorRoster`;
and `assertCheckerIndependence` considers only `editorModelIds`,
so a naturalness rewriter can currently sit in the checker roster
and certify its own rewrite during the recheck.

Exclusive attribution as this document originally phrased it is IMPOSSIBLE.
When `T2` rewrites a paragraph containing a `T1` insertion,
the final text has causal contributions from both stages,
so no rule assigns every shipped character to exactly one of them.
Guarantee REPLAYABLE stage operations,
T0 to T1 to T2,
instead,
and drop the claim that each change traces to exactly one stage.

## Still open before the lane can be built

Eligibility cannot read the original target's `DocumentNode`s:
accuracy edits shift offsets and can change block structure,
so `T1` has to be reparsed.

"No hard line break" is not a verse detector and must not be called one.
An mdast `break` node,
a soft source wrap inside `DocumentNode.text`,
and an HTML or MDX `<br>` are three different things,
and none of them identifies poetry.
`flattenContainers` also loses disclosure-container ancestry,
so `kind === 'paragraph'` does not prove ordinary top-level prose.
Ship a conservative ELIGIBILITY FILTER instead,
named as such:
exactly one physical source line,
no mdast `break`,
no HTML or MDX break element,
no excluded container ancestry,
no parse downgrade or masked region,
and both a minimum and a maximum length.
Single-line poetry still passes it and wrapped prose is still skipped;
that is acceptable for a filter and would be dishonest for a detector.

Structural gating compares an ORDERED sequence of protected atoms,
never a multiset.
A multiset admits swaps that change meaning while passing:
"3 cats and 5 dogs" becoming "5 cats and 3 dogs",
two links exchanging destinations,
two names exchanging positions.
Protected atoms cover link and image URLs,
references resolved through the whole-document definition map,
footnote identifiers and convention,
inline code,
number tokens,
CJK runs,
and raw HTML,
MDX expressions,
and JSX,
plus the handles,
identities,
and dates this document already named.
`identityLines` already exists in `repairTranslation` and should reach the lane.
Reference links cannot be resolved by parsing an isolated paragraph,
so the global definition map has to come from the assembled `T1` document,
and document-wide footnote and reference integrity has to be validated
by parsing the complete `T2` at the end;
paragraph-local reparsing cannot establish it.
Expose the reference collection under `buildFootnoteGraph`
rather than writing a second footnote grammar.
Numbers are not mdast nodes:
scan only text leaves with a linear character-state scanner,
never the Markdown syntax,
and decide up front whether signs,
decimal separators,
percentages,
dates,
ranges,
full-width digits,
and ordinal suffixes are one token or several.

`ChunkRepairOutcome.changed` currently means an accuracy candidate beat
unchanged,
so its TSDoc goes false the moment a refinement-only change exists.
Introduce a final slice outcome rather than overloading it.
Findings strings are also not enough for provenance:
refinement records need the `T1` paragraph identity and hash,
the replacement hash,
producer provenance,
the gate result,
the selection tally,
and the rollback result.

Issue rechecking proves nothing on a slice that had no accepted issues,
which is most of the lane's target.
The judge prompt is the only guard there,
so it must show source,
`T1`,
and candidate together,
rank faithfulness first,
and require a clear improvement over `T1` rather than a preference.

## Attribution warning

Round three changes the roster,
the editor,
the checker set,
the quorum rule,
the adjudication policy,
and adds this lane.
The user accepted that bundle explicitly.
A precision delta will therefore not be attributable to any single change,
and the round-three verdict must say so rather than implying otherwise.
