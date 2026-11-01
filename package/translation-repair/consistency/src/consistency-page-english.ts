import {
  sharedPrefixLength,
  sharedSuffixLength,
} from '@monochromatic-dev/translation-repair-assembly/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  isJsonRecord,
  requireFrontMatterRefusal,
  splitFrontMatter,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  type PlacedSlice,
  placeSlices,
  type SliceReplacement,
} from '@monochromatic-dev/translation-repair-page-pass/ts';
import { ConsistencyInvariantError, } from './consistency-invariant-error.ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { blankedOf, } from './consistency-prose-text.ts';

//region Consistency page English
// THE PAGE THE PROOF OF AN APOSTROPHE READS IS THE PAGE THAT SHIPS, in the glyphs that ship, with
// the replacements the pass is trying written into it. A quotation runs over paragraphs and over
// headings, and a closing `’` after `kittens` whose `‘` opened in an earlier slice is the end of
// that quotation, so a proof that reads one slice alone takes it for a plural possessive. The
// glyphs are the published ones, not the lane's: the replacements enter the page before typography
// curls the quotation marks of a wording the archive did not decide (`would-ship-text.ts`,
// `inArchiveTypography`) and before the page-assembly passes rewrite the composed page (a quote
// style unified, a spelling made Canadian, a quotation taken off a blockquote;
// `page-assembly-passes.ts`), so a mark that is half of a pair on the page that ships need not be
// half of one on the lane's English, and a mark the proof proved on the lane's English can be
// gone on the page. The pass holds the English of every slice of the page it works on
// (`pass-consistency-slices.ts`), and this file lays them out with the text the archive carries
// between them: the slices that carry English, in the order of the page, and between each two
// what stands there on the published page.
//
// THE TEXT BETWEEN THE SLICES IS READ, NOT DROPPED. The page the archive ships has text
// between its slices that no slice holds: a sealed span the translators' note calls the
// English original, the blank lines between blocks, a block no slice claims. A quotation
// character in a sealed span can pair with a mark of a slice, and the reader of the page
// cannot know the span is a copy, so it reads the span as the page ships it. A caller that
// holds the slices alone (a unit that reads one English) gives no text between, and the slices
// are then joined by a blank line, a block break, which the proof does not read as an end to
// a quotation. A slice with no English contributes nothing, as on the page.
//
// THE FRONT MATTER OF THE PAGE IS NO PROSE. The first slice of a page the lanes render is its
// metadata, fenced by `---` lines, and a quotation mark in a title is no partner of a mark in
// the words below it. The front matter at the start of the page (the text before the first
// slice and the first English) is blanked to spaces and line breaks, so no offset moves. A
// text whose fences the YAML grammar refuses is read as it stands, where every mark could be
// a partner, which can only keep a mark from being proven.
//
// THE PAGE IS ASSEMBLED BY THE CODE THAT PUBLISHES IT when the caller hands the layout of the page
// (`PageLayout`: the target text, the slice pairs, what the slices that ship no English write, and the
// writer of what every slice ships for the English given, `shipped`, which is the publishing's own
// code in `pass-consistency-shipped.ts`: the artifact composed with the English as consistency rows,
// the page-assembly guard over it, and `shippableReplacements`). `placeSlices` in `splice-slices.ts`
// writes what the writer gives into the spans, matching the line endings at the edge of a span and
// putting separators around an inserted anchor as it does for the published page, and reports where
// each slice's English starts in the text it wrote. A layout without a writer writes the English as it
// is given. A caller that holds the slices alone, or the text between them without the pairs, gets the
// page joined here (a blank line, or the pieces given): that page is a reading of text the caller
// gives, not of the published page.
//
// THE ENGLISH A READER HOLDS AND WHAT THE PAGE HOLDS FOR THE SLICE CAN DIFFER. Typography curls a
// quotation mark and a pass changes a spelling, so the page's words for a slice are not always the
// reader's (`PageEnglish.shipped`, `ReadPage.shipped`). `positionOnPage` carries a position of the
// reader's English to the page: the same where the length is the same, shifted where the words
// share their start or their end, and none for a character between two changes, which a reader
// takes as not proven.

/**
 What separates the English of two slices on the page the proof reads when no text between is given: a blank line.
 */
export const PAGE_SLICE_SEPARATOR = '\n\n';

/**
 The English of a whole page and where one slice's English starts in it.

 @example
 ```ts
 const page: PageEnglish = { text: 'Fill the bowl.\n\nThen nap.', offset: 16, };
 ```
 */
export type PageEnglish = {
  /**
   The published page: its slices that carry English in page order with the text the archive carries between them
   (a blank line between each two where none is given), the front matter blanked.
   */
  readonly text: string;

  /**
   Offset in the text where the English of the slice asked about starts.
   */
  readonly offset: number;

  /**
   What the page holds at that offset for the slice, when it is not the English the reader holds: typography or a page-assembly
   pass rewrote the slice's words, so a position of the reader's English is a position of this text through
   `positionOnPage`. Absent where the page holds the reader's English itself.
   */
  readonly shipped?: string;
};

/**
 One slice of a page that carries English.

 @example
 ```ts
 const slice: PageSlice = { sliceIndex: 3, english: 'Then nap.', };
 ```
 */
export type PageSlice = {
  /**
   Index the artifact names the slice by.
   */
  readonly sliceIndex: number;

  /**
   The slice's English.
   */
  readonly english: string;
};

/**
 The English with the front matter at its start blanked.

 @param english - the English of the first slice of a page

 @returns The English as long as it was, with the fenced block at its start (a mapping, as metadata is) turned into spaces
 and line breaks, or the English as it stands where it opens with none or its YAML is refused

 @example
 ```ts
 const prose = withoutFrontMatter({ english: '---\ntitle: it\'s\n---\n\nFill the bowl.', },); // the title line is blank
 ```
 */
function withoutFrontMatter({ english, }: { readonly english: string; },): string {
  try {
    /**
     The English split into its front matter and its body.
     */
    const { frontMatter, } = splitFrontMatter({ text: english, },);
    if (frontMatter === undefined)
      return english;
    if (!isJsonRecord(frontMatter.data,))
      return english;
    /**
     The fenced block as it stands in the English, and where it starts.
     */
    const { raw, } = frontMatter;
    /**
     What stands before the block.
     */
    const before = english.slice(
      0,
      frontMatter.startOffset,
    );
    /**
     What stands after it.
     */
    const after = english.slice(frontMatter.startOffset + raw.length,);
    return `${before}${blankedOf({ text: raw, },)}${after}`;
  }
  catch (error) {
    // Only the grammar's own refusal is an English to be read as it stands; anything else keeps propagating.
    requireFrontMatterRefusal({ error, },);
    return english;
  }
}

/**
 The page a reader of one slice reads: the page it was handed, or the slice alone where it was handed none.

 @param english - the English of the slice as its place holds it

 @param page - the English of the whole page and where the slice's English starts in it, absent for a caller that holds one
 English alone, whose slice is then the page

 @returns The page's English and where the slice starts in it

 @throws {@link ConsistencyInvariantError} when the page is too short to hold the slice at its offset, since a span of the
 slice would then name no words of the page

 @example
 ```ts
 const { text, offset, } = pageReadingOf({ english: 'Then nap.', page: { text: 'Fill the bowl.\n\nThen nap.', offset: 16, }, },);
 ```
 */
export function pageReadingOf(
  {
    english,
    page,
  }: {
    readonly english: string;
    readonly page?: PageEnglish;
  },
): PageEnglish {
  /**
   The English the proof reads, and where the slice starts in it.
   */
  const reading = page ?? {
    text: english,
    offset: 0,
  };
  /**
   Where the slice ends in the page: its words as the page holds them, which are the reader's unless typography or a
   page-assembly pass rewrote them.
   */
  const englishEnd = reading.offset + (reading.shipped ?? english).length;
  /**
   Where the page ends.
   */
  const { text: pageText, } = reading;
  /**
   How long the page is.
   */
  const pageEnd = pageText.length;
  if ((reading.offset < 0) || (englishEnd > pageEnd)) {
    throw new ConsistencyInvariantError({ invariant:
      `the page English handed to the proof ends at ${String(pageEnd,)} and the slice spans ${
        String(reading.offset,)
      } to ${String(englishEnd,)} in it, so the slice does not lie inside its page`, },);
  }
  return reading;
}

/**
 What the published page is assembled from: the target text, the slices cut out of it, and what the slices that ship no
 English write, which is the same input the run's publishing hands the assembler (`splice-slices.ts`).

 @example
 ```ts
 const layout: PageLayout = { targetText: 'Fill the bowl.\n', chunks: [], silent: [], };
 ```
 */
export type PageLayout = {
  /**
   The translation the slices were cut from.
   */
  readonly targetText: string;

  /**
   The slice pairs in document order, with the spans the assembler writes into.
   */
  readonly chunks: readonly ChunkPair[];

  /**
   What the slices that ship no English write: nothing for an anchor, an empty text over a span. A slice that ships English is
   not among them; its English is handed with the slices asked about.
   */
  readonly silent: readonly SliceReplacement[];

  /**
   Writes what every slice ships when the slices carry the English given, by the code that publishes the page: the wording of
   each slice through typography, the page-assembly passes and their guard, as the publishing runs them over a page whose
   consistency rows hold that English. Absent for a layout built by hand, where the English given is written as it stands and
   `silent` is written beside it.
   */
  readonly shipped?: (arguments_: { readonly slices: readonly PageSlice[]; },) => readonly SliceReplacement[];
};

/**
 What a reader of a page is given.
 */
export type ReadPage = {
  /**
   The page's English, the front matter at its start blanked.
   */
  readonly text: string;

  /**
   Where the English of each slice starts in the text, by the slice's index; a slice that writes nothing has none.
   */
  readonly offsets: ReadonlyMap<number, number>;

  /**
   What the page holds for each slice at that offset, by the slice's index: the English given where the page is built from
   the slices alone, and what the publishing's own code ships where the layout writes it (`PageLayout.shipped`).
   */
  readonly shipped: ReadonlyMap<number, string>;

  /**
   The text before the first slice, between each two and after the last, one piece more than there are slices.
   */
  readonly around: readonly string[];
};

/**
 The page the assembler writes, read.

 THE PAGE IS ASSEMBLED BY THE CODE THAT PUBLISHES IT (`placeSlices` in `splice-slices.ts`), with the English asked about in
 each slice that ships English and what the other slices write, so the line endings the assembler matches at the edge of a
 span, the separators it puts around an anchor and the text of a sealed span are the published page's, not a model of it. The
 front matter at the start of the page is blanked as it is for the page read without a layout.

 @param slices - the slices that carry English, in page order, with the English each is read with

 @param layout - what the published page is assembled from

 @returns The page, where each slice's English starts in it, and the text around the slices

 @throws {@link Error} when the assembler refuses the slices, which means the layout and the slices disagree

 @example
 ```ts
 const { text, offsets, } = laidOutPage({ slices, layout, },);
 ```
 */
function laidOutPage(
  {
    slices,
    layout,
  }: {
    readonly slices: readonly PageSlice[];
    readonly layout: PageLayout;
  },
): ReadPage {
  /**
   What every slice writes: what the publishing's code ships for the English given where the layout carries it, else the
   English as it stands with what the slices that ship none write beside it.
   */
  const replacements = (layout.shipped === undefined)
    ? [
      ...slices.map(function replacementOf({
        sliceIndex,
        english,
      },): SliceReplacement {
        return {
          sliceIndex,
          replacementText: english,
        };
      },),
      ...layout.silent,
    ]
    : layout.shipped({ slices, },);
  /**
   The page assembled with what the slices write in place.
   */
  const {
    text,
    placed,
  } = placeSlices({
    targetText: layout.targetText,
    slices: layout.chunks,
    replacements,
  },);
  /**
   What each slice writes, by the slice's index, mutated only inside this function.
   */
  const shipped = new Map<number, string>();
  for (const {
    sliceIndex,
    replacementText,
  } of replacements) {
    shipped.set(
      sliceIndex,
      replacementText,
    );
  }
  /**
   The range each slice's words occupy and where its English starts, by the slice's index, mutated only inside this function.
   */
  const byIndex = new Map<number, PlacedSlice>();
  for (const entry of placed) {
    byIndex.set(
      entry.sliceIndex,
      entry,
    );
  }
  /**
   Where the text around the slices ends and starts, mutated only inside this function.
   */
  const reached = { end: 0, };
  /**
   The text before each slice, in page order.
   */
  const around = slices.map(function beforeSlice({ sliceIndex, },): string {
    /**
     The range this slice's words occupy; none for a slice that writes nothing, which stands where the text so far ends.
     */
    const range = byIndex.get(sliceIndex,);
    /**
     Where the range starts, no earlier than the text reached.
     */
    const start = Math.max(
      range?.start ?? reached.end,
      reached.end,
    );
    /**
     The text before the slice.
     */
    const before = text.slice(
      reached.end,
      start,
    );
    reached.end = Math.max(
      range?.end ?? start,
      start,
    );
    return before;
  },);
  /**
   Where each slice's English starts, by the slice's index; none for a slice that writes nothing. Mutated only inside this
   function.
   */
  const offsets = new Map<number, number>();
  for (const { sliceIndex, } of slices) {
    /**
     The range this slice's words occupy.
     */
    const range = byIndex.get(sliceIndex,);
    if (range !== undefined) {
      offsets.set(
        sliceIndex,
        range.englishAt,
      );
    }
  }
  /**
   The first slice, whose words end where the front matter can reach furthest.
   */
  const [first,] = slices;
  /**
   The range the first slice's words occupy.
   */
  const firstRange = (first === undefined) ? undefined : byIndex.get(first.sliceIndex,);
  /**
   Where the first slice's words end, zero where it writes nothing.
   */
  const firstEnd = firstRange?.end ?? 0;
  /**
   The start of the page, where the front matter can stand.
   */
  const front = text.slice(
    0,
    firstEnd,
  );
  return {
    text: `${withoutFrontMatter({ english: front, },)}${text.slice(firstEnd,)}`,
    offsets,
    shipped,
    around: [
      ...around,
      text.slice(reached.end,),
    ],
  };
}

/**
 Joins the English of the slices of a page into the published page's English.

 @param slices - the slices of the page that carry English, in page order

 @param between - the text the page carries before the first slice, between each two and after the last, one piece more
 than there are slices; absent for a caller that holds the slices alone, whose slices are then a blank line apart

 @returns The page's English, and the offset where the English of each slice starts in it, by the slice's index

 @throws {@link ConsistencyInvariantError} when the text between the slices does not hold one more piece than there are
 slices, since a slice would then stand where no text says it goes

 @example
 ```ts
 const { text, offsets, } = pageOfSlices({ slices: [{ sliceIndex: 0, english: '## One\n\nFill the bowl.', }, { sliceIndex: 2, english: 'Then nap.', },], },); // offsets: 0 and 24
 ```
 */
export function pageOfSlices(
  {
    slices,
    between,
    layout,
  }: {
    readonly slices: readonly PageSlice[];
    readonly between?: readonly string[];
    readonly layout?: PageLayout;
  },
): ReadPage {
  if (layout !== undefined) {
    return laidOutPage({
      slices,
      layout,
    },);
  }
  /**
   How many slices the page has.
   */
  const { length: sliceCount, } = slices;
  /**
   How many pieces of text the page carries around them: one more than there are slices.
   */
  const pieceCount = sliceCount + 1;
  /**
   The text around the slices: the caller's, or a blank line between each two and nothing at the ends.
   */
  const around = between ?? Array.from(
    { length: pieceCount, },
    function separatorAt(
      _slice,
      at,
    ): string {
      return ((at === 0) || (at === sliceCount)) ? '' : PAGE_SLICE_SEPARATOR;
    },
  );
  /**
   How many pieces of text the caller gave.
   */
  const { length: givenCount, } = around;
  if (givenCount !== pieceCount) {
    /**
     The noun that counts the pieces given.
     */
    const pieceNoun = wordForCount({
      count: givenCount,
      one: 'piece',
      many: 'pieces',
    },);
    /**
     The noun that counts the slices.
     */
    const sliceNoun = wordForCount({
      count: sliceCount,
      one: 'slice',
      many: 'slices',
    },);
    throw new ConsistencyInvariantError({ invariant:
      `the text around the slices of the page holds ${String(givenCount,)} ${pieceNoun} for ${String(sliceCount,)} ${sliceNoun}, and it must hold one more piece than there are slices: the text before the first, between each two and after the last`, },);
  }
  /**
   What the page holds, in order, mutated only inside this function.
   */
  const pieces: string[] = [];
  /**
   Where each slice's English starts, by the slice's index, mutated only inside this function.
   */
  const offsets = new Map<number, number>();
  /**
   What the page holds for each slice, by the slice's index: the English given, mutated only inside this function.
   */
  const shipped = new Map<number, string>();
  /**
   The offset the text has reached, and where the first slice's English ends (as far as the front matter can reach), mutated
   only inside this function.
   */
  const reached = {
    length: 0,
    firstEnd: (around[0] ?? '').length,
  };
  for (const [position, slice,] of slices.entries()) {
    /**
     The text that stands before this slice.
     */
    const before = around[position] ?? '';
    /**
     The slice's English.
     */
    const { english, } = slice;
    pieces.push(before,);
    reached.length += before.length;
    offsets.set(
      slice.sliceIndex,
      reached.length,
    );
    pieces.push(english,);
    shipped.set(
      slice.sliceIndex,
      english,
    );
    reached.length += english.length;
    if (position === 0)
      reached.firstEnd = reached.length;
  }
  pieces.push(around[sliceCount] ?? '',);
  /**
   The page before its front matter is blanked.
   */
  const joined = pieces.join('',);
  /**
   The start of the page, where the front matter can stand.
   */
  const front = joined.slice(
    0,
    reached.firstEnd,
  );
  /**
   The rest of the page, which has no front matter.
   */
  const rest = joined.slice(reached.firstEnd,);
  return {
    text: `${withoutFrontMatter({ english: front, },)}${rest}`,
    offsets,
    shipped,
    around,
  };
}

/**
 What {@link positionOnPage} answers for a position of the reader's English that names a character the page no longer holds.
 */
export const NO_PLACE: unique symbol = Symbol('answer for a character that publishing rewrote on the page, so no position there names it',);

/**
 Where a position of the English a reader holds stands on the page, which holds that English as the publishing ships it.

 The page's words and the reader's are the same length where typography only curled a quotation mark, and a position stands
 where it did. Where a page-assembly pass changed the length (a spelling, a gloss, a quotation mark taken off), the two share
 the start and the end they have in common and nothing is said of a position in between: it names a character that is no longer
 the reader's, so it has no place.

 @param written - the English the reader holds for the slice

 @param shipped - what the page holds for the slice; the reader's English itself where the page holds no other

 @param offset - where the slice starts in the page

 @param at - the position in the reader's English

 @returns The position on the page, {@link NO_PLACE} where the page's words changed the character the reader names

 @example
 ```ts
 const place = positionOnPage({ written: 'The color of the dogs’', shipped: 'The colour of the dogs’', offset: 20, at: 21, },); // 42
 ```
 */
export function positionOnPage(
  {
    written,
    shipped,
    offset,
    at,
  }: {
    readonly written: string;
    readonly shipped: string;
    readonly offset: number;
    readonly at: number;
  },
): number | typeof NO_PLACE {
  if ((shipped === written) || (shipped.length === written.length))
    return offset + at;
  /**
   How many characters the two share at their start.
   */
  const sharedAtStart = sharedPrefixLength({
    left: written,
    right: shipped,
  },);
  if (at < sharedAtStart)
    return offset + at;
  /**
   How many characters the two share at their end, without taking any of the start twice.
   */
  const sharedAtEnd = sharedSuffixLength({
    left: written,
    right: shipped,
    prefix: sharedAtStart,
  },);
  if (at >= (written.length - sharedAtEnd))
    return (offset + at) + (shipped.length - written.length);
  return NO_PLACE;
}

//endregion Consistency page English
