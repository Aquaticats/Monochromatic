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

 @returns Indices of the first letters

 @example
 ```ts
 const at = laterLinesOf({ source: 'x\n\n\u200B\nimport a\n', },);
 ```
 */
function laterLinesOf({ source, }: { readonly source: string; },): readonly number[] {
  const [, node,] = parseMarkdownMathBody({ body: source, },).children;
  if (node === undefined)
    throw new Error('the text holds no second block',);
  return laterModuleLines({
    source,
    node,
    isTopLevel: true,
    range: rangeOf({ node, },),
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
          name: 'FINDS NOTHING AFTER A LINE OF VISIBLE CHARACTERS, in a block that is not top level, or in another '
            + 'kind of block, and none that start with a word of another spelling',
          fn: async () => {
            expect(laterLinesOf({ source: 'x\n\n\uFEFFy\nimport a\n', },),).toEqual([],);
            expect(laterLinesOf({ source: 'x\n\n\uFEFF\nimported a\n', },),).toEqual([],);
            expect(laterLinesOf({ source: 'x\n\n\uFEFF\n  import a\n', },),).toEqual([],);
            expect(laterLinesOf({ source: 'x\n\n# \uFEFF\nimport a\n', },),).toEqual([],);
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
