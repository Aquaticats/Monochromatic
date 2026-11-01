/**
 Guards class one hundred thirty-four (2026-09-25) on the sheets: the house
 policy's Canadian English bullet names the date order and the licorice
 spelling, and 药代 (a person who sells medication on others' behalf) is
 seeded, since one page read it as a pharmaceutical sales representative, the
 general sense of the abbreviation, where the story means she sold medication.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { RENDERING_GLOSSARY, } from '@monochromatic-dev/translation-repair-rendering-audit';
import { HOUSE_POLICY_BLOCK, } from '@monochromatic-dev/translation-repair-house-rule';
import { validateTranslatedSlice, } from '../dist/final/node/index.mjs';

/**
 Clause naming the date order, read off the house policy verbatim.
 */
const DATE_RULE = 'Dates are written month first (April 29, May 4, March 13, 2024), never day first (29th April, 4 May).';

/**
 Original in which the cat's years selling medication take her to many pharmacies.
 */
const MEDICATION_SELLER = '猫做药代的那几年，每天都要跑好几家药店。';

await describe({
  name: 'dates, licorice and 药代 on the sheets (class one hundred thirty-four)',
  children: [
    it({
      name: 'the house policy names the date order',
      fn: async () => {
        expect(HOUSE_POLICY_BLOCK,).toContain(DATE_RULE,);
      },
    },),
    it({
      name: 'the house policy names licorice',
      fn: async () => {
        expect(HOUSE_POLICY_BLOCK,).toContain('licorice',);
      },
    },),
    it({
      name: 'SEEDS 药代',
      fn: async () => {
        expect(RENDERING_GLOSSARY.some(function isTerm(entry,): boolean {
          return entry.term === '药代';
        },),).toBe(true,);
      },
    },),
    it({
      name: 'REFUSES the sales-representative calque and ACCEPTS the seller',
      fn: async () => {
        expect([
          validateTranslatedSlice({
            sourceText: MEDICATION_SELLER,
            candidateText: 'In her years as a pharmaceutical sales representative, the cat visited several pharmacies every day.',
          },).kind,
          validateTranslatedSlice({
            sourceText: MEDICATION_SELLER,
            candidateText: 'In the years she spent selling medication, the cat visited several pharmacies every day.',
          },).kind,
        ],).toEqual(['invalid', 'valid',],);
      },
    },),
  ],
},);
