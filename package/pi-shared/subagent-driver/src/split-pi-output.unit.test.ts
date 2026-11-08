/**
 Splitting Pi's output, fed the chunks directly, so a record cut inside a character is a case the test chooses rather than one a
 scheduler happens to produce.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { startLineSplitter, } from '../dist/final/node/index.mjs';

/**
 A line holding a two-byte character, as Pi would write it.
 */
const LINE = JSON.stringify({ text: 'café line', },);

/**
 Collects what a splitter hands out, one call at a time.

 @returns the lines it has been given, and the splitter
 */
function collectLines(): { readonly lines: string[]; readonly split: (chunk: Buffer,) => void } {
  const lines: string[] = [];
  return {
    lines,
    split: startLineSplitter({
      onLine: function keepLine(line,) {
        lines.push(line,);
      },
    },),
  };
}

await describe({
  name: startLineSplitter.name,
  children: [
    it({
      name: 'joins a line whose bytes arrive in two chunks, cut inside a two-byte character',
      fn: async () => {
        const { lines, split, } = collectLines();
        /**
         The line's bytes.
         */
        const bytes = Buffer.from(`${LINE}\n`,);
        /**
         Where the record is cut: one byte into the two bytes of the `é`.
         */
        const cutAt = bytes.indexOf(Buffer.from('é',),) + 1;
        split(bytes.subarray(
          0,
          cutAt,
        ),);
        expect(lines,).toEqual([],);
        split(bytes.subarray(cutAt,),);
        expect(lines,).toEqual([LINE,],);
      },
    },),
    it({
      name: 'gives each whole line of one chunk, and keeps only the unfinished tail',
      fn: async () => {
        const { lines, split, } = collectLines();
        split(Buffer.from(`one\ntwo\nhalf`,),);
        expect(lines,).toEqual(['one', 'two',],);
        split(Buffer.from('-a-line\n',),);
        expect(lines,).toEqual(['one', 'two', 'half-a-line',],);
      },
    },),
    it({
      name: 'keeps a Unicode line separator inside a line, because only a line feed ends one',
      fn: async () => {
        const { lines, split, } = collectLines();
        split(Buffer.from(`${JSON.stringify({ text: `a${String.fromCodePoint(0x20_28,)}b`, },)}\n`,),);
        expect(lines,).toEqual([JSON.stringify({ text: `a${String.fromCodePoint(0x20_28,)}b`, },),],);
      },
    },),
  ],
},);
