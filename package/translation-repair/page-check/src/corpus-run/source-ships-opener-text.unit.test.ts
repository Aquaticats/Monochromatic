/**
 Tests for how a stretch that held an opener of a comment that never closes is written as text once it is out of its code.

 The opener is written as a character reference, a comment that closes stands as it is, every character that starts
 syntax gets a backslash, and the marks of a container at the start of a line stay.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  endingAt,
  writtenAsText,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'a stretch written as text after an opener is taken out of code',
  children: [
    it({
      name: 'WRITES THE OPENER AS A CHARACTER REFERENCE AND PUTS A BACKSLASH BEFORE EACH CHARACTER THAT STARTS SYNTAX',
      fn: async () => {
        /**
         A line with an opener, emphasis marks, an ampersand and braces.
         */
        const source = 'x <!-- y *z* & {w}';
        expect(writtenAsText({
          source,
          range: {
            start: 0,
            end: source.length,
          },
          openers: [2,],
          closed: [],
        },),).toBe(String.raw`x &lt;\!\-\- y \*z\* \& \{w\}`,);
      },
    },),
    it({
      name: 'COPIES A COMMENT THAT CLOSES AS IT STANDS, since the page masks it',
      fn: async () => {
        /**
         A comment that closes, then an opener.
         */
        const source = 'a <!-- c --> <!-- d';
        expect(writtenAsText({
          source,
          range: {
            start: 0,
            end: source.length,
          },
          openers: [13,],
          closed: [
            {
              start: 2,
              end: 12,
            },
          ],
        },),).toBe(String.raw`a <!-- c --> &lt;\!\-\- d`,);
      },
    },),
    it({
      name: 'KEEPS THE MARKS OF A CONTAINER AT THE START OF A LINE AND ESCAPES A GREATER-THAN SIGN ELSEWHERE',
      fn: async () => {
        /**
         A stretch over two lines, the second in a block quote.
         */
        const source = 'a\n> b <!-- c';
        expect(writtenAsText({
          source,
          range: {
            start: 0,
            end: source.length,
          },
          openers: [source.indexOf('<!--',),],
          closed: [],
        },),).toBe('a\n> b &lt;\\!\\-\\- c',);
        /**
         A greater-than sign in the middle of a line.
         */
        const middle = 'a > b';
        expect(writtenAsText({
          source: middle,
          range: {
            start: 0,
            end: middle.length,
          },
          openers: [],
          closed: [],
        },),).toBe(String.raw`a \> b`,);
      },
    },),
    it({
      name: 'ENDS A LINE WITH A CARRIAGE RETURN AND A LINE FEED ONLY WHERE THE LINE FEED HAS A CARRIAGE RETURN BEFORE IT',
      fn: async () => {
        expect(endingAt({ source: 'a\r\nb', feed: 2, },),).toBe('\r\n',);
        expect(endingAt({ source: 'a\nb', feed: 1, },),).toBe('\n',);
        expect(endingAt({ source: '\nb', feed: 0, },),).toBe('\n',);
        expect(endingAt({ source: 'ab', feed: -1, },),).toBe('\n',);
      },
    },),
  ],
},);
