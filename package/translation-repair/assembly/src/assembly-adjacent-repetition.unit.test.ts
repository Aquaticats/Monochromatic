/**
 Tests for the adjacent-slice repetition check: what it names, and the four
 things it must NOT name.

 WHY IT EXISTS SEPARATELY from the document-scale check: that one requires two
 words of at least five letters before reporting anything, and the `lintong`
 example carries none, so the check written for that defect cannot see it.
 Adjacency is specific enough to need no content gate, which
 `doc/audit/an-archive-rebuilt-from-the-ledger-is-not-the-archive.md`
 measures at one hit in twenty-two lane readings.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  adjacentRepetitionFindings,
  findAdjacentRepetitions,
} from '../dist/final/node/index.mjs';

/**
 Wording long enough to report, with NO word of five letters, mirroring the
 shape of the `lintong` duplication.
 */
const SHORT_WORDED = 'and so we let it be';

/**
 Ordinary wording that carries content words, for the cases where the content
 gate is beside the point.
 */
const PASSAGE = 'the tabby waited by the garden gate';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: findAdjacentRepetitions.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES wording both neighbours ship that the archive said once',
          fn: async () => {
            const found = findAdjacentRepetitions({
              archiveText: `A kitten dozed. ${PASSAGE}. The end.`,
              shippedSlices: [
                {
                  sliceIndex: 2,
                  text: `A kitten dozed, and ${PASSAGE}.`,
                },
                {
                  sliceIndex: 3,
                  text: `${PASSAGE}, as it always had.`,
                },
              ],
            },);
            expect(found.length,).toBe(1,);
            expect(found[0]?.earlierSliceIndex,).toBe(2,);
            expect(found[0]?.laterSliceIndex,).toBe(3,);
            expect(found[0]?.archiveOccurrences,).toBe(1,);
          },
        },),
        it({
          name: 'KEEPS wording that is a character substring of longer shared wording across a word boundary',
          fn: async () => {
            const found = findAdjacentRepetitions({
              archiveText: 'A kitten dozed. The end.',
              shippedSlices: [
                {
                  sliceIndex: 2,
                  text: 'Then cat the garden lanterns were burning bright again. He stood at the garden lantern '
                    + 'before dawn.',
                },
                {
                  sliceIndex: 3,
                  text: 'So cat the garden lanterns were burning bright today. She waited at the garden lantern '
                    + 'after dusk.',
                },
              ],
            },);
            // BY LENGTH, since an adjacent finding carries word counts and never
            // wording: four words for `at the garden lantern`, seven for `cat the
            // garden lanterns were burning bright`.
            expect(found.map(function wordsOf(repeat,): number {
              return repeat.words;
            },)
              .toSorted(function ascending(left, right,): number {
                return left - right;
              },),).toStrictEqual([
                4,
                7,
              ],);
          },
        },),
        it({
          name: 'NAMES wording the archive never carried at all',
          fn: async () => {
            const found = findAdjacentRepetitions({
              archiveText: 'A kitten dozed by the stove. The end.',
              shippedSlices: [
                {
                  sliceIndex: 0,
                  text: `Well, ${PASSAGE}.`,
                },
                {
                  sliceIndex: 1,
                  text: `${PASSAGE} once more.`,
                },
              ],
            },);
            expect(found.length,).toBe(1,);
            expect(found[0]?.archiveOccurrences,).toBe(0,);
          },
        },),
        it({
          name: 'NAMES wording with no content word, which the document check REFUSES',
          fn: async () => {
            const found = findAdjacentRepetitions({
              archiveText: `A kitten dozed, ${SHORT_WORDED}. The end.`,
              shippedSlices: [
                {
                  sliceIndex: 2,
                  text: `A kitten dozed, ${SHORT_WORDED}.`,
                },
                {
                  sliceIndex: 3,
                  text: `${SHORT_WORDED}, said the tabby.`,
                },
              ],
            },);
            expect(found.length,).toBe(1,);
            // FIVE rather than the six words of the fixture, because punctuation
            // rides on its token: the earlier slice ends the run with `be.` and the
            // later one with `be,`, so the shared run stops one word short. That is
            // `whitespaceTokensOf`'s documented behaviour and the reason nothing is reported
            // as repeated which is not repeated verbatim.
            expect(found[0]?.words,).toBe(5,);
          },
        },),
        it({
          name: 'REFUSES wording the archive itself already repeated',
          fn: async () => {
            const found = findAdjacentRepetitions({
              archiveText: `${PASSAGE}. A kitten dozed. ${PASSAGE}. The end.`,
              shippedSlices: [
                {
                  sliceIndex: 2,
                  text: `${PASSAGE}.`,
                },
                {
                  sliceIndex: 3,
                  text: `${PASSAGE}.`,
                },
              ],
            },);
            expect(found.length,).toBe(0,);
          },
        },),
        it({
          name: 'REFUSES a repeat between slices that are NOT neighbours',
          fn: async () => {
            const found = findAdjacentRepetitions({
              archiveText: `A kitten dozed. ${PASSAGE}. The end.`,
              shippedSlices: [
                {
                  sliceIndex: 1,
                  text: `${PASSAGE}.`,
                },
                {
                  sliceIndex: 2,
                  text: 'A kitten dozed by the stove.',
                },
                {
                  sliceIndex: 3,
                  text: `${PASSAGE}.`,
                },
              ],
            },);
            expect(found.length,).toBe(0,);
          },
        },),
        it({
          name: 'REFUSES a shared run shorter than the four-word floor',
          fn: async () => {
            const found = findAdjacentRepetitions({
              archiveText: 'A kitten dozed. The end.',
              shippedSlices: [
                {
                  sliceIndex: 0,
                  text: 'The tabby sat by the stove.',
                },
                {
                  sliceIndex: 1,
                  text: 'A kitten dozed by the window.',
                },
              ],
            },);
            expect(found.length,).toBe(0,);
          },
        },),
        it({
          name: 'REPORTS one maximal match rather than every substring of it',
          fn: async () => {
            const found = findAdjacentRepetitions({
              archiveText: 'A kitten dozed. The end.',
              shippedSlices: [
                {
                  sliceIndex: 4,
                  text: `Look: ${PASSAGE} and dozed.`,
                },
                {
                  sliceIndex: 5,
                  text: `${PASSAGE} and dozed, again.`,
                },
              ],
            },);
            expect(found.length,).toBe(1,);
          },
        },),
        it({
          name: 'REFUSES to look at a single slice, which has no neighbour',
          fn: async () => {
            const found = findAdjacentRepetitions({
              archiveText: 'A kitten dozed. The end.',
              shippedSlices: [
                {
                  sliceIndex: 0,
                  text: `${PASSAGE}. ${PASSAGE}.`,
                },
              ],
            },);
            expect(found.length,).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: adjacentRepetitionFindings.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES the slice pair and the measurements, and NO wording',
          fn: async () => {
            const findings = adjacentRepetitionFindings({
              archiveText: `A kitten dozed. ${PASSAGE}. The end.`,
              shippedSlices: [
                {
                  sliceIndex: 2,
                  text: `A kitten dozed, and ${PASSAGE}.`,
                },
                {
                  sliceIndex: 3,
                  text: `${PASSAGE}, as it always had.`,
                },
              ],
            },);
            expect(findings.length,).toBe(1,);
            expect(findings[0],).toContain('adjacent-repetition',);
            expect(findings[0],).toContain('slices 2 and 3',);
            expect(findings[0],).toContain('archive 1',);
            // THE WHOLE POINT OF RENDERING COUNTS: a findings list travels into
            // logs and artifacts, and corpus wording must not travel with it.
            expect(findings[0],).not.toContain('tabby',);
          },
        },),
        it({
          name: 'RETURNS nothing when neighbours share no reportable wording',
          fn: async () => {
            const findings = adjacentRepetitionFindings({
              archiveText: 'A kitten dozed. The end.',
              shippedSlices: [
                {
                  sliceIndex: 0,
                  text: 'The tabby sat by the stove.',
                },
                {
                  sliceIndex: 1,
                  text: 'A kitten dozed by the window.',
                },
              ],
            },);
            expect(findings.length,).toBe(0,);
          },
        },),
      ],
    },),
  ],
},);
