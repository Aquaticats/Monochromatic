# Audit ledger: the weight floors removed, round 5

Round 5 of the work recorded in
[the weight floors removed and two ballots deciding](audit-ledger-weight-floors-removed-and-two-ballots-decide.md),
which holds B900 to B906,
[rounds 2 and 3](audit-ledger-weight-floors-removed-rounds-two-and-three.md),
which holds B907 to B916,
and [round 4](audit-ledger-weight-floors-removed-round-four.md),
which holds B917 to B919:
the answer to the fourth independent re-check
(0 major findings,
two minor findings F1 and F2,
the nits,
and one question Q-A),
and the owner's answer to Q-A.
Ids B920 and B921 belong to the DeepSeek pricing entries that main took,
so this round starts at B922.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## Round 5 at a glance

The merge of `origin/main` at `a50c2875a2` is `551a7320ea`.
It auto-merged the two ledger indexes and nothing else conflicted.
Main changed no cache version.

The owner's answer to Q-A is B922,
the minor findings are B923 and B924,
and the nit that the whole title round is asked again is B925.
The cache versions that moved are B926.

The commit-message slips of the fourth re-check are listed in the hand-back and nothing is posted.

## B922: an edit is stripped and a rewrite rolled back only on two ballots (round 5)

Status:
fixed on 2026-10-25 (UTC),
red `6cfcfbebb2`,
fix `51f65333b9`,
cache versions `4983330f74` (repair slice 49) and `4eab4ba8db` (refine 19).

Question Q-A of the fourth re-check:
the strip of a patch's unconfirmed edits (L3,
`repair/src/repair-worse-strip.ts`)
and the rollback of a refinement (L11,
`repair/src/refine-recheck.ts`)
acted on one `worse` ballot by the owner's rulings of 2026-09-28,
and the rule that one judge alone never decides is older than those rulings but was restated on 2026-10-25.
The owner answered on 2026-10-25,
"Two ballots for undo too":
the two-ballot rule covers both.

The rule now:
an edit is stripped,
and a rewrite is rolled back on an open issue,
only where the issue's tally flags it `regressed`,
which is when two or more ballots name `worse`
and the weight behind `worse` exceeds the weight behind `fixed` and `not-fixed` together.
There is one definition,
`regressed` in `tallyResolutionChecks` (`adjudication/src/tally-resolution.ts`),
and three sites read it:
the candidate ranking (`regressedKnownIssues` in `repair/src/chunk-measure.ts`),
the strip,
and the rollback.
`regressed` excludes `resolved`
(a `worse` weight that exceeds the rest cannot sit with a `fixed` weight that exceeds the rest),
so the strip no longer asks `resolved` as well.
The finding of a strip now says two or more worse ballots outweighing the rest.
A confirmed issue the rewrite broke still rolls the slice back on `resolved` alone,
as before:
that is a ruling about a repair that stopped holding,
not about a ballot count.

The readers of one `worse` ballot,
found by searching the translation-repair packages for `worse > 0`,
`worsened`,
`.worse`,
`worseBallots`,
`'worse'` and `regressed`:
- `repair-worse-strip.ts` and `refine-recheck.ts` were the two that acted on one ballot,
  and they are the two changed.
- `chunk-measure.ts` reads `regressed`.
- `bench/src/corpus-run/checker-sensitivity-print.ts` prints the tally's `worse` weight and decides nothing.
- `assembly/src/assembly-regressions.ts` has a function named `worsened`,
  which counts parse findings of a page and reads no ballot.
- `adjudication/src/resolution-wire.ts` and `tally-resolution.ts` name the verdict and count it.
- `refine-recheck.ts` keeps a local list of confirmed issues that are no longer resolved,
  which reads `resolved` and `heardEnough`.
  No other site acts on one `worse` ballot.

The tests:
- Red `6cfcfbebb2` fails eight cases,
  seven for their reason and one that the fix removed:
  a lone `worse` beside a `fixed` and a `not-fixed` on an open issue (the recheck),
  the same beside two `not-fixed` (the recheck and the phase that runs it),
  two `worse` level with two `fixed` on the recheck
  (a case the fix removed on a reason that did not hold,
  restored in round 6 with a verdict for the fifth seat (B927)),
  and for the strip:
  a lone `worse` beside one `fixed`,
  beside two `not-fixed`,
  and two `worse` level with two `fixed`
  (tallies from the real `tallyResolutionChecks`),
  and the whole pipeline with one `worse` among three checkers.
  The positive controls pass on both commits:
  two `worse` against one `fixed` or one `not-fixed` strips and rolls back.
  The commit message of the red commit says eleven cases fail;
  eight do.
- The recheck cases that rolled back on one ballot now pass the round on one ballot
  and roll back on two,
  in `refine-recheck.unit.test.ts`,
  `refine-slice-settle.unit.test.ts` and `refine-phase.unit.test.ts`.
- Two `worse` level with two `fixed` is tested at the strip and at the tally (`tally-resolution.unit.test.ts`).
  A case of it for the recheck was written and removed in this round,
  on the reason that the scripted verdicts reached only three ballots.
  That reason did not hold (finding M-A of the fifth re-check):
  the window of quorum plus one rotated past one scripted seat and asked the fifth,
  which had no verdict,
  and a verdict for the fifth seat reaches the level case.
  The case is restored in round 6 (B927).

The replay (`fix5/replay-undo-real.ts`,
output `fix5/replay-undo-real.out`)
runs the real functions of this worktree with a scripted client that reaches no provider.
The 99 recorded checker readings of `XingZ6021` hold no `worse` ballot,
so neither rule acts on any of them:
0 under the old rule (not resolved and `worse` greater than 0) and 0 under the new,
and the real-data null cannot move by construction.
The controls move as ruled.
For the strip:
a lone `worse` beside one `fixed` and beside two `not-fixed` strip nothing,
two `worse` against one `fixed` strip,
two against two strip nothing.
For the rollback:
a lone `worse` beside a `fixed` and a `not-fixed` and beside two `not-fixed` pass the round,
two `worse` against one `fixed` roll the slice back,
and two against two (four ballots asked on five seats) pass.

The cache versions:
the repair slice (48 to 49) holds the patch that kept or stripped an edit,
and the refine version (18 to 19) holds the rollback decision.
The TSDoc of each key carries the reason.

## B923: the sentence about a declined round is told per round (round 5)

Status:
fixed on 2026-10-25 (UTC),
red `70d01b2a51`,
fix `16af3a8d0a`,
cache versions `8ec4b815d4` (translate slice 33) and `948cb41cae` (consolidate 40).

Finding F1 of the fourth re-check.
The sentence B918 gave to a judge over wording that cannot ship promised the ship or the stay.
That is true only of the challenge of a caller that ships past a decline.
The places the sentence is rendered,
and what a decline does in each,
traced in `translate-stage/src/translate-judge.ts`,
`translate-runoff-tie.ts`,
`translate-retry.ts`,
`translate-stage-repair.ts` and `consolidate/src/consolidate-settle-judge.ts`:
- The first ask of a pair (`initial-selection`),
  for the translate lane's first call,
  its follow-up call and the consolidation of a withheld standing.
  A decline ships nothing and keeps nothing:
  `settleAbsentDecline` is called with `challenged: false`,
  no tie is broken,
  and it raises the absence,
  which `judgeSlateWithRetry` answers by asking the same panel again
  (over only the candidates a tie backed where a tie narrows the question).
- The challenge of the consolidation of a withheld standing,
  which is always challenged,
  and of the translate lane's follow-up call.
  They ship past a decline:
  the preferred candidate two or more judges named,
  or the wording the archive has where none drew two ballots.
- The challenge of the translate lane's first call
  (`shipPastDecline: input.incumbentWithheld && (followupEvidence !== undefined)` is false there).
  A decline ships only a tie across every candidate that two or more judges named
  (`breakRunoffTie`,
  which needs no `shipPastDecline`),
  and otherwise raises the absence,
  which the lane answers with a follow-up round of new proposals written against the rejected renderings.
  It never keeps the wording in place on its own.

`declineConsequenceFor` takes the round
(`responsibility` and `shipPastDecline`)
and returns the sentence that is true there:
`SETTLES_NOTHING_AND_ASKS_AGAIN` for the first ask,
`SHIPS_WHAT_TWO_JUDGES_NAMED` (unchanged) for the challenge that ships past a decline,
and `SHIPS_A_LEVEL_TIE_ELSE_PRODUCES_AGAIN` for the challenge that does not.
`judgeTranslateSlate` passes both.
No sentence has three stacked negatives or a promise another round keeps.

The tests render the real sheets:
`translate-decline-sentence-per-round.unit.test.ts` runs `judgeSlateWithRetry` with every judge declining
and reads the sentence of each round by its clauses,
and `slate-decline-two-judges-name-it.unit.test.ts` reads the consolidation's first and last sheets
and the four rounds of the translate lane over wording the floor refuses
(asked again,
sent back,
asked again,
ships).
The red commit fails seven cases for their reason (every sheet said the ship or the stay),
and the control,
the challenge that ships,
passes on both commits.

Known limit,
closed in round 6 (B928).
A tie that narrows the question asks the panel again over the candidates it backed,
before anything ships.
The first ask's sentence said so,
and the challenge sentences stated what the round ships or keeps when no run-off is left
without mentioning a run-off.
They now say that a tie backing only some candidates sends the panel to a run-off first.

## B924: the clause reader refuses swapped outcomes and a decline said to keep the wording (round 5)

Status:
fixed on 2026-10-25 (UTC),
red `4d50299739`,
fix `6d1c1200c8`,
the rounds added in the red commit of B923.

Finding F2 of the fourth re-check.
The clause reader of B918 read each clause as ordered phrases inside one part cut at `;`,
and the lead of the sentence shared a part with the first outcome.
So the sentence with the two outcomes swapped passed all five clauses,
because the lead's "no candidate" stood in the part of the swapped stay,
and a sentence that said declining "keeps the wording in place" passed the first clause.

The reader now cuts the text at every `;`,
`:` and full stop,
so the lead and each outcome are parts of their own,
and a clause carries refused phrases:
the stay may not say the preferred ships,
the ship may not say none was named,
that the wording stays,
or that the ship waits on an equal weight,
and the lead may not say `keeps`.
A round also refuses the promises of the others
(the first ask may not promise a ship or a stay;
a challenge may not say a decline settles nothing and asks again,
as reworded in round 6 (B928)).
The header states what it refuses and what it does not read:
a promise no clause names,
and a negation outside a short list since round 6 (B931).

The tests of the red commit:
the swapped sentence and the "keeps" sentence fail on that commit and pass the refusal on the fix;
the reversed lanes and a rewording that keeps every clause pass on both
(the reversed lanes were refused already,
and the rewording is the control that the refusals read meaning and not the constant's words).
The reader and its clauses are split into three fixture files for the line bound.

## B925: the whole page title round is asked again (round 5)

Status:
recorded,
no code change.

A nit of the fourth re-check on Q1 of round 4.
`passPageTitles` stores a round only when every title it asked is settled
(B919).
So the whole round is asked again on the next run,
not only its unsettled titles:
a page with one title the bench can never settle is asked on every run,
and a title that settled on one run can settle differently on another.
This is the known cost of the round-level storage ruled for Q1,
and the page title version (9) holds the records of settled rounds only.

## B926: the cache versions of round 5 (round 5)

Status:
done on 2026-10-25 (UTC).

Four versions moved,
each in its own commit with its reason in the key's TSDoc:
- repair slice 49,
  `4983330f74` (B922).
- refine 19,
  `4eab4ba8db` (B922).
- translate slice 33,
  `8ec4b815d4` (B923).
- consolidate 40,
  `948cb41cae` (B923).

The pinned keys moved with them:
`08057e3894` (repair slice),
`daf30b1435` (refine),
`a8cca93b15` (translate slice without a window)
and `a5be335384` (consolidate).
The lane contest (18),
pairing (3),
page title (9) and consistency (9) versions stay:
none of the four stores a checker reading the strip or the rollback reads,
renders a decline sentence,
or stores a title round differently.
Whether a record stands at the older numbers is not known,
and no slice-cache directory was searched.
