/**
 Tests for seed application, region tracking, hit matching, and derivation.
 Fixtures are cat-themed invention only.

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
  applySeededErrors,
  SEED_MATCH_TOLERANCE,
  SeedApplicationError,
  type SeededErrorSpec,
  seedHitByRegion,
} from '@monochromatic-dev/translation-repair-lane-contest';
import {
  deriveOmissionSeeds,
  splitSentences,
} from '../dist/final/node/index.mjs';

/**
 Clean target text seeds are planted into.
 */
const CLEAN = 'The cat naps in the sun. The cat also chases butterflies across the garden. The cat purrs.';

/**
 Deletion spec over the butterfly sentence.
 */
const DELETE_BUTTERFLIES: SeededErrorSpec = {
  id: 'seed/omission-0',
  category: 'accuracy/omission',
  kind: 'deletion',
  needle: ' The cat also chases butterflies across the garden.',
  replacement: '',
};

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: applySeededErrors.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'tracks a zero-width region at the deletion point',
          fn: async () => {
            /** Application of the single deletion. */
            const result = applySeededErrors({
              text: CLEAN,
              specs: [DELETE_BUTTERFLIES,],
            },);
            expect(result.seededText,).toBe('The cat naps in the sun. The cat purrs.',);
            expect(result.applications,).toEqual([{
              spec: DELETE_BUTTERFLIES,
              startOffset: 24,
              endOffset: 24,
            },],);
          },
        },),

        it({
          name: 'tracks replacement and insertion extents',
          fn: async () => {
            /** Application of one replacement and one insertion. */
            const result = applySeededErrors({
              text: CLEAN,
              specs: [
                {
                  id: 'seed/mistranslation-0',
                  category: 'accuracy/mistranslation',
                  kind: 'replacement',
                  needle: 'naps in the sun',
                  replacement: 'howls at the moon',
                },
                {
                  id: 'seed/addition-0',
                  category: 'accuracy/addition',
                  kind: 'insertion',
                  needle: 'The cat purrs.',
                  replacement: ' It signed the treaty.',
                },
              ],
            },);
            expect(result.seededText,).toBe(
              'The cat howls at the moon. The cat also chases butterflies across the garden. The cat purrs. It signed the treaty.',
            );
            // Replacement region covers the written text; insertion region covers
            // only the inserted addition. Slicing the seeded text by the tracked
            // region proves the coordinates instead of hand-counting them.
            expect(
              result.seededText.slice(
                result.applications[0]?.startOffset ?? 0,
                result.applications[0]?.endOffset ?? 0,
              ),
            ).toBe('howls at the moon',);
            expect(
              result.seededText.slice(
                result.applications[1]?.startOffset ?? 0,
                result.applications[1]?.endOffset ?? 0,
              ),
            ).toBe(' It signed the treaty.',);
          },
        },),

        it({
          name: 'rebases earlier regions when a later edit lands before them, writing both texts and '
            + 'moving the first region by the length the later edit added',
          fn: async () => {
            /** Spec editing text after the second spec's region. */
            const late: SeededErrorSpec = {
              id: 'seed/late',
              category: 'accuracy/mistranslation',
              kind: 'replacement',
              needle: 'purrs',
              replacement: 'meows loudly',
            };

            /** Spec editing text before the first spec's region, one character longer than its needle. */
            const early: SeededErrorSpec = {
              id: 'seed/early',
              category: 'accuracy/mistranslation',
              kind: 'replacement',
              needle: 'The cat naps',
              replacement: 'A kitten naps',
            };

            /** Later spec edits text before the first spec's region. */
            const result = applySeededErrors({
              text: CLEAN,
              specs: [late, early,],
            },);
            expect(result.seededText,).toBe(
              'A kitten naps in the sun. The cat also chases butterflies across the garden. The cat meows loudly.',
            );
            // The late region started at 84 in the clean text and moved to 85 by
            // the one character the early edit added ahead of it.
            expect(result.applications,).toEqual([
              {
                spec: late,
                startOffset: 85,
                endOffset: 97,
              },
              {
                spec: early,
                startOffset: 0,
                endOffset: 13,
              },
            ],);
          },
        },),

        it({
          name: 'REFUSES A NEEDLE THE TEXT LACKS with the whole message naming the seed, and a needle that '
            + 'occurs twice with its own, so a seed is never left half applied',
          fn: async () => {
            /**
             What a needle absent from the text threw.
             */
            const absent = caught(function seedsAbsentNeedle(): unknown {
              return applySeededErrors({
                text: CLEAN,
                specs: [{ ...DELETE_BUTTERFLIES, needle: 'the mouse', },],
              },);
            },);
            expect(absent,).toBeInstanceOf(SeedApplicationError,);
            expect(String(absent,),).toBe(
              'SeedApplicationError: seed seed/omission-0 cannot apply: needle absent from current text.',
            );
            /**
             What a needle occurring twice threw.
             */
            const ambiguous = caught(function seedsAmbiguousNeedle(): unknown {
              return applySeededErrors({
                text: CLEAN,
                specs: [{ ...DELETE_BUTTERFLIES, needle: 'The cat', },],
              },);
            },);
            expect(ambiguous,).toBeInstanceOf(SeedApplicationError,);
            expect(String(ambiguous,),).toBe(
              'SeedApplicationError: seed seed/omission-0 cannot apply: needle occurs more than once.',
            );
          },
        },),
      ],
    },),

    describe({
      name: seedHitByRegion.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'hits within tolerance of a zero-width region and misses beyond it',
          fn: async () => {
            /** Zero-width application at offset 100. */
            const application = {
              spec: DELETE_BUTTERFLIES,
              startOffset: 100,
              endOffset: 100,
            };
            expect(seedHitByRegion({
              spanStart: 100 - SEED_MATCH_TOLERANCE,
              spanEnd: (100 - SEED_MATCH_TOLERANCE) + 5,
              application,
            },),).toBe(true,);
            expect(seedHitByRegion({
              spanStart: 0,
              spanEnd: 100 - SEED_MATCH_TOLERANCE,
              application,
            },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: splitSentences.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'segments mixed-language text on terminators',
          fn: async () => {
            expect(splitSentences({ text: '猫猫晒太阳。The cat purrs. 还追蝴蝶！', },),).toEqual([
              '猫猫晒太阳。',
              'The cat purrs.',
              '还追蝴蝶！',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: deriveOmissionSeeds.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'derives deletion seeds from the longest unique sentences',
          fn: async () => {
            /** Body with two long sentences and one short one. */
            const body = 'The cat purrs. '
              + 'The cat also chases butterflies across the whole garden every morning. '
              + 'The neighbors say the cat naps on the warmest windowsill of the house.';
            /** Derived seeds capped at one. */
            const seeds = deriveOmissionSeeds({
              text: body,
              maxSeeds: 1,
            },);
            expect(seeds,).toHaveLength(1,);
            expect(seeds[0]?.kind,).toBe('deletion',);
            expect(seeds[0]?.category,).toBe('accuracy/omission',);
            expect(seeds[0]?.needle,)
              .toBe('The cat also chases butterflies across the whole garden every morning.',);
          },
        },),

        it({
          name: 'skips short sentences entirely',
          fn: async () => {
            expect(deriveOmissionSeeds({
              text: 'The cat purrs. 猫猫晒太阳。',
              maxSeeds: 3,
            },),).toEqual([],);
          },
        },),

        it({
          name: 'skips sentences carrying MDX expression or JSX delimiters',
          fn: async () => {
            /**
             Body whose longest sentences each hold half of a paired MDX
             construct; deleting any of them would break the seeded parse.
             */
            const body = "The cat opened {'a very long quoted expression about sunbeams. "
              + "It kept purring until the quoted expression finally closed here'} today. "
              + 'The cat also stared at <em>the fancy butterfly emphasis tag construct</em> daily. '
              + 'The plain sentence about the cat napping on the windowsill survives selection.';
            /** Derived seeds; only the delimiter-free sentence qualifies. */
            const seeds = deriveOmissionSeeds({
              text: body,
              maxSeeds: 3,
            },);
            expect(seeds,).toHaveLength(1,);
            expect(seeds[0]?.needle,)
              .toBe('The plain sentence about the cat napping on the windowsill survives selection.',);
          },
        },),
      ],
    },),
  ],
},);
