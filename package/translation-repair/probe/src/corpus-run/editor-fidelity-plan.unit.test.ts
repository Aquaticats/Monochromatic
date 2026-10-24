/**
 Tests for what the editor fidelity probe writes down and says: the per-card tally of how each trial read, and the
 lines that report it. Fixtures are cat-themed invention.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  decisionRuleText,
  EDITOR_DAMAGE_KINDS,
  editorDecisionRuleLine,
  editorDefectOnlyLine,
  editorFidelityRow,
  editorFidelityTrials,
  editorKeptLine,
  editorKindLines,
  editorPreflightLine,
  type EditorFidelityRow,
  editorTallyLine,
  editorTallyOf,
} from '../../dist/final/node/index.mjs';
import {
  HIGH_CARD,
  inventedReference,
  LOW_CARD,
} from '../editor-fidelity.test-fixture.ts';

/**
 A row of the given reading, for the card.

 @param cardId - card the row belongs to

 @param reading - how the trial read

 @returns The row

 @example
 ```ts
 const row = rowOf({ cardId: LOW_CARD, reading: { kind: 'refused', findings: [], }, },);
 ```
 */
function rowOf(
  {
    cardId,
    reading,
    damageKind = 'insertion',
  }: {
    readonly cardId: string;
    readonly reading: EditorFidelityRow['reading'];
    readonly damageKind?: EditorFidelityRow['damageKind'];
  },
): EditorFidelityRow {
  return {
    cardId,
    trialId: `tabby-reference/${damageKind}`,
    referenceId: 'tabby-reference',
    entryId: 'tabby',
    damageKind,
    changedChars: 12,
    damageDetail: 'inserted a borrowed sentence of 12 characters',
    reading,
  } as EditorFidelityRow;
}

/**
 Every way a trial can read, once, for one card, and one more row for another card.
 */
const ROWS: readonly EditorFidelityRow[] = [
  rowOf({
    cardId: LOW_CARD,
    reading: {
      kind: 'answered',
      score: {
        fixed: 'exact',
        collateral: false,
      },
      applied: 1,
      rejected: 0,
      findings: [],
    },
  },),
  rowOf({
    cardId: LOW_CARD,
    damageKind: 'alteration',
    reading: {
      kind: 'answered',
      score: {
        fixed: 'equivalent',
        collateral: false,
      },
      applied: 1,
      rejected: 0,
      findings: [],
    },
  },),
  rowOf({
    cardId: LOW_CARD,
    reading: {
      kind: 'answered',
      score: {
        fixed: 'defect-only',
        collateral: true,
      },
      applied: 1,
      rejected: 0,
      findings: [],
    },
  },),
  rowOf({
    cardId: LOW_CARD,
    damageKind: 'alteration',
    reading: {
      kind: 'answered',
      score: {
        fixed: 'none',
        collateral: true,
      },
      applied: 1,
      rejected: 0,
      findings: [],
    },
  },),
  rowOf({
    cardId: LOW_CARD,
    reading: {
      kind: 'answered',
      score: {
        fixed: 'none',
        collateral: false,
      },
      applied: 1,
      rejected: 0,
      findings: [],
    },
  },),
  rowOf({
    cardId: LOW_CARD,
    damageKind: 'alteration',
    reading: {
      kind: 'refused',
      findings: [],
    },
  },),
  rowOf({
    cardId: LOW_CARD,
    reading: {
      kind: 'unusable',
      cause: 'no-reply',
      rejected: 0,
      findings: [],
    },
  },),
  rowOf({
    cardId: HIGH_CARD,
    reading: {
      kind: 'refused',
      findings: [],
    },
  },),
];

await describe({
  name: '',
  children: [
    describe({
      name: editorTallyOf.name,
      children: [
        it({
          name: 'COUNTS EACH WAY A TRIAL READS APART, for the card alone and for each kind of defect: the clean fix is exact '
            + 'plus equivalent and leaves out defect-only, collateral counts any answered trial, and missed, refused and '
            + 'unusable are out of the card\'s trials',
          fn: async () => {
            expect(editorTallyOf({
              cardId: LOW_CARD,
              rows: ROWS,
            },),).toStrictEqual({
              cardId: LOW_CARD,
              trials: 7,
              fixed: 2,
              exact: 1,
              equivalent: 1,
              defectOnly: 1,
              collateral: 2,
              missed: 2,
              refused: 1,
              unusable: 1,
              byKind: {
                insertion: {
                  trials: 4,
                  exact: 1,
                  equivalent: 0,
                  defectOnly: 1,
                  collateral: 1,
                  missed: 1,
                  refused: 0,
                  unusable: 1,
                },
                alteration: {
                  trials: 3,
                  exact: 0,
                  equivalent: 1,
                  defectOnly: 0,
                  collateral: 1,
                  missed: 1,
                  refused: 1,
                  unusable: 0,
                },
              },
            },);
            expect(editorTallyOf({
              cardId: HIGH_CARD,
              rows: ROWS,
            },),).toStrictEqual({
              cardId: HIGH_CARD,
              trials: 1,
              fixed: 0,
              exact: 0,
              equivalent: 0,
              defectOnly: 0,
              collateral: 0,
              missed: 0,
              refused: 1,
              unusable: 0,
              byKind: {
                insertion: {
                  trials: 1,
                  exact: 0,
                  equivalent: 0,
                  defectOnly: 0,
                  collateral: 0,
                  missed: 0,
                  refused: 1,
                  unusable: 0,
                },
                alteration: {
                  trials: 0,
                  exact: 0,
                  equivalent: 0,
                  defectOnly: 0,
                  collateral: 0,
                  missed: 0,
                  refused: 0,
                  unusable: 0,
                },
              },
            },);
          },
        },),

        it({
          name: 'COUNTS A CARD WITH NO ROWS as zero trials rather than failing',
          fn: async () => {
            expect(editorTallyOf({
              cardId: 'claude-haiku-5-5',
              rows: ROWS,
            },).trials,).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: editorFidelityRow.name,
      children: [
        it({
          name: 'KEEPS HOW THE ASKS OF THE TRIAL ENDED in the row, in the order asked, and an empty list for a row built '
            + 'without them',
          fn: async () => {
            /**
             The invented reference's first trial.
             */
            const trial = nonNullishOrThrow(editorFidelityTrials({
              references: [inventedReference(),],
              damageKinds: EDITOR_DAMAGE_KINDS,
            },)[0],);
            /**
             A reading to build rows of.
             */
            const reading = { kind: 'refused', findings: [], } as const;
            expect(editorFidelityRow({
              cardId: LOW_CARD,
              trial,
              reading,
              stops: ['refused', 'ok',],
            },).stops,).toStrictEqual(['refused', 'ok',],);
            expect(editorFidelityRow({ cardId: LOW_CARD, trial, reading, },).stops,).toStrictEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: editorTallyLine.name,
      children: [
        it({
          name: 'SAYS EVERY COUNT and what it is out of, in one line a log search finds, with the clean fix first and '
            + 'defect-only apart from it',
          fn: async () => {
            expect(editorTallyLine({
              tally: editorTallyOf({
                cardId: LOW_CARD,
                rows: ROWS,
              },),
            },),).toBe(
              `EDITOR ${LOW_CARD}: clean fix 2 of 7 (exact 1, equivalent 1), defect only 1, collateral 2, missed 2, `
                + 'refused 1, unusable 1',
            );
          },
        },),
      ],
    },),

    describe({
      name: editorKindLines.name,
      children: [
        it({
          name: 'SAYS THE SAME COUNTS FOR EACH KIND OF DEFECT in its own line, insertion first, so an arm that '
            + 'solves one kind and not the other is seen',
          fn: async () => {
            expect(editorKindLines({
              tally: editorTallyOf({
                cardId: LOW_CARD,
                rows: ROWS,
              },),
            },),).toStrictEqual([
              `EDITOR ${LOW_CARD} insertion: clean fix 1 of 4 (exact 1, equivalent 0), defect only 1, collateral 1, `
                + 'missed 1, refused 0, unusable 1',
              `EDITOR ${LOW_CARD} alteration: clean fix 1 of 3 (exact 0, equivalent 1), defect only 0, collateral 1, `
                + 'missed 1, refused 1, unusable 0',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: editorDefectOnlyLine.name,
      children: [
        it({
          name: 'SAYS THE DEFECT-ONLY COUNT ON A LINE OF ITS OWN, as not part of the clean fix count, with its kinds',
          fn: async () => {
            expect(editorDefectOnlyLine({
              tally: editorTallyOf({
                cardId: LOW_CARD,
                rows: ROWS,
              },),
            },),).toBe(
              `EDITOR ${LOW_CARD} defect only, not in the clean fix count: 1 of 7 (insertion 1, alteration 0)`,
            );
          },
        },),
      ],
    },),

    describe({
      name: editorDecisionRuleLine.name,
      children: [
        it({
          name: 'STATES WHICH COUNTS ARE COMPARED, WHAT DIFFERENCE COUNTS, WHAT A TIE KEEPS AND WHAT THE INSERTION '
            + 'TRIALS MEASURE, in one line made from the same text as the rule the read step applies',
          fn: async () => {
            expect(editorDecisionRuleLine(),).toBe(
              'RULE clean fix is exact plus equivalent, counted over the trials of both kinds of defect. An effort '
                + 'other than the default (medium) is chosen only when its clean fix count is above the mean of the two '
                + 'medium arms by at least 12 trials, or by at least the two medium arms\' own difference when that is '
                + 'larger, and neither kind of defect (insertion, alteration) is more than 3 trials below the mean of '
                + 'the two medium arms for that kind. A smaller gap, a tie, an effort that fixes fewer, and a gap a kind '
                + 'contradicts all keep the default effort. With no true difference, a gap of 12 trials is the least '
                + 'that keeps the chance of choosing another effort under 0.05 at every true fix rate from 0.30 to 0.95, '
                + 'counting the two trials of an entry as one draw at the worst (editor-fidelity-decide-null.ts), so the '
                + 'bench cannot separate efforts that differ by less. The insertion trials measure locating an added '
                + 'sentence among its neighbours and copying the neighbours verbatim, not the span-only view production '
                + 'gives an editor.',
            );
            expect(editorDecisionRuleLine(),).toBe(`RULE ${decisionRuleText()}`,);
          },
        },),
      ],
    },),

    describe({
      name: editorPreflightLine.name,
      children: [
        it({
          name: 'SAYS WHAT A CAP OF ZERO BOUGHT: cards named and the references selected, that the corpus was read and '
            + 'no model was called, and that the ceiling is priced over every trial',
          fn: async () => {
            expect(editorPreflightLine({
              cardIds: [LOW_CARD,],
              selected: 3,
            },),).toBe(
              `preflight only: 3 pinned reference specifications selected for ${LOW_CARD}; the corpus was read and no model was called; the ceiling is priced over every trial of the pool`,
            );
            expect(editorPreflightLine({
              cardIds: [LOW_CARD, HIGH_CARD,],
              selected: 1,
            },),).toBe(
              `preflight only: 1 pinned reference specification selected for ${LOW_CARD}, ${HIGH_CARD}; the corpus was read and no model was called; the ceiling is priced over every trial of the pool`,
            );
          },
        },),
      ],
    },),

    describe({
      name: editorKeptLine.name,
      children: [
        it({
          name: 'SAYS HOW MANY ROWS WERE KEPT AND WHERE',
          fn: async () => {
            expect(editorKeptLine({
              count: 1,
              keptAt: '/runs/x.json',
            },),).toBe('kept 1 editor row at /runs/x.json',);
            expect(editorKeptLine({
              count: 4,
              keptAt: '/runs/x.json',
            },),).toBe('kept 4 editor rows at /runs/x.json',);
          },
        },),
      ],
    },),
  ],
},);
