/**
 Guards class one hundred eighty-six (the whole-package audit of
 2026-09-27): every glossary match read terms, renderings and refused forms
 as raw substrings. A lowercase or line-start OD in the original was never
 seen, so "od" shipped untranslated on XingZ6010 and XingZ6014; "JK裙"
 written without a space was never seen; refused forms fired inside longer
 words ("atori" in "laboratories", "minor trans" in "minor transgressions")
 and at the head of an accepted rendering ("inside her head" before "mask");
 tone-marked pinyin escaped the refusal; a rendering counted inside an
 unrelated word ("cured" in "secured") and hid a departure, while an
 inflected rendering ("healing") named one; and a term inside an HTML comment
 named a departure the floor never held.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  communityRenderingDepartures,
  communityTermsIn,
} from '@monochromatic-dev/translation-repair-glossary';
import { validateTranslatedSlice, } from '../dist/final/node/index.mjs';

/**
 One floor case: an original and the rendering under the floor.
 */
type FloorCase = {
  /**
   Original passage.
   */
  readonly sourceText: string;

  /**
   Rendering under the floor.
   */
  readonly candidateText: string;
};

/**
 Renderings the floor must refuse, each with the original it renders.
 */
const REFUSED: readonly FloorCase[] = [
  { sourceText: '小猫讨厌 od，od 害猫。', candidateText: 'The kitten hated od, and od harms cats.', },
  { sourceText: 'OD 让小猫很难受。', candidateText: 'OD made the kitten very ill.', },
  { sourceText: '小猫又 OD 了。', candidateText: 'The kitten ODed again.', },
  { sourceText: '这只猫的JK裙留给了朋友。', candidateText: 'The cat\'s JK skirt went to a friend.', },
  { sourceText: '小猫戴着头壳，闷在里面。', candidateText: 'The kitten wore it, stuffy inside her head.', },
  { sourceText: '小猫未成年时犯过一些小错。', candidateText: 'The kitten was a minor trans girl who erred.', },
  { sourceText: '猫是药娘。', candidateText: 'The cat was a Yào Niáng.', },
  { sourceText: '猫是药娘。', candidateText: 'The cats were yaoniangs.', },
  { sourceText: '小猫是个小药娘。', candidateText: 'The kitten was a little xiaoyaoniang.', },
  { sourceText: '小猫未成年时犯过一些小错。', candidateText: 'The kitten, a minor transgender girl, erred.', },
  { sourceText: '小猫被精神霸凌。', candidateText: 'The kitten was spiritually bullied.', },
];

/**
 Renderings the floor must accept, each with the original it renders.
 */
const ACCEPTED: readonly FloorCase[] = [
  { sourceText: '小猫又 OD 了。', candidateText: 'The odd kitten overdosed again.', },
  { sourceText: '小猫戴着头壳。', candidateText: 'Inside her head mask the kitten purred.', },
  { sourceText: '小猫戴着头壳。', candidateText: 'Inside her head-mask the kitten purred.', },
  { sourceText: '小猫戴着头壳。', candidateText: 'Inside her headgear the kitten purred.', },
  { sourceText: '亚托莉在猫舍午睡。', candidateText: 'Atri napped near the laboratories.', },
  { sourceText: '阿洛娜来自猫岛。', candidateText: 'Arona came from Badalona.', },
  {
    sourceText: '小猫未成年时犯过一些小错。',
    candidateText: 'As a minor the kitten committed a few minor transgressions.',
  },
];

/**
 Each case's verdict beside its rendering, so a failure names the case.

 @param cases - floor cases to run

 @returns One `kind: rendering` line per case

 @example
 ```ts
 verdictsOf({ cases: REFUSED, },);
 ```
 */
function verdictsOf(
  { cases, }: { readonly cases: readonly FloorCase[]; },
): readonly string[] {
  return cases.map(function verdictOf(floorCase,): string {
    return `${validateTranslatedSlice(floorCase,).kind}: ${floorCase.candidateText}`;
  },);
}

/**
 Departure lines for one candidate under one original.

 @param sourceText - original the candidate renders

 @param text - candidate the judge is shown

 @returns Departure lines

 @example
 ```ts
 departuresOf({ sourceText: '猫被治愈。', text: 'The cat was healed.', },);
 ```
 */
function departuresOf(
  {
    sourceText,
    text,
  }: {
    readonly sourceText: string;
    readonly text: string;
  },
): readonly string[] {
  return communityRenderingDepartures({
    sourceText,
    candidates: [{ label: 'CANDIDATE 1', text, },],
  },);
}

await describe({
  name: 'glossary matching at word boundaries (class one hundred eighty-six)',
  children: [
    it({
      name: 'REFUSES a kept OD in any case or position, a kept JK裙 without its space, a refused form '
        + 'before another word, tone-marked pinyin and a plural refused form',
      fn: async () => {
        expect(verdictsOf({ cases: REFUSED, },),).toEqual(REFUSED.map(function invalid(floorCase,): string {
          return `invalid: ${floorCase.candidateText}`;
        },),);
      },
    },),
    it({
      name: 'ACCEPTS "odd", a refused form opening an accepted rendering with a space or a hyphen, and a '
        + 'refused form inside a longer word',
      fn: async () => {
        expect(verdictsOf({ cases: ACCEPTED, },),).toEqual(ACCEPTED.map(function valid(floorCase,): string {
          return `valid: ${floorCase.candidateText}`;
        },),);
      },
    },),
    it({
      name: 'NAMES A DEPARTURE where a rendering stands only inside another word ("cured" in "secured")',
      fn: async () => {
        expect(departuresOf({ sourceText: '猫被治愈。', text: 'The cat was comforted and secured a nap.', },),)
          .toHaveLength(1,);
      },
    },),
    it({
      name: 'NAMES NO DEPARTURE for an inflected rendering ("healing") or a term only a comment carries',
      fn: async () => {
        expect(departuresOf({ sourceText: '猫可以治愈人。', text: 'A healing cat.', },),).toEqual([],);
        expect(departuresOf({ sourceText: '<!-- 猫被治愈 -->猫在睡觉。', text: 'The cat slept.', },),).toEqual([],);
        expect(departuresOf({ sourceText: '猫被治愈。', text: 'The cat slept.', },),).toHaveLength(1,);
      },
    },),
    it({
      name: 'FINDS NO OCCURRENCE of a term that is empty, which every text would otherwise be read to carry at every '
        + 'index, and still finds a term that is written',
      fn: async () => {
        /**
         Term that is empty, which no text carries.
         */
        const empty = {
          term: '',
          renderings: ['nothing',],
          refusedForms: [],
          why: 'an empty term',
        };
        /**
         Term that is written.
         */
        const written = {
          term: '猫',
          renderings: ['cat',],
          refusedForms: [],
          why: 'the animal',
        };
        expect(communityTermsIn({
          text: '猫被治愈。',
          glossary: [empty, written,],
        },),).toEqual([written,],);
      },
    },),
  ],
},);
