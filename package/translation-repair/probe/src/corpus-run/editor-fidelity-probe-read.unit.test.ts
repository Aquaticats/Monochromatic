/**
 Tests for the small readers the editor fidelity probe's run asks: which cards the command line seats, how big the
 sheet of a trial is, what a planned pair is called and how a reading is worded on the progress line. Fixtures are
 cat-themed invention only.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  EDITOR_DAMAGE_KINDS,
  editorDefectOnlyLine,
  editorFidelityRow,
  editorFidelityTrials,
  editorKindLines,
  editorTallyLine,
  type EditorTrialReading,
  pairName,
  readingWord,
  reportTallies,
  rowLineOf,
  seatedCardsOf,
  sheetCharsOf,
} from '../../dist/final/node/index.mjs';
import { statedRefusalMessage, } from '@monochromatic-dev/translation-repair-glossary/ts/stated-refusal-message.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';
import {
  HIGH_CARD,
  inventedReference,
  LOW_CARD,
} from '../editor-fidelity.test-fixture.ts';

/**
 The invented reference's trials.
 */
const TRIALS = editorFidelityTrials({
  references: [inventedReference(),],
  damageKinds: EDITOR_DAMAGE_KINDS,
},);

/**
 The first trial.
 */
const TRIAL = nonNullishOrThrow(TRIALS[0],);

/**
 Reads the cards a typed command line seats.

 @param typed - what the operator wrote after the script path

 @returns Cards, once each

 @throws StatedRefusalError when the reader refuses what was typed

 @example
 ```ts
 const cards = cardsTyped({ typed: ['--candidates', LOW_CARD, '--candidates-alone',], },);
 ```
 */
function cardsTyped({ typed, }: { readonly typed: readonly string[]; },): readonly string[] {
  return seatedCardsOf({
    line: lineOf({
      command: 'editor-fidelity-probe',
      typed,
    },),
  },);
}

await describe({
  name: '',
  children: [
    describe({
      name: seatedCardsOf.name,
      children: [
        it({
          name: 'SEATS THE NAMED CARDS ONCE EACH, in the order named, when --candidates-alone says so',
          fn: async () => {
            expect(cardsTyped({
              typed: ['--candidates', `${HIGH_CARD},${LOW_CARD},${HIGH_CARD}`, '--candidates-alone',],
            },),).toStrictEqual([HIGH_CARD, LOW_CARD,],);
          },
        },),
        it({
          name: 'REFUSES A RUN THAT DOES NOT SAY IT SEATS THE CARDS ALONE, since a roster seated beside them would '
            + 'spend on more than was asked',
          fn: async () => {
            expect(statedRefusalMessage({
              read: function seatsBeside(): void {
                cardsTyped({ typed: ['--candidates', LOW_CARD,], },);
              },
            },),).toBe(
              'this probe seats only the cards --candidates names; write --candidates-alone to say so',
            );
          },
        },),
        it({
          name: 'REFUSES A CARD THE ROSTER DOES NOT KNOW, naming it',
          fn: async () => {
            expect(statedRefusalMessage({
              read: function seatsStranger(): void {
                cardsTyped({ typed: ['--candidates', 'purr-9000', '--candidates-alone',], },);
              },
            },),).toContain('"purr-9000", which is not seatable',);
          },
        },),
        it({
          name: 'REFUSES A SEAT WITH NO CARD NAMED, in the words the judge probe uses for it',
          fn: async () => {
            expect(statedRefusalMessage({
              read: function seatsNobody(): void {
                cardsTyped({ typed: ['--candidates-alone',], },);
              },
            },),).toContain('named none',);
          },
        },),
        it({
          name: 'REFUSES A CARD WITH NO PRICE TABLE, since its ceiling cannot be printed',
          fn: async () => {
            expect(statedRefusalMessage({
              read: function seatsUnpriced(): void {
                cardsTyped({ typed: ['--candidates', 'hf:zai-org/GLM-5.3-Flash', '--candidates-alone',], },);
              },
            },),).toContain('has no price table',);
          },
        },),
      ],
    },),

    describe({
      name: sheetCharsOf.name,
      children: [
        it({
          name: 'COUNTS THE CHARACTERS OF EVERY MESSAGE the editor is sent, so the ceiling sizes the prompt by what '
            + 'goes out and not by the slice alone',
          fn: async () => {
            /**
             Characters of the sheet.
             */
            const chars = sheetCharsOf({ trial: TRIAL, },);
            expect(chars,).toBeGreaterThan(TRIAL.damagedText.length,);
          },
        },),
      ],
    },),

    describe({
      name: pairName.name,
      children: [
        it({
          name: 'NAMES A PAIR BY ITS CARD AND TRIAL HANDLES, with no corpus text in the name',
          fn: async () => {
            expect(pairName({
              item: {
                cardId: LOW_CARD,
                trial: TRIAL,
              },
            },),).toBe(`${LOW_CARD} on ${TRIAL.trialId}`,);
          },
        },),
      ],
    },),

    describe({
      name: rowLineOf.name,
      children: [
        it({
          name: 'WRITES THE ROW LINE with the row\'s place, the card and trial handles, how the trial read, the entry it '
            + 'comes from and how its asks ended, in that order, so a reader can group trials by entry',
          fn: async () => {
            /**
             A trial that read as a clean fix, asked once and heard once.
             */
            const reading: EditorTrialReading = {
              kind: 'answered',
              score: { fixed: 'exact', collateral: false, },
              applied: 1,
              rejected: 0,
              findings: [],
            };
            expect(rowLineOf({
              position: 4,
              total: 88,
              pair: {
                cardId: LOW_CARD,
                trial: TRIAL,
              },
              reading,
              stops: ['ok',],
            },),).toBe(
              `row 5 of 88: ${LOW_CARD} on ${TRIAL.trialId}: clean fix exact, nothing else touched; entry ${
                TRIAL.spec.entryId
              }; stopped ok`,
            );
            expect(rowLineOf({
              position: 0,
              total: 2,
              pair: {
                cardId: LOW_CARD,
                trial: TRIAL,
              },
              reading,
              stops: ['refused', 'ok',],
            },).endsWith(`; entry ${TRIAL.spec.entryId}; stopped refused+ok`,),).toBe(true,);
            // A trial asked again in a later pass names the pass, counted from one, after the entry.
            expect(rowLineOf({
              position: 90,
              total: 176,
              pair: {
                cardId: LOW_CARD,
                trial: TRIAL,
                pass: 1,
              },
              reading,
              stops: ['ok',],
            },),).toBe(
              `row 91 of 176: ${LOW_CARD} on ${TRIAL.trialId}: clean fix exact, nothing else touched; entry ${
                TRIAL.spec.entryId
              }; pass 2; stopped ok`,
            );
          },
        },),
      ],
    },),

    describe({
      name: reportTallies.name,
      children: [
        it({
          name: 'SAYS EACH CARD\'S TALLY, KIND LINES AND DEFECT-ONLY LINE in the order the cards were named, and '
            + 'returns the tallies it said',
          fn: async () => {
            /**
             One refused row for each card on the first trial.
             */
            const rows = [HIGH_CARD, LOW_CARD,].map(function rowOf(cardId,) {
              return editorFidelityRow({
                cardId,
                trial: TRIAL,
                reading: { kind: 'refused', findings: [], },
              },);
            },);
            /**
             Lines said, in order.
             */
            const said: string[] = [];
            /**
             The tallies returned.
             */
            const tallies = reportTallies({
              cardIds: [HIGH_CARD, LOW_CARD,],
              rows,
              emit: function note(line,): void {
                said.push(line,);
              },
            },);
            expect(tallies.map(function cardOf({ cardId, },) {
              return cardId;
            },),).toStrictEqual([HIGH_CARD, LOW_CARD,],);
            expect(said,).toStrictEqual(tallies.flatMap(function linesOf(tally,) {
              return [
                editorTallyLine({ tally, },),
                ...editorKindLines({ tally, },),
                editorDefectOnlyLine({ tally, },),
              ];
            },),);
          },
        },),
      ],
    },),

    describe({
      name: readingWord.name,
      children: [
        it({
          name: 'SAYS EACH KIND OF READING in a phrase of its own',
          fn: async () => {
            /**
             The readings and the phrase each earns.
             */
            const cases: readonly (readonly [EditorTrialReading, string])[] = [
              [{ kind: 'refused', findings: [], }, 'refused, no edit proposed',],
              [{ kind: 'unusable', cause: 'no-reply', rejected: 0, findings: [], }, 'unusable (no-reply)',],
              [{ kind: 'unusable', cause: 'edits-refused', rejected: 2, findings: [], }, 'unusable (edits-refused)',],
              [
                {
                  kind: 'answered',
                  score: { fixed: 'exact', collateral: false, },
                  applied: 1,
                  rejected: 0,
                  findings: [],
                },
                'clean fix exact, nothing else touched',
              ],
              [
                {
                  kind: 'answered',
                  score: { fixed: 'defect-only', collateral: true, },
                  applied: 1,
                  rejected: 0,
                  findings: [],
                },
                'defect only, not a clean fix, collateral',
              ],
              [
                {
                  kind: 'answered',
                  score: { fixed: 'none', collateral: true, },
                  applied: 1,
                  rejected: 0,
                  findings: [],
                },
                'not fixed, collateral',
              ],
            ];
            expect(cases.map(function worded([reading,],): string {
              return readingWord({ reading, },);
            },),).toStrictEqual(cases.map(function phrase([, word,],): string {
              return word;
            },),);
          },
        },),
      ],
    },),
  ],
},);
