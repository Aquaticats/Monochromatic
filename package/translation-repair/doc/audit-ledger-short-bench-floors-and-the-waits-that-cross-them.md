# Audit ledger: short bench floors and the waits that cross them

The ceiling a hold could lift a short bench to,
read from the meters and not from the holds,
the wait judged against the floor and the quorum of each bench,
the translate lane's first-time buys on a select bench under its floor,
the artifact's record of why a slice was kept,
and the wait for a provider whose return had served nothing (B1010),
whose replay,
the memory of skipped slices it removed and the cache account are in
[futile returns replayed and the memory of skipped slices removed][futile-ledger].
These continue [short benches and the last live seat](audit-ledger-short-benches-and-the-last-live-seat.md),
whose B833 to B835 they correct.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B837: the ceiling counted a provider its meter keeps dry as coming back

Status:
fixed on 2026-10-23 (UTC),
found by the re-check of B833,
red `4cf385e4b3`,
fix `39863c9691`.
The three branches of the ceiling that had no case
(a bench the return adds no seat to,
the digits of the translators clause,
and a direct call of `ceilingWordsOf`)
have their cases in `cc3b7dd3e7`,
each shown to fail by a scratch mutation of the code.

What the re-check found:
the wait and the ceiling of B833 read the dryness view,
which folds a hold into the meter's reading.
A provider that is held by a refusal and also dry by its meter looked the same as one whose hold alone keeps it out,
so the ceiling counted its seats as coming back when the hold ended,
and the wait was taken for a return that the end of the hold does not bring.

What changed:
`meterDryOf` (`run/src/corpus-run/run-seats-hold-need.ts`) reads the meter states with `routesAsDry`,
and `returningProvidersOf` leaves out a held provider whose meter reads dry.
`holdsThatCureShortBenches` and `ceilingWordsOf` both take that set,
so the wait and the ceiling agree.
`readMeterDry` in `run-seats-short-bench.ts` reads the meters whenever a hold runs and again after each wait.
NeuralWatt is the exception while its hold is its day start,
because the day starting again is what turns its meter wet.
A bench to which the return adds no seat says `the return brings back no seat`.

Cases:
`run-seats-ceiling.unit.test.ts` holds the red case (NeuralWatt held by a refusal with a dry meter,
translators 2 of 8,
no wait and no seat brought back),
its counterpart (a wet or unreadable meter,
or a day start,
waits),
and the three cases of `cc3b7dd3e7`.
The scratch mutations are recorded outside the repository:
dropping the no-seat wording,
reading the seats before the return instead of after it,
and reversing the order of the clauses each fail the named cases.

The rule for next time:
a view that folds two causes together cannot say which of them a return ends,
so the question that depends on the cause reads the cause itself.

Cache account:
no sheet or schema changed,
so no cache version moved.

Later,
the same day:
a refusal for payment is a second cause the states did not carry.
Reading the fix showed that a provider refused for payment (HTTP 402) is dry "until that meter moves" however
long the hold the refusal stated,
and however wet its meter reads (`provider-budget.ts`,
`paidOut`).
`readStates` reported the meter alone,
so `returningProvidersOf` counted such a provider as returning when its stated hold ended,
the reader waited the hold,
and the provider was still dry.
`readStates` now folds the payment mark into the states it reports:
a marked provider reads `dry` until its meter moves,
as the routed view already read it,
and the reading a later refusal is judged against (`markRefused`) still holds what the meter said,
so a second refusal on a wet meter is a rate limit and not an exhaustion.
Red `520fe704fa` (provider states) and `25a302ca06` (the reader),
fix `91c00da3d8`.
The readers of the meter states,
counted in the code:
- `readMeterStates` in `run-seats-states.ts`,
  reached from `readJudgeSeats` (the `JUDGE SEATS` line) and from
  `readMeterDry` in `run-seats-short-bench.ts`.
  The line prints the routed bit before the meter state,
  so a payment-refused provider printed `dry` before and
  prints `dry` now.
  `readMeterDry` feeds `meterDryOf`,
  and through it `returningProvidersOf`,
  `holdsThatCureShortBenches`,
  `ceilingWordsOf` and `seatWithoutWaiting`:
  a payment-refused provider now leaves the returning set,
  so the reader takes no wait for its stated hold,
  and the ceiling clause said the provider "reads dry by its meter",
  which was false for this provider until fix round 4 (its second correction is the clause's new wording).
- `providerMeterStates` in `run-config.ts` hands `readStates` to the run client,
  and the budget cases,
  `run-providers-held.unit.test.ts` and the fixtures that call `readStates` read the same record.
- `meter-sample-read.ts`,
  `meter-report` and `meter-dry-span.ts` read the `METERS` line of a log,
  which already printed a payment-refused provider as `dry`,
  and are unchanged.
- The router and `read` read the routed view,
  not the states,
  and are unchanged.
No cached record holds the states and no cache key covers them,
so no cache version moved.

Later,
fix round 4:
two corrections to the payment-refusal follow-up of this entry.
The control of `run-seats-payment-hold.unit.test.ts`,
a refusal that is not for payment,
waited out a hold of a minute on the real clock,
the rate-limit backoff,
so the suite `the seat reader and a provider refused for payment` took 62.7 s in the run package's `test:unit`,
and the raise of the stated wait to 250 ms that fix round 3 made did not shorten it.
The payment case could also pass without observing anything when a stalled machine let its 250 ms hold end
before the reader first asked for the holds.
Both cases now run on a fake clock (timers and `performance`,
which the budgets count holds down on) and state a wait of 90 s:
each reads the hold running (equal to the stated wait) before the reading starts,
then ticks the clock past the hold's end while the reading runs,
so a reader that waits for the hold reads twice in either case.
The suite takes 91 ms (`b0953ecf75`).
A scratch copy that sent the payment case a refusal without the payment flag failed with `expected 2 to equal 1`.
The second correction is the ceiling clause:
it said a held provider "reads dry by its meter",
which is false for a provider refused for payment whose meter reads wet,
and now says "by its meter or by a payment refusal that stands until the meter moves,
which a hold ending does not change".
The payment case reads the warning of a real payment refusal on a wet Hyper meter and pins the words,
beside the two cases of `run-seats-ceiling.unit.test.ts` that name a meter-dry NeuralWatt;
all three failed before the wording (`a301305cac`,
fix `bdea8b354e`).
No cached record holds the clause and no key covers it,
so no cache version moved.

Later,
fix round 5:
the cases of `run-seats-cure.unit.test.ts` and `run-seats-ceiling.unit.test.ts` scripted holds of 40 ms
and let the reader wait them out on the real clock,
in slices of 5 ms.
The listing of `real-clock-bounds.unit.test.ts` cannot carry them:
its scan reads holds of 1000 ms or more,
and an entry for a hold shorter than that is reported as a listed key the source no longer holds.
Both files now read the seats through `run-seats-fake-clock.test-fixture.ts`,
which fakes `setTimeout` and `clearTimeout`,
moves the clock past one hold and one poll,
and aborts the reading,
so a reading still waiting afterwards fails instead of hanging.
The two suites took 677 ms and 834 ms in the run package's `test:unit` before and 158 ms and 178 ms after
(`ac27c68dc6`);
those are times of the suites,
and the paragraph of fix round 6 gives the times of the processes.
A scratch copy of the fixture that moved the clock by 1 ms failed every waiting case of both suites.
No cached record holds a hold and no key covers one,
so no cache version moved.

Later,
fix round 6:
the fake clock of that fixture was installed after the module logger had begun its start check,
a native timer of five seconds,
and the fake `clearTimeout` refuses a timer it did not make:
it wrote `FakeTimers: clearTimeout was invoked to clear a native timer` to stderr
and left the timer holding the process.
The suites finished in milliseconds and each file's process stayed alive for five seconds after them.
The process wall of the whole `mise run` command of each file,
three runs each,
had a median of 6.12 s for the cure file (5.95 to 7.27 s)
and 5.94 s for the ceiling file (5.93 to 5.95 s),
each run printing one `FakeTimers` line,
while the suites themselves took 70 to 155 ms and 77 to 101 ms in the same runs.
The fixture now passes `shouldClearNativeTimers: true`,
which hands a clear of a native timer to the real `clearTimeout` (`c1d7e4db8d`).
The same three runs afterwards had a median of 1.18 s for both files
(1.18 to 1.22 s for the cure file and 1.16 to 1.19 s for the ceiling file),
with no `FakeTimers` line in any run,
and the suites took 85 to 96 ms and 89 to 92 ms,
unchanged within their spread.
The whole unit run of the run package prints no `FakeTimers` line either.
Three more installs of a fake clock in the family carried the same omission:
the probe client suite of the run package,
the 300-second hold case of the corpus-pass package
and the fake-clock walk of the test-fixture package,
which the two bench suites that walk it use.
The scan `fake-timer-native-clear.unit.test.ts` of the source-scan package now fails,
naming file and line,
on any `useFakeTimers` call that fakes `clearTimeout` without that option (`58b3bfe200`),
and read all four installs at its red.
A clock that fakes the date alone is left alone by rule.
What to do about the family is in the section "Fake clocks that leave native timers" of
[the fake-clock record of mistake prevention][clocks-family].
No cached record holds a clock and no key covers one,
so no cache version moved.

Later,
fix round 7:
the scan read an empty `toFake` list as a clock that fakes the date alone and left the install alone.
The fake-timers library fakes every timer for an empty list,
so `toFake: []` with no clearing option passed the scan and is the hazard it exists for.
No install did this,
so nothing was hidden;
the scan now reads an empty list as naming every timer,
and its header and the section "Fake clocks that leave native timers" of the fake-clock record say so
(red `ab657f918d`,
fix `b65d31ca09`).
The cause of the sixty seconds that the two bench suites hold their process at exit
(`model-health-probe.unit.test.ts` and `translate-probe-run.unit.test.ts`)
was found with a trace of every timer of a second or more:
the fake-clock walk of the test-fixture package races its calls against real hang-stop waits of one minute each
and never ended them,
five real timers in all,
and a real timer nobody ends keeps the process alive until it fires.
The cause is in a test fixture of the family,
not in the logger or the provider package.
The walk now hands those waits one abort signal and aborts it as it returns.
The process wall of the whole `mise run` command of each file,
two runs each before,
was 60.96 s and 61.05 s for the first and 61.78 s and 61.28 s for the second;
three runs each after took 0.98 to 1.26 s and 1.30 to 1.56 s.
The suites themselves are unchanged.
The case `LEAVES NO REAL TIMER LIVE WHEN THE WALK HAS ENDED` of `fake-clock-walk.unit.test.ts` counts the live timers
before and after a walk and failed at the parent of the fix
(ten live after,
nine before;
red `88d5f73355`,
fix `7e2340cf38`).
The record file names the family's second member and what to do about it,
in the section "Real waits that a fake-clock walk leaves running" of
[the fake-clock record of mistake prevention][clocks-family].
No cached record holds a clock and no key covers one,
so no cache version moved.

Later,
fix round 8:
the scan read `clearTimeout` alone,
yet the library's fake `clearInterval` and `clearImmediate` refuse a native timer in the same way
(`clearTimer` and `getClearHandler` of `@sinonjs/fake-timers` 15.4.0),
so a clock that faked only `setInterval` and `clearInterval`,
or only `setImmediate` and `clearImmediate`,
passed the scan with the hazard.
The re-check's scratch mutant,
the walk fixture's install changed to such a list with the option removed,
passed the scan.
No install of the family did this,
so nothing was hidden.
The scan now reads all three clear functions
(an empty list or no list names all three,
a list names the ones it holds),
a finding names the first clear function the clock fakes,
and its header,
the task description and the record file say so
(red `3a70e5cc1a`,
which holds one fixture install for each extra function and two controls that stay clean,
fix `bd5b9076e4`).
The case `LEAVES NO REAL TIMER LIVE WHEN THE WALK HAS ENDED` compared two counts of the whole process's live timers,
so a timer of the harness that ended between the counts hid a leaked wait:
with the abort removed from the fixture,
two hang stops leaked and the difference of the counts was one.
The case now gives the walk a hang-stop figure no other timer holds,
records the timers made with it through an async hook,
lets the process turn once for the batched end notices,
and expects none of them live and at least one made (`5de8d5ba20`).
It is a guard of what the fixture already did and has no red commit;
with the abort removed it fails with two live where none was expected,
and the other cases of the file pass.
No cached record holds a clock and no key covers one,
so no cache version moved.

## B838: a wait judged a return against the quorum alone

Status:
fixed on 2026-10-23 (UTC),
found by the re-check of B833,
red `228ddd405b` (the re-check's case),
`2a7d699bb2` (a bench under its floor with no hold running),
fix `39863c9691`,
wording `fe0639334b`,
and the case that ends the hold with the first wait `0fbf3cd3e9`.
The reading is the lead's and is open to the owner's veto.
It carries a risk that the owner should weigh before it stands,
which the paragraph "Open to the owner" states with the alternative and the measured trade.
The lead ruled on 2026-10-23,
after the second re-check,
to keep the floor-or-quorum lift as built.

What the re-check showed:
the waiting rule of `a019032c6d` (B833) asked only whether the returning providers lift a short bench to its quorum.
The re-check's case is the `XingZ6020` view with Synthetic held:
the wide bench reads 1 of 7,
the floor is 2,
the quorum is 4,
and Synthetic's return brings back two seats and lifts the bench to 3.
That never reaches 4,
so the reader took no wait,
and the lane contest and the consolidation then skipped every slice for the length of the hold,
because a bench under its floor buys no call (B835).
A single situation had two answers:
the stages treat a bench under its floor as one that decides nothing,
and the wait treated the same bench as one a return could not help.

The rule:
the reader waits for a hold when a return within it lifts a short bench across a threshold that changes what the
stages do.
A bench has two thresholds,
its floor and its quorum,
and a lift across either changes the result.
`run-seats-bench-floor.ts` names the stage code that makes each floor what it is;
all nine floors are two voices,
each for its own reason:
- `wide`,
  `late` and `roster` serve the gathers,
  where below `MIN_STAGE_VOICES` the quorum a short bench closes on exceeds the reachable seats
  (`reachableQuorum`,
  `stage/src/stage-reachable-quorum.ts`);
  `late` also serves the lane contest,
  which opens no round under `LANE_CONTEST_QUORUM` (`lane-contest/src/lane-contest-stage.ts`),
  and `roster` the preparation's pairing rounds and the admission question.
- `select` and `slate` serve the selection rounds,
  where a winner needs `MIN_SELECTION_BALLOTS` ballots whatever the weight (`translate-stage/src/candidate-select.ts`),
  the consolidation buys no slate for a standing under that reach (`consolidate/src/consolidate-slice-buy.ts`),
  and the translate lane buys no candidate for a passage the archive never translated (B839).
- `editors`,
  `refiners` and `translators` write,
  and under `WRITING_BENCH_FLOOR` a slate has no pair to choose between.
- `readers` read a picture twice (`READER_QUORUM`),
  and a single reading is `one-reader-only`.

Between the floor and the quorum a stage still decides on the reachable bench and marks it short:
a gather closes on the reduced voices with a `stage-short-bench` finding,
and a selection scales its minimum weight by reachable over quorum (`selectionMinimum`,
`translate-stage/src/candidate-select-minimum.ts`).
A return that lifts a bench over its floor changes whether the stages call at all.
A return that lifts it to its quorum changes the voices they settle on.
A bench at exactly its quorum,
above two voices,
keeps the short mark all the same:
a gather closes one voice below the quorum there and carries the finding
(`reachableQuorum`,
`stage/src/stage-reachable-quorum.ts:112-123`),
so only a lift past the quorum drops the mark.
The wait decision is the same either way,
because both lifts change the result.

Code paths:
- `liftOf` (`run-seats-bench-floor.ts`) reads what a return does to one bench:
  `toFloor` where the bench stands under its floor now and reaches it with the providers back,
  `toQuorum` where it stands under its quorum now and reaches the quorum it would then have.
- `holdsThatCureShortBenches` (`run-seats-hold-need.ts`) counts a hold when the returning providers lift some bench
  across either threshold between them and this provider's return adds a reachable seat to it.
  "Cures" in B826 and in `mistake-prevention-sheets-quorums-and-outcome-rules.md`
  is the quorum reading of the same test.
- `ceilingWordsOf` (`run-seats-ceiling.ts`) writes one clause per short bench:
  the ceiling,
  its floor and quorum,
  what the return lifts it to
  (`the return lifts it to the floor but not to the quorum`,
  `to the floor and to the quorum`,
  `to the quorum`,
  or `no return lifts it to the quorum`),
  and the seats the return brings back.
- `seatWithoutWaiting` (`run-seats-no-wait.ts`) writes the no-wait line,
  whose reason reads "it would lift no short bench over its floor or to its quorum" since `fe0639334b`.
  A bench under its floor with no hold running says `reads under its floor of 2 and no hold runs that could lift it`.

On the re-check's case the reader now waits (two dryness reads),
where the quorum alone read none.
The case that stood for the `XingZ6020` shape in `run-seats-ceiling.unit.test.ts` read no wait,
and `228ddd405b` changes its expectation because it tested the rule under change.
The failing lines against the parent are eight failures and four cases passing as guards.

Later,
the same day:
the seat table of `run-seats-wait.ts` names the bench each phase seats.
The lane contest,
the consolidation's gate and the page consistency pass judge with the late bench (`seats.lateJudges`,
7 seats),
which holds Kimi-K3 and not Qwen3.8-27B,
and the preparation and the admission lean on the roster.
`benchFloorOf` had no floor for the two new names,
so a return that lifted either over its floor read as no lift;
both now have the two voices of the gathers they serve (fix `2e6cf15c62`).
On the `XingZ6020` view the lane contest's bench is the late one:
1 of 7 with Synthetic held,
and 2 of 7 with it back,
which is the floor and not the quorum,
so the rule waits for it all the same.
The 3 of 7 that the paragraph "What the re-check showed" gives is the wide bench the reader then read.

Open to the owner,
with the lead's ruling:
the wait is right only where the return holds.
On `XingZ6020` it held rarely.
The lane contest waited 52 times for 15,008.7 s of hold,
and the consolidation 59 times for 16,501.1 s
(counted on the log,
lines 32811 to 34695 and 34785 to 39642),
and all 63 contests read 1 of 7 usable.
Counted on the log,
keyless,
a call was served by the returning provider before its next refusal after:
- 0 of 52 lane contest waits.
  The 52 waits are 14 returns (waits grouped by return time within 15 s),
  68.6 minutes of wall clock by the union of the waits.
- 4 of 59 consolidation waits.
  The 59 waits are 16 returns,
  78.2 minutes of wall clock,
  and the four waits that were followed by a served call all ended at 05:06:50 UTC,
  one return,
  after which Synthetic served 20 calls until its refusal at 05:12:12 UTC.
  Slices 27,
  28 and 33 then settled `2/7 usable, settled on consolidated` with a Kimi ballot in a two-ballot gate.
- 4 of 112 translate lane waits (25 calls,
  01:42:52 to 01:50:34 UTC),
  0 of 4 lanes waits and 0 of 1 page consistency wait.
So "Synthetic was refused again as soon as it returned" is true of every lane contest wait and of 55 of the
consolidation's 59,
and false of the one return that served.
The reader can state what a return could lift;
it cannot know that the provider will serve.
The rule as ruled takes those waits again on that shape.
Three rules against this log (counted on it;
the second and third are counterfactual,
since a run that does not wait does not reach the return):
- Floor and quorum,
  as built:
  31,509.8 s of hold summed over concurrent slices,
  146.8 minutes of wall clock by the union of the waits (68.6 and 78.2),
  and 3 gates settled with a seat the wait bought (slices 27,
  28 and 33).
  The 14 returns of the lane contest and cycles 8 to 16 of the consolidation (9,453 s) bought nothing.
- Quorum only (`a019032c6d`):
  0 s and no gate bought.
  With no wait both phases finish in minutes,
  long before the return at 05:06:50 UTC,
  so the 20 calls do not happen,
  and the stages skip each slice they cannot ask while the hold runs.
- Floor and quorum,
  with no further wait on a provider in a phase once a return of it was refused before it served:
  497.5 s of hold (one wait in the first return of each phase,
  216.5 s and 281.0 s),
  8.3 minutes of wall clock,
  and no gate bought,
  since the first return served nothing in either phase.
  It also needs a router surface that does not exist:
  the reader sees holds,
  dryness and meter states only,
  and has no served count or last-served time to reset the memory on.
The lead's ruling is to keep floor and quorum as built.
On this log only that rule bought seats,
and the current tree's meter reading removes the shape that cost the 146.8 minutes for Synthetic:
`synthetic-quota.ts` reads `planWeeklyLimit` and marks a credit at 0 percent dry,
`routesAsDry` is true for `dry` only,
and `returningProvidersOf` leaves a meter-dry provider out,
so the reader seats at once.
That is read from the code and has not been run against the log.
What remains is a provider whose meter reads `unreadable` or wet and holds for 60 s:
the rule costs at most one such hold per return.
Quality of the shipped page outranks wall clock here.
The choice stays open to the owner's veto.

The rule for next time:
a wait is judged against every threshold the waiting stages decide on,
and it names the thresholds in its line,
so a reader of the log sees which one the return would cross.

Cache account:
no sheet or schema changed,
so no cache version moved.

Later,
fix round 4:
the restored Claude-key case.
The second merge of origin/main took the phase-table version of `run-seats-cure.unit.test.ts` whole
and dropped the two Claude-key cases written for it.
The floor-lift case still applies:
in the lanes phase,
with only NeuralWatt wet,
the wide bench reads 1 of 8,
and a hold on the Claude key is waited out because its return makes the bench 2,
the floor of two voices (`69480f5560`;
a guard,
since it passes on the merged code,
and it fails when the wide bench's floor is raised by one).
The crosses-neither case does not apply:
in the merged phase table the Claude key serves the wide bench only,
the lanes phase is the only one that seats it,
and a scan of every wetness of the seven other providers
with only the Claude key's hold running finds the wide bench at 1,
at 3,
or at its quorum already,
and never at 2 of 8 without a writing bench stopping the phase first.
No cache version moved.

## B839: the translate lane bought candidates for a passage its select bench could not back

Status:
fixed on 2026-10-23 (UTC),
red `7a091d9428`,
the case that a repeat keeps what the slice was told `ba9654f88b`,
fix `11227fc801`.
The re-read of the translate hook under the floor is red `9991ee7cd1` and fix `2e76c32d0f`.
The reading is the lead's and is open to the owner's veto.

What `XingZ6020` showed:
B834 left the translate lane's first-time buy on a select bench under the floor open.
Counted on the log,
keyless,
by `d2b-savings.ts` outside the repository,
the unfilled slices that read a select reach under 2 across both attempts are 48,
and they cost
249 NeuralWatt calls,
87.020 Wh,
10 DeepSeek API spends
and 76.6 summed slice-minutes.
The saving the rule claims is the 249 calls and the 87.020 Wh.
Its wall-clock bound is at most about 9 minutes,
8.8 minutes outside the translate lane's hold waits:
the 18 slices of attempt 2 that cost no call add no time,
and the 30 slices that cost calls span 28.6 minutes by the union of their intervals,
19.7 minutes of which lie inside the translate lane's hold waits,
which this rule leaves in place because the hook waits before a batch of slices starts.
The 31.1 minutes that stood here is the union over all 48 slices,
and is not a saving.
Attempt 1 held 18 of them (slices 88 to 105:
153 calls,
58.414 Wh,
19.2 minutes of wall clock),
and attempt 2 held 30 (96 calls and 28.606 Wh for slices 106 to 117,
and 0 calls for slices 88 to 105,
which the payload store replayed from 161 prompt replies).
Every one of the 48 ended unfilled,
and slices 88 to 117 read a select reach of 1 on every line.
The 53 slices of attempt 1 that read a reach under 2 and ended with a computed wording are not counted,
and the rule does not touch them.

The rule:
a passage the archive never translated,
read at a select reach under `MIN_SELECTION_BALLOTS` before its first buy and after the hook's re-read,
buys no candidate.
The buy callback in `translate-slice-settle.ts` returns the unfilled `no-candidate-backed` outcome
the slice would have ended with
(`skippedBelowFloor` in `translate-lane/src/translate-floor-limited.ts`),
logs one warning per slice,
and reports the finding
`stage-bench-below-floor (translate-select reachable 1 of 7, floor 2, no call made)`.
A slice the archive translates ends as a settled slice whatever its bench,
so it is bought as before.

The paths that could back a candidate under the floor,
read before the skip was written:
- The stored-record resume answers before the skip,
  so a record any earlier run backed is still used.
- Twin reuse (`reuseTwinOrBuy`) answers before the skip,
  and runs the purchase only on a miss.
- The pass's memory (B834) answers first.
- The sole-candidate rule ships an incumbent,
  which a passage with no archive wording lacks.
- A challenge round's tie-break needs two candidates drawing weight.
- The preference shipped past a decline needs archive wording that cannot ship.
- The translate lane has no payload store of its own.
Mutations that move the skip before the resume and before the twin memo fail the named cases;
the record is kept outside the repository.

What changed with it:
`rememberFloorLimited` now reads the record it replaces and keeps its `told` flag,
so a repeat of the same ending does not unsay the one line.
That made the write non-blind,
and the `blind-map-writes` allowance for it is gone from the scan's list.
The two settlements known before any purchase moved to `translate-slice-unbought.ts` with their own test,
to keep `translate-slice-settle.ts` under the line limit.

Polish:
none reaches a skipped slice,
because `translate-lane/src` holds no polish code
and an unfilled slice carries no wording for the consolidation's polish.

Cache account,
for each of the eight versions the audit counts:
- Translate slice (25):
  the key is unchanged and an unfilled slice is never cached.
- Lane contest (16),
  consolidate (32),
  repair slice (44),
  refine (14),
  consistency (7),
  pairing (3) and page title (6):
  nothing they key on was touched.
The skip puts nothing into a key,
so no version moves.

The rule for next time:
a purchase whose every outcome is already known is not made,
and the stage that would have made it says so once per slice and keeps the outcome it would have ended with.

Later,
fix round 4:
the tagged line of `notAskedAgain` had no test.
The helper was reachable only through `translateDocument`,
where every record held at a reach under the floor is already told,
so the branch that writes its line never ran in a test and a change back to the untagged logger failed nothing.
The package now exports `notAskedAgain` and its decision type,
and a case seeds the memory with `rememberFloorLimited` without `toldWhy`,
calls the helper at the recorded reach and expects the kept unfilled outcome,
one line tagged `[notAskedAgain]` with its counts,
the told mark set,
no line on a second call,
and an ask without a line once the bench has grown (`cf57d99088`,
export `098cc14015`).
The red is the missing export only,
which stops the file from loading;
the case fails on the untagged logger at `translate-floor-limited.unit.test.ts:684`.
The branch stays,
since it is reachable through the helper's own contract.
No cache version moved.

## B840: the artifact did not say that a slice was kept for want of a bench

Status:
fixed on 2026-10-23 (UTC),
generation 21 declared in `93e9e77a82`,
red `2fb9ef9245`,
fix `b505b55f4e`.
The run package's publication-check cases that pinned generation 20 moved in `91b0edfbe6`,
and the real-clock listing of the gap fixture in `6568642b74`.
The reading is the lead's and is open to the owner's veto.

What the re-check found:
a lane contest slice of `quorum-not-met` with no ballots,
a consolidation slice of `incumbent-only` with no gate,
and a translate outcome of `unfilled` each read the same whether a roster was asked and declined
or the floor kept the slice from being asked at all.
The reason lived only in the log line and in the outcome's findings.
A reader of the artifact alone could not tell a skip from a real decline.

What the rule for a shape change required:
the artifact's readers require their exact keys (`requireExactKeys`),
so a reader built for generation 20 refuses a record that carries a new key.
That is the situation `tenseCandidates` was in when generation 15 added it
(`mistake-prevention-stored-files-and-cached-decisions.md`,
ledger B419):
a key a reader accepts is gated on the generation the file records,
and the writer writes it only where the reader reads it.
An absent key also means "unknown" under generation 20 and "a real decline" under 21,
which a reader cannot work out from the fields alone.
The rule therefore required a version bump,
and an optional key of 20 would have broken both readers and meaning.

What changed:
- `ARTIFACT_SCHEMA_VERSION_V21` is declared with its account in `artifact-two-lane-contract.ts`,
  and the builder and `pass-schema-guard.ts` stamp 21.
  `artifactGenerationRecordsBenchFloor` gates the key,
  and `artifactGenerationReadingRequirements` carries it to the readers as `benchFloorRecorded`.
  A reader still reads generation 20 and earlier as before,
  and a build of 21 refuses to resume a run directory stamped 20 or earlier
  (`assertResumableSchemaGeneration`).
- The consolidation slice record carries `benchBelowFloor: true` on `incumbent-only` with no gate
  (`describeConsolidateSlice`;
  read by `parseBenchFloorMark` in `artifact-two-lane-read-consolidate-slice.ts`),
  which refuses the mark on a slice that asked a slate.
- The contest slice record carries it on `quorum-not-met` with no ballots
  (`describeContestSlice`;
  read by `parseContestFloorMark` in `artifact-two-lane-read-contest-floor.ts`).
- The translate lane's `unfilled` outcome sits in the frozen version-2 outcome vocabulary of `artifact-pool`,
  so the mark joined that member (`ArtifactSliceOutcome`),
  and `UnfilledSlice.benchBelowFloor` in `translate-lane` and `buildLaneSliceTexts({belowFloorChunkIndices})` in
  `translate-floor` carry it from the skip to the artifact.
  `parseSliceOutcome({benchFloorRecorded})` reads it.

Limits:
- The contest skip is never persisted,
  because `worthResuming` needs `usable` at the quorum,
  so the mark exists in the artifact of the pass that skipped and nowhere in the cache.
- Another branch may claim generation 21 for its own change.
  Whichever lands second moves to 22,
  with the gate functions' thresholds and the account text.
- The dependents `page-check`,
  `bench` and `probe` read generation 21 and pass their suites.
- Twelve scratch mutations of the writers,
  the readers,
  the gate and the stamp each fail the named cases.

Cache account:
the cache keys of the passes do not read the generation,
and nothing the mark records goes into one.
No cache version moved.

## B1010: a stage waited for a provider whose return had served nothing

Status:
fixed on 2026-10-29 (UTC),
red `84e774c4a1` (provider) and `e72070b251` (run),
fix `bb86480943` (provider) and `8bf91de279` (run),
edges pinned in `6b48872d4a`.
The owner chose the rule.
Three readings are this round's choices and are open to the owner's veto:
the rule reaches every kind of hold and every lift,
its scope is one entry,
and it yields to the two exceptions of the cooldown give-up.

The owner's answer,
through the question tool on 2026-10-29,
quoted as it was asked.
The question:
when a provider is on hold and a stage's judge bench is short,
should the stage wait for the provider to return?
Measured on XingZ6020,
where Synthetic refused again each time it came back:
- in force,
  "wait if it lifts bench":
  wait only when the return would bring a bench up to its floor (two voices) or its quorum;
  31,509.8 s of holds summed (146.8 min wall clock) bought 3 consolidation gates (slices 27,
  28,
  33);
- "remember futile returns":
  as in force,
  plus skip a provider whose returns served nothing;
  497.5 s (8.3 min),
  0 gates bought;
  needs a served count in the router,
  which did not exist;
- "quorum only":
  0 s,
  0 gates.

The owner chose "Remember futile returns".
The measurements and the first statement of the rule are in option (c) of the re-check of the XingZ6020 fixes
(`d123-recheck2.md`,
outside the repository).

What a return that served nothing is,
as the code reads it
(`provider-budget-futile-returns.ts`,
called from `hold` in `provider-budget-refusal-holds.ts`):
- A return is the end of a hold a refusal set:
  the instant `endsAt` of the hold that a later refusal replaces.
  A hold of no time has no return:
  a refusal on a wet meter with no other wet provider sets one,
  and a refusal after it is not a provider coming back.
  A refusal that arrives inside a running hold is no return either,
  because it came from a call in flight when the hold was set.
- Serving is the router's report that a provider's client resolved with a reply for a call (`markServed`,
  called by `callOn` in `provider-router.ts` and by the decisions endpoint for OpenRouter).
  It is the event the cooldown give-up has read since `b95d7cf83d`,
  and it is not a content check:
  a reply that fails the stage's own checks afterwards still counts as served.
  The measurement counted the `SPEND` lines of the provider in the log instead.
  The question said the rule needed a served count in the router,
  which did not exist;
  the router has reported each served call to the budgets since the give-up,
  as a time stamp for each provider,
  so the rule needed a read on the budgets and no new surface on the router.
- A return served nothing when the provider's next refusal arrives after the end of the hold,
  the hold ended after the entry began,
  and the provider has answered no call since the end.
  The refusal is the first answer the provider gave after the wait,
  so a further wait for its next hold buys the same answer.
  Every kind of hold counts:
  a refusal cooldown,
  the rate-limit backoff,
  and a wait the refusal states.
- The mark stands until the provider answers a call,
  and the next entry (`beginEntry`) begins with none.
  The provider stays routable,
  so the stage that goes on short seats it again once its hold has ended,
  and a call it answers clears the mark.
- The scope is one entry.
  The measurement forgot the mark at the end of each phase,
  which is neither of the two scopes the round was asked to choose between.
  The budgets know no phases,
  the give-up they already keep is the entry's,
  and a mark that outlived a served call would be wrong,
  which the served call prevents.
  The figure of the phase scope is in B1012.

What the code does with the mark:
- `futileReturns` on the budgets lists each marked provider with its return (the hold that ended and how soon the
  provider refused again),
  and leaves out a provider whose wait is what lets the entry finish.
  That is the cooldown give-up's own weighing (`waitingHelps` in `provider-budget-give-up-aside.ts`,
  the decision `isAside` makes without the give-up it is about):
  the provider is the only writing provider not dry,
  or with it back no bench floor the entry needs would be left unmet while without it one is.
  The owner reaffirmed those two exceptions on 2026-10-29,
  so the rule does not defeat them.
- The run client carries it as `futileReturns` (`run-config.ts`),
  and the short-bench reading leaves the holds of those providers out of the holds it waits for
  (`leaveOutFutileReturns` in `run-seats-futile.ts`,
  at the first reading and after each wait).
  The questions asked of the holds that remain are unchanged:
  which holds cure a short bench,
  the ceiling,
  and the words for a reading that takes no wait.
  NeuralWatt is not left out while its hold is its day start.
- Each time a hold is left out while a bench is short,
  the reading says so:
  `JUDGE SEATS phase=lane contest not waiting for synthetic: its last return served nothing (its 300000ms hold ended
  1204ms before it refused again, and no call it answered came after), so its hold of 240000ms is not waited for while
  a bench is short; it is waited for again once it answers a call`.
  The budgets write one warn line when they mark a return,
  with the provider,
  the hold,
  how soon it refused again and the entry.

How the rule and the cooldown give-up of `b95d7cf83d` overlap:
- The evidence is the same.
  The give-up reads a hold that was a cooldown,
  ended after the entry began,
  with nothing served since,
  followed by a refusal.
  The mark reads the same without the condition that the hold was a cooldown.
- The consequences differ.
  The give-up makes the provider's cooldowns count for no wait and reads it dry until the entry ends,
  also after it has served.
  The mark skips a wait in the seat reader only,
  leaves the provider routable,
  and ends when the provider serves.
- For a refusal cooldown the seat reader's wait is skipped by both,
  with the same two exceptions.
  The give-up alone already reproduces the rule on the shape of XingZ6020 (B1012,
  reading C),
  because every one of Synthetic's holds there was a cooldown on an unreadable meter.
- Neither makes the other redundant.
  The give-up's `waitedLeft` is also what `holds` hands the router's wait when every provider reads dry
  (`budget-hold-wait.ts`),
  which the mark does not touch,
  and its routing half and its lasting past a served call are its own.
  The mark reaches the holds the give-up never cut:
  a wait the refusal states and the rate-limit backoff.
  The give-up stays.
- Where both stand on one provider,
  the give-up's rule that a stated return counts as a hold still shows in `holds`
  (`provider-budget-cooldown-give-up.unit.test.ts`),
  and the seat reader skips that hold because the return before it served nothing.
  The case that kept a stated return waited for every time
  (`run-seats-cooldown-repeat.unit.test.ts`)
  is unchanged,
  because its client carries no `futileReturns`,
  so it pins the give-up alone.
  Through the run client the stated return is skipped after a return that served nothing.

Cases:
- `provider-budget-futile-return.unit.test.ts` has 11 cases,
  all red at `84e774c4a1`
  (ten with `budgets.futileReturns is not a function`,
  the log case with `expected +0 to equal 1`),
  and green at `bb86480943`:
  no mark before a hold has ended or inside a running hold;
  a cooldown return and a stated return that served nothing;
  a call served after the hold ended;
  a served call clearing the mark;
  the next entry forgetting it;
  a hold that ran out before the entry began;
  the give-up and the mark on one provider;
  both exceptions of the give-up;
  and the log line.
  They read the marks as a record for each provider when they were written red,
  and as a map after the lint rule against nullish unions,
  so the green commit changes the reads to `.get`.
- `provider-budget-futile-returns.unit.test.ts` pins the edges of the rule:
  a refusal at the instant a hold ends,
  one instant inside it,
  a call served at that instant and one before it,
  a hold of no time,
  and an entry that began at the instant the hold ended.
- `run-seats-futile-return.unit.test.ts` has six cases over the real budgets and the real reader on a fake clock.
  Four were red at `e72070b251`
  (waits taken 3 where 1 was expected,
  2 where 1 was expected twice,
  and no skip line),
  and two passed as guards:
  a returner that served is waited for every time,
  and the only provider not dry is waited for every time.
  The others:
  a futile returner counted again after it serves,
  the give-up and the rule on one provider,
  and a stage that goes on short with the reason in the log.
- `run-seats-futile.unit.test.ts` asks the helpers directly:
  a client that carries no marks,
  a reading where no hold runs,
  a mark on a provider that is not held,
  and NeuralWatt's day start against a refusal's hold.

Limits,
open to the owner's veto:
- The rule is not limited to a lift that crosses the floor and not the quorum.
  The re-check that measured option (c) warned that applied to every lift it would also cut the translate lane's waits.
  Replayed over XingZ6020 it does (B1012):
  the translate lane keeps 8 of its 112 waits,
  and the 4 waits that were followed by Synthetic's 25 served calls at 01:42:52 UTC are not taken.
  Limiting the rule to the floor-only lift is one condition in `leaveOutFutileReturns`.
- A writing bench whose only lift is a futile returner,
  where neither exception holds,
  stops the entry with `WritingBenchUnreachableError` as a bench with nothing to wait for does.
  A judge bench goes on short.
- When the only hold running was left out,
  the line of a reading that takes no wait says `no hold runs that could lift it`,
  and the skip line before it says why.
- A reply counts as served when the client resolved with it,
  whatever the stage makes of it afterwards.

The rule for next time:
a wait is worth taking when the last wait for the same provider bought something,
and the provider's own record says whether it did:
the evidence for skipping a wait belongs where the refusals and the served calls are counted,
and the reader that decides the wait only asks.

[futile-ledger]:
  audit-ledger-futile-returns-replayed-and-the-skipped-slice-memory-removed.md
[clocks-family]:
  mistake-prevention-fake-clocks-and-native-timers.md#fake-clocks-that-leave-native-timers
