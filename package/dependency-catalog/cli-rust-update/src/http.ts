/**
 HTTP access to crates.io: the identifying user agent, the error type, and
 the text and gunzip fetchers the registry client uses.

 @module
 */

import { Readable, } from 'node:stream';
import { createGunzip, } from 'node:zlib';

//region Constants

/**
 User agent crates.io asks clients to send; names the repository, not a person.
 */
export const USER_AGENT = 'Monochromatic rust-catalog-update (https://github.com/Aquaticats/Monochromatic)';

//endregion Constants

//region Errors

/**
 Thrown when crates.io or the sparse index cannot answer.

 @example
 ```ts
 throw new RegistryError({ url: 'https://index.crates.io/an/yh/anyhow', status: 503, });
 ```
 */
export class RegistryError extends Error {
  /**
   Requested URL.
   */
  readonly url: string;

  /**
   HTTP status, or 0 when the request never got a response.
   */
  readonly status: number;

  /**
   Seconds the server asked to wait (`Retry-After`), or 0 when it named none.
   */
  readonly retryAfterSeconds: number;

  /**
   Builds the error.

   @param url - requested URL

   @param status - HTTP status, 0 without a response

   @param retryAfterSeconds - `Retry-After` seconds, 0 when absent

   @param cause - underlying failure
   */
  constructor({
    url,
    status,
    retryAfterSeconds = 0,
    cause,
  }: {
    readonly url: string;
    readonly status: number;
    readonly retryAfterSeconds?: number;
    readonly cause?: unknown
  },) {
    super(
      status === 0
        ? `Could not reach ${url}; check the network, then rerun.`
        : `${url} answered HTTP ${String(status,)}; rerun later, or check https://status.crates.io.`,
      { cause, },
    );
    this.name = 'RegistryError';
    this.url = url;
    this.status = status;
    this.retryAfterSeconds = retryAfterSeconds;
  }
}

//endregion Errors

//region Fetch

/**
 Fetches a URL's body as text.

 @example
 ```ts
 const fetchText: FetchText = async (url) => 'body';
 ```
 */
export type FetchText = (url: string,) => Promise<string>;

/**
 Sends one GET with {@link USER_AGENT}.

 @param url - URL to fetch

 @returns response of any status

 @throws RegistryError with status 0 when no response arrives

 @example
 ```ts
 await fetchOrThrow('https://index.crates.io/it/oa/itoa');
 ```
 */
async function fetchOrThrow(url: string,): Promise<Response> {
  try {
    return await fetch(
      url,
      { headers: { 'user-agent': USER_AGENT, }, },
    );
  } catch (error) {
    throw new RegistryError({
      url,
      status: 0,
      cause: error,
    },);
  }
}

/**
 Default {@link FetchText} over the global `fetch`, sending {@link USER_AGENT}.

 @param url - URL to fetch

 @returns response body

 @throws RegistryError when the request fails or the status is not 2xx

 @example
 ```ts
 await fetchTextWithUserAgent('https://index.crates.io/it/oa/itoa');
 ```
 */
export async function fetchTextWithUserAgent(url: string,): Promise<string> {
  /**
   Response; a request that gets none becomes a RegistryError.
   */
  const response = await fetchOrThrow(url,);
  if (!response.ok) {
    /**
     `Retry-After` in seconds; an HTTP-date or a missing header counts as 0.
     */
    const retryAfter = Math.trunc(Number(response.headers
      .get('retry-after',)
      ?? '0',),);
    throw new RegistryError({
      url,
      status: response.status,
      retryAfterSeconds: Number.isNaN(retryAfter,) ? 0 : retryAfter,
    },);
  }
  return await response.text();
}

/**
 Streams a gzipped body, decompressed, as byte chunks.

 @example
 ```ts
 const fetchGunzipped: FetchGunzipped = async (url) => createReadStream('dump.tar.gz').pipe(createGunzip());
 ```
 */
export type FetchGunzipped = (url: string,) => Promise<AsyncIterable<Uint8Array>>;

/**
 Default {@link FetchGunzipped} over the global `fetch`, sending {@link USER_AGENT}.

 @param url - URL of a gzipped file

 @returns decompressed bytes

 @throws RegistryError when the request fails or the status is not 2xx

 @example
 ```ts
 await fetchGunzippedWithUserAgent(DATABASE_DUMP_URL);
 ```
 */
export async function fetchGunzippedWithUserAgent(url: string,): Promise<AsyncIterable<Uint8Array>> {
  /**
   Response; a request that gets none becomes a RegistryError.
   */
  const response = await fetchOrThrow(url,);
  if ((!response.ok) || (response.body === null))
    throw new RegistryError({
      url,
      status: response.status,
    },);
  return Readable.fromWeb(response.body,)
    .pipe(createGunzip(),);
}

//endregion Fetch
