/**
 Exa HTTP request helpers.
 
 @module
 */

import { caughtValueText as errorMessage, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  ABORT_ERROR_NAME,
  CONTENT_TYPE_HEADER,
  HTTP_POST,
  JSON_CONTENT_TYPE,
  USER_AGENT_HEADER,
  USER_AGENT_VALUE,
} from './client-constants.ts';
import type {
  ExaClientRuntime,
  ExaPostJsonOptions,
} from './search-fetch-types.ts';

/**
 Logger root for pi-search-fetch after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: exaLogger, },);
 ```
 */
const exaLogger = tagged({ tag: 'pi-search-fetch', },);

/**
 Header used by Exa API keys.
 */
const EXA_API_KEY_HEADER = 'x-api-key' as const;

/**
 Module logger.
 */
const l = tagged({
  tag: 'exa-http',
  l: exaLogger,
},);

/**
 POST JSON to Exa and parse JSON response.
 
 @param runtime - client runtime dependencies
 
 @param endpoint - endpoint path
 
 @param body - JSON request body
 
 @param signal - optional cancellation signal
 
 @returns parsed JSON response
 
 @mutates runtime - `sendExaRequest` invokes configured fetch provider capability.
 
 @mutates body - `JSON.stringify` may invoke conversion hooks on request data.
 
 @mutates signal - Configured fetch provider may retain signal and register abort listeners.
 
 @example
 ```ts
 await postExaJson({ runtime, endpoint: '/search', body: { query: 'docs' } });
 ```
 */
async function postExaJson(
  {
    runtime,
    endpoint,
    body,
    signal,
  }: ExaPostJsonOptions & {
    readonly runtime: ExaClientRuntime;
  },
): Promise<unknown> {
  /**
   API key validated for this endpoint.
   */
  const apiKey = exaApiKeyForEndpoint({
    ...(runtime.apiKey === undefined ? {} : { apiKey: runtime.apiKey, }),
    ...(runtime.credentialHint === undefined ? {} : { credentialHint: runtime.credentialHint, }),
    endpoint,
  },);
  /**
   Full Exa request URL.
   */
  const requestUrl = `${runtime.baseUrl}${endpoint}`;
  /**
   Fetch response from Exa.
   */
  const response = await sendExaRequest({
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
    throw new Error(formatExaHttpError({
      endpoint,
      response,
      responseText,
    },),);

  return parseExaJsonResponse({
    endpoint,
    responseText,
  },);
}

/**
 Send Exa fetch request and normalize abort or network failures.
 
 @param request - Runtime capability and Exa request values.
 
 @returns fetch response
 
 @mutates request - `runtime.fetchImpl` may invoke provider behavior and retain request state;
   `JSON.stringify` may invoke hooks on `request.body`.
 */
async function sendExaRequest(
  request: {
    runtime: ExaClientRuntime;
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
        [EXA_API_KEY_HEADER]: apiKey,
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
        `Exa ${endpoint} request aborted`,
        { cause: error, },
      );
    throw new Error(
      `Exa ${endpoint} request failed: ${errorMessage(error,)}`,
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
function exaApiKeyForEndpoint(
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
      `Exa ${endpoint} missing API key. Set EXA_API_KEY in the environment or add it to .env.local.json.${credentialHint === undefined ? '' : ` Credential store: ${credentialHint}`}`,
    );
  return apiKey;
}

/**
 Parse successful Exa JSON response text.
 
 @param endpoint - endpoint path for diagnostics
 
 @param responseText - raw response text
 
 @returns parsed JSON response
 */
function parseExaJsonResponse(
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
      `Exa ${endpoint} returned invalid JSON response: ${errorMessage(error,)}`,
      { cause: error, },
    );
  }
}

/**
 Format non-2xx Exa failures without exposing request secrets.
 
 @param endpoint - endpoint path for diagnostics
 
 @param response - Exa response metadata
 
 @param responseText - raw response text
 
 @returns safe error message
 */
function formatExaHttpError(
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
   Exa message suffix.
   */
  const messageSuffix = exaErrorMessage(responseText,);
  return `Exa ${endpoint} failed with HTTP ${String(response.status,)}${statusText}${messageSuffix}`;
}

/**
 Extract Exa error message suffix from JSON response body.
 
 @param responseText - raw error response text
 
 @returns safe suffix beginning with colon when present
 */
function exaErrorMessage(responseText: string,): string {
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
     Parsed Exa nested error.
     */
    const nestedError = parsed.error;
    if ((typeof nestedError.message) !== 'string')
      return '';
    return `: ${nestedError.message}`;
  }
  catch (error: unknown) {
    l.debug(`ignoring unparsable Exa error response body: ${String(error,)}`,);
    return '';
  }
}

/**
 Return whether fetch failed because request was aborted.
 
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

export { postExaJson, };
