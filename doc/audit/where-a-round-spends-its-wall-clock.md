# Where a round spends its wall clock

## Current status, 2026-08-29

This audit is historical evidence from earlier roster,
models,
calibration overlap,
and waiting policy.
Generation 13 starts every production grace window at exact-half participation
and continues naturalness correction without finite quality ceiling.
Historical statement that grace buys every seat no longer describes required participation.
Grace can still retain additional usable evidence until bound expires.

Current entry-level performance measurement is
`doc/planning/translation-repair-entry-time-to-complete.md`.
Generation-12 `Weideriche_` and pull-request 386 runs were stopped and are partial traces,
not matched completion arms.
Fresh Carena median below two hours is objective without weakening quality.

Measured 2026-08-25 against the second editor calibration while it was still running,
at 52 rounds of an expected 250 or so.
Every number here is provisional and must be re-read when the run finishes.
The shape is already unambiguous,
and it points somewhere different from where `#213` was looking.

Read with `run-timing-report`,
which `#215` added and which had never been run on a live log:

```text
run-timing-report: 1 logs, 1997 lines
rounds                 51, 1.63h in total
  waiting after quorum 1.44h, 88.3% of round time
  voices never heard   15
calls in flight        mean 2.56, peak 10
  busy against span    4.22h of calls across 1.65h of run
```

## The two numbers that decide it

ROUNDS DO NOT OVERLAP.
Round time sums to 1.63 hours inside a run spanning 1.65 hours.
Nothing else is happening while a round waits.

A ROUND SPENDS 88.3% OF ITS LIFE PAST QUORUM.
Ten voices are dispatched together,
which the peak of 10 in flight confirms.
Most arrive quickly,
quorum is reached,
and then the round waits a median of 99 seconds
for the last one or two.
Mean calls in flight is 2.56 because that is what the round looks like most of the time:
eight seats finished,
two still running,
nothing else queued.

## So fan-out inside a round is not the lever

`#213` opened on eleven `no-await-in-loop` disables resting on a premise about per-model
concurrency,
and asked whether removing the serialization would speed the pipeline up.

For the stage round itself the answer is no,
and the reason is that the serialization
was never there:
a round already dispatches all ten seats at once.
Raising per-model concurrency cannot help a round that is waiting on one model's single call.

The lever is between INDEPENDENT UNITS OF WORK,
so that one unit's straggler wait is
another unit's dispatch window.
That is the same site `#213` had already singled out as having a clean argument,
`editor-ensemble.ts:248`,
on grounds that survive this measurement:
envelopes within a slice are independent,
nothing in the loop reads back what an earlier
envelope decided,
and `Promise.all` preserves order,
so folding the outcomes in index
order afterwards gives byte-identical output.
It is a pure latency change with no quality risk,
which under a max-quality guideline is
the only kind worth taking.

## The straggler window is close to right, and shortening it is expensive

`#214` asked whether 180 seconds is too generous.
Measured against what each shorter window would newly have cost,
counting only rounds
that did hear everyone,
since a round already at the wall loses nothing extra:

```text
  window   rounds that would newly lose a voice, of 52
     15s   40
     30s   38
     45s   32
     60s   27
     90s   18
    120s   12
    150s    4
    180s    0
```

Grace spent per round:
min 10.9s,
p25 53.2s,
median 98.8s,
p75 151.0s,
p90 180.0s.
No round finished within five seconds of quorum.
There is no fast mode to cut back to.
Ten of 52 rounds spent the whole window and still lost a voice.

So the window is not waste to be reclaimed.
It is the price of hearing every seat,
and the distribution says the price is real.

## The cost is concentrated in two models

Mean whole-call latency over completed streams,
all ten seats:

```text
   68286ms  qwen3.8-max                                        n=48  max 186808ms
   61518ms  hf:zai-org/GLM-5.2                                 n=48  max 185281ms
   39712ms  deepseek-v4-flash-0731                             n=53  max  83782ms
   38472ms  hf:Qwen/Qwen3.8-27B                                n=53  max 163304ms
   14982ms  minimax-m3                                         n=53  max  53809ms
   13900ms  hf:nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-NVFP4  n=53  max  76187ms
   12172ms  hf:moonshotai/Kimi-K3                              n=53  max  53964ms
   11379ms  gemma-4-26b-a4b-it                                 n=52  max 135918ms
    5202ms  deepseek-v4-pro-0813                               n=53  max 117479ms
    4190ms  hf:openai/gpt-oss-120b                             n=53  max  10526ms
```

A sixteenfold spread from fastest mean to slowest.
Ten stream cuts in the whole log,
and nine of them are the top two:
five `hf:zai-org/GLM-5.2`,
four `qwen3.8-max`,
one `gemma-4-26b-a4b-it`.

Fifteen voices were never heard against ten cuts,
so five went missing some other way
than the stream being cut.
Recounted at 54 rounds the numbers are 16 and 11,
and the gap of five closes exactly against the warning lines:

-   3 `MalformedCompletionError` on `panel gemma-4-26b-a4b-it`,
    a body that failed the
    protocol contract rather than a stream that ran out of time.
-   2 schema mismatches at `critic`:
    one on `qwen3.8-max` where the content was not valid
    JSON,
    one on `gemma-4-26b-a4b-it` where it parsed and failed the caller's guard.

So a voice is lost three ways,
not one,
and only the first is what the window governs.
Shortening the window does nothing about the other two.

### The gap also exposed a diagnostic that named the wrong provider

The `gemma-4-26b-a4b-it` warning read
`MalformedCompletionError: Synthetic completion body violated the OpenAI-compatible contract`,
and every one of that model's 51 billed calls in this run went to Charm Hyper.

The log could not settle it alone,
since a failed call bills no `SPEND` line,
so the deciding source was read instead:
`hyper-client.ts` imports `SyntheticHttpError` (renamed `ProviderHttpError` on 2026-10-19) from `completion-shape.ts`
and documents throwing `MalformedCompletionError`.
Both providers throw both classes,
and both messages named one of them.

`hyper-client.ts` already records why it reuses the class and that the rename is held
with the `SyntheticClient` rename
(`SyntheticClient` was renamed `ModelClient` on 2026-10-19),
so nothing was renamed.
That note covers the NAME,
read by whoever edits the file.
The MESSAGE is read by whoever is holding a broken run,
and is now provider-neutral,
with the two meter endpoints naming their own provider through the `summary` seam
that already existed for it.
Fixed in `4af3068fa`.

### The first of those two is a defect, not weather (`#228`)

Recounted at 768 streams the classes stand at 13 cuts,
4 `MalformedCompletionError`
and 2 `SyntaxError`,
still `panel gemma-4-26b-a4b-it` every time.

The message says `anthropic stream ended without message_stop`,
which is a stream the provider truncated,
arriving as HTTP 200.
Both clients extracted the completion AFTER `exchangeWithRetry` returned,
and 200 is not a retryable status,
so the ladder handed the reply straight back
and extraction threw with nothing left that could re-dispatch it.
Every other transport failure carries a status the ladder recognises;
this one wore a success status,
which is why it was invisible.

Fixed in `38a5178d7` by running the terminator check inside `attemptExchange`'s own try,
so a truncated body lands in the same catch,
behind the same `isSelfEndedStream` predicate,
as a dropped connection.
`doc/handover/translation-repair.md` carries the measurement.

So of the three ways a voice is lost,
the window governs one,
the second was a defect
and is now retried,
and the third,
a critic answer that fails its schema,
is still open.

## What `#211` actually bought

The earlier reading in `doc/audit/every-volume-guard-is-blind-to-one-model.md`
had 34 voices cut over 29 straggler events,
21 of the 34 `qwen3.8-max`,
in a 3.73 hour run.
`#213` said to land `#211` first because it might remove most of the straggler cost by itself.

It did not,
and it did help.
`qwen3.8-max` falls from 21 of 34 cuts to 4 of 10.
It is no longer the whole story;
`hf:zai-org/GLM-5.2` is now the larger share.

## Two measurement traps this walked into

BOTH ARE THE SAME TRAP,
a pattern that matched less than it appeared to.

A first pass counted stream outcomes with `stream [^:]+:`,
which cannot cross the colon
in `hf:zai-org/GLM-5.2`,
so every Hyper-side seat was silently excluded.
It reported five cuts across five models rather than ten across ten,
and the missing half was exactly the half carrying the most cuts.
The tell was arithmetic that did not close:
rounds reported ten seats and the model
table listed five.

A first pass at grace also grepped `[0-9]+ ms in grace`,
with a space.
The emitter writes `30001ms in grace`,
so nothing matched,
and the surrounding
`round` grep instead matched judges' prose containing the word.
Anchoring on the emitter's own template rather than on remembered wording fixed it.

## The zero-content count is zero, and the probe was proven able to see one

`#221` was opened to recount zero-content streams,
with `deepseek-v4-pro-0813` at
36 of 356 completed streams as the number to beat,
and an expectation that `qwen3.8-max` would stay high as an accounting artifact:
it is the sole `toolChoice: 'auto'` seat,
and an answer arriving as tool-call arguments
is not counted by `generatedChars.content`.

Both halves are refuted on this workload.
Across 527 model streams,
ZERO have no content.
The minimum is 3 characters,
p10 is 42,
the median is 350.

POSITIVE CONTROL FIRST,
because a null from an unvalidated probe means nothing.
The same probe,
run without the filter that drops the two meter endpoints,
reports `quotas` at 47 of 47 zero and `credits` at 47 of 47 zero,
which is correct:
they are not model calls and produce no content.
So the probe can see a zero.
There are none among the seats.

Content against reasoning,
which is where the old count was going wrong:

```text
  deepseek-v4-flash-0731                             content  443   reasoning     0   on  0 of 54
  gemma-4-26b-a4b-it                                 content  429   reasoning     0   on  0 of 53
  deepseek-v4-pro-0813                               content  436   reasoning   708   on  3 of 54
  hf:openai/gpt-oss-120b                             content  302   reasoning  2827   on 54 of 54
  hf:moonshotai/Kimi-K3                              content  440   reasoning  1867   on 54 of 54
  hf:nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-NVFP4  content  397   reasoning  7822   on 54 of 54
  minimax-m3                                         content  490   reasoning  7996   on 54 of 54
  hf:zai-org/GLM-5.2                                 content  472   reasoning  9222   on 48 of 49
  qwen3.8-max                                        content  320   reasoning 10948   on 48 of 48
  hf:Qwen/Qwen3.8-27B                                content  411   reasoning 14419   on 54 of 54
```

`deepseek-v4-pro-0813`,
the model that was 36 of 356,
is the one that now emits a
reasoning channel on 3 calls out of 54.
That is consistent with `#211`:
a ballot filed as reasoning leaves the content counter
at zero while the model in fact answered.
`qwen3.8-max` answers on the content channel every time despite being the `auto` seat,
so the predicted accounting artifact is not present here either.

WHAT THIS DOES NOT SHOW.
One workload.
An editor calibration asks for short ballots,
median 350 content characters,
where a corpus pass asks for whole slices and runs
stages this calibration never touches.
A clean count here is not a clean count for the pass.

## What is still owed

-   Re-run every number here when the calibration finishes,
    at roughly five times the rounds.
-   Prototype the `editor-ensemble.ts` change and measure it,
    rather than reasoning about it.
-   Recount zero content on a corpus pass,
    which asks for whole slices
    rather than short ballots and runs stages this calibration never touches.
-   Recount the three loss classes on a run built after `38a5178d7`.
    The truncation class should fall to whatever survives four retries,
    and any residue is a different defect than the one just fixed.

## A lost voice is a model still thinking, and two volume hypotheses died on the way there

Measured 2026-08-25 at 947 completed streams and 19 cut ones,
same run.

18 OF THE 19 CUT STREAMS HAD PRODUCED ZERO CONTENT CHARACTERS.
The one exception had 301.
So at the moment the window closed,
the model had not begun its answer;
it was still in the reasoning channel.
Waiting longer would not have salvaged a partial answer,
because there was no answer yet.

That refines what `#214` measured rather than contradicting it.
Shortening the window costs voices because the window is buying THINKING TIME,
not answer time.

### Two hypotheses this refuted first, both mine

VOLUME RUNAWAY.
The abandon line reads `cut-mid-reply after 306685 delivered chars`,
which looks like a model that would not stop.
It is not:
`partialText.length` counts RAW WIRE characters,
frames and JSON envelopes
included,
and a completed stream's median is 49595 with a maximum of 3627449.
Every cut stream is smaller than the largest completed one.
Raw volume does not separate the two populations at all.

A REASONING BOUND.
If cut streams were thinking runaways,
a bound would catch them:

```text
  cut streams        reasoning  median  30984   max  41953   n=19
  completed streams  reasoning  p99     35113   max  60674   n=947
```

The distributions overlap almost entirely.
A bound low enough to catch the cut ones kills more than one percent of completed streams,
and a bound high enough to be safe catches none of them.
So `#156`'s decision to decline a reasoning bound is CONFIRMED by this workload
rather than overturned.

### A recorded prediction that did not hold

`doc/handover/translation-repair.md` records `#211` predicting that post-fix bytes would
meet `contentCap`,
cutting a runaway at roughly 32000 rather than 300000,
and it says in the same breath to record the outcome rather than the prediction.

The outcome:
`qwen3.8-max` is still abandoned after 283620 to 345064 raw characters,
the same tight cluster as before the fix.
The reason is legible in the source rather than the log.
`maxAnswerChars` is passed by `translate-produce.ts` and `translate-repair.ts` and by nobody
else,
so `contentCap` is undefined for every judging stage,
and the 32000 bound `#156` set has never applied to a panel,
editor,
critic or select call.

### Where that points

Neither request builder sends any reasoning control.
Searching `src` for `thinking`,
`reasoning_effort`,
`reasoningEffort` and `budget_tokens`
returns scanners and comments only.
This pipeline has never asked a model to think less.

Filed as `#229`,
with the provider research owed first,
and with the measurement that decides it named in advance:
ballot agreement with and without a budget on the same slices.
Those seats contribute nothing today,
so the comparison is a possibly shallower vote
against no vote at all,
but that is a thing to measure rather than to assume.

### The parameter is real, and `low` is not the setting to reach for

Probed 2026-08-25 against the live Synthetic endpoint,
twelve calls in total.
`dev.synthetic.new/docs/openai/chat-completions` documents `reasoning_effort` as
"Control reasoning effort for thinking models:
low,
medium,
or high" and names no models,
so it was tested rather than believed.

POSITIVE CONTROL FIRST.
`hf:openai/gpt-oss-120b` exposes reasoning effort natively,
so a probe that cannot move that seat cannot show a change at all:

```text
  hf:openai/gpt-oss-120b   absent  338 reasoning chars   117 completion tokens   1974ms
  hf:openai/gpt-oss-120b   low      48 reasoning chars    33 completion tokens   1087ms
  hf:Qwen/Qwen3.8-27B      absent  306 reasoning chars    88 completion tokens   1693ms
  hf:Qwen/Qwen3.8-27B      low       0 reasoning chars     9 completion tokens   1154ms
```

So Synthetic honours it.
That was the question this needed answered and it is answered.

On a JUDGING-SHAPED task,
choosing between three renderings of an invented cat-themed
sentence,
which is the kind of question a panel seat actually gets:

```text
  hf:Qwen/Qwen3.8-27B   absent  2970 reasoning   146 content    7343ms   picked C
  hf:Qwen/Qwen3.8-27B   low        0 reasoning     1 content    1149ms   picked A
  hf:Qwen/Qwen3.8-27B   medium  3873 reasoning   314 content    9223ms   picked C
  hf:Qwen/Qwen3.8-27B   high    8138 reasoning   180 content   16153ms   picked A
  hf:zai-org/GLM-5.2    absent  3413 reasoning   256 content   15727ms   picked C
  hf:zai-org/GLM-5.2    low        0 reasoning   159 content    4067ms   picked A
  hf:zai-org/GLM-5.2    medium  2145 reasoning   201 content   17149ms   picked A
  hf:zai-org/GLM-5.2    high     277 reasoning   170 content   10218ms   picked A
```

WHAT IS ESTABLISHED.
`low` sets reasoning to exactly zero on both seats.
That is categorical rather than a noisy reading,
and it is not what we want:
`hf:Qwen/Qwen3.8-27B` answered with a ONE CHARACTER content at `low`,
which in production would reach the caller's schema guard as a mismatch
and lose the voice anyway,
faster and for a different reason.

WHAT IS NOT ESTABLISHED,
and must not be read off this table.
Every cell is one call.
`hf:zai-org/GLM-5.2` produced less reasoning at `high` than at
`medium` or absent,
which is either indifference to the value above the on switch
or ordinary per-call variance,
and one sample cannot tell those apart.
The run-to-run band has to be measured before any ladder is claimed.

THE RISK THIS EXPOSES,
which is the reason to measure quality rather than volume:
both seats picked C with reasoning on and A with reasoning off.
Cheaper thinking changed the verdict on the one question asked.
Whether it changes verdicts for the worse is what `#229` still has to measure,
on repeated calls,
against ballots the pipeline already trusts.

The Charm Hyper half is unprobed:
its docs say only that "all standard Anthropic
parameters are accepted" without naming `thinking`,
and the key for it is not in
this session's environment.

### The owner ruled the parameter out, so the lever is closed

2026-08-25,
mid-investigation:
"Please don't set any thinking parameter or budget tokens."

Read broadly,
since `reasoning_effort` is documented by the provider as
"Control reasoning effort for thinking models" and is the same lever under another name.
The pipeline sets none of these and will not.
The probe scripts were discarded;
nothing in `src` ever carried them,
which a search for `reasoning_effort`,
`reasoningEffort`,
`budget_tokens` and `thinking:`
confirms returns nothing.

The measurements above stay because they are about the pipeline rather than about the lever:
18 of 19 cut streams had produced no content,
raw volume separates nothing,
and a reasoning bound cannot separate the populations.
What changes is where those facts point.

WHAT IS LEFT TO TRY,
none of which touches a request parameter:

-   LENGTHEN the window rather than shorten it.
    `#214` measured only the cost of
    shortening.
    A voice lost while still thinking is a voice a longer window might hear,
    and nobody has measured what a 240s or 300s window would recover.
    The honest limit:
    a cut stream never finished,
    so the log cannot say when it would have,
    and only a run with a longer window can answer it.

-   RE-ASK the lost seat,
    which several stages already do for other refusal kinds,
    rather than counting the round complete without it.

-   DROP a seat that loses voices repeatedly.
    The owner has already authorised dropping
    models that are exceptionally bad,
    and `qwen3.8-max` plus `hf:zai-org/GLM-5.2`
    are most of every cut list measured so far.

## The re-ask exists, is gated on quorum, and has never run (`#230`, 2026-08-25)

`#230` was opened on the belief that production never re-asks a malformed answer
while `benchmark.ts` does.
Reading `stage-quorum.ts` corrects it in the more troubling direction:
production has the re-ask,
and the reason nobody has seen it work is that it never fires.

`gatherStageVoices` loops rounds and re-asks only the models the previous round lost,
which is the right shape.
It breaks on:

```ts
if ((round > 0) && (collected.length >= quorumNeeded))
  break;
```

Quorum is half the roster rounded up.
The roster is ten,
so quorum is five,
and the first fan-out has met it on every round measured.

### Counted on the calibration in flight, fifteen slices in

```text
109 rounds, roster of 10 on every one
1054 voices heard of 1090 asked
31 rounds lost at least one voice
0 retry rounds fired
```

Thirty-six voices lost and not one re-asked.
That is worse than having no mechanism:
`stage-quorum.ts` tells a reader that lost voices are retried,
and they are not.

### The benchmark's predicate would have recovered none of them

`isTruncatedAttempt` fires on three detail markers and on a completion count
at the 65536 ceiling.
Every one of those four strings appears zero times in the run log,
so lifting that predicate into production recovers nothing here.
The option is refuted on data rather than argued against.

### Two populations, and they behave oppositely

Of the 36:

-   13 are the model ANSWERING in a shape nothing could use,
    6 `MalformedCompletionError`,
    4 `schema-mismatch` and 3 `SyntaxError`.
    They spread over 7 distinct slices of the 15 seen,
    at most 2 on any one,
    so no input reliably breaks a model.
    Ten of the thirteen belong to two models:
    `gemma-4-26b-a4b-it` seven,
    five of them at the panel stage,
    and `qwen3.8-max` three.
-   23 are silence,
    which is a model still thinking.
    That is the finding this document already carries for the straggler window.

Re-asking both spends the expensive half to recover the cheap half.
A silent voice re-asked buys a second timeout on the critical path;
an unusable answer re-asked buys a fresh call to a model that already finished.

### The split is exact, not a heuristic

`ChatJsonOutcome` has three kinds and only `ok` is usable,
so "the model answered and we could not read it" is decidable at the call site:

-   a non-`ok` outcome carries `rawText`,
    so the model finished;
-   a thrown failure,
    a grace abandonment and an unfilled roster position
    all mean it never got there.

### What is decided, and what is still owed

Decided:
re-ask once,
after quorum,
only a voice that answered unusably.
Owed:
the recovery rate,
which cannot be measured until a run carries the change.
The spread across slices is consistent with per-call independence;
that is a reason to expect recovery,
not a measurement of it.

One more thing the same log shows:
a single `StreamCutShortError`,
on a bundle built before `#228` landed.
The defect `#228` fixed was live in production,
once in 1090 calls.

### Landed, and proven by removing it twice

`91f0c8ba5`.
`StageVoice`'s lost branch carries `answered`,
and one recovery round follows the quorum loop over the answered-but-unusable voices only.

Two guard proofs,
each failing only the cases it owns:

-   recovery round removed:
    four cases fail,
    including `RECOVERS a voice quorum did not
    need`,
    while the silent-voice case correctly still passes;
-   the `answered` split removed so both populations are re-asked:
    exactly one case fails,
    `REFUSES to re-ask a voice that never answered`,
    and nothing else.

The bound is one extra call per answered-but-unusable voice per gather.
A model broken all day costs one extra call per round rather than a ladder,
which on the shape measured here is about a tenth of the round budget in the worst case
and nothing at all in the ordinary one.

What to count on the next run is the line the recovery round logs:

```text
<stage>: recovery round for <n> unreadable answers
```

Against it,
count how many of those voices came back heard.
That ratio is the number this decision was made without.

### The round is bounded at one grace window, and the bound is proven too

`c5cd3d0a3`.
The round landed asking for all its voices back,
which let a single re-ask that hangs hold the whole gather for a full exchange deadline,
six minutes in a run.
Quorum already stands when the recovery round opens,
so it is owed nothing and is entitled to no more than a straggler window.

`heardNeeded: 0` leaves `runGatherRound` with nothing to wait for,
so the grace window opens at once and whatever has not arrived when it closes is abandoned.
A voice that comes back promptly is still collected,
because the window resolves as soon as every ask settles.

The case that covers it scripts the worst shape:
a model answers unusably,
is re-asked,
and never returns.
Removing the bound fails that one case and no other,
and the suite takes thirty seconds instead of milliseconds while it does,
which is the defect showing itself rather than an assertion describing it.

## The final reading, at 356 rounds rather than 51

The calibration finished on 2026-08-25 after 40 slices and 356 rounds,
seven times the sample the provisional reading above was taken on,
and every number in it survives.

```text
rounds                  356, 10.83h in total
  to quorum             1.38h
  waiting after quorum  9.45h, 87.2% of round time
  voices heard          3456 of 3560
```

The share moved from 88.3% to 87.2%.
The shape did not move at all.

ROUNDS STILL DO NOT OVERLAP.
Round wall clock sums to 10.83 hours inside a run that spans about the same,
so the serialisation the provisional reading found is not an artifact of a short window.

WAITING AFTER QUORUM BEATS REACHING IT IN 353 ROUNDS OF 356.
Median time to quorum is 10340 ms.
Median time in grace is 88695 ms.
The grace is not a tail that occasionally costs something;
it is where the run lives.

Per stage,
the share barely varies,
which is what says this is structural rather than one stage behaving badly:

```text
stage                     rounds   median total   median grace   grace share   rounds losing a voice
refiner                       22          81469          72786         93.2%                       4
editor                        32          87584          80140         91.8%                       8
select                       132         122736         104284         88.1%                      29
checker                       47          57316          48381         87.7%                       2
critic                        40         127369         108253         87.3%                      15
introduced-defect-probe       46          34554          28973         87.3%                      10
panel                         37         200859         179956         80.6%                      21
```

## What shortening the grace would actually buy, priced round by round

A round that ends before the cap ended because its last voice arrived,
so its recorded grace IS that voice's arrival time after quorum.
That makes the cost curve exact rather than modelled:
lowering the cap to G saves `max(0, grace - G)` on every round,
and costs a voice on every round whose recorded grace exceeded G.

```text
cap (ms)   saved      of run   rounds newly losing at least one voice
150000     0.68h       6.2%     18 of 356
120000     1.60h      14.7%     59 of 356
 90000     2.85h      26.3%    103 of 356
 60000     4.54h      41.9%    162 of 356
 45000     5.59h      51.6%    193 of 356
 30000     6.74h      62.2%    216 of 356
 20000     7.58h      69.9%    241 of 356
 15000     8.02h      74.0%    250 of 356
 10000     8.48h      78.3%    266 of 356
```

Halving the run means taking a voice off more than half the rounds in it.
`#214` said shortening the grace is expensive at 52 rounds;
at 356 the price is written out,
and it is worse than expensive,
it is close to linear.

The other end is measured too.
73 rounds of 356 burned the full 180000 ms and still lost 87 voices between them.
Those rounds are 20.5% of all rounds and 38.1% of all round wall clock.
The 283 rounds that ended early lost 17 voices in total.
So lengthening the cap would buy back voices only in the fifth of rounds
that are already the most expensive ones in the run.

## This is the whole argument for overlapping units, in one number

`#213` was redirected away from fanning out inside a round
and toward overlapping independent units.
That redirect now has its number:

87.2% of this run is a wait that another unit's work could be filling.

Nothing about that requires a shorter grace,
a different roster,
or a provider that answers faster.
It requires a second slice in flight while the first one waits,
and the calibration driver deliberately does not do that,
with a comment saying so.

The owner's note that Charm Hyper appears to have no concurrency limit
points at the same place from the provider side.

## The reader agrees with an independent parse, and hands `#215` its number

Every figure in the section above was computed by parsing the log directly.
`run-timing-report`,
which `#215` built,
was then run over the same file and agrees to the digit:

```text
run-timing-report: 1 logs, 14747 lines
rounds                 356, 10.83h in total
  waiting after quorum 9.45h, 87.2% of round time
  voices never heard   104
calls in flight        mean 2.56, peak 10
  busy against span    27.71h of calls across 10.83h of run
```

Two readings of one log by two different pieces of code,
neither derived from the other,
which is the only reason to trust either.
The 104 voices never heard match 3560 asked minus 3456 heard exactly.

THE CONCURRENCY LINE IS THE ONE `#215` OWED,
and it says what the grace share implies:

-   Mean calls in flight is 2.56.
    Peak is 10,
    so ten concurrent calls are reachable and were reached.
-   Calls are busy for 27.71 hours across a run spanning 10.83 hours,
    which is 2.56 times over,
    the same number from the other side.

The mean sat at 2.56 in the 51-round provisional reading too.
Seven times the sample did not move it.

A roster of ten that averages 2.56 in flight is a roster waiting,
not a roster saturating anything.
Whatever bounds this run,
it is not the number of calls the client will carry.

## The prototype, and what it is allowed to conclude

Built 2026-08-25 in a throwaway worktree at `/var/home/user/worktrees/tr-overlap`,
kept out of the main tree until a measurement says whether it earns its place.
The two changed files are saved as `~/temp/agent/proto-editor-calibrate.ts`
and `~/temp/agent/proto-slice-overlap.ts`.

WHAT CHANGED IS ONE LOOP.
`editor-calibrate` ran its slices in a `for...of`,
with a comment saying slices run one at a time on purpose.
It now maps them through `pLimit`,
keeping `Promise.all` so the results stay in SAMPLE order rather than completion order,
because every standing the run prints is computed off that array
and a report that depended on which slice finished first
would not be comparable between the two arms.

THE DIAL REFUSES RATHER THAN FALLS BACK.
`TRANSLATION_REPAIR_SLICE_OVERLAP` is read by `readOverlap`,
which throws on anything but a whole number of at least one:

```text
TRANSLATION_REPAIR_SLICE_OVERLAP must be a whole number
TRANSLATION_REPAIR_SLICE_OVERLAP must be at least 1
```

That is not defensive habit.
The entire comparison rests on two runs differing in this value and nothing else,
so a typo that silently became `1` would produce two sequential runs,
a null result,
and a conclusion that overlap does nothing.
Both refusals were exercised before either arm was launched,
at no quota cost.

THE CONTROL IS THE SAME BINARY.
`TRANSLATION_REPAIR_SLICE_OVERLAP=1` reproduces the sequential driver exactly,
so the control arm is the program under test with its dial at one
rather than a different program.
Each arm gets its own `TRANSLATION_REPAIR_RUNS_DIR`,
so neither can read the other's cache and finish early on work it did not do.
The sample is drawn by `pickSpreadSample`,
which carries no randomness,
so both arms draw the same four slices from the same pin.

WHAT THIS PROTOTYPE MAY AND MAY NOT CONCLUDE.
It can say whether overlapping units moves wall clock on this provider mix,
and whether it costs voices.
It cannot say that the production pass would gain the same,
because `runAttemptQueue` is a different loop over different units,
and it cannot settle a difference smaller than the run-to-run band,
which four slices is too small a sample to establish.
A difference of the size the 87.2% figure implies would clear any plausible band;
anything subtler wants a second pair of runs before it is believed.

## The per-model gate does not bind across slices, and that is the mechanism under test

Read from `editor-calibrate.ts` while the first arm was running.

`runOne` calls `createRunClient()` PER SLICE.
Each client carries its own `pLimit` per model,
so two overlapped slices hold two independent limiters
and can have two calls open on the same model at once.

That settles two things at once.

CORRECTNESS IS NOT AT RISK.
A slice shares no mutable state with its neighbours:
its own client,
its own logger,
its own `AbortController`.
The overlap changes when work runs,
not what it computes,
which is why `Promise.all` over the sample is enough
to keep the standings identical to the sequential arm's.

AND THE PROTOTYPE REALLY DOES TEST THE CLAIM IT SAYS IT TESTS.
The comments this change edits say aggregate concurrency beyond one stream per model
collapses throughput on this plan.
Because the limiter is per client and the client is per slice,
overlapping slices does not queue behind that gate,
it multiplies it:
overlap of four means up to four streams per model.
So the arms differ in exactly the quantity those comments name,
and a result either way is about that claim rather than about something adjacent to it.

WHAT WOULD SHOW THE OLD COMMENTS RIGHT:
the overlapped arm finishing no faster,
or finishing faster while losing more voices.
Voices heard is recorded per arm for exactly that reason.

## The control arm ran, finished, and is void (2026-08-25)

Recorded so nobody reads a number off it later.

The serial arm completed:
four slices,
22 rounds,
about twenty-one minutes,
clean exit at 02:15:58 UTC,
log at `~/temp/agent/overlap-arm-serial.log`.
It measures nothing this audit can use,
because every one of its 22 rounds heard
five voices out of ten.

The five it lost were the same five every round,
the Charm Hyper endpoint labels that carry no `hf:` prefix,
and each of them failed on a hundred percent of its calls,
with Synthetic answering `HTTP 400` and saying the model name should start with `hf:`.
The routing defect is filed as `#235` and is not yet located in the source.

WHY THIS IS FATAL TO THE MEASUREMENT rather than merely unfortunate.
The instrument this audit cares about is voices heard against wall clock.
An arm that lost half its roster before the first slice
has a wall clock shaped by five models rather than ten,
and a voices-heard column pinned at five by a cause that has nothing to do with overlap.
Both axes are compromised,
so neither the arm nor the comparison survives.

IT ALSO POISONS THE PLANNED COMPARISON,
which is the part worth naming.
Had the overlapped arm run in the same condition and finished faster,
the honest reading would have been that four half-dark slices beat one half-dark slice,
which is not the question.
Had it lost more voices,
the loss would have been unattributable:
overlap and misroute both subtract voices,
and nothing in the log separates them.
This is the ordinary trap of measuring while a second variable moves.

WHAT THE ARM DOES ESTABLISH,
and it is not nothing:
the pipeline runs a full four-slice calibration to a clean exit
with half its roster returning nothing but errors,
and reports success without ever saying so above `warn`.
That is a finding about resilience and about silence,
filed as the second half of `#235`.
It belongs in this audit only as the reason the wall-clock question is still open.

BOTH ARMS ARE STILL OWED,
back to back,
after `#235` lands.
