/**
 Tests for the progress line the editor calibration prints per slice.

 WHY THE NUMBER IS THE THING UNDER TEST. Above an overlap of one, slices
 finish out of order, and a line numbered by arrival would claim a position
 another slice owns while nothing claimed the one still running. The line is
 numbered by position in the sample, and the cases pin that the number comes
 from the position and from nothing else about the slice.

 Fixtures are cat-themed invention: the entry id names no real person and no
 corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { SelectionRound, } from '@monochromatic-dev/translation-repair-translate-floor';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { IssueAuthorship, } from '@monochromatic-dev/translation-repair-house-rule';
import {
  shippedAuthors,
  sliceProgressLine,
  type SliceRounds,
} from '../../dist/final/node/index.mjs';
import {
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_VISION,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

//region Fixtures

/**
 Entry the fixture slice is drawn from, invented.
 */
const ENTRY_ID = 'whisker-ledger';

/**
 Chunk of that entry the fixture slice is, chosen apart from every position
 the cases use so a line quoting the wrong number is caught.
 */
const CHUNK_INDEX = 7;

/**
 Slice every case prints a line about.
 */
const SLICE = {
  entryId: ENTRY_ID,
  index: CHUNK_INDEX,
};

/**
 One judged round with nobody in it: the line counts rounds, not voters.
 */
const EMPTY_ROUND: SelectionRound = {
  producers: [],
  ballots: [],
};

/**
 Editors credited with shipping text, in the order they are credited.
 */
const SHIPPERS: readonly RosterModelId[] = [
  SEAT_HYPER_VISION,
  SEAT_HYPER_TEXT_BEDROCK,
];

/**
 Builds what a slice produced from counts alone.

 @param editor - editor rounds judged

 @param refiner - refiner rounds judged

 @param refineAsked - whether the naturalness lane reached a rewriter

 @param shipping - editors credited with shipping text, at most the fixture
 roster's length

 @returns Rounds shaped as the driver collects them

 @example
 ```ts
 const rounds = roundsOf({ editor: 2, refiner: 1, refineAsked: true, shipping: 2, },);
 ```
 */
function roundsOf(
  {
    editor,
    refiner,
    refineAsked,
    shipping,
  }: {
    readonly editor: number;
    readonly refiner: number;
    readonly refineAsked: boolean;
    readonly shipping: number;
  },
): SliceRounds {
  return {
    editor: Array.from(
      { length: editor, },
      function judged(): SelectionRound {
        return EMPTY_ROUND;
      },
    ),
    refiner: Array.from(
      { length: refiner, },
      function judged(): SelectionRound {
        return EMPTY_ROUND;
      },
    ),
    refineAsked,
    editorShipped: SHIPPERS.slice(
      0,
      shipping,
    ),
    refinerShipped: [],
    refinerHeard: [],
  };
}

//endregion Fixtures

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: sliceProgressLine.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NUMBERS the slice by its position in the sample, counted from one, and not by the '
            + 'chunk it is or the order it finished in',
          fn: async () => {
            /**
             Line for the third slice of four.
             */
            const line = sliceProgressLine({
              position: 2,
              total: 4,
              slice: SLICE,
              rounds: roundsOf({
                editor: 0,
                refiner: 0,
                refineAsked: true,
                shipping: 0,
              },),
            },);

            expect(line,).toContain('slice 3 of 4',);
            expect(line,).not.toContain('slice 7',);
            expect(line,).not.toContain('slice 2 of',);
          },
        },),

        it({
          name: 'RENDERS the whole line as the report reads it: entry, chunk, both seats, shippers',
          fn: async () => {
            /**
             Line for a slice that produced rounds on both seats and shipped.
             */
            const line = sliceProgressLine({
              position: 0,
              total: 4,
              slice: SLICE,
              rounds: roundsOf({
                editor: 2,
                refiner: 1,
                refineAsked: true,
                shipping: 2,
              },),
            },);

            expect(line,).toBe(
              `  slice 1 of 4 (${ENTRY_ID} chunk 7): 2 editor rounds, 1 refiner round, 2 editors shipping`,
            );
          },
        },),

        it({
          name: 'SAYS nothing was eligible to rewrite only when the lane reached no rewriter, so an '
            + 'empty refiner count can be told from a rewriter roster that answered nothing',
          fn: async () => {
            /**
             Line for a slice the naturalness lane could not offer anybody.
             */
            const unreached = sliceProgressLine({
              position: 1,
              total: 2,
              slice: SLICE,
              rounds: roundsOf({
                editor: 1,
                refiner: 0,
                refineAsked: false,
                shipping: 1,
              },),
            },);

            /**
             Line for a slice the lane did offer, where nobody was judged.
             */
            const reached = sliceProgressLine({
              position: 1,
              total: 2,
              slice: SLICE,
              rounds: roundsOf({
                editor: 1,
                refiner: 0,
                refineAsked: true,
                shipping: 1,
              },),
            },);

            expect(unreached,).toContain('0 refiner rounds (nothing eligible to rewrite),',);
            expect(reached,).toContain('0 refiner rounds,',);
            expect(reached,).not.toContain('nothing eligible',);
          },
        },),
      ],
    },),

    describe({
      name: shippedAuthors.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CREDITS both halves of the authorship once each, whole-chunk writers first, so a model '
            + 'that wrote the chunk and served an issue inside it is one shipper rather than two',
          fn: async () => {
            /**
             Authorship where one model wrote the whole chunk and also served an
             issue, and another served an issue only.
             */
            const authorship: IssueAuthorship = {
              everyIssue: [SHIPPERS[0] as RosterModelId,],
              perIssue: {
                'issue-1': [
                  SHIPPERS[0] as RosterModelId,
                  SHIPPERS[1] as RosterModelId,
                ],
              },
            };

            expect(shippedAuthors({ authorship, },),).toStrictEqual(SHIPPERS,);
          },
        },),

        it({
          name: 'CREDITS nobody on a slice that shipped no repair, so the driver counts it as unshipped',
          fn: async () => {
            expect(shippedAuthors({
              authorship: {
                everyIssue: [],
                perIssue: {},
              },
            },),).toStrictEqual([],);
          },
        },),
      ],
    },),
  ],
},);
