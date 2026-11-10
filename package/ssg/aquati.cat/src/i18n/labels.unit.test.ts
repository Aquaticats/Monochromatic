/**
 Tests that every label the pages can ask for is present in every locale.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  i18n,
  locales,
  type Locale,
} from '../../dist/final/node/index.mjs';

/**
 Label keys the site renders, one entry per key in the `Label` union.
 
 Kept as a list rather than read from the type, because a runtime test cannot
 see a type; the union and the locale tables stay in lockstep through the
 `Record<Label, string>` annotation on each table, so a key added to the union
 without a table entry fails the type check, and a key removed from the union
 fails this list.
 */
const LABEL_KEYS = [
  'siteName',
  'siteDescription',
  'chooseALang',
  'searchPlaceholder',
  'postNotInLang',
  'redirectingToLangChooser',
  'themeToggle',
  'langSwitcher',
  'published',
  'updated',
  'brandImageAlt',
  'alertCaution',
  'alertImportant',
  'alertNote',
  'alertTip',
  'alertWarning',
] as const;

await describe({
  name: 'label vocabulary',
  children: [
    ...LABEL_KEYS.map(function labelKeyCase(key,) {
      return it({
        name: `resolves ${key} in every locale`,
        fn: async function resolvesInEveryLocale(): Promise<void> {
          for (const lang of locales as readonly Locale[]) {
            expect(
              i18n.label(
                lang,
                key,
              ).length,
            ).toBeGreaterThan(0,);
          }
        },
      },);
    },),
    it({
      name: 'leaves no key with a single English value in every locale',
      fn: async function leavesNoUntranslatedKey(): Promise<void> {
        /**
         Keys whose value is identical in all three locales.
         
         A brand name and a loanword can legitimately match; an alert label
         cannot, because leaking English chrome into a Catalan or Chinese page is
         the defect this suite guards.
         */
        const identicalEverywhere = LABEL_KEYS
          .filter(function sameInEveryLocale(key,) {
            /**
             Values this key resolves to, one per locale.
             */
            const values = new Set(locales.map(function valueIn(lang,) {
              return i18n.label(
                lang,
                key,
              );
            },),);
            return values.size === 1;
          },);

        expect(identicalEverywhere,).toEqual(['siteName',],);
      },
    },),
  ],
},);
