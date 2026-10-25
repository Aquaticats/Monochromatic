# Audit ledger: the weight floors removed and two ballots deciding

The owner's rulings of 2026-10-25 on the corpus pass `XingZ6021`:
no summed-weight floor stands between two answering judges and a decision,
a claim or contest is decided once two or more judges answered it,
and one judge alone never decides.
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
and the archive's text shipped with an invented quotation from *Metal Gear Solid 2*
and two paragraphs the source does not hold.
Across the run 234 of 341 adjudicated issues ended `needs-human`.

The rule now:
`decideStatus` in `adjudication/src/tally-votes.ts` counts the panelists that answered with a vote that carries weight
and lands `needs-human` below `MIN_PANEL_BALLOTS` (2).
`minBallotWeight` is gone from `AdjudicationConfig`
(`shared-contract/src/adjudicate-model.ts`),
and the decision, source-defect and run-off ratios stay.
A heavy weight does not turn one answer into two:
the red case `tally-two-ballots.unit.test.ts` holds one panelist at a weight of 5 and expects `needs-human`.

The reading adopted, open to the owner's veto:
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
the run-off of a condemned incumbent, and the condemned-fallback that seated a challenge proposal under the minimum.
The roster guard of `assertJudgeableEditorRoster` did weight arithmetic to refuse a roster that could not reach 2,
and the first-round fan-out asked the whole bench where the self-votes of a window alone could not reach 2
(a window of three halves on a bench of four).

The rule now:
`MIN_SELECTION_BALLOTS` (2, declared in `house-rule/src/candidate-select-model.ts`) ballots name a winner,
whatever their weight.
A self-vote counts as one answering judge and its half weight only ranks candidates
(the owner's ruling of 2026-10-20, "Count self-votes",
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
this is an assumption about what the trial meant, recorded here because the default window would have changed its arms.

The behaviour that changed and is visible:
a condemned incumbent that leads with two ballots is selected,
as it already was at a summed weight of 2 or more.

## B902: three non-translation votes were needed to block a slice, and two critics could not

Status:
fixed on 2026-10-25 (UTC),
red `90cb72fab1`,
follow-up test `d42e79be74`,
fix `1439ea1e5a`.

What `XingZ6021` showed:
slices 68 and 69 each drew two wire-level critical non-translation votes,
from both critics heard,
and the floor of three (`NON_TRANSLATION_BLOCK_VOTES`, `archive/src/non-translation-evidence.ts`) left the votes not standing.

The rule now:
two votes stand.
Two votes cannot come from fewer than two critics heard,
so one critic alone can never block.
The floor of three had been set after the first real-corpus false block (`AkiraComplex`, commit `72f5537c1`),
which discarded a faithful translation on two votes from two of seven critics on an English-epigraph slice.
What guards that shape now is the contradiction check
(`NON_TRANSLATION_CONTRADICTION_MIN`, eight validated content-critique claims anchored in the target)
and the good-translation anchor that vetoes a document-wide block.
This is an owner-visible risk: a false block still discards a faithful translation whole.

The follow-up test:
the block review of the archive asks three of its four seats after the selection fan-out change,
since a window of three self-votes decides.

## B903: a gate that heard one judge shipped a consolidation (N4)

Status:
fixed on 2026-10-25 (UTC),
red `86a1c58b32`,
fix `12ae74fc10`.

What `XingZ6021` showed:
five consolidations (slices 13, 14, 33, 44 and 84, log lines 17855 to 20299) shipped after
`consolidate gate: 1/7 usable, settled on neither, ships standing`
followed by
`settled on neither over an ineligible standing, so the proposal the slate chose ships (1 usable ballot)`.
The gate itself could not ship `consolidated` on one ballot,
because `settleGateBallots` needs `CONSOLIDATE_GATE_QUORUM` (2) ballots for a rendering.
The path was `shipPastForfeitStanding` (`consolidate/src/consolidate-ineligible-standing.ts`),
which read the gate's neither over an ineligible standing as an indecision among judges
and shipped the slate's choice, even when the neither was the want of a quorum.

Whether B835 had a fix that missed this path:
B835 (in [short benches and the last live seat](audit-ledger-short-benches-and-the-last-live-seat.md)) skipped rounds
that could not settle,
for cost,
and kept the four slices of `XingZ6020` (13, 44, 67 and 69) that read a slate reach of 1 with a withheld ineligible
standing "as they were, because skipping would change the text that ships".
That fix addressed the purchase of rounds and not the shipping rule,
so it did not reach this path:
the rule that ships the slate's choice past an undecided gate was never given a minimum number of judges.

The rule now:
a gate that heard fewer than two usable ballots returns its outcome unchanged with
`UNHEARD_GATE_KEEPS_STANDING_FINDING`.
The standing stays,
and an ineligible standing then leaves the archive kept through `nothingValidShips`
(the owner's ruling of 2026-09-27, "Keep archive, ship").
A gate that preferred an ineligible standing still ships the slate's choice,
since that is a verdict of a gate that settled on `standing`.

Not changed, named in B906:
the translate stage's `shipPastDecline` preference,
where a declined slate with a withheld standing ships its preferred candidate even when the leader was named by one judge.
The four slices B835 kept in `XingZ6020` look like that path,
and it rests on the owner's ruling of 2026-09-27, "Preference + polish".

## B904: the cache versions that moved with the rules

Status:
fixed on 2026-10-25 (UTC),
one commit per stage.

The key of each stage hashes its version, the shape of the run and the texts,
not the rule that reads the ballots,
so a record written before the rules would be served after.
The previous numbers were set in `170c1936f7` (slice, by merge) and `2aceac0487` (translate, refine and consolidate),
and no slice-cache directory was searched for a record written by code that holds those commits,
so records may stand at the previous numbers and the change cannot ride inside them.
- `SLICE_CACHE_VERSION` 46 to 47 in `c7d3c3e593` (`repair/src/repair-slice-key.ts`):
  the panel tally, the editor selection and the non-translation vote decide differently.
- `REFINE_CACHE_VERSION` 16 to 17 in `e11eecc500` (`repair/src/refine-slice-key.ts`):
  the refinement slate names a winner two ballots named and asks a window.
- `TRANSLATE_SLICE_CACHE_VERSION` 27 to 28 in `4183576804` (`translate-lane/src/translate-document-contract.ts`):
  the candidate selection and the removed run-off exceptions.
- `CONSOLIDATE_CACHE_VERSION` 34 to 35 in `b1cf1ad117` (`artifact/src/consolidate-key.ts`):
  the slate selection and the gate that ships nothing on one ballot.

Stay, with the reason:
the lane contest (it already decided on two ballots and keeps its floor),
the consistency, page title and pairing versions (their sheets and rules do not read these floors),
and the artifact generation (the artifact readers replay consistency, contest and naturalness,
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
the artifact `XingZ60.json` of the pass records, per adjudicated issue, each panelist's vote, weight and reason.
Each claim became a one-member cluster,
and its ballots went through the tally of the base `b975d68aaf` (copied by `git show`)
and through the tally of this tree.
The positive control: the base tally reproduces the recorded status for 449 of 449 claims in the 341 issues.
Not recorded and so not replayed: the severity re-grades of supporters (the claim's filed severity stands in),
which of the 449 claims a panel merged,
and the ballots of the slate and the editor selection,
which the artifact does not keep.

Figures, by claim:
- `needs-human` 295 at the base, 55 now: 240 became decided (219 accepted, 21 rejected).
- Every claim the base decided is decided the same way: 135 accepted and 19 rejected, none to `needs-human`.
- All 240 changes had exactly two answering panelists or more; none had fewer than two.
  Every issue in the artifact has at least two answering panelists (224 with two, 17 with three, 100 with four or more).
- The 240 newly decided: accepted by severity 5 critical, 102 major, 112 minor; rejected 3 major and 18 minor.
- What stays `needs-human` is all splits (a 1 to 1 vote among two answering or no majority among more).
- By issue (first member's status, four merged issues would split by member): 234 `needs-human` at the base, 49 now.

What slice 67's claims become (nine claims, all at a summed weight of 2):
- Accepted, 4: the two omission claims of the unkind world (2 supported of 2 answering each),
  and the two claims of the invented *Metal Gear Solid 2* quotation (2 of 2).
- Rejected, 3: "her life was short" (twice) and a closing assertion of remembrance;
  the panel found 她的一生虽然短暂 in the source's later body, so rejecting those is right.
- Stay `needs-human`, 2: the two "warrior on a battlefield" claims (1 supported, 1 unsupported).
Slices 68 and 69: all four critical non-translation claims accepted (2 of 2).
Slice 70: six of seven claims accepted, one 1 to 1 tie stays.

Downstream, with the accepted issues of the new tally fed to the code of this tree:
- The archive-dispute reader (`archive/src/archive-dispute.ts`, which reads only accepted issues, owner classes 107 and 176):
  slices with a dispute 3 at the base, 26 now;
  slice 67 has 4 disputing claims, slices 68 and 69 have 2 each, slice 70 has 5.
  Each is `standInEligible=false` with the refusal `archive-wording` in this replay,
  because the recorded chunks did not change (the editor never ran);
  whether a stand-in is eligible depends on what the editor writes and the checkers confirm, which no keyless replay knows.
- The editor's envelope cut (`shared-contract/src/patch-model.ts`, called at `repair/src/repair-chunk.ts:316-334`):
  slices with at least one envelope 18 at the base, 78 now (178 envelopes in all);
  slice 67 gets two envelopes (offsets 47 to 114 serving two issues, and 195 to 267 serving one),
  slice 68 one (0 to 152), slice 69 one (0 to 109), and slice 70 three.
  The "nothing to edit, unchanged" of slice 67 would not happen.
- The non-translation standing: slices 68 and 69 (two votes each, both critics heard, not contradicted)
  stand at two and did not at three.
  The document dominance stays unblocked: 835 of 32,887 target characters stand (2 slices),
  and 73 slices anchor translated content.
- The gate (`settleGateBallots` then the base rule and the rule of this tree, on the recorded ballots):
  slices 13, 14, 33, 44 and 84 shipped `consolidated` on one usable ballot at the base
  and keep the standing now, with the finding `unheard-gate-keeps-standing`;
  slice 101, which drew two usable ballots for `consolidated`, still ships it.
  Terminal `consolidated` on the gate path: 6 at the base, 1 now.
- The slate of slice 67 (the translate lane's faithful text lost to the standing for one judge) stays declined:
  the lone DeepSeek judge cannot decide, and the slate's floor of two stays (B906).

## B906: the floors that stay and the choices not taken

Status:
recorded on 2026-10-25 (UTC).

Stay, because one judge alone never decides:
- The lane contest skips a slice where one of seven seats is reachable against a floor of two
  (`XingZ6021` laneSelection, `benchBelowFloor` on 71 of 101 slices).
- The slate skips and declines where the slate reach is under two (`consolidate/src/consolidate-slice-buy.ts`),
  so the lone DeepSeek judge that called the archive a non-translation, one ballot under the floor of two, decides nothing.
- A leader one judge alone named is declined in the candidate selection and the gate.

Relation between the two-ballot rule and the stage quorum:
two answering judges decide the panel tally, the selection, the non-translation block and the gate.
The stage quorum (`reachableQuorum().needed`) is how many voices a round waits for before it closes.
It is a majority of the seated bench on a full bench and falls to two on a short bench after the grace (B816),
which is why `silentStages` stayed empty on every chunk of `XingZ6021`: each stage heard its reduced quorum of two.
It still marks `silentStages`, and it still gates the verdicts of the derivability probe and the restoration judge.

Not changed, with both readings, for the owner:
- A gather quorum above two (a majority of a bench of five or more).
  Reading one: it is a count of judges, not a weight, so it is not a weight floor and stays.
  Reading two: "Every weight floor" also means a round should close and decide on two voices.
  On a short bench it already is two.
- The pre-phase bench waits (`run/src/corpus-run/run-seats-wait.ts` and its siblings) hold a phase for a quorum of seats.
  Same two readings.
- The checker count of three (`MINIMUM_CHECKER_COUNT` in `assembly/src/repair-contract.ts`).
  Reading one: it sizes a roster of checkers, a count of seats, not a floor on summed weight.
  Reading two: it is a floor of three on judges where two ballots now decide.
- A self-vote.
  The brief says a self-vote's half weight "counts toward shares but not toward the count of answering judges".
  The owner's ruling of 2026-10-20, "Count self-votes", rejected "Exclude self-votes".
  This tree follows the ruling: a self-vote is one answering judge and its half weight only ranks candidates.
- The translate stage's `shipPastDecline` preference (see B903).
- The assumption in the window trial (B901).

The rule for next time:
when a rule is about how many judges decide, write the count where the code reads it,
and list the other counts that look alike with the reading that keeps them.
