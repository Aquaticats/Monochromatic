/**
 Tests for the absolute size predicates that decide which slices may set a
 document's expansion baseline, and which evidence each one reports.

 WHY THE BOUNDARIES ARE TESTED FROM BOTH SIDES. Every endpoint here is a fixed
 number chosen from a corpus reading, and the whole soundness
 argument for excluding what they name is that they never consult the baseline.
 A predicate that quietly moved by one character would keep passing a test that
 only ever checked the middle of each band.

 Fixtures are invented counts, not corpus text.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  SliceBlockCountRefusalError,
  sliceSizeOf,
  sliceSizesOf,
} from '@monochromatic-dev/translation-repair-corpus-source';
import {
  isPlausibleSlice,
  ratioImplausibility,
  sliceImplausibility,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: sliceImplausibility.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'STAYS SILENT on an ordinary slice, which is most of every document: the corpus '
            + 'expands about threefold and a slice sitting there is evidence of nothing',
          fn: async () => {
            expect(sliceImplausibility({
              slice: {
                sourceChars: 129,
                targetChars: 368,
                sourceBlocks: 2,
                targetBlocks: 2,
              },
            },),).toEqual([],);
          },
        },),

        it({
          name: 'RAISES target-far-shorter where the translation is shorter than the original, which '
            + 'cannot happen to a real Chinese-to-English rendering and means the content is '
            + 'somewhere else or nowhere',
          fn: async () => {
            expect(sliceImplausibility({
              slice: {
                sourceChars: 400,
                targetChars: 200,
                sourceBlocks: 1,
                targetBlocks: 1,
              },
            },),).toEqual(['target-far-shorter',],);
          },
        },),

        it({
          name: 'RAISES target-far-longer where the translation carries more than tenfold, which the '
            + 'original cannot account for and means content arrived from elsewhere',
          fn: async () => {
            expect(sliceImplausibility({
              slice: {
                sourceChars: 100,
                targetChars: 1_500,
                sourceBlocks: 1,
                targetBlocks: 1,
              },
            },),).toEqual(['target-far-longer',],);
          },
        },),

        it({
          name: 'RAISES block-count-gap on sizes that are otherwise unremarkable, since a pairing the '
            + 'two sides disagree about by more than one block makes the ratio a fact about the '
            + 'pairing rather than about the translation',
          fn: async () => {
            expect(sliceImplausibility({
              slice: {
                sourceChars: 200,
                targetChars: 600,
                sourceBlocks: 1,
                targetBlocks: 5,
              },
            },),).toEqual(['block-count-gap',],);
          },
        },),

        it({
          name: 'REPORTS EVERY REASON rather than the first, because the baseline wants only to know '
            + 'a slice is unclean while a judge is owed the whole evidence',
          fn: async () => {
            expect(sliceImplausibility({
              slice: {
                sourceChars: 100,
                targetChars: 2_000,
                sourceBlocks: 1,
                targetBlocks: 9,
              },
            },),).toEqual([
              'target-far-longer',
              'block-count-gap',
            ],);
          },
        },),

        it({
          name: 'STAYS SILENT ON AN EMPTY SIDE, both directions. A slice with no original has no '
            + 'ratio, and one with no translation is a section nobody rendered, which the '
            + 'displacement classifier already names: raising it here too would double-count it and '
            + 'call a missing section a rendering fault',
          fn: async () => {
            for (const slice of [
              {
                sourceChars: 0,
                targetChars: 900,
                sourceBlocks: 0,
                targetBlocks: 9,
              },
              {
                sourceChars: 900,
                targetChars: 0,
                sourceBlocks: 9,
                targetBlocks: 0,
              },
            ]) {
              expect(sliceImplausibility({ slice, },),).toEqual([],);
            }
          },
        },),

        it({
          name: 'HOLDS ITS ENDPOINTS EXACTLY: a ratio of 0.8, a ratio of 10 and a block gap of 1 are '
            + 'all inside, since each bound was picked from a corpus reading and a predicate that '
            + 'moved by one would keep passing a test that only checked the middle',
          fn: async () => {
            for (const slice of [
              {
                sourceChars: 500,
                targetChars: 400,
                sourceBlocks: 1,
                targetBlocks: 1,
              },
              {
                sourceChars: 100,
                targetChars: 1_000,
                sourceBlocks: 1,
                targetBlocks: 1,
              },
              {
                sourceChars: 200,
                targetChars: 600,
                sourceBlocks: 1,
                targetBlocks: 2,
              },
            ]) {
              expect(sliceImplausibility({ slice, },),).toEqual([],);
            }
          },
        },),

        it({
          name: 'RAISES ONE STEP PAST EACH ENDPOINT, which is the half of the boundary check that '
            + 'proves the bound is where it is claimed rather than merely somewhere further out',
          fn: async () => {
            expect(sliceImplausibility({
              slice: {
                sourceChars: 500,
                targetChars: 399,
                sourceBlocks: 1,
                targetBlocks: 1,
              },
            },),).toEqual(['target-far-shorter',],);
            expect(sliceImplausibility({
              slice: {
                sourceChars: 100,
                targetChars: 1_001,
                sourceBlocks: 1,
                targetBlocks: 1,
              },
            },),).toEqual(['target-far-longer',],);
            expect(sliceImplausibility({
              slice: {
                sourceChars: 200,
                targetChars: 600,
                sourceBlocks: 1,
                targetBlocks: 3,
              },
            },),).toEqual(['block-count-gap',],);
          },
        },),

        it({
          name: 'COUNTS A BLOCK GAP IN EITHER DIRECTION, since the pairing can leave the surplus on '
            + 'the original side just as easily as on the translated one',
          fn: async () => {
            expect(sliceImplausibility({
              slice: {
                sourceChars: 200,
                targetChars: 600,
                sourceBlocks: 5,
                targetBlocks: 1,
              },
            },),).toEqual(['block-count-gap',],);
          },
        },),
      ],
    },),

    describe({
      name: isPlausibleSlice.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS a slice raising nothing and REFUSES one raising anything, which is the whole '
            + 'question the baseline asks: it needs to know a slice is clean, not why it is not',
          fn: async () => {
            expect(isPlausibleSlice({
              slice: {
                sourceChars: 129,
                targetChars: 368,
                sourceBlocks: 2,
                targetBlocks: 2,
              },
            },),).toBe(true,);
            expect(isPlausibleSlice({
              slice: {
                sourceChars: 200,
                targetChars: 600,
                sourceBlocks: 1,
                targetBlocks: 5,
              },
            },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: ratioImplausibility.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS EACH RATIO TAIL off character counts alone, nothing for an ordinary ratio, and '
            + 'nothing where a side is empty, so the contest size note counts no blocks (ledger B68)',
          fn: async () => {
            expect([
              ratioImplausibility({ sourceChars: 56, targetChars: 10_381, },),
              ratioImplausibility({ sourceChars: 100, targetChars: 50, },),
              ratioImplausibility({ sourceChars: 100, targetChars: 250, },),
              ratioImplausibility({ sourceChars: 0, targetChars: 50, },),
              ratioImplausibility({ sourceChars: 100, targetChars: 0, },),
            ],).toStrictEqual([
              ['target-far-longer',],
              ['target-far-shorter',],
              [],
              [],
              [],
            ],);
          },
        },),
      ],
    },),

    describe({
      name: sliceSizeOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS CHARACTERS RAW AND BLOCKS AS THE PARSE READS THEM, which is the one place '
            + 'blocks are counted: two callers splitting them differently would run two estimators '
            + 'while reporting one number',
          fn: async () => {
            expect(sliceSizeOf({
              sourceText: 'first\n\nsecond',
              targetText: 'one\n\ntwo\n\nthree',
            },),).toEqual({
              sourceChars: 13,
              targetChars: 15,
              sourceBlocks: 2,
              targetBlocks: 3,
            },);
          },
        },),

        it({
          name: 'IGNORES A TRAILING SEPARATOR and any run of blank lines, because a formatting '
            + 'artifact at the end of a slice is not a block the pairing failed to match, and '
            + 'counting it would raise a gap on a pair that agrees',
          fn: async () => {
            const size = sliceSizeOf({
              sourceText: 'first\n\nsecond\n\n',
              targetText: 'one\n\n\n\ntwo',
            },);
            expect(size.sourceBlocks,).toBe(2,);
            expect(size.targetBlocks,).toBe(2,);
          },
        },),

        it({
          name: 'COUNTS A FENCE WITH A BLANK LINE INSIDE AS ONE BLOCK, as the parse reads it, and a '
            + 'loose list as one, so a side does not gain blocks for spacing its own content '
            + '(ledger B68)',
          fn: async () => {
            expect(sliceSizeOf({
              sourceText: 'The cat yawns.\n\n```text\nMittens\n\nstretches\n```\n\nThe end.',
              targetText: '- A nap.\n\n- A stretch.\n\n- A meal.',
            },),).toStrictEqual({
              sourceChars: 56,
              targetChars: 33,
              sourceBlocks: 3,
              targetBlocks: 1,
            },);
          },
        },),

        it({
          name: 'THROWS RATHER THAN GUESSES where the slice grammar refuses a side, naming the '
            + 'refusal, since a count read some other way would be a second estimator (ledger B68)',
          fn: async () => {
            /**
             What sizing a slice whose rendering opens on an unclosed component throws.
             */
            const refusal = caught(function sizeUnclosed(): void {
              sliceSizeOf({
                sourceText: '猫在睡觉。',
                targetText: '<Cat unclosed\n\nThe cat naps.',
              },);
            },);

            expect(refusal,).toBeInstanceOf(SliceBlockCountRefusalError,);
            expect((refusal as Error).message,).toBe(
              'slice block count could not be read off the parse: MdxParseError: MDX body refused to parse at '
                + '3:13 (micromark-extension-mdx-jsx/unexpected-character); corpus documents compile as MDX '
                + 'upstream, so failure signals corruption or an unsupported construct.',
            );
          },
        },),

        it({
          name: 'READS AN EMPTY SIDE AS ZERO BLOCKS rather than as one empty block, so a section '
            + 'nobody rendered does not arrive at the predicates carrying a block to compare',
          fn: async () => {
            const size = sliceSizeOf({
              sourceText: 'only this side',
              targetText: '',
            },);
            expect(size.targetChars,).toBe(0,);
            expect(size.targetBlocks,).toBe(0,);
          },
        },),
        it({
          name: 'SIZES EVERY SLICE OF A PREPARATION in slice order, each as sliceSizeOf reads it',
          fn: async () => {
            /**
             Two slices, the second an insertion anchor with no original.
             */
            const slices = [
              { source: { text: '猫在睡觉。', }, target: { text: 'The cat naps.', }, },
              { source: { text: '', }, target: { text: 'A kitten waits.\n\nThen it naps.', }, },
            ];
            // Each slice's size spelled out from its own texts, in order,
            // rather than by mapping the slices the way `sliceSizesOf` does.
            expect(sliceSizesOf({ slices, },),).toEqual([
              sliceSizeOf({ sourceText: '猫在睡觉。', targetText: 'The cat naps.', },),
              sliceSizeOf({ sourceText: '', targetText: 'A kitten waits.\n\nThen it naps.', },),
            ],);
            expect(sliceSizesOf({ slices, },)[1]?.targetBlocks,).toBe(2,);
          },
        },),
      ],
    },),
  ],
},);
