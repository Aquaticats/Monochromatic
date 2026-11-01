/**
 Guards the entry-content findings of the two glossary audits of 2026-09-27
 (ledger C3, C5, C6, C7, C10, R2, R4, R5, R7, R9, R11, R12 and R15): renderings
 that did not count once inflected ("to cure", "a transgender girl") or
 counted inside another word ("atrium"), whys that promised refusals the
 refused forms did not carry, sibling spellings with no entry, a term read
 inside a longer word, refusals that fired on ordinary English ("slid to the
 floor", "fossil-fuel car", "turned into in spring"), renderings that were
 another word's English ("dated", "intensive care"), and whys that wrote a
 form another entry refuses ("gaokao").

 Every refused form added here was replayed over the pinned archives and the
 settled pages before it landed: it fires only on the defect it names.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  RENDERING_GLOSSARY,
  renderingTermLines,
} from '@monochromatic-dev/translation-repair-rendering-audit';
import { HOUSE_POLICY_BLOCK, } from '@monochromatic-dev/translation-repair-house-rule';
import {
  COMMUNITY_GLOSSARY,
  communityRenderingDepartures,
  communityTermsIn,
  textCarriesForm,
} from '@monochromatic-dev/translation-repair-glossary';
import { validateTranslatedSlice, } from '../dist/final/node/index.mjs';

//region Fixtures

/**
 One floor case: an original, a rendering, and whether the floor accepts it.
 */
type FloorCase = readonly [
  label: string,
  sourceText: string,
  candidateText: string,
  expected: 'valid' | 'invalid',
];

/**
 Floor cases, each labelled by the ledger finding it guards.
 */
const FLOOR_CASES: readonly FloorCase[] = [
  ['C5 变娃 doll up', '猫从小就喜欢变娃。', 'The cat has loved to doll up since she was small.', 'invalid',],
  ['C5 变娃 dolls up', '猫每个周末都变娃。', 'The cat dolls up every weekend.', 'invalid',],
  ['C5 跨圈 crossdresser community', '猫在跨圈里交了很多朋友。', 'The cat made many friends in the crossdresser community.', 'invalid',],
  ['C5 跨圈 across communities', '猫在跨圈里交了很多朋友。', 'The cat made many friends across different communities.', 'invalid',],
  ['C5 跨圈 trans communities', '猫在跨圈里交了很多朋友。', 'The cat made many friends in trans communities.', 'valid',],
  ['C5 药娘 HRT girl', '那只猫是一个小药娘。', 'That cat was a little HRT girl.', 'invalid',],
  ['C5 药娘 medicine girl', '那只猫是一个小药娘。', 'That cat was a little medicine girl.', 'invalid',],
  ['C5 逆子 rebellious child', '黑猫被骂作「逆子」。', 'The black cat was called a “rebellious child”.', 'invalid',],
  ['C5 逆子 unfilial son', '黑猫被骂作「逆子」。', 'The black cat was called an “unfilial son”.', 'valid',],
  ['C7 跨性别圈 crossdressing community', '猫认识了跨性别圈的朋友。', 'The cat met friends from the crossdressing community.', 'invalid',],
  ['R2 燃油车 fossil-fuel car', '猫的车是一辆燃油车。', 'The cat’s car was a fossil-fuel car.', 'valid',],
  ['R2 燃油车 fuel car', '猫的车是一辆燃油车。', 'The cat’s car was a fuel car.', 'invalid',],
  ['R4 滑档 tears slid down', '猫滑档了，眼泪从脸上滑落。', 'The cat missed her chosen schools, and tears slid down her face.', 'valid',],
  ['R4 滑档 slid to the floor', '猫滑档了，瘫坐在地上。', 'The cat missed her chosen schools and slid to the floor.', 'valid',],
  ['R4 滑档 slipped into a mood', '猫滑档了，情绪低落。', 'The cat missed her chosen schools and slipped into a low mood.', 'valid',],
  ['R4 滑档 slipped down to a tier', '猫滑档了。', 'The cat slipped down to a second-tier school.', 'invalid',],
  ['R5 化作 turned into in spring', '那只蝴蝶，就是小猫在春天化作的。', 'That butterfly is what the kitten turned into in spring.', 'valid',],
  ['R7 矫正中心 correctional facility', '猫被送进了矫正中心。', 'The cat was sent to a correctional facility.', 'invalid',],
  ['R7 矫正中心 behaviour-correction centre', '猫被送进了矫正中心。', 'The cat was sent to a behaviour-correction centre.', 'valid',],
  ['R7 矫正学校 correctional school', '猫被送去一所矫正学校。', 'The cat was sent to a correctional school.', 'invalid',],
  ['R15 II型糖尿病 type II diabetic', '小猫患有II型糖尿病。', 'The kitten was a type II diabetic.', 'invalid',],
  ['R15 II型糖尿病 diabetes type II', '小猫患有II型糖尿病。', 'The kitten had diabetes type II.', 'invalid',],
  ['R15 以生命相逼 threatens her life', '猫妈妈以生命相逼。', 'The mother cat threatens her life.', 'invalid',],
];

/**
 One departure case: an original, a rendering, and whether the rendering is
 named as departing from the glossary's word.
 */
type DepartureCase = readonly [
  label: string,
  sourceText: string,
  candidateText: string,
  departs: boolean,
];

/**
 Departure cases, each labelled by the ledger finding it guards.
 */
const DEPARTURE_CASES: readonly DepartureCase[] = [
  ['C3 治愈 to cure', '这种病很难治愈。', 'This illness is hard to cure.', false,],
  ['C3 治愈 a healing cat', '猫可以治愈人。', 'A healing cat.', false,],
  ['C3 药娘 transgender girl', '猫是药娘。', 'The cat was a transgender girl.', false,],
  ['C3 变娃 becoming the doll', '猫今天变娃了。', 'Today the cat was becoming the doll.', false,],
  ['C3 治愈 soothing, a final e dropped', '可以治愈人的风景。', 'Soothing views everywhere.', false,],
  ['C3 治愈 cured, a d added', '猫的病治愈了。', 'The cat’s illness was cured.', false,],
  ['C3 跨圈 communities, a final y turned', '猫在跨圈里交了很多朋友。', 'The cat made friends in trans communities.', false,],
  ['C3 亚托莉 atrium', '亚托莉在猫舍。', 'The cat napped in the atrium.', true,],
];

/**
 Every entry of both glossaries.
 */
const ALL_ENTRIES = [
  ...COMMUNITY_GLOSSARY,
  ...RENDERING_GLOSSARY,
];

/**
 Terms a text carries, by their Han form.

 @param text - original to read

 @returns Terms present, in glossary order

 @example
 ```ts
 termsOf({ text: '她自切后', },); // => ['自切']
 ```
 */
function termsOf({ text, }: { readonly text: string; },): readonly string[] {
  return communityTermsIn({ text, },).map(function termOf(entry,): string {
    return entry.term;
  },);
}

//endregion Fixtures

await describe({
  name: 'glossary entry content (ledger C3 to C10, R2 to R15)',
  children: [
    it({
      name: 'FLOORS each rendering as the finding it guards requires',
      fn: async () => {
        expect(FLOOR_CASES.map(function verdictOf([label, sourceText, candidateText,],): string {
          return `${label}: ${validateTranslatedSlice({
            sourceText,
            candidateText,
          },).kind}`;
        },),).toEqual(FLOOR_CASES.map(function expectedOf({ 0: label, 3: expected, },): string {
          return `${label}: ${expected}`;
        },),);
      },
    },),
    it({
      name: 'NAMES A DEPARTURE only where the rendering lacks the word, inflection counted and other words not',
      fn: async () => {
        expect(DEPARTURE_CASES.map(function departureOf([label, sourceText, text,],): string {
          return `${label}: ${String(communityRenderingDepartures({
            sourceText,
            candidates: [{
              label: 'CANDIDATE 1',
              text,
            },],
          },).length > 0,)}`;
        },),).toEqual(DEPARTURE_CASES.map(function expectedOf({ 0: label, 3: departs, },): string {
          return `${label}: ${String(departs,)}`;
        },),);
      },
    },),
    it({
      name: 'LISTS NO RENDERING that is another word\'s English on the sheet lines: "dated" for 交往 on the one '
        + 'page where it never means dating (R11), "intensive care", the ward, for 抢救 (R12)',
      fn: async () => {
        // The rendering glossary's renderings reach the identity context only;
        // the departures block reads the community glossary. The first guard
        // of 2026-09-28 asked the departures for these two and could never
        // pass, so it was rewritten before the fix landed. Each term line reads
        // `- term: "rendering", ... (why)`, and the why may name a form to
        // avoid, so only the list before the why is read.
        /**
         Rendering list of each term line the two originals put on the sheet.
         */
        const renderingLists = [
          ...renderingTermLines({ text: '猫从没和狗交往过。', },),
          ...renderingTermLines({ text: '猫在医院抢救了三天。', },),
        ]
          .filter(function isTermLine(line,): boolean {
            return line.startsWith('- ',);
          },)
          .map(function renderingsOf(line,): string {
            return line.slice(
              0,
              line.indexOf(' (',),
            );
          },);
        expect(renderingLists,).toHaveLength(2,);
        expect(renderingLists.filter(function listsOtherWord(list,): boolean {
          return list.includes('"dated"',) || list.includes('"intensive care"',);
        },),).toEqual([],);
      },
    },),
    it({
      name: 'FINDS 爆柜 and 跨性别圈, and never 自切 inside 各自切, 亲自切 or 独自切 (C7, C10)',
      fn: async () => {
        expect(termsOf({ text: '猫上周爆柜了。', },),).toContain('爆柜',);
        expect(termsOf({ text: '猫认识了跨性别圈的朋友。', },),).toContain('跨性别圈',);
        expect(termsOf({ text: '她自切后休息了很久。', },),).toContain('自切',);
        expect([
          ...termsOf({ text: '我们各自切了一块鱼。', },),
          ...termsOf({ text: '猫亲自切了鱼。', },),
          ...termsOf({ text: '猫独自切了鱼。', },),
        ],).not
          .toContain('自切',);
      },
    },),
    it({
      name: 'STATES THE DOUBLED PREPOSITION as a house rule of grammar rather than a refusal keyed on 化作 (R5)',
      fn: async () => {
        expect(HOUSE_POLICY_BLOCK,).toContain('never turned into in',);
        expect(RENDERING_GLOSSARY.some(function isBecoming(entry,): boolean {
          return entry.term === '化作';
        },),).toBe(false,);
      },
    },),
    it({
      name: 'WRITES NO FORM ANOTHER ENTRY REFUSES in any why or rendering, save one it refuses itself (R9)',
      fn: async () => {
        /**
         Each why or rendering that carries another entry's refused form.
         */
        const clashes = ALL_ENTRIES.flatMap(function clashesOf(entry,): readonly string[] {
          return ALL_ENTRIES
            .filter(function isOther(other,): boolean {
              return other.term !== entry.term;
            },)
            .flatMap(function refusedBy(other,): readonly string[] {
              // A form the entry refuses too is one its why quotes to refuse
              // (跨圈 and 跨性别圈 share theirs), never its own English.
              return other.refusedForms
                .filter(function notOwnRefusal(form,): boolean {
                  return !entry.refusedForms.includes(form,);
                },)
                .filter(function carried(form,): boolean {
                  return [
                    entry.why,
                    ...entry.renderings,
                  ].some(function carries(text,): boolean {
                    return textCarriesForm({
                      text,
                      form,
                      end: 'plural',
                    },);
                  },);
                },)
                .map(function toClash(form,): string {
                  return `${entry.term} writes "${form}", which ${other.term} refuses`;
                },);
            },);
        },);
        expect(clashes,).toEqual([],);
      },
    },),
  ],
},);
