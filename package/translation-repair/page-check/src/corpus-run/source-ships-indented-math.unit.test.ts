/**
 Tests for display math indented four columns or more after a line of a paragraph.

 The plain parse reads such lines as a line of the paragraph with inline math that runs over them, and the strict grammar,
 which has no indented code, reads a display math there: a line of `$$` alone opens it, whatever its indentation, and the
 next such line closes it. Braces inside display math are no expression, so the lines are copied as they stand; joining
 them into one line, as is done for inline math a brace cuts, leaves the page an unclosed display math and drops the text.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  encodeAsLiteralMdx,
  readingDifferenceOf,
} from '../../dist/final/node/index.mjs';

/**
 What the encoder writes for a text, and what the comparison says of the page.

 @param source - text of a run of slices

 @returns The page text and the answer of the comparison

 @example
 ```ts
 const { written, kind, } = encodedReading({ source: 'a\n    $$\n    {a}\n    $$\n', },);
 ```
 */
function encodedReading(
  { source, }: { readonly source: string; },
): { readonly written: string; readonly kind: string; } {
  /**
   The page text the encoder writes.
   */
  const written = encodeAsLiteralMdx({ text: source, },);
  return {
    written,
    kind: readingDifferenceOf({
      source,
      carried: written,
    },).kind,
  };
}

await describe({
  name: 'source ships display math indented after a paragraph line',
  children: [
    it({
      name: 'COPIES THE LINES OF AN INDENTED DISPLAY MATH AFTER A PARAGRAPH LINE AS THEY STAND, a brace among them',
      fn: async () => {
        for (
          const source of [
            'a\n    $$\n    {a}\n    $$\n',
            'a\n\t$$\n\t{a}\n\t$$\n',
            'a\n    $$\n    {a}\n    $$   \n',
            '> a\n>     $$\n>     {a}\n>     $$\n',
            '- a\n      $$\n      {a}\n      $$\n',
          ] as const
        )
          expect(encodedReading({ source, },),).toEqual({
            written: source,
            kind: 'same',
          },);
      },
    },),

    it({
      name: 'ESCAPES THE TEXT AFTER AN INDENTED DISPLAY MATH AS ANY TEXT',
      fn: async () => {
        expect(encodedReading({ source: 'a\n    $$\n    {a}\n    $$\n\nb {\n', },),).toEqual({
          written: 'a\n    $$\n    {a}\n    $$\n\nb \\{\n',
          kind: 'same',
        },);
      },
    },),

    it({
      name: 'STILL JOINS THE LINES OF MATH THAT THE PAGE DOES NOT READ AS DISPLAY MATH, inline math a brace cuts among them',
      fn: async () => {
        for (
          const [source, written,] of [
            ['a $x\n{y$ z\n', 'a $x {y$ z\n',],
            ['a\n    $$\n    {a} $$\n', 'a\n    $$ {a} $$\n',],
            ['a\n    $$\n    x\n    $$\n', 'a\n    $$\n    x\n    $$\n',],
            ['a\n   $$\n   {a}\n   $$\n', 'a\n   $$\n   {a}\n   $$\n',],
          ] as const
        )
          expect(encodedReading({ source, },),).toEqual({
            written,
            kind: 'same',
          },);
      },
    },),
  ],
},);
