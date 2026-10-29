/**
 Copy-for-copy difference between two entry lists,
 generalizing `entryDifference` from
 `package/translation-repair/test-fixture/src/scan-findings.test-fixture.ts`
 (there `found`/`listed`,
 here `actual`/`expected`).

 Entries pair up by {@link entryKey},
 so one changed entry in a long list names that entry on one line instead of
 printing both lists whole beside each other.

 @module
 */

import {
  type EntryKeyRegistry,
  entryKey,
} from './value-entry-key.ts';

//region Entry difference

/**
 How two entry lists differ copy for copy:
 entries one side holds and the other does not,
 and whether the shared entries sit in another order.
 */
export type EntryDifference = {
  /**
   Entries the actual list holds and the expected list does not.
   */
  readonly actualNotExpected: readonly unknown[];
  /**
   Entries the expected list holds and the actual list does not.
   */
  readonly expectedNotActual: readonly unknown[];
  /**
   Whether both lists hold the same entries in a different order.
   */
  readonly reordered: boolean;
};

/**
 The copy-for-copy difference between two entry lists.

 Pairing runs through {@link entryKey} within one registry,
 so equal entries pair across sides no matter their position,
 and leftovers on each side become the named difference.
 Leftovers keep their original order so the rendered lines read like
 the lists they came from.
 Order only speaks up when the entries themselves match,
 since that is when order is the whole difference.

 @param actual - entries the assertion found

 @param expected - entries the assertion listed

 @param registry - shared identity table for reference-kind entries

 @returns Entries unmatched on each side in their original order, and whether order alone differs

 @mutates actual - Keying may invoke getters, proxy traps, `Symbol.toPrimitive`, `toString`, or `valueOf`.

 @mutates expected - The same reads run over the listed entries.

 @mutates registry - Reference kinds mint identity tokens.

 @example
 ```ts
 const registry = createEntryKeyRegistry();
 entryDifference({ actual: [1, 2,], expected: [1, 3,], registry, });
 // { actualNotExpected: [2], expectedNotActual: [3], reordered: false }
 ```
 */
export function entryDifference({
  actual,
  expected,
  registry,
}: {
  readonly actual: readonly unknown[];
  readonly expected: readonly unknown[];
  readonly registry: EntryKeyRegistry;
},): EntryDifference {
  /**
   Expected positions still unpaired, by entry key,
   so each actual entry consumes one copy of its match.
   */
  const openPositions = new Map<string, number[]>();
  /**
   Pairing key per expected position,
   kept so the order check rekeys nothing.
   */
  const expectedKeys: string[] = [];
  expected.forEach(function queuePosition(
    entry,
    index,
  ) {
    /**
     Pairing key of this expected entry.
     */
    const key = entryKey({
      value: entry,
      registry,
    },);
    expectedKeys.push(key,);
    /**
     Positions already queued for this key.
     */
    const queued = openPositions.get(key,);
    if (queued === undefined) {
      openPositions.set(
        key,
        [index,],
      );
    }
    else {
      queued.push(index,);
    }
  },);
  /**
   Entries the actual list holds without an expected copy.
   */
  const actualNotExpected: unknown[] = [];
  /**
   Expected positions some actual entry consumed.
   */
  const consumed = new Set<number>();
  /**
   Pairing key per actual position,
   kept so the order check rekeys nothing.
   */
  const actualKeys: string[] = [];
  actual.forEach(function pairEntry(entry,) {
    /**
     Pairing key of this actual entry.
     */
    const key = entryKey({
      value: entry,
      registry,
    },);
    actualKeys.push(key,);
    /**
     Open expected positions for this key, if any.
     */
    const queued = openPositions.get(key,);
    if ((queued === undefined) || (queued.length === 0)) {
      actualNotExpected.push(entry,);
      return;
    }
    /**
     Position this actual entry pairs with.
     */
    const position = queued.shift();
    if (position !== undefined)
      consumed.add(position,);
  },);
  /**
   Entries the expected list holds without an actual copy,
   gathered in the expected list's own order.
   */
  const expectedNotActual = expected.filter(function isUnpaired(
    _,
    index,
  ) {
    return !consumed.has(index,);
  },);
  /**
   Whether the shared entries sit in another order,
   true only when every entry paired and some position disagrees.
   */
  const reordered = (actualNotExpected.length === 0)
    && (expectedNotActual.length === 0)
    && actualKeys.some(function outOfOrder(
      key,
      index,
    ) {
      return key !== expectedKeys[index];
    },);
  return {
    actualNotExpected,
    expectedNotActual,
    reordered,
  };
}

/**
 The positions where two lists holding the same entries sit in another
 order,
 reported one index at a time so the reordering names its divergences.

 Only call when {@link entryDifference} reported `reordered`,
 since equal multisets of equal length are assumed.

 @param actual - entries the assertion found

 @param expected - entries the assertion listed

 @param registry - shared identity table for reference-kind entries

 @returns Positions whose entries differ, each with both sides

 @mutates actual - Keying may invoke getters, proxy traps, `Symbol.toPrimitive`, `toString`, or `valueOf`.

 @mutates expected - The same reads run over the listed entries.

 @mutates registry - Reference kinds mint identity tokens.

 @example
 ```ts
 const registry = createEntryKeyRegistry();
 orderMismatches({ actual: [2, 1,], expected: [1, 2,], registry, });
 // [{ index: 0, actual: 2, expected: 1, }, { index: 1, actual: 1, expected: 2, }]
 ```
 */
export function orderMismatches({
  actual,
  expected,
  registry,
}: {
  readonly actual: readonly unknown[];
  readonly expected: readonly unknown[];
  readonly registry: EntryKeyRegistry;
},): readonly {
  readonly index: number;
  readonly actual: unknown;
  readonly expected: unknown
}[] {
  /**
   Diverging positions collected in index order.
   */
  const mismatches: {
    index: number;
    actual: unknown;
    expected: unknown
  }[] = [];
  actual.forEach(function comparePosition(
    entry,
    index,
  ) {
    if (entryKey({
      value: entry,
      registry,
    },) === entryKey({
      value: expected[index],
      registry,
    },))
      return;
    mismatches.push({
      index,
      actual: entry,
      expected: expected[index],
    },);
  },);
  return mismatches;
}

//endregion Entry difference
