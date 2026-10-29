/**
 Tests for `createI18n` and the registry helpers it returns.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { createI18n, } from './create-i18n.ts';
import type { LocaleSpec, } from './locale-spec.ts';
import { defineCatalanLocale, } from './locale/ca/index.ts';
import { defineCustomLocale, } from './locale/custom.ts';
import { defineEnglishLocale, } from './locale/en/index.ts';
import { defineChineseLocale, } from './locale/zh/index.ts';
import {
  caLabels,
  caNouns,
  caSubjects,
  caVerbs,
  enLabels,
  enNouns,
  enSubjects,
  enVerbs,
  type TestLabel,
  type TestNoun,
  type TestSubject,
  type TestVerb,
  zhLabels,
  zhNouns,
  zhSubjects,
  zhVerbs,
} from './test-vocab/index.ts';

/** Locale list used across the test cases below. */
const locales = ['ca', 'en', 'zh',] as const;

/** Locale union derived from the const list, mirrored by the I18n type parameter. */
type Locale = typeof locales[number];

/** Pre-built specs for the three test locales. */
const specs = {
  ca: defineCatalanLocale({ labels: caLabels, subjects: caSubjects, nouns: caNouns,
    verbs: caVerbs, },),
  en: defineEnglishLocale({ labels: enLabels, subjects: enSubjects, nouns: enNouns,
    verbs: enVerbs, },),
  zh: defineChineseLocale({ labels: zhLabels, subjects: zhSubjects, nouns: zhNouns,
    verbs: zhVerbs, },),
} as const;

/** Shared i18n instance for the registry/label tests below. */
const i18n = createI18n({
  locales,
  defaultLocale: 'en',
  specs,
},);

/**
 Builds a minimal locale spec for type-level vocabulary tests.
 
 @returns locale spec whose render methods return deterministic placeholders
 
 @example
 ```ts
 passthroughSpec<'label', 'subject', 'verb', 'noun'>();
 ```
 */
function passthroughSpec<
  Label extends string,
  Subject extends string,
  Verb extends string,
  Noun extends string,
  >(): LocaleSpec<Label, Subject, Verb, Noun> {
  return defineCustomLocale<Label, Subject, Verb, Noun>({
    renderLabel(key,): string {
      return key;
    },
    renderNoun(key,): string {
      return key;
    },
    renderNounPhrase(): string {
      return 'noun phrase';
    },
    renderVerbPhrase(): string {
      return 'verb phrase';
    },
    renderSentence(): string {
      return 'sentence';
    },
    renderFragment(): string {
      return 'fragment';
    },
  },);
}

await describe({
  name: createI18n.name,
  children: [
    it({
      name: 'returns the const locale list and default locale verbatim',
      fn: async () => {
        expect(i18n.locales,).toEqual(['ca', 'en', 'zh',],);
        expect(i18n.defaultLocale,).toBe('en',);
      },
    },),

    it({
      name: 'isLocale narrows arbitrary strings to the registered locale union',
      fn: async () => {
        expect(i18n.isLocale('en',),).toBe(true,);
        expect(i18n.isLocale('ca',),).toBe(true,);
        expect(i18n.isLocale('zh',),).toBe(true,);
        expect(i18n.isLocale('fr',),).toBe(false,);
        expect(i18n.isLocale('',),).toBe(false,);
      },
    },),

    it({
      name: 'assertLocale returns the narrowed value for registered locales',
      fn: async () => {
        expect(i18n.assertLocale('en',),).toBe('en',);
        expect(i18n.assertLocale('ca',),).toBe('ca',);
      },
    },),

    it({
      name: 'assertLocale throws for unregistered locales',
      fn: async () => {
        expect(() => i18n.assertLocale('fr',)).toThrow('Expected one of',);
      },
    },),

    it({
      name: 'label resolves consumer label keys per locale',
      fn: async () => {
        expect(i18n.label('en', 'siteName',),).toBe('Aquaticat',);
        expect(i18n.label('zh', 'noResults',),).toBe('无结果',);
        expect(i18n.label('ca', 'noResults',),).toBe('Sense resultats',);
      },
    },),

    it({
      name: 'noun resolves bare noun surfaces per locale',
      fn: async () => {
        expect(i18n.noun('en', 'cat',),).toBe('cat',);
        expect(i18n.noun('zh', 'cat',),).toBe('猫',);
        expect(i18n.noun('ca', 'cat',),).toBe('gat',);
      },
    },),

    it({
      name: 'locale parameter is constrained to the const locale union',
      fn: async () => {
        /** Render call typed against the inferred locale union. */
        const out = i18n.label('en', 'siteName',);
        expectTypeOf(out,).toBeString();
        expectTypeOf(i18n.locales,).toEqualTypeOf<readonly Locale[]>();
      },
    },),

    it({
      name: 'label key is constrained to the consumer label union',
      fn: async () => {
        /** Parameter typed against the inferred Label union. */
        type FirstParam = Parameters<typeof i18n.label>[1];
        expectTypeOf<FirstParam>().toEqualTypeOf<TestLabel>();
      },
    },),

    it({
      name: 'noun key is constrained to the consumer noun union',
      fn: async () => {
        type FirstParam = Parameters<typeof i18n.noun>[1];
        expectTypeOf<FirstParam>().toEqualTypeOf<TestNoun>();
      },
    },),

    it({
      name: 'np and vp expose the consumer subject/verb unions on their AST parameter',
      fn: async () => {
        type NpAst = Parameters<typeof i18n.np>[1];
        /** Counted variant should accept any noun from the consumer union. */
        const counted: NpAst = { kind: 'noun.counted', count: 1, noun: 'cat', };
        expect(counted.noun,).toBe('cat',);
        type VpAst = Parameters<typeof i18n.vp>[1];
        const vp: VpAst = { kind: 'verbPhrase', verb: 'save', };
        expect(vp.verb,).toBe('save',);
        expectTypeOf<TestSubject>().toBeString();
        expectTypeOf<TestVerb>().toBeString();
      },
    },),

    it({
      name: 'spec records must expose the same vocabulary in every locale',
      fn: async () => {
        /** Config with extra vocabulary in only one locale should fail at the factory boundary. */
        const mismatchedConfig = {
          locales: ['left', 'right',],
          defaultLocale: 'left',
          specs: {
            left: passthroughSpec<
              'sharedLabel' | 'leftOnlyLabel',
              'sharedSubject' | 'leftOnlySubject',
              'sharedVerb' | 'leftOnlyVerb',
              'sharedNoun' | 'leftOnlyNoun'
              >(),
            right: passthroughSpec<
              'sharedLabel',
              'sharedSubject',
              'sharedVerb',
              'sharedNoun'
              >(),
          },
        } as const;
        // @ts-expect-error -- createI18n rejects locale specs whose vocabulary keys differ.
        createI18n(mismatchedConfig,);
        expect(mismatchedConfig.defaultLocale,).toBe('left',);
      },
    },),
  ],
},);
