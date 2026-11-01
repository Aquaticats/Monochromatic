/**
 Type-safe, no-codegen i18n composition library.
 
 Renders localized UI text from explicit semantic grammar nodes for
 `ca`, `en`, and `zh`. The package owns language mechanics; consuming
 packages own product semantics.
 
 Entry points:
 
 - {@link createI18n} builds the explicit-locale render surface from a
   set of locale specs.
 - {@link defineEnglishLocale}, {@link defineChineseLocale}, and
   {@link defineCatalanLocale} produce specs from consumer-owned
   vocabulary.
 - {@link defineCustomLocale} is an escape hatch when the supplied
   locale builders are insufficient.
 
 @example
 ```ts
 import {
   createI18n,
   defineCatalanLocale,
   defineChineseLocale,
   defineEnglishLocale,
 } from '\@monochromatic-dev/i18n-compose-core';
 
 const en = defineEnglishLocale({ labels, subjects, nouns, verbs });
 const zh = defineChineseLocale({ labels, subjects, nouns, verbs });
 const ca = defineCatalanLocale({ labels, subjects, nouns, verbs });
 
 export const locales = ['ca', 'en', 'zh'] as const;
 
 export const i18n = createI18n({
   locales,
   defaultLocale: 'en',
   specs: { ca, en, zh },
 });
 
 i18n.label('en', 'siteName');
 i18n.np('zh', { kind: 'noun.counted', count: 1, noun: 'cat' });
 ```
 
 @packageDocumentation
 */

//region Grammar primitives

export type {
  Capitalization,
  Countability,
  GrammaticalGender,
  GrammaticalNumber,
  Person,
  PersonNumberKey,
  Tense,
  VerbFragmentForm,
} from './grammar-primitives.ts';

export { personNumberKey, } from './grammar-primitives.ts';

//endregion Grammar primitives

//region Vocabulary entries

export type {
  ArticleForms,
  NounEntry,
  NounPlural,
  SubjectEntry,
} from './entries.ts';

//endregion Vocabulary entries

//region Grammar AST

export type {
  Adverbial,
  DeclarativeSentence,
  ExternalText,
  Fragment,
  FragmentPart,
  ImperativeSentence,
  NonFiniteComplement,
  NounPhrase,
  Possessor,
  Sentence,
  SubjectRef,
  VerbPhrase,
  WhQuestion,
  YesNoQuestion,
} from './ast.ts';

//endregion Grammar AST

//region Locale spec and factory

export type {
  LabelOf,
  LocaleSpec,
  NounOf,
  SubjectOf,
  VerbOf,
} from './locale-spec.ts';

export {
  createI18n,
  type CreateI18nConfig,
  type I18n,
} from './create-i18n.ts';

//endregion Locale spec and factory

//region Locale builders

export {
  defineEnglishLocale,
  type DefineEnglishLocaleInput,
  type EnglishVerbEntry,
} from './locale/en/index.ts';

export {
  type ChineseVerbEntry,
  defineChineseLocale,
  type DefineChineseLocaleInput,
} from './locale/zh/index.ts';

export {
  type CatalanVerbEntry,
  defineCatalanLocale,
  type DefineCatalanLocaleInput,
} from './locale/ca/index.ts';

export { defineCustomLocale, } from './locale/custom.ts';

//endregion Locale builders
