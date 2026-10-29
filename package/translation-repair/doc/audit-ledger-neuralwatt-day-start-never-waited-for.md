# Audit ledger: NeuralWatt's day start is never waited for

The wait for NeuralWatt's day to start again and the park of the pass between entries,
both removed on the owner's answer of 2026-10-29 (B1005),
and the interim page a mid-page dry stop ships on his second answer of the same day (B1006 to B1008).

The entry sits in a file of its own because the file of B826,
[providers and seats a run holds out](audit-ledger-providers-and-seats-a-run-holds-out.md),
stands at the line limit of a Markdown doc.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B1005: a run waited for NeuralWatt's day to start again, and the owner chose to stop at once and relaunch

Status:
fixed on 2026-10-29 (UTC),
tests `decfb6cf75` (provider),
`845884c447` (run) and `c1dd63224a` (corpus-pass),
committed red,
fix `702d149a74` across provider,
run,
corpus-pass,
slice-cache,
stage and the source scans,
cache versions none (the reason is under "Cache versions" in this entry).
The mid-page dry stop that the owner settled in the same round is not part of this entry and is not built by it;
B1006 builds it.
His answer was "Ship interim,
resumable":
"Ship settled slices,
plus archive text or the source text under its note elsewhere,
and keep the entry open so a later run replaces it
(as you ruled on 2026-10-23 for source-text slices)."

The question put to the owner on 2026-10-29 was this.
NeuralWatt gives 550 Wh of free energy a day,
restarting at 19:15:08 UTC;
when it is spent and the other routes are empty,
should a run wait for it?
In force was a bounded wait:
one router call waited at most 300 s in total for a return,
the seat reader waited for a return that fitted the time the pass and the entry had left,
and between entries the pass parked,
logged `PARKED` and resumed.
The owner chose **"Never wait,
relaunch"**:
"Stop at once and relaunch later."
The context the owner had was his own ruling of 2026-10-22,
"You don't need to align launches to NeuralWatt starts",
and that the NeuralWatt subscription reads `canceling`,
its period ending 2027-01-20 19:15:08 UTC.

What the answer does not touch:
the holds of other providers (a stated `Retry-After`,
a refusal cooldown,
a daily limit in a reply) are waited for as before,
and so is a refusal cooldown on NeuralWatt itself.
The dry stop still ends the run only when no funded provider serves a writing seat ("Stop when no writer left",
reaffirmed the same day),
and a provider that refuses right after a waited-out cooldown is still given up for the entry.

What was removed:

- The budget layer's hold for the day start,
  so `holds()` carries refusal holds alone,
  with the `returnBoundMs` argument,
  `neuralwattHeldByReturn`,
  `NEURALWATT_RETURN_WAIT_BOUND_MS`,
  the bound arithmetic of `readAfterHolds`,
  and the `neuralwatt-return-fit.ts` module (`returnFitsWithin`,
  `pastPassWordsOf`,
  `BoundOwner`).
- The seat reader's pass bound:
  `run-seats-pass-bound.ts` (`holdsWithinPass`,
  `passBoundOf`),
  the client's `passLeftMs` and `passBoundOwner`,
  and `withEntryBound` in `pass-entry-bound.ts`.
- The return the writing bench error carried
  (`neuralwattReturnsInMs`,
  `neuralwattReturnsAt`,
  `neuralwattReadAtMs`,
  `neuralwattDryness`),
  `returnLiftsWritingBenches`,
  and the same fields of the all-dry refusal.
- The park:
  the gate's `parks`,
  `endedParks`,
  `parked`,
  `parkedFor`,
  `awaitPark`,
  `recordOnPark` and the signal swap,
  the queue's `beforeNext` and `interrupted` hooks and the `NOT RESUMED` line,
  `ParkAnswer`,
  `ReturnTimeNotUtcError`,
  the `PARKED` status of the `TALLY` line,
  and `parksTheRun` in the stage package.

What stays,
and why:
the NeuralWatt meter,
its day start and the return mark.
The refusal that ends the run states the time of day in UTC and the distance counted down on the monotonic clock
(`at 19:15:08 UTC (in 5 min 1 s)`),
so the operator knows when to relaunch,
and the mark still tells the budget layer that its cached dry reading is stale at the day start.
Nothing waits on the mark.

Cache versions:
none moved.
No cached record changed meaning:
the change is in what the router,
the seat reader and the pass do with a dry NeuralWatt,
and in the words of a refusal and of two log lines,
none of which a cache file stores.

Words that changed in the logs:
a `TALLY` line never says `PARKED`,
`NOT RESUMED` and `the pass resumes` are not written,
the seat reader's no-wait line says "NeuralWatt's day return is not waited for" in place of
"no NeuralWatt return inside what the pass has left",
and the stop line carries the refusal's own message.

## B1006: a mid-page dry stop shipped no page for its entry, and the owner chose an interim page and an open entry

Status:
fixed on 2026-10-29 (UTC),
tests `65ae605fd2` (corpus-pass) and `c88d403d52` (run,
with the `interimEntry` fixture of page-check),
committed red,
fix `1f2bb6aef6` across glossary,
page-check,
corpus-pass and run,
with `c812c8fb92` for the findings of the source scans,
cache versions none (the reason is under "Cache versions" in this entry),
artifact generation unchanged (no field or kind is added to the artifact).

The question put to the owner on 2026-10-29 was this.
When every funded route goes dry in the middle of a page,
the run stops,
and the entry in flight has some slices settled and some not.
In force:
the entry stayed `INCOMPLETE`,
stopped and resumable,
earlier entries shipped,
and the stopped entry shipped no page.
The owner chose **"Ship interim,
resumable"**:
"Ship settled slices,
plus archive text or the source text under its note elsewhere,
and keep the entry open so a later run replaces it
(as you ruled on 2026-10-23 for source-text slices)."
The rules that still hold are the owner's:
a run always ships ("no matter what,
we must ship"),
"No valid wording:
keep the archive at that slice and ship" (2026-09-27),
the source-text note "This passage has not been translated yet.
The original text follows."
for a slice with no archive and no backed wording (2026-10-23),
and the dry stop ends the run only when no funded provider serves a writing seat
("Stop when no writer left",
reaffirmed 2026-10-29).

The lead's reading of "settled slice",
recorded here for the owner's veto:
a slice counts as settled when it has a consolidation record.
A stop inside the lanes phase,
before any contest or consolidation record exists,
therefore ships every slice as archive text,
or as the original under the note where the archive has none.

The design,
and the readers it touches.
The artifact is the done sentinel:
`pass-entry.ts` publishes the page before it writes the artifact on purpose,
the skip set (`artifactBackedIds`,
`finishedEntryIds`),
the order of the pending entries,
the republish (`republishRunPages` through `settledEntryIds`) and the census all read `artifacts/`.
Two designs were weighed.
A field `interim` in the artifact would put the entry in every one of those readers as "not done",
change the artifact shape and need a generation bump.
A record outside `artifacts/` touches only the writer,
the retirement after the artifact is written,
and `verify-published`,
which lists the published tree against the artifacts.
The second was chosen,
as the decline record (`declined/`) was before it:
`interim/<id>.json` beside `artifacts/`,
named by `INTERIM_DIR` in the glossary package.
Untouched:
the artifact parser and writer,
`artifactBackedIds`,
`finishedEntryIds`,
`entriesFinishedThisRun`,
`orderPendingEntries` (an entry with no artifact is pending by construction),
and `republishRunPages`,
which iterates the settled ids alone and so never rewrites or promotes an interim page.
Touched:
the entry's `catch` in `pass-entry.ts`,
the layout (`interimDir` in `RunsLayout`),
the settle chain that carries the directory,
and `verify-published-run.ts`.

How the page is made.
`runEntryPipeline` keeps the slicing,
the lanes' rows,
the contest's records and each consolidation record as the stages finish them (`pass-entry-interim.ts`),
since the locals of its `try` are out of reach of its `catch`
and the overlapped map discards every finished slice when a later one fails.
The consolidation hands each record to a sink the moment it is made (`onSliceSettled`).
When the stop is the every-provider-dry refusal and the slicing exists,
`publishInterimPage` (`interim-page.ts` in page-check) composes the page through the reading every page comes from:
a settled artifact's source with the slices no consolidation record settled listed as unsettled,
which `would-ship-text.ts` reads as the archive's text and,
through `carryingSource`,
as the original under the note where the archive has none.
There is no second assembler;
the page-level guard,
`publishFixedPage` and every content check run over it as over any page.
A stop before the lanes returned reads each slice from the prepared chunk alone,
as a row no lane reached.
The record is written after the page,
carrying the counts of settled,
archive and original slices,
the reason `dry-stop` and the SHA-256 of the page.
The entry logs `INTERIM <id> at=<time> attempt=<n> settled=<n> archive=<n> source=<n> reason=dry-stop`,
and the `TALLY <id> status=INCOMPLETE` line is printed as before.
A later run repeats the entry on its caches and overwrites the page;
a run that settles it removes the record after the artifact is written,
and a removal that fails is logged and does not turn the settled entry into an `ERROR`.
A stop before the entry is sliced ships no page.
A failure to ship the page is logged and the stop stands as it was.

`verify-published` names a page with no artifact and an interim record `INTERIM PAGE`,
with the counts,
and `INTERIM PAGE DISAGREES WITH ITS RECORD` when the digest of the page on disk is not the record's.

Cache versions:
none moved.
The sink only observes a record the consolidation already makes,
and neither the interim page nor its record is stored in any cache.
`cache-account-audit` read 427 runs directories of slice-cache records,
and no version's account names a change of this round.

Tests:
the pass-entry cases script the stop in the lanes,
in the consolidation of the last slice (one settled,
one archive),
once the lanes returned with an archive that lacks a paragraph,
over footnotes across a settled and an unsettled slice,
and in a later run that completes the entry;
the settled run retires the record,
the republish leaves the page alone and the entry is not among the finished ids.
An archive with no English at all is sliced into no slice,
so it settles as an empty page (`slices=0`) and has no interim page to ship.

## B1007: the interim page discards the bought work of slices no consolidation record settled

Status:
open,
a consequence of the lead's reading in B1006 that the owner may veto.

A slice the two lanes worded alike has no contest and no consolidation record by design,
and a contested slice the stop reached between its contest and its consolidation has none yet.
Under the reading of B1006 both ship as the archive text,
or as the original under the note where the archive has none,
though a wording both lanes agreed on,
or one the contest endorsed,
is in hand.
That work is not lost:
it stays in the slice caches and the next run uses it.
The page of the stopped run is poorer for it.
The reading is one predicate:
`interimSource` in `interim-page.ts` marks a slice unsettled when its index has no consolidation record,
and counting a lanes-agreed slice or a contest-decided one as settled is a change to that line,
since the reading of a page already gives those slices their wording.

Also open:
a later stop of the same entry on a different roster can re-buy slices under new keys
and ship an interim page with fewer settled slices than the page it replaces,
since the later page replaces the earlier whatever it holds.
Nothing guards it today.

## B1008: `PASSED_SLACK_MS` justified its 2 ms by timers firing early, and nothing arms a timer for the return

Status:
fixed on 2026-10-29 (UTC),
`9aa25312d8` (provider),
a comment change only.

The margin stayed after the timer was removed with the return wait (B1005).
Its TSDoc now says what it guards:
the budget layer asks `passed()` before it serves the cached dry reading,
and the margin makes a call inside the last hair before the stated return read the meters again.
The value is the one the timer measure of 2026-10-22 gave,
kept as a small margin and not re-derived.
