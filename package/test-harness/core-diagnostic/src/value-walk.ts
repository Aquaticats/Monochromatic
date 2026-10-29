/**
 Internal walks behind {@link ./value-difference.ts}:
 dispatch on shared value shape,
 list and map and record walks,
 and the leaf rendering of scalar pairs.

 Split out so the public difference module stays small;
 the walk recurses through {@link walkValue} and never imports back.

 @module
 */

import {
  readProperty,
} from './harness-frames.ts';
import {
  arrayLikeEntries,
  byteEntries,
  describeSide,
  enumerableKeys,
  renderValue,
  valueKind,
  valueRole,
} from './value-representation.ts';
import {
  type Ancestors,
  type DifferenceSink,
  pushLine,
} from './difference-sink.ts';
import {
  type EntryKeyRegistry,
  entryKey,
} from './value-entry-key.ts';
import {
  entryDifference,
  orderMismatches,
} from './list-difference.ts';
import {
  walkMap,
  walkRecord,
} from './value-walk-collections.ts';
import {
  stringDifferenceLines,
} from './string-difference.ts';

//region Value walk

/**
 The entries of an ordered kind in comparison order:
 bytes for byte-bearing kinds and indexed elements otherwise.

 @param value - ordered-kind value read

 @returns Entries in comparison order

 @mutates value - Entry reads may invoke proxy traps.
 */
function orderedEntries(value: object,): readonly unknown[] {
  /**
   Display tag of the value,
   deciding whether entries are bytes or indexed elements.
   */
  const tag = valueKind(value,);
  return ((tag === 'ArrayBuffer') || (tag === 'DataView'))
    ? byteEntries(value,)
    : arrayLikeEntries(value,);
}

/**
 Walks one compared pair into the sink,
 dispatching on the shape both sides share.

 A kind mismatch names both sides by kind and size,
 matching kinds walk their structure,
 and a cycle revisited stops with no extra lines since the ancestor walk
 already names its difference.

 @param actual - value the comparison found

 @param expected - value the comparison listed

 @param prefix - text opening every line

 @param sink - collector for this difference computation

 @param registry - shared identity table for reference kinds

 @param ancestors - container pairs being walked above this one

 @mutates actual - Walking may invoke getters, proxy traps, `Symbol.toPrimitive`, `toString`, or `valueOf`.

 @mutates expected - The same reads run over the listed value.

 @mutates sink - Appends lines or counts them past the cap.

 @mutates registry - Reference kinds mint identity tokens.

 @example
 ```ts
 walkValue({ actual: [1,], expected: [2,], prefix: '', sink, registry, ancestors: [], },);
 ```
 */
export function walkValue({
  actual,
  expected,
  prefix,
  sink,
  registry,
  ancestors,
}: {
  readonly actual: unknown;
  readonly expected: unknown;
  readonly prefix: string;
  readonly sink: DifferenceSink;
  readonly registry: EntryKeyRegistry;
  readonly ancestors: Ancestors;
},): void {
  if (entryKey({
    value: actual,
    registry,
  },) === entryKey({
    value: expected,
    registry,
  },))
    return;
  /**
   Kind of the value the comparison found.
   */
  const actualKind = valueKind(actual,);
  /**
   Kind of the value the comparison listed.
   */
  const expectedKind = valueKind(expected,);
  if (actualKind !== expectedKind) {
    pushLine({
      sink,
      line: `${prefix}kind differs: actual is ${describeSide(actual,)}, expected is ${describeSide(expected,)}`,
    },);
    return;
  }
  if (((typeof actual) === 'string') && ((typeof expected) === 'string')) {
    for (const line of stringDifferenceLines({
      actual,
      expected,
      prefix,
    },)) {
      pushLine({
        sink,
        line,
      },);
    }
    return;
  }
  if (((typeof actual) !== 'object') || (actual === null)
    || ((typeof expected) !== 'object')
    || (expected === null)) {
    walkLeaf({
      actual,
      expected,
      prefix,
      sink,
    },);
    return;
  }
  /**
   Compared container pairs below this one,
   extended so a self-referential pair stops on revisit.
   */
  const deeper: Ancestors = [
    ...ancestors,
    [
      actual,
      expected,
    ],
  ];
  /**
   Whether some ancestor pair holds these exact two containers.
   */
  const cyclic = ancestors.some(function isSamePair([seenActual, seenExpected],) {
    return (seenActual === actual) && (seenExpected === expected);
  },);
  if (cyclic)
    return;
  /**
   How both sides' kind is walked.
   */
  const role = valueRole(actual,);
  if (role === 'ordered') {
    walkList({
      actual: orderedEntries(actual,),
      expected: orderedEntries(expected,),
      prefix,
      sink,
      registry,
      orderMatters: true,
    },);
    return;
  }
  if (role === 'unordered') {
    if ((actual instanceof Map) && (expected instanceof Map)) {
      walkMap({
        actual,
        expected,
        prefix,
        sink,
        registry,
        ancestors: deeper,
        walkChild: walkValue,
      },);
      return;
    }
    if ((actual instanceof Set) && (expected instanceof Set)) {
      walkList({
        actual: [...actual],
        expected: [...expected],
        prefix,
        sink,
        registry,
        orderMatters: false,
      },);
      return;
    }
    /* v8 ignore next -- @preserve */
    throw new Error('unreachable: unordered role without Map or Set',);
  }
  if (role === 'record') {
    walkRecord({
      actual,
      expected,
      prefix,
      sink,
      registry,
      ancestors: deeper,
      walkChild: walkValue,
    },);
    return;
  }
  walkLeaf({
    actual,
    expected,
    prefix,
    sink,
  },);
}

/**
 Walks two unequal leaves of one kind,
 printing both values where their renderings differ
 and naming a reference difference where they render alike.

 @param actual - value the comparison found

 @param expected - value the comparison listed

 @param prefix - text opening the line

 @param sink - collector for this difference computation

 @mutates actual - Rendering may invoke getters, proxy traps, `Symbol.toPrimitive`, `toString`, or `valueOf`.

 @mutates expected - The same reads run over the listed value.

 @mutates sink - Appends the line or counts it past the cap.
 */
function walkLeaf({
  actual,
  expected,
  prefix,
  sink,
}: {
  readonly actual: unknown;
  readonly expected: unknown;
  readonly prefix: string;
  readonly sink: DifferenceSink;
},): void {
  /**
   Rendering of the value the comparison found.
   */
  const actualText = renderValue(actual,);
  /**
   Rendering of the value the comparison listed.
   */
  const expectedText = renderValue(expected,);
  if (actualText === expectedText) {
    pushLine({
      sink,
      line: `${prefix}different references with the same rendering: ${actualText}`,
    },);
    return;
  }
  pushLine({
    sink,
    line: `${prefix}actual ${actualText} and expected ${expectedText}`,
  },);
}

//endregion Value walk

//region List walk

/**
 Walks two entry lists copy for copy,
 naming entries one side holds without the other
 and naming diverging positions when the entries match in another order.

 Order only speaks up for order-sensitive kinds,
 since unordered kinds never fail on order alone.

 @param actual - entries the comparison found

 @param expected - entries the comparison listed

 @param prefix - text opening every line

 @param sink - collector for this difference computation

 @param registry - shared identity table for reference kinds

 @param orderMatters - whether the kind compares positionally

 @mutates actual - Keying may invoke getters, proxy traps, `Symbol.toPrimitive`, `toString`, or `valueOf`.

 @mutates expected - The same reads run over the listed entries.

 @mutates sink - Appends lines or counts them past the cap.

 @mutates registry - Reference kinds mint identity tokens.
 */
function walkList({
  actual,
  expected,
  prefix,
  sink,
  registry,
  orderMatters,
}: {
  readonly actual: readonly unknown[];
  readonly expected: readonly unknown[];
  readonly prefix: string;
  readonly sink: DifferenceSink;
  readonly registry: EntryKeyRegistry;
  readonly orderMatters: boolean;
},): void {
  /**
   Copy-for-copy difference between the two entry lists.
   */
  const difference = entryDifference({
    actual,
    expected,
    registry,
  },);
  for (const entry of difference.actualNotExpected) {
    pushLine({
      sink,
      line: `${prefix}only in actual: ${renderValue(entry,)}`,
    },);
  }
  for (const entry of difference.expectedNotActual) {
    pushLine({
      sink,
      line: `${prefix}only in expected: ${renderValue(entry,)}`,
    },);
  }
  if ((!orderMatters) || (!difference.reordered))
    return;
  pushLine({
    sink,
    line: `${prefix}the same entries sit in another order`,
  },);
  for (const mismatch of orderMismatches({
    actual,
    expected,
    registry,
  },)) {
    pushLine({
      sink,
      line: `${prefix}at position ${mismatch.index}: actual ${renderValue(mismatch.actual,)} and expected ${renderValue(mismatch.expected,)}`,
    },);
  }
}
