import type { ArchiveOriginalSpan, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { SliceReplacement, } from '@monochromatic-dev/translation-repair-page-pass/ts';
import { rewriteEverySlice, } from './page-slice-rewrite.ts';
import { runEnd, } from './text-runs.ts';
import { monthFirstDates, } from './canadian-date.ts';
import { isWordCharacter, } from './canadian-date-parts.ts';
import { canadianSpellings, } from './canadian-spelling.ts';
import {
  inProse,
  protectedRanges,
} from '@monochromatic-dev/translation-repair-glossary/ts';

import { applySpanRewrites, } from './span-rewrites.ts';

//region Canadian forms
// CLASS ONE HUNDRED THIRTY-FOUR (hulicaijia19, 2026-09-25): the page is
// Canadian English (class one hundred thirty-two, owner 2026-09-25: "The
// convention is and should be en_CA."), yet the archive's day-first dates
// ("29th April", "On 4 May") and its "liquorice" stood on slices no lane
// rewrote, beside the bench's "March 13". The sheets tell the bench, but
// nothing reads a slice the lanes never replaced. This page-assembly pass
// reads every slice as the page will carry it and writes its dates month
// first and a closed list of words the Canadian way, outside markup, links,
// code and comments. The front matter is published as the archive has it,
// and a span sealed as the English original is carried byte for byte, so
// both stand aside.

/**
 One form the pass changed.
 */
type FormRewrite = {
  readonly start: number;
  readonly end: number;
  readonly from: string;
  readonly to: string;
};

/**
 Lower-case words an original writes in English in its prose (ledger K9: a
 quoted English sentence kept its spelling on the page only if nothing
 respelled it), outside its markup, attributes and code.

 @param source - original slice

 @returns Its English words in lower case

 @example
 ```ts
 englishWordsOf({ source: '她写道“my favorite color”。', },); // Set { 'my', 'favorite', 'color' }
 ```
 */
function englishWordsOf(
  { source, }: { readonly source: string; },
): ReadonlySet<string> {
  /**
   The original's non-prose ranges.
   */
  const ranges = protectedRanges({ text: source, },);
  /**
   Every run of word characters, with its offsets.
   */
  const words: {
    readonly word: string;
    readonly start: number;
    readonly end: number;
  }[] = [];
  for (let at = 0; at < source.length;) {
    /**
     Where the next word ends, or the cursor where none starts here.
     */
    const end = runEnd({
      text: source,
      from: at,
      keeps: isWordCharacter,
    },);
    if (end > at) {
      words.push({
        word: source.slice(
          at,
          end,
        )
          .toLowerCase(),
        start: at,
        end,
      },);
    }
    at = Math.max(
      end,
      at + 1,
    );
  }
  return new Set(words
    .filter(function inOriginalProse({
      start,
      end,
    },): boolean {
      return inProse({
        ranges,
        start,
        end,
      },);
    },)
    .map(function wordOf({ word, },): string {
      return word;
    },),);
}

/**
 Rewrites one text's dates and spellings the Canadian way.

 @param text - text as the page would carry it

 @param source - the original slice, whose own English words keep their
 spelling

 @returns Rewritten text and each change as "from" to "to"

 @example
 ```ts
 canadianizeText({ text: 'On 4 May the cat ate liquorice.', source: '5月4日猫吃了甘草糖。', },).text; // 'On May 4 the cat ate licorice.'
 ```
 */
export function canadianizeText(
  {
    text,
    source,
  }: {
    readonly text: string;
    readonly source: string;
  },
): {
  readonly text: string;
  readonly changed: readonly string[];
} {
  /**
   The text's non-prose ranges.
   */
  const ranges = protectedRanges({ text, },);
  /**
   Every rewrite, dates and spellings; a date's span never holds a listed
   word, so the two never overlap, and the applier withholds any that would.
   */
  const rewrites: readonly FormRewrite[] = [
    ...monthFirstDates({
      text,
      ranges,
    },),
    ...canadianSpellings({
      text,
      ranges,
      kept: englishWordsOf({ source, },),
    },),
  ];
  /**
   Text with the rewrites applied, and the rewrites that were.
   */
  const rebuilt = applySpanRewrites({
    text,
    rewrites,
  },);
  return {
    text: rebuilt.text,
    changed: rebuilt.applied
      .map(function describe(rewrite,): string {
        return `"${rewrite.from}" to "${rewrite.to}"`;
      },),
  };
}

/**
 Writes every slice's dates and listed spellings the Canadian way, the
 slices no lane replaced included.

 @param slices - prepared pairs, whose archive text stands where no row replaces it

 @param replacements - what the page would write per slice

 @param archiveOriginalSpans - spans sealed as the English original

 @returns Replacements with the forms rewritten (a row added for an archive
 slice the pass changed), the rewritten rows alone, and one finding per
 slice changed

 @example
 ```ts
 const page = canadianizePage({ slices, replacements, archiveOriginalSpans: [], },);
 ```
 */
export function canadianizePage(
  {
    slices,
    replacements,
    archiveOriginalSpans,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly replacements: readonly SliceReplacement[];
    readonly archiveOriginalSpans: readonly ArchiveOriginalSpan[];
  },
): {
  readonly replacements: readonly SliceReplacement[];
  readonly restored: readonly SliceReplacement[];
  readonly findings: readonly string[];
} {
  return rewriteEverySlice({
    slices,
    replacements,
    archiveOriginalSpans,
    rewrite: canadianizeText,
    findingName: 'canadian-form-rewritten',
  },);
}
//endregion Canadian forms
