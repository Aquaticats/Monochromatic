/**
 Golden render tests for the English locale.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { defineEnglishLocale, } from './locale/en/index.ts';
import {
  enLabels,
  enNouns,
  enSubjects,
  enVerbs,
} from './test-vocab/index.ts';

/** English spec used by every test in this file. */
const en = defineEnglishLocale({ labels: enLabels, subjects: enSubjects, nouns: enNouns,
  verbs: enVerbs, },);

await describe({
  name: 'render-en',
  children: [
    //region Noun phrases

    describe({
      name: 'noun phrases',
      children: [
        it({
          name: 'bare noun renders surface',
          fn: async () => {
            expect(en.renderNounPhrase({ kind: 'noun.bare', noun: 'cat', },),).toBe(
              'cat',
            );
          },
        },),

        it({
          name: 'counted noun renders digit + singular at count 1',
          fn: async () => {
            expect(
              en.renderNounPhrase({ kind: 'noun.counted', count: 1, noun: 'cat', },),
            )
              .toBe('1 cat',);
          },
        },),

        it({
          name: 'counted noun renders digit + plural at non-1 counts',
          fn: async () => {
            expect(
              en.renderNounPhrase({ kind: 'noun.counted', count: 0, noun: 'cat', },),
            )
              .toBe('0 cats',);
            expect(
              en.renderNounPhrase({ kind: 'noun.counted', count: 2, noun: 'cat', },),
            )
              .toBe('2 cats',);
          },
        },),

        it({
          name: 'definite noun uses the noun entry article',
          fn: async () => {
            expect(en.renderNounPhrase({ kind: 'noun.definite', noun: 'cat', },),).toBe(
              'the cat',
            );
          },
        },),

        it({
          name: 'indefinite noun uses the noun entry article',
          fn: async () => {
            expect(en.renderNounPhrase({ kind: 'noun.indefinite', noun: 'cat', },),).toBe(
              'a cat',
            );
            expect(en.renderNounPhrase({ kind: 'noun.indefinite', noun: 'item', },),)
              .toBe('an item',);
          },
        },),

        it({
          name: 'possessed noun by subject key uses possessive surface',
          fn: async () => {
            expect(
              en.renderNounPhrase({
                kind: 'noun.possessed',
                possessor: { kind: 'possessor.subject', subject: 'I', },
                noun: 'cat',
              },),
            )
              .toBe('my cat',);
          },
        },),

        it({
          name: 'external text passes through verbatim, including braces',
          fn: async () => {
            expect(
              en.renderNounPhrase({ kind: 'noun.externalText',
                text: '{not a template} {time}', },),
            )
              .toBe('{not a template} {time}',);
          },
        },),

        it({
          name: 'mass nouns reject numeric counted phrases',
          fn: async () => {
            /** English spec with one mass-only noun to exercise countability validation. */
            const localSpec = defineEnglishLocale({
              labels: enLabels,
              subjects: enSubjects,
              nouns: { ...enNouns, cat: { ...enNouns.cat, countability: 'mass', }, },
              verbs: enVerbs,
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
      name: 'declarative sentences',
      children: [
        it({
          name: 'I have 1 cat (1s present, count 1)',
          fn: async () => {
            expect(en.renderSentence({
              kind: 'sentence.declarative',
              subject: { kind: 'subject.key', subject: 'I', },
              predicate: {
                kind: 'verbPhrase',
                verb: 'have',
                object: { kind: 'noun.counted', count: 1, noun: 'cat', },
              },
            },),)
              .toBe('I have 1 cat.',);
          },
        },),

        it({
          name: 'they have 2 cats (3p present uses base, count 2 plural)',
          fn: async () => {
            expect(en.renderSentence({
              kind: 'sentence.declarative',
              subject: { kind: 'subject.key', subject: 'they', },
              predicate: {
                kind: 'verbPhrase',
                verb: 'have',
                object: { kind: 'noun.counted', count: 2, noun: 'cat', },
              },
            },),)
              .toBe('They have 2 cats.',);
          },
        },),

        it({
          name: '3s present uses present3s form (he/she/it -> has)',
          fn: async () => {
            /** Synthetic third-singular subject derived from the existing `they` entry shape. */
            const subject = { kind: 'subject.key', subject: 'they', } as const;
            /**
             Deliberate surface/agreement mismatch: `they` still renders while 3s metadata
             isolates present3s agreement, because shared test vocabulary intentionally has
             no he/she/it subject.
             */
            const localSpec = defineEnglishLocale({
              labels: enLabels,
              subjects: { ...enSubjects,
                they: { ...enSubjects.they, person: 3, number: 'singular', }, },
              nouns: enNouns,
              verbs: enVerbs,
            },);
            expect(localSpec.renderSentence({
              kind: 'sentence.declarative',
              subject,
              predicate: { kind: 'verbPhrase', verb: 'have',
                object: { kind: 'noun.counted', count: 1, noun: 'cat', }, },
            },),)
              .toBe('They has 1 cat.',);
          },
        },),

        it({
          name: 'past tense uses the past form',
          fn: async () => {
            expect(en.renderSentence({
              kind: 'sentence.declarative',
              subject: { kind: 'subject.key', subject: 'I', },
              tense: 'past',
              predicate: { kind: 'verbPhrase', verb: 'see',
                object: { kind: 'noun.counted', count: 1, noun: 'cat', }, },
            },),)
              .toBe('I saw 1 cat.',);
          },
        },),

        it({
          name: 'future tense wraps base in will',
          fn: async () => {
            expect(en.renderSentence({
              kind: 'sentence.declarative',
              subject: { kind: 'subject.key', subject: 'I', },
              tense: 'future',
              predicate: { kind: 'verbPhrase', verb: 'see',
                object: { kind: 'noun.counted', count: 1, noun: 'cat', }, },
            },),)
              .toBe('I will see 1 cat.',);
          },
        },),
      ],
    },),

    //endregion Declaratives

    //region Yes/no questions (do-support)

    describe({
      name: 'yes/no questions',
      children: [
        it({
          name: 'do-support: 1s present uses Do',
          fn: async () => {
            expect(en.renderSentence({
              kind: 'sentence.question.yesNo',
              subject: { kind: 'subject.key', subject: 'I', },
              predicate: { kind: 'verbPhrase', verb: 'have',
                object: { kind: 'noun.counted', count: 1, noun: 'cat', }, },
            },),)
              .toBe('Do I have 1 cat?',);
          },
        },),

        it({
          name: 'do-support: past uses Did + base, NOT Did + past',
          fn: async () => {
            expect(en.renderSentence({
              kind: 'sentence.question.yesNo',
              subject: { kind: 'subject.key', subject: 'I', },
              tense: 'past',
              predicate: { kind: 'verbPhrase', verb: 'have',
                object: { kind: 'noun.counted', count: 1, noun: 'cat', }, },
            },),)
              .toBe('Did I have 1 cat?',);
          },
        },),

        it({
          name: 'do-support: 3s present uses Does',
          fn: async () => {
            /**
             Deliberate surface/agreement mismatch: `they` still renders while 3s metadata
             isolates do-support agreement, because shared test vocabulary intentionally has
             no he/she/it subject.
             */
            const localSpec = defineEnglishLocale({
              labels: enLabels,
              subjects: { ...enSubjects,
                they: { ...enSubjects.they, person: 3, number: 'singular', }, },
              nouns: enNouns,
              verbs: enVerbs,
            },);
            expect(localSpec.renderSentence({
              kind: 'sentence.question.yesNo',
              subject: { kind: 'subject.key', subject: 'they', },
              predicate: { kind: 'verbPhrase', verb: 'have',
                object: { kind: 'noun.counted', count: 1, noun: 'cat', }, },
            },),)
              .toBe('Does they have 1 cat?',);
          },
        },),

        it({
          name: 'copula strategy fronts the finite verb instead of adding do-support',
          fn: async () => {
            /** English spec reusing the save key as a copula entry for the regression. */
            const localSpec = defineEnglishLocale({
              labels: enLabels,
              subjects: enSubjects,
              nouns: enNouns,
              verbs: { ...enVerbs,
                save: { base: 'are', present3s: 'is', past: 'was',
                  auxiliaryStrategy: 'copula', }, },
            },);
            expect(localSpec.renderSentence({
              kind: 'sentence.question.yesNo',
              subject: { kind: 'subject.key', subject: 'you', },
              predicate: { kind: 'verbPhrase', verb: 'save',
                object: { kind: 'noun.externalText', text: 'ready', }, },
            },),)
              .toBe('Are you ready?',);
          },
        },),

        it({
          name: 'modal strategy fronts the modal and renders bare complements',
          fn: async () => {
            /** English spec reusing the save key as a modal entry for the regression. */
            const localSpec = defineEnglishLocale({
              labels: enLabels,
              subjects: enSubjects,
              nouns: enNouns,
              verbs: { ...enVerbs,
                save: { base: 'can', auxiliaryStrategy: 'modal', }, },
            },);
            expect(localSpec.renderSentence({
              kind: 'sentence.question.yesNo',
              subject: { kind: 'subject.key', subject: 'you', },
              predicate: {
                kind: 'verbPhrase',
                verb: 'save',
                complement: {
                  kind: 'complement.infinitive',
                  phrase: {
                    kind: 'verbPhrase',
                    verb: 'delete',
                    object: { kind: 'noun.externalText', text: 'README.md', },
                  },
                },
              },
            },),)
              .toBe('Can you delete README.md?',);
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
          name: 'wh.object fronts What and inserts do-support',
          fn: async () => {
            expect(en.renderSentence({
              kind: 'sentence.question.wh.object',
              wh: 'what',
              subject: { kind: 'subject.key', subject: 'I', },
              verb: 'see',
            },),)
              .toBe('What do I see?',);
          },
        },),

        it({
          name: 'wh.subject fronts Who and uses finite verb without do-support',
          fn: async () => {
            expect(en.renderSentence({
              kind: 'sentence.question.wh.subject',
              wh: 'who',
              predicate: { kind: 'verbPhrase', verb: 'see',
                object: { kind: 'noun.counted', count: 1, noun: 'cat', }, },
            },),)
              .toBe('Who sees 1 cat?',);
          },
        },),

        it({
          name: 'wh.adverbial fronts Where/When/Why/How and inserts do-support',
          fn: async () => {
            expect(en.renderSentence({
              kind: 'sentence.question.wh.adverbial',
              wh: 'where',
              subject: { kind: 'subject.key', subject: 'I', },
              predicate: { kind: 'verbPhrase', verb: 'see',
                object: { kind: 'noun.counted', count: 1, noun: 'cat', }, },
            },),)
              .toBe('Where do I see 1 cat?',);
          },
        },),
      ],
    },),

    //endregion Wh-questions

    //region Imperatives & complements

    describe({
      name: 'imperatives and complements',
      children: [
        it({
          name: 'imperative uses the imperative form when supplied',
          fn: async () => {
            expect(en.renderSentence({
              kind: 'sentence.imperative',
              predicate: { kind: 'verbPhrase', verb: 'save',
                object: { kind: 'noun.counted', count: 1, noun: 'item', }, },
            },),)
              .toBe('Save 1 item.',);
          },
        },),

        it({
          name: 'imperative falls back to base when no imperative is supplied',
          fn: async () => {
            expect(en.renderSentence({
              kind: 'sentence.imperative',
              predicate: { kind: 'verbPhrase', verb: 'delete',
                object: { kind: 'noun.externalText', text: 'README', }, },
            },),)
              .toBe('Delete README.',);
          },
        },),

        it({
          name: 'generic confirmation-like sentence via want + infinitive complement',
          fn: async () => {
            expect(en.renderSentence({
              kind: 'sentence.question.yesNo',
              subject: { kind: 'subject.key', subject: 'you', },
              predicate: {
                kind: 'verbPhrase',
                verb: 'want',
                complement: {
                  kind: 'complement.infinitive',
                  phrase: {
                    kind: 'verbPhrase',
                    verb: 'delete',
                    object: { kind: 'noun.externalText', text: 'README.md', },
                  },
                },
              },
            },),)
              .toBe('Do you want to delete README.md?',);
          },
        },),
      ],
    },),

    //endregion Imperatives & complements

    //region Fragments

    describe({
      name: 'fragments',
      children: [
        it({
          name: 'noun-phrase fragment renders the phrase with capitalization',
          fn: async () => {
            expect(en.renderFragment({
              kind: 'fragment.nounPhrase',
              phrase: { kind: 'noun.counted', count: 3, noun: 'cat', },
              capitalization: 'firstLetter',
            },),)
              .toBe('3 cats',);
          },
        },),

        it({
          name: 'verb-phrase fragment in infinitive form prefixes with to',
          fn: async () => {
            expect(en.renderFragment({
              kind: 'fragment.verbPhrase',
              phrase: { kind: 'verbPhrase', verb: 'save', },
              form: 'infinitive',
            },),)
              .toBe('to save',);
          },
        },),

        it({
          name:
            'verb-phrase fragment in gerund form uses the gerund or derives it from base',
          fn: async () => {
            expect(en.renderFragment({
              kind: 'fragment.verbPhrase',
              phrase: { kind: 'verbPhrase', verb: 'save', },
              form: 'gerund',
            },),)
              .toBe('saving',);
          },
        },),

        it({
          name: 'verb-phrase fragment in gerund form prefers explicit gerund',
          fn: async () => {
            /**
             Spec whose `save` key uses an `open` base to prove explicit gerund overrides
             derived `openning`.
             */
            const localSpec = defineEnglishLocale({
              labels: enLabels,
              subjects: enSubjects,
              nouns: enNouns,
              verbs: { ...enVerbs,
                save: { ...enVerbs.save, base: 'open', gerund: 'opening', }, },
            },);
            expect(localSpec.renderFragment({
              kind: 'fragment.verbPhrase',
              phrase: { kind: 'verbPhrase', verb: 'save', },
              form: 'gerund',
            },),)
              .toBe('opening',);
          },
        },),

        it({
          name: 'sequence joins label, noun phrase, and external text',
          fn: async () => {
            expect(en.renderFragment({
              kind: 'fragment.sequence',
              parts: [
                { kind: 'part.label', label: 'noResults', },
                { kind: 'part.externalText', text: '-', },
                { kind: 'part.nounPhrase',
                  phrase: { kind: 'noun.counted', count: 0, noun: 'message', }, },
              ],
              capitalization: 'firstLetter',
            },),)
              .toBe('No results - 0 messages',);
          },
        },),
      ],
    },),
    //endregion Fragments
  ],
},);
