//region Duration words
// A DURATION IN THE WORDS A REFUSAL STATES IT IN, whole seconds at the top: 301 000 ms is "5 min 1 s", not "301000ms",
// so a reader of the stop line does not divide.

/**
 Milliseconds in a second.
 */
const MS_PER_SECOND = 1_000;

/**
 Seconds in a minute.
 */
const SECONDS_PER_MINUTE = 60;

/**
 Seconds in an hour.
 */
const SECONDS_PER_HOUR = 3_600;

/**
 A duration in hours, minutes and seconds, each unit named only where it is not zero, rounded up to a whole second so
 a wait is never stated shorter than it is.

 @param ms - duration in milliseconds

 @returns Words, `0 s` for no time

 @example
 ```ts
 durationWordsOf({ ms: 301_000, },); // => '5 min 1 s'
 ```
 */
export function durationWordsOf({ ms, }: { readonly ms: number; },): string {
  /**
   Whole seconds, rounded up.
   */
  const total = Math.ceil(Math.max(
    0,
    ms,
  ) / MS_PER_SECOND,);
  /**
   Hours, minutes and seconds the total comes to.
   */
  const parts = [
    [
      Math.floor(total / SECONDS_PER_HOUR,),
      'h',
    ],
    [
      Math.floor((total % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE,),
      'min',
    ],
    [
      total % SECONDS_PER_MINUTE,
      's',
    ],
  ] as const;
  /**
   The units that are not zero, as words.
   */
  const named = parts.filter(function isNamed([count,],): boolean {
    return count > 0;
  },)
    .map(function wordOf([count, unit,],): string {
      return `${String(count,)} ${unit}`;
    },);
  return (named.length === 0) ? '0 s' : named.join(' ',);
}

//endregion Duration words
