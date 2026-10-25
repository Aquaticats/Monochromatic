# Audit ledger: the weight floors removed, round 6

Round 6 of the work recorded in
[the weight floors removed and two ballots deciding](audit-ledger-weight-floors-removed-and-two-ballots-decide.md),
which holds B900 to B906,
[rounds 2 and 3](audit-ledger-weight-floors-removed-rounds-two-and-three.md),
which holds B907 to B916,
[round 4](audit-ledger-weight-floors-removed-round-four.md),
which holds B917 to B919,
and [round 5](audit-ledger-weight-floors-removed-round-five.md),
which holds B922 to B926:
the answer to the fifth independent re-check
(0 major findings,
four minor findings M-A to M-D,
and the nits).
Ids B920 and B921 belong to main's DeepSeek pricing entries,
so this round continues at B927.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## Round 6 at a glance

The merge of `origin/main` at `b6927f7ba9` is `373eae358c`,
with `a50c2875a2` as the merge base.
Two keep-both conflicts arose,
both in the ledger indexes:
main renamed the B899 follow-up entries and retitled the entry about holds a test registers,
and the branch lists B900 to B919 and B922 to B926 in the same hunks (finding M-C of the fifth re-check).
No source or test file conflicted,
and main changed no cache version.

The rollback's level case is B927 (M-A),
the run-off the challenge sentences omitted is B928 (M-B),
the same judges that a round does not promise is B929 (M-D),
the sentences the sheet-wide scans never read is B930,
the nits and the clause reader's negations are B931,
and the cache versions that moved are B932.

The commit-message slips of the fifth re-check are listed in the hand-back and nothing is posted.

## B927: the rollback's two-worse-level-with-two-fixed case is reachable and tested (round 6)

Status:
fixed on 2026-10-25 (UTC),
test `d4b3ff01e1`,
no code change.

Finding M-A of the fifth re-check.
Commit `51f65333b9` removed the recheck's case of two `worse` level with two `fixed` on the five-seat bench,
saying the harness could not reach it and that the tally and strip cases cover it.
Both statements were wrong.
A bench of five seats asks the window of quorum plus one,
which is four seats (`stage/src/stage-fanout-window.ts`),
and all four can answer,
so two against two is a round production can reach.
The tally and strip cases cover the shared `regressed` flag and the strip,
not the rollback site in `repair/src/refine-recheck.ts`.
The removed case failed because the scripted verdicts named four seats,
the window (rotated by the sheet text) left one of them out and asked the fifth,
which had no verdict.
With a verdict scripted for the fifth seat the round hears four ballots,
and the tally is `fixed 2, notFixed 0, worse 2, resolved false, regressed false`.

The case is restored with that verdict.
Three seats are scripted `worse` and two `fixed`,
so the case needs the rotation of this sheet to leave out one of the `worse` seats.
A change of sheet text that moves the rotation to leave out a `fixed` seat gives three `worse` against one `fixed`,
which regresses,
so the case then fails and does not pass for the wrong reason.

The evidence is a mutation,
since the code was right and the case is green on this commit by design.
A rollback that reads `worse >= 2` (half the rule,
with no clause that the `worse` weight must exceed the rest) survives the test file before this commit:
only the two cases about the short bench fail,
as they do with no mutation on a source copy.
The restored case kills the mutant:
a third case fails with `retained: false` where `true` is expected,
and on the real rollback it passes.
The test reads the built `dist`,
so the runs used copies of the test that read `src/index.ts`
(`fix6/m-a-head.log`,
`fix6/m-a-mutant-headtests.log`,
`fix6/m-a-mutant-newtests.log`,
`fix6/m-a-control-newtests.log`).

The sentence of B922 that said the case could not be reached is corrected in place.

Corrected in round 7 (B933):
the sentence of this entry that the case needs the rotation of the sheet to leave out a `worse` seat
describes the case as committed here.
It was fragile,
since a one-word edit of the checker sheet moved the rotation and failed it,
and it now hands its verdicts out by order of call
and passes on the unedited sheet and after that edit.

## B928: the challenge sentences say what a partial tie does (round 6)

Status:
fixed on 2026-10-25 (UTC),
red `5752dccc70`,
fix `d8d0e9180b`,
cache versions `642f42f156` (translate slice 34) and `aba1f71544` (consolidate 41).

Finding M-B of the fifth re-check.
In a challenge,
`settleAbsentDecline` ships by preference,
ships a level tie or keeps the archive wording only when no run-off is left to narrow
(`narrows` in `translate-stage/src/translate-runoff-tie.ts`).
Otherwise it raises,
and `judgeSlateWithRetry` asks the same bench again over the finalists,
for as long as each round narrows (`translate-stage/src/translate-retry.ts`).
The finalists are the candidates tied at the top,
or the leader and its runner-up where one judge alone named the leader (`runoffFinalists`).
A judge who declined in such a round read an outcome the code applied only after the run-off.

Both challenge sentences (`SHIPS_WHAT_TWO_JUDGES_NAMED` and `SHIPS_A_LEVEL_TIE_ELSE_PRODUCES_AGAIN`)
now carry a part that says it:
when the round leaves only some candidates tied at the top,
or a leader only one judge named,
the panel is asked again first,
over those candidates alone or over that leader and its runner-up,
and again for as long as that narrows the question.
The outcomes then wait on it:
"when no such question is left" opens the stay and the ship of the first sentence,
and the level-tie ship of the second.
A level tie across every candidate narrows nothing
(`every-candidate-backed` is a whole-slate case),
and a round in which every judge declines is a rejection,
also a whole-slate case,
so neither meets a run-off and the outcomes read as before.
The first ask's sentence named only a tie;
it now names the leader-and-runner-up shape too and says the asking repeats while it narrows.

The tests:
the clause reader requires the run-off clause in all three rounds and gates the challenge outcomes on it.
Red `5752dccc70` fails five cases of `select-decline-consequence.unit.test.ts`
(the three real sentences and the first-ask and challenge cases),
four of `translate-decline-sentence-per-round.unit.test.ts`
and two of `slate-decline-two-judges-name-it.unit.test.ts`,
each because the old sentence lacks the clause.
The cases that read hand-built sentences
(a lost run-off,
outcomes that do not wait on it,
swapped outcomes,
a rewording that keeps every clause)
pass on both commits as controls for the reader.
Sheets are told per round as in B923;
nothing else about the rounds changed.

Known limit.
No real judge has read the new sentences,
since no provider was called.

Corrected in round 7 (B934):
the run-off part said a leader only one judge named sends the panel to a run-off,
which is false for a leader alone in drawing a ballot and for a leader beside its runner-up in a slate of two.
Both halves now carry the qualifier that the run-off is asked only where it leaves some candidates behind.

## B929: no sentence promises the same seats (round 6)

Status:
fixed on 2026-10-25 (UTC),
red `5752dccc70`,
fix `d8d0e9180b`,
same cache versions as B928.

Finding M-D of the fifth re-check.
The first ask's sentence said the same judges are asked again.
That is true of the bench and not of the seats.
A round asks a window of quorum plus one seats,
starting where a hash of the sheet's text puts it (`stage/src/stage-fanout-window.ts`),
and the challenge sheet differs from the first ask's
(another `translateSelectionTask` and another decline sentence),
so on a bench wider than the window the challenge can ask a different subset:
a judge that declined may not be asked again,
and one that was spared may be.

All three sentences now say that the panel is asked again,
though a round may seat other members of it.
The clause reader requires that in every round,
and `PROMISES NO ROUND THE SAME SEATS` fails any sentence that says the same judges
(it fails on the old first ask).
The sentence does not say which seats are asked.

## B930: the two new sentences are entries of the sheet-wide scans (round 6)

Status:
fixed on 2026-10-25 (UTC),
red `d6daab9758`,
entries `70a2a19725`,
fixture split and lint fixes `07dcbfcd0a`.

Finding of the fifth re-check on the gap the fifth round named.
`SETTLES_NOTHING_AND_ASKS_AGAIN` and `SHIPS_A_LEVEL_TIE_ELSE_PRODUCES_AGAIN` were rendered only by the per-round suites,
so the eight sheet-wide scans of the artifact package
(Canadian spelling,
translators' notes,
quoted tense,
context,
reader-protection scope,
house-policy wording,
apparatus kinds,
consistency labels)
and the system prompt invariance scan never read them.

Both fixtures gain an entry for each:
`slateSheets()` in `repair/src/rendered-sheets-slates.test-fixture.ts`
(sheets `translate first ask slate` and `translate deferring challenge slate`)
and the three translate stages over wording that cannot ship,
now in `repair/src/system-prompt-stages-rounds.test-fixture.ts`.
Each is built with the planted archive and repair candidates through one helper,
so the notes-reading scan reaches it with no list change.
The stages moved out of `system-prompt-stages-slates.test-fixture.ts`,
which the entries took to 324 counted lines against a limit of 300;
`system-prompt-invariance.unit.test.ts` spreads the new list beside the old.

Red `d6daab9758` asks each round's sheet for its own clauses and for none of the other rounds'.
It fails for its reason (the fixtures render no sheet named `translate first ask slate`).

The scans found nothing to fix:
the artifact package's `test:unit` passes with 122 suites and none failing,
and a listing of `renderedSheets()` shows both new sheets.
Known limit:
nothing in the sentences makes a scan fail,
so the reach of each scan on the new entries rests on the listing
and on each scan's own test that it reads every sheet it claims to;
no spelling or tense fault was planted in the sentences to see a scan fire.
Round 6's re-check planted spelling and translators' notes faults,
which fire,
and round 7 (B935) planted faults for the reader-protection,
house-policy and apparatus-kinds scans.

## B931: the nits, and the clause reader refuses negated outcomes (round 6)

Status:
fixed on 2026-10-25 (UTC),
red `5752dccc70`,
fix `d8d0e9180b`.

The nits of the fifth re-check:
- The TSDoc of `SHIPS_WHAT_TWO_JUDGES_NAMED` said the deferring challenge is told its own sentence
  "since a decline ships nothing and keeps nothing there".
  That challenge ships a level tie.
  The TSDoc now says the first ask settles nothing and the deferring challenge ships only a level tie
  and otherwise sends the passage back.
- The TSDoc line of `SHIPS_A_LEVEL_TIE_ELSE_PRODUCES_AGAIN` that ran past 120 columns is reflowed.
- The positional word used as a comparative four times in the ledger is reworded to "exceeds",
  "exceeding" and "greater than"
  (B922 twice,
  L3 and L11 of `audit-ledger-repair-lane.md`).
- The clause reader passed "the wording in place never stays" and "the preferred of those does not ship",
  because the slots found `stays` and `ship`.
  The outcome clauses now refuse `never`,
  `does not`,
  `will not`,
  `cannot`,
  `no longer`,
  `not stay` and `not ship`,
  and the header states that a negation put in other words is not read.
  A case plants both negations and expects both outcome clauses unmet;
  dropping `never` from the list makes it fail,
  so the case reads the list.

## B932: the cache versions of round 6 (round 6)

Status:
done on 2026-10-25 (UTC).

Two versions moved,
each in its own commit with its reason in the key's TSDoc:
- translate slice 34,
  `642f42f156` (B928 and B929).
- consolidate 41,
  `aba1f71544` (B928 and B929).

The pinned keys moved with them:
`863592e7d0` (translate slice without a window)
and `f973915d8e` (consolidate).
`git log --extended-regexp -G '_CACHE_VERSION = [0-9]+'` over `c934acc979..373eae358c`
lists exactly those two commits,
and the control over `8d0929bc42..c934acc979` lists the four of round 5,
so the query reads the declarations.
The repair slice (49),
refine (19),
lane contest (18),
pairing (3),
page title (9) and consistency (9) versions stay:
the restored case is a test,
and none of the six renders a decline sentence.
Whether a record stands at the older numbers is not known,
and no slice-cache directory was searched.
