# Audit ledger: labels and counts in messages

Paths written `src/...` in this record name the layout before 2026-10-18,
when the package `package/module/translation-repair` was split into the packages under `package/translation-repair`.
`doc/audit/translation-repair-package-split-map-2026-10-18.json` gives each file's current path.

Messages that named the wrong thing or stated a count wrongly:
labels that did not fit,
counts before a fixed plural,
one field carrying two kinds of text,
and a log line that reported nothing done and gave no reason (B628).
The entry B628 was written on 2026-10-15 (UTC) from the hand-back of n161 (`s232/handback-n161.md`),
the lead's notes of that day and the header of `src/corpus-run/pass-consistency-group-line.ts` at `277145a862`.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B37: the reference-count floor told a model a side carried none of an atom it carried

Status:
fixed 2026-09-30 (UTC) with the sixth T8 batch,
found reading `translate-atom-rendering.ts` 341,
the arm writing "N times" for an atom carried too often,
which no test reached.
The floor compares the atoms (link destinations,
footnote markers and the like) a candidate carries with the ones the original and the page carry,
as multisets,
and its findings go back to the model that wrote the candidate.
It wrote one sentence per missing copy and one per surplus atom,
and each sentence said the other side carried none of it:
a candidate carrying a link once where the original carries it twice was told
"The ORIGINAL carries link-url … and your translation does not.",
and a candidate carrying a link three times where the original carries it once was told
"Your translation carries link-url … 2 times and the ORIGINAL does not."
A link missing twice drew the same sentence twice.
A model told the original lacks a link it carries can remove every copy,
and is then refused for dropping it.
Of the log files on this machine carrying the floor's sentences,
701 carry one missing-atom sentence twice on one line,
and none carries a surplus sentence with a count
(`t8b6-b37-logscan.mjs` in the audit's scratch folder).
Those files are unit-suite logs and the logger's per-process files,
which do not say by path whether a pass or a test wrote them,
so how many pass runs sent such a sentence to a model is not measured.

Fix:
the floor counts each side per atom and writes one sentence per atom,
naming both counts where both are non-zero
("The ORIGINAL carries link-url … 2 times and your translation carries it once.");
a side carrying none keeps the sentence it had,
with the count where it is more than one.
The rendering pools,
which already named both counts,
are unchanged.
`translate-atom-rendering.unit.test.ts` pins every arm by its whole sentence:
missing twice,
short by one,
invented twice,
carried three times against once,
and a pooled rendering carried twice.
Run against the build from before the change,
the two new cases failed,
one on two identical sentences and one on a pooled rendering's "does not".
The finding reaches the model in the revision round,
so the change joins the translate cache account before a launch.

## B53: a file that is not an artifact advised as one of another generation

Found by the ninth coverage batch (census-SfAXnI),
at the schema guard's label for a `malformed` row,
"not an artifact this build recognizes at all",
which no test ran.
The schema census classifies a settled file that is not JSON,
or JSON that is not a record,
as `malformed`,
and its TSDoc says why that answer stands apart:
"its remedy is the opposite one.
Another generation's artifact is a sound result to keep;
this is a file to investigate".
The guard grouped it with the foreign generations anyway,
so `SchemaGenerationError` told an operator the directory
"holds artifacts of another schema generation"
and closed with the ways forward for a sound artifact,
among them moving it to an archive where
"the archived copy stays readable as the generation it is".

Reach:
the pass never met it.
It runs `assertArtifactsPlaceable` first (`corpus-pass.ts`),
whose placement census files an unparseable file as malformed
and a JSON list,
which carries no `id`,
as untagged (`artifact-placement.ts`),
and refuses both as unplaceable.
A caller running the schema guard alone met the wrong advice.

Fixed as a quality call open to veto (`1556de391`):
the guard refuses `malformed` rows first,
with `UnplaceableArtifactError`,
whose message names each file and says removing it is the whole remedy;
the generation label and grouping take only artifacts,
typed so a malformed row cannot reach them.
The case writes a sound version 1 artifact beside a non-JSON file and a JSON list,
and reads the refusal naming the two files with that remedy,
and naming neither "another schema generation" nor the sound artifact.
Guard-off proof:
with the refusal's condition set to more than a million files and a rebuild,
`pass-schema-guard.unit.test.ts` failed on that case alone
(its FAIL line and its describe's rollup),
the message opening "This artifacts directory holds artifa…";
restored with Edit,
`git diff --stat` empty,
and rebuilt.

Recurrence:
where a classification's TSDoc says one answer takes a different remedy,
check every refusal built from it gives that answer its own.

## B55: a reorder note for definitions already in order

Found with B54,
casing the relabel's reorder note.
`reorderFootnoteDefinitions` (`archive-footnote-order.ts`)
noted that the definitions were interleaved with other blocks,
or shared container delimiters,
before testing whether they already followed the order asked for.
Definitions already in the original's order with prose among them came back with the note,
and the relabel reported "archive definitions stand",
a withheld move where none was needed;
since B54 that finding reaches the artifact.
The note's own TSDoc said it explains why the text stands
"when it does and the order asked for differs".
A probe over the built package (`~/temp/agent/audit-glossary-fix/b55-probe.mjs`,
cat-themed fixtures) gave the same finding for in-order and out-of-order definitions.

Fixed (`2824c6259` red,
`e2edf037a`):
the already-in-order return comes before the two refusals.
The text returned is the same on every path;
only the note moves.

Recurrence:
a note or finding that says work was withheld is emitted only where the work was wanted.

## B66: lane messages printed a count before a fixed plural

Found by the thirteenth coverage batch,
reading `lane-slice-coverage-error.ts` beside the cases its cold sentences lacked.
A repeat fault prints how many entries a list claims and how many distinct indices they hold,
and the distinct count is 1 whenever two entries share one index,
so the sentences read "under 1 distinct indices" and "over 1 distinct slices".
`lane-slice-sets.unit.test.ts` asserted "lane reports 2 unfilled slices under 1 distinct indices",
agreeing with the defect.
The contest driver's log of the claims shown to the judges read "1 corroborated damage claims" for one claim.
By inference from `refusalText`'s callers,
the coverage error's message reaches only log lines and stored records,
and the driver's line is the run log,
so no model was shown either wording.

Fixed (`da74173a1` red,
`65deb5167`):
each count in those sentences takes the word `wordForCount` (`count-word.ts`) chooses,
and the number of decisions reads through `howOften`.
The lane contest's finding "heard N usable ballots,
below the 2 needed to settle"
and the polish gate's twin of it are left for the family's own change,
since both are stored in artifacts and move together.
`65deb5167`'s message quotes that finding with a 3;
both quorums are 2,
and a comment on the commit says so (M84).
Taking the fix back out failed the three red cases,
each with its describe,
and nothing else.

Open:
the same shape stands elsewhere
(the slice cache's "discarding N cached slices",
whose test's header quotes "discarding 1 cached slices",
the census report's own "1 stretches,
1 lines",
and the lane contest's and polish gate's usable-ballot findings);
the family is classified and fixed as one change,
red first,
with the sites a model reads apart from those a person reads.

Recurrence:
a count in a message takes the word `wordForCount` chooses,
or `howOften` for a number of times,
and a case pins the count of 1.

## B90: an Anthropic refusal that named the OpenAI-compatible contract

Found in T8's twenty-second batch,
in retry lines the unit suite logs
(`completion body violated the OpenAI-compatible contract: anthropic stream ended without message_stop`),
red in `0b9447b81`,
fixed in the commit adding this entry.
`MalformedCompletionError` worded every refusal as a breach of the OpenAI-compatible contract,
and the Anthropic readers throw it too,
so a failed Charm Hyper call read as a broken OpenAI-shaped body.
The note at the head of `completion-shape.ts` records an earlier fix to the same message:
a live log named Synthetic for a call that went to Charm Hyper,
and that fix took the provider's name out
but kept the contract,
which that Hyper call did not speak either.
The constructor now takes the wire format its reader read the body as
(`StreamWireFormat`,
required),
and the message names that format's contract;
the Anthropic details drop their own "anthropic" prefix,
which the contract now carries.

Recurrence:
a shared error class names what its thrower knows,
taken from the thrower;
`mistake-prevention.md`,
"Labels a message names".

## B98: counts printed before a fixed plural

Found 2026-10-01 (UTC) by a census of every count printed before a noun in the package's source
(`plural-census-report.md` in the audit's scratch folder),
fixed in the commit adding this entry.
Report lines,
log lines,
refusals and findings wrote a count before a fixed plural,
so a count of one printed "1 slices";
the census read about 230 such sites across 58 nouns,
at least five of them in findings the translating model reads on a retry.
One had shipped and was pinned:
`corpus-run/editor-calibrate-standing.ts` printed "1 rounds",
and its test asserted the line as written.

Each noun after a count that can be one is now chosen by `wordForCount` (`count-word.ts`),
across 139 source files,
from a patch an agent prepared and the session applied,
its two conflicting hunks (an import each) added by hand.
The patch chose nouns and left the verbs that agree with them,
so twelve sites then read "1 entry ... are" or "1 reference go";
each now chooses its verb by the same count,
or reads without one.
The patch's seven-line calls put three files over the line cap
(`candidate-select.ts`,
`lane-contest-driver.ts`,
`corpus-run/slice-census.ts`);
their log lines now print each count as a label (`ballots: 3`),
which no count can make wrong,
and eleven calls whose count is a member chain put each step on its own line.

The count-noun scan written after the patch (`count-nouns.unit.test.ts`) found three more sites it had left:
the line-structure finding told the translating model a rendering repeated a line
"N times where the ORIGINAL repeats no line more than 1 times",
since the bound is 1 for every original that repeats no line,
and now says "more than once" through `howOften`,
with a case;
the window trial's "1 of 3 arms survive" chooses its verb;
and the target-unclaimed alignment finding counts its characters.
Two tests that pinned the old wording now read the new:
`verify-published.unit.test.ts` ("1 of 1 page carries every wording its artifact promised")
and `artifact-two-lane-verify.unit.test.ts` ("has 1 row for a preparation of").

Model-facing text changes only at a count of one,
in the translate stage's validation findings
(`translate-validate-blocks.ts`,
`bilingual-pair-bound.ts`,
`line-structure-guard.ts`);
the archive source context's reader count is two or more wherever it is printed,
so its text is unchanged.
No cache version moves:
`TRANSLATE_SLICE_CACHE_VERSION` 15 was set after the newest slice-cache record any run wrote
(the cache-account audit of 2026-10-01 UTC:
newest record 2026-09-27T04:26Z,
every version set after it),
so no answer cached under the old wording can be served.

What enforces it:
`src/count-nouns.unit.test.ts`,
among the source scans,
fails on a `String(...)` interpolation followed by a noun the package counts with `wordForCount` somewhere,
so its noun list grows as nouns are counted,
outside the eleven sites it names with why
(counts that cannot be one,
and verbs after an index such as "slice 3 claims"),
and fails when a named site no longer stands.
A word followed by `=` is a log field's name and is passed over.
Its fixture case runs first;
with `document-preparation.ts`'s character count written back as a fixed plural,
its package case failed,
and passed once the site was restored.
Out of its reach:
a noun after an adjective ("3 source chars"),
a noun split from its count across a `+` between literals,
and a noun no `wordForCount` call names yet.

Calls made here are open to veto:

- log lines in the three files over the line cap print counts as labels rather than sentences;
- verbs agree through a second `wordForCount` call rather than a rewording.

Recurrence:
`mistake-prevention.md`,
"Counts in printed text".

## B109: counts printed before a modifier and a fixed plural

Found 2026-10-01 (UTC) by reading what B98's scan could not see,
red in `07621943a`,
fixed in the commit adding this entry.
B98's scan read a count only where a counted plural followed it directly ("1 slices"),
so a count before one or two modifiers and then a fixed plural passed it:
"discarding 1 cached slices",
"1 usable ballots".
Widened to read up to two modifier words,
a proper adjective among them ("Latin tokens"),
the scan named 75 printed sites in the package's source
(72 distinct keys,
three printed twice),
in log lines,
command-line output,
error messages
and stored findings;
none of them is text a model reads.

Each site now chooses its noun with `wordForCount`,
and five keep a fixed plural as exemptions,
each a count that cannot be one,
with why.
A separate reviewer read the 82 changed call sites
and found two verbs and pronouns still fixed to the plural
(`spend-report.ts` "has",
`score-attribution.ts` "its" and "It is");
both now agree with their count.
The stream line `reportStreamProgress` writes also printed "unreadable frames" after any count,
which the scan could not name because no `wordForCount` call counted frames yet;
it now does.

The stream line's change had a reader.
`cap-census-read.ts` found a completed stream's content count by the text " content chars",
so a stream that delivered one content character,
logged "1 content char" after this fix,
would have read as no completed stream and left its `SPEND` line unpaired.
The reader now takes the unit up to its plural ending and accepts both spellings,
since lines logged before this fix wrote "chars" after every count,
and the cap-census cases build their stream lines through `reportStreamProgress` rather than copying its wording,
with a case reading a one-character stream in each spelling.
The guard-off meant to show that case failing against the old reader was refused by the session's permission classifier,
so that case is unmeasured against it.

Counting the recovery round's nouns took `stage-quorum.ts` to 312 lines of code against the linter's 300,
so the recovery round moved to `stage-recovery-round.ts`,
the one self-contained step of the gather,
with a unit test of its own;
its two log lines now carry the tag `[runRecoveryRound]`.
`message-names-only.unit.test.ts` names the four new `wordForCount` parts that marked error classes interpolate.

No cache version moves:
the stored findings among the changed sites differ only in a noun's number,
and no code reads those words back
(a search of the package's source outside tests for each changed finding's noun phrase finds only TSDoc,
comments and prompt prose).

Reach:
`count-nouns.unit.test.ts` passes with its 16 exemptions,
and the cases touching every changed message,
the source scans and lint pass at this commit.

Recurrence:
`mistake-prevention.md`,
"Counts in printed text" and "Readers of the package's own log lines".

## B125: a decision seat's distribution recorded as an objection

Red in `1faf181ae`.
Found reading the T8 bundle 2 change to the typed ballot's reason.
A typed decision seat answers the select ballot with a number and a distribution,
and `readTypedBallot` (`candidate-select-decision.ts`) writes
`typed decision: probabilities ..., confidence ...` as the ballot's `reason`.
`shipPreferredPastDecline` (`translate-runoff-tie.ts`) took the reason of every ballot
that did not back the shipped candidate as an objection,
so a decision seat's numbers entered `shippedPastDecline.objections`
beside the written judges' words.
The red case seats the three written judges and `SEAT_OPENROUTER_DECISIONS` on a slate
declined over wording that cannot ship,
and the objections read the rejection and the distribution.

No page was harmed,
measured as far as the stores reach:
only the consolidation slate's objections reach a refiner (`consolidate-settle-gate.ts`),
and that slate seats no decision seat
(a probe on the built package,
`t8-par/b125/seated.mjs` in the audit scratch,
printed `late judges include a decision seat: false`,
with the decision seat among the eight select judges);
the translate lane's objections have no reader.
Of the `.log` files under the agent scratch,
174 outside the audit's own scratch record a typed ballot,
and none outside unit-test runs records a decline shipped by preference;
no stored JSON but coverage maps carries `shippedPastDecline`,
and the 290 JSON files in the worktree's runs directory hold no typed reason.

The fix:
`shipPreferredPastDecline` leaves out the ballot of a decision seat,
named by `isDecisionSeat` on its model rather than by the reason's label,
which a written judge's words could begin with.
A `run-seats.unit.test.ts` case pins that the consolidation slate seats no decision seat
under any reading,
with the select benches seating it as the control;
adding the decision seats to `slateJudges` turned it and two older cases red,
and the seating was restored.

Recurrence:
`mistake-prevention.md`,
"One field carrying two kinds of text".

## B215: a missing check named no checker, no issue by id and no cause

Red in `1e444543d`,
fixed in `2b0bfad75`.

Found on 2026-10-05 (UTC)
by the refine-lane review,
in the finding B161 quotes.
`resolveResolutionChecks` (`tally-resolution.ts`) wrote one `missing-check (<n>)`
for each issue a checker's report left without a verdict,
numbering the issue by its place on the sheet.
Two checkers skipping one issue wrote the same line twice,
and an issue answered only with a verdict the tally does not know landed there too,
with nothing to tell it from one never mentioned.

The fix:
the resolver takes the checker's id and writes `missing-check (<issueId>, <checkerModelId>, unanswered)`
where the report held no check on the issue,
and the same with `unknown-verdict` where every check it held on the issue carried an unknown verdict;
`unknown-resolution-verdict (<verdict>)` still stands beside the second.
`runCheckerStage` passes each voice's id.
The findings for a repeated check and for a check numbered past the sheet are unchanged.
No reader takes the finding's text:
the recheck counts the reading's ballots,
and the troubleshooting doc on unread signals lists the finding by name alone.

Cases:
`tally-resolution.unit.test.ts` holds an issue never mentioned,
one answered only with an unknown verdict,
and one answered with an unknown verdict and then a known one,
which is answered;
`repair-edit-stages.unit.test.ts` holds two checkers skipping one issue,
whose two findings now differ,
and the cause beside the unknown verdict itself.
Five cases of `refine-recheck.unit.test.ts` carry the finding in their expectations and failed on the lead's red tree.
Their helper now puts the findings of several checkers in code-point order,
since the agent saw the order differ between two runs of one case.

Cache:
no version moved;
a cached record holds the old wording,
and nothing reads it.

Open to the owner's veto:
the issue is named by its id and the checker by its seat id,
where the other way prefixes every ballot finding with the checker at the stage and changes more text.

Recurrence:
`mistake-prevention.md`,
"Labels a message names":
a finding one of several voices leaves names the voice,
the item by its id and the cause,
so two findings about one item can be told apart.

## B217: a dropped destination's refusal wrote its counts as `destination(s)` and `slice 4, 7`

Fixed in `843bd6ce0`,
with its four cases;
the fixing agent's red run failed all four.

Found on 2026-10-05 (UTC),
as the item ledger B174 left open.
`DroppedDestinationError` (`corpus-run/destination-completeness.ts`),
a marked class,
wrote `destination(s)` after any count
and named several slices under the singular `slice`.

The fix:
`wordForCount` chooses the noun for the destination count and for the slices in `whereCarried`,
and the message inventory names both calls.
The four cases,
in `corpus-run/dropped-destination-error.unit.test.ts`,
hold one destination in one slice,
two in two,
one in none and two in one shared slice,
each by its whole sentence.
The file is named for the class:
one named for the module would have taken the module's line out of the list of modules without a test,
which stands in a file the agent was not given.

The count-nouns scan had passed the old wording;
why it did was not looked into.

Open:
`corpus-run/destination-completeness.ts` still stands in that list,
its cases under another file's name.
Still so at `94de8258e` (`own-unit-tests.unit.test.ts`),
with no follow-up launched for it.

Recurrence:
`mistake-prevention.md`,
"Counts in printed text".

## B255: a repeated-line finding for an original with no content line read "more than 0 times"

Red in `042a9245d`,
fixed in `51ccd8680`.

Found on 2026-10-06 (UTC),
closing the item B229 left open.
An original holding no content line got the finding "repeats no line more than 0 times",
where the count helper's own contract starts at one.

The fix:
the finding says the original has no line to repeat;
the wording for a line allowed once is unchanged.
One case in `line-structure-guard.unit.test.ts` holds both wordings whole.
On the lead's red tree that file failed with three failure lines,
its new case by an assertion on the finding's words.

Open to the owner's veto:
the "has no line to repeat" wording,
where the count helper throwing on zero is the alternative.

Recurrence:
`mistake-prevention.md`,
"Counts in printed text".

## B308: counts in the runners' printed lines stood before a fixed plural, verb or pronoun

Fixed across the runner moves,
each with a case the moving agent ran red in its worktree,
or a red case in the commit named:

- `spend-report`:
  `1 call reported no usage block, so their tokens` (`387af02b7`);
- `score-attribution` and `score-crosscheck`:
  `1 artifact could not be read and are in NEITHER population`
  (red in `a13f71ba8` for the attribution report,
  fixed in `1088e72d9`);
- `cache-account-audit`:
  `1 cache versions`,
  on the terminal and in its progress line (`53e25b458`);
- `coverage-probe`:
  `1 unpaired passages` (`da8a963cc`);
- `corpus-pass`:
  "1 minutes" on the line naming the ceiling a run ran under
  (red in `ea16d3b51`,
  fixed in `9a68e3c78`),
  and "1 more cache records" and "its 1 cache records are" on the `REATTEMPT` and `STALLED` lines (`9a68e3c78`).

Found on 2026-10-06 (UTC) by the agents moving the runners,
whose cases pin whole printed lines.
Until the moves no test ran these commands as built,
by the messages of the commits that added their as-built suites.

The fix:
each noun,
verb and pronoun follows its count through `wordForCount`.
With `passage` a counted noun of the coverage probe,
the `count-nouns` scan named the fixed plural of `sideClause` (`quote-preservation.ts`),
which had an arm for one and a fixed plural for the rest.
The agent proposed an exemption;
the lead had the helper take its noun from `wordForCount`,
which says the same text without one.
One exemption is added to that scan for a line of `corpus-pass` where "records" is a verb.

Not examined:
why the `count-nouns` scan had passed each noun.
By `mistake-prevention.md`,
"Counts in printed text",
it reads a noun the package already counts with `wordForCount`,
and verbs and pronouns are reviewed by hand.

Recurrence:
`mistake-prevention.md`,
"Counts in printed text".

## B309: counts before a fixed plural in four more runners, and a noun chosen by a difference's sign

Red in `a9d447691` for the fidelity probe's preflight line,
fixed in `8206fa5c6`;
the other lines were run red by the agents in their worktrees.

Found on 2026-10-06 (UTC) by the agents moving seven runners into cased modules,
whose cases pin whole printed lines:

- `verify-published`:
  a page one character shorter than its artifact implies printed "-1 characters",
  since the noun followed the signed difference;
- `coverage-census`:
  a suite that printed one failure marker was refused as "1 FAIL markers";
- `judge-fidelity-probe`:
  a request selecting one reference printed "1 reviewed reference specifications selected",
  and the suite's own helper pinned that plural for one;
- `rendering-audit-settled`:
  "BUYING 1 of 1 selectable subjects";
- `rendering-audit-settled-report`:
  "1 subjects".

The fix:
each noun follows its count through `wordForCount`,
and the verifier's noun follows the size of the difference,
not its sign,
while the signed number is still printed so the direction stays visible.
Cases pin each line at one and at many,
the verifier's at minus one and minus seven.

Not examined:
why the `count-nouns` scan passed these;
by the coverage census agent's account,
its noun was a template literal in a file the move changed.

Recurrence:
`mistake-prevention.md`,
"Counts in printed text";
B308 records the same family in six other commands.

## B375: `offPin` counted artifacts its name did not say

Red in `bb2a8b9ad1e164974e54d2750432a25b267ef0de`,
fixed in `cfac5b981`.

Found on 2026-10-06 (UTC) by the review of the runner observations change,
its ninth finding.
The DONE line's `offPin=`,
named for artifacts of entries off the pin,
also counted the artifact of a finished entry whose English page the pin lacks,
though the pin holds that entry.
By the agent's run on the port build,
a new case putting such an entry through the DONE line fails,
with the renamed expectations.

The fix:
the field is `unpairedArtifacts`,
and its TSDoc names both populations it counts.

Open to the owner's veto:
The DONE line's field is `unpairedArtifacts=`;
the alternative is the old name,
`offPin=`.

Recurrence:
`mistake-prevention.md`,
"Labels a message names".

## B376: the `--only` line said a finished entry runs

Red in `bb2a8b9ad1e164974e54d2750432a25b267ef0de`,
fixed in `cfac5b981`.

Found on 2026-10-06 (UTC) by the review of the runner observations change,
its tenth finding.
The line said the entries it names run,
where a named entry that has finished does not.
By the agent's run on the port build,
a new case naming one finished entry beside one pending fails,
with four other expectations of the line.

The fix:
the verb comes from `wordForCount`,
"if still pending it runs" for one entry and "those still pending run" for more,
and the TSDoc says why.

Open to the owner's veto:
The `--only` line's verb reads "if still pending it runs" or "those still pending run";
the report names no alternative.

Recurrence:
`mistake-prevention.md`,
"Command lines".

## B390: the runs lock warned that a claim's staged text was left when no claim had written it

Red in `5ba14c371fdc57996fb2aba3fdffb32758f62b33`,
fixed in `f5ea0872bf9f4e8d551b92c64aeba925cf1933e2`.

Found on 2026-10-10 (UTC) by the agent working the runner and git stretches of the final census (T8),
in a catch the census listed as cold in `corpus-run/runs-lock.ts`,
lines 214 to 217.
`stagedRemoval` removes the name a claim stages its lock text under,
and when that removal failed it warned
`<staged> is left after a claim and could not be removed (<code>)`.
A runs directory whose lock file fits within `PATH_MAX` while the staged name does not
refuses the claim's write with `ENAMETOOLONG`,
and the removal of the same name fails the same way,
so the warning named a file left behind that was never written.
The agent's probe (`crowded-probe.mjs` in its scratch folder),
run against the built package over a runs directory built to 4,060 units,
printed the old warning,
with `ENAMETOOLONG`,
for that unwritten file.

The fix:
the warning reads
`<staged> could not be removed after a claim (<code>); it stands beside the lock if the claim wrote it`,
and the TSDoc of `stagedRemoval` says why.
The case is
"SAYS THE CLAIM'S STAGED TEXT STANDS ONLY IF THE CLAIM WROTE IT,
where removing it fails:
a runs directory whose path leaves no room for the staged name refuses the claim with the system's own error,
and the removal fails the same way",
over a real runs directory of that length.
Against the unchanged code,
`runs-lock.unit.test.ts` alone fails,
with 3 FAIL lines,
by assertion on the warning line,
and the other nine named test files pass.
Run on the merged tree before the fix commit:
lint clean,
49 scans passing,
and the suite at 2,033 passing with no failure,
one more than before for the suite the runs lock cases added.

Open to the owner's veto:
the wording;
the alternative tells a failed write from a failed removal and says the text is left only for the second.

Recurrence:
`mistake-prevention.md`,
"Claims without their evidence".

## B628: a consistency group line where nothing was carried named no reason

Found on 2026-10-15 by the lead,
in the read of `TianqiChen66624`.

Outcome written on 2026-10-15.
Red `5fe3173cc5`,
fixed in `277145a862`;
the fix is the round n161.

What a reader of the run would have seen:
the settled `TianqiChen66624` run logged a line that began
`group 1 of 2: refrain verdict=asked placed=4 unplaced=0`
and went on with the columns
`replaced=0 consistent=0 kept-context=0 kept-floor=0 overlapped=0 unlocated=0`,
all of them zero.
The bench was asked and nothing changed,
and the line said nothing of why,
because the outcomes `chosen`,
`kept-current` and `untouched` had no column,
and the reasons that the record holds were not printed.

The lead ruled on 2026-10-15:
the group line names why nothing was carried.

Fixed by n161.
The line is now `<repeat.kind> verdict=<kind> placed=N unplaced=N`
followed by the nine columns
`replaced= consistent= kept-context= kept-floor= overlapped= unlocated= chosen= kept-current= untouched=`,
so the nine outcomes add up to the places placed,
and,
when a place ended `kept-current` or the bench was not asked,
failed or named no winner,
it ends with `because=` and each distinct reason,
comma separated,
in the order the places stand.
A group already consistent names no reason.
Every reason is the code's own decision,
read from the record and not decided again by the line.
The code is the new `src/corpus-run/pass-consistency-group-line.ts`,
which `src/corpus-run/pass-consistency.ts` imports
(its old local `groupLine` and `placesEndingIn` are removed),
exported through `src/consistency-apply-barrel.ts`.
The reasons,
as the header of that file lists them:
- `not-asked:too-few-placed` and `not-asked:no-english`:
  the bench was not asked (`src/corpus-run/pass-consistency-unasked.ts`).
- `failed:<detail>`:
  trouble stopped the group before the ballots could be counted,
  and the detail names no corpus text.
- `no-winner:<reason>`:
  the bench was asked and no place won,
  with the `NoWinnerReason` `no-quorum`,
  `no-majority`,
  `chosen-unlocated`,
  `run-off-no-quorum` or `run-off-tied`,
  and every place stays as it stood.
- `carry:<reason>`:
  a place was to receive the chosen rendering and the carry left its English as it stood,
  for `no-alternative` (no text but the current one to choose among),
  `no-quorum` or `no-majority` (the choice reached no decision),
  `failed` (trouble stopped the carry),
  `chose-current` (the choice kept the English as it stands),
  `splice-cannot-reach-place`
  (the choice was the code splice and the place has no words that code can move the chosen words into),
  `chose-writer-text` (a writer's text was chosen and no replacement of this place was made),
  or `unrecorded` (the record holds no carry for the place).
- `kept:<reason>`:
  the pass left the place as it stood after the splice was chosen,
  for one of the `gain-` or `edge-` reasons of `ConsistencyKeptReason`
  (`gain-pairs`,
  `gain-unproves`,
  `gain-floors`,
  `gain-overlaps`,
  `gain-refused`,
  `edge-second-mark`,
  `edge-reading-changes` and `edge-reference-mark`),
  named ahead of the carry's own decision.

Test:
`src/corpus-run/pass-consistency-group-line.unit.test.ts` (new)
drives every path with a hand-built record and reads each line.
It failed with "groupLine is not a function" on the build before the fix,
then with 28 rows once the function existed without the `because=` clause,
which is the guard run.

Recurrence:
`mistake-prevention-messages-labels-and-counts.md`,
"Log lines that report nothing done".

## B629: a consistency group line named the edge refusal where the bench had kept the text

Found on 2026-10-15 by the review of n162 (`s233/review.md`,
finding Z1),
in the line that B628 added.

Outcome written on 2026-10-18.
Red `555bb74aa0`,
fixed in `29d6064708`;
the fix is the round n166.

What a reader of the log would have seen:
a group with a receiving place that the splice cannot reach (`plan.kind === 'unavailable'`) and an edge verdict on it,
and a carry whose writers were still asked and whose bench chose the current text,
ended with `because=kept:edge-second-mark`.
The line said the splice was refused at an edge,
and never said the bench had kept the text.
A reader concluded that nothing else was tried.
The probe row (`s233/probes/q2-groupline.ts`,
row C) printed `kept-current=1 untouched=0 because=kept:edge-second-mark`
for a record whose carry decision was `chosen` of a `current` candidate.

The cause:
`src/corpus-run/pass-consistency-settle.ts` records the edge reason for every receiving place that has an edge verdict,
whatever the carry did (lines 295 to 303 at `277145a862`),
`src/corpus-run/pass-consistency-apply-record.ts` writes it as the reason of any place that ended `kept-current`
(lines 567 to 585),
and `keptReasonOf` in `src/corpus-run/pass-consistency-group-line.ts` printed it ahead of the carry (lines 171 to 186).
The header of that file said a `kept:` reason is for a place left after the splice was chosen.
A block with no spliceable place is left to the writers or to the current block,
so no splice had been offered there.

Reach:
the log only.
The corpus reach could not be measured without a run,
since the replays check page text and not log lines,
and the group line of the settled `TianqiChen66624` run was not derived again under the new code.

The lead ruled on 2026-10-15,
on the work of n166,
that where the carry offered a splice and the bench chose the current text,
the line prints the recorded kept reason,
which is the rule as it was written;
the carry's own decision is then not shown for that place.

Fixed by n166.
`keptReasonOf` prints `kept:<reason>` only when the carry for the place has a candidate of kind `splice`,
and otherwise prints what the carry decided (`carry:...`),
or `carry:unrecorded` when the record holds no carry for the place.
`groupLine` takes a required one-based `groupNumber`
(`src/corpus-run/pass-consistency.ts` passes `position + 1`),
so that the refusals of B630 can name the group,
and the header of the file now says what each prefix means.

Test:
`src/corpus-run/pass-consistency-group-line.unit.test.ts` has 14 new tests
that were red against the build the round started from
(11 for the reasons of this entry,
3 for B630),
for example `expected 'kept:edge-second-mark' to equal 'carry:chose-current'`,
and one control that was green.
The same 14 are red again with the `kept` condition no longer needing a splice
(`s237/mut-a.log`).

What the round ended with:
the whole unit suite at 2,445 passes and 0 failures on its second whole run
(the first was red on seven tests that pinned cache keys or the order of seats,
which the sheet text of B637 moved),
lint,
types and the source scans clean,
both read-only replays as expected,
and no decision or character change on any of the 92 pages of the corpus,
with every positive control moving.
The rule breaches of the round are M231.
The round landed on 2026-10-18 with the rounds n168,
n169 and n171,
in `555bb74aa0` and `29d6064708`;
the combined tree passed 2,474 cases.

Open for the reviewer:
an unknown verdict kind still throws a bare `TypeError` in the probe's row I,
unchanged from the start of the round.

Recurrence:
`mistake-prevention-messages-labels-and-counts.md`,
"Log lines that report nothing done".

## B733: the log said a changed slice shipped unchanged

Status:
fixed on 2026-10-21 (UTC),
red `f68c4964de`,
refactor `ed053bb36c`,
fix `1ab73ab56d`.
Found in the read of `zheermao21` (finding 8,
class B).
No cache version moved.

What the code did:
`zheermao21` slice 3 closed its introduced-defect probe at 1 of 3 voices,
and the log said
"a stage heard fewer than quorum (stage-quorum-unmet (introduced-defect-probe 1/3)),
so the slice ships unchanged and is NOT cached".
The slice did not ship unchanged:
the repair result lists it in `changedSliceIndices`,
chunk 3 has `changed true` and `accuracyPatchSelected true`,
and its repaired text reached the page.
Only the caching was withheld.
The sentence in `repair-slice-buy.ts` was written for every refusal reason,
and a refusal changes what is stored,
never what is shipped.

The fix:
`cacheRefusalLine` in `repair-cache-gate.ts` words the line from the refusal reasons,
the slice index and whether the slice changed:
"chunk N: <reasons>,
so this settlement is NOT cached and a later run buys the slice again;
the slice's text changed|did not change in this run".
The red cases asserted the line holds neither `ships unchanged` nor a claim of a change the slice did not make,
and failed with `expected ... to not include 'ships unchanged'`.
A refactor commit gave the warn line a name first,
and is a separate commit on purpose.

Cache account:
none moves.
The line is log text,
and the settlement a refusal leaves uncached is the same one that was always left uncached.

## B806: the run-off line called a plurality a tie

Status:
fixed on 2026-10-22 (UTC),
red `e13dc11c610e9f5c9c8de1dc72fc5847ae6f03dd`,
fix `53040005b4ad4472c2658c4460954cce7d6a87ac`,
no cache version moved.
Found in the read of `zheermao22` (wording defect of slice 12) and fixed in `w263`.

What the log said:
slice 12 of `zheermao101` at 22:19:09 UTC:
"translate stage:
declined-indecision;
the standing drew no ballot,
so the same panel is asked again over the 2 tied
finalists".
The round before it named candidate 4 at 1.5 (weights 1 and 0.5),
candidates 5 and 3 at 1 each,
and said "winner drew only weight 1.5 ... against a minimum of 2.00".
Nothing was tied at the top.
`runoffFinalists` had chosen the leader and its runner-up,
and `runoffTiedFinalists` received the two candidates and no more.

The fix:
the `narrowed` run-off carries `basis`,
`tied-at-top` or `leader-and-runner-up`,
and `TranslateStageResult` carries it as `runoffBasis` beside `runoffFinalists`,
`TranslateAbsenceError` as `basis` beside `finalists`.
`runoffTiedFinalists` takes it as a required parameter,
and `runoffFinalistsPhrase` words it:
"the 2 tied finalists" for a tie,
"the leader and its runner-up (the leader fell short of the minimum weight)" for a plurality.
The consolidation's single-round door throws `unreachable:` if finalists arrive without the basis.
The two lines of the retry's loop that said "run-off tied again" now say "run-off left no winner",
because a round that leaves no winner is a tie or a plurality.
`translate-runoff-line.unit.test.ts` builds both shapes through the real judging and reads the line of each,
with controls that read the weights drawn;
it failed with `expected undefined to equal 'tied-at-top'` and `'leader-and-runner-up'`.
The count-noun and message scans of `source-scan` pass unchanged.

Not changed:
`runoffFinding` ("translate-runoff (finalists N of M)"),
which is scorecard-stable wording.
Candidates 5 and 3 drew the same weight and ballots in slice 12,
and the run-off kept 3 for its earlier slate position;
that rule is the existing one (`runoffFinalists`).

Recurrence:
`mistake-prevention-messages-labels-and-counts.md`,
"Labels a message names".
