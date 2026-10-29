/**
 Expected merge kinds that generators attach to objects the model cannot
 classify from the documentation alone.

 deepmerge-ts decides whether an object with a non-plain prototype merges
 as a record by an undocumented heuristic (its `isRecord`). The model
 (`./model.ts`) classifies by prototype only, so a generator that builds
 such objects registers the kind the accepted characterization in
 `./exotic-behaviour.unit.test.ts` and `./mutation-record.unit.test.ts`
 pins for each shape, and `kindOf` returns it.

 @module
 */

import type { ValueKind, } from './model.ts';

/**
 Returned by {@link expectedKindOf} for objects no generator registered.
 */
export const NO_EXPECTED_KIND: unique symbol = Symbol('object without a registered expected merge kind',);

/**
 Kinds registered by generators, keyed by object identity.
 */
const EXPECTED_KINDS = new WeakMap<object, ValueKind>();

/**
 Register the kind a generated object must merge as.

 @param value - Generated object.

 @param kind - Kind the accepted characterization pins for its shape.

 @returns `value`, so a generator can register inline.

 @example
 ```ts
 const recordLike = withExpectedKind({ value: Object.create(moduleTagged), kind: 'record', });
 ```
 */
export function withExpectedKind<const TValue extends object,>(
  {
    value,
    kind,
  }: {
    readonly value: TValue;
    readonly kind: ValueKind;
  },
): TValue {
  EXPECTED_KINDS.set(
    value,
    kind,
  );
  return value;
}

/**
 Kind a generator registered for an object.

 @param value - Object to look up.

 @returns Registered kind, or {@link NO_EXPECTED_KIND}.

 @example
 ```ts
 expectedKindOf({}); // NO_EXPECTED_KIND
 ```
 */
export function expectedKindOf(value: object,): ValueKind | typeof NO_EXPECTED_KIND {
  return EXPECTED_KINDS.get(value,) ?? NO_EXPECTED_KIND;
}
