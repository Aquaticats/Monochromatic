# Audit ledger: the polish gate's claims and the order of reason and vote

Sheets that withheld a claim the judges needed,
and ballots whose reply shape asked for the choice before the reason.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B731: the polish gate chose a rewording over an accepted repair claim it was never shown

Status:
fixed on 2026-10-21 (UTC),
red `b09097e067`,
fix `5419332cef`,
cache versions `4b9fc9e645`,
source scans `2e5c2f82c1`.
Found in the read of `CuspariaKLSY21` and `zheermao21`
(finding 2,
class A1 narrowed).

What the code did:
the repair panel's accepted claims reach a later sheet only through `archiveDisputeNote`,
and `disputesArchive` admits an addition at any severity but another accuracy claim only at a settled severity of major or worse.
A claim accepted at minor severity therefore reached no sheet after the repair lane.
Stored evidence of `CuspariaKLSY21` slice 0 (the hunger line):
chunk 0 holds one adjudicated issue,
status `accepted`,
severity `minor`,
with two `accuracy/mistranslation` claims,
each supported 4 and unsupported 0,
and listed in `resolvedIssueIds`.
The polish gate of that slice settled polished 3 and base 1 between "could not have a good meal" (the translate wording)
and "couldn't get a proper meal",
on a sheet that carried no claim and no dispute note,
so the accepted claim that "good meal" softens the source was not before the judges.
The answer to the question the brief put,
whether the slice had an accepted repair claim the gate sheet did not show,
is yes.

The fix:
`accepted-repair-claims.ts` in `archive` reads every accepted claim of a chunk's issues at the severity the panel settled
(`acceptedRepairClaimsOf`),
leaves out the claims the slice's dispute note already carries (`acceptedRepairClaimNotesOf`),
and writes a block headed `REPAIR PANEL CLAIMS ACCEPTED`
that says the claims are claims to check against the original and not facts
(`acceptedRepairClaimsNote`).
`repairClaimEvidenceOf` returns the disputes and the notes read off the same chunks.
`ConsolidationSubject` and `ConsolidationPolishGateSubject` carry `acceptedRepairClaimsNote`,
`buildConsolidationPolishGateMessages` renders it after the dispute block and counts it in the choice of the block fence,
and the corpus pass threads it from the repair lane's chunks to the polish round.
`consolidateSliceKey` appends `repair-claims` and the note only when a slice has one,
so a slice without accepted claims keeps its key.
The red cases failed with a fence of length 0 and a sheet without the block.

Decisions taken in this session:
- No severity filter.
  The block lists every accepted claim at the severity the panel settled;
  a filter at minor would have repeated the reading that hid the claim.
- The producers and the slate do not see it.
  The producers wrote against the claims,
  and the judges' own reasons in slice 0 cite the hunger line,
  so the evidence file found no measured slate that lacked it.
  Only the polish gate,
  the one judge that chose between a producer's text and its rewording with no claim in view,
  reads it.
- A claim the dispute note already carries is not named twice.

Open to the owner's veto:
showing the producers and the slate the same block,
which changes more sheets and every cache version they feed.

Not measured:
whether the judges then decide differently.
A red case shows the evidence reaches the sheet;
it cannot show the models choose otherwise,
which needs a rerun measured against run-to-run spread.

Cache account:
`CONSOLIDATE_CACHE_VERSION` 28 to 29,
with the tie rule of B730 and the reason order of B732.
The key of a slice with accepted claims differs by itself;
a slice without them was settled by the old tie rule,
which is why the number moves.

## B732: the contest, select and review ballots asked for the choice before the reason

Status:
fixed on 2026-10-21 (UTC),
red `8202d858a6`,
fix `e5bc27f9f0`,
cache versions `4b9fc9e645`,
source scans `2e5c2f82c1`.
Owner ruling of 2026-10-21,
"Reason first everywhere",
extending the ruling of 2026-09-27,
"Reason before vote",
from the panel verdict to the other ballots and verdicts
(`design-commitments-rulings-of-2026-10-21.md`).

What the code did:
only the panel verdict and the two consistency schemas declared the reason first.
Every other ballot declared its choice first,
so a decoder held to the schema could write the vote before it had reasoned.

What changed,
in the order the reply is written:
- the contest ballot (`lane-contest/src/contest-ballot-wire.ts`):
  `reason`,
  `unsupported`,
  `dropped`,
  `choice`,
  then `archive` where the contest wants one.
  Three sheets share it:
  the lane contest (`lane-contest-wire.ts`),
  the consolidation gate (`consolidate-gate-wire.ts`)
  and the polish gate (`consolidation-polish-gate-wire.ts`).
- the select ballot (`adjudication/src/candidate-select-wire.ts`):
  `reason`,
  then `best`.
- the absolute naturalness review (`archive/src/absolute-naturalness-review-wire.ts`):
  `reason`,
  `findings`,
  then `acceptable`.
- the coverage judge (`archive/src/coverage-wire.ts`):
  `reason`,
  `quote`,
  then `coverage`.
- the archive block review (`archive/src/archive-block-review-wire.ts`):
  `finding`,
  which is its reason,
  then `disposition`,
  `sourceQuote` and `replacementText`.

Left as they were,
each for a stated reason:
- the introduced-defect wire:
  its `reason` is empty on a verdict that finds no defect,
  which a reason-first order would contradict,
  and the probe is in shadow mode.
- the rewrite wire (`translate-repair-wire`):
  its `explanation` follows the translation it explains.
- the rendering audit:
  a verdict,
  then findings that carry a reason each.
- the resolution,
  restoration judge,
  derivability and reference attestation wires:
  they carry no reason of the ballot kind.
- the typed decision select (`candidate-select-decision.ts`):
  it carries no reason.
Already reason first:
the panel ballot (`adjudicate-wire`) and both consistency schemas.

Parser order:
every reader destructures by property name,
so the order of fields in a reply is read-independent;
`provider/src/schema-prompt.ts` restates the schema in the system prompt,
so the new order reaches the model there as well.
The tests that pin the order are `reason-before-vote.unit.test.ts` in `lane-contest`,
`consolidate`,
`adjudication` and `archive`,
all over `reason-order.test-fixture.ts`:
each asks that the schema declares and requires the reason first and that the rendered instruction names it first.
The red cases failed on the schema order and on the instruction text.
Four tests that matched the old instruction text through `GATE_MARKER`
and two schema pins in `repair` follow the new text.

Decisions taken in this session:
- Analysis fields keep their relative order between the reason and the choice,
  so the lists of unsupported and dropped content are written after the reason and before the vote.
  The alternative,
  lists first and then reason and vote,
  follows the claim,
  reason,
  vote order of the panel;
  both satisfy the ruling,
  and the choice was made to keep the ruling's wording literal (the reason is first).
- The archive block review's `finding` counts as its reason.

Open to the owner's veto:
the order of the analysis fields between the reason and the choice,
and the list of wires left as they were.

Risks,
not measured:
a reason first can move votes,
as the ruling of 2026-09-27 noted,
and no bench measures it;
a reply cut short by the completion cap now loses its choice and keeps its reason,
where it used to lose its reason and keep its choice.

Cache account:
the number moves for every version whose sheets carry one of these ballots.
`CONSOLIDATE_CACHE_VERSION` 29 (gate sheets,
naturalness review,
coverage sheet),
`LANE_CONTEST_CACHE_VERSION` 14,
`TRANSLATE_SLICE_CACHE_VERSION` 23 and `SLICE_CACHE_VERSION` 41 (select ballot and coverage sheet),
`REFINE_CACHE_VERSION` 12 (select ballot of the objection correction).
The consistency,
page title and pairing versions stay:
the consistency schemas were already reason first,
and the other two sheets do not carry these ballots.
The archive block review has no version of its own.

Note of 2026-10-22 (UTC):
the key names in this entry are the names of the day.
B762 renamed several of them in the reply,
and its entry lists the current names.

Consequence found in the same change:
five cases of `archive-composite-revision.unit.test.ts` failed on the new sheet text,
and are recorded as B734.

## B762: sixteen answer schemas declare their properties in an order Hyper does not keep

Status:
fixed on 2026-10-22 (UTC),
guard written red on 2026-10-21 (UTC) `d5562122314a`,
fix `a40de882b5`.

Found while the order of reason and vote (B732) was being checked against the replies of the two logs.
Charm Hyper returns the keys of a tool call in alphabetical order:
200 of 227 multi-key objects in zheermao22,
and 102 of 103 vote replies in CuspariaKLSY22
(a reply whose keys were already in alphabetical order cannot show it).
NeuralWatt,
OpenRouter and Synthetic follow the declared order.
So on Hyper the ruling of 2026-10-21,
"Reason first everywhere",
holds only where `reason` sorts before the choice,
and in most ballots it does not.

Does the repo sort:
no.
`canonicalPromptValue` digests the messages for the uniqueness check and never reaches a request body,
`buildAnthropicBody` and `renderToolSystemPrompt` place the schema as declared,
and no other path reorders a schema or a reply.
Whether Hyper sorts the schema it sends the model or the reply the model wrote is not known from the stored replies,
and no probe of it was made.
The only order that holds on every provider is one where declared order and alphabetical order agree.

The guard:
`source-scan/src/schema-property-order.unit.test.ts` reads every object literal that is the value of a `properties` key
in non-test source,
and names the file,
the line,
the declared order and the alphabetical order.
Its fixture case passes first and shows each form:
bare and quoted keys,
a sorted member,
a one-key member,
a spread,
a computed key,
an object that merely holds keys and a test file.
The package-wide case was red with sixteen findings and is green now.
The task `source-scan:source-scans` lists the file.

The sixteen schemas,
as they were declared and the order they have now:
- `adjudication/adjudicate-wire.ts:234`:
  `verdicts`,
  `groups` must read `groups`,
  `verdicts`.
- `adjudication/adjudicate-wire.ts:247`:
  `claim`,
  `reason`,
  `vote`,
  `severity` must read `claim`,
  `reason`,
  `severity`,
  `vote`.
- `adjudication/candidate-select-wire.ts:362`:
  `reason`,
  `best`.
- `adjudication/critic-wire.ts:158`:
  `summary` before `sourceQuote`.
- `adjudication/introduced-defect-wire.ts:741`:
  `region`,
  `verdict`,
  `category`,
  `severity`,
  `evidence`,
  `omittedText`,
  `reason`.
- `archive/absolute-naturalness-review-wire.ts:120`:
  `reason`,
  `findings`,
  `acceptable`.
- `archive/archive-block-review-wire.ts:314`:
  `finding`,
  `disposition`,
  `sourceQuote`,
  `replacementText`.
- `archive/coverage-wire.ts:406`:
  `reason`,
  `quote`,
  `coverage`.
- `artifact/page-title-lexicon-wire.ts:286`:
  `title`,
  `rendering`.
- `consistency/consistency-choice.ts:80`:
  `reason`,
  `choice`.
- `consistency/consistency-wire.ts:387`:
  `reason`,
  `quotes`,
  `choice`,
  `keep`.
- `lane-contest/contest-ballot-wire.ts:138`:
  `reason`,
  `unsupported`,
  `dropped`,
  `choice`.
- `rendering-audit/rendering-audit-wire.ts:341`:
  `verdict`,
  `findings`.
- `repair/edit-wire.ts:124`:
  `region`,
  `newText`.
- `repair/refine-wire.ts:130`:
  `paragraph`,
  `newText`.
- `translate-floor/translate-repair-wire.ts:220`:
  `resolution`,
  `translation`,
  `explanation`.

What changed:
the declared order of each of the sixteen is now its alphabetical order,
and the reply instruction of each sheet that lists the keys lists them in that order.
Every reader destructures by property name,
so a reordered reply changed nothing in a reader.

Reordered only,
nothing renamed,
because no ruling fixes their order:
- the panel verdict (`adjudication/src/adjudicate-wire.ts`):
  `groups` then `verdicts`,
  and each verdict reads `claim`,
  `reason`,
  `severity`,
  `vote`.
- the critic (`adjudication/src/critic-wire.ts`):
  `sourceQuote` before `summary`.
- the page title lexicon (`artifact/src/page-title-lexicon-wire.ts`):
  `rendering` then `title`.
- the rendering audit (`rendering-audit/src/rendering-audit-wire.ts`):
  `findings` then `verdict`,
  and the six fields of a finding in alphabetical order
  (`candidateFocus`,
  `candidateLocator`,
  `category`,
  `reason`,
  `sourceFocus`,
  `sourceLocator`).
  The scan cannot read the finding schema,
  which is built from a list,
  so `rendering-audit-wire.unit.test.ts` pins it.
- the editor's rewrite (`repair/src/edit-wire.ts`):
  `newText` then `region`.
- the refinement rewrite (`repair/src/refine-wire.ts`):
  `newText` then `paragraph`.

Renamed,
because a ruling fixes the order and the old name sorted elsewhere
(old name to new name,
and why):
- the select ballot (`adjudication/src/candidate-select-wire.ts`):
  `reason` to `assessment`.
  `reason` sorts after `best`,
  `assessment` before it.
- the absolute naturalness review (`archive/src/absolute-naturalness-review-wire.ts`):
  `reason` to `assessment`,
  and `acceptable` to `isAcceptable`.
  `acceptable` sorts before `assessment`,
  so the choice is named as the question it answers and now sorts after `findings`.
- the archive block review (`archive/src/archive-block-review-wire.ts`):
  `finding` to `assessment`,
  with `replacementText` before `sourceQuote`.
  `finding` was its reason and sorts after `disposition`.
- the coverage judge (`archive/src/coverage-wire.ts`):
  `reason` to `assessment`,
  and `quote` to `copiedSpan`.
  `quote` sorts before `reason`,
  and `copiedSpan` is the span copied from the translation,
  which sorts between the reason and `coverage`.
- the choice-only consistency ballot (`consistency/src/consistency-choice.ts`):
  `reason` to `assessment`.
- the consistency ballot (`consistency/src/consistency-wire.ts`):
  `reason` to `assessment`,
  and `quotes` to `candidateQuotes`.
  `quotes` sorts after `choice`,
  and the quotes are the analysis the choice rests on.
- the contest ballot (`lane-contest/src/contest-ballot-wire.ts`):
  `reason` to `assessment`,
  `choice` to `verdict`,
  and `archive` to `verdictOnArchive`,
  with `dropped` before `unsupported`.
  `choice` sorts before the two lists and `archive` before the reason;
  `verdictOnArchive` sorts after `verdict`,
  so the archive answer stays behind the lane answer.
  The schema spreads the archive member in,
  which the scan skips,
  so `contest-ballot-wire.unit.test.ts` pins its place.
- the introduced-defect wire (`adjudication/src/introduced-defect-wire.ts`):
  `reason` to `verdictReason`,
  and the rest in alphabetical order
  (`category`,
  `evidence`,
  `omittedText`,
  `region`,
  `severity`,
  `verdict`).
  The ruling of B732 keeps this reason last,
  and `verdictReason` sorts after `verdict`.
- the rewrite wire (`translate-floor/src/translate-repair-wire.ts`):
  `explanation` to `translatorNote`.
  The ruling of B732 keeps the explanation after the translation,
  and `translatorNote` sorts after `translation`,
  where `explanation` sorted before `resolution`.

Names chosen,
and why:
- `assessment` is the reason wherever a ballot gives its reason first.
  It sorts before every other key of those ballots,
  and one name serves the select ballot,
  the naturalness review,
  the archive block review,
  the coverage judge,
  both consistency ballots and the contest ballot.
  The panel verdict keeps `reason`,
  which already sorts where the ruling wants it,
  because a stored record spells it.
- `verdict` is the vote of the contest ballot and `verdictOnArchive` its archive answer.
  `verdictReason` is the reason that comes last in the introduced-defect wire.
  The three share a stem,
  so a reader sees which answer a key belongs to.
- `isAcceptable` is a yes or no and reads as the question.
  `candidateQuotes` and `copiedSpan` say whose words the field holds.
  `translatorNote` says who writes the explanation and where it goes.

Where the names live:
only the reply is renamed.
The wire types,
their guards,
the response formats,
the reply instructions and the one function per reply that maps it to the internal shape carry the new names.
The internal shapes,
the cache files,
the stored ballots,
the artifacts and the ledgers keep the old spelling,
so a record written before reads as it did and no reading under two spellings (as B107 did) is needed.
The mapping functions are
`readConsistencyBallotWire` and `readConsistencyChoiceWire` (`consistency`),
the readers in `lane-contest-wire.ts`,
`consolidate-gate-wire.ts` and `consolidation-polish-gate-wire.ts`,
the introduced-defect screen,
`candidate-select-count.ts`,
`coverage-verdict.ts`,
`archive-block-selection-evidence.ts`,
the naturalness review stage and `translate-repair.ts`.
A reader of a log meets both spellings:
the reply text says `assessment` where the stored ballot says `reason`.

Tests:
`reason-before-vote.unit.test.ts` in `lane-contest`,
`consolidate`,
`adjudication` and `archive` still pass over `reason-order.test-fixture.ts`:
each asks that the schema declares and requires the reason first and that the rendered instruction names it first.
The scripted replies of the consistency,
contest,
select,
coverage,
introduced-defect and rewrite tests,
and the fixtures of `provider`,
`repair` and `run`,
spell the new reply keys.
Two pins are hand-written because the scan cannot read the schema:
the finding schema of the rendering audit and the archive member of the contest ballot.
`source-scan:source-scans` is green with no allow-list.

Cache account:
the key of each version this account moves hashes the version,
the shape of the run and the texts,
not the sheet,
so a record written under the old reply instruction would be served under the new
and the number moves wherever a sheet carries a renamed or reordered schema
(as B732 did).
- `CONSOLIDATE_CACHE_VERSION` 30 to 31
  (main set 30 for B805 while this change was written,
  so this change took the next number when it merged main):
  the contest ballot of both gates,
  the naturalness review,
  the coverage judge,
  the select ballot and the introduced-defect check over a polish.
- `LANE_CONTEST_CACHE_VERSION` 14 to 15:
  the contest ballot.
- `TRANSLATE_SLICE_CACHE_VERSION` 23 to 24:
  the rewrite wire,
  the select ballot and the coverage judge.
- `SLICE_CACHE_VERSION` 41 to 42:
  the panel verdict,
  the critic,
  the introduced-defect check,
  the select ballot,
  the coverage judge and the editor's rewrite.
- `REFINE_CACHE_VERSION` 12 to 13:
  the refinement rewrite,
  the select ballot and the introduced-defect check.
- `PAGE_TITLE_CACHE_VERSION` 4 to 5:
  the page title lexicon reply.
Stay:
`CONSISTENCY_CACHE_VERSION` 6,
because only the reply is renamed,
what a stored answer must contain is what it was,
and its key digests the exact sheet,
whose reply instruction changed;
`PAIRING_CACHE_VERSION` 3,
since no pairing sheet carries a schema of this change.
The rendering audit has no version:
the only callers of `runRenderingAudit` are the probe runners
`probe/src/corpus-run/audit-sensitivity-run.ts` and `rendering-audit-settled-buy.ts`.
Evidence:
`cache-account-audit` read eight versions,
and `git log` shows the five slice versions and the consolidate version last set in `4b9fc9e645` on 2026-10-21 (UTC),
the page title version in `28f4b8fc75`.
It found 0 slice-cache records under this worktree's runs directory,
and no control that finds records was run,
so that absence shows nothing.
The pass that writes records runs in another worktree that this change did not read,
so whether a record stands at the old numbers is not known and none of the changes rides inside an old number.
No record has been written at any new number.

Known limits:
- Whether Hyper sorts the schema it sends the model or the reply the model wrote was not probed.
  The decision here is that declared order equals alphabetical order,
  which holds on both readings.
- The scan reads literal `properties` objects in non-test source.
  A schema built by spread or `Object.fromEntries` needs a pin of its own.
- The reason of the rewrite wire and the introduced-defect wire stays after the content it explains,
  by the rulings of B732,
  so a reply cut short by the completion cap still loses it first.
- A key named for its sort position is a cost to the reader of a log,
  who now meets two names for one field.
- Whether a ballot written reason first moves votes is not measured.

Open to the owner's veto:
the names chosen and the decision to rename rather than probe Hyper.

The rule for next time:
a ruling about the order of keys in a reply is checked against a provider that reorders them,
and what the reply order depends on is stated beside the ruling.
A key is named for its role so that it sorts where the ruling puts it,
and a schema is declared in the order its keys sort.
Recurrence:
`mistake-prevention-sheets-quorums-and-outcome-rules.md`,
"Ballots give their reason first".

## B805: the polish writers were never shown the claims the polish gate judged them against

Status:
fixed on 2026-10-22 (UTC),
red `e4a796cae6660ae4c2ffc952351b83b66e90cd18`,
fix `27db2c256c512c9d2b25c7da15b0e1e8c1868760`,
cache version `CONSOLIDATE_CACHE_VERSION` 29 to 30.
Found in the read of the corpus pass for `zheermao22` (class K3) and fixed in `w263`.
It widens B731,
which gave the claims to the polish gate alone.
Widened again before landing,
with red `7e7f5c16bb959c963cad51dfae64974f57f4a3c1`
and fix `c18194a50c933cb8e019381d6c3d5c354d20da46`,
to the dispute note and to the judges of the writers' rewrites (see "The widening before landing").

What the code did:
B731 threaded `acceptedRepairClaimsNote` from the repair lane's chunks to `runConsolidationPolishRound`,
where it was handed to the polish gate and to nothing else.
The same round called `runRefineStage` with no note,
so the writers of the polish rewrote the base without the claims,
and the gate then judged their rewording against claims they had never seen.
A claim accepted at minor severity,
which `disputesArchive` does not pass on as a dispute,
thus reached the judges and not the hands that could act on it.
The repo rule is that what a writer is shown,
the judges of its text are shown;
B731 applied it in one direction.

The fix:
`buildRefineMessages` and `runRefineStage` take `acceptedRepairClaimsNote`,
and the polish round passes the one it already holds to the gate.
The note is the one `acceptedRepairClaimsNote` of the archive package writes (B731);
no second builder was written.
The sheet shows it fenced under `REPAIR PANEL CLAIMS ACCEPTED`,
as the gate's sheet does,
with a fence chosen against it as well,
and says that the accepted claims are claims and not facts,
to be checked against the ORIGINAL and corrected only where it supports one.
The baseline policy of a round that carries claims no longer says that nobody has claimed anything wrong,
and names a paragraph a supported claim shows to be wrong as a clear improvement.
The repair lane's own refinement passes no note,
and its sheet is byte for byte what it was
(`refine-prompt.unit.test.ts`,
"LEAVES THE SHEET UNCHANGED WHEN NO CLAIMS WERE ACCEPTED").
The dispute rule is untouched:
a claim still disputes the archive only at the severity `disputesArchive` admits
(the owner's "Major+ accuracy",
2026-09-27).

Red and green:
`refine-prompt.unit.test.ts` reads the sheet for a note carrying a line of the old fixed fence and an instruction,
and for a round with objections and claims together.
`consolidation-polish-apply.unit.test.ts` reads every `refine_report` sheet of a polish,
with a control slice that has none.
`consolidate-driver.unit.test.ts` reads the writers' sheets through the real driver.
`rendered-sheets-context.unit.test.ts` gained the third block,
`REPAIR PANEL CLAIMS ACCEPTED`,
and names the sheets that must carry it (`polish gate`,
`polish refiner`),
so a sheet that loses the block cannot take its only mention of the heading with it;
the fixtures render a `polish refiner` sheet and give the gate sheet the note.
`sheet-translators-notes.unit.test.ts` lists the new sheet among the writers.
Five cases failed before the fix and pass after it;
the controls (a slice with none,
an empty and an absent note) passed throughout.

Cache account:
`CONSOLIDATE_CACHE_VERSION` moves from 29 to 30.
The consolidation key already appends the note when a slice has one,
so the key does not change when the writers' question does,
and a settlement stored at 29 for a slice with claims holds a polish written without them.
Runs `zheermao22` and `XingZ6017` log `START tip=2a336603bf`,
which holds the commit that set 29 (`4b9fc9e645`),
so records may stand at 29.
No slice-cache directory was searched.
The pinned `consolidateSliceKey` literal in `consolidate-key.unit.test.ts` follows the new version;
no test names the constant.
`REFINE_CACHE_VERSION` stays at 12:
the repair lane's refine sheet is unchanged.
The lane-contest,
translate slice and repair slice versions stay,
since none of their sheets is a polish writer's.

The widening before landing:
the first version of this entry left two items open,
and the parent session closed both before the branch landed,
open to the owner's veto.
- The dispute note (`archiveDisputeNote`) reached the polish gate and not the polish writers.
  The gate is shown it (`consolidation-polish-gate-wire.ts`,
  the block `ARCHIVE RENDERING DISPUTED`),
  so the writers must see what they are judged against:
  `runRefineStage` takes `archiveDisputeNote`,
  `buildRefineMessages` shows it fenced under that heading with the instruction to treat it as quoted review data,
  and the polish round passes the one it already holds to the gate.
  Where a slice has a dispute note and no other claim,
  the baseline policy names the block headed `ARCHIVE RENDERING DISPUTED`;
  where it has both notes,
  it names both blocks.
  A note that is absent or empty builds the sheet that was built before.
  This matters beyond a claim at minor severity:
  `acceptedRepairClaimNotesOf` leaves out a claim the dispute note already names,
  so on a slice whose accepted claims the dispute note all names,
  the writers saw no note at all.
- The judges of the writers' wording,
  the rewrite judges of the refine selection,
  were not shown the accepted claims either.
  `buildRefineSelectionContext` takes both notes and carries them as evidence under the gate's headings,
  the dispute first and the claims after,
  after the declared names and the cited pages.
  The select wire prints each evidence block as its label,
  a fence chosen against everything the sheet encloses,
  the text and the fence again,
  which is the shape every other select sheet gives `ARCHIVE RENDERING DISPUTED`
  (`translate-slate-evidence.ts`).
  `runRefineStage` hands both notes to the context and to the writers' sheet from one place,
  so the writers and the judges cannot disagree about whether a slice has them.
- The judges' criteria could not have used the notes unqualified.
  The comparative question asked that a candidate "say exactly what the CURRENT text says",
  and a rewrite made to a supported claim does not.
  The polish gate's policy is not qualified in that way,
  and its notes carry their own sentence that a candidate correcting a supported claim is not departing from the original.
  The select criteria are stricter,
  so where either note is shown the first criterion allows a correction that an accepted claim the ORIGINAL supports asks for,
  the third allows a candidate that makes such a correction,
  the task says so,
  and the objection mode's criterion about what a candidate may change names a supported claim beside a supported objection.
  Without either note the wording is what it was,
  and the control cases pass throughout.
- The repair lane's own refinement passes neither note,
  so its writer sheet,
  its judges' sheets and `REFINE_CACHE_VERSION` are unchanged.
  The floor-input scan lists that omission with this reason (`floor-inputs-stated.unit.test.ts`).

Red and green of the widening:
`refine-prompt.unit.test.ts` reads the writer's sheet for a dispute note carrying a line of the old fixed fence and an instruction,
for both notes together,
and for an absent and an empty note.
`refine-selection-notes.unit.test.ts` reads the evidence of both modes,
its order,
the criteria,
and the rendered judge sheet with a fence longer than the run in the note.
`refine-stage.unit.test.ts` reads the rewriter's sheet and each of the four judges' sheets in both modes,
with a control that is asked the same and carries neither heading.
`consolidation-polish-apply.unit.test.ts` and `consolidate-driver.unit.test.ts` read the writers' and the rewrite judges' sheets through the real polish and the real driver,
with a disputed slice and an undisputed one.
`rendered-sheets-polish.test-fixture.ts` renders the `polish refine slate` and the `polish refine objection slate`
and gives the polish gate and the polish refiner the dispute note beside the claims;
`rendered-sheets-context.unit.test.ts` names the four sheets that must carry both blocks
and scans every sheet naming `ARCHIVE RENDERING DISPUTED`.
Six of the new cases failed in the repair package,
one in the consolidate package,
two through the real driver and two in the rendered-sheets scan before the fix and pass after it.
A test written in the red commit read the fence of the second block from the wrong line,
and the fix commit corrects it.

Cache account of the widening:
no further bump.
Main holds `CONSOLIDATE_CACHE_VERSION` 29 (read off `origin/main` on the day of the merge) and this branch moved it to 30,
so one bump serves the landing.
The judges' question and the writers' dispute note are one more change to what a slice with a note is asked,
and the key already carries both notes
(`repair-claims` and `dispute` in `consolidateSliceKey`),
so a settlement stored at 29 holds a polish chosen without them and 30 recomputes it.
`REFINE_CACHE_VERSION` stays at 12:
the repair lane passes neither note,
and a context built without them is equal to the one built before
(`refine-selection-notes.unit.test.ts`,
"BUILDS THE SHEET IT BUILT BEFORE WHERE NO NOTE IS GIVEN").

Not measured:
whether a rewrite made to a claim now wins more often with the judges who can read the claim.
The sheets are tested,
not the votes,
and a vote needs a keyed rerun measured against run-to-run spread.

Open to the owner's veto:
the dispute note to the writers and the judges,
the judges' qualified criteria,
and the single cache bump.
Reverting the first two is the two optional parameters of `buildRefineSelectionContext` and `buildRefineMessages`.

Recurrence:
`mistake-prevention-sheets-quorums-and-outcome-rules.md`,
"What each sheet is shown":
a block added to a judge's sheet is threaded to the writer of the text it judges in the same change,
and to every other judge of that writer's text.
