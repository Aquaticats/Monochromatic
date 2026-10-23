# Preventing mistakes: sheets, quorums and outcome rules

Rules for what each model is shown,
which seats a quorum counts,
and how a stage reaches and records its outcome.

Part of [Preventing this package's mistakes](mistake-prevention.md),
which lists every file of this record.

## Which seats a quorum counts

What happened:
the short-bench rule moved every gather's quorum onto the seats a wet provider serves,
and the archive block review's outage test moved with it,
but the same function's second threshold,
the anchored voices a review needs,
stayed at the whole bench's quorum (ledger B27).
A short bench whose every reachable seat anchored its quote was left unresolved
and skipped its naturalness read.
Two sentences kept saying a short bench cannot settle,
which the rule had made false.
And two short-bench cases refused two seats of four,
which leaves a quorum,
since the quorum is half the bench rounded up;
each failed at its own premise before reaching the stage (M86).
And B80's first case expected one refusal from every roster seat,
where the gather asks a window of quorum plus one,
so it would have failed with the fix in place.

A refinement recheck that heard no checker,
or one of three,
passed the rewrite,
since it rolled back only on a tally saying `worse`
and dropped the checker stage's own `stage-quorum-unmet` finding with the rest (ledger B141).
A gate that passes on nobody objecting first reads,
through the stage's own account,
that enough voices answered,
and carries that stage's findings with its verdict.

The same day the damage probe shipped a rewrite after hearing none or one of three probers (ledger B160),
and the recheck passed an issue that two of three heard checkers skipped,
counting its one ballot against no quorum (ledger B161).
A per-issue threshold on a gather's ballots takes the quorum the gather closed on,
and positive evidence is read before silence wherever a rollback carries no report.

The recheck then took that quorum by rebuilding the gather's short-bench finding
for every count of refused seats and matching its wording,
since nothing else carried the number (ledger B214).
A gather returns the quorum it closed on,
and a reader takes that field,
never a number read back out of a finding or sized a second time.

The damage sample recorded a region no prober answered for as silent,
which is the human grade's negative reference (ledger B298).
No answer is never written down as an answer:
an unheard region is printed as unheard and left off the sheet.

The rule:
when a rule changes what a quorum counts,
census every threshold derived from a gather,
not only the gather:
every `rosterQuorumSize` call,
every "half the bench",
every count compared with a bench size,
and every sentence saying what a short bench can do.
A threshold on voices a gather produced takes the `reachableQuorum` the gather closed on,
computed once and shared by every test in the function;
only a size taken before any seat is lost
(the first round's window)
or the definition of short itself counts the whole bench.
A threshold is a floor a count reaches,
so a case holds the count exactly at it,
or a boundary mutant survives.
A short-bench fixture is sized from `rosterQuorumSize` read in the code:
a bench is short only when fewer seats are reachable than half the bench rounded up.
A case counting what a gather's voices did counts the seats its scripted client was asked,
recorded as each is asked,
never the roster.

What enforces it:
`archive-block-review-stage.unit.test.ts` holds a bench with most seats refused,
asserting from the exported helpers that the case discriminates,
and a count exactly at the quorum;
the gather,
the windowed rounds,
coverage,
the select minimum and the naturalness review each hold a refused seat
(`stage-quorum.unit.test.ts`,
`stage-windowed-rounds.unit.test.ts`,
`coverage-stage-reachable.unit.test.ts`,
`candidate-select.unit.test.ts`,
`absolute-naturalness-review-stage.unit.test.ts`).
`stage-quorum.unit.test.ts` also holds the quorum the gather returns,
on a bench every seat of which answers and on the same bench with most seats refused,
and `repair-edit-stages.unit.test.ts` the same number as the checker stage hands it on.

A failure is scoped to the narrowest thing it names (P18):
a refused key to its provider,
a model's refusal to that model on that provider,
a request's size to that request.
The seat tally once counted a provider's HTTP 401 as failed asks of the seat
and withheld two Gemma seats another wet provider served,
so the consolidation judged on three or four voices of seven.
A test of a scoped failure gives the seat a second provider that serves it
and asserts the seat is heard from there.
`credential-hold.unit.test.ts` and `provider-router.unit.test.ts` hold the case.

A round's straggler window follows what its replies are (ledger B713).
A reply that is a whole candidate (an editor's patch,
a translation,
a corrected block)
is a writer voice,
and the window that cuts it takes a candidate from the judges;
a reply that reads text which exists is one ballot among several.
The archive block review asked for whole corrected blocks and ran under the readers' window
because no decision had classified it,
and zheermao20's letter block lost a Qwen reply to it.
A new gather says in its call which kind it is,
and joins `WRITER_STAGE_LABELS` if it is a writer round.
Its case runs a straggler under windows that differ (`archive-block-review-windows.unit.test.ts`),
since `SCRIPTED_STAGE_SETTINGS` makes the two equal and so cannot tell which one a stage read.

A cut or abandoned call is classified by its cause and by what it delivered (ledger B761).
The round cuts a straggler by aborting its signal,
and a stream that carried only framing (0 content characters and 0 reasoning characters,
as 44 `inception/mercury-2.5` streams did in two logs) was counted in no column,
so a seat whose every ask ended that way was never withheld.
A cut counts as a failed ask only when the round made it (the abort reason names the round's cut)
and the call delivered no content and no reasoning;
a cut that had delivered reasoning is a working stream and stays out of the count.
The threshold for withholding is chosen on seats that later answered:
a run of 9 silent cuts was followed by an answer in CuspariaKLSY22,
so any threshold on consecutive silent cuts below 10 would have withheld a seat that was serving.
A seat is withheld when every ask failed,
at 4 asks or more.
`seat-tally-silent.unit.test.ts` holds the cases,
including a reasoner cut with reasoning delivered,
a caller's own stop,
and a seat that answered once among many silent asks.
A new wrapper that classifies an abort reads the abort reason and the delivered counts,
and not the fact that the signal aborted.

A bench or roster change is tested where the bench is read (ledger M287).
Taking `hf:Qwen/Qwen3.8-27B` off the late-judge bench ran the run package's tests only,
and the corpus-pass file `pass-seams-reseat.unit.test.ts` threw at load,
because its fixture guard found that a Synthetic dry-out no longer takes a late judge.
A change to a bench or a roster
(a hold on a model card,
a roster list,
a seat set that `judgeSeatsFor` builds)
runs the unit tests of every package that reads `judgeSeatsFor` or the bench constants,
and not only the package it edits:
`rg --files-with-matches '<NAME>' package/translation-repair --glob '*.ts'`
for `judgeSeatsFor` and each constant that moved lists them,
and on 2026-10-22 it found corpus-pass,
probe,
repair,
run,
stage and translate-stage.
A hold on a card also runs the provider package's tests,
whose `roster-seats.unit.test.ts` holds every seat's holds against its card,
and which names neither of those
(ledger M287,
the fourth miss).
A case that guards its own fixture reads the views it needs from a measurement of the new bench,
not from the view the old bench made distinct:
the 31 views with a provider dry were counted,
and 4 take a late judge.

## Sheets every model reads

What happened:
the rendered-sheets fixture said it rendered every model-facing sheet and held fifteen,
so the Canadian spelling guard never read the picture readers' "summarise" (X17).

The rule:
a new sheet is rendered by the rendered-sheets fixtures in the commit that adds it,
and sheet text is built by a named function or constant,
never inline where it is sent.
A sheet asked again of the same bench in a second round has text of its own,
since a client that answers a repeated model and prompt from its first payload replays the first round
and the second round is a copy of it (ledger B657).

What enforces it:
`rendered-sheets-census.unit.test.ts` fails when a `build…Messages` function goes unrendered;
sheets built without that naming need their own entry by hand.

## What each sheet is shown

What happened:
two sheets listed the apparatus kinds without the bound that says what apparatus never is;
the archive block review and its slate,
coverage,
the page-title lexicon and the refine slates carried house rules that point to a DECLARED NAMES block,
and never got one,
though each caller held the names (ledger B28).
The lexicon ran before preparation,
which built the identity lines inline where nothing else could read them.
The refiner was told a handle survives exactly,
and the judges choosing among its rewrites were not.
The fixture rendered 13 of its 38 sheets naming the block without it,
so no guard reading the rendered sheets could tell a sheet that lacks the block from one the fixture left bare,
and the fence each new block needed and the follow-up coverage sheet went untested until a mutation batch.
The rendering audit,
which reads settled renderings after a run,
was shown the declared names and never the cited references the critics and panels had read (ledger B29),
and the rule it needed could not be the critics' copy:
that one names a TRANSLATION the audit never shows and a category its wire rejects.

The relabel probe's arm that claims no issue list is known sent the production disclosure with an empty list,
which under a rendered disclosure writes a list's heading into the prompt (ledger B283).
An arm that claims a thing is absent sends the form production sends when it is absent.

The archive block review's selection sheet listed the reviewers' assessments by the text each reviewer proposed,
so the composite of disjoint revisions,
which no reviewer proposed,
reached the judges with none (ledger B716).
A candidate no one proposed carries the assessments of those whose lines it is made of,
copied as given and marked with the lines each covers;
the judges are never shown a verdict nobody gave,
and never a candidate they cannot check beside ones they can.

The rule:
a sheet whose rules name a block carries that block wherever its caller holds it.
Adding a block to a sheet threads it from where production holds it
(`pass-prepare.ts`,
`pass-insertion-admission.ts`,
the stage)
in the same change,
and a case drives the caller and reads the model call.
What a writer is shown,
the judges of its text are shown,
and so is any instrument that audits the text afterwards:
the names,
the references and the bounds.
A rule shown with a block names the texts and the categories of the sheet it sits on.
A block a sheet encloses goes into the texts its fence is chosen against,
and every branch of a sheet
(a first ask,
a follow-up)
carries the same blocks.
Context read by more than one sheet is assembled once and read by each
(`page-identity-lines.ts`,
`declared-names-evidence.ts`,
`translate-slate-evidence.ts`),
never rebuilt inline beside a caller.
The rendered-sheets fixtures give each sheet what production passes it,
through the functions production calls where there are any.

What enforces it:
`rendered-sheets-context.unit.test.ts` fails on a rendered sheet that names the DECLARED NAMES block or CITED REFERENCES without carrying the fixture's,
unless it is exempt by name with its reason,
and on an exemption gone stale;
the caller cases
(`pass-prepare-archive-review-context`,
`pass-insertion-admission-identity`,
`pass-prepare-lexicon-context`,
`refine-identity-threading`)
fail when a caller stops threading the names;
`sheet-fence.test-fixture.ts` lets a case read the fence a sheet opened a block with.
A sheet whose rules name no block,
and a caller whose fixture passes a block production does not,
are outside what these can see.

The judge that chooses between a producer's text and its rewording is shown what the repair panel accepted about the slice.
The dispute note named only additions and accuracy claims settled at major or worse,
so two accuracy claims accepted 4 to 0 at minor severity on `CuspariaKLSY21` slice 0 reached no sheet after the repair lane,
and the polish gate chose a second softening of the hunger line on a sheet that showed neither claim (ledger B731).
`acceptedRepairClaimsNote` states every accepted claim at the severity the panel settled,
without a severity filter,
as claims to check against the original,
and leaves out the claims the dispute note already names.
`polish-gate-page-rules.unit.test.ts` and `consolidation-polish-apply.unit.test.ts` read the sheet the gate is sent,
and the driver case in `corpus-pass` reads the block from a run that has the claim and from one that has none.
What they cannot see is whether a judge then decides differently,
which needs a rerun measured against run-to-run spread.
A new judge that chooses between texts takes the claims in the same way,
and a note that filters claims by severity is read for the claims it hides.

The rule runs both ways.
B731 gave the claims to the polish gate and not to the polish writers,
so a writer rewrote the base without the evidence the gate then judged the rewording against (ledger B805).
A block added to a judge's sheet is threaded,
in the same change,
to the writer of the text that judge reads,
through the same builder,
and the rendered-sheets fixture renders the writer's sheet with it.
`rendered-sheets-context.unit.test.ts` names the sheets that must carry the accepted claims and the dispute note
(the polish gate,
the polish refiner and the two sheets of the judges of the writers' rewrites),
because a scan that reads only sheets naming a heading cannot see a sheet that lost the block.
The first version of this rule left two gaps,
which B805 closed before landing:
the dispute note did not reach the polish writers,
and neither note reached the judges of the writers' rewording.
A judge whose criteria say that a candidate must say exactly what the text says is also given the sentence that lets it accept a correction the note asks for,
or the note is shown to a judge who must vote against what it permits.

## A rule a sheet states about tense and what it does not cover

What happened:
the house rule on tense listed what the person said among the life told in the past,
and named no exception for words quoted as she wrote them.
The last message of `TianqiChen666`,
written in her own present,
shipped in the past in two passes,
and the read of the first pass said the tense held,
because it read the narration only (ledger B627,
M220).
The review of n162 then found two sentences that the exception sat beside and did not touch:
the house bullet still listed "what the person did,
said,
made,
felt" among the life told in the past,
and the judge tail said there is no rule holding the life in the present (ledger B637).
The advisory tense candidates also read a quotation set in single curly marks (ledger B636).

The rule:
a sheet rule that moves text into another form names what it does not cover.
For tense,
words quoted as someone wrote or said them keep the speaker's tense,
and keeping their present is never a tense inconsistency.
Every sheet that states the tense rule carries that exception:
the full rule where the house policy is shown (`QUOTED_WORDS_RULE`),
and the one-sentence reminder beside every instruction to move a line of the life to the past,
in the house block's words or in the sheet's own (`QUOTED_WORDS_REMINDER`).
Both are built by named constants in `english-usage-policy.ts`,
never inline where they are sent.
A new sheet with a tense instruction carries the reminder in the commit that adds it.
Each sentence that lists what is told in the past,
or says no rule holds a life in the present,
carries the exception in that sentence and not only in the paragraph that holds it (ledger B637).
A reader of quotation marks in the tense candidates blanks the single curly quotation
as it blanks the double (ledger B636).
A sheet that carries no tense rule gets none,
as the pairing sheets,
the reference attestation,
the picture readings and the derivability sheets do;
a tense rule added to one carries the exception.

What enforces it:
`sheet-quoted-tense.unit.test.ts` renders every sheet of the rendered-sheets fixture and fails on
a sheet that carries the house rule without the rule sentences,
an instruction to move a line of the life to the past with neither the reminder nor the rule beside it,
a polish gate text without the reminder twice,
and the list of house corrections of form without "(never for quoted words)".
Each marker must be found in at least one sheet,
so the checks are not vacuous.
A tense instruction written in words that no marker of the test holds is outside what it can see,
and its marker is added to the test by hand.

## Ballots give their reason first

What happened:
the owner ruled on 2026-09-27 that an issue-panel verdict gives its reason before its vote,
and the contest,
select,
naturalness,
coverage and archive review ballots kept the choice first,
so a decoder held to the schema could write the vote before it had reasoned.
The owner extended the ruling on 2026-10-21
("Reason first everywhere";
ledger B732).

The rule:
a ballot or verdict schema that carries a reason declares it and requires it before the choice,
and the instruction of its sheet names the reason before the choice.
A field that is the reason under another name counts (`finding` of the archive block review).
A schema whose reason is empty on some verdicts,
or whose explanation follows the text it explains,
is listed with its reason and left as it is.
Reading is by property name,
so the order of a reply is free for the readers.

What enforces it:
`reason-before-vote.unit.test.ts` in `lane-contest`,
`consolidate`,
`adjudication` and `archive`,
over `reason-order.test-fixture.ts`,
which fails when a declared or required list does not lead with the reason,
and when the rendered instruction names the choice first.
A new ballot schema that no test names,
and a reply cut short by the completion cap,
are outside what these can see.

A ruling about the order of keys in a reply depends on the provider keeping that order (ledger B762).
Charm Hyper returns a tool call's keys alphabetically (200 of 227 multi-key objects in zheermao22),
and the repo sorts nothing,
so a reason declared first reaches the decoder first on three providers and last on one wherever `reason` sorts after the choice.
A key-order ruling is checked against a provider that reorders keys before it is called done,
and a schema is declared in the order its keys sort unless the ruling says otherwise.
`schema-property-order.unit.test.ts` in `source-scan` scans every `properties` literal in source for a declared order
that is not alphabetical,
and is green with no allow-list.
The reason of a ballot is named `assessment`,
so that it sorts before the choice;
a reason that a ruling puts last is named `<vote>Reason`.
A schema the scan cannot read,
one built by spread or `Object.fromEntries`,
gets a pin test of its own
(`contest-ballot-wire.unit.test.ts` and `rendering-audit-wire.unit.test.ts`).
Only the reply carries the sorted names;
the stored shape keeps its spelling,
and a function maps one to the other.

## Ties a gate settles

What happened:
the polish gate settled a tie on `neither` and shipped the base,
and the log carried the tally and the word `neither` and not that the vote had tied (ledger B730).

The rule:
a vote between two names says what a tie does,
in the owner's words,
and the outcome names the rule that decided,
so a lead,
a tie and an unsettled vote read differently in the log and the findings.
A tie needs a vote on each side,
and the minimum ballot count applies to the usable ballots in total.
The gate outcome keeps its shape:
the artifact reader requires `ships` to be `polished` exactly when `choice` is `polished`.

What enforces it:
`consolidation-polish-gate-stage.unit.test.ts` and `consolidation-polish-gate.unit.test.ts` read the choice,
the rule and the log lines for base,
base,
polished,
polished;
base,
polished;
and a refusal beside a vote.

## Exits an outcome rule reaches

What happened:
the owner's rule to keep wording that cannot ship and ship the slice reached the judge's decline exit,
and not its empty-slate exit beside it,
so a slice whose archive wording the floor refused and whose translators proposed nothing usable stopped the entry (ledger B39).

The rule:
a rule that changes what an outcome does (ship,
stop,
keep) lists every exit that produces that outcome:
each throw of the error class that carries it and each caller that rethrows it,
and states for each whether the rule applies and why.
Each exit it applies to has a case.

What enforces it:
habit and review;
an exit the rule missed can show in the coverage census as a rethrow no test reaches,
as this one did.

## Refusals inside a composed operation

What happened:
the footnote relabel renames the archive's labels and then moves its definitions into the original's order.
The move threw when the moved definitions would parse differently,
and the pass reads every rewrite refusal as keeping the archive unchanged,
so a sound rename was kept back with it,
under a message blaming the rename;
the move's other refusals (prose among the definitions,
a container across them) leave the definitions standing with a note and the rename applied.
A bounded probe over generated pages reached the throw 156 times in 2,000 (T8's eighteenth batch).
A fix for a span checked against the first of two nodes sharing an id made the anchor validator throw,
where every other defect it finds comes back as a rejection,
and a caller's gate turns any rejection into a named refusal of the claim;
the throw bypassed both,
and the whole suite on `9ff566830` went red on a caller's case (ledger B129).

The refine stage read its resolver's operations and not the findings returned beside them,
so a rewrite naming a paragraph the sheet never showed left no trace (ledger B142);
and an OCR reader that exited cleanly and wrote no transcript made the reading reject with `ENOENT`,
which the reading pair does not contain (ledger B148).
A caller of a function that returns operations beside findings reads both,
or says why it drops the findings.

A rewrite the atom gate refused reached an info line alone,
so the stage read as a refiner that proposed nothing (ledger B162).
A refusal is a finding on the stage that refused,
worded without the values it compared.

A refusal added to the walk builder,
for a pairing naming a block its section lacks,
came out of the artifact rebuild as a throw,
where the rebuild answers every other carve that departs from the run's as `moved`.
A republish then said only the class name,
and two instruments that walk every settled artifact would have stopped on one (ledger B170).
A refusal added inside a step is followed to every caller of the operation the step sits in,
and where that operation already answers a misfit as data,
the new refusal becomes that answer.

A reading pair's contract said every reader failure but an abort is contained,
and its deterministic reader was awaited with no catch,
so a full scratch device ended the entry (ledger B184).
The slice census converted a recorded pairing itself
and stopped on the refusal the rebuild answers as `moved` (ledger B196).
A contract that says a failure is contained is read against every await inside it,
and an instrument that walks every artifact answers a misfit as the operation it mirrors does.

The refine stage applied each rewriter's operations and read only what the patch applied,
so a rewrite the patch refused as unchanged left no finding (ledger B216),
the third time this stage read one part of a result and not the part beside it (ledger B142,
B162).
And the grouping returned its runs without a translation block no run could take,
saying nothing,
so the page was refused a layer up as a block that reached no slice (ledger B219).
A caller of a function that returns what it applied beside what it refused reads both,
or throws on the refusals its construction rules out;
and a step that cannot place something refuses it there,
by name.

A refusal added so that a dropped translation block would be loud left its whole page refused,
where the archive's own bytes could be kept:
the block is carried as unplaced and named in a finding,
and the refusal class is gone (ledger B247).
A block still held at a seal crossed it and was refused as a layout fault;
it becomes unplaced at the seal (ledger B248).
A step that cannot place a part keeps the part as it stands where that is sound,
and refuses the whole only where nothing can be kept.
One voice's reply nested thousands of markers deep exhausted the parser's stack inside the translate stage,
and the engine's own error left the stage and took every other voice's candidate with it (ledger B301).
Text one voice wrote is bounded before a parser that descends once per level reads it,
and every reader of that parser has its refusal.
The bench draw reports the original page's failure ahead of the English page's,
and no case mixed a read failure with a parse failure until review asked for one (ledger B353);
its pair step's skip line stays without a case,
since a bounded probe over 6,000 generated pairs reached it in none of the 5,773 that passed the step.
A precedence between a step's refusals has a case for each mix of them.

The composite the archive block review builds from disjoint revisions matched each revision to the block line for line,
and so built nothing when any revision added or removed a line (ledger B712).
In `zheermao22` it was tried three times and built nothing three times,
the email and the letter on a line count (ledger B803).
A refusal on a count mismatch is a narrowness where the mismatch is an ordinary edit:
a reviewer who drops or adds a line has not written a malformed revision.
The composite now reads each revision as the hunks a line diff makes to the block,
and refuses only where hunks collide.
Position pairing is for lists parallel by construction
("Pairing by position" in the doc on comparing text and reading structure).

The rule:
a step that can refuse on input a page can carry refuses as its sibling refusals do,
leaving the steps that held applied,
and a throw stays only for a broken invariant.
A module whose contract returns its refusals as data
refuses a new case as data too,
under a kind of its own;
a fix that turns such a refusal into a throw changes the contract,
and runs the whole suite,
every caller's cases with it,
before it is committed.
Before a step's throw is kept as unreachable,
a bounded probe over generated input counts what reaches it,
beside a count of what passed the step,
so a null is read from a probe that exercised the path.
A composition of edits to one text aligns each edit to the text by a diff and refuses where edits collide,
never where their lengths differ.
The diff is bounded by a stated cell limit and total beyond it (coarser hunks,
not a refusal),
and its tie rule is stated,
so identical lines attribute an edit the same way every time.

What enforces it:
the reorder and pass relabel cases on the unmovable definition;
`archive-composite-revision.unit.test.ts` (added lines,
removed lines,
both in one revision,
collisions at a line and at a gap,
list markers,
quotes,
blank lines and a note's markers)
and `archive-composite-hunks.unit.test.ts` (the diff held against a plain table over generated lines,
the bound and the tie rule);
habit and review for the probe,
whose counts the batch's ledger record states.
A search of the package's source for refusals on differing counts found the pairings of lists that are parallel by
construction (page headings,
signatures,
list items),
and no other comparison of a reviewer's edit with its base.

## Checks on one part of a composed text

What happened:
the archive block review read each revision's footnotes against the archive as it came,
while the pass splices every revision it accepts into one page,
so two blocks each dropping one marker of a note passed alone
and together shipped the note with nothing referencing it (ledger B80).
The check's own comment called a footnote a relation between blocks,
and the check still read one block's change at a time.
The attestation looked for a quote in a whole reference line,
the package's own head and failure notes included,
so a quote of a page's address verified as if the page had said it (ledger B104).

The rule:
a check guarding a relation between parts of a text
(a marker and its note,
a label and what it names,
a link and its anchor)
reads each part's change in the text the changes already accepted leave,
never in the input the composition started from,
and the composition applies its changes in an order that keeps every later part's place.
Its case pairs two changes each harmless alone and harmful together,
and asserts both premises before the composed refusal.
A check that looks for a source's words in text the package composed
reads only the part that carries those words
(`referenceLinePageText` for a reference line),
never the heads,
addresses or notes the package wrote around them,
and every reader of that composition goes through the same cut.

A text the package composes from parts that each passed a gate is a new text,
and goes through the same gates whole (ledger B712).
The archive block review offers the lines of disjoint revisions together as one candidate,
and `archive-composite-revision.unit.test.ts` pairs two revisions that each drop one marker of a note,
which leave it referenced alone and unreferenced together,
beside two that reword lines holding a marker,
and a pair that each blank a paragraph the other keeps.

What enforces it:
`corpus-run/archive-block-repair.unit.test.ts` holds the two-marker case,
and `reference-attest-verdict.unit.test.ts` the address and failure-note quotes;
habit and review elsewhere,
since no scan tells a relation check from a check on one part.

## Checks that vouch for a whole block

What happened:
the archive review's apparatus check accepted a block when any line declared contributors,
or when the block opened with a label,
a picture or a comment and ended with a closed one,
so prose beside the apparatus would have passed as apparatus with it (ledger B81).
The pinned archives held no such block,
which a census showed before the check was tightened,
so the looseness had decided nothing yet.

A report printed the first 400 units of a candidate and no word that it was cut,
so a reader took the opening for the whole (ledger B279).
An excerpt shown as evidence says that it is one,
and how much of the whole it shows.
A corpus read's failure text was cut at its cap and the kind and remedy appended with no mark,
so a cut path read as whole (ledger B373):
a cut followed by more text ends in a mark inside its cap.

The rule:
a check that vouches for a whole block's kind
reads every part a reader sees,
never one part,
the block's first characters
or its edges.
It reads the block as the page shows it:
comments blanked and a comment left open refused,
invisible lines skipped by `rendersAsNothing`,
and markup read by the parser rather than by a leading character.
Its case pairs each accepted shape with the same shape beside prose,
and a census over the pinned archives counts what it accepts before and after a change,
by block id.

What enforces it:
the stage test's apparatus case holds each shape beside prose and alone;
habit and review elsewhere.

## Records of more than one round

What happened:
the judges' retry kept the findings of every ask it made,
and the translate stage's follow-up round one level up returned its own record alone,
so a slice settled at depth two could not say its first slate was declined (ledger B41).

The rule:
code that runs a stage again and returns one result carries every earlier round's findings into it,
earliest first,
with a finding naming each new round and why it was asked,
on the returning path and the raising path alike.
Its test compares the whole record around that finding.

What enforces it:
habit and review;
the stage case and the B39 case fail if the depth-two layer drops a round again.

A caller that receives a stage's outcome records the findings it carries (ledger B714).
`repairArchiveBlocks` read the review stage's text and kind and never its findings,
so a block that lost voices to the window recorded only that it was reviewed,
the third time a caller dropped a stage's findings after B142 and B162.
Its case mutes seats so the stage loses voices,
and reads the returned findings and the log for them
(`corpus-run/archive-block-repair.unit.test.ts`).

## Work no floor can check

What happened:
where the floor could compare nothing
(an original no grammar reads),
the translate stage still bought translators and judges,
kept the archive off the slate on the floor's `unknown`,
and let a candidate stand on the same `unknown`,
so an admitted insertion wrote a rendering nobody could check into the page,
and on a slice the archive translates every call bought nothing
(ledger B43).
The consolidation asked its writers there too,
and the floor refused all they wrote
(ledger B45).

Commands bought work before a check they could have made for nothing:
the verify probe asked the whole roster and then refused a sheet already taken (ledger B282);
the width probe bought its control for a draw holding no slice (ledger B299);
a calibration drew the corpus for a roster of one seat (ledger B305);
and a setup check printed that the plan was in order over a corpus yielding no entry (ledger B306).
The settled rendering audit named its runs directory only after buying every subject,
so a directory it could not name cost the whole roster's calls (ledger B312).
State known at the start is checked before anything is drawn,
gathered or bought.

The rule:
a stage that buys work a deterministic check must pass asks first whether the check can pass anything on that input,
through the definition the check itself reads
(`floorReach`,
`translate-floor-ground.ts`),
and settles without buying where it cannot:
on the archive where there is one,
unfilled where there is none.
Past that question,
a verdict the caller has ruled out is read through a narrowing that raises
(`requireComparedVerdict`),
never through a branch that keeps a candidate on it.

What enforces it:
habit and review;
`translate-stage.unit.test.ts`,
`translate-document.unit.test.ts` and `consolidate-driver.unit.test.ts` count the calls on a blind slice,
and `translate-floor-ground.unit.test.ts` pins the reach and the floor to one reading.

## A refusal that ends the run

What happened:
the router's every-provider-dry refusal says it ends the run,
and for the whole of the run of XingZ6018 it was lost as one seat's voice,
in 990 lines of its log,
until the bench floor stopped the entry 77 s after the first mark (ledger B818).
The fix was tested with the pass guard in every call path,
so four catches that swallowed the refusal stayed green until a review read them (ledger M289).
The seat reader met the dry state before any chat call and stopped the entry as an error that every pending entry was charged for,
and the stop was announced only when another attempt was pending (ledger B818).
The decision seat has a branch for the refusal that the decisions route never raises (ledger B824),
and the re-ask elsewhere has a catch the refusal cannot reach (ledger B825).
The words "a run always ships" were read to cover the entry in flight (ledger B821),
and the refusal was raised for one model's providers and ended the whole run (ledger B822).
The case that pinned the seat reader's reading depends on which roster it reads (ledger B823).

The rule:
a layer that turns a failed call into a lost voice asks `endsTheRun` first and rethrows what it names,
and the pass guard is the cover for the layers that have no check and never the reason to leave one out.
A test of "this error ends the run" builds its client without the guard,
asserts that the error reaches the caller unchanged,
and adds a positive control:
the same case with an error that does not end the run is still contained.
A test that overrides the function it claims about (`settle`,
`decide`) shows the stub.
A branch written for an error is shown reachable from the function that raises it,
and a test that throws the class itself shows the branch and not the route;
where the error cannot arrive,
a comment says why and cites the router path.
A state that ends the run is handed to the run by whichever layer meets it first,
the seat reading included,
and an entry stopped that way is neither re-attempted nor charged an attempt.
The stop is logged where it trips and announced once,
also when the entry that met it was the last.
A rule that says a run ships names what it covers for work in flight:
the entry the stop interrupts stays `INCOMPLETE` and stopped,
with its cache,
because the archive path settles an entry and discards its cache.
A refusal that ends the run is raised for the run,
not for a model:
a model whose providers are all dry is a lost seat,
and the case that asserts a dryness reading pins the roster it is relative to.

What enforces it:
`endsTheRun` and `isDryRefusal` (`stage-run-end.ts`);
the cases without the guard in `stage-round.unit.test.ts` (grace window),
`overlapped-map.unit.test.ts`,
`image-reading-contain.unit.test.ts` and `image-reading-pair.unit.test.ts`;
`stage-decision-call.unit.test.ts` and `stage-call.unit.test.ts`;
`entry-error-outcome.unit.test.ts` and the `settleEntry` case of `pass-entry.unit.test.ts`;
`pass-dry-stop.unit.test.ts`,
`corpus-pass-queue.unit.test.ts` and `corpus-pass-run.unit.test.ts`;
and `run-seats-refusal.unit.test.ts` for the seat reading.
The ruling on the entry in flight and the ruling on model-level dryness are in
`design-commitments-rulings-of-2026-10-22.md`;
each is open to the owner's veto.

## A test fleet models every provider shape of the live roster

What happened:
the seat reader waited out a hold on any provider while a bench stood short.
The tests that pinned the wait used fleets with one wet provider and one held provider,
where the held provider was always one that cured the shortfall,
so a hold on a provider that cannot cure it
(DeepSeek,
whose one wide seat is a seat the wet providers already reach,
or Xiaomi,
which serves no wide seat)
was waited out in full and the bench was seated short all the same.
The second review of the Claude route found it with a fleet whose Claude key read dry
and the wide bench stood one seat short of its quorum;
the review of the seat bound found the same defect from the other side
(a hold on a provider that serves no seat of a short bench).

The rule:
a test fleet models every provider shape of the live roster,
not the shapes the case happens to need:
a provider that cures a short bench,
one that serves seats of it and cannot cure it,
one that serves none of its seats,
a dry key that is a meter reading and holds `0`,
and two or more holds that each cure the bench alone
(a necessity test,
"the bench stays short without this provider",
counts neither of them,
so the reader seated the bench short with two curing holds running;
round 5 found it).
The shapes come from the roster through `reachOf`,
and a case that asserts a wait asserts the control (a hold that must be waited for)
beside the hold that must not be.
A new provider joins the fleet of these cases when its route lands.

What enforces it:
habit and review;
`run-seats-cure.unit.test.ts` pins the shape on this tree's roster
(NeuralWatt and Bedrock wet,
the wide bench 3 of 7 against a quorum of 4,
with Synthetic and Hyper held,
Synthetic and OpenRouter held,
and,
with NeuralWatt wet,
Synthetic,
Bedrock and Hyper held),
and `run-seats-hold-need.unit.test.ts` pins it for the translate lane.
The ruling is the lead's and is open to the owner's veto.
