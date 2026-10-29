/**
 Read-only collection views typed as TypeScript's `ReadonlySet` and
 `ReadonlyMap` without being `Set` or `Map` instances, as observable or
 immutable collection libraries hand them out.

 deepmerge-ts classifies a value's static type structurally
 (`IsSet` and `IsMap` in its `index.d.mts` test `extends ReadonlySet` and
 `extends ReadonlyMap`) but its runtime value by prototype (`getObjectType` in
 `src/utils.ts` tests `instanceof Set` and `instanceof Map`). These views sit
 between the two, which is what `./surface-known-defect.unit.test.ts` pins.

 Each view inherits from {@link VIEW_PROTOTYPE}, whose `Symbol.toStringTag`
 makes deepmerge-ts's record heuristic (`isRecord` in `src/utils.ts`) reject
 it too, so it merges as an opaque leaf.

 @module
 */

/**
 Shared prototype of every view: not `Object.prototype` and not `null`, so
 the record fast path in `isRecord` does not apply, and tagged so its
 `Object.prototype.toString` is not `[object Object]`.
 */
const VIEW_PROTOTYPE = { [Symbol.toStringTag]: 'CollectionView', } as const;
Object.setPrototypeOf(
  VIEW_PROTOTYPE,
  null,
);
Object.freeze(VIEW_PROTOTYPE,);

/**
 Read-only view over a private `Set`.

 @param items - Members.

 @returns View with every `ReadonlySet` member, backed by a `Set` the caller cannot reach.

 @example
 ```ts
 const tags = setView(['a',],);
 tags.has('a',); // true
 ```
 */
export function setView<T,>(items: Iterable<T>,): ReadonlySet<T> {
  /**
   Backing set.
   */
  const members = new Set(items,);
  /**
   View methods, all delegating to {@link members}.
   */
  const view: ReadonlySet<T> = {
    [Symbol.iterator]: function iterate() {
      return members.values();
    },
    difference: function difference(other,) {
      return members.difference(other,);
    },
    entries: function entries() {
      return members.entries();
    },
    forEach: function forEach(visit,) {
      members.forEach(function forward(value,) {
        visit(
          value,
          value,
          view,
        );
      },);
    },
    has: function has(value,) {
      return members.has(value,);
    },
    intersection: function intersection(other,) {
      return members.intersection(other,);
    },
    isDisjointFrom: function isDisjointFrom(other,) {
      return members.isDisjointFrom(other,);
    },
    isSubsetOf: function isSubsetOf(other,) {
      return members.isSubsetOf(other,);
    },
    isSupersetOf: function isSupersetOf(other,) {
      return members.isSupersetOf(other,);
    },
    keys: function keys() {
      return members.keys();
    },
    size: members.size,
    symmetricDifference: function symmetricDifference(other,) {
      return members.symmetricDifference(other,);
    },
    union: function union(other,) {
      return members.union(other,);
    },
    values: function values() {
      return members.values();
    },
  };
  Object.setPrototypeOf(
    view,
    VIEW_PROTOTYPE,
  );
  return view;
}

/**
 Read-only view over a private `Map`.

 @param initial - Key and value pairs.

 @returns View with every `ReadonlyMap` member, backed by a `Map` the caller cannot reach.

 @example
 ```ts
 const lookup = mapView([['a', 1,],],);
 lookup.get('a',); // 1
 ```
 */
export function mapView<K, V,>(initial: Iterable<readonly [
  K,
  V,
]>,): ReadonlyMap<K, V> {
  /**
   Backing map.
   */
  const pairs = new Map(initial,);
  /**
   View methods, all delegating to {@link pairs}.
   */
  const view: ReadonlyMap<K, V> = {
    [Symbol.iterator]: function iterate() {
      return pairs.entries();
    },
    entries: function entries() {
      return pairs.entries();
    },
    forEach: function forEach(visit,) {
      pairs.forEach(function forward(
        value,
        key,
      ) {
        visit(
          value,
          key,
          view,
        );
      },);
    },
    get: function get(key,) {
      return pairs.get(key,);
    },
    has: function has(key,) {
      return pairs.has(key,);
    },
    keys: function keys() {
      return pairs.keys();
    },
    size: pairs.size,
    values: function values() {
      return pairs.values();
    },
  };
  Object.setPrototypeOf(
    view,
    VIEW_PROTOTYPE,
  );
  return view;
}
