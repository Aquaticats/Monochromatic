/**
 Tests for strict input validation: numeric-looking classification, canonical
 integer parsing, the `/int` query contract, and length-route resolution.

 Every refusal case here is a behaviour the Caddy origin could not express,
 because its template functions coerced instead of refusing.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  BadRequestError,
  looksNumeric,
  MAX_RANDOM_LENGTH,
  NotFoundError,
  parseCanonicalInteger,
  parseIntParams,
  parseLengthRoute,
} from '@monochromatic-dev/cloudflare-worker-rand';

/**
 Build the query a `/int` case sends.

 @param query - raw query string without the leading question mark

 @returns parsed query for the validator
 */
function params(query: string,): URLSearchParams {
  return new URLSearchParams(query,);
}

await describe({
  name: 'validate',
  children: [
    describe({
      name: looksNumeric.name,
      children: [
        it({
          name: 'accepts digits and numeric punctuation that carries a digit',
          fn: async () => {
            for (const text of [
              '1',
              '0',
              '64',
              '-1',
              '+5',
              '007',
              '1.5',
              '1e3',
              '-.5',
            ]) {
              expect(looksNumeric(text,),).toBe(true,);
            }
          },
        },),
        it({
          name: 'refuses empty text, punctuation alone, and any segment holding a letter',
          fn: async () => {
            for (const text of [
              '',
              'nope',
              '1a',
              'a1',
              '+',
              '-',
              '.',
              'e',
              '1 2',
              'uuidv4',
            ]) {
              expect(looksNumeric(text,),).toBe(false,);
            }
          },
        },),
      ],
    },),
    describe({
      name: parseCanonicalInteger.name,
      children: [
        it({
          name: 'accepts canonical decimal integers across the safe range',
          fn: async () => {
            expect(parseCanonicalInteger({
              text: '0',
              label: 'x',
            },),).toBe(0,);
            expect(parseCanonicalInteger({
              text: '5',
              label: 'x',
            },),).toBe(5,);
            expect(parseCanonicalInteger({
              text: '-3',
              label: 'x',
            },),).toBe(-3,);
            expect(parseCanonicalInteger({
              text: String(Number.MAX_SAFE_INTEGER,),
              label: 'x',
            },),).toBe(Number.MAX_SAFE_INTEGER,);
            expect(parseCanonicalInteger({
              text: String(-Number.MAX_SAFE_INTEGER,),
              label: 'x',
            },),).toBe(-Number.MAX_SAFE_INTEGER,);
          },
        },),
        it({
          name: 'refuses every non-canonical form the origin coerced to zero',
          fn: async () => {
            for (const text of [
              '',
              '-',
              '+5',
              ' 1',
              '1 ',
              '007',
              '-007',
              '-0',
              '1.5',
              '0x10',
              '1e3',
              'abc',
              '1_000',
              '１',
            ]) {
              expect(() => parseCanonicalInteger({
                text,
                label: 'query parameter "min"',
              },),).toThrow('query parameter "min"',);
            }
          },
        },),
        it({
          name: 'refuses magnitudes beyond the safe integer range instead of clamping',
          fn: async () => {
            for (const text of [
              '9007199254740992',
              '-9007199254740992',
              '100000000000000000000',
              '-100000000000000000000',
            ]) {
              expect(() => parseCanonicalInteger({
                text,
                label: 'path segment',
              },),).toThrow('must be an integer between',);
            }
          },
        },),
      ],
    },),
    describe({
      name: parseIntParams.name,
      children: [
        it({
          name: 'accepts an ordered inclusive interval, including a single value and negatives',
          fn: async () => {
            expect(parseIntParams({ searchParams: params('min=1&max=6',), },),).toEqual({
              min: 1,
              max: 6,
            },);
            expect(parseIntParams({ searchParams: params('min=5&max=5',), },),).toEqual({
              min: 5,
              max: 5,
            },);
            expect(parseIntParams({ searchParams: params('min=-3&max=-1',), },),).toEqual({
              min: -3,
              max: -1,
            },);
            expect(parseIntParams({
              searchParams: params('min=-1000&max=1000',),
            },),).toEqual({
              min: -1_000,
              max: 1_000,
            },);
          },
        },),
        it({
          name: 'ignores unknown extra parameters',
          fn: async () => {
            expect(parseIntParams({
              searchParams: params('min=1&max=6&utm_source=shell&foo=bar',),
            },),).toEqual({
              min: 1,
              max: 6,
            },);
          },
        },),
        it({
          name: 'refuses an absent or empty parameter, which the origin read as zero',
          fn: async () => {
            expect(() => parseIntParams({ searchParams: params('max=6',), },),).toThrow('"min" is required',);
            expect(() => parseIntParams({ searchParams: params('min=1',), },),).toThrow('"max" is required',);
            expect(() => parseIntParams({ searchParams: params('',), },),).toThrow('is required',);
            expect(() => parseIntParams({ searchParams: params('min=&max=6',), },),).toThrow('query parameter "min"',);
          },
        },),
        it({
          name: 'refuses a repeated parameter, which the origin collapsed into zero',
          fn: async () => {
            expect(() => parseIntParams({
              searchParams: params('min=1&max=6&min=9',),
            },),).toThrow('"min" must appear at most once',);
            expect(() => parseIntParams({
              searchParams: params('min=0&max=2&max=6',),
            },),).toThrow('"max" must appear at most once',);
          },
        },),
        it({
          name: 'refuses a reversed interval',
          fn: async () => {
            expect(() => parseIntParams({
              searchParams: params('min=6&max=1',),
            },),).toThrow('must not exceed',);
          },
        },),
        it({
          name: 'refuses a span wider than an exact uniform draw can represent',
          fn: async () => {
            // Boundary: a difference of exactly MAX_SAFE_INTEGER is representable
            // and must be accepted, while one more is not and must be refused.
            // The Caddy origin instead wrapped and answered 500.
            expect(parseIntParams({
              searchParams: params(`min=0&max=${String(Number.MAX_SAFE_INTEGER,)}`,),
            },),).toEqual({
              min: 0,
              max: Number.MAX_SAFE_INTEGER,
            },);
            expect(() => parseIntParams({
              searchParams: params(`min=-1&max=${String(Number.MAX_SAFE_INTEGER,)}`,),
            },),).toThrow('the difference between',);
            expect(() => parseIntParams({
              searchParams: params(`min=${String(-Number.MAX_SAFE_INTEGER,)}&max=${String(Number.MAX_SAFE_INTEGER,)}`,),
            },),).toThrow('the difference between',);
          },
        },),
        it({
          name: 'matches parameter names case-sensitively, as the origin did',
          fn: async () => {
            expect(() => parseIntParams({
              searchParams: params('MIN=1&max=6',),
            },),).toThrow('"min" is required',);
          },
        },),
      ],
    },),
    describe({
      name: parseLengthRoute.name,
      children: [
        it({
          name: 'resolves a single canonical segment inside the served range',
          fn: async () => {
            expect(parseLengthRoute({ pathname: '/1', },),).toBe(1,);
            expect(parseLengthRoute({ pathname: '/7', },),).toBe(7,);
            expect(parseLengthRoute({
              pathname: `/${String(MAX_RANDOM_LENGTH,)}`,
            },),).toBe(MAX_RANDOM_LENGTH,);
          },
        },),
        it({
          name: 'refuses a numeric-looking segment outside the range as 400',
          fn: async () => {
            for (const pathname of [
              '/0',
              '/65',
              '/007',
              '/-1',
              '/1.5',
              '/1e3',
              '/99999999999999999999',
            ]) {
              expect(() => parseLengthRoute({ pathname, },),).toThrow(BadRequestError,);
            }
            expect(() => parseLengthRoute({ pathname: '/65', },),).toThrow('length must be between 1 and 64',);
          },
        },),
        it({
          name: 'reports a non-numeric or multi-segment path as 404 listing the routes',
          fn: async () => {
            for (const pathname of [
              '/abc',
              '/nope',
              '/1a',
              '/',
              '/1/2',
              '',
            ]) {
              expect(() => parseLengthRoute({ pathname, },),).toThrow(NotFoundError,);
            }
            expect(() => parseLengthRoute({ pathname: '/abc', },),).toThrow('served routes are /uuidv4',);
          },
        },),
      ],
    },),
  ],
},);
