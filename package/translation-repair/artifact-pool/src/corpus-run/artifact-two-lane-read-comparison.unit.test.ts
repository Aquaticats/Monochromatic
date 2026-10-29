/**
 Tests for the reader's check of the comparison an artifact carries, at the
 level it is written rather than through a whole artifact.

 WHY NOT THROUGH THE READER: the reader parses every delivery row into a fresh
 object before this check runs, so a row that fails while being read cannot
 reach it from a file, and the case that a failure other than the
 comparison's own refusal passes through unchanged needs exactly that row.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ArtifactDeliveryRow,
  ArtifactParseError,
  assertRecordedComparisonMatches,
  compareLanes,
} from '../../dist/final/node/index.mjs';

/**
 Archive wording of the slice both lanes work on.
 */
const ARCHIVE_NAP = 'The cat sleeps on the sill.';

/**
 Path every message under test is built against.
 */
const COMPARISON_PATH = 'CatEntry1.comparison';

/**
 Row where the lane examined the archive's wording and kept it, which both
 lanes carry in the agreeing cases.
 */
const KEPT_ROW: ArtifactDeliveryRow = {
  sliceIndex: 0,
  sourceText: '猫猫在窗台上睡觉。',
  incumbentKind: 'present',
  incumbentText: ARCHIVE_NAP,
  outcome: {
    kind: 'decided',
    acceptedText: ARCHIVE_NAP,
  },
  shippedText: ARCHIVE_NAP,
  delivery: { kind: 'incumbent-retained', },
};

await describe({
  name: assertRecordedComparisonMatches.name,
  children: [
    it({
      name:
        'ACCEPTS a recorded comparison its ledgers derive, and returns the derived rows, which is the '
          + 'control the refusals are read against',
      fn: async () => {
        /**
         What version 2's rules derive from the two ledgers.
         */
        const derived = compareLanes({
          repair: [KEPT_ROW,],
          translate: [KEPT_ROW,],
        },);
        expect(assertRecordedComparisonMatches({
          recorded: derived,
          repair: [KEPT_ROW,],
          translate: [KEPT_ROW,],
          path: COMPARISON_PATH,
        },),).toEqual(derived,);
      },
    },),
    it({
      name:
        'REFUSES two ledgers the comparison refuses, carrying its sentence, which names the slice and '
          + 'what disagreed there and quotes nothing either ledger holds',
      fn: async () => {
        /**
         What differentOriginals raised, read for its class as well as its wording.
         */
        const refusalOfDifferentOriginals = caught(function differentOriginals() {
          assertRecordedComparisonMatches({
            recorded: [],
            repair: [KEPT_ROW,],
            translate: [
              {
                ...KEPT_ROW,
                sourceText: '猫猫在门口等着。',
              },
            ],
            path: COMPARISON_PATH,
          },);
        },);

        expect(refusalOfDifferentOriginals,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfDifferentOriginals as Error).message,).toBe(
          'artifact parse failed at CatEntry1.comparison: expected two ledgers this version can compare '
            + '(slice 0 carries a different original in each ledger, so the two ledgers were built over '
            + 'different slicings).',
        );
      },
    },),
    it({
      name:
        'RETHROWS an error that is not the comparison`s own refusal instead of reporting it as a malformed '
          + 'file: a defect in this reader dressed as an artifact refusal sends an operator to archive a run '
          + 'that was fine',
      fn: async () => {
        /**
         Row that fails while being read rather than while being compared,
         which is what a defect inside the comparison would look like from
         here.
         */
        const unreadable: ArtifactDeliveryRow = {
          ...KEPT_ROW,
          get sourceText(): never {
            throw new RangeError('reader defect, not a fact about the file',);
          },
        };

        /**
         What unreadableRow raised, read for its class.
         */
        const refusalOfUnreadableRow = caught(function unreadableRow() {
          assertRecordedComparisonMatches({
            recorded: [],
            repair: [KEPT_ROW,],
            translate: [unreadable,],
            path: COMPARISON_PATH,
          },);
        },);

        // BY TYPE, and by the same message, so a relabelled error cannot pass.
        expect(refusalOfUnreadableRow,).toBeInstanceOf(RangeError,);
        expect((refusalOfUnreadableRow as Error).message,).toBe('reader defect, not a fact about the file',);
      },
    },),
    it({
      name:
        'REFUSES a recorded comparison with a row count other than the slices its ledgers cover, naming '
          + 'both counts',
      fn: async () => {
        /**
         What countDiffers raised, read for its class as well as its wording.
         */
        const refusalOfCountDiffers = caught(function countDiffers() {
          assertRecordedComparisonMatches({
            recorded: [],
            repair: [KEPT_ROW,],
            translate: [KEPT_ROW,],
            path: COMPARISON_PATH,
          },);
        },);

        expect(refusalOfCountDiffers,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfCountDiffers as Error).message,).toBe(
          'artifact parse failed at CatEntry1.comparison: expected one row per slice the two ledgers cover, '
            + 'which is 1 here, rather than 0.',
        );
      },
    },),
    it({
      name:
        'REFUSES a recorded row its ledgers do not derive, naming the fields that differ and never their '
          + 'values, since the rows carry the archive`s wording and both lanes` output',
      fn: async () => {
        /**
         What version 2's rules derive from the two ledgers.
         */
        const derived = compareLanes({
          repair: [KEPT_ROW,],
          translate: [KEPT_ROW,],
        },);

        /**
         What rowDiffers raised, read for its class as well as its wording.
         */
        const refusalOfRowDiffers = caught(function rowDiffers() {
          assertRecordedComparisonMatches({
            recorded: derived.map(function retitle(row,) {
              return {
                ...row,
                laneRelation: 'both-differ' as const,
              };
            },),
            repair: [KEPT_ROW,],
            translate: [KEPT_ROW,],
            path: COMPARISON_PATH,
          },);
        },);

        expect(refusalOfRowDiffers,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfRowDiffers as Error).message,).toBe(
          'artifact parse failed at CatEntry1.comparison[0]: expected what this version\'s rules derive for '
            + 'slice 0 from the ledgers stored beside it; the stored row differs on laneRelation.',
        );
      },
    },),
  ],
},);
