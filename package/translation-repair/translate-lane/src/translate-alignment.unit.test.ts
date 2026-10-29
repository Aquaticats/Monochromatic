/**
 Tests for the guard that decides whether a slice's pairing is trustworthy
 enough to replace archive text.

 The failure this exists for is silent and destructive: when the aligner pairs
 a heading against a whole section, the judges are asked which of two unrelated
 texts better renders the heading, they answer correctly, and a passage is
 replaced by a sentence. Every case in this file is a shape measured in the corpus,
 rewritten with invented cat-themed content. No corpus text appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  alignmentRefusalFinding,
  assessSliceAlignment,
  MAX_INCUMBENT_TO_SOURCE_RATIO,
  MIN_PROTECTED_INCUMBENT,
} from '../dist/final/node/index.mjs';

/**
 Section-length English standing where a whole passage would.
 */
const PASSAGE = 'The cat sleeps on the windowsill through the long afternoon, '
  + 'waking only when the kettle sounds. She has done this every day since the '
  + 'spring, and the household has arranged itself around the habit rather than '
  + 'against it.';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: assessSliceAlignment.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES a heading paired against a whole passage, which is the '
            + 'measured shape: a three-character source drew a rendering that '
            + 'replaced a 226-character archive passage at every roster width',
          fn: async () => {
            const assessment = assessSliceAlignment({
              sourceText: '其一：',
              incumbentText: PASSAGE,
            },);
            expect(assessment.kind,).toBe('incumbent-dominates-source',);
            expect(assessment.sourceCodePoints,).toBe(3,);
          },
        },),

        it({
          name: 'ACCEPTS ordinary expansion, which is what a correctly paired slice '
            + 'looks like: Chinese becoming English runs about three to one across '
            + 'the corpus, and a guard refusing that would refuse everything',
          fn: async () => {
            expect(assessSliceAlignment({
              sourceText: '猫猫在窗台上打盹，直到水壶响了才醒来。她从春天起每天都这样，家里也就顺着这个习惯安排了。',
              incumbentText: PASSAGE,
            },).kind,).toBe('within-limit',);
          },
        },),

        it({
          name: 'ACCEPTS a BLANK incumbent whatever the source size, because that '
            + 'is the case the lane exists for: there is nothing to protect and '
            + 'everything to write, and a ratio against zero is not a measurement',
          fn: async () => {
            expect(assessSliceAlignment({
              sourceText: '猫猫喜欢追蝴蝶。'.repeat(20,),
              incumbentText: '',
            },).kind,).toBe('within-limit',);
          },
        },),

        it({
          name: 'ACCEPTS a short incumbent even at an extreme ratio, since below '
            + 'the floor a mispairing costs a phrase rather than a passage and the '
            + 'ratio alone would refuse ordinary short slices',
          fn: async () => {
            expect(assessSliceAlignment({
              sourceText: '猫',
              incumbentText: 'The cat.',
            },).kind,).toBe('within-limit',);
          },
        },),

        it({
          name: 'REFUSES a BLANK source against a substantial translation, where '
            + 'the ratio is undefined rather than large. Nothing can justify '
            + 'replacing archive text with a rendering of nothing',
          fn: async () => {
            expect(assessSliceAlignment({
              sourceText: '   \n  ',
              incumbentText: PASSAGE,
            },).kind,).toBe('incumbent-dominates-source',);
          },
        },),

        it({
          name: 'counts CODE POINTS rather than UTF-16 units, since a source of '
            + 'astral characters would otherwise measure twice its length and pass '
            + 'a ratio it should fail. The comparison runs between Chinese and '
            + 'English, which is exactly where that asymmetry lands',
          fn: async () => {
            /**
             Four astral characters, eight UTF-16 units.
             */
            const astral = '𩸽𩸽𩸽𩸽';
            expect(astral.length,).toBe(8,);
            expect(assessSliceAlignment({
              sourceText: astral,
              incumbentText: PASSAGE,
            },).sourceCodePoints,).toBe(4,);
          },
        },),

        it({
          name: 'carries the constants it applied into the assessment, so a record '
            + 'written today stays readable after either is retuned: a stored '
            + 'refusal that named no threshold could not be re-evaluated at all',
          fn: async () => {
            const assessment = assessSliceAlignment({
              sourceText: '其一：',
              incumbentText: PASSAGE,
            },);
            expect(assessment.minProtectedIncumbent,).toBe(MIN_PROTECTED_INCUMBENT,);
            expect(assessment.maxRatio,).toBe(MAX_INCUMBENT_TO_SOURCE_RATIO,);
          },
        },),

        it({
          name: 'sits exactly at the boundary it advertises, so the constants are '
            + 'the rule rather than an approximation of it',
          fn: async () => {
            /**
             Incumbent exactly at the floor.
             */
            const atFloor = 'x'.repeat(MIN_PROTECTED_INCUMBENT,);

            /**
             Source exactly large enough that the ratio is met but not exceeded.
             */
            const atRatio = 'x'.repeat(
              MIN_PROTECTED_INCUMBENT / MAX_INCUMBENT_TO_SOURCE_RATIO,
            );
            expect(assessSliceAlignment({
              sourceText: atRatio,
              incumbentText: atFloor,
            },).kind,).toBe('within-limit',);
            expect(assessSliceAlignment({
              sourceText: atRatio.slice(1,),
              incumbentText: atFloor,
            },).kind,).toBe('incumbent-dominates-source',);
          },
        },),
      ],
    },),

    describe({
      name: alignmentRefusalFinding.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'names the slice and every number behind the refusal, so a run '
            + 'that protected archive text says which slices and why rather than '
            + 'reporting a quiet lane',
          fn: async () => {
            const finding = alignmentRefusalFinding({
              sliceIndex: 7,
              assessment: assessSliceAlignment({
                sourceText: '其一：',
                incumbentText: PASSAGE,
              },),
            },);
            expect(finding,).toContain('slice 7',);
            expect(finding,).toContain('source 3 code points',);
            expect(finding,).toContain(`ratio limit ${String(MAX_INCUMBENT_TO_SOURCE_RATIO,)}`,);
          },
        },),
      ],
    },),
  ],
},);
