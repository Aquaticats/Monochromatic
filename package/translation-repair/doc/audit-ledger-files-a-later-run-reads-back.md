# Audit ledger: files a later run reads back

Paths written `src/...` in this record name the layout before 2026-10-18,
when the package `package/module/translation-repair` was split into the packages under `package/translation-repair`.
`doc/audit/translation-repair-package-split-map-2026-10-18.json` gives each file's current path.

Caches,
ledgers,
locks and stored stage results that a later run read back,
torn,
stale or holding what no writer writes.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B107: the attribution decoder read every generation's record index under one spelling

Found 2026-10-01 (UTC) measuring B106 over the stored artifacts,
red in `3410cef02`,
fixed in the commit adding this entry.
Generations 1 to 3 wrote each per-slice record's index as `chunkIndex`,
and generation 4 onward writes `sliceIndex`;
`artifact-key-vocabulary.ts` records both,
and the attribution reader already chose the critic record's own key through it.
`decodeChunkRecord` (`corpus-run/attribution-decode.ts`) read `value.sliceIndex` whatever the generation,
so every generation 2 or 3 critic record read as missing its index,
and the whole artifact was refused as malformed.
Over the stored artifacts,
all 246 generation 2 critic records and both generation 3 ones carry `chunkIndex`,
and the gather refused 42 artifacts,
41 of generation 2 and one of generation 3
(`b24/slice-index-values.mjs` in the audit's scratch folder).
The two cases meant to cover those generations wrote `sliceIndex` into their records,
which no writer of those generations produced,
so they agreed with the decoder rather than with the writers.
A census of every reader of stored artifacts found no other reading of a renamed key outside the vocabulary:
the two-lane readers take the vocabulary,
the page-assembly reader reads only generations that spell the index `sliceIndex`,
and the change-set reader refuses every generation past 1.

`decodeSliceCritics` now takes the generation's vocabulary,
and `decodeChunkRecord` the index key it names,
so a record is read under its own generation's spelling
and a record carrying the other spelling is refused,
naming the key its generation wrote.
The generation 2 and 3 cases spell their records as those generations did,
a generation 14 case reads as the control,
and a case refuses each spelling under the other generation;
`artifactWith` and the explicit generation 1 and 2 fixtures spell each record's index `chunkIndex`,
as does the sound generation 1 fixture in `corpus-run/sink-names-only.unit.test.ts`,
which wrote `sliceIndex`,
so the fix refused it and the full suite failed on it until it was respelled.
The red cases failed at the build of `cd3cab768`,
and pass here.

Reach:
the built gather over the 240 stored directories holding JSON
read 202 entries with 45 refused at the build before this fix,
and 244 entries,
all eligible,
with 3 refused here:
the three `Cat.json` test leftovers B106 names.

Recurrence:
`mistake-prevention.md`,
"Guards that cannot fail".

## B134: a lookup cache file cut short took its title's evidence away on every later run

Red in `8c7426a44`,
fixed in `971c8bec8`.
Found on 2026-10-05 (UTC) by a delegated agent
reading `readCachedLookup` (`lookup-cache.ts`) against its own `@returns`,
"a miss when the file is absent or is not a record".
The read ran `JSON.parse` on the file's text with no catch,
so a file cut short raised a `SyntaxError` out of `lookupWorkTitle` (`work-title-lookup.ts`).
`workTitleLookupLines` caught it with every other failed lookup,
logged that the title's lookup failed and contributes no line,
with the parser's words and no path,
and returned no line for the title.
Nothing was bought and nothing rewrote the file,
so every later run met it and the title never had its evidence again.
V8's refusal can quote ten characters of the text it refused,
so that warning could carry a cache file's opening.
An empty file,
which a write refused before its first byte leaves,
read as absent with no line,
and `textOrNothing` read every other read failure as absent too,
at debug level:
with a directory at the record's path,
the title was bought on each of two runs and never kept.
The write was a plain `writeFile`,
which truncates before it writes:
under a per-process file-size limit standing in for a full disk,
a plain write left 512 bytes of a longer record at the cache path.
These are the agent's measurements,
on its builds of `f0f449765` and of `7f069adc9`,
whose cache modules are the tip's.

No run was harmed as far as this machine's cache shows:
the agent counted 58 lookup records and 14 reference records,
all whole,
none empty;
the run logs were not searched.

The fix:
`cacheFileText` answers `CACHE_FILE_ABSENT` only where nothing stands at the path (`isMissingPathError`)
and raises `CacheFileUnreadableError`,
whose message names the filesystem code and the path,
for any other read failure;
`readCachedLookup` takes the parse failure as data (`parseModelJson`),
warns `cached lookup at <path> does not parse as JSON; ignoring it` and misses,
so the caller buys the record once and the write replaces the file;
`writeCachedLookup` writes through `writeFileAtomic`.
The class is listed among the classes whose message names only a code and a path
(`message-names-only.unit.test.ts`).

Red on the tree of `8c7426a44`:
both cache test files exit 1 at load,
since the cases assert the class the fix adds.
The per-case reds are the agent's,
on the build before the class existed.
The whole suite on `971c8bec8` printed 1581 [PASS] and 0 [FAIL] lines,
the lint `Found 0 warnings and 0 errors.`
and the scans 35 [PASS] lines.

Open to the owner's veto:
a file that is there and cannot be read is a refusal,
not a miss,
since a miss buys the record and then meets the same path at the write,
paying on every run and keeping nothing;
that is the package's rule in `missing-path-error.ts`,
and what `readAttemptMap`,
`readNamespaceGeneration` and `readPayloadText` do.

Recurrence:
`mistake-prevention.md`,
"Files a later run reads back".

## B135: a reference cache file cut short failed the entry citing its page on every run

Red in `8c7426a44`,
fixed in `971c8bec8`,
with B134,
whose twin `readCachedReference` (`reference-cache.ts`) is.
`citedReferenceBlock` (`cited-reference-lookup.ts`) reads every cited page's cache file before it buys anything,
outside the catch that turns one page's failed fetch into a line,
so the `SyntaxError` of one file cut short rejected the whole block:
on the agent's build of `f0f449765`,
a raw `SyntaxError` and no transport call.
The rejection leaves `preparePassEntry` and is tallied by `runEntryPipeline`
as an entry error naming no file
(read off `tally-error-text.ts` by the agent,
not run end to end),
on every run until someone found the file and deleted it.
A directory at a page's path read as a miss instead:
the page was bought on each of two runs,
its record never kept,
and its line said it could not be fetched.

The fix:
`readCachedReference` reads through `cacheFileText`,
takes the parse failure as data and misses with
`cached reference at <path> does not parse as JSON; ignoring it`,
so the page is bought once and its file replaced;
a path that is there and cannot be read raises `CacheFileUnreadableError`,
which rejects the block before anything is bought
and reaches the tally line with the code and the path;
`writeCachedReference` writes through `writeFileAtomic`.
`reference-cache.unit.test.ts` is new,
so `reference-cache.ts` leaves the own-tests allowlist.

Open to the owner's veto:
for a work title the existing catch in `workTitleLookupLines` turns the refusal into no line,
with a warning naming code and path;
for a cited page the block rejects and the entry is tallied as an error.
The asymmetry is the callers' existing design.

Open,
found by the same agent's audit of every `JSON.parse` of a file and every plain writer:
`readTrialLedger` (`corpus-run/window-trial-ledger.ts`) tolerates a line that does not parse only when it is the last,
and `appendTrialRow` appends a row with no newline before it,
so after a torn last line the first row a resumed run appends is joined onto the fragment and lost,
and a second makes the read throw a raw `SyntaxError`
(the agent's measurement:
one row and a fragment read as 1 row,
1 again after one appended row,
and a `SyntaxError` after two),
handed to the agent that owns that module;
`readBaselineCensus` (`corpus-run/coverage-census-input.ts`),
`bundleScriptsOf` (`corpus-run/coverage-file.ts`) and `readSourceMap` (`corpus-run/coverage-lines.ts`)
each run `JSON.parse` bare while each `@throws` promises its own class,
and `census.json` is written by a plain `writeFile`;
`writeFileAtomic` leaves `<path>.<pid>.partial` behind when its write fails (the agent's measurement);
`runs-lock.ts` creates the lock empty and then writes it,
and a starter that finds it unreadable takes it over,
so a second pass starting between the two could evict a live holder
(an inference from the code order,
not measured);
`work-title-lookup.ts` and `cited-reference-lookup.ts` log `String(error,)` of whatever they caught,
and V8's message for a provider body that is not JSON quotes ten characters of it,
where `refusalText` exists for that;
and `BedrockLedgerShapeError` names the ledger line and not the ledger's path.

Recurrence:
`mistake-prevention.md`,
"Files a later run reads back".

## B136: a lane's generation marker restamped in place on every open

Red in `83355d3a3`,
fixed in `fa2813de4`.
Found on 2026-10-05 (UTC) by the agent of B134 in the same audit.
`openNamespacedCache` (`corpus-run/slice-cache-namespace.ts`)
rewrote its lane's marker with a plain `writeFile` on every open,
also when the marker already named the running generation,
and a comment called the failure direction safe.
It was safe for correctness and not for cost.
A plain write truncates before it writes,
a marker left empty reads as another generation,
and the open that meets it discards every slice the lane holds for that entry,
which are then bought again.
The agent measured both halves:
under a file-size limit a marker holding `nap-3` read back empty after a refused write,
and an open over an emptied marker logged the discard of the lane's two cached slices.
The repair lane's cache is the first an entry opens,
so a disk filling up mid-run could empty that marker for each entry attempted meanwhile.

Making the write atomic and nothing else is wrong:
the agent tried it,
and a `pass-entry.unit.test.ts` case that resumes from a cache directory it may only read failed,
since a temporary file cannot be created there.

The red run printed 2 [PASS] and 4 [FAIL] lines:
a reader holding the marker open was shown the later bytes
(`expected 'nap-4\n' to equal 'nap-3\n'`),
and a reopen under the generation the marker names failed with `EACCES` in a directory it may only read.

The fix:
the restamp happens only where the marker does not already name the running generation,
which is where the open has just discarded and the lane holds nothing to lose,
and it goes through `writeFileAtomic`,
so a refused write leaves the earlier marker whole.
The patch is the agent's;
the restamp was folded into the discard's own block,
which tests the same condition.
The whole suite on `fa2813de4` printed 1582 [PASS] and 0 [FAIL] lines,
the lint `Found 0 warnings and 0 errors.`
and the scans 35 [PASS] lines.

Recurrence:
`mistake-prevention.md`,
"Files a later run reads back".

## B147: a `.png` picture decoded onto itself

Red in `e300e6c33`,
fixed in `d550bf04f`.

Found on 2026-10-05 (UTC),
replacing the trial's `ocrTool` seam with a required program runner (ledger M113).
The scratch copy of a picture was `asset.` plus its extension,
and the decoded copy was `asset.png`,
so for a `.png` asset the two were one file.
The fallback decoder rewrote its own input,
and a first decoder that wrote part of its output before failing
left the fallback to decode what it wrote.
The red case scripts a first decoder that writes garbage to its target and exits 1:
the reading came back as the garbage,
32 characters taken as text.
A `.webp` asset was never exposed,
its copy and target being different files.
No run is known to have been harmed:
the reviewer measured the real `dwebp` failing before it wrote,
and `magick` rewriting the file whole.
The fix:
the decoded copy is `decoded.png`,
outside every name the scratch copy takes,
and `@param png` says the two are never one file.
Recurrence:
`mistake-prevention.md`,
"File names derived from an input",
a new heading.

## B151: the consolidate store resumed a terminal beside a gate the stage never writes with it

Red in `9c03075d7`,
fixed in `341211543`.

Found 2026-10-05 (UTC),
reviewing the MiMo trial's test cases (review part 4,
F6).
`settleConsolidation` writes no gate on its five exits before the gate,
and `gateAndShip` always writes one,
naming the terminal from what that gate ships.
`isConsolidationSettlement` (`corpus-run/consolidate-cache-store.ts`)
checked each field's shape and never the two together.
A file with terminal `consolidated` and no gate was resumed,
and its `text` shipped under an artifact saying no gate was asked;
so was `consolidated` beside a gate shipping `standing`,
and `incumbent-only` beside a gate.
The trial's case pinned the first as resumable.

The fix:
`SETTLEMENT_TERMINALS` names beside each terminal the gate the stage writes with it,
and `gateFitsTerminal` refuses a record where the two disagree;
a refused record is bought again.
Other couplings the stage keeps and the store does not check are listed in the fix wave's report,
`settleGateBallots` against the stored `choice` first among them.
B187 refuses four of them and says why it leaves the rest.

Recurrence:
`mistake-prevention.md`,
"Guards that cannot fail".

## B152: a resumed window trial joined its first row onto a torn last line

Red in `9c03075d7`,
fixed in `341211543`.

Found 2026-10-05 (UTC) by the audit of file reads behind B134 and B135,
and reproduced on the build of the fix wave's worktree.
A kill mid-append leaves a last line with no newline,
and `appendTrialRow` appended the next row straight after it.
One appended row was lost and its arm bought again;
after a second,
the unreadable line was no longer last,
and `readTrialLedger` threw the parser's `SyntaxError`,
whose message quotes the text it refused,
at the opening and closing reads of every later run.

The fix:
before each append `appendTrialRow` reads the ledger's last byte,
and where it is no newline removes a fragment by an atomic rewrite,
or ends a whole row,
saying which through its logger;
the read warns of a torn last line and reports it as `tornTail`;
a line that ends in a newline and does not parse throws `TrialLedgerLineError`,
which names the file and the line number and none of its text.

Recurrence:
`mistake-prevention.md`,
"Files a later run reads back".

## B153: the window trial ledger dropped a whole line that was no row without a word

Fixed in `341211543`,
its case in `9c03075d7`.

Found 2026-10-05 (UTC),
reviewing the MiMo trial's test cases (review part 4,
F7).
`readTrialLedger` left out a line that parses and fails `isWindowTrialRow`,
and said nothing.
Rows of four older shapes were appended to this same file:
before `judgesHeard`,
before `position`,
and before `chunkIndex` became `sliceIndex`.

The fix:
`accountTrialLedger` returns the rows,
the count of whole lines left out,
and whether the last line was torn,
and says through its logger how many it left out and why;
`readTrialLedger` returns its rows,
and the window trial's closing report prints the count.

Recurrence:
`mistake-prevention.md`,
"Files a later run reads back".

## B163: three coverage census readers raised the parser's `SyntaxError` where their docs promise their own class

Red in `290110436`,
fixed in `2d5141018`.

Found on 2026-10-05 by the audit of B134,
and confirmed on the build of `e300e6c33`.
`readBaselineCensus` (`corpus-run/coverage-census-input.ts`),
`bundleScriptsOf` (`corpus-run/coverage-file.ts`)
and `readSourceMap` (`corpus-run/coverage-lines.ts`)
each ran `JSON.parse` bare,
while each `@throws` promises its own class for a file that does not read.
Over text cut short each threw `SyntaxError: Unterminated string in JSON at position 22 (line 1 column 23)`,
and over text that is no JSON at all V8's message quoted ten characters of it (the fixing agent's measurement).
The command line reports a class it does not know as a fault in the command,
read off `cli-refusal.ts`,
not run.
`census.json`,
which a later census reads back as its baseline,
was written by a plain `writeFile` in `corpus-run/coverage-census.ts`,
which truncates before it writes.

The fix:
each reader takes the parse failure as data (`parseModelJson`)
and throws its own class with `says: 'it is not JSON'`,
so the refusal names the file and quotes nothing;
`reportCensus` writes `census.json` through `writeFileAtomic`.
`path` and `says` were already named parts of the marker inventory.
Each reader's test file carries a file cut short,
asserting the class and the whole refusal.
The writer has no case:
only the census command reaches it,
and that command runs the whole suite under coverage.

Recurrence:
`mistake-prevention.md`,
"Files a later run reads back".

## B164: a failed atomic write left its temporary file, and two writes of one path in one process shared one

Red in `290110436`,
fixed in `2d5141018`.

Found on 2026-10-05,
the leftover by the audit of B134
and the shared name while fixing it.
`writeFileAtomic` (`corpus-run/atomic-write.ts`) wrote `<path>.<pid>.partial` and renamed it,
and nothing removed that file when either step failed.
A rename refused by a directory standing at the path left the whole temporary file (the fixing agent's measurement),
and a write refused part way under a per-process file-size limit,
standing in for a full disk,
left 8192 bytes of a 65536 byte text (the fixing agent's measurement).
Every reader steps over such a name,
through `directory-listing.ts` and its own suffix filter,
and none relies on finding one,
so nothing was misread;
the files only stayed.
The temporary name was unique to the process and not to the call,
so two writes of one path at once in one process opened the same file:
of 100 such pairs,
100 rejected one write with `ENOENT` at its rename,
and 8 left text at the path that was neither write whole (the fixing agent's measurement).
No production caller was shown to write one path twice at once;
the run queue runs entries one at a time.

The fix:
the temporary name carries the process id and a `randomUUID`,
and a write that fails removes its temporary file before its refusal goes on unchanged;
a removal that fails too is a warning naming the file and the filesystem code.
After it a refused rename and a refused write leave nothing beside the path,
and 100 pairs of writes at once rejected none and tore none (the fixing agent's measurement).
`atomic-write.unit.test.ts` carries the refused rename and the two writes at once.

Recurrence:
`mistake-prevention.md`,
"Files a later run reads back",
with one more clause:
a temporary name is unique to its call,
not to its process.

## B165: a second pass starting while the first claimed its lock took the lock over

Red in `290110436`,
fixed in `2d5141018`.

Found on 2026-10-05 by the audit of B134 as an inference from the code order,
and measured here.
`claim` (`corpus-run/runs-lock.ts`) created `pass.lock` empty with an exclusive open
and wrote the holder into it after,
and `lockRunsDir` takes over a lock it cannot read,
since a pass killed between the two leaves exactly that.
A second starter arriving between the create and the write
read the live holder's empty lock as unreadable,
moved it aside,
claimed its own,
and both passes held the directory:
two starters at once on a fresh directory in one process both acquired it in 28 of 200 rounds
(the fixing agent's measurement),
and four starters in 3 of 30 rounds (the fixing agent's measurement).
The module takes no seam to pause a claim,
so the case runs both starters in a child process that replaces `open` and `link` of `node:fs/promises`,
running the second starter to its end inside the first one's claim;
on the build of `e300e6c33` both acquired.

The fix:
the claim writes the holder's text to a name only it knows with an exclusive create
and hard-links that name to `pass.lock`,
which fails with `EEXIST` when anything stands there,
so the claim and the text are one filesystem operation;
the staged name is removed whatever the link answers.
After it no round of two,
four or eight starters ended with more than one holder (the fixing agent's measurement).
A runs directory on a filesystem without hard links now refuses the lock with that filesystem's own error.

Recurrence:
`mistake-prevention.md`,
"Files a later run reads back":
a lock another process may read as abandoned
appears with its content already in it.

## B183: an unreadable census baseline reached the command as the read's own error

Fixed in `ab11a69d6`,
its cases in the same commit,
since they call the reader the fix adds.

Found on 2026-10-05 (UTC)
by the agent that fixed B163,
under its other observations.
`runCoverageCensus` (`corpus-run/coverage-census.ts`) read each `--baseline` file with `readFile`,
so a path that is not there or is a directory raised Node's `ENOENT` or `EISDIR` error unchanged,
with no refusal that names the baseline.

The fix:
`readBaselineFile` (`corpus-run/coverage-census-input.ts`) reads the file and hands its text to `readBaselineCensus`,
and a failed read throws `CensusBaselineError` naming the path and the filesystem code
(`it could not be read (ENOENT)`),
with the failure as `cause`;
`runCoverageCensus` calls it.
Cases read a missing path and a directory under a scratch folder;
before the fix the same reads raised `ENOENT` and `EISDIR` (the fixing agent's red run).

Open when this entry was written,
closed in B221:
`reportingRefusals` (`corpus-run/cli-refusal.ts`) still prints a `CensusBaselineError`
followed by the fault-in-the-command lines and a stack,
as it did for the not-JSON refusal of B163,
though a baseline named wrongly is the operator's mistake and no fault of the command.

Recurrence:
`mistake-prevention.md`,
"Files a later run reads back":
a command that reads a file the operator names refuses an unreadable one by name before it spends a suite.

## B187: the consolidate store resumed a choice, two flags and a mark the stage never writes

Red in `7f99edb6a`,
fixed in `17646f215`.

Found on 2026-10-05 (UTC),
as the couplings ledger B151 left listed.
`isConsolidationSettlement` (`corpus-run/consolidate-cache-store.ts`) checked a stored gate's `choice` against nothing,
though the stage settles it from the stored ballots with `settleGateBallots`
and `shipPastForfeitStanding` rewrites only `ships`.
It resumed `demoted` beside any terminal,
where every writer sets it on `wrap-erased-difference` alone;
`rewrapped` beside a terminal that never wraps;
and a settlement marked `archiveKept`,
which `consolidationWorthResuming` never persists
(each the fixing agent's reading of the writers).

The fix:
the store refuses each of the four.
The table of terminals carries the `demoted` flag each one writes and whether it may rewrap,
`wrapFlagsFitTerminal` reads them,
and a stored `choice` must be what `settleGateBallots` gives for the stored ballots.
A control case keeps a choice that differs from what ships resumable,
since a forfeit standing ships the consolidation over a `standing` or `neither` choice.

Not refused,
on the fixing agent's reasons:
`choice` against `ships`,
which is no coupling;
the presence of `decided`,
which the guard documents as unchecked;
and the floor against the terminal,
or its valid models against the verdicts,
which change neither what ships nor what the artifact says of the gate
and were not proved against every writer.

Open to the owner's veto:
the four refusals themselves,
of which the `rewrapped` and `archiveKept` ones carry the least.

Open when this entry was written,
closed in B249 (`df807a8b5`),
which also drives settlements of the real stage through the store and back:
a gate whose `usable` count is short of `CONSOLIDATE_GATE_QUORUM` is still resumed,
though the stage never persists one,
since the store's own comment documents a gate that heard nobody as valid
and the agent did not trace which cases rest on it.
No settlement written by the real stage was driven through the new rules end to end:
they rest on a reading of the writers and on unit round trips.

Recurrence:
`mistake-prevention.md`,
"Guards that cannot fail".

## B249: the consolidate store resumed a gate that heard fewer voices than its quorum

Red in `3d4558d9c`,
fixed in `df807a8b5`.

Found on 2026-10-06 (UTC),
closing the item B187 left open:
the store had never been driven against the real stage.
`isGateOutcomeOrAbsent` checked a gate's count against its ballots and never against `CONSOLIDATE_GATE_QUORUM`,
and the real stage returns `gate-kept-standing` with `usable: 0` when no gate voice answers.
The persistence rule never writes that settlement,
but the store resumed it.

The fix:
the store refuses a gate short of the quorum,
since a thin panel is the hour and not the slice.
A new file,
`corpus-run/consolidate-settlement-roundtrip.unit.test.ts`,
drives ten settlements of the real `settleConsolidation` through `persistConsolidationSettlement`
into an `openConsolidateCache` store and resumes each from a second one.
The cases are "REFUSES TO RESUME A GATE THAT HEARD TOO FEW VOICES,
the settlement the real stage returns when no gate voice answers and which the persistence rule never writes,
since a short gate is the hour and not the slice",
and "WRITES AND RESUMES WHOLE every settlement the real stage leaves worth keeping,
and writes none of the rest:
the store refused nothing the persistence rule wrote".

A second rule came with it,
for a state no input reaches today.
`consolidationWorthResuming` kept `slate-unjudged-standing` whatever its decision.
The agent's sweep of 160 inputs to `settleConsolidation` reached `no-voice-heard` in 12,
all of them a blank standing text beside a blank lane text,
which `laneTextsForSlate` never hands the stage.
Such a settlement is no longer kept,
since a later run could hear a voice.

Open:
the persisted variant of `wrap-erased-difference` is still proven only by the store's hand-built fixture,
since the real one in the table is kept as the archive has it.

Open to the owner's veto:
the guard for a settlement nobody was heard on.

Recurrence:
`mistake-prevention.md`,
"Guards that cannot fail".

## B297: a draw that could not finish left or removed a sheet that blocks the next draw

Red in `17a66fa66` for the empty damage pool,
fixed in `097b6ac40`,
with the other two cased red first by the agent in its worktree.

Found by the move B295 records.

- A final draw over a pool with no accepted issue wrote an empty gate sheet,
  which a final path's exclusive create then protects for ever,
  blocking the real draw.
  It is refused before any file is written.
- A failed final draw removed a file it had not created.
  The tracker recorded a path before its write,
  so a write that found the path taken had another draw's sheet removed.
  The tracker releases a path the exclusive create did not make.
- An empty damage pool wrote a kept,
  empty sheet and manifest and ended at exit 0,
  blocking the next run.
  It is refused before any client opens.

The whole suite's run on `097b6ac40` is recorded in B295.

Recurrence:
`mistake-prevention.md`,
"Files a later run reads back":
a cleanup removes only what its own run created,
and a write that nothing may replace is made only once there is something to write.

## B313: a row of a settled run file that was no object was read as a row

Red in `a9d447691`,
fixed in `8206fa5c6`.

Found on 2026-10-06 (UTC) by the agent moving `rendering-audit-settled-report`.
`readRunRows` checked that a run file's `rows` was a list and cast what it held,
so a row such as `7` or `null` passed:
it was keyed as `undefined`,
or ended a later reading in a `TypeError` printed as a fault.

The fix:
the reader refuses a row that is no object as stated,
naming its position and never its content;
the fallback in `sliceIndexOf` (`corpus-run/rendering-audit-settled-repeat.ts`) that answered for such a row,
which the coverage census listed as cold,
is an `unreachable:` throw saying why.

Seen by the agents of B309 to B313 and left,
to rule on or fix after `8206fa5c6`:
the verifier counts a page's length in UTF-16 units and calls them characters;
a request-operation `FidelityReferenceError`
(an unknown entry,
`--context`)
prints as a fault at exit 5;
the unpaired census ranks entries by unpaired source characters alone;
a suite that exits 0 with no pass prints a census of nothing.

Open to the owner's veto,
for the three clusters of `8206fa5c6`:
the fidelity probe reading its build digest and closure when the process starts,
as the settled audit always did,
where it read them after the corpus;
the buyer of one slice passed into the window trial's walk;
the five census steps passed as one object;
each new refusal's wording.
The whole suite on `8206fa5c6`,
run in a side checkout:
1,982 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and 41 source scans passing.

Recurrence:
`mistake-prevention.md`,
"Files a later run reads back":
a file's elements are checked where the file is read.

## B439: the consistency cache resumed a stored gather whose quorum no gather closes on

Red `f134107681`,
fixed in `a7a2a62c28`.

`isConsistencyRecord` tested only `Number.isInteger(quorum)`,
so a stored quorum of -1,
0 or 2 to the 60th resumed,
as did a record of fewer voices than its quorum.
The bench stores a gather only when its quorum is met,
and a quorum is at least one.
A resumed quorum of -1 was written into the group's verdict,
and the artifact reader then refused the run's own artifact.
A record now resumes only with an integer quorum of at least one and no fewer voices,
which also bounds it by what JSON carries exactly;
any other is asked again (the B249 shape).

## B448: the consistency pass would have asked again a gather of one voice it had stored itself

Red `f0f542a9c8`,
fixed in `85ed31f072`.

Found on 2026-10-12 (UTC) in reconciling B437 to B440 with B443 to B446 after both merged (M156).
B443's `recordFits` refused a stored gather whose quorum was below `min(2, bench size)`,
yet a gather closes on one voice when the seated bench has one or two seats,
the run having withheld the others (B420):
`reachableQuorum` applies its floor of two only to a short bench,
and a bench that is not short closes on `rosterQuorumSize`,
which is `max(1, ceil(n / 2))`.
The key names the whole bench,
so on resume the pass would have asked every seat again,
at paid calls,
for a gather it had stored.
Cases red at `f42d69c2fe` run the real gather for a bench of two,
and for a bench of three after the run withheld two seats,
and feed back what it stored;
both asked every seat again.
`recordFits` now checks only that each seat is heard once and sits on the bench;
the quorum and the ballot count stay with `isConsistencyRecord` (B439),
which every production read of the cache passes first.

The same work brought four test files to the merged rules:
the reader's merged refusal sentences,
the reason vocabulary grown by `markup`,
`overlaps-another` and `slot-whole`,
and the one-based `ConsistencyInvariantError` sentences;
the case that read a `failed` group with no place now refuses one of no place or one,
since `settleFailed` runs only for a group the bench was asked about,
and `dispositionOf` sends fewer than two places to a skip.
New cases pin the slot bounds,
two places of one slice said to read alike,
each refrain quote's slot,
and the death and names in the consistency key.
The region comment of `pass-consistency-apply.ts` now says
a replacement already left out as overlapping stays `overlapped`.

Kept by the lead under the owner's instruction of 2026-10-11:
`recordFits` states no quorum bound,
since the cache is built only through `openConsistencyCache`,
whose reader states them;
the reader keeps the slot-bounds check,
which refuses only what no writer writes.
Changed by the lead in `4002dc006e`:
the comments on `MIN_STAGE_VOICES` said no stage is decided by a single model,
whatever the bench;
they now say the floor is a short bench's,
and that a bench of one or two seats closes on one voice,
which the code has always done.

## B624: the artifact reader throws where the old locator found the slot once

Found on 2026-10-15 by the review of n159
(n160,
finding Y3),
the one finding of that review that can stop a run and not only leave a place uncarried.

Outcome written on 2026-10-15.
The lead ruled that nothing changes,
and n161 did not touch the reader;
no commit.

A stored rendering whose quote holds markup inside one shown word,
with a slot that is one side of it,
read as `once` before n159 and as `none` after:
`kit*tens* ate` with the slot `kit` or `tens`,
`the *kit*tens` with `kit`,
`a [kit](u)tens b` with `kit`,
`a_b_c` with `b`,
and `reload a save file<rt>OVERLOAD</rt>` with `OVERLOAD`.
The reader in `src/corpus-run/artifact-two-lane-read-consistency-span-located.ts`
calls `locateWords` on the quote with `apostrophes: 'outside'`
and throws `ArtifactParseError`
("a slot that stands exactly once in the words of its quote")
when the answer is not `once`.
The shown-edge test already ran inside that call (n156),
and n159 widened what it refuses (`cutsShownWord` in `src/consistency-shown-edge.ts`,
reached from `locateWords` in `src/consistency-fold.ts`),
so an artifact written under the old reading with such a slot,
or one written by the branch while the corpus pass runs,
may no longer read.
`src/consistency-count.ts` treats the same answer as no slot and degrades,
and the reader does not.
A reader of a page sees nothing,
since a stored artifact that fails to load stops the replay or the verify step that reads it.

Reach on the corpus:
not shown by the review.
The replay of `TianqiChen66623` read its artifacts without a throw
(`matched=1`,
`wordings=24`,
`missing=0`),
and the review did not read any other run's artifacts,
which its brief put out of bounds.

The lead ruled on 2026-10-15:
no change.
The throw guards the integrity of what is stored,
and both stored runs read clean on the n159 build
(`TianqiChen66624`:
`wordings=24`,
`chars=8490`,
`missing=0`,
with no throw),
so the finding reaches neither stored run.
The replays on the build of n161 gave the same:
`TianqiChen66623` with `chars=8607` and `read-at-generation=15`,
and `TianqiChen66624` with `chars=8490` and `read-at-generation=17`,
each with 24 wordings,
2 consistency groups and none missing.
n161 restored the `<rt>` shape of the list and its shorter twin `file<rt>OVERLOAD</rt>` (B617),
which went from `none` to `once`.

Accepted limit:
an artifact whose slot a later reading of the word test refuses fails to load,
and the artifacts of runs other than the two replayed are unchecked.
