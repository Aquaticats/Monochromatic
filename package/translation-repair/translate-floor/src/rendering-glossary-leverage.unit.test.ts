/**
 Guards class one hundred fifty-seven (aiyysk1, 2026-09-26): under the
 owner's standing instruction of 2026-09-25 ("whenever you see anything that
 can be translated better do it"), 以生命相逼 joins the rendering glossary. The
 friend who pressed with her own life, threatening to kill herself if the
 other self-harmed, shipped as "threatened her life", which tells an English
 reader she threatened the other's life. The glossary seeds English that
 names whose life is staked and refuses the reversal.

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
 Original in which the old cat stakes her own life to stop the kitten.
 */
const LEVERAGE = '老猫以生命相逼，不许小猫再爬屋顶。';

await describe({
  name: 'the threat the rendering glossary renders (class one hundred fifty-seven)',
  children: [
    it({
      name: 'SEEDS 以生命相逼 with "threatened to take her own life" first',
      fn: async () => {
        /**
         The seeded entry.
         */
        const entry = RENDERING_GLOSSARY.find(function isTerm(candidate,): boolean {
          return candidate.term === '以生命相逼';
        },);
        expect(entry?.renderings[0],).toBe('threatened to take her own life',);
      },
    },),
    it({
      name: 'REFUSES "threatened her life" and passes the staked own life',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: LEVERAGE,
          candidateText: 'The old cat threatened her life to keep the kitten off the roof.',
        },).kind,).toBe('invalid',);
        expect(validateTranslatedSlice({
          sourceText: LEVERAGE,
          candidateText: 'The old cat threatened to take her own life if the kitten climbed the roof again.',
        },).kind,).toBe('valid',);
      },
    },),
  ],
},);
