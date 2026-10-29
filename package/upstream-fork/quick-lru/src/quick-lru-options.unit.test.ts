/**
 Tests for cache option validation and resolution, driven through the
 public `createQuickLru` entry point (the resolver itself is
 package-internal).
 
 Every case here pins upstream `quick-lru` 7.3.0's acceptance rules and
 coercion quirks, so the differential oracle can compare constructor input
 handling directly.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createQuickLru,
  InvalidMaxAgeError,
  InvalidMaxSizeError,
} from '../dist/final/neutral/index.mjs';

/**
 `maxSize` inputs upstream rejects with its `maxSize` diagnostic.
 */
const INVALID_MAX_SIZES: readonly unknown[] = [
  0,
  -1,
  -2.5,
  Number.NaN,
  Number.NEGATIVE_INFINITY,
  '',
  false,
  null,
  undefined,
  {},
  [],
];

/**
 `maxSize` inputs upstream accepts, including the coercion quirks where a
 non-number truthy value greater than `0` slips through.
 */
const ACCEPTED_MAX_SIZES: readonly unknown[] = [
  1,
  2,
  1.5,
  Number.POSITIVE_INFINITY,
  '3',
  true,
];

/**
 `maxAge` inputs upstream stores unchanged or rewrites, with the stored
 value the cache must report back.
 */
const RESOLVED_MAX_AGES: readonly (readonly [unknown, number | string])[] = [
  [
    undefined,
    Number.POSITIVE_INFINITY,
  ],
  [
    Number.POSITIVE_INFINITY,
    Number.POSITIVE_INFINITY,
  ],
  [
    500,
    500,
  ],
  [
    -5,
    -5,
  ],
  [
    Number.NaN,
    Number.POSITIVE_INFINITY,
  ],
  [
    'x',
    'x',
  ],
];

/**
 Labels one generated test case so coercion-quirk inputs like `''` and `[]`
 keep distinct suite names.
 
 @param value - Generated input under test.
 
 @returns Type-tagged string form of the input.
 
 @example
 ```ts
 caseLabel([],); // => 'object:'
 ```
 */
function caseLabel(value: unknown,): string {
  return `${typeof value}:${String(value,)}`;
}

/**
 Catches one thrown value so its class can be asserted.
 
 @param run - Invocation expected to throw.
 
 @returns Whatever the invocation threw, `undefined` when it did not.
 
 @example
 ```ts
 const caught = catchThrown(function thrower(): void { throw new Error(); });
 ```
 */
function catchThrown(run: () => unknown,): unknown {
  try {
    run();
    return undefined;
  }
  catch (error) {
    return error;
  }
}

await describe({
  name: 'cache option resolution',
  children: [
    //region Accepted shapes

    ...ACCEPTED_MAX_SIZES.map(function mapAcceptedMaxSize(maxSize: unknown,) {
      return it({
        name: `accepts maxSize ${caseLabel(maxSize,)} and reports it back`,
        fn: async () => {
          const lru = createQuickLru<string, string>({
            maxSize: maxSize as number,
          },);
          expect(lru.maxSize,).toBe(maxSize,);
          expect(lru.maxAge,).toBe(Number.POSITIVE_INFINITY,);
        },
      },);
    },),

    //endregion Accepted shapes

    //region Rejected shapes

    ...INVALID_MAX_SIZES.map(function mapInvalidMaxSize(maxSize: unknown,) {
      return it({
        name: `rejects maxSize ${caseLabel(maxSize,)} with InvalidMaxSizeError`,
        fn: async () => {
          /**
           Value thrown by the rejected constructor input.
           */
          const caught = catchThrown(function construct(): void {
            createQuickLru<string, string>({
              maxSize: maxSize as number,
            },);
          },);
          expect(caught,).toBeInstanceOf(InvalidMaxSizeError,);
        },
      },);
    },),

    it({
      name: 'rejects a missing options object with InvalidMaxSizeError like upstream',
      fn: async () => {
        /**
         Value thrown by the missing-argument constructor call.
         */
        const caught = catchThrown(function construct(): void {
          createQuickLru<string, string>(undefined as never,);
        },);
        expect(caught,).toBeInstanceOf(InvalidMaxSizeError,);
      },
    },),

    it({
      name: 'throws a plain TypeError for a null constructor input from the same property read upstream performs',
      fn: async () => {
        /**
         Value thrown by the null constructor input.
         */
        const caught = catchThrown(function construct(): void {
          createQuickLru<string, string>(null as never,);
        },);
        expect(caught,).toBeInstanceOf(TypeError,);
        expect(caught instanceof InvalidMaxSizeError,).toBe(false,);
        expect((caught as Error).name,).toBe('TypeError',);
      },
    },),

    //endregion Rejected shapes

    //region Global maxAge

    ...RESOLVED_MAX_AGES.map(function mapResolvedMaxAge(
      [maxAge, expected,]: readonly [unknown, number | string],
    ) {
      return it({
        name: `resolves maxAge ${caseLabel(maxAge,)} to ${caseLabel(expected,)}`,
        fn: async () => {
          const lru = createQuickLru<string, string>({
            maxSize: 1,
            maxAge: maxAge as number,
          },);
          expect(lru.maxAge,).toBe(expected,);
        },
      },);
    },),

    it({
      name: 'rejects maxAge 0 with InvalidMaxAgeError',
      fn: async () => {
        /**
         Value thrown by the rejected constructor input.
         */
        const caught = catchThrown(function construct(): void {
          createQuickLru<string, string>({
            maxSize: 1,
            maxAge: 0,
          },);
        },);
        expect(caught,).toBeInstanceOf(InvalidMaxAgeError,);
      },
    },),

    it({
      name: 'rejects maxAge negative zero with InvalidMaxAgeError, matching upstream strict equality',
      fn: async () => {
        /**
         Value thrown by the rejected constructor input.
         */
        const caught = catchThrown(function construct(): void {
          createQuickLru<string, string>({
            maxSize: 1,
            maxAge: -0,
          },);
        },);
        expect(caught,).toBeInstanceOf(InvalidMaxAgeError,);
      },
    },),

    it({
      name: 'validates maxSize before maxAge, matching upstream precedence',
      fn: async () => {
        /**
         Value thrown when both options are invalid at once.
         */
        const caught = catchThrown(function construct(): void {
          createQuickLru<string, string>({
            maxSize: 0,
            maxAge: 0,
          },);
        },);
        expect(caught,).toBeInstanceOf(InvalidMaxSizeError,);
      },
    },),

    //endregion Global maxAge

    //region resize validation

    it({
      name: 'applies the same maxSize acceptance rule to resize',
      fn: async () => {
        const lru = createQuickLru<string, string>({
          maxSize: 1,
        },);
        lru.resize(5,);
        expect(lru.maxSize,).toBe(5,);
      },
    },),

    ...INVALID_MAX_SIZES.map(function mapInvalidResize(maxSize: unknown,) {
      return it({
        name: `rejects resize(${caseLabel(maxSize,)}) with InvalidMaxSizeError`,
        fn: async () => {
          const lru = createQuickLru<string, string>({
            maxSize: 2,
          },);
          /**
           Value thrown by the rejected resize input.
           */
          const caught = catchThrown(function resize(): void {
            lru.resize(maxSize as number,);
          },);
          expect(caught,).toBeInstanceOf(InvalidMaxSizeError,);
          expect(lru.maxSize,).toBe(2,);
        },
      },);
    },),

    it({
      name: 'stores a coercion-quirk maxSize from resize unchanged, matching upstream',
      fn: async () => {
        const lru = createQuickLru<string, string>({
          maxSize: 1,
        },);
        lru.resize('3' as never,);
        expect(lru.maxSize,).toBe('3',);
      },
    },),

    //endregion resize validation
  ],
},);
