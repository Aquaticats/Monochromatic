# Audit ledger: NeuralWatt's day start is never waited for

The wait for NeuralWatt's day to start again and the park of the pass between entries,
both removed on the owner's answer of 2026-10-29 (B1005).

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
The mid-page dry stop that the owner settled in the same round is not part of this entry and is not built by it.
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
