# Translation repair handover snapshots: qwen3.8-max output booked as thinking, the volume guard blind to it, and what stragglers may cost (211), 2026-08-25

The 211 thread:
the symptom,
the diagnosis from logs and source,
the drop authorization and its cut-rate confound,
the production-scale confirmation that no volume guard can see the model,
the corrected prediction,
and the straggler and serialization findings it redirected (213,
214,
215),
with the deepseek anomaly and the tool-schema finding (216).

Part of [Translation repair handover snapshots: 2026-08-25 to 2026-08-27](translation-repair-handover-2026-08-25-to-27.md),
which lists every file of this record.

## `qwen3.8-max` books all of its output as thinking (`#211`)

Measured on the live run:
`qwen3.8-max` reports 0 content characters on every single call,
13 of 13,
against 204,258 reasoning characters.
Other seats do this occasionally,
`hf:zai-org/GLM-5.2` on 3 of 10 and two others on 1 of 10.
Thirteen of thirteen is categorical.

THE VOICE STILL LANDS.
The same model casts ballots with full reasons,
so the answer arrives and is used.
What is wrong is the accounting.

THE OBVIOUS EXPLANATION IS ALREADY REFUTED,
recorded so it is not re-walked.
Charm Hyper speaks the Anthropic protocol,
and forced tool use would deliver an answer as `input_json_delta` fragments.
`anthropic-delta-scan.ts` maps that delta to `content` and its own comment names this exact failure mode:

```text
`input_json_delta` IS THE ANSWER CHANNEL ... Routing them to `reasoning`
would leave every schema'd call looking like a model that thought at length
and answered nothing.
```

So the mapping is right and something else produces the symptom.
Settling it needs a captured frame from a live call,
which waits until the run releases the provider.

Not urgent:
nothing is broken and no quota is wasted.
But a reader of any run log would conclude the most expensive seat on the roster produced nothing,
and the `SPEND` line sidesteps it entirely,
since `completion_tokens` comes from the provider and is channel-agnostic.

## `#211` diagnosed from logs and source, without spending a call (2026-08-25)

The earlier note recorded this as needing a captured frame from a live call,
and put the count at 13.
Both were wrong.
The count is about seventy calls,
and most of the diagnosis was reachable by reading.

### The premise, remeasured

Across the calibration's first nine slices,
`qwen3.8-max` opened seventy-one streams and reported zero content characters on seventy of them,
while casting seventy-one ballots with full prose reasons.

### There are two independent parsers, and only one of them is blind

`anthropic-completion.ts` imports `json-guard.ts` and `completion-shape.ts` and nothing else.
It never touches `anthropic-delta-scan.ts`.
So the thing that extracts the answer and the thing that counts characters for the progress line
read the same bytes through different code.
The vote landing while the count reads zero is the two disagreeing,
not the model failing.

### The mechanism, from `channelFor` in `anthropic-delta-scan.ts`

Block type outranks delta type:

```ts
if (blockType === THINKING_BLOCK)
  return 'reasoning';
return DELTA_CHANNELS[deltaType] ?? UNREAD;
```

Its own comment says why that precedence exists:
providers have been seen sending plain text deltas inside a thinking block,
and reading only the delta type would file that as the answer.
The rule is right for the model it was written for.
If `qwen3.8-max` declares a thinking block and sends its answer deltas inside it,
the same rule files its entire reply as reasoning,
while the extractor,
which reads only `delta.type` and ignores blocks,
still recovers the answer.

`qwen3.8-max` is also the only model on the roster configured `toolChoice: 'auto'`;
every other Hyper seat is `forced`.
`hyper-catalog.ts` records that it answers HTTP 400 to every forced-tool variant tried.

### What separates this from the ordinary case

For every other model,
the zero-content count equals the cut count exactly:
`hf:zai-org/GLM-5.2` five and five,
`hf:Qwen/Qwen3.8-27B` two and two.
Those are streams that ended before any content arrived,
which is expected.

`qwen3.8-max` reports seventy zero-content streams against twelve cuts.
Fifty-eight of them completed cleanly and still counted nothing.
Cutting cannot explain that,
and no other model shows the pattern.

`deepseek-v4-pro-0813` shows a smaller separate anomaly,
eight zero-content streams with no cuts at all,
recorded here so it is not folded into this one.

### This is not cosmetic: the runaway guard is blind to this model

`stream-runaway-watch.ts` applies `CONTENT_OVERRUN_CAP` to the content channel only,
and its comment states the reasoning channel is deliberately untouched.
If this model's output is filed as reasoning,
the volume bound never sees it,
so nothing stops it early and it runs to the straggler deadline instead.

The cut rates match that prediction.
`qwen3.8-max` is cut on twelve of seventy-one streams,
near seventeen percent,
the highest on the roster by a factor of two and a half.
`hf:zai-org/GLM-5.2` is next at five of seventy-two.
Six of the eleven seats are cut zero times.
Each cut is a lost voice on a panel that was paid for.

### What is still owed

One thing,
and it is now a single cheap check rather than an investigation:
capture one `qwen3.8-max` stream and confirm it declares a thinking block
whose deltas carry the answer.
If it does,
the fix is at `channelFor`,
not at the model,
and the candidate shape is to keep the block-type override only for delta types
that are not themselves answer channels.
Do not change it before the frame is seen:
the override exists because a real provider needed it,
and removing it blind would restore the defect it was added for.

## Authorization to drop weak models, and why the worst-looking seat is not one (2026-08-25)

The owner granted authorization to drop models that are exceptionally bad.
Recorded here with the guard that has to travel with it.

### The instrument now names drop candidates, and warns about one confound

`~/temp/agent/standing-stats.mjs` previously reported a seat as `SETTLED` on the
absolute value of its z score,
which puts a seat far above the pooled null and
one far below it under the same word.
It now splits by direction:
`CARRIES ITS SEAT` above,
`DROP CANDIDATE` below,
each still requiring both the per-round and the per-slice reading to clear the
Bonferroni threshold for the actual number of comparisons.

It also prints a standing warning to check a seat's cut rate before dropping it.
A seat whose voice is lost scores low for a reason that is not its judgement.

### Its control is now a real log rather than pasted figures

The analyser learned the second header shape that `producer-calibrate` prints,
so the prior forty-round producer run reads directly.
That run is now the positive control:
pooled null 13.48% over 336 of 2492 ballots,
ten comparisons,
Bonferroni threshold 2.807.

### The evidence says drop nothing today, and says it twice

`hf:nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-NVFP4` sits at 3.0%,
z of -4.99,
which clears the threshold by a wide margin in the negative direction.
Its cut rate in the run now in flight is zero of seventy-eight,
so the score is not voice loss.
That is genuine weakness,
and it is already acted on:
the writer seats were settled at three from this same run,
and a model at 3.0% was never among them.
Nothing to drop.

`qwen3.8-max` is the opposite case and the important one.
It is the BEST producer on the roster at 27.0%,
z of +5.22,
and it carries the worst cut rate on the roster,
twelve of seventy-eight.
It earns the top score while losing twelve voices to the defect in `#211`.

So the seat that looks worst by cut rate is the strongest seat by judgement,
and dropping on the cut rate alone would have removed it.
That is exactly the confound the warning exists for.

### What this changes about `#211`

It stops being an accounting curiosity.
The scanner mis-files this model's output,
the volume bound therefore cannot see it,
it runs to the straggler deadline,
and the pipeline loses a sixth of the voices
of its highest-scoring producer.
Fixing it does not tidy a log line,
it recovers the best seat's lost ballots.

### What the authorization is waiting on

The editor standing,
which the run in flight is measuring and which is not the
producer standing.
`#136` already recorded a model that is worst at one stage and better than two
peers at another,
so a producer table cannot seat or unseat an editor.
When the run lands,
the analyser names candidates and the cut rates say which of
them are real.

## The measurement

Every seat was asked exactly 120 times,
so the denominators need no adjustment.
Seven of the ten lose no voice at all.
The 34 losses are `qwen3.8-max` 21,
`hf:zai-org/GLM-5.2` 11,
`hf:Qwen/Qwen3.8-27B` 2.

`qwen3.8-max` reports content characters of 0 at the median AND at the 95th percentile,
across all 100 of its completed streams.
Not one byte it sent reached the content channel.
Every other seat medians between 303 and 619.
That is `#211` confirmed at production scale,
where it had been proved on one captured frame run.

## Why that is worse than a telemetry error

`src/stream-runaway-watch.ts` applies its volume cap to the content channel only,
and its module note says the reasoning channel is untouched.
`#156` set that bound at 32000 and declined a reasoning bound deliberately.
So for this one seat the chain closes:
answer filed as reasoning,
cap reads content,
content is always zero,
no volume guard can ever fire.
The only thing that stops it is `STRAGGLER_GRACE_MS`,
and that is what its 21 cuts are.

## The correction to `#211`'s recorded prediction

`#211` predicted the cut rate would fall toward the roster's.
That is not supported and has been replaced on the task.
The cuts are volume runaways,
not stalls:
`qwen3.8-max` is cut mid-reply after 106,405 characters
once and then 19 of 21 times between 293,163 and 350,293,
a tight cluster because the cut is time-bound while the stream rate is steady.
Post-fix those bytes meet `contentCap`,
so the call is cut at roughly 32000 rather than 300,000,
in seconds rather than 180.
What changes is when and how expensively the voice is lost,
not obviously whether.
Two mechanisms pull opposite ways:
an early overrun rides the retry predicate `#156` built,
giving the model an attempt it never used to get,
against which a model emitting 300,000 characters may simply overrun again.
Record the outcome,
not the prediction.

## What it redirects

The straggler cuts group into 29 distinct events.
At 180 seconds each that is an upper bound of 1.45 hours inside a 3.73 hour run.
It is an upper bound and must be read as one:
`runStageRound` assembles from its `arrived` map rather than awaiting abandoned calls,
so the true per-event cost is at most the grace window and may be less.

That is potentially a larger lever than the serialization question `#213` was opened on,
and 21 of the 34 cuts are the seat `#211` already fixes.
So the order is now:
land `#211` and re-measure,
then re-derive the window,
then prototype fan-out.
Measuring fan-out first would be measuring a system about to change underneath it.

## Two tasks this opened

`#214` re-derives the straggler window.
`doc/decision/translation-repair-straggler-grace.md` rests on the claim that no hung call had ever
been recorded and every cut voice was slow-but-working.
A voice cut mid-reply after 3,020,068 characters refutes that.
The decision should be superseded rather than edited,
since its reasoning was correct for the
population it had,
and it should not be reverted:
60 seconds really did cut `hf:zai-org/GLM-5.2` inside its ordinary range,
and still would.

`#215` adds elapsed milliseconds to the stream completion line.
A production run currently cannot answer where its own wall-clock went,
because dispatch is logged at `debug` and production emits only `info` and `warn`,
while the completion line carries `firstByte` and `maxGap` but no duration.
That is why the 1.45 hour figure is a bound rather than a measurement,
and why `#213` cannot yet measure the baseline it needs.

## Corrected count for `#213`

The first pass said seven serialization sites and grepped four wrong paths.
The full sweep finds eleven resting on the refuted premise,
seven of them on the production path,
and `src/editor-ensemble.ts:248` is the one with a clean argument:
envelopes within a slice are independent,
nothing reads back an earlier envelope's outcome,
and `Promise.all` preserves order,
so gathering concurrently and folding in index order is
byte-identical output for pure latency.
The task carries the full list,
including the sites that are correctly sequential and must not be
touched.

## Two more findings from the same run (2026-08-25)

The deepseek zero-content anomaly is RESOLVED,
and it is the same defect as `#211` intermittently.
It had been held open because the instrument used then was a 40-line scan window over 9 samples
with a measured 39 percent hit rate on healthy calls.
The completion line's own content count answers it on roughly 1,200 streams:
`deepseek-v4-pro-0813` is zero-content on 12 of 127,
every one with reasoning above zero,
against a baseline of exactly zero across eight other seats.
What remains a prediction rather than a conclusion is whether `#211`'s fix removes those 12,
since the fix exempts `input_json_delta` and not `text_delta`.
Count them on the first post-fix run.

`#216` implements an owner instruction that turns out never to have been implemented:
"Please make sure to put even the full tool schema into system prompts."
Seventeen modules build a system message and NOT ONE carries its response schema;
the seven that name a schema at all name it only as the API-level `responseFormat`.
The supporting evidence is that all 6 schema failures in this run are on Charm Hyper seats,
`gemma-4-26b-a4b-it` 4 and `qwen3.8-max` 2,
with zero on Synthetic,
and one of them reads `{"checks": "\n[{\"region\":` ,
a JSON-stringified array where the schema declares an array of objects.
That is the wrong-tool-call-format failure the owner named.
The provider split is confounded,
since those seats are also different models,
so the task says so and does not claim the protocol causes it.
