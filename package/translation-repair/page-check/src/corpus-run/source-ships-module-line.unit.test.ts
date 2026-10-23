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
  rangeOf,
  startsEsModule,
} from '../../dist/final/node/index.mjs';

/**
 Whether the first block of a text is read as an ES module.

 @param source - the text

 @param isTopLevel - whether the block stands directly under the root

 @returns The answer

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

await describe({
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
      name: 'READS NOTHING ELSE AS ONE: an ATX heading, a word without the space, an upper-case word, indented text, a '
        + 'block that is not top level, a block of another kind',
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
},);
