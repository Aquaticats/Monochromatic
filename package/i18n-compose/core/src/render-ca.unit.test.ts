/**
 Golden render tests for the Catalan locale.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { defineCatalanLocale, } from './locale/ca/index.ts';
import {
  caLabels,
  caNouns,
  caSubjects,
  caVerbs,
} from './test-vocab/index.ts';

/** Catalan spec used by every test in this file. */
const ca = defineCatalanLocale({ labels: caLabels, subjects: caSubjects, nouns: caNouns,
  verbs: caVerbs, },);

await describe({
  name: 'render-ca',
  children: [
    //region Noun phrases

    describe({
      name: 'noun phrases',
      children: [
        it({
          name: 'bare noun renders surface',
          fn: async () => {
            expect(ca.renderNounPhrase({ kind: 'noun.bare', noun: 'cat', },),).toBe(
              'gat',
            );
          },
        },),

        it({
          name: 'counted noun renders digit + singular at count 1',
          fn: async () => {
            expect(
              ca.renderNounPhrase({ kind: 'noun.counted', count: 1, noun: 'cat', },),
            )
              .toBe('1 gat',);
          },
        },),

        it({
          name: 'counted noun renders digit + plural at non-1 counts',
          fn: async () => {
            expect(
              ca.renderNounPhrase({ kind: 'noun.counted', count: 2, noun: 'cat', },),
            )
              .toBe('2 gats',);
          },
        },),

        it({
          name: 'definite noun uses masculine singular article el',
          fn: async () => {
            expect(ca.renderNounPhrase({ kind: 'noun.definite', noun: 'cat', },),).toBe(
              'el gat',
            );
          },
        },),

        it({
          name: 'indefinite noun uses masculine singular article un',
          fn: async () => {
            expect(ca.renderNounPhrase({ kind: 'noun.indefinite', noun: 'cat', },),).toBe(
              'un gat',
            );
          },
        },),

        it({
          name: "definite for item uses elided l'",
          fn: async () => {
            expect(ca.renderNounPhrase({ kind: 'noun.definite', noun: 'item', },),).toBe(
              `l'article`,
            );
          },
        },),

        it({
          name: 'mass nouns reject numeric counted phrases',
          fn: async () => {
            /** Catalan spec with one mass-only noun to exercise countability validation. */
            const localSpec = defineCatalanLocale({
              labels: caLabels,
              subjects: caSubjects,
              nouns: { ...caNouns, cat: { ...caNouns.cat, countability: 'mass', }, },
              verbs: caVerbs,
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
          name: 'Jo tinc 1 gat. (1s present finite)',
          fn: async () => {
            expect(ca.renderSentence({
              kind: 'sentence.declarative',
              subject: { kind: 'subject.key', subject: 'I', },
              predicate: { kind: 'verbPhrase', verb: 'have',
                object: { kind: 'noun.counted', count: 1, noun: 'cat', }, },
            },),)
              .toBe('Jo tinc 1 gat.',);
          },
        },),

        it({
          name: 'past finite agreement for 1s tenir -> tenia',
          fn: async () => {
            expect(ca.renderSentence({
              kind: 'sentence.declarative',
              subject: { kind: 'subject.key', subject: 'I', },
              tense: 'past',
              predicate: { kind: 'verbPhrase', verb: 'have',
                object: { kind: 'noun.counted', count: 1, noun: 'cat', }, },
            },),)
              .toBe('Jo tenia 1 gat.',);
          },
        },),

        it({
          name: 'future finite agreement for 1s tenir -> tindré',
          fn: async () => {
            expect(ca.renderSentence({
              kind: 'sentence.declarative',
              subject: { kind: 'subject.key', subject: 'I', },
              tense: 'future',
              predicate: { kind: 'verbPhrase', verb: 'have',
                object: { kind: 'noun.counted', count: 1, noun: 'cat', }, },
            },),)
              .toBe('Jo tindré 1 gat.',);
          },
        },),

        it({
          name: '3p present uses tenen for they',
          fn: async () => {
            expect(ca.renderSentence({
              kind: 'sentence.declarative',
              subject: { kind: 'subject.key', subject: 'they', },
              predicate: { kind: 'verbPhrase', verb: 'have',
                object: { kind: 'noun.counted', count: 2, noun: 'cat', }, },
            },),)
              .toBe('Ells tenen 2 gats.',);
          },
        },),

        it({
          name: 'missing finite tense surface throws',
          fn: async () => {
            expect(() =>
              ca.renderSentence({
                kind: 'sentence.declarative',
                subject: { kind: 'subject.key', subject: 'I', },
                tense: 'future',
                predicate: { kind: 'verbPhrase', verb: 'see', },
              },)
            )
              .toThrow('no finite forms for tense',);
          },
        },),
      ],
    },),

    //endregion Declaratives

    //region Imperatives

    describe({
      name: 'imperatives',
      children: [
        it({
          name: 'imperative uses the supplied imperative surface',
          fn: async () => {
            expect(ca.renderSentence({
              kind: 'sentence.imperative',
              predicate: { kind: 'verbPhrase', verb: 'save',
                object: { kind: 'noun.counted', count: 1, noun: 'item', }, },
            },),)
              .toBe('Desa 1 article.',);
          },
        },),
      ],
    },),
    //endregion Imperatives

    //region Complements

    describe({
      name: 'complements',
      children: [
        it({
          name: 'sentence complements preserve nested objects',
          fn: async () => {
            expect(ca.renderSentence({
              kind: 'sentence.question.yesNo',
              subject: { kind: 'subject.key', subject: 'you', },
              predicate: {
                kind: 'verbPhrase',
                verb: 'want',
                complement: {
                  kind: 'complement.infinitive',
                  phrase: { kind: 'verbPhrase', verb: 'delete',
                    object: { kind: 'noun.externalText', text: 'README.md', }, },
                },
              },
            },),)
              .toBe('Tu vols esborrar README.md?',);
          },
        },),

        it({
          name: 'verb-phrase complements preserve nested objects',
          fn: async () => {
            expect(ca.renderVerbPhrase({
              kind: 'verbPhrase',
              verb: 'want',
              complement: { kind: 'complement.infinitive',
                phrase: { kind: 'verbPhrase', verb: 'delete',
                  object: { kind: 'noun.externalText', text: 'README.md', }, }, },
            },),)
              .toBe('voler esborrar README.md',);
          },
        },),

        it({
          name: 'fragment complements preserve nested objects',
          fn: async () => {
            expect(ca.renderFragment({
              kind: 'fragment.verbPhrase',
              form: 'infinitive',
              phrase: {
                kind: 'verbPhrase',
                verb: 'want',
                complement: { kind: 'complement.infinitive',
                  phrase: { kind: 'verbPhrase', verb: 'delete',
                    object: { kind: 'noun.externalText', text: 'README.md', }, }, },
              },
            },),)
              .toBe('voler esborrar README.md',);
          },
        },),
      ],
    },),
    //endregion Complements
  ],
},);
