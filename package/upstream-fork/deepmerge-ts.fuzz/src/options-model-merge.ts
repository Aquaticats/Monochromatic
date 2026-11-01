/**
 Default merge steps of the options-aware model (`./options-model.ts`):
 how each container kind merges once the dispatch chose the default, and
 which slot a child position gets in the into model.

 @module
 */

import {
  recordKeys,
  type ValueKind,
} from './model.ts';
import { SKIP, } from './options-model-choice.ts';

/**
 Whether a position is the target's own value, a Map entry only the target
 holds (which the into merge never visits), a key the target lacks, or part
 of a returning merge.
 */
export type Slot = 'none' | 'seeded' | 'target' | 'targetMapOnly';

/**
 One child position to merge one level deeper.
 */
export type ChildPosition = {
  readonly values: readonly unknown[];
  readonly slot: Slot;
};

/**
 Merges one child position; injected so the kind steps precede recursion.
 */
export type MergeChild = (child: ChildPosition,) => unknown;

/**
 Slot of a child position.

 @param slot - Slot of the parent position.

 @param targetHasKey - Whether the parent's first value (the target's, at a
   `target` slot) holds the child key.

 @returns Child slot.

 @example
 ```ts
 childSlot({ slot: 'target', targetHasKey: false, }); // 'seeded'
 ```
 */
function childSlot({
  slot,
  targetHasKey,
}: {
  readonly slot: Slot;
  readonly targetHasKey: boolean;
},): Slot {
  if (slot === 'none')
    return 'none';
  return ((slot === 'target') || (slot === 'targetMapOnly')) && targetHasKey ? 'target' : 'seeded';
}

/**
 Whether a record holds `key` as an own enumerable property.

 @param record - Record to probe.

 @param key - Key to look up.

 @returns Whether the merge visits `key` on `record`.

 @example
 ```ts
 holds({ record: { a: 1, }, key: 'a', }); // true
 ```
 */
function holds({
  record,
  key,
}: {
  readonly record: object;
  readonly key: PropertyKey;
},): boolean {
  return Object.prototype
    .propertyIsEnumerable
    .call(
    record,
    key,
    );
}

/**
 Default merge of Maps, children merged through `mergeChild`.

 @param maps - Maps being merged, target's first at a `target` slot.

 @param slot - Slot of this position.

 @param mergeChild - Child merge one level deeper.

 @returns Merged Map.

 @example
 ```ts
 mergeMapsModel({ maps, slot: 'none', mergeChild, });
 ```
 */
function mergeMapsModel(
  {
    maps,
    slot,
    mergeChild,
  }: {
    readonly maps: readonly ReadonlyMap<unknown, unknown>[];
    readonly slot: Slot;
    readonly mergeChild: MergeChild;
  },
): Map<unknown, unknown> {
  /**
   Union of keys in first-seen order.
   */
  const keys = [...new Set(maps.flatMap(function mapKeys(map,) {
    return [...map.keys(),];
  },),),];
  /**
   Target's Map at a `target` slot, otherwise the first source's.
   */
  const [first,] = maps;
  return new Map(keys.flatMap(function entry(key,): (readonly [
    unknown,
    unknown,
  ])[] {
    /**
     Maps holding this key.
     */
    const holders = maps.filter(function hasKey(map,) {
      return map.has(key,);
    },);
    /**
     Slot of the entry; a `target` Map's own entry no source holds is never visited.
     */
    const entrySlot = childSlot({
      slot,
      targetHasKey: (first !== undefined) && first.has(key,),
    },);
    /**
     Merged value for this key.
     */
    const merged = mergeChild({
      slot: (entrySlot === 'target') && (holders.length === 1) ? 'targetMapOnly' : entrySlot,
      values: holders.map(function valueOf(map,) {
        return map.get(key,);
      },),
    },);
    return merged === SKIP ? [] : [[
      key,
      merged,
    ],];
  },),);
}

/**
 Default merge of records, children merged through `mergeChild`.

 @param records - Records being merged, target's first at a `target` slot.

 @param slot - Slot of this position.

 @param mergeChild - Child merge one level deeper.

 @returns Merged plain record.

 @example
 ```ts
 mergeRecordsModel({ records, slot: 'none', mergeChild, });
 ```
 */
function mergeRecordsModel(
  {
    records,
    slot,
    mergeChild,
  }: {
    readonly records: readonly object[];
    readonly slot: Slot;
    readonly mergeChild: MergeChild;
  },
): object {
  /**
   Union of keys in first-seen order.
   */
  const keys = [...new Set(records.flatMap(function keysOf(record,) {
    return recordKeys(record,);
  },),),];
  /**
   First record: the target's own at a `target` slot.
   */
  const [first,] = records;
  return keys.reduce<object>(
    function addKey(
      result,
      key,
    ) {
      /**
       Merged value for this key.
       */
      const merged = mergeChild({
        slot: childSlot({
          slot,
          targetHasKey: (first !== undefined) && (holds({
            key,
            record: first,
          },)),
        },),
        values: records
          .filter(function hasKey(record,) {
            return holds({
              key,
              record,
            },);
          },)
          .map(function valueOf(record,): unknown {
            return Reflect.get(
              record,
              key,
            );
          },),
      },);
      if (merged !== SKIP) {
        Reflect.defineProperty(
          result,
          key,
          {
            configurable: true,
            enumerable: true,
            value: merged,
            writable: true,
          },
        );
      }
      return result;
    },
    {},
  );
}

/**
 Default merge of one kind, with children merged through `mergeChild`.

 @param kind - Kind of every value.

 @param values - Filtered values of that kind.

 @param slot - Slot of this position.

 @param mergeChild - Child merge one level deeper.

 @returns Default merge result.

 @example
 ```ts
 defaultMerge({ kind: 'array', values: [[1,], [2,],], slot: 'none', mergeChild, }); // [1, 2]
 ```
 */
export function defaultMerge(
  {
    kind,
    values,
    slot,
    mergeChild,
  }: {
    readonly kind: ValueKind;
    readonly values: readonly unknown[];
    readonly slot: Slot;
    readonly mergeChild: MergeChild;
  },
): unknown {
  if (kind === 'array') {
    return values.flatMap(function items(value,): unknown[] {
      return Array.isArray(value,) ? [...(value as readonly unknown[]),] : [];
    },);
  }
  if (kind === 'set') {
    return new Set(values.flatMap(function items(value,): unknown[] {
      return value instanceof Set ? [...(value as ReadonlySet<unknown>),] : [];
    },),);
  }
  if (kind === 'map') {
    return mergeMapsModel({
      maps: values.filter(function isMap(value,): value is ReadonlyMap<unknown, unknown> {
        return value instanceof Map;
      },),
      mergeChild,
      slot,
    },);
  }
  if (kind === 'record') {
    return mergeRecordsModel({
      mergeChild,
      records: values.filter(function isObject(value,): value is object {
        return ((typeof value) === 'object') && (value !== null);
      },),
      slot,
    },);
  }
  return values.at(-1,);
}

/**
 Copy of a record with one more own enumerable property, keeping its
 prototype and every existing descriptor; the model never mutates inputs.

 @param record - Record to extend.

 @param key - Added key.

 @param value - Added value.

 @returns Extended copy.

 @example
 ```ts
 withProperty({ record: { a: 1, }, key: 'b', value: 2, }); // { a: 1, b: 2 }
 ```
 */
export function withProperty(
  {
    record,
    key,
    value,
  }: {
    readonly record: object;
    readonly key: PropertyKey;
    readonly value: unknown;
  },
): object {
  /**
   Prototype the copy keeps.
   */
  const prototype: unknown = Object.getPrototypeOf(record,);
  /**
   Copy sharing the record's prototype and descriptors.
   */
  const copy = Object.defineProperties(
    {},
    Object.getOwnPropertyDescriptors(record,),
  );
  Reflect.setPrototypeOf(
    copy,
    ((typeof prototype) === 'object') ? prototype : null,
  );
  Reflect.defineProperty(
    copy,
    key,
    {
      configurable: true,
      enumerable: true,
      value,
      writable: true,
    },
  );
  return copy;
}
