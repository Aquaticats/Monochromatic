/**
 Guards class one hundred thirty-two (owner, 2026-09-25: "The convention is
 and should be en_CA. It should have been documented and in the prompts."):
 no sheet named the English the page is written in, so the bench mixed
 varieties ("counselor", "head of year", "a petrol car") and the glossaries
 seeded British and American forms as the renderings to use. The house
 policy now says the page is Canadian English with its spelling and
 vocabulary, every sheet carries it, the policy's own wording is Canadian,
 and no glossary rendering is British or American where Canadian differs.

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
import { COMMUNITY_GLOSSARY, } from '@monochromatic-dev/translation-repair-glossary';
import {
  buildTranslateMessages,
  validateTranslatedSlice,
} from '@monochromatic-dev/translation-repair-translate-floor';
import { buildCandidateSelectMessages, } from '../dist/final/node/index.mjs';
import { systemOf, } from '@monochromatic-dev/translation-repair-provider/ts/chat-message-reading.test-fixture.ts';

/**
 Clause the rule opens on, read off the house policy verbatim so a rewording
 that drops the variety fails here.
 */
const CANADIAN_RULE = 'The page is written in Canadian English (en_CA), in spelling and in vocabulary.';

/**
 British spellings the house policy itself used before the rule, which a
 Canadian page writes with -ize.
 */
const POLICY_BRITISH_FORMS = ['romanised', 'romanisation', 'neutralise', 'localisation',] as const;

/**
 Words a Canadian page does not use where Canadian English has its own:
 British vocabulary, American spellings, and the spelling of a role the
 Canadian style guides write the other way.
 */
const NON_CANADIAN_WORDS = [
  'petrol',
  'flatmate',
  'maths',
  'mum',
  'behavior',
  'center',
  'color',
  'favorite',
  'counselor',
  'adviser',
] as const;

/**
 Phrases a Canadian school does not use for its grade structure.
 */
const NON_CANADIAN_PHRASES = ['head of year', 'year head',] as const;

/**
 Letters a rendering's words are made of, once lowercased.
 */
const LOWERCASE_LETTERS = 'abcdefghijklmnopqrstuvwxyz';

/**
 Cat-themed source, since neither sheet varies with what it is given.
 */
const SOURCE_TEXT = '猫在窗台上睡觉。';

/**
 Original in which a neighbour's car keeps the cat awake.
 */
const GAS_CAR = '猫说邻居那辆旧燃油车吵得它睡不着。';

/**
 Original naming the teacher in charge of the cat's grade.
 */
const GRADE_HEAD = '年级组长对猫很好。';

/**
 Every rendering either glossary offers the bench, lowercased.

 @returns Renderings, one per entry and form
 @example
 ```ts
 const offered = glossaryRenderings();
 ```
 */
function glossaryRenderings(): readonly string[] {
  return [...COMMUNITY_GLOSSARY, ...RENDERING_GLOSSARY,].flatMap(function renderingsOf(entry,): readonly string[] {
    return entry.renderings.map(function lower(rendering,): string {
      return rendering.toLowerCase();
    },);
  },);
}

/**
 Offered renderings carrying a word or phrase a Canadian page does not use.

 @returns Each offending rendering
 @example
 ```ts
 const offending = nonCanadianRenderings();
 ```
 */
function nonCanadianRenderings(): readonly string[] {
  return glossaryRenderings().filter(function isNonCanadian(rendering,): boolean {
    const words = Array.from(rendering, function letterOrSpace(character,): string {
      return LOWERCASE_LETTERS.includes(character,) ? character : ' ';
    },).join('',).split(' ',);
    return NON_CANADIAN_WORDS.some(function hasWord(word,): boolean {
      return words.includes(word,);
    },) || NON_CANADIAN_PHRASES.some(function hasPhrase(phrase,): boolean {
      return rendering.includes(phrase,);
    },);
  },);
}

await describe({
  name: 'the page is Canadian English on every sheet (class one hundred thirty-two)',
  children: [
    it({
      name: 'the house policy names Canadian English',
      fn: async () => {
        expect(HOUSE_POLICY_BLOCK,).toContain(CANADIAN_RULE,);
      },
    },),
    it({
      name: 'the translate and select sheets carry it',
      fn: async () => {
        const translate = systemOf({
          messages: buildTranslateMessages({ sourceText: SOURCE_TEXT, existingText: 'The cat slept on the windowsill.', },).messages,
        },);
        const select = systemOf({
          messages: buildCandidateSelectMessages({
            task: 'Each candidate is a rendering of the passage below.',
            criteria: ['Complete coverage: every proposition of the ORIGINAL is rendered.',],
            evidence: [{ label: 'ORIGINAL (Chinese)', text: SOURCE_TEXT, },],
            rendered: ['The cat slept on the windowsill.', 'The cat is napping on the sill.',],
          },),
        },);
        expect([translate.includes(CANADIAN_RULE,), select.includes(CANADIAN_RULE,),],).toEqual([true, true,],);
      },
    },),
    it({
      name: 'the house policy carries none of the four -ise forms it once wrote (every word of every sheet: '
        + 'sheet-canadian-spelling.unit.test.ts)',
      fn: async () => {
        expect(POLICY_BRITISH_FORMS.filter(function present(form,): boolean {
          return HOUSE_POLICY_BLOCK.includes(form,);
        },),).toEqual([],);
      },
    },),
    it({
      name: 'no glossary rendering is British or American where Canadian differs',
      fn: async () => {
        expect(nonCanadianRenderings(),).toEqual([],);
      },
    },),
    it({
      name: 'REFUSES "petrol car" and "head of year", ACCEPTS the Canadian words',
      fn: async () => {
        expect([
          validateTranslatedSlice({ sourceText: GAS_CAR, candidateText: 'The cat said the neighbour\'s old petrol car kept it awake.', },).kind,
          validateTranslatedSlice({ sourceText: GRADE_HEAD, candidateText: 'The head of year was kind to the cat.', },).kind,
          validateTranslatedSlice({ sourceText: GAS_CAR, candidateText: 'The cat said the neighbour\'s old gas-powered car kept it awake.', },).kind,
          validateTranslatedSlice({ sourceText: GRADE_HEAD, candidateText: 'The grade coordinator was kind to the cat.', },).kind,
        ],).toEqual(['invalid', 'invalid', 'valid', 'valid',],);
      },
    },),
  ],
},);
