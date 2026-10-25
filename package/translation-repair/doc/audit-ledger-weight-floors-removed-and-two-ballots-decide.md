# Audit ledger: the weight floors removed and two ballots deciding

The owner's rulings of 2026-10-25 on the corpus pass `XingZ6021`:
no summed-weight floor stands between two answering judges and a decision,
a claim or contest is decided once two or more judges answered it,
and one judge alone never decides.
This file holds B900 to B906.
Rounds 2 and 3 of the same work (B907 to B916) are in
[the weight floors removed, rounds 2 and 3](audit-ledger-weight-floors-removed-rounds-two-and-three.md):
the owner's answers of the same day
("Allow two checkers",
"Keep waiting for half",
"Two judges must name it",
"Ship the slate's wording" and
"Ship the slate's pick (Recommended)"),
the lead's rulings on the findings of the independent re-checks,
the tie of a challenge round across every offered candidate,
and the page title.
Notes for round 3 also stand in B902 and B906 of this file.
The rulings are quoted in
[votes, repairs and run-offs](design-commitments-votes-repairs-and-run-offs.md)
and referenced from
[shipping and the archive](design-commitments-shipping-and-the-archive.md).

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B900: the panel tally asked for a summed weight of three, so a unanimous panel of two decided nothing

Status:
fixed on 2026-10-25 (UTC),
red `68a7056376`,
fix `85d2fa9946`.

What `XingZ6021` showed:
slice 67 drew nine claims (seven major),
slices 68 and 69 drew two critical `accuracy/non-translation` claims each,
and the panel of two (Claude Haiku 5.5 and DeepSeek V4.1 Flash) supported them unanimously.
The tally compared the electorate weight with `minBallotWeight` 3,
two panelists at the default weight of one summed to 2,
and every claim landed `needs-human`.
The editor had nothing to edit (`nothing to edit, unchanged`),
and the archive's text shipped with an invented quotation from Metal Gear Solid 2
and two paragraphs the source does not hold.
Across the run 234 of 341 adjudicated issues ended `needs-human`.

The rule now:
`decideStatus` in `adjudication/src/tally-votes.ts` counts the panelists that answered with a vote that carries weight
and lands `needs-human` below `MIN_PANEL_BALLOTS` (2).
`minBallotWeight` is gone from `AdjudicationConfig`
(`shared-contract/src/adjudicate-model.ts`),
and the decision,
source-defect and run-off ratios stay.
A heavy weight does not turn one answer into two:
the red case `tally-two-ballots.unit.test.ts` holds one panelist at a weight of 5 and expects `needs-human`.

The reading adopted,
open to the owner's veto:
a panelist whose vote carries no weight (a configured weight of 0) is not an answering judge,
since a voice that weighs nothing does not decide.

The rule for next time:
a floor on a summed weight is a floor on how many judges must agree dressed as a number,
and it fails silently whenever the default weight is one and the bench is short;
a rule about how many judges decide is written as a count.

## B901: the candidate selection's minimum weight, its scaled short-bench minimum and the exceptions that worked around it

Status:
fixed on 2026-10-25 (UTC),
red `976d973367`,
fix `1dee3ca797`.

What it was:
a winner needed a summed weight of 2 (`MIN_SELECTION_WEIGHT`),
scaled by reachable over quorum on a short bench,
so a self-vote at half weight and a full vote (1.5) declined,
a leader two judges named was declined "for weight",
and two exceptions existed to get around the floor:
the run-off of a condemned incumbent,
and the condemned-fallback that seated a challenge proposal under the minimum.
The roster guard of `assertJudgeableEditorRoster` did weight arithmetic to refuse a roster that could not reach 2,
and the first-round fan-out asked the whole bench where the self-votes of a window alone could not reach 2
(a window of three halves on a bench of four).

The rule now:
`MIN_SELECTION_BALLOTS` (2,
declared in `house-rule/src/candidate-select-model.ts`) ballots name a winner,
whatever their weight.
A self-vote counts as one answering judge and its half weight only ranks candidates
(the owner's ruling of 2026-10-20,
"Count self-votes",
in [outcome rules and stage records](audit-ledger-outcome-rules-and-stage-records.md)).
A leader one judge alone named is declined (`winner named by one judge alone`).
The run-off and condemned-fallback parameters are gone from the translate judge and its retry,
and from the consolidation's settle judge.
The selection fan-out asks the whole bench only where the window of quorum plus one is under two judges,
so a bench of four now asks three judges where it asked four.
`selectionBench` and `shortBenchFinding` stay for the finding `select-short-bench (reachable r of n, quorum q)`,
which records a round decided on fewer seats than the bench seated.
The roster guard needs at least two distinct judges,
so two authors judging each other and one author with one other judge now pass.
The window trial of `bench` asks the whole bench of each arm (`fanOut: 'whole-bench'`)
so that its arms stay whole panels:
this is an assumption about what the trial meant,
recorded here because the default window would have changed its arms.

The behaviour that changed and is visible:
a condemned incumbent that leads with two ballots is selected,
as it already was at a summed weight of 2 or more.

The corner for the owner (finding F6 of the independent re-check):
the exception `condemnedFallbackLoses` refused a leader that was the fallback itself when a contest majority had
condemned it and it drew two ballots under a summed weight of 2.
Two ballots from a full vote and a self-vote weigh 1.5,
so the old code declined and the new code selects the incumbent
(`repair/src/candidate-select.unit.test.ts`,
"SEATS the standing text when two ballots lead with it").
At 2 or more the old code already selected it,
so the change is only the 1.5 corner.
It touches the owner's ruling of 2026-10-11,
"Majority,
like the run-off";
the exception is removed under the answers of 2026-10-25 and is listed in `report.md` as an owner-visible change.

## B902: three non-translation votes were needed to block a slice, and two critics could not

Status:
fixed on 2026-10-25 (UTC),
red `90cb72fab1`,
follow-up test `d42e79be74`,
fix `1439ea1e5a`.

What `XingZ6021` showed:
slices 68 and 69 each drew two wire-level critical non-translation votes,
from both critics heard,
and the floor of three (`NON_TRANSLATION_BLOCK_VOTES`,
`archive/src/non-translation-evidence.ts`) left the votes not standing.

The rule now:
two votes stand.
Two votes cannot come from fewer than two critics heard,
so one critic alone can never block.
The floor of three had been set after the first real-corpus false block (`AkiraComplex`,
commit `72f5537c1`),
which discarded a faithful translation on two votes from two of seven critics on an English-epigraph slice.
What guards that shape now is the contradiction check
(`NON_TRANSLATION_CONTRADICTION_MIN`,
eight validated content-critique claims anchored in the target)
and the good-translation anchor that vetoes a document-wide block.
This is an owner-visible risk,
and a narrow one:
a false per-slice block leaves that slice unrepaired,
with the archive's wording as it stands,
and the document-level dominance block is report-only
(`repair/src/repair-translation.ts` logs it as "reported,
not blocking" and adds a finding),
so no block discards a translation.
The wording of the TSDoc in `archive/src/non-translation-evidence.ts` before commit `1439ea1e5a`,
that a false block
"still discards a faithful translation whole",
overstated this and was corrected in that commit series
(finding F1 of the first independent re-check).
What was measured at two votes:
at document level,
nothing newly blocks in the artifacts the first re-check scanned.
Two facts that sentence leaves out (finding F7 of re-check 2):
- The `AkiraComplex` false-block slice itself is measured by no recorded artifact.
  Every `AkiraComplex.json` on disk
  (the run directories `translation-repair-runs`,
  `translation-repair-runs-pass13`,
  `translation-repair-runs-pass14`,
  `round-one-archive` and `round-two-archive` of the scratch copies)
  holds statuses `repaired`,
  `accepted`,
  `rejected` and `needs-human` and no block,
  and no artifact stores the wire-level critic votes,
  so the shape of two critics of seven on an English-epigraph slice cannot be replayed.
  The only record after the fix is `probe-akiracomplex.log` of the sentinel probe on 2026-07-24:
  both slices drew 7 of 7 critics heard and one non-translation vote each,
  which stands under a floor of two as it did under three.
  The risk of that shape at two votes is therefore unmeasured.
  The guards are the contradiction check and the good-translation anchor named in this entry,
  and the case "does not stand one vote under the floor (AkiraComplex false block)" pins the one-vote shape.
- At slice level,
  slices 68 and 69 of `XingZ6021` newly stand at two votes (B905),
  where they did not stand at three.

The follow-up test:
the block review of the archive asks three of its four seats after the selection fan-out change,
since a window of three self-votes decides.

## B903: a gate that heard one judge shipped a consolidation (N4)

Status:
fixed on 2026-10-25 (UTC),
red `86a1c58b32`,
fix `12ae74fc10`.

What `XingZ6021` showed:
five consolidations (slices 13,
14,
33,
44 and 84,
log lines 17855 to 20299) shipped after
`consolidate gate: 1/7 usable, settled on neither, ships standing`
followed by
`settled on neither over an ineligible standing, so the proposal the slate chose ships (1 usable ballot)`.
The gate itself could not ship `consolidated` on one ballot,
because `settleGateBallots` needs `CONSOLIDATE_GATE_QUORUM` (2) ballots for a rendering.
The path was `shipPastForfeitStanding` (`consolidate/src/consolidate-ineligible-standing.ts`),
which read the gate's neither over an ineligible standing as an indecision among judges
and shipped the slate's choice,
even when the neither was the want of a quorum.

Whether B835 had a fix that missed this path:
B835 (in [short benches and the last live seat](audit-ledger-short-benches-and-the-last-live-seat.md)) skipped rounds
that could not settle,
for cost,
and kept the four slices of `XingZ6020` (13,
44,
67 and 69) that read a slate reach of 1 with a withheld ineligible
standing "as they were,
because skipping would change the text that ships".
That fix addressed the purchase of rounds and not the shipping rule,
so it did not reach this path:
the rule that ships the slate's choice past an undecided gate was never given a minimum number of judges.

The rule now (round 1):
a gate that heard fewer than two usable ballots returns its outcome unchanged with
`UNHEARD_GATE_KEEPS_STANDING_FINDING`.
The standing stays,
and an ineligible standing then leaves the archive kept through `nothingValidShips`
(the owner's ruling of 2026-09-27,
"Keep archive,
ship").
A gate that preferred an ineligible standing still ships the slate's choice,
since that is a verdict of a gate that settled on `standing`.
Round 2 refines this for a slate pick that two or more slate ballots chose (B909).

What it cost,
for the owner (finding F5 of the independent re-check):
five slices ship the archive text that failed the deterministic publication rule,
with the finding `no-valid-wording` (the page ships with it reported),
where the base shipped the slate's consolidation.
The recorded log shows why the slate's choice cannot stand in for the gate there (B909).

Not changed in round 1,
named in B906 and settled in round 2 (B908):
the translate stage's `shipPastDecline` preference.
It shipped a declined slate's candidate by a fixed order (repair lane,
then translate lane,
then slate order),
whatever any judge named,
so it was not a lone-judge path (finding F6) but a path on which no judge decided.
The four slices B835 kept in `XingZ6020` look like that path,
and it rested on the owner's ruling of 2026-09-27,
"Preference + polish".

## B904: the cache versions that moved with the rules

Status:
fixed on 2026-10-25 (UTC),
one commit per stage.

The key of each stage hashes its version,
the shape of the run and the texts,
not the rule that reads the ballots,
so a record written before the rules would be served after.
The previous numbers were set in `170c1936f7` (slice,
by merge) and `2aceac0487` (translate,
refine and consolidate),
and no slice-cache directory was searched for a record written by code that holds those commits,
so records may stand at the previous numbers and the change cannot ride inside them.
- `SLICE_CACHE_VERSION` 46 to 47 in `c7d3c3e593` (`repair/src/repair-slice-key.ts`):
  the panel tally,
  the editor selection and the non-translation vote decide differently.
- `REFINE_CACHE_VERSION` 16 to 17 in `e11eecc500` (`repair/src/refine-slice-key.ts`):
  the refinement slate names a winner two ballots named and asks a window.
- `TRANSLATE_SLICE_CACHE_VERSION` 27 to 28 in `4183576804` (`translate-lane/src/translate-document-contract.ts`):
  the candidate selection and the removed run-off exceptions.
- `CONSOLIDATE_CACHE_VERSION` 34 to 35 in `b1cf1ad117` (`artifact/src/consolidate-key.ts`):
  the slate selection and the gate that ships nothing on one ballot.

Stay,
with the reason:
the lane contest (it already decided on two ballots and keeps its floor),
the consistency,
page title and pairing versions (their sheets and rules do not read these floors),
and the artifact generation (the artifact readers replay consistency,
contest and naturalness,
not the tally or the selection).

Checked by `git log --extended-regexp -G '_CACHE_VERSION = [0-9]+' b975d68aaf..HEAD`,
which lists exactly those four commits,
and by the positive control over `c265e8df95~1..c265e8df95`,
which lists `c265e8df95`.

## B905: the replay of the recorded ballots of XingZ6021

Status:
measured on 2026-10-25 (UTC),
keyless,
no provider called.

Method:
the artifact `XingZ60.json` of the pass records,
per adjudicated issue,
each panelist's vote,
weight and reason.
Each claim became a one-member cluster,
and its ballots went through the tally of the base `b975d68aaf` (copied by `git show`)
and through the tally of this tree.
The positive control:
the base tally reproduces the recorded status for 449 of 449 claims in the 341 issues.
Not recorded and so not replayed:
the severity re-grades of supporters (the claim's filed severity stands in),
which of the 449 claims a panel merged,
and the ballots of the slate and the editor selection,
which the artifact does not keep.

Figures,
by claim:
- `needs-human` 295 at the base,
  55 now:
  240 became decided (219 accepted,
  21 rejected).
- Every claim the base decided is decided the same way:
  135 accepted and 19 rejected,
  none to `needs-human`.
- All 240 changes had exactly two answering panelists or more;
  none had fewer than two.
  Every issue in the artifact has at least two answering panelists (224 with two,
  17 with three,
  100 with four or more).
- The 240 newly decided:
  accepted by severity 5 critical,
  102 major,
  112 minor;
  rejected 3 major and 18 minor.
- What stays `needs-human` is all splits (a 1 to 1 vote among two answering or no majority among more).
- By issue (first member's status,
  four merged issues would split by member):
  234 `needs-human` at the base,
  49 now.

What slice 67's claims become (nine claims,
all at a summed weight of 2):
- Accepted,
  4:
  the two omission claims of the unkind world (2 supported of 2 answering each),
  and the two claims of the invented Metal Gear Solid 2 quotation (2 of 2).
- Rejected,
  3:
  "her life was short" (twice) and a closing assertion of remembrance;
  the panel found 她的一生虽然短暂 in the original,
  so rejecting those is right.
- Stay `needs-human`,
  2:
  the two "warrior on a battlefield" claims (1 supported,
  1 unsupported).
Slices 68 and 69:
  all four critical non-translation claims accepted (2 of 2).
Slice 70:
  six of seven claims accepted,
  one 1 to 1 tie stays.

Downstream,
with the accepted issues of the new tally fed to the code of this tree:
- The archive-dispute reader (`archive/src/archive-dispute.ts`,
  which reads only accepted issues,
  owner classes 107 and 176):
  slices with a dispute 3 at the base,
  26 now;
  slice 67 has 4 disputing claims,
  slices 68 and 69 have 2 each,
  slice 70 has 5.
  Each is `standInEligible=false` with the refusal `archive-wording` in this replay,
  because the recorded chunks did not change (the editor never ran);
  whether a stand-in is eligible depends on what the editor writes and the checkers confirm,
  which no keyless replay knows.
- The editor's envelope cut (`shared-contract/src/patch-model.ts`,
  called at `repair/src/repair-chunk.ts:316-334`):
  slices with at least one envelope 18 at the base,
  78 now (178 envelopes in all);
  slice 67 gets two envelopes (offsets 47 to 114 serving two issues,
  and 195 to 267 serving one),
  slice 68 one (0 to 152),
  slice 69 one (0 to 109),
  and slice 70 three.
  The "nothing to edit,
  unchanged" of slice 67 would not happen.
- The non-translation standing:
  slices 68 and 69 (two votes each,
  both critics heard,
  not contradicted)
  stand at two and did not at three.
  The document dominance stays unblocked:
  835 of 32,887 target characters stand (2 slices),
  and 73 slices anchor translated content.
- The gate (`settleGateBallots` then the base rule and the rule of this tree,
  on the recorded ballots):
  slices 13,
  14,
  33,
  44 and 84 shipped `consolidated` on one usable ballot at the base
  and keep the standing now,
  with the finding `unheard-gate-keeps-standing`;
  slice 101,
  which drew two usable ballots for `consolidated`,
  still ships it.
  Terminal `consolidated` on the gate path:
  6 at the base,
  1 now.
- The slate of slice 67 (the translate lane's faithful text lost to the standing for one judge) stays declined:
  the lone DeepSeek judge cannot decide,
  and the slate's floor of two stays (B906).

## B906: the floors that stay and the choices not taken

Status:
recorded on 2026-10-25 (UTC).

Stay,
because one judge alone never decides:
- The lane contest skips a slice where one of seven seats is reachable against a floor of two
  (`XingZ6021` laneSelection,
  `benchBelowFloor` on 71 of 101 slices).
- The slate skips and declines where the slate reach is under two (`consolidate/src/consolidate-slice-buy.ts`),
  so the lone DeepSeek judge that called the archive a non-translation,
  one ballot under the floor of two,
  decides nothing.
- A leader one judge alone named is declined in the candidate selection and the gate,
  and a declined round ships a candidate only when two or more judges named it (B908).

Relation between the two-ballot rule and the stage quorum:
two answering judges decide the panel tally,
the selection,
the non-translation block and the gate.
The stage quorum (`reachableQuorum().needed`) is how many voices a round waits for before it closes.
It is a majority of the seated bench on a full bench and falls to two on a short bench after the grace (B816),
which is why `silentStages` stayed empty on every chunk of `XingZ6021`:
each stage heard its reduced quorum of two.
It still marks `silentStages`,
and it still gates the verdicts of the derivability probe and the restoration judge.

Settled by the owner on 2026-10-25 (round 2),
after the two readings of each question were put to the owner:
- A gather quorum above two (a majority of a bench of five or more) and the pre-phase bench waits
  (`run/src/corpus-run/run-seats-wait.ts` and its siblings):
  "Keep waiting for half".
  Both stay;
  they are counts of seats that a round waits for,
  and on a short bench the quorum already falls to two.
- The checker count of three (`MINIMUM_CHECKER_COUNT` in `assembly/src/repair-contract.ts`,
  line 468 at the head;
  the first census gave the base's line 508):
  "Allow two checkers".
  It is two now (B907).
- The translate stage's `shipPastDecline` preference:
  "Two judges must name it" (B908).

Not changed,
with the reading kept:
- A self-vote.
  The brief says a self-vote's half weight "counts toward shares but not toward the count of answering judges".
  The owner's ruling of 2026-10-20,
  "Count self-votes",
  rejected "Exclude self-votes".
  This tree follows the ruling:
  a self-vote is one answering judge and its half weight only ranks candidates.
- The assumption in the window trial (B901).
- The tie of a challenge round across every offered candidate
  (`breakRunoffTie` in `translate-stage/src/translate-runoff-tie.ts`,
  the tenth addendum of
  `doc/decision/translation-repair-ineligible-standing.md`) shipped one by the fixed preference
  when this entry was written.
  It is the same class of path as the declined round,
  a choice on which no judge decided,
  and the lead's later ruling,
  settled by the owner's "Two judges must name it",
  put it under the same rule (B914).

Edges the two-ballot count does not cover,
neither new in kind (finding F7):
- A `source-defect` vote decides at a share of one third (`adjudication/src/tally-votes.ts`).
  With exactly two answering judges at equal weight,
  one `source-defect` vote against one supported vote is a share of one half and decides `source-defect`.
  That outcome is protective (no edit follows) and a case pins it ("COUNTS an ambiguous or source-defect answer").
- `disposeMerge` in the same file merges members on one opining panelist.
  Authority stays per member through `partitionGradedMembers`.

Counts the first census omitted (finding F8):
- `AGREEMENT_NEEDED = 2` in `stage/src/pair-sections-stage.ts` and `slice-reading/src/pair-blocks-read-outcomes.ts`:
  a two-ballot rule on the readers that agree on a pairing,
  and it stays.
- `NON_TRANSLATION_CONTRADICTION_MIN = 8` in `archive/src/non-translation-evidence.ts`:
  a count of claims anchored in the target,
  not a floor on judges,
  and it stays.
- The count of one in the opposite direction:
  a gather quorum below two,
  which a seated bench of one or two seats has by design (`stage/src/roster-quorum-size.ts`).
  A consistency round no longer decides on it (B911).
  The derivability probe (`repair/src/derivability-probe.ts`) and the restoration judge
  (`repair/src/restoration-judge.ts`) still read their verdicts at that quorum,
  and are left alone because they feed the benchmark scorecard only (`repair/src/repair-scorecard.ts`).

Single-voice effects that remain,
accepted (finding F8 of re-check 2),
each with its reason:
- `regressed` in `adjudication/src/tally-resolution.ts` keeps no ballot floor:
  a lone `worse` ballot,
  where the other checker cast no verdict on the issue,
  marks a patched candidate regressed
  (`worse > fixed + notFixed`),
  and `adjudication/src/select-candidate.ts` then ranks it below a candidate that regressed nothing.
  Accepted because it only demotes and never ships text,
  with one checker `worse` against one `fixed` or `notFixed` it ties and does not regress,
  and the checkers voted `worse` 55 times over the recent runs without `regressed` firing (ledger L1).
  It is more reachable at two checkers (B907) than at three.
- The derivability probe (`repair/src/derivability-probe.ts`) decides a seed on one cast verdict once its quorum is met.
  Accepted because its verdicts feed the benchmark scorecard only (`repair/src/repair-scorecard.ts`) and no page changes.
- The restoration judge (`repair/src/restoration-judge.ts`) reads its verdict the same way.
  Accepted for the same reason.
- A `source-defect` vote decides at a share of one third,
  and `disposeMerge` merges members on one opining panelist.
  Accepted because the outcome is protective (no edit follows) and authority stays per member.

The rule for next time:
when a rule is about how many judges decide,
write the count where the code reads it,
and list the other counts that look alike with the reading that keeps them.
