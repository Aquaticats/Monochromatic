/**
 Canonical entry keys for the copy-for-copy list difference in
 `list-difference.ts`.

 Two values share a key exactly when they should pair up in the multiset
 walk:
 primitives by value with the spellings apart that chai's deep
 equality keeps apart (`NaN` against `NaN` equal,
 `+0` against `-0` not),
 containers by canonical rendering with record keys,
 map entries,
 and set items sorted so order never changes a key,
 and reference kinds (functions,
 symbols,
 promises,
 weak collections) by identity through a shared registry.

 @module
 */

import {
  arrayLikeEntries,
  byteEntries,
  mapEntries,
  renderValue,
  setEntries,
  valueKind,
} from './value-representation.ts';

//region Identity registry

/**
 Shared identity table for reference kinds,
 so one function or symbol seen from both sides of a comparison receives one
 key and two distinct same-named functions never pair up.
 */
export type EntryKeyRegistry = {
  /**
   Tokens already assigned, by reference.
   */
  readonly identities: Map<object | symbol, string>;
  /**
   Token counter for references seen next.
   */
  next: number;
};

/**
 Creates a registry to pass through one difference computation.
 Keys are only comparable within one registry.

 @returns Fresh identity registry

 @example
 ```ts
 const registry = createEntryKeyRegistry();
 ```
 */
export function createEntryKeyRegistry(): EntryKeyRegistry {
  return {
    identities: new Map(),
    next: 0,
  };
}

/**
 The identity token of a reference-kind value,
 minted once per registry so the same reference keys alike on both sides.

 @param value - function, symbol, promise, or weak collection

 @param registry - shared identity table

 @returns Stable token for the reference

 @mutates registry - First sight of a value assigns its token.
 */
function identityKey({
  value,
  registry,
}: {
  readonly value: object | symbol;
  readonly registry: EntryKeyRegistry;
},): string {
  /**
   Token already assigned to this reference, if any.
   */
  const existing = registry.identities
    .get(value,);
  if (existing !== undefined)
    return existing;
  /**
   Fresh token for a reference seen for the first time.
   */
  const minted = `#${registry.next}`;
  registry.identities
    .set(
    value,
    minted,
    );
  registry.next += 1;
  return minted;
}

//endregion Identity registry

//region Canonical entry key

/**
 The canonical key of one value,
 equal for exactly the values a copy-for-copy difference should pair.

 Cycles mark their position in the open stack so same-shape cycles key
 alike and a cycle against a flat value never does.

 @param value - value to key

 @param registry - shared identity table for reference kinds

 @returns Canonical key string

 @mutates value - Keying may invoke getters, proxy traps, `Symbol.toPrimitive`, `toString`, or `valueOf`.

 @mutates registry - Reference kinds mint identity tokens.

 @example
 ```ts
 const registry = createEntryKeyRegistry();
 entryKey({ value: { a: 1, b: 2, }, registry, });
 // equals entryKey({ value: { b: 2, a: 1, }, registry, })
 ```
 */
export function entryKey({
  value,
  registry,
}: {
  readonly value: unknown;
  readonly registry: EntryKeyRegistry;
},): string {
  return entryKeyInner({
    value,
    registry,
    open: [],
  },);
}

/**
 Recursive worker behind {@link entryKey},
 carrying the container stack so a self-referential value marks its cycle
 position instead of recursing forever.

 @param value - value to key

 @param registry - shared identity table for reference kinds

 @param open - containers still being keyed above this value

 @returns Canonical key string

 @mutates value - Keying may invoke getters, proxy traps, `Symbol.toPrimitive`, `toString`, or `valueOf`.

 @mutates registry - Reference kinds mint identity tokens.
 */
function entryKeyInner({
  value,
  registry,
  open,
}: {
  readonly value: unknown;
  readonly registry: EntryKeyRegistry;
  readonly open: readonly object[];
},): string {
  if (value === null)
    return 'null';
  if ((typeof value) === 'string')
    return `s:${JSON.stringify(value,)}`;
  if ((typeof value) === 'number')
    return `n:${numberKey(value,)}`;
  if ((typeof value) === 'bigint')
    return `i:${value.toString()}`;
  if ((typeof value) === 'boolean')
    return `b:${String(value,)}`;
  if (value === undefined)
    return 'undefined';
  if ((typeof value) === 'symbol')
    return `y:${identityKey({
      value,
      registry,
    },)}`;
  if ((typeof value) === 'function')
    return `f:${identityKey({
      value,
      registry,
    },)}`;
  if (open.includes(value,))
    return `c:${open.indexOf(value,)}`;
  if ((typeof value) !== 'object')
    throw new Error(`unreachable: typeof ${typeof value} was not handled`,);
  return objectKey({
    value,
    registry,
    open: [
      ...open,
      value,
    ],
  },);
}

/**
 The canonical key of a number,
 keeping `NaN`,
 `+0`,
 and `-0` apart since chai's deep equality treats each as its own value.

 @param value - number to key

 @returns Number key

 @example
 ```ts
 numberKey(-0,); // '-0'
 ```
 */
function numberKey(value: number,): string {
  if (Number.isNaN(value,))
    return 'NaN';
  if (Object.is(
    value,
    -0,
  ))
    return '-0';
  return String(value,);
}

/**
 The canonical key of a non-array object:
 typed kinds by their rendered payload,
 reference kinds by identity,
 and records by sorted own enumerable keys so key order never changes a key.

 @param value - object to key

 @param registry - shared identity table for reference kinds

 @param open - containers still being keyed above this value, ending in this value

 @returns Canonical object key

 @mutates value - Keying may invoke getters, proxy traps, `Symbol.toPrimitive`, `toString`, or `valueOf`.

 @mutates registry - Reference kinds mint identity tokens.
 */
function objectKey({
  value,
  registry,
  open,
}: {
  readonly value: object;
  readonly registry: EntryKeyRegistry;
  readonly open: readonly object[];
},): string {
  /**
   Display tag of the object,
   choosing which payload below keys it.
   */
  const tag = valueKind(value,);
  if (tag === 'Array')
    return `A:[${keyOfEntries({
      entries: arrayLikeEntries(value,),
      registry,
      open,
    },)}]`;
  if (tag === 'Arguments')
    return `G:[${keyOfEntries({
      entries: arrayLikeEntries(value,),
      registry,
      open,
    },)}]`;
  if ((tag === 'ArrayBuffer') || (tag === 'DataView'))
    return `T:${tag}:[${keyOfEntries({
      entries: byteEntries(value,),
      registry,
      open,
    },)}]`;
  if (tag.endsWith('Array',))
    return `T:${tag}:[${keyOfEntries({
      entries: arrayLikeEntries(value,),
      registry,
      open,
    },)}]`;
  if (tag === 'Map') {
    /**
     One `key=>value` key per map entry,
     sorted since map equality ignores insertion order.
     */
    const parts = [...mapEntries(value,)].map(function keyOfEntry([key, entryValue],) {
      return `${entryKeyInner({
        value: key,
        registry,
        open,
      },)}=>${entryKeyInner({
        value: entryValue,
        registry,
        open,
      },)}`;
    },);
    return `M:{${parts
      .toSorted()
      .join(',',)}}`;
  }
  if (tag === 'Set') {
    /**
     One key per set item,
     sorted since set equality ignores insertion order.
     */
    const parts = setEntries(value,)
      .map(function keyOfItem(item,) {
        return entryKeyInner({
          value: item,
          registry,
          open,
        },);
      },);
    return `S:{${parts
      .toSorted()
      .join(',',)}}`;
  }
  if ((tag === 'Promise') || (tag === 'WeakMap')
    || (tag === 'WeakSet'))
    return `w:${identityKey({
      value,
      registry,
    },)}`;
  if ((tag === 'Date') || (tag === 'RegExp')
    || (tag === 'Error')
    || (tag === 'String')
    || (tag === 'Number')
    || (tag === 'Boolean'))
    return `L:${renderValue(value,)}`;
  /**
   One `key=value` part per own enumerable key,
   sorted so record key order never changes a record's key.
   */
  const parts = Reflect.ownKeys(value,)
    .filter(function isRelevant(key,) {
      return Object.prototype
        .propertyIsEnumerable
        .call(
        value,
        key,
        );
    },)
    .map(function keyOfMember(key,) {
      return `${entryKeyInner({
        value: key,
        registry,
        open,
      },)}=${entryKeyInner({
        value: Reflect.get(
          value,
          key,
        ),
        registry,
        open,
      },)}`;
    },);
  return `o:{${parts
    .toSorted()
    .join(',',)}}`;
}

/**
 The joined keys of positional entries,
 in order so index-sensitive kinds key by position.

 @param entries - entries to key

 @param registry - shared identity table for reference kinds

 @param open - containers still being keyed above the entries

 @returns Joined entry keys

 @mutates entries - Keying may invoke getters, proxy traps, `Symbol.toPrimitive`, `toString`, or `valueOf`.

 @mutates registry - Reference kinds mint identity tokens.
 */
function keyOfEntries({
  entries,
  registry,
  open,
}: {
  readonly entries: readonly unknown[];
  readonly registry: EntryKeyRegistry;
  readonly open: readonly object[];
},): string {
  return entries.map(function keyOfEntry(entry,) {
    return entryKeyInner({
      value: entry,
      registry,
      open,
    },);
  },)
    .join(',',);
}

//endregion Canonical entry key
