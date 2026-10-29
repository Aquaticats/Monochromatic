/**
 Guards the owner's answer of 2026-09-27 ("Allow it, because it's the proper
 name of an org."): a term that stands inside an organization's proper name
 is not the term there, so the floor, the sheet lines and the departures
 leave the name in its own form, while the ruling of 2026-09-24 holds
 everywhere else
 (`doc/decision/translation-repair-community-glossary.md`).

 The originals are cat-themed invention built around the entry's own listed
 contexts, so no corpus sentence appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  COMMUNITY_GLOSSARY,
  communityRenderingDepartures,
  communityTermLines,
} from '@monochromatic-dev/translation-repair-glossary';
import { validateTranslatedSlice, } from '../dist/final/node/index.mjs';

/**
 Contexts the 药娘 entry lists as proper names, empty where it lists none.
 */
const CONTEXTS: readonly string[] = COMMUNITY_GLOSSARY
  .find(function isTerm(entry,): boolean {
    return entry.term === '药娘';
  },)
  ?.properNameContexts ?? [];

/**
 Original in which a cat names its shop with the word.
 */
const SHOP_NAMED = '小猫开了一家店，以小药娘做字号。';

/**
 Original in which a cat works at the company.
 */
const WORKS_AT = '小猫在小药娘网络科技上班。';

await describe({
  name: 'an organization\'s proper name keeps its own form (owner, 2026-09-27)',
  children: [
    it({
      name: 'LISTS the company name and the shop-naming phrase as proper-name contexts of 药娘',
      fn: async () => {
        expect(CONTEXTS,).toContain('以小药娘做字号',);
        expect(CONTEXTS,).toContain('小药娘网络科技',);
      },
    },),
    it({
      name: 'ACCEPTS the name\'s own pinyin where the original names the company, and still refuses it elsewhere',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: SHOP_NAMED,
          candidateText: 'The kitten opened a shop and named it XiaoYaoNiang.',
        },).kind,).toBe('valid',);
        expect(validateTranslatedSlice({
          sourceText: WORKS_AT,
          candidateText: 'The kitten worked at XiaoYaoNiang (XYN) Technology.',
        },).kind,).toBe('valid',);
        expect(validateTranslatedSlice({
          sourceText: '小猫是小药娘。',
          candidateText: 'The kitten was a xiaoyaoniang.',
        },).kind,).toBe('invalid',);
      },
    },),
    it({
      name: 'PUTS NO 药娘 LINE on the sheet and names no departure where only the company carries it',
      fn: async () => {
        expect(communityTermLines({ text: WORKS_AT, },),).toEqual([],);
        expect(communityRenderingDepartures({
          sourceText: WORKS_AT,
          candidates: [{ label: 'CANDIDATE 1', text: 'The kitten worked at XYN.', },],
        },),).toEqual([],);
      },
    },),
  ],
},);
