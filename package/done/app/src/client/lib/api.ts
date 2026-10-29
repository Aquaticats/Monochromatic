/**
 Thin wrapper around `fetch` for calling the JSON API endpoints defined
 in `server/api/tasks.ts` and `server/api/timer.ts`.
 
 Automatically sets `Content-Type: application/json` and shows a toast on error.
 */
import { HTTP_NO_CONTENT, } from '@monochromatic-dev/const-core/ts';

import { showToast, } from '../component/toast-message.ts';

export { showToast, } from '../component/toast-message.ts';

/**
 Request configuration accepted by {@link api}; a readonly subset of `RequestInit`.
 */
export type ApiRequestOptions = {
  /**
   HTTP method, e.g. `"POST"`, `"PUT"`, `"DELETE"`.
   */
  readonly method?: string;
  /**
   Request body; callers serialize JSON to a string.
   */
  readonly body?: string;
  /**
   Extra request headers merged over the default JSON content type.
   */
  readonly headers?: Readonly<Record<string, string>>;
};

/**
 Sends a fetch request to a JSON API endpoint with standard headers and
 error handling (shows a toast via {@link showToast} on failure).
 
 @param path - API endpoint path
 
 @param options - Optional fetch request configuration
 
 @returns Parsed JSON response body, or undefined for 204 responses
 
 @example
 ```ts
 const task = await api<Task>({ path: '/api/tasks/abc-123', });
 await api({ path: '/api/tasks', options: { method: 'POST', body: JSON.stringify({ title: 'New', }), }, });
 ```
 */
export async function api<TResponse = unknown,>({
  path,
  options,
}: {
  readonly path: string;
  readonly options?: ApiRequestOptions;
},): Promise<TResponse> {
  /**
   Base headers merged with any caller-supplied overrides below.
   */
  const headers = new Headers({ 'Content-Type': 'application/json', },);
  if (options?.headers
    !== undefined) {
    new Headers({ ...options.headers, },).forEach(function applyHeader(
      value,
      key,
    ) {
      headers.set(
        key,
        value,
      );
    },);
  }
  /**
   Network response; status checked before the body is parsed.
   */
  const response = await fetch(
    path,
    {
      method: options?.method ?? 'GET',
      body: options?.body ?? null,
      headers,
    },
  );

  if (!response.ok) {
    /**
     Error payload (possibly invalid JSON, hence the catch).
     */
    let error: unknown = undefined;
    try {
      error = await response.json();
    }
    catch (errorBodyParseError: unknown) {
      // Error response body was not valid JSON; log it and use a generic payload.
      console.error(
        'api could not parse error response body as JSON:',
        errorBodyParseError,
      );
      error = { error: 'Request failed', };
    }
    /**
     Surface message extracted from `error.error` when present, otherwise the generic fallback.
     */
    const message = ((typeof error) === 'object')
      && (error !== null)
      && ('error' in error)
      && ((typeof error.error) === 'string')
      ? error.error
      : 'Request failed';
    showToast(message,);
    throw new Error(message,);
  }

  if (response.status
    === HTTP_NO_CONTENT) {
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- 204 responses have no body; caller expects TResponse
    return undefined as TResponse;
  }

  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- response JSON matches the TResponse shape by API contract
  return (await response.json()) as TResponse;
}
