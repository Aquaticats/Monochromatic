/**
 Tests that the chrome MDX components render follows the post's locale.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CalloutAlert,
  createMdxComponents,
  type Locale,
  renderMdx,
  siteHeaderHtml,
} from '../../dist/final/node/index.mjs';

/**
 Alert types the callout renders a label for, with the label each locale owes it.
 */
const ALERT_LABEL_CASES = [{
  type: 'note',
  labels: {
    ca: 'Nota',
    en: 'Note',
    zh: '说明',
  },
}, {
  type: 'warning',
  labels: {
    ca: 'Avís',
    en: 'Warning',
    zh: '警告',
  },
}] as const;

/**
 Alert body shared by the callout fixtures.
 */
const ALERT_BODY = 'Highlights information that users should take into account.';

/**
 Renders one callout in one locale.
 
 @param lang - locale to render in
 
 @param type - alert kind
 
 @returns rendered HTML
 
 @example
 ```ts
 calloutHtml('ca', 'note');
 ```
 */
function calloutHtml(
  lang: Locale,
  type: (typeof ALERT_LABEL_CASES)[number]['type'],
): string {
  return CalloutAlert({
    children: ALERT_BODY,
    lang,
    'data-type': type,
  },).html;
}

await describe({
  name: 'localized chrome',
  children: [
    describe({
      name: CalloutAlert.name,
      children: ALERT_LABEL_CASES.flatMap(function alertCases(caseOf,) {
        return Object.entries(caseOf.labels,)
          .map(function localeCase([lang, label,],) {
            return it({
              name: `labels a ${caseOf.type} callout "${label}" in ${lang}`,
              fn: async function rendersLocalizedLabel(): Promise<void> {
                /**
                 Rendered callout HTML.
                 */
                const html = calloutHtml(lang as Locale, caseOf.type,);

                expect(html,).toContain(`<span>${label}</span>`,);
              },
            },);
          },);
      },),
    },),
    describe({
      name: createMdxComponents.name,
      children: [
        it({
          name: 'binds the locale into the MDX callout scope',
          fn: async function bindsLocale(): Promise<void> {
            /**
             Component map an MDX render in Catalan would resolve against.
             */
            const components = createMdxComponents('ca',);

            expect(components.CalloutAlert({
              children: ALERT_BODY,
              'data-type': 'note',
            },).html,).toContain('<span>Nota</span>',);
          },
        },),
      ],
    },),
    describe({
      name: renderMdx.name,
      children: [
        it({
          name: 'renders a Catalan post callout with its Catalan label',
          fn: async function rendersCatalanCallout(): Promise<void> {
            /**
             Rendered body of a post whose only content is a note callout.
             */
            const html = await renderMdx({
              body: `<CalloutAlert data-type="note">${ALERT_BODY}</CalloutAlert>`,
              lang: 'ca',
            },);

            expect(html,).toContain('<span>Nota</span>',);
            expect(html,).not.toContain('<span>Note</span>',);
          },
        },),
      ],
    },),
    describe({
      name: 'site-header brand image',
      children: [
        it({
          name: 'names the brand image in the page locale',
          fn: async function namesBrandImageInLocale(): Promise<void> {
            /**
             Header markup rendered in Chinese.
             */
            const html = siteHeaderHtml({ lang: 'zh', },);

            expect(html,).toContain('alt="头像"',);
          },
        },),
      ],
    },),
  ],
},);
