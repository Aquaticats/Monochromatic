import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  straightenQuotes,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import {
  carriesContent,
  pastQuoteMarkers,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';

//region Bilingual pair bound
// CLASS EIGHTY, SECOND ARM (shi_Yumiaoya11 and 12, 2026-09-22). The sheet
// clause of `bilingual-line-clause.ts` reached the translators, the judges
// and the editors, and the slate still chose the candidate that rendered the
// film quote's Chinese line a second time in English: deepseek-v4.1-flash
// called the candidate carrying the English alone "a dropped line ...
// ineligible" under the line criterion, and gemma-4-26b reasoned that the
// Chinese line "is a literal translation of the sentiment, which is distinct
// from the actual English quote". A judge without the film cannot see that
// the two lines are one, and no wording on a sheet gives it the film.
//
// THE EXISTING TRANSLATION SETTLES IT, BLOCK BY BLOCK. The human translator
// carried the pair as one line, and that text is on the sheet as the PAGE AS
// IT STANDS. A first cut compared the whole slice's line count with the lines
// owed and slept on shi_Yumiaoya12, because the archive writes the slice's
// closing line as two lines, so the whole-slice count never matched. The
// bound now finds the pair's English line in the page and in the rendering
// and compares the block holding it (the quote's lines, or the paragraph's):
// a rendering whose block carries more lines than the page's has rendered
// the Chinese half again, and is refused before the judges see it, the way
// class sixty-six withholds a rule-refused candidate. Where there is no page
// text, the page does not carry that English line as written, the page
// carries the Chinese line too, or the rendering reworded the English line,
// nothing is refused and the judges decide on the clause. Arita,
// NIGHT81473140, gqt and seven other entries carry a Chinese line followed by
// an English line of DIFFERENT content, which the pair count cannot tell
// from a translation pair; their pages carry both lines in the block, so the
// bound is silent there.

/**
 One line the original gives in Chinese with its English beside it.

 @example
 ```ts
 const pair: BilingualPair = { han: '> 如果再也见不到猫，祝你早安。', english: '> And in case I don’t see the cat, good morning.', };
 ```
 */
export type BilingualPair = {
  /**
   Han line as the original carries it.
   */
  readonly han: string;

  /**
   English line beside it as the original carries it.
   */
  readonly english: string;
};

/**
 Block of consecutive lines holding one line: where it starts and how many
 content lines it carries.
 */
type LineBlock = {
  /**
   Index of the block's first line.
   */
  readonly start: number;

  /**
   Lines in the block carrying content.
   */
  readonly count: number;
};

/**
 Whether a line is inside a quote, by its marker.

 @param line - one raw line

 @returns True on a leading quote marker

 @example
 ```ts
 isQuoted({ line: '> From *The Cat Show*', },); // true
 ```
 */
function isQuoted({ line, }: { readonly line: string; },): boolean {
  /**
   Line without its leading whitespace.
   */
  const lead = line.trimStart();
  return lead.startsWith('>',);
}

/**
 Wording of a line for matching across page and rendering: quote markers
 and surrounding whitespace off, inner whitespace collapsed, curled quotes
 straightened by the typography fold (`straightenQuotes`, ledger B24), so a
 line matches its page copy through the typography restoration.

 @param line - one raw line

 @returns Wording, empty for a line carrying none

 @example
 ```ts
 wordingOf({ line: '>  From *The Cat Show*', },); // 'From *The Cat Show*'
 ```
 */
function wordingOf({ line, }: { readonly line: string; },): string {
  /**
   Words of the line past its markers.
   */
  const words = pastQuoteMarkers({ line, },)
    .split(' ',)
    .map(function trimWord(word,): string {
      return word.trim();
    },)
    .filter(function nonEmpty(word,): boolean {
      return word !== '';
    },);
  return straightenQuotes({ text: words.join(' ',), },);
}

/**
 Index of every line carrying a wording, in order.

 EVERY LINE, NOT THE FIRST (ledger B203, whose census found this site). A
 refrain stands on more than one line of a text, and each stands in a block
 of its own.

 @param lines - raw lines to search

 @param wording - wording to find, as `wordingOf` gives it

 @returns Indexes in order, empty when no line carries it

 @example
 ```ts
 const at = linesAt({ lines, wording: wordingOf({ line: pair.english, },), },);
 ```
 */
function linesAt(
  {
    lines,
    wording,
  }: {
    readonly lines: readonly string[];
    readonly wording: string;
  },
): readonly number[] {
  return lines.flatMap(function indexIfCarries(
    line,
    index,
  ): readonly number[] {
    return (wordingOf({ line, },) === wording) ? [index,] : [];
  },);
}

/**
 Whether a line belongs to the block of a quoted or an unquoted line.

 @param line - raw line to test

 @param quoted - whether the block is a quote

 @returns True inside the same block

 @example
 ```ts
 belongsToBlock({ line: '>', quoted: true, },); // true
 ```
 */
function belongsToBlock(
  {
    line,
    quoted,
  }: {
    readonly line: string;
    readonly quoted: boolean;
  },
): boolean {
  if (quoted)
    return isQuoted({ line, },);
  if (isQuoted({ line, },))
    return false;
  return carriesContent({ line, },);
}

/**
 First index of the block holding a line.

 @param lines - raw lines of the text

 @param at - index of the line

 @param quoted - whether the block is a quote

 @returns Index of the block's first line

 @example
 ```ts
 const start = blockStart({ lines, at, quoted, },);
 ```
 */
function blockStart(
  {
    lines,
    at,
    quoted,
  }: {
    readonly lines: readonly string[];
    readonly at: number;
    readonly quoted: boolean;
  },
): number {
  for (let start = at; start > 0; start -= 1) {
    /**
     Line before the cursor. ALWAYS PRESENT: `start` runs down from `at`, an
     index of `lines`, and stays above 0, and a `split` leaves no hole.
     */
    const before = nonNullishOrThrow(lines[start - 1],);
    if (!belongsToBlock({
      line: before,
      quoted,
    },))
      return start;
  }
  return 0;
}

/**
 Index one past the block holding a line.

 @param lines - raw lines of the text

 @param at - index of the line

 @param quoted - whether the block is a quote

 @returns Index after the block's last line

 @example
 ```ts
 const end = blockEnd({ lines, at, quoted, },);
 ```
 */
function blockEnd(
  {
    lines,
    at,
    quoted,
  }: {
    readonly lines: readonly string[];
    readonly at: number;
    readonly quoted: boolean;
  },
): number {
  for (let end = at + 1; end < lines.length; end += 1) {
    /**
     Line at the cursor. ALWAYS PRESENT: the loop stops at `lines.length`,
     and a `split` leaves no hole.
     */
    const here = nonNullishOrThrow(lines[end],);
    if (!belongsToBlock({
      line: here,
      quoted,
    },))
      return end;
  }
  return lines.length;
}

/**
 Block holding one line: the run of quoted lines around a quoted line, or
 the run of unquoted content lines around an unquoted one.

 @param lines - raw lines of the text

 @param at - index of the line whose block is wanted

 @returns Where the block starts and how many content lines it carries

 @example
 ```ts
 const block = blockAround({ lines, at, },);
 ```
 */
function blockAround(
  {
    lines,
    at,
  }: {
    readonly lines: readonly string[];
    readonly at: number;
  },
): LineBlock {
  /**
   Whether the block is a quote. The line at `at` is ALWAYS PRESENT:
   `pairBoundFindings` passes only a `linesAt` answer, an index of these same
   lines.
   */
  const quoted = isQuoted({ line: nonNullishOrThrow(lines[at],), },);
  /**
   First index of the block.
   */
  const start = blockStart({
    lines,
    at,
    quoted,
  },);
  /**
   Index one past the block.
   */
  const end = blockEnd({
    lines,
    at,
    quoted,
  },);
  /**
   Content lines inside the block.
   */
  const counted = lines
    .slice(
    start,
    end,
    )
    .filter(function carries(line,): boolean {
      return carriesContent({ line, },);
    },);
  return {
    start,
    count: counted.length,
  };
}

/**
 Names a governed rendering whose block holding a bilingual pair's English
 line carries more lines than the page's block holding it.

 A wording standing on several lines (a refrain) is compared place by place,
 the page's first place with the rendering's first, and so on, where both
 texts carry it equally often; where they do not, the bound says nothing.

 @param pairs - Han lines the original gives with their own English beside
 them, as the floor found them

 @param pageText - translation of the slice as the page stands, absent or
 empty where the slice has none

 @param candidateText - proposed rendering

 @returns One finding per block the rendering widened, none otherwise

 @example
 ```ts
 const found = pairBoundFindings({ pairs, pageText, candidateText, },);
 ```
 */
export function pairBoundFindings(
  {
    pairs,
    pageText,
    candidateText,
  }: {
    readonly pairs: readonly BilingualPair[];
    readonly pageText?: string;
    readonly candidateText: string;
  },
): readonly string[] {
  if ((pairs.length === 0) || (pageText === undefined))
    return [];
  /**
   Page text without its surrounding whitespace.
   */
  const trimmedPage = pageText.trim();
  if (trimmedPage === '')
    return [];
  /**
   Raw lines of the page.
   */
  const pageLines = pageText.split('\n',);
  /**
   Raw lines of the rendering.
   */
  const candidateLines = candidateText.split('\n',);
  /**
   Starts of rendering blocks already named, so two pairs in one quote name
   it once.
   */
  const named = new Set<number>();
  return pairs.flatMap(function toFinding(pair,): readonly string[] {
    /**
     Wording of the pair's English line.
     */
    const english = wordingOf({ line: pair.english, },);
    /**
     Where the page carries that line, in order.
     */
    const pageAts = linesAt({
      lines: pageLines,
      wording: english,
    },);
    if (pageAts.length === 0)
      return [];
    /**
     Where the page carries the Han line, if it kept both.
     */
    const pageHanAts = linesAt({
      lines: pageLines,
      wording: wordingOf({ line: pair.han, },),
    },);
    if (pageHanAts.length > 0)
      return [];
    /**
     Where the rendering carries the English line, in order.
     */
    const candidateAts = linesAt({
      lines: candidateLines,
      wording: english,
    },);
    // THE WORDING MUST STAND AS OFTEN IN BOTH TEXTS (ledger B203, whose census
    // found this site): then its places pair off in order, each place of the
    // page with the same place of the rendering. Where the counts differ, no
    // place of the rendering is known to be a page place, so the bound says
    // nothing and the repeated-line check speaks to a rendering that
    // repeats the line.
    if (candidateAts.length !== pageAts.length)
      return [];
    return pageAts.flatMap(function toPlaceFinding(
      pageAt,
      place,
    ): readonly string[] {
      /**
       Where the rendering carries the line at this place. ALWAYS PRESENT:
       `place` indexes `pageAts`, which has as many entries as
       `candidateAts`.
       */
      const candidateAt = nonNullishOrThrow(candidateAts[place],);
      /**
       Page block holding the line.
       */
      const pageBlock = blockAround({
        lines: pageLines,
        at: pageAt,
      },);
      /**
       Rendering block holding the line.
       */
      const candidateBlock = blockAround({
        lines: candidateLines,
        at: candidateAt,
      },);
      if ((candidateBlock.count <= pageBlock.count) || named.has(candidateBlock.start,))
        return [];
      named.add(candidateBlock.start,);
      return [
        `This slice is LINE-STRUCTURED and the ORIGINAL gives the line \`${english}\` twice, `
          + `once in Chinese and once in English directly beside it; that pair is ONE line whose English is `
          + `already its rendering, and the EXISTING TRANSLATION carries the block holding it as `
          + `${String(pageBlock.count,)} ${
            wordForCount({
              count: pageBlock.count,
              one: 'line',
              many: 'lines',
            },)
          }. Yours carries ${String(candidateBlock.count,)}. Drop the second `
          + `rendering of the pair (the Chinese line, or a second English wording of it), keeping the wording `
          + `you chose elsewhere.`,
      ];
    },);
  },);
}

//endregion Bilingual pair bound
