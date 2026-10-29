/**
 Guards class one hundred twenty-six (2026-09-25): under the owner's standing
 instruction of 2026-09-25 ("whenever you see anything that can be translated
 better do it"), five renderings on one page join the rendering glossary.
 年级组长, 未成年 (whose calque reads as "unimportant"), 骨灰骰子, 同居者 (whose
 calque names a romantic partner) and 精神霸凌 (shipped with its adjective
 dropped) each shipped wrong; the glossary seeds the English each should take
 and refuses the calques.

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
 Original in which the head of the cat's year gives it a paper flower.
 */
const YEAR_HEAD = '年级组长给这只猫发了一朵小红花。';

/**
 Original in which the cat shields an underage trans girl from bullies online.
 */
const UNDERAGE = '这只猫在网上替一位未成年药娘挡下了精神霸凌。';

/**
 Original in which the cat sees memorial dice in a shop window.
 */
const ASHES = '猫在纪念品店的橱窗里第一次见到骨灰骰子。';

/**
 Original in which the cat's housemate feeds it every morning.
 */
const HOUSEMATE = '这只猫的同居者每天早上给它开罐头。';

/**
 Terms class one hundred twenty-six seeds.
 */
const SEEDED_TERMS = [
  '年级组长',
  '未成年',
  '骨灰骰子',
  '同居者',
  '精神霸凌',
] as const;

/**
 Candidates the page's calques would ship, each with the original it renders.
 */
const REFUSED_CANDIDATES: readonly { readonly sourceText: string; readonly candidateText: string; }[] = [
  { sourceText: YEAR_HEAD, candidateText: 'The grade leader gave the cat a little red paper flower.', },
  { sourceText: UNDERAGE, candidateText: 'Online, the cat shielded a minor trans girl from psychological bullying.', },
  { sourceText: UNDERAGE, candidateText: 'Online, the cat shielded an underage trans girl from spiritual bullying.', },
  { sourceText: ASHES, candidateText: 'The cat first saw ash dice in the window of a keepsake shop.', },
  { sourceText: HOUSEMATE, candidateText: 'The cat\'s cohabitant opened a can for it every morning.', },
];

/**
 Candidates carrying the English meaning, each with the original it renders.
 */
const ACCEPTED_CANDIDATES: readonly { readonly sourceText: string; readonly candidateText: string; }[] = [
  { sourceText: YEAR_HEAD, candidateText: 'The grade coordinator gave the cat a little red paper flower.', },
  { sourceText: UNDERAGE, candidateText: 'Online, the cat shielded an underage trans girl from psychological bullying.', },
  { sourceText: ASHES, candidateText: 'The cat first saw memorial dice made from the ashes in the window of a keepsake shop.', },
  { sourceText: HOUSEMATE, candidateText: 'The cat\'s housemate opened a can for it every morning.', },
];

await describe({
  name: 'school-life and memorial calques the rendering glossary refuses (class one hundred twenty-six)',
  children: [
    it({
      name: 'SEEDS 年级组长, 未成年, 骨灰骰子, 同居者 and 精神霸凌',
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
