import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { compareCodePoints, } from '@monochromatic-dev/translation-repair-markup/ts';
import type {
  FoundRepeat,
  FoundSpan,
} from './repeat-groups-types.ts';

//region Repeat groups order
// The order repeat groups are listed in. It reads the repeats alone, so it never
// depends on the order the finders produced them in.

/**
 First occurrence of a found repeat, which orders the groups.

 @param repeat - found repeat with at least one occurrence

 @returns Its first occurrence in page order

 @throws {@link Error} when the repeat holds no occurrence, which no finder builds

 @example
 ```ts
 const first = firstSpanOf(repeat,);
 ```
 */
function firstSpanOf(repeat: FoundRepeat,): FoundSpan {
  /**
   First of the repeat's occurrences. Both finders list them in page order: the
   maximal finder sorts the starts it keeps, and the refrain finder sorts its members.
   */
  const [first,] = repeat.spans;
  if (first === undefined)
    throw new Error('unreachable: a found repeat holds no occurrence, though both finders keep at least two',);
  return first;
}

/**
 Order of two found repeats: first occurrence by start, then a maximal repeat
 before a refrain, then the first occurrence ending sooner, then the later
 occurrences by start, then the repeat with more occurrences first, then the
 prefixes and the suffixes of refrains by code point.

 @param first - one found repeat

 @param second - other found repeat

 @returns Negative when the first lists before the second, positive when after,
 zero when they match in every part the order reads

 @throws {@link Error} when a repeat holds no occurrence, which no finder builds

 @example
 ```ts
 const order = compareFoundRepeats({ first, second, },);
 ```
 */
export function compareFoundRepeats(
  {
    first,
    second,
  }: {
    readonly first: FoundRepeat;
    readonly second: FoundRepeat;
  },
): number {
  /**
   First occurrence of the first repeat.
   */
  const firstOfFirst = firstSpanOf(first,);

  /**
   First occurrence of the second repeat.
   */
  const firstOfSecond = firstSpanOf(second,);
  if (firstOfFirst.startOffset !== firstOfSecond.startOffset)
    return firstOfFirst.startOffset - firstOfSecond.startOffset;
  if (first.kind !== second.kind)
    return (first.kind === 'maximal') ? -1 : 1;
  if (firstOfFirst.endOffset !== firstOfSecond.endOffset)
    return firstOfFirst.endOffset - firstOfSecond.endOffset;

  /**
   Occurrences of the first repeat.
   */
  const firstSpans = first.spans;

  /**
   Occurrences of the second repeat.
   */
  const secondSpans = second.spans;

  /**
   Occurrences both repeats hold, from the second on.
   */
  const shared = Math.min(
    firstSpans.length,
    secondSpans.length,
  );
  for (let place = 1; place < shared; place++) {
    /**
     The first repeat's occurrence at the place.
     */
    const firstHere = nonNullishOrThrow(firstSpans[place],);

    /**
     The second repeat's occurrence at the place.
     */
    const secondHere = nonNullishOrThrow(secondSpans[place],);
    if (firstHere.startOffset !== secondHere.startOffset)
      return firstHere.startOffset - secondHere.startOffset;
  }
  if (firstSpans.length !== secondSpans.length)
    return secondSpans.length - firstSpans.length;

  /**
   Order of the two prefixes, which decides unless they match.
   */
  const byPrefix = compareCodePoints({
    left: first.prefix,
    right: second.prefix,
  },);
  if (byPrefix !== 0)
    return byPrefix;
  return compareCodePoints({
    left: first.suffix,
    right: second.suffix,
  },);
}

//endregion Repeat groups order
