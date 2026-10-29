/**
 Realm-independent classification helpers for the reference model.

 deepmerge-ts documents records, arrays, Sets, and Maps as merge buckets
 without restricting them to the current realm, so the model classifies an
 object from `node:vm` or another frame the same as a local one. `instanceof`
 and identity with this realm's `Object.prototype` only match the current
 realm; these helpers use brand checks and prototype shape instead.

 @module
 */

import { types, } from 'node:util';

/**
 Whether `prototype` is some realm's `Object.prototype`: the root of its chain
 whose `constructor.prototype` points back at it and which owns
 `isPrototypeOf`, a method only `Object.prototype` defines.

 @param prototype - Direct prototype of the object being classified.

 @returns Whether objects inheriting straight from `prototype` are plain.

 @example
 ```ts
 isObjectPrototype(Object.prototype); // true
 isObjectPrototype(Date.prototype); // false
 ```
 */
export function isObjectPrototype(prototype: unknown,): boolean {
  if (((typeof prototype) !== 'object') || (prototype === null))
    return false;
  if (Object.getPrototypeOf(prototype,) !== null)
    return false;
  if (!Object.hasOwn(
    prototype,
    'isPrototypeOf',
  ))
    return false;
  /**
   Constructor the prototype names, `Object` in its realm when plain.
   */
  const constructor: unknown = Reflect.get(
    prototype,
    'constructor',
  );
  return ((typeof constructor) === 'function')
    && (Reflect.get(
      constructor,
      'prototype',
    ) === prototype);
}

/**
 Whether `value` is a Set from any realm, by internal slot.

 @param value - Object to classify.

 @returns Whether `value` carries a Set's internal slot, subclasses included.

 @example
 ```ts
 isAnySet(new Set()); // true
 ```
 */
export function isAnySet(value: object,): value is ReadonlySet<unknown> {
  return types.isSet(value,);
}

/**
 Whether `value` is a Map from any realm, by internal slot.

 @param value - Object to classify.

 @returns Whether `value` carries a Map's internal slot, subclasses included.

 @example
 ```ts
 isAnyMap(new Map()); // true
 ```
 */
export function isAnyMap(value: object,): value is ReadonlyMap<unknown, unknown> {
  return types.isMap(value,);
}
