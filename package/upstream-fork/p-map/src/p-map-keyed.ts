/**
 TypeScript fork of `p-map`'s keyed variants: map over the values of an object
 or a `Map` with limited concurrency, like `Promise.allKeyed()` and
 `Promise.allSettledKeyed()` with a mapper.

 Derived from [`p-map`](https://github.com/sindresorhus/p-map) by Sindre
 Sorhus (MIT); see `LICENSES/MIT.txt` and this package's README for the full
 attribution. Semantics match `p-map` 7.1.1: an object contributes its own
 enumerable string and symbol keys, a `Map` its entries, every value is read
 and given a handler up front, a `pMapSkip` result leaves out the key, and
 `pMapStop` keeps only the keys before it.

 @module
 */

import { InvalidKeyedInputError, } from './errors.ts';
import type {
  MapOptions,
  SettledMapOptions,
} from './map-options.ts';
import {
  type MapControl,
  validateMapper,
} from './mapper.ts';
import { mapConcurrently, } from './p-map.ts';
import { pMapSkip, } from './p-map-skip.ts';
import { pMapStop, } from './p-map-stop.ts';

//region Types

/**
 The keys as the mapper gets them: the keys of a `Map`, or else the keys of
 the object, where a number key, like `1` in `{ 1: 'a' }`, is a string at
 runtime. It goes through each member of a union input, as `keyof` would only
 give the keys they share.

 @typeParam Input - object or `Map` whose values are mapped

 @example
 ```ts
 const key: KeyedKey<{ user: string; }> = 'user';
 ```
 */
export type KeyedKey<Input> = Input extends ReadonlyMap<infer Key, unknown>
  ? Key
  : Exclude<keyof Input, number> | `${Extract<keyof Input, number>}`;

/**
 The values as the mapper gets them: awaited, from the `Map` or the object.

 @typeParam Input - object or `Map` whose values are mapped

 @example
 ```ts
 const value: KeyedValue<{ user: Promise<string>; }> = 'ann';
 ```
 */
export type KeyedValue<Input> = Awaited<Input extends ReadonlyMap<unknown, infer Value> ? Value : Input[keyof Input]>;

/**
 The mapper as {@link pMapKeyed} and {@link pMapSettledKeyed} take it. Unlike
 {@link KeyedMapper}, it does not add the skip and stop sentinels to the
 return type, so they stay in the inferred value type, where
 {@link KeyedResult} sees whether a key can be left out.

 @typeParam Input - object or `Map` whose values are mapped

 @typeParam NewValue - value type the mapper returns, sentinels included

 @example
 ```ts
 const mapper: KeyedMapperFunction<{ a: number; }, string> = function show(value: number,): string {
   return String(value,);
 };
 ```
 */
export type KeyedMapperFunction<Input, NewValue> = (
  value: KeyedValue<Input>,
  key: KeyedKey<Input>,
) => NewValue | PromiseLike<NewValue>;

/* oxlint-disable typescript/no-explicit-any -- mirrors upstream `p-map`'s `KeyedMapper<Input extends object = any, ...>` default, so an unparameterized `KeyedMapper` accepts any input */
/**
 Function called for every value of the input with its key.

 @typeParam Input - object or `Map` whose values are mapped

 @typeParam NewValue - value type the mapper returns

 @example
 ```ts
 const mapper: KeyedMapper<{ a: number; }, string> = function show(value: number,): string {
   return String(value,);
 };
 ```
 */
export type KeyedMapper<Input extends object = any, NewValue = unknown> = KeyedMapperFunction<Input, NewValue | MapControl>;
/* oxlint-enable typescript/no-explicit-any */

/**
 Any other iterable, like an array or a `Set`, is accepted at runtime, like by
 `Promise.allKeyed`, but its keys are not what the types would describe, and
 for a `Set` there are none, so it is rejected here. An intersection, unlike a
 conditional type, also accepts the input of a generic caller.

 @typeParam Input - object or `Map` whose values are mapped

 @example
 ```ts
 const input: KeyedInput<{ a: number; }> = { a: 1, };
 ```
 */
export type KeyedInput<Input> = Input & (ReadonlyMap<unknown, unknown> | { readonly [Symbol.iterator]?: never; });

/* oxlint-disable no-restricted-syntax/no-optional-escape -- mirrors upstream `p-map`'s `KeyedResult`: a key the mapper can skip or stop at is genuinely absent from the result object, which only an optional mapped property types; `Partial` is banned by the same rule */
/**
 A key is only optional when the mapper can return `pMapSkip` or `pMapStop`,
 which leave it out. A mapper that returns `any` is not taken to return them.

 @typeParam Input - object or `Map` whose values are mapped

 @typeParam NewValue - value type the mapper returns, sentinels included

 @typeParam Value - value type of the result

 @example
 ```ts
 const result: KeyedResult<{ a: number; }, string, string> = { a: 'x', };
 ```
 */
export type KeyedResult<Input, NewValue, Value> = Input extends ReadonlyMap<infer Key, unknown>
  ? Map<Key, Value>
  : ([NewValue] extends [Exclude<NewValue, MapControl>] ? { -readonly [Key in keyof Input]: Value } : { -readonly [Key in keyof Input]?: Value });
/* oxlint-enable no-restricted-syntax/no-optional-escape */

/* oxlint-disable no-restricted-syntax/no-nullish-union, typescript/no-generated-empty-object-type -- mirrors upstream `p-map`'s `KeyedNewValue`, spelled out because `unknown` would absorb the sentinel types, and `{}` is how upstream writes any non-nullish value (`NonNullable<unknown>` is the same type) */
/**
 Any value returned from the mapper. It is `unknown` spelled out, as
 `unknown | MapControl` would be just `unknown`, and naming the sentinels makes
 TypeScript keep the `unique symbol` type of a mapper that only returns one of
 them instead of widening it to `symbol`.

 @example
 ```ts
 const value: KeyedNewValue = 1;
 ```
 */
export type KeyedNewValue = NonNullable<unknown> | null | undefined | MapControl;
/* oxlint-enable no-restricted-syntax/no-nullish-union, typescript/no-generated-empty-object-type */

/**
 One keyed-map run: the object or `Map` whose values are mapped, the mapper
 run per value, and the upstream-shaped options.

 The single destructured object replaces upstream `p-map`'s
 `pMapKeyed(input, mapper, options)` positional parameters, which repository
 lint bans.

 @typeParam Input - object or `Map` whose values are mapped

 @typeParam NewValue - value type the mapper returns, sentinels included

 @example
 ```ts
 const run: KeyedMapRun<{ a: number; }, number> = {
   input: { a: 1, },
   mapper: function double(value: number,): number {
     return value * 2;
   },
 };
 ```
 */
export type KeyedMapRun<Input extends object, NewValue extends KeyedNewValue> = {
  /**
   The object or `Map` whose values are mapped.
   */
  readonly input: KeyedInput<Input>;
  /**
   Function run per value with its key.
   */
  readonly mapper: KeyedMapperFunction<Input, NewValue>;
  /**
   Concurrency bound, `stopOnError` policy, and optional abort signal.
   */
  readonly options?: MapOptions;
};

/**
 One settled keyed-map run: the same shape as {@link KeyedMapRun}, without
 `stopOnError`.

 @typeParam Input - object or `Map` whose values are mapped

 @typeParam NewValue - value type the mapper returns, sentinels included

 @example
 ```ts
 const run: SettledKeyedMapRun<{ a: number; }, number> = {
   input: { a: 1, },
   mapper: function double(value: number,): number {
     return value * 2;
   },
 };
 ```
 */
export type SettledKeyedMapRun<Input extends object, NewValue extends KeyedNewValue> = {
  /**
   The object or `Map` whose values are mapped.
   */
  readonly input: KeyedInput<Input>;
  /**
   Function run per value with its key.
   */
  readonly mapper: KeyedMapperFunction<Input, NewValue>;
  /**
   Concurrency bound and optional abort signal.
   */
  readonly options?: SettledMapOptions;
};

/**
 One value of the input, read and given its handler: fulfilled with the
 awaited value, or rejected with its reason.
 */
type SettledValue =
  | {
    /**
     The value fulfilled.
     */
    readonly status: 'fulfilled';
    /**
     The awaited value.
     */
    readonly value: unknown;
  }
    | {
    /**
     The value rejected.
     */
      readonly status: 'rejected';
    /**
     The rejection reason.
     */
      readonly reason: unknown;
    };

/**
 One key with its settled value, as the engine receives it as an element.
 */
type KeyedEntry = readonly [
  key: unknown,
  settled: SettledValue,
];

/**
 The loosely typed mapper the implementation calls; the public signatures
 carry the precise types.
 */
type LooseKeyedMapper = (
  value: unknown,
  key: unknown,
) => unknown;

/**
 What the engine's wrapper mapper returns for one entry: the skip or stop
 sentinel, or the key with its result.
 */
type MappedEntry = MapControl | MappedPair;

/**
 One key with its mapped result.
 */
type MappedPair = readonly [
  key: unknown,
  result: unknown,
];

/**
 A keyed-map run as the engine sees it: the precise types are the public
 signatures' business, and the engine validates the input and the mapper at
 runtime.
 */
type LooseKeyedRun = {
  /**
   The object or `Map` whose values are mapped, as the caller supplied it.
   */
  readonly input: unknown;
  /**
   The mapper, as the caller supplied it.
   */
  readonly mapper: unknown;
  /**
   Concurrency bound, `stopOnError` policy, and optional abort signal.
   */
  readonly options?: MapOptions;
};

//endregion Types

//region Entries

/**
 Settles one value of the input into an entry.

 @param key - Key the value belongs to.

 @param pending - The value as a native promise, so the handler is attached
 right away, when this function is called.

 @returns The key with the value fulfilled or the reason it rejected.

 @example
 ```ts
 const entry = await settleEntry({ key: 'a', pending: Promise.resolve(1,), },);
 ```
 */
async function settleEntry(
  {
    key,
    pending,
  }: {
    /**
     Key the value belongs to.
     */
    readonly key: unknown;
    /**
     The value as a native promise.
     */
    readonly pending: Promise<unknown>;
  },
): Promise<KeyedEntry> {
  try {
    return [
      key,
      {
        status: 'fulfilled',
        value: await pending,
      },
    ];
  }
  catch (error) {
    return [
      key,
      {
        status: 'rejected',
        reason: error,
      },
    ];
  }
}

/**
 Whether an own property of the input is enumerable.

 @param input - Object whose own property is checked.

 @param key - Own property key, a string or a symbol.

 @returns `true` when the property is an own enumerable one; a key a getter
 deleted from the input before it is reached is not.

 @example
 ```ts
 isEnumerableOwn({ input: { a: 1, }, key: 'a', },); // => true
 ```
 */
function isEnumerableOwn(
  {
    input,
    key,
  }: {
    /**
     Object whose own property is checked.
     */
    readonly input: object;
    /**
     Own property key.
     */
    readonly key: string | symbol;
  },
): boolean {
  /**
   The property's descriptor, or nothing when the key is gone.
   */
  const descriptor = Object.getOwnPropertyDescriptor(
    input,
    key,
  );
  return descriptor?.enumerable === true;
}

/**
 Reads the entries of a `Map`, or else the own enumerable keys, strings and
 symbols, in the order and the way `Promise.allKeyed` does, all up front.

 Each value gets a handler right away, also like `Promise.allKeyed`, so a
 value that rejects before a mapper slot reaches it is not an unhandled
 rejection.

 @param input - The object or `Map` whose values are mapped.

 @returns One promise per entry, in order.

 @throws InvalidKeyedInputError when the input is neither an object nor a
 function.

 @example
 ```ts
 const entries = getKeyedEntries({ a: 1, },);
 ```
 */
function getKeyedEntries(input: unknown,): Promise<KeyedEntry>[] {
  if ((input === null) || (((typeof input) !== 'object') && ((typeof input) !== 'function')))
    throw new InvalidKeyedInputError(input,);

  /**
   Entry promises in reading order.
   */
  const entries: Promise<KeyedEntry>[] = [];

  /**
   Starts settling one value, reading it as the caller's input defines.

   @param key - Key the value belongs to.

   @param value - The value as the input holds it, possibly a promise.
   */
  function addEntry(
    {
      key,
      value,
    }: {
      /**
       Key the value belongs to.
       */
      readonly key: unknown;
      /**
       The value as the input holds it.
       */
      readonly value: unknown;
    },
  ): void {
    // Resolve the whole entry, so the engine checks termination after the
    // value settles and before the mapper runs.
    entries.push(settleEntry({
      key,
      pending: Promise.resolve(value,),
    },),);
  }

  if (input instanceof Map)
    for (const [
      key,
      value,
    ] of input)
      addEntry({
        key,
        value,
      },);
  else
    for (const key of Reflect.ownKeys(input,))
      if (isEnumerableOwn({
        input,
        key,
      },))
        addEntry({
          key,
          value: Reflect.get(
            input,
            key,
          ),
        },);

  return entries;
}

//endregion Entries

//region Run

/**
 The engine behind the keyed variants: the entries are read up front, then
 mapped concurrently, and the mapped entries become a `Map` or a null-prototype
 object.

 @param run - The caller's input, mapper, and options, destructured here so a
 failure rejects the returned promise.

 @param isSettled - Whether a rejected value or mapper gives a rejected
 result instead of rejecting the run.

 @returns The mapped entries as a new `Map` for a `Map` input, else an object
 with a `null` prototype, like `Promise.allKeyed`.

 @throws InvalidKeyedInputError when the input is not an object or a `Map`.

 @throws MapperRequiredError when the mapper is not a function.

 @example
 ```ts
 const mapped = await mapKeyed({
   run: { input: { a: 1, }, mapper: identity, },
   isSettled: false,
 },);
 ```
 */
async function mapKeyed(
  {
    run,
    isSettled,
  }: {
    /**
     The caller's run, untouched.
     */
    readonly run: LooseKeyedRun;
    /**
     Whether a rejected value or mapper gives a rejected result.
     */
    readonly isSettled: boolean;
  },
): Promise<unknown> {
  /**
   The caller's run destructured inside the async body, so a non-object run
   rejects the returned promise.
   */
  const {
    input,
    mapper,
    options,
  } = run;
  /**
   Settling entries, one per value, read before anything else is checked.
   */
  const entries = getKeyedEntries(input,);

  // After the values have their handlers, like the other argument errors,
  // which the engine checks. It cannot check this one, as it gets the wrapper
  // below as its mapper.
  validateMapper(mapper,);

  /**
   The caller's mapper under its loose runtime type.
   */
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the validated mapper is a function; the public signature carries the precise key and value types
  const callMapper = mapper as LooseKeyedMapper;

  /**
   Maps one entry, turning a failure into a rejected result when settling.

   @param entry - Key with the settled value from the input.

   @returns The sentinel the mapper returned, or the key with its result.

   @throws Whatever the value's rejection or the mapper threw, unless
   settling.
   */
  async function mapEntry(entry: KeyedEntry,): Promise<MappedEntry> {
    /**
     Key and settled value of this entry.
     */
    const [
      key,
      inputResult,
    ] = entry;

    try {
      if (inputResult.status === 'rejected')
        throw inputResult.reason;

      /**
       The mapper's result for this value.
       */
      const result = await callMapper(
        inputResult.value,
        key,
      );

      if ((result === pMapSkip) || (result === pMapStop))
        return result;

      return [
        key,
        isSettled
          ? {
            status: 'fulfilled',
            value: result,
          }
          : result,
      ];
    }
    catch (error) {
      if (!isSettled)
        throw error;

      return [
        key,
        {
          status: 'rejected',
          reason: error,
        },
      ];
    }
  }

  /**
   The mapped entries in input order, without the skipped and the stopped.
   */
  const mappedEntries = await mapConcurrently<KeyedEntry, MappedEntry, MappedPair>({
    run: {
      iterable: entries,
      mapper: mapEntry,
      ...(options === undefined
        ? {}
        : { options, }),
    },
    isSettled: false,
  },);

  if (input instanceof Map)
    return new Map(mappedEntries,);

  // A null prototype, like `Promise.allKeyed`, so a key like `__proto__` or
  // `toString` is an own property like any other.
  return Object.setPrototypeOf(
    Object.fromEntries(mappedEntries,),
    null,
  );
}

/**
 Runs a version of `pMap` for the values of an object or a `Map`, like
 `Promise.allKeyed()` is for `Promise.all()`.

 For an object it uses the own enumerable keys, strings and symbols; for a
 `Map`, the entries, and a key can be any value. All the values are read right
 away, and each value is awaited before the mapper is called for it. A value
 that rejects gets a handler right away, so it is never an unhandled
 rejection, even when `concurrency` has not reached it yet. A `pMapSkip` result
 leaves out the key, and `pMapStop` keeps only the keys before it.

 @typeParam Input - object or `Map` whose values are mapped

 @typeParam NewValue - value type the mapper returns, sentinels included

 @param run - The object or `Map`, the mapper called with each value and its
 key, and the options.

 @returns Promise settling with an object with the same keys, in the same
 order, and the values the mapper returned, or rejecting if any value or
 mapper rejects. The object has a `null` prototype; for a `Map` it is a new
 `Map`.

 @throws InvalidKeyedInputError when the input is not an object or a `Map`;
 rejected through the returned promise.

 @throws MapperRequiredError when the mapper is not a function; rejected
 through the returned promise.

 @example
 ```ts
 import { pMapKeyed, } from '\@monochromatic-dev/upstream-fork-p-map';

 const { user, posts, } = await pMapKeyed({
   input: {
     user: 'https://example.com/api/user',
     posts: 'https://example.com/api/posts',
   },
   mapper: async function readJson(url: string,): Promise<unknown> {
     return (await fetch(url,)).json();
   },
   options: { concurrency: 2, },
 });
 ```
 */
export function pMapKeyed<Input extends object, NewValue extends KeyedNewValue>(
  run: KeyedMapRun<Input, NewValue>,
): Promise<KeyedResult<Input, NewValue, Exclude<NewValue, MapControl>>> {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the engine builds the object or `Map` the precise return type describes
  return mapKeyed({
    run,
    isSettled: false,
  },) as Promise<KeyedResult<Input, NewValue, Exclude<NewValue, MapControl>>>;
}

/**
 Runs a version of {@link pMapKeyed} for `Promise.allSettledKeyed()`: an error
 from the mapper, or a value of the input that rejects, does not reject the
 run.

 A value that rejects gets a `{ status: 'rejected', reason }` result and the
 mapper is not called for it. The run still rejects when reading the input
 throws, and when the `signal` aborts.

 @typeParam Input - object or `Map` whose values are mapped

 @typeParam NewValue - value type the mapper returns, sentinels included

 @param run - The object or `Map`, the mapper called with each value and its
 key, and the options without `stopOnError`.

 @returns Promise settling with an object with the same keys, in the same
 order, each value a `PromiseSettledResult`. The object has a `null`
 prototype; for a `Map` it is a new `Map`.

 @throws InvalidKeyedInputError when the input is not an object or a `Map`;
 rejected through the returned promise.

 @throws MapperRequiredError when the mapper is not a function; rejected
 through the returned promise.

 @example
 ```ts
 import { pMapSettledKeyed, } from '\@monochromatic-dev/upstream-fork-p-map';

 const { user, posts, } = await pMapSettledKeyed({
   input: {
     user: 'https://example.com/api/user',
     posts: 'https://example.com/api/posts',
   },
   mapper: async function readJson(url: string,): Promise<unknown> {
     return (await fetch(url,)).json();
   },
   options: { concurrency: 2, },
 });
 ```
 */
export function pMapSettledKeyed<Input extends object, NewValue extends KeyedNewValue>(
  run: SettledKeyedMapRun<Input, NewValue>,
): Promise<KeyedResult<Input, NewValue, PromiseSettledResult<Exclude<NewValue, MapControl>>>> {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the engine builds the object or `Map` the precise return type describes
  return mapKeyed({
    run,
    isSettled: true,
  },) as Promise<KeyedResult<Input, NewValue, PromiseSettledResult<Exclude<NewValue, MapControl>>>>;
}

//endregion Run
