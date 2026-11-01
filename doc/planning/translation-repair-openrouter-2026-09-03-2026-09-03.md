# OpenRouter and the corpus passes: 2026-09-03

What was launched,
read and decided on 2026-09-03.
Entries:
keyword233.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## What landed, 2026-09-03

Commits `0aa800ab4` (source) and `433279f3c` (tests and lint),
on top of `90dcb8745` (the `[DONE]` skip):

- `provider-name.ts`:
  `ProviderName` with `openrouter`,
  `PROVIDER_ORDER`,
  `ProviderRecord`,
  `providerRecord`,
    `otherProviders`,
    `isProviderName`.
- `openrouter-catalog.ts`,
  `openrouter-client.ts`,
  `openrouter-credits.ts`,
  `openrouter-cost.ts`:
  the chat completions client with `provider: { zdr: true, require_parameters: true, ignore: [] }` on every body,
    the credits meter,
    and the per-call USD cost read off the final chunk onto the `SPEND` line as `cost=`.
    gemma's OpenRouter row reports no pictures until a transcription is measured,
    so the reader roster is unchanged.
- `budget-routing.ts`:
  `routeProviderFor` walks `PROVIDER_ORDER` over dryness and saturation records;
    `providerServing` answers the seat reader's question;
    `EveryProviderDryError` replaces the two-provider name (old name in the local forbidden-strings appendix).
- `provider-budget.ts`,
  `budget-hold-wait.ts`:
  three meters,
  `METERS` gains `openrouter=` and `openrouterUsd=`,
    a wet refuser is held only while some other provider is wet,
    the all-dry wait is provider-generic.
- `provider-router.ts` with `provider-router-slots.ts` and `provider-router-reask.ts`:
  callers keyed by provider,
    one attempt per provider on refusals,
    the re-ask on the next wet provider serving the model,
    slots counted only where a provider states a ceiling.
- `run-seats.ts`:
  benches derive from `providerServing`;
  Hyper-slow rules apply where Hyper would serve;
    `OPENROUTER_WITHHELD` (Kimi-K3) applies where OpenRouter would;
    `OPENROUTER_CHECKER_SUBSTITUTE` (gemma) keeps the checker floor,
    and both checker assertions run per phase;
    the `JUDGE SEATS` line names every provider's state and every withheld model.
- `run-config.ts`,
  `run-client-contract.ts`,
  `required-providers.ts`,
  `budget-sample.ts`:
  the third key,
  optional and loud;
    the run client exposes `providerDryness`;
    `--require-providers` accepts `openrouter`.
- `spend-read.ts`,
  `spend-cost.ts`,
  `meter-sample-read.ts`,
  `meter-dry-span.ts`,
  `meter-report.ts`:
  the cost field and an OpenRouter USD bucket kept apart from hypercredits;
    older `METERS` lines read with the third state absent.
- `roster-blocklist.ts`:
  `google/gemini-3.8-flash` and its `:batch` spelling.

Guards shown to fail:
the OpenRouter fallthrough (routing and router tests) with `openrouter` excluded from the usable providers,
and the Kimi withholding (seat test) with the check replaced by `true`;
both restored and passing.
915 unit tests pass;
oxlint and the type check are clean.

## The first live pass with OpenRouter in the order, keyword233, 16:38 to 16:54 UTC

Launched from the main repo with the Hyper key unset so the walk went Synthetic,
then OpenRouter (`~/temp/agent/openrouter-live-20260903.log`,
artifacts beside it):

- `TALLY keyword233 status=SETTLED slices=3 ... ms=957655`,
  inside the band of the day's earlier keyword233 runs (653 to 1,164 seconds).
    `verify-published` answered 1 of 1 pages with every promised wording at the implied length.
    The page reads as the earlier runs' pages did.
- `METERS synthetic=wet hyper=dry openrouter=wet` throughout;
  `JUDGE SEATS` at every phase seated the full benches with `withheld=none`,
    since Synthetic served Kimi-K3 and Qwen3.8-27B.
- 146 OpenRouter calls at 0.38 USD by the wire's `cost=`,
  111 Synthetic calls,
  no Hyper call,
  no refusal from either;
    the meter read 56.94 before and 56.46 after,
    the probes of the same hour included.
- **MiniMax M3 came back empty on 16 of 31 OpenRouter calls**:
  `finish_reason=stop`,
  no content,
  some reasoning characters.
    The per-endpoint probe (`~/temp/agent/openrouter-minimax-endpoints-20260903`,
    corpus-sized json_schema request,
    `provider.only` per zero-data-retention endpoint) showed Parasail putting the whole JSON
    answer in the reasoning channel and closing content empty (0 of 2 conformant,
    2 rate-limited),
    ModelRun answering 4 of 4,
    and the five other zero-data-retention endpoints refusing `response_format` with `404 No
    endpoints found that can handle the requested parameters`.
    Default routing without a preference went to ModelRun on 3 of 4 and Parasail on 1;
    with `ignore: ['parasail']` it went to ModelRun on 8 of 8,
    every one conformant.
    The catalog row now carries `ignoredEndpoints: ['parasail']` and the client sends it as
    `provider.ignore` (`7d680d7fa`,
    `3991637a2`);
    both guards shown to fail with the entry removed,
    restored and passing.
    The cost fit on the run's own `SPEND` lines agrees:
    the answered calls priced as ModelRun,
    the empty ones lower.
- Cut streams on OpenRouter,
  provider of the endpoint unknown because nothing logged it:
  `deepseek-v4-pro-0813` twice at 76 and 90 seconds with reasoning only,
    `gemma-4-26b-a4b-it` twice at 66 and 77 seconds with content arriving at under twenty characters a second,
    `deepseek-v4-flash-0731` once at 209 seconds in consolidation.
    Medians on OpenRouter:
    glm-5.3 1.6 s,
    MiniMax 2.0 s,
    DeepSeek Flash 3.8 s,
    gemma 5.3 s,
    DeepSeek Pro 17.5 s;
    ninetieth percentiles 2.2,
    4.3,
    42,
    66 and 39 seconds.
    The endpoint name goes on the `SPEND` line next so the slow tail can be attributed without another probe.
- Nothing on this pass exercised the all-dry benches (Kimi-K3 withheld,
  gemma as substitute checker) or Qwen3.8-27B served by OpenRouter;
    both wait for a Synthetic-dry hour.

## The second live pass, keyword233, 17:06 to 17:28 UTC, with Parasail ignored and endpoints named

Launched as the first was (`~/temp/agent/openrouter-live2-20260903.log`,
artifacts beside it),
after `7d680d7fa` (Parasail ignored for MiniMax M3) and `c21437745` (endpoint on every `SPEND` and stream line):

- `TALLY keyword233 status=SETTLED slices=3 ... ms=1293410`,
  above the day's band (653 to 1,164 seconds).
    The lanes phase took 860 seconds against the first pass's 582;
    lane contest and consolidation each came within a minute of the first pass.
    Two things differed in that phase and this log does not separate their shares:
    a Synthetic 502/500 burst at 17:13 to 17:14 UTC (15 retry lines,
    none reaching the fifth attempt,
    none on the first pass),
    and 11 abandonments at the 60 second straggler grace against the first pass's 8.
    `verify-published` answered 1 of 1 (`wordings=3 silent=0 chars=787=expected missing=0`);
    the page reads as before.
- **The Parasail ignore held**:
  36 of 36 MiniMax calls went to ModelRun and every one completed,
  mean 2.9 seconds,
    with one schema-mismatch (an answer whose every string field was `", "`),
    against 16 empty and 6 mismatched on the first pass.
- 0.4454 USD over 135 OpenRouter calls by the wire's `cost=` (DeepSeek V4 Pro 0.29 USD and two thirds of it,
    glm-5.3 0.08,
    MiniMax 0.05,
    DeepSeek Flash 0.01,
    gemma 0.01),
    117 Synthetic calls,
    no Hyper call,
    no refusal,
    no exhausted retry ladder;
    the meter read 56.37 before and 55.91 after.
- **Per-endpoint attribution**,
  read off `served by` on the stream lines:
    - DeepSeek V4 Flash:
      OpenInference finished 2 of 6 (mean 58.8 s when it finished;
      cut at 67 to 117 s with at most one content character over 6.7k to 14.9k reasoning characters),
        Parasail 12 of 13 (mean 42.8 s),
        Inceptron 4 of 5 (mean 29.9 s),
        Makora 1 of 1,
        Together 1 of 1 (11.5 s).
        Every cut was the straggler grace ending a stream still in its reasoning channel.
        OpenInference is now in the row's `ignoredEndpoints` (`08dffd481`),
        the catalog and client guards shown failing with the entry removed,
        restored and passing.
    - gemma 4 26B:
      DeepInfra 9 of 10 (mean 20.3 s,
      the cut at 83 s),
      SiliconFlow 19 of 19 (mean 4.2 s).
        Not ignored:
        one cut in ten,
        the seat is a checker off the critical path,
        and with DeepInfra gone SiliconFlow would serve alone,
        so a rate limit there would lose the voice outright,
        since OpenRouter is the last provider in the order.
        Revisit if a later pass shows DeepInfra cutting again.
    - DeepSeek V4 Pro:
      Parasail 34 of 35 (mean 21.8 s),
      one cut at 68 s in the consolidation gate.
    - glm-5.3:
      Together 16 of 16 (mean 1.9 s);
      Modal 1 of 1 at 74 s with 27k reasoning characters and a 10 s first byte.
        One sample;
        watch it before acting.
- **Rejected:
  re-routing a voice whose transient-retry ladder is exhausted.**
    The idea was to treat five failed attempts on 5xx as a refusal and walk to the next provider.
    Measured before building:
    the four-entry Carena run of 2026-09-01 had no exhausted ladder,
    the whole archive holds one voice lost that way (`xiept2-postscript-20260903.log`,
    HTTP 503),
    and both OpenRouter passes had none.
    Not worth a code path.
- Still not exercised:
  the all-dry benches and Qwen3.8-27B served by OpenRouter;
  Synthetic stayed wet.

## The third live pass, keyword233, 18:15 to 18:36 UTC, OpenRouter alone

Launched from the worktree at tip `f26c5fb60` with the Synthetic and Hyper keys unset,
which the run reads as both dry (`~/temp/agent/openrouter-live3-20260903.log`,
artifacts beside it):

- `METERS synthetic=dry hyper=dry openrouter=wet` throughout;
    `JUDGE SEATS` at every phase read `wide=6 select=6 late=7 slate=7 checkers=3 withheld=hf:moonshotai/Kimi-K3`,
    the all-dry bench with gemma as the substitute checker.
- `TALLY keyword233 status=SETTLED slices=3 ... ms=1247533`;
  lanes 621 seconds,
  lane contest 68,
  consolidation 548.
    `verify-published` answered 1 of 1 (`chars=805=expected missing=0`);
    the page reads as before.
- 0.7590 USD floor over 235 costed OpenRouter calls (two carried no cost),
  meter 55.89 before and 55.05 after.
    DeepSeek V4 Pro 0.19 USD,
    Qwen3.8-27B 0.18,
    Kimi-K3 0.17 from six calls,
    glm-5.3 0.10,
    gpt-oss-120b 0.04,
    MiniMax 0.04,
    GLM-5.3-Flash 0.02,
    DeepSeek Flash 0.01,
    gemma 0.01.
    No refusal,
    no 5xx retry line,
    no exhausted ladder.
- **The withhold reached only the judge benches.**
  Kimi-K3 wrote six translations on OpenRouter (Fireworks 3,
    Modal 3),
    a quarter of the pass's bill,
    while every bench had it out:
    `judgeSeatsFor` filtered the wide,
    late,
    select and checker seats by `seated` and left `translatorModelIds` as the static `RUN_TRANSLATORS`,
    and `pass-entry.ts` passed the catalog's `RUN_READER_MODELS` to the picture stage unfiltered.
    Fixed in `8848f070e`:
    `JudgeSeats` carries `translators` and `readers` filtered the same way,
    the translate lane takes its writers from there,
    and the picture stage reads its own seats (`JUDGE SEATS phase=pictures`,
    `pass-seated-pictures.ts`);
    the seat guards shown failing with the filters removed,
    restored and passing,
    and the entry driver test now counts four meter readings per entry.
    The fourth pass (19:33 UTC,
    launched on that fix) then bought Kimi-K3's first call from the block-pairing
    round six seconds before any bench was read:
    preparation,
    insertion admission and the consolidation writers all took the static `RUN_ROSTER`.
    `68ad11530` adds `roster` to `JudgeSeats`,
    the whole roster less any withheld model,
    read for `phase=preparation` before the pairing round,
    and the other two stages take their own reading's roster;
    guard shown failing with the filter removed,
    restored and passing.
    Not yet exercised live past the pairing round;
    the next OpenRouter-only pass must show no `SPEND provider=openrouter model=moonshotai/kimi-k3` line.
- **The Parasail ignore and the OpenInference ignore both held**:
  36 of 36 MiniMax calls to ModelRun,
  all completed,
    three schema-mismatches (two consolidation gates,
    one translate vote);
    no DeepSeek Flash stream on OpenInference.
- **Qwen3.8-27B served by OpenRouter conformed**:
  31 of 31 completed answers usable,
  all on Parasail,
    completed p50 18.3 s,
    p90 52.8 s,
    max 147 s,
    against p50 16.5 s,
    p90 48.8 s,
    max 228 s on Synthetic on the second pass.
    The endpoint is not the slow part;
    the model reasons long on either provider.
- **Every cut was a reasoning-only stream on Parasail ending at the 60 second straggler grace**,
  14 of them:
    Qwen 7 of 38 asks,
    DeepSeek Flash 6 of 27 (Parasail 17 of 23 completed,
    p50 16.4 s,
    p90 49.5 s;
    Makora 4 of 4 at 6.9 s),
    DeepSeek Pro 1 of 37 (Parasail 20 of 21 at 21.3 s;
    Sail Research 16 of 16 at 1.5 s).
    Cut at 67 to 188 seconds with no content character and 15k to 43k reasoning characters each,
    across critic,
    panel,
    select,
    lane-contest and consolidation-gate rounds.
    `run-timing-report`:
    47 rounds,
    28.7 of 34.7 round-minutes waiting after quorum (82.7 percent),
    19 voices never heard,
    against 43 rounds,
    62.3 percent and 12 on the second pass.
    The all-OpenRouter bench reaches quorum sooner (gpt-oss on Cerebras and Groq at 1 to 2 seconds,
    MiniMax at 2.6,
    gemma at 4 to 8) and the reasoning seats then have less absolute time before the grace ends.
    No ignore fits this:
    Parasail's completed latencies match Synthetic's for the same model,
    and Qwen has no other endpoint on this run.
    The lever is the grace,
    `TRANSLATION_REPAIR_STRAGGLER_GRACE_MS` (default 180 s in `stage-round.ts`,
    set to 60 s on every keyword233 pass of this day),
    and that is a speed-against-width tradeoff put to the owner.
- Other endpoints seen,
  for the record:
  gemma on DeepInfra 30 of 30 at 8.4 s (the ignore stays rejected),
    gpt-oss on Nebius 14 at 6.6 s,
    glm-5.3 on Modal 3 at 40.5 s against Together 15 at 5.2 s,
    GLM-5.3-Flash on Together 8 at 65.8 s and Makora 1 at 70 s against Venice 2 at 5.2 s and Reka 1 at 1.8 s.
    GLM-5.3-Flash's slow endpoints cut nothing (14 of 14 usable),
    so nothing is ignored on one pass's counts.

## The fourth live pass, keyword233, 19:33 to 19:55 UTC, OpenRouter alone at a 120 s grace

Launched from the worktree at tip `f4d59bf65` (the translator and reader withhold,
before the roster-wide one) with `TRANSLATION_REPAIR_STRAGGLER_GRACE_MS=120000`,
everything else as the third pass (`~/temp/agent/openrouter-live4-20260903.log`):

- `TALLY keyword233 status=SETTLED slices=3 ... ms=1321136`;
  `verify-published` 1 of 1 (`chars=773=expected missing=0`);
    no refusal,
    no 5xx retry,
    one schema-mismatch.
- Cut streams 7 (Qwen3.8-27B 6,
  gemma 1) against 14 at 60 s;
  `run-timing-report` 37 rounds,
    34.6 of 37.7 round-minutes waiting after quorum (91.7 percent),
    8 voices never heard against 19.
- 0.4502 USD;
  Kimi-K3 bought 3 calls (0.07 USD):
  one from the block-pairing round and two as consolidation writers,
    the roster-wide gap `68ad11530` closes.
    `JUDGE SEATS` read `translators=6 readers=3` at every phase.
- Put to the owner with the third pass's figures;
  **decided:
  120 s**,
  now the built-in (`doc/decision/translation-repair-straggler-grace.md`,
    "Decision 2026-09-03").
- Decided at the same asking:
  the recovery round keeps its complaint-appended re-ask
    (`doc/decision/translation-repair-recovery-reask.md`) and per-slice semantic wrap stays
    (`doc/decision/translation-repair-page-shape-per-slice-wrap.md`).

## The fifth live pass, keyword233, 19:58 to 20:21 UTC: the withhold holds through every stage

Launched from the worktree at tip `47a292e2a` (the roster-wide withhold) at the 120 s dial,
OpenRouter alone (`~/temp/agent/openrouter-live5-20260903.log`):

- **No Kimi-K3 call at all**:
  no `SPEND` line and no stream for `moonshotai/kimi-k3` from preparation to consolidation;
    `JUDGE SEATS` read `roster=8 translators=6 readers=3 withheld=hf:moonshotai/Kimi-K3` at all five readings,
    `phase=preparation` first.
    The owner's withhold is now verified at the user boundary.
- `TALLY keyword233 status=SETTLED slices=3 ... ms=1358549`;
  `verify-published` 1 of 1 (`chars=796=expected missing=0`);
    no refusal,
    one 5xx retried and recovered,
    two schema-mismatches.
- 10 cut streams (DeepSeek Flash 5,
  Qwen 3,
  GLM-5.3-Flash 2),
  12 voices never heard,
  40 rounds with 92.5 percent of round time waiting after quorum;
    0.4999 USD;
    meter 54.45 before,
    53.89 after.
    Against the fourth pass's 7 cuts and 8 never heard at the same window:
    single runs on this entry spread that wide,
    and neither pair is a window effect on its own.
    DeepSeek Flash this time went mostly to Phala (16 of 18 completed) with Parasail 2 of 5;
    the routing moves between passes,
    which is one more reason single-pass endpoint counts do not earn an ignore.
- The sixth pass launched at 20:21 UTC on `e0509047b` with no straggler dial,
  to run the built-in 120 s window
    and carry the first `recovery round heard N of M` count (`~/temp/agent/openrouter-live6-20260903.log`).

## The sixth live pass, keyword233, 20:21 to 20:43 UTC: the built-in window, no dial

- No `STRAGGLER GRACE OVERRIDDEN` line;
  every reader-round abandonment reads `abandoned 120000ms after quorum`
    (six of them) and the one writer-round abandonment `180000ms`,
    the writer dial still set at launch.
    The built-in 120 s is what the pass runs.
- `TALLY keyword233 status=SETTLED slices=3 ... ms=1312386`;
  `verify-published` 1 of 1 (`chars=806=expected missing=0`);
    no Kimi-K3 call,
    no refusal,
    no schema-mismatch,
    7 cut streams,
    7 voices never heard,
    37 rounds with 94.3 percent of round time waiting after quorum;
    0.4281 USD;
    meter 53.89 before,
    53.41 after.
- No recovery round ran,
  since no answer came back unreadable,
  so the `recovery round heard N of M` line has no first count yet;
    the re-ask's yield is read off the earlier passes instead (next bullet).
- **The complaint-appended re-ask recovers about half**,
  read off today's first,
  third and fourth passes by
    pairing each `recovery round for N unreadable` line with its stage's next `round: x/N heard` line:
    first pass 2 of 8 parsed rounds heard (the misses were MiniMax on Parasail answering unreadably again,
    before the ignore),
    third pass 3 of 3,
    one round on each of the first and fourth passes with an unparsed stage label.
    5 of 11 in all,
    and no `PROMPT-REUSE source=memory` after any of them:
    the nudge makes the digest new every time.
    Issue 473 closed on this measurement.
