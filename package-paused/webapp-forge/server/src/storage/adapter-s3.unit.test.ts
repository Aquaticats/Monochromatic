/**
 * Unit tests for the S3 storage adapter.
 *
 * The tests inject a fake `S3FetchClient` whose `fetch` method consults
 * an in-memory `Map<key, body>` plus a recorded request log. Real network
 * I/O is out of scope here; those checks belong in stress tests against
 * Garage (Phase 2 task #17).
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';

import {
  createS3Storage,
  encodeS3Key,
  type S3FetchClient,
} from './adapter-s3.ts';

/** HTTP 200 OK status code. */
const STATUS_OK = 200;

/** HTTP 204 No Content status code. */
const STATUS_NO_CONTENT = 204;

/** HTTP 404 Not Found status code. */
const STATUS_NOT_FOUND = 404;

/** Fixed endpoint for fake S3 requests. */
const FAKE_ENDPOINT = 'http://localhost:3900';

/** Fixed bucket name for fake S3 requests. */
const FAKE_BUCKET = 'fragments';

/** Status of the request log entry recorded by the fake client. */
type RecordedRequest = {
  readonly method: string;
  readonly url: string;
  readonly bodyByteLength: number;
};

/** Captured state of a fake S3 server. */
type FakeServer = {
  readonly client: S3FetchClient;
  readonly store: Map<string, Uint8Array>;
  readonly log: RecordedRequest[];
};

/**
 * Resolves the various `fetch` input shapes to a string URL.
 *
 * @param input - URL string, URL instance, or Request
 *
 * @returns absolute URL string
 *
 * @example
 * ```ts
 * resolveUrl(new URL('http://x/y')); // 'http://x/y'
 * ```
 */
function resolveUrl(input: string | URL | Request,): string {
  if ((typeof input) === 'string')
    return input;
  if (input instanceof URL)
    return input.href;
  return input.url;
}

/**
 * Builds a fake S3 server backed by an in-memory `Map`. The returned
 * client matches the `S3FetchClient` shape and routes PUT/GET/DELETE/list
 * to the map.
 *
 * @returns fake-server bundle: `client`, in-memory `store`, request `log`
 *
 * @example
 * ```ts
 * const fake = createFakeServer();
 * const storage = createS3Storage({ client: fake.client, ... });
 * ```
 */
function createFakeServer(): FakeServer {
  const store = new Map<string, Uint8Array>();
  const log: RecordedRequest[] = [];

  return {
    client: {
      async fetch(
        input: string | URL | Request,
        init?: RequestInit,
      ): Promise<Response> {
        const url = resolveUrl(input,);
        const method = init?.method ?? 'GET';
        const body = init?.body;
        const bodyByteLength = body instanceof Uint8Array ? body.byteLength : 0;
        log.push({
          method,
          url,
          bodyByteLength,
        },);

        const u = new URL(url,);
        const path = u.pathname.replace(`/${FAKE_BUCKET}/`, '',);
        const isList = (u.pathname === `/${FAKE_BUCKET}`)
          && (u.searchParams.get('list-type',) === '2');

        if (isList) {
          const prefix = u.searchParams.get('prefix',) ?? '';
          const matched = [...store.keys(),]
            .filter(function hasPrefix(k,) {
              return k.startsWith(prefix,);
            },)
            .toSorted(function compareAsc(
              a,
              b,
            ) {
              return a < b ? -1 : 1;
            },);
          const xmlContents = matched
            .map(function entry(k,) {
              return `<Contents><Key>${k}</Key></Contents>`;
            },)
            .join('',);
          const xml =
            `<?xml version="1.0"?><ListBucketResult><IsTruncated>false</IsTruncated>${xmlContents}</ListBucketResult>`;
          return new Response(xml, {
            status: STATUS_OK,
            headers: { 'content-type': 'application/xml', },
          },);
        }

        const decodedKey = decodeURIComponent(path,);

        if (method === 'PUT') {
          if (body instanceof Uint8Array) {
            store.set(
              decodedKey,
              new Uint8Array(body,),
            );
          }
          return new Response(null, { status: STATUS_OK, },);
        }

        if (method === 'GET') {
          const value = store.get(decodedKey,);
          if (value === undefined)
            return new Response(null, { status: STATUS_NOT_FOUND, },);
          const respBody = new Uint8Array(value,);
          return new Response(
            respBody,
            { status: STATUS_OK, },
          );
        }

        if (method === 'DELETE') {
          store.delete(decodedKey,);
          return new Response(null, { status: STATUS_NO_CONTENT, },);
        }

        return new Response(null, { status: STATUS_NOT_FOUND, },);
      },
    },
    store,
    log,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: encodeS3Key.name,
      concurrency: 1,
      children: [
        it({
          name: 'preserves slashes and encodes special characters',
          async fn() {
            await Promise.resolve();
            expect(encodeS3Key('a/b/c',),).toBe('a/b/c',);
            expect(encodeS3Key('a/b c/d',),).toBe('a/b%20c/d',);
            expect(encodeS3Key('a/b?c/d',),).toBe('a/b%3Fc/d',);
          },
        },),
      ],
    },),
    describe({
      name: createS3Storage.name,
      concurrency: 1,
      children: [
        it({
          name: 'put + get round-trip',
          async fn() {
            const fake = createFakeServer();
            const storage = createS3Storage({
              client: fake.client,
              endpoint: FAKE_ENDPOINT,
              bucket: FAKE_BUCKET,
            },);
            const body = new TextEncoder().encode('hello',);
            await storage.put(
              'issues/r1/i1/detail',
              body,
            );
            const fetched = await storage.get('issues/r1/i1/detail',);
            expect(fetched,).toBeDefined();
            expect(new TextDecoder().decode(fetched,),).toBe('hello',);
          },
        },),
        it({
          name: 'get returns undefined for unknown key',
          async fn() {
            const fake = createFakeServer();
            const storage = createS3Storage({
              client: fake.client,
              endpoint: FAKE_ENDPOINT,
              bucket: FAKE_BUCKET,
            },);
            const fetched = await storage.get('does/not/exist',);
            expect(fetched,).toBe(undefined,);
          },
        },),
        it({
          name: 'putBatch issues parallel PUTs',
          async fn() {
            const fake = createFakeServer();
            const storage = createS3Storage({
              client: fake.client,
              endpoint: FAKE_ENDPOINT,
              bucket: FAKE_BUCKET,
            },);
            const items = [
              {
                key: 'a',
                body: new TextEncoder().encode('A',),
              },
              {
                key: 'b',
                body: new TextEncoder().encode('B',),
              },
              {
                key: 'c',
                body: new TextEncoder().encode('C',),
              },
            ];
            await storage.putBatch(items,);
            const a = await storage.get('a',);
            const b = await storage.get('b',);
            const c = await storage.get('c',);
            expect(new TextDecoder().decode(a,),).toBe('A',);
            expect(new TextDecoder().decode(b,),).toBe('B',);
            expect(new TextDecoder().decode(c,),).toBe('C',);
          },
        },),
        it({
          name: 'delete is idempotent for unknown key',
          async fn() {
            const fake = createFakeServer();
            const storage = createS3Storage({
              client: fake.client,
              endpoint: FAKE_ENDPOINT,
              bucket: FAKE_BUCKET,
            },);
            await storage.delete('absent',);
            const value = await storage.get('absent',);
            expect(value,).toBe(undefined,);
          },
        },),
        it({
          name: 'list returns sorted keys with prefix filter',
          async fn() {
            const fake = createFakeServer();
            const storage = createS3Storage({
              client: fake.client,
              endpoint: FAKE_ENDPOINT,
              bucket: FAKE_BUCKET,
            },);
            await storage.put(
              'issues/r1/i1',
              new Uint8Array([1,],),
            );
            await storage.put(
              'issues/r1/i2',
              new Uint8Array([1,],),
            );
            await storage.put(
              'issues/r2/i1',
              new Uint8Array([1,],),
            );
            await storage.put(
              'repos/r1/info',
              new Uint8Array([1,],),
            );
            const issuesKeys = await storage.list('issues/',);
            expect(issuesKeys,).toEqual([
              'issues/r1/i1',
              'issues/r1/i2',
              'issues/r2/i1',
            ],);
            const repoR1Keys = await storage.list('issues/r1/',);
            expect(repoR1Keys,).toEqual([
              'issues/r1/i1',
              'issues/r1/i2',
            ],);
          },
        },),
        it({
          name: 'put encodes special characters in the key path',
          async fn() {
            const fake = createFakeServer();
            const storage = createS3Storage({
              client: fake.client,
              endpoint: FAKE_ENDPOINT,
              bucket: FAKE_BUCKET,
            },);
            await storage.put(
              'a/b c/d',
              new TextEncoder().encode('hello',),
            );
            const lastPut = fake.log.find(function isPut(r,) {
              return r.method === 'PUT';
            },);
            expect(lastPut?.url.includes('a/b%20c/d',),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
