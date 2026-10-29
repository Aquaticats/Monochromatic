// oxlint-disable typescript/no-unsafe-type-assertion -- API response types require assertions
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type {
  VoyageApiRequest,
  VoyageApiResponse,
} from './types.voyage-api.ts';

/**
 Logger root for image-diff after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'image-diff', },);

/**
 Voyage AI multimodal embeddings API endpoint.
 */
export const VOYAGE_API_URL = 'https://api.voyageai.com/v1/multimodalembeddings';

/**
 Resolve the Voyage AI API key from config or environment.
 
 @param configKey - explicitly provided API key, if any
 
 @returns resolved API key
 
 @throws when no API key is available from either source
 
 @example
 ```ts
 const key = resolveVoyageApiKey(undefined);
 ```
 */
export function resolveVoyageApiKey(configKey?: string,): string {
  /**
   Logger pre-tagged with this function's name so call-site context is preserved across debug lines.
   */
  const rl = tagged({
    tag: resolveVoyageApiKey.name,
    l,
  },);
  /**
   Resolved key from explicit config, then preferred env var, then fallback env var; blank triggers the explicit error.
   */
  const key = configKey
    ?? process
      .env
      .IMAGE_DIFF_VOYAGE_API_KEY
    ?? process
      .env
      .VOYAGE_API_KEY;
  if ((key === undefined) || (key === '')) {
    throw new Error(
      'Voyage AI API key is required. Provide it via config.apiKey or set IMAGE_DIFF_VOYAGE_API_KEY (or VOYAGE_API_KEY) environment variable.',
    );
  }
  rl.debug('Voyage API key resolved',);
  return key;
}

/**
 Send a request to the Voyage AI multimodal embeddings API.
 
 @param requestBody - serializable request payload
 
 @param apiKey - Voyage AI API key for authorization
 
 @returns parsed API response
 
 @mutates requestBody - JSON.stringify may invoke getters, proxy traps, toJSON, or other conversion hooks on request body and reachable values.
 
 @throws on non-OK HTTP status with the error body
 
 @example
 ```ts
 const response = await callVoyageApi({ requestBody: request, apiKey: 'pa-...' });
 ```
 */
export async function callVoyageApi({
  requestBody,
  apiKey,
}: {
  readonly requestBody: ForeignBorrowed<VoyageApiRequest>;
  readonly apiKey: string;
},): Promise<VoyageApiResponse> {
  /**
   Logger pre-tagged with this function's name so call-site context is preserved across debug lines.
   */
  const rl = tagged({
    tag: callVoyageApi.name,
    l,
  },);

  rl.debug(
    `calling Voyage API with model "${requestBody.model}", ${
      String(requestBody
        .inputs
        .length,)
    } input(s)`,
  );

  /**
   Raw `fetch` response; status checked before parsing JSON so errors surface with their body.
   */
  const response = await fetch(
    VOYAGE_API_URL,
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
    rl.error(`API returned ${String(response.status,)}: ${errorBody}`,);
    throw new Error(`Voyage AI API error (${String(response.status,)}): ${errorBody}`,);
  }

  /**
   Parsed Voyage API payload; structure validated by the response-type assertion above.
   */
  const result = await response.json() as VoyageApiResponse;
  rl.debug(
    `received ${String(result.data
      .length,)} embedding(s), total tokens: ${
      String(result
        .usage
        .total_tokens,)
    }`,
  );
  return result;
}
