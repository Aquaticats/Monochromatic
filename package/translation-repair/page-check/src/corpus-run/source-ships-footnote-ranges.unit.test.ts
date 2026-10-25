/**
 Tests the join of the footnote ranges of a reader of the source with those only the page starts, the writing of a link
 reference definition at such a start, and the reading of a text's tables and footnote definitions.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import { parseMarkdownMathBody, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  footnoteReadingsOf,
  NO_PAGE_ONLY_DEFINITION,
  pageOnlyDefinitionWritten,
  withPageOnlyStarts,
} from '../../dist/final/node/index.mjs';

await describe({
  name: withPageOnlyStarts.name,
  children: [
    it({
      name: 'ADDS A PAGE RANGE THAT STARTS WHERE NO RANGE OF THE SOURCE STANDS, in text order',
      fn: async () => {
        expect(withPageOnlyStarts({
          source: [
            { start: 0, end: 5, },
            { start: 20, end: 30, },
          ],
          page: [
            { start: 9, end: 15, },
          ],
        },),).toEqual([
          { start: 0, end: 5, },
          { start: 9, end: 15, },
          { start: 20, end: 30, },
        ],);
      },
    },),
    it({
      name: 'KEEPS THE SOURCE\'S RANGE WHERE THE PAGE STARTS ONE AT THE SAME PLACE OR INSIDE IT',
      fn: async () => {
        expect(withPageOnlyStarts({
          source: [
            { start: 0, end: 5, },
          ],
          page: [
            { start: 0, end: 12, },
            { start: 3, end: 4, },
          ],
        },),).toEqual([
          { start: 0, end: 5, },
        ],);
      },
    },),
    it({
      name: 'LEAVES THE SOURCE\'S RANGES AS THEY ARE WHERE THE PAGE HAS NONE, and the page\'s where the source has none',
      fn: async () => {
        expect(withPageOnlyStarts({ source: [{ start: 1, end: 2, },], page: [], },),).toEqual([{ start: 1, end: 2, },],);
        expect(withPageOnlyStarts({ source: [], page: [{ start: 9, end: 23, },], },),).toEqual([{ start: 9, end: 23, },],);
        expect(withPageOnlyStarts({ source: [], page: [], },),).toEqual([],);
      },
    },),
  ],
},);

await describe({
  name: pageOnlyDefinitionWritten.name,
  children: [
    it({
      name: 'WRITES THE CARET OF A FOOTNOTE LABEL WITH A BACKSLASH WHERE THE PAGE STARTS A FOOTNOTE AT A DEFINITION',
      fn: async () => {
        /**
         A definition and a line indented four columns after it, which the plain reading takes for a second definition.
         */
        const source = '[r]: u\n    [^1]: y\n';
        /**
         The second definition.
         */
        const node = nonNullishOrThrow(parseMarkdownMathBody({ body: source, },).children[1],);
        expect(node.type,).toBe('definition',);
        expect(pageOnlyDefinitionWritten({
          source,
          node,
          starts: [11,],
          endLine: 1,
        },),).toEqual({
          edit: { start: 12, end: 13, replacement: String.raw`\^`, },
          label: { start: 13, end: 14, },
          state: { open: false, endLine: 2, endsInLiteral: false, prefixLength: 0, opensRawHtml: false, },
        },);
      },
    },),
    it({
      name: 'WRITES NOTHING FOR A DEFINITION THE PAGE STARTS NO FOOTNOTE AT, or without a footnote label, or for another block',
      fn: async () => {
        /**
         A definition, a definition with a footnote label, and a paragraph.
         */
        const source = '[r]: u\n    [^1]: y\n\np\n';
        /**
         The blocks of the text.
         */
        const blocks = parseMarkdownMathBody({ body: source, },).children;
        /**
         Each block with the starts the page is given for it: the definition with no footnote label, the one with it and no
         start of the page at its bracket, and the paragraph.
         */
        const cases = [
          [nonNullishOrThrow(blocks[0],), [0,],],
          [nonNullishOrThrow(blocks[1],), [],],
          [nonNullishOrThrow(blocks[1],), [3,],],
          [nonNullishOrThrow(blocks[2],), [source.indexOf('p',),],],
        ] as const;
        for (const [node, starts,] of cases) {
          expect(pageOnlyDefinitionWritten({
            source,
            node,
            starts,
            endLine: 1,
          },),).toBe(NO_PAGE_ONLY_DEFINITION,);
        }
      },
    },),
  ],
},);

await describe({
  name: footnoteReadingsOf.name,
  children: [
    it({
      name: 'READS THE TABLES, THE FOOTNOTE RANGES OF THE PAGE AND OF BOTH READINGS, AND THE STARTS ONLY THE PAGE READS',
      fn: async () => {
        /**
         A paragraph with a line indented four columns, which only the page reads as a footnote.
         */
        const readings = footnoteReadingsOf({ readable: 'a\n    [^1]: y\n', },);
        expect(readings.pageOnlyStarts,).toEqual([6,],);
        expect(readings.tables,).toEqual([],);
        expect(readings.pageFootnoteRanges,).toEqual([{ start: 6, end: 13, },],);
        expect(readings.footnoteRanges,).toEqual([{ start: 6, end: 13, },],);
      },
    },),
    it({
      name: 'TAKES OUT THE START OF A LINE THAT FOLLOWS A ROW OF A TABLE',
      fn: async () => {
        /**
         A table with a line indented four columns right after its last row.
         */
        const readings = footnoteReadingsOf({ readable: '| a |\n| - |\n| b |\n    [^1]: y\n', },);
        expect(readings.tables.length,).toBe(1,);
        expect(readings.pageOnlyStarts,).toEqual([],);
      },
    },),
  ],
},);
