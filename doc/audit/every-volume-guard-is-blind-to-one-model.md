# Every volume guard is blind to one model, and the straggler window is all that stops it

Measured 2026-08-25 against the live full-roster editor calibration
(`editor-calibrate.mjs 40`,
15 of 40 slices,
3 hours 44 minutes elapsed,
zero `[error]` lines).
Read from the run log alone.
No corpus wording appears here:
model ids,
counts and durations only.

## What was measured

Every model was asked exactly 120 times,
so the denominators need no adjustment.
Streams that completed,
plus voices abandoned,
per model:

-   `minimax-m3`:
    120 completed,
    0 abandoned.
-   `hf:openai/gpt-oss-120b`:
    120 completed,
    0 abandoned.
-   `hf:nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-NVFP4`:
    120 completed,
    0 abandoned.
-   `hf:moonshotai/Kimi-K3`:
    120 completed,
    0 abandoned.
-   `deepseek-v4-pro-0813`:
    120 completed,
    0 abandoned.
-   `deepseek-v4-flash-0731`:
    120 completed,
    0 abandoned.
-   `gemma-4-26b-a4b-it`:
    120 completed,
    0 abandoned,
    4 schema mismatches.
-   `hf:Qwen/Qwen3.8-27B`:
    118 completed,
    2 abandoned,
    1.7 percent.
-   `hf:zai-org/GLM-5.2`:
    109 completed,
    11 abandoned,
    9.2 percent.
-   `qwen3.8-max`:
    98 completed,
    21 abandoned,
    17.6 percent.

Seven of ten seats lose no voice at all.
The loss is 34 voices,
and 21 of them are one model.

## The finding

`qwen3.8-max` reports content characters of 0 at the median AND at the 95th percentile,
across all 100 of its completed streams in this run.
Not one byte it has sent in this run was accounted to the content channel.
Its 34,202 character maximum is entirely reasoning.

Every other seat reports content between 303 and 619 characters at the median.

This is `#211` at production scale.
`#211` was proved on a single captured frame sequence:
the provider emits two `content_block_start`
frames for the same block index,
the second declaring `thinking`,
so the `input_json_delta` frames
carrying the tool-call arguments are routed to `reasoning` by `channelFor` in
`src/anthropic-delta-scan.ts`.
`qwen3.8-max` is the one seat on `toolChoice: 'auto'`,
so its whole answer arrives as
`input_json_delta`,
so its whole answer is filed as reasoning.

## Why that is worse than a telemetry error

`src/stream-runaway-watch.ts` applies its volume cap to the content channel only:

```ts
// src/stream-runaway-watch.ts
const contentChars = ratioDetectors.content
  .charsSeen();
if (contentChars >= contentCap)
  return {
    kind: 'overrun',
    channel: 'content',
    charsSeen: contentChars,
    cap: contentCap,
  };
```

The module note beside it states the intent plainly,
that the reasoning channel is untouched.
`#156` measured the answer bound at 32000 and declined a reasoning bound deliberately.
`maxAnswerChars` reaches this cap as `contentCap` through `drainBody` in `src/stream-drain.ts`,
and `producedVolumeBound` scales it per call for the translate lane.

So for `qwen3.8-max` the chain is:
its answer is filed as reasoning,
the volume cap reads content,
content remains zero,
and no volume guard can ever fire on it.
The ratio and recurrence detectors still watch reasoning,
but they did not fire on any of the 21 cuts.

The only thing that stops this model is `STRAGGLER_GRACE_MS`.
That is what the 21 cuts are.

## What the cuts look like

The abandoned voices were cut mid-reply having delivered,
per model:

-   `qwen3.8-max`:
    106,405 characters once,
    then 19 of 21 clustered between 293,163 and 350,293.
-   `hf:zai-org/GLM-5.2`:
    248,535 to 3,020,068.
-   `hf:Qwen/Qwen3.8-27B`:
    656,600 and 1,276,871.

The `qwen3.8-max` cluster is tight because the cut is time-bound rather than volume-bound:
at a steady stream rate,
180 seconds buys a roughly constant number of characters.

## The decision this refutes

`doc/decision/translation-repair-straggler-grace.md`,
taken 2026-08-17,
moved the window from 60 to
180 seconds.
Its central claim:

> NOT ONE HUNG CALL WAS RECORDED.
> The window exists to stop a model that answers nothing from
> delaying the pipeline,
> and in 602 exchanges it has never once met that model.
> Every voice it has
> taken was a slow-but-working one.

That premise does not survive this run.
A voice cut mid-reply after 3,020,068 characters is not slow-but-working.
The decision also records a latency table whose worst observation was 88.6 seconds,
and set 180 as
"more than a factor of two" above it.
Against a model emitting millions of characters,
the multiple of a healthy maximum is not the
quantity that matters.

The decision should not simply be reverted.
It was right that `hf:zai-org/GLM-5.2` was being cut inside its ordinary operating range at 60
seconds,
and that is still true.
What has changed is that the population now contains a second kind of call the window was never
sized against,
and one window cannot serve both.

## What this costs

The 34 cut voices fall into 29 distinct straggler events,
grouping voices cut within 5 seconds of each other as one round.
At 180 seconds each that is 1.45 hours of waiting,
inside a run that has so far taken 3.73 hours.

That figure is an upper bound and should be read as one.
It holds only where the round's completion actually waits on the straggler.
`runStageRound` in `src/stage-round.ts` assembles its result from the `arrived` map rather than by
awaiting abandoned calls,
precisely so a client that ignored an abort could not hang the stage,
so the true cost per event is at most the grace window and may be less.
Confirming it needs the dispatch timestamps the run does not currently record.

## The logging gap this exposes

The question "how concurrent was this run,
and where did its wall-clock go" cannot be answered from
the run's own logs.

-   The client logs dispatch at `debug` (`-> modelId: N messages` in `src/synthetic-client.ts`).
-   This run emits only `info` and `warn`,
    3,642 and 41 lines.
-   `reportStreamProgress` logs completion at `info` but carries no duration:
    its fields are `firstByte`,
    `maxGap`,
    raw characters,
    unreadable frames,
    content and reasoning
    characters.

So a stream's start time is not recoverable,
and neither is its duration.
Adding a duration field to the existing completion line is the cheaper of the two fixes:
it adds no line volume,
and start time follows by subtraction.

### Closed by `#215`, with the measurement still owed

Both halves landed on 2026-08-25.

`reportStreamProgress` now prints `elapsed <n>ms` beside the outcome,
so a completion line plus its own timestamp gives every call an interval.

A SECOND GAP WAS FOUND WHILE FIXING THIS ONE,
and it is the larger of the two.
The log has no round boundary either.
Surveying every line shape in the live calibration finds three tags and no others:
`reportStreamProgress`,
`takeReading` (the availability meter) and `exchangeWithRetry`.
Nothing delimits a fan-out,
so even with per-call durations
the waiting could not be attributed to the rounds that did it.
`runGatherRound` now writes one line per round:

```text
editor round: 6/7 heard, 91402ms total, 61401ms to quorum, 30001ms in grace
```

`ms in grace` is this section's figure,
measured rather than bounded.

THE 1.45 HOUR FIGURE ABOVE REMAINS AN UPPER BOUND until a run emits these lines.
The calibration running when they landed was launched from the older build,
so the first real reading comes from the next pass.
`mise run //package/module/translation-repair:run-timing-report -- <log>` (moved on 2026-10-18 to
`//package/translation-repair/run:run-timing-report`) reads them back,
and on a log written before this it says so rather than reporting zeros.

What could be measured on the OLD log,
as a floor rather than the figure:
each completion carries `firstByte` and `maxGap`,
and the largest gap falls strictly
after the first byte,
so their sum bounds a call's duration from below.
Sweeping those subset intervals over 2405 calls across 6.15 hours gives
a mean of at least 0.39 in flight and a peak of at least 9.
That is a floor and says nothing about the true figure.

## What follows

-   `#211`'s fix routes `input_json_delta` to content,
    which is what lets any volume guard see this
    model at all.
    That is its real value,
    larger than the telemetry correction it was opened as.
-   `#211`'s recorded expectation,
    that the cut rate falls,
    is not supported.
    The likelier outcome is that the cut rate holds near 17 percent while each cut costs a fraction
    of the time and bytes,
    because `contentCap` fires long before 180 seconds.
    Record the outcome either way rather than the prediction.
-   The straggler window needs re-deriving against a population that now contains runaways.
-   Whether the reasoning channel should carry a volume cap of its own is a separate question.
    52 of 1,182 completed streams exceeded 32000 reasoning characters and completed fine,
    `hf:Qwen/Qwen3.8-27B` reaching 99,244,
    so a naive cap would cut working voices.

## The deepseek anomaly is the same defect, intermittently (resolved 2026-08-25)

`#211` recorded a second,
smaller anomaly on `deepseek-v4-pro-0813` and held it open,
because the instrument used then was a 40-line scan window over 9 samples,
and a positive control on `hf:openai/gpt-oss-120b` showed that instrument had a 39 percent hit rate
on healthy calls.
Nine samples against a 39 percent baseline separate nothing,
and the honest conclusion was that the
log could not answer it.

The same run answers it now,
because the right instrument is the completion line's own content
count and this run carries roughly 1,200 of them.
Zero-content streams per seat,
at 15 of 40 slices:

-   `qwen3.8-max`:
    104 of 106,
    98.1 percent.
-   `deepseek-v4-pro-0813`:
    12 of 127,
    9.4 percent.
-   `gemma-4-26b-a4b-it`:
    0 of 127.
-   `deepseek-v4-flash-0731`:
    0 of 127.
-   `minimax-m3`:
    0 of 127.
-   `hf:openai/gpt-oss-120b`:
    0 of 127.
-   `hf:nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-NVFP4`:
    0 of 127.
-   `hf:moonshotai/Kimi-K3`:
    0 of 127.
-   `hf:Qwen/Qwen3.8-27B`:
    0 of 124.
-   `hf:zai-org/GLM-5.2`:
    0 of 115.

THE BASELINE IS EXACTLY ZERO,
not 39 percent.
Eight seats across roughly 1,000 streams produce not one zero-content call,
so 12 of 127 is not something a null distribution supplies.
Every one of the 12 carried reasoning characters above zero,
which is the same signature as `qwen3.8-max`:
bytes arrived,
and none were accounted to content.

The split follows the provider.
Both affected seats are served by Charm Hyper,
which speaks the Anthropic protocol that
`src/anthropic-delta-scan.ts` reads,
and every `hf:` seat on Synthetic is clean.
Three other Hyper seats are also clean,
so this is not "the Hyper path is broken":
it is a frame pattern only some upstream models emit.

### What is still not settled, and the prediction that settles it

Why constant for one seat and intermittent for the other is not readable from the log.
`qwen3.8-max` is the one seat on `toolChoice: 'auto'`,
so its whole answer always arrives as
tool-call arguments,
which is exactly the delta type `#211` found misrouted.
For `deepseek-v4-pro-0813` two readings remain,
and they differ in whether `#211`'s fix helps:

-   Same mechanism,
    occurring on 9.4 percent of calls because the duplicate
    `content_block_start` is what varies.
    `#211`'s fix removes all 12.
-   A different delta type,
    `text_delta` inside a thinking block,
    which `#211`
    deliberately does NOT exempt.
    The fix removes none of the 12.

That is a clean falsifiable prediction rather than an open question:
count zero-content streams for `deepseek-v4-pro-0813` on the first post-`#211` run.
Twelve becoming zero confirms the first reading;
twelve staying put confirms the second and opens
its own task.
Do not fold the answer into `#211` before that count exists.

## Generated volume, and a correction to what zero content means

Counted from the finished full-roster calibration,
40 of 40 slices.

```text
run-wide          3149 streams   content  1242020   reasoning 21392195   94.5% reasoning
                  (measured at 30 slices; the shares below are the finished run)

zero content, against COMPLETED streams only:
  qwen3.8-max            285 completed   281 zero-content   98.6%
  deepseek-v4-pro-0813   356 completed    36 zero-content   10.1%
  hf:Qwen/Qwen3.8-27B    344 completed     4 zero-content    1.2%
  hf:zai-org/GLM-5.2     323 completed     2 zero-content    0.6%
  the remaining six seats                  0
```

### THE `qwen3.8-max` ROW IS AN ACCOUNTING ARTIFACT, NOT A LOST ANSWER

An earlier version of this section read the 281 as answers that never arrived,
and said the
pipeline "believed the call succeeded and received nothing".
THAT IS WRONG,
and the finished
standing is what refuted it.

`qwen3.8-max` placed SECOND in the editor column at 22.7 percent over 81 candidates,
won 10
editor rounds and 3 refinements,
and cast 97 ballots.
A seat that delivered nothing cannot
do any of that.

The cause is in `src/hyper-catalog.ts:73`,
which states it outright:
of the nine Hyper
models,
eight honour `tool_choice: {type: 'tool'}` and `qwen3.8-max` REFUSES it.
It is
therefore the one seat configured `toolChoice: 'auto'` (`src/hyper-catalog.ts:180`),
so its
whole answer arrives as TOOL-CALL ARGUMENTS.
`generatedChars.content` counts the content
channel and does not count tool-call arguments,
so for this one seat a correct,
complete,
round-winning answer is recorded as zero content characters.

Zero content here means the counter cannot see this seat's delivery channel.
It does not
mean the seat was silent.

### What survives the correction

`deepseek-v4-pro-0813` at 36 of 356,
10.1 percent,
IS a real anomaly.
It runs `forced` tool
choice,
so the auto-channel explanation does not cover it,
and its rate held steady across
three samples:
9.4 percent,
then 11.2 percent,
now 10.1 percent.
Stable intermittent
behaviour rather than drift.

`hf:Qwen/Qwen3.8-27B` at 4 and `hf:zai-org/GLM-5.2` at 2 remain small and unexplained.
Both
were previously recorded as losing voices to CUTS,
a different column,
and neither was known
to complete a stream empty.

A high reasoning share is also not the finding.
Seven seats sit between 73 and 98 percent
reasoning and produce real content throughout,
because that is where thinking models spend.
`gemma-4-26b-a4b-it` emits no reasoning at all and still reached 30737 content characters on
a single call.
Reading the run-wide 94.5 percent as waste would be wrong.

### The prediction this document still carries

Count zero-content streams for `deepseek-v4-pro-0813` on the first post-`#211` run.
Thirty-six becoming zero confirms the first reading recorded here;
thirty-six staying put
confirms the second and opens its own task.

`qwen3.8-max` is NOT part of that prediction any more.
Its 281 are expected and correct,
and
will stay at roughly 281 after `#211` lands,
because nothing about the tool-call channel is
a defect.
Any guard written against "completed with zero content" must exempt the `auto`
seat or it will fire on every one of its calls (`#221`).

The instrument is `~/temp/agent/zero-content-from-log.mjs`,
which takes one or more log paths
and was cross-checked against an independently written pass over the same log rather than
against itself.
Its output must be read with the tool-choice column beside it;
the counter
alone cannot tell an empty answer from an unseen channel.
