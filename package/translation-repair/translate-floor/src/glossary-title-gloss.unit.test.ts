/**
 Guards ledger F-11 (2026-09-27): the Han title floor accepts a title the
 original brackets in 《》 when the rendering keeps it with its English in
 parentheses after it, and the glossary floors refused the same rendering for
 the glossary word inside that title. A term the original writes only inside
 such a title is owed nothing more; the same term outside it still is.
 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { validateTranslatedSlice, } from '../dist/final/node/index.mjs';

await describe({
  name: 'a glossary word inside a glossed Han title (ledger F-11)',
  children: [
    it({
      name: 'ACCEPTS a title kept in Han with its English after it, though the title carries a glossary word',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: '猫读完了《高考猫》。',
          candidateText: 'The cat finished reading 《高考猫》 (The Exam Cat).',
        },).kind,).toBe('valid',);
      },
    },),
    it({
      name: 'STILL REFUSES the same word left in Han outside the title, and a title kept bare',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: '猫读完了《高考猫》，然后准备高考。',
          candidateText: 'The cat finished reading 《高考猫》 (The Exam Cat), then prepared for the 高考.',
        },).kind,).toBe('invalid',);
        expect(validateTranslatedSlice({
          sourceText: '猫读完了《高考猫》。',
          candidateText: 'The cat finished reading 《高考猫》.',
        },).kind,).toBe('invalid',);
      },
    },),
  ],
},);
