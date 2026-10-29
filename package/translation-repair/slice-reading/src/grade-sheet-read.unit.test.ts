/**
 Tests for reading a filled grading sheet, and for scoring a blind pre-grade
 against it.

 The fixture shapes are taken from the two sheets a human has actually graded,
 which differ from each other and from anything specified: bracketed and
 unbracketed answers, a verdict letter followed by prose, and answers that are
 not verdicts at all. The prose itself is cat-themed invention.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  type GradedItem,
  parseGradedSheet,
  parsePreGrades,
  scoreGradeAgreement,
  scoreGradedPrecision,
} from '../dist/final/node/index.mjs';
import { statedRefusalMessage, } from '@monochromatic-dev/translation-repair-glossary/ts/stated-refusal-message.test-fixture.ts';

/**
 Legend every graded line carries, which bounds the grader's answer.
 */
const LEGEND = '  (Y = real defect · N = false positive)';

/**
 Builds a sheet from raw grade answers, one item per answer.

 @param answers - text each grader answer carries, in sheet order

 @returns Sheet text the parser reads

 @example
 ```ts
 const sheet = catSheet({ answers: ['[Y]', 'N',], },);
 ```
 */
function catSheet(
  { answers, }: { readonly answers: readonly string[]; },
): string {
  return [
    '# Milestone 3 precision grading sheet',
    '',
    '---',
    '',
    ...answers.flatMap(function toItem(
      answer,
      position,
    ) {
      return [
        `### ${String(position + 1,)}. grade: ${answer}${LEGEND}`,
        '- entry: Kitten · band: medium',
        '',
      ];
    },),
  ].join('\n',);
}

/**
 Builds pre-grades for positions one upward.

 @param verdicts - verdict per position, in sheet order

 @returns Pre-graded items

 @example
 ```ts
 const agent = catPreGrades({ verdicts: ['real-defect',], },);
 ```
 */
function catPreGrades(
  { verdicts, }: { readonly verdicts: readonly GradedItem['verdict'][]; },
): readonly GradedItem[] {
  return verdicts.map(function toItem(
    verdict,
    position,
  ): GradedItem {
    return {
      index: position + 1,
      verdict,
      note: '',
    };
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: parseGradedSheet.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reads the bracketed form one round used and the bare form the '
            + 'other did, since both are how a human has really filled a sheet',
          fn: async () => {
            const items = parseGradedSheet({
              text: catSheet({
                answers: [
                  '[Y]',
                  'Y',
                  '[N]',
                  'N',
                ],
              },),
            },);
            expect(items.map(function toVerdict(item,) {
              return item.verdict;
            },),).toEqual([
              'real-defect',
              'real-defect',
              'false-positive',
              'false-positive',
            ],);
            expect(items.map(function toIndex(item,) {
              return item.index;
            },),).toEqual([
              1,
              2,
              3,
              4,
            ],);
          },
        },),

        it({
          name: 'keeps the rationale a grader wrote after the verdict, which '
            + 'nothing else in the run reproduces',
          fn: async () => {
            const items = parseGradedSheet({
              text: catSheet({
                answers: [
                  '[Y, but the warmer word would be "naps"]',
                  'N. The original does quote the purr.',
                ],
              },),
            },);
            expect(items[0]?.verdict,).toBe('real-defect',);
            expect(items[0]?.note,).toBe('but the warmer word would be "naps"',);
            expect(items[1]?.verdict,).toBe('false-positive',);
            expect(items[1]?.note,).toBe('The original does quote the purr.',);
          },
        },),

        it({
          name: 'does NOT read an answer beginning with a verdict letter mid-word '
            + 'as that verdict, which is exactly how "Not enough context to grade" '
            + 'would otherwise become a false positive',
          fn: async () => {
            const items = parseGradedSheet({
              text: catSheet({
                answers: [
                  '[Not enough context to grade]',
                  '[Not sure which tense fits a cat]',
                  '[Yesterday I would have said otherwise]',
                ],
              },),
            },);
            expect(items.map(function toVerdict(item,) {
              return item.verdict;
            },),).toEqual([
              'unscored',
              'unscored',
              'unscored',
            ],);
            expect(items[0]?.note,).toBe('Not enough context to grade',);
          },
        },),

        it({
          name: 'reads an untouched box as unscored rather than as a verdict',
          fn: async () => {
            const items = parseGradedSheet({ text: catSheet({ answers: ['[ ]',], },), },);
            expect(items[0]?.verdict,).toBe('unscored',);
            expect(items[0]?.note,).toBe('',);
          },
        },),

        it({
          name: 'bounds the answer at the legend even when the rationale quotes '
            + 'the legend\'s own wording',
          fn: async () => {
            const items = parseGradedSheet({
              text: catSheet({
                answers: ['[N, this reads to me like (Y = real defect) but it is not]',],
              },),
            },);
            expect(items[0]?.verdict,).toBe('false-positive',);
            expect(items[0]?.note,).toBe('this reads to me like (Y = real defect) but it is not',);
          },
        },),

        it({
          name: 'ignores lines that are not item headings, so surrounding prose '
            + 'never becomes an item',
          fn: async () => {
            const items = parseGradedSheet({
              text: `> PRELIMINARY draw, grade: nothing here\n${
            catSheet({ answers: ['[Y]',], },)
          }`,
            },);
            expect(items,).toHaveLength(1,);
          },
        },),

        it({
          name: 'READS THE WHOLE REST as the answer when no legend bounds it, its verdict letter and every '
            + 'word of its note',
          fn: async () => {
            expect(parseGradedSheet({ text: '### 1. grade: Y, the cat naps on the sill', },),).toEqual([{
              index: 1,
              verdict: 'real-defect',
              note: 'the cat naps on the sill',
            },],);
          },
        },),

        it({
          name: 'READS A DUPLICATE verdict where the answer opens with the word, keeping the note the grader '
            + 'wrote',
          fn: async () => {
            expect(parseGradedSheet({
              text: catSheet({ answers: ['Duplicate: the same page twice',], },),
            },),).toEqual([{
              index: 1,
              verdict: 'duplicate',
              note: 'Duplicate: the same page twice',
            },],);
          },
        },),
      ],
    },),

    describe({
      name: scoreGradedPrecision.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'excludes declined items from the denominator, since a question '
            + 'the grader refused is not evidence either way',
          fn: async () => {
            const tally = scoreGradedPrecision({
              human: parseGradedSheet({
                text: catSheet({
                  answers: [
                    '[Y]',
                    '[Y]',
                    '[N]',
                    '[Not enough context to grade]',
                  ],
                },),
              },),
            },);
            expect(tally.scored,).toBe(3,);
            expect(tally.realDefects,).toBe(2,);
            expect(tally.unscored,).toEqual([4,],);
          },
        },),

        it({
          name: 'LISTS THE DUPLICATE INDICES beside the rates and leaves them out of every denominator, since a '
            + 'duplicate row is one opinion counted once',
          fn: async () => {
            /**
             One real defect and the same defect graded again at the next position.
             */
            const human: readonly GradedItem[] = [{
              index: 1,
              verdict: 'real-defect',
              note: '',
            }, {
              index: 2,
              verdict: 'duplicate',
              note: 'the same page twice',
            },];
            expect(scoreGradedPrecision({ human, },),).toEqual({
              scored: 1,
              realDefects: 1,
              unscored: [],
              duplicates: [2,],
              gradeable: 1,
            },);
          },
        },),
      ],
    },),

    describe({
      name: parsePreGrades.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reads well-formed pre-grades, defaulting a missing note to empty',
          fn: async () => {
            const items = parsePreGrades({
              text: JSON.stringify([
                {
                  index: 1,
                  verdict: 'real-defect',
                  note: 'clear omission',
                },
                {
                  index: 2,
                  verdict: 'unscored',
                },
              ],),
            },);
            expect(items,).toHaveLength(2,);
            expect(items[0]?.note,).toBe('clear omission',);
            expect(items[1]?.verdict,).toBe('unscored',);
            expect(items[1]?.note,).toBe('',);
          },
        },),

        it({
          name: 'throws rather than dropping a malformed pre-grade, since a '
            + 'silently skipped one shifts the agreement denominator',
          fn: async () => {
            for (
              const [
                text,
                expected,
              ] of [
                [
                  '{}',
                  'array',
                ],
                [
                  JSON.stringify(['not an object',],),
                  'not an object',
                ],
                [
                  JSON.stringify([{ verdict: 'real-defect', },],),
                  'numeric index',
                ],
                [
                  JSON.stringify([
                    {
                      index: 1,
                      verdict: 'maybe',
                    },
                  ],),
                  'outside the vocabulary',
                ],
              ] as const
            ) {
              /** Failure raised by this malformed file. */
              let refusal: unknown;
              try {
                parsePreGrades({ text, },);
              }
              catch (error) {
                refusal = error;
              }
              expect(refusal,).toBeInstanceOf(Error,);
              expect((refusal as Error).message,).toContain(expected,);
            }
          },
        },),

        it({
          name: 'REFUSES a pre-grade whose verdict is no string as a stated refusal naming its position, since '
            + 'nothing there names a verdict the grader gave',
          fn: async () => {
            expect(statedRefusalMessage({
              read: function readsNumericVerdict(): void {
                parsePreGrades({ text: '[{"index": 1, "verdict": 42}]', },);
              },
            },),).toBe('pre-grade 1 carries a verdict outside the vocabulary',);
          },
        },),

        it({
          name: 'READS a pre-grade marking its item a duplicate, since a blind grader meets the same repeated '
            + 'defects the sheet reader marks and a file saying so must not be refused whole',
          fn: async () => {
            expect(parsePreGrades({
              text: JSON.stringify([{
                index: 1,
                verdict: 'duplicate',
                note: 'the same nap as item 3',
              },],),
            },),).toEqual([{
              index: 1,
              verdict: 'duplicate',
              note: 'the same nap as item 3',
            },],);
          },
        },),
      ],
    },),

    describe({
      name: scoreGradeAgreement.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'names every disagreement and scores only over items the human '
            + 'graded',
          fn: async () => {
            /** Human grades: real, real, false, declined. */
            const human = parseGradedSheet({
              text: catSheet({
                answers: [
                  '[Y]',
                  '[Y]',
                  '[N]',
                  '[Not sure]',
                ],
              },),
            },);
            const tally = scoreGradeAgreement({
              agent: catPreGrades({
                verdicts: [
                  'real-defect',
                  'false-positive',
                  'false-positive',
                  'real-defect',
                ],
              },),
              human,
            },);
            expect(tally.compared,).toBe(3,);
            expect(tally.agreed,).toBe(2,);
            expect(tally.disagreed,).toEqual([2,],);
            // The declined item is reported, never silently folded into agreement.
            expect(tally.unscored,).toEqual([4,],);
          },
        },),

        it({
          name: 'NAMES a pre-grade\'s duplicate on an item the human scored as a disagreement, and leaves out an '
            + 'item the human marked duplicate whatever the pre-grade said',
          fn: async () => {
            expect(scoreGradeAgreement({
              agent: catPreGrades({
                verdicts: [
                  'duplicate',
                  'duplicate',
                  'real-defect',
                ],
              },),
              human: parseGradedSheet({
                text: catSheet({
                  answers: [
                    '[Y]',
                    'Duplicate: the same nap as item 1',
                    '[Y]',
                  ],
                },),
              },),
            },),).toEqual({
              compared: 2,
              agreed: 1,
              disagreed: [1,],
              unscored: [],
            },);
          },
        },),

        it({
          name: 'refuses to compare two sets that cover different draws, which '
            + 'would otherwise report an agreement rate across rounds',
          fn: async () => {
            /** Failure raised by the mismatched coverage. */
            let refusal: unknown;
            try {
              scoreGradeAgreement({
                agent: catPreGrades({ verdicts: ['real-defect',], },),
                human: parseGradedSheet({
                  text: catSheet({
                    answers: [
                      '[Y]',
                      '[N]',
                    ],
                  },),
                },),
              },);
            }
            catch (error) {
              refusal = error;
            }
            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect((refusal as Error).message,).toContain('not the same draw',);
          },
        },),
      ],
    },),

    describe({
      name: 'stated refusals over graded sheets and pre-grades',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES a pre-grade file that misses a sheet position, even when its count matches, since every '
            + 'lookup would otherwise read as a disagreement rather than as a file fault',
          fn: async () => {
            expect(function comparesShifted(): void {
              scoreGradeAgreement({
                agent: [
                  {
                    index: 0,
                    verdict: 'real-defect',
                    note: '',
                  },
                  {
                    index: 1,
                    verdict: 'false-positive',
                    note: '',
                  },
                ],
                human: parseGradedSheet({
                  text: catSheet({
                    answers: [
                      '[Y]',
                      '[N]',
                    ],
                  },),
                },),
              },);
            },).toThrow(StatedRefusalError,);
          },
        },),

        it({
          name: 'REFUSES a pre-grades file that is not an array as a stated refusal, so the boundary prints the '
            + 'sentence rather than a fault with frames',
          fn: async () => {
            expect(function parsesObject(): void {
              parsePreGrades({ text: '{}', },);
            },).toThrow(StatedRefusalError,);
          },
        },),

        it({
          name: 'REFUSES a sheet whose printed item number disagrees with its position, since a heading added, '
            + 'deleted or duplicated by hand would renumber every later item against the pre-grades',
          fn: async () => {
            /**
             Sheet whose second heading says 3.
             */
            const renumbered = catSheet({
              answers: [
                '[Y]',
                '[N]',
              ],
            },)
              .replace('### 2.', '### 3.',);

            expect(statedRefusalMessage({
              read: function readsRenumbered(): void {
                parseGradedSheet({ text: renumbered, },);
              },
            },),).toBe('sheet item at position 2 is headed 3; a heading was added, deleted or duplicated by hand',);
          },
        },),

        it({
          name: 'REFUSES a heading that carries no number, saying so rather than calling it headed zero (ledger B73)',
          fn: async () => {
            /**
             Sheet whose first heading carries no number.
             */
            const sheet = catSheet({ answers: ['[Y]',], },).replace('### 1.', '### .',);
            expect(statedRefusalMessage({
              read: function readsUnnumbered(): void {
                parseGradedSheet({ text: sheet, },);
              },
            },),).toBe(
              'sheet item at position 1 is headed with no number; a heading was added, deleted or duplicated by hand',
            );
          },
        },),

        it({
          name: 'REFUSES a heading whose number is not written in digits as the sheet writes it, though `Number` '
            + 'reads it as the right position (ledger B73)',
          fn: async () => {
            /**
             Sheet whose first heading writes its number with an exponent.
             */
            const sheet = catSheet({ answers: ['[Y]',], },).replace('### 1.', '### 1e0.',);
            expect(statedRefusalMessage({
              read: function readsExponent(): void {
                parseGradedSheet({ text: sheet, },);
              },
            },),).toBe('sheet item at position 1 is headed "1e0"; a heading was added, deleted or duplicated by hand',);
          },
        },),
      ],
    },),
  ],
},);
