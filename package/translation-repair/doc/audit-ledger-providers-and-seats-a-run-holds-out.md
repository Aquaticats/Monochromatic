# Audit ledger: providers and seats a run holds out

A provider an owner ruling keeps off,
a seat the tally never withheld although its streams carried nothing,
a meter line that read a held provider differently from the router,
and a model an owner answer took off the late-judge bench.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.
## B760: the owner's ruling to leave Bedrock off was held only as prose, so every run still asked it

Status:
fixed on 2026-10-21 (UTC).
Red `d162a9e9c3`,
fixed in `402f95801e`.

The ruling of 2026-10-20,
"Leave Bedrock off",
stood in the handover only.
Every run still held the expired Bedrock key,
asked once,
was refused (HTTP 401,
`SEAT CREDENTIAL REFUSED google.gemma-4-e2b on bedrock`,
CuspariaKLSY22 log lines 214 and 215)
and only then held the provider out.
The brief for the fix said an existing mechanism turned a provider off.
Read in the source,
none did:
what exists is the unset key,
since a provider with no key has no caller and is never routed to,
and `UNCONFIGURED_METER` reads dry.

Decided:
the ruling becomes data the run reads,
and the unset-key path carries it.
`run/src/corpus-run/providers-off.ts` lists `PROVIDERS_LEFT_OFF` with a provider and the reason for each,
and `configureProviders` (`run/src/corpus-run/run-providers.ts`) takes it as `off` and withholds that provider's key.
The Bedrock client is not deleted;
turning Bedrock on again is removing its entry from the list.
The launch of a corpus pass prints `PROVIDER OFF bedrock: <reason>` after the required-provider lines
(`corpus-pass/src/corpus-run/corpus-pass-lines.ts`),
and `--require-providers bedrock` is refused with the reason "left off" before any key is looked at.

What changed in the run:
no Bedrock exchange is made even when the key is set;
`google.gemma-4-e2b`,
which only Bedrock serves,
is refused as unreachable (`NoProviderForModelError`)
and is named unserved from the first `JUDGE SEATS` line,
where CuspariaKLSY22 read Bedrock wet once.

Red cases:
`run-client-bedrock-off.unit.test.ts` (no Bedrock exchange with the key set;
dry from the first reading),
the required-providers case "REFUSES BEDROCK WHILE IT IS LEFT OFF"
and the launch-line cases of `corpus-pass-lines.unit.test.ts`.
`providers-off.unit.test.ts` pins the list itself.

Not changed:
`budget-sample-keys.ts` and the meter sampler (`budget-sample-run.ts`) still build the Bedrock client
for a read of its local spend ledger;
no request leaves the machine,
so the run does not ask Bedrock anything.

Open to the owner's veto:
whether `google.gemma-4-e2b` stays in the benches as an unserved seat
(it does,
by the short-bench rule of 2026-09-09).

The rule for next time:
an owner ruling that changes what a run does is written as data the run reads and prints at launch,
and the handover keeps the reason.
Recurrence:
`mistake-prevention-inputs-defaults-and-values-read-from-text.md`,
"Rulings held as prose".

Cache account:
no sheet or schema changed,
so no cache version moved.


## B761: a call the round cut that had delivered nothing counted for its seat as no ask at all

Found in the read of CuspariaKLSY22 and zheermao22 for the round `w265`.
It narrows B687.

Status:
fixed on 2026-10-21 (UTC).
Red `2f0116657a` (stage and provider-stream),
fixed in `5473f03750`.

What the logs hold:
B687 counted a call that the round cut after quorum (`runGatherRound` aborts the straggler's signal)
under `abandoned`,
which adds to neither `asked` nor `threw`,
so a seat could never be withheld for slow calls.
That protection is right for a reasoning model caught mid-thought,
and wrong for a stream that never carried any model text.
`inception/mercury-2.5` left 33 streams in CuspariaKLSY22 and 11 in zheermao22 that read
`0 content chars, 0 reasoning chars` while the raw count grew with the wait
(7,450 raw characters at 125 s,
first byte at 703 ms):
framing and no text.
The 33 waited 123 to 179 s each,
4,634 s in all,
and the 11 waited 128 to 219 s each,
1,698 s in all.
The same two logs hold 12 cuts of slow reasoners (Qwen3.8-27B 10,
GLM-5.3-Flash 1,
Kimi-K3 1) that had delivered 25,400 to 69,336 reasoning characters and no answer yet.
The first kind is a dead stream and the second a working one,
and only the first may count against a seat.

Measured threshold,
by `abandon-census.ts` in the round's scratch directory,
a read of every `reportStreamProgress` line in the two logs:
- CuspariaKLSY22:
  mercury 36 completed and 33 silent,
  longest run of silent cuts 9,
  and a completed answer followed after a run of every length from 1 to 9.
- zheermao22:
  mercury 108 completed and 11 silent,
  longest run 3.
- No other seat has a silent cut in either log;
  every other seat's cuts that delivered reasoning are not counted.

Decided:
a silent cut counts as a failed ask when the seat has been asked at least `SEAT_WITHHOLD_ASKS` (4) times
and every ask failed (threw or ended silent).
A seat with one answer in its census is never withheld,
so mercury is withheld in neither log,
and any threshold on a run of consecutive silent cuts below 10 would have withheld a seat that went on to answer
(its longest run was 9 and it answered afterwards).
The threshold is therefore the one the code already had for a thrown call,
and not a new number.
A call that delivered content or reasoning stays out of the count,
and so does a cut that the round did not make
(a caller's own stop is still `abandoned`).

Mechanism:
the round aborts a straggler with a `StragglerCutError` (`stage/src/straggler-cut.ts`),
so the tally tells a cut by the round from a cut by the caller by the abort reason and not by the fact of an abort.
`deliveredNoContent` (`provider-stream/src/abandon-silence.ts`) is true for a `StreamCutShortError`
whose answer and reasoning counts are both zero,
or,
without a count,
whose first byte never arrived (`firstByteMs` below zero);
it is false for every other error.
`seatTallyClient` records `silent` for such a cut:
it adds to `asked` and to `silent`,
and `withheld` reads `threw` plus `silent` against `asked`.
The `SEAT` line gains `silent=N`,
the dark line appends `, silent N` when N is more than zero,
and the refusal of a withheld seat says `after all N asks failed (X threw, Y ended with nothing delivered)`.

Red cases:
`seat-tally-silent.unit.test.ts` (tally,
wrapped client and a `runGatherRound` reason case)
and `abandon-silence.unit.test.ts`,
both failing on the missing export and on the old count.

Not recovered:
the wait itself.
A silent stream still holds the round for the grace window before the cut,
and an interleaving seat like mercury that answers some asks is not shortened by this change.
Cutting a stream that has sent no model text sooner belongs to the idle and first-byte guard of the stream layer.

The rule for next time:
an abort is classified by its cause and by what the call had delivered,
and a threshold for a count is chosen on the seats that later answered.
Recurrence:
`mistake-prevention-sheets-quorums-and-outcome-rules.md`,
"Which seats a quorum counts".

Cache account:
no sheet or schema changed,
so no cache version moved.

## B763: the METERS line and the JUDGE SEATS line read a held provider two ways

Status:
fixed on 2026-10-21 (UTC).
Red `ac43cd02e0`,
fixed in `3542f6650c`.

CuspariaKLSY22 printed `bedrock=wet` on all 27 of its `METERS` lines
while its `JUDGE SEATS` lines read `bedrock=dry` from the second on.
The key was refused on the first call and the router held the provider out for the rest of the run,
but `readStates` printed what the meter said,
and the `METERS` line is the only record of when a provider was available.
Three things keep a provider out of routing beyond its meter:
a refused credential,
a timed hold after a refusal
and a refusal for payment.
The routed view applied all three and the printed line none.

Fixed:
`createProviderBudgets` takes `heldOut`,
which defaults to nothing held,
and a provider prints as `dry` when it is held out,
has hold time left or is paid out.
`configureProviders` hands it the credential holds,
and returns them so that `runClientFrom` shares one set with the router.
The state the refusal logic reads (`read()`) and the routed view are unchanged,
so no decision moved:
only the line.

Red cases:
`provider-budget-held-meters.unit.test.ts`,
one for each of the three holds
(the timed hold is shown dry and then wet again once it lapses),
and `run-providers-held.unit.test.ts`,
which holds Bedrock through the run's credential holds and expects `bedrock=dry` on the line.

Consequence:
after B760 Bedrock reads dry from the first line for its own reason,
and the same reading now holds for a key refused later.

The rule for next time:
two readers of one fact take it from one place;
a printed state that skips a layer the router applies is a second source.
Recurrence:
`mistake-prevention-messages-labels-and-counts.md`,
"Two readers of one fact".

Cache account:
none.

## B815: Qwen3.8-27B sat on the lane contest and consolidation benches, and the grace cut 40 of its 95 streams

Status:
fixed on 2026-10-22 (UTC) in `tr-w277`,
on the owner's answer "Remove from both benches",
red `99eca0e2af`,
fix `9639e19d85`,
reseat test follow-on `89067fb49a` (M287).
Cache versions:
the consolidate (31) and lane-contest (15) keys hash the judges' ids,
so the bench change rides inside both numbers,
which then moved to 32 and 16 with B816.
The reading of "both benches" is open to the owner's veto.

The question and the evidence put to the owner:
B810 (`audit-ledger-provider-streams-usage-and-retries.md`) ended by asking
whether `hf:Qwen/Qwen3.8-27B` should sit on the consolidation and late-judge benches at all while Synthetic serves it.
In `XingZ6017`,
served by Synthetic,
40 of its 95 streams were cut after the grace with no answer,
and the mean consolidation slice cost rose from 149 s to 315 s and 346 s while it was seated.
On `Carena0442`,
when Hyper served it,
17 of 19 lane-contest rounds and 7 of 14 consolidation-gate rounds cut it
(TSDoc of `LATE_JUDGE_DROPPED`,
`run-config.ts`).

The reading recorded:
"both benches" is the lane-contest bench and the consolidation bench,
which are seated from one list,
the late judges (`RUN_LATE_JUDGES`,
filtered by `judgeSeatsFor` in `run-seats.ts`;
the consolidation slate's judges are taken from it).
Qwen keeps its wide seats
(the critic,
the panel and the judges of both lanes),
where `run-seats.ts` still seats it per provider
and `HYPER_SLOW_JUDGES` still withholds it where Hyper would serve it.
The wider reading,
"remove it from every judge seat",
was not taken,
because the question the answer followed named two benches.

What changed:
the `hf:Qwen/Qwen3.8-27B` card in `shared-contract/src/model-cards.ts` holds `late-judge-dropped` beside
`openrouter-dropped`,
and `LATE_JUDGE_DROPPED` in `run-config.ts` reads that hold,
so `RUN_LATE_JUDGES` is seven whichever provider is wet
(it was eight with Synthetic wet and seven with it dry).
The `JUDGE SEATS` line reads `late=7 slate=7` with every provider wet,
where it read `late=8 slate=8`.
The hold is on the card and not in `run-seats.ts`,
so the seat stays off the bench when Synthetic goes dry and when only OpenRouter is left.

Red and green:
`run-seats.unit.test.ts` now expects the late judges without Qwen while the wide and select seats keep it
(the case for OpenRouter alone),
and `run-seats-read.unit.test.ts` expects the new counts in the `JUDGE SEATS` line.
The red commit fails against the card,
which held only `openrouter-dropped`.
The run package's tests were run on the fix commit;
the corpus-pass package's were not,
which is the follow-on.

The follow-on:
`pass-seams-reseat.unit.test.ts` in the corpus-pass package failed at load after the fix commit,
because its fixture guard found that a Synthetic dry-out no longer takes a late judge.
Of the 31 views with at least one provider dry,
4 take a late judge (`hf:moonshotai/Kimi-K3`):
Synthetic and Hyper dry with OpenRouter wet,
with any of NeuralWatt and Bedrock.
`89067fb49a` moved the contest and consolidation reseat cases to that view.
The process mistake is M287.

Not verified:
B810 does not split the 95 streams by bench,
so how many of the 40 cuts fell on the two benches left is not known from the record.
No run has gathered since the change,
so the round time it saves is not measured.

Open to the owner's veto:
the reading of "both benches",
and with it whether the wide seats keep Qwen
(settled on 2026-10-29:
the owner took the wide seats too,
B1030).

## B826: the writing-bench floor stopped an entry for want of a named return, though NeuralWatt's return was 48 s away

Status:
fixed on 2026-10-23 (UTC),
found by the lead's read of `XingZ6019` on 2026-10-22 (UTC) (the brief's E2).
The fix landed on main in `9eea0009d1`,
the merge of the work the lead labelled D4 and E2 with its fix rounds 1 to 9;
fix rounds 10 and 11 follow it on the branch that took that merge
(round 10 ends at `e9d79e6f0d`).
The design that landed
(the decision doc `doc/decision/translation-repair-direct-providers.md` states it,
open to the owner's veto):
the seat reader waits for a hold only if it would cure a short bench,
and no longer than the smaller of what the pass has left and what the entry's hard cap has left.
A NeuralWatt return inside what the pass has left but past what the entry has left parks the pass between entries:
the gate records a park instead of tripping,
the queue puts the entry back at the front,
and the gate waits for the return and resumes.
The writing bench error carries the return,
and so parks the pass,
only when the return would lift the bench to its floor.
The entry's `TALLY` line says `PARKED`,
with `returnsInMs=` and `returnsAt=` taken from the park,
which `parkedFor` answers for the failure the entry ended with,
and never from that failure's own fields.
When the budget ends the pass after the wait,
the queue prints `NOT RESUMED <id>` for the entry the park cut short.
The D4 of those briefs is the model-level dryness work of B822,
not the D4 candidate of the `XingZ6018` read.

What the run showed:
`XingZ6019` was launched at 18:54 UTC on the frozen build `sha256-tree-v1:44ce3051...` (commit `1d46232457`),
entry `XingZ60`,
and ended `DONE processed=0`,
`artifacts=0/1` at 19:14:20 UTC after 19.2 min.
The `JUDGE SEATS` lines counted editors 1 of 3 and refiners 1 of 3 reachable against a floor of 2,
`TALLY` printed `status=ERROR` twice,
and the entry was `STALLED` with 16 cache records.
NeuralWatt was dry for the whole run,
because its 550 Wh day had been spent by the earlier runs.
That is not a defect:
the owner ruled on 2026-10-22 that launches need not be aligned to NeuralWatt's day.
The editors were `hf:zai-org/GLM-5.3-Flash` (served by NeuralWatt only),
`glm-5.3` (Hyper,
whose balance was empty)
and `deepseek-v4.1-flash` (called directly,
wet),
so one editor was reachable.
`xiaomi/mimo-v2.6-flash` is a checker only (measured) and was never reached.

The defect:
NeuralWatt's dryness has a known return.
The run decides the account's day anchor at its start,
and the next day began at 19:15:08 UTC,
48 s after the stop.
The seat reader logged `no provider has named its return`,
and the floor stopped the entry instead of waiting those 48 s.
The owner's ruling of 2026-09-08 stops an entry only when a writing bench has no reachable seat
"and no provider has named its return" (`doc/decision/translation-repair-writing-bench-floor.md`),
so a return the run itself holds belongs on the waiting side of the ruling.

The cause,
read from the source at the branch tip and not run:
`readDrynessPastShortBench` (`run/src/corpus-run/run-seats-read.ts`) waits only for the shortest of
`client.providerHolds()`,
the holds a provider's refusal set,
and when that is zero with a writing bench below `WRITING_BENCH_FLOOR` in a phase that stops,
it logs `writing bench unreachable ... stopping the entry`
and raises the refusal (`writtenRefusal`).
A provider dry by its meter alone,
NeuralWatt among them,
has no entry in the holds,
although the budgets can state NeuralWatt's return (`neuralwattReturn`,
which B819 added for the refusal's text).
The build of `XingZ6019` predates B818 to B820,
which are not among its ancestors,
and the source was read at the branch tip `b18a4f7a40`,
which holds them.

The red cases:
`67513d8de3` (`run-seats-neuralwatt-return.unit.test.ts`:
a NeuralWatt return 301 away with 600 left waits,
one 700 away with 600 left refuses at once),
`a17a70dc00` (the bench error names the return,
so a return past the entry's cap but inside the pass parks the pass instead of stopping the entry),
`0309ef74b1` (the park waits for an instant on the gate's clock,
and a refusal for a day start already waited out trips the gate),
and `02cb04bfb4` (two held providers that each cure the bench alone are waited for).

Prevention rules
(recorded in `mistake-prevention-sheets-quorums-and-outcome-rules.md` where they concern test fleets):
a bound names the deadline that forces it and applies only where that deadline applies,
so a wait longer than a call deadline belongs between calls;
when a brief names a bound,
the code implements that bound,
since "bounded by the abort" is not "bounded by what the pass has left";
a refusal field that a later stage reads is set from the final reading,
never from the path that reached it;
a wait that may start later than the reading it came from stores an instant,
not a duration;
and a test fleet models every provider shape of the live roster:
a provider dry,
held,
wet,
absent,
or the only one serving a seat,
and two holds that each cure a bench alone.

Cache versions:
none moved.
`git log --extended-regexp -G '_CACHE_VERSION = [0-9]+'` over `495341919a^..263312b539`
(the D4 and E2 commits)
and over `263312b539..581597e62d`
(the fix rounds and the main they merged)
lists no commit,
and the same search over `c265e8df95~1..c265e8df95` lists that commit
(it set `SLICE_CACHE_VERSION = 44`),
so the search can find a bump.
The work decides when the seat reader waits,
when the pass parks and what the `TALLY` line prints;
no commit of it touches a cache key,
store or version file.

Recurrence:
the thirteenth and fifteenth classes of the defect-class history (the hold wait and the writing-bench floor),
and B819 (a NeuralWatt time the router knew and a text did not state).

## B860: the seat reader judged the lane contest and four other phases against a bench their stage does not seat

Status:
fixed on 2026-10-23 (UTC),
found by the re-check of fix round 9 of the Claude route (its finding on the lane contest row).
The id is the lead's allocation for fix round 10 and may be renumbered at merge time.
The design that landed is open to the owner's veto.

The defect:
the phase table in `run/src/corpus-run/run-seats-wait.ts` named,
for each phase,
the benches the seat reader holds to a quorum.
Five rows named a bench the phase's stage does not seat.
The lane contest judges with `seats.lateJudges` (`pass-contest.ts`) but its row read the wide bench,
on the premise that the late bench holds every seat of the wide bench and one more.
That is false:
the wide bench is the roster less the two GLM-5.3 models that hold `wide-seat-dropped`,
the late bench is the roster less GLM-5.3-Flash,
Qwen3.8-27B and Haiku 5.5,
which hold `late-judge-dropped`,
and Haiku is a wide seat served by the Claude key alone.
With NeuralWatt and Bedrock wet and the rest dry the Claude key back takes the wide bench from 3 of 8 to 4 of 8
and leaves the late bench at 3 of 7 against a quorum of 4,
so the reader waited out a Claude hold for a quorum the contest never gained.
The same mismatch stood in the page consistency pass and the consolidation gate (late bench read as wide),
and in the preparation and the insertion admission
(the whole roster,
`seats.roster`,
read as the wide bench).

The fix:
`BenchName` gains `late` and `roster`,
the names the `JUDGE SEATS` line prints,
`benchesOf` returns them,
and the table reads the late bench for the lane contest,
the slate and the late bench for the consolidation,
the late bench and the translators for the page consistency pass,
and the roster for the preparation and the insertion admission.
The lanes row stays on the wide bench:
the lanes also seat the select judges,
and across all 256 dryness views the select judges are short only where the wide bench is,
which `run-seats-phase-bench.unit.test.ts` checks.
The table leaves the consolidation's writers,
polish refiners and polish judges,
and the lanes' checkers,
unread by design,
which the TSDoc of the table says.

The red cases:
`602c813204` and `74ffa8bb48` (a Claude hold in the lane contest and the page consistency pass is waited out,
the preparation and the insertion admission name no roster shortfall,
the consolidation and the page consistency warnings name no late bench,
and the phase table names the wide bench for five phases).
The first of them expected a roster quorum of 6 and a select shortfall.
The quorum is the rounded-up half (5),
and no view shows a select shortfall,
so the second corrected both.
The control that the same hold still waits where the wide bench is the one short is the lanes phase,
and the pictures phase holds the case where another hold cures the readers and the Claude key adds no reader.

Prevention rule:
a table of benches per phase is checked against the call that seats each phase,
and the relation between two lists of seats is measured from the lists
(`RUN_WIDE_SEATS`,
`RUN_LATE_JUDGES`),
never assumed.

Cache versions:
none moved.
The reader decides when a phase waits and holds no cached record.

Recurrence:
the thirteenth class of the defect-class history (the hold wait).

## B965: the seat reader waited out the refusal cooldown of a provider that refused at once after it, for every slice

Status:
fixed on 2026-10-25 (UTC),
found in XingZ6020 (the corpus pass of 2026-10-22 and 2026-10-23,
build `1d46232457`).
The id is the lead's allocation (B948 to B964 are spoken for by other branches) and may be renumbered at merge time.
The rule is the lead's and is open to the owner's veto.
The rule of the give-up was corrected on 2026-10-25 (round 4):
round 3's wording,
that the give-up holds only while the providers left meet every bench floor,
was the lead's error and is superseded by "The corrected rule" in this entry.
The owner ruled on 2026-10-25 that the gather quorums above two and the bench waits that count judges stay as they are.

The defect:
the pass seated Synthetic while its meter read `unreadable-treated-as-spendable`,
because its `/quotas` body carried no `weeklyTokenLimit` object (the meter half,
fixed by S1 in `0204b7c8ba`).
Every Synthetic call was refused with HTTP 429.
A refusal on a meter that is not wet holds the provider out for `REFUSAL_COOLDOWN_MS` (300 s),
and the seat reader waits the hold out when the provider's return would lift a short bench.
The first cycle in the log `corpus-pass-XingZ6020.log`:
the refusal at 00:29:00 UTC (lines 19970 and 19971),
the wait of 267481 ms (line 20031),
the `JUDGE SEATS` line with `waited=267481ms` at 00:34:02 UTC (line 20133),
the refusal 12 s after the wait ended,
at 00:34:14 UTC (line 20260),
and the next wait of 299850 ms (line 20355).
It repeated every 5 min 17 s,
and 58 distinct minutes between 00:34 UTC and 06:05 UTC carry a `waited=` of 240 to 300 s,
with 395 lines of the refusal `synthetic: refused us while its meter reads unreadable`.
The two counts,
re-run on 2026-10-25 against the log:

```sh
# distinct minutes with a seat-reader wait of 240 to 300 s on 2026-10-23, then their number
rg --only-matching --no-line-number --replace '$1' \
  '^\[[a-z]+\] \[(2026-10-23T[0-9]{2}:[0-9]{2}):[0-9]{2}\.[0-9]+Z\] .*JUDGE SEATS .*waited=(?:2[4-9][0-9]{4}|300000)ms' \
  /var/mnt/encrypted/temp/agent/corpus-pass-XingZ6020.log | sort --unique | wc --lines
# lines recording a refusal of Synthetic on an unreadable meter
rg --count-matches 'synthetic: refused us while its meter reads unreadable' \
  /var/mnt/encrypted/temp/agent/corpus-pass-XingZ6020.log
```

The first prints 58 (the first minute 00:34 UTC,
the last 06:05 UTC) and the second 395.
The brief for the fix said about 25 waits.
That figure was the lead's error:
a gap scan the count of the `waited=` lines does not reproduce.
The 58 and the 395 both count what one provider that never served after its cooldowns cost.

The trace on `35c7a95a34`:
the 429 reaches the router's catch (`provider-router.ts` 583 and 588),
`markRefused` reads the meters at once and,
for a state that is not wet,
sets the cooldown
(`provider-budget.ts` 845 to 847 and 863).
The next slice's reading finds the hold (`pass-reseat-hook.ts` 122),
`readDrynessPastShortBench` counts it as a hold worth waiting for
(`run-seats-short-bench.ts` 230 to 248,
`run-seats-hold-need.ts` 275),
waits it out (341) and reads the dryness again (351),
where the hold has ended and an unreadable meter routes as wet (`provider-budget.ts` 747 to 767),
so the bench is met,
the stage asks Synthetic,
and the call is refused at once.
`withEntryBound` (D4 and E2,
`a0b38f5a45`) does not bound this loop:
it narrows `passLeftMs`,
the budgets apply that bound to NeuralWatt's day start alone (`provider-budget.ts` 884 to 894),
and `run-client-contract.ts` 107 says a refusal cooldown is waited for whatever the bound is.
Only the entry's own abort ended it.

The fix:
`provider-budget-refusal-holds.ts` owns the holds a refusal sets (it took over `heldUntil` from `provider-budget.ts`)
and a mark per provider for the entry.
A refusal gives its provider up for the entry when the hold in force was a cooldown (the meter did not say wet and the
refusal named no return),
that hold has ended,
it ended after the entry began,
and the provider has served nothing since.
Where the give-up holds,
a given-up provider reads dry in the budgets' view and its meter word,
its cooldown counts for no wait in `holds`,
and one warn line names the provider,
the entry and the cooldown that ran out.
Where it is set aside,
the provider is the only writing provider not dry,
or its return would leave no bench floor unmet where without it one is
(see "The corrected rule" in this entry).
The router reports each served call (`markServed`,
from `callOn` and the decisions route),
the run client carries `beginEntry`,
and `settleEntry` calls it before the pipeline,
so each attempt of an entry starts with nothing given up.
The seat reader,
the bench tables,
the quorum sizes and `holdsThatCureShortBenches` are unchanged:
with the provider dry and its cooldown uncounted,
the reader seats the other providers' seats without waiting.

The operational meaning of "waited out",
open to the owner's veto:
the brief's rule says the entry waited the cooldown out,
and the budget layer cannot see who waited.
It reads the evidence it has,
a cooldown that ended after the entry began and a refusal before the provider served anything.
That covers a cooldown the entry waited for and one that ran out unwaited while the entry did other work,
and it leaves out a cooldown that ran out before the entry began,
which the entry did not wait out (the second entry waits once,
then decides).
A time window ("refused within 60 s of the end") was rejected:
it would have to outlast the transport's retry ladder,
whose length no measurement fixes,
and it would give up a provider that served for a minute and then refused.

The corrected rule (round 4):
the lead's ruling of 2026-10-25,
open to the owner's veto.
Giving a provider's cooldowns up narrows where a call may go,
and waiting for the provider buys something only when its return would make the entry finishable.
The owner's rule is that a run always ships.
So the give-up of a provider is set aside,
and its cooldown waited out and the provider asked again as before the give-up existed,
in two cases only:
(a) the given-up provider is the only writing provider not dry (a give-up never ends the run),
or (b) with that provider back no bench floor the entry needs would be left unmet
while without it at least one is.
Otherwise the give-up holds:
the entry waits for none of that provider's cooldowns again.
The decision is taken for each provider given up.

How the rule came about,
the forms of rounds 1 to 3 superseded.
Round 1 read a given-up provider dry for the rest of the entry.
When that provider was the last writing provider not dry,
the seat reader and the router raised `EveryProviderDryError`,
which `pass-dry-stop.ts` turns into the end of the run,
for a provider that serves again after its cooldown.
Round 2 added (a) and set the give-up aside there.
Round 3 found a wet DeepSeek too small to lift a writing bench
(it serves one of the eight translators where the floor is two),
which ended the entry with `WritingBenchUnreachableError`,
and widened the set-aside to "the providers left cannot meet every bench floor".
That wording was the lead's error and is superseded.
It set the give-up aside whenever the providers left could not meet every floor,
whatever the given-up provider's return would do,
so waiting was bought for entries it could not help:
over the 518 readings of XingZ6020 it sets the give-up of Synthetic aside in all 518,
and the 58 waits recur.
Round 3 had measured that and named it in its handback;
the lead retracted the wording in round 4.
Synthetic's return in XingZ6020 would have lifted two benches and left five short
(the roster at 4 of 10 against a quorum of 5,
the wide and select benches at 3 of 8,
the late and slate benches at 2 of 7,
against 2 of 10,
1 of 8,
1 of 8,
1 of 7 and 1 of 7 without it),
so the entry could not have finished either way.
Reading (b) as "the return lifts any unmet floor" would set the give-up aside there too,
since Synthetic lifts the readers and the translators to their quorums,
and is not the rule.

What (b) reads.
The floors are the seat reader's own and are not copied:
`unmetBenchFloors` (`run-seats-give-up-floors.ts`) asks `judgeSeatsFor`,
`unreachableWritingBenches` and `shortBenches` about a dryness view in which only some providers are wet,
for the union of the benches of all phases (`BENCHES_ANY_PHASE_LEANS_ON`).
The writing floor is `WRITING_BENCH_FLOOR` and the quorum of a bench is `rosterQuorumSize` of its seats
(a pair for the readers).
The decision (`provider-budget-give-up-aside.ts`) asks it twice for each given-up provider:
with the providers left wet (not dry by the last meter reading,
not held by a refusal,
not given up),
and again with that provider added.
A provider whose own meter reads dry is not added back,
since waiting does not bring it back.
Another provider given up counts as unavailable in both views.
(a) reads the writing roster (`RUN_WRITING_PROVIDERS`),
which `runClientFrom` hands the budgets again as `writing`:
round 3 had dropped it,
and (a) needs it,
because Xiaomi wet beside Synthetic leaves floors unmet with Synthetic back as well,
so only the roster sets Synthetic's give-up aside there.
The decision is taken from each meter reading and from the holds as they stand,
so the router and the seat reader,
which read only the dryness and the holds,
change nothing.
Three lines say what it found.
A warn line says the give-up is set aside and names the condition:
"synthetic is the only writing provider not dry (how each other provider reads)",
or "with synthetic back no bench floor the entry needs would be left unmet,
while without it these stay unmet" with the floors.
A warn line says the give-up holds when it is first decided,
and names the floors that stay unmet with the provider back
("would still leave these bench floors unmet:
...;
waiting for synthetic would not make the entry finishable"),
or says the providers left meet every floor without the provider.
An info line says a set-aside give-up is back in force.

What the corrected rule costs,
measured on the roster of 2026-10-25
by a scratch script over `unmetBenchFloors`
that counted,
for each given-up provider and each set of the other providers (Bedrock off),
the sets that meet every floor without it,
those it cures and the rest.
The script and its output are not kept in the repository.

With Bedrock off and Synthetic given up,
48 of the 64 sets of the six other providers meet every floor without it
(each holds Hyper or OpenRouter),
and the other 16 leave a floor unmet with Synthetic back as well:
no set is one that Synthetic's return cures.
So with Bedrock off (b) never sets a give-up of Synthetic,
NeuralWatt,
DeepSeek,
Xiaomi or the Claude key aside,
and only (a) does.
With Hyper or OpenRouter given up,
32 sets meet every floor without it and the other 32 are cured by its return,
because each alone meets every floor.
With Bedrock on,
Synthetic's return cures 8 sets,
every one holding NeuralWatt and Bedrock.
The pin of (b) in the run client tests therefore gives Hyper up beside a wet NeuralWatt,
and no test models Bedrock as live (the owner keeps it off and never tops it up).
So the entry waits for a given-up provider's cooldowns again only for Hyper or OpenRouter
or for a provider that is the only writing provider not dry.

The replay over XingZ6020,
read only,
the log `/var/mnt/encrypted/temp/agent/corpus-pass-XingZ6020.log`:

```sh
# the 518 METERS readings, time and the seven states the build of the log printed (it printed no Claude field)
rg --only-matching --no-line-number --replace '$1 $2' \
  '^\[[a-z]+\] \[([0-9T:.Z-]+)\] .*\[takeReading\] METERS (neuralwatt=\S+ synthetic=\S+ bedrock=\S+ deepseek=\S+ xiaomi=\S+ hyper=\S+ openrouter=\S+)' \
  /var/mnt/encrypted/temp/agent/corpus-pass-XingZ6020.log > meters.txt
# the times of the seat-reader lines with a wait of 240 to 300 s on 2026-10-23, then reduced to distinct minutes
rg --only-matching --no-line-number --replace '$1' \
  '^\[[a-z]+\] \[(2026-10-23T[0-9]{2}:[0-9]{2}:[0-9]{2}\.[0-9]+Z)\] .*JUDGE SEATS .*waited=(?:2[4-9][0-9]{4}|300000)ms' \
  /var/mnt/encrypted/temp/agent/corpus-pass-XingZ6020.log > waits.txt
```

A scratch replay (its script and output are not kept in the repository) builds the real decision
(`createGiveUpAside` over `unmetBenchFloors` and `RUN_WRITING_PROVIDERS`) with Synthetic given up and no provider held,
feeds it each of the 518 readings as the last meter reading,
and asks whether the give-up is set aside.
It ran twice,
once with Synthetic's meter counted spendable throughout
(its quota body could not be read) and once with its printed `dry` taken as its meter:
all 518 readings hold the give-up in both runs.
Round 3's reading over the same states sets it aside in 518 of 518.
The positive control,
Synthetic alone with every other provider dry,
reads "set aside" (the only writing provider not dry).
For the waits,
each of the 58 distinct minutes between 00:34 UTC and 06:05 UTC on 2026-10-23 is matched to the last reading at or before its end:
all 58 are held.
The corrected rule takes the one wait that comes before the give-up
(a give-up needs a refusal after a waited-out cooldown)
and none after it:
1 wait against the 58 measured.
The pass is one entry,
so the count is for one entry.
Another attempt of the entry,
or another entry,
would wait once more and then stop.

Where the give-up holds and a call names a model only the given-up provider serves,
the call ends at the router as `NoProviderForModelError` with no request sent.
The trace on this branch:
`routeProviderFor` finds a wet writing provider,
so no `EveryProviderDryError` (`budget-routing.ts` 756),
and no usable provider serves the model (`budget-routing.ts` 786 to 802),
which returns `unreachable`;
`chooseProvider` refuses it through `refuseDarkModel` (`provider-router.ts` 368 to 375;
`model-dark-reason.ts` 201 to 204).
In the stage,
`attemptStageCall` rethrows only a request fault (`stage-call.ts` 464) and the end of the run (`endsTheRun`,
`stage-run-end.ts` 46 to 48,
which is `EveryProviderDryError` alone),
so the `NoProviderForModelError` becomes `seat unreachable` (`stage-call.ts` 475 to 484),
and the voice is unheard,
unanswered and unreachable.
The seat tally counts it as unreachable and never withholds the seat for it (`seat-tally.ts` 494 to 499),
the round frees the seat's place (`refusedByRouter` at `stage-round.ts` 373 to 381,
counted by `noteRefusal` at 543 and read by the quorum at 567),
and the quorum and bench rules decide from there.
The decision seats (`stage-decision-call.ts` 217) and the picture reader (`image-reading-past-refusal.ts` 291) read it the same way.
The entry's gate parks only on `EveryProviderDryError` and `WritingBenchUnreachableError` (`pass-dry-stop.ts` 307 to 309),
and ends the run only on `EveryProviderDryError`.
`stage/src/stage-call-given-up-provider.unit.test.ts` runs the real budgets and the real router with a stub caller that fails
if it is asked,
and passes:
no request was sent,
and the voice is unheard,
unanswered and unreachable.

The limits that remain.
The decision reads the last meter reading,
which is up to the freshness window (60 s) old,
and the holds as they stand:
a provider that went dry inside that window is counted wet until a new reading is taken,
and a change caused only by a hold ending is applied at once but logged when a new reading is taken.
Where the give-up holds beside a provider too small for a writing bench
(DeepSeek alone beside Synthetic),
the entry ends with `WritingBenchUnreachableError` after the one wait,
which parks the entry and never ends the run.
That is the price of the corrected rule,
and the lead accepted it:
the entry could not finish with Synthetic back either,
because the editors and the refiners stay below their floor.
The run client test pins it (two readings end that way after the wait).

The red cases and the commits,
oldest first within each round.
Round 1:
`dd85fdefb2` (`run-seats-cooldown-repeat.unit.test.ts`,
the real budgets over a Synthetic meter that rejects and the real seat reader on one fake clock,
three refusals each right after the cooldown the reading waited out;
it failed on the parent with `expected 3 to equal 1`),
`3d5997c42b` (the fix,
with the pins that pass on the parent's behavior:
a provider that served after its cooldown is waited for each time,
a refusal that states its return is waited for each time,
and a second entry waits once),
`670d3cf8cd` (the logic moved to `provider-budget-refusal-holds.ts`
to keep `provider-budget.ts` under its 300 code lines)
and `d64365b29f` (the source-scan findings).
Round 2:
`cd0e879442` (the Synthetic-alone case as its own pinned case
and the wet-alternative case beside it over NeuralWatt,
with Bedrock kept off;
two of three readings ended in `EveryProviderDryError` on the parent),
`92088c6a8e` (five cases at the level of the budgets and the router's wait),
`4988ea3d5c` (the fix),
`8ba3f07c3f`,
`0ed0c824c0`,
`83d09f26da` and `ccf97ca721`.
Round 3:
`cbd38bd7aa` (the red over `runClientFrom`,
renamed `run-client-create.unit.test.ts`:
on `79182398c5` two of three readings ended with `WritingBenchUnreachableError`
and NeuralWatt beside it read `expected 1 to equal 3` waits),
`c15b734fb5` (the floor source and its tests),
`844cf137f8` (the router's roster case),
`b12fbab84e`,
`2d50a30231` (a merge of `origin/main`)
and `3a706a1b7b`.
Round 4:
`7fa07e4943` (the red;
the state of XingZ6020 reads `expected 3 to equal 1` waits on the parent,
the DeepSeek and NeuralWatt cases that round 3 pinned at three waits now state one and fail the same way,
and the budgets case with floors that stay unmet beside Synthetic reads
`expected 300000 to equal 0` for the cooldown),
`1a5ab1bd0c` (the fix,
with the pins that pass on the parent:
Hyper refusing beside NeuralWatt keeps waiting every time,
Synthetic refusing beside a wet Xiaomi that writes nothing is not ended),
`e22924fc78` (a duplicate-body finding),
`0cb74dd0fb` (two cases of the decision's own unit test,
for the mutants that survived the other cases)
and the merge of `origin/main`.
Two slips of round 2 stay on record:
the red `92088c6a8e` failed `lint:types` as well as its assertions
(its Xiaomi case passed `writing` to a function that did not take it yet),
which a red must not do,
and its stub errors were corrected in `4988ea3d5c` without the body saying so,
and the subject of `8ba3f07c3f` is 106 columns,
over the limit of 100.
Neither commit was amended.

Mutants,
each built and run on the code it was made from and then restored,
the tree equal to the commit after the last.
Each failed at least the cases named.
Round 1:
M1 the served check removed (the holds unit file,
the cooldown give-up file and the rig),
M2 the entry-start check removed (the holds unit file and the rig),
M3 a stated return counted as a cooldown (the cooldown give-up file and the rig),
M4 a cooldown still waited after the give-up (six files of the cooldown family).
Round 2:
M5 the set-aside never applying (every file of the family but the cooldown give-up file),
M6 the roster ignored by the floor of a caller with no benches (the decision's unit file,
once its Xiaomi case existed),
M7 a given-up provider counted as an alternative (the decision's unit file,
the holds and set-aside files,
the run client and the rig),
M8 a held provider counted as an alternative (the decision's unit file and the holds unit file),
M9 `configureProviders` not handing the floors on (the run client and `run-providers-writing.unit.test.ts`).
Round 3:
M10 `runClientFrom` handing no floors (the run client),
M11 the router's roster dropped from `runClientFrom` (only the roster case of the run package),
M12 the floors without the judging quorums (the run client and the floors unit file),
M13 the floors without the writing floors (only the floors unit file:
the quorum is at least the floor for every writing bench,
so the decision does not move and the clause does),
M14 the floors read for one phase only (the run client and the floors unit file).
Round 4:
M15 (a) dropped (the decision's unit file,
the holds and set-aside files,
the Xiaomi case of the run client and the roster case of `run-providers-writing.unit.test.ts`),
M16 (b) dropped (the decision's unit file,
the set-aside file,
the Hyper case of the run client and the floors case of `run-providers-writing.unit.test.ts`),
M17 (b) read as "the return lifts any unmet floor"
(first only the run client cases;
the unit case of one floor lifted and one left unmet was added in `0cb74dd0fb` and now fails for it too),
M18 the floors read with the provider absent in both views (the decision's unit file,
the set-aside file,
the run client and `run-providers-writing.unit.test.ts`),
M19 the run client not handing the budgets the writing roster (the Xiaomi case of the run client),
and M20 a provider whose own meter reads dry added back (the decision's unit file).
M6 survived its run in round 4 before that case existed,
since (a) decides before the roster in the floor is read wherever no writing provider is left,
and the unit case of a given-up Xiaomi outside the roster (`0cb74dd0fb`) now fails for it.

A flaky case of round 4,
found and fixed on 2026-10-25:
the first case of `run-client-create.unit.test.ts` failed once in gate g215 while another job loaded the host
(`expected [ +0 ] to deeply equal [ 300000 ]` at the positive control of the hold after the first ask)
and passed alone right after.
The cause was in the rig and not in the budgets.
A slice took an ask still unsettled after 50 turns of the event loop for one the router holds on the cooldown,
and moved the fake clock 301000 ms.
The ask awaits NeuralWatt's ledger file in each meter reading
(`neuralwatt-ledger.ts`,
`readTextOrEmptyIfMissing`),
which the thread pool finishes when it can.
A read that outlasted the turns let the refusal land at fake time zero while the clock was ticking,
and the clock then jumped past its 300000 ms hold,
which read zero.
Giving the ask one turn instead of 50 reproduced the same failure on `0a8846cfcc`
(`holds=[0,0]` where `[300000,0]` is expected,
with the requests logged at fake time zero inside the tick).
A quota of 10 and of 2 percent of one core did not reproduce it,
nor did a delay of 1,
3 or 30 ms on every ledger read:
the read has to end inside the few milliseconds the clock's tick takes,
so the failure is rare,
and the cases with NeuralWatt wet (three of the six) are the ones exposed.
Production is unaffected:
the hold is stamped and read on the monotonic clock
(`now()` in `provider-budget-refusal-holds.ts`,
at the stamp and at the read),
nothing moves that clock between the two,
and the decision reads the last meter reading and the holds live.
`df2350a6ee` awaits each ask to its end
and moves the clock during an ask only in the case where the router holds the ask
(`routerHoldsTheAsk`:
Xiaomi alone,
no ledger file is read).
The other cases of the round that move a fake clock
(`run-seats-cooldown-repeat.unit.test.ts`,
the set-aside file,
`run-providers-writing.unit.test.ts`)
refuse and tick one step after another over stub meters,
so no real read lies between a refusal and its hold;
each ran singly with a 30 ms delay on every file read and passed.
Three real-clock waits stayed in tests at `c6185d4086`:
`provider-budget-neuralwatt-return.unit.test.ts` at lines 398 and 453 (`pollMs: 10`)
and `pass-entry.unit.test.ts` at line 972 (`wait(10,)`).
[B985](audit-ledger-real-clock-waits-left-after-the-cooldown.md) replaced all three on 2026-10-29.

Prevention rule:
a wait that repeats an ask is judged by what the first wait bought,
see "A wait that buys the same refusal is not repeated" in
[Preventing mistakes: spend guards and the attempts a client makes](mistake-prevention-spend-guards-and-client-retries.md).

Cache versions:
none moved.
`git log --extended-regexp -G '_CACHE_VERSION = [0-9]+'` over `35c7a95a34..HEAD` prints nothing,
and its control `c265e8df95~1..c265e8df95` prints that commit.
The seat reader decides when a phase waits and which seats it asks and holds no cached record.

Recurrence:
the thirteenth class of the defect-class history (the hold wait).
