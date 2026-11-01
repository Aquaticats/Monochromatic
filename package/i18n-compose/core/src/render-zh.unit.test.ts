/**
 Golden render tests for the Chinese locale.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { defineChineseLocale, } from './locale/zh/index.ts';
import {
  zhLabels,
  zhNouns,
  zhSubjects,
  zhVerbs,
} from './test-vocab/index.ts';

/** Chinese spec used by every test in this file. */
const zh = defineChineseLocale({ labels: zhLabels, subjects: zhSubjects, nouns: zhNouns,
  verbs: zhVerbs, },);

await describe({
  name: 'render-zh',
  children: [
    //region Noun phrases

    describe({
      name: 'noun phrases',
      children: [
        it({
          name: 'bare noun renders surface only',
          fn: async () => {
            expect(zh.renderNounPhrase({ kind: 'noun.bare', noun: 'cat', },),).toBe(
              '猫',
            );
          },
        },),

        it({
          name: 'counted noun uses ASCII space between digit and classifier',
          fn: async () => {
            expect(
              zh.renderNounPhrase({ kind: 'noun.counted', count: 1, noun: 'cat', },),
            )
              .toBe('1 只猫',);
            expect(
              zh.renderNounPhrase({ kind: 'noun.counted', count: 5, noun: 'message', },),
            )
              .toBe('5 条消息',);
          },
        },),

        it({
          name: 'definite noun uses 这 + classifier + surface',
          fn: async () => {
            expect(zh.renderNounPhrase({ kind: 'noun.definite', noun: 'cat', },),).toBe(
              '这只猫',
            );
          },
        },),

        it({
          name: 'indefinite noun uses 一 + classifier + surface',
          fn: async () => {
            expect(zh.renderNounPhrase({ kind: 'noun.indefinite', noun: 'cat', },),).toBe(
              '一只猫',
            );
          },
        },),

        it({
          name: 'possessed noun uses possessor + noun without space',
          fn: async () => {
            expect(zh.renderNounPhrase({
              kind: 'noun.possessed',
              possessor: { kind: 'possessor.subject', subject: 'I', },
              noun: 'cat',
            },),)
              .toBe('我的猫',);
          },
        },),

        it({
          name: 'external text passes through verbatim, braces included',
          fn: async () => {
            expect(
              zh.renderNounPhrase({ kind: 'noun.externalText', text: '{time}', },),
            )
              .toBe('{time}',);
          },
        },),

        it({
          name: 'mass nouns reject numeric counted phrases',
          fn: async () => {
            /** Chinese spec with one mass-only noun to exercise countability validation. */
            const localSpec = defineChineseLocale({
              labels: zhLabels,
              subjects: zhSubjects,
              nouns: { ...zhNouns, cat: { ...zhNouns.cat, countability: 'mass', }, },
              verbs: zhVerbs,
            },);
            expect(() =>
              localSpec.renderNounPhrase({ kind: 'noun.counted', count: 2, noun: 'cat', },)
            )
              .toThrow('Cannot count mass noun',);
          },
        },),
      ],
    },),

    //endregion Noun phrases

    //region Declaratives

    describe({
      name: 'declaratives',
      children: [
        it({
          name: '我有 1 只猫。 (1s present)',
          fn: async () => {
            expect(zh.renderSentence({
              kind: 'sentence.declarative',
              subject: { kind: 'subject.key', subject: 'I', },
              predicate: { kind: 'verbPhrase', verb: 'have',
                object: { kind: 'noun.counted', count: 1, noun: 'cat', }, },
            },),)
              .toBe('我有 1 只猫。',);
          },
        },),

        it({
          name: 'past tense appends 了',
          fn: async () => {
            expect(zh.renderSentence({
              kind: 'sentence.declarative',
              subject: { kind: 'subject.key', subject: 'I', },
              tense: 'past',
              predicate: { kind: 'verbPhrase', verb: 'see',
                object: { kind: 'noun.counted', count: 1, noun: 'cat', }, },
            },),)
              .toBe('我看见了 1 只猫。',);
          },
        },),

        it({
          name: 'future tense prepends 会',
          fn: async () => {
            expect(zh.renderSentence({
              kind: 'sentence.declarative',
              subject: { kind: 'subject.key', subject: 'I', },
              tense: 'future',
              predicate: { kind: 'verbPhrase', verb: 'see',
                object: { kind: 'noun.counted', count: 1, noun: 'cat', }, },
            },),)
              .toBe('我会看见 1 只猫。',);
          },
        },),
      ],
    },),

    //endregion Declaratives

    //region Yes/no questions

    describe({
      name: 'yes/no questions',
      children: [
        it({
          name: 'appends 吗 particle and Chinese question mark',
          fn: async () => {
            expect(zh.renderSentence({
              kind: 'sentence.question.yesNo',
              subject: { kind: 'subject.key', subject: 'I', },
              predicate: { kind: 'verbPhrase', verb: 'have',
                object: { kind: 'noun.counted', count: 1, noun: 'cat', }, },
            },),)
              .toBe('我有 1 只猫吗？',);
          },
        },),
      ],
    },),

    //endregion Yes/no questions

    //region Wh-questions

    describe({
      name: 'wh-questions',
      children: [
        it({
          name: 'wh.object renders 什么 in-situ, NO 吗 particle',
          fn: async () => {
            /** Rendered Chinese wh-object question; the absence of 吗 is the assertion under test. */
            const out = zh.renderSentence({
              kind: 'sentence.question.wh.object',
              wh: 'what',
              subject: { kind: 'subject.key', subject: 'I', },
              verb: 'see',
            },);
            expect(out,).toBe('我看见什么？',);
            expect(out.includes('吗',),).toBe(false,);
          },
        },),

        it({
          name: 'wh.subject renders 谁 in head position with NO 吗',
          fn: async () => {
            const out = zh.renderSentence({
              kind: 'sentence.question.wh.subject',
              wh: 'who',
              predicate: { kind: 'verbPhrase', verb: 'see',
                object: { kind: 'noun.counted', count: 1, noun: 'cat', }, },
            },);
            expect(out,).toBe('谁看见 1 只猫？',);
            expect(out.includes('吗',),).toBe(false,);
          },
        },),

        it({
          name: 'wh.adverbial places the wh-word in the adverbial slot, NO 吗',
          fn: async () => {
            const out = zh.renderSentence({
              kind: 'sentence.question.wh.adverbial',
              wh: 'where',
              subject: { kind: 'subject.key', subject: 'I', },
              predicate: { kind: 'verbPhrase', verb: 'see',
                object: { kind: 'noun.counted', count: 1, noun: 'cat', }, },
            },);
            expect(out,).toBe('我在哪里看见 1 只猫？',);
            expect(out.includes('吗',),).toBe(false,);
          },
        },),
      ],
    },),

    //endregion Wh-questions

    //region Punctuation

    describe({
      name: 'punctuation',
      children: [
        it({
          name: 'declarative uses 。',
          fn: async () => {
            expect(zh
              .renderSentence({
                kind: 'sentence.declarative',
                subject: { kind: 'subject.key', subject: 'I', },
                predicate: { kind: 'verbPhrase', verb: 'have', },
              },)
              .endsWith('。',),)
              .toBe(true,);
          },
        },),

        it({
          name: 'imperative with ! terminator uses ！',
          fn: async () => {
            expect(zh
              .renderSentence({
                kind: 'sentence.imperative',
                predicate: { kind: 'verbPhrase', verb: 'save', },
                terminator: '!',
              },)
              .endsWith('！',),)
              .toBe(true,);
          },
        },),
      ],
    },),
    //endregion Punctuation
  ],
},);
