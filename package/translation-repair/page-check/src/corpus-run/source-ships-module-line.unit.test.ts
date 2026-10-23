/**
 Tests for the lines MDX reads as an ES module, which prose that starts with `import ` or `export ` is taken for.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { parseMarkdownMathBody, } from '@monochromatic-dev/translation-repair-document-structure';

import {
  laterModuleLines,
  moduleEdits,
  moduleLetterEdit,
  rangeOf,
  startsEsModule,
} from '../../dist/final/node/index.mjs';

/**
 Whether the first block of a text is read as an ES module.

 @param source - the text

 @param isTopLevel - whether the block stands directly under the root

 @returns Answer

 @example
 ```ts
 const esm = firstBlockIsModule({ source: 'export a\n', isTopLevel: true, },);
 ```
 */
function firstBlockIsModule(
  {
    source,
    isTopLevel,
  }: {
    readonly source: string;
    readonly isTopLevel: boolean;
  },
): boolean {
  const [node,] = parseMarkdownMathBody({ body: source, },).children;
  if (node === undefined)
    throw new Error('the text holds no block',);
  return startsEsModule({
    source,
    node,
    isTopLevel,
    range: rangeOf({ node, },),
  },);
}

/**
 Where the later module lines of the first block of a text start.

 @param source - the text

 @param isNested - whether the text is read apart from the document

 @param afterFootnote - whether the block follows or opens a footnote definition

 @returns Indices of the first letters

 @example
 ```ts
 const at = laterLinesOf({ source: 'x\n\n\u200B\nimport a\n', },);
 ```
 */
function laterLinesOf(
  {
    source,
    isNested = false,
    afterFootnote = false,
  }: {
    readonly source: string;
    readonly isNested?: boolean;
    readonly afterFootnote?: boolean;
  },
): readonly number[] {
  const [, node,] = parseMarkdownMathBody({ body: source, },).children;
  if (node === undefined)
    throw new Error('the text holds no second block',);
  return laterModuleLines({
    source,
    node,
    range: rangeOf({ node, },),
    isNested,
    afterFootnote,
  },);
}

await describe({
  name: 'the lines MDX reads as an ES module',
  children: [
    describe({
      name: laterModuleLines.name,
      children: [
        it({
          name: 'FINDS AN import OR export LINE THAT FOLLOWS A LINE OF SEPARATORS OR FORMAT CHARACTERS ONLY, a '
            + 'byte order mark, a no-break space and a zero-width space among them, in a paragraph and in a setext '
            + 'heading',
          fn: async () => {
            expect(laterLinesOf({ source: 'x\n\n\uFEFF\nimport a\n', },),).toEqual([5,],);
            expect(laterLinesOf({ source: 'x\n\n\u00A0\nexport to PDF\n---\n', },),).toEqual([5,],);
            expect(laterLinesOf({ source: 'x\n\n\u200B\r\nimport a\n', },),).toEqual([6,],);
          },
        },),

        it({
          name: 'FINDS AN import OR export LINE IN A RAW HTML BLOCK THAT FOLLOWS A BLANK LINE OR A LINE OF FORMAT '
            + 'CHARACTERS, which the plain reading keeps in the block and the strict grammar reads as a paragraph',
          fn: async () => {
            expect(laterLinesOf({ source: 'x\n\n<script>\n\nimport a\n', },),).toEqual([13,],);
            expect(laterLinesOf({ source: 'x\n\n<details>\n\uFEFF\nexport to PDF\n', },),).toEqual([15,],);
          },
        },),

        it({
          name: 'FINDS AN import OR export LINE THAT FOLLOWS A LINE OF A LIST MARKER ALONE IN A BLOCK THAT FOLLOWS A '
            + 'FOOTNOTE DEFINITION, a bullet, a dash, a plus and an ordered marker among them, since the strict '
            + 'grammar starts a list at the marker there and the line after it is a block of its own',
          fn: async () => {
            expect(laterLinesOf({
              source: '[^1]: 注\n*\nexport to PDF\n',
              afterFootnote: true,
            },),).toEqual([10,],);
            expect(laterLinesOf({
              source: '[^1]: 注\n-\nimport a\n',
              afterFootnote: true,
            },),).toEqual([10,],);
            expect(laterLinesOf({
              source: '[^1]: 注\n+ \nimport a\n',
              afterFootnote: true,
            },),).toEqual([11,],);
            expect(laterLinesOf({
              source: '[^1]: 注\n 12)\nimport a\n',
              afterFootnote: true,
            },),).toEqual([13,],);
            expect(laterLinesOf({
              source: '[^1]: 注\n*\n*\nexport to PDF\n',
              afterFootnote: true,
            },),).toEqual([12,],);
          },
        },),

        it({
          name: 'FINDS NOTHING AFTER A LIST MARKER WHEN THE BLOCK FOLLOWS NO FOOTNOTE DEFINITION, where the marker '
            + 'is a line of the paragraph, and nothing after a marker that has text or is no marker',
          fn: async () => {
            expect(laterLinesOf({ source: 'x\n\nabc\n*\nexport to PDF\n', },),).toEqual([],);
            expect(laterLinesOf({
              source: '[^1]: 注\n* a\nexport to PDF\n',
              afterFootnote: true,
            },),).toEqual([],);
            expect(laterLinesOf({
              source: '[^1]: 注\n**\nexport to PDF\n',
              afterFootnote: true,
            },),).toEqual([],);
            expect(laterLinesOf({
              source: '[^1]: 注\n*\nfoo\nexport to PDF\n',
              afterFootnote: true,
            },),).toEqual([],);
          },
        },),

        it({
          name: 'FINDS NOTHING IN A TEXT READ APART FROM THE DOCUMENT, a block none of whose lines stands at the first '
            + 'column of it',
          fn: async () => {
            expect(laterLinesOf({
              source: 'x\n\n\uFEFF\nimport q\n',
              isNested: true,
            },),).toEqual([],);
          },
        },),

        it({
          name: 'FINDS NOTHING AFTER A LINE OF VISIBLE CHARACTERS, in a block that is not top level, or in another '
            + 'kind of block, and none that start with a word of another spelling',
          fn: async () => {
            expect(laterLinesOf({ source: 'x\n\n\uFEFFy\nimport a\n', },),).toEqual([],);
            expect(laterLinesOf({ source: 'x\n\n\uFEFF\nimported a\n', },),).toEqual([],);
            expect(laterLinesOf({ source: 'x\n\n\uFEFF\n  import a\n', },),).toEqual([],);
            expect(laterLinesOf({ source: 'x\n\n# \uFEFF\nimport a\n', },),).toEqual([],);
            expect(laterLinesOf({ source: 'x\n\n> a\n> \uFEFF\n> import q\n', },),).toEqual([],);
            expect(laterLinesOf({ source: 'x\n\n<script>\n\n  import a\n', },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: moduleEdits.name,
      children: [
        it({
          name: 'WRITES THE FIRST WORD OF A MODULE PARAGRAPH AND THE FIRST WORD OF EACH LATER MODULE LINE, each as a '
            + 'character reference',
          fn: async () => {
            /**
             The text and its first block.
             */
            const source = 'import a\n\uFEFF\nimport b\n';
            const [node,] = parseMarkdownMathBody({ body: source, },).children;
            if (node === undefined)
              throw new Error('the text holds no block',);
            expect(moduleEdits({
              source,
              node,
              range: rangeOf({ node, },),
              isTopLevel: true,
              isNested: false,
            },),).toEqual([
              {
                start: 0,
                end: 1,
                replacement: '&#105;',
              },
              {
                start: 11,
                end: 12,
                replacement: '&#105;',
              },
            ],);
          },
        },),

        it({
          name: 'WRITES NOTHING FOR A BLOCK THAT STARTS NO MODULE, a word that only begins like the keyword',
          fn: async () => {
            /**
             The text and its first block.
             */
            const source = 'imported a\n';
            const [node,] = parseMarkdownMathBody({ body: source, },).children;
            if (node === undefined)
              throw new Error('the text holds no block',);
            expect(moduleEdits({
              source,
              node,
              range: rangeOf({ node, },),
              isTopLevel: true,
              isNested: false,
            },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: moduleLetterEdit.name,
      children: [
        it({
          name: 'WRITES THE LETTER AS A CHARACTER REFERENCE OF ITS CODE POINT, over that letter alone',
          fn: async () => {
            expect(moduleLetterEdit({
              source: 'x import',
              at: 2,
            },),).toEqual({
              start: 2,
              end: 3,
              replacement: '&#105;',
            },);
          },
        },),
      ],
    },),

    describe({
      name: startsEsModule.name,
      children: [
        it({
          name: 'READS A PARAGRAPH OR A SETEXT HEADING AT THE FIRST COLUMN OF THE ROOT STARTING import OR export AS '
            + 'A MODULE',
          fn: async () => {
            expect(firstBlockIsModule({
              source: 'import a from "b"\n',
              isTopLevel: true,
            },),).toBe(true,);
            expect(firstBlockIsModule({
              source: 'export to PDF\n---\n',
              isTopLevel: true,
            },),).toBe(true,);
          },
        },),

        it({
          name: 'READS NOTHING ELSE AS ONE: an ATX heading, a word without the space, an upper-case word, indented '
            + 'text, a block that is not top level, a block of another kind',
          fn: async () => {
            expect(firstBlockIsModule({
              source: '# import a\n',
              isTopLevel: true,
            },),).toBe(false,);
            expect(firstBlockIsModule({
              source: 'imported a\n',
              isTopLevel: true,
            },),).toBe(false,);
            expect(firstBlockIsModule({
              source: 'Import a\n',
              isTopLevel: true,
            },),).toBe(false,);
            expect(firstBlockIsModule({
              source: '  import a\n',
              isTopLevel: true,
            },),).toBe(false,);
            expect(firstBlockIsModule({
              source: 'import a\n',
              isTopLevel: false,
            },),).toBe(false,);
            expect(firstBlockIsModule({
              source: '- import a\n',
              isTopLevel: true,
            },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
