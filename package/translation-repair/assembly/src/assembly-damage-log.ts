import type { AdjacentSliceText, } from './assembly-adjacent-repetition.ts';
import { lostDistinctiveWords, } from './assembly-content-survival.ts';
import {
  findIntroducedRepetitions,
  whitespaceTokensOf,
} from './assembly-repetition.ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { foldedLatinWords, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Assembly damage log
// WHERE THE DOCUMENT-SCALE DAMAGE IS, for the log (ledger L12). The findings
// `introduced-repetition` and `content-survival` are counted across runs and
// stay free of corpus wording, so they name neither slices nor words, and
// nothing else located either: a reader of a run reran the check by hand to
// find which slices repeated a passage or lost the archive's specifics. These
// lines name the slices, with the phrase and the lost words, and are logged
// beside the findings rather than carried in them.

/**
 One log line per introduced repetition, naming the slices whose shipped
 wording carries the phrase.

 A phrase no single slice carries whole crosses a slice boundary, and the line
 says so rather than naming nothing.

 @param archiveText - translation as it stood before the lane ran

 @param shippedText - assembled document the lane produced

 @param shippedSlices - wording each slice contributed, in document order

 @returns Lines naming slices, counts and the phrase

 @example
 ```ts
 const lines = repetitionLogLines({ archiveText, shippedText, shippedSlices, },);
 ```
 */
export function repetitionLogLines(
  {
    archiveText,
    shippedText,
    shippedSlices,
  }: {
    readonly archiveText: string;
    readonly shippedText: string;
    readonly shippedSlices: readonly AdjacentSliceText[];
  },
): readonly string[] {
  /**
   Each slice's wording in the finder's word form, padded so a phrase matches
   whole words only.
   */
  const padded = shippedSlices.map(function toWords(slice,): {
    readonly sliceIndex: number;
    readonly words: string
  } {
    return {
      sliceIndex: slice.sliceIndex,
      words: ` ${whitespaceTokensOf({ text: slice.text, },)
        .join(' ',)} `,
    };
  },);
  return findIntroducedRepetitions({
    archiveText,
    shippedText,
  },)
    .map(function toLine(found,): string {
      /**
       Slices whose wording carries the phrase whole.
       */
      const carrying = padded
        .filter(function carries(slice,) {
          return slice.words
            .includes(` ${found.phrase} `,);
        },)
        .map(function toIndex(slice,) {
          return String(slice.sliceIndex,);
        },);
      /**
       Where the phrase stands.
       */
      const where = (carrying.length === 0) ? 'across a slice boundary' : `in slices ${carrying.join(', ',)}`;
      return `introduced repetition ${where} (archive ${String(found.archiveCount,)}, shipped ${
        String(found.shippedCount,)
      }): ${JSON.stringify(found.phrase,)}`;
    },);
}

/**
 One log line per archive slice that held a distinctive word the assembled
 document no longer carries, naming the words.

 @param archiveText - translation as it stood before the pipeline ran

 @param shippedText - assembled document the lane produced

 @param archiveSlices - the archive's own wording of each slice

 @returns Lines naming the slice and its lost words, none when nothing was lost

 @example
 ```ts
 const lines = contentLossLogLines({ archiveText, shippedText, archiveSlices, },);
 ```
 */
export function contentLossLogLines(
  {
    archiveText,
    shippedText,
    archiveSlices,
  }: {
    readonly archiveText: string;
    readonly shippedText: string;
    readonly archiveSlices: readonly AdjacentSliceText[];
  },
): readonly string[] {
  /**
   Distinctive archive words the document lost.
   */
  const lost = new Set(lostDistinctiveWords({
    archiveText,
    shippedText,
  },),);
  if (lost.size === 0)
    return [];
  return archiveSlices.flatMap(function toLine(slice,): readonly string[] {
    /**
     Lost words this slice's archive wording held, each once.
     */
    const held = [...new Set(foldedLatinWords({ text: slice.text, },),),]
      .filter(function wasLost(word,) {
        return lost.has(word,);
      },);
    return (held.length === 0)
      ? []
      : [`content lost at slice ${String(slice.sliceIndex,)}: ${String(held.length,)} distinctive archive ${
        wordForCount({
          count: held.length,
          one: 'word',
          many: 'words',
        },)
      } the document no longer carries: ${
        held.join(', ',)
      }`,];
  },);
}

//endregion Assembly damage log
