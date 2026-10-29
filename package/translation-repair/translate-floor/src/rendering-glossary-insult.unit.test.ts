/**
 Guards class one hundred fifty-five (shi_Yumiaoya38, 2026-09-26): under the
 owner's standing instruction of 2026-09-25 ("whenever you see anything that
 can be translated better do it"), the father's insult 没本事 joins the
 rendering glossary. It shipped as "someone with no capability", word for
 word, where shi_Yumiaoya37 wrote "a failure"; an English parent calls a
 child "good-for-nothing". The glossary seeds that English and refuses the
 calque.

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
 Original in which the old cat calls the kitten useless.
 */
const INSULT = '老猫骂小猫没本事。';

await describe({
  name: 'the insult the rendering glossary renders (class one hundred fifty-five)',
  children: [
    it({
      name: 'SEEDS 没本事 with "good-for-nothing" first',
      fn: async () => {
        /**
         The seeded entry.
         */
        const entry = RENDERING_GLOSSARY.find(function isTerm(candidate,): boolean {
          return candidate.term === '没本事';
        },);
        expect(entry?.renderings[0],).toBe('good-for-nothing',);
      },
    },),
    it({
      name: 'REFUSES "no capability" and passes "good-for-nothing"',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: INSULT,
          candidateText: 'The old cat called the kitten someone with no capability.',
        },).kind,).toBe('invalid',);
        expect(validateTranslatedSlice({
          sourceText: INSULT,
          candidateText: 'The old cat called the kitten good-for-nothing.',
        },).kind,).toBe('valid',);
      },
    },),
  ],
},);
