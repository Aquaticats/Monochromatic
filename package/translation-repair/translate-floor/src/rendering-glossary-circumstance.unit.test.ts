/**
 Guards class one hundred fifty-nine (aiyysk2, 2026-09-26): under the
 owner's standing instruction of 2026-09-25 ("whenever you see anything that
 can be translated better do it"), two terms join the rendering glossary.
 环境的问题 shipped as "the environment", which an English reader takes for
 nature and pollution where the speaker meant the place she lived; 工程机 as
 "engineering phone", which names nothing in English, where the phone world
 says "prototype" or "engineering sample". The glossary seeds the English
 each should take and refuses the calques. The glossary audit of 2026-09-27
 took 环境的问题 out, a word joined to a noun by one sentence, for the
 idiomatic English rule's "a word takes the sense its context gives it"
 (`glossary-dictionary-terms.unit.test.ts`).

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
 Original in which the cat finds a prototype phone that will not boot.
 */
const PROTOTYPE = '猫捡到一台工程机，开不了机。';

/**
 Terms class one hundred fifty-nine seeds.
 */
const SEEDED_TERMS = ['工程机',] as const;

/**
 Candidates the page's calques would ship, each with the original it renders.
 */
const REFUSED_CANDIDATES: readonly { readonly sourceText: string; readonly candidateText: string; }[] = [
  { sourceText: PROTOTYPE, candidateText: 'The cat found an engineering phone that would not boot.', },
];

/**
 Candidates carrying the English meaning, each with the original it renders.
 */
const ACCEPTED_CANDIDATES: readonly { readonly sourceText: string; readonly candidateText: string; }[] = [
  { sourceText: PROTOTYPE, candidateText: 'The cat found a prototype phone that would not boot.', },
];

await describe({
  name: 'circumstance and hardware words the rendering glossary renders (class one hundred fifty-nine)',
  children: [
    it({
      name: 'SEEDS 工程机',
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
