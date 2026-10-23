/**
 Tests for reading a recorded contest back out of a settled artifact.

 WHAT IS UNDER TEST is what the reader refuses. A recorded verdict is a claim
 about ballots stored beside it and a recorded contest is a claim about which
 slices the two lanes worded differently, so both are re-derived here and a
 disagreement is an error rather than a reading.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  describeContestSlice,
  describeSoleLaneSlice,
  type LaneContestBallot,
  withTenseCandidates,
} from '@monochromatic-dev/translation-repair-lane-contest';
import {
  type ArtifactComparisonRow,
  ArtifactParseError,
} from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  parseLaneSelection,
  SLICE_SPELLED_KEYS,
} from '../../dist/final/node/index.mjs';
import { ARCHIVE_NAP, catRow, } from '@monochromatic-dev/translation-repair-artifact-pool/ts/two-lane-comparison-row.test-fixture.ts';
import { insertionRow, } from '@monochromatic-dev/translation-repair-artifact-pool/ts/sole-lane-comparison-row.test-fixture.ts';
import { tenseLabelsOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/tense-sheet.test-fixture.ts';

/**
 Generation that records a sole-lane verdict at an archive-absent insertion.
 */
const SOLE_LANE_GENERATION = 15;

/**
 Generation before a sole-lane verdict existed, whose insertions asked the roster.
 */
const ROSTER_ONLY_GENERATION = 14;

/**
 Wording the repair lane left.
 */
const REPAIR_NAP = 'The cat naps in the bookshop attic.';

/**
 Wording the translate lane left, differing from both.
 */
const TRANSLATE_NAP = 'The cat dozes in the attic of the bookshop.';

/**
 Path every message under test is built against.
 */
const SELECTION_PATH = 'CatEntry1.laneSelection';

/**
 Ballot backing the repair lane, in the recorded rather than the typed form.
 */
const FOR_REPAIR = {
  choice: 'repair',
  unsupported: [],
  unsupportedRaw: [],
  dropped: [],
  droppedRaw: [],
  reason: 'keeps the bookshop, which the original names',
};

/**
 Ballot backing the translate lane.
 */
const FOR_TRANSLATE = {
  choice: 'translate',
  unsupported: [],
  unsupportedRaw: [],
  dropped: [],
  droppedRaw: [],
  reason: 'reads more naturally and says the same thing',
};

/**
 Ballot backing neither, which is a reading rather than a silence.
 */
const FOR_NEITHER = {
  choice: 'neither',
  unsupported: [
    'repair',
    'translate',
  ],
  unsupportedRaw: ['both add an attic the original does not mention',],
  dropped: [],
  droppedRaw: [],
  reason: 'both candidates invent a detail',
};

/**
 Slice both lanes worded differently, which a contest may answer.
 */
const CONTESTED_ROW = catRow({
  sliceIndex: 0,
  repairText: REPAIR_NAP,
  translateText: TRANSLATE_NAP,
},);

/**
 Comparison where slice 0 and slice 2 are worded differently by the two lanes
 and slice 1 is not, so eligibility is neither every slice nor none of them.
 */
const MIXED: readonly ArtifactComparisonRow[] = [
  CONTESTED_ROW,
  catRow({
    sliceIndex: 1,
    repairText: ARCHIVE_NAP,
    translateText: ARCHIVE_NAP,
  },),
  catRow({
    sliceIndex: 2,
    repairText: REPAIR_NAP,
    translateText: ARCHIVE_NAP,
  },),
];

/**
 Comparison whose one slice the two lanes worded differently.
 */
const ONE_CONTESTED: readonly ArtifactComparisonRow[] = [CONTESTED_ROW,];

/**
 Source metadata making archive and repair identities inadmissible.
 */
const FRONT_SOURCE = '---\nname: 猫猫\ninfo:\n  alias: 猫猫\n---\n';

/**
 Archive metadata retaining directory id as visible name.
 */
const FRONT_ARCHIVE = '---\nname: CatEntry\ninfo:\n  alias: Maomao\n---\n';

/**
 Source-backed translated metadata.
 */
const FRONT_TRANSLATE = '---\nname: Maomao\ninfo:\n  alias: Maomao\n---\n';

/**
 Front matter row whose only admissible lane is translate.
 */
const FRONT_ROW: ArtifactComparisonRow = {
  ...catRow({
    sliceIndex: 0,
    repairText: FRONT_ARCHIVE,
    translateText: FRONT_TRANSLATE,
  },),
  incumbentText: FRONT_ARCHIVE,
};

/**
 A contest over the one contested slice whose two refusing ballots judged the
 archive alike, a quorum of voices on it, with the verdict the case records.

 @param judged - what each ballot made of the archive

 @param verdict - verdict the record claims

 @returns Selection as an artifact carries it
 */
function archiveContest(
  {
    judged,
    verdict,
  }: {
    readonly judged: 'publishable' | 'flawed';
    readonly verdict: Readonly<Record<string, string>>;
  },
) {
  return {
    kind: 'contested',
    slices: [{
      sliceIndex: 0,
      verdict,
      ballots: [
        {
          ...FOR_NEITHER,
          archive: judged,
        },
        {
          ...FOR_NEITHER,
          archive: judged,
        },
      ],
      usable: 2,
    },],
  };
}

await describe({
  name: parseLaneSelection.name,
  children: [
    it({
      name:
        'ACCEPTS the pending kind, which every artifact settled before the contest existed carries '
          + 'and which stays legal for exactly that reason',
      fn: async () => {
        expect(parseLaneSelection({
          value: { kind: 'pending-human-decision', },
          comparison: MIXED,
          path: SELECTION_PATH,
          keys: SLICE_SPELLED_KEYS,
        },),).toEqual({ kind: 'pending-human-decision', },);
      },
    },),
    it({
      name:
        'ACCEPTS a contest answering every eligible slice, carrying a win, a settled refusal and an '
          + 'unheard roster side by side',
      fn: async () => {
        /**
         Selection the reader returned, narrowed before its slices are read.
         */
        const selection = parseLaneSelection({
          value: {
            kind: 'contested',
            slices: [
              {
                sliceIndex: 0,
                verdict: {
                  kind: 'lane-won',
                  lane: 'repair',
                },
                ballots: [
                  FOR_REPAIR,
                  FOR_REPAIR,
                  FOR_TRANSLATE,
                ],
                usable: 3,
              },
              {
                sliceIndex: 2,
                verdict: { kind: 'settled-neither', },
                ballots: [
                  FOR_NEITHER,
                  FOR_NEITHER,
                ],
                usable: 2,
              },
            ],
          },
          comparison: MIXED,
          path: SELECTION_PATH,
          keys: SLICE_SPELLED_KEYS,
        },);
        if (selection.kind !== 'contested')
          throw new Error('reader returned a pending selection for a contested one',);
        expect(selection.slices
          .map(function nameVerdict(slice,): string {
            return slice.verdict
              .kind;
          },),).toEqual([
            'lane-won',
            'settled-neither',
          ],);
      },
    },),
    it({
      name:
        'ACCEPTS A SOLE-LANE VERDICT at an archive-absent insertion in a generation that records one, '
          + 'with no ballots and no usable voices, since no roster was asked',
      fn: async () => {
        const selection = parseLaneSelection({
          value: {
            kind: 'contested',
            slices: [
              {
                sliceIndex: 0,
                verdict: {
                  kind: 'sole-lane',
                  lane: 'translate',
                },
                ballots: [],
                usable: 0,
              },
            ],
          },
          comparison: [insertionRow({
            sliceIndex: 0,
            translateText: TRANSLATE_NAP,
          },),],
          path: SELECTION_PATH,
          keys: SLICE_SPELLED_KEYS,
          generation: SOLE_LANE_GENERATION,
        },);
        expect(selection,).toEqual({
          kind: 'contested',
          slices: [
            {
              sliceIndex: 0,
              verdict: {
                kind: 'sole-lane',
                lane: 'translate',
              },
              ballots: [],
              usable: 0,
            },
          ],
        },);
      },
    },),
    it({
      name:
        'READS A ROSTER CONTEST AT AN ARCHIVE-ABSENT INSERTION UNCHANGED in a generation before sole-lane, '
          + 'since that file asked the roster and its verdict is the one its ballots settle',
      fn: async () => {
        const selection = parseLaneSelection({
          value: {
            kind: 'contested',
            slices: [
              {
                sliceIndex: 0,
                verdict: {
                  kind: 'lane-won',
                  lane: 'translate',
                },
                ballots: [
                  FOR_TRANSLATE,
                  FOR_TRANSLATE,
                  FOR_TRANSLATE,
                ],
                usable: 3,
              },
            ],
          },
          comparison: [insertionRow({
            sliceIndex: 0,
            translateText: TRANSLATE_NAP,
          },),],
          path: SELECTION_PATH,
          keys: SLICE_SPELLED_KEYS,
          generation: ROSTER_ONLY_GENERATION,
        },);
        if (selection.kind !== 'contested')
          throw new Error('reader returned a pending selection for a contested one',);
        expect(selection.slices,).toEqual([
          {
            sliceIndex: 0,
            verdict: {
              kind: 'lane-won',
              lane: 'translate',
            },
            ballots: [
              FOR_TRANSLATE,
              FOR_TRANSLATE,
              FOR_TRANSLATE,
            ],
            usable: 3,
          },
        ],);
      },
    },),
    it({
      name: 'RECOMPUTES SCHEMA 7 VERDICT from raw ballots after syntax eligibility floor',
      fn: async () => {
        const selection = parseLaneSelection({
          value: {
            kind: 'contested',
            slices: [{
              sliceIndex: 0,
              verdict: {
                kind: 'lane-won',
                lane: 'translate',
              },
              ballots: [
                FOR_REPAIR,
                FOR_REPAIR,
                FOR_TRANSLATE,
                FOR_TRANSLATE,
              ],
              usable: 4,
              eligibility: {
                syntax: 'front-matter',
                sourceText: FRONT_SOURCE,
                archive: 'ineligible',
                repair: 'ineligible',
                translate: 'eligible',
              },
            },],
          },
          comparison: [FRONT_ROW,],
          path: SELECTION_PATH,
          keys: SLICE_SPELLED_KEYS,
          generation: 7,
        },);
        if (selection.kind !== 'contested')
          throw new Error('reader returned pending selection for eligibility record',);
        expect(selection.slices[0]?.verdict,).toEqual({
          kind: 'lane-won',
          lane: 'translate',
        },);
        expect(selection.slices[0]?.ballots,).toHaveLength(4,);
      },
    },),

    it({
      name: 'KEEPS the tense candidates a contested slice records at generation fifteen, beside a verdict they do not touch',
      fn: async () => {
        /**
         Ballot backing the repair lane, typed as the stage returns it.
         */
        const forRepair: LaneContestBallot = {
          choice: 'repair',
          unsupported: [],
          unsupportedRaw: [],
          dropped: [],
          droppedRaw: [],
          reason: 'keeps the bookshop, which the original names',
        };
        /**
         The slice as the writer records it, with the tense candidates of its texts.
         */
        const recorded = withTenseCandidates({
          slices: [
            describeContestSlice({
              sliceIndex: 0,
              outcome: {
                choice: 'repair',
                ballots: [
                  forRepair,
                  forRepair,
                  forRepair,
                ],
                usable: 3,
                findings: [],
              },
            },),
          ],
          comparison: ONE_CONTESTED,
          lifeEnded: true,
          subjectNames: ['The cat',],
        },);
        expect(recorded[0]?.tenseCandidates?.length,).toBe(3,);
        expect(parseLaneSelection({
          value: {
            kind: 'contested',
            slices: recorded,
          },
          comparison: ONE_CONTESTED,
          path: SELECTION_PATH,
          keys: SLICE_SPELLED_KEYS,
          generation: 15,
        },),).toEqual({
          kind: 'contested',
          slices: recorded,
        },);
      },
    },),

    it({
      name: 'READS WHAT THE WRITER RECORDS for a lane that won, a slice the archive stands at and a slice one lane applies '
        + 'to alone, the finding on the first two and none on the third',
      fn: async () => {
        /**
         Ballot backing the repair lane, typed as the stage returns it.
         */
        const forRepair: LaneContestBallot = {
          choice: 'repair',
          unsupported: [],
          unsupportedRaw: [],
          dropped: [],
          droppedRaw: [],
          reason: 'keeps the bookshop, which the original names',
        };
        /**
         Ballot backing neither lane, typed as the stage returns it.
         */
        const forNeither: LaneContestBallot = {
          choice: 'neither',
          unsupported: [
            'repair',
            'translate',
          ],
          unsupportedRaw: ['both add an attic the original does not mention',],
          dropped: [],
          droppedRaw: [],
          reason: 'both candidates invent a detail',
        };
        /**
         Rows of the three slices: two the lanes worded differently, and an insertion only the translate lane filled.
         */
        const comparison: readonly ArtifactComparisonRow[] = [
          CONTESTED_ROW,
          catRow({
            sliceIndex: 1,
            repairText: REPAIR_NAP,
            translateText: TRANSLATE_NAP,
          },),
          insertionRow({
            sliceIndex: 2,
            translateText: TRANSLATE_NAP,
          },),
        ];
        /**
         The three records as the contest driver writes them, with the finding added.
         */
        const recorded = withTenseCandidates({
          slices: [
            describeContestSlice({
              sliceIndex: 0,
              outcome: {
                choice: 'repair',
                ballots: [
                  forRepair,
                  forRepair,
                  forRepair,
                ],
                usable: 3,
                findings: [],
              },
            },),
            describeContestSlice({
              sliceIndex: 1,
              outcome: {
                choice: 'neither',
                ballots: [
                  forNeither,
                  forNeither,
                  forNeither,
                ],
                usable: 3,
                findings: [],
              },
            },),
            describeSoleLaneSlice({
              sliceIndex: 2,
              lane: 'translate',
            },),
          ],
          comparison,
          lifeEnded: true,
          subjectNames: ['The cat',],
        },);
        expect(tenseLabelsOf({ slices: recorded, },),).toEqual([
          [
            'ARCHIVE RENDERING',
            'CANDIDATE "repair"',
            'CANDIDATE "translate"',
          ],
          [
            'ARCHIVE RENDERING',
            'CANDIDATE "repair"',
            'CANDIDATE "translate"',
          ],
          [],
        ],);
        expect(parseLaneSelection({
          value: {
            kind: 'contested',
            slices: recorded,
          },
          comparison,
          path: SELECTION_PATH,
          keys: SLICE_SPELLED_KEYS,
          generation: SOLE_LANE_GENERATION,
        },),).toEqual({
          kind: 'contested',
          slices: recorded,
        },);
      },
    },),

    it({
      name: 'READS A SLICE THE FLOOR KEPT FROM THE ROSTER apart from a roster that declined: with generation '
        + 'twenty-one the mark is read back as true, and a quorum-not-met slice with no ballots and no mark is a '
        + 'real decline',
      fn: async () => {
        /**
         Selection with one skipped slice, as the writer records it.
         */
        const skipped = parseLaneSelection({
          value: {
            kind: 'contested',
            slices: [{
              sliceIndex: 0,
              verdict: { kind: 'quorum-not-met', },
              ballots: [],
              usable: 0,
              benchBelowFloor: true,
            },],
          },
          comparison: ONE_CONTESTED,
          path: SELECTION_PATH,
          keys: SLICE_SPELLED_KEYS,
          generation: 21,
        },);
        /**
         The same selection with no mark, which a roster that was asked and heard nobody leaves.
         */
        const declined = parseLaneSelection({
          value: {
            kind: 'contested',
            slices: [{
              sliceIndex: 0,
              verdict: { kind: 'quorum-not-met', },
              ballots: [],
              usable: 0,
            },],
          },
          comparison: ONE_CONTESTED,
          path: SELECTION_PATH,
          keys: SLICE_SPELLED_KEYS,
          generation: 21,
        },);
        expect({
          skipped: (skipped.kind === 'contested') ? skipped.slices[0]?.benchBelowFloor : 'pending',
          declined: (declined.kind === 'contested') ? declined.slices[0]?.benchBelowFloor : 'pending',
        },).toEqual({
          skipped: true,
          declined: undefined,
        },);
      },
    },),

    it({
      name: 'REFUSES THE FLOOR MARK in generation twenty, which did not record it, naming the keys that '
        + 'generation allows',
      fn: async () => {
        const refusal = caught(function markBeforeTwentyOne() {
          return parseLaneSelection({
            value: {
              kind: 'contested',
              slices: [{
                sliceIndex: 0,
                verdict: { kind: 'quorum-not-met', },
                ballots: [],
                usable: 0,
                benchBelowFloor: true,
              },],
            },
            comparison: ONE_CONTESTED,
            path: SELECTION_PATH,
            keys: SLICE_SPELLED_KEYS,
            generation: 20,
          },);
        },);
        expect(refusal,).toBeInstanceOf(ArtifactParseError,);
        expect(String(refusal,),).toBe(
          'ArtifactParseError: artifact parse failed at CatEntry1.laneSelection.slices[0].benchBelowFloor: '
            + 'expected no key here beyond sliceIndex, verdict, ballots, usable, eligibility, tenseCandidates.',
        );
      },
    },),

    it({
      name: 'REFUSES THE FLOOR MARK ON A SLICE A ROSTER ANSWERED, or set to false, since a slice asked about carries '
        + 'ballots and the mark says none was asked',
      fn: async () => {
        /**
         Reads one recorded slice at generation twenty-one and returns the refusal text.

         @param slice - slice as recorded

         @returns Refusal
         */
        function refusalOf(slice: Readonly<Record<string, unknown>>,): string {
          return String(caught(function readMarked() {
            return parseLaneSelection({
              value: {
                kind: 'contested',
                slices: [slice,],
              },
              comparison: ONE_CONTESTED,
              path: SELECTION_PATH,
              keys: SLICE_SPELLED_KEYS,
              generation: 21,
            },);
          },),);
        }
        expect({
          answered: refusalOf({
            sliceIndex: 0,
            verdict: {
              kind: 'lane-won',
              lane: 'repair',
            },
            ballots: [FOR_REPAIR, FOR_REPAIR, FOR_TRANSLATE,],
            usable: 3,
            benchBelowFloor: true,
          },),
          falsy: refusalOf({
            sliceIndex: 0,
            verdict: { kind: 'quorum-not-met', },
            ballots: [],
            usable: 0,
            benchBelowFloor: false,
          },),
        },).toEqual({
          answered: 'ArtifactParseError: artifact parse failed at CatEntry1.laneSelection.slices[0].benchBelowFloor: '
            + 'expected no ballots, since a bench read below its floor was not asked, rather than 3.',
          falsy: 'ArtifactParseError: artifact parse failed at CatEntry1.laneSelection.slices[0].benchBelowFloor: '
            + 'expected true or absent.',
        },);
      },
    },),

    it({
      name: 'REFUSES a tense candidate record in generation fourteen, the last generation before the field',
      fn: async () => {
        const refusal = caught(function tenseBeforeFifteen() {
          return parseLaneSelection({
            value: {
              kind: 'contested',
              slices: [{
                sliceIndex: 0,
                verdict: {
                  kind: 'lane-won',
                  lane: 'repair',
                },
                ballots: [FOR_REPAIR, FOR_REPAIR, FOR_TRANSLATE,],
                usable: 3,
                tenseCandidates: [],
              },],
            },
            comparison: ONE_CONTESTED,
            path: SELECTION_PATH,
            keys: SLICE_SPELLED_KEYS,
            generation: 14,
          },);
        },);
        expect(refusal,).toBeInstanceOf(ArtifactParseError,);
        expect(String(refusal,),).toBe(
          'ArtifactParseError: artifact parse failed at CatEntry1.laneSelection.slices[0].tenseCandidates: '
            + 'expected no key here beyond sliceIndex, verdict, ballots, usable, eligibility.',
        );
      },
    },),

    it({
      name: 'REFUSES a tense candidate record on a sole-lane slice, which asked no judge and showed no sheet, rather than dropping it unread',
      fn: async () => {
        const refusal = caught(function tenseOnSoleLane() {
          return parseLaneSelection({
            value: {
              kind: 'contested',
              slices: [
                {
                  sliceIndex: 0,
                  verdict: {
                    kind: 'sole-lane',
                    lane: 'translate',
                  },
                  ballots: [],
                  usable: 0,
                  tenseCandidates: [{
                    label: 'CANDIDATE "translate"',
                    found: [{
                      sentence: 'She naps in the attic of the bookshop.',
                      matched: 'She naps',
                      rule: 's-ending-verb',
                    },],
                  },],
                },
              ],
            },
            comparison: [insertionRow({
              sliceIndex: 0,
              translateText: TRANSLATE_NAP,
            },),],
            path: SELECTION_PATH,
            keys: SLICE_SPELLED_KEYS,
            generation: SOLE_LANE_GENERATION,
          },);
        },);
        expect(refusal,).toBeInstanceOf(ArtifactParseError,);
        expect(String(refusal,),).toBe(
          'ArtifactParseError: artifact parse failed at CatEntry1.laneSelection.slices[0].tenseCandidates: '
            + 'expected no tense candidates, since no roster was asked about a slice one lane applies to.',
        );
      },
    },),

    it({
      name: 'REFUSES a tense candidate naming a rule the detector never writes',
      fn: async () => {
        const refusal = caught(function tenseRuleUnknown() {
          return parseLaneSelection({
            value: {
              kind: 'contested',
              slices: [{
                sliceIndex: 0,
                verdict: {
                  kind: 'lane-won',
                  lane: 'repair',
                },
                ballots: [FOR_REPAIR, FOR_REPAIR, FOR_TRANSLATE,],
                usable: 3,
                tenseCandidates: [{
                  label: 'CANDIDATE "repair"',
                  found: [{
                    sentence: 'The cat naps.',
                    matched: 'cat naps',
                    rule: 'guess',
                  },],
                },],
              },],
            },
            comparison: ONE_CONTESTED,
            path: SELECTION_PATH,
            keys: SLICE_SPELLED_KEYS,
            generation: 15,
          },);
        },);
        expect(refusal,).toBeInstanceOf(ArtifactParseError,);
        expect(String(refusal,),).toBe(
          'ArtifactParseError: artifact parse failed at CatEntry1.laneSelection.slices[0].tenseCandidates[0].found[0].rule: '
            + 'expected a present-tense rule the detector writes.',
        );
      },
    },),

    it({
      name:
        'KEEPS THE JUDGE each ballot names, so a settled page answers which models called a note '
          + 'an addition (hulicaijia26, 2026-09-26)',
      fn: async () => {
        /**
         Selection the reader returned, narrowed before its ballots are read.
         */
        const selection = parseLaneSelection({
          value: {
            kind: 'contested',
            slices: [{
              sliceIndex: 0,
              verdict: {
                kind: 'lane-won',
                lane: 'repair',
              },
              ballots: [
                {
                  ...FOR_REPAIR,
                  modelId: 'whiskers-judge',
                },
                {
                  ...FOR_REPAIR,
                  modelId: 'mittens-judge',
                },
                {
                  ...FOR_TRANSLATE,
                  modelId: 'tabby-judge',
                },
              ],
              usable: 3,
            },],
          },
          comparison: ONE_CONTESTED,
          path: SELECTION_PATH,
          keys: SLICE_SPELLED_KEYS,
        },);
        if (selection.kind !== 'contested')
          throw new Error('reader returned pending selection for a contested record',);
        expect(selection.slices[0]?.ballots.map(function judgeOf(cast,) {
          return cast.modelId;
        },),).toEqual([
          'whiskers-judge',
          'mittens-judge',
          'tabby-judge',
        ],);
      },
    },),
    it({
      name: 'REFUSES SCHEMA 7 FRONT MATTER CONTEST missing eligibility record',
      fn: async () => {
        const refusal = await caught(() => parseLaneSelection({
          value: {
            kind: 'contested',
            slices: [{
              sliceIndex: 0,
              verdict: {
                kind: 'lane-won',
                lane: 'translate',
              },
              ballots: [FOR_TRANSLATE, FOR_TRANSLATE,],
              usable: 2,
            },],
          },
          comparison: [FRONT_ROW,],
          path: SELECTION_PATH,
          keys: SLICE_SPELLED_KEYS,
          generation: 7,
        },),);
        expect(refusal,).toBeInstanceOf(ArtifactParseError,);
        expect((refusal as Error).message,).toContain(
          `at ${SELECTION_PATH}.slices[0].eligibility: expected source-backed syntax eligibility record rather than `
            + 'absence',
        );
      },
    },),

    it({
      name: 'REFUSES SCHEMA 7 ELIGIBILITY STATUS disagreeing with source and lane texts',
      fn: async () => {
        const refusal = await caught(() => parseLaneSelection({
          value: {
            kind: 'contested',
            slices: [{
              sliceIndex: 0,
              verdict: { kind: 'settled-neither', },
              ballots: [FOR_REPAIR, FOR_REPAIR,],
              usable: 2,
              eligibility: {
                syntax: 'front-matter',
                sourceText: FRONT_SOURCE,
                archive: 'ineligible',
                repair: 'eligible',
                translate: 'eligible',
              },
            },],
          },
          comparison: [FRONT_ROW,],
          path: SELECTION_PATH,
          keys: SLICE_SPELLED_KEYS,
          generation: 7,
        },),);
        expect(refusal,).toBeInstanceOf(ArtifactParseError,);
        expect((refusal as Error).message,).toContain(
          `at ${SELECTION_PATH}.slices[0].eligibility.repair: expected ineligible, which deterministic syntax guard `
            + 'derives, rather than eligible.',
        );
      },
    },),

    it({
      name: 'ACCEPTS a slice too few of whose voices arrived, recording it as unsettled rather than as a refusal',
      fn: async () => {
        /**
         Selection the reader returned, narrowed before its slices are read.
         */
        const selection = parseLaneSelection({
          value: {
            kind: 'contested',
            slices: [
              {
                sliceIndex: 0,
                verdict: { kind: 'quorum-not-met', },
                ballots: [FOR_REPAIR,],
                usable: 1,
              },
            ],
          },
          comparison: ONE_CONTESTED,
          path: SELECTION_PATH,
          keys: SLICE_SPELLED_KEYS,
        },);
        if (selection.kind !== 'contested')
          throw new Error('reader returned a pending selection for a contested one',);
        expect(selection.slices
          .at(0,)
          ?.verdict,).toEqual({ kind: 'quorum-not-met', },);
      },
    },),
    it({
      name:
        'ROUND-TRIPS an ARCHIVE VERDICT on a ballot, and gives a ballot carrying none back with NO '
          + 'archive key rather than an undefined one: the archive is judged as its own question, so a '
          + 'ballot written before that question existed has to read as silent about it, and a key set '
          + 'to undefined is not silence to anything that asks whether the ballot answered',
      fn: async () => {
        /**
         Selection whose one ballot judged the archive beside the two lanes.
         */
        const judged = parseLaneSelection({
          value: {
            kind: 'contested',
            slices: [
              {
                sliceIndex: 0,
                verdict: { kind: 'quorum-not-met', },
                ballots: [{
                  ...FOR_REPAIR,
                  archive: 'flawed',
                },],
                usable: 1,
              },
            ],
          },
          comparison: ONE_CONTESTED,
          path: SELECTION_PATH,
          keys: SLICE_SPELLED_KEYS,
        },);
        if (judged.kind !== 'contested')
          throw new Error('reader returned a pending selection for a contested one',);
        expect(judged.slices
          .at(0,)
          ?.ballots
          .at(0,)
          ?.archive,).toBe('flawed',);

        /**
         Same selection with the ballot silent about the archive.
         */
        const silent = parseLaneSelection({
          value: {
            kind: 'contested',
            slices: [
              {
                sliceIndex: 0,
                verdict: { kind: 'quorum-not-met', },
                ballots: [FOR_REPAIR,],
                usable: 1,
              },
            ],
          },
          comparison: ONE_CONTESTED,
          path: SELECTION_PATH,
          keys: SLICE_SPELLED_KEYS,
        },);
        if (silent.kind !== 'contested')
          throw new Error('reader returned a pending selection for a contested one',);

        /**
         Ballot the reader gave back, whose KEYS are what this half reads.
         */
        const ballot = silent.slices
          .at(0,)
          ?.ballots
          .at(0,);
        if (ballot === undefined)
          throw new Error('reader gave back a slice with no ballots',);
        expect(Object.hasOwn(
          ballot,
          'archive',
        ),).toBe(false,);
      },
    },),
    it({
      name:
        'REFUSES an archive verdict this version does not describe, which the lane choice beside it '
          + 'cannot catch: the two fields are read against different lists, and a judge answering the '
          + 'archive question with a lane name would otherwise be recorded as having judged it',
      fn: async () => {
        /**
         What archiveVerdictUnknown raised, read for its class as well as its wording.
         */
        const refusalOfArchiveVerdictUnknown = caught(function archiveVerdictUnknown() {
          parseLaneSelection({
            value: {
              kind: 'contested',
              slices: [
                {
                  sliceIndex: 0,
                  verdict: { kind: 'quorum-not-met', },
                  ballots: [{
                    ...FOR_REPAIR,
                    archive: 'repair',
                  },],
                  usable: 1,
                },
              ],
            },
            comparison: ONE_CONTESTED,
            path: SELECTION_PATH,
            keys: SLICE_SPELLED_KEYS,
          },);
        },);

        expect(refusalOfArchiveVerdictUnknown,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfArchiveVerdictUnknown as Error).message,).toContain('one of publishable, flawed',);
      },
    },),
    it({
      name: 'REFUSES a kind this version does not describe, rather than reading it as pending',
      fn: async () => {
        /**
         What parseLaneSelection raised, read for its class as well as its wording.
         */
        const refusalOfParseLaneSelection = caught(() => {
          parseLaneSelection({
            value: { kind: 'shipped-repair', },
            comparison: MIXED,
            path: SELECTION_PATH,
            keys: SLICE_SPELLED_KEYS,
          },);
        },);

        expect(refusalOfParseLaneSelection,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfParseLaneSelection as Error).message,).toContain(
          `at ${SELECTION_PATH}.kind: expected one of pending-human-decision, contested.`,
        );
      },
    },),
    it({
      name:
        'REFUSES a field beside the pending kind, since a pending selection carrying a decision is '
          + 'two answers at once',
      fn: async () => {
        expect(() => {
          parseLaneSelection({
            value: {
              kind: 'pending-human-decision',
              slices: [],
            },
            comparison: MIXED,
            path: SELECTION_PATH,
            keys: SLICE_SPELLED_KEYS,
          },);
        },).toThrow(`at ${SELECTION_PATH}.slices: expected no key here beyond kind.`,);
      },
    },),
    it({
      name: 'REFUSES A CONTESTED SLICE the recomputed comparison does not hold, since no lane row can say what '
        + 'its ballots chose between',
      fn: async () => {
        /**
         The read of a contest answering slice 5 of a one-slice comparison.
         */
        const read = () =>
          parseLaneSelection({
            value: {
              kind: 'contested',
              slices: [{
                sliceIndex: 5,
                verdict: { kind: 'quorum-not-met', },
                ballots: [FOR_REPAIR,],
                usable: 1,
              },],
            },
            comparison: ONE_CONTESTED,
            path: SELECTION_PATH,
            keys: SLICE_SPELLED_KEYS,
          },);
        expect(read,).toThrow(ArtifactParseError,);
        expect(read,).toThrow(
          `at ${SELECTION_PATH}.slices[0].sliceIndex: expected an index naming a slice the recomputed comparison `
            + 'holds.',
        );
      },
    },),
    it({
      name: 'ACCEPTS A SETTLED REFUSAL NAMING WHAT THE ROSTER MADE OF THE ARCHIVE, endorsed where enough voices '
        + 'would publish it and declined where enough found it flawed',
      fn: async () => {
        for (const [judged, archive,] of [
          ['publishable', 'endorsed',],
          ['flawed', 'declined',],
        ] as const) {
          /**
           Selection whose two refusing ballots judged the archive alike.
           */
          const selection = parseLaneSelection({
            value: archiveContest({
              judged,
              verdict: {
                kind: 'settled-neither',
                archive,
              },
            },),
            comparison: ONE_CONTESTED,
            path: SELECTION_PATH,
            keys: SLICE_SPELLED_KEYS,
          },);
          if (selection.kind !== 'contested')
            throw new Error('reader returned a pending selection for a contested one',);
          expect(selection.slices
            .at(0,)
            ?.verdict,).toEqual({
              kind: 'settled-neither',
              archive,
            },);
        }
      },
    },),
    it({
      name: 'REFUSES A RECORDED ARCHIVE OUTCOME the ballots do not settle on: one left out, one reversed, one this '
        + 'version does not describe, and one beside a kind that carries none',
      fn: async () => {
        for (const [verdict, says,] of [
          [
            { kind: 'settled-neither', },
            '.verdict: expected settled-neither:endorsed, which is what these ballots settle on, rather than '
              + 'settled-neither.',
          ],
          [
            {
              kind: 'settled-neither',
              archive: 'declined',
            },
            '.verdict: expected settled-neither:endorsed, which is what these ballots settle on, rather than '
              + 'settled-neither:declined.',
          ],
          [
            {
              kind: 'settled-neither',
              archive: 'adored',
            },
            '.verdict.archive: expected one of endorsed, declined.',
          ],
          [
            {
              kind: 'quorum-not-met',
              archive: 'endorsed',
            },
            '.verdict.kind: expected one of settled-neither.',
          ],
        ] as const) {
          /**
           The read, repeated for each check.
           */
          const read = () =>
            parseLaneSelection({
              value: archiveContest({
                judged: 'publishable',
                verdict,
              },),
              comparison: ONE_CONTESTED,
              path: SELECTION_PATH,
              keys: SLICE_SPELLED_KEYS,
            },);
          expect(read,).toThrow(ArtifactParseError,);
          expect(read,).toThrow(`at ${SELECTION_PATH}.slices[0]${says}`,);
        }
      },
    },),
  ],
},);
