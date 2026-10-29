/**
 fast-check generators for exotic merge inputs: value kinds the tree
 generators in `./arbitraries.ts` never produce.

 Leaves: typed arrays, `ArrayBuffer`, `DataView`, boxed primitives, errors,
 weak collections, promises, and functions carrying own properties.
 Containers: Proxy-wrapped records and arrays, arrays from another realm
 (holding primitives only), subclass instances of Array, Set, and Map, and
 record-like objects over non-plain prototypes (`./exotic-record-like.ts`),
 which merge as records or leaves by deepmerge-ts's record heuristic.

 Deliberately absent, each pinned elsewhere instead:
 cross-realm records (the model classifies them as leaves, deepmerge-ts as
 records, and the model is the one that is wrong),
 cross-realm Sets and Maps (a known defect, see
 `./known-defect-exotic.unit.test.ts`),
 and `arguments` objects (their strict-mode `callee` accessor throws when the
 structural comparison reads it; `./exotic-behaviour.unit.test.ts` covers them).

 @module
 */

import {
  createContext,
  runInContext,
} from 'node:vm';

import {
  array,
  boolean,
  constant,
  constantFrom,
  integer,
  letrec,
  oneof,
  string,
  tuple,
  uniqueArray,
  type Arbitrary,
} from 'fast-check';

import { keyOfEntry, } from './arbitraries.ts';
import { recordLikeArbitrary, } from './exotic-record-like.ts';

//region Realms and subclasses

/**
 Second V8 realm whose built-ins differ by identity from this one's.
 */
const FOREIGN_REALM = createContext({},);

/**
 Evaluate source in {@link FOREIGN_REALM}.

 @param source - JavaScript expression.

 @returns Value built by the other realm's constructors.

 @example
 ```ts
 foreign('new Set([1])'); // a Set that fails `instanceof Set` here
 ```
 */
export function foreign(source: string,): unknown {
  return runInContext(
    source,
    FOREIGN_REALM,
  );
}

/**
 Marker constructor for Array subclass instances; only its prototype chain matters.
 */
function TaggedArraySubclass(): void {
  // Intentionally empty: `Reflect.construct` uses it only as `newTarget`.
}

/**
 Marker constructor for Set subclass instances; only its prototype chain matters.
 */
function TaggedSetSubclass(): void {
  // Intentionally empty: `Reflect.construct` uses it only as `newTarget`.
}

/**
 Marker constructor for Map subclass instances; only its prototype chain matters.
 */
function TaggedMapSubclass(): void {
  // Intentionally empty: `Reflect.construct` uses it only as `newTarget`.
}

// The layout `class Tagged extends Base {}` produces, without class syntax.
Object.setPrototypeOf(
  TaggedArraySubclass.prototype,
  Array.prototype,
);
Object.setPrototypeOf(
  TaggedSetSubclass.prototype,
  Set.prototype,
);
Object.setPrototypeOf(
  TaggedMapSubclass.prototype,
  Map.prototype,
);

/**
 Error for a subclass construction that produced the wrong built-in.
 */
export class TaggedConstructionError extends Error {
  /**
   @param kind - Built-in that was expected.
   */
  constructor(kind: string,) {
    super(`Reflect.construct did not produce a ${kind}`,);
    this.name = 'TaggedConstructionError';
  }
}

/**
 Array subclass instance holding the given items.

 @param items - Elements, in order.

 @returns Array whose prototype is a subclass of `Array.prototype`.

 @throws {@link TaggedConstructionError} Never for a working engine.

 @example
 ```ts
 Array.isArray(taggedArray([1,])); // true
 ```
 */
export function taggedArray(items: readonly unknown[],): unknown[] {
  /**
   Empty subclass instance; items are pushed so a lone number is not a length.
   */
  const built: unknown = Reflect.construct(
    Array,
    [],
    TaggedArraySubclass,
  );
  if (!Array.isArray(built,))
    throw new TaggedConstructionError('Array',);
  built.push(...items,);
  return built;
}

/**
 Set subclass instance holding the given items.

 @param items - Elements, in order.

 @returns Set whose prototype is a subclass of `Set.prototype`.

 @throws {@link TaggedConstructionError} Never for a working engine.

 @example
 ```ts
 taggedSet([1,]) instanceof Set; // true
 ```
 */
export function taggedSet(items: readonly unknown[],): Set<unknown> {
  /**
   Subclass instance built by the Set constructor.
   */
  const built: unknown = Reflect.construct(
    Set,
    [items,],
    TaggedSetSubclass,
  );
  if (!(built instanceof Set))
    throw new TaggedConstructionError('Set',);
  return built;
}

/**
 Map subclass instance holding the given entries.

 @param entries - Key and value pairs, in order.

 @returns Map whose prototype is a subclass of `Map.prototype`.

 @throws {@link TaggedConstructionError} Never for a working engine.

 @example
 ```ts
 taggedMap([['k', 1,],]) instanceof Map; // true
 ```
 */
export function taggedMap(entries: readonly (readonly [
  unknown,
  unknown,
])[],): Map<unknown, unknown> {
  /**
   Subclass instance built by the Map constructor.
   */
  const built: unknown = Reflect.construct(
    Map,
    [entries,],
    TaggedMapSubclass,
  );
  if (!(built instanceof Map))
    throw new TaggedConstructionError('Map',);
  return built;
}

//endregion Realms and subclasses

//region Leaves

/**
 Exotic leaves: every one must resolve to the last value, by identity.
 */
export const exoticLeafArbitrary: Arbitrary<unknown> = oneof(
  integer({
    min: 0,
    max: 255,
  },)
    .map(function toTypedArray(byte,) {
      return new Uint8Array([byte,],);
    },),
  constant(null,)
    .map(function toBuffer() {
      return new ArrayBuffer(2,);
    },),
  constant(null,)
    .map(function toDataView() {
      return new DataView(new ArrayBuffer(2,),);
    },),
  string({ maxLength: 2, },)
    .map(function toBoxedString(text,) {
    // oxlint-disable-next-line unicorn/new-for-builtins, no-new-wrappers -- a boxed primitive is the leaf under test.
      return new String(text,);
    },),
  integer({
    min: -2,
    max: 2,
  },)
    .map(function toBoxedNumber(value,) {
    // oxlint-disable-next-line unicorn/new-for-builtins, no-new-wrappers -- a boxed primitive is the leaf under test.
      return new Number(value,);
    },),
  boolean()
    .map(function toBoxedBoolean(value,) {
    // oxlint-disable-next-line unicorn/new-for-builtins, no-new-wrappers -- a boxed primitive is the leaf under test.
      return new Boolean(value,);
    },),
  string({ maxLength: 3, },)
    .map(function toError(message,) {
      return new Error(message,);
    },),
  constant(null,)
    .map(function toWeakMap() {
      return new WeakMap();
    },),
  constant(null,)
    .map(function toWeakSet() {
      return new WeakSet();
    },),
  integer()
    .map(function toPromise(value,) {
      return Promise.resolve(value,);
    },),
  integer()
    .map(function toFunctionWithProps(value,) {
      return Object.assign(
        function leafFunction(): number {
          return value;
        },
        { value, },
      );
    },),
);

/**
 Primitive leaves mixed in so containers also merge ordinary values.
 */
const primitiveArbitrary: Arbitrary<unknown> = oneof(
  constant(null,),
  boolean(),
  integer({
    min: -2,
    max: 9,
  },),
  string({
    maxLength: 2,
    unit: constantFrom(
      'a',
      'b',
    ),
  },),
);

//endregion Leaves

//region Trees

/**
 Record keys; a small pool so inputs overlap.
 */
const keyArbitrary = constantFrom(
  'a',
  'b',
  'c',
);

/**
 Recursive generator scope over exotic leaves and containers.
 */
const exoticScope = letrec<{
  value: unknown;
  container: unknown;
  record: object;
  recordLike: object;
}>(function build(tie,) {
  /**
   Record entries shared by the plain and Proxy-wrapped record shapes.
   */
  const entries = uniqueArray(
    tuple(
      keyArbitrary,
      tie('value',),
    ),
    {
      maxLength: 3,
      selector: keyOfEntry,
    },
  );
  /**
   Children shared by the array, Set, and Map shapes.
   */
  const items = array(
    tie('value',),
    { maxLength: 3, },
  );
  return {
    value: oneof(
      {
        maxDepth: 3,
        depthIdentifier: 'exotic',
      },
      primitiveArbitrary,
      exoticLeafArbitrary,
      tie('container',),
    ),
    container: oneof(
      { depthIdentifier: 'exotic', },
      tie('record',),
      // Record-likes merge as records or leaves; never a root record, where a leaf-kind source
      // would hit the known deepmergeInto root no-op (`./known-defect-exotic.unit.test.ts`).
      tie('recordLike',),
      items,
      items.map(taggedArray,),
      items.map(function toProxyArray(values,) {
        return new Proxy(
          values,
          {},
        );
      },),
      array(
        primitiveArbitrary,
        { maxLength: 3, },
      )
        .map(function toForeignArray(values,) {
          return foreign(JSON.stringify(values,),);
        },),
      items.map(function toSet(values,) {
        return new Set(values,);
      },),
      items.map(taggedSet,),
      uniqueArray(
        tuple(
          keyArbitrary,
          tie('value',),
        ),
        {
          maxLength: 3,
          selector: keyOfEntry,
        },
      )
        .map(function toMap(pairs,) {
          return new Map(pairs,);
        },),
      uniqueArray(
        tuple(
          keyArbitrary,
          tie('value',),
        ),
        {
          maxLength: 3,
          selector: keyOfEntry,
        },
      )
        .map(taggedMap,),
    ),
    record: oneof(
      entries.map(function toRecord(pairs,) {
        return Object.fromEntries(pairs,);
      },),
      entries.map(function toProxyRecord(pairs,) {
        return new Proxy(
          Object.fromEntries(pairs,),
          {},
        );
      },),
    ),
    recordLike: recordLikeArbitrary(entries,),
  };
},);

/**
 Any exotic tree value.
 */
export const exoticTreeArbitrary: Arbitrary<unknown> = exoticScope.value;

/**
 Exotic trees rooted at a (possibly Proxy-wrapped) record.
 */
export const exoticRecordArbitrary: Arbitrary<object> = exoticScope.record;

/**
 Record-like objects over non-plain prototypes (`./exotic-record-like.ts`).
 */
export const exoticRecordLikeArbitrary: Arbitrary<object> = exoticScope.recordLike;

/**
 Argument lists: records most of the time, otherwise any exotic trees, or
 records and record-likes side by side, where a record-like's kind decides
 between merging and taking the last value.
 */
export const exoticArgumentsArbitrary: Arbitrary<readonly unknown[]> = oneof(
  {
    weight: 3,
    arbitrary: array(
      exoticRecordArbitrary,
      {
        minLength: 1,
        maxLength: 3,
      },
    ),
  },
  {
    weight: 1,
    arbitrary: array(
      exoticTreeArbitrary,
      {
        minLength: 1,
        maxLength: 3,
      },
    ),
  },
  {
    weight: 1,
    arbitrary: array(
      oneof(
        exoticRecordArbitrary,
        exoticRecordLikeArbitrary,
      ),
      {
        minLength: 2,
        maxLength: 3,
      },
    ),
  },
);

//endregion Trees
