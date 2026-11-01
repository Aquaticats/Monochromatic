//region Count word
// The word a count takes in a report line: one form at exactly one, the other
// at any other count. Before this helper each report module chose it inline
// (`count === 1 ? '' : 's'`, `'is' : 'are'`), a branch each report's tests had
// to reach on its own (ledger T8, twelfth batch).

/**
 The word a count takes: `one` at exactly one, `many` at any other count,
 zero included, as English writes "0 artifacts".

 @param count - how many

 @param one - word at exactly one

 @param many - word at any other count

 @returns The word

 @example
 ```ts
 const noun = wordForCount({ count: 2, one: 'artifact', many: 'artifacts', },); // 'artifacts'
 ```
 */
export function wordForCount(
  {
    count,
    one,
    many,
  }: {
    readonly count: number;
    readonly one: string;
    readonly many: string;
  },
): string {
  return (count === 1) ? one : many;
}

/**
 A count of occurrences as a finding says it. Moved here from
 `translate-atom-rendering.ts`, whose copy `translate-address-drop.ts` had
 written out again inline.

 @param count - occurrences, at least one

 @returns "once", or the count and "times"

 @example
 ```ts
 const often = howOften({ count: 2, },); // '2 times'
 ```
 */
export function howOften({ count, }: { readonly count: number; },): string {
  return wordForCount({
    count,
    one: 'once',
    many: `${String(count,)} times`,
  },);
}

//endregion Count word
