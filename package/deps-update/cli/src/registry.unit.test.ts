/**
 Unit tests for packument URLs and publish-time lookup, with a stubbed fetch.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type FetchLike,
  fetchPublishTime,
  packumentUrl,
  PublishTimeError,
} from './registry.ts';

/**
 Spec used across lookups.
 */
const SPEC = {
  name: '@earendil-works/chord',
  version: '0.87.1',
} as const;

/**
 HTTP status for a missing packument.
 */
const NOT_FOUND = 404;

/**
 Builds a fetch stub answering every request with one response.

 @param body - JSON body to return

 @param status - HTTP status

 @returns fetch stub plus the URLs it saw

 @example
 ```ts
 const { fetchImpl, urls } = stubFetch({ body: {}, status: 200 });
 ```
 */
function stubFetch({
  body,
  status,
}: {
  readonly body: unknown;
  readonly status: number;
},): {
  readonly fetchImpl: FetchLike;
  readonly urls: readonly string[];
} {
  /**
   Requested URLs, recorded for assertions.
   */
  const urls: string[] = [];
  return {
    urls,
    fetchImpl: async function fakeFetch(url,): Promise<Response> {
      urls.push(url,);
      return Response.json(body, { status, },);
    },
  };
}

/**
 Awaits a lookup and returns its rejection.

 @param body - packument body to serve

 @param status - HTTP status to serve

 @returns rejection value, or a marker string when it resolved

 @example
 ```ts
 await rejection({ body: {}, status: 404 });
 ```
 */
async function rejection({
  body,
  status,
}: {
  readonly body: unknown;
  readonly status: number;
},): Promise<unknown> {
  try {
    await fetchPublishTime({
      registry: 'https://registry.example/',
      spec: SPEC,
      fetchImpl: stubFetch({ body, status, },).fetchImpl,
    },);
    return 'resolved';
  }
  catch (error) {
    return error;
  }
}

await describe({
  name: '',
  children: [
    describe({
      name: packumentUrl.name,
      children: [
        it({
          name: 'encodes the scope slash',
          fn: async () => {
            expect(packumentUrl({ registry: 'https://registry.npmjs.org/', name: '@a/b', },),).toBe(
              'https://registry.npmjs.org/@a%2fb',
            );
          },
        },),
        it({
          name: 'adds a missing trailing slash',
          fn: async () => {
            expect(packumentUrl({ registry: 'https://registry.example', name: 'left-pad', },),).toBe(
              'https://registry.example/left-pad',
            );
          },
        },),
      ],
    },),
    describe({
      name: fetchPublishTime.name,
      children: [
        it({
          name: 'returns the publish instant for the version',
          fn: async () => {
            /**
             Stub serving one timestamp.
             */
            const stub = stubFetch({
              body: { time: { '0.87.1': '2026-09-22T19:38:00.606Z', }, },
              status: 200,
            },);
            /**
             Resolved instant.
             */
            const publishedAt = await fetchPublishTime({
              registry: 'https://registry.example/',
              spec: SPEC,
              fetchImpl: stub.fetchImpl,
            },);
            expect(publishedAt.toISOString(),).toBe('2026-09-22T19:38:00.606Z',);
            expect(stub.urls,).toEqual(['https://registry.example/@earendil-works%2fchord',],);
          },
        },),
        it({
          name: 'rejects a non-OK response',
          fn: async () => {
            expect(await rejection({ body: {}, status: NOT_FOUND, },),).toBeInstanceOf(PublishTimeError,);
          },
        },),
        it({
          name: 'rejects a packument without this version',
          fn: async () => {
            expect(await rejection({ body: { time: { '0.87.0': '2026-09-21T16:41:30.718Z', }, }, status: 200, },),)
              .toBeInstanceOf(PublishTimeError,);
          },
        },),
        it({
          name: 'rejects a packument without a time map',
          fn: async () => {
            expect(await rejection({ body: { name: 'x', }, status: 200, },),).toBeInstanceOf(PublishTimeError,);
          },
        },),
        it({
          name: 'rejects an unparseable timestamp',
          fn: async () => {
            expect(await rejection({ body: { time: { '0.87.1': 'soon', }, }, status: 200, },),)
              .toBeInstanceOf(PublishTimeError,);
          },
        },),
      ],
    },),
  ],
},);
