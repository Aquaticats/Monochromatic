/**
 Equivalence tests for `firstGlobMetaIndex`.
 
 Capture the pre-refactor behavior of the glob metacharacter scanner so
 the linear-pass rewrite stays behavior-identical: empty input, a literal
 path with no metacharacter, each metacharacter in `*?{[`, the first of
 several metacharacters winning, path separators (`/` and `\`) treated as
 ordinary characters rather than metacharacters, and a long no-match run.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { firstGlobMetaIndex, } from '../dist/final/node/testing.mjs';

/** Iteration count for the long no-match run; large enough to exercise the linear scan, fast to compare. */
const LONG_RUN = 100_000;

await describe({
  name: '',
  children: [
    describe({
      name: firstGlobMetaIndex.name,
      children: [
        it({
          name: 'returns -1 for empty input',
          fn: async () => {
            expect(firstGlobMetaIndex('',),).toBe(-1,);
          },
        },),

        it({
          name: 'returns -1 for a literal path with no metacharacter',
          fn: async () => {
            expect(firstGlobMetaIndex('src/index.ts',),).toBe(-1,);
          },
        },),

        it({
          name: 'returns -1 for an all-whitespace string',
          fn: async () => {
            expect(firstGlobMetaIndex('     ',),).toBe(-1,);
          },
        },),

        it({
          name: 'finds a metacharacter at index 0',
          fn: async () => {
            expect(firstGlobMetaIndex('*.ts',),).toBe(0,);
          },
        },),

        it({
          name: 'finds a star after a static prefix',
          fn: async () => {
            expect(firstGlobMetaIndex('src/*.ts',),).toBe(4,);
          },
        },),

        it({
          name: 'finds a trailing star',
          fn: async () => {
            expect(firstGlobMetaIndex('src/**',),).toBe(4,);
          },
        },),

        it({
          name: 'finds a question mark',
          fn: async () => {
            expect(firstGlobMetaIndex('a?b',),).toBe(1,);
          },
        },),

        it({
          name: 'finds an opening brace',
          fn: async () => {
            expect(firstGlobMetaIndex('a{b,c}',),).toBe(1,);
          },
        },),

        it({
          name: 'finds an opening bracket',
          fn: async () => {
            expect(firstGlobMetaIndex('a[bc]',),).toBe(1,);
          },
        },),

        it({
          name: 'returns the first of several metacharacters',
          fn: async () => {
            expect(firstGlobMetaIndex('a*b?c',),).toBe(1,);
          },
        },),

        it({
          name: 'finds a metacharacter after a leading separator',
          fn: async () => {
            expect(firstGlobMetaIndex('/abs/*.ts',),).toBe(5,);
          },
        },),

        it({
          name: 'treats both path separators as ordinary characters',
          fn: async () => {
            expect(firstGlobMetaIndex(
              String.raw`src\sub/*.ts`,
            ),)
              .toBe(8,);
          },
        },),

        it({
          name: 'returns -1 for a long no-metacharacter run',
          fn: async () => {
            expect(
              firstGlobMetaIndex('a'.repeat(LONG_RUN,),),
            ).toBe(-1,);
          },
        },),

        it({
          name: 'finds a metacharacter after a long static prefix',
          fn: async () => {
            const prefix = 'a'.repeat(LONG_RUN,);
            expect(firstGlobMetaIndex(`${prefix}*`,),).toBe(LONG_RUN,);
          },
        },),
      ],
    },),
  ],
},);
