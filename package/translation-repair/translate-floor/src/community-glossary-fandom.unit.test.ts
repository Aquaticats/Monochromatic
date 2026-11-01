/**
 Guards class one hundred sixty (2026-09-26): under the
 owner's standing instruction of 2026-09-25 ("whenever you see anything that
 can be translated better do it"), three fandom words join the community
 glossary. 头壳, a kigurumi performer's head mask, shipped as "inside her
 head", the wearer's own head; 阿洛娜 and 亚托莉, the characters Arona of
 Blue Archive and Atri of ATRI -My Dear Moments-, shipped as the archive's
 "Alona and Atori", and no candidate on the run wrote the official names.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { COMMUNITY_GLOSSARY, } from '@monochromatic-dev/translation-repair-glossary';
import { validateTranslatedSlice, } from '../dist/final/node/index.mjs';
import { expectAllAccepted, expectAllRefused, } from './rendering-glossary-candidate-kinds.test-fixture.ts';

/**
 Original in which the kitten dances on stage in a kigurumi head.
 */
const HEAD = '小猫戴着头壳，在舞台上给小朋友们跳了一支舞。';

/**
 Original in which the kitten dresses as the two characters.
 */
const CHARACTERS = '小猫扮成了阿洛娜和亚托莉。';

/**
 Terms class one hundred sixty seeds.
 */
const SEEDED_TERMS = [
  '头壳',
  '阿洛娜',
  '亚托莉',
] as const;

/**
 Candidates the page's misreadings would ship, each with the original it renders.
 */
const REFUSED_CANDIDATES: readonly { readonly sourceText: string; readonly candidateText: string; }[] = [
  { sourceText: HEAD, candidateText: 'Inside her head, the kitten danced for the children on stage.', },
  { sourceText: CHARACTERS, candidateText: 'The kitten dressed up as Alona and Atori.', },
];

/**
 Candidates carrying the community's words, each with the original it renders.
 */
const ACCEPTED_CANDIDATES: readonly { readonly sourceText: string; readonly candidateText: string; }[] = [
  { sourceText: HEAD, candidateText: 'Inside the headpiece, the kitten danced for the children on stage.', },
  { sourceText: CHARACTERS, candidateText: 'The kitten dressed up as Arona and Atri.', },
  // CLASS ONE HUNDRED SIXTY-THREE (2026-09-26): the refused
  // form "inside her head" is the start of the rendering "inside her
  // headpiece", and every candidate for the slice was refused on it.
  { sourceText: HEAD, candidateText: 'Inside her headpiece, the kitten danced for the children on stage.', },
];

/**
 Candidates carrying a refused form beside an accepted rendering, so the
 rendering excuses no other occurrence.
 */
const MIXED_CANDIDATE = {
  sourceText: HEAD,
  candidateText: 'Inside her headpiece, and inside her head, the kitten danced.',
} as const;

await describe({
  name: 'fandom words the community glossary renders (class one hundred sixty)',
  children: [
    it({
      name: 'SEEDS 头壳, 阿洛娜 and 亚托莉',
      fn: async () => {
        expect(SEEDED_TERMS.filter(function isSeeded(term,): boolean {
          return COMMUNITY_GLOSSARY.some(function isTerm(entry,): boolean {
            return entry.term === term;
          },);
        },),).toEqual([...SEEDED_TERMS,],);
      },
    },),
    it({
      name: 'REFUSES the head read as the wearer\'s and the archive\'s transliterated names',
      fn: async () => {
        expectAllRefused({ candidates: REFUSED_CANDIDATES, },);
      },
    },),
    it({
      name: 'PASSES the headpiece and the official names',
      fn: async () => {
        expectAllAccepted({ candidates: ACCEPTED_CANDIDATES, },);
      },
    },),
    it({
      name: 'SEEDS 高性能机器人 as Atri\'s own words (class one hundred seventy-eight)',
      fn: async () => {
        // One entry (2026-09-26): the archive glossed the performer's
        // "high-performance robot" image away as a character she cosplayed;
        // the owner named the character: Atri, whose catchphrase it is.
        /**
         The seeded entry.
         */
        const entry = COMMUNITY_GLOSSARY.find(function isRobot(candidate,): boolean {
          return candidate.term === '高性能机器人';
        },);
        expect(entry?.renderings[0],).toBe('high-performance robot',);
        expect(entry?.why.includes('Atri',),).toBe(true,);
      },
    },),
    it({
      name: 'REFUSES a refused form standing apart even where the rendering appears elsewhere',
      fn: async () => {
        expect(validateTranslatedSlice(MIXED_CANDIDATE,).kind,).toBe('invalid',);
      },
    },),
  ],
},);
