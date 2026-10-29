import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  isCommentFinding,
  type RepairDocument,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  BREAK,
  QUOTE_CLOSERS,
  QUOTE_OPENERS,
  STRAIGHT_QUOTE,
  TAG_LETTERS,
  URL_SCHEMES,
} from './repeat-groups-bounds.ts';
import type {
  HiddenRange,
  WordingStream,
} from './repeat-groups-types.ts';

//region Repeat groups wording
// The wording stream of a page: the characters of its body paragraphs, with
// breaks where inline markup cuts a run and where a paragraph ends. Blocks are
// read off the package's parse (`parseDocument`), so front matter, headings,
// blockquotes, footnote definitions, lists and code never reach this reading.
// What a reader never sees is read as whitespace: the HTML comments the parse
// blanked before reading, and the tags of the containers it dissolved, which
// the parse widens an edge paragraph over. Neither cuts a run, and neither
// lends its own text to the inline cuts or to the test for markup.

/**
 Index of the first close character after a start, when no line break comes first.

 @param body - raw paragraph text

 @param from - index to search from

 @param close - character sought

 @returns Index of the close character, or minus one when a line break or the end comes first

 @example
 ```ts
 const close = closingIndex({ body: '`code`', from: 1, close: '`', },);
 ```
 */
function closingIndex(
  {
    body,
    from,
    close,
  }: {
    readonly body: string;
    readonly from: number;
    readonly close: string;
  },
): number {
  for (let index = from; index < body.length; index++) {
    /**
     Character at this index.
     */
    const char = body.charAt(index,);
    if (char === close)
      return index;
    if (char === '\n')
      return -1;
  }
  return -1;
}

/**
 Index one past the inline cut that begins at an index, or minus one when no cut begins there.

 Bare URLs, inline code, footnote references and link destinations are cut;
 each is matched the way the single-line patterns read them, so a cut never
 crosses a line break.

 @param body - raw paragraph text

 @param index - index the cut must begin at

 @returns Exclusive end of the cut, or minus one

 @example
 ```ts
 const end = cutEndAt({ body: 'see https://cat.example.invalid/x now', index: 4, },);
 ```
 */
export function cutEndAt(
  {
    body,
    index,
  }: {
    readonly body: string;
    readonly index: number;
  },
): number {
  for (const scheme of URL_SCHEMES) {
    if (!body.startsWith(
      scheme,
      index,
    ))
      continue;
    /**
     End of the URL: its scheme, then every character up to a space.
     */
    let end = index + scheme.length;
    while ((end < body.length) && (body.charAt(end,)
      .trim()
      !== ''))
      end++;
    return (end > (index
      + scheme.length)) ? end : -1;
  }

  /**
   Character the cut could begin with.
   */
  const opener = body.charAt(index,);
  if (opener === '`') {
    /**
     Closing backtick of the inline code.
     */
    const close = closingIndex({
      body,
      from: index + 1,
      close: '`',
    },);
    return (close < 0) ? -1 : close + 1;
  }
  if ((opener === '[') && (body.charAt(index + 1,) === '^')) {
    /**
     Closing bracket of the footnote reference.
     */
    const close = closingIndex({
      body,
      from: index + 2,
      close: ']',
    },);
    return (close < 0) ? -1 : close + 1;
  }
  if ((opener === ']') && (body.charAt(index + 1,) === '(')) {
    /**
     Closing parenthesis of the link destination.
     */
    const close = closingIndex({
      body,
      from: index + 2,
      close: ')',
    },);
    return (close < 0) ? -1 : close + 1;
  }
  return -1;
}

/**
 Indexes of a paragraph that inline cuts cover, scanned left to right so that
 a cut consumes its own text, as a global pattern does.

 @param body - raw paragraph text

 @returns Set of covered indexes

 @example
 ```ts
 const covered = cutSetOf({ body: '猫咪`代码`', },);
 ```
 */
export function cutSetOf({ body, }: { readonly body: string; },): ReadonlySet<number> {
  /**
   Indexes covered by a cut.
   */
  const covered = new Set<number>();

  /**
   Index being scanned.
   */
  let index = 0;
  while (index < body.length) {
    /**
     End of the cut beginning here, or minus one.
     */
    const cutEnd = cutEndAt({
      body,
      index,
    },);
    if (cutEnd < 0) {
      index++;
      continue;
    }
    for (let inside = index; inside < cutEnd; inside++)
      covered.add(inside,);
    index = cutEnd;
  }
  return covered;
}

/**
 Whether a paragraph carries an inline tag or an expression, which makes the
 whole paragraph markup, since its inner text is not wording apart from the tag.

 @param body - raw paragraph text

 @returns Whether a tag opens in the text or an expression brace appears

 @example
 ```ts
 const markup = hasMarkupOpener({ body: '猫咪<b>晒</b>', },);
 ```
 */
export function hasMarkupOpener({ body, }: { readonly body: string; },): boolean {
  if (body.includes('{',))
    return true;
  for (let index = 0; (index + 1) < body.length; index++) {
    if (body.charAt(index,) !== '<')
      continue;
    /**
     Character after the less-than sign.
     */
    const next = body.charAt(index + 1,);
    if ((next === '/') || TAG_LETTERS.includes(next,))
      return true;
  }
  return false;
}

/**
 Ranges of a page that no reader sees: each HTML comment the parse blanked
 before reading, closed or left open, and the two tags of each container it
 dissolved. The parse widens an edge paragraph over the tags of its container,
 so a paragraph's raw text can hold them.

 @param document - parsed page

 @returns Absolute offset ranges, in no particular order

 @example
 ```ts
 const ranges = hiddenRangesOf({ document, },);
 ```
 */
export function hiddenRangesOf({ document, }: { readonly document: RepairDocument; },): readonly HiddenRange[] {
  return [
    ...document.parseFindings
      .filter(function isComment(finding,): boolean {
        return isCommentFinding({ finding, },);
      },)
      .map(function commentRange(finding,): HiddenRange {
        return {
          startOffset: finding.startOffset,
          endOffset: finding.endOffset,
        };
      },),
    ...document.containers
      .flatMap(function tagRanges(container,): readonly HiddenRange[] {
        return [
          {
            startOffset: container.openerStartOffset,
            endOffset: container.openerEndOffset,
          },
          {
            startOffset: container.closerStartOffset,
            endOffset: container.closerEndOffset,
          },
        ];
      },),
  ];
}

/**
 Indexes of a paragraph that fall inside a hidden range.

 @param ranges - hidden ranges of the page, with absolute offsets

 @param startOffset - absolute offset where the paragraph begins

 @param endOffset - absolute exclusive offset where the paragraph ends

 @returns Set of covered indexes, relative to the paragraph's start

 @example
 ```ts
 const covered = hiddenCoverOf({ ranges, startOffset: 0, endOffset: 12, },);
 ```
 */
export function hiddenCoverOf(
  {
    ranges,
    startOffset,
    endOffset,
  }: {
    readonly ranges: readonly HiddenRange[];
    readonly startOffset: number;
    readonly endOffset: number;
  },
): ReadonlySet<number> {
  /**
   Indexes covered by a hidden range.
   */
  const covered = new Set<number>();
  for (const range of ranges) {
    for (
      let offset = Math.max(
        range.startOffset,
        startOffset,
      );
      offset < Math.min(
        range.endOffset,
        endOffset,
      );
      offset++
    )
      covered.add(offset - startOffset,);
  }
  return covered;
}

/**
 A paragraph with the units at some indexes replaced by spaces, so that what
 no reader sees lends the inline cuts and the test for markup no text of its own.

 @param body - raw paragraph text

 @param covered - indexes, in UTF-16 units, to blank

 @returns Text of the same length with each covered unit a space

 @example
 ```ts
 const seen = blankedAt({ body: '猫<!-- 注 -->咪', covered: new Set([1, 2, 3, 4, 5, 6, 7, 8, 9, 10,]), },);
 ```
 */
export function blankedAt(
  {
    body,
    covered,
  }: {
    readonly body: string;
    readonly covered: ReadonlySet<number>;
  },
): string {
  /**
   Units of the paragraph, each blanked where covered.
   */
  const units: string[] = [];
  for (let index = 0; index < body.length; index++) {
    units.push(covered.has(index,) ? ' ' : body.charAt(index,),);
  }
  return units.join('',);
}

/**
 Absolute exclusive end offset of one character of the wording stream.

 @param wording - wording stream of the page

 @param index - stream index of a character, never a break

 @returns Offset one past the character's last UTF-16 unit

 @example
 ```ts
 const end = wordingEndAt({ wording, index: 11, },);
 ```
 */
export function wordingEndAt(
  {
    wording,
    index,
  }: {
    readonly wording: WordingStream;
    readonly index: number;
  },
): number {
  return nonNullishOrThrow(wording.offsets[index],)
    + nonNullishOrThrow(wording.chars[index],)
      .length;
}

/**
 Reads the body paragraphs of a parsed document as one wording stream.

 A paragraph carrying markup outside its inline cuts is left out whole. An inline cut leaves a break,
 and so does the end of each paragraph. Whitespace, an HTML comment the parse
 blanked and the tags of a dissolved container are dropped without a break,
 since the rule compares the characters a reader sees and a comment inside a
 sentence does not end it. Neither a comment nor a container tag lends its text
 to the inline cuts or to the test for markup, and neither does a cut: a brace
 or a tag inside inline code or a bare URL is no expression and no tag.

 A quotation opens at an opening mark (or at a straight double quotation mark
 with none open in the paragraph) and closes at a closing mark (or the next
 straight one). The opening mark counts as inside it and the closing mark as
 outside. A mark that never closes quotes the rest of its paragraph, and no
 later one, since the next paragraph of a quotation opens it again. A stray
 closing mark leaves the depth at zero, so later quoted speech in the paragraph
 still reads as quoted.

 @param text - full page text, the offsets of the document's nodes index into it

 @param document - parsed page

 @returns Wording characters with their offsets and quotation depths

 @example
 ```ts
 const stream = wordingStreamOf({ text, document, },);
 ```
 */
export function wordingStreamOf(
  {
    text,
    document,
  }: {
    readonly text: string;
    readonly document: RepairDocument;
  },
): WordingStream {
  /**
   Wording characters, in page order.
   */
  const chars: string[] = [];

  /**
   Absolute offset of each wording character, minus one at a break.
   */
  const offsets: number[] = [];

  /**
   Quotation depth at each wording character, zero at a break.
   */
  const depths: number[] = [];

  /**
   Ranges of the page no reader sees, read once for every paragraph.
   */
  const hiddenRanges = hiddenRangesOf({ document, },);
  for (const node of document.nodes) {
    if ((node.zone !== 'body') || (node.kind !== 'paragraph'))
      continue;

    /**
     Raw source text of this paragraph.
     */
    const body = text.slice(
      node.startOffset,
      node.endOffset,
    );

    /**
     Indexes of this paragraph that no reader sees.
     */
    const hidden = hiddenCoverOf({
      ranges: hiddenRanges,
      startOffset: node.startOffset,
      endOffset: node.endOffset,
    },);

    /**
     The paragraph as the inline cuts read it.
     */
    const seen = blankedAt({
      body,
      covered: hidden,
    },);

    /**
     Indexes of this paragraph that inline cuts cover.
     */
    const cuts = cutSetOf({ body: seen, },);

    /**
     The paragraph as the markup test reads it: what a cut covers is no markup of the paragraph.
     */
    const outsideCuts = blankedAt({
      body: seen,
      covered: cuts,
    },);
    if (hasMarkupOpener({ body: outsideCuts, },))
      continue;

    /**
     Open quotation depth so far in this paragraph.
     */
    let depth = 0;

    /**
     Whether a straight double quotation mark is open in this paragraph.
     */
    let straightOpen = false;

    /**
     Whether the last character pushed was a break, so that no two breaks stand together.
     */
    let broken = true;
    /**
     UTF-16 index of the character being read, which a character outside the basic plane advances by two.
     */
    let index = 0;
    for (const char of body) {
      /**
       Index this character stands at.
       */
      const here = index;
      index += char.length;
      if (hidden.has(here,))
        continue;
      if (cuts.has(here,)) {
        if (!broken) {
          chars.push(BREAK,);
          offsets.push(-1,);
          depths.push(0,);
          broken = true;
        }
        continue;
      }
      if (char.trim() === '')
        continue;
      if (QUOTE_OPENERS.includes(char,))
        depth++;
      if (QUOTE_CLOSERS.includes(char,))
        depth = Math.max(
          0,
          depth - 1,
        );
      if (char === STRAIGHT_QUOTE) {
        depth = straightOpen
          ? Math.max(
            0,
            depth - 1,
          )
          : depth + 1;
        straightOpen = !straightOpen;
      }
      chars.push(char,);
      offsets.push(node.startOffset + here,);
      depths.push(depth,);
      broken = false;
    }
    if (!broken) {
      chars.push(BREAK,);
      offsets.push(-1,);
      depths.push(0,);
    }
  }

  return {
    chars,
    offsets,
    depths,
  };
}

//endregion Repeat groups wording
