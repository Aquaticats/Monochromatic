import { CLAUDE_CARDS, } from './claude-card-sides.ts';
import {
  DEEPSEEK_V4_1_FLASH_ON_DEEPSEEK,
  MIMO_V2_6_FLASH_ON_XIAOMI,
} from './direct-card-sides.ts';
import type { ModelCard, } from './model-card.ts';
import {
  DEEPSEEK_V4_1_FLASH_ON_NEURALWATT,
  GLM_5_3_FLASH_ON_NEURALWATT,
} from './neuralwatt-card-sides.ts';
import {
  DEEPSEEK_V4_1_FLASH_ON_OPENROUTER,
  GLM_5_3_FLASH_ON_OPENROUTER,
  MINIMAX_M3_ON_OPENROUTER,
} from './openrouter-card-sides.ts';
import type { RosterModelId, } from './roster-id.ts';

//region Model cards
// THE ROSTER, ONE CARD EACH. Ordered by introduction: the Synthetic four,
// the Hyper-origin seats, then the Bedrock-only and OpenRouter-only
// arrivals; `ROSTER_MODEL_IDS` keeps this order. Evidence for each field
// and each hold lives beside the table that reads it (`hyper-catalog.ts`,
// `openrouter-catalog.ts`, `bedrock-catalog.ts`, `completion-cap.ts`,
// `openrouter-abandoned-spend.ts`, `run-config.ts`, `run-seats.ts`) and in
// `doc/decision/translation-repair-roster-seating-2026-09-01.md`. The
// NeuralWatt sides and the OpenRouter sides that name their endpoints sit
// in `neuralwatt-card-sides.ts` and `openrouter-card-sides.ts`, and the two
// direct sides (DeepSeek's and Xiaomi's own APIs, each serving one model) in
// `direct-card-sides.ts`, and the eight Claude cards in `claude-card-sides.ts`, with their evidence.
//
// TO ADD A MODEL: run `mise run roster-card -- <provider> <served id>` for a
// card with the live listing's fields, paste it here with `completionCap:
// 'pooled-p99'` and the holds the seating rule requires (`judge-unmeasured`
// for a single-provider candidate, `writer-unmeasured` and
// `reader-unmeasured` for every candidate), then run the package checks.
// TO REMOVE ONE: delete its card and add its spellings to
// `roster-blocklist.ts` with the owner's words.

/**
 Every model this pipeline may seat, keyed by roster id, so a roster id
 without a card and a card without a roster id are both type errors.

 @example
 ```ts
 const card = MODEL_CARDS['minimax-m3'];
 ```
 */
export const MODEL_CARDS: Readonly<Record<RosterModelId, ModelCard>> = {
  'hf:zai-org/GLM-5.3-Flash': {
    synthetic: {
      id: 'hf:zai-org/GLM-5.3-Flash',
      family: 'zai',
      readsImages: true,
      contextLength: 524_288,
      maxOutputLength: 65_536,
      promptDollarsPerToken: 0.00000015,
      completionDollarsPerToken: 0.0000005,
    },
    hyper: {
      id: 'glm-5.3-flash',
      readsImages: true,
      maxOutputLength: 131_072,
    },
    openrouter: GLM_5_3_FLASH_ON_OPENROUTER,
    // NEURALWATT, 2026-10-20: the standard lane of `glm-5.3-flash`, which the
    // owner's free daily energy covers (`neuralwatt-card-sides.ts`). No hold
    // changes with this side; the holds on this card were all recorded before
    // NeuralWatt joined and stay as they were.
    neuralwatt: GLM_5_3_FLASH_ON_NEURALWATT,
    // Hyper p99 over 886 calls; Synthetic 16,342 over 4,775; OpenRouter
    // 13,070 over 1,853 (the 2026-09-09 table).
    completionCap: 18_316,
    // Owner, 2026-09-02: "Unseat GLM-5.3-Flash as a judge, keep it as editor".
    // Off Synthetic since 2026-09-24 (class one hundred eighteen): over the
    // runs from 2026-09-21 it averaged 74.5 s a stream there (892 streams,
    // p50 51.9 s, 43 with no content) against 8.6 s on Hyper (305, p50 6.5 s,
    // none empty), and on CuspariaKLSY13 one editor round waited 175 s in
    // grace for its 230 s cap-cut runaway and the recovery round 180 s more
    // for nothing, with no other chunk in flight.
    holds: [
      'wide-seat-dropped',
      'late-judge-dropped',
      'synthetic-withheld',
    ],
  },
  'hf:Qwen/Qwen3.8-27B': {
    synthetic: {
      id: 'hf:Qwen/Qwen3.8-27B',
      family: 'qwen',
      readsImages: true,
      contextLength: 262_144,
      maxOutputLength: 65_536,
      promptDollarsPerToken: 0.00000045,
      completionDollarsPerToken: 0.0000022,
    },
    hyper: {
      id: 'qwen3.8-27b',
      readsImages: true,
      maxOutputLength: 128_000,
    },
    // Synthetic p99 over 7,312 calls; OpenRouter 11,127 over 4,538; Hyper
    // 10,541 over 1,921 (the 2026-09-09 table).
    completionCap: 20_894,
    // Off the OpenRouter catalog since 2026-09-09: 24 percent of its calls
    // there were abandoned and billed to the end.
    //
    // Off the late-judge bench (lane contest, consolidation slate, consolidation
    // gate) by the owner's answer of 2026-10-22 (UTC), "Remove from both
    // benches": in XingZ6017, once Synthetic served it again, 40 of its 95
    // streams were cut after the grace with no answer and the mean
    // consolidation slice rose from 149 s to 315 s and 346 s (ledger B810, B815).
    // The wide seats keep it while Synthetic serves it (`run-seats.ts`).
    holds: [
      'openrouter-dropped',
      'late-judge-dropped',
    ],
  },
  'hf:moonshotai/Kimi-K3': {
    synthetic: {
      id: 'hf:moonshotai/Kimi-K3',
      family: 'moonshot',
      readsImages: true,
      contextLength: 524_288,
      maxOutputLength: 65_536,
      promptDollarsPerToken: 0.000003,
      completionDollarsPerToken: 0.000015,
    },
    hyper: {
      id: 'kimi-k3',
      readsImages: true,
      maxOutputLength: 16_000,
    },
    openrouter: {
      id: 'moonshotai/kimi-k3',
      readsImages: true,
      maxOutputLength: 943_718,
      promptUsdPerMillion: 3,
      completionUsdPerMillion: 15,
      ignoredEndpoints: [],
      preferredEndpoints: [],
      // Raw characters per completion token over pass-run logs, median of 443 run
      // streams, none since 2026-09-21 (ledger P7, 2026-09-28).
      rawCharsPerToken: 140,
    },
    // Hyper p99 over 2,777 calls; OpenRouter 8,254 over 488; Synthetic
    // 4,350 over 7,051 (the 2026-09-09 table).
    completionCap: 10_921,
    // Withheld from OpenRouter on cost (owner, 2026-09-03: 3 and 15 USD per
    // million); too slow in the select seats when Hyper serves it
    // (2026-09-03, `run-seats.ts`).
    holds: [
      'openrouter-withheld',
      'hyper-slow-select',
    ],
  },
  'hf:openai/gpt-oss-120b': {
    synthetic: {
      id: 'hf:openai/gpt-oss-120b',
      family: 'openai',
      readsImages: false,
      contextLength: 131_072,
      maxOutputLength: 65_536,
      promptDollarsPerToken: 0.0000001,
      completionDollarsPerToken: 0.0000001,
    },
    hyper: {
      id: 'gpt-oss-120b',
      readsImages: false,
      maxOutputLength: 13_107,
    },
    openrouter: {
      id: 'openai/gpt-oss-120b',
      readsImages: false,
      maxOutputLength: 117_964,
      promptUsdPerMillion: 0.037,
      completionUsdPerMillion: 0.17,
      ignoredEndpoints: [],
      preferredEndpoints: [],
      // Raw characters per completion token over pass-run logs, median of 1,326 streams,
      // none since 2026-09-21 (ledger P7, 2026-09-28).
      rawCharsPerToken: 286,
    },
    bedrock: {
      id: 'openai.gpt-oss-120b',
      readsImages: false,
      contextLength: 131_072,
      maxOutputLength: 16_384,
      route: 'v1',
      streamEnd: 'usage-chunk',
      // Culled from every role on 2026-09-24; never measured for a bound.
      streamBoundMs: 'unbounded',
      promptUsdPerMillion: 0.1545,
      completionUsdPerMillion: 0.618,
    },
    // Own p99 at most 3,649 (Hyper, 2,479 calls) over 21,111 calls on four
    // providers, under the pooled 90th (the 2026-09-09 table).
    completionCap: 'pooled-p90',
    // Producer calibration of 2026-09-01: 5 of 207 disinterested ballots,
    // z -4.53 against the pooled null.
    //
    // CULLED FROM EVERY ROLE ON 2026-09-24 by the owner ("That particular
    // model got cause and effect wrong. Cull it." and then "cull it from every
    // role"): on CuspariaKLSY10 slice 1 it filed, alone, the accepted
    // omission claim reading 服用激素，成为跨性别的原因 as hormones being the
    // reason for transitioning, and on zheermao8 slice 9 it declined every
    // slate candidate over "quoted lines in present tense" on a slice with no
    // quoted line. The card stays: it is the one four-provider identity the
    // unit fixture seats as SEAT_SYNTHETIC_TEXT_EVERYWHERE, and the catalogs
    // still type it; `owner-culled` empties every bench.
    holds: [
      'translator-dropped',
      'owner-culled',
    ],
  },
  'minimax-m3': {
    hyper: {
      id: 'minimax-m3',
      readsImages: true,
      maxOutputLength: 512_000,
    },
    openrouter: MINIMAX_M3_ON_OPENROUTER,
    // Hyper p99 over 27,361 calls; OpenRouter 718 over 6,657 (the
    // 2026-09-09 table).
    completionCap: 10_822,
    holds: [],
  },
  'gemma-4-26b-a4b-it': {
    hyper: {
      id: 'gemma-4-26b-a4b-it',
      readsImages: false,
      maxOutputLength: 25_600,
    },
    openrouter: {
      id: 'google/gemma-4-26b-a4b-it',
      // THE LISTING'S TRUTH SINCE 2026-10-21: `input_modalities` image, text
      // and video, with 13 endpoints that keep nothing (`GET
      // /api/v1/endpoints/zdr`). Until then this said false "until a
      // transcription is measured" (`doc/decision/translation-repair-openrouter-fallback.md`),
      // and the 2026-09-08 measurement was through Bedrock alone. The route
      // was held shut by `openrouter-pictures-unmeasured` until `reader-measure`
      // met both clauses of the reader rule on this route (2026-10-21).
      readsImages: true,
      maxOutputLength: 16_384,
      promptUsdPerMillion: 0.07,
      completionUsdPerMillion: 0.34,
      ignoredEndpoints: [],
      preferredEndpoints: [],
      // Raw characters per completion token over pass-run logs, median of 1,797 streams;
      // the 40 since 2026-09-21 read 289 (ledger P7, 2026-09-28).
      rawCharsPerToken: 292,
    },
    bedrock: {
      id: 'google.gemma-4-26b-a4b',
      readsImages: true,
      contextLength: 262_144,
      maxOutputLength: 262_144,
      route: 'openai-v1',
      streamEnd: 'done-sentinel',
      // CLASS ONE HUNDRED FORTY-EIGHT (hulicaijia29, 2026-09-26): over the
      // 200 newest pass logs no Bedrock Gemma completion ran past 27,340 ms
      // of 68,235 unless Bedrock had queued it (a frame every 15 s, content
      // trickling), and queued calls ran 25 to 304 s. A call past this bound
      // holds Bedrock out for the model and goes to its next provider.
      streamBoundMs: 60_000,
      promptUsdPerMillion: 0.13,
      completionUsdPerMillion: 0.4,
    },
    // Own p99 at most 483 over 16,251 calls on three providers, under the
    // pooled 90th (the 2026-09-09 table).
    completionCap: 'pooled-p90',
    // PICTURES ON OpenRouter MEASURED, 2026-10-21 (XingZ6016): Bedrock was the
    // only route this seat's transcription had been measured on (nine pictures,
    // mean overlap 0.668, 2026-09-08) and its key is never renewed, which left
    // two live picture readers. The owner: "Yes try gemma". `reader-measure`
    // then met both clauses of the reader rule on OpenRouter (nine pictures
    // from two entries, all nine readings corroborated by every seated reading
    // of their picture, none by none; `doc/roster-changes.md`), so
    // the hold `openrouter-pictures-unmeasured` came off.
    holds: [],
  },
  'glm-5.3': {
    hyper: {
      id: 'glm-5.3',
      readsImages: false,
      maxOutputLength: 262_144,
    },
    // OpenRouter p99 over 2,673 calls; Hyper 17,118 over 3,343 (the
    // 2026-09-09 table).
    completionCap: 22_067,
    // The roster's slowest voice in every measured role (2026-09-01); off
    // the OpenRouter catalog since 2026-09-09.
    holds: [
      'wide-seat-dropped',
      'openrouter-dropped',
    ],
  },
  // THE 1017 WEIGHTS SINCE 2026-10-20, UNDER THE SAME CARD NAME. The owner:
  // "Migrate to deepseek v4 flash 1017 where possible." The model is
  // DeepSeek V4.1 Flash 1017 (`deepseek-ai/DeepSeek-V4.1-Flash-1017`).
  // Charm Hyper lists it as `deepseek-v4.1-flash-1017` and OpenRouter as
  // `deepseek/deepseek-v4.1-flash-1017` (created 2026-10-18), beside the
  // older ids, which both listings still carry; Hyper's listing does not
  // name the older id's weights. The Hyper and OpenRouter sides of this card
  // send the 1017 ids. The roster id stays `deepseek-v4.1-flash`, so caches,
  // ledgers, calibrations and logs keep one name for the seat; Hyper's
  // catalog row ties the two (`HyperModelInfo.rosterId`). NeuralWatt's
  // `deepseek-v4.1-flash` needed no change: its listing already names the
  // 1017 weights. Synthetic lists `hf:deepseek-ai/DeepSeek-V4.1-Flash-1017`
  // and this card has no Synthetic side; seating one would be a new route
  // and not a migration, so none was added.
  //
  // DEEPSEEK'S OWN API IS A ROUTE OF THIS CARD SINCE 2026-10-22, as
  // `deepseek-flash`, which DeepSeek's documentation says is served by
  // DeepSeek-V4.1-Flash-1017: the 1017 claim rests on that documentation,
  // since no dated id exists to send. The owner's grant limits that key to
  // this model, and the route carries no retention promise (the owner's PII
  // ruling of 2026-10-05 permits it).
  //
  // THE MEASURED VALUES ON THIS CARD WERE MEASURED ON THE PRE-1017 IDS and
  // carry over as an assumption until the first runs on 1017 measure them
  // again: the completion cap, Hyper's `maxOutputLength`, `rawCharsPerToken`,
  // the holds and the judge and writer calibrations. Hyper's listing for the
  // new id states the same context window, output ceiling, vision flag and
  // price as the old id (read 2026-10-20); OpenRouter's Morph price differs
  // (see the price comment on `DEEPSEEK_V4_1_FLASH_ON_OPENROUTER`).
  'deepseek-v4.1-flash': {
    hyper: {
      id: 'deepseek-v4.1-flash-1017',
      readsImages: true,
      maxOutputLength: 26_214,
    },
    openrouter: DEEPSEEK_V4_1_FLASH_ON_OPENROUTER,
    // NEURALWATT, 2026-10-20: the standard lane of `deepseek-v4.1-flash`,
    // which the owner's free daily energy covers (`neuralwatt-card-sides.ts`).
    // Its listing names the 1017 weights (`huggingface_id` of
    // `deepseek-ai/DeepSeek-V4.1-Flash-1017`), so this id is already the
    // migrated one.
    // The hold on this card was recorded before NeuralWatt joined and stays: no
    // transcription of a picture has been measured on any stack of this model.
    neuralwatt: DEEPSEEK_V4_1_FLASH_ON_NEURALWATT,
    // DEEPSEEK'S OWN API, 2026-10-22: `deepseek-flash`, the one model the
    // owner's DeepSeek key is granted for ("you may only use deepseek v4.1
    // flash 1017 with that key"; `direct-card-sides.ts`). No hold changes.
    deepseek: DEEPSEEK_V4_1_FLASH_ON_DEEPSEEK,
    // HYPER'S 99TH PERCENTILE IS THE CAP ITSELF (ledger P10, 2026-09-28).
    // The card said "no completed-call distribution of its own yet" long
    // after it had one. Over pass-run logs since the caps went on the wire,
    // 75 of 4,062 Hyper calls ran to 13,082 and 74 of them streamed no
    // content, a reasoning runaway the cap exists to stop; OpenRouter's p99
    // is 7,531 over 63,291, and of its 237 calls at the cap the 70 that pair
    // with a stream line are 69 runaways and one answer. A capped call cannot
    // show a longer one, so the rule's highest p99 reads as "at least one
    // percent ran into it", not as a length the model needs; the number is
    // the pooled 99th it rode before, now its own.
    completionCap: 13_082,
    // Judge seats on the fidelity probe of 2026-09-11. Writes since the
    // 40-round producer calibration of 2026-09-19 (29 of 121 disinterested
    // ballots over 27 candidates, z +0.79 against a 21.0 percent pooled
    // null, not separated from it; 55 of 55 asks usable; the most round wins
    // of the bench at 8.3 of 35). No transcription measurement yet.
    holds: ['reader-unmeasured',],
  },
  'google.gemma-4-e2b': {
    bedrock: {
      id: 'google.gemma-4-e2b',
      readsImages: false,
      contextLength: 131_072,
      maxOutputLength: 131_072,
      route: 'openai-v1',
      streamEnd: 'done-sentinel',
      // CLASS ONE HUNDRED FORTY-EIGHT (hulicaijia29, 2026-09-26): over the
      // 200 newest pass logs no Bedrock Gemma completion ran past 27,340 ms
      // of 68,235 unless Bedrock had queued it (a frame every 15 s, content
      // trickling), and queued calls ran 25 to 304 s. A call past this bound
      // holds Bedrock out for the model and goes to its next provider.
      streamBoundMs: 60_000,
      promptUsdPerMillion: 0.04,
      completionUsdPerMillion: 0.08,
    },
    // Own p99 483 over 5,506 Bedrock calls, under the pooled 90th (the
    // 2026-09-09 table).
    completionCap: 'pooled-p90',
    // Judge seats on the fidelity probe of 2026-09-07, under the rule
    // pre-registered in the planning log before the seated roster was
    // measured: a candidate joins critic, panel and judge when, over the same
    // distinct questions, it chooses the complete text at least as often as
    // the median seated judge and the damaged text no more often than the
    // seated judge who chooses it most. It chose the complete text on 11 of
    // 12 with one damaged pick, against a seated median of 9.5 and a worst
    // damaged count of 1 (Kimi-K3 12, deepseek-v4-pro-0813 11, minimax-m3 11,
    // deepseek-v4-flash-0731 8, gpt-oss-120b 7, gemma-4-26b-a4b-it 4).
    // Bedrock serves it alone, so it was the third judge that answered while
    // Hyper was held out by its daily limit. (This record sat on
    // `SEATED_BEDROCK_JUDGES` in `run-config.ts` until 2026-09-29, a set only
    // the seat tests read; ledger B31.)
    holds: [],
  },
  'google.gemma-4-31b': {
    bedrock: {
      id: 'google.gemma-4-31b',
      readsImages: true,
      contextLength: 262_144,
      maxOutputLength: 262_144,
      route: 'openai-v1',
      streamEnd: 'done-sentinel',
      // No queued-call measurement on this card: 125 calls, none queued.
      streamBoundMs: 'unbounded',
      promptUsdPerMillion: 0.14,
      completionUsdPerMillion: 0.4,
    },
    // Bedrock p99 over 125 calls, the thinnest measurement in the 2026-09-09
    // table.
    completionCap: 8_194,
    // Stayed out of the judge seats on the 2026-09-07 fidelity probe; reads
    // pictures since the 2026-09-08 transcription measurement. Checks since
    // the checker benchmark of 2026-09-27 (76 of 85 unchanged texts called
    // not fixed, 82 of 85 fixes called fixed, sharing the fewest errors
    // with the other checkers); the checker seat is not a judge seat, so
    // the hold stands for the fidelity probe it names.
    holds: ['judge-unmeasured',],
  },
  'inception/mercury-2.5': {
    openrouter: {
      id: 'inception/mercury-2.5',
      readsImages: false,
      maxOutputLength: 65_536,
      promptUsdPerMillion: 0.04,
      completionUsdPerMillion: 0.15,
      ignoredEndpoints: [],
      preferredEndpoints: [],
      // Raw characters per completion token over pass-run logs, median of 37,120 streams
      // since 2026-09-21: it bills far more tokens than it streams (ledger P7,
      // 2026-09-28).
      rawCharsPerToken: 0.9,
      // FIRST TEXT WITHIN 90 SECONDS (ledger B809, read on 2026-10-22 UTC).
      // In XingZ6017, twenty-six rounds waited 250 to 360 s on a stream of
      // this model (two rounds on two in a row) that sent only `: OPENROUTER PROCESSING` comment lines
      // (about 727 of them, first byte under a second, a gap never past a
      // second) until the gateway ended it with HTTP 502 at about 301 s; the
      // retry then answered in 5 to 9 s. Over the 24 pass logs on disk
      // (CuspariaKLSY16 to XingZ6017) 4,636 streams of this model delivered
      // text, p50 6.7 s, p99 20.3 s, the longest 56,772 ms from arming to the
      // end of the stream, so the first text came no later than that; and of
      // the 130 streams that ran a minute or more with no text, none
      // delivered any. 90,000 ms is 59 percent over the longest, the margin
      // B804 gave the shared window over its own longest observation (56
      // percent). The shared window is 180,000 ms. Open to the owner's veto.
      firstContentMs: 90_000,
    },
    // Own p99 3,063 over 136 OpenRouter calls, under the pooled 90th (read
    // at 20:05 UTC on 2026-09-09).
    completionCap: 'pooled-p90',
    // Judge seats on the fidelity probe under the same rule as
    // `google.gemma-4-e2b`, measured at 17:00 UTC on 2026-09-09 over the
    // three settled artifacts on disk (gqt, hakureico, noname; fourteen
    // distinct questions, twelve deletions and insertions and two
    // alterations): 14 of 14 chose the complete text, no damaged pick, no
    // decline, position two on 7 of 14, every one of its 14 asks usable. The
    // seated roster's reading of the same fourteen is in the planning log of
    // 2026-09-09 ("The owner asks where 200 USD went"); a candidate at the
    // maximum meets both clauses of the rule whatever the seated median
    // reads. From that reading the cheapest seat on the per-token provider
    // (0.04 and 0.15 USD per million) sat wherever the anchor judge did.
    // (This record sat on `SEATED_OPENROUTER_JUDGES` in
    // `run-config.ts` until 2026-09-29, a set only the seat tests read;
    // ledger B31.)
    // Seated as a writer by the calibration of 2026-09-09 (z -0.43); the
    // calibration of 2026-09-19 read it below the pooled null (10 of 108
    // disinterested ballots over 28 candidates, adjusted 9.3 percent,
    // z -3.00 against 21.0, across the Bonferroni threshold of 2.77), so
    // by the seating rule it leaves the translator seat and keeps the
    // consolidation seat, the shape of deepseek-v4-pro-0813's exit.
    holds: ['translator-dropped',],
  },
  'xiaomi/mimo-v2.6-flash': {
    // Read from https://openrouter.ai/api/v1/models on 2026-09-27
    // (`mise run roster-card -- openrouter xiaomi/mimo-v2.6-flash`).
    openrouter: {
      id: 'xiaomi/mimo-v2.6-flash',
      readsImages: true,
      maxOutputLength: 131_072,
      promptUsdPerMillion: 0.14,
      completionUsdPerMillion: 0.28,
      // DEEPINFRA IS THE ONLY ENDPOINT THIS ROUTE CAN REACH, so no preference
      // is named. Of the two endpoints listed on 2026-09-27 only DeepInfra is
      // on OpenRouter's zero-data-retention list (`/api/v1/endpoints/zdr`), and
      // every request here carries `zdr: true`; Xiaomi's own endpoint is not,
      // so `order: ['xiaomi']` changed nothing (17 of 17 calls still
      // DeepInfra). Through the run client the 170 checker sheets answered at
      // 3,859 ms median and 22,406 ms at the ninetieth percentile, one stream
      // cut short; by raw fetch without `zdr` they answered at 349 and 809 ms.
      // SINCE 2026-10-22 THE PIPELINE ALSO REACHES XIAOMI'S OWN ENDPOINT
      // DIRECTLY, with the owner's key and under the `xiaomi` side of this
      // card: no retention promise, permitted by the owner's PII ruling of
      // 2026-10-05 and by the grant ("You may only use v2.6-flash with this
      // key"). That route is ahead of this one in the provider order.
      ignoredEndpoints: [],
      preferredEndpoints: [],
      // Raw characters per completion token over pass-run logs, median of 78 run
      // streams, all since 2026-09-21 (ledger P7, 2026-09-28).
      rawCharsPerToken: 93,
    },
    // XIAOMI'S OWN API, 2026-10-22: `mimo-v2.6-flash`, the one model the
    // owner's Xiaomi key is granted for (`direct-card-sides.ts`). No hold
    // changes.
    xiaomi: MIMO_V2_6_FLASH_ON_XIAOMI,
    // 78 completed calls on OpenRouter, p99 1,719, under the 100 calls the
    // rule reads (`completion-cap.ts`; ledger P10, 2026-09-28).
    completionCap: 'pooled-p99',
    // Owner, 2026-09-27: approved on OpenRouter with Mimo v2.6 Pro. Checks
    // since the checker benchmark that day: through the run client 81 of 85
    // unchanged texts called not fixed and 80 of 85 fixes called fixed (82
    // and 79 by raw fetch). Pro read 81 and 76 by raw fetch at ten times
    // Flash's latency there, sharing 8 of Flash's 9 errors, so Pro holds no
    // card. Every other role waits for its own measurement.
    holds: [
      'judge-unmeasured',
      'writer-unmeasured',
      'reader-unmeasured',
    ],
  },
  'typesafe/jev-1.13': {
    decisions: {
      id: 'typesafe/jev-1.13',
      contextLength: 32_000,
      promptUsdPerMillion: 0.042,
      completionUsdPerMillion: 0,
    },
    // Answers are typed decisions, never a completion; the cap table needs
    // a row for every roster id and this one is never sent.
    completionCap: 'pooled-p90',
    // Owner, 2026-09-18: approved on OpenRouter. 28 of 28 on the reviewed
    // fidelity matrix by a scratch measurement that night, then 12 of 12
    // through the production probe's decisions route beside the seated
    // roster at 01:40 UTC (the seating decision's 2026-09-18 addendum):
    // seated as a select judge. Writing and reading are never its roles;
    // the holds stay because a typed decision writes and reads nothing.
    holds: [
      'writer-unmeasured',
      'reader-unmeasured',
    ],
  },
  // THE EIGHT CARDS OF THE OWNER'S CLAUDE KEY, 2026-10-22 (`claude-card-sides.ts`):
  // Sonnet 5.5 and Haiku 5.5, each with no effort and at three efforts.
  ...CLAUDE_CARDS,
};

//endregion Model cards
