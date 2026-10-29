/**
 Tests the reader that says where the strict grammar first refused a
 document, which the assembly guard blames a structural break by (class
 fifty-eight). Cat-themed invention throughout.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { strictRefusalOffset, } from '../dist/final/node/index.mjs';

/**
 Document the strict grammar accepts.
 */
const WHOLE = '## The cat\n\nThe cat naps on the windowsill[^1].\n\n[^1]: Its favourite spot.\n';

/**
 Document with an expression that never closes, on its second paragraph.
 */
const BROKEN = '## The cat\n\nThe cat naps on the windowsill.\n\nA bird sits there too. {\'unclosed\n';

/**
 Document whose element is still open when it ends, which the parser
 refuses with no place named.
 */
const OPEN_AT_END = 'The cat naps.\n\n<div>\n\nA bird sits there too.\n';

/**
 Document whose first paragraph leaves an element open, refused on leaving
 that paragraph.
 */
const OPEN_IN_PARAGRAPH = 'The cat <b>naps on the windowsill.\n\nA bird sits there too.\n';

await describe({
  name: strictRefusalOffset.name,
  children: [
    it({
      name: 'ANSWERS NOTHING for a document the strict grammar accepts',
      fn: async () => {
        expect(strictRefusalOffset({ text: WHOLE, },),).toEqual({ refused: false, },);
      },
    },),
    it({
      name: 'NAMES THE OFFSET OF THE FIRST REFUSAL, on the line and column the grammar reports',
      fn: async () => {
        /**
         Where the refusal is read.
         */
        const reading = strictRefusalOffset({ text: BROKEN, },);
        expect(reading.refused,).toBe(true,);
        if (!reading.refused)
          throw new Error('the reading is asserted refused',);
        expect(reading.offset,).toBeGreaterThanOrEqual(BROKEN.indexOf('A bird',),);
        expect(reading.offset,).toBeLessThanOrEqual(BROKEN.length,);
      },
    },),
    it({
      name: 'COUNTS THE FRONT MATTER into the offset, so the number indexes the whole text',
      fn: async () => {
        /**
         Same break behind a front matter block.
         */
        const fenced = `---\nname: cat\n---\n\n${BROKEN}`;
        /**
         Offsets with and without the fence.
         */
        const bare = strictRefusalOffset({ text: BROKEN, },);
        const withFence = strictRefusalOffset({ text: fenced, },);
        if ((!bare.refused) || (!withFence.refused))
          throw new Error('both documents carry the break',);
        expect(withFence.offset - bare.offset,).toBe(fenced.length - BROKEN.length,);
      },
    },),
    it({
      name: 'NAMES THE END OF THE DOCUMENT for an element still open there, where the grammar stopped though the '
        + 'parser names no place, rather than the start of the body (ledger B86)',
      fn: async () => {
        expect(strictRefusalOffset({ text: OPEN_AT_END, },),).toEqual({ refused: true, offset: OPEN_AT_END.length, },);
        /**
         Same document behind a front matter block.
         */
        const fenced = `---\nname: cat\n---\n\n${OPEN_AT_END}`;
        expect(strictRefusalOffset({ text: fenced, },),).toEqual({ refused: true, offset: fenced.length, },);
      },
    },),
    it({
      name: 'NAMES THE END OF THE SPAN THE PARSER NAMES, which it refuses on leaving: an element left open in a '
        + 'paragraph stops the grammar where the paragraph ends, not where it starts (ledger B86)',
      fn: async () => {
        expect(strictRefusalOffset({ text: OPEN_IN_PARAGRAPH, },),).toEqual({
          refused: true,
          offset: OPEN_IN_PARAGRAPH.indexOf('\n',),
        },);
      },
    },),
    it({
      name: 'NAMES THE OFFSET IN THE TEXT AS WRITTEN for a refusal on the first line of a document opening with a byte '
        + 'order mark, the mark counted',
      fn: async () => {
        expect(strictRefusalOffset({ text: 'The cat naps </b> here\n', },),).toEqual({ refused: true, offset: 15, },);
        expect(strictRefusalOffset({ text: '\uFEFFThe cat naps </b> here\n', },),).toEqual({
          refused: true,
          offset: 16,
        },);
      },
    },),
    it({
      name: 'NAMES THE OFFSET IN THE TEXT AS WRITTEN for a refusal after the first line of a document opening with a '
        + 'byte order mark, the mark counted',
      fn: async () => {
        expect(strictRefusalOffset({ text: `\uFEFF${OPEN_IN_PARAGRAPH}`, },),).toEqual({
          refused: true,
          offset: OPEN_IN_PARAGRAPH.indexOf('\n',) + 1,
        },);
      },
    },),
  ],
},);
