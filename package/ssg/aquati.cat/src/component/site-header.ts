/**
 Site header bar with navigation.
 
 Renders the brand logo, site name, and nav shell as a
 `<site-header>` custom element. The nav contains the
 `<theme-toggle>` and `<site-search>` sub-components.
 */
import {
  cssCalc,
  cssRem,
  cssVar,
  hCss as $,
  hHtml as h,
} from '@monochromatic-dev/hyperscript-core/ts';

import {
  i18n,
  type Locale,
} from '../i18n/index.ts';
import {
  BORDER_WIDTH_REM,
  GAP,
  GAP_SMALL,
} from '../style/constants.ts';
import { html as langSwitcherHtml, } from './lang-switcher.ts';
import { html as siteSearchHtml, } from './site-search.ts';
import { html as themeToggleHtml, } from './theme-toggle.ts';

//region CSS

/**
 Header bar layout, brand link, and nav styles.
 
 @returns CSS string for the `<site-header>` element
 
 @example
 ```ts
 const styles = css();
 ```
 */
export function css(): string {
  return [
    $({
      rule: 'site-header header',
      decls: {
        display: 'flex',
        'align-items': 'center',
        'justify-content': 'space-between',
        'padding-block': cssRem(GAP,),
        'padding-inline': cssRem(GAP,),
        'border-block-end-style': 'solid',
        'border-block-end-width': cssCalc(BORDER_WIDTH_REM,),
        'border-block-end-color': cssVar('color-border',),
      },
    },),
    $({
      rule: 'site-header .brand',
      decls: {
        display: 'flex',
        'align-items': 'center',
        gap: cssRem(GAP_SMALL,),
        'text-decoration-line': 'none',
        color: 'inherit',
        'font-weight': 600,
      },
      children: [
        $({
          rule: '& img',
          decls: {
            'inline-size': cssRem(2,),
            'block-size': cssRem(2,),
          },
        },),
      ],
    },),
    $({
      rule: 'site-header nav',
      decls: {
        display: 'flex',
        'align-items': 'center',
        gap: cssRem(GAP_SMALL,),
      },
    },),
  ]
    .join('\n',);
}

//endregion CSS

//region HTML

/**
 Renders the site header as a `<site-header>` custom element.
 
 @param lang - current language code for localized text and links
 
 @param currentName - current post slug; forwarded to the lang switcher
 so its items can link to the same post in the target locale
 
 @param availableInLangs - locales in which the current post exists;
 forwarded to the lang switcher to compute per-locale fallbacks
 
 @returns HTML string for the header
 
 @example
 ```ts
 const markup = html({ lang: 'en', currentName: 'hello', availableInLangs: ['en', 'ca'] });
 ```
 */
export function html(
  {
    lang,
    currentName,
    availableInLangs,
  }: {
    readonly lang: Locale;
    readonly currentName?: string;
    readonly availableInLangs?: readonly Locale[];
  },
): string {
  return h({
    tag: 'site-header',
    attrs: { 'data-is': '', },
    children: [
      h({
        tag: 'header',
        children: [
          h({
            tag: 'a',
            attrs: { href: `/${lang}`, },
            class: 'brand',
            children: [
              h({
                tag: 'img',
                attrs: {
                  src: '/favicon.svg',
                  alt: 'avatar',
                },
              },),
              h({
                tag: 'span',
                text: i18n.label(
                  lang,
                  'siteName',
                ),
              },),
            ],
          },),
          h({
            tag: 'nav',
            children: [
              langSwitcherHtml({
                currentLang: lang,
                ...(currentName !== undefined ? { currentName, } : {}),
                ...(availableInLangs !== undefined ? { availableInLangs, } : {}),
              },),
              themeToggleHtml(lang,),
              siteSearchHtml(lang,),
            ],
          },),
        ],
      },),
    ],
  },);
}

//endregion HTML
