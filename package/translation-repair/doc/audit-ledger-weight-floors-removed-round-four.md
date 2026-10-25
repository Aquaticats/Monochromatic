# Audit ledger: the weight floors removed, round 4

Round 4 of the work recorded in
[the weight floors removed and two ballots deciding](audit-ledger-weight-floors-removed-and-two-ballots-decide.md),
which holds B900 to B906,
and
[rounds 2 and 3](audit-ledger-weight-floors-removed-rounds-two-and-three.md),
which holds B907 to B916:
the lead's rulings on the findings of the third independent re-check
(0 major findings,
three minor findings M1 to M3,
one question Q1 and the nits N1 to N7).
Each ruling is a lead ruling settled by the owner's rules of 2026-10-25,
not a new owner answer:
remove every weight floor so that two ballots decide,
one judge alone never decides,
and a self-vote counts as an answering judge.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## Round 4 at a glance

The merge of `origin/main` at `85e21653a2` (finding M2) is `14e6a4466b`.
One keep-both conflict in `audit-ledger-entries-by-id.md`,
resolved in id order:
main's line for the follow-ups of round 11 of the editor bench,
then B900 to B906,
then B907 to B916,
then B920.
Main changed no cache version and none of the files this branch changed apart from the two ledger indexes.

The nits:
- N1 and N7 are in B918.
- N2:
  the heading of B901 lost a clause so that it fits in 120 characters,
  and the B906 line that was too long was wrapped.
- N3:
  the blank line between the `ResolvedVoice` type and the TSDoc of `settlePageTitles` is restored.
- N4:
  nothing changed.
  The count test of the page title lexicon already imported `../dist/final/node/index.mjs` at the head of round 3,
  as its siblings do.
- N5:
  the commit-message slips are listed in the hand-back and nothing is posted.
- N6:
  nothing.

The cache versions that moved in this round,
each in its own commit with its reason in the key's TSDoc:
repair slice 48 and refine 18 (B917),
translate slice 32 and consolidate 39 (B918),
page title 9 (B919).
Lane contest 18,
pairing 3 and consistency 9 stay:
none of the three stores a checker reading or reads the sheet sentence or the page title storage rule.

## B917: a regressed issue needs two ballots naming worse (round 4)

Status:
fixed on 2026-10-25 (UTC),
red `3b7574b01c`,
fix `e72b4cf966`,
cache versions `65028fadef` (repair slice 48) and `5090cff144` (refine 18).

Finding M1 of re-check 3,
a lead ruling settled by the owner's rules.
`regressed` was `worse > fixed + notFixed` on weights.
B906 had accepted it with no floor,
for three reasons,
and the re-check showed two of them false.
With one checker `worse` against one `fixed` or `not-fixed` the flag ties and does not regress
only when the `fixed` ballot is a whole vote;
when the `fixed` voice wrote the text it counts for half,
so one `worse` against it regressed.
And the flag does more than demote:
`compareCandidates` ranks `regressedKnownIssues` third,
above total resolution and preservation,
so a regressed candidate loses to an unregressed one that resolves less,
and one judge's `worse` could choose which of two shippable candidates ships.
That breaks the rule that one judge alone never decides.

The rule now:
an issue is regressed only when two or more ballots name `worse`
and the weight behind `worse` exceeds the weight behind `fixed` and `not-fixed` together.
The count is of ballots naming `worse`,
not of ballots cast and not of weight,
so a `worse` against a text author's own half-weight `fixed` is still one voice.
A self-voted `worse` counts as one of the two voices.
The constant is `MIN_REGRESSION_BALLOTS` in `adjudication/src/tally-resolution.ts`,
beside `MIN_RESOLUTION_BALLOTS`,
whose TSDoc says cast ballots and so was not bent to a different count.
The TSDoc of the field,
the paragraph of `tallyResolutionChecks` and the field `regressedKnownIssues` of the measurements carry the rule.

The tests:
- A lone `worse` with the other checker silent,
  and one `worse` against a self-vote `fixed` (weights 1 against 0.5),
  fail on the red commit and pass on the fix.
- Two `worse` against one `fixed` regress on both commits (the positive control),
  and two `worse` against two `fixed` do not regress on either (the level weights stay level).
- An author's own `worse` beside one independent `worse` regresses on both commits.
- The existing cases that assert a regression go through two or more `worse` ballots
  (`tally-resolution.unit.test.ts`,
  `self-certification-weight.unit.test.ts`,
  the three checkers of `repair-edit-stages.unit.test.ts`),
  and none flipped.
  One case that asserted the flag on a single ballot did:
  the stored recheck reading of one `worse` ballot in `refine-recheck.unit.test.ts`,
  which expected `regressed: true` and now expects `false`.
  It is the reading the refine cache version retires.

The consumers of the flag,
found by searching the translation-repair packages for `regressed`:
- `repair/src/chunk-measure.ts` (`measurePatchedCandidate`,
  `regressedKnownIssues`),
  read by `compareCandidates` in `adjudication/src/select-candidate.ts`.
  This is the consumer the change is for:
  the count ranks third and can choose the shipped candidate,
  so it needs two voices.
  The change is right there.
- `bench/src/corpus-run/checker-sensitivity-print.ts` prints `regressed=` in the lines of the checker sensitivity probe.
  No decision reads it.
  The printed flag follows the new definition,
  and the probe's cases have every seated checker answer alike,
  so the lines do not change.
- The stored copies:
  `checkerReadings` and `recheckReadings` of the repair outcome (`assembly/src/repair-contract.ts`)
  hold the tally whole,
  and the repair slice record,
  the refinement record (`repair/src/refine-slice-settle.ts`) and the issue rows of the published page
  (`assembly/src/repair-record.ts`) carry them.
  They are records,
  and no decision reads the flag from them.
  The repair slice and refine cache versions move for them.
- Not consumers,
  though the word is near:
  the local list of issue ids in `repair/src/refine-recheck.ts` reads `resolved` and the `worse` weight,
  and `repair/src/repair-worse-strip.ts` reads `resolved` and the `worse` weight.
  Both roll back or strip on one `worse` ballot by the owner's ruling of 2026-09-28 (ledger L3),
  which restores the text before the edit and ships nothing new,
  and this round leaves it as it is.
  Superseded on 2026-10-25 (B922):
  the owner answered "Two ballots for undo too",
  and both now read the `regressed` flag this entry defines.
  `repair-chunk-verdict.ts` and `candidateConfirmedIssueIds` read `resolved` only.

The cache versions:
the repair slice (47 to 48) holds the candidate that selection ranked first,
so a slice whose checker round held one `worse` ballot can ship another candidate for the same texts.
Refine (17 to 18) decides nothing on the flag,
but a refinement record embeds the readings of its recheck round and of the repair outcome it rewrote.
The translate,
consolidate,
lane contest,
consistency,
pairing and page title versions stay:
none of them runs the resolution checkers
(the writers and readers of the readings are the files of `repair`,
`assembly`,
`adjudication`,
`bench` and `shared-contract` that the search for `runCheckerStage`,
`IssueCheckerReading`,
`checkerReadings` and `tallyResolutionChecks` lists).

The artifact generation stays.
It is `ARTIFACT_SCHEMA_VERSION_V21` (`artifactSchemaVersion` in `artifact/src/corpus-run/artifact-two-lane-build.ts`).
The stored readings keep their keys and shape,
and only the value of one boolean can differ,
and no reader of the published artifact recounts the checker tally or reads the flag
(the gates on generation in `artifact-two-lane-contract.ts` guard rules that a reader re-derives,
as the last-seat grace of generation 20 was).
`shared-contract/src/panel-reading.ts` names the reading type only in a comment.

The replay of `XingZ6021`:
the artifact holds 99 checker readings
(89 of the checker round and 10 of the recheck round)
in the chunks of the repair lane.
The replay rebuilds the ballots and the authorship of each reading from the record
and calls the real `tallyResolutionChecks` of this worktree.
The harness reproduces the recorded tally on 99 of 99 readings,
and the recorded flag equals the old expression (`worse > fixed + notFixed`) on 99 of 99.
None holds a `worse` ballot,
so the flag is false under both definitions on every one and no reading moves.
Because a null is only as good as a probe that can show a move,
the same script runs four hand-built cases through the real function:
a lone `worse` and one `worse` against a self-vote `fixed` give not regressed where the old expression gave regressed,
two `worse` against one `fixed` stay regressed,
and two `worse` against two `fixed` stay not regressed.
The losing candidates of a selection are not in the artifact,
so whether a ranking would have changed is not replayable.

B906 listed this flag among the single-voice effects it accepted.
It no longer does:
the bullet there points at this entry.

## B918: the sheet sentence about a declined round says what a decline does not do (round 4)

Status:
fixed on 2026-10-25 (UTC),
red `d7209c12b4`,
fix `0557f443f9`,
cache versions `1276122231` (translate slice 32) and `7ed5f1d79a` (consolidate 39).

Findings M3,
N1 and N7 of re-check 3,
lead rulings.
B916 made the sentence true to the code,
but it put "cannot ship unreported",
"not a quiet keep" and "stays" in one sentence,
and did not say that a judge's own decline names no candidate,
so a judge could read its decline as the way to the stay.

The sentence now,
after "Declining is a real answer and is better than endorsing a candidate you would not ship;":
declining does not by itself keep the wording in place,
and your own decline names no candidate;
when no candidate is named by two or more judges the wording in place stays and the page reports it,
with your reason recorded;
when one or more candidates are named by two or more judges the preferred of those ships,
by a fixed order (the repair lane's text,
then the translate lane's,
then the first on the slate)
and not by your judgment.
It is true to the code in the same places as B916:
`shipPreferredPastDecline` ranks only the candidates that drew two ballots by `rankedByPreference`,
and where none did the wording in place stays under `translate-slate-declined-archive-kept`.

N1:
`select-decline-consequence.unit.test.ts` no longer asks whether the constant includes a copied substring.
`decline-consequence-clauses.test-fixture.ts` lists the five meanings a judge must read
(declining does not by itself keep the wording;
a judge's own decline names no candidate;
with none named by two or more judges the wording stays and is reported;
with one or more named by two or more judges the preferred of those ships;
by a fixed order,
the repair lane's text first,
then the translate lane's,
then the first on the slate)
as ordered slots of alternative phrasings,
each read inside one part of the text between semicolons,
so a rewording that keeps the meaning passes and a lost clause fails
(regex literals are refused by the lint rules of the repository,
so the reader uses string search).
A control case shows the reader names the clauses a shortened sentence lacks.
The test of the consolidation that copied two substrings of the old sentence
(`consolidate/src/slate-decline-two-judges-name-it.unit.test.ts`) reads the sheets the same way.

N7:
`repair/src/rendered-sheets-slates.unit.test.ts` renders the translate slate over withheld wording through the fixtures
and reads the text between the line "Declining is a real answer" and the line that asks for the JSON reply,
which is the sentence as a judge meets it with the surrounding template.
A second case is the control:
no other slate sheet carries all five clauses,
since the other callers keep trusted text or have nothing to keep.
The red commit fails three cases (the constant,
the rendered sheet and the control).

The cache versions:
a stored slice or settlement holds ballots cast under the sentence a judge read,
and the keys hash the version,
the shape of the run and the texts,
not the sheet
(`source-scan/src/cache-key-sheet-blindness.unit.test.ts` reads the imports of the key files),
so only the number retires a record.
The translate slice moves from 31 to 32 and the consolidate from 38 to 39.
The repair slice (48) and refine (18) versions moved in B917 for the flag and not for this sentence:
the repair lane's slates use `CHUNK_DECLINE_CONSEQUENCE`,
and neither reads `SHIPS_WHAT_TWO_JUDGES_NAMED`.

B916 gave the wording of version 31 and 38:
that wording is history,
and this entry gives the current one.

## B919: a page title round is stored only when every title it asked is settled (round 4)

Status:
fixed on 2026-10-25 (UTC),
red `db4cacd51e` and `901285d5c1`,
fix `8cafe3ff68`,
cache version `4347b354bb` (page title 9).

Question Q1 of re-check 3,
a lead ruling settled by the owner's rule that one judge alone never decides.
After B915 a round that heard two or more voices could settle no title
(a split of one voice apiece),
and `passPageTitles` still stored it,
as an empty or partial list under the key of its bench.
A later run on a fuller bench then resumed the gap instead of asking again,
unlike a round that heard fewer than two voices,
which was never stored because the bench was unreachable and not undecided.

The rule now:
`passPageTitles` stores a round only when the settled titles equal the titles it asked,
which also means at least two voices were heard,
since a title settles on a rendering two voices gave.
A round that leaves any title unsettled
(a split,
or a title no voice rendered)
is not stored,
so the next run asks again on whatever bench it has.
The lines returned for the run still carry the titles that did settle.

The tests in `pass-page-titles.unit.test.ts`:
- two seats of two answer one against one:
  nothing is stored,
  and a run on the same bench asks again;
- two seats of seven answer one against one (the 2-of-7 split of the ruling):
  nothing is stored,
  and a run on the same bench asks again;
- two titles asked,
  one settled and one split:
  nothing is stored,
  and the settled title is still in the lines of the run;
- two titles asked and both settled on two voices apiece:
  stored once
  (the control,
  passing on both commits).
The red commits fail three cases between them.
The fake client gained an option naming the renderings of each seat,
and the helper an option naming the bench.

The cache version moves from 8 to 9:
a record stored at 8 may hold a split or partial round,
and the key hashes the version,
the titles,
the identity and the roster,
not the rule.

The replay:
the log of `XingZ6021` holds one page title round,
`PAGE TITLES heard=4 asked=6 settled=6`,
under the rule of that run.
Per-title voice counts are in neither the artifact nor the log,
so whether the stricter rules of B915 and this entry would have left a title unsettled in it cannot be replayed.
If all six settled on two voices or more,
the round would be stored as before.
