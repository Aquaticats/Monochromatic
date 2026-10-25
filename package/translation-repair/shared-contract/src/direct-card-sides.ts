import type {
  DeepSeekCard,
  XiaomiCard,
} from './model-card.ts';

//region Direct card sides
// THE SIDES OF THE TWO CARDS THE OWNER'S OWN KEYS REACH DIRECTLY, 2026-10-22,
// kept beside `model-cards.ts` for the reason `neuralwatt-card-sides.ts` gives:
// that table sits at the package's line budget. Each side serves exactly ONE
// model, because each key was granted for one model only:
//
//   "You can also use the DEEPSEEK_API_KEY as you wish, but you may only use
//   deepseek v4.1 flash 1017 with that key."
//   "I've added TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY in main worktree's json
//   file. ... You may only use v2.6-flash with this key."
//
// NEITHER KEY IS MODEL-SCOPED (as the parent session reported its listings of 2026-10-22, which this change did
// NOT repeat: DeepSeek `GET /models` lists `deepseek-flash` and `deepseek-v4-pro`; Xiaomi's lists
// `mimo-v2.6-pro`, `mimo-v2.6-pro-ultraspeed` and the speech models beside
// `mimo-v2.6-flash`), so the lock is in our code, three ways: the served-id
// type of each is a single literal (`roster-id.ts`), each side appears on one
// card only (`direct-cards.unit.test.ts` fails on a second), and the client
// throws for any other roster model before any request leaves
// (`direct-client.ts`).
//
// NO ZERO DATA RETENTION on either route, and none promised. The owner's ruling
// of 2026-10-05 (the corpus is public, PII is allowed) and the two grants quoted here
// are what permit it. Record: `doc/decision/translation-repair-direct-providers.md`.
//
// NO HOLD CHANGES WITH THESE SIDES. Both cards keep every hold they carried
// before the provider joined.

/**
 DeepSeek's side of the DeepSeek V4.1 Flash card.

 THE SERVED ID IS `deepseek-flash`, AND THE 1017 CLAIM RESTS ON DEEPSEEK'S
 DOCUMENTATION. `GET https://api.deepseek.com/models` lists `deepseek-flash` and
 `deepseek-v4-pro` only, and there is no dated 1017 id (the parent session's listing; not fetched by this
 change). The pricing page
 (`https://api-docs.deepseek.com/quick_start/pricing`, read 2026-10-22) says
 `deepseek-flash` "is now served by DeepSeek-V4.1-Flash-1017", lists a context
 of 1M and a maximum output of 384K (393,216 tokens, the figure NeuralWatt
 lists for the same weights), and JSON output. Every reply carries
 `system_fingerprint`, which the client logs.

 IMAGE INPUT IS PROBED, NOT MEASURED: a red pixel was answered "Red" on
 2026-10-22 (the parent session's probe; not repeated by this change), so the side reports images; the card's
 `reader-unmeasured` hold
 still keeps pictures off this model, as on its NeuralWatt side.

 THE CARD HOLDS THE PEAK RATES. The page lists 0.006 (cache hit), 0.3 (cache
 miss) and 1.2 (output) USD per million tokens at peak and half of each off peak.
 Peak is 01:00 to 04:00 UTC and 06:00 to 10:00 UTC, Monday to Friday, excluding
 Chinese public holidays. The provider package prices a call at these rates when
 its request started, or its reply completed, inside a weekday peak window (UTC)
 and at half of them only when both instants lie outside every window
 (`provider/src/direct-cost.ts`). It reads no holiday calendar, so a holiday
 inside a window can only make that call cheaper than booked, and the page does
 not say whether the start or the end of a request decides its rate, so a call
 that crosses a window edge is booked at peak: the booked figure is an upper
 bound as to holidays and as to which instant decides. It is not bounded as to a
 Saturday that China makes a working day, which the page's wording books off
 peak and which DeepSeek may bill at peak.

 @example
 ```ts
 const side = DEEPSEEK_V4_1_FLASH_ON_DEEPSEEK;
 ```
 */
export const DEEPSEEK_V4_1_FLASH_ON_DEEPSEEK: DeepSeekCard = {
  id: 'deepseek-flash',
  readsImages: true,
  maxOutputLength: 393_216,
  promptCacheHitUsdPerMillion: 0.006,
  promptCacheMissUsdPerMillion: 0.3,
  completionUsdPerMillion: 1.2,
};

/**
 Xiaomi's side of the MiMo V2.6 Flash card.

 The served id is `mimo-v2.6-flash`, the id the pay-as-you-go key answers to
 (`GET https://api.xiaomimimo.com/v1/models` as the parent session's probe of
 2026-10-22 reported it; this change fetched no listing). The chat reference
 (`https://mimo.mi.com/docs/en-US/api/chat/openai-api`, read 2026-10-22) gives
 `max_completion_tokens` a range of 1 to 131,072 with that default for this
 model, the same ceiling OpenRouter's listing states.

 IMAGE INPUT IS PROBED, NOT MEASURED: a picture sent as an `image_url` data
 URL was accepted and read on 2026-10-22 (the parent session's probe; not repeated by this change), with
 `image_tokens` in the usage; the
 card's `reader-unmeasured` hold keeps pictures off this model.

 PRICES are the overseas real-time rates of the pay-as-you-go page
 (`https://mimo.mi.com/docs/en-US/price/pay-as-you-go`, read 2026-10-22, update
 time October 21, 2026): cache hit 0.0028, cache miss 0.14 and output 0.28 USD
 per million tokens. Cache write is "Limited-time Free". The miss and output
 rates equal the OpenRouter card's.

 @example
 ```ts
 const side = MIMO_V2_6_FLASH_ON_XIAOMI;
 ```
 */
export const MIMO_V2_6_FLASH_ON_XIAOMI: XiaomiCard = {
  id: 'mimo-v2.6-flash',
  readsImages: true,
  maxOutputLength: 131_072,
  promptCacheHitUsdPerMillion: 0.0028,
  promptCacheMissUsdPerMillion: 0.14,
  completionUsdPerMillion: 0.28,
};

//endregion Direct card sides
