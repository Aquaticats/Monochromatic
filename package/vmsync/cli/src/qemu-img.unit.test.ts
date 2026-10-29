/**
 Equivalence tests for `firstWhitespaceIndex`.
 
 Captures the pre-refactor behavior of the sha256sum hash-token scanner
 (formerly the nested recursive `findFirstWhitespace` inside `checksum`)
 so the linear-pass rewrite stays behavior-identical: empty input, no
 whitespace at all, each whitespace kind (space, tab, newline, carriage
 return), leading and all-whitespace input, the real "<hash>  file"
 shape, and a long no-whitespace run the old recursion would overflow on.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { firstWhitespaceIndex, } from './qemu-img.ts';

/** Iteration count for the long-run stack-safety case; far exceeds the V8 recursion depth the old walker would have overflowed at, yet computes instantly. */
const LONG_RUN = 100_000;

//region firstWhitespaceIndex: locates the first whitespace char, or string end

await describe({
  name: firstWhitespaceIndex.name,
  children: [
    it({
      name: 'returns 0 for empty input',
      fn: async () => {
        expect(firstWhitespaceIndex('',),).toBe(0,);
      },
    },),

    it({
      name: 'returns length when no whitespace is present',
      fn: async () => {
        const input = 'nowhitespacehere';
        expect(firstWhitespaceIndex(input,),).toBe(input.length,);
      },
    },),

    it({
      name: 'stops at an interior space',
      fn: async () => {
        expect(firstWhitespaceIndex('abc def',),).toBe(3,);
      },
    },),

    it({
      name: 'stops at a tab',
      fn: async () => {
        expect(firstWhitespaceIndex('ab\tcd',),).toBe(2,);
      },
    },),

    it({
      name: 'stops at a newline',
      fn: async () => {
        expect(firstWhitespaceIndex('ab\ncd',),).toBe(2,);
      },
    },),

    it({
      name: 'stops at a carriage return',
      fn: async () => {
        expect(firstWhitespaceIndex('ab\rcd',),).toBe(2,);
      },
    },),

    it({
      name: 'returns 0 for leading whitespace',
      fn: async () => {
        expect(firstWhitespaceIndex('  abc',),).toBe(0,);
      },
    },),

    it({
      name: 'returns 0 for all-whitespace input',
      fn: async () => {
        expect(firstWhitespaceIndex('   ',),).toBe(0,);
      },
    },),

    it({
      name: 'extracts the hash token from sha256sum output',
      fn: async () => {
        const hash = 'a'.repeat(64,);
        expect(firstWhitespaceIndex(`${hash}  /path/to/disk.qcow2`,),).toBe(hash.length,);
      },
    },),

    it({
      name: 'handles a long no-whitespace run without stack overflow',
      fn: async () => {
        expect(
          firstWhitespaceIndex('a'.repeat(LONG_RUN,),),
        ).toBe(LONG_RUN,);
      },
    },),
  ],
},);

//endregion firstWhitespaceIndex
