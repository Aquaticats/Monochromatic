// oxlint-disable typescript/no-unsafe-type-assertion -- API response types require assertions
import type {
  ChatCompletionResponse,
  ChatRole,
  ContentPart,
} from '@monochromatic-dev/llm-llm-type/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { ABSENT, } from './describe.absent.ts';
import { describeViaGemini, } from './describe.gemini.ts';
import { toImageUri, } from './encoding.uri.ts';
import type { ImageInput, } from './types.ts';

/**
 Logger root for image-diff after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'image-diff', },);

/**
 OpenRouter chat completions endpoint URL.
 */
const OPENROUTER_API_URL = 'https://openrouter.ai/api/v1/chat/completions';

/**
 Model ID for Gemini 3.1 Pro Preview on OpenRouter.
 */
const MODEL = 'google/gemini-3.1-pro-preview';

/**
 Prompt instructing the model to describe visual differences between two images.
 */
const DESCRIBE_PROMPT =
  `Compare these two images and describe all visual differences in detail.
Cover: layout changes, color differences, typography changes, spacing modifications,
elements that were added or removed, and any other noticeable changes.
Image A is the first image, Image B is the second.`;

//region OpenRouter API types

/**
 Vision chat message: one user turn carrying interleaved text and image parts.
 */
type VisionMessage = {
  readonly role: Extract<ChatRole, 'user'>;
  readonly content: readonly ContentPart[];
};

/**
 Request body for the OpenRouter chat completions API.
 */
type ChatCompletionRequest = {
  readonly model: string;
  readonly messages: readonly VisionMessage[];
};

//endregion OpenRouter API types

/**
 Resolve the OpenRouter API key from the environment.
 Returns `null` when no key is available, allowing callers
 to skip the description gracefully.
 
 @returns resolved API key, or {@link ABSENT} if not configured
 
 @example
 ```ts
 const key = resolveOpenRouterApiKey();
 if (key === ABSENT) return ABSENT;
 ```
 */
function resolveOpenRouterApiKey(): string | typeof ABSENT {
  /**
   Logger pre-tagged with this function's name so call-site context is preserved across debug lines.
   */
  const rl = tagged({
    tag: resolveOpenRouterApiKey.name,
    l,
  },);
  /**
   Resolved API key from preferred-then-fallback env var; treated as missing when blank.
   */
  const key = process.env
    .IMAGE_DIFF_OPENROUTER_API_KEY
    ?? process
      .env
      .OPENROUTER_API_KEY;
  if ((key === undefined) || (key === '')) {
    rl.debug('no OpenRouter API key found, skipping description',);
    return ABSENT;
  }
  rl.debug('OpenRouter API key resolved',);
  return key;
}

/**
 Describe visual differences between two images using the native Gemini API
 (preferred) or OpenRouter as a fallback. Returns {@link ABSENT} when no API key
 is configured for either backend.
 
 @param imageA - first image (before)
 
 @param imageB - second image (after)
 
 @returns detailed description of visual differences, or {@link ABSENT} when no API key is configured
 
 @throws when the API call itself fails (key is present but request errors)
 
 @example
 ```ts
 const description = await describeImageDifference({
   imageA: { path: './before.png' },
   imageB: { path: './after.png' },
 });
 if (description !== ABSENT) console.log(description);
 ```
 */
export async function describeImageDifference({
  imageA,
  imageB,
}: {
  readonly imageA: ImageInput;
  readonly imageB: ImageInput;
},): Promise<string | typeof ABSENT> {
  /**
   Logger pre-tagged with this function's name so call-site context is preserved across debug lines.
   */
  const rl = tagged({
    tag: describeImageDifference.name,
    l,
  },);

  // Prefer the native Gemini API: avoids the OpenRouter proxy overhead
  /**
   Description from the preferred Gemini backend; `null` when no Gemini key is configured.
   */
  const geminiResult = await describeViaGemini({
    imageA,
    imageB,
  },);
  if (geminiResult !== ABSENT) {
    rl.debug('description obtained via native Gemini API',);
    return geminiResult;
  }

  // Fall back to OpenRouter when no Gemini API key is available
  /**
   OpenRouter credential; absence triggers an early {@link ABSENT} return so callers can skip the description step.
   */
  const apiKey = resolveOpenRouterApiKey();
  if (apiKey === ABSENT)
    return ABSENT;

  rl.debug('describing image differences via Gemini 3.1 Pro Preview on OpenRouter',);
  /**
   Both images encoded as data URIs in parallel so the request body can embed them inline.
   */
  const [uriA, uriB,] = await Promise.all([
    toImageUri(imageA,),
    toImageUri(imageB,),
  ],);

  /**
   OpenRouter chat-completions payload pairing the diff prompt with the two image URIs.
   */
  const requestBody: ChatCompletionRequest = {
    model: MODEL,
    messages: [
      {
        role: 'user',
        content: [
          {
            type: 'text',
            text: DESCRIBE_PROMPT,
          },
          {
            type: 'image_url',
            image_url: { url: uriA, },
          },
          {
            type: 'image_url',
            image_url: { url: uriB, },
          },
        ],
      },
    ],
  };

  rl.debug(`calling OpenRouter API with model ${MODEL}`,);

  /**
   Raw `fetch` response; status checked before parsing JSON so errors surface with their body.
   */
  const response = await fetch(
    OPENROUTER_API_URL,
    {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${apiKey}`,
      },
      body: JSON.stringify(requestBody,),
    },
  );

  if (!response.ok) {
    /**
     Raw response body captured for both the log line and the thrown error message.
     */
    const errorBody = await response.text();
    rl.error(`OpenRouter API returned ${String(response.status,)}: ${errorBody}`,);
    throw new Error(`OpenRouter API error (${String(response.status,)}): ${errorBody}`,);
  }

  /**
   Parsed chat-completion payload; structure validated by the discriminating-empty-choices check below.
   */
  const result = await response.json() as ChatCompletionResponse;
  /**
   First choice destructured for content access; guarded against the empty-choices case.
   */
  const [choice,] = result.choices;
  if (choice === undefined)
    throw new Error('OpenRouter API returned no choices',);

  /**
   Model's textual diff description; returned directly to the caller after a debug-log of its length.
   */
  const description = choice.message
    .content;
  rl.debug(`received description (${String(description.length,)} chars)`,);
  return description;
}
