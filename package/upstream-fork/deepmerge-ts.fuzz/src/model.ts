/**
 Reference model of deepmerge-ts's default merge semantics.

 Written from the documented behaviour, not from upstream's source: the README
 example and `docs/API.md` for records, arrays, Sets, Maps, `undefined`
 filtering, last-value replacement, and `maxDepth`. The model deliberately
 omits cycle handling; properties that compare against it generate trees only.

 Documented rules the model encodes:

 - `undefined` inputs are dropped before merging (default `filterValues`);
   nothing left means `undefined`.
 - One remaining value is returned as-is (same reference).
 - At or beyond `maxDepth` the last value wins, with no signal (Q10).
 - Values of differing kinds, or of kind `other`, resolve to the last value.
 - Records merge key-wise over own enumerable string keys, then own
   enumerable symbol keys, in first-seen order; the result is a fresh plain
   object.
 - Arrays concatenate, Sets union, Maps merge value-wise per key.

 Array holes follow concatenation (length is the sum of input lengths); the
 installed implementation drops them, a divergence under an upstream intent
 question (Q11) that the properties avoid by generating dense arrays.

 Every case built from this model is JSON-serializable when its inputs are,
 so a later export can feed the Rust unified-linter merge (Q13).

 @module
 */

import {
  expectedKindOf,
  NO_EXPECTED_KIND,
} from './expected-kind.ts';
import {
  isAnyMap,
  isAnySet,
  isObjectPrototype,
} from './realm.ts';

/**
 Merge category of a value, mirroring the documented type buckets.
 */
export type ValueKind = 'array' | 'map' | 'other' | 'record' | 'set';

/**
 Default `maxDepth` documented in `docs/API.md`.
 */
export const DEFAULT_MAX_DEPTH = 1_000;

/**
 Classify a value into its documented merge bucket. Plain and null-prototype
 objects are records; class instances, dates, and other objects are leaves.
 Buckets are realm-independent (`./realm.ts`): a Set or plain object from
 `node:vm` classifies like a local one. Objects with other prototypes that a
 generator registered (`./expected-kind.ts`) take the registered kind.

 @param value - Any merge input or output, inspected by prototype only.

 @returns Bucket that decides how the value merges.

 @example
 ```ts
 kindOf(Object.create(null)); // 'record'
 ```
 */
export function kindOf(value: unknown,): ValueKind {
  if (((typeof value) !== 'object') || (value === null))
    return 'other';
  if (Array.isArray(value,))
    return 'array';
  /**
   Kind a generator registered for this object, if any.
   */
  const expected = expectedKindOf(value,);
  if (expected !== NO_EXPECTED_KIND)
    return expected;
  /**
   Prototype deciding whether an object counts as a plain record.
   */
  const prototype: unknown = Object.getPrototypeOf(value,);
  if ((prototype === null) || isObjectPrototype(prototype,))
    return 'record';
  if (isAnySet(value,))
    return 'set';
  if (isAnyMap(value,))
    return 'map';
  return 'other';
}


/**
 Whether a record holds `key` as an own enumerable property, the merge's
 membership test.

 @param record - Record to probe.

 @param key - Key to look up.

 @returns Whether the merge visits `key` on `record`.

 @example
 ```ts
 hasEnumerable({ record: { a: 1, }, key: 'a', }); // true
 ```
 */
function hasEnumerable(
  {
    record,
    key,
  }: {
    readonly record: object;
    readonly key: PropertyKey;
  },
): boolean {
  return Object.prototype
    .propertyIsEnumerable
    .call(
    record,
    key,
    );
}

/**
 Own enumerable keys of a record in documented merge order: string keys as
 `Object.keys` lists them, then enumerable symbols.

 @param record - Record whose merge keys are read without invoking getters.

 @returns Keys the merge visits for this record.

 @example
 ```ts
 recordKeys({ b: 1, a: 2, }); // ['b', 'a']
 ```
 */
export function recordKeys(record: object,): readonly PropertyKey[] {
  /**
   Enumerable own symbols, which `Object.keys` omits.
   */
  const symbols = Object
    .getOwnPropertySymbols(record,)
    .filter(function isEnumerableSymbol(symbol,) {
      return hasEnumerable({
        record,
        key: symbol,
      },);
    },);
  return [
    ...Object.keys(record,),
    ...symbols,
  ];
}

/**
 Read one property the way a merge reads it: through `[[Get]]`, so getters run.

 @param record - Record holding the key.

 @param key - Key known to be an own enumerable property.

 @returns Current value, flattened from any accessor.

 @example
 ```ts
 readKey({ record: { a: 1, }, key: 'a', }); // 1
 ```
 */
function readKey(
  {
    record,
    key,
  }: {
    readonly record: object;
    readonly key: PropertyKey;
  },
): unknown {
  return Reflect.get(
    record,
    key,
  );
}

/**
 Define an own enumerable data property, never triggering the
 `__proto__` setter that plain assignment would.

 @param target - Fresh result record being filled.

 @param key - Key to define.

 @param value - Merged value to store.

 @example
 ```ts
 defineData({ target: {}, key: '__proto__', value: 1, });
 ```
 */
function defineData(
  {
    target,
    key,
    value,
  }: {
    readonly target: object;
    readonly key: PropertyKey;
    readonly value: unknown;
  },
): void {
  Object.defineProperty(
    target,
    key,
    {
      configurable: true,
      enumerable: true,
      value,
      writable: true,
    },
  );
}

/**
 Merges the values found at one child position (a record key or Map key);
 injected so the bucket steps can be declared before the recursive step.
 */
type MergeChild = (values: readonly unknown[],) => unknown;

/**
 Merge records key-wise into a fresh plain object.

 @param records - Same-bucket inputs, already filtered.

 @param mergeChild - Merges each key's values one level deeper.

 @returns Fresh plain object with every visited key.

 @example
 ```ts
 mergeRecords({ records: [{ a: 1, }, { a: 2, },], mergeChild: function last(values) { return values.at(-1); }, });
 ```
 */
function mergeRecords(
  {
    records,
    mergeChild,
  }: {
    readonly records: readonly object[];
    readonly mergeChild: MergeChild;
  },
): object {
  /**
   Union of merge keys in first-seen order.
   */
  const keys = [
    ...new Set(records.flatMap(function keysOf(record,) {
      return recordKeys(record,);
    },),),
  ];
  return keys.reduce<object>(
    function addKey(
      result,
      key,
    ) {
      /**
       Values of this key from every record that has it enumerably.
       */
      const keyValues = records
        .filter(function holdsKey(record,) {
          return hasEnumerable({
            record,
            key,
          },);
        },)
        .map(function valueOf(record,) {
          return readKey({
            record,
            key,
          },);
        },);
      defineData({
        target: result,
        key,
        value: mergeChild(keyValues,),
      },);
      return result;
    },
    {},
  );
}

/**
 Merge Maps per key into a fresh Map, keys in first-seen order.

 @param maps - Same-bucket inputs, already filtered.

 @param mergeChild - Merges each key's values one level deeper.

 @returns Fresh Map holding each key's merged value.

 @example
 ```ts
 mergeMaps({ maps: [new Map([[1, 'a',],]),], mergeChild: function last(values) { return values.at(-1); }, });
 ```
 */
function mergeMaps(
  {
    maps,
    mergeChild,
  }: {
    readonly maps: readonly ReadonlyMap<unknown, unknown>[];
    readonly mergeChild: MergeChild;
  },
): Map<unknown, unknown> {
  /**
   Union of Map keys in first-seen order.
   */
  const keys = [
    ...new Set(maps.flatMap(function keysOf(map,) {
      return [...map.keys(),];
    },),),
  ];
  return new Map(keys.map(function entryOf(key,): readonly [
    unknown,
    unknown,
  ] {
    /**
     Values of this key from every Map holding it.
     */
    const keyValues = maps
      .filter(function holdsKey(map,) {
        return map.has(key,);
      },)
      .map(function valueOf(map,) {
        return map.get(key,);
      },);
    return [
      key,
      mergeChild(keyValues,),
    ];
  },),);
}

/**
 Same-bucket inputs tagged by bucket, so each merge step receives its own
 element type without an assertion; `mixed` means the last value wins.
 */
type Bucket =
  | {
    readonly kind: 'array';
    readonly items: readonly (readonly unknown[])[];
  }
    | {
      readonly kind: 'map';
      readonly items: readonly ReadonlyMap<unknown, unknown>[];
    }
    | {
      readonly kind: 'mixed';
    }
    | {
      readonly kind: 'record';
      readonly items: readonly object[];
    }
    | {
      readonly kind: 'set';
      readonly items: readonly ReadonlySet<unknown>[];
    };

/**
 Bucket meaning the inputs share no mergeable kind.
 */
const MIXED: Bucket = { kind: 'mixed', };

/**
 Group inputs into one mergeable bucket.

 @param values - Filtered inputs, at least two.

 @returns Bucket when every input shares one mergeable kind, else {@link MIXED}.

 @example
 ```ts
 bucketOf([[1,], [2,],]); // { kind: 'array', items: [[1], [2]] }
 ```
 */
function bucketOf(values: readonly unknown[],): Bucket {
  /**
   Kind every input must share.
   */
  const kind = kindOf(values[0],);
  if (values.some(function differs(value,) {
    return kindOf(value,) !== kind;
  },))
    return MIXED;
  if (kind === 'array') {
    return {
      kind,
      items: values.filter(function isArray(value,) {
        return Array.isArray(value,);
      },),
    };
  }
  if (kind === 'set') {
    return {
      kind,
      items: values.filter(function isSet(value,) {
        return value instanceof Set;
      },),
    };
  }
  if (kind === 'map') {
    return {
      kind,
      items: values.filter(function isMap(value,) {
        return value instanceof Map;
      },),
    };
  }
  if (kind === 'record') {
    return {
      kind,
      items: values.filter(function isObject(value,): value is object {
        return ((typeof value) === 'object') && (value !== null);
      },),
    };
  }
  return MIXED;
}

/**
 Recursive merge step shared by every bucket.

 @param values - Inputs at this position, unfiltered.

 @param depth - Nesting depth of this position; the root is `0`.

 @param maxDepth - Depth at which merging stops and the last value wins.

 @returns Merged value for this position.

 @example
 ```ts
 mergeAt({ values: [{ a: 1, }, { b: 2, },], depth: 0, maxDepth: 1000, });
 ```
 */
function mergeAt(
  {
    values,
    depth,
    maxDepth,
  }: {
    readonly values: readonly unknown[];
    readonly depth: number;
    readonly maxDepth: number;
  },
): unknown {
  /**
   Inputs left after the default `undefined` filter.
   */
  const present = values.filter(function isPresent(value,) {
    return value !== undefined;
  },);
  if (present.length === 0)
    return undefined;
  /**
   Last present value, the result whenever merging stops.
   */
  const last = present.at(-1,);
  if ((depth >= maxDepth) || (present.length === 1))
    return last;
  /**
   Shared bucket of every input.
   */
  const bucket = bucketOf(present,);
  if (bucket.kind === 'mixed')
    return last;
  /**
   Child merge one level deeper, shared by records and Maps.

   @param childValues - Values found at one record key or Map key.

   @returns Merged value for that child position.
   */
  function mergeChild(childValues: readonly unknown[],): unknown {
    return mergeAt({
      values: childValues,
      depth: depth + 1,
      maxDepth,
    },);
  }
  if (bucket.kind === 'record') {
    return mergeRecords({
      records: bucket.items,
      mergeChild,
    },);
  }
  if (bucket.kind === 'map') {
    return mergeMaps({
      maps: bucket.items,
      mergeChild,
    },);
  }
  if (bucket.kind === 'array') {
    return bucket.items
      .flatMap(function elementsOf(array,) {
        return [...array,];
      },);
  }
  return new Set(bucket.items
    .flatMap(function elementsOf(set,) {
      return [...set,];
    },),);
}

/**
 Predict `deepmerge(...values)` (or `deepmergeCustom({ maxDepth })`) for tree
 inputs.

 @param values - Inputs in argument order.

 @param maxDepth - Stop depth; defaults to the documented `1000`.

 @returns Predicted merge result.

 @example
 ```ts
 modelMerge({ values: [{ a: [1,], }, { a: [2,], },], }); // { a: [1, 2] }
 ```
 */
export function modelMerge(
  {
    values,
    maxDepth = DEFAULT_MAX_DEPTH,
  }: {
    readonly values: readonly unknown[];
    readonly maxDepth?: number;
  },
): unknown {
  return mergeAt({
    values,
    depth: 0,
    maxDepth,
  },);
}
