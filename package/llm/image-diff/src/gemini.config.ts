import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type { GeminiModel, } from './types.ts';

/**
 Logger root for image-diff after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'image-diff', },);

/**
 Gemini API base URL for embedding endpoints.
 */
export const GEMINI_API_BASE = 'https://generativelanguage.googleapis.com/v1beta/models';

/**
 Default Gemini model; latest multimodal embedding preview.
 */
export const DEFAULT_GEMINI_MODEL: GeminiModel = 'gemini-embedding-2-preview';

/**
 Resolve the Gemini API key from config or environment.
 
 @param configKey - explicitly provided API key, if any
 
 @returns resolved API key
 
 @throws when no API key is available from either source
 
 @example
 ```ts
 const key = resolveGeminiApiKey(undefined);
 ```
 */
export function resolveGeminiApiKey(configKey?: string,): string {
  /**
   Logger pre-tagged with this function's name so call-site context is preserved across debug lines.
   */
  const rl = tagged({
    tag: resolveGeminiApiKey.name,
    l,
  },);
  /**
   Resolved key from explicit config, then preferred env var, then fallback env var; blank triggers the explicit error.
   */
  const key = configKey
    ?? process
      .env
      .IMAGE_DIFF_GEMINI_API_KEY
    ?? process
      .env
      .GEMINI_API_KEY;
  if ((key === undefined) || (key === '')) {
    throw new Error(
      'Gemini API key is required. Provide it via config.apiKey or set IMAGE_DIFF_GEMINI_API_KEY (or GEMINI_API_KEY) environment variable.',
    );
  }
  rl.debug('Gemini API key resolved',);
  return key;
}
