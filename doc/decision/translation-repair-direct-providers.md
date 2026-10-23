# Translation repair: DeepSeek and Xiaomi are called directly, each for one model

Granted by the owner on 2026-10-22.
Extends `doc/decision/translation-repair-neuralwatt-provider.md` (the fifth provider),
`doc/decision/translation-repair-openrouter-fallback.md` (OpenRouter)
and the Bedrock decision of 2026-09-07 (`package/translation-repair/doc/provider-availability.md`).
The code is in `package/translation-repair/provider/src/direct-*.ts`,
`deepseek-*.ts` and `xiaomi-*.ts`.

## Why

The owner's two grants of 2026-10-22,
quoted as the code quotes them:

- DeepSeek:
  "You can also use the DEEPSEEK_API_KEY as you wish,
  but you may only use deepseek v4.1 flash 1017 with that key."
- Xiaomi:
  "You may only use v2.6-flash with this key."

Neither key is scoped to a model by the provider,
so the restriction is held three ways in code:
the served-id types are single literals (`DeepSeekServedId`,
`XiaomiServedId` in `shared-contract/src/roster-id.ts`),
the model cards carry exactly one side each (`shared-contract/src/direct-card-sides.ts`),
and both clients throw before any request is built for any other roster model
(`DeepSeekModelNotServedError`,
`XiaomiModelNotServedError`).
`shared-contract/src/direct-cards.unit.test.ts` and `provider/src/direct-client.unit.test.ts` fail when any of the three is removed.

The roster does not change.
No model is seated in a new role and no owner hold is lifted:
the same seats are reached by one more route.

## Facts the design rests on

All read in the providers' documentation on 2026-10-22,
or taken from the parent session's probes of that day,
unless stated.
This change itself used no key and made no request;
every live probe cited here was run by the parent session on 2026-10-22 with the owner's keys,
and none was repeated by this change.
The statements that rest on documentation alone are the prices,
the context and output ceilings,
the rate limits,
the error codes,
the claim that `deepseek-flash` is the 1017 build,
and the addresses of the model listings.

- DeepSeek serves `deepseek-flash` (roster seat `deepseek-v4.1-flash`) on `https://api.deepseek.com/chat/completions`.
  Its `response_format` of type `json_schema` is refused with HTTP 400,
  and `{type: 'json_object'}` is accepted
  when the word "json" is in the prompt.
  The client sends JSON mode and states the schema in the system prompt,
  whose text carries that word.
- Xiaomi serves `mimo-v2.6-flash` (roster seat `xiaomi/mimo-v2.6-flash`) on `https://api.xiaomimimo.com/v1/chat/completions`.
  It accepts `json_schema` with `strict` unchanged.
- Both put reasoning in `delta.reasoning_content`,
  which the stream reader drops,
  and both send `:` comment lines,
  which it ignores.
  Neither request carries a reasoning or thinking member for now:
  the owner's standing instruction of 2026-08-25 was lifted for NeuralWatt on 2026-10-21
  and,
  on 2026-10-22,
  for DeepSeek direct,
  Xiaomi direct and Anthropic,
  so both routes are cleared for their documented thinking parameters.
  None is sent until an effort bench has measured one,
  and a value that disables thinking is never sent.
- DeepSeek reports `system_fingerprint` on every reply.
  The client writes it on the completion log line (encoded,
  bounded) as the one per-call evidence of the build.
- Limits:
  DeepSeek documents 2,500 concurrent requests for `deepseek-flash` (HTTP 429 over),
  Xiaomi documents 100 requests a minute and 10 million tokens a minute for `mimo-v2.6-flash`.
  The DeepSeek client bounds concurrency at 2,500.
  The Xiaomi client paces every attempt,
  retries included,
  to 100 in any minute (`createRequestPace`).
- An exhausted balance answers HTTP 402 "Insufficient Balance" on both
  (`https://api-docs.deepseek.com/quick_start/error_codes`,
  `https://mimo.mi.com/docs/en-US/api/guidance/error-codes`).
  The router passes it to `markRefused` as `paymentRequired`.
- No cache key in the pipeline includes the serving provider.
  The slice,
  translate-slice and image-reading keys carry no provider,
  and `servedBy` is stored beside a replayed reply,
  not in its key,
  so a reply replays whichever route first produced it.

## Where they sit in the order

`PROVIDER_ORDER` is now NeuralWatt,
Synthetic,
Bedrock,
DeepSeek,
Xiaomi,
Hyper,
OpenRouter.
**This order is open to the owner's veto.**
The reasoning follows the Bedrock precedent:
Hyper's balance is never renewed and the Hyper-only seats run on it,
DeepSeek's prepaid balance was given to use "as you wish",
and Xiaomi bills per token and serves only the seat OpenRouter otherwise serves,
so both direct routes are tried before Hyper and OpenRouter spend money the owner tops up.
Neither is placed ahead of NeuralWatt's free daily energy.

## Meters

DeepSeek has a balance endpoint.
`GET /user/balance` is parsed strictly (`DeepSeekBalanceShapeError` for any body off the documented shape),
and the provider reads dry when `is_available` is false or the USD total is at or under `DEEPSEEK_DRY_FLOOR_USD`,
which is zero.
The floor has no margin above zero because its balance is prepaid,
one call costs at most a few cents at the peak rates the card prices,
and a balance too small for the next call surfaces as the 402 at the wire.
Bedrock alone carries a margin,
because its overshoot reaches the owner's card.

Xiaomi documents no balance or billing endpoint.
Its meter answers wet always and reports this process's priced spend,
so the `METERS` line shows `xiaomi=wet xiaomiUsdSpent=...`.
Exhaustion is learned from the 402.

Because that meter never reads dry by itself,
Xiaomi reads dry only once it has refused a call for payment.
Whether its wet reading hides a dry fleet depends on what it serves.

A wet provider counts against "every provider dry" only if it serves a model seated in a writing bench,
that is an editor,
a refiner or a translator,
so that a writing bench could still reach its floor from it.
The run-wide stop,
the router's all-dry hold wait and the seat reader's written refusal all read that one predicate.
**The predicate is open to the owner's veto**
(the lead's ruling of 2026-10-22 on the review's M3,
made after the review found the first form of D4 hid a fleet that could write nothing).
It is derived from the roster,
not listed:
`RUN_WRITING_PROVIDERS` in `run/src/corpus-run/run-config.ts` takes every model seated as an editor,
refiner or translator and marks the providers that serve it (`reachOf`),
and `createRoutingClient` receives the record as `writing`.
On the roster of 2026-10-22 every provider counts except Xiaomi,
whose only seat is a checker.
A funded Xiaomi serving only checkers therefore no longer keeps the run going:
the stop fires,
and its text says "Xiaomi reads wet but serves no seat that writes".
A roster that seats a writer on Xiaomi makes its wet reading count again with no edit.

Per-call routing is unchanged,
which keeps D4:
a model whose own providers are all dry while another provider is wet is a lost seat
(`NoProviderForModelError`),
and the run goes on.
Xiaomi,
wet,
is such a provider for the models it does not serve,
and it still buys MiMo V2.6 Flash for its own seat.
A funded real-meter provider that serves only some models (DeepSeek,
for one) behaves the same way:
it keeps its own wet reading for per-call routing.
The first form of D4 read Xiaomi as dry for a model it does not serve
(`dryForTheModelAsked` in `budget-routing.ts`,
added and removed on 2026-10-22);
that form stays removed.
To restore the stop for a funded Xiaomi,
change `routeProviderFor` in `provider/src/budget-routing.ts` and `readBudgetsPastHolds` in
`provider/src/budget-hold-wait.ts` to treat `writing` as every provider (`EVERY_PROVIDER_COUNTS`),
or revert the two commits that introduced D4:
the red test `11cddf0a37` and its fix `8a54f4c72e`.
Commits built on them since (the NeuralWatt return,
the park and the seat bounds) are not part of D4
and are reverted separately.
The `NoProviderForModelError` for a lost seat names the model and how each provider read
(`modelDarkReason` in `model-dark-reason.ts`),
and the router logs it as `<model> went dark: ...`,
once per model per 300 s.

## NeuralWatt's day start

A NeuralWatt reading that is dry because the day's energy is spent carries its return:
the UTC time of day the day starts again,
never midnight,
counted down on the budget layer's monotonic clock,
plus a margin of one second.
The case is XingZ6019 (2026-10-22):
NeuralWatt dry 48 s before its day started,
a writing bench short of its floor,
and the entry stopped 48 s before the energy came back.
The budget layer reports the return as a hold,
so every wait that already waits out holds waits for it,
and it reads the meter again at the return whatever the age of the cached reading.
Only the true cause is stated:
a blocked key states `keyBlockedUntil`,
energy held by calls in flight (the settled spend alone leaves more than the smallest estimate) names no return,
a day the account still shows spent after its return passed is the account lagging and names no return either,
and a refusal names the day start only when the day-start hold is the one that won.

**The bounds are open to the owner's veto.**
They are the lead's rulings of fix rounds 1 to 4,
not the owner's.
Three waits meet the return,
each bounded by what can cut it.

- A router call bounds the sum of its waits for a return at 300 s (`NEURALWATT_RETURN_WAIT_BOUND_MS`),
  the budget layer's own refusal cooldown (`REFUSAL_COOLDOWN_MS`).
  The 360 s per-call deadline (`RUN_PER_CALL_TIMEOUT_MS`) is not the reason:
  it is armed inside each client's per-model slot,
  after routing,
  and the router waits on the stage's round signal (the caller's abort and the straggler cut),
  so the deadline never reaches the router's wait.
  The bound is a sum:
  what the waits so far took comes off it before the holds are asked again,
  so a 300 s hold waited out leaves no room for a return 250 s away,
  and that return ends the call with the refusal at once.
  The distance is compared with its margin included (`returnFitsWithin`),
  the way the refusal's clause and the pass's park compare it.
  A return past the bound ends the call with the all-dry refusal,
  which states the time of day and the distance (`at 19:15:08 UTC (in 5 min 1 s)`)
  and carries the distance as `neuralwattReturnsInMs`.
  The distance stays on the refusal while the return still lies ahead,
  also after the call waited out another hold first,
  so the pass can park on it.
- The seat reader sits outside every exchange,
  so no call deadline applies to it.
  Its bound is what the pass has left:
  the shorter of the soft budget remaining and the hard cap,
  and inside an entry also what that entry's own hard cap has left,
  so a wait for a NeuralWatt return never outlasts the entry's timer (`withEntryBound` in `pass-entry-bound.ts`;
  the client carries the bound as `passLeftMs` and whose it is as `passBoundOwner`).
  Only a NeuralWatt return is filtered by the bound:
  a stated hold (Hyper's daily limit,
  a rate limit's)
  and a refusal cooldown are waited for whatever the bound is,
  and the entry's abort cuts them when the entry's timer ends first.
  A return inside it is waited for,
  as Hyper's stated holds are (923 s and 2 h 25 m on record).
  A return past it is refused at once,
  with the time of day in UTC and the distance stated and the note that it is past what the bound has left,
  naming the bound that won
  ("the entry has left" when the entry's own cap is the smaller,
  "this pass has left" otherwise),
  whichever refusal ends the reading:
  the bench error,
  the error line of a phase that goes on without its writers,
  the warning of a bench short of quorum,
  or the all-dry refusal,
  which also carries `neuralwattReturnsInMs`.
  Nothing here is bounded by 300 s.
  The reader waits only for a hold whose provider,
  back,
  would help cure a bench that stands short:
  the held providers are judged together,
  and a held provider counts when the benches they cure between them include one
  to which its own return adds a reachable seat
  (the seats are derived again with the held providers back,
  from the roster through `reachOf`,
  `run-seats-hold-need.ts`).
  Two holds that only cure a bench jointly are both waited for,
  and so are two holds that each cure it alone;
  an earlier test,
  "the bench stays short without this provider",
  counted neither of the second pair and seated the bench short without a wait.
  A counted hold that proves unneeded costs one extra read and no extra wall time,
  since the loop reads again after every wait.
  A hold on a provider that adds no seat to such a bench cannot bring it back and is not waited for,
  which includes DeepSeek on a wide bench the wet providers already reach through its one seat.
  When a bench is seated short,
  a warning names it,
  the reachable and needed counts,
  and the providers whose return alone would reach the quorum.
  When the reader does not wait,
  the line names the running holds it skipped and why
  (`held: xiaomi (2 h), deepseek (40 min); neither would help cure a short bench`),
  and says that no NeuralWatt return lies inside what the pass has left only when NeuralWatt is dry with no hold.
  The re-seat hook still reads the seats under any hold,
  since its reading is what re-seats a slice
  (a provider going dry changes the seating whether or not its hold would cure anything,
  so the hook cannot skip the read on "no running hold would cure",
  which is also the state of a hold with no short bench),
  and takes the same rule for its waiting through the reader.
  It writes the no-wait warning once per state,
  where a state is the set of held providers and the names of the benches that stand short.
  The time a hold has left,
  the distance to a NeuralWatt return and the figure past what the pass has left shrink between slices
  and never make a new line,
  so a hold of hours over hundreds of slices writes one line and not hundreds.
  A change of the held set or of the short benches writes one more,
  and so does a hold that ends and comes back,
  or a bench that recovers and falls short again under the same holds
  (a reading that finds no unmet bench,
  or that waited,
  forgets the last state).
  **This seat-reader bound is the lead's ruling after the first fix waited at any distance,
  its entry-scoped form the lead's ruling of fix round 4,
  the would-help rule,
  the skipped-holds line and the bound-owner wording the lead's rulings of fix round 5,
  and all are open to the owner's veto.**
- The pass parks when every provider reads dry and the return is known and inside what the pass has left,
  which is the shorter of the soft budget remaining and the hard cap (the park sits between entries on a fresh timer,
  so the entry's cap does not narrow it).
  The run ends only when the return is unknown or past what the pass has left,
  and the stop line states the return time.
  A seat reader's all-dry refusal for a return inside what the pass has left but past what the entry has left
  therefore parks the pass and ends nothing.
  The writing bench error parks the pass the same way when NeuralWatt's return would lift the bench to its floor
  (asked of the roster,
  `returnLiftsWritingBenches`,
  with no provider name list):
  it then carries the distance,
  the time of day and the spent day,
  the entry scheduler hands it to the gate,
  and the gate applies the same checks and the same requeue.
  A bench error that carries no return,
  or one the pass cannot wait for,
  ends its entry as before and never stops the run;
  its message says the pass may park and names that other outcome,
  since the gate parks only where the return fits what the pass has left.
  When slices of one entry fail with both the all-dry refusal and such a bench error,
  the overlapped map throws the refusal,
  which ends the run,
  whichever arrived first,
  and the bench error only when no refusal is among the failures;
  the map only hands the failure over,
  and the gate decides whether the pass parks.
  **The park on the bench error is the lead's ruling of fix round 5 and open to the owner's veto.**

The park works at the seam between entries,
because the wait must sit outside any exchange:

1. The dry gate (`pass-dry-stop.ts`) receives the refusal (or the bench error that carries the return),
   records a park instead of tripping,
   logs the start with the reason and the UTC return time,
   and aborts the shared signal so the entry in flight ends.
2. The queue (`runAttemptQueue`) sees the pass is parked,
   puts that entry back at the front with no verdict line,
   so none of `REATTEMPT`,
   `STALLED` and `INCOMPLETE` is printed,
   and calls the gate's wait before asking the budget whether the pass goes on.
3. The gate waits for what is left of the return,
   stored as an instant on its monotonic clock,
   counted from the read that named the return where the failure carries that stamp
   (the seat reader's bench error and its all-dry refusal both do,
   as `neuralwattReadAtMs`,
   so slices that unwind late do not push the wait past the return)
   and from the moment the failure arrived otherwise,
   as for the router's own refusal,
   which names the distance as it is raised,
   logs the end,
   and gives the pass a fresh signal.
4. A refusal for the day start of a park that has ended trips instead of parking again,
   since the account has then not rolled to the new day it should have,
   and parking again would wait a day in a loop.
   This replaces a 60 s window copied from the budget layer's freshness window,
   which was not the mechanism and let a refusal an hour later park for another day.

Known costs of the park:
the attempt it interrupts is already counted in the attempt map,
and its `TALLY` line says `PARKED`,
with `returnsInMs=` and `returnsAt=`
(an ISO 8601 time with `Z` for UTC,
one token)
before the reason,
so the cap on the reason never cuts them,
and not `ERROR` or `INCOMPLETE`.
The only writer of the time gives it as `<time> UTC`,
so the line keeps one rule for it,
and a time in any other zone throws `ReturnTimeNotUtcError` naming the text,
since stating it would need a conversion that nothing does.
The status comes from the gate's answer for that entry's own failure (`parkedFor`):
the gate keeps the failure it chose to park for,
and every other parking failure it sees while the park lasts
(attempts run one at a time,
so each belongs to the attempt that parked),
and answers true for those very objects only,
never from whether the gate happens to be parked or from the class of the failure.
It answers for all of them because the layers that choose which failure an entry ends with
(`runGatherRound` in `stage-round.ts`,
`image-reading-contain.ts`,
and `mapOverlapped` in `overlapped-map.ts`)
choose by another order than the failures' arrival at the gate:
the order in which the asks caught them,
the roster slot order,
and the order of insertion with the run-ending failures searched first.
The entry may therefore end with a parking failure that was not the first the gate saw.
This was the lead's ruling after the eighth re-check,
open to the owner's veto.
When the budget ends the pass after the wait,
the queue prints `NOT RESUMED <id>` for the entry the park cut short,
so the log names the entry a later pass must resume.
Nothing in the code reads the printed status:
the scheduler works from the disposition it is returned,
and the tests match lines by prefix,
so the status is read by people and by the runbook only,
and `PARKED` is neither a failure nor a settlement.
A class tested with `instanceof` across packages is the same class only when the importing package lists its owner under
`dependencies`,
since a library build bundles every other workspace import
and a copy is not an instance of the real class.
`bundled-class-identity.unit.test.ts` in the source-scan package fails on a class test against a package the importer
does not keep external,
and slice-cache therefore asks stage's `isDryRefusal` rather than testing `EveryProviderDryError` itself.
A long NeuralWatt hold also makes the per-slice re-seating (`pass-reseat-hook.ts`) read the seats on every slice
while it runs,
as it does for any hold.

How the hold behaves,
in both directions (pinned in `provider-budget-xiaomi-hold.unit.test.ts`):
the budget layer clears a payment mark when the meter's level text changes,
and the level here is the run's own spend.

- With no in-flight call settling after the 402 the spend does not move,
  so the refusal holds Xiaomi dry for the rest of the process.
  That is the current behaviour,
  and the precondition matters:
  in a busy run every Xiaomi call already in flight settles after the refusal and moves the spend,
  so the hold clears early (next item) and Xiaomi flaps rather than staying out.
  The process cannot know that the account was topped up,
  so a top-up within a run is seen on the next launch.
  Xiaomi's error list names 402 as "Insufficient Balance"
  but this change did not check what else may return 402.
- A call that was already in flight settles after the refusal and moves the spend,
  which clears the mark early and lets one more call through to meet the refusal again.
  The cost is one extra refused request per such settle.

A run without a key logs `deepseek=dry` or `xiaomi=dry` on the `METERS` line,
as every unconfigured provider does,
so the duty cycle `meter-report` prints for a provider the run never had reads as fully dry.
Logs written before today carry neither field,
and the reader (`meter-sample-read.ts`) reads them as absent.

## Keys

`DEEPSEEK_API_KEY` (the owner's name,
without the package prefix) and `TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY`.
Both are optional.
An absent key leaves the provider unconfigured,
with one info line naming the variable,
printed after the no-key refusal so a launch with no key at all still prints its refusal alone.
No key falls back to another provider's.
A 401 from either provider reaches the log without the provider's own words,
because a refusal of a key may echo a fragment of it (DeepSeek's documented text shows the last four characters),
the transport's mask finds whole credentials and not tails,
and masking every four-character piece of a key would rewrite ordinary words in answers that carry no refusal.
The status alone is what the router and an operator act on (`withoutRefusedKeyBody` in `provider/src/direct-client.ts`).
Neither is a valid `--require-providers` value:
no arm can depend on an optional route being ready.
Both names are in the no-key error text,
the budget sample,
the roster-card tool and the tests' named cases;
the child-environment scrub and the keyless scans match the `_API_KEY` suffix,
so both are covered there without a list.

## Prices

DeepSeek is priced at peak always:
0.006 (cache hit),
0.3 (cache miss) and 1.2 (output) USD per million tokens,
with the hit and miss split read from the usage block.
The page halves every rate off peak (01:00 to 04:00 UTC and 06:00 to 10:00 UTC on weekdays other than Chinese public holidays are peak),
and a run cannot know the holidays,
so a peak price is an upper bound and never an under-read of the balance.
Xiaomi is 0.0028 (cached),
0.14 and 0.28 USD per million tokens,
with the cached count read from the usage block.

## The Xiaomi token cap

Xiaomi's chat reference lists `max_completion_tokens` and not `max_tokens`,
and the shared wire body sends `max_tokens`.
It is kept,
because the parent session measured it on 2026-10-22 with the owner's Xiaomi key
(a probe this change did not run or repeat).
On `mimo-v2.6-flash` with a cap of 64,
both members returned `finish_reason` "length" with `completion_tokens` 64
(`reasoning_tokens` 22 included).
The two result lines,
as the probe wrote them:

```jsonl
{"field":"max_tokens","status":200,"finish":"length","usage":{"completion_tokens":64,"prompt_tokens":28,"total_tokens":92,"completion_tokens_details":{"reasoning_tokens":22},"prompt_tokens_details":{"cached_tokens":0}}}
{"field":"max_completion_tokens","status":200,"finish":"length","usage":{"completion_tokens":64,"prompt_tokens":28,"total_tokens":92,"completion_tokens_details":{"reasoning_tokens":22},"prompt_tokens_details":{"cached_tokens":0}}}
```

The limit of this evidence:
it is one non-streamed call pair at a cap of 64.
The production body streams with `stream_options`,
and no call has shown the member honoured on a stream or at the real cap (13,082 style values).
The first streamed Xiaomi call of a real run should show `completion_tokens` at or under the cap on its `SPEND` line.
The probe's output file is kept in the parent session's scratch area (`w287-xingz18/xiaomi-max-tokens-probe.out`)
and is a convenience only:
this section stands without it.

## No zero data retention

Neither route promises zero data retention.
The owner's ruling of 2026-10-05 (the corpus is public,
PII is allowed) and the two grants are what permit it.
The OpenRouter route of Xiaomi's model sends `zdr: true` and is behind this one in the order.

## The 1017 limit

The grant names "deepseek v4.1 flash 1017".
The client cannot ask for a build:
`deepseek-flash` is the only wire id,
and the claim that it is the 1017 build rests on DeepSeek's documentation,
with `system_fingerprint` logged per call as the only evidence a reader can compare across days.
This change made no call with a key to check it.

## Not done

- **Undecided:
  recovery from a Xiaomi 402 within a run.**
  The owner has not ruled on it.
  If a run should recover (a top-up,
  or a 402 that was transient),
  the Xiaomi meter level needs a coarse time bucket so the mark clears on a schedule.

- This change used no real key and sent no request:
  every test runs over a recorded transport,
  and neither `roster-card` listing address was fetched (DeepSeek's rests on its documentation,
  Xiaomi's on the OpenAI-compatible protocol's).
  The keyed probes this record cites were the parent session's,
  not this change's.
- The rotation sweep and the gate were not run.
- The cap census maps both providers' served ids to their seats,
  but no call has been counted.
