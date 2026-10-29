# Audit ledger: real-clock waits left in tests after the cooldown round

Three waits on the real clock that the cooldown round of B965 left in unit tests,
and how each was replaced by an event or a fake clock.

The entry sits in a file of its own because the file of B965,
[providers and seats a run holds out](audit-ledger-providers-and-seats-a-run-holds-out.md),
stands at the line limit of a Markdown doc.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B985: three tests still waited on the real clock, and the scan listed one of them as a reason

Status:
fixed on 2026-10-29 (UTC),
tests `be445c4aa6`
(corpus-pass,
with the scan's list in `source-scan`)
and `6c4e41e410`
(provider),
cache versions none.
A member of the family of B318 and B817:
a test that waits on the real clock pays the wait in every gate and lets host load decide what it reads.
The brief for the round named gate g215,
in which a case of this family went red under load and green alone.

Where the waits stood:

- `package/translation-repair/corpus-pass/src/corpus-run/pass-entry.unit.test.ts`,
  `entryClient`,
  `await wait(10,);`.
- `package/translation-repair/provider/src/provider-budget-neuralwatt-return.unit.test.ts`,
  the case "ENDS THE RUN WITH THE DAY START STATED",
  `pollMs: 10` handed to `refusalAfterHolds`,
  on budgets with no injected monotonic clock and a wall clock driven by `performance.now()`.
- The same file,
  the case "LETS THE ROUTER'S WAIT SEE THE RETURN",
  `pollMs: 10` handed to `readBudgetsPastHolds`,
  on budgets with no injected monotonic clock.

What the first wait was for:
the scripted client counts the calls in flight at each of five stages
(repair,
refine,
translate,
contest,
consolidate),
and a call returned at once would be counted in and out within one turn.
The ten milliseconds kept each call in flight so that the calls of a second slice,
started beside the first by the overlap driver,
were counted with it.
The case's assertion depended on the wait:
without a hold every peak reads one slice's worth,
and `overlapped.peak` is not above `serial.peak`.
Measured on the parent with a temporary print,
the sequential arm peaked at 5,
3,
5,
5
and 6 calls
(the calls of one slice,
which are several voices at once)
and the overlapped arm at 10,
6,
10,
10
and 12.
So a count of calls cannot tell a second slice from the voices of the first,
and the content of a call cannot either:
the repair critic's sheet names both slices.
How load could change the case:
if the second slice reaches the client after any real file work,
a window of ten milliseconds is a race between that work and a timer,
and a window the work outran would read the peaks equal and fail the case on a loaded machine.
Whether the second slice does reach the client after such work was not measured,
and no failure of this case under load was observed.
The scan listed the site with the reason that the overlap is in place within one turn of the event loop,
from the census of 2026-10-06 (UTC).
That reason was not re-tested,
and it was not needed after the change.

What the other two waits were for:
both sleep the 70 ms to NeuralWatt's return (50 to the day start and 20 of margin) in slices of `pollMs`
through `waitOutHold`,
whose sleep is the global `setTimeout`,
and then read the meter again.
The first case needs the wall clock to pass the day start during the wait,
so that the second reading names the day after;
the second needs the monotonic clock to pass the return,
so that the reading is not served from the cache (`reads.count` is 2).
Both therefore wait for time to pass and assert what the passing of that time brought.
Host load only lengthens a real sleep,
so neither assertion fails because of it;
the cost was 70 ms of real time each and a dependence on the slack of Node's timers.
No red under load was found for these two.

What replaced them:

- Pass entry.
  A call of the overlapped arm holds at a rendezvous until more calls are in flight at its stage than the sequential arm's
  peak for that stage,
  which only a second slice can cause,
  and the rendezvous then stays open.
  The hold ends on the request's abort,
  so a rendezvous never reached fails at the entry's hard cap.
  The sequential arm yields one event-loop turn and holds nothing else.
  No timer decides the overlapped arm.
  The assertions are unchanged.
  Load can only lower the sequential peak,
  which lowers the threshold,
  so it cannot turn the case red.
- Provider.
  Both cases install a fake clock on the case's sandbox (`fakeClockStarted`,
  `setTimeout` and `clearTimeout` only,
  native timers cleared),
  hand it to the budgets as the monotonic clock,
  drive the wall clock of the first case from it,
  start the call unawaited and tick past the hold.
  The assertions are unchanged.
  The cache window of these cases (`RETURN_CACHE_WINDOW` in the real-clock scan) was said to be on a clock the case
  moves by hand;
  that is true of these two cases only since this change.

The red demonstrations,
each a temporary local change that was restored and never committed.
The sources of the three production behaviours were not edited,
so each break is in the test's own rig and stands for the production break named.

- Pass entry,
  the overlapped arm handed `overlap: 1` beside the rendezvous
  (a driver that ignored the overlap):
  no second slice arrives,
  the entry's 60 s hard cap aborts the held calls,
  and the case fails with `expected 5 to be above 5` after 60.6 s,
  the hang stop doing its work.
- Pass entry,
  the same `overlap: 1` with the overlapped arm built without the rendezvous:
  the case fails in 1.3 s with `expected 5 to be above 5`.
- Provider,
  the day-start case with the meter answering the fresh day once the fake clock has moved
  (the account no longer reading the day spent):
  `Expected the call to refuse, but it returned`.
- Provider,
  the router-wait case with `staysSpent: true`
  (the account still reading the day spent after the return):
  the read rejects with `EveryProviderDryError` instead of returning a view.

The scan finding.
The pass-entry wait was listed in `LISTED` of
`package/translation-repair/source-scan/src/real-clock-bounds.unit.test.ts`
under the key `corpus-pass/corpus-run/pass-entry.unit.test.ts#entryClient: wait of module-async-time 10`;
the key is removed in the same commit as the test change.
The two provider sites were never in reach of the scan:
its module doc leaves out `pollMs` and an elapsed time read with `performance.now`,
and it has no way to see budgets built without an injected `now`,
which then run on the real monotonic clock.
That is a gap in the scan,
recorded here and not changed in this round:
a test that builds `createProviderBudgets` (or the helper `budgetsBefore`) with no `now`
and then drives a wait through `readBudgetsPastHolds` reaches a real sleep that no listed key names.

Left open.
In the same provider file the case "KEEPS THE DISTANCE ON THE REFUSAL AFTER WAITING OUT ANOTHER HOLD"
(`hyperCooldownMs: 40`,
`pollMs: 1`,
no clock) still sleeps 40 ms of real time.
The scan lists its key `cooldownMs: hyperCooldownMs` as a bound the case reaches,
and the sibling case of `pass-dry-stop.unit.test.ts` does the same with `cooldownMs: 40`.
The round's brief named three waits,
so these were not converted.

Prevention rule:
a wait in a test is the event it stands for,
see "Tests on the real clock" in
[Preventing mistakes: the real world, clocks and arrival order](mistake-prevention-tests-clocks-and-arrival-order.md#tests-on-the-real-clock).

Recurrence:
the family of B318 (a test whose verdict a bound on the real clock decides).
