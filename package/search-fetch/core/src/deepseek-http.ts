/**
 DeepSeek Tools API HTTP request helpers.
 
 @module
 */

import { caughtValueText as errorMessage, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  AUTHORIZATION_HEADER,
  CONTENT_TYPE_HEADER,
  HTTP_POST,
  JSON_CONTENT_TYPE,
  USER_AGENT_HEADER,
  USER_AGENT_VALUE,
} from './client-constants.ts';
import type {
  DeepseekClientRuntime,
  DeepseekPostJsonOptions,
} from './search-fetch-types.ts';

/**
 Logger root for pi-search-fetch after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: deepseekLogger, },);
 ```
 */
const deepseekLogger = tagged({ tag: 'pi-search-fetch', },);

/**
 Client-side upper bound on one DeepSeek Search or Fetch request.
 */
const DEEPSEEK_REQUEST_TIMEOUT_MS = 30_000;

/**
 Accepted response media type.
 */
const ACCEPT_HEADER = 'Accept' as const;

/**
 Module logger.
 */
const l = tagged({
  tag: 'deepseek-http',
  l: deepseekLogger,
},);

/**
 POST JSON to DeepSeek and parse the JSON response.
 
 @param runtime - client runtime dependencies
 
 @param endpoint - endpoint path
 
 @param body - JSON request body
 
 @param signal - optional caller cancellation signal
 
 @returns parsed JSON response
 
 @mutates runtime - `sendDeepseekRequest` invokes the configured fetch provider capability.
 
 @example
 ```ts
 await postDeepseekJson({ runtime, endpoint: '/v1/tools/search', body: { query: 'docs' } });
 ```
 */
async function postDeepseekJson(
  {
    runtime,
    endpoint,
    body,
    signal,
  }: DeepseekPostJsonOptions & {
    readonly runtime: DeepseekClientRuntime;
  },
): Promise<unknown> {
  /**
   API key validated for this endpoint.
   */
  const apiKey = deepseekApiKeyForEndpoint({
    ...(runtime.apiKey === undefined ? {} : { apiKey: runtime.apiKey, }),
    ...(runtime.credentialHint === undefined ? {} : { credentialHint: runtime.credentialHint, }),
    endpoint,
  },);
  /**
   Full DeepSeek request URL.
   */
  const requestUrl = `${runtime.baseUrl}${endpoint}`;
  /**
   Fetch response from DeepSeek.
   */
  const response = await sendDeepseekRequest({
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
    throw new Error(formatDeepseekHttpError({
      endpoint,
      response,
      responseText,
    },),);

  return parseDeepseekJsonResponse({
    endpoint,
    responseText,
  },);
}

/**
 Send one DeepSeek request and normalize abort, timeout, or network failures.
 
 @param request - runtime capability and DeepSeek request values
 
 @returns fetch response
 
 @mutates request - `runtime.fetchImpl` may invoke provider behavior and retain request state;
   `JSON.stringify` may invoke hooks on `request.body`.
 */
async function sendDeepseekRequest(
  request: {
    readonly runtime: DeepseekClientRuntime;
    readonly endpoint: string;
    readonly requestUrl: string;
    readonly apiKey: string;
    body: unknown;
    signal?: AbortSignal;
  },
): Promise<Response> {
  /**
   Request values extracted after naming the provider effect boundary.
   */
  const {
    runtime,
    endpoint,
    requestUrl,
    apiKey,
    body,
    signal,
  } = request;
  /**
   Local timeout signal bounding one request independently of caller cancellation.
   */
  const timeoutSignal = AbortSignal.timeout(DEEPSEEK_REQUEST_TIMEOUT_MS,);
  /**
   Request signal aborted by either the caller or the local timeout.
   */
  const requestSignal = signal === undefined
    ? timeoutSignal
    : AbortSignal.any([
      signal,
      timeoutSignal,
    ],);
  try {
    /**
     Fetch request init without undefined optional properties.
     */
    const requestInit: RequestInit = {
      method: HTTP_POST,
      headers: {
        [AUTHORIZATION_HEADER]: `Bearer ${apiKey}`,
        [CONTENT_TYPE_HEADER]: JSON_CONTENT_TYPE,
        [ACCEPT_HEADER]: JSON_CONTENT_TYPE,
        [USER_AGENT_HEADER]: USER_AGENT_VALUE,
      },
      body: JSON.stringify(body,),
      signal: requestSignal,
    };
    return await runtime.fetchImpl(
      requestUrl,
      requestInit,
    );
  }
  catch (error: unknown) {
    if (signal?.aborted === true)
      throw new Error(
        `DeepSeek ${endpoint} request aborted`,
        { cause: error, },
      );
    if (timeoutSignal.aborted)
      throw new Error(
        `DeepSeek ${endpoint} request timed out after ${String(DEEPSEEK_REQUEST_TIMEOUT_MS,)} ms`,
        { cause: error, },
      );
    throw new Error(
      `DeepSeek ${endpoint} request failed: ${errorMessage(error,)}`,
      { cause: error, },
    );
  }
}

/**
 Return the API key or throw an endpoint-specific missing-key error.
 
 @param apiKey - optional configured API key
 
 @param endpoint - endpoint path for diagnostics
 
 @param credentialHint - optional value-free store diagnostic
 
 @returns configured API key
 */
function deepseekApiKeyForEndpoint(
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
      `DeepSeek ${endpoint} missing API key. Set DEEPSEEK_API_KEY in the environment or add it to .env.local.json.${credentialHint === undefined ? '' : ` Credential store: ${credentialHint}`}`,
    );
  return apiKey;
}

/**
 Parse a successful DeepSeek JSON response text.
 
 @param endpoint - endpoint path for diagnostics
 
 @param responseText - raw response text
 
 @returns parsed JSON response
 */
function parseDeepseekJsonResponse(
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
      `DeepSeek ${endpoint} returned invalid JSON response: ${errorMessage(error,)}`,
      { cause: error, },
    );
  }
}

/**
 Format non-2xx DeepSeek failures without exposing request secrets.
 
 @param endpoint - endpoint path for diagnostics
 
 @param response - DeepSeek response metadata
 
 @param responseText - raw response text
 
 @returns safe error message
 */
function formatDeepseekHttpError(
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
   DeepSeek message suffix.
   */
  const messageSuffix = deepseekErrorMessage(responseText,);
  return `DeepSeek ${endpoint} failed with HTTP ${String(response.status,)}${statusText}${messageSuffix}`;
}

/**
 Extract a DeepSeek error message suffix from an OpenAI-compatible error body.
 
 @param responseText - raw error response text
 
 @returns safe suffix beginning with colon when present
 */
function deepseekErrorMessage(responseText: string,): string {
  try {
    /**
     Parsed error response.
     */
    const parsed = JSON.parse(responseText,) as unknown;
    if (!isRecord(parsed,))
      return '';
    if ((typeof parsed.message) === 'string')
      return `: ${parsed.message}`;
    if ((typeof parsed.error) === 'string')
      return `: ${parsed.error}`;
    if (!isRecord(parsed.error,))
      return '';
    /**
     Parsed nested error.
     */
    const nestedError = parsed.error;
    if ((typeof nestedError.message) !== 'string')
      return '';
    return `: ${nestedError.message}`;
  }
  catch (error: unknown) {
    l.debug(`ignoring unparsable DeepSeek error response body: ${String(error,)}`,);
    return '';
  }
}

/**
 Return whether a value is a non-null object record.
 
 @param value - unknown value
 
 @returns whether the value can be read by string keys
 */
function isRecord(value: unknown,): value is Record<string, unknown> {
  return (value !== null)
    && ((typeof value) === 'object')
    && (!Array.isArray(value,));
}

export {
  DEEPSEEK_REQUEST_TIMEOUT_MS,
  postDeepseekJson,
};
