/**
 Guards class one hundred eighty (TianqiChen66613, 2026-09-27): under the
 owner's standing instruction of 2026-09-25 ("whenever you see anything that
 can be translated better do it"), two words on the page join the rendering
 glossary. 亲友 shipped as the archive's "close friends", dropping the family
 the word names, and 激素 as "medication", a word that hides the hormones the
 passage is about.

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
 Original in which the cat's friends and family remember its sunbathing.
 */
const REMEMBERED = '亲友都记得这只猫爱晒太阳。';

/**
 Candidate that narrows 亲友 to friends alone.
 */
const NARROWED = 'The cat\'s close friends all remembered that it loved sunbathing.';

/**
 Candidate carrying the English the glossary seeds.
 */
const WHOLE = 'The cat\'s friends and family all remembered that it loved sunbathing.';

/**
 First English a seeded term offers.

 @param term - glossary key

 @returns Seeded rendering, empty where the term is not seeded

 @example
 ```ts
 const first = firstRendering({ term: '亲友', },);
 ```
 */
function firstRendering({ term, }: { readonly term: string; },): string {
  return RENDERING_GLOSSARY.find(function isTerm(entry,): boolean {
    return entry.term === term;
  },)?.renderings[0] ?? '';
}

await describe({
  name: 'kin and hormone words the rendering glossary renders (class one hundred eighty)',
  children: [
    it({
      name: 'SEEDS 亲友 with "friends and family" first',
      fn: async () => {
        expect(firstRendering({ term: '亲友', },),).toBe('friends and family',);
      },
    },),
    it({
      name: 'REFUSES "close friends" for 亲友',
      fn: async () => {
        expect(validateTranslatedSlice({ sourceText: REMEMBERED, candidateText: NARROWED, },).kind,)
          .toBe('invalid',);
      },
    },),
    it({
      name: 'PASSES "friends and family"',
      fn: async () => {
        expect(validateTranslatedSlice({ sourceText: REMEMBERED, candidateText: WHOLE, },).kind,).toBe('valid',);
      },
    },),
    it({
      name: 'SEEDS 激素 with "hormones" first',
      fn: async () => {
        expect(firstRendering({ term: '激素', },),).toBe('hormones',);
      },
    },),
  ],
},);
