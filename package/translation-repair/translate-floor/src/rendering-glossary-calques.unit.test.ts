/**
 Guards class one hundred twenty-five (shi_Yumiaoya25, 2026-09-25): under the
 owner's standing instruction of 2026-09-25 ("whenever you see anything that
 can be translated better do it"), two calques on the page join the rendering
 glossary. 滑档二本 shipped as "slid down into a second-tier admission slot",
 where the English is that she missed her chosen schools and ended up at a
 second-tier university. Each form appears once in the pinned corpus, on
 that page. The class also seeded 用这种方式 ("using this way to tell"); the
 glossary audit of 2026-09-27 took it out, a construction rather than a word,
 for the idiomatic English rule (`glossary-dictionary-terms.unit.test.ts`).

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
 Original in which the cat misses its chosen schools and lands at a
 second-tier university.
 */
const ADMISSION = '这只猫志愿填错了，滑档二本。';

/**
 Entry the glossary holds for a term, or undefined.

 @param term - Han form to look up

 @returns Entry the glossary seeds for the term

 @throws Error when the glossary lacks the term, which is the failure this
 guard exists to show

 @example
 ```ts
 const entry = entryFor({ term: '二本', },);
 ```
 */
function entryFor({ term, }: { readonly term: string; },): (typeof RENDERING_GLOSSARY)[number] {
  /**
   Entry for the term, if seeded.
   */
  const found = RENDERING_GLOSSARY.find(function isTerm(entry,): boolean {
    return entry.term === term;
  },);
  if (found === undefined)
    throw new Error(`rendering glossary does not seed ${term}`,);
  return found;
}

await describe({
  name: 'calques the rendering glossary refuses (class one hundred twenty-five)',
  children: [
    it({
      name: 'SEEDS 滑档 and 二本 with the English the page uses',
      fn: async () => {
        expect(entryFor({ term: '二本', },).renderings[0],).toBe('second-tier university',);
        expect(entryFor({ term: '二本', },).refusedForms,).toContain('admission slot',);
        // Ledger R4: the slide is refused onto a tier, never as a bare motion verb.
        expect(entryFor({ term: '滑档', },).refusedForms,).toContain('slid down into a second tier',);
        expect(entryFor({ term: '滑档', },).refusedForms,).not
          .toContain('slid down',);
      },
    },),
    it({
      name: 'REFUSES the calques the page shipped',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: ADMISSION,
          candidateText: 'The cat filled in the wrong choices and slid down into a second-tier admission slot.',
        },).kind,).toBe('invalid',);
        expect(validateTranslatedSlice({
          sourceText: ADMISSION,
          candidateText: 'The cat filled in the wrong choices and slipped to a second-tier university.',
        },).kind,).toBe('invalid',);
      },
    },),
    it({
      name: 'ACCEPTS the English meaning',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: ADMISSION,
          candidateText: 'The cat filled in the wrong choices, missed its chosen schools and ended up at a second-tier university.',
        },).kind,).toBe('valid',);
      },
    },),
  ],
},);
