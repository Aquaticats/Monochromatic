import { StatedRefusalError, } from '../stated-refusal.ts';
import {
  isNegativeWholeNumberText,
  isWholeNumberText,
  WHOLE_NUMBER_RULE,
} from '@monochromatic-dev/translation-repair-markup/ts';

//region Asked count
// How many units a person asked a bench or calibration to run, read off the
// command line and refused when it is not a count.
//
// FOUR CLIs SPELLED THIS `Number(process.argv[2] ?? String(DEFAULT_SLICES,),)`
// and none of them checked the result. `Number('fourty')` is `NaN`,
// `pickSpreadSample` takes `slice(0, NaN)` and returns nothing, and the run
// then edits, judges and reports over an empty sample. It prints its roster, it
// prints its totals, it exits zero, and every number in it is over no slices at
// all. It was measured: `count NaN -> picked 0`.
//
// A CALIBRATION THAT MEASURED NOTHING IS WORSE THAN ONE THAT REFUSED, because
// the operator reads the clean exit as evidence. That is the whole argument for
// putting a refusal here rather than a fallback: falling back to the default
// would also hide the typo, and would spend a roster doing it.
//
// ZERO IS REFUSED TOO, unlike the audit's `--cap 0`. That cap has a use, namely
// reading a whole archive and buying nothing, and these have none: a bench over
// zero slices asks nobody anything and reports nothing. Where a run wants to do
// nothing, not running it is the way to say so.
//
// DIGITS ONLY, by the package's one count rule (ledger B73). This reader once
// truncated `40.9` to 40 and took `4e1`, `0x28` and `+40` as forty: a count
// nobody typed, run without a word. A minus sign before digits is still
// answered as a count below one, since that is what was typed.

/**
 Smallest count worth running, since a bench over none measures nothing.
 */
const AT_LEAST = 1;

/**
 Reads how many units a run was asked for.

 @param line - the bench's command line, read whole by `reportingRefusals`,
 whose first position is the count; passed in so this is testable without a
 subprocess

 @param fallback - count to run when the person named none

 @param asks - what this run calls the things it counts, for the refusal
 sentence; a person reading `slices` should not have to guess

 @returns Count asked for, or the fallback when none was named

 @throws StatedRefusalError when a count was named that is not a whole number
 written in digits, or is below one

 @example
 ```ts
 const wanted = readAskedCount({ line, fallback: 6, asks: 'slices', },);
 ```
 */
export function readAskedCount(
  {
    line,
    fallback,
    asks,
  }: {
    readonly line: { readonly positionals: readonly string[]; };
    readonly fallback: number;
    readonly asks: string;
  },
): number {
  /**
   Count as written, absent when the person named none. AN EMPTY ARGUMENT IS
   NAMED, and refused like any other count that is no number: it once read
   as no count and ran the default (ledger B75).
   */
  const [written,] = line.positionals;
  if (written === undefined)
    return fallback;

  if ((!isWholeNumberText({ text: written, },)) && (!isNegativeWholeNumberText({ text: written, },)))
    throw new StatedRefusalError({
      says: `${asks} must be ${WHOLE_NUMBER_RULE}, and ${JSON.stringify(written,)} is not one`,
    },);

  /**
   Count as written, which `Number` now reads exactly: digits, with or
   without a minus sign before them.
   */
  const asked = Number(written,);

  if (asked < AT_LEAST)
    throw new StatedRefusalError({
      says: `${asks} must be at least ${String(AT_LEAST,)}, and ${written} is not`,
    },);

  return asked;
}

//endregion Asked count
