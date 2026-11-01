/**
 Record-like objects for the exotic generators (`./exotic-arbitraries.ts`):
 own data over a non-plain prototype, in the shapes whose merge kind
 deepmerge-ts's undocumented record heuristic decides.

 Each shape registers the kind the accepted characterization pins
 (`./exotic-behaviour.unit.test.ts`, `./mutation-record.unit.test.ts`) with
 `./expected-kind.ts`, so the model classifies it the same way:

 - records: a `Module`-tagged prototype over `Object.prototype`; a
   null-prototype `Module`-tagged prototype whose constructor points back to
   it and which owns `isPrototypeOf`; a prototype without a constructor; an
   ordinary object as the prototype;
 - leaves: a constructor whose prototype is an array owning `isPrototypeOf`;
   a class-like constructor whose prototype lacks an own `isPrototypeOf`; a
   `Module`-tagged constructor prototype lacking one; a custom-tagged
   prototype; a constructor whose prototype is `null`.

 @module
 */

import {
  constantFrom,
  tuple,
  type Arbitrary,
} from 'fast-check';

import {
  expectedKindOf,
  NO_EXPECTED_KIND,
  withExpectedKind,
} from './expected-kind.ts';
import {
  kindOf,
  type ValueKind,
} from './model.ts';

/**
 Own `isPrototypeOf` for hand-built prototypes; the record heuristic only
 asks whether the key is own, never calls it.

 @returns Always false.

 @example
 ```ts
 Object.assign(prototype, { isPrototypeOf: isPrototypeOfStub, });
 ```
 */
function isPrototypeOfStub(): boolean {
  return false;
}

/**
 Object with the given prototype and own properties.

 @param prototype - Prototype of the new object.

 @param properties - Own properties to copy on.

 @returns New object.

 @example
 ```ts
 inheriting({ prototype: { inherited: 1, }, properties: { a: 1, }, });
 ```
 */
function inheriting(
  {
    prototype,
    properties,
  }: {
    readonly prototype: object;
    readonly properties: Readonly<Record<PropertyKey, unknown>>;
  },
): object {
  /**
   Object under construction.
   */
  const made = {};
  Reflect.setPrototypeOf(
    made,
    prototype,
  );
  return Object.assign(
    made,
    properties,
  );
}

/**
 Null-prototype object with the given own properties.

 @param properties - Own properties to copy on.

 @returns Object whose prototype is `null`.

 @example
 ```ts
 bare({ a: 1, });
 ```
 */
function bare(properties: Readonly<Record<PropertyKey, unknown>>,): object {
  /**
   Object under construction.
   */
  const made = {};
  Reflect.setPrototypeOf(
    made,
    null,
  );
  return Object.assign(
    made,
    properties,
  );
}

/**
 Null-prototype `Module`-tagged prototype whose `constructor.prototype` is
 itself and which owns `isPrototypeOf`.
 */
const MODULE_LIKE: object = bare({
  [Symbol.toStringTag]: 'Module',
  isPrototypeOf: isPrototypeOfStub,
},);
Object.assign(
  MODULE_LIKE,
  { constructor: { prototype: MODULE_LIKE, }, },
);

/**
 Class-like prototype: its constructor's prototype is itself, an ordinary
 object without an own `isPrototypeOf`.
 */
const CLASS_LIKE: object = {};
Object.assign(
  CLASS_LIKE,
  { constructor: { prototype: CLASS_LIKE, }, },
);

/**
 One record-like shape: its prototype and the kind the characterization pins.
 */
type RecordLikeShape = {
  readonly name: string;
  readonly prototype: object;
  readonly kind: ValueKind;
};

/**
 Every generated shape.
 */
const SHAPES: readonly RecordLikeShape[] = [
  {
    kind: 'record',
    name: 'Module-tagged prototype over Object.prototype',
    prototype: { [Symbol.toStringTag]: 'Module', },
  },
  {
    kind: 'record',
    name: 'null-prototype Module-like prototype',
    prototype: MODULE_LIKE,
  },
  {
    kind: 'record',
    name: 'prototype without a constructor',
    prototype: bare({},),
  },
  {
    kind: 'record',
    name: 'ordinary object prototype',
    prototype: { inherited: 1, },
  },
  {
    kind: 'other',
    name: 'constructor prototype is an array owning isPrototypeOf',
    prototype: {
      constructor: {
        prototype: Object.assign(
          [],
          { isPrototypeOf: isPrototypeOfStub, },
        ),
      },
    },
  },
  {
    kind: 'other',
    name: 'class-like constructor prototype',
    prototype: CLASS_LIKE,
  },
  {
    kind: 'other',
    name: 'Module-tagged constructor prototype without isPrototypeOf',
    prototype: { constructor: { prototype: { [Symbol.toStringTag]: 'Module', }, }, },
  },
  {
    kind: 'other',
    name: 'custom-tagged prototype',
    prototype: { [Symbol.toStringTag]: 'Custom', },
  },
  {
    kind: 'other',
    name: 'constructor prototype is null',
    prototype: { constructor: { prototype: null, }, },
  },
];

/**
 Record-like objects: generated own entries over one of the shapes'
 prototypes, registered with the shape's expected kind.

 @param entries - Own enumerable entries.

 @returns Generator of record-like objects.

 @example
 ```ts
 const recordLikes = recordLikeArbitrary(entries);
 ```
 */
export function recordLikeArbitrary(entries: Arbitrary<readonly (readonly [
  string,
  unknown,
])[]>,): Arbitrary<object> {
  return tuple(
    constantFrom(...SHAPES,),
    entries,
  )
    .map(function toRecordLike([shape, pairs,],) {
      return withExpectedKind({
        kind: shape.kind,
        value: inheriting({
          properties: Object.fromEntries(pairs,),
          prototype: shape.prototype,
        },),
      },);
    },);
}

/**
 Whether a value is a registered record-like object.

 @param value - Any merge value.

 @returns Whether a generator registered an expected kind for it.

 @example
 ```ts
 isRecordLike({}); // false
 ```
 */
function isRecordLike(value: unknown,): boolean {
  return ((typeof value) === 'object')
    && (value !== null)
    && (expectedKindOf(value,) !== NO_EXPECTED_KIND);
}

/**
 Whether some merge position holds a record-like object beside another
 value, where its kind decides between merging and taking the last value.
 Positions are followed key-wise through records only, as the merge does.

 @param values - Merge arguments, trees only.

 @returns Whether a record-like meets another value somewhere.

 @example
 ```ts
 meetsRecordLike([{ a: recordLike, }, { a: {}, },]);
 ```
 */
export function meetsRecordLike(values: readonly unknown[],): boolean {
  /**
   Positions still to inspect, each the values merged there.
   */
  const pending: (readonly unknown[])[] = [values,];
  for (let next = pending.pop(); next !== undefined; next = pending.pop()) {
    /**
     Values the default filter keeps.
     */
    const present = next.filter(function isPresent(value,) {
      return value !== undefined;
    },);
    if ((present.length >= 2) && present.some(isRecordLike,))
      return true;
    /**
     Record values, when every value is one.
     */
    const records = present.filter(function isRecord(value,): value is object {
      return kindOf(value,) === 'record';
    },);
    if (records.length === present.length) {
      for (const key of new Set(records.flatMap(function keysOf(record,) {
        return Object.keys(record,);
      },),)) {
        pending.push(records
          .filter(function holds(record,) {
            return Object.hasOwn(
              record,
              key,
            );
          },)
          .map(function valueOf(record,): unknown {
            return Reflect.get(
              record,
              key,
            );
          },),);
      }
    }
  }
  return false;
}
