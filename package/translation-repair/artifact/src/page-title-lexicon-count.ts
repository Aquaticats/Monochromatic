import { foldedLine, } from '@monochromatic-dev/translation-repair-rendering-audit/ts';
import {
  rendersAsNothing,
  straightenQuotes,
} from '@monochromatic-dev/translation-repair-typography/ts';

//region Page title lexicon count
// HOW THE BENCH'S RENDERINGS OF ONE TITLE ARE COUNTED (ledger H16, B24, B40): the wrapping marks come off, renderings
// that differ only in case, spacing or quote style count as one, a rendering that shows a reader nothing counts for
// no voice, and the leader is the rendering most voices gave, the earliest seat breaking a tie. THE COUNT IS NO
// DECISION: a leader that one voice alone gave is returned with its count of one, and `settlePageTitles` settles only
// on a count of two or more (the lead's ruling of 2026-10-25, re-check 2 finding F1).

/**
 Marks a rendering may arrive wrapped in, taken off before comparing: quotes
 of every style a voice answers in, and Markdown emphasis of either kind
 (ledger B24).
 */
const WRAPPERS: ReadonlySet<string> = new Set([
  '"',
  '\'',
  '*',
  '_',
  '“',
  '”',
  '‘',
  '’',
  '《',
  '》',
  '「',
  '」',
  '『',
  '』',
],);

/**
 A rendering with its wrapping marks and surrounding space off.

 @param rendering - rendering as a voice gave it

 @returns Rendering as the sheet shows it

 @example
 ```ts
 unwrappedRendering({ rendering: ' “Song of the Cat” ', },); // 'Song of the Cat'
 ```
 */
export function unwrappedRendering({ rendering, }: { readonly rendering: string; },): string {
  /**
   Rendering without surrounding space; every wrapper is one UTF-16 unit, so
   an index walks them from either end.
   */
  const trimmed = rendering.trim();
  /**
   Bounds of the wrapped words: first unit that is no wrapper, and one past the last.
   */
  const bounds = {
    start: 0,
    end: trimmed.length,
  };
  while ((bounds.start < bounds.end) && WRAPPERS.has(trimmed.charAt(bounds.start,),))
    bounds.start += 1;
  while ((bounds.end > bounds.start) && WRAPPERS.has(trimmed.charAt(bounds.end - 1,),))
    bounds.end -= 1;
  return trimmed
    .slice(
    bounds.start,
    bounds.end,
    )
    .trim();
}

/**
 Form two renderings share when they differ only in case, spacing of any
 kind, or the style of their apostrophes and quotation marks: the typography
 fold, since the typography restoration makes such renderings one (ledger
 B24).

 @param rendering - unwrapped rendering

 @returns Comparison key

 @example
 ```ts
 comparisonKey({ rendering: 'The  Cat’s song', },); // "the cat's song"
 ```
 */
function comparisonKey({ rendering, }: { readonly rendering: string; },): string {
  return foldedLine({ text: straightenQuotes({ text: rendering, },), },)
    .toLowerCase();
}

/**
 The rendering most voices gave one title, the earliest seat breaking a tie.
 The count it returns can be one, which the caller does not settle on.

 A RENDERING THAT SHOWS A READER NOTHING IS NONE, asked of `rendersAsNothing`
 (ledger B40): asked of an empty-string comparison after the trim, two seats
 answering a zero-width space outvoted the one seat answering in words, and
 the title settled on a rendering no reader sees.

 @param answers - each voice's rendering of the title, in roster order, empty
 for a voice that gave none

 @returns Rendering and how many voices gave it, none where no voice rendered it

 @example
 ```ts
 const chosen = mostGiven({ answers: ['Song of the Cat', 'song of the cat', 'Cat Song',], },);
 ```
 */
export function mostGiven(
  { answers, }: { readonly answers: readonly string[]; },
): readonly {
  readonly rendering: string;
  readonly voices: number;
}[] {
  /**
   Voices per comparison key, with the first wording given under it.
   */
  const tally = answers
    .filter(function given(answer,): boolean {
      return !rendersAsNothing({ text: answer, },);
    },)
    .reduce(
    function count(
      seen,
      answer,
    ) {
        /**
         Key the answer counts under.
         */
      const key = comparisonKey({ rendering: answer, },);
        /**
         Count so far under that key.
         */
      const held = seen.get(key,);
      seen.set(
        key,
        {
          rendering: held?.rendering ?? answer,
          voices: (held?.voices ?? 0) + 1,
        },
      );
      return seen;
    },
    new Map<string, {
      readonly rendering: string;
      readonly voices: number
    }>(),
    );
  return [...tally.values(),]
    .reduce(
    function best(
      chosen: readonly {
        readonly rendering: string;
        readonly voices: number
      }[],
      candidate,
    ) {
        /**
         Leader so far, none before the first.
         */
      const [leader,] = chosen;
      return ((leader === undefined) || (candidate.voices > leader.voices)) ? [candidate,] : chosen;
    },
    [],
    );
}

//endregion Page title lexicon count
