/**
 Guards class one hundred nineteen (2026-09-24): the
 community term 药娘 (and 小药娘, which carries it) shipped in Han where
 two earlier runs wrote "little HRT girl" and "little yaoniang". The owner
 answered on 2026-09-24 that the word is disrespectful, used neutrally by
 only some of the community, and that the neutrality does not carry into
 English: the page says "trans woman" or "trans girl". The glossary seeds
 the term with those renderings, and a candidate that keeps a glossary
 term in Han, or writes a form the entry refuses (the pinyin), is refused
 before any judge reads it. The owner added the same day that the term is
 refused even where the existing translation keeps it. A rendering in
 English and a term standing inside an HTML comment are left alone.
 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { COMMUNITY_GLOSSARY, } from '@monochromatic-dev/translation-repair-glossary';
import { validateTranslatedSlice, } from '../dist/final/node/index.mjs';

/**
 Original calling the cat by the term.
 */
const NAMED = '这只猫是一只爱晒太阳的小药娘[^1]。';

/**
 Rendering that left the term in Han.
 */
const LEFT = 'This cat was a sun-loving 小药娘[^1].';

/**
 Rendering that wrote the term in pinyin.
 */
const PINYIN = 'This cat was a sun-loving little yaoniang[^1].';

/**
 Rendering that wrote the hyphenated pinyin.
 */
const HYPHENATED = 'This cat napped beside a yao-niang[^1].';

/**
 Rendering in the owner's English.
 */
const RENDERED = 'This cat was a sun-loving trans girl[^1].';

await describe({
  name: 'a community term the glossary renders (class one hundred nineteen)',
  children: [
    it({
      name: 'SEEDS 药娘 with "trans girl" first and "trans woman" beside it, and refuses its pinyin',
      fn: async () => {
        /**
         The seeded entry.
         */
        const entry = COMMUNITY_GLOSSARY.find(function isTerm(candidate,): boolean {
          return candidate.term === '药娘';
        },);
        expect(entry?.renderings[0],).toBe('trans girl',);
        expect(entry?.renderings,).toContain('trans woman',);
        expect(entry?.refusedForms,).toContain('yaoniang',);
      },
    },),
    it({
      name: 'REFUSES a candidate that leaves the term in Han',
      fn: async () => {
        /**
         Verdict on the rendering that kept the Han.
         */
        const verdict = validateTranslatedSlice({
          sourceText: NAMED,
          candidateText: LEFT,
        },);
        expect(verdict.kind,).toBe('invalid',);
        if (verdict.kind !== 'invalid')
          throw new Error('unreachable',);
        expect(verdict.findings.join('\n',),).toContain('药娘',);
        expect(verdict.findings.join('\n',),).toContain('trans girl',);
      },
    },),
    it({
      name: 'REFUSES the term in Han even where the existing translation keeps it (owner, 2026-09-24)',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: NAMED,
          candidateText: LEFT,
          pageText: LEFT,
        },).kind,).toBe('invalid',);
      },
    },),
    it({
      name: 'REFUSES a candidate that writes a refused form of the term, in any casing',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: NAMED,
          candidateText: PINYIN,
        },).kind,).toBe('invalid',);
        expect(validateTranslatedSlice({
          sourceText: NAMED,
          candidateText: HYPHENATED,
        },).kind,).toBe('invalid',);
        expect(validateTranslatedSlice({
          sourceText: NAMED,
          candidateText: 'This cat was a sun-loving little Yaoniang[^1].',
        },).kind,).toBe('invalid',);
      },
    },),
    it({
      name: 'ACCEPTS the term rendered, standing in a comment, or absent from the source',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: NAMED,
          candidateText: RENDERED,
        },).kind,).toBe('valid',);
        expect(validateTranslatedSlice({
          sourceText: '<!-- 小药娘 -->猫睡了。',
          candidateText: 'The cat slept.',
        },).kind,).toBe('valid',);
        expect(validateTranslatedSlice({
          sourceText: '猫睡了。',
          candidateText: 'The cat slept, a yaoniang said.',
        },).kind,).toBe('valid',);
      },
    },),
  ],
},);
