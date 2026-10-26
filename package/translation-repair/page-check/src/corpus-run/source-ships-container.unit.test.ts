/**
 Tests for how a paragraph of a block quote or a list item is read where a footnote definition starts on a later line:
 the lines before it as a text of their own, the definition with what follows it as another.

 A footnote line that carries the marks of the quote or the hanging indent of the item stays in the container, and each
 line after it gives up the marks; a lazy line, with fewer marks or less indent, is one at which the strict grammar
 closes the container, so the text from it on is read as it stands.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { pageFootnoteDefinitionRangesOf, } from '@monochromatic-dev/translation-repair-document-structure';

import {
  footnoteInContainerSpans,
  spansOf,
  type SourceSpans,
} from '../../dist/final/node/index.mjs';

/**
 Reads the paragraph of a text whose first block is a paragraph in a container, cut at its first footnote definition.

 @param source - text, of which the whole is the paragraph from `start` on

 @param start - where the paragraph starts

 @param end - where the paragraph ends

 @returns Spans of both parts

 @example
 ```ts
 const spans = readParagraph({ source: '> a\n> [^1]: b\n', start: 2, end: 13, },);
 ```
 */
function readParagraph(
  {
    source,
    start,
    end,
  }: {
    readonly source: string;
    readonly start: number;
    readonly end: number;
  },
): SourceSpans {
  return footnoteInContainerSpans({
    source,
    readable: source,
    readSpans: spansOf,
    footnotes: pageFootnoteDefinitionRangesOf({ body: source, },),
    range: {
      start,
      end,
    },
    split: source.indexOf('[^1]',),
    end,
  },);
}

/**
 Whether a stretch read as text holds a piece of the source.

 @param spans - what the reading found

 @param source - text

 @param piece - piece looked for

 @param from - index the search starts at

 @returns Whether some stretch of text covers the whole piece, the first one found from the index

 @example
 ```ts
 const holds = textHolds({ spans, source: 'a {b}', piece: '{b}', from: 0, },);
 ```
 */
function textHolds(
  {
    spans,
    source,
    piece,
    from = 0,
  }: {
    readonly spans: SourceSpans;
    readonly source: string;
    readonly piece: string;
    readonly from?: number;
  },
): boolean {
  /**
   Where the piece stands.
   */
  const at = source.indexOf(
    piece,
    from,
  );
  return spans.text.some(function covers({
    start,
    end,
  },): boolean {
    return (start <= at) && ((at + piece.length) <= end);
  },);
}

await describe({
  name: footnoteInContainerSpans.name,
  children: [
    it({
      name: 'READS THE LINES BEFORE A FOOTNOTE LINE THAT CARRIES THE MARKS OF A BLOCK QUOTE AS A TEXT OF THEIR OWN AND THE '
        + 'FOOTNOTE AS ANOTHER, so a code span the plain reading runs across the line holds nothing past it and the '
        + 'braces on both sides of it are text',
      fn: async () => {
        /**
         A paragraph in a block quote whose code span would run across the definition's line.
         */
        const source = '> a `b\n> c {e}\n> [^1]: f {g}` h\n';
        const spans = readParagraph({
          source,
          start: 2,
          end: source.length - 1,
        },);
        expect(textHolds({ spans, source, piece: '{e}', },),).toBe(true,);
        expect(textHolds({ spans, source, piece: '{g}', },),).toBe(true,);
        expect(spans.literal,).toEqual([],);
        expect(spans.text.some(function holdsMark({ start, end, },): boolean {
          return source.slice(start, end,).includes('>',);
        },),).toBe(false,);
      },
    },),

    it({
      name: 'TAKES THE MARKS OFF EVERY LINE OF A FOOTNOTE THAT STAYS IN THE QUOTE, so a tag alone and a definition line '
        + 'after it are read as they are at the top: the line ending that ends the footnote is doubled',
      fn: async () => {
        /**
         A footnote in a block quote, a tag alone, then a line CommonMark takes for a definition.
         */
        const source = '> a\n> [^1]: x\n> <a href="x">\n> [r]: u{b}\n';
        const { edits, } = readParagraph({
          source,
          start: 2,
          end: source.length - 1,
        },);
        /**
         Where the footnote ends: after the tag.
         */
        const stop = source.indexOf('\n> [r]',);
        expect(edits,).toContainEqual({ start: stop, end: stop + 1, replacement: '\n\n', },);
      },
    },),

    it({
      name: 'READS THE TEXT FROM A LAZY FOOTNOTE LINE ON AS IT STANDS, since the strict grammar closes the quote there '
        + 'and a line with a mark after it starts a quote of its own that ends the footnote\'s paragraph: the '
        + 'brace after a code span that the plain reading would join with the next line is text',
      fn: async () => {
        /**
         A lazy footnote line indented three spaces, then a line with a mark.
         */
        const source = '> a `x\n   [^1]: {c}` {d}\n> a `x\n';
        const spans = readParagraph({
          source,
          start: 2,
          end: source.length - 1,
        },);
        expect(textHolds({ spans, source, piece: '{c}', },),).toBe(true,);
        expect(textHolds({ spans, source, piece: '{d}', },),).toBe(true,);
      },
    },),

    it({
      name: 'CUTS AT EACH FOOTNOTE LINE OF THE PARAGRAPH AND READS EACH PART BY WHETHER ITS LINE STAYS IN THE QUOTE: a '
        + 'line with a mark after a lazy footnote line opens a quote of its own, so the brace after the next code '
        + 'span is text where the plain reading joins the lines',
      fn: async () => {
        /**
         A footnote line with the mark, a lazy one, and one with the mark again, a code span opened before each.
         */
        const source = '> a `x\n>  [^1]: {c}` {d}\n> a `x\n  [^1]: {c}` {d}\n> a `x\n>  [^1]: <b>` {d}\n';
        const spans = readParagraph({
          source,
          start: 2,
          end: source.length - 1,
        },);
        /**
         Where the {d} after the lazy footnote line stands, the second of three.
         */
        const lazy = source.indexOf(
          '{d}',
          source.indexOf('{d}',) + 1,
        );
        expect(textHolds({ spans, source, piece: '{d}', from: lazy, },),).toBe(true,);
        expect(textHolds({ spans, source, piece: '{d}', from: source.lastIndexOf('{d}',), },),).toBe(true,);
      },
    },),

    it({
      name: 'KEEPS A FOOTNOTE LINE AT THE CONTENT COLUMN OF A LIST ITEM IN THE ITEM AND READS ONE SHORT OF IT AS A LAZY '
        + 'LINE, the braces after the colon being text for both',
      fn: async () => {
        for (const source of ['- a `x\n  [^1]: {c}` {d}\n', '- a `x\n [^1]: {c}` {d}\n',]) {
          const spans = readParagraph({
            source,
            start: 2,
            end: source.length - 1,
          },);
          expect(textHolds({ spans, source, piece: '{c}', },),).toBe(true,);
          expect(textHolds({ spans, source, piece: '{d}', },),).toBe(true,);
        }
      },
    },),
  ],
},);
