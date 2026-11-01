# Roster calibration: the producer and editor standings, reliability and spend

The nine-model roster measured and how the standings are read;
the 40-round producer standing and the 6-slice editor and refiner replicate;
the reliability of the two new admissions,
the meters and spend,
and the slice-clustered reading of the replicate.

Part of [Roster calibration under the owner blocklist, 2026-09-01](translation-repair-roster-calibration-2026-09-01.md),
which lists every file of this record.

## The roster measured

Synthetic (subscription):
`hf:Qwen/Qwen3.8-27B`,
`hf:moonshotai/Kimi-K3`,
`hf:openai/gpt-oss-120b`,
`hf:zai-org/GLM-5.3-Flash`.
Charm Hyper (metered credits):
`glm-5.3`,
`gemma-4-26b-a4b-it`,
`minimax-m3`,
`deepseek-v4-pro-0813`,
`deepseek-v4-flash-0731`.
`glm-5.3` and `GLM-5.3-Flash` are the two admissions of the post-blocklist catalog refresh;
`qwen3.8-flash` and `qwen3.8-2.4t-a95b` were culled before either run on the forced-tool-choice probe.

Seats in production at the time of measurement (`src/corpus-run/run-config.ts`):
critics,
panel,
judges and translators are the whole roster;
editors and refiners are Kimi-K3,
Qwen3.8-27B and gemma-4-26b-a4b-it;
checkers are Qwen3.8-27B,
Kimi-K3 and gpt-oss-120b with self-certification permitted at half weight.

Quorum arithmetic at nine:
`gatherStageVoices` needs `ceil(9 / 2)`,
which is 5 voices.
`minBallotWeight` stays the absolute 3,
so the share of the panel that must cast a non-abstain ballot is 3 of 9,
33 percent,
inside the range already lived at ten models (30 percent) and eight (37.5 percent).

## How the numbers are read

Every standing counts disinterested ballots only:
a judge's ballot for its own candidate is excluded.
The pooled null is every disinterested win over every disinterested ballot in that standing,
and each model's z is its share against that null with the null's own standard error,
which is the ballot-level statistic the 2026-08-24 seating used.
The Bonferroni two-sided threshold for nine comparisons is z 2.77.
The availability-adjusted share charges a model zero for every round it produced nothing in:
raw share times candidates over rounds.
Ballots within one round share a slate and are not independent,
so the z is a ranking aid rather than a p-value.

## Producer calibration, 40 rounds

Every model wrote each slice from the source and every model judged.
Ran 18:15:54Z to 20:57:58Z (9734 s),
one slice at a time,
under the 180000 ms production straggler window because `producer-calibrate` has no calibration dial yet
(`doc/decision/translation-repair-calibration-overlap.md` lists that as follow-up work).

Standing as printed,
best first:

-   `hf:Qwen/Qwen3.8-27B`:
    35.5 percent (83 of 234 disinterested ballots,
    over 40 candidates)
-   `glm-5.3`:
    17.2 percent (35 of 204,
    over 36 candidates)
-   `gemma-4-26b-a4b-it`:
    16.4 percent (36 of 219,
    over 40 candidates)
-   `hf:zai-org/GLM-5.3-Flash`:
    12.7 percent (27 of 213,
    over 38 candidates)
-   `minimax-m3`:
    11.8 percent (25 of 211,
    over 40 candidates)
-   `hf:moonshotai/Kimi-K3`:
    9.5 percent (21 of 221,
    over 40 candidates)
-   `deepseek-v4-pro-0813`:
    6.9 percent (15 of 218,
    over 40 candidates)
-   `hf:openai/gpt-oss-120b`:
    2.4 percent (5 of 207,
    over 40 candidates)
-   `deepseek-v4-flash-0731`:
    2.4 percent (5 of 208,
    over 40 candidates)

Pooled null 13.02 percent (252 wins of 1935 ballots).
Read against it:

-   `Qwen3.8-27B` clears the null at z +10.20,
    adjusted share 35.5 percent,
    40 of 40 candidates.
-   `gemma-4-26b-a4b-it` z +1.50 and `glm-5.3` z +1.75 (adjusted 15.4 percent,
    36 of 40) are not separable from the null.
-   `GLM-5.3-Flash` z -0.15 (adjusted 12.0 percent,
    38 of 40),
    `minimax-m3` z -0.51,
    `Kimi-K3` z -1.56,
    `deepseek-v4-pro-0813` z -2.69:
    not separable.
-   `gpt-oss-120b` z -4.53 and `deepseek-v4-flash-0731` z -4.55 sit under the null.

Seat lines:
every model answered every ask usably except `Kimi-K3` (78 of 79,
one thrown),
`GLM-5.3-Flash` (77 of 82,
five thrown) and `glm-5.3` (63 of 78,
fifteen thrown).

## Editor calibration, 6-slice default

Every model edited each slice through the whole repair lane and every model judged;
the refiner standing comes off the same spend.
Ran 20:59:21Z to 21:35:36Z (2185 s),
four slices in flight,
300000 ms window.
Two of the six slices (GLaDOSister chunk 0,
zheermao101 chunk 8) carried nothing eligible and bought no round;
the other four (Everythings99 chunk 6,
lxyddice chunk 2,
shihai4h chunk 12,
donotexist_A chunk 1)
bought 14 editor rounds and 4 refiner rounds.

EDITOR standing over 14 judged rounds,
from 4 of 6 slices:

-   `hf:zai-org/GLM-5.3-Flash`:
    28.8 percent (30 of 104,
    over 16 candidates)
-   `glm-5.3`:
    25.8 percent (24 of 93,
    over 16 candidates)
-   `hf:Qwen/Qwen3.8-27B`:
    20.0 percent (20 of 100,
    over 16 candidates)
-   `hf:moonshotai/Kimi-K3`:
    13.1 percent (13 of 99,
    over 16 candidates)
-   `deepseek-v4-pro-0813`:
    7.1 percent (6 of 85,
    over 14 candidates)
-   `minimax-m3`:
    6.8 percent (6 of 88,
    over 15 candidates)
-   `hf:openai/gpt-oss-120b`:
    5.5 percent (5 of 91,
    over 15 candidates)
-   `gemma-4-26b-a4b-it`:
    1.1 percent (1 of 90,
    over 12 candidates)
-   `deepseek-v4-flash-0731`:
    0.0 percent (0 of 90,
    over 14 candidates)

Pooled null 12.50 percent (105 of 840).
`GLM-5.3-Flash` z +5.04 and `glm-5.3` z +3.88 clear it;
`Qwen3.8-27B` z +2.27 and `Kimi-K3` z +0.19 do not separate;
`gemma-4-26b-a4b-it` z -3.27 and `deepseek-v4-flash-0731` z -3.59 sit under it.
The gemma reading is the one that matters for the current seats:
a model that writes from the source at the roster's second-best rate wrote the worst edits on this sample,
which is the divergence the editor instrument exists to catch and the writer instrument cannot.

REFINER standing over 4 judged rounds,
from 4 of 6 slices:

-   `hf:zai-org/GLM-5.3-Flash`:
    34.4 percent (11 of 32,
    over 4 candidates)
-   `minimax-m3`:
    31.3 percent (5 of 16,
    over 2 candidates)
-   `deepseek-v4-pro-0813`:
    18.8 percent (6 of 32,
    over 4 candidates)
-   `hf:moonshotai/Kimi-K3`:
    18.8 percent (3 of 16,
    over 2 candidates)
-   `gemma-4-26b-a4b-it`:
    9.4 percent (3 of 32,
    over 4 candidates)
-   `hf:openai/gpt-oss-120b`:
    4.2 percent (1 of 24,
    over 3 candidates)
-   `glm-5.3`:
    4.2 percent (1 of 24,
    over 3 candidates)
-   `hf:Qwen/Qwen3.8-27B`:
    0.0 percent (0 of 16,
    over 2 candidates)
-   `deepseek-v4-flash-0731`:
    0.0 percent (0 of 8,
    over 1 candidate)

Pooled null 15.00 percent (30 of 200);
only `GLM-5.3-Flash` clears it (z +3.07),
and four rounds decide nothing.

Editors shipped on 4 of 6 slices:
`glm-5.3` wrote shipping text on two,
`GLM-5.3-Flash` on one,
`Qwen3.8-27B` on one.
Shipping is not a preference:
it includes text every editor proposed identically,
which no judge was asked about.

Seat lines:
every model was asked 54 times;
`Kimi-K3` and `glm-5.3` each lost one,
the rest answered all.
Other warnings:
`deepseek-v4-flash-0731` failed the select schema guard twice (parsed JSON,
wrong shape,
recovery round failed the same way),
`Qwen3.8-27B` returned one select answer with a bad control character,
one HTTP 503 was retried.

Why this run does not seat anyone:
`doc/decision/translation-repair-calibration-overlap.md` records a four-slice
EDITOR standing moving one model from 52.2 percent to 26.7 to 10.0 across identical runs,
and names more slices as the remedy.
`bench-sample.ts` draws a deterministic stride over slices ordered by source size,
so the 40-slice draw is a different sample from the 6-slice draw and the two are not pooled.

## Reliability of the two new admissions

The two instruments ran under different straggler windows,
and that is where the `glm-5.3` numbers come from:

-   Under the 180000 ms production window (producer calibration) `glm-5.3` lost 14 streams to the window
    after quorum (10 at select,
    4 at translate) plus one select AbortError:
    15 of 78 asks,
    19 percent.
    Read per role:
    4 of 40 translate asks (10 percent) as a writer,
    11 of 38 select asks (29 percent) as a judge
    over a slate of up to ten candidates.
    Each cut stream had delivered about 2 M raw SSE characters,
    zero content characters and tens of thousands
    of reasoning characters:
    the model reasons past the window and never reaches its answer.
-   Under the 300000 ms calibration window (editor calibration) it lost 1 of 54,
    a panel stream cut after 3.45 M
    raw characters.
-   The corpus pass runs under the 180000 ms window (`corpus-pass.ts` resolves `STRAGGLER_GRACE_MS`),
    so the 19 percent figure is the one that describes production.
    A lost judge voice thins a nine-judge panel rather than biasing it,
    and the stage had already reached quorum
    every time;
    the cost is metered output.
-   `GLM-5.3-Flash` lost 6 streams the same way in the producer run (3 select,
    2 translate,
    one transport cut
    retried) and none in the editor run.

Cut streams emit no `SPEND` line:
`reportSpend` in `hyper-client.ts` and `synthetic-client.ts` runs after
`extractAnthropicCompletion` on a drained reply,
and a stream abandoned at the window never reaches it.
`spend-report` therefore understates `glm-5.3`.
Summing the reasoning characters the cut streams delivered (954109 in the producer run,
121962 in the editor run)
at four characters per token gives about 238657 and 30491 output tokens,
which at the 95.8144 credits per million output rate is about 22.9 and 2.9 credits unbilled in the logs.

## Meters and spend

Synthetic weekly allowance remaining:
19.25 percent when the producer run started (18:15Z),
19.16 at its last reading (20:53Z),
19.07 when the editor run started (20:59Z),
17.83 at its last reading (21:34Z).
The five-hour meter never fell under 2737 of 2750.
Both runs cost near 1.6 points of the weekly allowance gross once the allowance's continuous regeneration
over each run's wall clock is added back.
Charm Hyper balance:
7272 to 7135 credits across the producer run,
7134 to 7077 across the editor run
(one credit is five cents).

Every Synthetic seat has a Hyper route (`qwen3.8-27b`,
`kimi-k3`,
`gpt-oss-120b`,
`glm-5.3-flash`),
so a dry Synthetic allowance moves cost onto credits rather than emptying seats.

Token totals from `SPEND` lines,
prompt then completion:

-   Producer run,
    687 lines:
    `deepseek-v4-flash-0731` 89966 / 16960 over 78 calls;
    `deepseek-v4-pro-0813` 183768 / 16779 over 79;
    `gemma-4-26b-a4b-it` 160555 / 10246 over 79;
    `glm-5.3` 97159 / 434729 over 63;
    `minimax-m3` 46382 / 280952 over 77;
    `Qwen3.8-27B` 153309 / 498935 over 77;
    `Kimi-K3` 162449 / 67601 over 78;
    `gpt-oss-120b` 155093 / 93144 over 79;
    `GLM-5.3-Flash` 146386 / 403609 over 77.
-   Editor run,
    484 lines:
    `deepseek-v4-flash-0731` 133204 / 11596 over 54;
    `deepseek-v4-pro-0813` 147386 / 8964 over 54;
    `gemma-4-26b-a4b-it` 115793 / 6988 over 54;
    `glm-5.3` 96001 / 334010 over 53;
    `minimax-m3` 60414 / 150677 over 54;
    `Qwen3.8-27B` 128109 / 270724 over 54;
    `Kimi-K3` 130550 / 32810 over 53;
    `gpt-oss-120b` 125548 / 53706 over 54;
    `GLM-5.3-Flash` 121381 / 281479 over 54.

Hyper rates in credits per million tokens as read 2026-09-01 (`src/corpus-run/hyper-price.ts`):
`glm-5.3` 30.4864 in and 95.8144 out;
`minimax-m3` 6.5328 and 26.1312;
`deepseek-v4-pro-0813` 28.74432 and 86.23296;
`deepseek-v4-flash-0731` 8.8 and 26.4;
`gemma-4-26b-a4b-it` 2.4 and 8.4;
`glm-5.3-flash` 3.2664 and 10.888;
`kimi-k3` 65.328 and 326.64.

## Slice-clustered reading of the 6-slice replicate

A second reviewer (sol,
reading the standings file) made two points the first rules had missed:
the slice,
not the ballot,
is the independent unit (14 editor rounds came from 4 slices),
and "the leader clears the null" says nothing about seats two and three or the boundary between three and four.
The log does not carry the authorship of losing candidates,
so per-slice ballot shares cannot be rebuilt from it;
what it does carry is every round's winner
(`[decideBestCandidate] candidate N from <author> won weight W across B ballots`,
tagged by slice),
which supports a round-win share per model and a bootstrap over whole slices.
The reader is `~/temp/agent/round-wins-bootstrap-20260901.mjs`;
a composite winner credits each author one share,
as the standing does.
Control on the 6-slice log:
it found 12 winner-bearing editor rounds of the 14 judged
(two rounds cast ballots and crowned nobody) and 4 of 4 refiner rounds.

EDITOR round wins over 12 rounds from 4 slices,
with top-three inclusion over 4000 slice resamples:
`GLM-5.3-Flash` 5 wins (92.7 percent inclusion),
`glm-5.3` 5 (82.9),
`Qwen3.8-27B` 4 (72.8),
`Kimi-K3` 3 (50.7),
`gpt-oss-120b` 1 (0.0),
`minimax-m3` 1 (0.0);
the rest won nothing.
Three of the twelve winners were composites credited to several authors.
REFINER round wins over 4 rounds:
`GLM-5.3-Flash` 2 (94.0 percent inclusion),
`deepseek-v4-pro-0813` 1 (69.7),
`minimax-m3` 1 (67.8).

The replicate also shows the editor and refiner standings reversing for four models
(`glm-5.3` and `Qwen3.8-27B` second and third as editors,
near zero as refiners;
`deepseek-v4-pro-0813` and `minimax-m3` the other way round),
which four rounds cannot establish but which is enough to say "refiners are the editors" is a transfer assumption,
not a measurement.
