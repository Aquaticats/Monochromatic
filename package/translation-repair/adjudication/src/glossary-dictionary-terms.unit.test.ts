/**
 Guards the glossary audit of 2026-09-27 (owner, on class one hundred
 eighty-three: "'kigurumi的记忆结束' - this isn't really a dictionary keyword
 though?", then "Also check if other items in the glossary have the same
 problems and fix them too."). The rendering glossary holds dictionary terms:
 words, set phrases, names and the community's vocabulary. Entries keyed on a
 fragment of one sentence (a construction, a pronoun, a clause's grammar) came
 out, and the lessons they carried became general house rules on every sheet:
 idiomatic English over the Chinese construction, grammatical English, a
 credit's maker named with "by", and a game's jargon in that game's English.
 Words kept in the glossary no longer refuse forms that are the right English
 for the same word elsewhere (交往 of a romance, 喘不过气 after a run).

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { RENDERING_GLOSSARY, } from '@monochromatic-dev/translation-repair-rendering-audit';
import { HOUSE_POLICY_BLOCK, } from '@monochromatic-dev/translation-repair-house-rule';
import {
  buildTranslateMessages,
  validateTranslatedSlice,
} from '@monochromatic-dev/translation-repair-translate-floor';
import { buildCandidateSelectMessages, } from '../dist/final/node/index.mjs';
import { seededAmong, } from '@monochromatic-dev/translation-repair-rendering-audit/ts/rendering-glossary-seeded.test-fixture.ts';
import { systemOf, } from '@monochromatic-dev/translation-repair-provider/ts/chat-message-reading.test-fixture.ts';

/**
 Rule replacing the entries keyed on one sentence's construction.
 */
const IDIOMATIC_ENGLISH =
  'The English says what the ORIGINAL means the way an English writer would say it, never the Chinese construction '
    + 'word for word. A rendering that follows the Chinese construction into English no one writes is a mistranslation '
    + 'however faithful each word looks: a stock phrase takes the English stock phrase for the same meaning; a word takes '
    + 'the sense its context gives it rather than its first dictionary gloss (the place someone lived is their '
    + 'surroundings, not the environment); a word the Chinese construction needs and English does not (相关 before a '
    + 'noun, 巨大的 before an effect) is said plainly or left out; and an abstract noun the Chinese makes a subject or '
    + 'object becomes the verb or adjective English uses (it had many causes, not there were many sides to the cause; '
    + 'she was good-natured, not a person of such a good nature). The English already on the page carrying such a '
    + 'construction is a mistranslation to correct, not a reading to keep.';

/**
 Rule replacing the entries keyed on one sentence's grammar slip.
 */
const GRAMMATICAL_ENGLISH =
  'Every sentence is grammatical English: a verb after make or let is bare (made her meet, never made her met), a verb\'s '
    + 'preposition is written once before its object (turned into a small box, never turned into in a small box), a tag '
    + 'question matches its clause (she deserved better, didn\'t she), a phrase attaches to the noun it describes, and a '
    + 'pronoun keeps the speaker\'s own point of view (when I leave you all, never when I leave us).';

/**
 Rule replacing the entry keyed on a credit line's comma.
 */
const CREDIT_BY =
  'A credit naming who made a work (作者, 词 or 曲 before a name) reads by, lyrics by or music by before the name, '
    + 'never ", author" before it.';

/**
 Rule replacing the entry keyed on one card-game line.
 */
const GAME_JARGON =
  'A word used as a game\'s or a hobby\'s jargon takes that game\'s English term, never its everyday sense (in UNO, 加 '
    + 'is stacking the draw cards on the next player, not adding).';

/**
 Every general rule the audit adds.
 */
const GENERAL_RULES = [
  IDIOMATIC_ENGLISH,
  GRAMMATICAL_ENGLISH,
  CREDIT_BY,
  GAME_JARGON,
] as const;

/**
 Entries keyed on one sentence's words, which the glossary no longer holds.
 */
const SENTENCE_PATCHES = [
  '用这种方式',
  '原因是多方面的',
  '特例',
  '陷入癫狂',
  '环境的问题',
  '相关医院',
  '巨大的影响',
  '主动提出',
  'ICU 抢救',
  '代替',
  '性格非常好',
  '人生中的第一颗',
  '留下了巨大的创伤',
  '，作者',
  '被她治愈',
  '应该会有更好的生活',
  '遇到的却是',
  '在隙中',
  '一切都会有机会',
  '离开我们的时候',
  '所以她是个',
  'uno，真的加了很多',
  'kigurumi的记忆结束',
] as const;

/**
 Renderings that are the right English for a kept word in another context,
 each with an original giving it that context.
 */
const CONTEXT_RIGHT: readonly { readonly sourceText: string; readonly candidateText: string; }[] = [
  { sourceText: '那只公猫和她们都交往过。', candidateText: 'The tomcat had dated them both.', },
  { sourceText: '猫追了一路老鼠，跑得喘不过气。', candidateText: 'The cat chased the mouse all the way and was out of breath.', },
  { sourceText: '纸上写满了密密麻麻的字。', candidateText: 'The paper was covered in densely packed writing.', },
  { sourceText: '这三只猫是小区里的三剑客。', candidateText: 'These three cats were the Three Musketeers of the neighbourhood.', },
  { sourceText: '消防员对树上的猫展开了营救行动。', candidateText: 'The firefighters launched a rescue operation for the cat in the tree.', },
];

/**
 Rules a sheet's system text lacks.

 @param system - sheet's joined system text

 @returns General rules absent from it, empty when it carries all of them

 @example
 ```ts
 const missing = rulesMissingFrom({ system, },);
 ```
 */
function rulesMissingFrom({ system, }: { readonly system: string; },): readonly string[] {
  return GENERAL_RULES.filter(function isMissing(rule,): boolean {
    return !system.includes(rule,);
  },);
}

await describe({
  name: 'the glossary holds dictionary terms; one sentence\'s lesson is a general rule (glossary audit, 2026-09-27)',
  children: [
    it({
      name: 'the house policy states every general rule',
      fn: async () => {
        expect(rulesMissingFrom({ system: HOUSE_POLICY_BLOCK, },),).toEqual([],);
      },
    },),
    it({
      name: 'the translate sheet carries every general rule',
      fn: async () => {
        expect(rulesMissingFrom({
          system: systemOf({
            messages: buildTranslateMessages({ sourceText: '猫在窗台上睡觉。', existingText: 'The cat sleeps on the sill.', },)
              .messages,
          },),
        },),).toEqual([],);
      },
    },),
    it({
      name: 'the select sheet carries every general rule',
      fn: async () => {
        expect(rulesMissingFrom({
          system: systemOf({
            messages: buildCandidateSelectMessages({
              task: 'Each candidate is a rendering of the passage below.',
              criteria: ['Complete coverage: every proposition of the ORIGINAL is rendered.',],
              evidence: [{ label: 'ORIGINAL (Chinese)', text: '猫在窗台上睡觉。', },],
              rendered: ['The cat slept on the sill.', 'The cat sleeps on the sill.',],
            },),
          },),
        },),).toEqual([],);
      },
    },),
    it({
      name: 'LEAVES no entry keyed on one sentence\'s words',
      fn: async () => {
        expect(seededAmong({ terms: SENTENCE_PATCHES, },),).toEqual([],);
      },
    },),
    it({
      name: 'SEEDS 抢救 as the word, where the glossary held only ICU 抢救',
      fn: async () => {
        expect(RENDERING_GLOSSARY.find(function isTerm(entry,): boolean {
          return entry.term === '抢救';
        },)?.renderings[0],).toBe('emergency treatment',);
      },
    },),
    it({
      name: 'ACCEPTS the English a kept word takes in another context',
      fn: async () => {
        expect(CONTEXT_RIGHT.map(function kindOf(candidate,): string {
          return validateTranslatedSlice(candidate,).kind;
        },),).toEqual(CONTEXT_RIGHT.map(function valid(): string {
          return 'valid';
        },),);
      },
    },),
  ],
},);
