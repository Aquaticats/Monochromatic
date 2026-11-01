/**
 Guards class one hundred twenty-three (2026-09-25): one page shipped
 师范学院 and 学霸 as word-for-word calques an English reader stumbles on. The owner answered on 2026-09-25 that anything which can be
 translated better should be. The rendering glossary carries such words onto
 every sheet with the English the page uses, and refuses a known calque before
 any judge reads it, as the community glossary does for the community's
 words. An official institution name keeps its own English ("Normal
 University"), since only the generic words are entered.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  RENDERING_GLOSSARY,
  renderingTermLines,
} from '@monochromatic-dev/translation-repair-rendering-audit';
import { validateTranslatedSlice, } from '../dist/final/node/index.mjs';

/**
 Original sending the cat to a generic teachers' college.
 */
const COLLEGE = '这只猫进入了一所师范学院。';

/**
 Original calling the cat a top student.
 */
const TOP_STUDENT = '考试前一晚，这只猫的学霸属性突然发作了。';

/**
 Original naming an institution whose official English carries "Normal".
 */
const OFFICIAL = '这只猫在北京师范大学晒太阳。';

await describe({
  name: 'a word the rendering glossary renders (class one hundred twenty-three)',
  children: [
    it({
      name: 'SEEDS 师范学院 and 学霸 with the English the page uses and refuses their calques',
      fn: async () => {
        /**
         Entry for the generic teachers' college.
         */
        const college = RENDERING_GLOSSARY.find(function isTerm(entry,): boolean {
          return entry.term === '师范学院';
        },);
        /**
         Entry for the top student.
         */
        const topStudent = RENDERING_GLOSSARY.find(function isTerm(entry,): boolean {
          return entry.term === '学霸';
        },);
        expect(college?.renderings[0],).toBe('teachers\' college',);
        expect(college?.refusedForms,).toContain('normal college',);
        expect(topStudent?.renderings[0],).toBe('top student',);
        expect(topStudent?.refusedForms,).toContain('top-student trait',);
      },
    },),
    it({
      name: 'PUTS the word on the sheet with its rendering and why, only where the original carries it',
      fn: async () => {
        /**
         Identity-context lines for the college original.
         */
        const lines = renderingTermLines({ text: COLLEGE, },);
        expect(lines[0],).toContain('RENDERINGS',);
        expect(lines.join('\n',),).toContain('teachers\' college',);
        expect(renderingTermLines({ text: '猫在窗台上睡觉。', },),).toEqual([],);
      },
    },),
    it({
      name: 'REFUSES a candidate that writes a calque, in any casing, or leaves the word in Han',
      fn: async () => {
        /**
         Verdict on the calqued college.
         */
        const verdict = validateTranslatedSlice({
          sourceText: COLLEGE,
          candidateText: 'The cat entered a Normal College.',
        },);
        expect(verdict.kind,).toBe('invalid',);
        if (verdict.kind !== 'invalid')
          throw new Error('unreachable',);
        expect(verdict.findings.join('\n',),).toContain('teachers\' college',);
        expect(verdict.findings.join('\n',),).not.toContain('community term',);
        expect(validateTranslatedSlice({
          sourceText: TOP_STUDENT,
          candidateText: 'The night before the exam, the cat\'s top-student trait suddenly kicked in.',
        },).kind,).toBe('invalid',);
        expect(validateTranslatedSlice({
          sourceText: COLLEGE,
          candidateText: 'The cat entered a 师范学院.',
        },).kind,).toBe('invalid',);
      },
    },),
    it({
      name: 'ACCEPTS the English the page uses and an official name that carries "Normal"',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: COLLEGE,
          candidateText: 'The cat entered a teachers\' college.',
        },).kind,).toBe('valid',);
        expect(validateTranslatedSlice({
          sourceText: TOP_STUDENT,
          candidateText: 'The night before the exam, the cat suddenly turned into a top student.',
        },).kind,).toBe('valid',);
        expect(validateTranslatedSlice({
          sourceText: OFFICIAL,
          candidateText: 'The cat sunbathed at Beijing Normal University.',
        },).kind,).toBe('valid',);
      },
    },),
  ],
},);
