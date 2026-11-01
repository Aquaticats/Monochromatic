// oxlint-disable typescript/no-unsafe-type-assertion -- API response types require assertions
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { toGeminiInlineData, } from './encoding.gemini.ts';
import {
  DEFAULT_GEMINI_MODEL,
  GEMINI_API_BASE,
  resolveGeminiApiKey,
} from './gemini.config.ts';
import type {
  GeminiBatchEmbedRequest,
  GeminiBatchEmbedResponse,
} from './types.gemini-api.ts';
import type {
  BatchEmbeddingResult,
  GeminiModel,
  ImageDiffConfig,
  ImageInput,
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
 Compute embeddings for multiple images via the Gemini batchEmbedContents API.
 
 @param inputs - array of images to embed
 
 @param config - client configuration
 
 @returns embedding vectors (in input order) and aggregate usage metadata
 
 @example
 ```ts
 const { embeddings } = await geminiEmbedBatch({
   inputs: [{ path: 'a.png' }, { path: 'b.png' }],
   config: {},
 });
 ```
 */
export async function geminiEmbedBatch({
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
    tag: geminiEmbedBatch.name,
    l,
  },);
  rl.debug(
    `computing batch embeddings via Gemini for ${String(inputs.length,)} image(s)`,
  );

  /**
   Resolved Gemini credential; pulled here once and forwarded into the API call.
   */
  const apiKey = resolveGeminiApiKey(config.apiKey,);
  /**
   Effective model id; user override or {@link DEFAULT_GEMINI_MODEL}.
   */
  const model = (config.model
    ?? DEFAULT_GEMINI_MODEL) as GeminiModel;

  /**
   Gemini-shaped inline data payloads converted from each caller image, in input order.
   */
  const inlineDataItems = await Promise.all(
    inputs.map(function convertInput(input,) {
      return toGeminiInlineData(input,);
    },),
  );

  /**
   batchEmbedContents request body wrapping each inline item in its own per-request entry.
   */
  const requestBody: GeminiBatchEmbedRequest = {
    requests: inlineDataItems.map(function wrapInlineData(inlineData,) {
      return {
        model: `models/${model}`,
        content: {
          parts: [{ inline_data: inlineData, },],
        },
      };
    },),
  };

  /**
   Full batchEmbedContents endpoint URL with the resolved model interpolated.
   */
  const url = `${GEMINI_API_BASE}/${model}:batchEmbedContents`;
  rl.debug(`calling Gemini batch API: ${url}, ${String(inputs.length,)} input(s)`,);

  /**
   Raw `fetch` response; status checked before parsing JSON so errors surface with their body.
   */
  const response = await fetch(
    url,
    {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'x-goog-api-key': apiKey,
      },
      body: JSON.stringify(requestBody,),
    },
  );

  if (!response.ok) {
    /**
     Raw response body captured for both the log line and the thrown error message.
     */
    const errorBody = await response.text();
    rl.error(`Gemini batch API returned ${String(response.status,)}: ${errorBody}`,);
    throw new Error(`Gemini API error (${String(response.status,)}): ${errorBody}`,);
  }

  /**
   Parsed batchEmbedContents payload; embedding vectors arrive in input order at `embeddings[]`.
   */
  const result = await response.json() as GeminiBatchEmbedResponse;
  rl.debug(`received ${String(result.embeddings
    .length,)} embedding(s)`,);

  return {
    embeddings: result.embeddings
      .map(function extractValues(e,) {
        return e.values;
      },),
    usage: {
      textTokens: 0,
      imagePixels: 0,
      totalTokens: 0,
    },
  };
}
