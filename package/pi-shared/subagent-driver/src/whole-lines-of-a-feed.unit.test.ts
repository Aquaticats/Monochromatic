/**
 The lines of a feed a writer has finished: what a reader may parse while another process is still appending to the file, and the
 fragment it must leave alone.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { wholeLinesOf, } from '../dist/final/node/index.mjs';

/**
 One whole record of a feed, which a writer ended with a line feed.
 */
const A_WHOLE_LINE = '{"kind":"turn_start","n":1}';

/**
 A record the writer has not finished, cut inside its last string.
 */
const A_LINE_STILL_BEING_WRITTEN = String.raw`{"kind":"tool_end","tool":"process","result":"{\"details\":{\"process`;

await describe({
  name: wholeLinesOf.name,
  children: [
    it({
      name: 'reads every line of a feed that ends whole, and nothing of the empty piece after its last line feed',
      fn: async () => {
        expect(wholeLinesOf({ shown: `${A_WHOLE_LINE}\n`, },),).toEqual([A_WHOLE_LINE,],);
        expect(wholeLinesOf({ shown: `${A_WHOLE_LINE}\n${A_WHOLE_LINE}\n`, },),).toEqual([A_WHOLE_LINE, A_WHOLE_LINE,],);
      },
    },),
    it({
      name: 'leaves out the line a writer has not finished, so a reader never parses a record cut in half',
      fn: async () => {
        expect(wholeLinesOf({ shown: `${A_WHOLE_LINE}\n${A_LINE_STILL_BEING_WRITTEN}`, },),).toEqual([A_WHOLE_LINE,],);
        // The same text as a reader sees it mid-write: nothing whole yet, so nothing to read.
        expect(wholeLinesOf({ shown: A_LINE_STILL_BEING_WRITTEN, },),).toEqual([],);
      },
    },),
    it({
      name: 'reads nothing from a feed no writer has written to yet',
      fn: async () => {
        expect(wholeLinesOf({ shown: '', },),).toEqual([],);
      },
    },),
  ],
},);
