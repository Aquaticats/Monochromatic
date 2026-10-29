/**
 * S3-compatible storage adapter implementing the {@link Storage} interface.
 *
 * Speaks the standard AWS S3 path-style HTTP API. Works against:
 *
 * - Cloudflare R2 (production deployment target)
 * - Garage (stress-test container)
 * - SeaweedFS (documented fallback)
 *
 * Signing is delegated to a pluggable client (`S3FetchClient`) whose
 * `fetch` method matches `aws4fetch`'s `AwsClient.fetch`. Tests pass a
 * stub; production wires `new AwsClient({...})` from `aws4fetch`.
 *
 * Path style: `${endpoint}/${bucket}/${key}`. Keys may contain `/`:
 * each segment is `encodeURIComponent`d so reserved characters
 * (`?`, `&`, `#`, `+`, etc.) round-trip safely.
 */

import {
  HTTP_NOT_FOUND,
  HTTP_OK,
} from '@monochromatic-dev/const-core';
import pLimit from 'p-limit';

import type {
  Storage,
  StoragePutItem,
} from './adapter.ts';

/**
 * HTTP client used to issue signed S3 requests.
 */
export type S3FetchClient = {
  /**
   * Issues an HTTP request against an S3-compatible endpoint with AWS
   * Signature V4 signing applied. Mirrors `aws4fetch`'s `AwsClient.fetch`.
   *
   * @param input - request URL or `Request` object
   *
   * @param init - request init (method, body, headers, etc.)
   *
   * @returns the HTTP response
   */
  fetch(
    input: string | URL | Request,
    init?: RequestInit,
  ): Promise<Response>;
};

/**
 * Options accepted by {@link createS3Storage}.
 */
export type S3StorageOptions = {
  /**
   * Signed-fetch client (typically `new AwsClient(...)` from `aws4fetch`).
   */
  readonly client: S3FetchClient;

  /**
   * S3 endpoint base URL, no trailing slash.
   */
  readonly endpoint: string;

  /**
   * S3 bucket name. Path-style URL: `${endpoint}/${bucket}/${key}`.
   */
  readonly bucket: string;

  /**
   * Concurrency limit for `putBatch`. Defaults to 64.
   */
  readonly putBatchConcurrency?: number;
};

/**
 * Default concurrency for `putBatch` parallel writes.
 */
const DEFAULT_PUT_BATCH_CONCURRENCY = 64;

/**
 * ListObjectsV2 page size cap (S3 max).
 */
const LIST_PAGE_SIZE = 1_000;

/**
 * HTTP status code: redirect threshold (300 and above).
 */
const HTTP_REDIRECT = 300;

/**
 * Encodes an S3 key for use in a URL path. Slashes (`/`) are preserved
 * as path separators; every other character is `encodeURIComponent`d so
 * reserved query-string characters do not change the URL semantics.
 *
 * @param key - storage key
 *
 * @returns URL-encoded key suitable for path interpolation
 *
 * @example
 * ```ts
 * encodeS3Key('issues/r1/i1/detail');
 * // => 'issues/r1/i1/detail'
 * encodeS3Key('issues/r1/with space/detail');
 * // => 'issues/r1/with%20space/detail'
 * ```
 */
export function encodeS3Key(key: string,): string {
  return key
    .split('/',)
    .map(function encodeSegment(segment,) {
      return encodeURIComponent(segment,);
    },)
    .join('/',);
}

/**
 * Builds the full HTTP URL for an S3 object operation.
 *
 * @param row - URL inputs
 *
 * @returns absolute URL for the object
 *
 * @example
 * ```ts
 * objectUrl({ endpoint: 'http://localhost:3900', bucket: 'fragments', key: 'a/b' });
 * // => 'http://localhost:3900/fragments/a/b'
 * ```
 */
function objectUrl(row: {
  readonly endpoint: string;
  readonly bucket: string;
  readonly key: string;
},): string {
  return `${row.endpoint}/${row.bucket}/${encodeS3Key(row.key,)}`;
}

/**
 * Reads the `<Key>` values from a ListObjectsV2 response body.
 *
 * Uses a non-greedy global regex; the response shape is fixed (each
 * `<Contents>` block has exactly one `<Key>` first child) so a tiny
 * regex parser is sufficient and avoids pulling in a DOM parser.
 *
 * @param xml - response body as a string
 *
 * @returns extracted keys in document order
 *
 * @example
 * ```ts
 * parseKeys('<Contents><Key>a</Key></Contents>');
 * // => ['a']
 * ```
 */
function parseKeys(xml: string,): string[] {
  /**
   * Accumulator for the matched `<Key>` values in document order.
   */
  const keys: string[] = [];
  /* oxlint-disable no-restricted-syntax/no-regex -- Streaming XML extractor over an S3 ListObjectsV2 response body bounded by `maxKeys`; non-greedy quantifier with no alternation prevents catastrophic backtracking. */
  /**
   * Pattern matching `<Key>...</Key>` non-greedily so adjacent tags do not merge.
   */
  const pattern = /<Key>(.+?)<\/Key>/gu;
  /* oxlint-enable no-restricted-syntax/no-regex */
  for (const match of xml.matchAll(pattern,)) {
    if (match[1]
      !== undefined)
      keys.push(decodeXmlEntities(match[1],),);
  }
  return keys;
}

/**
 * Reads `<NextContinuationToken>` from a ListObjectsV2 response body.
 *
 * @param xml - response body as a string
 *
 * @returns the continuation token, or `undefined` when the response is final
 *
 * @example
 * ```ts
 * parseContinuationToken('<IsTruncated>true</IsTruncated><NextContinuationToken>x</NextContinuationToken>');
 * // => 'x'
 * ```
 */
function parseContinuationToken(xml: string,): string | undefined {
  /* oxlint-disable no-restricted-syntax/no-regex -- Extracts S3 pagination control fields from a ListObjectsV2 response body bounded by AWS; non-greedy single-capture pattern with no alternation prevents catastrophic backtracking. */
  /**
   * IsTruncated flag; absent or false means the response is the final page.
   */
  const truncated = /<IsTruncated>(.+?)<\/IsTruncated>/u.exec(xml,);
  /* oxlint-enable no-restricted-syntax/no-regex */
  if (truncated?.[1]
    !== 'true')
    return undefined;
  /* oxlint-disable no-restricted-syntax/no-regex -- Extracts S3 pagination control fields from a ListObjectsV2 response body bounded by AWS; non-greedy single-capture pattern with no alternation prevents catastrophic backtracking. */
  /**
   * Raw continuation token; XML-escaped values are decoded before returning.
   */
  const token = /<NextContinuationToken>(.+?)<\/NextContinuationToken>/u.exec(xml,);
  /* oxlint-enable no-restricted-syntax/no-regex */
  return token?.[1]
    === undefined ? undefined : decodeXmlEntities(token[1],);
}

/**
 * Decodes the XML entity references S3 emits in `<Key>` payloads.
 *
 * @param value - encoded XML text
 *
 * @returns decoded text
 *
 * @example
 * ```ts
 * decodeXmlEntities('a&amp;b');
 * // => 'a&b'
 * ```
 */
function decodeXmlEntities(value: string,): string {
  return value
    .replaceAll(
      '&amp;',
      '&',
    )
    .replaceAll(
      '&lt;',
      '<',
    )
    .replaceAll(
      '&gt;',
      '>',
    )
    .replaceAll(
      '&quot;',
      '"',
    )
    .replaceAll(
      '&apos;',
      "'",
    );
}

/**
 * Throws when an S3 response is non-2xx (and not 404 for GET/HEAD).
 *
 * @param row - response context
 *
 * @example
 * ```ts
 * await throwOnError({ response, operation: 'put', key: 'a/b' });
 * ```
 */
async function throwOnError(row: {
  readonly response: Response;
  readonly operation: string;
  readonly key: string;
},): Promise<void> {
  if ((row.response
    .status
    >= HTTP_OK) && (row.response
      .status
      < HTTP_REDIRECT))
    return;
  /**
   * Response body included verbatim in the thrown error for diagnostics.
   */
  const body = await row.response
    .text();
  throw new Error(
    `S3 ${row.operation} ${row.key} failed: ${
      String(row.response
        .status,)
    } ${row.response
      .statusText} ${body}`,
  );
}

/**
 * Issues a single ListObjectsV2 page request.
 *
 * @param row - list inputs
 *
 * @returns parsed `(keys, nextToken)` pair
 *
 * @example
 * ```ts
 * const page = await listOnePage({
 *   client,
 *   endpoint: 'http://localhost:3900',
 *   bucket: 'b',
 *   prefix: 'issues/',
 *   continuationToken: undefined,
 * });
 * ```
 */
async function listOnePage(row: {
  readonly client: S3FetchClient;
  readonly endpoint: string;
  readonly bucket: string;
  readonly prefix: string;
  readonly continuationToken: string | undefined;
},): Promise<{
  keys: string[];
  nextToken: string | undefined;
}> {
  /**
   * ListObjectsV2 query parameters built into the request URL below.
   */
  const params = new URLSearchParams();
  params.set(
    'list-type',
    '2',
  );
  params.set(
    'prefix',
    row.prefix,
  );
  params.set(
    'max-keys',
    String(LIST_PAGE_SIZE,),
  );
  if (row.continuationToken
    !== undefined) {
    params.set(
      'continuation-token',
      row.continuationToken,
    );
  }
  /**
   * Fully qualified ListObjectsV2 URL with all query parameters set.
   */
  const url = `${row.endpoint}/${row.bucket}?${params.toString()}`;
  /**
   * Signed HTTP response with the raw XML page body.
   */
  const response = await row.client
    .fetch(
    url,
    { method: 'GET', },
  );
  await throwOnError({
    response,
    operation: 'list',
    key: row.prefix,
  },);
  /**
   * Response body parsed below by `parseKeys`/`parseContinuationToken`.
   */
  const xml = await response.text();
  return {
    keys: parseKeys(xml,),
    nextToken: parseContinuationToken(xml,),
  };
}

/**
 * Creates an S3-compatible Storage adapter.
 *
 * @param options - adapter configuration
 *
 * @returns a {@link Storage} bound to the given bucket
 *
 * @example
 * ```ts
 * import { AwsClient } from 'aws4fetch';
 *
 * const aws = new AwsClient({
 *   accessKeyId: process.env.AWS_ACCESS_KEY_ID!,
 *   secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY!,
 *   region: 'auto',
 *   service: 's3',
 * });
 *
 * const storage = createS3Storage({
 *   client: aws,
 *   endpoint: 'https://account.r2.cloudflarestorage.com',
 *   bucket: 'fragments',
 * });
 * ```
 */
export function createS3Storage(options: S3StorageOptions,): Storage {
  /**
   * Concurrency limiter shared across every batch operation on the adapter.
   */
  const limit = pLimit(options.putBatchConcurrency
    ?? DEFAULT_PUT_BATCH_CONCURRENCY,);

  /**
   * Single PUT to the bucket.
   *
   * @param key - storage key
   *
   * @param body - bytes to upload
   *
   * @example
   * ```ts
   * await putOne({ key: 'a/b', body: new Uint8Array(...) });
   * ```
   */
  async function putOne(
    {
      key,
      body,
    }: {
      readonly key: string;
      readonly body: Uint8Array;
    },
  ): Promise<void> {
    /**
     * Fully qualified S3 object URL for the PUT request.
     */
    const url = objectUrl({
      endpoint: options.endpoint,
      bucket: options.bucket,
      key,
    },);
    /**
     * Fresh buffer copy ensures the body satisfies `Uint8Array<ArrayBuffer>`.
     */
    const reqBody = new Uint8Array(body,);
    /**
     * Signed HTTP response from the PUT operation.
     */
    const response = await options.client
      .fetch(
      url,
      {
        method: 'PUT',
        body: reqBody,
      },
    );
    await throwOnError({
      response,
      operation: 'put',
      key,
    },);
  }

  /**
   * Parallel PUTs of every item, bounded by `putBatchConcurrency`.
   *
   * @param items - put operations
   *
   * @example
   * ```ts
   * await storage.putBatch([{key: 'a', body: ...}, {key: 'b', body: ...}]);
   * ```
   */
  async function putBatch(items: readonly StoragePutItem[],): Promise<void> {
    await Promise.all(
      items.map(function schedule(item,) {
        return limit(function performPut() {
          return putOne({
            key: item.key,
            body: item.body,
          },);
        },);
      },),
    );
  }

  /**
   * Single GET from the bucket.
   *
   * @param key - storage key
   *
   * @returns body bytes, or `undefined` for 404
   *
   * @example
   * ```ts
   * const body = await storage.get('a/b');
   * ```
   */
  async function get(key: string,): Promise<Uint8Array | undefined> {
    /**
     * Fully qualified S3 object URL for the GET request.
     */
    const url = objectUrl({
      endpoint: options.endpoint,
      bucket: options.bucket,
      key,
    },);
    /**
     * Signed HTTP response from the GET operation.
     */
    const response = await options.client
      .fetch(
      url,
      { method: 'GET', },
    );
    if (response.status
      === HTTP_NOT_FOUND) {
      // Drain the body so connection reuse works.
      await response.arrayBuffer();
      return undefined;
    }
    await throwOnError({
      response,
      operation: 'get',
      key,
    },);
    /**
     * ArrayBuffer body returned wrapped in a Uint8Array view.
     */
    const buffer = await response.arrayBuffer();
    return new Uint8Array(buffer,);
  }

  /**
   * Single DELETE in the bucket. Idempotent (404 is treated as success).
   *
   * @param key - storage key
   *
   * @example
   * ```ts
   * await storage.delete('a/b');
   * ```
   */
  async function deleteFn(key: string,): Promise<void> {
    /**
     * Fully qualified S3 object URL for the DELETE request.
     */
    const url = objectUrl({
      endpoint: options.endpoint,
      bucket: options.bucket,
      key,
    },);
    /**
     * Signed HTTP response from the DELETE operation.
     */
    const response = await options.client
      .fetch(
      url,
      { method: 'DELETE', },
    );
    if (response.status
      === HTTP_NOT_FOUND) {
      await response.arrayBuffer();
      return;
    }
    await throwOnError({
      response,
      operation: 'delete',
      key,
    },);
  }

  /**
   * Lists keys matching a prefix in lexicographic order. Pages through
   * 1000-key chunks until exhausted.
   *
   * @param prefix - prefix filter; empty string lists everything
   *
   * @returns sorted keys array
   *
   * @example
   * ```ts
   * const keys = await storage.list('issues/');
   * ```
   */
  async function list(prefix: string,): Promise<string[]> {
    /**
     * Keys accumulated across pages; scoped to an IIFE so the pagination cursor stays inside the loop.
     */
    const accumulated = await (async function paginate(): Promise<string[]> {
      /**
       * Keys accumulated across pages; sorted before returning.
       */
      const acc: string[] = [];
      /**
       * Continuation token threaded through pagination; undefined ends the loop.
       */
      let continuationToken: string | undefined = undefined;
      do {
        /* oxlint-disable no-await-in-loop -- pagination requires the previous token to fetch the next page */
        /**
         * Next ListObjectsV2 page: keys and the continuation token for the page after.
         */
        const page = await listOnePage({
          client: options.client,
          endpoint: options.endpoint,
          bucket: options.bucket,
          prefix,
          continuationToken,
        },);
        acc.push(...page.keys,);
        continuationToken = page.nextToken;
        /* oxlint-enable no-await-in-loop */
      }
      while (continuationToken !== undefined);
      return acc;
    })();
    return accumulated.toSorted(function compareAsc(
      a,
      b,
    ) {
      return a < b ? -1 : 1;
    },);
  }

  return {
    put(
      key: string,
      body: Uint8Array,
    ): Promise<void> {
      return putOne({
        key,
        body,
      },);
    },
    putBatch,
    get,
    delete: deleteFn,
    list,
  };
}
