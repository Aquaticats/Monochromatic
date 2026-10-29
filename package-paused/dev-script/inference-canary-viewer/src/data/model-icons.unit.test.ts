/**
 * Equivalence tests for `extractAndStripDefs`.
 *
 * Capture the pre-refactor behavior of the SVG `<defs>` partitioner so the
 * linear-pass rewrite stays behavior-identical: empty and all-whitespace
 * input, no `<defs>` block, a single block (the documented example), an
 * unterminated opener kept verbatim in content, a stray closing tag, an
 * empty block, leading and trailing content, adjacent blocks, multiple
 * interleaved blocks, and a long repeated run that would overflow the old
 * recursion under a non-tail-call engine.
 *
 * @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { extractAndStripDefs, } from './model-icons.ts';

/** Block repetitions for the long-run case; large enough to exercise the linear scan, fast to compare. */
const LONG_RUN = 50_000;

await describe({
  name: '',
  children: [
    describe({
      name: extractAndStripDefs.name,
      children: [
        it({
          name: 'returns empty defs and empty content for empty input',
          fn: async () => {
            expect(extractAndStripDefs('',),).toEqual({
              defs: '',
              content: '',
            },);
          },
        },),

        it({
          name: 'passes all-whitespace input through as content',
          fn: async () => {
            expect(extractAndStripDefs('   ',),).toEqual({
              defs: '',
              content: '   ',
            },);
          },
        },),

        it({
          name: 'keeps markup with no <defs> block entirely as content',
          fn: async () => {
            expect(extractAndStripDefs('<path d="M0 0"/>',),).toEqual({
              defs: '',
              content: '<path d="M0 0"/>',
            },);
          },
        },),

        it({
          name: 'splits the documented single-block example',
          fn: async () => {
            expect(
              extractAndStripDefs('<defs><linearGradient id="g"/></defs><path/>',),
            ).toEqual({
              defs: '<linearGradient id="g"/>',
              content: '<path/>',
            },);
          },
        },),

        it({
          name: 'keeps an unterminated <defs> opener verbatim in content',
          fn: async () => {
            expect(extractAndStripDefs('<defs>X',),).toEqual({
              defs: '',
              content: '<defs>X',
            },);
          },
        },),

        it({
          name: 'leaves a stray </defs> with no opener as content',
          fn: async () => {
            expect(extractAndStripDefs('P</defs>Q',),).toEqual({
              defs: '',
              content: 'P</defs>Q',
            },);
          },
        },),

        it({
          name: 'collapses an empty <defs></defs> to empty defs and content',
          fn: async () => {
            expect(extractAndStripDefs('<defs></defs>',),).toEqual({
              defs: '',
              content: '',
            },);
          },
        },),

        it({
          name: 'preserves leading content before a <defs> block',
          fn: async () => {
            expect(extractAndStripDefs('AA<defs>G</defs>',),).toEqual({
              defs: 'G',
              content: 'AA',
            },);
          },
        },),

        it({
          name: 'concatenates adjacent blocks with no content between',
          fn: async () => {
            expect(extractAndStripDefs('<defs>A</defs><defs>B</defs>',),).toEqual({
              defs: 'AB',
              content: '',
            },);
          },
        },),

        it({
          name: 'partitions multiple blocks interleaved with content',
          fn: async () => {
            expect(extractAndStripDefs('<defs>A</defs>P<defs>B</defs>Q',),).toEqual({
              defs: 'AB',
              content: 'PQ',
            },);
          },
        },),

        it({
          name: 'keeps content between leading and trailing blocks',
          fn: async () => {
            expect(extractAndStripDefs('<defs>A</defs>M<defs>B</defs>',),).toEqual({
              defs: 'AB',
              content: 'M',
            },);
          },
        },),

        it({
          name: 'partitions a long repeated run in one linear pass',
          fn: async () => {
            expect(
              extractAndStripDefs('<defs>x</defs>'.repeat(LONG_RUN,),),
            ).toEqual({
              defs: 'x'.repeat(LONG_RUN,),
              content: '',
            },);
          },
        },),
      ],
    },),
  ],
},);
