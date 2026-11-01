/**
 Structural comparison and snapshots for merge inputs and outputs.

 `shapeMismatch` is stricter than a generic deep-equal on purpose: it
 compares prototypes, own-key order, array holes, Set and Map iteration
 order, and leaf identity (`Object.is`), because each is observable merge
 behaviour. `snapshot` copies the containers of a tree while keeping leaf
 identity, so an input can be checked for mutation after a merge and a
 fast-check value can be mutated without corrupting later shrink attempts.

 Both walk trees only; callers never pass cyclic values.

 @module
 */

import { kindOf, } from './model.ts';
import {
  isAnyMap,
  isAnySet,
} from './realm.ts';

/**
 Describe the key at one step of a mismatch path.

 @param key - Record key, Map key, or index.

 @returns Readable path segment.

 @example
 ```ts
 segment('a'); // '.a'
 ```
 */
function segment(key: unknown,): string {
  if ((typeof key) === 'symbol')
    return `[${String(key,)}]`;
  if ((typeof key) === 'number')
    return `[${String(key,)}]`;
  return `.${String(key,)}`;
}

/**
 Returned by {@link shapeMismatch} when the trees match; every real mismatch
 report starts with a path, so it is never empty.
 */
export const NO_MISMATCH = '';

/**
 Compare two ordered key lists by `Object.is`.

 @param left - Keys of the actual value.

 @param right - Keys of the expected value.

 @returns Whether both lists hold the same keys in the same order.

 @example
 ```ts
 sameKeys({ left: ['a',], right: ['a',], }); // true
 ```
 */
function sameKeys(
  {
    left,
    right,
  }: {
    readonly left: readonly unknown[];
    readonly right: readonly unknown[];
  },
): boolean {
  return (left.length === right.length) && left.every(function sameAt(
    key,
    index,
  ) {
    return Object.is(
      key,
      right[index],
    );
  },);
}

/**
 Whether a child comparison reported a mismatch.

 @param mismatch - Child report.

 @returns Whether the report is not {@link NO_MISMATCH}.

 @example
 ```ts
 isMismatch('$.a: prototype differs'); // true
 ```
 */
function isMismatch(mismatch: string,): boolean {
  return mismatch !== NO_MISMATCH;
}

/**
 Find the first structural difference between two trees.

 @param actual - Value produced by the implementation under test.

 @param expected - Value predicted by the model or captured by `snapshot`.

 @param path - Location of this pair, for the report.

 @returns Human-readable mismatch location and reason, or {@link NO_MISMATCH}
   when the trees match.

 @example
 ```ts
 shapeMismatch({ actual: [1,], expected: [1,], }); // ''
 ```
 */
export function shapeMismatch(
  {
    actual,
    expected,
    path = '$',
  }: {
    readonly actual: unknown;
    readonly expected: unknown;
    readonly path?: string;
  },
): string {
  if (Object.is(
    actual,
    expected,
  ))
    return NO_MISMATCH;
  /**
   Bucket of the actual value; the expected one must match it.
   */
  const kind = kindOf(actual,);
  if ((kind === 'other') || (kind !== kindOf(expected,))
    || ((typeof actual) !== 'object')
    || ((typeof expected) !== 'object')
    || (actual === null)
    || (expected === null))
    return `${path}: expected ${String(expected,)}, got ${String(actual,)}`;
  if (Object.getPrototypeOf(actual,) !== Object.getPrototypeOf(expected,))
    return `${path}: prototype differs`;
  if (isAnySet(actual,) && isAnySet(expected,)) {
    return sameKeys({
      left: [...actual,],
      right: [...expected,],
    },)
      ? NO_MISMATCH
      : `${path}: Set elements or order differ`;
  }
  if (isAnyMap(actual,) && isAnyMap(expected,)) {
    if (!sameKeys({
      left: [...actual.keys(),],
      right: [...expected.keys(),],
    },))
      return `${path}: Map keys or order differ`;
    return [...actual.keys(),]
      .map(function compareEntry(key,) {
        return shapeMismatch({
          actual: actual.get(key,),
          expected: expected.get(key,),
          path: `${path}${segment(key,)}`,
        },);
      },)
      .find(isMismatch,)
      ?? NO_MISMATCH;
  }
  /**
   Own keys including holes' absence and non-enumerable properties.
   */
  const actualKeys = Reflect.ownKeys(actual,);
  if (!sameKeys({
    left: actualKeys,
    right: Reflect.ownKeys(expected,),
  },)) {
    return `${path}: own keys differ (${actualKeys
      .map(String,)
      .join(', ',)})`;
  }
  return actualKeys
    .map(function compareProperty(key,) {
      return shapeMismatch({
        actual: Reflect.get(
          actual,
          key,
        ),
        expected: Reflect.get(
          expected,
          key,
        ),
        path: `${path}${segment(key,)}`,
      },);
    },)
    .find(isMismatch,)
    ?? NO_MISMATCH;
}

/**
 Copy every container of a tree, keeping leaves and property descriptors.

 Records keep their prototype, enumerability, and accessors; arrays keep
 holes; Sets keep elements by identity; Maps keep keys by identity and
 snapshot their values.

 @param value - Tree to copy.

 @returns Structurally identical tree sharing no container with `value`.

 @example
 ```ts
 const copy = snapshot({ a: [1,], });
 ```
 */
export function snapshot(value: unknown,): unknown {
  /**
   Bucket deciding how to copy this node.
   */
  const kind = kindOf(value,);
  if ((kind === 'other') || ((typeof value) !== 'object')
    || (value === null))
    return value;
  /**
   Prototype the copy inherits, so subclasses and other realms survive.
   */
  const prototype: unknown = Object.getPrototypeOf(value,);
  if ((prototype !== null) && ((typeof prototype) !== 'object'))
    throw new Error('snapshot: prototype is neither null nor an object',);
  if (isAnySet(value,) || isAnyMap(value,)) {
    /**
     Fresh collection: Set elements by identity, Map values snapshotted.
     */
    const collection = isAnySet(value,)
      ? new Set(value,)
      : new Map([...value,].map(function copyEntry([
        key,
        entry,
      ],): readonly [
        unknown,
        unknown,
      ] {
        return [
          key,
          snapshot(entry,),
        ];
      },),);
    Reflect.setPrototypeOf(
      collection,
      prototype,
    );
    return collection;
  }
  /**
   Fresh container with the same prototype as the original.
   */
  const copy: object = Array.isArray(value,) ? [] : {};
  Reflect.setPrototypeOf(
    copy,
    prototype,
  );
  for (const key of Reflect.ownKeys(value,)) {
    /**
     Original descriptor; data values are copied recursively.
     */
    const descriptor = Reflect.getOwnPropertyDescriptor(
      value,
      key,
    );
    if (descriptor === undefined)
      throw new Error(`snapshot: own key ${String(key,)} vanished during the copy`,);
    /**
     Data value to copy, absent for accessors.
     */
    const inner: unknown = descriptor.value;
    Reflect.defineProperty(
      copy,
      key,
      ('value' in descriptor)
        ? {
          ...descriptor,
          value: snapshot(inner,),
        }
        : descriptor,
    );
  }
  if (Object.isFrozen(value,))
    Object.freeze(copy,);
  return copy;
}

/**
 {@link snapshot} for callers that hold an object and must keep one, such as
 a `deepmergeInto` target.

 @param value - Object tree to copy.

 @returns Copy of `value`, narrowed back to an object.

 @throws When the copy is not an object, which {@link snapshot} never does
   for object input.

 @example
 ```ts
 const target = snapshotObject({ a: [1,], });
 ```
 */
export function snapshotObject(value: object,): object {
  /**
   Copy, typed `unknown` by {@link snapshot}.
   */
  const copy = snapshot(value,);
  if (((typeof copy) !== 'object') || (copy === null))
    throw new Error('snapshotObject: copy of an object is not an object',);
  return copy;
}
