import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import { foldCarriageReturns, } from './line-endings.ts';
import {
  parseSliceBody,
  requireMdxRefusal,
  withMdxExhaustionRefused,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Line structure
// Whether a slice is line-structured, meaning each block is a unit rather than
// a paragraph: verse, chat transcripts, lists of short statements.
//
// COMPUTED RATHER THAN JUDGED. The editor prompt previously asked the model to
// recognize this itself, and an attempt to write the same rule as a heuristic
// failed its positive control outright, ranking the one entry known to be verse
// 42nd of 54. A number the pipeline computes and hands over as a fact removes
// that guess from both sides.
//
// MEASURED over every corpus chunk: 49 of 275 trip, across 31 entries, and
// `Toka_ls`, whose editor fabricated three lines, trips at median 22 while its
// two prose chunks sit at 49 and 86. A threshold of 20 would have missed it,
// which is why the control was run before the threshold was chosen.
//
// The counts were 55 of 286 across 34 entries when first taken. Nothing about
// this predicate changed; the ALIGNER did, and chunk boundaries are
// its output. Re-measured through the shipped predicate on the forced aligner.
//
// BLOCKS ARE THE PARSE'S TOP-LEVEL NODES (ledger B68), not runs between blank
// lines: a loose list was five blocks to a split and is one list to the
// floor, and a tight list was one to both. Both counts in this header were
// taken through the split.

/**
 Blocks a slice needs before its shape means anything.

 Under this, a slice is too small to tell a stanza from a short paragraph.
 */
const MIN_BLOCKS = 5;

/**
 Longest median block a line-structured slice may have.

 30 rather than 20: `Toka_ls`'s verse has a median of 22, and its prose
 chunks sit at 49 and 87, so the gap is wide and the threshold sits inside it.
 */
const MAX_MEDIAN_LENGTH = 30;

/**
 Whether blocks of these lengths read as units rather than paragraphs: enough
 of them, and a short median.

 @param lengths - each top-level block's length

 @returns Whether the blocks clear `MIN_BLOCKS` and their median sits at or
 under `MAX_MEDIAN_LENGTH`

 @example
 ```ts
 const short = shortBlocks({ lengths: [12, 9, 14, 11, 10,], },); // true
 ```
 */
function shortBlocks({ lengths, }: { readonly lengths: readonly number[]; },): boolean {
  if (lengths.length < MIN_BLOCKS)
    return false;
  /**
   The lengths in ascending order.
   */
  const ascending = lengths.toSorted(function byLength(
    left,
    right,
  ): number {
    return left - right;
  },);
  /**
   The middle length, present because the block-count floor holds (ledger
   T8, 2026-10-04).
   */
  const middle = nonNullishOrThrow(ascending[Math.floor(ascending.length / 2,)],);
  return middle <= MAX_MEDIAN_LENGTH;
}

/**
 Reports whether a slice is line-structured.

 BLOCKS ARE THE SLICE GRAMMAR'S TOP-LEVEL NODES, as the floor reads them
 (ledger B68), each measured as its extent in UTF-16 code units, the unit
 `MAX_MEDIAN_LENGTH` was measured in. A slice the grammar refuses is not
 line-structured: it has no blocks to measure, as one under `MIN_BLOCKS` has
 too few.

 @param text - full text of one slice

 @returns True when each block reads as a unit rather than a paragraph

 @example
 ```ts
 const lineStructured = isLineStructured({ text: targetText, },);
 ```
 */
export function isLineStructured(
  {
    text,
  }: {
    readonly text: string;
  },
): boolean {
  /**
   Text the slice grammar reads, with each line break one character.

   FOLDED FIRST so a block's length counts a line break as the corpus read
   gives it: the grammar reads `\r\n` itself, but a text read by another
   route would otherwise add one to a block's length for each `\r` it holds.
   */
  const folded = foldCarriageReturns({ text, },)
    .text;
  try {
    return shortBlocks({
      lengths: withMdxExhaustionRefused({
        parse: parseSliceBody,
        input: { text: folded, },
      },)
        .root
        .children
        .map(function toLength(block,): number {
          return nonNullishOrThrow(block.position
            ?.end
            .offset,)
            - nonNullishOrThrow(block.position
              ?.start
              .offset,);
        },),
    },);
  }
  catch (error) {
    // Only the grammar's own refusal answers no; anything else is an
    // unexpected state that must keep propagating.
    requireMdxRefusal({ error, },);
    return false;
  }
}

//endregion Line structure
