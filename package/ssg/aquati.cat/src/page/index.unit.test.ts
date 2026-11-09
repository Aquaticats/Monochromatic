/**
 Tests for the root language picker page.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  locales,
  type Locale,
} from '../i18n/index.ts';
import { LANG_NAMES, } from '../i18n/lang-names.ts';

import { indexPage, } from './index.ts';

/**
 Canonical URL the picker fixture renders for.
 */
const CANONICAL_URL = 'https://aquati.cat/';

/**
 Languages offered by the picker fixture.
 */
const FIXTURE_LANGS: readonly Locale[] = locales;

await describe({
  name: indexPage.name,
  children: [
    it({
      name: 'labels each choice with the language autonym',
      fn: async function labelsChoicesWithAutonyms(): Promise<void> {
        /**
         Rendered root index HTML.
         */
        const html = indexPage({
          langs: FIXTURE_LANGS,
          canonicalUrl: CANONICAL_URL,
        },);

        for (const lang of FIXTURE_LANGS) {
          expect(html,).toContain(`<a href="/${lang}">${LANG_NAMES[lang]}</a>`,);
        }
      },
    },),
    it({
      name: 'never shows a bare locale code as the link text',
      fn: async function neverShowsBareLocaleCode(): Promise<void> {
        /**
         Rendered root index HTML.
         */
        const html = indexPage({
          langs: FIXTURE_LANGS,
          canonicalUrl: CANONICAL_URL,
        },);

        for (const lang of FIXTURE_LANGS)
          expect(html,).not.toContain(`>${lang}</a>`,);
      },
    },),
  ],
},);
