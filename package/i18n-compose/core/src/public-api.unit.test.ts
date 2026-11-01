/**
 Public package import tests for consumer-facing i18n-compose usage.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createI18n,
  defineEnglishLocale,
} from '@monochromatic-dev/i18n-compose-core';

/** Consumer-owned label fixture used through the package root import. */
const labels = {
  siteName: 'Aquaticat',
} as const;

/** Consumer-owned subject fixture used through the package root import. */
const subjects = {
  you: {
    surface: 'you',
    possessive: 'your',
    person: 2,
    number: 'singular',
  },
} as const;

/** Consumer-owned noun fixture used through the package root import. */
const nouns = {
  item: {
    surface: 'item',
    plural: 'items',
    articles: {
      definite: { singular: 'the', plural: 'the', },
      indefinite: { singular: 'an', },
    },
  },
} as const;

/** Consumer-owned verb fixture used through the package root import. */
const verbs = {
  can: {
    base: 'can',
    auxiliaryStrategy: 'modal',
  },
  save: {
    base: 'save',
    present3s: 'saves',
    past: 'saved',
  },
} as const;

/** English locale spec built exclusively through the public package import. */
const en = defineEnglishLocale({
  labels,
  subjects,
  nouns,
  verbs,
},);

/** I18n instance built exclusively through the public package import. */
const i18n = createI18n({
  locales: ['en',] as const,
  defaultLocale: 'en',
  specs: { en, },
},);

await describe({
  name: 'public package import',
  children: [
    it({
      name: 'renders modal complements through package-name imports',
      fn: async () => {
        expect(i18n.sentence('en', {
          kind: 'sentence.question.yesNo',
          subject: { kind: 'subject.key', subject: 'you', },
          predicate: {
            kind: 'verbPhrase',
            verb: 'can',
            complement: {
              kind: 'complement.infinitive',
              phrase: {
                kind: 'verbPhrase',
                verb: 'save',
                object: { kind: 'noun.counted', count: 1, noun: 'item', },
              },
            },
          },
        },),)
          .toBe('Can you save 1 item?',);
      },
    },),
  ],
},);
