/**
 App-local i18n entry point for the Aquaticat site.
 
 Wraps `@monochromatic-dev/i18n-compose-core`: each locale spec is built
 from a static label table (the site uses no grammar AST, nouns, verbs, or
 subjects, so those vocabulary records are empty). Consumers import `i18n`,
 `locales`, `Locale`, `isLocale`, and `assertLocale` from here; nothing
 imports the composition package directly.
 
 @module
 */

import {
  createI18n,
  defineCatalanLocale,
  defineChineseLocale,
  defineEnglishLocale,
} from '@monochromatic-dev/i18n-compose-core/ts';

import caLabels from './ca/index.ts';
import enLabels from './en/index.ts';
import zhLabels from './zh/index.ts';

/**
 Catalan locale spec; grammar vocabulary is empty because only labels are rendered.
 */
const ca = defineCatalanLocale({
  labels: caLabels,
  subjects: {},
  nouns: {},
  verbs: {},
},);

/**
 English (base) locale spec; grammar vocabulary is empty because only labels are rendered.
 */
const en = defineEnglishLocale({
  labels: enLabels,
  subjects: {},
  nouns: {},
  verbs: {},
},);

/**
 Chinese locale spec; grammar vocabulary is empty because only labels are rendered.
 */
const zh = defineChineseLocale({
  labels: zhLabels,
  subjects: {},
  nouns: {},
  verbs: {},
},);

/**
 Supported locale codes; the literal union flows through every render call.
 */
export const locales = [
  'ca',
  'en',
  'zh',
] as const;

/**
 Supported locale code union derived from {@link locales}.
 */
export type Locale = typeof locales[number];

/**
 Explicit-locale render surface and registry helpers for the site.
 */
export const i18n = createI18n({
  locales,
  defaultLocale: 'en',
  specs: {
    ca,
    en,
    zh,
  },
},);

/**
 Type guard narrowing an arbitrary string to a supported {@link Locale}.
 */
export const {isLocale} = i18n;

/**
 Asserts a string is a supported {@link Locale}, returning it narrowed or throwing.
 */
export const {assertLocale} = i18n;
