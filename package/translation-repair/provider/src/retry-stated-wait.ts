import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  isAsciiDigit,
  isAsciiLowerLetter,
} from '@monochromatic-dev/translation-repair-markup/ts';

//region Stated retry wait
// The wait a provider's refusal names in its own body ("try again in 2h25m18s"),
// read by one forward scan. Split from `transient-retry.ts`, which sleeps it,
// for the line budget when ledger E11 widened the units it reads.

/**
 Phrase a rate-limit refusal uses to name its wait:
 Hyper's 429 body reads "You've hit your hourly rate limit. Please try again
 in 1s" (also 2s, 3s, 4s; measured on XIEPT2, 2026-09-03), and its daily one
 "You've hit your daily rate limit. Please try again in 2h25m18s" (measured
 on Huasheng, 2026-09-07, 84 bodies all counting down to 19:54 UTC).
 */
const RETRY_AFTER_PHRASE = 'try again in ';

/**
 Milliseconds in one second.
 */
const SECOND_MS = 1_000;

/**
 Seconds in one minute.
 */
const MINUTE_S = 60;

/**
 Minutes in one hour.
 */
const HOUR_MIN = 60;

/**
 Milliseconds in one minute.
 */
const MINUTE_MS = MINUTE_S * SECOND_MS;

/**
 Milliseconds in one hour.
 */
const HOUR_MS = HOUR_MIN * MINUTE_MS;

/**
 Milliseconds each unit of a stated wait stands for, by the whole run of
 letters after its number: Hyper writes "2h25m18s", and a refusal may write
 "500ms" or "1 minute 30 seconds" (ledger E11: the first parser read the
 first letter alone, so "500ms" read as 500 minutes and "12 minutes" as no
 wait).
 */
const WAIT_UNIT_MS: ReadonlyMap<string, number> = new Map([
  ...[
    'ms',
    'millisecond',
    'milliseconds',
  ].map(function asMs(unit,): readonly [
    string,
    number,
  ] {
    return [
      unit,
      1,
    ];
  },),
  ...[
    's',
    'sec',
    'secs',
    'second',
    'seconds',
  ].map(function asSeconds(unit,): readonly [
    string,
    number,
  ] {
    return [
      unit,
      SECOND_MS,
    ];
  },),
  ...[
    'm',
    'min',
    'mins',
    'minute',
    'minutes',
  ].map(function asMinutes(unit,): readonly [
    string,
    number,
  ] {
    return [
      unit,
      MINUTE_MS,
    ];
  },),
  ...[
    'h',
    'hr',
    'hrs',
    'hour',
    'hours',
  ].map(function asHours(unit,): readonly [
    string,
    number,
  ] {
    return [
      unit,
      HOUR_MS,
    ];
  },),
],);

/**
 What `indexOf` returns for an absent phrase.
 */
const NOT_FOUND = -1;

/**
 Index just past the run of digits starting at `from`.

 @param text - text to scan

 @param from - where the run may start

 @returns `from` itself when no digit sits there

 @example
 ```ts
 const end = digitRunEnd({ text: '12s', from: 0, },);
 ```
 */
function digitRunEnd(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): number {
  for (let cursor = from; cursor < text.length; cursor += 1) {
    if (!isAsciiDigit({ character: nonNullishOrThrow(text[cursor],), },))
      return cursor;
  }
  return text.length;
}

/**
 Index just past the run of ASCII lower-case letters starting at `from`.

 @param text - text to scan, already lower-cased

 @param from - where the run may start

 @returns `from` itself when no letter sits there

 @example
 ```ts
 const end = letterRunEnd({ text: 'ms later', from: 0, },); // 2
 ```
 */
function letterRunEnd(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): number {
  for (let cursor = from; cursor < text.length; cursor += 1) {
    /**
     Character under the cursor.
     */
    const character = text.charAt(cursor,);
    if (!isAsciiLowerLetter({ character, },))
      return cursor;
  }
  return text.length;
}

/**
 Where the next part of a stated wait starts, past the spaces, commas and
 "and" that join its parts ("1 minute and 30 seconds").

 @param text - lower-cased body

 @param from - offset just past a unit

 @returns Offset of the next part's number, or of whatever else stands there

 @example
 ```ts
 nextPartAt({ text: '1 minute and 30 seconds', from: 8, },); // 13
 ```
 */
function nextPartAt(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): number {
  /**
   Past the spaces and commas.
   */
  const gapEnd = (function pastGap(): number {
    for (let cursor = from; cursor < text.length; cursor += 1) {
      if ((text.charAt(cursor,) !== ' ') && (text.charAt(cursor,) !== ','))
        return cursor;
    }
    return text.length;
  })();
  return text.startsWith(
    'and ',
    gapEnd,
  )
    ? gapEnd + 'and '.length
    : gapEnd;
}

/**
 One part of a stated wait: its milliseconds and where it ends.
 */
type WaitPart = Readonly<{
  /**
   Milliseconds the part names.
   */
  ms: number;

  /**
   Offset just past its unit.
   */
  end: number;
}>;

/**
 Reads one part of a stated wait: a number, a whole number or a decimal,
 then at most one space and a known unit.

 @param text - lower-cased body

 @param at - offset of the part's first digit

 @returns Part, or nothing where no number and known unit stand there

 @example
 ```ts
 readWaitPart({ text: '500ms', at: 0, },); // [{ ms: 500, end: 5 }]
 ```
 */
function readWaitPart(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): readonly WaitPart[] {
  /**
   Past the whole digits.
   */
  const wholeEnd = digitRunEnd({
    text,
    from: at,
  },);
  /**
   Past a decimal fraction, where one follows.
   */
  const numberEnd = ((text.charAt(wholeEnd,) === '.') && isAsciiDigit({ character: text.charAt(wholeEnd + 1,), },))
    ? digitRunEnd({
      text,
      from: wholeEnd + 1,
    },)
    : wholeEnd;
  /**
   Where the unit starts, past at most one space.
   */
  const unitAt = (text.charAt(numberEnd,) === ' ') ? numberEnd + 1 : numberEnd;
  /**
   Past the unit's letters.
   */
  const unitEnd = letterRunEnd({
    text,
    from: unitAt,
  },);
  /**
   Milliseconds one of that unit stands for.
   */
  const unitMs = WAIT_UNIT_MS.get(text.slice(
    unitAt,
    unitEnd,
  ),);
  if ((wholeEnd === at) || (unitMs === undefined))
    return [];
  return [{
    ms: Number(text.slice(
      at,
      numberEnd,
    ),) * unitMs,
    end: unitEnd,
  },];
}

/**
 Wait the refusal itself asks for, when its body names one.

 A single forward scan, in any case: the phrase, then one or more parts,
 each a number and its unit ("2h25m18s", "500ms", "1 minute 30 seconds",
 "1.5s"), summed; the scan stops at the first thing that is no part, and a
 body whose first part is no part names no wait.

 @param bodyText - reply body of the refused attempt

 @returns Milliseconds the body asks the caller to wait, or zero

 @example
 ```ts
 const ms = retryAfterMsOf({ bodyText: 'Please try again in 14m40s', },);
 // => 880_000
 ```
 */
export function retryAfterMsOf({ bodyText, }: { readonly bodyText: string; },): number {
  /**
   The body in lower case, where the phrase and units are read.
   */
  const text = bodyText.toLowerCase();
  /**
   Where the phrase sits in the body.
   */
  const at = text.indexOf(RETRY_AFTER_PHRASE,);
  if (at === NOT_FOUND)
    return 0;

  return (function sumParts(): number {
    /**
     Milliseconds named by the parts read so far.
     */
    let totalMs = 0;
    for (let cursor = at + RETRY_AFTER_PHRASE.length; cursor < text.length;) {
      /**
       The part starting at the cursor, if any.
       */
      const [part,] = readWaitPart({
        text,
        at: cursor,
      },);
      if (part === undefined)
        return Math.round(totalMs,);
      totalMs += part.ms;
      cursor = nextPartAt({
        text,
        from: part.end,
      },);
    }
    return Math.round(totalMs,);
  })();
}

//endregion Stated retry wait
