/**
 Thin wrapper around `fetch` for calling the JSON API endpoints defined
 in `server/api/tasks.ts` and `server/api/timer.ts`.
 
 Automatically sets `Content-Type: application/json` and shows a toast on error.
 */
import { HTTP_NO_CONTENT, } from '@monochromatic-dev/const-core/ts';

import { showToast, } from '../component/toast-message.ts';

export { showToast, } from '../component/toast-message.ts';

/**
 Subset of fetch request configuration accepted by {@link api}.
 
 Deeply readonly and narrower than `RequestInit`,
 which carries mutable nested capabilities this wrapper never consumes.
 Covers exactly the fields callers use:
 a method,
 a JSON-string body,
 and optional extra headers.
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
 Sends a fetch request to a JSON API endpoint with standard headers and error handling.
 
 @param path - API endpoint path
 
 @param options - Optional fetch request configuration
 
 @returns Parsed JSON response body, or undefined for 204 responses
 
 @example
 ```ts
 const task = await api<Task>({ path: '/api/tasks/uuid-123' });
 await api({ path: '/api/tasks/uuid-123/complete', options: { method: 'POST' } });
 ```
 */
export async function api<TResponse = unknown,>(
  {
    path,
    options,
  }: {
    readonly path: string;
    readonly options?: ApiRequestOptions;
  },
): Promise<TResponse> {
  /**
   Combined header set; starts with the JSON content type and absorbs any caller-supplied headers.
   */
  const mergedHeaders = new Headers({ 'Content-Type': 'application/json', },);
  if (options?.headers
    !== undefined) {
    /**
     Caller-supplied headers normalised through the `Headers` ctor before merging.
     */
    const extra = new Headers({ ...options.headers, },);
    extra.forEach(function applyHeader(
      value: string,
      key: string,
    ): void {
      mergedHeaders.set(
        key,
        value,
      );
    },);
  }
  /**
   Raw fetch response shared by both the error and success paths below.
   */
  const response = await fetch(
    path,
    {
      method: options?.method ?? 'GET',
      body: options?.body ?? null,
      headers: mergedHeaders,
    },
  );

  if (!response.ok) {
    /**
     Parsed body or fallback object; assigned in `try`, defaulted in `catch`.
     */
    let error: unknown = undefined;
    try {
      error = await response.json();
    }
    catch (parseError) {
      console.error(
        'Parsing error response body failed:',
        parseError,
      );
      error = { error: 'Request failed', };
    }
    /**
     Human-readable error surfaced both via toast and the thrown `Error`.
     */
    const message = (
      ((typeof error) === 'object')
        && (error !== null)
        && ('error' in error)
        && ((typeof error.error) === 'string')
    )
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
