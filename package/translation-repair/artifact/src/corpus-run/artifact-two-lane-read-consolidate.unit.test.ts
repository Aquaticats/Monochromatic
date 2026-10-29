/**
 Tests for reading what the third rendering settled over one document.

 THE SHIPPED FIELD IS WHAT THESE ARE ABOUT. Every other field an artifact
 carries is evidence about a decision; `shipped` is the decision's OUTPUT, and
 a consumer writes its text into the document. So the two directions of
 disagreement between it and the terminal each get a case: a record claiming
 text on a terminal that changed nothing would ship a passage nobody settled
 on, and one claiming no change on the consolidated terminal would silently
 drop a passage the roster did settle on.

 THE ABSENCE HAS TWO MEANINGS and both are pinned. Every artifact settled
 before this field existed carries no key at all, and a reader that read that
 as "the pass declined to ask" would count the whole earlier archive as
 declines. Measured before these were written: six settled artifacts from four
 earlier runs all read, all six report `unrecorded`, none refused.

 THE CONTEST BOUNDS A SETTLED STAGE. The driver writes one record at every
 slice the contest settled, walking the comparison rows, so a settled stage is
 held to exactly the contest's slices in its order, the way the contest is held
 to the comparison. Measured before those cases were written: all 28 artifacts
 on this machine carrying the field name exactly their contest's slices in its
 order, one of them naming none.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  ArtifactContestSlice,
  ArtifactLaneSelection,
} from '@monochromatic-dev/translation-repair-lane-contest';
import {
  type ArtifactConsolidateSlice,
  parseConsolidation,
  SLICE_SPELLED_KEYS,
  withShippedTenseCandidates,
} from '../../dist/final/node/index.mjs';
import { tenseLabelsOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/tense-sheet.test-fixture.ts';

/**
 Path every case reports its refusals under.
 */
const AT = 'whiskers.consolidation';

/**
 How a refusal under that path reads once `readingOf` has caught it, so each
 case checks the whole message: the class, the path and the reason.
 */
const REFUSED_AT = `ArtifactParseError: artifact parse failed at ${AT}`;

/**
 One slice as the driver records a consolidation that shipped.
 */
const SHIPPED_SLICE = {
  sliceIndex: 0,
  terminal: 'consolidated',
  shipped: {
    kind: 'consolidated',
    text: 'The cat naps in the window.\nShe wakes at four.',
  },
  rewrapped: true,
  demoted: false,
  verdicts: [
    {
      modelId: 'hf:cat/Cat-A',
      kind: 'valid',
      findings: [],
    },
  ],
  gate: {
    kind: 'asked',
    ballots: [
      {
        choice: 'consolidated',
        unsupported: [],
        unsupportedRaw: [],
        dropped: ['standing',],
        droppedRaw: ['the standing text drops the hour she wakes',],
        reason: 'the consolidation keeps both',
      },
    ],
    usable: 1,
  },
};

/**
 One slice as the driver records an incumbent that stood in for an
 ineligible lane standing and was kept by the gate (owner, 2026-09-09).
 */
const INCUMBENT_SLICE = {
  sliceIndex: 2,
  terminal: 'gate-kept-standing',
  shipped: {
    kind: 'incumbent',
    text: 'The cat naps in the window.',
  },
  rewrapped: false,
  demoted: false,
  verdicts: [],
  gate: { kind: 'not-asked', },
};

/**
 One slice as the driver records a slate the floor refused.
 */
const FLOORED_SLICE = {
  sliceIndex: 1,
  terminal: 'incumbent-only',
  shipped: { kind: 'unchanged', },
  rewrapped: false,
  demoted: false,
  verdicts: [],
  gate: { kind: 'not-asked', },
};

/**
 Tense candidates a consolidation slice records for the wording it ships.
 */
const TENSE_CANDIDATES = [
  {
    label: 'CANDIDATE "consolidated"',
    found: [
      {
        sentence: 'She naps in the window.',
        matched: 'She naps',
        rule: 's-ending-verb',
      },
    ],
  },
];

/**
 Tense candidates a consolidation slice records for an incumbent that stood in, which the sheets label as the archive's.
 */
const ARCHIVE_TENSE_CANDIDATES = [
  {
    label: 'ARCHIVE RENDERING',
    found: TENSE_CANDIDATES[0]?.found ?? [],
  },
];

/**
 Contest that settled exactly these slices, in this order, each on the
 verdict that ships nothing so the fixture carries no lane text.

 @param sliceIndexes - slices the contest answered

 @returns Selection as the pass records it

 @example
 ```ts
 const selection = contestOf({ sliceIndexes: [0, 1,], },);
 ```
 */
function contestOf(
  { sliceIndexes, }: { readonly sliceIndexes: readonly number[]; },
): ArtifactLaneSelection {
  return {
    kind: 'contested',
    slices: sliceIndexes.map(function settle(sliceIndex,): ArtifactContestSlice {
      return {
        sliceIndex,
        verdict: { kind: 'settled-neither', },
        ballots: [],
        usable: 0,
      };
    },),
  };
}

/**
 Contest that settled the two slices the fixtures answer, in that order.
 */
const CONTEST_OF_TWO = contestOf({
  sliceIndexes: [
    0,
    1,
  ],
},);

/**
 Reads a consolidation field, reporting the refusal rather than raising it.

 @param value - field as an artifact would carry it

 @param laneSelection - contest recorded beside it, the two-slice one unless
 a case says otherwise

 @param tenseRecorded - whether the generation under test records tense candidates

 @param benchFloorRecorded - whether the generation under test records a slice kept for want of a bench

 @param translateWinnerRecorded - whether the generation under test records why the archive keeps a slice and the
 translate lane winner

 @returns What was parsed, or the refusal text

 @example
 ```ts
 const read = readingOf({ value, },);
 ```
 */
function readingOf(
  {
    value,
    laneSelection = CONTEST_OF_TWO,
    tenseRecorded = false,
    benchFloorRecorded = false,
    translateWinnerRecorded = false,
  }: {
    readonly value: unknown;
    readonly laneSelection?: ArtifactLaneSelection;
    readonly tenseRecorded?: boolean;
    readonly benchFloorRecorded?: boolean;
    readonly translateWinnerRecorded?: boolean;
  },
): {
  readonly kind: string;
  readonly reason: string;
} {
  try {
    return {
      kind: parseConsolidation({
        value,
        laneSelection,
        path: AT,
        keys: SLICE_SPELLED_KEYS,
        ...(tenseRecorded ? { tenseRecorded, } : {}),
        ...(benchFloorRecorded ? { benchFloorRecorded, } : {}),
        ...(translateWinnerRecorded ? { translateWinnerRecorded, } : {}),
      },).kind,
      reason: '',
    };
  }
  catch (error: unknown) {
    return {
      kind: 'refused',
      reason: String(error,),
    };
  }
}

await describe({
  name: parseConsolidation.name,
  children: [
    it({
      name: 'READS AN ABSENT FIELD AS UNRECORDED rather than as a pass that declined to ask. Every '
        + 'artifact settled before this field existed carries no key at all, and reading that as a '
        + 'decline would count the whole earlier archive as one. Six real artifacts from four earlier '
        + 'runs were read this way before this case was written',
      fn: async () => {
        expect(readingOf({ value: undefined, },).kind,).toBe('unrecorded',);
      },
    },),

    it({
      name: 'KEEPS A STATED DECLINE APART FROM THAT ABSENCE, so a pass that ran only the two lanes is '
        + 'a decision somebody recorded rather than a field nobody wrote',
      fn: async () => {
        expect(readingOf({ value: { kind: 'not-run', }, },).kind,).toBe('not-run',);
      },
    },),

    it({
      name: 'READS A SETTLED STAGE with a consolidation that shipped and a slate the floor refused, '
        + 'which is the positive control: a parser refusing everything would pass every case of this file '
        + 'that expects a refusal '
        + 'while making the field unreadable',
      fn: async () => {
        const read = readingOf({
          value: {
            kind: 'settled',
            slices: [
              SHIPPED_SLICE,
              FLOORED_SLICE,
            ],
          },
        },);

        expect(read.reason,).toBe('',);
        expect(read.kind,).toBe('settled',);
      },
    },),

    it({
      name: 'READS A SLICE UNDER THE RETIRED TERMINAL, which 11 rows across four settled entries '
        + 'carry. The name covered three states that are now spelled apart, and none of those rows '
        + 'can be told apart after the fact because no artifact recorded the judged round beside '
        + 'the terminal. Refusing the spelling would make those entries unreadable to buy a name',
      fn: async () => {
        const read = readingOf({
          value: {
            kind: 'settled',
            slices: [
              {
                ...FLOORED_SLICE,
                terminal: 'slate-kept-standing',
              },
            ],
          },
          laneSelection: contestOf({ sliceIndexes: [1,], },),
        },);

        expect(read.reason,).toBe('',);
        expect(read.kind,).toBe('settled',);
      },
    },),

    it({
      name: 'READS AN INCUMBENT THAT STOOD IN under a kept-standing terminal as text to ship, and '
        + 'refuses it under a consolidated terminal, which ships its consolidation (owner, 2026-09-09)',
      fn: async () => {
        const read = readingOf({
          value: {
            kind: 'settled',
            slices: [INCUMBENT_SLICE,],
          },
          laneSelection: contestOf({ sliceIndexes: [2,], },),
        },);
        expect(read.reason,).toBe('',);
        expect(read.kind,).toBe('settled',);

        const refused = readingOf({
          value: {
            kind: 'settled',
            slices: [
              {
                ...INCUMBENT_SLICE,
                terminal: 'consolidated',
              },
            ],
          },
          laneSelection: contestOf({ sliceIndexes: [2,], },),
        },);
        expect(refused.reason,).toBe(
          `${REFUSED_AT}.slices[0].shipped: expected the consolidation a slice whose terminal is consolidated `
            + 'ships, rather than the incumbent.',
        );
      },
    },),

    it({
      name: 'READS THE ARCHIVE KEPT where no wording passed the rule, under a terminal that kept what '
        + 'stood, and refuses it under a consolidated terminal or carrying text of its own: the archive '
        + 'it names is the comparison row\'s, so text here would be a second copy that could disagree '
        + '(owner, 2026-09-27, "Keep archive, ship")',
      fn: async () => {
        /**
         One slice as the driver records it.
         */
        const archiveSlice = {
          ...FLOORED_SLICE,
          shipped: { kind: 'archive', },
        };
        const read = readingOf({
          value: {
            kind: 'settled',
            slices: [archiveSlice,],
          },
          laneSelection: contestOf({ sliceIndexes: [1,], },),
        },);
        expect(read.reason,).toBe('',);
        expect(read.kind,).toBe('settled',);

        const consolidated = readingOf({
          value: {
            kind: 'settled',
            slices: [
              {
                ...archiveSlice,
                terminal: 'consolidated',
              },
            ],
          },
          laneSelection: contestOf({ sliceIndexes: [1,], },),
        },);
        expect(consolidated.reason,).toBe(
          `${REFUSED_AT}.slices[0].shipped: expected the consolidation a slice whose terminal is consolidated `
            + 'ships, rather than the archive.',
        );

        const carryingText = readingOf({
          value: {
            kind: 'settled',
            slices: [
              {
                ...archiveSlice,
                shipped: {
                  kind: 'archive',
                  text: 'The cat naps in the window.',
                },
              },
            ],
          },
          laneSelection: contestOf({ sliceIndexes: [1,], },),
        },);
        expect(carryingText.reason,).toBe(`${REFUSED_AT}.slices[0].shipped.text: expected no key here beyond kind.`,);
      },
    },),

    it({
      name: 'READS WHY THE ARCHIVE KEEPS A SLICE AND THE TRANSLATE LANE WINNER from generation twenty-two, requires the '
        + 'reason there, and refuses the reason and the kind in a generation that never recorded them (owner, '
        + '2026-10-29, "Ship the two-ballot text")',
      fn: async () => {
        /**
         Slice that kept the archive, with the reason it names.
         */
        const kept = {
          ...FLOORED_SLICE,
          terminal: 'slate-declined-standing',
          shipped: {
            kind: 'archive',
            reason: 'final-round-unseated',
          },
        };
        /**
         Slice that shipped the translate lane winner.
         */
        const winner = {
          ...FLOORED_SLICE,
          sliceIndex: 0,
          shipped: {
            kind: 'translate-winner',
            text: 'The cat naps on the sill.',
          },
        };
        const contest = contestOf({
          sliceIndexes: [
            0,
            1,
          ],
        },);
        expect(readingOf({
          value: {
            kind: 'settled',
            slices: [
              winner,
              kept,
            ],
          },
          laneSelection: contest,
          translateWinnerRecorded: true,
        },),).toStrictEqual({
          kind: 'settled',
          reason: '',
        },);
        // A reason outside the three, and an archive that names none, are refused where the generation records it.
        expect(readingOf({
          value: {
            kind: 'settled',
            slices: [
              winner,
              {
                ...kept,
                shipped: {
                  kind: 'archive',
                  reason: 'because',
                },
              },
            ],
          },
          laneSelection: contest,
          translateWinnerRecorded: true,
        },).reason,).toBe(
          `${REFUSED_AT}.slices[1].shipped.reason: expected one of no-wording-passed, final-round-unseated, `
            + 'slate-declined.',
        );
        expect(readingOf({
          value: {
            kind: 'settled',
            slices: [
              winner,
              {
                ...kept,
                shipped: { kind: 'archive', },
              },
            ],
          },
          laneSelection: contest,
          translateWinnerRecorded: true,
        },).reason,).toBe(
          `${REFUSED_AT}.slices[1].shipped.reason: expected one of no-wording-passed, final-round-unseated, `
            + 'slate-declined.',
        );
        // The winner replaces no consolidation, so a consolidated terminal refuses it.
        expect(readingOf({
          value: {
            kind: 'settled',
            slices: [
              {
                ...winner,
                terminal: 'consolidated',
              },
              kept,
            ],
          },
          laneSelection: contest,
          translateWinnerRecorded: true,
        },).reason,).toBe(
          `${REFUSED_AT}.slices[0].shipped: expected the consolidation a slice whose terminal is consolidated ships, `
            + 'rather than the translate lane winner.',
        );
        // An older generation records neither the kind nor the key.
        expect(readingOf({
          value: {
            kind: 'settled',
            slices: [
              winner,
              {
                ...kept,
                shipped: { kind: 'archive', },
              },
            ],
          },
          laneSelection: contest,
        },).reason,).toBe(
          `${REFUSED_AT}.slices[0].shipped.kind: expected one of archive, consolidated, incumbent, unchanged.`,
        );
        expect(readingOf({
          value: {
            kind: 'settled',
            slices: [
              {
                ...FLOORED_SLICE,
                sliceIndex: 0,
                shipped: { kind: 'unchanged', },
              },
              kept,
            ],
          },
          laneSelection: contest,
        },).reason,).toBe(`${REFUSED_AT}.slices[1].shipped.reason: expected no key here beyond kind.`,);
      },
    },),

    it({
      name: 'REFUSES TEXT FROM A SLICE THAT SETTLED ON NO CHANGE, which would write a passage into '
        + 'the document that no round decided on. The terminal and the shipped kind answer the same '
        + 'question, so a record disagreeing with itself about it is not a record of anything',
      fn: async () => {
        const read = readingOf({
          value: {
            kind: 'settled',
            slices: [
              {
                ...SHIPPED_SLICE,
                terminal: 'gate-kept-standing',
              },
            ],
          },
        },);

        expect(read.reason,).toBe(
          `${REFUSED_AT}.slices[0].shipped: expected no text to ship, since a slice whose terminal is `
            + 'gate-kept-standing settled on no change.',
        );
      },
    },),

    it({
      name: 'REFUSES A CONSOLIDATED SLICE CARRYING NO TEXT, which is the same disagreement the other '
        + 'way round and the more dangerous one: the roster settled on a rendering and the record '
        + 'offers nothing to write, so the passage silently keeps wording the gate rejected',
      fn: async () => {
        const read = readingOf({
          value: {
            kind: 'settled',
            slices: [
              {
                ...SHIPPED_SLICE,
                shipped: { kind: 'unchanged', },
              },
            ],
          },
        },);

        expect(read.reason,).toBe(
          `${REFUSED_AT}.slices[0].shipped: expected the text a slice whose terminal is consolidated ships.`,
        );
      },
    },),

    it({
      name: 'REFUSES A TERMINAL THIS VERSION DOES NOT NAME, since the terminal is what a census '
        + 'counts and what decides whether the slice ships, so one nothing can read makes both '
        + 'unanswerable',
      fn: async () => {
        const read = readingOf({
          value: {
            kind: 'settled',
            slices: [
              {
                ...FLOORED_SLICE,
                terminal: 'kept-it-i-guess',
              },
            ],
          },
        },);

        expect(read.reason,).toBe(
          `${REFUSED_AT}.slices[0].terminal: expected one of incumbent-only, no-standing-text, `
            + 'slate-endorsed-standing, slate-unjudged-standing, slate-declined-standing, slate-kept-standing, '
            + 'gate-kept-standing, wrap-erased-difference, consolidated.',
        );
      },
    },),

    it({
      name: 'REFUSES TWO RECORDS NAMING ONE SLICE, following the contest reader for the reason it '
        + 'gives: the driver writes one record per contested slice, so two are two answers to one '
        + 'question, and a consumer keying by sliceIndex would keep whichever it read last',
      fn: async () => {
        const read = readingOf({
          value: {
            kind: 'settled',
            slices: [
              FLOORED_SLICE,
              FLOORED_SLICE,
            ],
          },
        },);

        expect(read.reason,).toBe(`${REFUSED_AT}.slices: expected one record per slice; slice 1 appears more than once.`,);
      },
    },),

    it({
      name: 'REFUSES A GATE WHOSE USABLE COUNT DISAGREES WITH ITS BALLOTS, mirroring the contest '
        + 'reader: that count is what the quorum reads, so a record claiming six voices behind one '
        + 'ballot describes a panel that never sat',
      fn: async () => {
        const read = readingOf({
          value: {
            kind: 'settled',
            slices: [
              {
                ...SHIPPED_SLICE,
                gate: {
                  ...SHIPPED_SLICE.gate,
                  usable: 6,
                },
              },
            ],
          },
        },);

        expect(read.reason,).toBe(`${REFUSED_AT}.slices[0].gate.usable: expected 1 matching stored ballot.`,);
      },
    },),

    it({
      name: 'REFUSES A BALLOT NAMING A RENDERING THAT DOES NOT EXIST, because the evidence fields are '
        + 'read as choices rather than as prose. The gate was once found shipping a rendering its own '
        + 'ballots named faultier because nothing counted them, and a name outside the three would be '
        + 'counted as nothing and weaken that evidence silently',
      fn: async () => {
        const read = readingOf({
          value: {
            kind: 'settled',
            slices: [
              {
                ...SHIPPED_SLICE,
                gate: {
                  kind: 'asked',
                  usable: 1,
                  ballots: [
                    {
                      ...SHIPPED_SLICE.gate.ballots[0],
                      dropped: ['the other one',],
                    },
                  ],
                },
              },
            ],
          },
        },);

        expect(read.reason,).toBe(
          `${REFUSED_AT}.slices[0].gate.ballots[0].dropped[0]: expected one of consolidated, standing, neither.`,
        );
      },
    },),

    it({
      name: 'READS THE TENSE CANDIDATES A SLICE RECORDS for the wording it ships, in a generation that records them, '
        + 'a consolidation and an incumbent that stood in alike',
      fn: async () => {
        /**
         The settled stage as parsed, with the finding on both slices that ship wording.
         */
        const parsed = parseConsolidation({
          value: {
            kind: 'settled',
            slices: [
              {
                ...SHIPPED_SLICE,
                tenseCandidates: TENSE_CANDIDATES,
              },
              {
                ...INCUMBENT_SLICE,
                sliceIndex: 1,
                tenseCandidates: ARCHIVE_TENSE_CANDIDATES,
              },
            ],
          },
          laneSelection: CONTEST_OF_TWO,
          path: AT,
          keys: SLICE_SPELLED_KEYS,
          tenseRecorded: true,
        },);
        if (parsed.kind !== 'settled')
          throw new Error('the settled stage was read as another kind',);
        expect(parsed.slices.map(function findingOf(slice,) {
          return slice.tenseCandidates;
        },),).toEqual([
          TENSE_CANDIDATES,
          ARCHIVE_TENSE_CANDIDATES,
        ],);
      },
    },),

    it({
      name: 'READS WHAT THE WRITER RECORDS for each wording a slice ships or leaves, a consolidation, an incumbent that '
        + 'stood in, the archive kept and nothing changed, the finding on the first two alone',
      fn: async () => {
        /**
         The four slices as the driver's records stand, each with the text a sentence of which is a candidate.
         */
        const driven: readonly ArtifactConsolidateSlice[] = [
          {
            sliceIndex: 0,
            terminal: 'consolidated',
            shipped: {
              kind: 'consolidated',
              text: 'Mittens naps in the window.',
            },
            rewrapped: false,
            demoted: false,
            verdicts: [],
            gate: { kind: 'not-asked', },
          },
          {
            sliceIndex: 1,
            terminal: 'gate-kept-standing',
            shipped: {
              kind: 'incumbent',
              text: 'Mittens naps in the window.',
            },
            rewrapped: false,
            demoted: false,
            verdicts: [],
            gate: { kind: 'not-asked', },
          },
          {
            sliceIndex: 2,
            terminal: 'incumbent-only',
            shipped: { kind: 'archive', },
            rewrapped: false,
            demoted: false,
            verdicts: [],
            gate: { kind: 'not-asked', },
          },
          {
            sliceIndex: 3,
            terminal: 'incumbent-only',
            shipped: { kind: 'unchanged', },
            rewrapped: false,
            demoted: false,
            verdicts: [],
            gate: { kind: 'not-asked', },
          },
        ];
        /**
         The same slices after the writer added the finding.
         */
        const written = driven.map(function recorded(slice,) {
          return withShippedTenseCandidates({
            slice,
            lifeEnded: true,
            subjectNames: ['Mittens',],
          },);
        },);
        expect(tenseLabelsOf({ slices: written, },),).toEqual([
          ['CANDIDATE "consolidated"',],
          ['ARCHIVE RENDERING',],
          [],
          [],
        ],);
        /**
         The stage as the reader returns it from what the writer recorded.
         */
        const parsed = parseConsolidation({
          value: {
            kind: 'settled',
            slices: written,
          },
          laneSelection: contestOf({
            sliceIndexes: [
              0,
              1,
              2,
              3,
            ],
          },),
          path: AT,
          keys: SLICE_SPELLED_KEYS,
          tenseRecorded: true,
        },);
        if (parsed.kind !== 'settled')
          throw new Error('the settled stage was read as another kind',);
        expect(parsed.slices,).toEqual(written,);
      },
    },),

    it({
      name: 'REFUSES THE TENSE CANDIDATES OF A CONSOLIDATION under the label of the other shipped wording, and of an '
        + 'incumbent that stood in under the consolidation\'s',
      fn: async () => {
        const consolidated = readingOf({
          value: {
            kind: 'settled',
            slices: [
              {
                ...SHIPPED_SLICE,
                tenseCandidates: ARCHIVE_TENSE_CANDIDATES,
              },
            ],
          },
          laneSelection: contestOf({ sliceIndexes: [0,], },),
          tenseRecorded: true,
        },);
        expect(consolidated.reason,).toBe(
          `${REFUSED_AT}.slices[0].tenseCandidates[0].label: expected a label from CANDIDATE "consolidated", each once and in that order.`,
        );
        const incumbent = readingOf({
          value: {
            kind: 'settled',
            slices: [
              {
                ...INCUMBENT_SLICE,
                sliceIndex: 0,
                tenseCandidates: TENSE_CANDIDATES,
              },
            ],
          },
          laneSelection: contestOf({ sliceIndexes: [0,], },),
          tenseRecorded: true,
        },);
        expect(incumbent.reason,).toBe(
          `${REFUSED_AT}.slices[0].tenseCandidates[0].label: expected a label from ARCHIVE RENDERING, each once and in that order.`,
        );
      },
    },),

    it({
      name: 'READS A SLICE THE SLATE FLOOR KEPT apart from a slate that declined: with the mark recorded the slice '
        + 'reads back as kept for want of a bench, and the same slice with no mark is a real decline',
      fn: async () => {
        /**
         The settled stage as parsed, with the mark on one slice and none on the other.
         */
        const parsed = parseConsolidation({
          value: {
            kind: 'settled',
            slices: [
              {
                ...FLOORED_SLICE,
                sliceIndex: 0,
                benchBelowFloor: true,
              },
              FLOORED_SLICE,
            ],
          },
          laneSelection: CONTEST_OF_TWO,
          path: AT,
          keys: SLICE_SPELLED_KEYS,
          benchFloorRecorded: true,
        },);
        if (parsed.kind !== 'settled')
          throw new Error('the settled stage was read as another kind',);
        expect(parsed.slices.map(function markOf(slice,) {
          return slice.benchBelowFloor;
        },),).toEqual([
          true,
          undefined,
        ],);
      },
    },),

    it({
      name: 'REFUSES THE FLOOR MARK in a generation that does not record it, naming the keys that generation allows',
      fn: async () => {
        const read = readingOf({
          value: {
            kind: 'settled',
            slices: [
              {
                ...FLOORED_SLICE,
                benchBelowFloor: true,
              },
            ],
          },
          laneSelection: contestOf({ sliceIndexes: [1,], },),
        },);

        expect(read.reason,).toBe(
          `${REFUSED_AT}.slices[0].benchBelowFloor: expected no key here beyond sliceIndex, terminal, shipped, `
            + 'rewrapped, demoted, verdicts, gate.',
        );
      },
    },),

    it({
      name: 'REFUSES THE FLOOR MARK ON A SLICE THAT ASKED A SLATE, or set to false, since the mark says no slate was '
        + 'asked and a slice with verdicts or a gate was',
      fn: async () => {
        /**
         Refusal text for one marked slice, read in a generation that records the mark.

         @param slice - slice as recorded

         @returns Refusal
         */
        function refusalOf(slice: Readonly<Record<string, unknown>>,): string {
          return readingOf({
            value: {
              kind: 'settled',
              slices: [slice,],
            },
            laneSelection: contestOf({ sliceIndexes: [Number(slice.sliceIndex,),], },),
            benchFloorRecorded: true,
          },).reason;
        }
        expect({
          asked: refusalOf({
            ...SHIPPED_SLICE,
            benchBelowFloor: true,
          },),
          falsy: refusalOf({
            ...FLOORED_SLICE,
            benchBelowFloor: false,
          },),
        },).toEqual({
          asked: `${REFUSED_AT}.slices[0].benchBelowFloor: expected a slice that asked no slate (terminal `
            + 'incumbent-only, no verdicts, no gate), rather than terminal consolidated.',
          falsy: `${REFUSED_AT}.slices[0].benchBelowFloor: expected true or absent.`,
        },);
      },
    },),

    it({
      name: 'REFUSES THE TENSE CANDIDATES in a generation that does not record them, naming the keys that generation allows',
      fn: async () => {
        const read = readingOf({
          value: {
            kind: 'settled',
            slices: [
              {
                ...SHIPPED_SLICE,
                tenseCandidates: TENSE_CANDIDATES,
              },
            ],
          },
          laneSelection: contestOf({ sliceIndexes: [0,], },),
        },);

        expect(read.reason,).toBe(
          `${REFUSED_AT}.slices[0].tenseCandidates: expected no key here beyond sliceIndex, terminal, shipped, `
            + 'rewrapped, demoted, verdicts, gate.',
        );
      },
    },),

    it({
      name: 'REFUSES THE TENSE CANDIDATES OF A SLICE THAT SHIPS NO WORDING OF ITS OWN, the archive kept or nothing changed, '
        + 'since the contest record holds the candidates of the lanes and the archive',
      fn: async () => {
        const unchanged = readingOf({
          value: {
            kind: 'settled',
            slices: [
              {
                ...FLOORED_SLICE,
                tenseCandidates: TENSE_CANDIDATES,
              },
            ],
          },
          laneSelection: contestOf({ sliceIndexes: [1,], },),
          tenseRecorded: true,
        },);
        expect(unchanged.reason,).toBe(
          `${REFUSED_AT}.slices[0].tenseCandidates: expected no tense candidates, since a slice that ships no wording of its own has no shipped text to read them from.`,
        );
        const archive = readingOf({
          value: {
            kind: 'settled',
            slices: [
              {
                ...FLOORED_SLICE,
                shipped: { kind: 'archive', },
                tenseCandidates: TENSE_CANDIDATES,
              },
            ],
          },
          laneSelection: contestOf({ sliceIndexes: [1,], },),
          tenseRecorded: true,
        },);
        expect(archive.reason,).toBe(unchanged.reason,);
      },
    },),

    it({
      name: 'REFUSES A KEY THIS VERSION DOES NOT NAME, so a field added by a later schema and read by '
        + 'this one is a refusal rather than a silent drop, which is the property every other reader '
        + 'in this generation already holds',
      fn: async () => {
        const read = readingOf({
          value: {
            kind: 'settled',
            slices: [FLOORED_SLICE,],
            heardTranslators: 5,
          },
        },);

        expect(read.reason,).toBe(`${REFUSED_AT}.heardTranslators: expected no key here beyond kind, slices.`,);
      },
    },),

    it({
      name: 'READS A SETTLED STAGE THAT NAMES NO SLICE beside a contest that settled none, which is '
        + 'what a document whose lanes never differed records: the stage was asked and had nothing '
        + 'to answer. One artifact on this machine has exactly this shape',
      fn: async () => {
        const read = readingOf({
          value: {
            kind: 'settled',
            slices: [],
          },
          laneSelection: contestOf({ sliceIndexes: [], },),
        },);

        expect(read.reason,).toBe('',);
        expect(read.kind,).toBe('settled',);
      },
    },),

    it({
      name: 'REFUSES A SETTLED STAGE MISSING A CONTESTED SLICE, mirroring the contest reader. The '
        + 'driver writes one record at every slice the contest settled, so a consumer keying by '
        + 'sliceIndex would otherwise read the missing slice as left with what the contest settled',
      fn: async () => {
        const read = readingOf({
          value: {
            kind: 'settled',
            slices: [SHIPPED_SLICE,],
          },
        },);

        expect(read.reason,).toBe(
          `${REFUSED_AT}.slices: expected slices [0,1], one per slice the contest settled and in its order, `
            + 'rather than [0].',
        );
      },
    },),

    it({
      name: 'REFUSES A SETTLED STAGE NAMING A SLICE THE CONTEST NEVER SETTLED, since no run of this '
        + 'pipeline consolidates a slice the two lanes did not leave differently worded',
      fn: async () => {
        const read = readingOf({
          value: {
            kind: 'settled',
            slices: [
              SHIPPED_SLICE,
              FLOORED_SLICE,
            ],
          },
          laneSelection: contestOf({ sliceIndexes: [0,], },),
        },);

        expect(read.reason,).toBe(
          `${REFUSED_AT}.slices: expected slices [0], one per slice the contest settled and in its order, `
            + 'rather than [0,1].',
        );
      },
    },),

    it({
      name: 'REFUSES RECORDS OUT OF CONTEST ORDER, because the driver walks the comparison rows and '
        + 'a list in another order was not written by it; sorting it here would accept that file and '
        + 'hide which run produced it',
      fn: async () => {
        const read = readingOf({
          value: {
            kind: 'settled',
            slices: [
              FLOORED_SLICE,
              SHIPPED_SLICE,
            ],
          },
        },);

        expect(read.reason,).toBe(
          `${REFUSED_AT}.slices: expected slices [0,1], one per slice the contest settled and in its order, `
            + 'rather than [1,0].',
        );
      },
    },),

    it({
      name: 'REFUSES RECORDS BESIDE A SELECTION NOBODY CONTESTED, since the stage runs after the '
        + 'contest and answers only what it settled; a pending selection can sit beside a settled '
        + 'stage only when that stage names nothing',
      fn: async () => {
        const read = readingOf({
          value: {
            kind: 'settled',
            slices: [FLOORED_SLICE,],
          },
          laneSelection: { kind: 'pending-human-decision', },
        },);

        expect(read.reason,).toBe(
          `${REFUSED_AT}.slices: expected slices [], one per slice the contest settled and in its order, `
            + 'rather than [1].',
        );
      },
    },),

    it({
      name: 'REFUSES A STAGE KIND and a gate kind this version does not name, rather than reading either as a '
        + 'stage or a gate that did not run',
      fn: async () => {
        expect(readingOf({
          value: {
            kind: 'napping',
            slices: [],
          },
        },).reason,).toBe(`${REFUSED_AT}.kind: expected one of settled, not-run.`,);
        expect(readingOf({
          value: {
            kind: 'settled',
            slices: [
              SHIPPED_SLICE,
              {
                ...FLOORED_SLICE,
                gate: { kind: 'sleepy', },
              },
            ],
          },
        },).reason,).toBe(`${REFUSED_AT}.slices[1].gate.kind: expected one of asked, not-asked.`,);
      },
    },),
  ],
},);
