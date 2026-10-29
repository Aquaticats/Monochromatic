/**
 Guards class one hundred thirty-five (2026-09-25): 豆花 is seeded as
 "douhua", since one page's front matter, archive and every other slice wrote
 "douhua" while the closing quote shipped "tofu pudding", one food under two
 names on one page.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { RENDERING_GLOSSARY, } from '@monochromatic-dev/translation-repair-rendering-audit';
import { validateTranslatedSlice, } from '../dist/final/node/index.mjs';

/**
 Original in which the cat sniffs at a bowl of douhua.
 */
const LIKE_DOUHUA = '猫凑过去闻了闻那碗豆花。';

await describe({
  name: '豆花 on the sheets (class one hundred thirty-five)',
  children: [
    it({
      name: 'SEEDS 豆花',
      fn: async () => {
        expect(RENDERING_GLOSSARY.some(function isTerm(entry,): boolean {
          return entry.term === '豆花';
        },),).toBe(true,);
      },
    },),
    it({
      name: 'REFUSES tofu pudding and ACCEPTS douhua',
      fn: async () => {
        expect([
          validateTranslatedSlice({
            sourceText: LIKE_DOUHUA,
            candidateText: 'The cat leaned in and sniffed the bowl of tofu pudding.',
          },).kind,
          validateTranslatedSlice({
            sourceText: LIKE_DOUHUA,
            candidateText: 'The cat leaned in and sniffed the bowl of douhua.',
          },).kind,
        ],).toEqual(['invalid', 'valid',],);
      },
    },),
  ],
},);
