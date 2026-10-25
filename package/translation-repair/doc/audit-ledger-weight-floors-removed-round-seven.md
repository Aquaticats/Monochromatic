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

Round 8 answers the seventh independent re-check
(0 major findings,
one minor finding,
and five nits)
in this same file,
at the next free ids B937 to B943,
under the heading "Round 8 at a glance".

Round 9 answers the rotation sweep of round 8,
which found four cases that passed at salt 0 and failed under other salts,
in this same file,
at the next free ids B944 to B947,
under the heading "Round 9 at a glance".

Round 10 answers the nits that the eighth and ninth re-checks left,
in [its own file](audit-ledger-weight-floors-removed-round-ten.md),
at the next free ids B948 to B950,
since this file would pass the limit of a thousand lines.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## Round 7 at a glance

`origin/main` moved from `b6927f7ba9` to `96ed19a2b1` during this round
(ten commits,
all in `package/desktop-app/ide` and its docs,
none in `package/translation-repair`),
and the branch merged it in `938e84afb9` with no conflict.
The first version of this sentence said `origin/main` stayed put and no merge was made;
it was written before the merge and corrected in round 8 (B942).

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

The evidence was gathered on scratch copies of the tree,
since there is no failing state to commit,
and its figures are kept here so that the entry stands without the scratch files
(round 8,
B941).
A copy holds the `src` of `adjudication` and `repair`,
each `dist` re-exporting its `src`,
the edit under test applied to the copy,
and `node refine-recheck.unit.test.ts` is run from the copy's `repair/src`
with the search path and home directory only and no key.
Nothing in the worktree is written.
On every copy the two cases about the bench the router left short fail,
mutated or not
(at lines 1328 and 1403 of the test file),
as in the fifth and sixth re-checks;
they pass in the worktree itself,
and why they fail on a copy was not traced.
Each run is read by its failure lines and by the count its suite reports.
- Control,
  no edit:
  the suite `retainsResolvedIssues` reports 2 children failed,
  the two bench cases,
  so the level case passes on the unedited sheet.
- The one-word sheet edit,
  `- worse: the revision introduced new damage around this issue`
  to `- worse: the revision introduced fresh damage around this issue`
  in `RESOLUTION_JUDGING_RULES` of `adjudication/src/resolution-wire.ts`:
  2 children failed,
  the same two,
  so the level case still passes.
- The committed seat-keyed case
  (the test file as at `534ac1533e`)
  with the same edit:
  3 children failed,
  the third being the seat-keyed case itself,
  which expected `retained: true` and got `retained: false`,
  so the edit moves the rotation and the control for the edit is positive.
- The rollback written as `.tally.worse >= 2` in place of `.tally.regressed`
  in `madeItWorse` of `repair/src/refine-recheck.ts`:
  3 children failed,
  the third being the level case,
  which expected `retained: true` and got `retained: false`.

The seventh re-check repeated this on copies of its own.
It applied five other one-word edits to the same sheet
and the level case passed under all five,
a rollback written as `.tally.worse > 0` failed it,
and the repository's rotation sweep
(`source-scan/src/rotation-sweep/rotation-salt-hooks.test-fixture.ts`,
salts 0 to 4,
which cover the five seats)
passed the case at every salt,
where the seat-keyed case failed at two of them.

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

Corrected in round 8 (B937):
the part naming the rounds that leave no candidate behind gave the word behind two meanings,
since the run-off part counts a candidate that drew no ballot as behind.
That part now says a round that leaves nothing to narrow asks no such question.

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
that no fault was planted in the sentences to see a scan fire,
is closed for three scans:
Canadian spelling,
translators' notes,
and apparatus kinds.
It is not closed for the other two scans planted here.
The reader-protection scope scan fires only when the scope phrase is lost elsewhere,
so a fault in the sentences cannot fail it,
and the house-policy wording scan cannot fire for these entries at all.
The first version of this paragraph said the limit was closed by all five checks;
it is corrected in place in round 8 (B939).

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

Both versions moved again in round 8 (B943),
to 36 and 43.

## Round 8 at a glance

The seventh independent re-check of round 7 found no major finding,
one minor finding and five nits,
and round 8 fixes all of them.
`origin/main` moved from `96ed19a2b1` to `6457f1f09c` during the round
(one commit,
touching only `doc/handover/slint-ide-0x.md`,
none in `package/translation-repair`),
and the branch merged it in `e73f350789` with no conflict.
`6457f1f09c` is an ancestor of the branch,
so landing is a fast-forward.
The first version of this paragraph said no merge was made in round 8;
it was written before the merge and corrected in the same round,
the pattern B942 records for round 7.

The phrase that gave the word behind two meanings is B937.
The position scan that passed a positional comment is B938.
The closing sentence of B935 that overstated what fired is B939.
The measuring-sheets case that read one sheet of five is B940.
The evidence of B933 that lived outside the repository is B941.
The sentence of round 7 about the merge is B942.
The cache versions that moved are B943.

## B937: the word behind has one meaning in the challenge sentences (round 8)

Status:
fixed on 2026-10-25 (UTC),
red `20eddc6451`,
fix `3ef8596000`,
cache versions `53e23a728c` (translate slice 36) and `6c81859c79` (consolidate 43).

Nit 1 of the seventh re-check.
The run-off part of both challenge sentences says a tie at the top,
or a leader only one judge named,
sends the panel to a run-off "with others left behind".
In the code a candidate is left behind when the run-off does not offer it
(`runoffFinalists` in `translate-stage/src/translate-runoff.ts`:
the candidates to offer are the tied leaders,
or the leader and its runner-up,
and every other candidate of the slate is behind),
which counts a candidate that drew no ballot.
Two tied candidates and a third that drew nothing are narrowed to the two.
The part B934 added for the rounds that ask no run-off said "a round that leaves no candidate behind"
and listed a leader alone in drawing a ballot.
In a slate wider than two a lone leader leaves other candidates that drew nothing,
so under the meaning of the run-off part they are behind,
and the one phrase carried two meanings.
The behaviour told to the judge was right in every shape,
since the parenthetical list names what the code does
(the whole slate tied,
every judge declining,
a leader alone in drawing a ballot,
and a slate of only two ask no run-off),
and only the label of the list was wrong.

The part now says a round that leaves nothing to narrow asks no such question,
which is true in each of the four shapes:
a run-off needs two candidates to offer,
and in those shapes the candidates to offer are the whole slate or fewer than two.
Behind now means one thing in the sheet,
a candidate a narrowing would drop.

The tests.
The clause reader requires "nothing to narrow" (or "nothing left to narrow") in that part
and refuses the word behind (and "left out") in it,
so the word cannot slip back in as a second meaning.
The six shapes of `translate-decline-sentence-shapes.unit.test.ts` were the red:
that file already holds the lone leader in a slate of four,
so no new shape was needed.
Red `20eddc6451` on the unchanged sheet,
node on single files:
- adjudication `select-decline-consequence.unit.test.ts`:
  3 cases plus the suite.
- translate-stage `translate-decline-sentence-per-round.unit.test.ts`:
  2 cases plus the suite.
- translate-stage `translate-decline-sentence-shapes.unit.test.ts`:
  6 cases plus the suite,
  every one on the challenge sentence assertion.
- consolidate `slate-decline-two-judges-name-it.unit.test.ts`:
  2 cases plus the suite.

The new refusal case in the adjudication file reads the old wording through the new reader
and passes on both commits,
since it tests the reader and not the sheet.
All four files pass on `3ef8596000` after a build of adjudication.

The first ask does not carry the phrase and is unchanged.

Known limit.
The reader reads clauses and a banned word;
it does not read a meaning no clause names.
No real judge has read the sentence.

## B938: the position scan read the verb ties as the object of a comparison (round 8)

Status:
fixed on 2026-10-25 (UTC),
red `fa21600a7e`,
fix `ae3712c7b6`.

Nit 2 of the seventh re-check.
The comment of the rollback's level case in `repair/src/refine-recheck.unit.test.ts`
pointed at an assertion by a positional word,
and `position-references.unit.test.ts` passed it.

Why it passed.
It was not an exemption and not a file the scan skips:
the file is among the 3925 texts the guard reads,
and the scan found nothing in it.
It was a gap in the word list.
`COMPARED_OBJECTS` in `source-scan/src/position-references-vocabulary.test-fixture.ts`
holds the words that,
after a position,
make it a comparison
("a rate above one",
"ranks the house rules above its own"),
and it held "ties" for "preservation above ties" in `adjudication/src/select-candidate.unit.test.ts`.
The sentence used "ties" as a verb,
and the scan read the verb as the object of a comparison.
The positive control:
the scan finds the same positional phrase before any other verb,
and the existing case finds the fixture sentence about a count in a doc.
A probe over the texts the guard reads showed the gap
(the sentence put in a doc was not found either)
and,
after the fix,
showed no unexempted finding.

The fix.
"ties" leaves the list,
the one comparison that used the noun is exempted with its reason,
and the comment is reworded to name the asserted count of exchanges.
The list keeps "total",
which can be a verb too:
no sentence of the texts reads it as one after a position,
and the TSDoc of the list says so.
Red `fa21600a7e` adds the case that scans the sentence and expects the phrase;
it fails on the unchanged list with one case plus the suite and passes on `ae3712c7b6`.

Known limit.
The scan matches words and not grammar,
so another word that is both a noun and a verb,
added to the list later,
can let a pointer through.
That sentence was incomplete when it was written
(re-check 8,
nit A):
words already in the list were verbs too,
the quantifiers all,
both,
each and some floated away from their noun and read as objects,
and "that" opening a clause read as an object.
The scan now reads a listed word with the word after it:
a verb or a clause opener compares unless a determiner,
a quantifier or a pronoun follows,
and a floating quantifier compares only when a word that names what is compared follows.
It still lets a listed word before a bare verb pass
("the lines below that write it"),
and a quantifier before "of".
B949 states both in full.

## B939: the closing sentence of B935 overstated what the plants showed (round 8)

Status:
corrected on 2026-10-25 (UTC),
in place in B935 and in a note in B930.

Nit 3 of the seventh re-check.
The last paragraph of B935 said the known limit of B930
was "closed by those two checks and these three".
Only the apparatus-kinds plant fires from a fault in the sentences.
The reader-protection scope scan fires only when the scope phrase is lost elsewhere
(from the house policy,
the translate wire and the cited-reference rule),
and the house-policy wording scan cannot fire for these entries by construction.
The body of B935 said so;
its closing paragraph did not.
The paragraph now names the three scans that are closed
(Canadian spelling,
translators' notes,
apparatus kinds)
and the two that are not,
and the sentence of B930 that listed the plants carries a note.

The message of the docs commit `128ad40f71b091845004f2aef0455b0305af8797` says
"faults planted for the reader-protection,
house-policy and apparatus-kinds scans",
which reads as three scans that fire.
It cannot be amended,
and no comment was posted.
It is listed in the hand-back.

## B940: the measuring-sheets case reads all five measuring sheets (round 8)

Status:
fixed on 2026-10-25 (UTC),
test `1f98f40da2`,
no code change and no red commit.

Nit 4 of the seventh re-check.
`artifact/src/house-policy-wording.unit.test.ts`,
the case "TELLS THE MEASURING SHEETS reader protection covers
attempts and a place that was the means",
read the `resolution` sheet only,
while `naturalness review`,
`rendering audit`,
`restoration judge` and `introduced defect`
carry the same reader-protection sentence.
Each of the five splices `MEASUREMENT_POLICY_BLOCK` in its own file
(`archive/src/absolute-naturalness-review-wire.ts`,
`rendering-audit/src/rendering-audit-prompt.ts`,
`adjudication/src/resolution-wire.ts`,
`repair/src/restoration-judge-wire.ts`,
`adjudication/src/introduced-defect-wire.ts`),
so a fault at one splice leaves the other four whole and the case saw only one of the five.

The case now requires the sentence on each of the five,
and requires that the sheets carrying the paragraph that holds it are exactly those five,
so a sixth measuring sheet fails the case until it is listed.
No red commit is possible:
the sheets were right and the case read one.

Evidence on scratch copies of `adjudication`,
`repair`,
`artifact`,
`archive` and `rendering-audit`
(the same scratch method as B941,
with the case run from the copy's `artifact/src`):
- The control with no plant passes.
- A fault planted at each sheet's own splice
  (the splice becomes the block with "where it happened when the place was the means,
  " removed)
  fails the case for each of the five,
  and the failure names that sheet alone as lacking the sentence,
  with the unlisted and missing lists empty.
- The case as it stood before the commit,
  run on the same five planted copies,
  fails only for the `resolution` plant and passes for the other four,
  which shows the gap.

## B941: B933 stands without the scratch files (round 8)

Status:
done on 2026-10-25 (UTC),
docs only.

Nit 5 of the seventh re-check.
B933 and the commit `3c756660ad` cited logs in the coordinator's scratch directory.
The method,
the edits,
the commands in words and the figures of those logs are now in the text of B933,
and the entry no longer cites a path outside the repository.
The commit message cannot be amended;
it says the logs are outside the repository,
which stays true of the logs and no longer matters to the entry.

## B942: the round 7 sentence about the merge was false at head (round 8)

Status:
corrected on 2026-10-25 (UTC),
in place in "Round 7 at a glance".

The minor finding of the seventh re-check.
"Round 7 at a glance" said `origin/main` stayed at `b6927f7ba9`
and no merge was made.
The head of round 7,
`938e84afb9`,
is the merge of ten main commits (`96ed19a2b1`),
all in `package/desktop-app/ide` and its docs,
none in `package/translation-repair`,
and the docs commit that held the sentence was written before the merge and not touched after it.
The report of round 7 stated the merge correctly,
so the two records disagreed.
The sentence now states the merge.

## B943: the cache versions of round 8 (round 8)

Status:
done on 2026-10-25 (UTC).

Two versions moved,
each in its own commit with its reason in the key's TSDoc,
and each after a red commit that expects the new number:
- translate slice 36,
  red `acae13b423`,
  fix `53e23a728c` (B937).
- consolidate 43,
  red `9734f75d38`,
  fix `6c81859c79` (B937).

The pinned keys moved with them:
`5925d39143` (translate slice without a window)
and `20703993fb` (consolidate).
The sentences reach a stored decision only through `declineConsequenceFor`,
whose callers are the translate lane and consolidation,
as in B936.
`git log --extended-regexp -G '_CACHE_VERSION = [0-9]+'` over `938e84afb9..HEAD` lists exactly those two commits,
and the control over `534ac1533e..938e84afb9` lists the two of round 7 (`8f628973ad` and `8e331290a0`),
so the query reads the declarations.
The repair slice (49),
refine (19),
lane contest (18),
pairing (3),
page title (9) and consistency (9) versions stay,
since none of the six renders a decline sentence,
and the other four fixes of this round are tests,
a scan vocabulary and a comment.
Whether a record stands at 35 or 42 is not known,
and no slice-cache directory was searched.

## Round 9 at a glance

The rotation sweep of `b37685721e`,
the head of round 8,
refused it:
four cases passed at salt 0 and failed under other salts,
where the sweep of main at `b6927f7ba9` was clean.
All four trace to one change of this branch,
`1dee3ca797`,
which sized the selection fan-out by the number of judges a winner needs
and so turned a bench of four from a whole bench into a window of three.
All four are defects of the test and none of the code.
`origin/main` did not move during the round (`6457f1f09c`),
so no merge was made.

The archive case that held back the fourth seat is B944.
The chunk case that had the fourth judge name a candidate is B945.
The two refine cases that read the refiner's self-vote are B946.
The cache versions,
the controls and the lesson are B947.

## B944: the archive window case held a seat the selection round may not ask (round 9)

Status:
fixed on 2026-10-25 (UTC),
test `a62c92771a` and `5553a18d38`,
no code change and no red commit.

Finding of the rotation sweep of round 8.
The case "KEEPS THE READERS' WINDOW IN THE SELECTION ROUND (ledger B713)"
of `archive/src/archive-block-review-windows.unit.test.ts`
failed under salts 1,
3,
5,
7 and 9.
It held back the fourth seat of the roster in the selection round
and expected the finding `stage-voice-lost (select <that seat>)`.
Since `1dee3ca797` a selection round asks the window of quorum plus one wherever the window holds two judges.
Before it,
three self-votes at half weight fell short of the weight minimum,
so a bench of four was asked whole;
`d42e79be74` records the change in the asked count of the stage case,
from four to three.
The selection round now asks three of the four seats,
and the seat it leaves out follows the rotation,
a hash of the sheet moved by the salt.
Under the salts that left the fourth seat out nothing was held,
all three judges answered at once (`select round: 3/3 heard`),
and the finding was absent.

The test was wrong.
The code follows the rulings that two ballots decide and one judge never decides alone,
and which seats are asked is a saving and not a verdict
(the comment of `translate-stage/src/candidate-select-fanout.ts`).
The case now holds the third ballot asked,
whichever seat that is,
by order of call,
through a sink that records the seats asked,
and asserts that three were asked and that the finding names the third.
The review-round case still holds a seat by name,
since that round asks the whole bench (`fanOut: 'whole-bench'`).
The stage hands `fanOut` to the review round only,
so the selection round has no seam to ask a named seat,
and none was added,
for the reason given in B933.

Evidence,
all keyless and on single test files with the sweep's own hook
(`node --import rotation-salt-hooks.test-fixture.ts?salt=N`,
once for each salt from 0 to 9):
- Before:
  the one-salt rerun of the sweep (`--package archive --salt 1`) lists the case,
  and the file run at salt 1 fails with
  `expected [ Array(1) ] to include 'stage-voice-lost (select hf:openai/gpt-oss-120b)'`.
- After:
  salts 0 to 9 all pass.
- Cause control,
  standing in for the bisect:
  with `selectionFanOut` planted to return the whole bench,
  the file as at `b37685721e` passes under all ten salts.
- Guard control:
  with the selection round planted to wait the writers' window (`grace.writerMs` in place of `grace.roundMs`
  in `translate-stage/src/candidate-select.ts`),
  the new case fails and names the seat it held,
  and the review-round case passes.
  Both plants were restored and the status of the tree was empty.

Known limit.
No one-salt rerun was made at a commit inside `b6927f7ba9..b37685721e`:
the attribution to `1dee3ca797` rests on its diff of `candidate-select-fanout.ts`,
the log line,
and the cause control,
and the two-point check at `1dee3ca797` and its parent was not run.

## B945: the chunk decline case had a seat name the candidate that the window may leave out (round 9)

Status:
fixed on 2026-10-25 (UTC),
test `a62c92771a`,
no code change and no red commit.

Finding of the rotation sweep of round 8.
The case "SENDS the fallback repair on when most judges decline and one names a candidate"
of `repair/src/chunk-decline-consequence.unit.test.ts`
failed under salts 2 and 6.
The one judge that named candidate 1 was the fourth of four.
Where the window of three left it out,
every judge asked declined,
and the round kept the existing English (`The cat hates the sun.`),
which is the other behaviour the file pins.

The test was wrong,
and the code is right in both outcomes:
a round in which every judge asked declines keeps the text,
and one name among declines is indecision,
which sends the fallback repair on.
The judge that names is now picked by its place in the order of call,
the first asked,
so the round is always two declines and one name.
The sheet says "only when every judge declines";
on a bench the window does not cover,
that means every judge asked,
as it did before this branch for benches of five and over
(corrected in round 10 from "benches of seven";
the note closes this entry).

Evidence:
- Before:
  salt 2 fails with `expected 'The cat hates the sun.' to equal 'The cat likes the sun.'`.
- After:
  salts 0 to 9 all pass.
- Cause control:
  with the whole bench planted,
  the file as at `b37685721e` passes under all ten salts.
- Guard control:
  with indecision planted to ship the rejection fallback (`editor-ensemble.ts`),
  the case fails at salt 2 and the other two cases pass.
  The plant was restored.

Correction of round 10
(re-check 9,
nit 2).
The sentence about the sheet said benches of seven.
The window applied from a bench of five at `b6927f7ba9`,
where the deleted test named `THINNEST_WINDOWED_BENCH` as 5,
and nothing in the docs ties production to seven.
It reads benches of five and over.

## B946: the two refine cases read a self-vote that holds only when the refiner is asked (round 9)

Status:
fixed on 2026-10-25 (UTC),
test `a62c92771a`,
no code change and no red commit.

Finding of the rotation sweep of round 8.
Two cases of `repair/src/refine-stage.unit.test.ts` failed:
"NAMES the first of two paragraphs as an unchanged rewrite beside the second,
which ships" under salts 1,
5 and 9,
and "NAMES each rewrite the resolver dropped beside the one that ships" under salts 3 and 7.
Both expect `select-self-vote (<the refiner>)` and `refine-selected (weight 2.5 of 3 ballots)`.
The refiner is the first of four judges,
and the window of three leaves one seat out.
Where it left the refiner out,
the three judges asked gave three full ballots,
and the findings read `refine-selected (weight 3 of 3 ballots)` with no self-vote.

The test was wrong.
Both cases now seat three judges,
the first three of the bench with the refiner among them,
so the window of three holds all of them whatever the rotation,
and the rotation changes only the order of the calls.
A bench of three keeps the asserted strings as they were.
A pinned start or a whole bench was not used:
`runRefineStage` takes neither (`fanOut` and `benchStart` are not in `refine-stage.ts`),
a seam for two cases would add a field and pin the rotation the sweep exercises,
and the order of call cannot say which seat the self-vote names.

Evidence:
- Before:
  the failure lines `only in actual: "refine-selected (weight 3 of 3 ballots)"` and
  `only in expected: "select-self-vote (...)"` at salts 1 and 3.
- After:
  salts 0 to 9 all pass.
- Guard control:
  with the self-vote finding planted away (`candidate-select.ts`),
  both cases fail at salt 1 on their findings assertions.
  The plant was restored.

## B947: the cache versions, the gates and the lesson of round 9 (round 9)

Status:
done on 2026-10-25 (UTC).

No code changed,
so no cache version moved.
`git log --extended-regexp -G '_CACHE_VERSION = [0-9]+'` over `b37685721e..HEAD` lists nothing,
and the control over `938e84afb9..b37685721e` lists `6c81859c79` and `53e23a728c`,
so the query reads the declarations.

The lesson is in `mistake-prevention-tests-clocks-and-arrival-order.md`,
in "Orders and seats that a prompt hash decides":
a change to the rule that sizes a window moves which benches get one,
and the cases on a bench that was always asked whole become rotation-dependent
without any sheet text changing.
`1dee3ca797` and `d42e79be74` ran no rotation sweep,
and the next sweep found four cases.

The gates ran on the tree of `5553a18d38`,
keyless and one at a time.
Archive and repair:
`build`,
`lint:types` and `lint:oxlint` (0 warnings and 0 errors) passed,
and `test:unit` passed 65 and 149 suites with none failed.
The first oxlint run of archive found the counter of the new predicate (`no-function-root-let`),
which `5553a18d38` removes.
Source-scan `test:unit` passed 68 and `source-scans` 53 on the tree with this text.
The full rotation sweep ran once after this text was committed.
This entry had named the hand-back of the round as the place of its result;
round 10
(re-check 9,
nit 5)
puts the lines of the log here,
with the first line wrapped at its semicolon:

```text
// rotation sweep of round 9, result lines
rotation sweep: test files 1321, packages 26, salts 0 1 2 3 4 5 6 7 8 9;
the canary saw each salt move the stage package's rotation
rotation sweep: cases failing under a salt that salt 0 passes 0, cases red with no shift 0
rotation sweep: clean
```

Commit-message slips of this round:
- `a62c92771ab60c85630054efdc355701c4d77ae2` states the cause of all four as fact from the diff of the fan-out,
  where the brief asked for a bisect and none was run;
  its subject uses the scope `*` and its body has no per-package form.
- `5553a18d385a207c77d7f9a5495660ee6f032106` says `a62c92771a` states that lint was not yet run;
  it does not.
  Neither was amended.
