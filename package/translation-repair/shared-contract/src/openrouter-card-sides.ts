import type { OpenRouterCard, } from './model-card.ts';

//region OpenRouter card sides
// THE OPENROUTER SIDES THAT NAME THEIR ENDPOINTS, kept beside `model-cards.ts`
// rather than in it because that table sits at the package's line budget
// (moved 2026-10-22, when the owner's answer taking Qwen3.8-27B off the
// late-judge bench pushed it over). Each side here carries the measurements
// that chose which upstream endpoints the seat names or ignores; the rest of
// each card, its completion cap and holds included, stays in the table.

/**
 OpenRouter's side of the GLM-5.3-Flash card, routed to Wafer.

 @example
 ```ts
 const side = GLM_5_3_FLASH_ON_OPENROUTER;
 ```
 */
export const GLM_5_3_FLASH_ON_OPENROUTER: OpenRouterCard = {
  id: 'z-ai/glm-5.3-flash',
  readsImages: true,
  maxOutputLength: 131_072,
  // WAFER'S PRICE, THE ENDPOINT THIS SEAT BUYS (ledger P13, 2026-09-28).
  // The card carried DeepInfra's, the cheapest listing when it was
  // written, 0.075 and 0.25, while all 253 calls on TianqiChen66616, 66619
  // and 66620 went to Wafer, listed that day at 0.9 and 0.5; fitted from
  // what those calls were charged, 0.43 a million prompt tokens and 0.5 a
  // million completion tokens. Only the abandoned-spend estimate reads
  // these.
  promptUsdPerMillion: 0.9,
  completionUsdPerMillion: 0.5,
  ignoredEndpoints: [],
  // WAFER AHEAD OF THE PRICE SORT (class ninety-three, XingZ626,
  // 2026-09-23). The price sort lands this seat on endpoints that reason
  // at length by default: Together served 187 streams at 91 s and 23,666
  // characters a reply, Parasail 38 at 99 s and 22,050, InferenceNet 52
  // at 114 s and 14,247, while Wafer served 40 at 14.5 s and 651 (XingZ624
  // and XingZ625 the same shape: Together 73 s and 82 s, Wafer 6.4 s and
  // 9 s). OpenRouter is not cleared for thinking parameters (the owner's
  // rulings of 2026-10-22 name no tier for it, so it counts as third tier),
  // so the seat names the measured endpoint instead.
  preferredEndpoints: ['wafer',],
  // Raw characters per completion token over pass-run logs, median of 15,239 streams
  // since 2026-09-21 (ledger P7, 2026-09-28).
  rawCharsPerToken: 302,
};

/**
 OpenRouter's side of the minimax-m3 card, routed to Together, then CoreWeave.

 @example
 ```ts
 const side = MINIMAX_M3_ON_OPENROUTER;
 ```
 */
export const MINIMAX_M3_ON_OPENROUTER: OpenRouterCard = {
  id: 'minimax/minimax-m3',
  readsImages: true,
  maxOutputLength: 512_000,
  promptUsdPerMillion: 0.3,
  completionUsdPerMillion: 1.2,
  // Parasail answered into the reasoning channel and left content empty
  // (2026-09-03); ModelRun timed out 119 of 300 streams in-stream on
  // 2026-09-04 (`d55d83082`). This said both "cut a quarter or more of at
  // least twenty streams" until ledger H12.
  ignoredEndpoints: [
    'parasail',
    'modelrun',
  ],
  // TOGETHER, THEN COREWEAVE, NAMED AHEAD OF THE PRICE SORT (ledger H12,
  // 2026-09-28). Ignoring one endpoint at a time left the price sort to
  // pick the next, unmeasured. Only three endpoints in the listing of
  // 2026-09-28 support structured outputs, which OpenRouter routes a
  // schema request to alone (provider-selection docs): CoreWeave, Together
  // and the ignored ModelRun, all FP4 (Together's own model table says FP4
  // where OpenRouter lists it unknown), so precision does not choose
  // between them. Over pass-run logs from 2026-09-21 to 2026-09-28 Together
  // served 7,601 streams, p50 1.9 s and p90 5.5 s, 18 not completed;
  // CoreWeave 37,870, p50 2.7 s and p90 9.7 s, 99 not completed. Observed,
  // not controlled for load. Together costs 0.30 and 1.20 a million, the
  // prices this card carries; CoreWeave 0.23 and 0.96. Fallbacks stay allowed.
  preferredEndpoints: [
    'together',
    'coreweave',
  ],
  // Raw characters per completion token over pass-run logs, median of 45,307 streams
  // since 2026-09-21 (ledger P7, 2026-09-28).
  rawCharsPerToken: 130,
};

/**
 OpenRouter's side of the DeepSeek V4.1 Flash card, which sends the 1017 id
 and is routed to Morph.

 @example
 ```ts
 const side = DEEPSEEK_V4_1_FLASH_ON_OPENROUTER;
 ```
 */
export const DEEPSEEK_V4_1_FLASH_ON_OPENROUTER: OpenRouterCard = {
  id: 'deepseek/deepseek-v4.1-flash-1017',
  readsImages: true,
  maxOutputLength: 384_000,
  // MORPH'S PRICE, THE ENDPOINT THIS SEAT BUYS (ledger P13, 2026-09-28).
  // The card carried the model's catalog price, 0.3 and 1.2, while 812 of
  // 817 calls on TianqiChen66616, 66619 and 66620 went to Morph, listed
  // that day at 0.12 and 0.468 and charged at 0.662 to 0.674 of it. Only
  // the abandoned-spend estimate reads these.
  //
  // 2026-10-20: THE 1017 ID'S MORPH LISTING READS 0.158 AND 0.6
  // (`GET https://openrouter.ai/api/v1/models/deepseek/deepseek-v4.1-flash-1017/endpoints`,
  // tag `morph/fp8`, pricing prompt 0.000000158 and completion 0.0000006).
  // The pre-1017 id's Morph price on this card was 0.12 and 0.468.
  promptUsdPerMillion: 0.158,
  completionUsdPerMillion: 0.6,
  // THE ENDPOINT WAS THE SLOW SEAT, NOT THE MODEL (XingZ607, 2026-09-18).
  // Over 990 completed judge calls the price sort spread across three
  // upstreams: Morph answered at 11 s median and 25 s at the ninetieth
  // percentile, DeepInfra at 45 s and 158 s with 89 of the 97 cut streams,
  // Wafer at 26 s and 122 s with the other 8. Every other judge seat
  // answers under 14 s at the ninetieth percentile, so on DeepInfra this
  // seat set the quorum time in 553 of 2,320 rounds (689 of 1,173 minutes
  // of quorum wait) and held 15 rounds to the 360 s timeout. XingZ605,
  // served by Morph alone, was the fastest XingZ60 run at 4h08m. Same
  // shape as class twenty-four (2026-09-10): route by the measured
  // endpoint. Slugs from the gateway's providers listing of 2026-09-18.
  //
  // OPENINFERENCE JOINED THEM (class ninety-one, XingZ624 and XingZ625,
  // 2026-09-23). Once DeepInfra and Wafer were off the wire the price
  // sort fell to OpenInference: XingZ624 served 324 streams there at
  // 137 s mean beside Morph's 579 at 11.5 s, XingZ625 738 at 116 s
  // beside Morph's 23 at 3.4 s. OpenInference also reasons past the
  // completion cap: 450 cap-cut replies on XingZ625 against 150 on
  // XingZ624 (93 in the editor seat, where 87 of 164 rounds heard
  // nobody), and the entry took 6h54m against XingZ608's 2h16m on
  // Morph. Slug from the same listing.
  //
  // DEKALLM AND SAIL RESEARCH FOLLOWED, AND THE SEAT NOW NAMES MORPH
  // (class ninety-three, XingZ626, 2026-09-23). With OpenInference off
  // the wire the price sort fell to DekaLLM, 432 streams at 54 s beside
  // Morph's 847 at 6.2 s, and Sail Research, 58 at 40 s. Neither is slow
  // at generating (186 and 219 characters a second after the first byte
  // against Morph's 168): they reason at length by default, 8,813 and
  // 8,355 characters a reply against Morph's 514, and the pipeline sends
  // no reasoning parameter to OpenRouter, which is not cleared for one
  // (third tier, the owner's rulings of 2026-10-22). DekaLLM also answered 22
  // in-stream 502s. Ignoring the next cheapest endpoint each run chases
  // the listing (DeepInfra, Wafer, OpenInference, now these two, Relace
  // next and unmeasured), so the seat names Morph ahead of the price
  // sort (`provider.order`, fallbacks allowed): 11.1 s, 11.5 s, 3.4 s
  // and 6.2 s a stream over XingZ608, 624, 625 and 626. Slugs from the
  // listing of 2026-09-23.
  //
  // 2026-10-20, THE 1017 ID'S ENDPOINT LISTING against these slugs: the
  // slugs are the prefix of each endpoint's tag (`deepinfra/fp8`,
  // `wafer/mxfp4`, `morph/fp8`), the form the gateway's `provider.ignore`
  // and `provider.order` take. DeepInfra and Wafer list the 1017 id, so
  // their ignores bite. OpenInference, DekaLLM and Sail Research list no
  // endpoint for it today, so those three ignores are inert until one
  // does; they stay because the measurements behind them are about those
  // upstreams, not about the weights.
  ignoredEndpoints: [
    'deepinfra',
    'wafer',
    'open-inference',
    'dekallm',
    'sail-research',
  ],
  preferredEndpoints: ['morph',],
  // Raw characters per completion token over pass-run logs, median of 47,186 streams
  // since 2026-09-21; all time reads 257 (ledger P7, 2026-09-28).
  rawCharsPerToken: 228,
};

//endregion OpenRouter card sides
