# OpenRouter and the corpus passes: the providers of XingZ6017 on build 2a336603bf (2026-10-22)

What was read on 2026-10-22 of the providers behind the run of the third pass after the Gemma readers were lifted:
the order in which they ran dry,
the seat tallies,
the streams that held rounds open or were cut,
the NeuralWatt meter and energy,
the text calls that ended at the cap,
and the one picture with the four readings of it.
Entry:
XingZ6017 (entry `XingZ60`).
The log,
the wall clock,
the class verdict,
the lost claims and the classes named are in
[the run of XingZ6017](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-xingz6017-run.md),
and the shipped page,
with its departures from the Chinese,
is in
[the page of XingZ6017](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-xingz6017-page.md).

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record;
the build and what the passes of the restart share are in
[the Gemma-lift restart overview](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart.md).

## XingZ6017 provider read, 2026-10-22, 04:07 UTC: Bedrock held out, then Synthetic, Hyper and NeuralWatt dry in turn; OpenRouter paid 9.218 USD

The provider read is part of the read of `XingZ6017` (SETTLED in 268.7 min),
made at 04:07 UTC on 2026-10-22 on tip `2a336603bf38a6a53aeb4f3e70f0a3447acf5487`.
This build predates commit `e336d8efc7` (first-content window of 180 s,
notes passed to the polish judges and writers;
ledger B804 to B807)
and commit `f829fc31b6` (composite by line difference;
ledger B803),
which landed after the run started.
The K2 counts and the K3 claims in this record are a baseline from before those fixes,
not recurrences after them.
Every count names its method:
a script over the pass log or the artifact,
or an `rg --count` over the log.
The log crosses midnight UTC:
times from 00:00 UTC onward fall on 2026-10-22.

### Providers, in order of failure

Five providers carried the run.
The order of exhaustion is the spine of the slowness.

- 23:05:31.4 UTC (log lines 219 and 220):
  one Bedrock HTTP 401;
  `bedrock is held out for every model for the rest of the run`.
  The first `JUDGE SEATS` line (210) reads `bedrock=wet`,
  and 30 later `JUDGE SEATS` phase lines read `bedrock=dry`;
  all 222 `METERS` lines read `bedrock=wet` with `bedrockUsd=5.02`.
  The seat `google.gemma-4-e2b` was `unreachable` 1200 times and `google.gemma-4-31b` once.
- 23:27:48 UTC (line 1142):
  `synthetic: refused us while its meter reads dry`,
  held out 300 s.
  The 429 run is 23:27:34 to 23:27:50 UTC.
  The `JUDGE SEATS` line at 23:16:50 UTC (478) had already shown `synthetic=dry` for the picture phase.
- 00:03:36 UTC (line 6548):
  Hyper HTTP 402 `You're out of credits` on `glm-5.3` in slice 32;
  line 6555 (00:03:38 UTC) reads `hyper=dry`.
  `hyperBalance` fell from 166 to 0 in about 58 min.
- 01:39:35 UTC (line 23537):
  first `neuralwatt=dry` `METERS` line (`neuralwattWh=-1.170`);
  `routedText ... neuralwatt refused us` at 01:39:20 UTC (lines 23497,
  23506).
- 02:39:44 UTC (line 36261):
  Synthetic reads `wet` again with the weekly meter at 2 percent.
- 03:26:35 UTC (lines 38846 and 38847):
  Synthetic `dry` again,
  `held out for 300000ms`.

From 01:39 UTC the NeuralWatt,
Hyper and Synthetic seats were all dry:
OpenRouter alone carried the run for about 114 min
(Synthetic returned for 47 min inside that span,
in the section on N2).
OpenRouter's balance fell from 13.42 to 4.15 USD;
the `SPEND` lines sum to 9.218 USD:
`minimax/minimax-m3` 4.1606 USD over 1325 calls,
`deepseek/deepseek-v4.1-flash-1017` 1.8472 over 744,
`google/gemma-4-26b-a4b-it` 1.3442 over 1204,
`inception/mercury-2.5` 1.2142 over 1535,
and four smaller lines.
The two prior reads of this build spent 0.13 and 0.07 USD on OpenRouter.
Seat calls by provider:
Hyper 584 (all at 0 cost),
NeuralWatt 1033,
OpenRouter 5632,
Synthetic 85 (Qwen 66,
Kimi 19).

Seat tallies at the end (the `SEAT` lines,
41171 to 41182):

- `google.gemma-4-e2b`:
  asked 0,
  unreachable 1200.
- `gemma-4-26b-a4b-it`:
  asked 1355,
  usable 1348,
  unusable 7.
- `minimax-m3`:
  asked 1515,
  usable 1489,
  unusable 5,
  threw 21,
  abandoned 1.
- `hf:moonshotai/Kimi-K3`:
  asked 192,
  usable 191,
  unusable 1,
  unreachable 397.
- `hf:zai-org/GLM-5.3-Flash`:
  asked 489,
  all usable.
- `inception/mercury-2.5`:
  asked 1455,
  usable 1403,
  unusable 40,
  threw 12,
  abandoned 52.
- `deepseek-v4.1-flash`:
  asked 1508,
  usable 1491,
  unusable 17,
  abandoned 2.
- `glm-5.3`:
  asked 66,
  usable 65,
  threw 1,
  unreachable 682.
- `google.gemma-4-31b`:
  asked 0,
  unreachable 1.
- `hf:Qwen/Qwen3.8-27B`:
  asked 67,
  usable 67,
  abandoned 43,
  unreachable 764.
- `typesafe/jev-1.13`:
  asked 365,
  all usable.
- `xiaomi/mimo-v2.6-flash`:
  asked 244,
  all usable.

`seat unreachable` appears on 3043 lines:
1200 for `gemma-4-e2b` and 1 for `gemma-4-31b` (the Bedrock credential),
and 1843 for out of budget (Qwen 764,
Kimi 397,
`glm-5.3` 682).
`SEATS DARK` never prints.
`HTTP 429` appears on 15 lines.

### The stragglers

#### Hung streams that hold the quorum (new, N1)

When only about four voices are live,
the quorum equals the live bench
and the 120 s grace never starts.
The round then waits for every voice,
including one that is a dead stream.
Examples,
each a round that waited 298 to 319 s with `0ms in grace`:

- Line 15503 (00:46:39 UTC),
  slice 19 checker:
  `2/3 heard, 305342ms total, 305342ms to quorum, 0ms in grace`.
- Line 16005 (00:51:03 UTC),
  slice 31 checker:
  298,286 ms.
- Line 16732 (00:57:58 UTC),
  slice 50 checker:
  306,580 ms.
- Line 17268 (01:01:08 UTC),
  slice 57 introduced-defect probe:
  318,612 ms.
- Line 17524 (01:07:20 UTC),
  slice 81 introduced-defect probe:
  308,898 ms.
  This one was the last event of the repair lane:
  it waited from 01:02:11 UTC for a mercury stream that ended in a 502 (lines 17519 to 17524).
- Line 27349 (01:54:37 UTC),
  lane contest slice 40:
  `4/7 heard, 307817ms total ... 0ms in grace`.
  This single round held the contest open for 2.5 min after every other slice was done.

Scripts over the log and the stream lines show the mechanism.
Mercury has 1535 streams:
1482 completed and 53 cut.
105 had 0 content chars,
and 52 of those 105 were `completed`:
21 ran 290 s or more (about 301 s,
18,181 raw chars,
followed by `code 502 type provider_unavailable ... retrying`),
5 ran 250 to 290 s,
1 ran 120 to 250 s,
6 ran 10 to 120 s,
and 19 ran under 10 s (the 403 `content_policy_violation` and fast 502 lines).
35 of the 52 are followed by a 502 line;
there are 36 transport-failure lines in all.
The 301 s is a gateway or provider bound.
That it is not this pipeline's bound is an inference:
`callConfig.perCallTimeoutMs` is 360000.

Rounds that waited 250 s or more with quorum unmet:
5 in repair chunk,
5 in repair refine,
1 in the contest and 13 in consolidation,
24 in all,
122 round-min.
Ledger B809 gave the Mercury card a first-content window of its own,
90000 ms (open to the owner's veto),
on 2026-10-22 (UTC);
no run has measured it yet.

#### Streams cut by the grace (K2, recurring)

The `abandoned ... after quorum` lines number 98 in all (211 summed grace-min over overlapping rounds),
which equals the `SEAT` abandoned columns (mercury 52,
Qwen 43,
deepseek 2,
minimax 1).
The run did not have commit `e336d8efc7`,
so this split is a baseline from before the fix.

- Mercury,
  52:
  all with 0 answer chars and 0 reasoning chars;
  45 at the 120 s grace and 7 at the 180 s writer grace.
  By phase:
  8 in preparation,
  29 repair,
  3 translate,
  12 consolidation.
  The `SPEND` lines carry `estimated=abandoned` on 100 mercury lines,
  22 minimax lines and 2 deepseek lines;
  the read did not reconcile those with the `SEAT` counts.
- Qwen,
  43:
  40 in consolidation between 02:43:28 and 03:29:10 UTC,
  every one with `answer=0` and a large reasoning body,
  and 3 in preparation (23:20:57 UTC and 23:23:39 UTC on `coverage`,
  23:25:30 UTC).
  This is runaway reasoning,
  not the comment-only stream of K2,
  but the grace cuts it the same way.
- The `SEAT` tallies never count an abandoned call as asked.
  A seat is therefore never withheld for it:
  `inception/mercury-2.5 asked=1455 ... abandoned=52` and
  `hf:Qwen/Qwen3.8-27B asked=67 ... abandoned=43`.
  Every later round waits for the seat again.
  Ledger B810 examined a hold on re-asking abandoned seats and rejected it on 2026-10-22 (UTC):
  at every setting tried it forwent more answers than it saved cuts.

#### Qwen came back for 47 minutes and was wasted (new, N2)

At 02:39:44 UTC Synthetic reads `wet` (2 percent weekly),
and the seat roster re-seats Qwen.
The `slice re-seated under a hold` lines (17 in all;
first at line 38855,
03:26:35 UTC,
last at line 40661,
03:31:00 UTC) list `hf:Qwen/Qwen3.8-27B` as writer and slate judge.
Between 02:43:28 and 03:29:10 UTC 40 Qwen calls were cut at the grace with no answer,
each costing 120 or 180 s of grace in its round,
and the mean consolidation slice cost went from 149 s to 315 s and 346 s.
When Synthetic went `dry` again at 03:26:35 UTC (line 38847) the slice time fell to 104 s.
The read did not test whether leaving Qwen unseated would have saved the time;
the sequence and the numbers point that way.

### NeuralWatt: baseline, dry reading and energy

The brief's sixth question,
answered with the three reads' figures beside it.

- Baseline:
  `NeuralWatt quota at the start of the run: 3.7493 kWh charged this period` (line 202);
  day start `2026-10-21T19:15:08Z` (line 203).
  The pass ran 23:05 to 03:34 UTC,
  inside one NeuralWatt day (the day ends at 19:15 UTC on 2026-10-22).
- First `METERS` line (209):
  487.978 Wh left,
  62.022 Wh account used.
  62.022 is `zheermao22` (40.910) plus `CuspariaKLSY22` (21.112),
  so nothing else drew on the account between the two reads and this pass.
- Last `METERS` reading:
  -5.188 Wh left and 555.188 Wh account used (line 36261 and every later line).
- The meter fell 493.166 Wh (222 meter lines).
  The 1033 settled `NeuralWatt energy` calls sum to 493.1665 Wh measured and 173.5046 Wh estimated;
  the meter and the settled calls agree.
- Measured to estimated is 2.84
  (`glm-5.3-flash` 269 calls,
  281.7212 measured against 91.2949 estimated,
  3.09;
  `deepseek-v4.1-flash` 764 calls,
  211.4453 against 82.2097,
  2.57).
  Over the six runs on this build,
  in the order they ran,
  the ratio read 1.23,
  2.00,
  0.87,
  1.39,
  1.94 and 2.84 (`XingZ6016` held only 31 calls).
  It follows the entry and not the date:
  the same entry read 1.23 and then 1.39 (`zheermao101`),
  and 2.00 and then 1.94 (`CuspariaKLSY`).
  `XingZ60` sent 415 calls of 16,384 prompt tokens or more where the other five runs sent 8 in all,
  and used more energy per thousand completion tokens;
  why that entry costs more is not traced.
- By the hour of the call,
  measured against estimated:
  23:00 UTC hour 231 calls,
  91.43 against 36.59;
  00:00 UTC hour 497 calls,
  191.40 against 82.82;
  01:00 UTC hour 305 calls,
  210.34 against 54.10 (until 01:39 UTC).
- 282 calls of 0.5 Wh or more account for 379.69 Wh,
  77 percent of the total.
- The pass used 90 percent of the 550 Wh free day by itself and passed zero by 5.188 Wh.
  Two calls (4.466 Wh) settled after the first `neuralwatt=dry` line (01:39:35 UTC),
  the last at 01:39:58.094 UTC.
- No NeuralWatt stop line fired.
  That is the designed behaviour and not a defect:
  dry is a routing verdict,
  and calls already in flight settle.
  The defect is the estimate:
  it under-read the measured energy by 2.84 times,
  and ledger B808 recalibrated it from the rotation's settled calls on 2026-10-22 (UTC);
  no run has measured the new figures yet.
- The estimate on each `settleNeuralWattCall` line scales with prompt size
  (0.3725 Wh for 8264 prompt tokens on the `glm-5.3-flash` picture call at line 485,
  0.0389 Wh for 2152 on a `glm-5.3-flash` text call and 0.0233 Wh for 2153 on a `deepseek-v4.1-flash` text call,
  lines 519 and 522),
  so it sees the prompt and not the reasoning output.
  That is why the picture call was over-estimated about 11 times (0.0323 measured)
  while the whole pass under-read by 2.84 times:
  282 calls of 0.5 Wh or more carry 379.69 Wh,
  77 percent of the total.
  That the reservation logic uses this estimate is an inference:
  the reservation path was not read
  (`neuralwattReckonedWh` reads 0.000 on every `METERS` line sampled,
  and the field is built in `package/translation-repair/provider/src/neuralwatt-day-energy.ts:119`,
  which was not traced).
  The admission of 493 Wh of calls against a 488 Wh balance fits an estimate that under-reads (N3).
- `NeuralWatt PLAN ENERGY SPENT: 54.3 Wh` (line 41170),
  which puts the end baseline at about 3.8036 kWh (computed,
  not printed).
  The run's own overshoot is 5.188 Wh,
  so about 49 Wh of the plan figure is not this pass's covered-model use.
  `CuspariaKLSY22` showed the same shape (37.3 Wh with no overshoot).
  It was not traced and is open.
- The `sum` of left and account fell 0.2 to 0.4 Wh short of 550 on some meter lines.
  The earlier reads explained that as in-flight holds;
  it was not re-verified here.

#### K1 recurred in text calls (13 events, 29.91 Wh)

Two scripts find 13 NeuralWatt calls that ended at the completion cap with exactly 0 content chars
(`reportStreamProgress ... 0 content chars`):

- `glm-5.3-flash`,
  7 calls,
  each 18,316 completion tokens,
  20.2083 Wh,
  at log lines 16833 (00:58:53 UTC,
  repair slice 62),
  18840 (01:13:23 UTC,
  translate slice 15),
  19976 (01:19:34 UTC),
  20840 (01:26:40 UTC),
  20902 (01:27:21 UTC),
  21912 (01:32:00 UTC) and
  23712 (01:39:58 UTC).
- `deepseek-v4.1-flash`,
  6 calls,
  each at the 13,082-token cap,
  9.704 Wh,
  at lines 7632,
  16365,
  16866,
  19791,
  20208 and 21425.
- 29.912 Wh in all,
  6.1 percent of the measured 493.17 Wh.

Each was followed by `reaskElsewhereNudged: schema-mismatch on neuralwatt; re-asking another provider`.
All 7 `glm-5.3-flash` events were answered usably on OpenRouter,
and 5 of the 6 `deepseek-v4.1-flash` events were;
the sixth has no outcome line within 400 lines of it (the script matches on the slice tag,
so this is approximate).
The picture call did not show K1:
GLM read the screenshot in 10.4 s at 1619 completion tokens and 70 reasoning chars (the section on pictures).
Text calls on NeuralWatt still go out without `reasoning_effort`;
the ledger entry for the picture lift is `B719`
(`package/translation-repair/doc/audit-ledger-provider-streams-usage-and-retries.md:518`)
and the provider README says the client sends the effort on picture calls only
(`package/translation-repair/provider/README.md:126`).

### Pictures

One picture,
`image0.webp` (2400 by 2616,
1.27 MB;
a maimai DX rating screenshot:
header "maimai でらっくす FESTiVAL PLUS",
`RATING 11600`,
a rank plate `XLL 四段`,
`B35: 8021 + B15: 3579 = 11600`,
50 song cards in rows of five).
The reader viewed it and compared each reply.
Log lines 479 to 496 (23:16:50 to 23:17:55 UTC):

- Local OCR read 58 characters (line 481,
  23:16:51.942 UTC) of a screen of about 50 cards.
- `glm-5.3-flash` on NeuralWatt:
  completed in 10,382 ms,
  2526 content chars,
  70 reasoning chars,
  8188 prompt and 1619 completion tokens,
  0.0323 Wh measured against 0.3725 Wh estimated (lines 482 to 485).
  Payload `12f7e73f33…`.
  Close to the screenshot:
  rating,
  rank,
  `B35` and `B15` figures,
  the song titles and scores of the first rows
  (`ヒバナ 100.5716% 1236/1422`,
  `Altale 97.6053% 939/1206`).
  Wrong header:
  it reads the logo as "オンゲキ",
  and it adds an invented line,
  "うにゃりのレンジャ".
- Kimi-K3 on Hyper:
  63,555 ms,
  2537 content chars,
  8359 prompt and 3729 completion tokens (line 494).
  Payload `b2c50f9aa9…`.
  Best reading:
  the header reads "maimai でらっくす FESTiVAL PLUS",
  `XLL 四段`,
  and the song rows track GLM's.
- `gemma-4-26b-a4b-it` on OpenRouter,
  endpoint DekaLLM:
  21,421 ms,
  2783 content chars,
  465 prompt tokens,
  2070 completion tokens (lines 491 to 493).
  Payload `841ff50ef2…`.
  Diverges:
  `XLL 四級` for 四段,
  `[left]` and `[image]` scaffolding on every card,
  "Altaf" for "Altale",
  `コントロール` for ゴーストルール,
  `サイコパス` for BETTER CHOICE,
  and many invented digits (`10.1 → 945`,
  `1235/1422`,
  `11103/1255`).
- `minimax-m3` on Hyper:
  19,514 ms,
  1685 content chars,
  829 prompt and 2247 completion tokens (lines 488 to 489).
  Payload `d5e71735dc…`.
  Diverges most:
  "Festival♪ PLUS",
  `B15: 8021▶ B15: 3579▶` (two B15 lines;
  the screen has B35 and B15),
  "リスペリ" for ヒバナ,
  "SigSig" for Altale,
  and songs that are not on the screen ("Lavender",
  "DOOMSDAY FESTIVAL",
  "Master Glow Up!
  Song",
  "SEWER ∀LEH").
- `gemma-4-31b`:
  failed outright (Bedrock credential,
  line 495).
  Qwen and Kimi were withheld by the Synthetic meter in the seat line (478)
  but Kimi read the picture through Hyper;
  mercury and deepseek do not read pictures.
  `readers=5 seeing=4` (line 478).

The payload-to-model mapping is by content-character count:
2526,
2783,
2537 and 1685 equal the four `stream ... content chars` figures on lines 482,
491,
494 and 488.

#### What corroborated what

The log prints one line:
`corroborated by 4 of 4 readers at overlap 0.885` (line 496).
That is the closest pair,
not every pair.
`reading-cluster.ts` computes every pair's overlap (line 141) and keeps only the closest for the log (line 178),
and `image-reading-pair.ts:703` logs the vouched count and that closest overlap.
No per-pair overlap is in the log.
A script recomputed the trigram overlap of every pair from the payloads
(the share of the smaller reading's trigrams that the larger carries,
`CORROBORATION_TRIGRAM_SHARE` 0.3,
`reading-corroboration.ts:47`):

- GLM and Kimi:
  0.885 (the printed figure,
  which confirms the mapping).
- GLM and gemma:
  0.570.
- Kimi and gemma:
  0.581.
- GLM and minimax:
  0.358.
- Kimi and minimax:
  0.359.
- gemma and minimax:
  0.333.

`clusterReadings` vouches for any reading that some pair puts at 0.3 or more.
Minimax,
the poorest reading,
passed at 0.358 and 0.359 against the two good ones,
and gemma at 0.333.
So "4 of 4" counts two poor readings as corroborated by the any-pair rule.
That the rule admits them is code;
that their errors do not matter for the pairing sheet is an inference,
because `pairingPictureContext` (`pairing-pictures.ts:35`) reduces the set to one transcript.
`mostCarriedReading` (`most-carried-reading.ts`) picks the reading with the highest mean overlap against the others:
Kimi 0.609,
GLM 0.604,
gemma 0.495,
minimax 0.350.
The 2580-character transcript on the pairing sheet (line 521) is therefore Kimi's reading
(the pipeline logs Kimi's reading as 2134 characters at line 494,
against 2537 content chars on the stream line;
the same gap holds for every reader:
GLM 2526 and 2085,
gemma 2783 and 2321,
minimax 1685 and 1437.
The read did not find what the gap is,
so it does not reconcile 2580 with either figure;
the 2580 is the sheet's rendering of the picture,
headings and side legend included).
The margin over GLM is 0.005.
The picture's content (rating 11600,
the song list) has no text on the page:
"Transcend Lights" (page lines 403 and 1202) comes from source footnote [^3],
not from the reading.

The lift of `B719` is the visible difference from `XingZ6016`:
there the same NeuralWatt call ran to the cap at 18,316 completion tokens with no content;
here it finished at 1619.
The Gemma route is the other change:
the OpenRouter card for `gemma-4-26b-a4b-it` now reads `readsImages: true`
(`package/translation-repair/shared-contract/src/model-cards.ts`,
lines 270 to 277 at `2a336603bf`,
with the comment "THE LISTING'S TRUTH SINCE 2026-10-21"),
and the Hyper card stays `false` (line 265),
so this reader exists only through OpenRouter.
The low prompt-token counts on the Hyper minimax call (829) and the OpenRouter Gemma call (465)
against 8188 and 8359 on GLM and Kimi suggest those two saw a reduced-resolution image.
That is an inference and was not tested.
