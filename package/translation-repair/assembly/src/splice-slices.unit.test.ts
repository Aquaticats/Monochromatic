/**
 Tests for rebuilding a translation from per-slice replacements.

 The function is small and its failure is not: it assembles the text that
 actually ships. A splice applied in the wrong order silently corrupts every
 slice after the first, and because each slice is individually well-formed the
 result still looks like plausible prose. So the cases here are mostly about
 ORDER and about offsets that would drift.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { SliceIndexingError, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  placeSlices,
  type SliceReplacement,
  SliceSpliceError,
  spliceSlices,
} from '@monochromatic-dev/translation-repair-page-pass';
import {
  type ChunkRepairOutcome,
  repairReplacements,
} from '../dist/final/node/index.mjs';
import { anchorAt, } from '@monochromatic-dev/translation-repair-slice-reading/ts/insertion-anchor-pair.test-fixture.ts';

/**
 Translation the slices are cut from.

 Three paragraphs of deliberately different lengths, so an off-by-one in
 offset handling cannot coincidentally produce the right answer.
 */
const TARGET_TEXT = 'The cat sleeps.\n\nShe chases butterflies in the garden all afternoon.\n\nShe purrs.';

/**
 Builds one target-side chunk covering a span of {@link TARGET_TEXT}.

 @param sliceIndex - position of this chunk

 @param startOffset - absolute start in target text

 @param endOffset - absolute exclusive end in target text

 @returns Chunk pair whose target side carries the span

 @example
 ```ts
 const pair = chunkAt({ sliceIndex: 0, startOffset: 0, endOffset: 15, },);
 ```
 */
function chunkAt(
  {
    sliceIndex,
    startOffset,
    endOffset,
  }: {
    readonly sliceIndex: number;
    readonly startOffset: number;
    readonly endOffset: number;
  },
): {
  readonly source: {
    readonly sliceIndex: number;
    readonly nodes: readonly never[];
    readonly startOffset: number;
    readonly endOffset: number;
    readonly text: string;
  };
  readonly target: {
    readonly sliceIndex: number;
    readonly nodes: readonly never[];
    readonly startOffset: number;
    readonly endOffset: number;
    readonly text: string;
  };
} {
  /**
   Span this chunk covers, read from the target text so offsets and text
   cannot disagree.
   */
  const text = TARGET_TEXT.slice(
    startOffset,
    endOffset,
  );

  /**
   Shared shape for both sides; only the target side is read by splicing.
   */
  const side = {
    sliceIndex,
    nodes: [],
    startOffset,
    endOffset,
    text,
  } as const;

  return {
    source: side,
    target: side,
  };
}


/**
 Paragraph separator the fixture text uses.
 */
const PARAGRAPH_BREAK = '\n\n';

/**
 Slices covering the three paragraphs, in document order.

 Offsets are DERIVED from the text rather than written down. Hand-counted
 offsets are how a splice test ends up asserting the bug it was written to
 catch, and this fixture only has to agree with itself.
 */
const SLICES = TARGET_TEXT.split(PARAGRAPH_BREAK,)
  .map(function toSlice(
    paragraph,
    sliceIndex,
  ) {
    /**
     Absolute start of this paragraph, unique because the three differ.
     */
    const startOffset = TARGET_TEXT.indexOf(paragraph,);
    return chunkAt({
      sliceIndex,
      startOffset,
      endOffset: startOffset + paragraph.length,
    },);
  },);

/**
 Where the final paragraph begins, which is where a slice with no existing
 translation would be written in.
 */
const FINAL_START = TARGET_TEXT.indexOf('She purrs.',);

/**
 Builds one replacement.

 @param sliceIndex - slice to write into

 @param replacementText - text to write there

 @returns Replacement as a lane emits it

 @example
 ```ts
 const replacement = write({ sliceIndex: 0, replacementText: 'The cat naps.', },);
 ```
 */
function write(
  {
    sliceIndex,
    replacementText,
  }: {
    readonly sliceIndex: number;
    readonly replacementText: string;
  },
): SliceReplacement {
  return {
    sliceIndex,
    replacementText,
  };
}

/**
 Builds one repair outcome, for the mapping that feeds splicing.

 @param sliceIndex - slice this outcome is for

 @param repairedText - winning text for that slice

 @param changed - whether the repair beat unchanged

 @returns Outcome carrying only the fields the mapping reads

 @example
 ```ts
 const outcome = outcomeFor({ sliceIndex: 0, repairedText: 'The cat naps.', changed: true, },);
 ```
 */
function outcomeFor(
  {
    sliceIndex,
    repairedText,
    changed,
  }: {
    readonly sliceIndex: number;
    readonly repairedText: string;
    readonly changed: boolean;
  },
): ChunkRepairOutcome {
  return {
    sliceIndex,
    repairedText,
    changed,
    issues: [],
    resolvedIssueIds: [],
    candidateResolvedIssueIds: [],
    // No checker round in this fixture, so nothing was said about any issue.
    checkerReadings: {},
    recheckReadings: {},
    repairRegions: [],
    // The mapping reads only sliceIndex, repairedText, and changed. The rest
    // are filled to satisfy the contract, deliberately at their inert values so
    // nothing here can look like a repair that happened.
    accuracyPatchSelected: false,
    refined: false,
    rounds: [],
    droppedDeclaredNames: [],
    // Hand-written fixture text, so nothing here has a model author.
    authorship: {
      perIssue: {},
      everyIssue: [],
    },
    nonTranslationVotes: 0,
    nonTranslationContradicted: false,
    nonTranslationStanding: false,
    heardCritics: 0,
    heardCriticIds: [],
    claimAttributions: [],
    findings: [],
    silentStages: [],
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: spliceSlices.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'returns the translation untouched when there is nothing to write, '
            + 'so a run that repaired nothing cannot alter a single byte',
          fn: async () => {
            expect(
              spliceSlices({
                targetText: TARGET_TEXT,
                slices: SLICES,
                replacements: [],
              },),
            ).toBe(TARGET_TEXT,);
          },
        },),

        it({
          name: 'splices a single slice in place, leaving the text on either side '
            + 'of it byte-identical',
          fn: async () => {
            expect(
              spliceSlices({
                targetText: TARGET_TEXT,
                slices: SLICES,
                replacements: [
                  write({
                    sliceIndex: 1,
                    replacementText: 'She chases butterflies all afternoon.',
                  },),
                ],
              },),
            ).toBe(
              'The cat sleeps.\n\nShe chases butterflies all afternoon.\n\nShe purrs.',
            );
          },
        },),

        it({
          name: 'writes a replacement whose lane left a trailing line ending with the '
            + 'span\'s own edges, so the page keeps one block separator where the '
            + 'archive had one (class seventy-five, CuspariaKLSY2: a list-intro '
            + 'slice shipped its text with a trailing newline into a span the archive separator '
            + 'already followed, and the page carried a double blank line the '
            + 'archive never had)',
          fn: async () => {
            expect(
              spliceSlices({
                targetText: TARGET_TEXT,
                slices: SLICES,
                replacements: [
                  write({
                    sliceIndex: 1,
                    replacementText: '\nShe chases butterflies all afternoon.\n',
                  },),
                ],
              },),
            ).toBe(
              'The cat sleeps.\n\nShe chases butterflies all afternoon.\n\nShe purrs.',
            );
          },
        },),

        it({
          name: 'applies MULTIPLE slices correctly even though each replacement '
            + 'changes the length of the text: this is the case that breaks if '
            + 'splicing runs in ascending order, because the first replacement '
            + 'shifts every later offset',
          fn: async () => {
            expect(
              spliceSlices({
                targetText: TARGET_TEXT,
                slices: SLICES,
                replacements: [
                  // A much SHORTER replacement first, so ascending-order splicing
                  // would read the third slice's offsets against a text that had
                  // already shrunk by 36 characters.
                  write({
                    sliceIndex: 1,
                    replacementText: 'She chases butterflies.',
                  },),
                  write({
                    sliceIndex: 2,
                    replacementText: 'She purrs loudly and at length.',
                  },),
                  write({
                    sliceIndex: 0,
                    replacementText: 'The cat naps.',
                  },),
                ],
              },),
            ).toBe(
              'The cat naps.\n\nShe chases butterflies.\n\nShe purrs loudly and at length.',
            );
          },
        },),

        it({
          name: 'ignores the order replacements arrive in, since a driver hands '
            + 'them back in completion order rather than document order',
          fn: async () => {
            /**
             Replacement for the first slice, spliced twice in opposite orders.
             */
            const first = write({
              sliceIndex: 0,
              replacementText: 'The cat naps.',
            },);

            /**
             Replacement for the final slice.
             */
            const last = write({
              sliceIndex: 2,
              replacementText: 'She purrs loudly.',
            },);

            expect(
              spliceSlices({
                targetText: TARGET_TEXT,
                slices: SLICES,
                replacements: [
                  first,
                  last,
                ],
              },),
            ).toBe(
              spliceSlices({
                targetText: TARGET_TEXT,
                slices: SLICES,
                replacements: [
                  last,
                  first,
                ],
              },),
            );
          },
        },),

        it({
          name: 'keeps a replacement that is EMPTY, because deleting a slice is a '
            + 'legitimate result and must not read as no-change',
          fn: async () => {
            expect(
              spliceSlices({
                targetText: TARGET_TEXT,
                slices: SLICES,
                replacements: [
                  write({
                    sliceIndex: 2,
                    replacementText: '',
                  },),
                ],
              },),
            ).toBe(
              'The cat sleeps.\n\nShe chases butterflies in the garden all afternoon.\n\n',
            );
          },
        },),

        it({
          name: 'INSERTS into a zero-length span at exactly that offset, which is '
            + 'the case the translate lane exists for: a passage with no existing '
            + 'translation has nothing to replace and everything to write',
          fn: async () => {
            expect(
              spliceSlices({
                targetText: TARGET_TEXT,
                slices: [
                  ...SLICES.slice(
                    0,
                    2,
                  ),
                  anchorAt({
                    sliceIndex: 2,
                    offset: FINAL_START,
                  },),
                ],
                replacements: [
                  write({
                    sliceIndex: 2,
                    replacementText: 'She dozes in the sun. ',
                  },),
                ],
              },),
            ).toBe(
              'The cat sleeps.\n\nShe chases butterflies in the garden all afternoon.\n\n'
                + 'She dozes in the sun.\n\nShe purrs.',
            );
          },
        },),

        it({
          name: 'writes SEVERAL insertions sharing one offset in document order '
            + 'rather than reversed, which is what a section whose translation is '
            + 'missing entirely looks like once slicing cuts it into several source '
            + 'slices with one place to put them',
          fn: async () => {
            expect(
              spliceSlices({
                targetText: TARGET_TEXT,
                slices: [
                  ...SLICES.slice(
                    0,
                    2,
                  ),
                  anchorAt({
                    sliceIndex: 2,
                    offset: FINAL_START,
                  },),
                  anchorAt({
                    sliceIndex: 3,
                    offset: FINAL_START,
                  },),
                ],
                replacements: [
                  write({
                    sliceIndex: 3,
                    replacementText: 'Then she stretches. ',
                  },),
                  write({
                    sliceIndex: 2,
                    replacementText: 'She dozes in the sun. ',
                  },),
                ],
              },),
            ).toBe(
              'The cat sleeps.\n\nShe chases butterflies in the garden all afternoon.\n\n'
                + 'She dozes in the sun.\n\nThen she stretches.\n\nShe purrs.',
            );
          },
        },),

        it({
          name: 'THROWS when a replacement names a slice that does not exist, '
            + 'rather than silently dropping it: a lost slice means the driver and '
            + 'the slicer disagree, and shipping the remaining splices would hide it',
          fn: async () => {
            /**
             What spliceMissingSlice raised, read for its class as well as its wording.
             */
            const refusalOfSpliceMissingSlice = caught(function spliceMissingSlice() {
              spliceSlices({
                targetText: TARGET_TEXT,
                slices: SLICES,
                replacements: [
                  write({
                    sliceIndex: 9,
                    replacementText: 'The dog barks.',
                  },),
                ],
              },);
            },);

            expect(refusalOfSpliceMissingSlice,).toBeInstanceOf(SliceSpliceError,);
            expect((refusalOfSpliceMissingSlice as Error).message,).toContain('no slice 9',);
          },
        },),

        it({
          name: 'THROWS when two SLICES carry one index, which has happened: a '
            + 'section only one side carried came back holding its section index '
            + 'while every other path stamped the global one. Keyed by index, the '
            + 'second slice would replace the first and one of them would become '
            + 'unreachable while its replacement landed on the other',
          fn: async () => {
            /**
             What spliceCollidingSlices raised, read for its class as well as its wording.
             */
            const refusalOfSpliceCollidingSlices = caught(function spliceCollidingSlices() {
              spliceSlices({
                targetText: TARGET_TEXT,
                slices: [
                  ...SLICES,
                  anchorAt({
                    sliceIndex: 0,
                    offset: FINAL_START,
                  },),
                ],
                replacements: [
                  write({
                    sliceIndex: 0,
                    replacementText: 'The cat naps.',
                  },),
                ],
              },);
            },);

            expect(refusalOfSpliceCollidingSlices,).toBeInstanceOf(SliceSpliceError,);
            expect((refusalOfSpliceCollidingSlices as Error).message,).toContain('two slices carry one index',);
          },
        },),

        it({
          name: 'THROWS when two replacements name ONE slice, since whichever '
            + 'applied second would overwrite the other and the winner would depend '
            + 'on sort order. Two lanes writing the same slice is exactly the shape '
            + 'that produces this, and it must not resolve itself quietly',
          fn: async () => {
            /**
             What spliceDuplicate raised, read for its class as well as its wording.
             */
            const refusalOfSpliceDuplicate = caught(function spliceDuplicate() {
              spliceSlices({
                targetText: TARGET_TEXT,
                slices: SLICES,
                replacements: [
                  write({
                    sliceIndex: 1,
                    replacementText: 'She chases butterflies.',
                  },),
                  write({
                    sliceIndex: 1,
                    replacementText: 'She chases moths.',
                  },),
                ],
              },);
            },);

            expect(refusalOfSpliceDuplicate,).toBeInstanceOf(SliceSpliceError,);
            expect((refusalOfSpliceDuplicate as Error).message,).toContain('two replacements name one slice',);
          },
        },),

        it({
          name: 'writes an anchor BEFORE the content span it shares a start with, which is the shape a '
            + 'missing paragraph makes at a section that is otherwise translated: the new rendering belongs '
            + 'ahead of the passage that follows it, not after',
          fn: async () => {
            expect(
              spliceSlices({
                targetText: TARGET_TEXT,
                slices: [
                  ...SLICES.slice(
                    0,
                    2,
                  ),
                  anchorAt({
                    sliceIndex: 2,
                    offset: FINAL_START,
                  },),
                  chunkAt({
                    sliceIndex: 3,
                    startOffset: FINAL_START,
                    endOffset: TARGET_TEXT.length,
                  },),
                ],
                replacements: [
                  write({
                    sliceIndex: 3,
                    replacementText: 'She purrs loudly.',
                  },),
                  write({
                    sliceIndex: 2,
                    replacementText: 'Then she yawns. ',
                  },),
                ],
              },),
            ).toBe(
              'The cat sleeps.\n\nShe chases butterflies in the garden all afternoon.\n\n'
                + 'Then she yawns.\n\nShe purrs loudly.',
            );
          },
        },),

        it({
          name: 'writes an anchor at the END of the document, which is where a section the translation never '
            + 'reached belongs: the archive stops early, and the rendering goes after everything that is there',
          fn: async () => {
            expect(
              spliceSlices({
                targetText: TARGET_TEXT,
                slices: [
                  ...SLICES,
                  anchorAt({
                    sliceIndex: 3,
                    offset: TARGET_TEXT.length,
                  },),
                ],
                replacements: [
                  write({
                    sliceIndex: 3,
                    replacementText: '\n\nShe sleeps again.',
                  },),
                ],
              },),
            ).toBe(`${TARGET_TEXT}\n\nShe sleeps again.\n`,);
          },
        },),

        it({
          name: 'THROWS when slice indices are unique but NOT positions, which is the case that writes '
            + 'plausible text in the wrong order: two anchors at one boundary are written in descending index '
            + 'so they land ascending, and that is document order only while an index is a position',
          fn: async () => {
            /**
             What spliceShuffledIndices raised, read for its class as well as its wording.
             */
            const refusalOfSpliceShuffledIndices = caught(function spliceShuffledIndices() {
              spliceSlices({
                targetText: TARGET_TEXT,
                slices: [
                  ...SLICES,
                  anchorAt({
                    sliceIndex: 4,
                    offset: TARGET_TEXT.length,
                  },),
                  anchorAt({
                    sliceIndex: 3,
                    offset: TARGET_TEXT.length,
                  },),
                ],
                replacements: [
                  write({
                    sliceIndex: 3,
                    replacementText: 'B',
                  },),
                  write({
                    sliceIndex: 4,
                    replacementText: 'A',
                  },),
                ],
              },);
            },);

            expect(refusalOfSpliceShuffledIndices,).toBeInstanceOf(SliceIndexingError,);
            expect((refusalOfSpliceShuffledIndices as Error).message,).toContain('reads that index as the position',);
          },
        },),

        it({
          name: 'THROWS when an anchor is handed blank text for an original that says something. The slice '
            + 'has no existing translation, so writing nothing leaves the passage missing while every count '
            + 'reports it delivered',
          fn: async () => {
            /**
             What spliceBlankInsertion raised, read for its class as well as its wording.
             */
            const refusalOfSpliceBlankInsertion = caught(function spliceBlankInsertion() {
              spliceSlices({
                targetText: TARGET_TEXT,
                slices: [
                  ...SLICES,
                  anchorAt({
                    sliceIndex: 3,
                    offset: TARGET_TEXT.length,
                  },),
                ],
                replacements: [
                  write({
                    sliceIndex: 3,
                    replacementText: '   \n',
                  },),
                ],
              },);
            },);

            expect(refusalOfSpliceBlankInsertion,).toBeInstanceOf(SliceSpliceError,);
            expect((refusalOfSpliceBlankInsertion as Error).message,).toContain('writes none',);
          },
        },),

        it({
          name: 'THROWS as well when an anchor is handed only invisible characters, which trim() keeps and no '
            + 'reader sees, and writes blank text at an anchor whose original shows nothing either (ledger B40)',
          fn: async () => {
            /**
             What spliceInvisibleInsertion raised, read for its class as well as its wording.
             */
            const refusalOfSpliceInvisibleInsertion = caught(function spliceInvisibleInsertion() {
              spliceSlices({
                targetText: TARGET_TEXT,
                slices: [
                  ...SLICES,
                  anchorAt({
                    sliceIndex: 3,
                    offset: TARGET_TEXT.length,
                  },),
                ],
                replacements: [
                  write({
                    sliceIndex: 3,
                    replacementText: '\u{200B}\u{3164}',
                  },),
                ],
              },);
            },);

            expect(refusalOfSpliceInvisibleInsertion,).toBeInstanceOf(SliceSpliceError,);
            expect((refusalOfSpliceInvisibleInsertion as Error).message,).toContain('writes none',);

            /**
             Anchor whose original is a zero-width space alone.
             */
            const silentAnchor = anchorAt({
              sliceIndex: 3,
              offset: TARGET_TEXT.length,
            },);

            expect(spliceSlices({
              targetText: TARGET_TEXT,
              slices: [
                ...SLICES,
                {
                  ...silentAnchor,
                  source: {
                    ...silentAnchor.source,
                    text: '\u{200B}',
                  },
                },
              ],
              replacements: [
                write({
                  sliceIndex: 3,
                  replacementText: '',
                },),
              ],
            },),).toBe(TARGET_TEXT,);
          },
        },),
      ],
    },),

    describe({
      name: placeSlices.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'GIVES THE SAME TEXT AS spliceSlices and the range each content span was written over, in the order of the page, '
            + 'wherever earlier writes grew or shrank the text',
          fn: async () => {
            /**
             Replacements that grow the first span and shrink the last, written in no order.
             */
            const replacements = [
              write({ sliceIndex: 2, replacementText: 'Purr.', },),
              write({ sliceIndex: 0, replacementText: 'The cat naps all through the long afternoon.', },),
            ];
            const { text, placed, } = placeSlices({ targetText: TARGET_TEXT, slices: SLICES, replacements, },);
            expect(text,).toBe(spliceSlices({ targetText: TARGET_TEXT, slices: SLICES, replacements, },),);
            expect(placed.map(function rangeText({ start, end, },): string {
              return text.slice(
                start,
                end,
              );
            },),).toEqual(['The cat naps all through the long afternoon.', 'Purr.',],);
            expect(placed.map(function indexOfPlaced({ sliceIndex, },): number {
              return sliceIndex;
            },),).toEqual([0, 2,],);
            expect(placed.every(function startsAtEnglish({ start, englishAt, },): boolean {
              return start === englishAt;
            },),).toBe(true,);
          },
        },),
        it({
          name: 'PLACES THE FIRST CHARACTER OF THE ENGLISH where it would stand, before the range when the lane carried more line '
            + 'endings at its start than the span kept, and after the start when it carried fewer, since a span keeps its own edges',
          fn: async () => {
            /**
             A translation whose second and third spans each start with a blank line.
             */
            const targetText = 'One.\n\nTwo.\n\nThree.';
            /**
             The slices over it: the first a plain sentence, the others each holding the blank line before it.
             */
            const slices = [
              [0, 4,],
              [4, 10,],
              [10, 18,],
            ].map(function pairOf(
              [startOffset = 0, endOffset = 0,],
              sliceIndex,
            ) {
              /**
               The side both halves of the pair share, since only the target is read.
               */
              const side = {
                sliceIndex,
                nodes: [],
                startOffset,
                endOffset,
                text: targetText.slice(
                  startOffset,
                  endOffset,
                ),
              } as const;
              return { source: side, target: side, };
            },);
            /**
             The English written into the second and third spans: three blank lines to the span's two, and none to its two.
             */
            const englishes = ['\n\n\nTwo cats.', 'Three cats.',];
            const { text, placed, } = placeSlices({
              targetText,
              slices,
              replacements: englishes.map(function replacementOf(
                replacementText,
                at,
              ): SliceReplacement {
                return write({ sliceIndex: at + 1, replacementText, },);
              },),
            },);
            expect(text,).toBe('One.\n\nTwo cats.\n\nThree cats.',);
            expect(placed.map(function wordsAtEnglish({ sliceIndex, englishAt, },): string {
              /**
               The English written for this slice.
               */
              const english = englishes[sliceIndex - 1] ?? '';
              /**
               The English without the line endings it starts with.
               */
              const words = english.trimStart();
              return text.slice(
                englishAt + (english.length - words.length),
                englishAt + english.length,
              );
            },),).toEqual(['Two cats.', 'Three cats.',],);
            expect(placed.map(function englishAtOf({ englishAt, },): number {
              return englishAt;
            },),).toEqual([3, 17,],);
          },
        },),
        it({
          name: 'PLACES THE BODY OF EACH ANCHORED FRAGMENT, without the blank lines assembly put around it or cut from it, and the first '
            + 'character of the fragment before the body when blank lines were cut',
          fn: async () => {
            /**
             Two anchors at the end of the second paragraph.
             */
            const slices = [
              ...SLICES.slice(
                0,
                2,
              ),
              anchorAt({ sliceIndex: 2, offset: FINAL_START, },),
              anchorAt({ sliceIndex: 3, offset: FINAL_START, },),
            ];
            const { text, placed, } = placeSlices({
              targetText: TARGET_TEXT,
              slices,
              replacements: [
                write({ sliceIndex: 3, replacementText: 'Then she stretches. ', },),
                write({ sliceIndex: 2, replacementText: '\n\nShe dozes in the sun. ', },),
              ],
            },);
            /**
             Where the first fragment's body starts.
             */
            const firstAt = text.indexOf('She dozes',);
            /**
             Where the second fragment's body starts.
             */
            const secondAt = text.indexOf('Then she',);
            expect(placed,).toEqual([
              { sliceIndex: 2, start: firstAt, end: firstAt + 'She dozes in the sun.'.length, englishAt: firstAt - 2, },
              { sliceIndex: 3, start: secondAt, end: secondAt + 'Then she stretches.'.length, englishAt: secondAt, },
            ],);
          },
        },),
        it({
          name: 'GIVES NO PLACE TO A FRAGMENT THAT SHOWS A READER NOTHING, which writes nothing, and keeps the places of the others',
          fn: async () => {
            /**
             An anchor at the end whose original is a zero-width space, written nothing.
             */
            const silentAnchor = anchorAt({ sliceIndex: 3, offset: TARGET_TEXT.length, },);
            const { text, placed, } = placeSlices({
              targetText: TARGET_TEXT,
              slices: [
                ...SLICES,
                { ...silentAnchor, source: { ...silentAnchor.source, text: '\u{200B}', }, },
              ],
              replacements: [
                write({ sliceIndex: 3, replacementText: '', },),
                write({ sliceIndex: 0, replacementText: 'The cat naps.', },),
              ],
            },);
            expect(text.startsWith('The cat naps.',),).toBe(true,);
            expect(placed.map(function indexOfPlaced({ sliceIndex, },): number {
              return sliceIndex;
            },),).toEqual([0,],);
          },
        },),
      ],
    },),

    describe({
      name: repairReplacements.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'drops outcomes that changed nothing rather than writing them back '
            + 'over themselves, since a no-op write still reads as a slice this '
            + 'lane touched in every later diff and count',
          fn: async () => {
            expect(repairReplacements({ outcomes: [
              outcomeFor({
                sliceIndex: 0,
                repairedText: 'The cat naps.',
                changed: false,
              },),
              outcomeFor({
                sliceIndex: 2,
                repairedText: 'She rumbles.',
                changed: true,
              },),
            ], },),).toEqual([
              {
                sliceIndex: 2,
                replacementText: 'She rumbles.',
              },
            ],);
          },
        },),

        it({
          name: 'carries an EMPTY repair through, because a repair that deletes a '
            + 'slice is a change and dropping it would ship the text it deleted',
          fn: async () => {
            expect(repairReplacements({ outcomes: [
              outcomeFor({
                sliceIndex: 1,
                repairedText: '',
                changed: true,
              },),
            ], },),).toHaveLength(1,);
          },
        },),
      ],
    },),
  ],
},);
