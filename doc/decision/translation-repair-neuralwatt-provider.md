# Translation repair: NeuralWatt is the fifth provider and the first in order

Decided by the owner on 2026-10-20.
Extends `doc/decision/translation-repair-multi-provider.md` (Synthetic and Charm Hyper),
`doc/decision/translation-repair-openrouter-fallback.md` (OpenRouter)
and the Bedrock decision of 2026-09-07 (`package/translation-repair/doc/provider-availability.md`).
The code is in `package/translation-repair/provider/src/neuralwatt-*.ts`.

## Why

The owner,
2026-10-20:
"I just added TRANSLATION_REPAIR_NEURALWATT_API_KEY .
It includes 550Wh of energy on DeepSeek V4.1 Flash and GLM 5.3 Flash per day.
And we need to meter (using estimations) ourselves because overage eats into my normal plan which I use for other stuff."

The same day,
on the order:
"Use the neuralwatt free energy front and center."

Free daily energy that does not carry over is the most perishable budget the pipeline has,
so NeuralWatt is first in `PROVIDER_ORDER` (`shared-contract/src/provider-name.ts`),
and every seat of DeepSeek V4.1 Flash and GLM-5.3-Flash goes to it first in every role the model holds.
The roster does not change:
no model is seated in a new role and no owner hold is lifted
(GLM-5.3-Flash stays unseated as a judge,
as the owner decided on 2026-09-02).

## Facts the design rests on

All read on 2026-10-20.

- **The API is OpenAI-compatible.**
  Base address `https://api.neuralwatt.com/v1`,
  bearer key,
  `POST /v1/chat/completions` with streaming
  (`https://docs.neuralwatt.com`).
- **Two models are covered,
  on their standard ids.**
  `GET https://api.neuralwatt.com/v1/models` (public) lists `deepseek-v4.1-flash`
  (context 1,048,560,
  maximum output 393,216,
  0.15 and 0.60 USD per million tokens in and out,
  image input,
  reasoning on by default)
  and `glm-5.3-flash`
  (context 1,048,560,
  no listed output ceiling,
  0.15 and 0.50 USD,
  image input,
  reasoning mandatory at the default effort `max`).
  The `-flex` ids,
  the `-speed` ids and the `nw-flash` alias are not used.
- **The daily free energy is 550 Wh.**
  `https://docs.neuralwatt.com/billing/daily-free-energy`:
  500 Wh from the Max plan and 50 Wh for the email address,
  reset at 00:00 UTC
  (superseded 2026-10-20 by the account's own anchor,
  19:15 UTC;
  see "The day boundary and the account-wide count",
  in this record),
  unused energy does not carry over,
  it covers the standard and Flex tiers and not Speed,
  and it is used before anything else.
  The page says no API reports what is left of it;
  `GET /v1/usage/by-model` reports what the whole account used,
  which the design missed until 2026-10-20.
- **Per-request energy is on the stream.**
  `https://docs.neuralwatt.com/energy-methodology.md`:
  before `data: [DONE]` the stream carries a comment line `: energy {"energy_kwh": ...}`
  (only when measurement is available)
  and `: cost {"request_cost_usd": ...}`
  (on every request with cost metadata,
  free ones included,
  where it may be 0).
  `energy_kwh` is the energy consumed;
  the energy charged is that times the lane's multiplier,
  which is 1.0 on the standard lane.
- **`GET /v1/quota` reports the plan and not the free energy.**
  `https://docs.neuralwatt.com/api/quota`:
  `subscription.kwh_used` is the charged energy of the billing period,
  with `kwh_remaining` and `in_overage`;
  the limit is one request per second.
  It says nothing about the daily free energy.
- **NeuralWatt publishes its own per-request means.**
  `https://portal.neuralwatt.com/api/energy-pricing/matrix`
  (generated 2026-10-20T20:04:27Z,
  seven-day window,
  mean watt-hours per request on the charged basis,
  by prompt-size bucket).

## What the pipeline does

### Order and seating

`'neuralwatt'` is first in `PROVIDER_ORDER`,
with the owner's sentence in the comment beside it.
The compile errors at every `ProviderRecord` site were the checklist;
the Bedrock commit `7b532ae314` and the order move `a317f4e032` were the precedents.
Both existing cards gain a `neuralwatt` side with the served id,
the picture flag,
the context window and the output ceiling the listing states
(`shared-contract/src/neuralwatt-card-sides.ts`);
GLM-5.3-Flash's listing states no ceiling,
so the context window stands in.
A test holds every hold of both cards as it was.

### The meter

The pipeline meters the free energy itself,
because nothing reports it and overage is the owner's plan.

- **A durable ledger,
  one line per call,
  by UTC day**
  (by the NeuralWatt day once the fix in "The day boundary and the account-wide count" lands).
  `~/.local/state/translation-repair/neuralwatt-energy.jsonl` unless `TRANSLATION_REPAIR_NEURALWATT_LEDGER` names another place.
  The day's allowance is 550 Wh unless `TRANSLATION_REPAIR_NEURALWATT_DAILY_WH` says otherwise
  (a non-negative number;
  anything else is refused at once).
- **A call holds its estimate before it is sent.**
  The estimate is the bucket mean for the model and the prompt-size bucket,
  the matrix's or the pipeline's measured one ("The margin"),
  times the margin.
  The prompt size is the request body's bytes over `PROMPT_BYTES_PER_TOKEN`,
  rounded up.
  Concurrent calls each hold their share,
  so a burst cannot all read the same remainder
  (a guard test inserts an awaited read between the check and the hold and fails).
  A call whose estimate does not fit is refused by name before any request leaves
  (`NeuralWattAllowanceError`),
  and the router moves it to the next provider without holding NeuralWatt out for the small calls behind it.
- **The measured energy replaces the estimate at settlement.**
  The `: energy` comment's figure is written to the ledger.
  A call whose comment is missing or will not read settles at its estimate,
  marked `unreported-estimate`;
  an attempt abandoned before its stream ended settles at its estimate,
  marked `abandoned-estimate`.
  The `METERS` line carries `neuralwattWh=` (left)
  and `neuralwattReckonedWh=` (the share of the day's spend resting on estimates).
- **Dry when the day cannot hold even the smallest call.**
  The smallest call's hold is the first bucket's lowest mean times the margin.
- **A reported cost is not a billing signal.**
  The first design read a `request_cost_usd` above zero as plan spend
  and ended the UTC day for NeuralWatt on it.
  The first keyed call disproved that
  (see "The first keyed call",
  in this record),
  and the stop was removed the same day.
  The cost is still parsed and reported on the `SPEND` line;
  nothing routes on it.
- **An absent key leaves the provider unconfigured.**
  It reads dry and is never routed to,
  as for every other provider.

### The margin

The margin is 1,
uniform across models and buckets,
since 2026-10-21;
it was 2 until the first keyed runs were read.

- **What it sizes.**
  Only the hold a call has while in flight.
  Settlement replaces it with the measured figure,
  so the margin decides how much is held and never how much was spent.
- **The measured means.**
  Run `zheermao19`
  (build `5b5dca49a9`,
  2026-10-20)
  settled 197 NeuralWatt calls,
  and nearly all fell in the 4,096 to 16,383 token bucket:
  140 of 143 DeepSeek V4.1 Flash calls for 10.613 Wh,
  48 of 54 GLM-5.3-Flash calls for 17.882 Wh.
  Their means,
  0.0758 and 0.3725 Wh,
  replace the matrix's 0.0696 and 0.1015 for that bucket.
  Every estimate in the run was exactly twice its bucket's matrix mean,
  so the bucket each call was held in is the one its logged prompt tokens name.
  The other buckets keep the matrix's figures:
  the run held two to four calls in each.
  The matrix was near for DeepSeek and read GLM at about a quarter of its use;
  its means come from traffic whose prompts were mostly served from the prefix cache,
  and this pipeline's sheets are fresh.
  Those figures,
  0.0758 and 0.3725 Wh,
  were the table's from 2026-10-20 until 2026-10-22 (UTC).
  On 2026-10-22 (UTC) they were replaced,
  in ledger B808
  (`package/translation-repair/doc/audit-ledger-neuralwatt-replies-cut-at-the-default-effort-and-the-energy-estimate.md`),
  by the means of the settled calls of the rotation `zheermao22`,
  `CuspariaKLSY22` and `XingZ6017`
  (1,297 calls,
  555.188 Wh,
  the whole of that NeuralWatt day),
  because the rotation measured 2.6 times the old figures in the main bucket,
  and `XingZ6017` alone measured 2.84 times its hold.
  The 4,096 to 16,384 token bucket now holds 0.1958 Wh for DeepSeek V4.1 Flash (558 calls)
  and 0.9577 Wh for GLM-5.3-Flash (298 calls).
  The 16,384 to 65,536 token bucket,
  which held the matrix's 0.1395 and 0.2009 Wh,
  now holds 0.3205 Wh (385 calls) and 1.2239 Wh (30 calls,
  all from one entry).
  The module note in `package/translation-repair/provider/src/neuralwatt-estimate.ts`
  holds the counts and the per-entry means.
  The other buckets keep the matrix's figures;
  the rotation held at most 15 calls in any of them and none in two.
  On 2026-10-23 (UTC) the four means were replaced again,
  in ledger B836
  (same file),
  by those of the settled calls of `XingZ6020`
  (1,779 calls,
  451.711 Wh),
  the first pass on a build that sends the GLM seat at `low` effort (B807):
  0.3044 and 0.3237 Wh for DeepSeek V4.1 Flash in the two main buckets (817 and 539 calls),
  and 0.0714 and 0.0556 Wh for GLM-5.3-Flash (358 and 43 calls).
  The DeepSeek means are those of that run alone,
  above the pooled 0.2603 and 0.3224 Wh,
  which keeps the hold on the safe side for DeepSeek;
  the GLM means are not pooled with the earlier ones because the effort differs.
  The figures of the margin in this section are the record of 2026-10-22.
- **The spread is in the output.**
  The largest single calls were 0.840 Wh for DeepSeek,
  eleven times its mean,
  and 1.626 Wh for GLM,
  4.4 times its mean.
  Joined to the `SPEND` line of the same stamp
  (193 of the 197 calls have one),
  a call's energy tracked its completion tokens with a correlation of 0.87 for both models:
  about 0.057 Wh per thousand completion tokens for DeepSeek
  and 0.100 for GLM.
  A prompt-size bucket cannot see that before the call is sent;
  the completion cap bounds the worst call,
  and GLM's three calls at its cap of 18,316 completion tokens measured 1.312 to 1.626 Wh.
  The first design's argument for 2 rested on prefill
  (a mean over a cached share h understates a fresh prompt by at most 1/(1-h)),
  and the run showed decode is where these reasoning models spend.
- **Why 1.**
  With the bucket that matters at its measured mean,
  a hold at the mean is right on average.
  A short hold costs only at the moment the day runs out,
  bounded by the calls in flight then:
  in that run up to 30 calls settled within one minute,
  and 30 GLM calls each 1.25 Wh past the mean would overrun the day by about 37 Wh
  (6.8 percent of 550 Wh),
  a bound no measured minute came near.
  The recalibration of 2026-10-22 (UTC) recomputed the bound at the rotation's figures:
  30 GLM calls each at the 90th percentile (2.23 Wh against 0.96 Wh held) overrun the day by about 38 Wh
  (7.0 percent of 550 Wh),
  and the rotation's day,
  held at about a third of its use,
  ended 5.188 Wh (0.94 percent) past zero.
  A margin of 2 held DeepSeek at 1.8 times its use at that run's figures,
  and a larger margin sends the last calls of every day to a paid provider while the free energy still covers them;
  the owner asked for the margin "as small as can be justified,
  so free energy is not left unspent".
- **What it can strand.**
  At the end of a day a call is refused only if its hold does not fit,
  so what is left unspent is less than one hold of the size being sent:
  at a margin of 1,
  from 0.0011 Wh
  (DeepSeek under 256 tokens)
  to 1.2239 Wh
  (GLM from 16,384 to 65,536 tokens,
  0.22 percent of 550 Wh;
  the matrix's largest,
  GLM from 262,144 tokens up,
  is 0.9089 Wh,
  0.17 percent),
  with 0.1958 and 0.9577 Wh for DeepSeek and GLM in the 4,096 to 16,384 token bucket.
  Until 2026-10-22 (UTC) the largest hold was the matrix's 0.9089 Wh,
  and the 4,096 to 16,384 token bucket held 0.0758 and 0.3725 Wh.
- **What the margin cannot do.**
  It cannot stop plan spend by itself.
  The calls in flight together can pass the day's end by their combined excess before the first one settles;
  settlement at the measured figure corrects the ledger as each call ends,
  and an overrun shows in the plan-energy difference a run prints at its end.
  The owner accepted that risk on 2026-10-20
  ("I can tolerate some potential losses").
- **Recalibrate from later runs.**
  Every settled call logs its measured energy beside its estimate and its prompt tokens,
  and the `SPEND` line of the same stamp carries its completion tokens,
  so a later run can replace the other buckets' figures,
  or bucket by completion tokens once a run supplies enough calls to set a per-model rate.

### The spend report

A run that calls NeuralWatt reads `GET /v1/quota` once before its first call
and once when it ends,
and prints the difference in `subscription.kwh_used` and whether `in_overage` is true.
The reading exists only in a keyed run that made a NeuralWatt call;
no test reads it,
and a failed read never blocks a call.
A starting read that fails is tried again,
no sooner than 60 seconds after the last failure on the injected clock,
the spacing of the day anchor's retry,
and a later read of the quota that the run makes for its own reasons takes its place the same way.
A comparison that starts at such a later reading is printed as starting there:
the line gives the reading's time and says that plan energy spent before it is not counted.
A run whose every start read fails says how many failed and when the last was.
The warning on the first failure is written once.
Ledger P20 in `package/translation-repair/doc/audit-ledger-providers-routing-and-seating.md` records the miss.
Its end report is registered when the run client is built and printed by `reportingRefusals`
(`run/src/corpus-run/run-end-report.ts`).

`spend-report` lists NeuralWatt seats in a bucket of their own,
in no credit total and no USD total.

### Credentials

`TRANSLATION_REPAIR_NEURALWATT_API_KEY` is named wherever another provider key is:
the run client,
the start guard of `--require-providers`,
the budget sample,
the roster-card command,
the no-key refusal and its fixtures,
and the setup step of `doc/runbook/translation-repair-corpus-pass.md`.
A built command's refusal to start in a process that holds keys,
and the child-process environment scrub,
match any variable ending in `_API_KEY`,
so they cover it without a list.
The key's value is masked in every logged exchange by the same credential masking as the others.

## What the meter cannot see

- **The owner's other use of covered models.**
  The free energy is per account.
  Anything the owner runs on DeepSeek V4.1 Flash or GLM-5.3-Flash outside this pipeline spends the same 550 Wh,
  and the ledger alone does not know it.
  The account-wide count ("The day boundary and the account-wide count") reads it back from the by-model usage
  on every meter reading,
  so the stop now sees that use,
  tracking-alias calls included,
  as of the last reading;
  a call the owner makes between two readings is still unseen until the next one.
  One such user is known:
  the owner's dsh harness,
  whose NeuralWatt route (`package/dsh-plugin/neuralwatt`) reads `NEURALWATT_API_KEY`,
  is on the same account (owner,
  2026-10-22).
  Its use of other models shows only in the plan figure (`PLAN ENERGY SPENT`),
  which is why runs that made no NeuralWatt call printed 37.3,
  54.3,
  326.9 and 255.7 Wh there.
  No per-request field tells free energy from plan energy
  (the first keyed call reported a cost on a free request).
  The quota difference in the spend report is the only signal of plan energy spent,
  and it includes the owner's other use of the plan in the same hours.
- **Whether there is any free energy today.**
  The billing page says the daily free energy exists during promotions
  (often weekends,
  with amounts that can change)
  and that without a promotion there is none.
  The pipeline cannot read the promotion calendar,
  and on a day without one its calls would be charged to the plan unseen;
  the owner can set `TRANSLATION_REPAIR_NEURALWATT_DAILY_WH=0` to keep the provider dry from the start.
  The dashboard's hover note names when the promotion ends.
  The page also says the free energy does not apply to per-token accounts,
  the free trial or a past-due account.
- **Another process on the same ledger.**
  A reservation lives in the process that made it.
  Two pipeline processes run at once on one ledger each read the other's settled lines but not its holds,
  so together they can start more than the remainder covers.
  Only the calls each process has in flight at once bound the effect,
  the same exposure as at the end of a day ("The margin").

## The first keyed call

Run `zheermao18`,
started 2026-10-20 at 22:22 UTC on build `1dde8c5cd6`.
Its quota reading before the first call:
1.9861 kWh charged this period.
Its first NeuralWatt call,
DeepSeek V4.1 Flash at 22:23 UTC with 2,954 prompt tokens by the byte estimate
(2,634 by the reply's usage),
measured 0.0167 Wh against an estimate of 0.0466 Wh,
and its cost comment reported `request_cost_usd` 0.000107.
The billed stop never weighed the energy:
it read any cost above zero as proof that the free energy was spent and the plan was being charged,
so after one call of 0.0167 Wh out of 550 Wh it marked NeuralWatt out of energy until 00:00 UTC,
and the run went on with Hyper and OpenRouter
until it was stopped at 22:30 UTC after 21 Hyper calls,
so that free energy would not stand idle while prepaid credit was spent.

The energy methodology page
(`https://docs.neuralwatt.com/energy-methodology.md`,
read 2026-10-20 at 21:40 UTC)
defines the field as the "Billed cost for this request in USD"
and names "`0` for trial and free requests";
the daily free energy page says of covered requests "you pay nothing for them until it runs out for the day".
Neither names the free-energy case in the field's own description,
and the design had taken a zero there as given.
The owner read NeuralWatt's dashboard the same evening:
"Your call only used 60.3J,
which isn't even 0.0%."
60.3 J is 0.01675 Wh,
the measured figure,
counted against the free energy.
So the reported cost is nominal on a free request.
The owner's ruling,
2026-10-20:
"From here on just use NeuralWatt normally,
I can tolerate some potential losses.
If it works wrongly I'll tell you."
The billed stop was removed
(ledger P16 in `package/translation-repair/doc/audit-ledger-providers-routing-and-seating.md`).

The one call also shows the estimate running high:
2.8 times the measured figure in the 1,024 to 4,095 bucket.
One call is no mean;
the margin stays until a run's settled lines give per-bucket figures
(they did with `zheermao19`;
see "The margin").

## The day boundary and the account-wide count

The owner,
2026-10-20 at about 23:15 UTC:
`The stop logic should obviously count total free energy used per day. Every neuralwatt day starts at 15:15`.
Two read-only calls the same minute
(no inference;
the raw output was kept outside the repository)
settled both points.

- **The day starts at 19:15 UTC.**
  The owner's zone is America/New_York,
  so 15:15 EDT there was 19:15 UTC on 2026-10-20,
  and `GET /v1/quota` reports `subscription.current_period_start` as `2026-10-20T19:15:08Z`
  (`kwh_reset_date` `2026-11-19T19:15:12Z`).
  The documentation's 00:00 UTC does not describe this account.
  The pipeline takes the anchor's UTC time of day from that field,
  never a local clock,
  so a daylight-saving change does not move it.
- **The account's whole use is readable.**
  `GET /v1/usage/by-model` with a timestamp window
  (a date-only window of one day came back empty)
  returned,
  for 00:00 to 23:15 UTC,
  `glm-5.3-flash` 9 requests and 0.002811 kWh
  and `deepseek-v4.1-flash` 8 requests and 0.001284 kWh:
  4.095 Wh,
  against the ledger's 17 lines summing 4.094 Wh at the same moment.
  The figure is current to the request and counts every key on the account.
  The same window carried 3.83 kWh on `mimo-v2.6-pro`,
  which the free energy does not cover and the stop must not count.

So the stop counts every covered request on the account since the anchor,
never less than the ledger's own settled lines,
plus the calls in flight;
the meter's blind spot for the owner's other use,
in "What the meter cannot see",
closed when that landed.
Ledger P17 in `package/translation-repair/doc/audit-ledger-providers-routing-and-seating.md` records the miss.

### What landed

- **The anchor and where it comes from.**
  A NeuralWatt day is the 24 hours from the latest instant,
  at or before now,
  whose UTC time of day is that of the quota's `subscription.current_period_start`.
  Where the quota gives none,
  `TRANSLATION_REPAIR_NEURALWATT_DAY_START` (`HH:MM`,
  UTC) does;
  where neither does,
  the day is unknown and NeuralWatt is dry and refused,
  with one warning that names the missing field and both remedies.
  There is no midnight fallback,
  since an assumed 00:00 UTC would count the wrong day's energy.
  Run start logs the anchor and its source.
- **An unknown anchor is tried again.**
  Only a known anchor is kept for the run.
  While it is unknown,
  a later reading or call reads the quota afresh
  (not the failed starting reading,
  which that fresh reading then replaces as the comparison's start),
  no sooner than 60 seconds after the last failed attempt on the injected clock;
  calls in between stay refused.
  The outage is warned about once,
  and the day finally learned is logged with its source.
  One transient quota failure at the start of a run therefore no longer sends the whole run to a paid provider.
- **What a day has spent.**
  The larger of the account's covered energy since the anchor and the ledger's own settled energy since it,
  never their sum,
  plus the holds of calls in flight.
  The ledger's days are the anchored days too.
  The by-model reply's echoed `period` is checked against the window asked,
  and a reply for another window is refused.
  With the route unreadable,
  the last figure for the same day stands,
  else the ledger counts alone,
  with one warning per outage that names the endpoint and the status.
  `energy_kwh` is consumed energy,
  so a flex row is counted at more than it was charged,
  which errs on the safe side.
- **Tracking aliases count.**
  A call made under an alias such as `nw-flash` shows in `products[]` with the alias as its `requested_model`,
  so a count of card ids alone would miss the owner's DeepSeek V4.1 Flash use through it.
  `GET /v1/models` is read at each account reading,
  and an entry counts,
  with its `-flex` twin,
  when its `metadata.huggingface_id` equals that of a covered model's own entry,
  so an alias that moves is followed.
  With that listing unreadable,
  every `nw-` row that is not a speed lane and was served by a covered model counts,
  with one warning per outage.
  Speed lanes never count.
- **The key's own cap.**
  The owner set a 1 kWh daily limit on this key on 2026-10-20.
  The quota reports it as `key.allowance`,
  which still read `null` that day,
  so what a live cap reports is unconfirmed;
  the pipeline reads only the fields the schema states.
  A blocked cap holds NeuralWatt dry until its `reset_at`,
  or until the next day's anchor where it names none,
  with a warning;
  a cap that will not read is warned about once and ignored.
  The 550 Wh is under the 1 kWh,
  so the cap is a backstop and the free-energy stop stays the primary one.
- **The cost of the count.**
  Each account reading makes three GET requests:
  the quota (for the cap),
  the models listing and the by-model usage.
  The budget layer reads the meter every 60 seconds.
  The `METERS` line carries `neuralwattAccountWh`,
  `neuralwattDayStart` and the cap's fields beside the two it had.

### Assumptions left

- A flex twin is named `<id>-flex`.
  The catalog names none,
  and the listing's own twin entries are read when it has them.
- The anchor is kept to the second (`19:15:08`),
  as the quota reports it;
  whether the service's day boundary is the second or the minute is not measured.
- The cap's USD figures are not converted to energy,
  and no kWh field is invented for it.

### When a pass launches

The owner,
2026-10-22 at about 12:30 UTC:
"You don't need to align launches to NeuralWatt starts."
A pass launches when its build is ready,
whatever the hour,
and no pass waits for 19:15 UTC.
This retires the wait recorded for `zheermao23` in
`doc/planning/translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart.md`
and the handovers of 2026-10-22.

A pass that runs across the boundary takes the new day as it comes:
`dayFor` in `package/translation-repair/provider/src/neuralwatt-day-session.ts`
works out the day's first instant from the clock at every reading,
and the case "DROPS A FIGURE FROM A PREVIOUS DAY AFTER AN OUTAGE"
in `package/translation-repair/provider/src/neuralwatt-day.unit.test.ts`
moves one client's clock past the anchor and reads the new day's start and its full 550 Wh.
Before the boundary,
with the day spent,
NeuralWatt stays dry and the other providers serve.

## Not verifiable without a key

- How NeuralWatt handles a picture on either model.
  The listing reports image input for both,
  so both cards say so and a picture call may route there;
  DeepSeek V4.1 Flash keeps its `reader-unmeasured` hold,
  so no picture reader is seated on it.
  A failed picture call is not rerouted,
  because only budget refusals reroute.
- The cache share and the energy of this pipeline's real prompts.
- Whether `stream_options.include_usage` is honoured.
- Whether the quota's `kwh_used` moves for free energy at all.
  If it does not,
  the report shows no plan energy spent on a clean run,
  which is what the owner wants it to show.
- Any concurrency ceiling per model.
  None is known,
  so the client's default is no ceiling,
  as for Synthetic since 2026-10-19.

## Cache versions

None moves.
No cache key names a provider,
the roster is unchanged,
and the build `pipeline=` digest discards namespaced slice caches by itself.
The Bedrock decision moved none either.
This is not a claim that anything rides inside an existing version:
no existing version's account is edited,
and `slice-cache:cache-account-audit` reads the unchanged accounts.
The Bedrock credit paragraphs in those accounts
(for example `lane-contest/src/lane-contest-key.ts`)
say that moving a Gemma call from Bedrock to OpenRouter sooner "changes whose voices a round hears",
and they record it as riding inside version 6 only after checking that no slice-cache file was newer than that version.
The Synthetic quota change of 2026-10-19
(`package/translation-repair/doc/audit-ledger-follow-ups-emphasis-definitions-cache-accounts-and-synthetic-quota.md`)
is the later precedent:
"Routing changes whose voices a round hears",
no version moved,
and the commits went "to the next pre-launch account with the rest".
These commits follow that precedent.
The model id is unchanged on every call,
so a cached decision is still a decision of the same model,
and a bump would discard caches that remain valid.
Adopted on that reading 2026-10-20 at 21:40 UTC,
open to the owner's veto.

## Rejected

- **A flat daily call count.**
  Energy per call spans three orders of magnitude with prompt size,
  so a count either strands most of the day or overshoots it.
- **Reading the quota to decide dryness.**
  It reports the plan and not the free energy,
  and it is limited to one request per second.
- **A per-model or per-bucket margin.**
  The measured per-model means in the bucket the pipeline's calls fall in do that work;
  a margin over them would only scale every hold alike.
- **Seating either model in a new role,
  or lifting a hold,
  because the energy is free.**
  The roster is the owner's decision;
  this record changes where calls go and not who is asked.
