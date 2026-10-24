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

import { withModuleLinesWritten, } from '../../dist/final/node/index.mjs';

/**
 Writes the module lines of a text.

 @param text - text with every other rewrite made

 @returns Text with the first letter of each module line written as a character reference

 @example
 ```ts
 const written = writtenOf({ text: 'x\n\nimport a\n', },);
 ```
 */
function writtenOf({ text, }: { readonly text: string; },): string {
  return withModuleLinesWritten({ text, },);
}

await describe({
  name: withModuleLinesWritten.name,
  children: [
    it({
      name: 'WRITES THE FIRST LETTER OF A LINE THAT STARTS A MODULE AS A CHARACTER REFERENCE, at the start of the '
        + 'text and after a blank line, and leaves a lazy line of a paragraph alone',
      fn: async () => {
        expect(writtenOf({ text: 'export to PDF\n', },),).toBe('&#101;xport to PDF\n',);
        expect(writtenOf({ text: '她说\n\nimport data\n\nexport to PDF\n', },),)
          .toBe('她说\n\n&#105;mport data\n\n&#101;xport to PDF\n',);
        expect(writtenOf({ text: '她说\nimport data\n', },),).toBe('她说\nimport data\n',);
        expect(writtenOf({ text: 'imported data\n', },),).toBe('imported data\n',);
      },
    },),

    it({
      name: 'LEAVES A TEXT WITH NO MODULE LINE AS IT IS, an indented line, a line in a container and a line in code '
        + 'among them',
      fn: async () => {
        expect(writtenOf({ text: ' import data\n', },),).toBe(' import data\n',);
        expect(writtenOf({ text: '> import data\n', },),).toBe('> import data\n',);
        expect(writtenOf({ text: '```\nimport data\n```\n', },),).toBe('```\nimport data\n```\n',);
        expect(writtenOf({ text: '<!--\n\nimport data\n\n-->\n', },),).toBe('<!--\n\nimport data\n\n-->\n',);
      },
    },),

    it({
      name: 'WRITES THE LINE AFTER A LINE OF INVISIBLE CHARACTERS THE PACKAGE MASKS, a byte order mark, a no-break '
        + 'space and a zero-width space among them, in a paragraph and in a setext heading',
      fn: async () => {
        expect(writtenOf({ text: 'x\n\uFEFF\nimport a\n', },),).toBe('x\n\uFEFF\n&#105;mport a\n',);
        expect(writtenOf({ text: 'x\n\u00A0\nexport to PDF\n---\n', },),).toBe('x\n\u00A0\n&#101;xport to PDF\n---\n',);
        expect(writtenOf({ text: 'x\n\u200B\r\nimport a\n', },),).toBe('x\n\u200B\r\n&#105;mport a\n',);
      },
    },),

    it({
      name: 'LEAVES THE LINE AFTER A LINE OF CHARACTERS THE PACKAGE DOES NOT MASK, a soft hyphen and a left-to-right '
        + 'mark, which the paragraph keeps',
      fn: async () => {
        expect(writtenOf({ text: 'x\n\u00AD\nimport a\n', },),).toBe('x\n\u00AD\nimport a\n',);
        expect(writtenOf({ text: 'x\n\u200E\nimport a\n', },),).toBe('x\n\u200E\nimport a\n',);
      },
    },),

    it({
      name: 'WRITES THE LINE AFTER A SETEXT UNDERLINE, in a list item and in a block quote after a line of invisible '
        + 'characters, which the strict grammar reads as a block of its own',
      fn: async () => {
        expect(writtenOf({ text: '1. 项\n\uFEFF\nimport q\n===\nexport to PDF\n', },),)
          .toBe('1. 项\n\uFEFF\n&#105;mport q\n===\n&#101;xport to PDF\n',);
        expect(writtenOf({ text: '> a\n\uFEFF\nimport q\n===\nimport r\n', },),)
          .toBe('> a\n\uFEFF\n&#105;mport q\n===\n&#105;mport r\n',);
        expect(writtenOf({ text: '- a\n  b\n\u00A0\nimport q\nmore\n===\nexport to PDF\n', },),)
          .toBe('- a\n  b\n\u00A0\n&#105;mport q\nmore\n===\n&#101;xport to PDF\n',);
      },
    },),

    it({
      name: 'WRITES THE LINE AFTER A LIST MARKER ALONE THAT FOLLOWS A FOOTNOTE DEFINITION, wherever the definition '
        + 'stands, and leaves it where a paragraph is open',
      fn: async () => {
        expect(writtenOf({ text: '[^1]: 注\n*\nexport to PDF\n', },),).toBe('[^1]: 注\n*\n&#101;xport to PDF\n',);
        expect(writtenOf({ text: 'x\n[^1]: 注\n2.\nimport a\n', },),).toBe('x\n[^1]: 注\n2.\n&#105;mport a\n',);
        expect(writtenOf({ text: '\uFEFF\n*\nimport a\n', },),).toBe('\uFEFF\n*\n&#105;mport a\n',);
        expect(writtenOf({ text: 'abc\n*\nexport to PDF\n', },),).toBe('abc\n*\nexport to PDF\n',);
      },
    },),

    it({
      name: 'WRITES A LINE IN A RAW HTML BLOCK OR INDENTED CODE THAT FOLLOWS A BLANK LINE, which the strict grammar '
        + 'reads as a paragraph',
      fn: async () => {
        expect(writtenOf({ text: '<script>\n\nimport a\n', },),).toBe('<script>\n\n&#105;mport a\n',);
        expect(writtenOf({ text: '猫\n\n    code\n\nimport a\n', },),).toBe('猫\n\n    code\n\n&#105;mport a\n',);
      },
    },),

    it({
      name: 'JOINS A MODULE LINE IN A CODE SPAN OR INLINE MATH TO THE LINE BEFORE IT, where a character reference would '
        + 'show as its characters: the plain reading keeps the span across a line of invisible characters',
      fn: async () => {
        expect(writtenOf({ text: '`a\n\u200B\nimport x` b\n', },),).toBe('`a\n\u200B import x` b\n',);
        expect(writtenOf({ text: '$a\n\uFEFF\nexport x$ b\n', },),).toBe('$a\n\uFEFF export x$ b\n',);
        expect(writtenOf({ text: '`a\r\n\u200B\r\nimport x` b\r\n', },),).toBe('`a\r\n\u200B import x` b\r\n',);
      },
    },),

    it({
      name: 'JOINS EACH OF TWO MODULE LINES IN ONE SPAN AND WRITES A MODULE LINE OUTSIDE ANY SPAN AS A REFERENCE, then reads '
        + 'the result again until no line is left',
      fn: async () => {
        expect(writtenOf({ text: '`a\n\u00A0\nimport x\n\u00A0\nimport y`\n\nexport to PDF\n', },),)
          .toBe('`a\n\u00A0 import x\n\u00A0 import y`\n\n&#101;xport to PDF\n',);
      },
    },),

    it({
      name: 'READS A BYTE ORDER MARK THAT OPENS THE TEXT AS A CHARACTER OF ITS FIRST LINE, as the page does',
      fn: async () => {
        expect(writtenOf({ text: '\uFEFFimport a\n', },),).toBe('\uFEFFimport a\n',);
        expect(writtenOf({ text: '\uFEFF\nimport a\n', },),).toBe('\uFEFF\n&#105;mport a\n',);
      },
    },),

    it({
      name: 'COUNTS POSITIONS IN UTF-16 UNITS, so a character beyond the first plane before a module line does not '
        + 'shift the letter',
      fn: async () => {
        expect(writtenOf({ text: '\u{1F408}\n\nimport a\n', },),).toBe('\u{1F408}\n\n&#105;mport a\n',);
      },
    },),
  ],
},);
