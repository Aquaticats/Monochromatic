/**
 Test-only English vocabulary tables.
 
 @module
 */

import type {
  NounEntry,
  SubjectEntry,
} from '../entries.ts';
import type { EnglishVerbEntry, } from '../locale/en/index.ts';
import type {
  TestLabel,
  TestNoun,
  TestSubject,
  TestVerb,
} from './types.ts';

/**
 English label table for tests.
 */
export const enLabels: Record<TestLabel, string> = {
  siteName: 'Aquaticat',
  noResults: 'No results',
  page: 'page',
};

/**
 English subject table for tests.
 */
export const enSubjects: Record<TestSubject, SubjectEntry> = {
  I: {
    surface: 'I',
    possessive: 'my',
    person: 1,
    number: 'singular',
  },
  you: {
    surface: 'you',
    possessive: 'your',
    person: 2,
    number: 'singular',
  },
  they: {
    surface: 'they',
    possessive: 'their',
    person: 3,
    number: 'plural',
  },
  who: {
    surface: 'who',
    possessive: 'whose',
    person: 3,
    number: 'singular',
  },
};

/**
 English noun table for tests.
 */
export const enNouns: Record<TestNoun, NounEntry> = {
  cat: {
    surface: 'cat',
    plural: 'cats',
    articles: {
      definite: {
        singular: 'the',
        plural: 'the',
      },
      indefinite: { singular: 'a', },
    },
  },
  message: {
    surface: 'message',
    plural: 'messages',
    articles: {
      definite: {
        singular: 'the',
        plural: 'the',
      },
      indefinite: { singular: 'a', },
    },
  },
  item: {
    surface: 'item',
    plural: 'items',
    articles: {
      definite: {
        singular: 'the',
        plural: 'the',
      },
      indefinite: { singular: 'an', },
    },
  },
};

/**
 English verb table for tests.
 */
export const enVerbs: Record<TestVerb, EnglishVerbEntry> = {
  have: {
    base: 'have',
    present3s: 'has',
    past: 'had',
    pastParticiple: 'had',
  },
  see: {
    base: 'see',
    present3s: 'sees',
    past: 'saw',
    pastParticiple: 'seen',
  },
  delete: {
    base: 'delete',
    present3s: 'deletes',
    past: 'deleted',
  },
  want: {
    base: 'want',
    present3s: 'wants',
    past: 'wanted',
  },
  save: {
    base: 'save',
    present3s: 'saves',
    past: 'saved',
    imperative: 'Save',
  },
};
