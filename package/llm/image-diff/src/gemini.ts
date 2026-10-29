// oxlint-disable typescript/no-unsafe-type-assertion -- API response types require assertions; provider interface requires async
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { toGeminiInlineData, } from './encoding.gemini.ts';
import { geminiEmbedBatch, } from './gemini.batch.ts';
import {
  DEFAULT_GEMINI_MODEL,
  GEMINI_API_BASE,
  resolveGeminiApiKey,
} from './gemini.config.ts';
import type {
  GeminiEmbedContentRequest,
  GeminiEmbedContentResponse,
} from './types.gemini-api.ts';
import type {
  EmbeddingProvider,
  EmbeddingResult,
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
 Compute a single image embedding via the Gemini embedContent API.
 
 @param input - image to embed, in any supported format
 
 @param config - client configuration
 
 @returns embedding vector and usage metadata
 
 @example
 ```ts
 const { embedding } = await geminiEmbed({ input: { path: './photo.png' }, config: {} });
 ```
 */
async function geminiEmbed({
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
    tag: geminiEmbed.name,
    l,
  },);
  rl.debug('computing single image embedding via Gemini',);

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
   Gemini-shaped inline data payload converted from the caller's image input.
   */
  const inlineData = await toGeminiInlineData(input,);

  /**
   embedContent request body wrapping the inline data in Gemini's `content.parts[]` shape.
   */
  const requestBody: GeminiEmbedContentRequest = {
    content: {
      parts: [{ inline_data: inlineData, },],
    },
  };

  /**
   Full embedContent endpoint URL with the resolved model interpolated.
   */
  const url = `${GEMINI_API_BASE}/${model}:embedContent`;
  rl.debug(`calling Gemini API: ${url}`,);

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
    rl.error(`Gemini API returned ${String(response.status,)}: ${errorBody}`,);
    throw new Error(`Gemini API error (${String(response.status,)}): ${errorBody}`,);
  }

  /**
   Parsed embedContent payload; embedding vector lives at `embedding.values`.
   */
  const result = await response.json() as GeminiEmbedContentResponse;
  rl.debug(
    `received embedding with ${String(result.embedding
      .values
      .length,)} dimensions`,
  );

  return {
    embedding: result.embedding
      .values,
    usage: {
      textTokens: 0,
      imagePixels: 0,
      totalTokens: 0,
    },
  };
}

/**
 Gemini embedding provider.
 Implements the {@link EmbeddingProvider} interface for the Gemini multimodal API.
 
 @example
 ```ts
 const result = await geminiProvider.embed({ path: 'photo.png' }, {});
 ```
 */
export const geminiProvider: EmbeddingProvider = {
  embed: geminiEmbed,
  embedBatch: geminiEmbedBatch,
};
