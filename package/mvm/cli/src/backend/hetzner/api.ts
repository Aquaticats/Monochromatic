/**
 Hetzner Cloud HTTP request core: authenticated fetch, error mapping,
 pagination, and action polling. Typed resource helpers live in
 `./api-resources.ts`.
 
 Action polling uses the generic `GET /actions/{id}` because the
 resource-instance action endpoint is deprecated.
 
 @module
 */

import { wait, } from '@monochromatic-dev/module-async-time/ts';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { requireToken, } from './config.ts';
import type { HetznerAction, } from './types.ts';

/**
 Logger root for mvm after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

//region Constants and errors

/**
 Base URL for the Hetzner Cloud API.
 */
const API_BASE = 'https://api.hetzner.cloud/v1';

/**
 Items requested per page on list endpoints.
 */
const PER_PAGE = 50;

/**
 Delay between action-status polls.
 */
const POLL_INTERVAL_MS = 1_000;

/**
 Maximum time to wait for an action to leave the `running` state. Generous
 because snapshot (create_image) actions for a clone can take a few minutes.
 */
const ACTION_TIMEOUT_MS = 300_000;

/**
 HTTP status for an empty (no-body) response, returned by DELETE.
 */
const HTTP_NO_CONTENT = 204;

/**
 Error carrying the HTTP status and Hetzner error code so callers can branch
 (for example, falling back across locations on `resource_unavailable`).
 
 @example
 ```ts
 try { await createServer(...); }
 catch (err) { if (err instanceof HetznerApiError && err.code === 'resource_unavailable') retry(); }
 ```
 */
export class HetznerApiError extends Error {
  /**
   HTTP status code of the failed response.
   */
  readonly status: number;
  /**
   Hetzner error code (e.g. `resource_unavailable`, `not_found`).
   */
  readonly code: string;

  /**
   @param status - HTTP status code
   
   @param code - Hetzner error code
   
   @param message - human-readable error message
   */
  constructor(
    {
      status,
      code,
      message,
    }: {
      readonly status: number;
      readonly code: string;
      readonly message: string;
    },
  ) {
    super(message,);
    this.name = 'HetznerApiError';
    this.status = status;
    this.code = code;
  }
}

//endregion Constants and errors

//region Core request

/**
 Parses an error response body into a code and message, falling back to the
 HTTP status text when the body is absent or not JSON.
 
 @param res - non-ok fetch response
 
 @returns Hetzner error code and message
 
 @example
 ```ts
 const { code, message } = await parseApiError(res);
 ```
 */
async function parseApiError(
  res: Response,
): Promise<{
  readonly code: string;
  readonly message: string
}> {
  /**
   Logger scoped to this parse so non-JSON bodies are noted.
   */
  const rl = tagged({
    tag: parseApiError.name,
    l,
  },);
  try {
    /**
     Parsed error envelope; fields default below when absent.
     */
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- Hetzner API error envelope
    const errBody = await res.json() as {
      readonly error?: {
        readonly code?: string;
        readonly message?: string;
      };
    };
    return {
      code: errBody.error
        ?.code
        ?? 'unknown',
      message: errBody.error
        ?.message
        ?? res.statusText,
    };
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    rl.debug('error response body was not JSON',);
    return {
      code: 'unknown',
      message: res.statusText,
    };
  }
}

/**
 Issues an authenticated JSON request to the Hetzner Cloud API.
 
 @param body - optional JSON request body
 
 @param method - HTTP method
 
 @param path - path under the API base, beginning with `/`
 
 @returns parsed JSON response, or `undefined` for empty (204) responses
 
 @throws {@link HetznerApiError} when the response status is not ok
 
 @mutates body - `JSON.stringify` may invoke `toJSON`, getters, or proxy traps.
 
 @example
 ```ts
 const { servers } = await hcloud<{ servers: HetznerServer[] }>({ method: 'GET', path: '/servers' });
 ```
 */
export async function hcloud<T>(
  {
    body,
    method,
    path,
  }: {
    readonly body?: unknown;
    readonly method: string;
    readonly path: string;
  },
): Promise<T> {
  /**
   Logger scoped to this request so debug lines are namespaced.
   */
  const rl = tagged({
    tag: hcloud.name,
    l,
  },);
  /**
   Base headers; `Content-Type` is added only when a body is sent.
   */
  const headers: Record<string, string> = {
    Authorization: `Bearer ${requireToken()}`,
  };
  if (body !== undefined) {
    headers['Content-Type'] = 'application/json';
  }
  rl.debug(`${method} ${path}`,);

  /**
   Raw HTTP response; inspected for ok-ness before parsing.
   */
  const res = await fetch(
    `${API_BASE}${path}`,
    {
      method,
      headers,
      ...(body !== undefined ? { body: JSON.stringify(body,), } : {}),
    },
  );

  if (!res.ok) {
    /**
     Parsed error code and message from the body or status text.
     */
    const {
      code,
      message,
    } = await parseApiError(res,);
    throw new HetznerApiError({
      code,
      message,
      status: res.status,
    },);
  }

  if (res.status === HTTP_NO_CONTENT) {
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- 204 No Content has no body
    return undefined as T;
  }
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- trusted Hetzner API JSON response
  return await res.json() as T;
}

//endregion Core request

//region Pagination

/**
 Fetches every page of a list endpoint, accumulating one resource array.
 
 @param key - response envelope key holding the resource array
 
 @param path - list path with any filters already applied (no page params)
 
 @returns all items across pages, in order
 
 @example
 ```ts
 const servers = await fetchAllPages<HetznerServer>({ path: '/servers', key: 'servers' });
 ```
 */
export function fetchAllPages<T>(
  {
    key,
    path,
  }: {
    readonly key: string;
    readonly path: string;
  },
): Promise<readonly T[]> {
  /**
   Separator for appending pagination params, accounting for existing query.
   */
  const sep = path.includes('?',) ? '&' : '?';
  return (async function paginate(): Promise<readonly T[]> {
    /**
     Accumulated items across every page.
     */
    const items: T[] = [];
    /**
     Pagination cursor and continuation state; the API advances the page
     number and explicitly ends traversal when no next page exists.
     */
    const pagination = {
      page: 1,
      hasNextPage: true,
    };
    while (pagination.hasNextPage) {
      /**
       One page of results plus pagination metadata.
       */
      // oxlint-disable-next-line no-await-in-loop -- deliberate sequential pagination
      const body = await hcloud<
        & { readonly meta: { readonly pagination: { readonly next_page?: number; }; }; }
          & Readonly<Record<string, readonly T[]>>
        >({
          method: 'GET',
          path: `${path}${sep}page=${String(pagination.page,)}&per_page=${String(PER_PAGE,)}`,
        },);
      items.push(...(body[key] ?? []),);
      /**
       Next page number; absent (null on the wire) on the last page.
       */
      const next = body.meta
        .pagination
        .next_page;
      if ((typeof next) === 'number')
        pagination.page = next;
      else
        pagination.hasNextPage = false;
    }
    return items;
  })();
}

//endregion Pagination

//region Action polling

/**
 Polls the generic action endpoint until the action succeeds.
 
 @param id - action id from a mutating response
 
 @throws Error when the action ends in `error` or does not finish before the timeout
 
 @example
 ```ts
 const { action } = await createServer(...);
 await waitForAction({ id: action.id });
 ```
 */
export async function waitForAction({ id, }: { readonly id: number; },): Promise<void> {
  /**
   Deadline after which polling gives up.
   */
  const deadline = Date.now() + ACTION_TIMEOUT_MS;
  while (Date.now() < deadline) {
    /**
     Latest action state for this id.
     */
    // oxlint-disable-next-line no-await-in-loop -- deliberate serial polling
    const { action, } = await hcloud<{ readonly action: HetznerAction; }>({
      method: 'GET',
      path: `/actions/${String(id,)}`,
    },);
    if (action.status === 'success') {
      return;
    }
    if (action.status === 'error') {
      throw new Error(
        `Hetzner action ${String(id,)} failed: ${action.error
          ?.message
          ?? 'unknown error'}`,
      );
    }
    // oxlint-disable-next-line no-await-in-loop -- deliberate serial polling
    await wait(POLL_INTERVAL_MS,);
  }
  throw new Error(
    `Hetzner action ${String(id,)} did not finish within ${String(ACTION_TIMEOUT_MS,)}ms`,
  );
}

//endregion Action polling
