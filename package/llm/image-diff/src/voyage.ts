// oxlint-disable typescript/no-unsafe-type-assertion, prefer-destructuring -- API response types require assertions
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { toVoyageContentItem, } from './encoding.voyage.ts';
import type {
  BatchEmbeddingResult,
  EmbeddingProvider,
  EmbeddingResult,
  ImageDiffConfig,
  ImageInput,
  VoyageModel,
} from './types.ts';
import type { VoyageApiRequest, } from './types.voyage-api.ts';
import {
  callVoyageApi,
  resolveVoyageApiKey,
} from './voyage.api.ts';

/**
 Logger root for image-diff after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'image-diff', },);

/**
 Default Voyage model; latest and highest quality.
 */
const DEFAULT_VOYAGE_MODEL: VoyageModel = 'voyage-multimodal-3.5';

/**
 Compute a single image embedding via the Voyage AI API.
 
 @param input - image to embed, in any supported format
 
 @param config - client configuration
 
 @returns embedding vector and usage metadata
 
 @example
 ```ts
 const { embedding } = await voyageEmbed({ input: { path: './photo.png' }, config: {} });
 ```
 */
async function voyageEmbed({
  input,
  config,
}: {
  readonly input: ImageInput;
  readonly config: ImageDiffConfig;
},): Promise<EmbeddingResult> {
  /**
   Logger pre-tagged with this function's name so call-site context is preserved across debug lines.
   */
  const rl = tagged({
    tag: voyageEmbed.name,
    l,
  },);
  rl.debug('computing single image embedding via Voyage',);

  /**
   Resolved Voyage credential; pulled here once and forwarded into the API call.
   */
  const apiKey = resolveVoyageApiKey(config.apiKey,);
  /**
   Effective model id; user override or {@link DEFAULT_VOYAGE_MODEL}.
   */
  const model = (config.model
    ?? DEFAULT_VOYAGE_MODEL) as VoyageModel;
  /**
   Voyage-shaped content payload converted from the caller's image input.
   */
  const contentItem = await toVoyageContentItem(input,);

  /**
   Single-input request body wrapping the content item in Voyage's nested `inputs[].content[]` shape.
   */
  const request: VoyageApiRequest = {
    inputs: [{ content: [contentItem,], },],
    model,
    input_type: 'document',
    truncation: true,
  };

  /**
   Voyage API response; contains the embedding plus usage counters.
   */
  const response = await callVoyageApi({
    requestBody: request,
    apiKey,
  },);
  /**
   First (and only) data entry; guarded by the empty-array check below before use.
   */
  const firstData = response.data[0];
  if (firstData === undefined)
    throw new Error('Voyage API returned empty data array',);

  return {
    embedding: firstData.embedding,
    usage: {
      textTokens: response.usage
        .text_tokens,
      imagePixels: response.usage
        .image_pixels,
      totalTokens: response.usage
        .total_tokens,
    },
  };
}

/**
 Compute embeddings for multiple images in a single batch via the Voyage AI API.
 
 @param inputs - array of images to embed
 
 @param config - client configuration
 
 @returns embedding vectors (in input order) and aggregate usage metadata
 
 @example
 ```ts
 const { embeddings } = await voyageEmbedBatch({
   inputs: [{ path: 'a.png' }, { path: 'b.png' }],
   config: {},
 });
 ```
 */
async function voyageEmbedBatch({
  inputs,
  config,
}: {
  readonly inputs: readonly ImageInput[];
  readonly config: ImageDiffConfig;
},): Promise<BatchEmbeddingResult> {
  /**
   Logger pre-tagged with this function's name so call-site context is preserved across debug lines.
   */
  const rl = tagged({
    tag: voyageEmbedBatch.name,
    l,
  },);
  rl.debug(
    `computing batch embeddings via Voyage for ${String(inputs.length,)} image(s)`,
  );

  /**
   Resolved Voyage credential; pulled here once and forwarded into the API call.
   */
  const apiKey = resolveVoyageApiKey(config.apiKey,);
  /**
   Effective model id; user override or {@link DEFAULT_VOYAGE_MODEL}.
   */
  const model = (config.model
    ?? DEFAULT_VOYAGE_MODEL) as VoyageModel;

  /**
   Voyage-shaped content payloads converted from each caller image, in input order.
   */
  const contentItems = await Promise.all(
    inputs.map(function convertInput(input,) {
      return toVoyageContentItem(input,);
    },),
  );

  /**
   Batched request body wrapping each content item in its own `inputs[]` entry.
   */
  const request: VoyageApiRequest = {
    inputs: contentItems.map(function wrapContent(item,) {
      return { content: [item,], };
    },),
    model,
    input_type: 'document',
    truncation: true,
  };

  /**
   Voyage API response; data entries may arrive out of order and need re-sorting below.
   */
  const response = await callVoyageApi({
    requestBody: request,
    apiKey,
  },);

  /**
   Sort by index to guarantee input order.
   */
  const sorted = [...response.data,].toSorted(function byIndex(
    a,
    b,
  ) {
    return a.index
      - b
        .index;
  },);

  return {
    embeddings: sorted.map(function extractEmbedding(d,) {
      return d.embedding;
    },),
    usage: {
      textTokens: response.usage
        .text_tokens,
      imagePixels: response.usage
        .image_pixels,
      totalTokens: response.usage
        .total_tokens,
    },
  };
}

/**
 Voyage AI embedding provider.
 Implements the {@link EmbeddingProvider} interface for the Voyage multimodal API.
 
 @example
 ```ts
 const result = await voyageProvider.embed({ path: 'photo.png' }, {});
 ```
 */
export const voyageProvider: EmbeddingProvider = {
  embed: voyageEmbed,
  embedBatch: voyageEmbedBatch,
};
