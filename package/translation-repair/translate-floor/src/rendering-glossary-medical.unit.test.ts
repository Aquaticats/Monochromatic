/**
 Guards class one hundred sixty-two (TianqiChen6663, 2026-09-26): under the
 owner's standing instruction of 2026-09-25 ("whenever you see anything that
 can be translated better do it"), II型糖尿病 joins the rendering glossary.
 The page shipped the archive's "type II diabetes", a dated form; current
 English, Diabetes Canada's among it, writes "type 2 diabetes".

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
 Original in which the old cat lives with the disease.
 */
const DIAGNOSED = '老猫患有II型糖尿病，每天都要打针。';

/**
 Candidates the dated form would ship.
 */
const REFUSED_CANDIDATES: readonly string[] = [
  'The old cat had type II diabetes and needed an injection every day.',
  'The old cat had Type-II diabetes and needed an injection every day.',
];

/**
 Candidate carrying the English the glossary seeds.
 */
const ACCEPTED_CANDIDATE = 'The old cat had type 2 diabetes and needed an injection every day.';

await describe({
  name: 'medical terms the rendering glossary renders (class one hundred sixty-two)',
  children: [
    it({
      name: 'SEEDS II型糖尿病 with "type 2 diabetes" first',
      fn: async () => {
        expect(RENDERING_GLOSSARY.find(function isTerm(entry,): boolean {
          return entry.term === 'II型糖尿病';
        },)?.renderings[0],).toBe('type 2 diabetes',);
      },
    },),
    it({
      name: 'REFUSES the dated Roman numeral',
      fn: async () => {
        expect(REFUSED_CANDIDATES.map(function kindOf(candidateText,): string {
          return validateTranslatedSlice({ sourceText: DIAGNOSED, candidateText, },).kind;
        },),).toEqual(['invalid', 'invalid',],);
      },
    },),
    it({
      name: 'PASSES "type 2 diabetes"',
      fn: async () => {
        expect(validateTranslatedSlice({ sourceText: DIAGNOSED, candidateText: ACCEPTED_CANDIDATE, },).kind,)
          .toBe('valid',);
      },
    },),
  ],
},);
