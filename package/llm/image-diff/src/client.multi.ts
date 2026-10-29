/**
 Multi-provider embedding operations that dispatch to all available
 embedding providers concurrently.
 
 @module
 */
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type {
  MultiProviderBatchEmbedEntry,
  MultiProviderEmbedEntry,
} from './client.multi.types.ts';
import {
  embed,
  embedBatch,
} from './client.ts';
import type {
  ImageInput,
  Provider,
} from './types.ts';

/**
 Logger root for image-diff after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'image-diff', },);

/**
 All available provider names, used when dispatching to all providers.
 */
const ALL_PROVIDERS: readonly Provider[] = [
  'voyage',
  'gemini',
];

/**
 Embed a single image using all available providers concurrently.
 Each provider uses its own API key (from env vars) and default latest model.
 
 @param input - image to embed
 
 @returns array of results, one per provider
 
 @example
 ```ts
 const results = await embedAll({ path: './photo.png' });
 for (const { provider, result } of results) {
   console.log(`${provider}: ${result.embedding.length} dimensions`);
 }
 ```
 */
export async function embedAll(
  input: ImageInput,
): Promise<readonly MultiProviderEmbedEntry[]> {
  /**
   Logger pre-tagged with this function's name so call-site context is preserved across debug lines.
   */
  const rl = tagged({
    tag: embedAll.name,
    l,
  },);
  rl.debug(`embedding image across all ${String(ALL_PROVIDERS.length,)} providers`,);

  /**
   Per-provider embedding entries collected from concurrent calls; one entry per {@link ALL_PROVIDERS} member.
   */
  const results = await Promise.all(
    ALL_PROVIDERS.map(async function embedWithProvider(provider,) {
      /**
       Single-provider embedding result; paired with the provider name in the returned entry.
       */
      const result = await embed({
        input,
        config: { provider, },
      },);
      return {
        provider,
        result,
      };
    },),
  );

  rl.debug('all provider embeddings complete',);
  return results;
}

/**
 Batch-embed multiple images using all available providers concurrently.
 Each provider uses its own API key (from env vars) and default latest model.
 
 @param inputs - array of images to embed
 
 @returns array of results, one per provider
 
 @example
 ```ts
 const results = await embedBatchAll([{ path: 'a.png' }, { path: 'b.png' }]);
 ```
 */
export async function embedBatchAll(
  inputs: readonly ImageInput[],
): Promise<readonly MultiProviderBatchEmbedEntry[]> {
  /**
   Logger pre-tagged with this function's name so call-site context is preserved across debug lines.
   */
  const rl = tagged({
    tag: embedBatchAll.name,
    l,
  },);
  rl.debug(
    `batch embedding ${String(inputs.length,)} image(s) across all ${
      String(ALL_PROVIDERS.length,)
    } providers`,
  );

  /**
   Per-provider batch entries collected from concurrent calls; one entry per {@link ALL_PROVIDERS} member.
   */
  const results = await Promise.all(
    ALL_PROVIDERS.map(async function embedBatchWithProvider(provider,) {
      /**
       Single-provider batch result; paired with the provider name in the returned entry.
       */
      const result = await embedBatch({
        inputs,
        config: { provider, },
      },);
      return {
        provider,
        result,
      };
    },),
  );

  rl.debug('all provider batch embeddings complete',);
  return results;
}
