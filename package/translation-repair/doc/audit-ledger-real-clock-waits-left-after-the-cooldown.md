# Audit ledger: real-clock waits left in tests after the cooldown round

Three waits on the real clock that the cooldown round of B965 left in unit tests,
and how each was replaced by an event or a fake clock (B985),
then the two 40 ms sleeps that round left open and the three forms of wait the real-clock scan could not see (B995).

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
B995 closed the gap on 2026-10-29 (UTC).

Left open.
In the same provider file the case "KEEPS THE DISTANCE ON THE REFUSAL AFTER WAITING OUT ANOTHER HOLD"
(`hyperCooldownMs: 40`,
`pollMs: 1`,
no clock) still sleeps 40 ms of real time.
The scan lists its key `cooldownMs: hyperCooldownMs` as a bound the case reaches,
and the sibling case of `pass-dry-stop.unit.test.ts` does the same with `cooldownMs: 40`.
The round's brief named three waits,
so these were not converted.
B995 converted both cases on 2026-10-29 (UTC).

Prevention rule:
a wait in a test is the event it stands for,
see "Tests on the real clock" in
[Preventing mistakes: the real world, clocks and arrival order](mistake-prevention-tests-clocks-and-arrival-order.md#tests-on-the-real-clock).

Recurrence:
the family of B318 (a test whose verdict a bound on the real clock decides).

## B995: two 40 ms sleeps behind a scan that could not see polls, elapsed readings or clockless budgets

Status:
fixed on 2026-10-29 (UTC),
scan `dd0cdc0cf9` (the widening,
red on purpose)
and `232e4da794` (the list),
tests `a122eb68e2` (provider),
`b5622dabda` (corpus-pass)
and `b0f9212a4e` (run),
cache versions none.
One file of the provider package is deferred (see "Left open").
A member of the family of B318, B817 and B985:
a test that waits on the real clock pays the wait in every gate and lets host load decide what it reads.
The brief for the round named gate g228 of 2026-10-29,
in which eleven cases went red while two sweeps loaded the host and passed alone.
The gate's log does not say which cases those were,
and the two cases below can only be lengthened by load, not turned red by it,
so this entry does not claim they were among the eleven.

What the two sleeps were for.
Both cases build the real budgets over stub meters, with NeuralWatt's spent day 600 s from starting again
(past the router's own 300 s) and Hyper's meter dry,
refuse Hyper once so that it is held out for a 40 ms cooldown,
and hand the budgets to `readBudgetsPastHolds` with `pollMs` 1.
The router's call then sleeps the 40 ms of Hyper's hold in slices of a millisecond (`waitOutHold`),
reads the budgets again,
finds no hold left that it may wait for,
and refuses with NeuralWatt's distance on the refusal.
The sleep lets the hold run out so that the refusal is made on the path of a call that waited,
which is the path where the distance was once dropped (R1 of `budget-hold-wait.ts`).

- Provider, "KEEPS THE DISTANCE ON THE REFUSAL AFTER WAITING OUT ANOTHER HOLD".
  Its assertion on the message, `after waiting`, depended on the sleep:
  the words are written only on the path of a call that waited,
  so a case whose Hyper hold were not registered would have failed on it.
  The two bounds on `neuralwattReturnsInMs` (above 590,000 and at most 600,000) did not depend on it.
  The wall clock of the case is a stub that does not move,
  and `neuralWattDrynessOf` takes the distance from that stub at the reading,
  so the distance is 600,000 on any monotonic clock (read from the code, not printed).
  By the code, load could not turn the case red:
  it only lengthens the slices, and a timer that fires a millisecond early leaves a hold of a millisecond
  that the loop sleeps again.
- Corpus-pass, "PARKS ON THE REFUSAL THE ROUTER MAKES AFTER WAITING OUT ANOTHER HOLD".
  Its assertions were on the park: the gate is not tripped and is parked.
  None of them depended on the sleep.
  With Hyper held for no time the router refuses at once,
  and a refusal made without waiting also carries the distance,
  so by the code the park assertions would read the same
  (read from `dryCauseOf`, not run on the old form).
  The case could pass without reaching the path it is named for.
  One assertion is added: the message contains `after waiting`.

What the scan could not see.
Its module doc left out three things, and each is now a read:

- `pollMs`, the slice a wait on a hold sleeps in, is a bound key,
  except inside a function that puts a fake clock under the case:
  one that calls `useFakeTimers`,
  one that calls a function of its own file that does (`fakeClockStarted`),
  or one that calls a name imported from a module whose path holds `fake-clock` (`readOnFakeClock`).
- Every call of `performance.now` in a test is a finding.
- A `createProviderBudgets` call in a file that imports `readBudgetsPastHolds` or `waitOutHold`,
  whose options literal names no `now` key of its own, is a finding.
  A spread does not count as naming it,
  so the helper `budgetsBefore`, which spread `now` in only where a case passed a clock, was found.

Out of reach:
a wait reached through a router built over the budgets,
a fake clock installed at module level,
and a helper that installs a fake clock through a variable rather than a function declaration of its own file.

What the red scan found.
The scan commit left the listed keys alone and the package-wide case failed with 85 findings;
the fixture cases, the new one with its planted samples among them, passed.
By form:
2 budgets with no `now`,
58 `pollMs`,
25 `performance.now`.
By disposition:

- 18 fixed by making the case deterministic:
  the two cases above (2 findings in `pass-dry-stop.unit.test.ts`, 2 in the provider file),
  and 14 in four files of the run package
  (`run-seats-hold-need`: 1,
  `run-seats-neuralwatt-return`: 5,
  `run-seats-pass-bound`: 1,
  `run-seats-read`: 7).
  These cases slept holds of 40 to 301 ms on the real clock;
  each now runs through `readOnFakeClock`,
  which ticks past the hold and aborts a reading that is still waiting, so a stuck wait fails and does not hang.
- 48 listed with a reason:
  23 `performance.now` spans of seven files that measure a sleep they make on purpose or a bound well above the work,
  25 `pollMs` slices of readings that refuse or seat at their first reading and sleep nothing,
  of which one is a child script's poll of a marker file.
- 19 deferred in `provider/budget-hold-wait.unit.test.ts` (17 `pollMs`, 2 `performance.now`).

What replaced the sleeps.

- Provider.
  `fakeClockStarted`, the fake clock of B985, is handed to the budgets as the monotonic clock,
  the router's call is started unawaited and the case ticks 41 ms.
  `budgetsBefore` now requires its clock and always passes `now`,
  so no caller can build budgets on the real monotonic clock;
  the case above was the only one that omitted it.
- Corpus-pass.
  The case installs the same kind of fake clock on its sandbox (`setTimeout` and `clearTimeout` only, native timers
  cleared), passes `now` to the budgets, starts the call, ticks 41 ms, aborts the call and ticks once more.
  The abort turns a wait that outlives the hold into a failure and not a hang
  (a tick of 38 ms ends the case with `This operation was aborted`).
  The top describe of the file is now `concurrency: 1`,
  since the case fakes the process-wide timer function.
- Run.
  The shared `readOnFakeClock` starts its clock at 1,000 ms and hands the reading the clock as `now`,
  so budgets the reading builds count a return on it,
  and no longer lets a reading that rejects during a tick surface as an unhandled rejection.
  Two helpers in `run-seats-neuralwatt-return.unit.test.ts` carry its five sleeping cases.
  Three suites of `run-seats-read.unit.test.ts` that held sleeping cases moved from the default concurrency to 1.

The red demonstrations,
each a temporary local change that was restored and never committed.

- Scan: the red commit above.
- Provider, Hyper's cooldown set to 0 (the refusal no longer holds it):
  `expected 'Every provider is out of budget at on…' to include 'after waiting'`.
- Provider, the account still showing a fresh day after the wait (`staysSpent` false):
  `Expected the call to refuse, but it returned`.
- Corpus-pass, `cooldownMs` 0:
  `expected 'Every provider is out of budget at on…' to include 'after waiting'`.
- Corpus-pass, the pass left only 300,000 ms against a return 600,000 ms away:
  `expected true to equal false` (the gate is tripped and not parked).
- Corpus-pass, a tick of 38 ms:
  `expected 'This operation was aborted' to include 'after waiting'`, after the abort.
- Run, the tick of the control case of `run-seats-pass-bound` cut to 10 ms:
  `expected DOMException{ stack: 'AbortError: Th…' } to be an instance of WritingBenchUnreachableError`.
- Run, the tick of a `run-seats-neuralwatt-return` case cut to 10 ms:
  the case fails with `AbortError: This operation was aborted`.

Left open.
`provider/budget-hold-wait.unit.test.ts` is in `DEFERRED` of the scan.
Its cases that wait out a hold sleep it on the real clock in slices of 1 or 2 ms,
and the case of `waitOutHold` itself measures its sleep with `performance.now`.
They need a fake clock the case ticks,
which the provider package does not yet have as a fixture
(`readOnFakeClock` lives in the run package, which the provider package cannot import).
The deferral carries the three keys the file held before.
Cases that the listed keys describe as sleeping nothing are described from the case's scripted holds;
a case that comes to register a hold the reading waits for would raise the count of its key and fail the scan.

Prevention rule:
a wait in a test is the event it stands for,
see "Tests on the real clock" in
[Preventing mistakes: the real world, clocks and arrival order](mistake-prevention-tests-clocks-and-arrival-order.md#tests-on-the-real-clock).

Recurrence:
the family of B318 (a test whose verdict a bound on the real clock decides),
and of B985.
