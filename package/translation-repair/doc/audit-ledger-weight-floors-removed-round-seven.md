# Audit ledger: the weight floors removed, round 7

Round 7 of the work recorded in
[the weight floors removed and two ballots deciding](audit-ledger-weight-floors-removed-and-two-ballots-decide.md),
which holds B900 to B906,
[rounds 2 and 3](audit-ledger-weight-floors-removed-rounds-two-and-three.md),
which holds B907 to B916,
[round 4](audit-ledger-weight-floors-removed-round-four.md),
which holds B917 to B919,
[round 5](audit-ledger-weight-floors-removed-round-five.md),
which holds B922 to B926,
and [round 6](audit-ledger-weight-floors-removed-round-six.md),
which holds B927 to B932:
the answer to the sixth independent re-check
(0 major findings,
two minor findings,
and the nits).
Ids B920 and B921 belong to main's DeepSeek pricing entries,
so this round continues at B933.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## Round 7 at a glance

`origin/main` stayed at `b6927f7ba9`,
an ancestor of the branch,
so no merge was made in this round.

The rollback's level case,
which a one-word edit to the checker sheet could fail,
is B933 (M-A).
The leader clause that was false in two shapes is B934 (M-B).
The scans of the two new sheet entries and the nits are B935.
The cache versions that moved are B936.

The commit-message slips of the sixth re-check are listed in the hand-back and nothing is posted.

## B933: the rollback's level case no longer depends on the sheet text (round 7)

Status:
fixed on 2026-10-25 (UTC),
test `3c756660ad`,
no code change and no red commit.

Finding M-A of the sixth re-check.
The case of two `worse` ballots level with two `fixed` on the five-seat bench scripted a verdict per seat.
The checker round asks a window of four seats,
starting where a hash of the checker sheet's text rotates the bench,
so which seat was left out depended on the sheet.
The case passed only because the rotation of that sheet left out a `worse` seat.
The sixth re-check changed one word of the judging rules in `adjudication/src/resolution-wire.ts`
("new damage" to "fresh damage"),
the rotation moved to leave out a `fixed` seat,
the tally became `fixed 1, worse 3, regressed true`,
and the case failed with `retained: false`.
It failed closed,
since no window gives a pass that is not the level case,
but a red test that reads as a regression of the rollback is a poor result of a wording edit.

The case now hands the verdicts out by order of call.
The scripted bench takes a `verdictsByCall` list,
and the case gives the first four calls `worse`,
`fixed`,
`worse`,
`fixed`.
Any four calls hear two of each,
whichever seats the window reaches,
and the client reads its verdict at entry,
before any await,
so a reordering of the launch moves only which seat holds which verdict.
The case also asserts that four exchanges were sent,
which ties the script to the window of quorum plus one:
if quorum or the spare changes,
that assertion fails first and says why.

Production is unchanged,
and that is the reason for the choice.
Threading `benchStart` and `fanOut` through `runCheckerStage` to `gatherStageVoices`
would add a seam and a stage-settings field for one test,
and it would pin the rotation the case should exercise live.
The editor stages pin their start for the reason that their fixtures seat known voices;
the checker stage has no such need outside this case.

Evidence is on scratch copies of the tree (`fix7/ma-*.log`),
since there is no failing state to commit.
On every copy the two cases about the bench the router left short fail,
mutated or not,
as in the fifth and sixth re-checks.
- Control:
  only those two cases fail,
  so the level case passes on the unedited sheet.
- The one-word sheet edit:
  only those two fail,
  so the level case still passes.
- The committed seat-keyed case with the same edit:
  a third case fails,
  which shows that the edit moves the rotation.
- The rollback written as `worse >= 2`:
  a third case fails,
  which is the level case.

Known limit.
The case assumes a checker bench of five seats and a window of four.

## B934: the leader clause was false in two shapes the code does not narrow (round 7)

Status:
fixed on 2026-10-25 (UTC),
red `9f3a1cb02c`,
fix `5012d6899b`,
cache versions `8f628973ad` (translate slice 35) and `8e331290a0` (consolidate 42).

Finding 2 of the sixth re-check.
B928 added to the challenge sentences a part that said:
when the round leaves only some candidates tied at the top,
or a leader only one judge named,
the panel is asked again first.
The tie half is true only where it leaves some candidates behind.
The leader half was false in two shapes
(`runoffFinalists` in `translate-stage/src/translate-runoff.ts`,
`narrows` in `settleAbsentDecline`):
- A leader alone in drawing a ballot,
  with no other candidate backed,
  is `whole-slate fewer-than-two-backed`.
- A leader beside its runner-up in a slate of two is `whole-slate every-candidate-backed`,
  since the leader and its runner-up are the whole slate.
In both the shipping challenge ships or keeps at once,
the deferring one raises to the lane,
and no run-off is asked.
The shapes that narrow are a tie at the top that leaves some candidates behind
and a leader beside a runner-up with some candidates behind.
The shapes that do not are those two,
the whole slate tied,
and a round in which every judge declined.

Measured on the recorded XingZ6021 run.
The log carries no slate width,
so the width comes from the recorded artifact (`artifacts/XingZ60.json`),
whose translate slices record the run-off finding and the candidate count,
and the first round's ballots come from the log's `chose candidate` lines.
Of 118 translate slices with a stage result,
27 were challenged
(the log has 32 challenge lines;
the other five are the consolidation gate's and were not classified).
- 10 were ties at the top that left a candidate behind (a run-off of 2 of 3).
- 8 were a leader and its runner-up with a candidate behind (a run-off of 2 of 3).
- 8 were a leader beside its runner-up in a slate of two,
  the shape the leader clause had wrong.
- 1 was a whole slate of two tied.
- None was a leader alone in drawing a ballot,
  and none was a round in which every judge declined.

So the false shape was not rare:
8 of 27 challenges,
all in slates of two,
and the lone leader of a wider slate did not occur in this run.
The run was degraded
(the slices read here show two of eight judge seats reachable,
so rounds closed on two voices),
which makes ties and single-ballot leaders common,
and the counts are not a rate for a healthy bench.

The sentences now say it.
Both halves of the run-off part carry the qualifier
("with others left behind"),
both challenges add a part naming the rounds that leave no candidate behind
("the whole slate tied,
every judge declining,
a leader alone in drawing a ballot,
or a slate of only two")
as asking no such question,
and the first ask carries the qualifier on both halves and says it is otherwise asked over the whole slate.
The outcomes still open on "when no such question is left",
which is true at once in those shapes.

The tests.
The clause reader requires the qualifier on both halves
and the no-question part in the challenges,
and takes a shape of the declined round
(`decline-clause-shapes.test-fixture.ts`),
for which it requires that shape's clause.
`translate-decline-sentence-shapes.unit.test.ts` scripts the judges to six first-ask shapes,
renders the real sheets of the first ask and of the challenge through `judgeSlateWithRetry`,
for a caller that ships past a decline and for one that defers,
reads how many candidates the challenge was put to
(2 of 4,
2 of 4,
2 of 2,
4 of 4,
4 of 4,
2 of 2),
and requires the sentence to say what that shape did.
Red `9f3a1cb02c` fails four cases of `select-decline-consequence.unit.test.ts`,
four of `translate-decline-sentence-per-round.unit.test.ts`,
all twelve cases of the new file as first committed
(one per shape and kind of caller,
each on its first sentence assertion,
so the candidate counts pass on that tree)
and two of `slate-decline-two-judges-name-it.unit.test.ts`.
The source scans then required cases with a static name,
so the gate fix made the twelve six cases that each read both kinds of caller.
The hand-built reader cases pass on both commits as controls.
Two mutants on a scratch copy of the fix die:
the no-question part without "a slate of only two" fails the slate-of-two cases,
and the first ask without "otherwise over the whole slate" fails every case.

Known limit.
The reader reads clauses and shape words;
it does not read a meaning no clause names.
No real judge has read the new sentences.
The consolidation gate's challenges were not classified by shape,
and the wording of B928 was true there only in the shapes that narrow.

## B935: the scans of the two new entries, and the nits (round 7)

Status:
done on 2026-10-25 (UTC).

Planted faults on a scratch copy of the tree,
one per scan,
to see whether each reads the two new entries
(`translate first ask slate` and `translate deferring challenge slate`):
- Apparatus kinds:
  the list of kinds put into both sentences,
  with no narrative bound,
  makes the pairing case fail,
  and its failure names both entries
  (and `translate front matter slate`,
  which shares the shipping sentence).
  Before the plant neither entry lists the kinds.
- Reader-protection scope:
  a sentence that says reader protection reaches the whole page does not fire it,
  because both sheets already carry the scope phrase from the house policy.
  The scan fires on these sheets when the scope phrase is lost from the house policy,
  the translate wire and the cited-reference rule,
  and its failure then names both entries.
  The scan reads them,
  and a fault in the sentence alone cannot fail it.
- House-policy wording:
  a planted sentence that tells the writer to keep a word left in the original language
  reaches both sheets and the scan passes.
  The scan reads the sheets named `refiner`,
  `resolution` and `rendering audit` and the house policy block,
  so it cannot fire for these entries by construction.

The Canadian spelling and translators' notes scans were planted in round 6 and fire.
B930's known limit,
that no fault was planted,
is closed by those two checks and these three.

The nits of the sixth re-check:
- The commits `aba1f71544` and `642f42f156` stated as fact
  that a stored record holds ballots cast under version 40 (33),
  while the TSDoc of both keys says that whether a record stands there is not known.
  The TSDoc paragraphs of versions 41 and 34 said "a stored settlement holds" and "a stored slice holds"
  beside that sentence;
  they now say that a record stored at the older number would hold them.
  The ledger (B932) already said it is not known.
  The paragraphs of those two TSDocs for versions before these use the same phrase and were not changed.
  Neither commit message can be amended;
  they are listed in the hand-back.
- B931 quoted the positional word it had reworded,
  which the positional-word scan matches.
  The line now names the word by its use and not by quoting it.
- The commit `5752dccc70` called `decline-clause.test-fixture.ts` new.
  The file exists since `70d01b2a51` (183 lines there) and that commit grew it to 305.

## B936: the cache versions of round 7 (round 7)

Status:
done on 2026-10-25 (UTC).

Two versions moved,
each in its own commit with its reason in the key's TSDoc:
- translate slice 35,
  `8f628973ad` (B934).
- consolidate 42,
  `8e331290a0` (B934).

The pinned keys moved with them:
`508eb67b3f` (translate slice without a window)
and `52b4539249` (consolidate).
`git log --extended-regexp -G '_CACHE_VERSION = [0-9]+'` over `534ac1533e..HEAD` lists exactly those two commits,
and the control over `c934acc979..534ac1533e` lists the two of round 6 (`642f42f156` and `aba1f71544`),
so the query reads the declarations.
The repair slice (49),
refine (19),
lane contest (18),
pairing (3),
page title (9) and consistency (9) versions stay:
the rollback's case is a test,
and none of the six renders a decline sentence.
Whether a record stands at 34 or 41 is not known,
and no slice-cache directory was searched.
