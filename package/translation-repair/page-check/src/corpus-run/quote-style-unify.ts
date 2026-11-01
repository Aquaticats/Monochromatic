import type { ArchiveOriginalSpan, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  pageTextBySlice,
  pageTextOf,
  restoreTypography,
  type SliceReplacement,
} from '@monochromatic-dev/translation-repair-page-pass/ts';
import { pageDefinitionsOf, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  noteRefusedMask,
  type ProseMaskReading,
  proseMaskReading,
} from '@monochromatic-dev/translation-repair-typography/ts';
import {
  rewriteEverySlice,
  type TextRewrite,
} from './page-slice-rewrite.ts';

//region Quote style unify
// CLASS ONE HUNDRED FORTY-TWO (XingZ6012, 2026-09-26): the page wrote its
// quotes curly (134 curly double marks, 108 curly apostrophes against two
// straight lines of each), yet four archive paragraphs no lane rewrote kept
// their straight marks ("take it slow", "didn't"), because the typography
// restoration reads only the text a lane replaced. This page-assembly pass
// counts the prose quote marks on every slice the page carries and, where the
// curly form of a mark outnumbers the straight form, curls that mark on every
// slice through the same restoration, so tags, code, unbalanced doubles and
// the ellipsis stay as they are. The front matter and a span sealed as the
// English original stand aside, as for every whole-page pass.
//
// THE PAGE IS READ ONCE. Each slice's prose mask is read one time with the page's definitions (`page-definitions.ts`), so a
// reference whose definition stands in another slice is no prose at either end, and the tally and the rewrite use that one
// reading. A slice the grammar refuses has no prose and is named in a warning (`typography-mask-refusal.ts`).

/**
 The name this pass gives in the warning of a slice the grammar refuses.
 */
const READER = 'unifyQuoteStyle';

/**
 Straight double quote.
 */
const STRAIGHT_DOUBLE = '"';

/**
 Straight single quote, in prose an apostrophe.
 */
const STRAIGHT_SINGLE = '\'';

/**
 Curly double quotes, opening and closing.
 */
const CURLY_DOUBLES = '“”';

/**
 Curly apostrophe.
 */
const CURLY_SINGLE = '’';

/**
 Prose quote marks counted on the page, by form.
 */
type QuoteTally = {
  readonly straightDouble: number;
  readonly curlyDouble: number;
  readonly straightSingle: number;
  readonly curlySingle: number;
};

/**
 Tally with nothing counted.
 */
const EMPTY_TALLY: QuoteTally = {
  straightDouble: 0,
  curlyDouble: 0,
  straightSingle: 0,
  curlySingle: 0,
};

/**
 Adds one text's prose quote marks to a tally.

 @param tally - marks counted so far

 @param text - one slice's text as the page carries it

 @param mask - which units of the text are prose, read once by the caller for the tally and the rewrite

 @returns Tally with the text's marks added

 @example
 ```ts
 const text = 'The cat said "meow".';
 tallyQuotes({ tally: EMPTY_TALLY, text, mask: proseMaskReading({ text, },).mask, },).straightDouble; // 2
 ```
 */
function tallyQuotes(
  {
    tally,
    text,
    mask,
  }: {
    readonly tally: QuoteTally;
    readonly text: string;
    readonly mask: readonly boolean[];
  },
): QuoteTally {
  // The mask holds one flag per UTF-16 unit, so the text is read by unit: a
  // handle in astral letters would shift a code-point index off the mask.
  return mask.reduce(
    function count(
      counted,
      prose,
      index,
    ): QuoteTally {
      if (!prose)
        return counted;
      /**
       Unit under the scan.
       */
      const character = text.charAt(index,);
      if (character === STRAIGHT_DOUBLE)
        return {
          ...counted,
          straightDouble: counted.straightDouble + 1,
        };
      if (CURLY_DOUBLES.includes(character,))
        return {
          ...counted,
          curlyDouble: counted.curlyDouble + 1,
        };
      if (character === STRAIGHT_SINGLE)
        return {
          ...counted,
          straightSingle: counted.straightSingle + 1,
        };
      if (character === CURLY_SINGLE)
        return {
          ...counted,
          curlySingle: counted.curlySingle + 1,
        };
      return counted;
    },
    tally,
  );
}

/**
 What the page pass read of the page's slices once: the prose mask of each carried text, and the page's definitions the masks were
 read with.
 */
type PageReadings = {
  /**
   The reading of each carried text, by the text.
   */
  readonly masks: ReadonlyMap<string, ProseMaskReading>;

  /**
   The page's link and footnote definitions, which a reference in a slice resolves against whichever slice holds the definition.
   */
  readonly definitions: string;
};

/**
 The reading of a text the page carries: the one made when the page was read, or a new one when the text was not among them.

 @param page - what the page pass read of the slices

 @param text - the text of a slice as the page carries it

 @returns The text's prose mask and whether the grammar refused it

 @example
 ```ts
 const { mask, } = readingOf({ page, text: 'The cat said "meow".', },);
 ```
 */
function readingOf(
  {
    page,
    text,
  }: {
    readonly page: PageReadings;
    readonly text: string;
  },
): ProseMaskReading {
  /**
   What the page pass read of the slices.
   */
  const {
    masks,
    definitions,
  } = page;
  /**
   The reading made when the page was read, none for a text the page did not carry then.
   */
  const known = masks.get(text,);
  if (known !== undefined)
    return known;
  return proseMaskReading({
    text,
    definitions,
  },);
}

/**
 Reads the prose mask of every body slice the page carries once, with the page's definitions, and warns of each slice the grammar
 refuses.

 @param slices - prepared pairs

 @param carried - text the page carries per slice

 @returns The readings by carried text, and the definitions they were read with

 @example
 ```ts
 const page = readPage({ slices, carried, },);
 ```
 */
function readPage(
  {
    slices,
    carried,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly carried: ReturnType<typeof pageTextBySlice>;
  },
): PageReadings {
  /**
   The body slices and the text the page carries at each.
   */
  const bodies = slices
    .filter(function isBody(slice,): boolean {
      return slice.syntax !== 'front-matter';
    },)
    .map(function bodyOf(slice,): {
      readonly sliceIndex: number;
      readonly text: string;
    } {
      return {
        sliceIndex: slice.target
          .sliceIndex,
        text: pageTextOf({
          pageText: carried,
          sliceIndex: slice.target
            .sliceIndex,
        },),
      };
    },);
  /**
   The page's definitions, from the slices that carry text.
   */
  const definitions = pageDefinitionsOf({
    pageText: bodies
      .map(function textOf({ text, },): string {
        return text;
      },)
      .join('\n\n',),
  },);
  /**
   The readings, mutated only inside this function.
   */
  const masks = new Map<string, ProseMaskReading>();
  for (const {
    sliceIndex,
    text,
  } of bodies) {
    if (masks.has(text,))
      continue;
    /**
     The one reading of this slice's text.
     */
    const reading = proseMaskReading({
      text,
      definitions,
    },);
    masks.set(
      text,
      reading,
    );
    if (reading.refused) {
      noteRefusedMask({
        reader: READER,
        name: `slice ${String(sliceIndex,)}`,
      },);
    }
  }
  return {
    masks,
    definitions,
  };
}

/**
 The curly marks the page writes more often than their straight forms, as a
 convention the typography restoration reads; empty where neither mark is
 curly on most of the page.

 @param slices - prepared pairs

 @param carried - text the page carries per slice

 @param page - the prose mask of each carried text, read once, with the page's definitions

 @returns Convention text holding the curly marks the page mostly uses

 @example
 ```ts
 const convention = pageQuoteConvention({ slices, carried, page, },);
 ```
 */
function pageQuoteConvention(
  {
    slices,
    carried,
    page,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly carried: ReturnType<typeof pageTextBySlice>;
    readonly page: PageReadings;
  },
): string {
  /**
   Every prose quote mark on the page outside the front matter.
   */
  const tally = slices
    .filter(function isBody(slice,): boolean {
      return slice.syntax !== 'front-matter';
    },)
    .reduce(
    function add(
      counted,
      slice,
    ): QuoteTally {
        /**
         The text the page carries at the slice.
         */
      const text = pageTextOf({
        pageText: carried,
        sliceIndex: slice.target
          .sliceIndex,
      },);
        /**
         The reading of the text, whose mask the tally reads.
         */
      const { mask, } = readingOf({
        page,
        text,
      },);
      return tallyQuotes({
        tally: counted,
        text,
        mask,
      },);
    },
    EMPTY_TALLY,
    );
  /**
   Whether curly doubles are the page's form.
   */
  const curlyDoubles = tally.curlyDouble > tally.straightDouble;
  /**
   Whether the curly apostrophe is the page's form.
   */
  const curlySingles = tally.curlySingle > tally.straightSingle;
  return `${curlyDoubles ? CURLY_DOUBLES : ''}${curlySingles ? CURLY_SINGLE : ''}`;
}

/**
 Counts the UTF-16 units two texts of one length differ in; curling a mark
 swaps one unit for one unit, so the lengths agree.

 @param before - text as the page carried it

 @param after - text after the restoration

 @returns Units changed

 @example
 ```ts
 changedUnits({ before: '"a"', after: '“a”', },); // 2
 ```
 */
function changedUnits(
  {
    before,
    after,
  }: {
    readonly before: string;
    readonly after: string;
  },
): number {
  /**
   Whether each unit differs.
   */
  const differences = Array.from(
    { length: after.length, },
    function differs(
      _unused,
      index,
    ): boolean {
      return after.charAt(index,) !== before.charAt(index,);
    },
  );
  return differences
    .filter(Boolean,)
    .length;
}

/**
 Brings every slice's prose quote marks to the page's majority style, the
 slices no lane replaced included.

 @param slices - prepared pairs, whose archive text stands where no row replaces it

 @param replacements - what the page would write per slice

 @param archiveOriginalSpans - spans sealed as the English original

 @returns Replacements with the marks curled (a row added for an archive slice
 the pass changed), the rewritten rows alone, and one finding per slice changed

 @example
 ```ts
 const page = unifyQuoteStyle({ slices, replacements, archiveOriginalSpans: [], },);
 ```
 */
export function unifyQuoteStyle(
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
  /**
   Text the page carries per slice.
   */
  const carried = pageTextBySlice({
    slices,
    replacements,
  },);
  /**
   What the page's slices read once: their prose masks and the page's definitions.
   */
  const page = readPage({
    slices,
    carried,
  },);
  /**
   Curly marks the page mostly writes.
   */
  const convention = pageQuoteConvention({
    slices,
    carried,
    page,
  },);
  if (convention === '') {
    return {
      replacements,
      restored: [],
      findings: [],
    };
  }
  return rewriteEverySlice({
    slices,
    replacements,
    archiveOriginalSpans,
    rewrite: function curl({ text, },): TextRewrite {
      /**
       Text with the page's curly marks restored.
       */
      const curled = restoreTypography({
        replacement: text,
        replaced: '',
        convention,
        reading: readingOf({
          page,
          text,
        },),
      },);
      if (curled === text) {
        return {
          text,
          changed: [],
        };
      }
      return {
        text: curled,
        changed: [`${changedUnits({
          before: text,
          after: curled,
        },)} quote marks to the page's curly style`,],
      };
    },
    findingName: 'quote-style-unified',
  },);
}

//endregion Quote style unify
