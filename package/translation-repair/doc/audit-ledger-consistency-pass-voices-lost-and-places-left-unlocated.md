# Audit ledger: the consistency pass, voices lost and places left unlocated

Paths written `src/...` in this record name the layout before 2026-10-18,
when the package `package/module/translation-repair` was split into the packages under `package/translation-repair`.
`doc/audit/translation-repair-package-split-map-2026-10-18.json` gives each file's current path.

The consistency pass of the run `TianqiChen66625` lost voices and left a chosen place unlocated.
The round n170 read the run and found why (`s241/notes-n170.md`,
which the lead wrote from n170's report,
since the tool refused n170's own Write of the report),
the lead ruled on each finding on 2026-10-15,
and the round n171 built the rulings.
This file holds the findings and what n171 fixed:
replies refused for the way they named a label (B639),
a run-off whose result the log never stated and a re-ask whose failure it hid (B640),
a carry choice that split and was not run off (B641),
a chosen place that no quote located (B642),
and an abandonment line that counted the wire and called it delivered characters (B643).
It also holds two defects that n171 introduced and found while fixing the lint findings of its landing:
a blank quote that hung the pass (B644),
and word edges that did not read a combining mark (B645).
The fifth finding of n170,
the empty truncations of the `minimax` seat,
needed no change beyond B639:
the seat sent a `max_tokens` of 10,822 and was cut with no answer and 40,377,
46,009 and 41,654 characters of reasoning,
and the vote was lost to the label rule on the re-ask and not to the truncation.

A later finding of the same pass,
from the lead's replay of the run's decisions on the n171 build,
is filed here too:
a first round or a carry choice with a lone leader and no strict majority was kept and not run off (B656).

The rounds landed on 2026-10-18.
n171 landed with n166,
n168 and n169 in the tests `555bb74aa0` and the code `29d6064708`,
with lint red at that commit by the message's own account (21 errors and 46 warnings,
all in files n171 added or changed).
The follow-up `71b0057920` (tests) and `0199690299` (code) fixes the lint findings and the two defects named here.
The consistency cache version moved from 4 to 5 in `29d6064708`,
since the rulings change what the pass asks and decides.
The blankness scan that the landing gate then failed is B646,
filed in
[Audit ledger: the source scans and the coverage census](audit-ledger-source-scans-and-coverage-census.md).
The lead's read of the run that these findings corrected is M226,
filed in
[Audit ledger: process mistakes, claims and commit messages](audit-ledger-process-claims-and-commit-messages.md).

Written on 2026-10-18 (UTC) from the notes of n170 (`s241/notes-n170.md`),
the briefs of n171 and n173 (`prompts/prompt-n171-consistency-voices.md`),
the four commit messages named in this file and the headers of the files they added.
n171 returned no hand-back file,
so its red logs,
its corpus measure and its replays are not among the sources,
and this file states none of them.
The brief asked n171 to rebuild the three ballots of the second group in a test
and to report which span the locator finds;
the header of `src/consistency-span-agreement.ts` states the answer,
and no report of it is among the sources.
Log lines are `L` followed by the number of the line in `corpus-pass-TianqiChen66625.log`.
The labels Q1 to Q5 are the notes' own and are no ledger ids.
No line numbers of source files are cited,
since the tree has since moved.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B639: replies that named the winning label in another form were refused

Found on 2026-10-15 by n170,
in the log and the stored replies of the run `TianqiChen66625`
(`s241/notes-n170.md`,
Q1).

Outcome written on 2026-10-18.
Red `555bb74aa0`,
fixed in `29d6064708`;
the fix is the round n171.

What the run showed:
seven replies of the consistency pass were refused for the way they wrote a label,
and six of them named the option that won anyway.
The labels are `OCCURRENCE n` (a shown place) and `CANDIDATE n` (a carry candidate).
`mercury` answered `Occurrence 3` and `Occurrence 2` (L 9533 and L 9572),
`glm` answered `"3"` twice (L 9538 and L 9577),
`minimax` answered `"2"` and then,
on the re-ask,
`Candidate 2` (L 9626 and L 9625),
and later `Candidate 2` and then `Candidate 1` (L 9640),
and `Kimi` answered `Candidate 4` and then,
on the re-ask,
`"4"` (L 9650).
The first ask of the run was refused by the same rule for the re-asks of `minimax`,
which answered `Occurrence 1` after its empty truncations.
The pass of `TianqiChen66624` ran the same guard code and had nine such refusals
(`mercury` `"1"` and `Occurrence 4`,
`glm` `"1"` and `"3"`,
`minimax` `Candidate 1`).
Over both passes the seats lost were `minimax` 7,
`mercury` 6,
`glm` 5 and `Kimi` 1,
and the asks were the first ask 10,
the run-off 4 and the carry choice 5.

The cause:
the refusal site (`src/chat-json-outcome.ts`) runs the caller's guard,
and every guard compared the written label to the shown ones exactly.
The run-off used `usableChoiceFor({ labels })` in `src/consistency-choice.ts`,
whose rule is `labels.includes(ballot.choice)`,
the carry choice used the same function,
and the first round used `isUsableConsistencyBallot` and `namesShownOnce` in `src/consistency-wire.ts`
for the choice,
the `keep` labels and the quote labels.
The sheets gave only a placeholder,
the labels appeared only in the headers of the fences,
and the response format held the choice as a plain string with no `enum`.
When a re-ask failed too,
`src/nudged-reask.ts` returned the first outcome,
so the log showed the first reply only (B640).

The lead ruled on 2026-10-15:
a reply that names exactly one shown label in another form
(case,
a bare number,
`Occurrence n`,
`Candidate n`,
surrounding white space)
counts as that label,
stored in its canonical form,
for the choice,
the keep labels and the quote labels;
an ambiguous or unknown name is still refused;
the sheets name the exact labels the reply may use,
and the response format lists them as an `enum`;
the re-ask's own failure is logged beside the first.

Fixed by n171.
`src/consistency-label-form.ts` (new) reads a name against the shown labels:
case,
white space around and inside the name,
and a bare number all count,
whether the name is the label or only the number the label ends in;
a name that stands for none of the shown labels,
or for two,
is refused,
and so is the word of another sheet
(a choice among candidates is shown places too,
so `Occurrence 2` there would name a place and not a candidate).
The label a reply is stored under is the shown one,
in the sheet's own form.
`src/consistency-label-sentence.ts` (new) writes the sentence
that names the exact labels in each sheet that asks for one
(`src/consistency-sheet.ts`,
`src/consistency-carry-choice-sheet.ts`),
and the reply formats list the labels as an `enum`
(`src/consistency-choice.ts`,
`src/consistency-wire.ts`).
The lint fixes of `0199690299` read white space and digits through the shared predicates
and no longer through regular expressions.

Test:
`src/consistency-label-forms.unit.test.ts`,
`src/consistency-label-sheets.unit.test.ts`,
`src/consistency-label-sentence.unit.test.ts`
and `src/corpus-run/pass-consistency-label-forms.unit.test.ts`,
with the refusal fixture `src/corpus-run/consistency-refusal.test-fixture.ts`
and the rendered sheets of the carry (`src/rendered-sheets-carry.test-fixture.ts`).
The tests commit `555bb74aa0` is red at its own commit by design,
and the code commit carries the fix.
The red logs of n171 are not among the sources.

Recurrence:
`mistake-prevention-inputs-defaults-and-values-read-from-text.md`,
"Two layers reading one refusal".

## B640: the log never stated what a run-off came to, and hid the failure of a re-ask

Found on 2026-10-15 by n170 (`s241/notes-n170.md`,
Q2),
and misread by the lead's first read of the run (M226).

Outcome written on 2026-10-18.
Red `555bb74aa0`,
fixed in `29d6064708`;
the fix is the round n171.

What the run showed:
the first ask of group 1 tied at 1,
1 and 1 of 3 usable.
The run-off heard 2 of 4 in its first window (L 9551,
"no quorum"),
the retry asked the other two (L 9552),
and 2 of 2 were heard (L 9562);
the recovery round of L 9563 to L 9579 asked again for the two refused seats whose votes were surplus.
The artifact's `runOff` holds `Qwen` O3,
`Kimi` O3,
`minimax` O3 and `deepseek` O2,
and the winner is OCCURRENCE 3 by 3 of 4.
That matches the owner's rule of 2026-10-11 in `doc/design-commitments.md`.
The log printed the first window's "no quorum" and no line for the run-off's final result,
so a reader of the log alone drew the wrong conclusion,
as the lead's read did.

Fixed by n171.
`runOffResultLine` in `src/corpus-run/pass-consistency-run-off-line.ts` (new)
writes one line after the run-off's final count:
how many of the choices heard were usable,
the quorum they were held to,
and the winner or the reason for none
(`the run-off counted 4 usable of 4 heard (quorum 2): OCCURRENCE 3 won`).
The voice-lost line now names the failures of both answers when a re-ask fails too,
not only the first's (`src/nudged-reask.ts`).

Test:
the tests commit `555bb74aa0` names
"one log line for a run-off's result and the re-ask's own failure logged beside the first".
No further list of the cases is among the sources.

Recurrence:
`mistake-prevention-messages-labels-and-counts.md`,
"Log lines that report nothing done".

## B641: a carry choice with no strict majority was not run off

Found on 2026-10-15 by n170 (`s241/notes-n170.md`,
Q2).

Outcome written on 2026-10-18.
Red `555bb74aa0`,
fixed in `29d6064708`;
the fix is the round n171.

What the run showed:
the fourth place of group 1,
the carry choice for OCCURRENCE 4,
was asked at L 9662 and heard 3 of 4 at L 9678:
`glm` chose CANDIDATE 3,
`deepseek` CANDIDATE 5 and `mercury` CANDIDATE 1.
`carryChoiceOf` in `src/consistency-carry.ts` found no strict majority,
and a split carry choice was not run off,
while the owner's rule of 2026-10-11 runs off a tie among contests.

The lead ruled on 2026-10-15:
a carry choice with no strict majority is run off among its tied leaders,
as the owner's rule does for contests ("run off the tied finalists"),
with the quorum,
the window and the label rules of the bench's run-off,
and a second split keeps the page as it stands.
The ruling is the lead's adoption of the owner's rule for this ask,
open to the owner's veto.

Fixed by n171.
`src/corpus-run/pass-consistency-carry-choose.ts` finds the candidates tied for first
when the choices reached the quorum and none has a strict majority (`carryRunOffFinalistsOf`),
asks them again,
counts the answers against the finalists only with the run-off's own quorum,
and records the run-off with its ballots;
a second split keeps the page as it stands,
and the pass logs how the run-off ended in words.
The artifact reader recounts a recorded carry run-off
(`src/corpus-run/artifact-two-lane-read-consistency-carry-recount.ts`,
with a unit test of the carry run-off reading).

Still owed:
the brief asked for the adoption to be recorded in `doc/design-commitments.md`,
open to the owner's veto,
and that file holds no record of it in the tree this entry was written from.

Recurrence:
`mistake-prevention-sheets-quorums-and-outcome-rules.md`,
"Which seats a quorum counts".

## B642: a chosen place whose judges quoted nested spans was located by none

Found on 2026-10-15 by n170 (`s241/notes-n170.md`,
Q3).

Outcome written on 2026-10-18.
Red `555bb74aa0`,
fixed in `29d6064708`;
the fix is the round n171.

What the run showed:
group 2 of `TianqiChen66625` chose OCCURRENCE 1 by 2 of 3
(`Kimi` O1,
`minimax` O1 with a keep of O2,
`mercury` O2).
The three judges quoted three different spans of the English of OCCURRENCE 1:
`Kimi` from "She wanted",
`minimax` from "stay beside",
and `mercury` from "wanted to stay".
`majorityText` in `src/consistency-majority.ts` needs one identical folded text from a strict majority,
`agreedWordsOf` found none,
`locatedRendering` gave `unlocated: no-agreement` in `src/consistency-count.ts`,
and `winnerOf` gave `none: chosen-unlocated`.
The group was lost.
The span of `Kimi` also lacks the page's "also" and stands nowhere on the page.
`TianqiChen66624` carried the same group because its three ballots quoted the same span.

The lead ruled on 2026-10-15:
a chosen place is located by the longest of its quotes that stands once in its English
and that a strict majority of the quotes naming that place contain;
the risk,
that a shorter span may leave words a judge meant to carry,
is recorded.

Fixed by n171.
`src/consistency-span-agreement.ts` (new):
a candidate span is one of the place's quotes,
read in the fold,
that a strict majority of the usable ballots' quotes of that place contain,
itself included,
at the edges of words;
a ballot that gave no quote of the place counts against every candidate.
The place is located by the longest candidate that stands exactly once in its English,
and a candidate that stands nowhere or more than once is passed over for the next longest.
Two candidates of one length are told apart by their code points,
so the same ballots always give the same span.
The risk is in the TSDoc of the file:
the span located is the longest one a majority can stand behind,
so it can be shorter than the words a judge meant to carry,
and the header states that the first place of group 2 is located from "wanted",
leaving out the "She also" that a judge's quote began with.
The owner may veto the rule.

Test:
`src/consistency-span-agreement.unit.test.ts`.
The reader of the located span in a recorded artifact
(`src/corpus-run/artifact-two-lane-read-consistency-span.ts`) is changed in the same commit.
The defects found in the first version are B644 and B645.

Recurrence:
`mistake-prevention-text-comparison-and-structure.md`,
"Quotes a judge gives back".

## B643: the abandonment line counted the wire and called it delivered characters

Found on 2026-10-15 by n170 (`s241/notes-n170.md`,
Q4),
and misread by the lead's first read of the run (M226).

Outcome written on 2026-10-18.
Red `555bb74aa0`,
fixed in `29d6064708`;
the fix is the round n171.

What the run showed:
three streams of `Qwen` were cut at the straggler deadline (L 9497,
L 9661 and L 9680) with no answer and 49,293,
36,301 and 37,784 characters of reasoning.
The line said they had "delivered" 1,308,758 characters and so on.
That figure is the length of the raw stream,
framing included (`partialText.length` in `src/abandon-kind.ts` and `src/stream-drain.ts`),
about 27 times the reasoning.
The provider route sends a `max_tokens` of 20,894 (`src/synthetic-client.ts` and the model card),
and a completed `Qwen` call hit it (L 9498 and L 9499).
The runaway watch bounds the volume of the answer only (`src/stream-runaway-watch.ts`
records why a reasoning bound was refused),
and no answer arrived,
so nothing fired.
No tail sample exists,
since a cut stream is not stored.

The lead ruled on 2026-10-15:
the line reports the characters of the answer and of the reasoning,
and no reasoning bound is added,
since one was measured and refused.

Fixed by n171.
`describeAbandon` in `src/abandon-kind.ts` reads the characters of the answer and of the reasoning
that the stream counted,
carried on the cut error as `generatedChars`
(the commit changes `src/stream-cut.ts`,
`src/stream-drain.ts`,
`src/chat-contract.ts` and `src/stage-call.ts` beside it),
and prints `cut-mid-reply after N answer chars and M reasoning chars (K chars on the wire), first byte at Tms`.
Where the stream counted none,
it prints the wire length alone under its own name.
The comment in the code gives the run's figures as the reason.

Test:
the tests commit `555bb74aa0` names "answer and reasoning characters on the abandonment line".
No further list of the cases is among the sources.

Recurrence:
`mistake-prevention-messages-labels-and-counts.md`,
"Counts in printed text".

## B644: a blank quote hung the consistency pass

Found on 2026-10-18 by the author of the lint fixes that followed the landing,
in the span agreement of B642.

Outcome written on 2026-10-18.
Red `71b0057920`,
fixed in `0199690299`.

What a reader of a run would have seen:
a pass that did not end.
`containsWords` in `src/consistency-span-agreement.ts` asks whether the words of one quote stand in another.
The search for an empty text finds it at every offset and again at the end,
so the loop that steps one offset past each find never stops once the outer quote ends in a letter.
A reply whose quote folds to nothing passes intake,
so any judge could stall a run.

Fixed in `0199690299`:
an empty text is contained only in another empty one.

Test:
a case in `src/consistency-span-agreement.unit.test.ts` reads the candidates in a worker
that the shared hang stop ends:
at the red commit both readings report "hung" after the stop.

Recurrence:
`mistake-prevention-text-comparison-and-structure.md`,
"Text that shows nothing".

## B645: the word edges of the span agreement did not read a combining mark

Found on 2026-10-18 by the author of the lint fixes that followed the landing,
in the span agreement of B642.

Outcome written on 2026-10-18.
Red `71b0057920`,
fixed in `0199690299`.

What a reader of a run would have seen:
the edges of a word were read by the category of the letter or number beside them and read no combining mark.
The words "cafe" therefore counted as contained in "cafe" written with a decomposed accent,
which the fold leaves as it is,
and the located span cut a word in two.
The locator reads a mark as part of the word (`src/consistency-shown-edge.ts`),
and the span agreement did not.

Fixed in `0199690299`:
a word runs on through a combining mark,
using the locator's predicate,
so a decomposed accent after a match makes it part of a longer word.

Test:
a case in `src/consistency-span-agreement.unit.test.ts`.
The commit pins in a third case,
which passes,
the order of candidates by code points and not by UTF-16 units,
since the fix replaces the string spreads that the linter refuses.
Two of the three new cases fail at the red commit.

Recurrence:
`mistake-prevention-text-comparison-and-structure.md`,
"Words inside words".

Revised on 2026-10-19 (UTC):
a word that runs on through a combining mark was not the whole of the locator's test of an edge.
The locator also refuses a match at a joint of a longer word,
such as an apostrophe or a hyphen between letters or digits,
and the span agreement did not.
B675 holds the correction.

## B656: a lone leader without a strict majority was kept and not run off

Found on 2026-10-18 by the lead's replay of the `TianqiChen66625` decisions on the n171 build.

Outcome written on 2026-10-18.
Red `c3d881805b`,
fixed in `7123ffb2f8`.

What the replay showed:
in the first round of group 1,
four usable ballots split 2-1-1.
No label had a strict majority and one label led alone,
so `tiedForFirstOf` in `consistency-majority.ts` named no finalists,
no run-off was asked and the page was kept.
The build before n171 ran off the first three choices and chose one of them.
The rule of B641 for a carry choice had the same gap:
it ran off ties only.

The lead adopted a rule on 2026-10-18,
open to the owner's veto,
extending the owner's rule of 2026-10-11 ("Run off the tied finalists"),
which covered ties only.
When the quorum is met and no label has a strict majority,
the finalists are the labels with the top count.
When exactly one label has the top count,
the labels with the second-highest count join them.
A tie for first behaves as before,
and a second split keeps the page.
The rule applies to the carry run-off alike.
The entry is recorded in `design-commitments.md`.

Fixed in `7123ffb2f8`.
`runOffFinalistsOf` (renamed from `tiedForFirstOf`) and `runOffChoiceFinalistsOf` (renamed from `tiedChoicesOf`)
in the consistency package add the labels with the second-highest count to a lone leader,
and name none when fewer than two labels drew a ballot.
`carryRunOffFinalistsOf` and `countConsistency` take the finalists from them,
so the first round and the carry choice both ask the run-off of a 2-1-1 split.
The run-off reason (`runOffReasonOf`) says that the choices reached no strict majority
and names the finalists with their ballots,
and the invariant messages of the pass speak of finalists and not of a tie.
The sheet of the run-off already said that the first round was split between the places shown,
so its wording is unchanged.

Cache version:
stays at 5.
The finalists are counted from the stored ballots and are not stored,
and the sheet of a run-off shows only its finalists under a key that digests the sheet,
so a run-off among three labels asks a question under a key of its own
and a run-off among tied labels keeps its key byte for byte.

Consolidation:
the run-off of a tied slate (`runoffTiedFinalists` in the translate stage)
shares no function with this rule,
since only the corpus-pass and artifact packages depend on the consistency package,
and it is unchanged.

Still owed:
the artifact reader recounts a recorded group from its ballots,
so it now refuses a group recorded with a lone leader's split and no run-off,
which the previous rule accepted (the 2-1-1 group of the n171 run is one).
A recorded carry with no run-off is still read as written.
Whether the reader keeps a rule for groups recorded before this one,
or the artifact generation moves,
is for the lead and the owner.

Test:
cases for the finalists in `consistency-majority.unit.test.ts` (2-1-1 of 4 names three,
2-2-1 of 5 names the two tied,
2-1 of 3 names none,
1-1-1-1 names all four,
labels sharing the second count all join),
the carry mirrors in `consistency-carry-run-off.unit.test.ts` and `pass-consistency-carry-run-off.unit.test.ts`,
the run-off of a 2-1-1 refrain and a second split that keeps the page in `pass-consistency.unit.test.ts`,
the reason in `pass-consistency-settle.unit.test.ts`,
and the recount in `artifact-two-lane-read-consistency-span-winner.unit.test.ts`.
The carry case that pinned "a single leader without a majority is not a tie" now pins the new rule.
Most of the new cases fail at the red commit for the reason stated:
a lone leader names no finalists.

Recurrence:
`mistake-prevention-sheets-quorums-and-outcome-rules.md`,
"Which seats a quorum counts".

## B657: the carry run-off sent the first ask's sheet when every candidate was a finalist

Found on 2026-10-18 by an independent review of the n166 to n173 production diff (finding R1,
medium).

Outcome written on 2026-10-18.
Red `9684e3e00f`,
fixed in `c351d52644`.

What the review showed:
a carry choice with two candidates can only fail to have a strict majority by splitting evenly,
and a three-way split makes all three candidates finalists,
so `carryRunOffFinalistsOf` named every candidate and the run-off sheet was the first ask's sheet byte for byte.
The production client (`promptUniqueClient`) answers a repeated model and prompt from the first payload,
in memory and from the store,
so every seat replayed its first ballot,
the tie recurred,
and the artifact and the log recorded a second round that was a copy of the first.
The sheet of the run-off among places already differed from its first round's,
so only the carry choice had the gap.

Fixed in `c351d52644`.
`buildCarryChoiceMessages` in `consistency-carry-choice-sheet.ts` takes `runOff`,
and its rules then say that a first round of judges was split between the candidates in the fenced content,
that none had more than half of the choices,
that the judge is asked again between these candidates only,
and that the choices of the first round are not shown and bind no one.
`askCarryChoice` passes the flag it already held.
The user message is unchanged.
The rendered-sheets fixture holds the run-off sheet with every candidate a finalist
(`rendered-sheets-carry.test-fixture.ts`),
and the places the carry sheets share moved to `rendered-sheets-carry-places.test-fixture.ts`
so the fixture stays under the line limit.

Cache version:
stays at 5.
The sheet of the first ask is byte for byte what version 5 has,
so no first-ask key moves.
The sheet of the run-off changed,
and its key (`choice-run-off`) digests the sheet,
so a carry run-off stored by an earlier build is asked afresh under a new key,
which is the intended effect:
the stored one was a replay of the first round whenever every candidate was a finalist.
The reason is recorded in `consistency-cache-version.ts`.

Test:
three cases in `pass-consistency-carry-run-off.unit.test.ts`
(the sheets of the first ask and of the run-off differ in a three-way split,
a tie breaks when the seats answer a new sheet anew,
and behind `promptUniqueClient` the provider is called six times and not three)
and a builder case in `consistency-carry-choice-sheet.unit.test.ts`.
All three corpus-pass cases failed at the red commit,
and the guard demo (reverting the rules in the sheet builder only) fails them again together with the builder case.

Recurrence:
`mistake-prevention-sheets-quorums-and-outcome-rules.md`,
"Sheets every model reads".

## B658: the slot of a frame was agreed only beside a quote equal to the located words

Found on 2026-10-18 by the same review (finding R2,
medium-low).

Outcome written on 2026-10-18.
Red `9684e3e00f`,
fixed in `c351d52644`.

What the review showed:
the place is located by the longest quote that stands once and that more than half of all the usable ballots' quotes contain,
but `agreedSlotWordsOf` counted a slot only beside a quote equal to the located words.
Three judges who quoted three nested spans of one refrain and gave the same slot beside their own quotes
produced one slot against three usable ballots,
so the place was located with no slot and left to the writers.

Fixed in `c351d52644`.
`agreedSlotWordsOf` counts a slot given beside any usable quote that contains the located words,
with the same test as the locator (`containsWords`,
moved to `consistency-contains-words.ts` so both read one test).
The denominator is still every usable ballot.
`markEvidenceOf` stays exact,
since a mark beside `the cat naps` is not beside `the cat`.
The header of `consistency-slot-in-span.ts` says so.

Stored artifacts:
the recount reads the same rule through `slotInSpan`,
and it would have read a stored artifact differently,
so the rule is gated by generation (B664):
a section of generation 17 or earlier is recounted with `exact-quote`,
as it was written,
and a section of generation 18 with `containing-quote`.
The five stored run directories read accepted under the new build (see B664).

Test:
a nested-quote case in `consistency-span-agreement.unit.test.ts` (slot `naps`,
expected at 4)
and a pin that a non-containing quote shares no slot,
the two agreement modes in `consistency-majority.unit.test.ts`,
and the generation split in `artifact-two-lane-read-consistency-span.unit.test.ts`.
The nested-quote case failed at the red commit.

Recurrence:
`mistake-prevention-text-comparison-and-structure.md`,
"Quotes a judge gives back".

## B659: the proof read no link twin, so it disagreed with the cut about the same apostrophe

Found on 2026-10-18 by the same review (finding R3,
low).

Outcome written on 2026-10-18.
Red `9684e3e00f`,
fixed in `c351d52644`.

What the review showed:
the cut of a word reads the twin of the English a reader sees (`closingJointsOf`),
in which a link that shows nothing is not there,
but `joinedMarksOf` read the raw fold,
so `see [](u)'kit here` recorded the apostrophe as joined and the proof treated it as inert,
where the same text with no link reads as a leading mark.

Fixed in `c351d52644`.
`joinedMarksOf` moved to `consistency-joined-marks.ts` and builds the folded links of the English
and calls `closingJointsOf`,
so the proof and the cut read one unit alike.
Its TSDoc says that it makes the reading `closingJointsOf` makes.
The proof reads one English alone,
so it holds that English's own links and none of the page's definitions,
and a reference defined in another slice is plain text to it
(the same limit as B661,
stated in the header of the new file).

Test:
two `readProof` cases in `consistency-fold-apostrophes.unit.test.ts`:
`see [](u)'kit here` joins nothing and reads as a leading mark,
and `see [\]](u)'kit here` is a joint (a pin that passed at the red commit),
and the cases moved with the function to `consistency-joined-marks.unit.test.ts`.
The first failed at the red commit.

Recurrence:
`mistake-prevention-text-comparison-and-structure.md`,
"Words inside words".

## B660: a link with any label counted as a closing bracket before an apostrophe

Found on 2026-10-18 by the same review (finding R4,
low).

Outcome written on 2026-10-18.
Red `9684e3e00f`,
fixed in `c351d52644`.

What the review showed:
`[kit!](u)'s bowl`,
`[Kit.](u)'s` and `[kit ](u)'s` read in plain text as `kit!'s`,
where `!` closes nothing,
yet `afterBracket` in `consistency-word-joints.ts` took the `)` of the destination as a closing bracket
because only links of marks alone were excluded,
and the forward reading (`closingJointsOf`) stepped the destination back to the `]` that closes.
A stand that ends at `kit!` or begins at `s bowl` was cut that the same text without a link does not cut.

Fixed in `c351d52644`.
A link with a label is a closing bracket before an apostrophe only when its label ends in a closing mark.
`linkLabelEndsOpenOf` names the label that ends in neither a closing mark nor a Latin letter or digit,
`labelledLinksOf` records it (`endsOpen`),
and `foldedLinksOf` carries `openLabelEnds` (the backward reading)
and `openLabelStretches` (the forward one,
which `linkTwinOf` removes like an empty link's stretches).
A label that ends in a closing mark (`[kit)](u)'s`) still joins,
and a label that ends in a letter keeps its reading.

Test:
`consistency-closing-links.unit.test.ts` holds the joints of the three open-label links against their unlinked twins,
the joints of `[kit)](u)'s bowl`,
and `locateWords` for `s bowl` on the linked and plain forms;
`consistency-joined-marks.unit.test.ts` holds the proof's reading.
The two cases of `consistency-closing-links.unit.test.ts` failed at the red commit,
and the guard demo (switching off the backward and forward readings only) fails them again with the proof case.

Recurrence:
`mistake-prevention-text-comparison-and-structure.md`,
"Words inside words".

## B661: the artifact recount hands the slot readers no page definitions

Found on 2026-10-18 by the same review (finding R5,
low).

Outcome written on 2026-10-18.
Red `9684e3e00f` (a pin that passes),
limitation recorded in `c351d52644`.

What the review showed:
the pass reads a slot with the link and footnote definitions of the whole page,
and the recount (`slotInSpan` in `artifact-two-lane-read-consistency-span.ts`)
and `locateWords` in `artifact-two-lane-read-consistency-span-located.ts` read it over the span alone.
A span `a[b]c` whose `[b]` is a link only under a definition in another slice is one word for the pass,
which records no slot,
and a word `b` with a slot for the recount,
which then expects a slot the pass could not find.

Decision,
made by the lead:
the limitation is written down and pinned,
and the readers are not changed.
The choice was between rebuilding the definitions and accepting the limit,
and the definitions cannot be rebuilt from the file:
the section reader is handed only the section,
its path,
its generation and the slice indexes,
the `slices` rows of the consistency section hold only the slices the pass changed (as they ended),
the comparison rows hold each lane's text from before the consolidation,
and the consolidated English the pass read is recorded nowhere,
so rebuilding it takes the corpus files at the artifact's commit (`rebuildPreparation`),
which a reader of one file does not have.

Recorded in `c351d52644`:
the head of both readers says so,
and the case `a[b]c` with slot `b` and no recorded slot is refused with the slot refusal is pinned
in `artifact-two-lane-read-consistency-span.unit.test.ts`.
The shape needs a definition in another slice and a slot at a word joint.
If a reader of whole runs (with the corpus at hand) ever needs it,
the definitions would be passed there and not through the section reader.

Superseded for generation 18 by B665:
the pass records the definitions that move a slot and the recount reads them,
so this limit and its pin remain for generation 17 and earlier only.

## B662: five TSDoc contradictions of the consistency package

Found on 2026-10-18 by the same review (finding R6,
low).

Outcome written on 2026-10-18.
Fixed in `c351d52644`,
no test (wording).

What the review showed and what changed:
the version 5 paragraph of `consistency-cache-version.ts` said both that no stored answer of version 4 replies to the question
and that a version 4 record of the choice stands for the first ask:
the key hashes the number,
so no version 4 record is read,
and the paragraph says so.
The same paragraph said a run-off among the tied labels keeps its key and that the wording of the sheets is the same,
which stopped being true for the carry run-off (B657).
"A strict majority of the quotes naming the place" in `consistency-cache-version.ts`,
`consistency-count.ts`,
`consistency-span-agreement.ts`
and the artifact recount says that more than half of all the usable ballots' quotes contain the span,
as the code and its test do.
`consistency-markup-ranges.ts` said five kinds of markup in its header and three in the type's doc and holds eight fields:
both say eight.
`consistency-link-label.ts` said that `marks` holds no letter or digit,
and the test is the Latin letters and digits and the forms that fold to them,
so a Han,
Cyrillic or Greek label is `marks`:
the wording says so.
The `joinedMarksOf` TSDoc is B659.
The phrases "tied for first" and "tie for first" were reworded where they meant the finalists of a split
(the headers and fixtures of the run-off tests,
the artifact generation docs);
they stay where a scene really is a tie.

Left as it is,
then corrected in B667:
the refusal text in `artifact-two-lane-read-consistency-span.ts`
("words that a strict majority of the quotes naming the place contain")
was pinned by two tests,
so it was not reworded here.

## B663: a pinned bench start was not checked

Found on 2026-10-18 by the same review (finding R7,
low).

Outcome written on 2026-10-18.
Red `9684e3e00f`,
fixed in `c351d52644`.

What the review showed:
`editorBenchStart` and `benchStart` are documented as a whole number below the size of the bench,
and `rotatedBench` handed them to `slice`,
so `-1` counted from the end,
`NaN` and the size itself left the bench unrotated,
and `1.5` truncated,
and a fixture that pinned a seat got another seating than it named.

Fixed in `c351d52644`.
`assertBenchStart` in `stage-bench-start.ts` refuses a pinned start that is not a whole number from 0 to the size less one
with `StageBenchStartError`,
which names the value,
the last seat and the size of the bench and is marked as naming only numbers.
`rotatedBench` asks once,
where the value and the bench are both in hand,
so `StageSettings.editorBenchStart` and `gatherStageVoices` (`benchStart`) are covered by one check,
and their TSDoc says so.
The source scans' inventories name the new class and its message parts.

Test:
`stage-fanout-window.unit.test.ts` refuses `-1`,
`NaN`,
`1.5` and the size and accepts 0 and the size less one,
and `stage-bench-start.unit.test.ts` holds the check and the error.
The refusal case failed at the red commit.

Recurrence:
`mistake-prevention-inputs-defaults-and-values-read-from-text.md`,
"Numbers read back from text".

## B664: artifact generation 18 and the readers that keep the earlier rules

Decided on 2026-10-18 by the lead,
when the findings of the review were repaired.

Outcome written on 2026-10-18.
Red `9684e3e00f`,
fixed in `c351d52644`.

What moved:
the pass stamps generation 18 (`ARTIFACT_SCHEMA_VERSION_V18`).
Two recounts changed from generation 17.
The finalists of a run-off are the labels with the top count and,
for a lone leader without a majority,
the labels with the second-highest count (`runOffFinalistsOf`),
where generation 17 and earlier ran off only the labels tied for the highest count (`runOffTiedFinalistsOf`).
The slot of a frame counts beside any quote that contains the located words (`containing-quote`),
where generation 17 and earlier counted only beside a quote equal to them (`exact-quote`).
The predicates `artifactGenerationRunsOffLoneLeader` and `artifactGenerationReadsSlotsBesideContainingQuotes` gate both,
and `expectedWinnerOf` and `assertSpanRenderingAgrees` take the generation.
No blanket tolerance was added to the group reader:
a generation 18 group with a lone leader and no run-off is refused,
and a generation 17 group with a run-off recorded for a lone leader is refused,
which no pass of that generation wrote.

Reader check:
`read-all.ts` over the `artifacts` directories of `TianqiChen66621`,
`TianqiChen66622`,
`TianqiChen66623`,
`corpus-pass-TianqiChen66624` and `corpus-pass-TianqiChen66625`,
with this build,
accepted all five artifacts (generations 14,
14,
15,
17 and 17).
The group of `TianqiChen66624` with a lone leader and no run-off,
which the previous reader refused,
reads as it was written.

Side effects:
no cache key reads the schema constant,
and the consistency cache hashes the sheet and its own version,
which did not move,
so the bump changes no key.
No page is rewritten:
the page republisher and the published-run checks read the generation only to choose how to read the entry,
and every predicate they use is a `>=` that 18 satisfies,
so the published-run checks print `read-at-generation=18` for artifacts the pass settles and 17 for those it did not.
A pass refuses to resume into a directory of generation 17 artifacts (`SchemaGenerationError`),
which names the artifacts and the ways forward,
so a run in progress is moved aside or finished with the build that began it.

Carry reader,
tolerance left as it is,
then gated in B666:
the carry recount read a split recorded with no run-off as written,
and requires a recorded run-off's finalists to equal `runOffChoiceFinalistsOf`.
It was left as the brief said,
since the carry recount was not handed the generation.
B666 hands it the generation:
a generation 18 carry with a split and no run-off is refused,
and generation 17 and earlier keep the tolerance.
B665 adds the page definitions to the generation 18 record.

Test:
`corpus-pass-guards.unit.test.ts` refuses a generation 17 directory,
the generation split of the recount is in `artifact-two-lane-read-consistency-span.unit.test.ts`,
the finalists of both rules in `consistency-majority.unit.test.ts`,
and the generation printed by the published-run checks in the `run` package tests.

Recurrence:
`mistake-prevention-stored-files-and-cached-decisions.md`,
"Files a later run reads back".
