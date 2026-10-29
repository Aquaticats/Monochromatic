/**
 Tests for request routing through the library entry: the served routes, the
 case-insensitive path matching kept from the Caddy origin, the strict forms
 the origin could not express, and the method matrix.

 Requests inside one case are issued concurrently through `Promise.all`
 rather than awaited in a loop, since nothing here touches a network and the
 cases only need every result at the end.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ALPHANUMERIC_CHARSET,
  handleRequest,
  MAX_RANDOM_LENGTH,
} from '@monochromatic-dev/cloudflare-worker-rand';

/**
 Logger shared by every case.
 */
const l = tagged({ tag: 'index.unit.test', },);

/**
 Origin every request targets.
 */
const ORIGIN = 'https://rand.test';

/**
 Draws taken where a case needs every outcome of a small domain to appear.
 */
const MANY_DRAWS = 400;

/**
 Exact cache policy every response must carry.
 */
const EXPECTED_CACHE_CONTROL = 'no-store, no-cache, must-revalidate';

/**
 Status and body of one routed request.
 */
type ServedResult = {
  /**
   HTTP status the router produced.
   */
  readonly status: number;
  /**
   Response body text.
   */
  readonly body: string;
};

/**
 Route one request through the library entry.

 @param path - path and query to request

 @param method - HTTP method to use

 @returns the response the router produced
 */
async function route(path: string, method = 'GET',): Promise<Response> {
  return handleRequest({
    request: new Request(`${ORIGIN}${path}`, { method, },),
    l,
  },);
}

/**
 Read one served value's status and body together.

 @param path - path and query to request

 @returns status and body of the response
 */
async function served(path: string,): Promise<ServedResult> {
  /**
   Response to inspect.
   */
  const response = await route(path,);
  return {
    status: response.status,
    body: await response.text(),
  };
}

/**
 Route several distinct paths concurrently.

 @param paths - paths and queries to request

 @returns one status and body per path, in the order given
 */
async function servedAll(paths: readonly string[],): Promise<ServedResult[]> {
  return await Promise.all(paths.map(async (path) => await served(path,),),);
}

/**
 Route the same path many times concurrently.

 @param path - path and query to request repeatedly

 @param count - how many requests to issue

 @returns one status and body per request
 */
async function servedMany(path: string, count: number,): Promise<ServedResult[]> {
  return await Promise.all(Array.from({ length: count, }, async () => await served(path,),),);
}

/**
 Whether `text` is a lowercase canonical version 4 UUID.

 Checked by index scan rather than pattern match: the shape is fully
 expressible as character positions, and the repository bans regex literals
 without a scoped justification.

 @param text - value to classify

 @returns whether every position matches the version 4 UUID layout
 */
function isUuidV4(text: string,): boolean {
  if (text.length !== 36) {
    return false;
  }
  /**
   Characters a hexadecimal position may hold.
   */
  const hex = '0123456789abcdef';
  for (let index = 0; index < 36; index += 1) {
    /**
     Character at this position.
     */
    const character = text.charAt(index,);
    if ((index === 8) || (index === 13) || (index === 18) || (index === 23)) {
      if (character !== '-') {
        return false;
      }
      continue;
    }
    if (!hex.includes(character,)) {
      return false;
    }
  }
  // Position 14 is the version nibble and position 19 the variant nibble.
  return (text.charAt(14,) === '4') && '89ab'.includes(text.charAt(19,),);
}

await describe({
  name: handleRequest.name,
  children: [
    describe({
      name: 'the uuidv4 route',
      children: [
        it({
          name: 'serves 36 bytes with no trailing newline and the shared headers',
          fn: async () => {
            /**
             Response for the UUID route.
             */
            const response = await route('/uuidv4',);
            expect(response.status,).toBe(200,);
            expect(response.headers.get('Content-Type',),).toBe('text/plain; charset=utf-8',);
            expect(response.headers.get('Content-Length',),).toBe('36',);
            expect(response.headers.get('Cache-Control',),).toBe(EXPECTED_CACHE_CONTROL,);
            expect(response.headers.get('Access-Control-Allow-Origin',),).toBe('*',);
            /**
             Served UUID.
             */
            const body = await response.text();
            expect(isUuidV4(body,),).toBe(true,);
          },
        },),
        it({
          name: 'matches case-insensitively, as the origin did',
          fn: async () => {
            /**
             Statuses for the uppercase spellings the origin also served.
             */
            const results = await servedAll([
              '/UUIDV4',
              '/Uuidv4',
              '/uUidV4',
            ],);
            for (const result of results) {
              expect(result.status,).toBe(200,);
              expect(isUuidV4(result.body,),).toBe(true,);
            }
          },
        },),
      ],
    },),
    describe({
      name: 'the int route',
      children: [
        it({
          name: 'includes both bounds, the fix the origin needed a template workaround for',
          fn: async () => {
            /**
             Every draw for the interval 1 to 3.
             */
            const results = await servedMany('/int?min=1&max=3', MANY_DRAWS,);
            for (const result of results) {
              expect(result.status,).toBe(200,);
            }
            expect([
              ...new Set(results.map((result) => result.body,),),
            ].toSorted(),).toEqual([
              '1',
              '2',
              '3',
            ],);
          },
        },),
        it({
          name: 'serves the single value of a degenerate interval the origin answered with 500',
          fn: async () => {
            /**
             Statuses and bodies for the two single-value intervals.
             */
            const results = await servedAll([
              '/int?min=5&max=5',
              '/int?min=0&max=0',
            ],);
            expect(results.map((result) => result.body,),).toEqual([
              '5',
              '0',
            ],);
          },
        },),
        it({
          name: 'serves negative intervals inside the bounds',
          fn: async () => {
            /**
             Every draw for the interval -3 to -1.
             */
            const results = await servedMany('/int?min=-3&max=-1', MANY_DRAWS,);
            for (const result of results) {
              expect(result.status,).toBe(200,);
            }
            expect([
              ...new Set(results.map((result) => result.body,),),
            ].toSorted(),).toEqual([
              '-1',
              '-2',
              '-3',
            ],);
          },
        },),
        it({
          name: 'refuses with 400 everything the origin coerced to zero or answered with 500',
          fn: async () => {
            /**
             Statuses and bodies for every malformed query.
             */
            const results = await servedAll([
              '/int',
              '/int?min=1',
              '/int?max=6',
              '/int?min=&max=',
              '/int?min=abc&max=5',
              '/int?min=1.5&max=5.5',
              '/int?min=0x10&max=0x20',
              '/int?min=6&max=1',
              '/int?min=1&max=6&min=9',
              '/int?min=0&max=2&max=6',
              '/int?min=0&max=100000000000000000000',
              '/int?min=0&max=9223372036854775807',
              '/int?min=-99999999999999999999&max=0',
            ],);
            for (const result of results) {
              expect(result.status,).toBe(400,);
              expect(result.body.length > 0,).toBe(true,);
            }
          },
        },),
        it({
          name: 'names the offending parameter in the refusal',
          fn: async () => {
            /**
             Bodies for three distinct refusals.
             */
            const results = await servedAll([
              '/int?max=6',
              '/int?min=1&max=6&min=9',
              '/int?min=6&max=1',
            ],);
            expect(results[0]?.body,).toMatch('query parameter "min" is required',);
            expect(results[1]?.body,).toMatch('must appear at most once',);
            expect(results[2]?.body,).toMatch('must not exceed',);
          },
        },),
        it({
          name: 'matches parameter names case-sensitively and ignores unknown ones',
          fn: async () => {
            /**
             Statuses for a mis-cased name and for an unknown extra parameter.
             */
            const results = await servedAll([
              '/int?MIN=1&max=6',
              '/int?min=1&max=6&utm_source=shell',
            ],);
            expect(results[0]?.status,).toBe(400,);
            expect(results[1]?.status,).toBe(200,);
          },
        },),
        it({
          name: 'matches the route path case-insensitively',
          fn: async () => {
            expect((await served('/INT?min=1&max=3',)).status,).toBe(200,);
          },
        },),
      ],
    },),
    describe({
      name: 'the length routes',
      children: [
        it({
          name: 'serves exactly the requested count from the 62-character charset',
          fn: async () => {
            /**
             Lengths exercised, spanning the shortest, a middle and the longest
             served route.
             */
            const lengths = [
              1,
              2,
              7,
              32,
              MAX_RANDOM_LENGTH,
            ];
            /**
             Response and body for each length.
             */
            const results = await Promise.all(lengths.map(async (length) => {
              /**
               Response for this length route.
               */
              const response = await route(`/${String(length)}`,);
              return {
                length,
                status: response.status,
                contentLength: response.headers.get('Content-Length',),
                body: await response.text(),
              };
            },),);
            for (const result of results) {
              expect(result.status,).toBe(200,);
              expect(result.contentLength,).toBe(String(result.length,),);
              expect(result.body,).toHaveLength(result.length,);
              for (const character of result.body) {
                expect(ALPHANUMERIC_CHARSET.includes(character,),).toBe(true,);
              }
            }
          },
        },),
        it({
          name: 'serves the maximum length on the root path, as the origin did',
          fn: async () => {
            /**
             Response for the root route.
             */
            const response = await route('/',);
            expect(response.status,).toBe(200,);
            expect(response.headers.get('Content-Length',),).toBe(String(MAX_RANDOM_LENGTH,),);
            expect(await response.text(),).toHaveLength(MAX_RANDOM_LENGTH,);
          },
        },),
        it({
          name: 'answers 400 for a numeric length outside the served range',
          fn: async () => {
            /**
             Statuses for lengths that look numeric but are not served.
             */
            const results = await servedAll([
              '/0',
              '/65',
              '/007',
              '/-1',
              '/1.5',
            ],);
            for (const result of results) {
              expect(result.status,).toBe(400,);
            }
            expect((await served('/65',)).body,).toMatch('length must be between 1 and 64',);
          },
        },),
        it({
          name: 'answers 404 for a path that never looked numeric',
          fn: async () => {
            /**
             Statuses for paths that are unknown rather than malformed.
             */
            const results = await servedAll([
              '/abc',
              '/nope',
              '/1a',
              '/uuidv',
            ],);
            for (const result of results) {
              expect(result.status,).toBe(404,);
            }
            expect((await served('/nope',)).body,).toMatch('served routes are /uuidv4',);
          },
        },),
        it({
          name: 'answers 404 for the encoded and doubled-slash forms the origin matched',
          fn: async () => {
            // The origin percent-decoded and collapsed slashes before matching,
            // so /%75uidv4 and //uuidv4 both served. URL normalization does
            // neither, so both are unknown paths here. This is a deliberate
            // strictness difference, not a port gap.
            /**
             Statuses for the normalized forms the origin matched.
             */
            const results = await servedAll([
              '/%75uidv4',
              '//uuidv4',
              '/uuidv4/',
              '/1/',
              '/1%2f',
            ],);
            for (const result of results) {
              expect(result.status,).toBe(404,);
            }
          },
        },),
      ],
    },),
    describe({
      name: 'the method matrix',
      children: [
        it({
          name: 'serves HEAD with the real length and no body',
          fn: async () => {
            /**
             HEAD response for the UUID route.
             */
            const response = await route('/uuidv4', 'HEAD',);
            expect(response.status,).toBe(200,);
            expect(response.headers.get('Content-Length',),).toBe('36',);
            expect(response.headers.get('Content-Type',),).toBe('text/plain; charset=utf-8',);
            expect(await response.text(),).toBe('',);
          },
        },),
        it({
          name: 'answers OPTIONS with a preflight on any path, including an unknown one',
          fn: async () => {
            /**
             Preflight responses for a known and an unknown path.
             */
            const responses = await Promise.all([
              '/uuidv4',
              '/nope',
            ].map(async (path) => await route(path, 'OPTIONS',),),);
            for (const response of responses) {
              expect(response.status,).toBe(204,);
              expect(response.headers.get('Allow',),).toBe('GET, HEAD, OPTIONS',);
              expect(response.headers.get('Access-Control-Allow-Methods',),).toBe('GET, HEAD, OPTIONS',);
            }
          },
        },),
        it({
          name: 'answers 405 with Allow for every other method, where the origin served them all',
          fn: async () => {
            /**
             Write methods this endpoint has no semantics for.
             */
            const methods = [
              'POST',
              'PUT',
              'PATCH',
              'DELETE',
            ];
            /**
             Refusal body for each method, in the same order. The status and
             the `Allow` header are asserted while each response is still in
             hand, so no body read is left to await inside a loop.
             */
            const bodies = await Promise.all(methods.map(async (method) => {
              /**
               Refusal for this method.
               */
              const response = await route('/uuidv4', method,);
              expect(response.status,).toBe(405,);
              expect(response.headers.get('Allow',),).toBe('GET, HEAD, OPTIONS',);
              return await response.text();
            },),);
            for (const [
              index,
              body,
            ] of bodies.entries()) {
              expect(body,).toMatch(`${methods[index] ?? ''} is not served`,);
            }
          },
        },),
      ],
    },),
  ],
},);
