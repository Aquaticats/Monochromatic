# Provider availability, measured

Part of [the family README](../README.md).

## Eight-seat schema-9 latency diagnosis

Fresh eight-seat `Weideriche_` validation from `88049530a` failed closed after first entry spent
6,433,300 milliseconds.
It wrote no page or artifact;
operator stopped second entry after first terminal refusal.
Matched prior first entries took 3,014,684 and 3,774,160 milliseconds.
This was active work rather than deadlock:
round logs assign 4,797,358 milliseconds,
79.96 minutes,
to post-quorum grace.
GLM-5.3-Flash reached 180,000-millisecond grace cap eleven times,
compared with two and four in matched runs.
Current pass also bought more quality work,
including twenty-seven consolidation rounds compared with ten and fourteen.

Exact final-candidate replay heard all eight seats.
Seven accepted;
GPT-OSS returned one idiomatic-naturalness finding in paragraph three.
Affected phrase preceded second correction and survived all three second-correction proposals.
Previous-candidate replay was also unacceptable,
with GPT-OSS and DeepSeek Flash each returning one different finding;
adjacent GPT findings did not match.
Evidence supports iterative defect discovery,
not claim latest required finding was ignored.
Final floor correctly refused publication after correction cap.
No reviewer wording or corpus wording is retained in readiness record.

Existing round logs proved aggregate cause,
but lacked active consolidation slice,
terminal consolidation exit,
and per-seat absolute-review status.
Pipeline now logs `SLICE-START` plus `SLICE-COST` for consolidation,
distinguishes cache reuse,
unsettled,
failure,
and abort exits,
and reports only absolute-review seat ids,
statuses,
finding counts,
paragraph numbers,
and wording digests.
See `doc/troubleshooting/translation-repair-schema9-latency.md`.

No GLM-5.2 quality result or role calibration transfers to GLM-5.3-Flash.
It enters only catalog-derived broad-roster and image-reading paths;
it does not inherit an editor,
refiner,
checker,
or default benchmark-judge seat.
Admission evidence is replacement-specific:
the live endpoint facts were read on 2026-08-29,
the package client completed 20 of 20 schema-constrained calls,
the full roster health probe heard all nine seats,
and the image-reading boundary returned a usable transcription without exposing its text.
The image-boundary call completed in 78.4 seconds,
inside the production 360-second per-call deadline.
One call does not characterize its latency tail and does not justify a timeout change.
These checks establish wire and modality compatibility,
not actual-output quality.
Fresh schema-9 passage validation remains mandatory before this replacement contributes readiness evidence.

## Running out of budget is normal, and the seven providers run out differently

They are not the same kind of limit,
and a former version of this section had Charm Hyper backwards.
There is one meter per provider,
in `PROVIDER_ORDER` (NeuralWatt,
Synthetic,
Bedrock,
DeepSeek,
Xiaomi,
Hyper,
OpenRouter;
`package/translation-repair/shared-contract/src/provider-name.ts`).

Charm Hyper is a **prepaid balance**,
priced per token and per model.
`GET /v1/credits` returns `balance`,
which `parseHyperCredits` reads and the `METERS` line prints as `hyperBalance`.
Spending draws it down and it does not refill on a schedule:
it read `0` continuously across the whole of 2026-08-24,
before,
during and after a pass,
and reached `10000` on 2026-08-25 only because credits were bought.
A reader who hits `hyperBalance=0` and waits is waiting for something that has not been observed to happen.

Synthetic is a **subscription allowance**,
a weekly credit limit since 2026-10-19
(its rate-limit page of that day gives back 2% of the weekly limit every 202 minutes
and names no five-hour request limit and no concurrency limit),
which the `METERS` line prints as `syntheticWeekly`.
Before that day it was weekly and five-hourly,
and a reply that still carries the five-hour window is honoured:
the line prints it as `syntheticFiveHour` (remaining over maximum) and `syntheticThrottled` (`yes` or `no`),
and prints `syntheticFiveHour=none syntheticThrottled=none` for a reply with no window.
Records of both forms read back the same way.
That one does refill on its own schedule,
and the account owner can sometimes reset it,
but not reliably and not on demand.

OpenRouter is paid credit the owner tops up,
read from `GET /api/v1/credits` as purchased less used and printed as `openrouterUsd`.
Amazon Bedrock is prepaid credit that expires and is never topped up;
the provider shows no balance to a bearer key,
so `bedrockUsd` is the configured credit less the sum of the package's own spend ledger
(`TRANSLATION_REPAIR_BEDROCK_LEDGER` and `TRANSLATION_REPAIR_BEDROCK_CREDIT_USD` in [Configuration](configuration.md)).

NeuralWatt is **free energy that resets once a day,
at the time of day the subscription's period began**:
550 Wh a day on DeepSeek V4.1 Flash and GLM-5.3-Flash,
unused energy not carried over,
and overage charged to the owner's plan.
The owner's day starts at 15:15 EDT,
which is 19:15 UTC,
read off the quota's `subscription.current_period_start` and never assumed to be midnight UTC.
The provider reports no remainder,
so the package counts the larger of its own ledger of the energy each call used
and the account's covered energy since the day began (`GET /v1/usage/by-model`),
which also sees calls other processes made,
those under a tracking alias such as `nw-flash` among them
(`TRANSLATION_REPAIR_NEURALWATT_LEDGER`,
`TRANSLATION_REPAIR_NEURALWATT_DAILY_WH`
and `TRANSLATION_REPAIR_NEURALWATT_DAY_START` in [Configuration](configuration.md)).
The `METERS` line prints `neuralwattWh` (left in that day),
`neuralwattReckonedWh` (the share resting on estimates),
`neuralwattAccountWh` (the account's figure or `none`),
`neuralwattDayStart` (the day's first instant in UTC or `none`)
and the key's cap:
`neuralwattKeyAllowance=none`,
or `neuralwattKeyRemainingUsd` and `neuralwattKeyBlocked`.
The owner caps this key at 1 kWh a day as a backstop under the 550 Wh,
and a blocked cap holds NeuralWatt dry until the cap's reset.
A cost a request reports does not end the day for NeuralWatt:
the provider reports a nominal cost on free-energy requests too,
and the dashboard counts such a request against the free energy.
The only signal of plan energy spent is the difference between the quota readings at the start and the end of a run.
A picture in a request is held by its patches and not by its base64 length (ledger B720),
so a call that carries one does not hold the top energy bucket.
The record is `doc/decision/translation-repair-neuralwatt-provider.md`.

Plan a pass around these facts rather than around a clean window,
because a clean window still cannot be arranged.

A provider that is out of budget does not fail a run.
The budget layer raises `NoProviderForModelError` for each model no reachable provider can take,
the stage records a lost voice,
and the run continues on whoever answered.
On 2026-08-24 a pass ran with Charm Hyper dry from its first second:
the five Hyper-only models were refused,
the five Synthetic-served models kept streaming,
and both meter endpoints kept reading.

Two consequences for reading a run.
A per-entry cost measured while a provider is dry is not the cost a run with every provider wet pays,
and should be labelled with the outage.
Any quality figure measured then rests on whoever was awake,
as on 2026-08-24,
when five of ten models contributed nothing to it.

## A refused credential holds a provider out, and no meter shows it

A meter says whether money is left,
not whether the provider accepts the key.
The Bedrock key expired during zheermao19 (2026-10-20),
every Bedrock call answered HTTP 401,
and the Bedrock meter kept reading wet,
because it is the configured credit less the package's own ledger and an expired token cannot move it.
The router reads a 401 as the provider's credential refused,
holds that provider out of every model for the rest of the run
and asks the next provider serving the model,
so `gemma-4-26b-a4b-it` went to OpenRouter and not out of the bench.
A model only that provider serves reads unreachable without a call,
which a round counts out of its quorum.
A hold from a refused credential has no end inside a run,
unlike a hold on a stream bound or on a budget refusal.
The view the benches are derived from reads such a provider as dry,
so a seat only that provider serves reads as unreachable before it is asked.
The `SEAT CREDENTIAL REFUSED` warn line is the one place the run says it,
once per provider
(see [Configuration](configuration.md)).

## A provider that refuses at once after its cooldown is given up for the entry

A refusal on a meter that says dry or cannot be read holds the provider out for a cooldown of 300 s
(`REFUSAL_COOLDOWN_MS` in `package/translation-repair/provider/src/provider-budget.ts`),
and the seat reader waits that hold out when the provider's return would lift a short bench
(`readDrynessPastShortBench` in `package/translation-repair/run/src/corpus-run/run-seats-short-bench.ts`).
The wait is a guess that the provider comes back.
XingZ6020 (2026-10-23) showed the guess failing.
Synthetic was seated while its meter read `unreadable-treated-as-spendable`
(its `/quotas` body carried no `weeklyTokenLimit` object,
so the meter could not be read),
every call was refused with HTTP 429,
and each refusal set a fresh 300 s cooldown.
The first cycle in the log is the whole defect:
the refusal at 00:29:00 UTC (line 19970),
the wait of 267481 ms that began at 00:29:33 UTC (line 20031),
the seat line `waited=267481ms` at 00:34:02 UTC (line 20133),
the refusal at 00:34:14 UTC,
twelve seconds after the wait ended (line 20260),
and the next wait of 299850 ms at 00:34:18 UTC (line 20355).
The cycle ran every 5 min 17 s:
58 distinct minutes between 00:34 UTC and 06:05 UTC carry a `waited=` of 240 to 300 s.

The bound an entry carried (`withEntryBound` in `corpus-pass/src/corpus-run/pass-entry-bound.ts`,
removed on 2026-10-29 with the NeuralWatt return wait it served) never cut this loop.
It narrowed `passLeftMs`,
and the budgets applied that bound to NeuralWatt's day start alone (`holds` in `provider-budget.ts`),
so a cooldown was waited for whatever the bound was and only the entry's own abort ended the waiting.
Since 2026-10-29 nothing waits for NeuralWatt's day start at all:
the owner chose "Never wait,
relaunch"
("Stop at once and relaunch later."),
so the budgets name no hold for it,
a call whose only way forward is NeuralWatt's day refuses on its first reading,
and the refusal states the time of day in UTC the day starts again.
A refusal cooldown or a stated `Retry-After` of any provider,
NeuralWatt included,
is still waited for.

What the entry in flight ships when every provider goes dry in the middle of its page
is the owner's answer of 2026-10-29,
"Ship interim,
resumable".
The dry gate ends the run as before,
and the entry's own `catch` ships an interim page before it files the stop as `INCOMPLETE`:
the slices with a consolidation record as they settled,
the archive's text for the others,
and the original under its note for a slice the archive lacks.
The entry gets no artifact,
so it stays pending,
and `interim/<id>.json` in the runs directory says the page is interim.
A stop in the lanes phase has no consolidation record yet,
so it ships the archive text,
and the original under the note where the archive has none.
A slice the lanes agreed on or the contest decided,
with no consolidation record yet,
ships the same way,
and its bought work stays in the caches for the next run.

A provider that refuses right after a cooldown ran out,
with nothing served in between,
is now given up for the entry.
The budget layer (`provider-budget-refusal-holds.ts`) keeps,
for each provider,
the last hold a refusal set,
when the provider last served (the router reports each call it served),
and whether the entry gave the provider up.
A refusal gives its provider up when four things hold together:
the hold in force was a cooldown (a meter that did not say wet and a refusal that named no return),
that hold has ended,
it ended after the entry began,
and the provider has served nothing since it ended.
Reading "nothing served since" off the router,
and not off a time window,
is the operational meaning of "waited out" here:
a window would have to outlast the transport's retry ladder,
whose length no measurement fixes.
Where the give-up holds (see the corrected rule in this section),
a given-up provider reads dry to the router and to the seat reader until the entry ends,
its cooldowns count for no wait,
and the reader seats the other providers' seats without waiting.
One warn line from `hold` says it:
`synthetic: refused again right after its 300000ms refusal cooldown ran out, with nothing served in between;
entry <id> stops waiting for synthetic's cooldowns and treats it as unreachable until the entry ends,
unless synthetic is the only writing provider not dry or its return would leave no bench floor unmet
where without it one is`.
A second warn line,
written by the decision,
says whether the give-up holds or is set aside,
and why.
Each attempt of an entry starts with nothing given up (`beginEntry`,
called by `settleEntry`),
so the next entry waits once and decides on its own evidence,
and `beginEntry` writes an info line naming the providers it asks again.

What the rule leaves alone:
a hold that names its own return (a `Retry-After` header,
a daily limit in the body),
a payment refusal,
the quorum sizes of the benches,
and the waits the benches take for the judges they count.
A provider that served after its cooldown is waited for each time,
however often it refuses later.

The give-up costs the entry nothing that waiting would have bought:
the lead's rulings of 2026-10-25,
corrected in round 4 and open to the owner's veto.
Waiting for a given-up provider buys something only when its return would make the entry finishable.
So the give-up is set aside in two cases only,
and the reader and the router then wait the cooldown out and ask the provider again,
as they did before the give-up existed
(the entry's own abort is again the only bound on that loop):
(a) the given-up provider is the only writing provider not dry,
since reading it dry would then end the run for a provider that serves again after its cooldown;
or (b) with that provider back no bench floor the entry's stages need would be left unmet,
while without it at least one is.
Otherwise the give-up holds,
and the entry waits for none of that provider's cooldowns again.
The decision is taken for each provider given up.
Round 3 worded the rule otherwise,
and round 4 superseded that wording.
The account of the superseded wording is kept in one place,
[the entry B965 of the ledger][b965-entry].

The floors are the seat reader's own,
not copies:
`unmetBenchFloors` (`run-seats-give-up-floors.ts`) asks `judgeSeatsFor`,
`unreachableWritingBenches` and `shortBenches` about a dryness view in which only some providers are wet,
for the union of the benches of all phases (`BENCHES_ANY_PHASE_LEANS_ON`).
That is the writing floor (`WRITING_BENCH_FLOOR`) for the editors,
refiners and translators,
and the quorum of every bench any phase reads,
the judging benches and the readers among them.
The decision (`provider-budget-give-up-aside.ts`) asks for the floors twice for each given-up provider:
with the providers left wet (not dry by their meter,
not held by a refusal,
not given up),
and again with that provider added.
A provider whose own meter reads dry is not added back,
since waiting does not bring it back.
(a) reads the writing roster (`RUN_WRITING_PROVIDERS`),
which `runClientFrom` hands the budgets as `writing`:
a provider that is wet but writes nothing (Xiaomi,
whose one seat is a checker) is no alternative,
as it is none against "every provider dry".
The budgets take the floors as `unmetFloors`.
`run-config.ts` takes them as a required parameter,
because the seat reader imports that file for the roster and a dependency cycle is refused by the lint;
`run-client-create.ts` hands them over in the `createRunClient` and `runClientFrom` the package exports.
A caller that knows no roster of benches gets the floor of round 2's ruling,
that some writing provider can be asked (`writingProviderFloor`).
The decision is taken from each meter reading and from the holds as they stand.
One warn line from it says when the give-up is set aside and names the condition,
the provider that is the only writing provider not dry or the floors that stay unmet without it;
one warn line says when the give-up holds,
and names the floors that stay unmet with the provider back;
and one info line says when a set-aside give-up is back in force.

What the corrected rule costs,
measured on the roster of 2026-10-25 with Bedrock off
(scratch script over `unmetBenchFloors`,
every set of the other providers beside the given-up one):
with Synthetic given up,
48 of the 64 sets of the six other providers meet every floor without it,
each holding Hyper or OpenRouter,
and in the other 16 a floor stays unmet with Synthetic back as well,
so no set is one that its return cures.
With Bedrock off,
(b) therefore never sets aside a give-up of Synthetic,
NeuralWatt,
DeepSeek,
Xiaomi or the Claude key,
and only (a) does.
With Hyper or OpenRouter given up,
32 sets meet every floor without it and its return cures the other 32,
because each alone meets every floor.
Replayed over XingZ6020 (the 518 `METERS` readings,
the real decision),
the give-up of Synthetic holds in all 518,
and the corrected rule takes 1 of the 58 measured waits,
the one before the give-up
(the commands are in ledger B965).

Where the give-up holds,
a call for a model only the given-up provider serves ends at the router as `NoProviderForModelError`,
with no request sent.
The stage reads that as that seat unreachable,
never as an error of the entry or the end of the run,
and the quorum and bench rules decide from there
(the trace is in ledger B965).
A pass whose providers left are too small for a writing bench
(DeepSeek alone beside Synthetic)
ends its entry with `WritingBenchUnreachableError` after the one wait,
which ends the entry as a failure a later pass resumes and does not end the run;
the entry could not have finished with Synthetic back either.

The limit that remains:
the decision reads the last meter reading,
which is up to the freshness window (60 s) old,
and the holds as they stand,
so a provider that went dry inside that window is counted wet until a new reading is taken.
The rules are the lead's of 2026-10-25 and are open to the owner's veto;
the owner kept the gather quorums above two and the bench waits as they were the same day.
The record is ledger B965.

## Gemma's pictures reach OpenRouter since 2026-10-21

With Bedrock's key expired,
the only provider showing `gemma-4-26b-a4b-it` a picture was Bedrock,
so a pass with Bedrock out counted the seat in the readers bench (served text by OpenRouter and Hyper)
and could not send it a picture (`seeing=` in the `JUDGE SEATS` line,
ledger B722).
OpenRouter lists the model as reading images,
and `reader-measure` found its transcription through that route to meet both clauses of the reader rule
on 2026-10-21 (UTC),
over 9 pictures of two entries (`roster-changes.md` has the three runs).
Since the lift the seat is shown pictures by OpenRouter whenever OpenRouter is wet,
Hyper still shows it none,
and the state `XingZ6016` started in (Bedrock expired,
OpenRouter wet) now has `readers=5 seeing=3` where it had two seats that could see.

## Measuring how much of the time each provider was there

On 2026-08-24 three writer seats sat on models only Charm Hyper served,
and until then the argument for them rested on a quality pass
plus an availability adjustment that was reasoned about rather than measured.

The budget layer already read its meters every sixty seconds and already knew dry from wet,
but said so at `debug` level,
which a run does not record.
It now says so at `info`,
as one line per reading,
here from 2026-08-24,
when two providers were metered:

```text
[info] [2026-08-24T19:22:07.104Z] [translation-repair] [takeReading] METERS synthetic=wet hyper=dry syntheticWeekly=97% syntheticFiveHour=48/50 syntheticThrottled=no hyperBalance=0
```

Since Bedrock joined on 2026-09-07 the line carried four states,
since NeuralWatt joined on 2026-10-20 it carried five,
and since DeepSeek and Xiaomi joined on 2026-10-22 it carries seven,
one per provider in `PROVIDER_ORDER`,
then every number each meter reported,
`neuralwattWh=`,
`bedrockUsd=` and `openrouterUsd=` among them (`package/translation-repair/provider/src/provider-budget.ts`).

Three states,
not two.
A meter that could not be reached at all reads as `unreadable`,
which still routes as spendable,
because a monitoring failure must not become an outage.
It is kept distinct in the record because a duty cycle
that counted an unreachable endpoint as a working provider would report an outage as uptime.

The numbers after the states are what those states were decided from,
and they are there because the verdict alone could not be checked.
`hyper=dry` is equally what an empty balance and a wrong threshold in `budget-routing.ts` look like,
and separating them once took a live call to the provider,
which no longer exists for a moment already past.
Each provider's numbers and its verdict come off one read,
so they can never describe different instants.

They also separate causes that route identically.
Synthetic goes dry when its weekly budget empties,
or,
where a reply still carries the five-hour window,
when that window empties
or when the account is actively throttled;
those are one bit at the router and up to three different problems to a reader.

A field the reader has never been taught the name of is carried through,
because a level is defined as any field whose value is not a meter state.
Records written before the numbers existed carry states alone,
and the report names that rather than printing nothing.

Read a collection of those lines back with `meter-report`,
passing one or more log paths after `--`.
It spends no quota:

```sh
mise run //package/translation-repair/run:meter-report -- run3.log run4.log
```

It reports,
per provider,
how many readings fell in each state,
the fraction of answering readings that found budget,
and the longest outage as a range.
The range matters.
A reading happens when a run asks to spend,
so two readings can be a minute apart or a day apart,
and an outage seen at one and gone by the next began and ended somewhere in between.
An outage with no wet reading before it,
or none after it,
is reported open rather than as a number,
since it may have started before the record or may still be running.

Every figure is availability **when we were asking**,
not availability.
That is the quantity that prices a seat,
and it is not the same thing.

## Sampling between runs, so a recovery gets observed

An outage that stops a pass also stops the readings,
so nothing observes when the provider came back,
and every outage that ended a run reads as open-ended forever.

```sh
mise run //package/translation-repair/run:budget-sample > sample.log 2>&1
```

One reading of all four providers' meters;
it refuses unless all four provider keys are set,
since a sample missing a provider would read as that provider answering
(`package/translation-repair/run/src/corpus-run/budget-sample.ts`).
No model is called and no token is produced;
a live run took 2.4 seconds.
Capture both streams:
the reading is at `info` and an unreadable meter warns at `warn`.

Repeat it on a timer to fill the quiet stretches,
for example `watch --interval 300`,
and point `meter-report` at the collected logs.

## DeepSeek and Xiaomi are called directly since 2026-10-22

Two more providers join the order,
between Bedrock and Hyper:
DeepSeek (`deepseek`) and Xiaomi (`xiaomi`),
each on the owner's own key and each for exactly one model
(`deepseek-v4.1-flash` and `xiaomi/mimo-v2.6-flash`).
The order is open to the owner's veto.
The record is `doc/decision/translation-repair-direct-providers.md`.

DeepSeek has a meter:
`GET /user/balance`,
dry when `is_available` is false or the USD total is at or under zero,
printed as `deepseekUsd=` on the `METERS` line.
Xiaomi documents no balance,
so its meter answers wet and prints the run's own priced spend as `xiaomiUsdSpent=`,
and its exhaustion is learned from HTTP 402,
which the router passes to `markRefused` as `paymentRequired` for both providers.
A run without either key logs that provider as `dry`,
and `meter-report` reads logs written before today as having no reading for them.

[b965-entry]:
  audit-ledger-providers-and-seats-a-run-holds-out.md#b965-the-seat-reader-waited-out-the-refusal-cooldown-of-a-provider-that-refused-at-once-after-it-for-every-slice
