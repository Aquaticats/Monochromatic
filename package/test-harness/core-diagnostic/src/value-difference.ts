/**
 The difference between two values a failing comparison checked,
 rendered one difference per line.

 Chai's own rendering cuts any value over `truncateThreshold`,
 hiding what differs; this walk names the difference instead:
 entries one list holds and the other lacks,
 keys missing,
 extra,
 or changed,
 differing string lines,
 and scalar pairs,
 with a cap on how many lines render and never a cap on the compared
 values themselves.

 @module
 */

import {
  readProperty,
} from './harness-frames.ts';
import {
  createEntryKeyRegistry,
} from './value-entry-key.ts';
import type {
  DifferenceSink,
} from './difference-sink.ts';
import {
  walkValue,
} from './value-walk.ts';

/**
 Default cap on rendered difference lines,
 generous enough that a realistic scan failure
 (dozens of findings)
 prints whole,
 while a pathological pair of huge lists stays bounded.
 */
export const DEFAULT_MAX_DIFFERENCES = 100;
//region Public entry points

/**
 Whether one side can hold content the error headline truncates:
 strings,
 functions,
 and containers,
 the values chai's display cuts past its threshold.

 When neither side qualifies,
 both render whole in the headline already,
 and for matchers like `satisfy` the fields carry a predicate result
 rather than compared values at all.

 @param value - one compared side read from the error

 @returns Whether a diff line could name something the headline cut
 */
function sideCarriesLongContent(value: unknown,): boolean {
  return ((typeof value) === 'string')
    || ((typeof value) === 'function')
    || (((typeof value) === 'object') && (value !== null));
}

/**
 The lines describing how an error-like value's `actual` and `expected`
 differ, for errors whose assertion compared two values.

 Errors without both fields produce no lines,
 errors chai marked `showDiff: false` (truthiness,
 containment,
 and similar matchers whose `expected` is not a compared value) stay
 silent,
 and pairs where neither side can hold content the headline truncated
 stay silent too,
 so matchers like `satisfy` whose fields carry a predicate result never
 gain a misleading diff block.

 @param error - error-like value read

 @returns Difference lines; empty where the error compares no values

 @mutates error - Field reads may invoke getters or proxy traps.

 @example
 ```ts
 errorDifferenceLines({ error: { actual: [1,], expected: [2,], showDiff: true, }, });
 // ['only in actual: 1', 'only in expected: 2']
 ```
 */
export function errorDifferenceLines({ error, }: { readonly error: object; },): readonly string[] {
  if (!(Reflect.has(
    error,
    'actual',
  ) && Reflect.has(
    error,
    'expected',
  )))
    return [];
  if (readProperty({
    source: error,
    key: 'showDiff',
  },) === false)
    return [];
  /**
   Value the comparison found, read once for the gate and the walk.
   */
  const actual = readProperty({
    source: error,
    key: 'actual',
  },);
  /**
   Value the comparison listed, read once for the gate and the walk.
   */
  const expected = readProperty({
    source: error,
    key: 'expected',
  },);
  if (!(sideCarriesLongContent(actual,)
    || sideCarriesLongContent(expected,)))
    return [];
  return valueDifferenceLines({
    actual,
    expected,
  },);
}


/**
 The lines describing how two compared values differ,
 one difference per line.

 Walks lists copy for copy,
 records and maps key by key,
 strings line by line,
 and leaves as rendered pairs,
 pruning any subtree whose values pair up.
 Equal values yield the equality line a failing strict or negated
 comparison needs, since that equality is the whole story there.

 @param actual - value the comparison found

 @param expected - value the comparison listed

 @param prefix - text opening every line, naming where the difference sits

 @param maxDifferences - cap on rendered lines; the rest count in one marker

 @returns Difference lines, capped with an overflow marker

 @mutates actual - Walking may invoke getters, proxy traps, `Symbol.toPrimitive`, `toString`, or `valueOf`.

 @mutates expected - The same reads run over the listed value.

 @example
 ```ts
 valueDifferenceLines({ actual: [1, 2,], expected: [1, 3,], });
 // ['only in actual: 2', 'only in expected: 3']
 ```
 */
export function valueDifferenceLines({
  actual,
  expected,
  prefix = '',
  maxDifferences = DEFAULT_MAX_DIFFERENCES,
}: {
  readonly actual: unknown;
  readonly expected: unknown;
  readonly prefix?: string;
  readonly maxDifferences?: number;
},): readonly string[] {
  /**
   Collector bounding rendered lines and counting the rest.
   */
  const sink: DifferenceSink = {
    lines: [],
    suppressed: 0,
    max: maxDifferences,
  };
  walkValue({
    actual,
    expected,
    prefix,
    sink,
    registry: createEntryKeyRegistry(),
    ancestors: [],
  },);
  if (sink.suppressed > 0) {
    sink.lines
      .push(`and ${sink.suppressed} more differences`,);
    return sink.lines;
  }
  if (sink.lines
    .length
    > 0)
    return sink.lines;
  return equalityLines({
    actual,
    expected,
  },);
}

/**
 The one line a failing comparison prints when the walk found no
 difference: the values are deeply equal,
 which is exactly why a negated comparison failed,
 or the same value,
 which leaves identity nothing to explain.

 @param actual - value the comparison found

 @param expected - value the comparison listed

 @returns One equality line

 @mutates actual - `Object.is` reads no properties.
 */
function equalityLines({
  actual,
  expected,
}: {
  readonly actual: unknown;
  readonly expected: unknown;
},): readonly string[] {
  return [Object.is(
    actual,
    expected,
  )
    ? 'no difference found: actual and expected are the same value'
    : 'no structural difference found: the values are deeply equal',];
}

//endregion Public entry points
