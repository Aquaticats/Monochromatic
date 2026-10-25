/**
 Tests that a comment that never closes, opened inside a fence, a display math, a code span or inline math, is shown as the
 source shows it.

 The package's own parser masks a comment by looking for the four characters of its opener, whatever code they stand in, so
 the opener cannot stand in code on the page, and the character reference that stands for it elsewhere shows as written
 there. What holds the opener is taken out of the code: a fence or display math is closed before the line, the line is
 written as a paragraph and the block is opened again; a code span or inline math is closed before the opener and the rest of
 its content is written as text.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { encodeAsLiteralMdx, } from '../../dist/final/node/index.mjs';
import { readerEqualityCases, } from './source-reader-equality.test-fixture.ts';

/**
 Texts a reader must see as the source shows them: what the case is, and the text.
 */
const CASES: readonly (readonly [string, string])[] = [
  [
    'AN OPENER INSIDE A FENCE THAT NEVER CLOSES, WITH A DEFINITION AFTER IT',
    '```\nx {a}\n<!-- never\n[a]: u\n',
  ],
  [
    'AN OPENER INSIDE A FENCE THAT CLOSES, WITH TEXT AFTER IT',
    '```\n<!-- a\n```\nB {x}\n',
  ],
  [
    'AN OPENER ON THE FIRST LINE OF A TILDE FENCE WITH AN INFO STRING',
    '~~~js {x}\n<!-- a <!-- b\ncode {y}\n~~~\n',
  ],
  [
    'AN OPENER AFTER CODE ON ITS LINE INSIDE A FENCE',
    '```js\ncode <!-- a\nmore {b}\n```\n',
  ],
  [
    'AN OPENER ON EACH OF TWO LINES OF A FENCE',
    '```\none <!-- a\ntwo {c}\nthree <!-- b\n```\nend {d}\n',
  ],
  [
    'AN OPENER BEHIND A COMMENT THAT CLOSES, ON ONE LINE OF A FENCE',
    '```\n<!-- seen --> <!-- never\nlater {e}\n',
  ],
  [
    'AN OPENER INSIDE A FENCE THAT SYNTAX CHARACTERS SHARE A LINE WITH',
    '```\n# <!-- a * _ [b](c) & {d} <e>\n```\n',
  ],
  [
    'AN OPENER INSIDE A FENCE LENGTHENED FOR A MARK INDENTED FOUR SPACES',
    '```\n    ```\n<!-- a\n{b}\n',
  ],
  [
    'AN OPENER INSIDE A FENCE INDENTED TWO SPACES AT THE TOP, ON A LINE INDENTED ONE',
    '  ```\n <!-- a\n  {b}\n',
  ],
  [
    'AN OPENER INSIDE A FENCE IN A LIST ITEM',
    '- ```\n  {\n  <!-- a\n  ```\n',
  ],
  [
    'AN OPENER INSIDE A FENCE IN A NUMBERED LIST ITEM IN A LIST ITEM',
    '- a\n  1. ```\n     {\n     <!-- a\n     [b]: u\n',
  ],
  [
    'AN OPENER INSIDE A FENCE IN A BLOCK QUOTE',
    '> ```\n> x {a}\n> <!-- never\n> y {b}\n',
  ],
  [
    'AN OPENER INSIDE A FENCE THAT FOLLOWS A BYTE ORDER MARK AND A FENCE LINE',
    '﻿```\n{\n```\n<!-- a\n[a]: u\n',
  ],
  [
    'AN OPENER INSIDE A FENCE IN A TEXT OF WINDOWS LINE ENDINGS',
    '```\r\nx {a}\r\n<!-- never\r\n[a]: u\r\n',
  ],
  [
    'AN OPENER INSIDE A FENCE IN A TEXT OF LONE CARRIAGE RETURNS',
    '```\rx {a}\r<!-- never\r[a]: u\r',
  ],
  [
    'AN OPENER INSIDE A DISPLAY MATH THAT NEVER CLOSES',
    '$$\nx\n<!-- never\n$$\nB {c}\n',
  ],
  [
    'AN OPENER ON THE FIRST LINE OF A DISPLAY MATH',
    '$$\n<!-- a\n',
  ],
  [
    'AN OPENER INSIDE A DISPLAY MATH WITH A BRACE BEFORE IT',
    '$$\nx < {y}\n\n<!-- a\n[a]: u\n',
  ],
  [
    'AN OPENER INSIDE A CODE SPAN, AFTER TEXT',
    'a `x <!-- y` z {d}\n',
  ],
  [
    'AN OPENER AT THE START OF A CODE SPAN',
    'a `<!-- y` z {d}\n',
  ],
  [
    'AN OPENER AT THE END OF A CODE SPAN',
    'a `x <!--` z\n',
  ],
  [
    'AN OPENER INSIDE A CODE SPAN OF TWO BACKTICKS THAT HOLDS ONE',
    'a ``x ` <!-- y`` z {d}\n',
  ],
  [
    'TWO OPENERS INSIDE A CODE SPAN',
    'a `x <!-- y <!-- z` w {d}\n',
  ],
  [
    'AN OPENER INSIDE A CODE SPAN IN A BLOCK QUOTE',
    '> a `x <!-- y` z\n',
  ],
  [
    'AN OPENER INSIDE A CODE SPAN THAT RUNS OVER A LINE IN A BLOCK QUOTE',
    '> a `x <!-- y\n> more {b}` z\n',
  ],
  [
    'AN OPENER INSIDE A CODE SPAN THAT RUNS OVER A LINE IN A LIST ITEM',
    '- a `x <!-- y\n  more` z\n',
  ],
  [
    'AN OPENER INSIDE INLINE MATH',
    'a $x <!-- y$ z {d}\n',
  ],
  [
    'AN OPENER INSIDE INLINE MATH OF TWO DOLLAR SIGNS',
    'a $$x <!-- y$$ z {d}\n',
  ],
  [
    'AN OPENER BESIDE A CODE SPAN THAT HOLDS NONE',
    'a `x` <!-- y {d}\n',
  ],
];

await describe({
  name: 'a comment that never closes, opened inside code or math',
  children: [
    ...readerEqualityCases({ cases: CASES, },),
    it({
      name: 'CLOSES THE FENCE BEFORE THE LINE, WRITES THE LINE AS A PARAGRAPH AND OPENS THE FENCE AGAIN',
      fn: async () => {
        expect(encodeAsLiteralMdx({ text: '```\nx {a}\n<!-- never\n[a]: u\n', },),).toBe(
          '```\nx {a}\n```\n\n&lt;\\!\\-\\- never\n\n```\n[a]: u\n```',
        );
      },
    },),
    it({
      name: 'CARRIES THE QUOTE MARKS ON EACH LINE IT WRITES FOR A FENCE IN A BLOCK QUOTE',
      fn: async () => {
        expect(encodeAsLiteralMdx({ text: '> ```\n> x {a}\n> <!-- never\n> y {b}\n', },),).toBe(
          '> ```\n> x {a}\n> ```\n>\n> &lt;\\!\\-\\- never\n>\n> ```\n> y {b}\n',
        );
      },
    },),
    it({
      name: 'WRITES THE LINES IT ADDS WITH THE LINE ENDING THE TEXT HAS, a carriage return and a line feed',
      fn: async () => {
        expect(encodeAsLiteralMdx({ text: '```\r\nx {a}\r\n<!-- never\r\n[a]: u\r\n', },),).toBe(
          '```\r\nx {a}\r\n```\r\n\r\n&lt;\\!\\-\\- never\r\n\r\n```\r\n[a]: u\r\n```',
        );
      },
    },),
    it({
      name: 'KEEPS THE CODE BEFORE THE OPENER IN THE SPAN AND WRITES THE REST AS TEXT',
      fn: async () => {
        expect(encodeAsLiteralMdx({ text: 'a `x <!-- y` z {d}\n', },),).toBe('a `x `&lt;\\!\\-\\- y z \\{d\\}\n',);
      },
    },),
    it({
      name: 'WRITES THE WHOLE SPAN AS TEXT WHEN ONLY WHITE SPACE STANDS BEFORE THE OPENER',
      fn: async () => {
        expect(encodeAsLiteralMdx({ text: 'a `<!-- y` z {d}\n', },),).toBe('a &lt;\\!\\-\\- y z \\{d\\}\n',);
      },
    },),
  ],
},);
