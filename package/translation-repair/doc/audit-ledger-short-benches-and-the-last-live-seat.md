# Audit ledger: short benches and the last live seat

Quorums sized on the seats that could answer,
how long a round waits on the last of them,
the ceiling a hold could lift a short bench to,
the rounds and purchases not made where the seats that could answer fall short of the floor,
and the artifact's record of why a slice was kept.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B816: a short bench waited on its last live seat, so one stream that held without content held the whole round

Status:
fixed on 2026-10-22 (UTC) in `tr-w277`,
on the owner's answer "Let the grace open",
red `42855d2266`,
fix `1ed1690ab6`,
artifact generation 20 in `19dc276377`,
cache versions in `eb4db609ff`,
TSDocs in `9ed9b5d14a`.
The pins in other packages that the version and generation moves left behind moved in `205eed4f57` and `aed73b727b`,
and the TSDoc and scan fixes are in `8de619e404` (M287).

What B809 showed:
26 of the 27 rounds that took 250 s or more in `XingZ6017` waited on one comment-only stream,
and the grace never applied,
because a consolidation bench of 7 with 3 seats unreachable needs 4 voices,
which is every live seat,
and the grace opens only once the quorum stands.

The question and the answer:
B809 left open whether a short bench that needs every live seat should wait on its last seat at all
(the rule of 2026-09-09).
The owner was asked with two options.
"Keep current rule (Recommended)":
a short bench still waits on every live seat;
pro,
no verdict decided on fewer voices,
quality first;
con,
a stuck seat costs wall-clock.
"Let the grace open":
allow a short bench to close on a quorum below every live seat once the grace ends;
pro,
faster rounds;
con,
some verdicts decided with one voice fewer.
The owner chose "Let the grace open" on 2026-10-22,
against the recommendation.

The rule:
`reachableQuorum` (`stage/src/stage-reachable-quorum.ts`) takes `lastSeatGrace`,
true by default.
Where the reachable seats equal the bench quorum and that quorum is above `MIN_STAGE_VOICES` (2),
it returns `needed: benchQuorum - 1` and `short: true`,
and the findings carry `stage-short-bench` as for any short bench.
Every other shape keeps the rule of 2026-09-09.
Seven seats with three dark leave four,
which now close on three;
eleven seats with five dark leave six,
which close on five.
The round hears the reduced quorum and then the straggler grace runs for the last live seat,
so a verdict is decided with one voice fewer only when that seat does not answer inside the grace.

Refusals inside a round:
round 0 is sized before the router has named any seat,
so a bench the refusals leave with exactly its quorum would still have waited on every live seat.
`runGatherRound` (`stage/src/stage-round.ts`) takes `heardNeededAfterRefusals`,
and the wait reads the need again at every settled ask and at every router refusal,
reserve seats included.
A refusal that hands its place to a reserve seat settles no ask,
so it wakes the wait itself through a one-shot signal per stretch between refusals.
The gather (`stage-quorum.ts`) and the windowed rounds (`stage-windowed-rounds.ts`) pass a function of the refusal count,
and the round records the need it ended on.
Three cases in `stage-round-refill.unit.test.ts` run it for the gather,
a windowed stage and a whole-bench round,
and the refill case for a seat lost in transport now scripts two failing seats,
since two answering seats close the reduced quorum in the first round.

The shape the rule does not reach:
a bench of 3 with 1 unreachable (`{3,1}`) still needs 2 of 2 live seats.
Its quorum is 2,
and the first branch needs a quorum above the floor of two voices,
so `reachableQuorum` returns `needed: 2, short: false` as before.
The repair checker and probe benches with Kimi-K3 unreachable are this shape (B809).
A stuck seat there is bounded by the stream guards of B804 and B809 and by the call deadline,
not by the grace.

Artifact generation 20:
a reader recomputes each naturalness round's verdict from its seats and refuses a record whose stored verdict differs
(`parseNaturalnessReviewRound`).
A stored round whose live seats equal its quorum reads `acceptable` under the new rule and `quorum-not-met` under the old,
so the reader gates the rule on the generation the file records
(`artifactGenerationLetsGraceOpenOnLastSeat`,
`ARTIFACT_SCHEMA_VERSION_V20` in `artifact/src/corpus-run/artifact-two-lane-contract.ts`),
and `reachableQuorum` takes `lastSeatGrace: false` for a record of generation 19 or older.
`1ed1690ab6` landed with that gap stated in its message,
and `19dc276377` closed it.
The pass writes 20,
`page-check` reads its own rounds under the new rule,
and a build of 20 refuses to resume a run directory stamped 19 or earlier
(`assertResumableSchemaGeneration`,
`slice-cache/src/corpus-run/pass-schema-guard.ts`).
`readSettledArtifact` still reads those files,
each under its own generation's rules.

Cache versions:
every stage that caches a gather runs through the rule,
and no key holds the rule.
Consolidate moved from 31 to 32,
lane contest from 15 to 16,
translate slice from 24 to 25,
repair slice from 42 to 43,
refine from 13 to 14,
consistency from 6 to 7
and page title from 5 to 6.
Pairing rides inside 3 under its account of 2026-10-18.
Run zheermao23 is armed for 2026-10-22 19:16 UTC on `65e8325303`,
which holds the previous numbers,
so records may stand at them and the change could not ride inside them.
Each version's TSDoc gives its account.

Not verified:
how often the reduced quorum decides a verdict,
and how much wall-clock it saves,
since no run has gathered under it.
The `19dc276377` message says the corpus-pass suite was still running when it was committed.

Open to the owner:
whether the floor of two voices should give way on a bench of three,
which the answer did not reach and which was not changed.

## B833: a short bench waited on a hold, or did not, and the line never said what the hold could lift it to

Status:
fixed on 2026-10-23 (UTC) for the log line,
red `e28fd62214`,
fix `dc62056a10`.
The waiting rule itself had landed before this entry in `a019032c6d` on 2026-10-23 at 01:50 UTC,
which `XingZ6020` (build `1d46232457`) predates:
`git merge-base --is-ancestor a019032c6d 1d46232457` answers no.
Superseded in part by B837 and B838:
the clause "so no return lifts it" was judged against the quorum alone,
and B838 replaces that reading with the floor and the quorum.

What `XingZ6020` showed:
the lane contest and the consolidation ran 73 minutes of mostly hold waits on a bench that read one seat of seven
(`wide 1 of 7 reachable, quorum 4`).
Only NeuralWatt,
DeepSeek and Xiaomi were wet.
Synthetic,
Bedrock,
Hyper and OpenRouter were dry,
and a Synthetic hold was running.
A Synthetic return brings back two seats of the wide bench (`hf:moonshotai/Kimi-K3` and `hf:Qwen/Qwen3.8-27B`),
which lifts it from 1 to 3 against a quorum of 4,
so no return of any hold in that view lifts it.

The ruling (the lead's,
open to the owner's veto):
a phase waits for a hold only when the hold can change a bench the phase leans on.
Before waiting,
the line states per short bench which seats a return would bring back and whether that lifts the bench to its quorum,
with the all-wet reading as the ceiling.
When no short bench can reach quorum,
the phase proceeds at once on one line that names each bench,
its ceiling and its quorum.

What the head already did,
measured by the red case on the `XingZ6020` view:
the lane contest and the consolidation took no wait (one dryness read),
and the translate lane waited (two reads),
because its translators bench is lifted from 2 to 4 of 8 by the Synthetic return.
`holdsThatCureShortBenches` (`run/src/corpus-run/run-seats-hold-need.ts`) counts a hold only when the held set cures
an unmet bench and the provider's return adds a reachable seat.
Those two behavior cases were green on the red commit,
and the five text cases were red.

What changed:
`benchReadings` (`run-seats-wait.ts`) is the one reading of a bench against its quorum
that `benchShortfalls` and the new `ceilingWordsOf` (`run-seats-hold-need.ts`) both build on.
While a hold runs,
the no-wait line and the wait line carry one clause per short bench:
`<bench> ceiling <c> of <n> with <held providers> back, quorum <q>, so the return lifts it`
or `so no return lifts it (the return brings back <seat ids or no seat>)`.
With no hold running there is no ceiling to state,
so the line is the one it was.
The no-wait branch moved to `run-seats-no-wait.ts`,
because `run-seats-short-bench.ts` exceeded the file limit.
The line on the `XingZ6020` view for the lane contest now carries the clause
`wide ceiling 3 of 7 with synthetic back, quorum 4, so no return lifts it`,
followed by the two seat ids the return brings back
and by `so the phase runs on what is reachable`.

Red cases:
`run-seats-ceiling.unit.test.ts` (seven cases on the `XingZ6020` view,
including a phase with no hold,
whose line keeps `no provider has named a return the pass can wait for`).
Two older cases constrained where the clause goes:
`run-seats-cure.unit.test.ts` expects `; no NeuralWatt return inside what the pass has left,`
and `run-seats-read.unit.test.ts` pins the no-hold line,
so the clause sits before that text and is omitted when no hold runs.
The slate ceiling in the first draft of the red case read 3 of 7 and was corrected to 2 of 7 in the fix commit,
since Qwen is on the wide bench only (B815).

The rule for next time:
a wait or a refusal to wait states what it was weighed against,
the number the held thing could change,
not only that it would or would not help.
Recurrence:
`mistake-prevention-sheets-quorums-and-outcome-rules.md`,
"Which seats a quorum counts".

Cache account:
no sheet or schema changed,
so no cache version moved.

## B834: a slice that ended unfilled for want of a second ballot was bought again by the next attempt of the same pass

Status:
fixed on 2026-10-23 (UTC),
red `60f141495e`,
fix `d59562881c`.
The reading of "unless that bench's reachable count has grown" as any growth,
not growth to the floor,
is open to the owner's veto.
The translate hook's re-read under the floor,
which this entry first listed as a limit,
landed in red `9991ee7cd1` and fix `2e76c32d0f`.
The first-time buy that this entry left open is B839.

What `XingZ6020` showed:
slices 88 to 105 of the translate lane were tried in the first attempt
with the select bench at one reachable seat of seven
(`[judgeTranslateSlate] select: bench short of quorum, reachable 1 of 7`
on all of the 30 slices tried in both attempts).
Every one ended unfilled as `no-candidate-backed`:
`winner was named by 1 ballot against a floor of 2`.
The `REATTEMPT` at line 28092 asked the same 18 again.

Measured on the log,
keyless,
by `d2-savings.ts` outside the repository:
- Attempt 1,
  slices 88 to 105:
  153 NeuralWatt calls,
  58.414 Wh,
  8 DeepSeek API spends,
  50.8 slice-minutes summed,
  19.2 minutes of wall clock.
- Attempt 2,
  the same 18 slices:
  0 NeuralWatt calls and 0 Wh,
  because 161 prompt replies came back from the payload store (`PROMPT-REUSE source=memory`),
  1 DeepSeek API spend,
  6.4 slice-minutes summed,
  2.8 minutes of wall clock.
- Attempt 2,
  slices 106 to 117,
  never opened in attempt 1:
  96 calls,
  28.606 Wh,
  19.4 slice-minutes.
  The rule as ruled does not touch them,
  since they had no earlier attempt.

So the saving of the rule as ruled on this log is about nothing in energy and calls,
one DeepSeek API call,
and about 6.4 slice-minutes (2.8 minutes of wall clock),
because the payload store already replayed the second attempt.
The hold waits of the `beforeSlice` hook run before the skip decision,
so the wall-clock saving is bounded near 2.8 minutes.
The rule stays on the re-check's finding:
it saves next to nothing here and guards the shape,
a repeat of a purchase that nothing could change,
where the payload store holds no replay of the first attempt.
It removes the repeat rather than the first buy.

Decided,
as ruled:
a slice that ends unfilled as `no-candidate-backed`
while the select bench's reachable count is under the selection floor
(`MIN_SELECTION_BALLOTS`,
2)
is recorded in a memory the pass holds,
one per entry (`floorLimitedByEntry` in `corpus-pass/src/corpus-run/pass-floor-limited.ts`).
A later attempt of the same pass reads the select bench again,
and where the count is not above the recorded one it makes no call,
keeps the unfilled outcome and its reason,
reports the findings it reported,
adds `translate-not-asked-again (chunk N, select reach R of B, recorded R0, floor 2)`,
and says why on one `INFO` line the first time it skips the slice.
The slice is still not cached (`translate-slice-buy.ts`),
twins are untouched,
and a later pass starts with an empty memory.
A later pass reaches the slice only while its entry is unsettled,
or reattempted by the queue,
because a pass selects no entry that carries an artifact or a decline record (`selectPendingEntries`),
so "a later pass asks again" is true of those entries and of no other.
The question is named by the slice index,
the original and the wording it stands on,
so a different preparation in a later attempt does not match.
A reach that has grown asks again,
a reach at the floor is never recorded,
and a slice settled on a later ask is forgotten.

The reach comes from the translate lane's hook (`TranslateSliceSeating.selectReachable`),
seeded from the lanes reading the attempt started on
(`readLanesSeats` and `lanesHooksFor`'s `startedSelectReachable`)
and refreshed under a hold.
It is the count of the select judges (`translateModels.judgeModelIds`,
the same list as `selectJudges` in `run-seats.ts`) that a wet provider would serve.

Limit:
the translate hook reads the select bench again under a hold,
and since `2e76c32d0f` also while the reach it hands is under the floor
(`staleWhen: selectReachBelowFloor` in `corpus-pass/src/corpus-run/pass-reseat.ts`),
as the lane contest and consolidation hooks do.
A provider that returns mid-attempt without a hold lifts the comparison within the attempt,
at the cost of one budget reading per sixty seconds (`BUDGET_FRESH_MS`).
The attempt's own slices are never skipped by this memory;
B839 adds the skip before a passage's first buy.

Left open when this entry was written,
and decided in B839:
the translate lane's first-time buy on a select bench under the floor.
On this log it was 249 calls and 87.020 Wh (153 and 58.414 in attempt 1,
96 and 28.606 in attempt 2)
for 30 slices that could not be filled.

Red cases:
`translate-floor-limited.unit.test.ts` (five cases over a single insertion slice:
two attempts at one reach make no call in the second,
the line is said once across three attempts,
a grown reach asks again,
a reach at the floor is not recorded,
and a pass without a memory asks as before),
`pass-floor-limited.unit.test.ts`,
the translate-hook cases of `pass-reseat.unit.test.ts`
and the memory case of `corpus-pass-run.unit.test.ts`.

The rule for next time:
a repeat of a purchase that nothing could change is a cost of the loop,
and the loop remembers what it has seen of the thing that decides the outcome.
Recurrence:
B835,
and `mistake-prevention-sheets-quorums-and-outcome-rules.md`,
"Which seats a quorum counts".

Cache account:
no key,
sheet or schema changed.
The memory is in the process,
an unfilled slice is not cached,
so no cache version moved.

## B835: the lane contest and the consolidation opened rounds for slices their bench could not settle

Status:
fixed on 2026-10-23 (UTC),
red `a425c3f0ec` (lane contest) and `dc635a867b` (consolidation),
fix `6bc22e4160` and `51ee45924b`,
follow-up red `a5063fb88e`,
follow-up fix `7e16bcb1bf`.
The reading that an endorsed standing is still polished is open to the owner's veto.
The artifact's record of the skip is B840,
and the singular slate-judge count in the consolidation's log line is `130d025b24`.

What `XingZ6020` showed:
the lane contest opened 63 rounds,
every one `lane contest: 1/7 usable, settled on neither`,
for 63 NeuralWatt calls,
17.501 Wh and 18.3 summed slice-minutes,
and the artifact records 63 `quorum-not-met`.
The consolidation bought 63 slices:
232 NeuralWatt calls,
55.386 Wh,
20 Synthetic spends and 1 DeepSeek API spend.
Fifty-five ended `slate-declined-standing` with `winner named by one judge alone`.

Measured on the log,
keyless,
by `d3-savings.ts` outside the repository
(each slice's slate reach is the `reachable N of 7` of its own `select: bench short of quorum` lines):
- The rule skips 54 consolidation slices that read a slate reach under 2 on every round and carry an eligible standing:
  164 NeuralWatt calls,
  39.746 Wh,
  0 Synthetic spends,
  31.5 summed slice-minutes.
  All 54 had ended `slate-declined-standing`,
  and all 54 recorded the polish as `not-run (unsafe-baseline)`,
  since the contest had heard nobody to endorse the standing,
  so the calls skipped were slate calls only.
- Four slices (13,
  44,
  67,
  69) read a slate reach of 1 with a withheld ineligible standing and shipped a lone judge's proposal as `consolidated`:
  33 calls,
  6.563 Wh.
  They are kept as they were,
  because skipping would change the text that ships.
- Five slices (27,
  28,
  33 and two more) read a slate reach of 2 at some round:
  35 calls,
  9.076 Wh,
  20 Synthetic spends.
  They are kept.
- With the lane contest's 63 rounds,
  the rule would have skipped 227 NeuralWatt calls and 57.247 Wh of the 295 and 72.887.
  The honest consolidation count is 54,
  not 63 and not 56:
  the seven `consolidated` slices were 13,
  27,
  28,
  33,
  44,
  67 and 69.

Decided,
as ruled:
- The lane contest opens no round for a slice when the reachable count of its bench before the first call is under
  `LANE_CONTEST_QUORUM` (2).
  `contestLaneSlice` takes `reachable`,
  logs one `WARN` line,
  and returns `neither` with no ballot,
  `usable` 0,
  and three findings:
  the zero-ballot finding,
  the short-bench finding sized to the reachable count,
  and the new `stage-bench-below-floor (lane-contest reachable 1 of 7, floor 2, no call made)`
  (`benchBelowFloorStageFinding`,
  `stage/src/stage-reachable-quorum.ts`).
  The slice keeps its incumbent.
  The artifact slice reads `quorum-not-met` with 0 usable and no ballot,
  where `XingZ6020` read `quorum-not-met` with 1 usable and one ballot;
  the reason is in the log line and the outcome's findings,
  and from artifact generation 21 also in the record's `benchBelowFloor` mark (B840).
  The outcome is not persisted (`worthResuming` in `lane-contest-persist.ts` needs `usable` at the quorum),
  so the skip is never cached,
  and a later pass asks the slice again only while its entry is unsettled.
- The consolidation buys no slate for a slice when its slate bench reads fewer reachable judges than
  `MIN_SELECTION_BALLOTS` (2),
  its standing is eligible and no lane text is on offer.
  `buyConsolidationAttempt` takes `slateReachable`,
  writes one `WARN` line,
  and settles on the existing `incumbent-only` exit with the same `stage-bench-below-floor` finding for stage
  `consolidate-slate`.
  The terminal is `incumbent-only`,
  where the same slice read `slate-declined-standing`,
  which avoids an artifact terminal.
  The settlement carries `benchBelowFloor: true`,
  which `consolidationWorthResuming` and the consolidation cache's reader both refuse,
  so a pass on a healthier bench asks the slice again,
  only while its entry is unsettled.
  An ineligible standing is still asked,
  since its slate may be the only wording that ships.
- An endorsed standing is still polished.
  `final-naturalness-completeness.ts` refuses a body slice whose polish is not `settled`,
  or `not-run` for `unsafe-baseline` or `front-matter`,
  so stripping the polish configuration would stop the entry at publish.
  The skip saves the slate's writers and judges,
  and the naturalness roles are asked as before;
  the driver case pins both halves.
  None of the 54 slices of `XingZ6020` had that polish to run.

The reach comes from each hook:
`contestHooksFor` takes `startedReachable` (the late judges a wet provider would serve in the reading the phase started
on),
`consolidationHooksFor` takes `startedSlateReachable`,
and both refresh it under a hold.
They also read again while the reach they hold is under the floor though no hold runs,
because a provider whose hold ended is wet again and the reading taken before it would stand for the rest of the phase
(`reseatHookFor`'s `staleWhen`).

The follow-up:
the first version of that re-read reset the hook's memo of its no-wait line before falling through,
so every below-floor slice wrote the no-wait warning again,
and the hook's own line,
`slice re-seated under a hold`,
though no hold ran.
The reset now happens only when the hook returns its memo,
and the hook writes that line only when a hold ran.
A below-floor re-read costs the reading's own `JUDGE SEATS` line
and a budget reading that is cached for sixty seconds (`BUDGET_FRESH_MS`),
so the file's guarantee of no dryness read while nothing is held
now holds only while the reach is at or above the floor.
The headers of `pass-reseat-hook.ts` and `pass-consolidate-reseat.ts` say so since `9a7fedead5`.
The follow-up red case fails on the first version:
three warnings and three hold lines for three slices,
where it expects one warning and none.

The first consolidation red (`dc635a867b`) fails through an incidental fixture throw (`provider-unavailable`)
and not through the call count it asserts.
The other consolidation cases fail for their stated reason,
and the green run shows the call count itself holds.
The red text is partly accidental,
and it was not rewritten.

Red cases:
`lane-contest-bench-floor.unit.test.ts`,
`consolidate-bench-floor.unit.test.ts`,
the contest and consolidation cases of `pass-reseat.unit.test.ts`,
the driver cases of `lane-contest-driver.unit.test.ts` and `consolidate-driver.unit.test.ts`,
the finding case of `stage-reachable-quorum.unit.test.ts`,
and the cache-store refusal in `consolidate-cache-store.unit.test.ts`.

Cache account,
for each version:
- Lane contest (16):
  the key is unchanged,
  and a skipped round has `usable` 0,
  which `worthResuming` never persists.
  No version moved.
- Consolidate (32):
  the key is unchanged,
  a skipped settlement is refused by `consolidationWorthResuming` and by the cache store's reader,
  and what is persisted is what it was.
  No version moved.
- Translate slice (25):
  the key is unchanged and an unfilled slice is never cached.
  No version moved.
- Repair slice (44),
  refine (14),
  consistency (7),
  pairing (3) and page title (6):
  nothing they key on was touched.
- The NeuralWatt estimate (B836) and the ceiling text (B833) feed no key.

The rule for next time:
a stage states the floor on its voices and checks the bench against it before the first call,
and a settlement bought on a bench under the floor says so on its own record and is never kept as a verdict.
Recurrence:
B816 and B834.

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
which the paragraph "Open to the owner" states with the alternative.

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
all seven floors are two voices,
each for its own reason:
- `wide` serves the gathers,
  where below `MIN_STAGE_VOICES` the quorum a short bench closes on exceeds the reachable seats
  (`reachableQuorum`,
  `stage/src/stage-reachable-quorum.ts`),
  and the lane contest,
  which opens no round under `LANE_CONTEST_QUORUM` (`lane-contest/src/lane-contest-stage.ts`).
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
A return that lifts it to its quorum changes the voices they settle on and drops the short mark.

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

Open to the owner:
the wait is right only where the return holds.
On `XingZ6020` it did not hold.
The lane contest waited 52 times for 15,009 s of hold,
and the consolidation 59 times for 16,501 s
(counted on the log,
lines 32811 to 34695 and 34785 to 39642),
and all 63 contests still read 1 of 7 usable,
because Synthetic was refused again as soon as it returned.
The reader can state what a return could lift;
it cannot know that the provider will serve.
The rule as ruled takes those waits again on that shape.
The alternative is the quorum-only reading of `a019032c6d`:
- Quorum only:
  no wait on this shape,
  the 73 minutes of hold waits do not recur,
  and the stages skip each slice they cannot ask while the hold runs.
  A return that would have lifted the bench over its floor goes unwaited,
  and the slices skipped in the meantime wait for a later pass.
- Floor and quorum (as ruled):
  the stages get their bench back when a return does hold,
  and a shape like `XingZ6020` pays its hold waits again.
The two differ in what to do,
and neither can be measured on a log that holds one such shape,
so the choice is the owner's.

The rule for next time:
a wait is judged against every threshold the waiting stages decide on,
and it names the thresholds in its line,
so a reader of the log sees which one the return would cross.

Cache account:
no sheet or schema changed,
so no cache version moved.

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
10 DeepSeek API spends,
76.6 summed slice-minutes
and 31.1 minutes of wall clock by the union of their intervals.
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
