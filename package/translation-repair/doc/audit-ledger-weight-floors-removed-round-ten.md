# Audit ledger: the weight floors removed, round 10

Round 10 of the work recorded in
[the weight floors removed and two ballots deciding](audit-ledger-weight-floors-removed-and-two-ballots-decide.md),
[rounds 2 and 3](audit-ledger-weight-floors-removed-rounds-two-and-three.md),
[round 4](audit-ledger-weight-floors-removed-round-four.md),
[round 5](audit-ledger-weight-floors-removed-round-five.md),
[round 6](audit-ledger-weight-floors-removed-round-six.md)
and [rounds 7 to 9](audit-ledger-weight-floors-removed-round-seven.md),
which holds B933 to B947:
the nits that the eighth and ninth independent re-checks left,
answered on a branch cut from the merge of main at `16c2c7ddb1` (`1362f15c40`).
The corrections in place are in the entries they correct,
in the file of rounds 7 to 9:
B938 (the known limit),
B945 (benches of seven) and B947 (the sweep's result lines).
This round continues at B948.
Round 11,
the second round of the follow-up,
is kept in this file at B951,
since the file of rounds 7 to 9 is near the limit of a thousand lines.
Round 12,
the third round of the follow-up,
is kept in this file at B952.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## Round 10 at a glance

The eighth re-check (nit A and the nit on a TSDoc line)
and the ninth re-check (minor finding 1 and nits 2,
4 and 5)
left five items that no code defect forced.
They are answered on a branch cut from the merge of main at `16c2c7ddb1` (`1362f15c40`).
No re-check found a defect in the code of the rounds that the linked files record.

The four-seat bench that a split settles by the seat the rotation leaves out is B948.
The position scan that read a verb,
a floating quantifier and "that" as the object of a comparison,
and the thirteen words that left its list,
are B949.
The refiner left out of the window,
the TSDoc reflow,
the cache versions and the gates are B950.
The corrections in place are in B938 (the known limit),
B945 (benches of seven) and B947 (the sweep's result lines).

## Round 11 at a glance

The review of round 10 left five items that no code defect forced:
the scan still passed "that" before a bare verb,
B944's known limit was confirmed by the re-check and still stood as open,
the glance of round 9 said that main did not move,
two test titles still called words members of the compared list that had left it,
and this record had no entry for the round.
They are answered in B951,
and the corrections in place are in B938,
B944,
B949 and the glance of round 9.

## Round 12 at a glance

The re-check of round 11 found no major finding,
three minor findings and six nits.
The ruling of 2026-10-25 on its measured counts replaces the list of verbs after "that" by the inverse reading,
which is B952.
The same entry pins the nouns and the opener after a quantifier,
corrects the commit that B948 named for the window,
and answers the nits.
The corrections in place are in B938,
B945,
B948,
B949 and B951.

## B948: a split four-seat bench is settled by the seat the window leaves out (round 10)

Status:
recorded on 2026-10-25 (UTC);
no code change.
On the owner's veto-open list.

Minor finding 1 of the ninth re-check.
Since `1dee3ca797` the selection window is quorum plus one,
so a bench of four asks three seats where it asked four.
Two shapes follow,
both measured by the re-check on the real `selectChunkPatch` with ballots scripted by seat,
under the sweep's salts 0 to 9:
- A 2-2 split:
  two seats name one candidate and two name another.
  The full bench is a tie,
  which is indecision and sends the fallback repair on,
  but the window of three decides,
  and the winner follows the seat it omits:
  candidate 2 ships at salts 0,
  3,
  4,
  7 and 8,
  and candidate 1 at salts 1,
  2,
  5,
  6 and 9.
- A lone namer:
  one seat names and three decline.
  Where the window leaves the namer out,
  every judge asked declines and the existing English ships (salts 2 and 6);
  otherwise the fallback repair goes on.

The same already held for benches of five and over,
where the window has applied since `b71a553855b05333cf985f3161409d9d98da2993`
(2026-09-09,
"ask quorum plus one seat per round"),
which asked the whole bench on four seats and the window from five;
it now holds for a bench of four.
Correction in place (2026-10-25,
round 12):
this said "since `b6927f7ba9`",
a docs commit of main at which the claim was verified,
which reads as the commit that introduced the window.
`git log -S THINNEST_WINDOWED_BENCH` lists `b71a553855` as the commit that added the constant,
and the constant is 5 there.
It breaks no ruling:
two ballots decide,
one judge alone never decides,
a run always ships,
and whatever goes on still has to pass the resolution checkers and beat the existing English on the measurements.
It is a property of every window narrower than its bench,
and it is on the owner's veto-open list.
The one-line alternative,
asking the whole bench on benches of fewer than five seats,
would make four-seat benches deterministic;
it is the owner's choice and this record does not recommend it.
The docs do not say whether any real roster's selection bench is four seats,
so how often the effect occurs in a run is not known.

## B949: the position scan read a verb, a floating quantifier and "that" as the object of a comparison (round 10)

Status:
fixed on 2026-10-25 (UTC),
red `a871f02aaf`,
fix `e1fcb14d66`;
thirteen words removed from the list,
red `ba23352795`,
fix `1af8000a84`.

Nit A of the eighth re-check.
The comparison list of the position scan (`COMPARED_OBJECTS`) hid a pointer in three ways that B938 did not name.
A word of the list can be a verb
("the lines below zero the counter",
with zero,
exact,
baseline,
threshold,
half and total),
the quantifiers all,
both,
each and some float away from their noun
("the cases below all pass"),
and "that" can open a clause
("the statement above that a run-off keeps its key holds").
Two pointers in the texts passed for the last reason,
in `consistency/src/consistency-cache-version.ts` and `slice-reading/src/pairing-cache-version.ts`,
and the same shape hid two more in ledger entries.

What the scan reads now.
The words of the list that are verbs or open a clause
(baseline,
zero,
threshold,
total and that)
compare unless a determiner,
a quantifier or a pronoun follows them,
joined by spaces alone.
The quantifiers all and each compare only when a determiner,
of,
else,
other or others follows,
or when no word joins them to the word after them
("above all,
it naps").
The reading sits in `source-scan/src/position-references-compared.test-fixture.ts`,
which the line limit of the scan file required,
and the vocabulary doc of `COMPARED_OBJECTS` states it.

What it still does not read.
- A word of the list followed by a bare verb:
  "the lines below that write it" (`typography/src/semantic-wrap-binary.ts:247`)
  and "the paragraphs above that explain"
  (`audit-ledger-test-suite-unrun-code-reverification-of-2026-10-05.md:15`)
  are pointers in the texts and the scan does not find them,
  since it cannot tell a verb from a noun.
  They were not reworded.
  Correction in place (2026-10-25,
  round 11):
  B951 makes the scan find "that" before a verb it names and rewords both pointers;
  a verb it does not name still passes.
  Correction in place (2026-10-25,
  round 12):
  B952 replaces that list by the inverse reading,
  so "that" before any word that is not a noun the scan knows is found.
- A quantifier followed by "of" compares:
  "below each of them" is left,
  and so is "the cases below all of which fail".
- Half left the list with the thirteen words:
  "votes below half the bowl" reads as a pointer,
  and the one comparison of that shape in the texts is exempted with its reason.
  Putting half back is the owner's call.
- Any other word that is both a noun and a verb,
  added to the list later,
  lets a pointer through unless it joins a group.

What it found.
Over the 4113 texts the guard reads (the package's sources,
docs,
READMEs and task files,
and the living repository docs),
the tightened scan finds ten phrases more than before.
Seven are pointers and were reworded:
the two cache-version docs,
the ledger entries `audit-ledger-process-launch-checks-and-cache-versions.md` and
`audit-ledger-which-seats-a-quorum-counts.md`,
`adjudication/src/self-certification-weight.unit.test.ts`,
`typography/README.md` and `doc/decision/translation-repair-front-matter-guard.md`.
Three are comparisons and are exempted with their reasons:
"above both reference rules" (`archive/src/reference-precedence.unit.test.ts`),
"above some slice count" (`run/src/corpus-run/cap-override.ts`)
and "below half an even roster" (`stage/src/stage-quorum.unit.test.ts`).
The raw findings outside the guard's own files are 77 before and 80 after,
the same 77 keys and the three exempted ones;
unexempted findings are none before and after,
and no exemption is stale.

The thirteen words.
Re-check 8 named 13 of the 40 words as changing nothing on 3925 texts.
Re-measured on the 4113 texts of this tree before the fix,
deleting one word at a time and counting unexempted findings,
11 words changed nothing:
an,
his,
her,
exact,
regression,
four,
six,
seven,
eight,
nine and ten.
Anything and three each hit in the living decision docs
(`translation-repair-runaway-call-termination-answer-volume-bound-measured-and-corrected.md:171`,
`translation-repair-multi-provider.md:14` and `:495`),
so they stay.
After the fix,
counting raw findings with no exemption applied over the 4114 texts (the new fixture file is one),
13 words changed nothing:
an,
his,
her,
some,
half,
exact,
both,
four,
six,
seven,
eight,
nine and ten.
Regression is not one of them:
its one hit (`adjudication/src/select-candidate.unit.test.ts:132`,
"resolution above regression count") is a true comparison that the window of a neighbouring exemption covers,
so counting unexempted findings alone would have removed it.
The 13 words left the list,
which now holds 27,
and the raw findings outside the guard's own files are the same 80 keys before and after the removal.
A position before one of the 13 words is a pointer,
so a comparison such as "a rate above four" now needs an exemption with its reason.
The owner may prefer to keep the words the texts do not need but a reader might write.

Commit-message slips of this round:
`e1fcb14d66` is titled for the two cache-version pointers and rewords seven pointers;
its body lists all seven.
No false statement was found in the other messages.

## B950: the refiner left out of the window is pinned, with the cache versions and gates (round 10)

Status:
done on 2026-10-25 (UTC),
test `b0c05bdcd5`,
loop correction `80923e74b7`,
reflow `41faa2539b`;
no production code changed.

Nit 4 of the ninth re-check.
Round 9 moved the two refine cases that read the self-vote to a bench of three,
so the other half had no case:
on a bench of four,
a window that leaves the refiner out has no self-vote finding
and the winner draws 3 of 3 ballots.
The case "LEAVES OUT THE SELF-VOTE FINDING AND THE DISCOUNT WHEN THE WINDOW OF QUORUM PLUS ONE LEFT THE REFINER OUT ..."
of `repair/src/refine-stage.unit.test.ts` seats the refiner at each of the four places of the bench in turn.
The four seatings send one selection sheet
(twelve sheets sent,
one distinct),
so the window leaves out one seat of the four whatever the rotation,
and the refiner takes each seat once:
it is left out in exactly one seating.
The case expects three seats asked in every seating,
the refiner left out once,
and the findings of each seating to follow whether it was asked:
`select-self-vote (<the refiner>)` with `refine-selected (weight 2.5 of 3 ballots)`,
or neither the finding nor the discount
(`refine-selected (weight 3 of 3 ballots)`).
A first draft varied the closing sentence of the paragraph to move the rotation and was dropped:
over 200 sheets the hash started the bench only at the third or the fourth seat
(the low bits of FNV-1a over the sheets this fixture can make),
so the refiner,
the first seat,
was never left out at salt 0.

Evidence:
- Under salts 0 to 9,
  run on the single file with the sweep's hook from the package directory,
  every case of the file passes,
  the new one included.
- Planted fault,
  in a scratch copy of the family at `80923e74b7` (`translate-stage/src/candidate-select.ts`):
  the self-vote findings named from the stake in candidate 1 instead of from the ballots cast,
  so the finding appears for a refiner the window left out.
  The case fails at all ten salts,
  with `select-self-vote` only in the actual findings and `refine-selected (weight 3 of 3 ballots)`
  only in the expected ones,
  and the other cases of the file pass.
  The control copy without the fault passes at all ten.

The TSDoc nit of the eighth re-check.
The paragraph of `SHIPS_A_LEVEL_TIE_ELSE_PRODUCES_AGAIN` in `adjudication/src/select-decline-consequence.ts`
that broke a line after "once" is rewrapped at 120 columns (`41faa2539b`).
No word and no string constant changed.

Cache versions.
`git log --extended-regexp -G '_CACHE_VERSION = [0-9]+' 1362f15c40..HEAD` lists nothing,
and the control over `938e84afb9..b37685721e` lists `6c81859c79` and `53e23a728c`,
so the query reads the declarations.
No prompt,
sheet or stored decision changed,
so no number needed to move.

Gates,
keyless and one at a time,
on the tree of `80923e74b7`.
Build,
`lint:types` and `lint:oxlint` (0 warnings and 0 errors) passed in repair,
adjudication,
consistency,
slice-reading,
typography and source-scan.
`test:unit` passed 149,
118,
198,
59,
43 and 68 suites in those packages with none failed.
`source-scans` passed 53 on the tree with this text.

## B951: the position scan reads "that" before a bare verb, and the items the review left are closed (round 11)

Status:
fixed on 2026-10-25 (UTC),
red `a5d22db40b0fef9dd0ce15585ae4ffa0697550df`,
fix `44e3b92e03980859d5beca29a16e205a6388be9f`;
the corrections in place are in B938,
B944,
B949 and the glance of round 9.

The scan.
B949 left two pointers that the scan passed:
"the lines below that write it" (`typography/src/semantic-wrap-binary.ts:247`)
and "the paragraphs above that explain"
(`audit-ledger-test-suite-unrun-code-reverification-of-2026-10-05.md:15`).
"that" now reads as a pointer when a word of `VERBS_AFTER_THAT` follows it,
joined by spaces alone.
The list holds 29 verbs in the form a plural subject takes
(write,
explain,
say,
cover,
name and the like),
and it sits in `source-scan/src/position-references-vocabulary.test-fixture.ts`;
the reading is in `position-references-compared.test-fixture.ts`.
A list,
because the scan cannot tell a verb from a noun.
Words that are also nouns stay out when a text could compare with them,
as a precaution that the texts neither support nor contradict
(count,
use,
run,
set,
test,
call and return),
and name,
list,
record,
note,
check and hold stay in because no text the guard reads compares with them.
After a position and "that",
the words that follow in the texts are floor,
speed,
size,
percentile,
calibration,
opening,
"is" and "and" for the comparisons,
and "write" and "explain" for the pointers;
none of the 29 verbs is among the comparisons.

What it still does not read.
A verb outside the list ("the lines below that enumerate it") passes.
A quantifier followed by "of" passes,
as B949 states
("below each of them" and "the cases below all of which fail").
The TSDoc of `COMPARED_OBJECTS` and B938 say both.
Correction in place (2026-10-25,
round 12):
B952 replaces the list of 29 verbs (`VERBS_AFTER_THAT`) by six nouns and "of" (`NOUNS_AFTER_THAT`),
so a verb outside the list no longer passes;
the quantifier before "of" is the one limit that remains.

What it found.
Over the 4115 texts the guard reads
(the 4114 of B949 and the file of round 10),
the red commit has 263 raw findings.
With the scan changed and nothing reworded there are 271 raw findings and 2 unexempted,
the two pointers.
With both reworded there are 269 raw,
none unexempted and none stale.
Outside the guard's own files the raw findings are 80 before and 80 after,
the same keys.
The 6 more raw findings are in the guard's own files,
the new vocabulary text and the new case.
The pointer in the typography file now reads "the log messages of this function that write it".
The pointer in the ledger of 2026-10-05 now names the eight paragraphs by file,
with a note dated 2026-10-25 in place;
its claim is unchanged.

The titles.
The review named three titles that still called exact,
half,
both and some members of the compared list.
Two did:
the case of the verbs and the case of the floating quantifiers.
Each now says which of its words the list holds and which it no longer holds.
The titles of the thirteen words and of the clause opener read true and were not changed.

The two corrections in place that no code needed.
B944's known limit is closed by the result of the ninth re-check:
the three files pass salts 0 to 9 at the parent of `1dee3ca797`,
and fail exactly the sweep's salts at `1dee3ca797`.
The glance of round 9 said that main did not move and no merge was made;
main moved to `16c2c7ddb1` and the landing merged it in `1362f15c40`.

Cache versions.
`git log --extended-regexp -G '_CACHE_VERSION = [0-9]+' 765c036343..HEAD` lists nothing,
and the control over `938e84afb9..b37685721e` lists `6c81859c79` and `53e23a728c`,
so the query reads the declarations.
Only a test file,
the fixtures of a scan,
a TSDoc line and Markdown changed,
so no number needed to move.

Gates,
keyless and one at a time,
on the tree of `44e3b92e03`.
In source-scan,
`lint:types` and `lint:oxlint` (0 warnings and 0 errors) passed,
`test:unit` passed 68 suites,
and `source-scans` passed 53;
the package has no build task.
In typography,
`build`,
`lint:types` and `lint:oxlint` (0 warnings and 0 errors) passed,
and `test:unit` passed 43 suites.
The first `test:unit` of source-scan failed the Markdown lint on the first draft of the note in the ledger of 2026-10-05
(13 findings of `markdown/semantic-line-breaks`);
the note was rewrapped before the commit.

Slips of this round:
the fix commit `44e3b92e03` carries the title changes with the scan change,
since both are in one file and interactive staging is not available here;
its subject names the scan and the pointers and its body names the titles.

## B952: after a position, "that" compares only before a noun the scan knows (round 12)

Status:
fixed on 2026-10-25 (UTC),
red `f97d87462a0cc3f9f018b403932dfe6a97f5bcba`,
fix `c943fed7c68ddf82ee1521f1fa5540231f8b7591`,
pins `38288733722680f870da326b6f66e3ea59ee4ebe`,
reflow `955736c03bb23382b769518826465e440a12d31c`.

Minor finding 1 of the re-check of round 11 and the ruling on it.
B951 named 29 verbs in one form,
so a pointer with a singular subject,
a verb in the past or a verb outside the list still passed ("the line below that writes it",
"the paragraph above that explained",
"the lines below that enumerate it",
"the cases below that follow it",
"the lines below that are indented").
The re-check built the inverse reading in a scratch copy and measured it over the 4115 texts:
with no noun list it adds 8 findings outside the guard's own files,
all 8 comparisons and none a pointer,
and the words that follow "that" in them are floor,
speed,
size,
opening,
calibration and percentile (6 findings),
"and" and "is" (2 findings).
With those six nouns and "of" for "that of" it catches all seven probe shapes.
The extended verb list (87 words) would have caught 4 of the 7.
The ruling of 2026-10-25 adopts the inverse reading,
so the scan no longer depends on guessing a verb or its form.

What the scan reads now.
After a position,
"that" compares only when a word of `NOUNS_AFTER_THAT` follows it,
joined by spaces alone:
floor,
speed,
size,
opening,
calibration,
percentile,
and "of" for "that of".
Any other word,
a verb in any form,
a clause,
a punctuation mark or the end of the phrase,
reads the position as a pointer.
`VERBS_AFTER_THAT` and `COMPARED_NOT_BEFORE_BARE_VERB` no longer exist,
and "that" left `COMPARED_UNLESS_OBJECT_FOLLOWS` for `COMPARED_ONLY_BEFORE_KNOWN_NOUN`.
The vocabulary doc of `COMPARED_OBJECTS`,
the header of the test and the TSDoc of the reading state the rule.

What it still does not read.
A quantifier followed by "of" compares:
"below each of them" is left,
and so is "the cases below all of which fail".
That limit remains,
and it is the only one the vocabulary doc names for the words it reads with the word after them.
The cost of the new reading is that a comparison such as "a rate above that floor"
with a noun the list does not hold is found
until the noun joins `NOUNS_AFTER_THAT`
or the sentence is exempted with its reason;
the texts held 8 such sentences in 4115,
6 of them before a noun the list now holds and 2 that needed an exemption or a reword.

The two comparisons it found.
`page-check/src/corpus-run/archive-stub.ts` says that the pipeline translated the whole ORIGINAL "below that" and,
since no slice covered the marker,
published the stub over a finished translation;
the words point into the archive page,
a position in the page's own text,
and the sentence is exempted with that reason (`position-references-exemptions.test-fixture.ts`);
a reword would have put page-check in the gates for a comment.
`audit-ledger-short-bench-floors-and-the-waits-that-cross-them.md` said "a hold below that"
for a hold shorter than a thousand milliseconds,
which reads better without the position word and now says "a hold shorter than that".

The pins of the re-check (minor finding 2).
Each noun has a case of its own with its sentence written out in the test
(a loop over the exported list would drop a noun along with the list),
and one case pins the determiner or pronoun after a floating quantifier ("above all the others",
"below all its peers").
Mutants in scratch copies of the tree at `c943fed7c68ddf82ee1521f1fa5540231f8b7591` with the new test file:
the unmutated copy passes;
the seven noun drops each fail the case of that noun and the `opens` test removed fails the quantifier case;
the noun list emptied fails 10 cases and the `that` group emptied fails 4.

Minor finding 3 of the re-check.
B948 said that the window had applied "since" `b6927f7ba9`,
a docs commit of main at which the claim was verified.
`git log -S THINNEST_WINDOWED_BENCH` lists `b71a553855b05333cf985f3161409d9d98da2993` (2026-09-09,
"ask quorum plus one seat per round") as the commit that added the constant,
and the constant is 5 there (`MIN_SELECTION_WEIGHT` 2,
`SELF_VOTE_WEIGHT` 1/2,
`FANOUT_SPARE` 1).
B948 is corrected in place,
and B945 names the commit.

The nits of the re-check.
The relative-position wordings (this file said "the earlier rounds",
and B945 said that "the note closes this entry") name what they point at.
The note of 2026-10-25 in `audit-ledger-test-suite-unrun-code-reverification-of-2026-10-05.md`
opens with "Note" and not in capitals.
The two TSDoc lines of 149 and 144 columns in `consistency-cache-version.ts`
and `pairing-cache-version.ts` are rewrapped at 120
with no word changed.

Slips of round 11 that the re-check named and that stand,
since a commit is not amended:
- `44e3b92e03980859d5beca29a16e205a6388be9f` uses the scope `translation-repair-docs` in the third line of its body,
  where every other docs line of the range uses `translation-repair`.
- The bodies of `a5d22db40b0fef9dd0ce15585ae4ffa0697550df`
  and `44e3b92e03980859d5beca29a16e205a6388be9f` hold single lines of 190 characters and more.
- The body of `a5d22db40b` does not say that its red commit also added two lines to the case that passes at its parent.

Findings of the scan,
over the 4115 texts the guard reads,
raw meaning no exemption applied.
Head `e039edb013`:
269 raw,
none unexempted and none stale (189 in the guard's own files).
With the scan changed and nothing exempted or reworded:
283 raw (201 in the guard's own files) and 2 unexempted,
the two unexempted being exactly the two comparisons.
After the exemption and the reword:
283 raw (202 in the guard's own files),
none unexempted and none stale.
Outside the guard's own files the raw findings are 80,
then 82,
then 81,
and the keys added are the two comparisons (one of them reworded away afterwards).

Cache versions.
`git log --extended-regexp -G '_CACHE_VERSION = [0-9]+' e039edb013..HEAD` lists nothing,
and the control over `938e84afb9..b37685721e` lists `6c81859c79` and `53e23a728c`,
so the query reads the declarations.
Only a test file,
scan fixtures,
TSDoc lines and Markdown changed,
so no number needed to move.

Gates,
keyless and one at a time.
Source-scan:
`lint:types`,
`lint:oxlint` (0 warnings and 0 errors),
`test:unit` (68 suites passed) and `source-scans` (53 passed) on the tree with this text;
the package has no build task.
Consistency and slice-reading,
where the reflow changed a comment:
`build`,
`lint:types`,
`lint:oxlint` (0 and 0) and `test:unit` (198 and 59 suites passed).
