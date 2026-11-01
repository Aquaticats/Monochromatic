/**
 Guards class one hundred fifty-six (2026-09-26): under the
 owner's standing instruction of 2026-09-25 ("whenever you see anything that
 can be translated better do it"), three words from a student's life join the
 rendering glossary. 初中 shipped as "junior middle school" where four of the
 five archive passages that render it write "junior high school"; 自考 as
 "self-taught exams", which names no examination in English; 压力话 as
 "pressured remarks". The glossary seeds the English each should take and
 refuses the calques.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { expectAllAccepted, expectAllRefused, } from './rendering-glossary-candidate-kinds.test-fixture.ts';
import { seededAmong, } from '@monochromatic-dev/translation-repair-rendering-audit/ts/rendering-glossary-seeded.test-fixture.ts';

/**
 Original in which the cat joined the choir in junior high.
 */
const JUNIOR_HIGH = '猫在初中时加入了合唱团。';

/**
 Original in which the cat prepares for the self-study examinations.
 */
const SELF_STUDY = '猫一边打工，一边准备明年的自考。';

/**
 Original in which relatives nag the cat at New Year.
 */
const NAGGED = '每到过年，亲戚们总爱对猫说压力话。';

/**
 Terms class one hundred fifty-six seeds.
 */
const SEEDED_TERMS = [
  '初中',
  '自考',
  '压力话',
] as const;

/**
 Candidates the page's calques would ship, each with the original it renders.
 */
const REFUSED_CANDIDATES: readonly { readonly sourceText: string; readonly candidateText: string; }[] = [
  { sourceText: JUNIOR_HIGH, candidateText: 'The cat joined the choir in junior middle school.', },
  { sourceText: SELF_STUDY, candidateText: 'The cat worked part-time while preparing for next year\'s self-taught exams.', },
  { sourceText: NAGGED, candidateText: 'Every New Year the relatives loved to make pressured remarks to the cat.', },
];

/**
 Candidates carrying the English meaning, each with the original it renders.
 */
const ACCEPTED_CANDIDATES: readonly { readonly sourceText: string; readonly candidateText: string; }[] = [
  { sourceText: JUNIOR_HIGH, candidateText: 'The cat joined the choir in junior high school.', },
  { sourceText: SELF_STUDY, candidateText: 'The cat worked part-time while preparing for next year\'s self-study examinations.', },
  { sourceText: NAGGED, candidateText: 'Every New Year the relatives loved to pressure and nag the cat.', },
];

await describe({
  name: 'words from a student\'s life the rendering glossary renders (class one hundred fifty-six)',
  children: [
    it({
      name: 'SEEDS 初中, 自考 and 压力话',
      fn: async () => {
        expect(seededAmong({ terms: SEEDED_TERMS, },),).toEqual([...SEEDED_TERMS,],);
      },
    },),
    it({
      name: 'REFUSES the calques the page shipped',
      fn: async () => {
        expectAllRefused({ candidates: REFUSED_CANDIDATES, },);
      },
    },),
    it({
      name: 'PASSES the English meaning',
      fn: async () => {
        expectAllAccepted({ candidates: ACCEPTED_CANDIDATES, },);
      },
    },),
  ],
},);
