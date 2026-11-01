# First production traffic through the stream guards

Read 2026-08-18 from `~/temp/agent/flagged-pass-2.log`,
covering the first 48 minutes of the targeted corpus pass started 01:29Z,
which is the first traffic of any kind through the `#118` cut reporting
and the `#119` degeneration guard.

Everything below is read from a pass still running,
so every count is a floor rather than a total.

## What the window held

```text
streams                347
total raw characters   219574656
smallest stream             4662
median                    357921
ninetieth percentile     1689688
largest                  5558524
```

RAW CHARACTERS ARE WIRE BYTES,
envelope included,
which is the only length column production records.
Nothing here reports generated characters,
and that gap decides how the rest of this document reads.

## Both new instruments worked

A voice was lost and,
for the first time,
the log says WHY:

```text
panel hf:zai-org/GLM-5.2: abandoned 180000ms after quorum
  (cut-mid-reply after 2915229 delivered chars, first byte at 2087ms), voice lost
```

Before `#118` that line read only `voice lost`,
and the 2.9 million characters the call had already delivered were discarded.
The companion progress line records `maxGap 460ms`,
so this call was streaming steadily when it was ended.
It was not stalled,
and the idle guard could never have seen it:
both idle constants sit at 600000 ms and the largest gap was under half a second.

WHAT ENDED IT WAS THE STRAGGLER GRACE,
exactly as designed,
180000 ms after quorum.
That is the first direct look at what the grace actually cuts,
and it cut a call that was still producing.

## The second abandonment proves the point of the first

A second voice was lost the same way and it is a DIFFERENT SHAPE,
which is the whole reason `#118` was worth building:

```text
panel  hf:zai-org/GLM-5.2: abandoned 180000ms after quorum
  (cut-mid-reply after 2915229 delivered chars, first byte at 2087ms), voice lost
select hf:zai-org/GLM-5.2: abandoned 180000ms after quorum
  (cut-mid-reply after  123534 delivered chars, first byte at 1877ms), voice lost
```

Both were cut exactly 180000 ms after quorum,
so both streams ran at least that long,
and dividing delivered characters by that shared floor gives an upper bound on each rate
computed the same way for both:
at most 16196 characters a second for the first
and at most 686 for the second.
The second was therefore at least twenty three times slower per second than the first.

ONE WAS PRODUCING AT FULL SPEED AND THE OTHER WAS CRAWLING,
and they want opposite remedies:
the first is a long reply the grace should probably have waited for,
the second is a call worth ending sooner.
Before tonight both printed the same three words and nothing else.

## Neither guard has refused a call

No `degenerate in` line appears anywhere in the window.
The single `cut` line is the straggler above.

THE OBVIOUS SEARCH LIES,
and it is worth writing down.
Grepping this log for `cut` returns ten confident matches,
every one of them the word "cute" inside a rationale about 撒娇.
The searches that answer the question are `stream .*: cut` and `degenerate in`.

## The case the guards were built for happened, and nothing can diagnose it

```text
stream hf:zai-org/GLM-4.7-Flash: completed, firstByte 920ms, maxGap 91ms,
  2052766 raw chars, 0 unreadable frames, 2052766 delivered chars
select hf:zai-org/GLM-4.7-Flash: schema-mismatch
  (content is not valid JSON: SyntaxError: Unexpected end of JSON input) raw="", voice lost
```

The model streamed two million characters of wire and delivered NO ANSWER AT ALL.
The stream completed on its own,
so this is not a runaway in the sense of a call that never ends,
and the guard was right not to refuse it.
Whether the model spent that budget thinking usefully or cycling
is exactly the question the reasoning-channel detector exists to answer,
and the log cannot say,
because:

-   the progress line reports raw characters twice and generated characters never,
    so the size of the thinking trace is unknown;
-   the opening excerpt is shown only on a cut,
    and this stream completed;
-   and had it been shown it would have carried envelope rather than words,
    since `partialText` is the raw event stream and every such body opens
    `data: {"id":"`,
    with a bare hexadecimal id in production
    and `chatcmpl-tabby` in the fixture that demonstrated it.

All three are recorded in `doc/decision/translation-repair-runaway-call-termination.md`.

## Why the unmeasured envelope ratio decides how much the guard sees

A verdict needs about 131000 characters of GENERATED text.
The pass records only raw characters.
The two are related by a ratio that has never been measured,
and the choice of ratio moves the answer by a factor of six:

```text
if the envelope costs 10x   45 of 347 streams reach the bar, about 13 percent
if the envelope cost  1x   282 of 347 streams reach the bar, about 81 percent
```

So the open question is not academic bookkeeping.
It decides whether this guard judges one stream in eight or four in five,
and today nobody knows which.

## The straggler grace is almost entirely a GLM-5.2 tax

Read at 1148 completed-or-cut streams,
four hours into the pass.
Call counts are near enough equal across the roster that this is a RATE difference
rather than a volume artefact:

```text
model                                              streams   cuts     rate
hf:zai-org/GLM-5.2                                     186      9   0.0484
hf:moonshotai/Kimi-K3                                  183      1   0.0055
hf:zai-org/GLM-4.7-Flash                               183      0   0.0000
hf:Qwen/Qwen3.6-27B                                    199      0   0.0000
hf:openai/gpt-oss-120b                                 199      0   0.0000
hf:nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-NVFP4      199      0   0.0000
```

NINE OF TEN CUTS ARE ONE MODEL,
and four of the six models have never been cut at all.
Every cut is a straggler abandonment at 180000 ms after quorum;
no `degenerate in` line has appeared,
so the degeneration guard has still never fired in production.

NONE OF THEM WAS SLOW TO START.
First byte across the ten runs from 751 ms to 2120 ms,
which is the ordinary range for this pass.
What they were is LONG,
and their delivered sizes at the cut vary by more than twenty times,
from 123534 characters to 2915229.

THIS IS NOT A CLAIM THAT GLM-5.2 IS SLOW,
a claim already retracted once in this work.
It is verbose on this workload and the grace ends verbose calls.
The reading that matters is that the window's cost falls on one voice,
so the ensemble it protects loses the same member over and over.

AND NOTHING RE-ASKS IT,
which is by design and worth stating so it is not mistaken for a bug.
`stage-quorum.ts` runs three retry rounds after the initial fan-out,
and the loop reads `if ((round > 0) && (collected.length >= quorumNeeded)) break`.
A voice lost while quorum is already satisfied is therefore never re-asked.
That is exactly what the straggler grace is for.
The consequence is the part worth deciding on:
on the stages where this happens the ensemble is effectively five of six,
always missing the same member,
rather than five of six missing a different one each time.

CHECKED AND CLEARED while looking at this,
so it is not re-investigated:
`isRetryableAttempt` in `attempt-retry.ts` would call these truncated replies retryable,
and its only caller is `benchmark.ts`.
That is not a production gap.
Production retries at the STAGE level through `stage-quorum.ts` rather than per attempt,
and the two policies answer different questions.

## Voice loss has two shapes and both are now legible

Nine voices lost in the window,
in two kinds that want different remedies:

```text
straggler abandonment  6   GLM-5.2 five, Kimi-K3 one
schema mismatch        3   GLM-4.7-Flash two, gpt-oss-120b one
```

THE SCHEMA MISMATCHES ARE THE MORE INTERESTING HALF.
Each reports `content is not valid JSON: SyntaxError: Unexpected end of JSON input` with `raw=""`,
so the model completed its stream and delivered NO ANSWER,
which for GLM-4.7-Flash once meant 2052766 characters of wire for nothing.

THIS QUALIFIES AN EARLIER FINDING rather than overturning it.
Previous work established that voice loss was GLM and never Kimi-K3.
On this pass it is seven of nine GLM,
one Kimi-K3 and one gpt-oss-120b,
so the model is no longer alone even if it still dominates.
Five hand-picked entries are not a corpus sample and this does not replace the earlier reading.

## The idle windows rest on a premise the current traffic contradicts

`stream-idle-guard.ts` sets both windows to 600000 ms so they never fire,
and the reason it gives is that silence cannot discriminate on this provider:

> Across 32 successful streams,
> time to first byte ran p50 95.6 s,
> p75 123 s,
> p90 134 s,
> max 147.5 s.
> A window cannot separate "stalled and silent" from "working and silent"
> when working looks like that.

MEASURED OVER THIS PASS,
working no longer looks like that:

```text
                p50     p75     p90     p99      max
firstByte ms    1174    1349    1726    3091     9084
maxGap ms        207     257     463     989    11659
```

395 streams,
which is every stream that reached the drain in the window,
cut and completed alike.
Time to first byte is about eighty times faster at the median
than the figure the constant rests on.
Every one of the 395 received a first byte:
zero progress lines carry the negative `firstByteMs` that marks a call which never got one.

THIS DISTRIBUTION IS UNCENSORED,
which the old one was not.
The old sample was taken with a 150 s window live,
so anything slower was aborted instead of recorded,
and the guard's own comment says so.
Here both windows sit at 600000 ms and neither fired,
so these maxima are maxima rather than the shadow of a guard.

IT IS ALSO THE UNBIASED SUCCESSOR TO A RETRACTED FIGURE.
A reading of "1349 streams,
mean firstByte 1822 ms" was retracted for survivorship bias,
because abandoned calls were excluded by construction.
Since `#118`,
progress is reported on the cut path too,
so an abandoned call is in this sample rather than missing from it.

WHAT IT DOES NOT SAY.
It does not say the constants are wrong,
and it does not explain the gap between the two readings:
different stage shapes,
provider capacity,
routing and time of day are all unexcluded,
and one pass over five entries is one workload.
What it does say is that the stated reason for disabling the guard
is not what the current traffic looks like,
and that a first-byte window is worth re-deriving rather than assumed useless.

## Three workloads agree on the middle and disagree with any small window

The single-pass reading above is not enough to set a constant,
so the same two columns were read from every pass log that carries them.
`grace-remeasure-snapshot.log` is excluded,
and the exclusion is proven rather than inferred:
`cmp` reports it byte for byte identical to the first 811723 bytes of `corpus-pass-20260817.log`,
so it is a snapshot of that run taken while it was still going,
and counting it would count 1677 of those streams twice.
The two that remain are genuinely separate runs,
one spanning 2026-08-13T17:15Z to 2026-08-14T02:03Z
and the other 2026-08-17T17:59Z to 2026-08-18T00:41Z.

```text
log                        streams   p50    p75    p90     p99      max
resume-run-output             2888  1032   1191   1477    3955   183755
corpus-pass-20260817          2166  1237   1451   1951   10163    91843
flagged-pass-2 (this one)      416  1173   1352   1726    2768     9084
```

THE MIDDLE IS STABLE ACROSS FIVE DAYS AND THREE WORKLOADS,
at one to one and a third seconds,
and none of them resembles the 95.6 second median the constant was set from.

THE TAIL IS REAL,
and it is the half a single pass could not have shown.
One call waited 183 seconds for its first byte and another 92.
Counted over the 5470 streams in the three logs:

```text
first byte over    30 s   10 streams   about 1 in 550
                   60 s    2 streams
                  120 s    1 stream
                  180 s    1 stream
                  300 s    0 streams

largest gap over   30 s    4 streams
                   60 s    1 stream
                  120 s    0 streams
```

SO A SHORT WINDOW IS NOT FREE,
which is the correction this second reading makes to the first.
A 30 second first-byte window would have fired on about one call in 550,
and at six voices a stage that is roughly one stage in ninety losing a voice to the guard itself.
A 300 second window would have fired on none of the 5470
and would still end a dead call in five minutes rather than never.

THESE COUNTS ARE LOWER BOUNDS.
Two of the three logs predate `#118`,
so they record only streams that completed,
and a call abandoned before completing is exactly the one most likely to have waited longest
for a first byte.
The true exceedance is at or above every figure above,
which argues against the small windows rather than for them.

IT ALSO RE-FRAMES WHAT THE STRAGGLER GRACE CUTS.
If a call reaches its first byte in about a second,
then one still running 180 s after quorum is not waiting to start;
it is producing.
The single abandonment in this window proves it directly:
2915229 characters delivered,
largest gap 460 ms.
`doc/audit/straggler-grace-remeasure.md` measured what widening the grace did to voice loss
and did not measure this,
so the two readings sit beside each other rather than in conflict.

## What this changes

-   `#120` gains production evidence rather than fixture evidence.
    The generated-character count is not a tidiness item;
    it is the difference between a guard that is known to be watching
    and one that is assumed to be.
-   The straggler grace now has one observed case with a full description,
    which `doc/decision/translation-repair-straggler-grace.md` never had.
    One case decides nothing;
    it is the first of a population that can now be counted at all.
-   Voice loss in this window is two calls,
    one cut after quorum and one that answered with an empty string,
    and they want opposite remedies.

## The same instruments at ten times the traffic, and the population is now countable

Read 2026-08-25 from `~/temp/agent/editor-calibrate-postguard-20260825.log`,
the completed forty-slice `editor-calibrate` run,
14746 lines covering 3561 model streams plus 584 quota and credit polls.
The 347-stream window recorded further up this document could count two lost voices;
this one counts every stream every roster model opened.

```text
model streams   3561
completed       3481
cut               79
overrun            1
```

`overrun` is the `#119` degeneration guard,
and one firing in 3561 streams is the first time it has been seen on real traffic at all.

## Every stream that finished produced content, and every stream that produced nothing was cut

The cross-tabulation is total,
with no cell needing a qualifier:

```text
completed, content > 0   3481
cut,       content > 0      7
cut,       content = 0     72
overrun,   content > 0       1
```

There is no completed stream anywhere in the run with zero content characters.
That single fact answers `#221` on a population seven times larger than the one that first refuted it,
and answers it the same way:
a model that reaches the end of its stream always wrote something.

## The seventy-two are not silence, they are reasoning that never reached the answer

All 72 of the zero-content streams carry reasoning characters greater than zero.
Not one of them is a model that opened a connection and said nothing.
Every one is a model that spent the whole straggler grace on the reasoning channel
and was cut before it emitted its first content character.

This is a third shape of voice loss,
distinct from both shapes named earlier in this document:
not a cut mid-reply after millions of delivered characters,
and not an answer that arrived empty.

It also concentrates,
by model,
far harder than the traffic does:

```text
qwen3.8-max          cut 57 of 357 (16.0%), zero content 55
hf:zai-org/GLM-5.2   cut 16 of 356 ( 4.5%), zero content 13
hf:Qwen/Qwen3.8-27B  cut  3 of 356 ( 0.8%), zero content  3
deepseek-v4-pro-0813 cut  1 of 356 ( 0.3%), zero content  1
gemma-4-26b-a4b-it   cut  2 of 356 ( 0.6%), zero content  0
```

The remaining five roster models were cut zero times in 356 streams each.

## Losing a voice is not the same as the model being bad, and here the two point opposite ways

`qwen3.8-max` looks like the obvious candidate to drop:
it owns 55 of the 72 zero-content streams and 57 of the 79 cuts,
while half the roster was never cut once.

The run's own ledger says the opposite.
Across 132 selection rounds,
counting how often each model's candidate was the one selected:

```text
model                  selected / produced   availability adjusted
hf:zai-org/GLM-5.2          51 / 134  38.1%   38.6%
qwen3.8-max                 34 /  94  36.2%   25.8%
hf:Qwen/Qwen3.8-27B         42 / 130  32.3%   31.8%
deepseek-v4-pro-0813        35 / 126  27.8%   26.5%
hf:moonshotai/Kimi-K3       36 / 132  27.3%   27.3%
deepseek-v4-flash-0731      27 / 119  22.7%   20.5%
hf:nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-NVFP4
                            23 / 137  16.8%   17.4%
hf:openai/gpt-oss-120b      23 / 145  15.9%   17.4%
gemma-4-26b-a4b-it          17 / 133  12.8%   12.9%
minimax-m3                  12 / 119  10.1%    9.1%
```

Shares do not sum to one because identical candidates are credited to every model that wrote them.
The adjusted column charges each model zero for rounds it never reached,
which is the reading `#199` settled on.

`qwen3.8-max` has the second best rate among the rounds it survived
and sits fifth of ten once its absences are charged against it.
It is not exceptionally bad.
It is a model whose answers win when they arrive and whose answers are cut before they arrive one time in six,
and the remedy for that is recovering the voice rather than removing the model.

## The recovery round has still never run, and this is exactly the population it was built for

`grep` for the recovery round's own line over the whole log returns nothing:

```text
<stage>: recovery round for <n> unreadable answers
```

The run directory was created at 10:14 local and the recovery round landed in `91f0c8ba5` at 13:57,
so the bundle this run executed cannot contain it.
The rate `#230` owes is therefore still owed,
and the next run carrying the change is where it comes from.

What this reading adds is the size of the prize.
The 72 losses are 2.0% of all model streams,
but they are 15.4% of one model's streams and 3.7% of another's,
and both of those models are in the top three by selection rate.
A recovery round that re-asks them buys back participation
where participation is worth the most.

## What the third shape changes about the grace

Nothing here argues for a shorter straggler grace.
Cut elapsed times for `qwen3.8-max` run from 76858 ms to 254000 ms with a median of 195939 ms,
and its first byte arrives in 1467 ms to 4735 ms.
The connection is healthy and fast to open;
what runs long is the reasoning.
Since this repository never sets a thinking parameter or a token budget,
by standing instruction,
the grace is the only dial that touches this population at all,
and shortening it would convert answers that arrive into answers that do not.
