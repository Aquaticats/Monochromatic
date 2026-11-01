/**
 Guards class one hundred twenty-seven (owner, 2026-09-25: "overdosing",
 "sailor uniform", "National College Entrance Examination"): the owner
 answered three renderings the shi_Yumiaoya27 read had left as written. OD
 shipped as "ODing", jk 裙 as "jk skirt" and 高考 as "Gaokao"; the rendering
 glossary seeds the owner's English and refuses the rest. MOD stays out by
 word boundaries (`glossary-match.unit.test.ts` holds a lowercase and a
 line-start OD), not by the leading space the term once carried.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { expectAllAccepted, expectAllRefused, } from './rendering-glossary-candidate-kinds.test-fixture.ts';

/**
 Original in which the cat overdoses day and night.
 */
const OVERDOSE = '这只猫没日没夜地 OD。';

/**
 Original in which the cat writes game mods.
 */
const MODS = '这只猫喜欢给游戏写 MOD。';

/**
 Original in which the cat's sailor uniform goes to a friend.
 */
const UNIFORM = '这只猫的 jk 裙留给了朋友。';

/**
 Original in which the cat sits the college entrance examination.
 */
const ENTRANCE = '这只猫的高考分数很高。';

/**
 Candidates the owner's answer refuses, each with the original it renders.
 */
const REFUSED_CANDIDATES: readonly { readonly sourceText: string; readonly candidateText: string; }[] = [
  { sourceText: OVERDOSE, candidateText: 'The cat was ODing day and night.', },
  { sourceText: UNIFORM, candidateText: 'The cat\'s JK skirt went to a friend.', },
  { sourceText: ENTRANCE, candidateText: 'The cat\'s Gaokao score was high.', },
];

/**
 Candidates carrying the owner's English, and a mod left as the original
 writes it, each with the original it renders.
 */
const ACCEPTED_CANDIDATES: readonly { readonly sourceText: string; readonly candidateText: string; }[] = [
  { sourceText: OVERDOSE, candidateText: 'The cat was overdosing day and night.', },
  { sourceText: MODS, candidateText: 'The cat liked writing game MODs.', },
  { sourceText: UNIFORM, candidateText: 'The cat\'s sailor uniform went to a friend.', },
  { sourceText: ENTRANCE, candidateText: 'The cat\'s National College Entrance Examination score was high.', },
];

await describe({
  name: 'the owner\'s renderings of OD, jk 裙 and 高考 (class one hundred twenty-seven)',
  children: [
    it({
      name: 'REFUSES "ODing", "JK skirt" and "Gaokao"',
      fn: async () => {
        expectAllRefused({ candidates: REFUSED_CANDIDATES, },);
      },
    },),
    it({
      name: 'ACCEPTS the owner\'s English and leaves MOD alone',
      fn: async () => {
        expectAllAccepted({ candidates: ACCEPTED_CANDIDATES, },);
      },
    },),
  ],
},);
