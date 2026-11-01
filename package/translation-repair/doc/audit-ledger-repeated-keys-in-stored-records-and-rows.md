# Audit ledger: repeated keys in stored records and rows

Stored records,
rosters,
ledger rows and log lines that name one key more than once,
read by the first or the last without saying so.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B205: a claim's proposers were taken from the last chunk that names it

Red in `49e3b8dfd`,
fixed in `21f2ec369`.

Found on 2026-10-05 (UTC) by the same census,
reading the maps filled by `set` in a loop.
`buildCrosscheckCensus` (`corpus-run/judge-crosscheck.ts`) set a claim's proposers
at each chunk of an entry's slice critics,
so where two chunks carry one claim id the last chunk's list replaced the others,
and an author from another chunk could be seated as a judge of its own claim.
`indexProposers` (`corpus-run/attribution-report.ts`) merges such chunks,
and its comment says why.

The fix:
the proposers are merged across chunks,
each model once,
in the order first met.

Whether two chunks can carry one claim id was not proved:
the fix rests on that comment and on being harmless,
and over 226 stored artifact files holding 21,572 attributions
no claim id stands in more than one chunk of an entry
(the fixing agent's measurement,
with a control it had to flag).

Recurrence:
`mistake-prevention.md`,
"Tables keyed by text".

## B206: a stored page assembly was read by the first row naming a slice

Red in `49e3b8dfd`,
fixed in `21f2ec369`.

Found on 2026-10-05 (UTC) by the same census.
`pageAssemblyOverrideAt` (`corpus-run/artifact-two-lane-page-assembly.ts`) takes the first trimmed row naming a slice,
and `parsePageAssembly` admitted a file holding two.
The writer cannot write two:
`guardPageAssembly` composes the guard's trims,
one per slice by the splice,
with the rows of a map keyed by slice.
A file edited by hand or written by another build could,
and the reader answered with whichever row stood first.

The fix:
`parsePageAssembly` refuses the second row,
`expected one row per slice; slice N appears more than once`,
in the words `parseConsolidation` uses for its slices.
`withdrawn` is left alone:
it is read by membership,
where a repeat changes no answer.

No stored file is refused
(the fixing agent's measurement):
of 212 stored sections holding 221 trimmed rows,
none names a slice twice.

Recurrence:
`mistake-prevention.md`,
"Tables keyed by text".

## B207: the delivery ledger keyed trimmed texts by slice without the check its two index lists have

Red in `49e3b8dfd`,
fixed in `21f2ec369`.

Found on 2026-10-05 (UTC) by the same census.
`buildSliceDelivery` (`slice-delivery.ts`) refuses a shipped or a withdrawn index named twice,
checked against the arrays because a set forgets the repeat,
and built a map of trimmed texts by slice with no such check,
so of two trimmed replacements for one slice the later would have been the text the ledger reports.
Its callers pass the assembly guard's trims,
which the splice holds to one per slice,
so no run reaches it;
the signature admits any list.

The fix:
a second trimmed replacement for a slice is the fault `trim-repeats`
(`slice-delivery-fault.ts`),
worded `slice N carries more than one trimmed replacement, so which text the document holds there cannot be read`.

Open to the owner's veto:
the fault is a kind of its own,
where the other way widens the set name the two sibling repeats share,
which a third fault also carries and could then word wrongly;
and the refusal stands where uniqueness is proved in another file,
neither by type nor by the lines beside it.

Recurrence:
`mistake-prevention.md`,
"Tables keyed by text".

## B208: a roster card was printed from the first row a listing carries under an id

Red in `49e3b8dfd`,
fixed in `21f2ec369`.

Found on 2026-10-05 (UTC) by the same census.
`listingRowFor` (`corpus-run/roster-card-render.ts`) took the first row of a provider's listing carrying the served id.
A listing is a provider's reply,
and nothing in it makes an id name one row:
the card would describe whichever the provider listed first.

The fix:
every row under the id is read;
rows that say the same are one answer,
and rows that differ are a `StatedRefusalError` naming the id the operator typed and nothing from the listing.
The one caller is the `roster-card` command,
which states its other refusals the same way.

Open to the owner's veto:
rows that say the same count as one answer,
where the other way refuses any repeat.

Recurrence:
`mistake-prevention.md`,
"Tables keyed by text".

## B209: three readers of the package's own log lines settled a repeated field by order

Red in `49e3b8dfd`,
fixed in `21f2ec369`.

Found on 2026-10-05 (UTC) by the same census.
`reportSpend`,
`takeReading` and `armSliceCost` each write a field once,
so a line naming one twice is two records run together or a line cut and resumed.
`namedFields` (`corpus-run/spend-read.ts`) and `fieldsOf` (`slice-cost-read.ts`) kept the later value,
the second by its own TSDoc,
and `fieldValue` (`corpus-run/meter-sample-read.ts`) read the first:
each returned one clean record out of two.

The fix:
`readSpendLine` answers `'unreadable'`,
`readMeterLine` answers `'skipped'`,
for a repeated state or level,
and `readSliceCosts` drops the line and names each repeated field,
each module's own value for a record that will not read.
The meter's check is asked once both required states have read,
so that an older case about an absent first reading keeps passing for the reason its name gives;
the agent's reviewer caught a first placement that made it pass for the new reason.

No report over a stored log changes
(the fixing agent's measurement over 1,928 stored run logs,
the builds before and after the fix side by side,
with a control):
922,240 `SPEND` records and 32,751 `METERS` records read alike,
and of 292,458 `SLICE-COST` lines the 9 that read differently are test output.

Open to the owner's veto:
a damaged record is a hole,
where the other way reads its first or its last value.

Recurrence:
`mistake-prevention.md`,
"Readers of the package's own log lines".

## B238: a roster seating one model twice was gathered as several voices

Red in `d00bc71b4`,
fixed in `8ce45ae86`.

Found on 2026-10-06 (UTC)
by the second pass over the first-match census of B203,
which read the sites that census described and left.
`gatherStageVoices` counted each seat of a roster as a voice toward quorum.
The panel,
the critics and the page title bench keyed what they read by model id,
so a roster naming one model twice met quorum on voices that were one model and kept one of its replies.
Editors and checkers were refused a repeated id where their rosters are composed.
The agent's probe of the unfixed build gathered the panel,
critic and page-title stages over a roster of three seats naming one id twice,
and all three were accepted.

The fix:
such a roster is refused at the start of `gatherStageVoices`,
before any seat is asked,
as `StageRosterRepeatError` (`stage-roster-repeat.ts`),
marked,
naming the stage and each repeated id once.
The lead merged the inventory lines for the class and its `stage` part by hand beside another change's additions.

Measured by the lead:
no roster a run can seat trips the refusal.
`judgeSeatsFor`,
over all 16 combinations of four providers dry or wet,
returned 278 arrays of ids with no repeated member,
and 70 exported roster-like arrays hold none;
the same reader finds the repeat in a control roster.
Rosters a stage builds from these,
and a roster an operator's flag names,
were not read.

The whole suite's run on `8ce45ae86` is recorded in B235.

Open to the owner's veto:
a refusal in the gather for any stage.

Recurrence:
`mistake-prevention.md`,
"Tables keyed by text".

## B239: a stored naturalness confirmation bound to the first of two rounds it fits

Red in `d00bc71b4`,
fixed in `8ce45ae86`.

Found by the second pass B238 records.
`parseNaturalnessConfirmations` bound each confirmation to the first decisive round carrying its candidate's digests.
A correction chain that returns to a candidate it had reviewed leaves two rounds that fit,
and `assertNaturalnessCorrectionChain` does not refuse such a chain,
so the order and roster checks read the first round while the confirmation names neither.

The fix:
the reader gathers every fitting round and refuses a confirmation that fits more than one,
as `ArtifactParseError` naming the confirmation's position.

The agent's count,
not rerun by the lead:
76 stored artifacts carry no confirmation,
so the measurement refuses nothing and proves nothing,
and the writer records one round today.

Recurrence:
`mistake-prevention.md`,
"Tables keyed by text".

## B240: probe tallies took the later of two regions under one envelope id

Red in `d00bc71b4`,
fixed in `8ce45ae86`.

Found by the second pass B238 records.
`readRecordedTallies` built the tally table with `Object.fromEntries`,
so a second region under one envelope id in a stored record replaced the first tally without a word.

The fix:
the reader refuses the second region as `ArtifactParseError`,
naming its position and not the id.

The agent's count,
not rerun by the lead:
3,187 tally blocks in 76 stored artifacts,
none repeating an envelope id.

Recurrence:
`mistake-prevention.md`,
"Tables keyed by text".

## B241: the first slice carrying a replaced text was taken as the region's slice

Red in `d00bc71b4`,
fixed in `8ce45ae86`.

Found by the second pass B238 records.
`locateSlice` and the control gatherer of the relabel probe took the first slice
whose translation included a region's replaced text.
A short replaced text can stand in several slices,
and the probe would then rebuild its prompt from the wrong one.

The fix:
`holdingSlices` gathers every carrying slice and refuses more than one,
as `ArtifactParseError` naming the text's length,
in the case builder and in the control.

The agent's count,
not rerun by the lead:
each of the eight damaged-case positions has one carrying slice on the stored round-three run,
so none is refused,
and a two-letter text stood in 5 of 5 slices as the positive control.
The control's candidate regions were not counted.

Open to the owner's veto:
the relabel control refusing an ambiguous region where skipping it is the alternative.

Recurrence:
`mistake-prevention.md`,
"Tables keyed by text".

## B242: the window trial report let a later ledger row replace an earlier one for an arm

Red in `d00bc71b4`,
fixed in `8ce45ae86`.

Found by the second pass B238 records.
`groupBySlice` set each arm of a slice with `Map.set`,
so a ledger holding two rows for one arm credited the later one.

The fix:
a slice with two rows for one arm is left out whole and counted in `ClassReport.repeated`.
A refusal was rejected,
since the ledger is append-only
and the report would stay unreadable for want of a repair nobody may make.
`windowTrialReportLine` (`window-trial-report.ts`),
moved out of the probe's `main` so a case can read it,
ends with the count of slices left out for an arm the ledger holds twice.
The case is "PRINTS THE SLICES LEFT OUT FOR AN ARM THE LEDGER HOLDS TWICE
beside the incomplete and degraded counts,
with the noun agreeing with the count",
for two slices and for one.

Not measured:
no stored window-trial ledger was found to count repeats in.

Open to the owner's veto:
a count where a refusal is the alternative.

Recurrence:
`mistake-prevention.md`,
"Tables keyed by text".

## B243: settled audit runs were paired by a subject key built from a file the reader casts

Red in `d00bc71b4`,
fixed in `8ce45ae86`.

Found by the second pass B238 records.
`auditRepeatsAcross` keyed a run's rows by a subject key from rows the reader casts.
A repeated subject replaced the earlier row,
and a row without a numeric `sliceIndex` keyed as `undefined`.
The six stored run files,
written before commit `49e5a41cd` renamed the field `chunkIndex` to `sliceIndex`,
hold 124 rows that the old key collapsed to 15
and that are distinct by `chunkIndex` (the agent's count).

The fix:
both runs go through one reader that refuses a row naming no subject by either name,
and a subject on two rows,
as `StatedRefusalError` naming positions.
`sliceIndexOf` reads `chunkIndex` for a row with no `sliceIndex`,
so a run of that generation pairs with one of this generation by that value.
The case is "PAIRS A RUN WRITTEN BEFORE THE RENAME OF `chunkIndex` TO `sliceIndex`
with a run of this generation by that value,
since the rename changed the field's name and not what it holds".

Measured by the lead:
the rename was a rename alone in the audit's sources.
In commit `49e5a41cd`,
across the nine rendering-audit files it touches,
every removed line is one of that file's added lines with the field's name respelled;
across all 205 files of that commit the same test leaves 80 lines in 18 files unexplained,
so the test can show a difference.
Whether slicing itself changed between the stored runs of 2026-08-17 and that commit was not read.

Open to the owner's veto:
reading `chunkIndex` where refusing a run of that generation is the alternative.

Recurrence:
`mistake-prevention.md`,
"Tables keyed by text".

## B244: a pre-grade file naming one sheet position twice was scored by its later verdict

Red in `d00bc71b4`,
fixed in `8ce45ae86`.

Found by the second pass B238 records.
`scoreGradeAgreement` keyed pre-grades by sheet position with a map built from pairs,
so a file naming one position twice and carrying one row too many
matched the sheet's length and scored the later verdict.

The fix:
the map is built in a loop that refuses a repeated position as `StatedRefusalError` naming the position,
and the length check compares the count of rows.
`grade-agreement.unit.test.ts` is the module's first test file of its own,
so its line left the allow-list of `own-unit-tests.unit.test.ts`.
That scan was red at `d00bc71b4` for the stale line,
by the agent's run,
and the red commit's message says so.

Recurrence:
`mistake-prevention.md`,
"Tables keyed by text".

## B245: a prober answering one region twice was counted by its first check without a word

Red in `d00bc71b4`,
fixed in `8ce45ae86`.

Found by the second pass B238 records.
`screenIntroducedDefects` searched each prober's checks for the first on a region
and said nothing of a second.

The fix:
`resolveProberChecks` resolves each prober once,
first kept,
and writes `<model id>: duplicate-check (<region>)` for each later check,
as `resolveResolutionChecks` does for checkers.
The tallies are unchanged.
`runIntroducedDefectProbe` appends those findings to `IntroducedDefectReport.findings` after the gather's own.
The case is "REPORTS A PROBER THAT ANSWERS ONE REGION TWICE as a duplicate-check finding under its model id,
and counts its first check once".

Open:
the findings are not written into artifacts,
since `repair-record.ts` rebuilds the probe block field by field and leaves them out on purpose,
and the agent did not trace which scorecard prints them.

Recurrence:
`mistake-prevention.md`,
"Tables keyed by text".

## B246: a card dropped by a repeated served id was invisible to the key check

Fixed in `8ce45ae86`,
with its case,
which needs a name the fix adds.

Found by the second pass B238 records.
`servedRecord` (`model-card-derive.ts`) built its record with `Object.fromEntries` and checked key sets afterwards,
which cannot see a card replaced by a later card with the same served id.

The fix:
`recordOfDistinctIds` refuses the repeat where the record is built,
naming the provider and the id.
The real cards are distinct,
since the catalogs build at import and every test imports them,
so the case drives the helper.
The refusal was a `RangeError` at this commit and became an `Error` in `71974714f`,
whose message leaves that open to the owner's veto.

Recurrence:
`mistake-prevention.md`,
"Tables keyed by text".

## B265: no scan held a map write over a key its function never reads

Added in `a6d1fe4f7`.

Found by the census of B263,
which read every two-argument `set` in production code by hand.
Nothing stopped a new one from replacing an earlier value without a word,
the shape of B242 and B263.

The fix:
`blind-map-writes.unit.test.ts` fails a two-argument `set` whose enclosing functions never `get` or `has`
the same key of the same map,
unless an allowance says what makes the key unique,
and an allowance for a write that is gone fails too.
The lead wired it into the `source-scans` task,
which ran 38 files from then,
and edited three allow-list lines for sites that changed after the agent's base:
two writes added
(`rowsBySubject`,
refused through a second map under the same key;
`resolveProberChecks`,
keyed by the own keys of one record)
and one allowance gone stale
(`groupBySlice`,
whose reader now refuses a repeat).

Not reached,
by the agent's account:
a read of the key under another name or in another function passes;
first-match loops,
sets that drop a differing repeat and doubled keys in parsed JSON are not held by a scan.

Open to the owner's veto:
the scan counts a read in any enclosing function as a read.

Recurrence:
`mistake-prevention.md`,
"Tables keyed by text".
