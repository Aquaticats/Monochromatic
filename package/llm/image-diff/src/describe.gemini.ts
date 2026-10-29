import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { ABSENT, } from './describe.absent.ts';
import { toGeminiInlineData, } from './encoding.gemini.ts';
import { GEMINI_API_BASE, } from './gemini.config.ts';
import type { ImageInput, } from './types.ts';

/**
 Logger root for image-diff after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'image-diff', },);

//region Gemini generateContent API types

/**
 Content part for the Gemini generateContent API.
 Supports both text and inline image data.
 */
type GenerateContentPart =
  | { readonly text: string; }
    | { readonly inline_data: {
      readonly mime_type: string;
      readonly data: string;
    }; };

/**
 Request body for the Gemini generateContent API.
 */
type GenerateContentRequest = {
  readonly contents: readonly {
    readonly parts: readonly GenerateContentPart[];
  }[];
};

/**
 Response from the Gemini generateContent API.
 */
type GenerateContentResponse = {
  readonly candidates: readonly {
    readonly content: {
      readonly parts: readonly { readonly text: string; }[];
    };
  }[];
};

//endregion Gemini generateContent API types

/**
 Model for image description via the native Gemini API.
 Matches the model used via OpenRouter (`google/gemini-3.1-pro-preview`).
 */
const GEMINI_DESCRIBE_MODEL = 'gemini-3.1-pro-preview';

/**
 Prompt instructing the model to describe visual differences between two images.
 */
const DESCRIBE_PROMPT =
  `Compare these two images and describe all visual differences in detail.
Cover: layout changes, color differences, typography changes, spacing modifications,
elements that were added or removed, and any other noticeable changes.
Image A is the first image, Image B is the second.`;

/**
 Resolve Gemini API key for description, returning {@link ABSENT} when
 unavailable instead of throwing.
 
 @returns resolved API key, or {@link ABSENT} if not configured
 */
function resolveGeminiDescribeKey(): string | typeof ABSENT {
  /**
   Resolved Gemini key from preferred-then-fallback env var; treated as missing when blank.
   */
  const key = process.env
    .IMAGE_DIFF_GEMINI_API_KEY
    ?? process
      .env
      .GEMINI_API_KEY;
  if ((key === undefined) || (key === ''))
    return ABSENT;
  return key;
}

/**
 Describe visual differences between two images using the native Gemini
 generateContent API. Preferred over OpenRouter when a Gemini API key is available
 because it avoids the proxy overhead.
 
 @param imageA - first image (before)
 
 @param imageB - second image (after)
 
 @returns detailed description of visual differences, or {@link ABSENT} when no Gemini API key is configured
 
 @throws when the API call itself fails (key is present but request errors)
 
 @example
 ```ts
 const description = await describeViaGemini({
   imageA: { path: './before.png' },
   imageB: { path: './after.png' },
 });
 ```
 */
export async function describeViaGemini({
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
    tag: describeViaGemini.name,
    l,
  },);

  /**
   Gemini credential; absence triggers an early {@link ABSENT} return so the OpenRouter fallback can run.
   */
  const apiKey = resolveGeminiDescribeKey();
  if (apiKey === ABSENT)
    return ABSENT;

  rl.debug('describing image differences via native Gemini API',);
  /**
   Both images encoded as Gemini `inline_data` parts in parallel so the request body can embed them.
   */
  const [inlineA, inlineB,] = await Promise.all([
    toGeminiInlineData(imageA,),
    toGeminiInlineData(imageB,),
  ],);

  /**
   Gemini generateContent payload pairing the diff prompt with the two inline image parts.
   */
  const requestBody: GenerateContentRequest = {
    contents: [
      {
        parts: [
          { text: DESCRIBE_PROMPT, },
          { inline_data: {
            mime_type: inlineA.mime_type,
            data: inlineA.data,
          }, },
          { inline_data: {
            mime_type: inlineB.mime_type,
            data: inlineB.data,
          }, },
        ],
      },
    ],
  };

  /**
   Full Gemini generateContent endpoint URL with the description model interpolated.
   */
  const url = `${GEMINI_API_BASE}/${GEMINI_DESCRIBE_MODEL}:generateContent`;
  rl.debug(`calling Gemini generateContent: ${url}`,);

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
    rl.error(
      `Gemini generateContent returned ${String(response.status,)}: ${errorBody}`,
    );
    throw new Error(
      `Gemini generateContent error (${String(response.status,)}): ${errorBody}`,
    );
  }

  /**
   Parsed generateContent payload; structure validated by the candidate/part guards below.
   */
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- API response type assertion
  const result = await response.json() as GenerateContentResponse;
  /**
   First candidate destructured for content access; guarded against the empty-candidates case.
   */
  const [candidate,] = result.candidates;
  if (candidate === undefined)
    throw new Error('Gemini generateContent returned no candidates',);

  /**
   First content part destructured for text access; guarded against the empty-parts case.
   */
  const [part,] = candidate.content
    .parts;
  if (part === undefined)
    throw new Error('Gemini generateContent returned no content parts',);

  /**
   Model's textual diff description; returned directly to the caller after a debug-log of its length.
   */
  const description = part.text;
  rl.debug(
    `received description (${String(description.length,)} chars) via native Gemini`,
  );
  return description;
}
