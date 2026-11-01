/**
 Guards class one hundred thirty-one (2026-09-25): under the owner's standing
 instruction of 2026-09-25 ("whenever you see anything that can be translated
 better do it"), words on one page join the rendering glossary. 辅导员 (whose
 calque reads as a therapist to an English reader), 矫正机构 (whose calque
 reads as a prison), 营救计划 and ICU 抢救 each shipped as a calque.

 The glossary audit of 2026-09-27 took out the class's entries keyed on one
 sentence's words (主动提出, 代替, 性格非常好) for the idiomatic English rule,
 seeded 抢救 as the word in place of ICU 抢救, and dropped 营救's refused
 forms, since a rescue operation is the English wherever the rescue is one
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
 Original in which the cat's university advisor helps it choose courses.
 */
const ADVISER = '猫的辅导员帮它选了下学期的课。';

/**
 Original in which the news reports a behaviour-correction centre closing.
 */
const CORRECTION = '新闻里说，那家矫正机构已经被关停了。';

/**
 Original in which the cats plan to rescue a kitten stuck in a tree.
 */
const FREE = '猫们商量了一个营救计划，要把树上的小猫接下来。';

/**
 Original in which the vet works through the night to save a weak kitten.
 */
const SAVED_KITTEN = '小猫出生时很虚弱，兽医连夜抢救了它。';

/**
 Terms class one hundred thirty-one seeds, as the audit left them.
 */
const SEEDED_TERMS = [
  '辅导员',
  '矫正机构',
  '营救',
  '抢救',
] as const;

/**
 Candidates the page's calques would ship, each with the original it renders.
 */
const REFUSED_CANDIDATES: readonly { readonly sourceText: string; readonly candidateText: string; }[] = [
  { sourceText: ADVISER, candidateText: 'The cat\'s counselor helped it pick next term\'s courses.', },
  { sourceText: CORRECTION, candidateText: 'The news said the correctional facility had been shut down.', },
];

/**
 Candidates carrying the English meaning, each with the original it renders.
 */
const ACCEPTED_CANDIDATES: readonly { readonly sourceText: string; readonly candidateText: string; }[] = [
  { sourceText: ADVISER, candidateText: 'The cat\'s student advisor helped it pick next term\'s courses.', },
  { sourceText: CORRECTION, candidateText: 'The news said the behaviour-correction centre had been shut down.', },
  { sourceText: FREE, candidateText: 'The cats worked out a rescue to bring the kitten down from the tree.', },
  { sourceText: SAVED_KITTEN, candidateText: 'The kitten was weak at birth, and the vet worked through the night to save it.', },
];

await describe({
  name: 'wording calques the rendering glossary refuses (class one hundred thirty-one)',
  children: [
    it({
      name: 'SEEDS 辅导员, 矫正机构, 营救 and 抢救',
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
