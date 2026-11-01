/**
 Structural fingerprint for detecting input mutation on exotic values.

 `./shape.ts`'s `snapshot` rebuilds Sets and Maps with their base
 constructors, so a subclass instance's copy differs by prototype before any
 merge runs. A fingerprint needs no copy: it serializes, in one walk, each
 container's prototype identity, own-key order, property descriptors (data
 values recursively, accessors by function identity), Set elements, Map
 entries, array holes, and leaf identity. Equal fingerprints before and after
 a merge mean the merge changed nothing observable.

 Walks trees only; callers never pass cyclic values.

 @module
 */

import { kindOf, } from './model.ts';

/**
 Stable numeric identities for objects and symbols within one fingerprint
 session, so leaf identity and prototypes compare by reference.
 */
export type IdentityTable = {
  readonly ids: WeakMap<object, number>;
  readonly symbols: Map<symbol, number>;
  readonly counter: { next: number; };
};

/**
 Start an identity session; share it between the before and after
 fingerprints so equal references get equal ids.

 @returns Empty identity table.

 @example
 ```ts
 const table = identityTable();
 ```
 */
export function identityTable(): IdentityTable {
  return {
    counter: { next: 0, },
    ids: new WeakMap(),
    symbols: new Map(),
  };
}

/**
 Id of an object or symbol, assigned on first sight.

 @param table - Session table.

 @param value - Object, function, or symbol.

 @returns Positive id stable within the session.

 @example
 ```ts
 identityOf({ table, value: Symbol.iterator, });
 ```
 */
function identityOf({
  table,
  value,
}: {
  readonly table: IdentityTable;
  readonly value: object | symbol;
},): number {
  /**
   Id already assigned to this reference, if any.
   */
  const known = (typeof value) === 'symbol' ? table.symbols
    .get(value as symbol,) : table.ids
      .get(value as object,);
  if (known !== undefined)
    return known;
  table.counter
    .next += 1;
  if ((typeof value) === 'symbol')
    table.symbols
      .set(
      value as symbol,
      table.counter
        .next,
      );
  else
    table.ids
      .set(
      value as object,
      table.counter
        .next,
      );
  return table.counter
    .next;
}

/**
 Fingerprint one value.

 @param table - Session table shared with the fingerprint it is compared to.

 @param value - Tree to describe.

 @returns Deterministic description of everything a merge could mutate.

 @example
 ```ts
 fingerprint({ table: identityTable(), value: { a: [1,], }, });
 ```
 */
export function fingerprint({
  table,
  value,
}: {
  readonly table: IdentityTable;
  readonly value: unknown;
},): string {
  if ((typeof value) === 'symbol')
    return `sym#${String(identityOf({
      table,
      value: value as symbol,
    },),)}`;
  if ((typeof value) === 'number')
    return Object.is(
      value,
      -0,
    ) ? 'num:-0' : `num:${String(value,)}`;
  if ((((typeof value) !== 'object') && ((typeof value) !== 'function')) || (value === null))
    return `${typeof value}:${String(value,)}`;
  /**
   Object being described.
   */
  const object = value as object;
  /**
   Identity of the object itself; leaves are described by identity only.
   */
  const self = `#${String(identityOf({
    table,
    value: object,
  },),)}`;
  /**
   Merge bucket deciding whether contents matter.
   */
  const kind = kindOf(object,);
  if (kind === 'other')
    return self;
  /**
   Prototype identity, so a replaced prototype is visible.
   */
  const prototype: unknown = Object.getPrototypeOf(object,);
  /**
   Prototype description.
   */
  const prototypeId = ((typeof prototype) !== 'object') || (prototype === null) ? 'null' : String(identityOf({
    table,
    value: prototype,
  },),);
  if (object instanceof Set) {
    return `${self}<${prototypeId}>set[${
      [...object,].map(function describeElement(element,) {
        return fingerprint({
          table,
          value: element,
        },);
      },)
        .join(',',)
    }]`;
  }
  if (object instanceof Map) {
    return `${self}<${prototypeId}>map[${
      [...object,].map(function describeEntry([key, entry,],) {
        return `${fingerprint({
          table,
          value: key,
        },)}=>${fingerprint({
          table,
          value: entry,
        },)}`;
      },)
        .join(',',)
    }]`;
  }
  return `${self}<${prototypeId}>{${
    Reflect.ownKeys(object,)
      .map(function describeKey(key,) {
      /**
       Descriptor of this own key; present because the key came from ownKeys.
       */
      const descriptor = Reflect.getOwnPropertyDescriptor(
        object,
        key,
      );
      /**
       Key label.
       */
      const label = (typeof key) === 'symbol' ? fingerprint({
        table,
        value: key,
      },) : key;
      if (descriptor === undefined)
        return `${label}:gone`;
      /**
       Attribute flags.
       */
      const flags = `${descriptor.enumerable === true ? 'e' : ''}${descriptor.configurable === true ? 'c' : ''}${descriptor.writable === true ? 'w' : ''}`;
      return 'value' in descriptor
        ? `${label}/${flags}=${fingerprint({
          table,
          value: descriptor.value,
        },)}`
        : `${label}/${flags}=get:${fingerprint({
          table,
          value: descriptor.get,
        },)},set:${fingerprint({
          table,
          value: descriptor.set,
        },)}`;
    },)
      .join(';',)
  }}`;
}
