/**
 Linkup HTTP request helpers.
 
 @module
 */

import { caughtValueText as errorMessage, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  ABORT_ERROR_NAME,
  AUTHORIZATION_HEADER,
  CONTENT_TYPE_HEADER,
  HTTP_POST,
  JSON_CONTENT_TYPE,
  USER_AGENT_HEADER,
  USER_AGENT_VALUE,
} from './client-constants.ts';
import type {
  ClientRuntime,
  ExtractedLinkupErrorMessage,
  PostJsonOptions,
} from './client-types.ts';

/**
 Logger root for pi-search-fetch after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: linkupLogger, },);
 ```
 */
const linkupLogger = tagged({ tag: 'pi-search-fetch', },);

/**
 Module logger.
 */
const l = tagged({
  tag: 'client-http',
  l: linkupLogger,
},);

//region Request helpers

/**
 POST JSON to Linkup and parse JSON response.
 
 @param runtime - client runtime dependencies
 
 @param endpoint - endpoint path
 
 @param body - JSON request body
 
 @param signal - optional cancellation signal
 
 @returns parsed JSON response
 
 @mutates runtime - `sendRequest` invokes configured fetch provider capability.
 
 @mutates body - `JSON.stringify` may invoke conversion hooks on request data.
 
 @mutates signal - Configured fetch provider may retain signal and register abort listeners.
 
 @example
 ```ts
 await postJson({ runtime, endpoint: '/search', body: { q: 'docs' } });
 ```
 */
async function postJson(
  {
    runtime,
    endpoint,
    body,
    signal,
  }: PostJsonOptions & {
    readonly runtime: ClientRuntime;
  },
): Promise<unknown> {
  /**
   API key validated for this endpoint.
   */
  const apiKey = apiKeyForEndpoint({
    ...(runtime.apiKey === undefined ? {} : { apiKey: runtime.apiKey, }),
    ...(runtime.credentialHint === undefined ? {} : { credentialHint: runtime.credentialHint, }),
    endpoint,
  },);
  /**
   Full Linkup request URL.
   */
  const requestUrl = `${runtime.baseUrl}${endpoint}`;
  /**
   Fetch response from Linkup.
   */
  const response = await sendRequest({
    runtime,
    endpoint,
    requestUrl,
    apiKey,
    body,
    ...(signal === undefined ? {} : { signal, }),
  },);
  /**
   Raw response text.
   */
  const responseText = await response.text();

  if (!response.ok)
    throw new Error(formatHttpError({
      endpoint,
      response,
      responseText,
    },),);

  return parseJsonResponse({
    endpoint,
    responseText,
  },);
}

/**
 Send fetch request and normalize abort or network failures.
 
 @param request - Runtime capability and Linkup request values.
 
 @returns fetch response
 
 @mutates request - `runtime.fetchImpl` may invoke provider behavior and retain request state;
   `JSON.stringify` may invoke hooks on `request.body`.
 */
async function sendRequest(
  request: {
    runtime: ClientRuntime;
    readonly endpoint: string;
    readonly requestUrl: string;
    readonly apiKey: string;
    body: unknown;
    signal?: AbortSignal;
  },
): Promise<Response> {
  /**
   Request values extracted after naming provider effect boundary.
   */
  const {
    runtime,
    endpoint,
    requestUrl,
    apiKey,
    body,
    signal,
  } = request;
  try {
    /**
     Fetch request init without undefined optional properties.
     */
    const requestInit: RequestInit = {
      method: HTTP_POST,
      headers: {
        [AUTHORIZATION_HEADER]: `Bearer ${apiKey}`,
        [CONTENT_TYPE_HEADER]: JSON_CONTENT_TYPE,
        [USER_AGENT_HEADER]: USER_AGENT_VALUE,
      },
      body: JSON.stringify(body,),
      ...(signal === undefined ? {} : { signal, }),
    };
    return await runtime.fetchImpl(
      requestUrl,
      requestInit,
    );
  }
  catch (error: unknown) {
    if (isAbortError({
      error,
      ...(signal === undefined ? {} : { signal, }),
    },))
      throw new Error(
        `Linkup ${endpoint} request aborted`,
        { cause: error, },
      );
    throw new Error(
      `Linkup ${endpoint} request failed: ${errorMessage(error,)}`,
      { cause: error, },
    );
  }
}

/**
 Return API key or throw endpoint-specific missing-key error.
 
 @param apiKey - optional configured API key
 
 @param endpoint - endpoint path for diagnostics
 
 @returns configured API key
 */
function apiKeyForEndpoint(
  {
    apiKey,
    endpoint,
    credentialHint,
  }: {
    readonly apiKey?: string;
    readonly endpoint: string;
    readonly credentialHint?: string;
  },
): string {
  if ((apiKey === undefined) || (apiKey.trim() === ''))
    throw new Error(
      `Linkup ${endpoint} missing API key. Set LINKUP_API_KEY in the environment or add it to .env.local.json.${credentialHint === undefined ? '' : ` Credential store: ${credentialHint}`}`,
    );
  return apiKey;
}

//endregion Request helpers

//region Response helpers

/**
 Parse successful Linkup JSON response text.
 
 @param endpoint - endpoint path for diagnostics
 
 @param responseText - raw response text
 
 @returns parsed JSON response
 
 @throws when response is not valid JSON
 */
function parseJsonResponse(
  {
    endpoint,
    responseText,
  }: {
    readonly endpoint: string;
    readonly responseText: string;
  },
): unknown {
  try {
    return JSON.parse(responseText,) as unknown;
  }
  catch (error: unknown) {
    throw new Error(
      `Linkup ${endpoint} returned invalid JSON response: ${errorMessage(error,)}`,
      { cause: error, },
    );
  }
}

/**
 Format non-2xx HTTP failures without exposing request secrets.
 
 @param endpoint - endpoint path for diagnostics
 
 @param response - Linkup response metadata
 
 @param responseText - raw response text
 
 @returns safe error message
 */
function formatHttpError(
  {
    endpoint,
    response,
    responseText,
  }: {
    readonly endpoint: string;
    readonly response: Response;
    readonly responseText: string;
  },
): string {
  /**
   Status text with leading space, when present.
   */
  const statusText = response.statusText
    .trim()
    === ''
    ? ''
    : ` ${response.statusText}`;
  /**
   Linkup error message parsed from response body, when present.
   */
  const linkupMessage = extractLinkupErrorMessage(responseText,);
  /**
   Linkup message suffix.
   */
  const messageSuffix = linkupMessage.found
    ? `: ${linkupMessage.message}`
    : '';
  return `Linkup ${endpoint} failed with HTTP ${String(response.status,)}${statusText}${messageSuffix}`;
}

/**
 Extract Linkup error message from a JSON error body.
 
 @param responseText - raw error response text
 
 @returns extraction result
 */
function extractLinkupErrorMessage(responseText: string,): ExtractedLinkupErrorMessage {
  /**
   Logger tagged for this extraction call.
   */
  const innerL = tagged({
    tag: extractLinkupErrorMessage.name,
    l,
  },);
  try {
    /**
     Parsed error response.
     */
    const parsed = JSON.parse(responseText,) as unknown;
    if (!isRecord(parsed,))
      return { found: false, };
    /**
     Linkup error payload, when present.
     */
    const { error, } = parsed;
    if (!isRecord(error,))
      return { found: false, };
    if ((typeof error.message) !== 'string')
      return { found: false, };
    return {
      found: true,
      message: error.message,
    };
  }
  catch (error: unknown) {
    innerL.debug(`ignoring unparsable Linkup error response body: ${String(error,)}`,);
    return { found: false, };
  }
}

/**
 Return whether fetch failed because the request was aborted.
 
 @param error - thrown fetch error
 
 @param signal - optional abort signal
 
 @returns whether failure is an abort
 */
function isAbortError(
  {
    error,
    signal,
  }: {
    readonly error: unknown;
    readonly signal?: AbortSignal;
  },
): boolean {
  if (signal?.aborted === true)
    return true;
  return (Error.isError(error,))
    && (error.name === ABORT_ERROR_NAME);
}

/**
 Return whether value is a non-null object record.
 
 @param value - unknown value
 
 @returns whether value can be read by string keys
 */
function isRecord(value: unknown,): value is Record<string, unknown> {
  return (value !== null)
    && ((typeof value) === 'object')
    && (!Array.isArray(value,));
}

//endregion Response helpers

export {
  postJson,
};
