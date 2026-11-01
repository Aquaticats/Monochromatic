# OpenRouter and the corpus passes: the providers of XingZ6018 on build 8864316eb2 (2026-10-22)

What was read on 2026-10-22 of how each provider ran out during `XingZ6018` (entry `XingZ60`),
how the router and the stage layer behaved after the dry signal,
the third recheck of the Mercury first-content window,
and the other seats.
In this run the every-provider-dry refusal printed,
which it had not in the passes before it on this build.

The log,
the stages and the plan energy are in
[the run of XingZ6018](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6018-run.md),
which also says how the record was made and that a second reader checked it.
The defects are in
[the defects of XingZ6018](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6018-defects.md).
Line numbers are of the log `temp/agent/corpus-pass-XingZ6018.log`.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## The meters

There are 63 `METERS` lines (`meters.ts`),
from line 210 (13:45:02.299 UTC) to line 14514 (14:46:58.577 UTC).

- NeuralWatt,
  Synthetic and Bedrock read dry on all 63.
  NeuralWatt reads `neuralwattWh=-5.188`,
  `neuralwattReckonedWh=0.000` and `neuralwattAccountWh=555.188`,
  with the day start 2026-10-21T19:15:08.000Z,
  on all 63
  (`rg --count` of the full triple finds 63).
  Synthetic reads `syntheticWeekly=0%` with no five-hour window.
- The Hyper balance,
  by log line:
  153 (210),
  147 (292),
  147 (337),
  145 (383),
  144 (442),
  142 (479),
  130 (738),
  122 (841),
  118 (915),
  110 (996),
  107 (1098),
  99 (1246),
  90 (1442),
  83 (1597),
  76 (1732),
  72 (1968),
  64 (2203),
  58 (2438),
  53 (2631),
  51 (2729),
  51 (2983),
  49 (3100),
  48 (3168),
  47 (3489),
  45 (3690),
  44 (3766),
  44 (3779),
  44 (3809),
  44 (3823),
  43 (4275),
  42 (4444),
  39 (4612),
  34 (4841),
  32 (4943),
  29 (5089),
  26 (5280),
  22 (5481),
  19 (5694),
  17 (5831),
  14 (5957),
  10 (6090),
  8 (6217),
  5 (6343),
  2 (6518),
  0 (6720),
  and 0 on the 18 later lines (7058 to 14514).
  At 153 credits over 775 `SPEND provider=hyper` lines the mean is 0.197 credits per call (`reconcile.ts`).
- The OpenRouter balance in USD,
  by log line:
  3.92 (210 to 479),
  3.88 (1098),
  3.79 (2631),
  3.72 (2983),
  3.55 (3766),
  3.47 (4275),
  3.27 (4943),
  3.18 (6720),
  2.92 (7648),
  2.36 (9054),
  1.85 (9889),
  1.24 (11325),
  0.71 (12214),
  0.46 (13139),
  0.18 (13645),
  -0.03 (14096) and -0.08 (14514).

## Hyper

- Three short holds came first,
  and they were not exhaustion.
  Hyper answered HTTP 429 (80 lines in all,
  the first at 2672 and the last at 3816;
  `rg`),
  mostly between 14:05 and 14:10 UTC,
  and the router marked it refused for 60,000 ms each time while its balance was still 44 or more:
  `refused us while its meter reads wet ... held out for 60000ms`
  (lines 2730,
  3169,
  3767,
  3780,
  3810,
  3811 and 3824;
  `refused.ts`).
  `METERS` printed `hyper=dry` with `hyperBalance=44` for those holds (lines 3779,
  3809 and 3823)
  and wet again at line 4275.
- Exhaustion.
  Hyper's own refusal lines are four `chatText <- <model>: HTTP 402` warnings at 14:28:13.844 to 14:28:14.367 UTC:
  `qwen3.8-27b` (line 6705),
  `minimax-m3` (6708),
  `gemma-4-26b-a4b-it` (6710) and `deepseek-v4.1-flash-1017` (6715).
  They carry no provider text.
  The next reading (line 6720,
  14:28:15.121 UTC) is `hyper=dry hyperBalance=0`,
  and the previous one (line 6518,
  14:27:33 UTC) read 2.
  Then came `markRefused hyper: refused us for payment while its meter reads dry (hyperBalance=0); reads dry until that
  meter moves` (line 6721,
  four identical lines)
  and four `routedText ... hyper refused us, asking the next provider`.
  From then on every Hyper-served model went to OpenRouter.
- Spend after the mark:
  4 `SPEND provider=hyper` lines (6732,
  6739,
  6857 and 7014),
  the last at 14:29:14.592 UTC.
  Each belongs to a stream that started before the mark
  (start times 14:27:33 to 14:27:54 UTC from the stream end minus its elapsed time;
  `postdry.ts`).
  `newstreams.ts` finds 0 Hyper-model streams that started after 14:28:15.121 UTC.
- Hyper lasted 43.2 min of the 62.1 min.
  The OpenRouter bill rose from 0.7375 USD over the first 43.2 min (487 calls)
  to 3.2263 USD over the last 18.7 min (1,365 calls),
  about ten times the rate (`reconcile.ts`,
  `summary.ts`).

## OpenRouter

- `SPEND provider=openrouter` has 1,852 lines,
  the last at 14:46:57.334 UTC (line 14498).
- The `cost=` fields sum to 3.963782 USD (`summary.ts`;
  every one of the 1,852 lines has a `cost=`).
  Of that,
  14 lines carry `estimated=abandoned` and sum to 0.066274 USD:
  8 Mercury lines
  (the seven `code 403 content_policy_violation` streams and the one `code 502` stream:
  lines 267,
  481,
  1273,
  1392,
  1451,
  6184,
  10285 and 11696)
  and 6 `minimax/minimax-m3` lines
  (five content-bound overruns,
  lines 3606,
  8999,
  10314,
  10618 and 12951,
  and one runaway,
  line 14498).
  Settled costs sum to 3.897507 USD.
  The estimated costs are counted in the 3.963782 as the log states them;
  the sums here split them out.
  The second reader recomputed the 3.963782,
  0.066274 and 3.897507 sums and found them as given.
- By model (`summary.ts`):
  `deepseek/deepseek-v4.1-flash-1017` 331 calls,
  1.364937 USD;
  `minimax/minimax-m3` 290 calls,
  1.319446;
  `google/gemma-4-26b-a4b-it` 310 calls,
  0.498058;
  `inception/mercury-2.5` 593 calls,
  0.481888;
  `typesafe/jev-1.13` 137 calls,
  0.182020;
  `xiaomi/mimo-v2.6-flash` 144 calls,
  0.061333;
  `z-ai/glm-5.3-flash` 47 calls,
  0.056101.
- The meter fell 4.00 USD (3.92 to -0.08) against 3.9638 booked.
  The meter lags the booked costs:
  the difference swung between -0.15 (line 4275) and +0.03 (line 12214) during the run,
  so the final +0.036 (the reading at line 14514,
  one second after the last `SPEND`) is inside what the meter showed
  earlier and says nothing by itself;
  each reading is rounded to a cent (`reconcile.ts`).
- The first 402.
  Four `chatText <- <model>: HTTP 402` warnings came at 14:45:42.292 to 14:45:42.341 UTC
  (`minimax/minimax-m3` 14065,
  `google/gemma-4-26b-a4b-it` 14067,
  `deepseek/deepseek-v4.1-flash-1017` 14069 and
  `inception/mercury-2.5` 14072),
  and a fifth,
  `typesafe/jev-1.13`,
  has only its stream line (14070);
  the body of that one is quoted at line 14105
  (`Insufficient credits. Add more using https://openrouter.ai/settings/credits`,
  `code 402`).
  The Mercury 402 is quoted at line 14106.
  `HTTP 402` appears on ten lines of the log in all:
  Hyper 6705,
  6708,
  6710 and 6715;
  OpenRouter 14065,
  14067,
  14069 and 14072;
  and 14105 and 14106,
  which quote the provider's body.
  No earlier OpenRouter 402 exists
  (the other hits on `402`,
  at lines 9547 and 12036,
  are `402 content chars`).
  The router then read the meters at 14:45:44.316 UTC (line 14096,
  `openrouter=dry openrouterUsd=-0.03`)
  and marked OpenRouter refused (line 14097,
  five identical lines).
  The last balance before that,
  at line 13645 (14:44:59 UTC),
  was 0.18 USD.
  The provider's own remedy text says the request must fit the remaining balance,
  so a 402 can precede a zero balance.
- Spend after the first 402:
  6 `SPEND` lines (14079,
  14177,
  14433,
  14435,
  14496 and 14498) totalling 0.026143 USD,
  all for streams that started before 14:45:42.292 UTC (`postdry.ts`);
  the stream of line 14498 is the 78.5 s runaway recorded as defect D5.
  `newstreams.ts` finds 0 OpenRouter streams that started after 14:45:42.292 UTC.

## Calls refused after the dry signal

- Wire 402 responses:
  5 on OpenRouter (lines 14065 to 14072,
  and the body quoted at 14105)
  and 4 on Hyper (6705 to 6715).
- From line 14105 to the end (`after.ts`,
  `refusals.ts`,
  and the second reader's recount):
  - There are 994 `voice lost` lines (14105 to 16237).
    They split into 2 wire-402 lines (14105 and 14106),
    990 lines that carry the every-provider text,
    1 line of the runaway minimax stream (14499,
    a `degenerate` stream in slice 82),
    and 1 summary line (16237,
    `SEATS DARK`,
    which only quotes the words).
  - 992 lines in all carry the every-provider text (`Every provider is out of budget at once`),
    the first at 14107 and the last at 16205:
    903 before the stop at line 15797 and 89 in the two re-entries.
    Of those 992,
    990 are `voice lost` lines and two (lines 16010 and 16011) are picture-reader `failed outright`
    lines.
  - By stage,
    the 990:
    refiner 720,
    panel 80,
    block-pairing 76,
    select 47,
    critic 32,
    checker 12,
    introduced-defect-probe 12,
    section-pairing 8,
    reference-attest 2 and page-title-lexicon 1.
  - By model:
    `deepseek-v4.1-flash` 288,
    `minimax-m3` 275,
    `hf:zai-org/GLM-5.3-Flash` 244,
    `hf:Qwen/Qwen3.8-27B` 64,
    `inception/mercury-2.5` 48,
    `gemma-4-26b-a4b-it` 42,
    `glm-5.3` 14,
    `xiaomi/mimo-v2.6-flash` 8,
    `google.gemma-4-e2b` 4 and `hf:moonshotai/Kimi-K3` 3,
    with two lines unparsed.
    They touched 64 distinct repair slices.
  - 24 `no provider can take ...` lines (a seat unreachable;
    14118 to 16168):
    4 before the stop and 20 after.
  - 240 `retry round` lines:
    92 of round 1 from line 14116 (the 93rd,
    at line 10621,
    stands before the dry mark),
    74 of round 2 and 74 of round 3,
    which is the ladder of three rounds.
  - 63 `stage-quorum-unmet` mentions (14181 to 15757).
- Lines 14096 to 16238 are 2,143 lines and 1,106,003 bytes,
  19.6 percent of the log's bytes (`sed` and `wc`),
  because the refusal text is about 890 characters and is printed 992 times.
- The per-seat `threw` counts in the `SEAT` lines are these refusals.
  `deepseek-v4.1-flash` 288 equals its 288 refusal lines;
  `minimax-m3` 281 equals 275 refusals plus 5 content-bound overruns plus the runaway;
  and `inception/mercury-2.5` 56 equals 48 refusals plus 7 content-policy refusals plus the one 402.

## How the stop behaved

What worked as designed:

- No chat stream was started after either mark
  (0 streams started after the Hyper mark or after the first OpenRouter 402),
  and no `SPEND` line followed except for streams already in flight.
  The meter polls did continue.
  After the OpenRouter 402 the checker found 4 `credits`,
  3 `quota`,
  2 `quotas`,
  2 `models` and 2 `usage-by-model`
  streams started,
  and between the two marks 32 `credits` and 16 each of `models`,
  `quota`,
  `quotas` and `usage-by-model`.
  These are reads and not purchases,
  so the finding about spend stands,
  but a statement that no request at all left the process would be too strong.
- The message names all five providers and the measured meter states
  (`meters read neuralwatt dry, synthetic dry, bedrock dry, hyper dry, openrouter dry`,
  first at line 14107;
  the variant `(just refused this call)` on the calls that hit the mark).
- The entry stopped through the writing-bench floor (line 15797),
  and the driver then stopped re-entering it (`STALLED`,
  line 16221).

What did not match the design text or the facts:

- The error says `this run ends`,
  and its class doc says `ENDS THE RUN, at the owner's instruction`
  (`provider/src/budget-routing.ts:49` and `:83` to `:89`).
  In the log it ended nothing by itself:
  each throw became a `voice lost`,
  rounds closed with no quorum,
  three retry rounds ran per stage,
  60 slices went through a refine pass that could not hear anybody,
  and the entry ended 77.7 s after the mark (14:45:44.316 to 14:47:01.971 UTC) through the bench floor.
  This is defect D1 (ledger B818).
- Most of those 77.7 s were one call in flight,
  not the retries.
  The runaway `minimax/minimax-m3` stream of slice 82 ran from 14:45:38.9 to 14:46:57.3 UTC (line 14497,
  78,465 ms),
  and the chunk pass could not close that slice before it ended.
  The refine pass,
  the shipping steps and the bench check then took 3.4 s.
- The message says NeuralWatt's free energy returns at 00:00 UTC,
  while the same log says the NeuralWatt day starts at 19:15:08 UTC (lines 205 and 210).
  This is defect D2 (ledger B819).

## Third Mercury recheck

`inception/mercury-2.5` on OpenRouter,
under the 90,000 ms first-content window the owner kept as "Keep,
recheck".
The method is `mercury.ts` over the `stream inception/mercury-2.5` lines (`calls.ts` parses them) and `rg`;
the second reader recounted the stream totals,
the percentiles and the maximum from the log.

- Calls:
  594 stream lines,
  all `completed`;
  none `cut`,
  none `overrun`,
  none `degenerate`.
  `SEAT inception/mercury-2.5 asked=668 usable=606 unusable=6 threw=56 silent=0 abandoned=0 unreachable=0` (line 16228).
  Abandoned 0 by the tally and by the log
  (the only `abandoned ... after quorum` line is Qwen's,
  line 6638).
  593 `SPEND` lines;
  the 594th stream is the HTTP 402 stream of line 14071,
  which has no `SPEND`.
  The tally's `asked` (668) is not the stream count:
  it also counts 27 `PROMPT-REUSE` answers from memory and the refused calls,
  and the counters were not reconciled further.
  The cost was 0.481888 USD for 1,421,327 completion tokens,
  and the endpoint was `Inception` on all 585 settled lines (the 8 estimated lines have none).
- First content,
  over the 584 streams that carry one (10 completed with no content,
  listed in this section):
  - minimum 2,415 ms (line 7546),
    median 6,674 ms,
    mean 6,850 ms,
    90th percentile 9,136 ms,
    95th 9,749 ms,
    99th 13,110 ms
    (nearest rank,
    the zero-based index of the floor of the count times the share).
  - The five slowest were 15,283 ms (line 4544,
    repair slice 19,
    14:13:38 UTC),
    14,794 ms (line 1646,
    slice 2),
    13,667 ms (line 6402,
    slice 34),
    13,209 ms (line 1601,
    slice 5)
    and 13,208 ms (line 6406,
    slice 35).
    The sixth and seventh are 13,110 ms (line 393) and 12,581 ms (line 8080).
  - The longest elapsed time was 15,475 ms (line 4544),
    and the median elapsed time 6,749 ms.
  - First byte came at a median of 904 ms and a maximum of 2,064 ms.
  - The ten-minute medians run from 6,140 to 7,236 ms;
    the median of the first half is 6,601 ms and of the second half 6,729 ms.
- The 90,000 ms window was never reached:
  0 streams at 20 s or more of first content or elapsed time,
  and 0 at 30,
  60 or 90 s.
  The largest first content is 17 percent of the window (5.9 times smaller).
  The window is printed nowhere in the log and no artifact was written,
  so what the run used is read from the source:
  `shared-contract/src/model-cards.ts:431` names `firstContentMs: 90_000` at the run's tip.
- Completed streams with no content (10;
  `mercury.ts`,
  `firstContent -1ms`):
  - 7 are `code 403, type content_policy_violation, served by Inception` refusals,
    each followed by `voice lost` and no re-ask because repeating meets the same refusal
    (lines 268,
    482,
    1274,
    1393,
    1452,
    6185 and 10286);
    they took 0.8 to 2.5 s.
    `XingZ6017` had 11 such lines;
    `zheermao23` and `CuspariaKLSY23` had none (`rg --count`).
  - 1 is a `code 502, type provider_unavailable` stream retried in 817 ms
    (stream line 11695,
    warning 11697,
    7,945 ms).
  - 1 is the HTTP 402 stream (line 14071,
    414 ms).
  - 1 is an empty reply with 2,513 completion tokens billed and 0 content characters after 6,250 ms (line 801),
    followed by a `schema-mismatch` re-ask (line 803)
    and a `voice lost` for an empty raw reply in the coverage stage.
    It is the only stream that sent bytes and no content;
    it ended in 6.3 s,
    so it is not the stall shape.
- Unusable answers:
  6 (the `schema-mismatch` voice losses on Mercury,
  the first at line 767).
- The comparison,
  same build and pipeline,
  runs in order:
  - Before the window (from the `zheermao23` read):
    `zheermao22` 11 abandoned of 107 asked,
    `CuspariaKLSY22` 33 cut of 69 streams,
    `XingZ6017` 53 cut of 1,535 streams.
  - First recheck,
    `zheermao23` (12:31 to 13:21 UTC):
    121 streams,
    0 stalled,
    maximum first content 12,820 ms (line 2112),
    median 6,504 ms,
    2 no-content streams (502).
  - Second recheck,
    `CuspariaKLSY23` (13:22 to 13:39 UTC):
    64 of 64 completed,
    maximum first content 21,327 ms (line 1530),
    median 6,656 ms.
  - Third recheck,
    this run (13:45 to 14:47 UTC):
    594 completed,
    maximum 15,283 ms,
    median 6,674 ms.
  - Together,
    779 completed streams in three runs since the window,
    none cut or abandoned.
    This run alone is five times the sample of the first recheck and 62 minutes long;
    the busiest ten minutes (14:30 to 14:40 UTC,
    189 streams with first content)
    had a median of 6,758 ms and a maximum of 12,581 ms.
- What it shows and what it does not:
  Mercury answered within 15.5 s on every one of 594 streams across 62 minutes,
  with a median the same as the model card's (6.7 s,
  from the `CuspariaKLSY23` read).
  It does not exercise the window,
  because nothing came near it,
  so it still neither confirms that the window works nor shows that it is needed.
  Three runs make a stronger absence of stalls than two,
  but the earlier stalls were seen in a different part of the day (21:59 to 03:34 UTC),
  and this record cannot separate a time effect from the window.

## Other seats

The `SEAT` lines 16225 to 16236 were printed after `DONE`;
their `threw` figures are mostly the refusals recorded in "Calls refused after the dry signal".

- `google.gemma-4-e2b`:
  asked 4,
  usable 0,
  threw 4,
  unreachable 359 (Bedrock off,
  line 198).
- `deepseek-v4.1-flash`:
  asked 785,
  usable 496,
  unusable 1,
  threw 288.
- `gemma-4-26b-a4b-it`:
  asked 520,
  usable 474,
  unusable 4,
  threw 42.
- `inception/mercury-2.5`:
  asked 668,
  usable 606,
  unusable 6,
  threw 56.
- `minimax-m3`:
  asked 753,
  usable 471,
  unusable 1,
  threw 281.
- `glm-5.3`:
  asked 97,
  usable 83,
  threw 14,
  unreachable 44.
- `hf:zai-org/GLM-5.3-Flash`:
  asked 390,
  usable 146,
  threw 244.
- `hf:moonshotai/Kimi-K3`:
  asked 120,
  usable 117,
  threw 3,
  unreachable 1.
- `google.gemma-4-31b`:
  asked 1,
  threw 1,
  unreachable 1 (the picture step,
  line 459).
- `typesafe/jev-1.13`:
  asked 138,
  usable 137,
  threw 1,
  unreachable 3.
- `xiaomi/mimo-v2.6-flash`:
  asked 152,
  usable 144,
  threw 8.
- `hf:Qwen/Qwen3.8-27B`:
  asked 172,
  usable 107,
  threw 65,
  abandoned 1,
  unreachable 334.
- `SEATS DARK` (line 16237) names `google.gemma-4-e2b` and `google.gemma-4-31b` only,
  both of which never had a provider.

Notes against the earlier reads (read by the first reader only;
the second reader did not recount these):

- Kimi was not cut at the grace this run.
  `hf:moonshotai/Kimi-K3` (Hyper `kimi-k3`) has 82 streams,
  all `completed`,
  the longest 98,913 ms (line 1661)
  and the slowest first content 37,025 ms (line 2224,
  slice 4);
  `CuspariaKLSY23` had one cut and `zheermao23` none.
- Qwen was cut at the grace once.
  Hyper `qwen3.8-27b`,
  stream `cut` after 140,726 ms with 1,168 answer characters and 22,311 reasoning characters
  (line 6615),
  `abandoned 120000ms after quorum` (line 6638,
  slice 33 select);
  the round lost the full 120,001 ms (round line 6614).
  `zheermao23` had six Qwen cuts.
  Qwen ran only on Hyper (107 calls) because Synthetic was dry from the start;
  after Hyper's 429 holds and dry mark it was unreachable 334 times.
- `minimax-m3` cap-outs and runaways:
  - Hyper:
    one `max_tokens` payload at 10,822 completion tokens
    (the card's cap;
    `SPEND` line 1184,
    slice 3,
    0 text characters),
    against three in `CuspariaKLSY23`;
    the 21,645-token figure did not recur.
  - OpenRouter (`Together`):
    five `overrun` streams ended at the 32,000-character content bound with 32,006 to 33,070 characters
    (lines 3605,
    8998,
    10313,
    10617 and 12950),
    after 103.8,
    24.6,
    83.0,
    272.7 and 130.9 s,
    and one `degenerate` stream (line 14497,
    defect D5).
    The 272.7 s stream (slice 51) is the longest of the run.
    All six cost an estimate (`estimated=abandoned`).
- `hf:zai-org/GLM-5.3-Flash` on Hyper:
  three `max_tokens` payloads at 18,316 completion tokens with 0 text characters,
  all in the preparation (`SPEND` lines 457,
  833 and 884);
  the picture reader returned nothing (line 458).
  Each was re-asked elsewhere in the coverage stage.
- `gemma-4-26b-a4b-it`:
  Hyper had 11 streams with 30 s or more to first content,
  the slowest 60,654 ms (line 870,
  preparation);
  four are in the preparation (lines 314,
  752,
  812 and 870)
  and four in the first three minutes of the repair lane (lines 1224,
  1228,
  1346 and 1363);
  4 `schema-mismatch` voice losses.
  OpenRouter (`Makora`) had two `length` replies at 3,831 completion tokens (lines 7132 and 9982),
  and the slowest first content was 12,760 ms (line 10181).
  `panel ballot ... irregular` (group index out of range) appears 43 times,
  37 from this model and 6 from Mercury (`rg`).
  `CuspariaKLSY23`'s slowest first content of any model was this model at 28,953 ms.
- `glm-5.3` on Hyper:
  66 streams,
  the longest 178,925 ms (line 986,
  completed)
  and the slowest first content 9,192 ms (line 4417).
- The slowest first content per model (`slow.ts`):
  Hyper `gemma-4-26b-a4b-it` 60,654 ms (870);
  Hyper `kimi-k3` 37,025 (2224);
  OpenRouter `minimax/minimax-m3` 19,821 (10823);
  `xiaomi/mimo-v2.6-flash` 17,670 (8417);
  `inception/mercury-2.5` 15,283 (4544);
  OpenRouter `google/gemma-4-26b-a4b-it` 12,760 (10181);
  Hyper `qwen3.8-27b` 10,186 (6075);
  `glm-5.3` 9,192 (4417);
  `glm-5.3-flash` 8,130 (1035);
  Hyper `minimax-m3` 7,349 (686);
  `deepseek-v4.1-flash-1017` 4,921 (967);
  OpenRouter `deepseek/deepseek-v4.1-flash-1017` 3,913 (4058);
  and `z-ai/glm-5.3-flash` 1,757 (8021).
  The largest first content anywhere is under the `zheermao23` extreme (122,849 ms)
  and over `CuspariaKLSY23`'s 28,953 ms.
  Only one stream (Hyper gemma,
  line 870) reached 60 s to first content,
  and none reached 90 s.
- Grace:
  of 717 rounds before the dry mark,
  2 had 100 s or more in grace
  (line 6614,
  Qwen,
  120,001 ms;
  line 988,
  coverage,
  104,743 ms;
  `rounds.ts`),
  against 7 of 205 in `zheermao23` and 1 of 85 in `CuspariaKLSY23`.
  The ordinary grace is 120 s and the writer grace 180 s (line 197);
  no round reached 140 s in grace.
- The OpenRouter endpoints seen were `Inception`,
  `Morph` (deepseek),
  `Makora` (gemma,
  mimo),
  `Together` (minimax)
  and `Wafer` (glm-5.3-flash).
