/**
 Shared fixtures for the mutation-derived tests (`./mutation-*.unit.test.ts`).

 @module
 */

/**
 Minimal utils view the custom merge functions read.
 */
export type ActionUtils = {
  readonly actions: {
    readonly defaultMerge: symbol;
    readonly skip: symbol;
  };
};

/**
 Minimal utils view the custom into merge functions read; into has no skip.
 */
export type IntoActionUtils = {
  readonly actions: {
    readonly defaultMerge: symbol;
  };
};

/**
 Value a custom mergeOthers returns so a test can tell it ran.
 */
export const OTHERS_MARK = 'custom mergeOthers ran';

/**
 Custom `filterValues` removing strings, so a key holding only strings keeps
 no values at all.

 @param values - Values at one position.

 @returns Values that are not strings.

 @example
 ```ts
 keepNonStrings(['a', 1,]); // [1]
 ```
 */
export function keepNonStrings(values: readonly unknown[],): unknown[] {
  return values.filter(function isNotString(value,) {
    return (typeof value) !== 'string';
  },);
}

/**
 Custom merge function that defers by returning `undefined`, which only
 `enableImplicitDefaultMerging` turns into the default merge.

 @returns Nothing.

 @example
 ```ts
 deepmergeCustom({ mergeOthers: returnsUndefined, });
 ```
 */
export function returnsUndefined(): undefined {
  return undefined;
}

/**
 Custom mergeOthers marking that it was called.

 @returns {@link OTHERS_MARK}.

 @example
 ```ts
 deepmergeCustom({ mergeOthers: markOthers, })(1, 2,); // OTHERS_MARK
 ```
 */
export function markOthers(): string {
  return OTHERS_MARK;
}

/**
 Record whose `self` key references the record itself.

 @param extra - Leaf stored beside the cycle, so two such records differ.

 @returns New cyclic record.

 @example
 ```ts
 const looped = cyclicSelf(1,); // looped.self === looped
 ```
 */
export function cyclicSelf(extra: unknown,): Record<string, unknown> {
  /**
   Record under construction.
   */
  const record: Record<string, unknown> = { extra, };
  record.self = record;
  return record;
}

/**
 Read one property of a value whose static type is opaque.

 @param holder - Value expected to be an object.

 @param key - Property to read.

 @returns Property value.

 @throws When `holder` is not an object, which means the merge shape changed.

 @example
 ```ts
 read({ holder: { a: 1, }, key: 'a', }); // 1
 ```
 */
export function read(
  {
    holder,
    key,
  }: {
    readonly holder: unknown;
    readonly key: PropertyKey;
  },
): unknown {
  if (((typeof holder) !== 'object') || (holder === null))
    throw new Error(`expected an object holding ${String(key,)}, got ${String(holder,)}`,);
  /**
   Read value, typed `unknown` rather than `any`.
   */
  const value: unknown = Reflect.get(
    holder,
    key,
  );
  return value;
}

/**
 Record with an own enumerable `__proto__` data key, as `JSON.parse` builds.

 @param inner - Value stored under `__proto__`.

 @returns Record whose prototype is still `Object.prototype`.

 @example
 ```ts
 Object.hasOwn(ownProto({ p: 1, },), '__proto__',); // true
 ```
 */
export function ownProto(inner: unknown,): Record<string, unknown> {
  /**
   Record under construction.
   */
  const record: Record<string, unknown> = {};
  Object.defineProperty(
    record,
    '__proto__',
    {
      configurable: true,
      enumerable: true,
      value: inner,
      writable: true,
    },
  );
  return record;
}
