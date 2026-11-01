/**
 Guards ledger H9 (XingZ60, 2026-09-28): a rare character a page repeats in
 slices judged apart took a different rendering in each, and the partial
 archive gave the bench no English to follow. The rendering glossary now
 carries 螐 with one English form, which reaches every sheet whose original
 writes it, and the source-carry floor refuses the character left in Han.

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
 Original in which the kitten watches a little 螐 on a leaf.
 */
const LEAF = '小猫盯着叶子上的一只螐儿看了很久。';

await describe({
  name: 'a rare character the rendering glossary carries (ledger H9)',
  children: [
    it({
      name: 'SEEDS 螐 with the insect as its English',
      fn: async () => {
        expect(RENDERING_GLOSSARY.filter(function isRare(entry,): boolean {
          return entry.term === '螐';
        },).map(function renderingsOf(entry,): readonly string[] {
          return entry.renderings;
        },),).toEqual([['caterpillar', 'little caterpillar',],],);
      },
    },),
    it({
      name: 'PUTS the entry on the sheet of an original that writes it, and on no other',
      fn: async () => {
        expect(renderingTermLines({ text: LEAF, },).some(function namesIt(line,): boolean {
          return line.includes('螐',) && line.includes('caterpillar',);
        },),).toBe(true,);
        expect(renderingTermLines({ text: '小猫盯着叶子上的一只蝴蝶看了很久。', },).some(function namesIt(line,): boolean {
          return line.includes('螐',);
        },),).toBe(false,);
      },
    },),
    it({
      name: 'REFUSES the character left in Han and ACCEPTS the English',
      fn: async () => {
        expect([
          validateTranslatedSlice({ sourceText: LEAF, candidateText: 'The kitten stared at a little 螐 on the leaf for a long time.', },).kind,
          validateTranslatedSlice({ sourceText: LEAF, candidateText: 'The kitten stared at a little caterpillar on the leaf for a long time.', },).kind,
        ],).toEqual(['invalid', 'valid',],);
      },
    },),
  ],
},);
