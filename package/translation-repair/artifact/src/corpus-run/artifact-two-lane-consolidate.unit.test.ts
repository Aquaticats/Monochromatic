/**
 Tests for the record one consolidated slice leaves behind.

 WHAT THESE PIN is the field that decides what reaches the reader. The stage
 can settle six different ways and exactly one of them produces wording an
 assembly should write; the other five leave the slice with whatever the lane
 contest put there. A record that carried a bare text field per slice would
 make those indistinguishable, and one of the five, `no-standing-text`,
 carries the EMPTY STRING, so an assembly reading it naively would delete
 every slice whose contest named neither lane.

 The named-absence half is therefore the half with teeth: these assert that a
 settlement which changes nothing offers NO text to write, rather than
 offering text that happens to match.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  ConsolidationSettlement,
  ConsolidationTerminal,
} from '@monochromatic-dev/translation-repair-consolidate';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  describeConsolidateSlice,
  parseConsolidationPolish,
} from '../../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Builds a settlement that left the stage the way a case needs.

 ONLY THE FIELDS THE RECORD READS are real here. The floor and the judged
 round are whole objects in production and neither is projected into the
 record, so building them would assert nothing this file is about.

 @param terminal - how the slice left the stage

 @param text - wording the settlement carries, whatever the terminal

 @returns Settlement shaped as the stage returns one

 @example
 ```ts
 const settlement = settledAs({ terminal: 'consolidated', text: 'The cat naps.', },);
 ```
 */
function settledAs(
  {
    terminal,
    text,
  }: {
    readonly terminal: ConsolidationTerminal;
    readonly text: string;
  },
): ConsolidationSettlement {
  return {
    terminal,
    text,
    floor: {
      kind: 'proposals',
      validModelIds: ['hf:cat/Cat-A',],
    },
    verdicts: [],
    rewrapped: false,
    demoted: false,
    findings: [],
  } as ConsolidationSettlement;
}

/**
 One passage as a consolidation that won would carry it.
 */
const CONSOLIDATED_TEXT = 'The cat naps in the window.\nShe wakes at four.';

/**
 Wording already in place, which several terminals hand back unchanged.
 */
const STANDING_TEXT = 'A cat sleeps by the window, and wakes in the afternoon.';

await describe({
  name: describeConsolidateSlice.name,
  children: [
    it({
      name: 'CARRIES THE WORDING TO WRITE when a consolidation won both rounds, which is the whole '
        + 'point of the stage: a third rendering that reached neither the record nor the document '
        + 'would be a round of calls bought to change nothing',
      fn: async () => {
        const slice = describeConsolidateSlice({
          sliceIndex: 4,
          settlement: settledAs({ terminal: 'consolidated', text: CONSOLIDATED_TEXT, },),
        },);

        expect(slice.sliceIndex,).toBe(4,);
        expect(slice.shipped.kind,).toBe('consolidated',);
        if (slice.shipped.kind !== 'consolidated')
          throw new Error('consolidated by construction',);
        expect(slice.shipped.text,).toBe(CONSOLIDATED_TEXT,);
      },
    },),

    it({
      name: 'OFFERS NO TEXT AT ALL FOR A DECLINED CONTEST, which is the case that would delete a '
        + 'slice: the contest named neither lane, so nothing stands, and the settlement carries the '
        + 'empty string as its text. A record with a bare text field would hand an assembly an empty '
        + 'string to write over a passage nobody decided to remove',
      fn: async () => {
        const slice = describeConsolidateSlice({
          sliceIndex: 0,
          settlement: settledAs({ terminal: 'no-standing-text', text: '', },),
        },);

        expect(slice.terminal,).toBe('no-standing-text',);
        expect(slice.shipped.kind,).toBe('unchanged',);
        expect(Object.hasOwn(slice.shipped, 'text',),).toBe(false,);
      },
    },),

    it({
      name: 'OFFERS NO TEXT FOR EVERY TERMINAL THAT KEPT WHAT STOOD, whichever round stopped it. The '
        + 'floor, the judges, the gate and the wrap all end with the standing text in hand, and a '
        + 'record that offered it back would report a decision to change nothing as a change',
      fn: async () => {
        /**
         Every way the stage can end without producing a replacement.
         */
        const keptStanding: readonly ConsolidationTerminal[] = [
          'incumbent-only',
          'slate-endorsed-standing',
          'slate-unjudged-standing',
          'slate-declined-standing',
          'gate-kept-standing',
          'wrap-erased-difference',
        ];

        for (const terminal of keptStanding) {
          const slice = describeConsolidateSlice({
            sliceIndex: 1,
            settlement: settledAs({ terminal, text: STANDING_TEXT, },),
          },);

          expect(slice.terminal,).toBe(terminal,);
          expect(slice.shipped.kind,).toBe('unchanged',);
        }
      },
    },),

    it({
      name: 'SHIPS THE ARCHIVE, CARRYING NO TEXT, where no wording passed the rule (owner, 2026-09-27, '
        + '"Keep archive, ship"), ahead of an incumbent that stood in: on a disputed slice the stand-in '
        + 'is the repair lane\'s text, which the rule refused too, and the archive is what the ruling '
        + 'keeps. The reader takes the archive from the comparison row, so a silent archive stays silent '
        + 'rather than becoming an empty string written into the page',
      fn: async () => {
        for (const incumbentStandsIn of [
          false,
          true,
        ]) {
          const slice = describeConsolidateSlice({
            sliceIndex: 5,
            settlement: {
              ...settledAs({ terminal: 'incumbent-only', text: STANDING_TEXT, },),
              archiveKept: true,
            },
            incumbentStandsIn,
          },);

          expect(slice.terminal,).toBe('incumbent-only',);
          expect(slice.shipped,).toStrictEqual({ kind: 'archive', },);
        }
      },
    },),

    it({
      name: 'RECORDS WHY THE ARCHIVE KEEPS A SLICE where the settlement says, and the translate lane winner with its '
        + 'text where the final round could not seat two judges (owner, 2026-10-29, "Ship the two-ballot text")',
      fn: async () => {
        const kept = describeConsolidateSlice({
          sliceIndex: 6,
          settlement: {
            ...settledAs({ terminal: 'slate-declined-standing', text: STANDING_TEXT, },),
            archiveKept: true,
            archiveReason: 'final-round-unseated',
          },
        },);
        expect(kept.shipped,).toStrictEqual({
          kind: 'archive',
          reason: 'final-round-unseated',
        },);
        const winner = describeConsolidateSlice({
          sliceIndex: 7,
          settlement: {
            ...settledAs({ terminal: 'slate-declined-standing', text: 'The cat naps on the sill.', },),
            translateWinnerShipped: true,
          },
        },);
        // A settlement that names no reason is the one written before the swap existed, which replaced a refused standing.
        expect(winner.shipped,).toStrictEqual({
          kind: 'translate-winner',
          text: 'The cat naps on the sill.',
          reason: 'refused-standing',
        },);
      },
    },),

    it({
      name: 'RECORDS WHY THE TRANSLATE LANE WINNER SHIPS: over a standing the rule refused, or over an unendorsed '
        + 'standing the tense detector flagged (owner, 2026-10-29, "Swap only where flagged")',
      fn: async () => {
        const swapped = describeConsolidateSlice({
          sliceIndex: 8,
          settlement: {
            ...settledAs({ terminal: 'slate-declined-standing', text: 'Mittens was a tabby.', },),
            translateWinnerShipped: true,
            translateWinnerReason: 'present-tense-standing',
          },
        },);
        expect(swapped.shipped,).toStrictEqual({
          kind: 'translate-winner',
          text: 'Mittens was a tabby.',
          reason: 'present-tense-standing',
        },);
        const refused = describeConsolidateSlice({
          sliceIndex: 9,
          settlement: {
            ...settledAs({ terminal: 'slate-declined-standing', text: 'Mittens was a tabby.', },),
            translateWinnerShipped: true,
            translateWinnerReason: 'refused-standing',
          },
        },);
        expect(refused.shipped,).toStrictEqual({
          kind: 'translate-winner',
          text: 'Mittens was a tabby.',
          reason: 'refused-standing',
        },);
      },
    },),

    it({
      name: 'READS THE TERMINAL RATHER THAN COMPARING TEXT, so a consolidation that agreed with the '
        + 'standing text everywhere but was still settled as a change is recorded as one. Deriving '
        + 'this from a string comparison would re-decide at the record what the wrap already decided, '
        + 'and the two could disagree',
      fn: async () => {
        const slice = describeConsolidateSlice({
          sliceIndex: 2,
          settlement: settledAs({ terminal: 'consolidated', text: STANDING_TEXT, },),
        },);

        expect(slice.shipped.kind,).toBe('consolidated',);
      },
    },),

    it({
      name: 'ROUND-TRIPS WRITTEN TWO-CORRECTION POLISH through JSON and schema-nine reader',
      fn: async () => {
        /** Exact reviewed candidates in bounded order. */
        const texts = [
          'The cat performs sleeping.',
          'The cat is sleeping.',
          'The cat sleeps.',
        ] as const;
        /** Located findings authorizing each correction. */
        const findings = [
          [{ paragraph: 1, problem: 'Replace nominalized verb.', },],
          [{ paragraph: 1, problem: 'Prefer concise habitual form.', },],
        ] as const;
        /** Review rounds written by runtime settlement. */
        const reviews = texts.map(function reviewOf(text, index,) {
          const rejectedFindings = findings[index] ?? [];
          const acceptable = index === (texts.length - 1);
          return {
            quorumOver: 2,
            unreachable: 0,
            candidateDigest: hashContent({ content: text, },),
            candidateText: text,
            paragraphCount: 1,
            paragraphDigests: [hashContent({ content: text, },),],
            seats: [
              {
                modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                status: acceptable ? 'acceptable' as const : 'unacceptable' as const,
                findings: rejectedFindings,
                reason: acceptable ? 'ready' : 'material defect remains',
              },
              {
                modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
                status: 'acceptable' as const,
                findings: [],
                reason: 'ready',
              },
            ],
            usable: 2,
            verdict: acceptable ? 'acceptable' as const : 'unacceptable' as const,
            findings: rejectedFindings,
          };
        },);
        /** Every review narrowed from fixed three-candidate fixture. */
        const [initialReview, firstReview, finalReview,] = reviews;
        if ((initialReview === undefined)
          || (firstReview === undefined)
          || (finalReview === undefined))
          throw new Error('fixed schema-nine review fixture lost a round',);
        /** Consolidation settlement carrying schema-nine audit chain. */
        const settlement: ConsolidationSettlement = {
          ...settledAs({ terminal: 'consolidated', text: texts[2], },),
          polish: {
            kind: 'settled',
            baseText: texts[0],
            proposedText: texts[2],
            text: texts[2],
            changed: true,
            refinersHeard: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
            contributors: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
            rounds: [],
            gate: {
              choice: 'polished',
              ships: 'polished',
              ballots: [
                {
                  choice: 'polished',
                  unsupported: [],
                  unsupportedRaw: [],
                  dropped: [],
                  droppedRaw: [],
                  reason: 'faithful correction',
                },
                {
                  choice: 'polished',
                  unsupported: [],
                  unsupportedRaw: [],
                  dropped: [],
                  droppedRaw: [],
                  reason: 'faithful correction',
                },
              ],
              usable: 2,
              findings: [],
            },
            review: {
              correctionCount: 2,
              corrections: [
                {
                  inputDigest: initialReview.candidateDigest,
                  findingsDigest: hashContent({ content: JSON.stringify(findings[0],), },),
                  gatedTextDigest: firstReview.candidateDigest,
                },
                {
                  inputDigest: firstReview.candidateDigest,
                  findingsDigest: hashContent({ content: JSON.stringify(findings[1],), },),
                  gatedTextDigest: finalReview.candidateDigest,
                },
              ],
              rounds: [
                initialReview,
                firstReview,
                finalReview,
              ],
              confirmations: [finalReview,],
            },
            findings: [],
          },
        };
        const slice = describeConsolidateSlice({ sliceIndex: 2, settlement, },);
        /** JSON bytes crossing artifact persistence boundary. */
        const json = JSON.stringify(slice.polish,);
        /** Parsed artifact polish as reader receives it. */
        const serialized: unknown = JSON.parse(json,);
        const parsed = parseConsolidationPolish({
          value: serialized,
          path: 'consolidation.slices[0].polish',
          reviewRequired: true,
          correctionChainRequired: true,
          quorumBasisRequired: true,
        },);
        expect(parsed,).toEqual(slice.polish,);
      },
    },),

    it({
      name: 'NAMES A GATE THAT WAS NEVER ASKED rather than omitting the key, because a missing field '
        + 'and a round deliberately not bought look identical in JSON, and a census of this stage is '
        + 'counting exactly that difference',
      fn: async () => {
        const slice = describeConsolidateSlice({
          sliceIndex: 3,
          settlement: settledAs({ terminal: 'incumbent-only', text: STANDING_TEXT, },),
        },);

        expect(slice.gate.kind,).toBe('not-asked',);
      },
    },),

    it({
      name: 'RECORDS A SLICE THE SLATE FLOOR KEPT with the mark, and a slate that declined without it, so a reader '
        + 'of the record alone tells a slate never bought from one the judges refused',
      fn: async () => {
        /**
         Slice as the stage settles it when the floor keeps the standing, and one the judges declined.
         */
        const records = {
          skipped: describeConsolidateSlice({
            sliceIndex: 3,
            settlement: {
              ...settledAs({ terminal: 'incumbent-only', text: STANDING_TEXT, },),
              benchBelowFloor: true,
            },
          },),
          declined: describeConsolidateSlice({
            sliceIndex: 3,
            settlement: settledAs({ terminal: 'incumbent-only', text: STANDING_TEXT, },),
          },),
        };
        expect({
          skipped: records.skipped.benchBelowFloor,
          declinedHasKey: 'benchBelowFloor' in records.declined,
        },).toEqual({
          skipped: true,
          declinedHasKey: false,
        },);
      },
    },),
  ],
},);
