/**
 Guards class one hundred eighty-four (2026-09-27): under the
 owner's standing instruction of 2026-09-25 ("whenever you see anything that
 can be translated better do it"), two words of the kigurumi and fan
 community join the fandom glossary. 变娃 (the performers' word for putting on
 the costume and becoming the doll) shipped on one page as a game of becoming
 a doll, and English "doll up" means dressing smartly. 治愈 (the fan sense of
 healing, as in 治愈系) first led with "comforted"; the owner wrote "I kinda
 disagree here" (2026-09-27), so "healed" leads ("healing" is the fandom's own
 English, and the archive reads "healed"; those reasons are the change's, not
 the owner's, ledger H3). The contrast with 安慰 that this guard, the entry and
 the handover once gave the owner was a misquotation: the page writes 安抚,
 never 安慰 (ledger C6).

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  COMMUNITY_GLOSSARY,
  communityTermsIn,
} from '@monochromatic-dev/translation-repair-glossary';
import { validateTranslatedSlice, } from '../dist/final/node/index.mjs';

/**
 Original in which the cat puts on its kigurumi for the first time at a convention.
 */
const SUITED = '周末的漫展上，猫第一次变娃娃，还拍了很多照片。';

/**
 Original in which the cat's purring heals a sleepless kitten.
 */
const HEALED = '那只总是失眠的小猫，被它治愈了。';

/**
 First rendering the glossary seeds for a term.

 @param term - Han form to look up

 @returns Leading rendering the glossary offers for the term

 @throws Error when the glossary lacks the term or seeds it with no rendering,
 which is the failure this guard exists to show

 @example
 ```ts
 const first = firstRendering({ term: '变娃', },);
 ```
 */
function firstRendering({ term, }: { readonly term: string; },): string {
  /**
   Leading rendering of the seeded entry, if any.
   */
  const [first,] = COMMUNITY_GLOSSARY.find(function isTerm(entry,): boolean {
    return entry.term === term;
  },)?.renderings ?? [];
  if (first === undefined)
    throw new Error(`community glossary does not seed ${term}`,);
  return first;
}

await describe({
  name: 'kigurumi and fan words the community glossary renders (class one hundred eighty-four)',
  children: [
    it({
      name: 'SEEDS 变娃 with "put on the kigurumi" first and 治愈 with "healed" first, with no contrast to 安慰, '
        + 'a word the kigurumi page never writes (ledger C6: it writes 安抚)',
      fn: async () => {
        expect(firstRendering({ term: '变娃', },),).toBe('put on the kigurumi',);
        expect(firstRendering({ term: '治愈', },),).toBe('healed',);
        expect(COMMUNITY_GLOSSARY.find(function isTerm(entry,): boolean {
          return entry.term === '治愈';
        },)?.why,).not
          .toContain('安慰',);
      },
    },),
    it({
      name: 'FINDS 变娃 inside 变娃娃 and 治愈 inside 被它治愈',
      fn: async () => {
        expect(communityTermsIn({ text: SUITED, },).map(function termOf(entry,): string {
          return entry.term;
        },),).toContain('变娃',);
        expect(communityTermsIn({ text: HEALED, },).map(function termOf(entry,): string {
          return entry.term;
        },),).toContain('治愈',);
      },
    },),
    it({
      name: 'REFUSES "dolled up", the English idiom for dressing smartly',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: SUITED,
          candidateText: 'At the weekend comic convention, the cat got all dolled up for the first time and took lots of photos.',
        },).kind,).toBe('invalid',);
      },
    },),
    it({
      name: 'PASSES the kigurumi sense and the healing sense',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: SUITED,
          candidateText: 'At the weekend comic convention, the cat put on its kigurumi for the first time and took lots of photos.',
        },).kind,).toBe('valid',);
        expect(validateTranslatedSlice({
          sourceText: HEALED,
          candidateText: 'The kitten who could never sleep was healed by it.',
        },).kind,).toBe('valid',);
      },
    },),
  ],
},);
