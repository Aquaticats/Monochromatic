/**
 Tests for the removal of ANSI color sequences from log lines.
 
 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { stripAnsi, } from './ansi.ts';

await describe({
  name: stripAnsi.name,
  children: [
    it({
      name: 'leaves text without escape characters untouched',
      fn: async ({ expect, }) => {
        expect(stripAnsi('plain text',),).toBe('plain text',);
        expect(stripAnsi('',),).toBe('',);
      },
    },),
    it({
      name: 'removes color sequences with and without parameters',
      fn: async ({ expect, }) => {
        expect(stripAnsi('\u001B[32mINFO\u001B[0m ready',),).toBe('INFO ready',);
        expect(stripAnsi('\u001B[mx',),).toBe('x',);
        expect(stripAnsi('\u001B[1;31;40mred\u001B[39;49m',),).toBe('red',);
      },
    },),
    it({
      name: 'removes several sequences in a row',
      fn: async ({ expect, }) => {
        expect(stripAnsi('\u001B[2m\u001B[32mdim green\u001B[0m',),).toBe('dim green',);
      },
    },),
    it({
      name: 'keeps escape characters that do not start a color sequence',
      fn: async ({ expect, }) => {
        expect(stripAnsi('a\u001Bb',),).toBe('a\u001Bb',);
        expect(stripAnsi('\u001B[2Kcleared',),).toBe('\u001B[2Kcleared',);
        expect(stripAnsi('cut\u001B[3',),).toBe('cut\u001B[3',);
        expect(stripAnsi('end\u001B',),).toBe('end\u001B',);
      },
    },),
    it({
      name: 'reads the parameters up to the first letter that ends them',
      fn: async ({ expect, }) => {
        // The parameters run to the 'm'; the text after it stays.
        expect(stripAnsi('\u001B[12;34mok',),).toBe('ok',);
        // A letter that is not 'm' ends no color sequence, so the segment stays whole.
        expect(stripAnsi('\u001B[32Kx',),).toBe('\u001B[32Kx',);
        // Parameters that reach the end of the text without a final letter.
        expect(stripAnsi('ok\u001B[1;',),).toBe('ok\u001B[1;',);
      },
    },),
    it({
      name: 'handles a long line with many sequences in linear time',
      fn: async ({ expect, }) => {
        expect(
          stripAnsi('\u001B[1mx\u001B[0m'.repeat(20_000,),),
        ).toBe('x'.repeat(20_000,),);
      },
    },),
  ],
},);
