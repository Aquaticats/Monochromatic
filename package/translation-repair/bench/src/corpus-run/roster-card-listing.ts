import {
  BEDROCK_MANTLE_BASE_URL,
  DEEPSEEK_BASE_URL,
  NEURALWATT_BASE_URL,
  SYNTHETIC_CHAT_BASE_URL,
  XIAOMI_BASE_URL,
} from '@monochromatic-dev/translation-repair-provider/ts';
import type { ListedCardProvider, } from './roster-card-ask.ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import type { ModelTransport, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import {
  LISTING_TIMEOUT_MS,
  readProviderListing,
} from '@monochromatic-dev/translation-repair-insertion/ts';

//region Roster card listing
// FETCHES THE LIVE LISTING OF ONE PROVIDER for `roster-card`, with the key that
// provider is read with, over the transport the caller hands in.

/**
 Where each provider lists what it serves.
 */
export const LISTING_URL: Readonly<Record<ListedCardProvider, string>> = {
  synthetic: `${SYNTHETIC_CHAT_BASE_URL}/models`,
  hyper: 'https://hyper.charm.land/v1/models',
  openrouter: 'https://openrouter.ai/api/v1/models',
  bedrock: `${BEDROCK_MANTLE_BASE_URL}/v1/models`,
  neuralwatt: `${NEURALWATT_BASE_URL}/models`,
  // THE TWO DIRECT PROVIDERS SPEAK THE OPENAI-COMPATIBLE PROTOCOL, whose listing is `GET /models`. NEITHER ADDRESS
  // WAS PROBED by the change that added them (no key was used and nothing was fetched): DeepSeek's rests on its
  // documentation, and Xiaomi's API reference was read on 2026-10-22 without a listing page, so its address is the
  // protocol's alone.
  deepseek: `${DEEPSEEK_BASE_URL}/models`,
  xiaomi: `${XIAOMI_BASE_URL}/models`,
};

/**
 Environment variable carrying each provider's key, injected by mise.
 */
const KEY_VAR: Readonly<Record<ListedCardProvider, string>> = {
  synthetic: 'TRANSLATION_REPAIR_SYNTHETIC_API_KEY',
  hyper: 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY',
  openrouter: 'TRANSLATION_REPAIR_OPENROUTER_API_KEY',
  bedrock: 'TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY',
  neuralwatt: 'TRANSLATION_REPAIR_NEURALWATT_API_KEY',
  deepseek: 'DEEPSEEK_API_KEY',
  xiaomi: 'TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY',
};

/**
 Fetches one provider's listing.

 @param provider - whose listing

 @param env - environment the provider's key is read from: `process.env` in a run

 @param transport - transport the listing is fetched over: `fetchTransport` in a run

 @returns Decoded body

 @throws {@link StatedRefusalError} When the key is unset or the provider
 answers with a non-ok status

 @example
 ```ts
 const body = await fetchListing({ provider: 'synthetic', env: process.env, transport: fetchTransport, },);
 ```
 */
export async function fetchListing(
  {
    provider,
    env,
    transport,
  }: {
    readonly provider: ListedCardProvider;
    readonly env: Readonly<NodeJS.ProcessEnv>;
    readonly transport: ModelTransport;
  },
): Promise<unknown> {
  /**
   Key by the provider's variable name, never printed.
   */
  const apiKey = env[KEY_VAR[provider]] ?? '';
  if (apiKey === '') {
    throw new StatedRefusalError({
      says: `${KEY_VAR[provider]} is not set; run under mise so sops injects it`,
    },);
  }
  return await readProviderListing({
    url: LISTING_URL[provider],
    apiKey,
    transport,
    timeoutMs: LISTING_TIMEOUT_MS,
    statusRefusal: function says({ status, },): string {
      return `${LISTING_URL[provider]} answered ${String(status,)}; the reason phrase is the provider's wording and is dropped`;
    },
  },);
}

//endregion Roster card listing
