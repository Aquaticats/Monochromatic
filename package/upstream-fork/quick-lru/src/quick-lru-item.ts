/**
 Stored cache item model and its expiry-stamp semantics.
 
 Mirrors upstream `quick-lru`'s internal `{value, expiry}` record, which is
 observable through the `__oldCache` test hook. An item that never expires
 carries no `expiry` property at all; upstream carries `expiry: undefined`,
 and every read path treats the two shapes identically.
 
 @module
 */

//region Types

/**
 One stored cache item: its value and optional absolute expiry timestamp.
 
 @typeParam Value - cache value type
 
 @example
 ```ts
 const item: QuickLruItem<number> = {
   value: 7,
   expiry: 1_700_000_000_000,
 };
 ```
 */
export type QuickLruItem<Value> = {
  /**
   Stored value, returned while the item is live.
   */
  readonly value: Value;
  /**
   `Date.now()` reading after which the item counts as expired, absent for
   items that never expire.
   */
  readonly expiry?: number;
};

//endregion Types

//region Entry references

/**
 One keyed item reference as the cache internals pass it around: the key
 an item is stored under plus the item itself.
 
 @typeParam Key - cache key type
 
 @typeParam Value - cache value type
 
 @example
 ```ts
 const entry: QuickLruItemEntry<string, number> = {
   key: 'a',
   item: { value: 1, },
 };
 ```
 */
export type QuickLruItemEntry<Key, Value> = {
  /**
   Key the item is stored under.
   */
  readonly key: Key;
  /**
   Stored item under that key.
   */
  readonly item: QuickLruItem<Value>;
};

//endregion Entry references

//region Expiry stamps

/**
 Checks whether one item carries a usable expiry stamp.
 
 Keeps upstream's truthiness gate on the timestamp exactly: `undefined`,
 `0`, `-0`, and `NaN` all read as "never expires", so a zero-timestamp item
 stays readable through `get` while `has` still expires it, matching
 upstream's split between `#getItemValue` and `#deleteIfExpired`.
 
 @typeParam Value - cache value type
 
 @param item - Stored item to inspect.
 
 @returns Whether the item's expiry stamp is truthy, narrowing the stamp to
 `number` when true.
 
 @example
 ```ts
 hasExpiryStamp({ value: 1, },); // => false
 ```
 */
export function hasExpiryStamp<Value>(
  item: QuickLruItem<Value>,
): item is QuickLruItem<Value> & { readonly expiry: number } {
  return Boolean(item.expiry,);
}

//endregion Expiry stamps
