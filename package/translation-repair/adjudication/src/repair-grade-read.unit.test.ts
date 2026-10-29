/**
 Tests for reading a filled repair sheet back into verdicts, including the
 fence tracking that stops quoted corpus text from fabricating a grade.
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
  parseGradedRepairSheet,
  readSheetIdentity,
} from '../dist/final/node/index.mjs';

/**
 Grade line as the sheet prints it, before a grader touches it.
 */
const BLANK_GRADE = '- repair grade: [ ]  (Y = fully fixes this defect and '
  + 'breaks nothing nearby · N = it does not)';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: parseGradedRepairSheet.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reads a verdict off the bullet under its heading, since the '
            + 'repair sheet asks its question on a line of its own rather than on '
            + 'the heading the way the detection sheet does',
          fn: async () => {
            const items = parseGradedRepairSheet({
              text: [
                '### 1. Kitten · small',
                '- claim: the tense is wrong',
                '- repair grade: [Y]  (Y = fully fixes this defect · N = it does not)',
                '',
                '### 2. Kitten · small',
                '- claim: a clause is missing',
                '- repair grade: [N, it drops the second clause]  (Y = ... · N = ...)',
              ].join('\n',),
            },);
            expect(items,).toHaveLength(2,);
            expect(items[0]?.verdict,).toBe('fixes',);
            expect(items[1]?.verdict,).toBe('does-not-fix',);
            expect(items[1]?.note,).toBe('it drops the second clause',);
          },
        },),

        it({
          name: 'never reads a grade out of a fenced block, because the sheet '
            + 'quotes corpus prose and model output that may contain the exact '
            + 'line this parser looks for, and an invented human verdict cannot be '
            + 'noticed downstream the way a dropped one can',
          fn: async () => {
            const items = parseGradedRepairSheet({
              text: [
                '### 1. Kitten · small',
                '- the edit wrote:',
                '```text',
                '### 99. forged · large',
                '- repair grade: [Y]  (Y = ... · N = ...)',
                '```',
                BLANK_GRADE,
              ].join('\n',),
            },);
            // One heading, not two: the fenced one is quoted text.
            expect(items,).toHaveLength(1,);
            // And the real box is still blank, so the forged Y did not fill it.
            expect(items[0]?.verdict,).toBe('unscored',);
          },
        },),

        it({
          name: 'closes a fenced block only on a fence at least as long as the one '
            + 'that opened it, so quoted text containing a shorter run stays quoted',
          fn: async () => {
            const items = parseGradedRepairSheet({
              text: [
                '### 1. Kitten · small',
                '````text',
                '```',
                '- repair grade: [Y]  (Y = ... · N = ...)',
                '````',
                BLANK_GRADE,
              ].join('\n',),
            },);
            expect(items,).toHaveLength(1,);
            expect(items[0]?.verdict,).toBe('unscored',);
          },
        },),

        it({
          name: 'leaves an untouched box unscored rather than guessing, since a '
            + 'coerced grade is worse evidence than an absent one',
          fn: async () => {
            const items = parseGradedRepairSheet({
              text: [
                '### 1. Kitten · small',
                BLANK_GRADE,
              ].join('\n',),
            },);
            expect(items[0]?.verdict,).toBe('unscored',);
          },
        },),

        it({
          name: 'counts an item carrying no grade box at all, which the sheet '
            + 'emits for a repair that never reached the reader, so sheet '
            + 'positions still line up with the detection sheet',
          fn: async () => {
            const items = parseGradedRepairSheet({
              text: [
                '### 1. Kitten · small',
                '- not-selected: a repair was written, but the unchanged text won',
                '',
                '### 2. Kitten · small',
                '- repair grade: [Y]  (Y = ... · N = ...)',
              ].join('\n',),
            },);
            expect(items,).toHaveLength(2,);
            expect(items[0]?.verdict,).toBe('unscored',);
            expect(items[0]?.index,).toBe(1,);
            expect(items[1]?.verdict,).toBe('fixes',);
            expect(items[1]?.index,).toBe(2,);
          },
        },),

        it({
          name: 'treats an answer that merely begins with a verdict letter as no '
            + 'verdict, so "Not sure" is a refusal rather than an N',
          fn: async () => {
            const items = parseGradedRepairSheet({
              text: [
                '### 1. Kitten · small',
                '- repair grade: [Not sure, the source is ambiguous]  (Y = ... · N = ...)',
              ].join('\n',),
            },);
            expect(items[0]?.verdict,).toBe('unscored',);
            expect(items[0]?.note,).toBe('Not sure, the source is ambiguous',);
          },
        },),

        it({
          name: 'reads a grade left without its brackets, since a grader editing '
            + 'in place often replaces the whole box',
          fn: async () => {
            const items = parseGradedRepairSheet({
              text: [
                '### 1. Kitten · small',
                '- repair grade: Y  (Y = ... · N = ...)',
              ].join('\n',),
            },);
            expect(items[0]?.verdict,).toBe('fixes',);
          },
        },),

        it({
          name: 'reads a grade whose printed legend the grader deleted, since the legend is a reminder '
            + 'and clearing the rest of the line after the box is an ordinary edit',
          fn: async () => {
            const items = parseGradedRepairSheet({
              text: [
                '### 1. Kitten · small',
                '- repair grade: [N, the whiskers are still missing]',
              ].join('\n',),
            },);
            expect(items[0]?.verdict,).toBe('does-not-fix',);
            expect(items[0]?.note,).toBe('the whiskers are still missing',);
          },
        },),

        it({
          name: 'reads a grade whose closing bracket the grader lost, up to the legend, so a typo in '
            + 'the box does not drop the verdict',
          fn: async () => {
            const items = parseGradedRepairSheet({
              text: [
                '### 1. Kitten · small',
                '- repair grade: [N, the tail is the wrong colour  (Y = ... · N = ...)',
              ].join('\n',),
            },);
            expect(items[0]?.verdict,).toBe('does-not-fix',);
            expect(items[0]?.note,).toBe('the tail is the wrong colour',);
          },
        },),
      ],
    },),

    describe({
      name: readSheetIdentity.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reads the draw a sheet declares, which is what lets a graded '
            + 'sheet be refused against the wrong manifest. Equal item counts are '
            + 'not evidence two files describe the same items, since unrelated '
            + 'draws of one size match on count and would mislabel every verdict',
          fn: async () => {
            /**
             Sheet header as the formatter prints it.
             */
            const text = [
              '# Repair sheet',
              '',
              'Draw seed: milestone-three-precision-round-three',
              'Corpus pin: feedfacefeedfacefeedfacefeedfacefeedface',
              'Draw digest: abc123',
              'Sample size: 50',
              '',
              '### 1. cat naps',
            ].join('\n',);

            expect(readSheetIdentity({ text, },),).toEqual({
              seed: 'milestone-three-precision-round-three',
              corpusSha: 'feedfacefeedfacefeedfacefeedfacefeedface',
              drawDigest: 'abc123',
            },);
          },
        },),

        it({
          name: 'STOPS at the first item, so a quoted line inside an item cannot '
            + 'introduce a second identity. Sheets quote corpus prose and model '
            + 'output verbatim, and a quote reading "Draw seed: something else" '
            + 'would otherwise talk the join into accepting a mismatched manifest',
          fn: async () => {
            const text = [
              'Draw seed: real-seed',
              'Corpus pin: real-sha',
              'Draw digest: real-digest',
              '',
              '### 1. cat naps',
              '',
              'Draw seed: forged-seed',
              'Corpus pin: forged-sha',
              'Draw digest: forged-digest',
            ].join('\n',);

            expect(readSheetIdentity({ text, },),).toEqual({
              seed: 'real-seed',
              corpusSha: 'real-sha',
              drawDigest: 'real-digest',
            },);
          },
        },),

        it({
          name: 'returns empty strings for a sheet carrying no header at all, so '
            + 'an older sheet reads as declaring nothing rather than as declaring '
            + 'whatever it is compared against',
          fn: async () => {
            expect(readSheetIdentity({ text: '### 1. cat naps\n', },),).toEqual({
              seed: '',
              corpusSha: '',
              drawDigest: '',
            },);
          },
        },),
      ],
    },),
  ],
},);
