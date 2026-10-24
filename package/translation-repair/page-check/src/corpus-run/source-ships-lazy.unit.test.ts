/**
 Tests for the lines of a footnote's paragraph that the strict grammar reads as paragraph text and a text of its own
 would read as a setext underline.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { lazyUnderlinesBlanked, } from '../../dist/final/node/index.mjs';

await describe({
  name: lazyUnderlinesBlanked.name,
  children: [
    it({
      name: 'WRITES LETTERS OVER A LINE OF EQUALS SIGNS OR HYPHENS THAT IS NOT INDENTED, with up to three spaces '
        + 'before it, and keeps the length of the text',
      fn: async () => {
        for (
          const [text, written,] of [
            ['[^1]: x\ny\n===\n[r]: u\n', '[^1]: x\ny\nxxx\n[r]: u\n',],
            ['[^1]: x\ny\n--\n', '[^1]: x\ny\nxx\n',],
            ['[^1]: x\ny\n   =\n', '[^1]: x\ny\n   x\n',],
            ['[^1]: x\r\ny\r\n==  \r\nz', '[^1]: x\r\ny\r\nxx  \r\nz',],
            ['[^1]: x\n===\n-\n', '[^1]: x\nxxx\nx\n',],
          ] as const
        ) {
          expect(lazyUnderlinesBlanked({
            readable: text,
            range: {
              start: 6,
              end: text.length,
            },
          },),).toBe(written,);
        }
      },
    },),

    it({
      name: 'LEAVES THE FIRST LINE, A LINE INDENTED FOUR SPACES OR BY A TAB, A LINE OF MIXED MARKS, A LINE OF OTHER '
        + 'MARKS AND A LINE OUTSIDE THE RANGE AS THEY STAND, and gives the text itself',
      fn: async () => {
        for (
          const text of [
            '[^1]: ===\ny\n',
            '[^1]: x\n    ===\n',
            '[^1]: x\n\t===\n',
            '[^1]: x\n=-=\n',
            '[^1]: x\n***\n',
            '[^1]: x\n\n',
            '[^1]: x',
          ]
        ) {
          expect(lazyUnderlinesBlanked({
            readable: text,
            range: {
              start: 6,
              end: text.length,
            },
          },),).toBe(text,);
        }
        /**
         A line of equals signs after the range.
         */
        const outside = '[^1]: x\ny\n===\n';
        expect(lazyUnderlinesBlanked({
          readable: outside,
          range: {
            start: 6,
            end: 10,
          },
        },),).toBe(outside,);
      },
    },),
  ],
},);
