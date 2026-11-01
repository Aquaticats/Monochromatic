# Lane outcome vocabulary handover: what the stream guards saw in production traffic, 2026-08-18

What the stream idle windows and the straggler grace looked like on real traffic during the flagged pass,
including the re-read across every pass log that carries the stream columns,
the voice-loss shapes,
and the defects tracked from them.

Part of [Handover: the lane outcome vocabulary and what is mid-flight](two-lane-outcome-vocabulary.md),
which lists every file of this record.

## The idle windows rest on a premise the current traffic contradicts

TRACKED AS `#121`,
measurement only,
nothing decided and no constant changed.
`doc/audit/stream-guards-first-production-traffic.md` holds it.

`stream-idle-guard.ts` disables both windows at 600000 ms on the stated ground that first-byte
silence is normal here,
citing p50 95.6 s over 32 streams.
Over 395 streams in this pass:

  firstByte ms   p50 1174   p75 1349   p90 1726   p99 3091   max 9084
  maxGap ms      p50  207   p75  257   p90  463   p99  989   max 11659

About eighty times faster at the median.
Every one of the 395 got a first byte:
zero negative
`firstByteMs` values,
checked by re-grepping with the sign after the first pattern would have
silently dropped them.

TWO REASONS THIS SAMPLE BEATS THE ONE IT DISAGREES WITH:
it is UNCENSORED,
since neither window
fired,
where the old one was taken with a 150 s window live;
and it is UNBIASED BY SURVIVORSHIP,
since `#118` reports progress on the cut path,
where the retracted "1349 streams,
mean firstByte
1822 ms" excluded abandoned calls by construction.

RE-READ ACROSS EVERY PASS LOG THAT CARRIES THE COLUMNS,
5470 streams over three workloads and five
days (`grace-remeasure-snapshot.log` excluded as a copy of `corpus-pass-20260817.log`):

  resume-run-output       2888 streams  p50 1032  p90 1477  p99  3955  max 183755
  corpus-pass-20260817    2166 streams  p50 1237  p90 1951  p99 10163  max  91843
  flagged-pass-2           416 streams  p50 1173  p90 1726  p99  2768  max   9084

THE MIDDLE HOLDS EVERYWHERE and the TAIL IS REAL,
which one pass could not have shown.
Exceedances
over all 5470:
first byte over 30 s in 10 streams,
over 60 s in 2,
over 120 s in 1,
over 300 s in
NONE;
largest gap over 30 s in 4,
over 60 s in 1,
over 120 s in none.

SO A SMALL WINDOW IS RULED OUT rather than supported.
A 30 s first-byte window would fire on about
one call in 550,
roughly one stage in ninety losing a voice to the guard itself.
A 300 s one would
have fired on none and would still end a dead call in five minutes rather than never.
Both counts
are LOWER BOUNDS,
since two of the three logs predate `#118` and hold only completed streams.

IT STILL DOES NOT SAY THE CONSTANTS ARE WRONG.
The change is a decision-doc revision to
`doc/decision/translation-repair-straggler-grace.md`,
which owns the reasoning,
and it belongs to
the user.

## What the first production traffic showed, written up in full

`doc/audit/stream-guards-first-production-traffic.md`.
Read over the pass's first 48 minutes:
347
streams,
219,574,656 raw characters,
median 357,921,
largest 5,558,524.

BOTH NEW INSTRUMENTS WORKED.
The first straggler abandonment now says why:
`panel hf:zai-org/GLM-5.2: abandoned 180000ms after quorum (cut-mid-reply after 2915229 delivered
chars, first byte at 2087ms), voice lost`.
Before `#118` that line read only `voice lost`.
Its
companion progress line records `maxGap 460ms`,
so the call was streaming steadily when the grace
ended it,
and the idle guard could never have seen it.

THE DEGENERATION GUARD HAS STILL NEVER FIRED,
and the straggler grace has cut ten calls.
The
searches that answer this are `stream .*: cut` and `degenerate in`.
A bare `cut` returns false
positives,
all the word "cute" in a rationale about 撒娇.

SUPERSEDED,
and the figure below was a MID-FLIGHT FLOOR read before the pass finished.
Read to
the end of the log by `#121`:
SIXTEEN OF NINETEEN cuts,
84%,
are `hf:zai-org/GLM-5.2`.

  GLM-5.2       328 streams  16 cuts
  Qwen3.6-27B   354 streams   2 cuts   (its two largest deliveries,
  absent from the earlier read)
  Kimi-K3       322 streams   1 cut
  the other three            0 cuts

The earlier reading,
kept because two numbers moved rather than one and a reader comparing them
should see both:

  GLM-5.2      186 streams  9 cuts  0.0484
  Kimi-K3      183 streams  1 cut   0.0055
  GLM-4.7-Flash 183 streams 0 cuts  0.0000
  Qwen3.6-27B  199 streams  0 cuts  0.0000
  gpt-oss-120b 199 streams  0 cuts  0.0000
  Nemotron     199 streams  0 cuts  0.0000

Qwen moving from zero to two matters more than GLM's count rising:
a model with no cuts at all is
a different claim from one whose largest deliveries get cut.

So the grace's cost falls on ONE VOICE and the ensemble loses the same member repeatedly.
NOT a
claim that GLM-5.2 is slow,
which was retracted once already:
first byte on the cut runs ran 751 to
2120 ms,
ordinary for this pass.
They were long,
not slow,
and delivered sizes at the cut vary
twenty-fold from 123534 to 2915229 characters.

VOICE LOSS SPLITS INTO TWO SHAPES,
nine so far:
six straggler abandonments (GLM-5.2 five,
Kimi-K3
one) and three schema mismatches (GLM-4.7-Flash two,
gpt-oss-120b one),
each of the latter reporting
`raw=""` after a stream that COMPLETED.
That qualifies the earlier finding that voice loss was GLM
and never Kimi-K3:
on this pass it is seven of nine GLM but no longer GLM alone.
Five hand-picked
entries are not a corpus sample.

THE CASE THE GUARDS EXIST FOR HAPPENED AND NOTHING CAN DIAGNOSE IT:
GLM-4.7-Flash streamed
2,052,766 raw characters,
COMPLETED on its own,
and delivered `raw=""`,
an empty answer.
Whether it
thought usefully or cycled is unanswerable from the log.

FIVE DEFECTS FOUND AND TRACKED AS `#120`,
all held until the freeze lifts,
all written up in
`doc/decision/translation-repair-runaway-call-termination.md`:

  1  long-period looping escapes the detector entirely:
  what decides a verdict is gcd(period,
  32),
     so a 501-character loop reads healthy at 0.1223 while a 500-character one is refused at 0.0305.
     Probe at `~/temp/agent/degeneration-period-probe.mjs`,
  gcd prediction matched observation 15
     of 15,
  control of never-repeating text scores 1.0000.
  2  `raw chars` and `delivered chars` are the same number,
  235 of 235,
  because `drainBody` returns
     the wire format itself.
  The generated-character count nothing records.
  3  a deliberate termination is filed as `cut`,
  the same word a stall gets.
  4  the runaway error is built from `response.url` rather than `label`,
  so it names the endpoint
     every model shares.
  Driven through the built drain,
  the message opens with a bare colon.
  5  the cut excerpt shows `data: {"id":"chatcmpl-` in every case and never the model's words,
     which is the one thing it was added to show.

WHY 2 MATTERS MOST:
a verdict needs about 131,000 GENERATED characters and production counts only
raw ones.
At a 10x envelope 45 of 347 streams reach the bar;
at 1x,
282 of 347.
The unmeasured
ratio decides whether this guard judges one stream in eight or four in five.
