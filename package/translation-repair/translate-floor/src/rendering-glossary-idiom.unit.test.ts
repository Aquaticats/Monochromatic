/**
 Guards class one hundred thirty (2026-09-25): under the owner's standing
 instruction of 2026-09-25 ("whenever you see anything that can be translated
 better do it"), nine more phrasings on one page join the rendering glossary.
 三剑客, 燃油车, 喘不过气, 密密麻麻, 志愿填写, 命运的齿轮, 相关医院, 巨大的影响 and
 贴贴计划 each shipped as a calque; the glossary seeds the English each should
 take and refuses the calques.

 The glossary audit of 2026-09-27 took out 相关医院 and 巨大的影响 (ordinary
 words joined by one sentence) for the idiomatic English rule, and dropped
 the refused forms of 三剑客, 喘不过气 and 密密麻麻 that are the right English
 for the same word elsewhere (`glossary-dictionary-terms.unit.test.ts`).

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
 Original in which the cat watches the trio's car show every Saturday.
 */
const TRIO = '猫每周六都守在电视前看三剑客的节目。';

/**
 Original in which a neighbour's car keeps the cat awake.
 */
const GAS_CAR = '猫说邻居那辆旧燃油车吵得它睡不着。';

/**
 Original in which a day of chasing butterflies tires the cat.
 */
const CRUSHING = '追了一整天蝴蝶，猫累得喘不过气。';

/**
 Original in which the cat's desk is covered in notes.
 */
const NOTES = '猫的书桌上堆满了密密麻麻的笔记。';

/**
 Original in which the cat asks for help with its university applications.
 */
const APPLICATIONS = '猫请老师帮它检查志愿填写。';

/**
 Original in which fate turns for the cat at the attic door.
 */
const FATE = '那天猫推开了阁楼的门，命运的齿轮开始转动。';

/**
 Original in which the cat maps the neighbourhood for its cuddle meetups.
 */
const CUDDLES = '猫为贴贴计划画了一张街区地图。';

/**
 Terms class one hundred thirty seeds.
 */
const SEEDED_TERMS = [
  '三剑客',
  '燃油车',
  '喘不过气',
  '密密麻麻',
  '志愿填写',
  '命运的齿轮',
  '贴贴计划',
] as const;

/**
 Candidates the page's calques would ship, each with the original it renders.
 */
const REFUSED_CANDIDATES: readonly { readonly sourceText: string; readonly candidateText: string; }[] = [
  { sourceText: GAS_CAR, candidateText: 'The cat said the neighbour\'s old fuel-powered car kept it awake.', },
  { sourceText: APPLICATIONS, candidateText: 'The cat asked its teacher to check its application preferences.', },
  { sourceText: FATE, candidateText: 'That day the cat pushed open the attic door, and the gears of fate began to turn.', },
  { sourceText: CUDDLES, candidateText: 'The cat drew a neighbourhood map for its cuddling plan.', },
];

/**
 Candidates carrying the English meaning, each with the original it renders.
 */
const ACCEPTED_CANDIDATES: readonly { readonly sourceText: string; readonly candidateText: string; }[] = [
  { sourceText: TRIO, candidateText: 'Every Saturday the cat sat by the TV for the Top Gear trio\'s car show.', },
  { sourceText: GAS_CAR, candidateText: 'The cat said the neighbour\'s old gas-powered car kept it awake.', },
  { sourceText: CRUSHING, candidateText: 'After chasing butterflies all day, the cat was out of breath.', },
  { sourceText: NOTES, candidateText: 'The cat\'s desk was covered in notes.', },
  { sourceText: APPLICATIONS, candidateText: 'The cat asked its teacher to check its university applications.', },
  { sourceText: FATE, candidateText: 'That day the cat pushed open the attic door, and the wheels of fate began to turn.', },
  { sourceText: CUDDLES, candidateText: 'The cat drew a neighbourhood map for its cuddle meetups.', },
];

await describe({
  name: 'idiom calques the rendering glossary refuses (class one hundred thirty)',
  children: [
    it({
      name: 'SEEDS 三剑客, 燃油车, 喘不过气, 密密麻麻, 志愿填写, 命运的齿轮 and 贴贴计划',
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
