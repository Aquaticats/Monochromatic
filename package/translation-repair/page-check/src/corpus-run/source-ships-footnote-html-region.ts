import type { BlockRange, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { footnoteContainerPrefixOf, } from './source-ships-footnote-container.ts';
import { footnoteOpensRawHtml, } from './source-ships-footnote-html.ts';
import { afterLiteralBlockEdits, } from './source-ships-separation.ts';
import type {
  SourceSpans,
  TextRange,
} from './source-ships-ranges.ts';

//region Source ships footnote html region
// THE LINES OF THE BLOCK OF RAW HTML A FOOTNOTE DEFINITION OPENS WITH. A tag alone on the definition's line starts a block
// of raw HTML to a reader of the source, and that block holds the lines after it up to the next blank line, whatever they
// start with: a fence mark or the mark of a display math on one is text there (`source-ships-footnote-html.ts`). The plain
// reading splits those lines into the blocks of a paragraph, and reads two fence marks in it as the delimiters of a code
// span and two marks of a display math as the delimiters of inline math, which the encoder must not copy or take a comment
// opener out of: on the page, which has no indented code, the lines are text, and every character Markdown reads there is
// written with a backslash. This module says where those lines stand and which stretches of a block are their content.

/**
 The line ending that ends a line.
 */
const LINE_FEED = '\n';

/**
 Where the lines of a block of raw HTML that a footnote definition opens with stand, and what the container of the
 definition puts before each line of them.
 */
export type RawHtmlRegion = TextRange & {
  /**
   What the container puts before the label of the footnote on its line, such as `> ` or `- `, which every line carries
   the marks of.
   */
  readonly prefix: string;
};

/**
 Names of the elements whose block of raw HTML ends at the closing tag.
 */
const RAW_TEXT_ELEMENTS: readonly string[] = [
  'script',
  'pre',
  'style',
  'textarea',
];

/**
 Letters, of which a declaration starts with one after its exclamation mark.
 */
const LETTERS = 'abcdefghijklmnopqrstuvwxyz';

/**
 What ends a block of raw HTML of the first five kinds, by what the block starts with: a comment, a processing instruction, a
 declaration, a CDATA section and a raw text element. These do not end at a blank line, and the other two kinds do.

 @param text - text of the block's first line

 @returns Characters that end it, empty string for a block a blank line ends

 @example
 ```ts
 const mark = endMarkOf({ text: '<!-- c', },); // '-->'
 ```
 */
function endMarkOf({ text, }: { readonly text: string; },): string {
  /**
   The text in lower case, since the names of elements are matched without case.
   */
  const lower = text.toLowerCase();
  if (lower.startsWith('<!--',))
    return '-->';
  if (lower.startsWith('<?',))
    return '?>';
  if (lower.startsWith('<![cdata[',))
    return ']]>';
  /**
   The character after `<!`, empty when the text ends there.
   */
  const third = lower.charAt(2,);
  /**
   Whether the text starts a declaration: `<!` and a letter.
   */
  const isDeclaration = lower.startsWith('<!',) && (third !== '');
  if (isDeclaration && LETTERS.includes(third,))
    return '>';
  /**
   The element the block starts with, when it is one of those.
   */
  const element = RAW_TEXT_ELEMENTS.find(function startsHere(name,): boolean {
    return lower.startsWith(`<${name}`,) && (' \t>'.includes(lower.charAt(name.length + 1,),) || (lower.length === (name.length + 1)));
  },);
  return (element === undefined) ? '' : `</${element}>`;
}

/**
 Where the lines of a block of raw HTML that a footnote definition opens with end: at the first blank line, which ends a
 block of the last two kinds, or at the line that holds what ends a block of the first five, or at the end of the footnote.

 @param source - text

 @param range - where the footnote stands, from its label

 @param prefix - what the container puts before the label on its line

 @param from - where the lines start

 @returns Index where the lines end

 @example
 ```ts
 const end = regionEndOf({ source, range, prefix: '> ', from: 12, },);
 ```
 */
function regionEndOf(
  {
    source,
    range,
    prefix,
    from,
  }: {
    readonly source: string;
    readonly range: TextRange;
    readonly prefix: string;
    readonly from: number;
  },
): number {
  /**
   Where the text after the label starts, on the definition's own line.
   */
  const bodyAt = source.indexOf(
    ']:',
    range.start,
  ) + 2;
  /**
   The first line of the block, less the white space before it.
   */
  const lineText = source.slice(
    bodyAt,
    from,
  );
  /**
   The same less its white space.
   */
  const first = lineText.trim();
  /**
   What ends the block, none where a blank line does.
   */
  const mark = endMarkOf({ text: first, },);
  for (let lineStart = from; lineStart < range.end;) {
    /**
     Where the line ends.
     */
    const feed = source.indexOf(
      LINE_FEED,
      lineStart,
    );
    /**
     Where the line's content ends, before its line ending.
     */
    const lineEnd = ((feed === (-1)) || (feed > range.end)) ? range.end : feed;
    /**
     What its container takes of the line.
     */
    const container = footnoteContainerPrefixOf({
      source,
      at: lineStart,
      pattern: prefix,
    },);
    /**
     What the line holds after the marks of its container.
     */
    const rest = source.slice(
      lineStart + container.length,
      lineEnd,
    );
    // A block of the first five kinds ends at the line that holds what ends it, and a block of the other two at a blank line.
    // A line without the marks of a block quote is a lazy line of a paragraph and ends nothing.
    if (mark !== '') {
      if (rest.includes(mark,))
        return lineEnd;
    }
    else if (container.marked && (rest.trim() === '')) {
      return lineStart;
    }
    lineStart = lineEnd + 1;
  }
  return range.end;
}

/**
 Where the lines of the block of raw HTML that a footnote definition opens with stand, for each footnote that opens with
 one: from the line after the definition's own to where the block ends, or to the end of the footnote.

 @param source - text the footnotes were read from

 @param footnotes - where a footnote definition stands, for a reader of the source and for the page

 @returns One range for each footnote that opens with a block of raw HTML that goes on over a line, where its lines stand

 @example
 ```ts
 const regions = rawHtmlRegionsOf({ source: '> [^1]: <b>\n>     ```\n', footnotes: [{ start: 2, end: 21, },], },);
 ```
 */
export function rawHtmlRegionsOf(
  {
    source,
    footnotes,
  }: {
    readonly source: string;
    readonly footnotes: readonly BlockRange[];
  },
): readonly RawHtmlRegion[] {
  /**
   Regions found.
   */
  const regions: RawHtmlRegion[] = [];
  /**
   Where each footnote read so far starts, since a footnote may be listed for both readers.
   */
  const seen = new Set<number>();
  for (const range of footnotes) {
    if (seen.has(range.start,))
      continue;
    seen.add(range.start,);
    if (!footnoteOpensRawHtml({
      source,
      range,
    },))
      continue;
    /**
     Where the definition's own line ends.
     */
    const firstFeed = source.indexOf(
      LINE_FEED,
      range.start,
    );
    if ((firstFeed === (-1)) || (firstFeed >= range.end))
      continue;
    /**
     What its container puts before the label on that line.
     */
    const prefix = source.slice(
      source.lastIndexOf(
        LINE_FEED,
        range.start - 1,
      ) + 1,
      range.start,
    );
    regions.push({
      start: firstFeed + 1,
      end: regionEndOf({
        source,
        range,
        prefix,
        from: firstFeed + 1,
      },),
      prefix,
    },);
  }
  return regions;
}

/**
 What a block in the lines of a block of raw HTML comes to for the page: the content of each line of the block as the
 stretches shown as they stand (`literal`), the lines of the block past the region as text, and the blank line put after the
 block where it ends the region, so that a definition line right after it is not a lazy line of the text on the page (the
 edits).
 */
export type RawHtmlBlockReading = SourceSpans & {
  /**
   Whether the block stands in the lines of such a block of raw HTML.
   */
  readonly inRegion: boolean;
};

/**
 The reading of a block that stands in none of those lines.
 */
const NOT_RAW_HTML: RawHtmlBlockReading = {
  inRegion: false,
  text: [],
  literal: [],
  edits: [],
  insideLinks: [],
  streamStarts: [],
};

/**
 The reading of a block that stands in the block of raw HTML a footnote definition opens with: the content of each of its
 lines less what its container puts before it, and a blank line after it where it ends the block.

 @param source - text

 @param readable - whole text with its comments blanked, the parse's own input

 @param regions - where those lines stand

 @param range - where the block stands, from the first character after the marks of its container on its first line

 @param inContainer - whether the block stands in a block quote or a list item, the only places the lines of a block of raw
 HTML are read this way; the top reads a footnote's body as a text of its own

 @param footnoteStarts - where a footnote definition starts, for the reader of the source and for the page; a block that
 goes on past its region and holds one is left to the footnote's own reading

 @returns One stretch for each line of the block that has content, none when the block does not start in a region

 @example
 ```ts
 const html = rawHtmlBlockReading({ source, readable, regions, range: { start: 18, end: 31, }, inContainer: true, },);
 ```
 */
export function rawHtmlBlockReading(
  {
    source,
    readable,
    regions,
    range,
    inContainer,
    footnoteStarts = [],
  }: {
    readonly source: string;
    readonly readable: string;
    readonly regions: readonly RawHtmlRegion[];
    readonly range: TextRange;
    readonly inContainer: boolean;
    readonly footnoteStarts?: readonly number[];
  },
): RawHtmlBlockReading {
  /**
   The region the block starts in.
   */
  const region = regions.find(function holds({
    start,
    end,
  },): boolean {
    return (start <= range.start) && (range.start < end);
  },);
  if ((!inContainer) || (region === undefined))
    return NOT_RAW_HTML;
  // A block that goes on past the region and holds the start of another footnote definition is read with that footnote.
  if (footnoteStarts.some(function startsPast(start,): boolean {
    return (start >= region.end) && (start < range.end);
  },))
    return NOT_RAW_HTML;
  /**
   Where the lines of the block that stand in the region end.
   */
  const limit = Math.min(
    range.end,
    region.end,
  );
  /**
   Stretches found.
   */
  const lines: TextRange[] = [];
  /**
   Where the last line in the region ends.
   */
  const last = { end: range.start, };
  for (let lineStart = range.start; lineStart < limit;) {
    /**
     Where the line ends.
     */
    const feed = source.indexOf(
      LINE_FEED,
      lineStart,
    );
    /**
     Where the line's content ends, before its carriage return.
     */
    const lineEnd = ((feed === (-1)) || (feed > limit)) ? limit : feed;
    last.end = lineEnd;
    /**
     What its container takes of the line, which the first line has none of.
     */
    const container = footnoteContainerPrefixOf({
      source,
      at: lineStart,
      pattern: region.prefix,
    },);
    /**
     Where the content starts: at once on the first line, after the marks of the container on the others.
     */
    const contentStart = (lineStart === range.start) ? lineStart : (lineStart + container.length);
    /**
     Where the content ends, a carriage return not part of it.
     */
    const contentEnd = ((lineEnd > contentStart) && (source.charAt(lineEnd - 1,) === '\r')) ? (lineEnd - 1) : lineEnd;
    if (contentStart < contentEnd) {
      lines.push({
        start: contentStart,
        end: contentEnd,
      },);
    }
    lineStart = lineEnd + 1;
  }
  // A block that goes on past the region is a lazy line of a paragraph to the plain reading, and a block of its own to a
  // reader of the source: the lines in the region are text, a blank line follows them, and the rest is read as it was.
  if (range.end > region.end) {
    return {
      ...NOT_RAW_HTML,
      inRegion: true,
      literal: lines,
      text: [
        {
          start: last.end + 1,
          end: range.end,
        },
      ],
      edits: afterLiteralBlockEdits({
        readable,
        range: {
          start: range.start,
          end: last.end,
        },
        inContainer: true,
      },),
    };
  }
  /**
   Where the line after the block starts.
   */
  const nextLine = range.end + 1;
  return {
    ...NOT_RAW_HTML,
    inRegion: true,
    literal: lines,
    edits: (nextLine < region.end)
      ? []
      : afterLiteralBlockEdits({
        readable,
        range,
        inContainer: true,
      },),
  };
}

//endregion Source ships footnote html region
