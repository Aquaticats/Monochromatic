/**
 Key access over the in-memory store object.
 
 Wraps upstream `conf`'s `dot-prop` dependency so dot-notation semantics
 (escapes,
 bracket segments,
 prototype guards) stay identical while the
 store's non-dot mode keeps upstream's literal-key behavior.
 
 @module
 */

import { isDeepStrictEqual, } from 'node:util';

import {
  deleteProperty,
  getProperty,
  hasProperty,
  setProperty,
} from 'dot-prop';

//region Constants

/**
 Creates a store object with a null prototype,
 so no key can ever reach `Object.prototype`.
 
 @returns Empty object whose prototype is `null`.
 
 @example
 ```ts
 const store = createPlainObject();
 Object.getPrototypeOf(store); // => null
 ```
 */
export function createPlainObject(): Record<string, unknown> {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- Object.create(null) is the only way to mint a null-prototype dictionary and is typed `any`; the annotation carries the intended shape.
  return Object.create(null,) as Record<string, unknown>;
}

/**
 Key names the non-dot write path refuses to assign,
 mirroring upstream `conf`'s prototype-pollution guard.
 
 @example
 ```ts
 NON_DOT_FORBIDDEN_KEYS.has('__proto__'); // => true
 ```
 */
const NON_DOT_FORBIDDEN_KEYS: ReadonlySet<string> = new Set([
  '__proto__',
  'constructor',
  'prototype',
],);

//endregion Constants

//region Reads

/**
 Reports whether two store snapshots hold the same contents,
 ignoring top-level prototype differences.
 
 Upstream `conf` compares its defaults merge with `assert.deepEqual`,
 which is prototype-insensitive:
 a null-prototype read result and a plain-object stub with identical keys
 count as unchanged. Spreading both sides normalizes the top level the
 same way while keeping strict comparison of nested JSON values.
 
 @param left - One store snapshot.
 
 @param right - The other store snapshot.
 
 @returns `true` when the snapshots hold equal contents.
 
 @example
 ```ts
 isStoreContentEqual({
   left: { theme: 'dark', },
   right: Object.assign(Object.create(null,), { theme: 'dark', },),
 }); // => true
 ```
 */
export function isStoreContentEqual({
  left,
  right,
}: {
  readonly left: Readonly<Record<string, unknown>>;
  readonly right: Readonly<Record<string, unknown>>;
},): boolean {
  return isDeepStrictEqual(
    {
      ...left,
    },
    {
      ...right,
    },
  );
}

/**
 Reads one key from the store,
 in dot-notation or literal-key mode.
 
 @param store - Store object to read from.
 
 @param key - Key path as the caller wrote it.
 
 @param defaultValue - Value returned when the key is absent.
 
 @param accessPropertiesByDotNotation - Whether dots address nested
 properties.
 
 @returns Stored value or `defaultValue`.
 
 @example
 ```ts
 getStoreValue({
   store: { nested: { theme: 'dark', }, },
   key: 'nested.theme',
   defaultValue: 'light',
   accessPropertiesByDotNotation: true,
 }); // => 'dark'
 ```
 */
export function getStoreValue({
  store,
  key,
  defaultValue,
  accessPropertiesByDotNotation,
}: {
  readonly store: Readonly<Record<string, unknown>>;
  readonly key: string;
  readonly defaultValue?: unknown;
  readonly accessPropertiesByDotNotation: boolean;
},): unknown {
  if (accessPropertiesByDotNotation)
    return getProperty(
      store,
      key,
      defaultValue,
    );
  return key in store ? store[key] : defaultValue;
}

/**
 Reports whether one key exists in the store,
 in dot-notation or literal-key mode.
 
 @param store - Store object to probe.
 
 @param key - Key path as the caller wrote it.
 
 @param accessPropertiesByDotNotation - Whether dots address nested
 properties.
 
 @returns `true` when the key resolves to a present value.
 
 @example
 ```ts
 hasStoreValue({
   store: { theme: 'dark', },
   key: 'theme',
   accessPropertiesByDotNotation: true,
 }); // => true
 ```
 */
export function hasStoreValue({
  store,
  key,
  accessPropertiesByDotNotation,
}: {
  readonly store: Readonly<Record<string, unknown>>;
  readonly key: string;
  readonly accessPropertiesByDotNotation: boolean;
},): boolean {
  if (accessPropertiesByDotNotation)
    return hasProperty(
      store,
      key,
    );
  return key in store;
}

//endregion Reads

//region Writes

/**
 Builds a store copy carrying one placed key,
 in dot-notation or literal-key mode.
 
 The literal-key mode silently refuses `__proto__`,
 `constructor`,
 and `prototype` keys exactly as upstream `conf` does. Top-level keys are
 copied so the caller's object keeps its own state;
 dotted writes reach the copy's shared nested objects,
 which callers hand over freshly-read stores for.
 
 @param store - Store object read for the copy.
 
 @param key - Key path as the caller wrote it.
 
 @param value - JSON-representable value to place.
 
 @param accessPropertiesByDotNotation - Whether dots address nested
 properties.
 
 @returns Null-prototype store copy carrying the placed value.
 
 @mutates store - Dotted writes reach shared nested objects of the input.
 
 @example
 ```ts
 const next = withStoreValue({
   store,
   key: 'nested.theme',
   value: 'dark',
   accessPropertiesByDotNotation: true,
 });
 ```
 */
export function withStoreValue({
  store,
  key,
  value,
  accessPropertiesByDotNotation,
}: {
  readonly store: Readonly<Record<string, unknown>>;
  readonly key: string;
  readonly value: unknown;
  readonly accessPropertiesByDotNotation: boolean;
},): Record<string, unknown> {
  /**
   Null-prototype copy carrying the placed value.
   */
  const next = Object.assign(
    createPlainObject(),
    store,
  );
  if (accessPropertiesByDotNotation) {
    setProperty(
      next,
      key,
      value,
    );
    return next;
  }
  if (NON_DOT_FORBIDDEN_KEYS.has(key,))
    return next;
  next[key] = value;
  return next;
}

/**
 Builds a store copy without one key,
 in dot-notation or literal-key mode.
 
 @param store - Store object read for the copy.
 
 @param key - Key path as the caller wrote it.
 
 @param accessPropertiesByDotNotation - Whether dots address nested
 properties.
 
 @returns Null-prototype store copy without the key.
 
 @mutates store - Dotted deletes reach shared nested objects of the input.
 
 @example
 ```ts
 const next = withoutStoreValue({
   store,
   key: 'nested.theme',
   accessPropertiesByDotNotation: true,
 });
 ```
 */
export function withoutStoreValue({
  store,
  key,
  accessPropertiesByDotNotation,
}: {
  readonly store: Readonly<Record<string, unknown>>;
  readonly key: string;
  readonly accessPropertiesByDotNotation: boolean;
},): Record<string, unknown> {
  /**
   Null-prototype copy missing the removed key.
   */
  const next = Object.assign(
    createPlainObject(),
    store,
  );
  if (accessPropertiesByDotNotation) {
    deleteProperty(
      next,
      key,
    );
    return next;
  }
  Reflect.deleteProperty(
    next,
    key,
  );
  return next;
}

//endregion Writes
