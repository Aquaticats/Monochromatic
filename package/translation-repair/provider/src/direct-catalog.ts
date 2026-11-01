import {
  type DeepSeekCard,
  type DeepSeekServedId,
  type RosterModelId,
  servedRecord,
  type XiaomiCard,
  type XiaomiServedId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Direct catalog
// WHAT DEEPSEEK'S AND XIAOMI'S OWN APIS SERVE FOR THIS PIPELINE, how each
// spells it and where each is called. Each provider serves exactly ONE model,
// the one the owner's key there is granted for (2026-10-22, quoted in
// `direct-card-sides.ts`), so each catalog has one row, read off the card.
//
// BOTH SPEAK THE OPENAI-COMPATIBLE PROTOCOL, over the same transport seam as
// every other client, with a bearer key. Facts, all read or probed 2026-10-22:
//
//   DeepSeek: base `https://api.deepseek.com`, `POST /chat/completions`,
//   `GET /user/balance` (`https://api-docs.deepseek.com`).
//   Xiaomi: base `https://api.xiaomimimo.com/v1`, `POST /v1/chat/completions`
//   (`https://mimo.mi.com/docs/en-US/api/chat/openai-api`). It documents no
//   balance or billing endpoint (the API reference, read 2026-10-22), so its
//   meter is the run's own priced spend (`xiaomi-spend.ts`).
//
// NEITHER KEY IS MODEL-SCOPED, which is why the lock is in the types and the
// client and not in the provider's account: `direct-client.ts` throws for any
// roster model but the row's before a request leaves.

/**
 Base address every DeepSeek route hangs off.
 */
export const DEEPSEEK_BASE_URL = 'https://api.deepseek.com';

/**
 Base address every Xiaomi route hangs off.
 */
export const XIAOMI_BASE_URL = 'https://api.xiaomimimo.com/v1';

/**
 Header carrying either key, a bearer token as on the other providers.
 */
export const DIRECT_AUTH_HEADER = 'Authorization';

/**
 One direct provider's verified per-model facts the router and the request
 builder read.

 @example
 ```ts
 const info: DirectModelInfo<DeepSeekCard> = DEEPSEEK_MODELS['deepseek-flash'];
 ```
 */
export type DirectModelInfo<CardT,> = CardT & {
  /**
   The roster seat this spelling reaches.
   */
  readonly sharedWith: RosterModelId;
};

/**
 Every model DeepSeek serves for this pipeline, read off the cards: one.

 @example
 ```ts
 const info = DEEPSEEK_MODELS['deepseek-flash'];
 ```
 */
export const DEEPSEEK_MODELS: Readonly<Record<DeepSeekServedId, DirectModelInfo<DeepSeekCard>>> = servedRecord({
  provider: 'deepseek',
  toRow: function deepSeekRow(card,): DirectModelInfo<DeepSeekCard> {
    return {
      ...card.deepseek,
      sharedWith: card.id,
    };
  },
},);

/**
 Every model Xiaomi serves for this pipeline, read off the cards: one.

 @example
 ```ts
 const info = XIAOMI_MODELS['mimo-v2.6-flash'];
 ```
 */
export const XIAOMI_MODELS: Readonly<Record<XiaomiServedId, DirectModelInfo<XiaomiCard>>> = servedRecord({
  provider: 'xiaomi',
  toRow: function xiaomiRow(card,): DirectModelInfo<XiaomiCard> {
    return {
      ...card.xiaomi,
      sharedWith: card.id,
    };
  },
},);

/**
 DeepSeek chat completions URL.

 @param baseUrl - base address, overridable for tests

 @returns Absolute URL to POST the body to

 @example
 ```ts
 deepSeekChatUrlFor({ baseUrl: DEEPSEEK_BASE_URL, },);
 // => 'https://api.deepseek.com/chat/completions'
 ```
 */
export function deepSeekChatUrlFor({ baseUrl, }: { readonly baseUrl: string; },): string {
  return `${baseUrl}/chat/completions`;
}

/**
 DeepSeek balance URL, the one meter endpoint either provider documents.

 @param baseUrl - base address, overridable for tests

 @returns Absolute URL to GET

 @example
 ```ts
 deepSeekBalanceUrlFor({ baseUrl: DEEPSEEK_BASE_URL, },);
 // => 'https://api.deepseek.com/user/balance'
 ```
 */
export function deepSeekBalanceUrlFor({ baseUrl, }: { readonly baseUrl: string; },): string {
  return `${baseUrl}/user/balance`;
}

/**
 Xiaomi chat completions URL.

 @param baseUrl - base address, overridable for tests

 @returns Absolute URL to POST the body to

 @example
 ```ts
 xiaomiChatUrlFor({ baseUrl: XIAOMI_BASE_URL, },);
 // => 'https://api.xiaomimimo.com/v1/chat/completions'
 ```
 */
export function xiaomiChatUrlFor({ baseUrl, }: { readonly baseUrl: string; },): string {
  return `${baseUrl}/chat/completions`;
}

//endregion Direct catalog
