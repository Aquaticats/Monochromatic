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
`shared-contract/src/direct-cards.unit.test.ts` and `provider/src/direct-client.unit.test.ts`
fail when any of the three is removed.

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
- Xiaomi serves `mimo-v2.6-flash` (roster seat `xiaomi/mimo-v2.6-flash`)
  on `https://api.xiaomimimo.com/v1/chat/completions`.
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

**NeuralWatt's day start is never waited for.**
The owner answered on 2026-10-29.
The question was whether a run should wait for NeuralWatt's 550 Wh day to start again at 19:15:08 UTC
when it is spent and other routes are empty.
A bounded wait was then in force:
one call waited at most 300 s in total for a return,
the seat reader waited for a return that fitted the time the pass and the entry had left,
and between entries the pass parked,
logged `PARKED` and resumed.
The owner chose
"Never wait,
relaunch":
"Stop at once and relaunch later."
He had said on 2026-10-22,
"You don't need to align launches to NeuralWatt starts",
and the NeuralWatt subscription reads `canceling`,
its period ending 2027-01-20 19:15:08 UTC.
The router's 300 s bound,
the seat reader's pass and entry bound,
the park and its `PARKED`,
`NOT RESUMED` and `the pass resumes` lines are therefore removed.
The budget layer names no hold for the day start,
a call whose only way forward is the day start refuses on its first reading,
and the refusal states the time of day in UTC and the distance (`at 19:15:08 UTC (in 5 min 1 s)`),
counted down from the reading that named it.
Holds of other providers (a refusal cooldown or a stated `Retry-After`) are waited for as before,
and so is a refusal cooldown on NeuralWatt itself.
The seat reader waits only for a hold whose provider,
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
  and says that NeuralWatt's day return is not waited for only when NeuralWatt is dry with no hold.
  The re-seat hook still reads the seats under any hold,
  since its reading is what re-seats a slice
  (a provider going dry changes the seating whether or not its hold would cure anything,
  so the hook cannot skip the read on "no running hold would cure",
  which is also the state of a hold with no short bench),
  and takes the same rule for its waiting through the reader.
  It writes the no-wait warning once per state,
  where a state is the set of held providers and the names of the benches that stand short.
  The time a hold has left shrinks between slices
  and never makes a new line,
  so a hold of hours over hundreds of slices writes one line and not hundreds.
  A change of the held set or of the short benches writes one more,
  and so does a hold that ends and comes back,
  or a bench that recovers and falls short again under the same holds
  (a reading that finds no unmet bench,
  or that waited,
  forgets the last state).
The writing bench error never carries a return and never stops the run:
it ends its entry as a failure a later pass resumes.
When slices of one entry fail with both the all-dry refusal and another failure,
the overlapped map throws the refusal,
which ends the run,
whichever arrived first.
The dry gate (`pass-dry-stop.ts`) trips on the refusal at once,
logs the cause,
and aborts the shared signal so the entry in flight ends;
the queue starts no further entry,
and the stop line carries the refusal's message,
which states the time of day in UTC NeuralWatt's day starts again.
Nothing puts an entry back to wait for it.
The mechanisms that did were removed with the owner's answer of 2026-10-29:
the park,
`ParkAnswer`,
`ReturnTimeNotUtcError`,
`parksTheRun`,
the entry bound `withEntryBound`,
and the seat reader's `passLeftMs` and `passBoundOwner`.
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

DeepSeek is priced at the rate of the hours of its start and its completion.
The card holds the peak rates:
0.006 (cache hit),
0.3 (cache miss) and 1.2 (output) USD per million tokens,
with the hit and miss split read from the usage block.
The page halves every rate off peak
(01:00 to 04:00 UTC and 06:00 to 10:00 UTC on weekdays other than Chinese public holidays are peak,
and every other hour is off peak,
weekends and Chinese public holidays in full).
A call is booked at the peak rates when its request started,
or its reply completed,
inside one of those windows on a Monday to Friday in UTC,
and at half of them only when both instants lie outside every window.
Each window is read as half-open
(01:00:00 UTC and 06:00:00 UTC are peak,
04:00:00 UTC and 10:00:00 UTC are off peak),
because the page does not say whether its last instant belongs to a window.
The page does not say either whether the start or the end of a request decides its rate,
so a call that starts or completes inside a window is booked at peak,
and a call that spans a whole window with both instants outside is booked at half,
which is wrong only if DeepSeek bills by an instant between the two.
The client reads the clock it is given
(`now` of `createDeepSeekClient`,
the system clock by default)
before the first attempt and again when the attempt that succeeded has completed,
so a retried call is bracketed by its first start and its last completion.
The run reads no holiday calendar,
and a holiday only makes a weekday window cheaper,
so the booked figure is an upper bound as to holidays and as to which instant decides.
It is not bounded as to a Saturday that China makes a working day,
which the page books off peak and DeepSeek may bill at peak,
nor as to a call that spans a whole window with both instants outside it.
This replaced "peak always" on 2026-10-25,
after the XingZ6021 run (Saturday 22:49 UTC to Sunday 00:55 UTC,
off peak throughout)
booked 0.147093 USD while the balance fell 0.07.
SPEND lines and run meters written before this change book every DeepSeek call at peak,
so they over-read off-peak calls by up to twice.
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

## Addendum: the owner's Claude key (2026-10-22)

The owner,
2026-10-22:
"I've added TRANSLATION_REPAIR_CLAUDE_API_KEY .
You may use latest Haiku and latest Sonnet with this key.
This key would have $200 every month so be careful what you use."
Seats:
"Sonnet edits,
Haiku wide".
Effort:
"Why don't you bench?"
Code:
`provider/src/claude-*.ts` and `shared-contract/src/claude-card-sides.ts`.

- The key lists Opus,
  Fable and older models too,
  so the lock is held four ways:
  the wire model of every card is one of two literals (`ANTHROPIC_WIRE_MODELS`),
  `claude-cards.unit.test.ts` fails on a third,
  `cardFor` in `claude-client.ts` throws before a request is built,
  and `roster-card` refuses the Claude key by name.
- One card per model and effort (`claude-sonnet-5-5`,
  `claude-sonnet-5-5-effort-low` and so on),
  eight in all.
  The cards named by the model alone send no effort,
  which is the model's own default.
  The six effort cards carry `judge-unmeasured`,
  so no derived bench seats them until the effort bench names a level.
  Two cards of one model share a wire model,
  so no bench may seat two of them together:
  `assertOneWireModelPerBench` (`shared-contract/src/bench-wire-model.ts`) refuses it,
  and `run-config.ts` calls it at load on every list in `RUN_BENCHES`.
- Seats in `run/src/corpus-run/run-config.ts`:
  Sonnet 5.5 in `editorModelIds` ahead of `glm-5.3`,
  Haiku 5.5 in the wide seats through its card holds
  and in `RUN_CHECKER_ORDER` behind the three the 2026-09-27 benchmark ranks,
  so it fills the checker bench only when a provider serving one of those three is dry.
  The editor calibration (`editor-calibrate`) seats Sonnet 5.5 by name beside the roster's writers
  and gives Haiku 5.5,
  which holds `writer-unmeasured`,
  no editor or refiner chair.
  That second part is the narrower reading of "Haiku wide" and is open to the owner's veto.
- Sonnet 5.5 cache reads are priced at 0.10 USD a million,
  from the owner's brief of the pricing page.
  The API skill bundled with the session lists 0.20 for the same line.
  The difference is on cache reads only.
- `PROVIDER_ORDER` places `anthropic` after `xiaomi`.
  That position is open to the owner's veto and has no routing effect,
  since every Claude card is served by this provider alone.

### Not done

- No call with the key was made:
  every test runs over a recorded transport.
  Where a refusal's `stop_details` sits on a stream is not documented,
  so `claude-stream-facts.ts` looks in the delta,
  on the frame and in the message.
- Whether `output_config.format` accepts every stage's schema is still unverified live.
  The bundled API skill says every object needs `additionalProperties: false`
  and that the numeric,
  string and array bounds are not supported;
  `claude-schema.ts` sends exactly that,
  `corpus-pass/src/claude-schemas.unit.test.ts` reads all 23 schema-declaring files through the body builder,
  and the block pairing and contest ballot schemas were closed at source.
  The one bound dropped anywhere is `minimum: 1` on a naturalness finding's paragraph,
  which `isAbsoluteNaturalnessReviewWire` rejects below one.
  One owner-run probe per distinct schema shape would close it.
- Thinking streams only if `display: "summarized"` does what the skill says.
  The default `omitted` leaves the stream silent while the model thinks,
  which the idle guard (`STREAM_FIRST_CONTENT_MS`,
  180 s)
  would read as a stall and cut,
  and every attempt that delivered something is billed.
  The request now asks for the summary,
  the skill says billing is the same under every setting,
  and both are unverified here:
  whether summary text arrives in time to reset the guard,
  and whether a think longer than the window still produces no `thinking_delta` before its summary is ready.
  If a cut still shows in the first live log,
  the alternative is a `firstContentMs` on the Claude cards above the measured think time.
- `max_tokens` is 32,000 on every Claude card,
  a starting value:
  thinking counts against it on both models,
  and the pooled p99 of 13,082 measured on other providers could be spent thinking.
  A call that runs to it costs at most 0.32 USD on Sonnet.
  The cap census reads the `SPEND` lines of the first live calls and flags the card when its own p99 disagrees.
- Each stage's system prompt is marked for the five-minute cache on every call.
  A write costs 2.50 against 2.00 base on Sonnet,
  and pays back only if the same prefix is read again within five minutes.
  That holds for instructions that repeat and not for a system text that embeds per-call content,
  and no test shows which stages are which.
  The `cached=` field of the first live `SPEND` lines settles it.
- The effort bench and the rotation sweep were not run.

## Addendum: the keyed Claude probes (2026-10-23)

The lead ran `probe3.ts` against the owner's Claude key with `claude-haiku-5-5`,
three times:
26 calls on 2026-10-22,
30 calls on 2026-10-23,
which cost an estimated 0.00215 USD and sent no baseline,
and 34 calls on 2026-10-23,
which cost an estimated 0.0023 USD and sent a baseline for each accepted control.
The figures of this section are those of the third run.
The probe sits outside this repository,
so this record holds what it measured.
The route's source comments cite this section.

### What the probe shows

Each accepted control was sent twice,
once with the keyword and once as a baseline with the keyword removed from the schema.
The ask told the model to break the keyword in both.

- `pattern` (three capital letters):
  with the keyword the reply was `{"code":"CAT"}`,
  without it `{"code":"cat"}`.
- A 25-label `enum`:
  with it the reply was `LABEL_25`,
  and its `reason` text said the asked-for `OTHER` was not among the allowed values;
  without it the reply was `OTHER`.
- `minItems` 1:
  with it the reply held one item,
  without it `{"items":[]}`.
- A listed `format` (`date-time`):
  with it the reply was `7681-07-01T00:00:00Z`,
  without it `banana`.
- `minItems` 2 and an unlisted `format` (`color`) were sent raw,
  and each was refused with HTTP 400 and `invalid_request_error`.
  The `minItems` message was
  `output_config.format.schema: For 'array' type, 'minItems' values other than 0 or 1 are not supported (got: [2, 5])`.
  The `format` message was
  `output_config.format.schema: For 'string' type, format 'color' is not supported`.
  Both refusals agree with the page,
  and neither reached a model.
- All 24 formats of the closed-vocabulary census answered HTTP 200.
  The probe's validator was a plain object check and not the stage guards.

So in this sample a keyword changes the reply.
Each of the four contrasts rests on one reply with the keyword and one without,
and the probe passes no sampling setting beyond the output limit of 512 tokens
(no `temperature` appears in `probe3.ts`),
so the API's default sampling applied.
Four pairs are a pattern and not a measurement of how often a reply obeys.
That is all the probe shows.

### What it cannot show

The probe cannot tell a compiled grammar from the model reading the keyword.
The input tokens of each control exceeded those of its baseline.
The excess is our own measurement,
control minus baseline with both arms structured,
and the page does not account for it:
the page says nothing about what a grammar costs in input tokens.
The excess was 20 for `pattern`,
261 for the 25-label `enum`,
9 for `minItems` 1 and
10 for `format`.
Our inference is that the keyword text reaches the model,
because the excess grows with the keyword text (261 tokens for 25 labels).
If it does,
the model reading it explains every reply in the list of results without any grammar.
The reason text of the `enum` reply reads as the model quoting the schema,
which is consistent with that inference and does not prove it.
The probe therefore does not show that the API enforces any keyword.
A reply that obeys a keyword once is no guarantee for the next reply,
and the stage guards remain the check.

### The documented mechanism

<https://platform.claude.com/docs/en/build-with-claude/structured-outputs>,
read 2026-10-23.
Quoted:

- "Structured outputs guarantee schema-compliant responses through constrained decoding"
  (section "Why use structured outputs").
- "Structured outputs use constrained sampling with compiled grammar artifacts."
  (section "Grammar compilation and caching").
- "When using structured outputs,
  Claude automatically receives an additional system prompt explaining the expected output format."
  and "Your input token count is slightly higher."
  (section "Prompt modification and token costs").
  The page gives this as the reason a structured request costs more input tokens than one without structured outputs.
  It does not account for the excess measured in this section,
  where both arms were structured.
- "Although structured outputs guarantee schema compliance in most cases,
  there are scenarios where the output might not match your schema"
  (section "Invalid outputs"),
  which names refusals and the token limit.
- "**API-enforced constraints** (sent in the schema):
  `description`,
  `format`,
  `const`,
  `itemClass`,
  `minItems` (0 or 1 only)."
  This sentence sits in the PHP tab of the page,
  under the accordion "Constraints with the #[Constrained] attribute",
  so it describes that SDK's attribute handling and not the API reference.
  It is the sentence of the page that uses "enforced" of `format`,
  `const` and `minItems`,
  and it was fetched on 2026-10-23.
  It names which constraints the PHP SDK sends in the schema,
  and the probe cannot say whether the API then enforces them by grammar or by the model reading them.
- "Structured outputs don't guarantee the capitalization of string `enum` and `const` values:
  Claude might return a value that differs from your schema only in capitalization,
  typically in the first letter of a word following a space."
  "The response completes normally,
  with no error and no special `stop_reason`."
  "Compare enum values case-insensitively,
  and avoid enum values that differ only in capitalization."
  (section "Invalid outputs").

The mechanism is the documented grammar,
and the probe is consistent with it and does not prove it.

### What the route does with this

- Source comments that cited the probe say a keyword "changes the reply" or "is accepted",
  and say "enforced" only of the stage guards,
  which are our own code.
- The page's capitalization caveat is why `foldEnumCase` (`provider/src/enum-case-fold.ts`) takes the schema's spelling
  when exactly one listed value matches case-insensitively.
  It logs a warning naming the field,
  the value received and the value taken,
  and leaves the refusal in place when none or several match.
  The fold runs in `readJsonOutcome` for the four sites that read a reply under a response format.
- No troubleshooting document under `doc/troubleshooting/` is cited by the Claude route's source or notes,
  so this section is the one record of the keyed result.
