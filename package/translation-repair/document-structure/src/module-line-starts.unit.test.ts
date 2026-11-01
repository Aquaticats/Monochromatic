/**
 Tests the reading of where the strict grammar starts an ES module in a text that stands in the middle of a page: a line
 that opens a fresh block at the first column and starts with `import ` or `export `. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { moduleLineStartsOf, } from '../dist/final/node/index.mjs';

/**
 Index of the first letter of each occurrence of a word in a text, in text order.

 @param text - text to look in

 @param word - word to look for

 @returns Indices found

 @example
 ```ts
 const at = indicesOfWord({ text: 'a\n\nimport b\n', word: 'import', },);
 ```
 */
function indicesOfWord(
  {
    text,
    word,
  }: {
    readonly text: string;
    readonly word: string;
  },
): readonly number[] {
  /**
   Indices found so far.
   */
  const found: number[] = [];
  for (
    let at = text.indexOf(word,);
    at !== (-1);
    at = text.indexOf(
      word,
      at + 1,
    )
  )
    found.push(at,);
  return found;
}

await describe({
  name: moduleLineStartsOf.name,
  children: [
    it({
      name: 'READS EACH LINE THAT OPENS A FRESH BLOCK AT THE FIRST COLUMN AND STARTS WITH import OR export, after '
        + 'the start of the text and after a blank line, and no lazy line of a paragraph',
      fn: async () => {
        expect(moduleLineStartsOf({ body: 'import a from "b"\n', },),).toEqual([0,],);
        expect(moduleLineStartsOf({ body: '她说\n\nimport a\n\nexport to PDF\n', },),).toEqual([4, 14,],);
        expect(moduleLineStartsOf({ body: '她说\nimport a\n', },),).toEqual([],);
        expect(moduleLineStartsOf({ body: 'imported a\nexport\nImport a\n', },),).toEqual([],);
      },
    },),

    it({
      name: 'READS THE LINE AFTER A SETEXT UNDERLINE AS A FRESH BLOCK too, which the first line of a heading is not',
      fn: async () => {
        /**
         A heading that starts with the word, and a second module line after its underline.
         */
        const body = 'import q\n===\nexport to PDF\n';
        expect(moduleLineStartsOf({ body, },),).toEqual([0, body.indexOf('export',),],);
      },
    },),

    it({
      name: 'READS A LINE OF INVISIBLE CHARACTERS AS A BLANK LINE, as the package reads a page, so the line after it '
        + 'is a fresh block in a paragraph, a list item and a block quote',
      fn: async () => {
        for (const opening of ['x', '- x', '1. 项', '> x', '- x\n  y',]) {
          for (const invisible of ['\uFEFF', '\u00A0', '\u200B', '\u3000',]) {
            /**
             Text with a line of one invisible character between a container's lines and a module line.
             */
            const body = `${opening}\n${invisible}\nimport q\n===\nexport to PDF\n`;
            expect(moduleLineStartsOf({ body, },),).toEqual([
              ...indicesOfWord({
                text: body,
                word: 'import',
              },),
              ...indicesOfWord({
                text: body,
                word: 'export',
              },),
            ],);
          }
        }
      },
    },),

    it({
      name: 'READS A LINE OF INVISIBLE CHARACTERS THE PACKAGE DOES NOT MASK AS A LINE OF THE PARAGRAPH: a soft hyphen '
        + 'and a left-to-right mark show nothing, and the line after one is a lazy line',
      fn: async () => {
        expect(moduleLineStartsOf({ body: 'x\n\u00AD\nimport q\n', },),).toEqual([],);
        expect(moduleLineStartsOf({ body: 'x\n\u200E\nimport q\n', },),).toEqual([],);
      },
    },),

    it({
      name: 'READS A LIST MARKER ALONE RIGHT AFTER A FOOTNOTE DEFINITION AS THE START OF A LIST, so the line after it '
        + 'is a fresh block, and as a line of the paragraph where a paragraph is open',
      fn: async () => {
        expect(moduleLineStartsOf({ body: '[^1]: 注\n*\nexport to PDF\n', },),).toEqual([10,],);
        expect(moduleLineStartsOf({ body: '[^1]: 注\n2.\nimport a\n', },),).toEqual([11,],);
        expect(moduleLineStartsOf({ body: 'abc\n*\nexport to PDF\n', },),).toEqual([],);
        expect(moduleLineStartsOf({ body: '[^1]: 注\n*\nfoo\nexport to PDF\n', },),).toEqual([],);
      },
    },),

    it({
      name: 'READS NO LINE INSIDE A CONTAINER, A FENCE, A CLOSED COMMENT OR AN INDENTED LINE',
      fn: async () => {
        expect(moduleLineStartsOf({ body: '> import a\n', },),).toEqual([],);
        expect(moduleLineStartsOf({ body: '- import a\n', },),).toEqual([],);
        expect(moduleLineStartsOf({ body: ' import a\n', },),).toEqual([],);
        expect(moduleLineStartsOf({ body: '```\nimport a\n```\n', },),).toEqual([],);
        expect(moduleLineStartsOf({ body: '<!--\n\nimport a\n\n-->\n', },),).toEqual([],);
        expect(moduleLineStartsOf({ body: '[^1]: x\n    import a\n', },),).toEqual([],);
      },
    },),

    it({
      name: 'READS A LINE AFTER A BLOCK QUOTE OR A LIST ITEM THAT A LINE AT THE FIRST COLUMN ENDS, and a raw HTML '
        + 'block as paragraphs, which MDX has no such block for',
      fn: async () => {
        expect(moduleLineStartsOf({ body: '> a\n\nimport b\n', },),).toEqual([5,],);
        expect(moduleLineStartsOf({ body: '- a\n\nimport b\n', },),).toEqual([5,],);
        expect(moduleLineStartsOf({ body: '<script>\n\nimport b\n', },),).toEqual([10,],);
      },
    },),

    it({
      name: 'READS A BYTE ORDER MARK THAT OPENS THE TEXT AS A CHARACTER OF ITS FIRST LINE, since the text stands in '
        + 'the middle of a page, so the offsets index the text as written',
      fn: async () => {
        expect(moduleLineStartsOf({ body: '\uFEFFimport a\n', },),).toEqual([],);
        expect(moduleLineStartsOf({ body: '\uFEFF\nimport a\n', },),).toEqual([2,],);
      },
    },),

    it({
      name: 'READS NO LINE OF A TEXT NESTED PAST THE BOUND, which the strict grammar refuses whatever it holds',
      fn: async () => {
        expect(moduleLineStartsOf({ body: `${'> '.repeat(300,)}a\n\nimport b\n`, },),).toEqual([],);
      },
    },),
  ],
},);
