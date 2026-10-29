/**
 Key-by-key walks behind {@link ./value-walk.ts}:
 maps and records,
 each recursing into values both sides hold differently.

 Split out so each walk file stays under the line limit;
 the recursion arrives as {@link WalkChild} so this module never
 imports the dispatcher back.

 @module
 */

import {
  readProperty,
} from './harness-frames.ts';
import {
  enumerableKeys,
  renderValue,
} from './value-representation.ts';
import {
  type EntryKeyRegistry,
  entryKey,
} from './value-entry-key.ts';
import {
  type Ancestors,
  type DifferenceSink,
  pushLine,
} from './difference-sink.ts';

/**
 Signature of the recursion the dispatcher hands in,
 matching the dispatcher's destructured parameter.
 */
export type WalkChild = (parameters: {
  readonly actual: unknown;
  readonly expected: unknown;
  readonly prefix: string;
  readonly sink: DifferenceSink;
  readonly registry: EntryKeyRegistry;
  readonly ancestors: Ancestors;
}) => void;


/**
 Walks two maps key by key,
 naming keys one side holds without the other
 and recursing into values both hold differently.

 Map keys pair by deep equality,
 matching how chai's deep equality matches them.

 @param actual - map the comparison found

 @param expected - map the comparison listed

 @param prefix - text opening every line

 @param sink - collector for this difference computation

 @param registry - shared identity table for reference kinds

 @param ancestors - container pairs being walked above this one

 @param walkChild - recursion into shared values, handed in so this module never imports the dispatcher

 @mutates actual - Key and value reads may invoke getters or proxy traps.

 @mutates expected - The same reads run over the listed map.

 @mutates sink - Appends lines or counts them past the cap.

 @mutates registry - Reference kinds mint identity tokens.

 @example
 ```ts
 walkMap({ actual: m1, expected: m2, prefix: '', sink, registry, ancestors: [], walkChild, },);
 ```
 */
export function walkMap({
  actual,
  expected,
  prefix,
  sink,
  registry,
  ancestors,
  walkChild,
}: {
  readonly actual: ReadonlyMap<unknown, unknown>;
  readonly expected: ReadonlyMap<unknown, unknown>;
  readonly prefix: string;
  readonly sink: DifferenceSink;
  readonly registry: EntryKeyRegistry;
  readonly ancestors: Ancestors;
  readonly walkChild: WalkChild;
},): void {
  /**
   Expected keys by their pairing key,
   so each actual key finds its counterpart or proves itself extra.
   */
  const expectedByKey = new Map<string, unknown>();
  for (const key of expected.keys()) {
    expectedByKey.set(
      entryKey({
        value: key,
        registry,
      },),
      key,
    );
  }
  /**
   Keys both maps hold,
   actual order preserved for the recursion below.
   */
  const shared: unknown[] = [];
  for (const key of actual.keys()) {
    if (expectedByKey.has(entryKey({
      value: key,
      registry,
    },))) {
      shared.push(key,);
      continue;
    }
    pushLine({
      sink,
      line: `${prefix}only in actual: ${renderValue(key,)} => ${renderValue(actual.get(key,),)}`,
    },);
  }
  for (const [keyKey, key] of expectedByKey) {
    /**
     Whether some shared key already pairs with this expected key.
     */
    const paired = shared.some(function isSameKey(known,) {
      return entryKey({
        value: known,
        registry,
      },) === keyKey;
    },);
    if (paired)
      continue;
    pushLine({
      sink,
      line: `${prefix}only in expected: ${renderValue(key,)} => ${renderValue(expected.get(key,),)}`,
    },);
  }
  for (const key of shared) {
    /**
     Expected-side key paired with this actual key,
     found by pairing key since deep-equal map keys need not share identity.
     */
    const pairedKey = expectedByKey.get(entryKey({
      value: key,
      registry,
    },));
    if (pairedKey === undefined) {
      /* v8 ignore next 3 -- @preserve */
      throw new Error('unreachable: shared key lost its expected pair',);
    }
    walkChild({
      actual: actual.get(key,),
      expected: expected.get(pairedKey,),
      prefix: `${prefix}${renderValue(key,)}: `,
      sink,
      registry,
      ancestors,
    },);
  }
}

//endregion List walk

//region Record walk

/**
 Walks two records key by key,
 naming keys one side holds without the other
 and recursing into values both hold differently.

 Keys pair by identity,
 matching how chai's deep equality matches symbol keys,
 and only own enumerable keys enter the walk,
 the only keys chai's deep equality reads.

 @param actual - record the comparison found

 @param expected - record the comparison listed

 @param prefix - text opening every line

 @param sink - collector for this difference computation

 @param registry - shared identity table for reference kinds

 @param ancestors - container pairs being walked above this one

 @param walkChild - recursion into shared values, handed in so this module never imports the dispatcher

 @mutates actual - Key and value reads may invoke getters or proxy traps.

 @mutates expected - The same reads run over the listed record.

 @mutates sink - Appends lines or counts them past the cap.

 @mutates registry - Reference kinds mint identity tokens.

 @example
 ```ts
 walkRecord({ actual: { a: 1, }, expected: { a: 2, }, prefix: '', sink, registry, ancestors: [], walkChild, },);
 ```
 */
export function walkRecord({
  actual,
  expected,
  prefix,
  sink,
  registry,
  ancestors,
  walkChild,
}: {
  readonly actual: object;
  readonly expected: object;
  readonly prefix: string;
  readonly sink: DifferenceSink;
  readonly registry: EntryKeyRegistry;
  readonly ancestors: Ancestors;
  readonly walkChild: WalkChild;
},): void {
  /**
   Keys the expected record holds, by pairing key,
   so each actual key finds its counterpart or proves itself extra.
   */
  const expectedByKey = new Map<string, string | symbol>();
  for (const key of enumerableKeys(expected,)) {
    expectedByKey.set(
      entryKey({
        value: key,
        registry,
      },),
      key,
    );
  }
  /**
   Keys both records hold,
   actual order preserved for the recursion below.
   */
  const shared: (string | symbol)[] = [];
  for (const key of enumerableKeys(actual,)) {
    /**
     Text labelling the key on every line naming it.
     */
    const label = renderValue(key,);
    if (expectedByKey.has(entryKey({
      value: key,
      registry,
    },))) {
      shared.push(key,);
      continue;
    }
    pushLine({
      sink,
      line: `${prefix}key ${label}: only in actual: ${renderValue(readProperty({
        source: actual,
        key,
      },),)}`,
    },);
  }
  for (const [keyKey, key] of expectedByKey) {
    /**
     Text labelling the key on every line naming it.
     */
    const label = renderValue(key,);
    /**
     Whether some shared key already pairs with this expected key.
     */
    const paired = shared.some(function isSameKey(known,) {
      return entryKey({
        value: known,
        registry,
      },) === keyKey;
    },);
    if (paired)
      continue;
    pushLine({
      sink,
      line: `${prefix}key ${label}: only in expected: ${renderValue(readProperty({
        source: expected,
        key,
      },),)}`,
    },);
  }
  for (const key of shared) {
    walkChild({
      actual: readProperty({
        source: actual,
        key,
      },),
      expected: readProperty({
        source: expected,
        key,
      },),
      prefix: `${prefix}key ${renderValue(key,)}: `,
      sink,
      registry,
      ancestors,
    },);
  }
}

//endregion Record walk
