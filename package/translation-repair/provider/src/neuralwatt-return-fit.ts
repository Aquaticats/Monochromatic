import { durationWordsOf, } from './duration-words.ts';

//region NeuralWatt return fit
// ONE COMPARISON OF A NEURALWATT RETURN AGAINST A BOUND, shared by every site that decides whether a return can be
// waited for: the hold the budgets name (`neuralwatt-return-mark.ts`), the clause a refusal adds when the return lies
// past the pass (`hold-reason.ts`), and the pass's park (`pass-dry-stop.ts`). The distance compared is the one the
// refusal states, margin included, so the three cannot disagree on a return that sits within the margin of the bound.

/**
 Whose bound a pass-left figure is: the entry's own hard cap when that is the smaller of the two bounds an entry runs
 under, the pass's otherwise.

 @example
 ```ts
 const owner: BoundOwner = 'entry';
 ```
 */
export type BoundOwner = 'entry' | 'pass';

/**
 The words naming whose bound a return lies past, for the sentence of a refusal.

 @param boundOwner - whose bound it is

 @returns "the entry has left" or "this pass has left"

 @example
 ```ts
 boundWordsOf({ boundOwner: 'entry', },); // => 'the entry has left'
 ```
 */
function boundWordsOf({ boundOwner, }: { readonly boundOwner: BoundOwner; },): string {
  return (boundOwner === 'entry') ? 'the entry has left' : 'this pass has left';
}

/**
 Whether a return lies inside a bound, the distance counted as the refusal states it.

 @param returnsInMs - milliseconds until the day starts again, the margin included

 @param boundMs - longest wait the caller takes: what a router call allows, or what the pass or the entry has left

 @returns Whether the return can be waited for

 @example
 ```ts
 const fits = returnFitsWithin({ returnsInMs: 700_000, boundMs: 600_000, },); // false
 ```
 */
export function returnFitsWithin(
  {
    returnsInMs,
    boundMs,
  }: {
    readonly returnsInMs: number;
    readonly boundMs: number;
  },
): boolean {
  return returnsInMs <= boundMs;
}

/**
 The words that say a return lies past what the pass has left, none where it fits.

 @param returnsInMs - milliseconds until the day starts again, the margin included

 @param passLeftMs - what the pass has left, absent where nothing bounds the wait

 @param boundOwner - whose bound passLeftMs is, the entry's own hard cap when that is the smaller of the two bounds an
 entry runs under (default: the pass)

 @returns Words as the one entry of a list, or an empty list when the return fits or nothing bounds the wait

 @example
 ```ts
 // => ['past the 10 min this pass has left',]
 const words = pastPassWordsOf({ returnsInMs: 700_000, passLeftMs: 600_000, },);
 ```
 */
export function pastPassWordsOf(
  {
    returnsInMs,
    passLeftMs,
    boundOwner = 'pass',
  }: {
    readonly returnsInMs: number;
    readonly passLeftMs?: number;
    readonly boundOwner?: BoundOwner;
  },
): readonly string[] {
  return ((passLeftMs === undefined) || returnFitsWithin({
    returnsInMs,
    boundMs: passLeftMs,
  },))
    ? []
    : [`past the ${durationWordsOf({ ms: passLeftMs, },)} ${boundWordsOf({ boundOwner, },)}`,];
}

//endregion NeuralWatt return fit
