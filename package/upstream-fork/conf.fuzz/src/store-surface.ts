/**
 Operation surface both store implementations expose to the differential
 oracle, plus the entry-shape conversion their multi-`set` payloads share.

 The surface's `get` answers with an explicit found-or-missing value rather
 than a nullish return, so the observation model never has to distinguish an
 absent value from a stored one.

 @module
 */

import type {
  JsonRecord,
  JsonValue,
} from './store-operations.ts';

//region Types

/**
 One `get` answer:
 the stored value when the key resolved, or the missing marker.
 */
export type GetLookup =
  | {
    /**
     Key resolved to a stored value.
     */
    readonly kind: 'found';
    /**
     Value read.
     */
    readonly value: JsonValue;
  }
    | {
    /**
     Key resolved to no stored value.
     */
      readonly kind: 'missing';
    };

/**
 Minimal store surface the runners drive:
 one implementation adapter per store under comparison.
 */
export type StoreSurface = {
  /**
   Path of the config file this store persists to.
   */
  readonly path: string;
  /**
   Reads one key,
   with an optional default for missing keys.
   */
  readonly get: (input: {
    readonly key: string;
    readonly useDefault: boolean;
    readonly defaultValue: JsonValue;
  },) => GetLookup;
  /**
   Places one keyed value.
   */
  readonly setOne: (input: {
    readonly key: string;
    readonly value: JsonValue;
  },) => void;
  /**
   Places many keyed values at once.
   */
  readonly setMany: (input: {
    readonly entries: readonly (readonly [
      string,
      JsonValue
    ])[];
  },) => void;
  /**
   Probes one key.
   */
  readonly has: (key: string,) => boolean;
  /**
   Removes one key.
   */
  readonly delete: (key: string,) => void;
  /**
   Appends one item to one key's array value.
   */
  readonly appendToArray: (input: {
    readonly key: string;
    readonly value: JsonValue;
  },) => void;
  /**
   Restores listed keys to their defaults.
   */
  readonly reset: (input: {
    readonly keys: readonly string[];
  },) => void;
  /**
   Restores the whole store to its defaults.
   */
  readonly clear: () => void;
  /**
   Replaces the whole store.
   */
  readonly replaceStore: (input: {
    readonly value: JsonRecord;
  },) => void;
  /**
   Reads the whole store.
   */
  readonly readStore: () => JsonRecord;
  /**
   Reads the item count.
   */
  readonly readSize: () => number;
};

//endregion Types

//region Payloads

/**
 Converts one operation's entries to the values record both multi-`set`
 call shapes accept.

 @param entries - Key-value entries generated for the multi-`set`.

 @returns Record built through `Object.fromEntries`,
 keeping generated `__proto__` keys own.

 @example
 ```ts
 entriesRecord([['a', 1],]); // => { a: 1 }
 ```
 */
export function entriesRecord(entries: readonly (readonly [
  string,
  JsonValue
])[],): Record<string, JsonValue> {
  return Object.fromEntries(entries,);
}

//endregion Payloads
