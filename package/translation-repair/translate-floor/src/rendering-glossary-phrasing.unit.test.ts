/**
 Guards class one hundred twenty-nine (2026-09-25): under the owner's
 standing instruction of 2026-09-25 ("whenever you see anything that can be
 translated better do it"), 摆烂 and 万千世界, both shipped as calques on one
 page, join the rendering glossary.

 The class seeded three more entries keyed on one sentence's construction.
 The glossary audit of 2026-09-27 took them out for the idiomatic English
 rule on every sheet, which `glossary-dictionary-terms.unit.test.ts` guards,
 and which names them.

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
 Original in which the cat stops resisting bath time.
 */
const GAVE_UP = '一到洗澡时间，猫就开始摆烂。';

/**
 Original in which the cat on the windowsill eyes the world every day.
 */
const WIDE_WORLD = '窗台上的猫每天都好奇地打量着这万千世界。';

/**
 Terms class one hundred twenty-nine seeds, as the audit left them.
 */
const SEEDED_TERMS = [
  '摆烂',
  '万千世界',
] as const;

/**
 Candidates the page's calques would ship, each with the original it renders.
 */
const REFUSED_CANDIDATES: readonly { readonly sourceText: string; readonly candidateText: string; }[] = [
  { sourceText: GAVE_UP, candidateText: 'Whenever bath time came, the cat\'s attitude turned to one of giving up.', },
  { sourceText: GAVE_UP, candidateText: 'Whenever bath time came, the cat slacked off.', },
  { sourceText: WIDE_WORLD, candidateText: 'Every day the cat on the windowsill eyed this myriad world with curiosity.', },
];

/**
 Candidates carrying the English meaning, each with the original it renders.
 */
const ACCEPTED_CANDIDATES: readonly { readonly sourceText: string; readonly candidateText: string; }[] = [
  { sourceText: GAVE_UP, candidateText: 'Whenever bath time came, the cat just gave up.', },
  { sourceText: WIDE_WORLD, candidateText: 'Every day the cat on the windowsill eyed this vast world with curiosity.', },
];

await describe({
  name: 'phrasing calques the rendering glossary refuses (class one hundred twenty-nine)',
  children: [
    it({
      name: 'SEEDS 摆烂 and 万千世界',
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
      name: 'ACCEPTS the English meaning',
      fn: async () => {
        expectAllAccepted({ candidates: ACCEPTED_CANDIDATES, },);
      },
    },),
  ],
},);
