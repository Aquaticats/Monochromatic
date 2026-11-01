/**
 Language switcher dropdown.
 
 Renders a globe-icon trigger that opens a menu listing each supported
 locale by its autonym. Items link to the same post in the target
 locale when post context is available, falling back to the locale
 landing otherwise.
 
 Uses native `<details>` / `<summary>` so the open/close click is
 handled by the browser; no client-side JavaScript ships. Click-
 outside-to-close is not provided (an Esc handler is also not added);
 the user toggles by clicking the summary again. This is an
 intentional trade-off for shipping without JS; see
 `doc/tech-debt/lang-switcher.md`.
 */
import {
  cssCalc,
  cssPercent,
  cssRem,
  cssVar,
  hCss as $,
  hHtml as h,
} from '@monochromatic-dev/hyperscript-core/ts';

import {
  i18n,
  locales,
  type Locale,
} from '../i18n/index.ts';
import { LANG_NAMES, } from '../i18n/lang-names.ts';
import { icon, } from '../lib/icon/icon.ts';
import {
  BORDER_WIDTH_REM,
  GAP_SMALL,
  TOUCH_TARGET,
} from '../style/constants.ts';

//region Constants

/**
 Globe glyph from Material Symbols Outlined; codepoint U+E894.
 */
const LANG_ICON = icon('language',);

/**
 Minimum menu width in rem; comfortably fits the longest autonym.
 */
const MENU_MIN_INLINE_REM = 8;

//endregion Constants

//region CSS

/**
 Trigger summary, menu list, and option styles.
 
 @returns CSS string for the `<lang-switcher>` element
 
 @example
 ```ts
 const styles = css();
 ```
 */
export function css(): string {
  return [
    $({
      rule: 'lang-switcher',
      decls: {
        position: 'relative',
        display: 'inline-flex',
      },
    },),
    $({
      rule: 'lang-switcher details',
      decls: { display: 'inline-flex', },
    },),
    $({
      rule: 'lang-switcher summary',
      decls: {
        display: 'inline-flex',
        'align-items': 'center',
        'justify-content': 'center',
        'min-inline-size': cssRem(TOUCH_TARGET,),
        'min-block-size': cssRem(TOUCH_TARGET,),
        cursor: 'pointer',
        color: 'inherit',
        'list-style-type': 'none',
      },
      children: [
        $({
          rule: '&:focus-visible',
          decls: {
            'outline-color': cssVar('color-focus-ring',),
            'outline-style': 'solid',
            'outline-width': cssCalc(BORDER_WIDTH_REM,),
            'outline-offset': cssCalc(BORDER_WIDTH_REM,),
          },
        },),
      ],
    },),
    $({
      rule: 'lang-switcher summary::-webkit-details-marker',
      decls: { display: 'none', },
    },),
    $({
      rule: 'lang-switcher .menu',
      decls: {
        position: 'absolute',
        'inset-block-start': cssPercent(100,),
        'inset-inline-end': 0,
        'inset-block-end': 'auto',
        'inset-inline-start': 'auto',
        'margin-block': 0,
        'margin-inline': 0,
        'padding-block': cssRem(GAP_SMALL,),
        'padding-inline': 0,
        'background-color': cssVar('color-bg',),
        'border-style': 'solid',
        'border-width': cssCalc(BORDER_WIDTH_REM,),
        'border-color': cssVar('color-border',),
        'list-style-type': 'none',
        'min-inline-size': cssRem(MENU_MIN_INLINE_REM,),
      },
    },),
    $({
      rule: 'lang-switcher .menu a',
      decls: {
        display: 'block',
        'padding-block': cssRem(GAP_SMALL,),
        'padding-inline': cssRem(GAP_SMALL,),
        'text-decoration-line': 'none',
        color: cssVar('color-fg',),
      },
      children: [
        $({
          rule: '&:hover',
          decls: { 'background-color': cssVar('color-border',), },
        },),
        $({
          rule: '&:focus-visible',
          decls: {
            'outline-color': cssVar('color-focus-ring',),
            'outline-style': 'solid',
            'outline-width': cssCalc(BORDER_WIDTH_REM,),
            'outline-offset': 0,
          },
        },),
        $({
          rule: '&[aria-current="page"]',
          decls: { 'font-weight': 600, },
        },),
        $({
          rule: '&[aria-current="page"]::before',
          decls: {
            content: String.raw`"\2713\00a0"`,
          },
        },),
      ],
    },),
  ]
    .join('\n',);
}

//endregion CSS

//region HTML

/**
 Computes the href for a target locale's switcher item.
 
 @param targetLang - locale this item links to
 
 @param currentName - current post slug if rendering inside a post page
 
 @param availableInLangs - locales in which the current post exists
 
 @returns absolute path the item should link to
 
 @example
 ```ts
 resolveHref({ targetLang: 'ca', currentName: 'hello', availableInLangs: ['en', 'ca'] });
 // '/ca/hello'
 ```
 */
function resolveHref(
  {
    targetLang,
    currentName,
    availableInLangs,
  }: {
    readonly targetLang: Locale;
    readonly currentName?: string;
    readonly availableInLangs?: readonly Locale[];
  },
): string {
  if (currentName !== undefined) {
    /**
     Treats an unset `availableInLangs` as "all locales available" so non-post pages keep deep links.
     */
    const hasTarget = (availableInLangs === undefined)
      || availableInLangs
        .includes(targetLang,);
    if (hasTarget)
      return `/${targetLang}/${currentName}`;
  }
  return `/${targetLang}`;
}

/**
 Renders the language switcher as a `<lang-switcher>` custom element.
 
 @param currentLang - locale of the page rendering this switcher
 
 @param currentName - current post slug; enables same-post links
 
 @param availableInLangs - locales in which the current post exists;
 when set, locales outside this list fall back to `/{lang}`
 
 @returns HTML string for the language switcher
 
 @example
 ```ts
 const markup = html({ currentLang: 'en', currentName: 'hello', availableInLangs: ['en', 'ca'] });
 ```
 */
export function html(
  {
    currentLang,
    currentName,
    availableInLangs,
  }: {
    readonly currentLang: Locale;
    readonly currentName?: string;
    readonly availableInLangs?: readonly Locale[];
  },
): string {
  return h({
    tag: 'lang-switcher',
    attrs: { 'data-is': '', },
    children: [
      h({
        tag: 'details',
        children: [
          h({
            tag: 'summary',
            attrs: { 'aria-label': i18n.label(
              currentLang,
              'langSwitcher',
            ), },
            children: [
              h({
                tag: 'span',
                class: 'material-symbols-outlined',
                text: LANG_ICON,
              },),
            ],
          },),
          h({
            tag: 'ul',
            class: 'menu',
            children: locales.map(function renderItem(targetLang,) {
              /**
               URL path computed once per locale row before the link element is built.
               */
              const href = resolveHref({
                targetLang,
                ...(currentName !== undefined ? { currentName, } : {}),
                ...(availableInLangs !== undefined ? { availableInLangs, } : {}),
              },);
              /**
               Marks the current-locale item so aria-current renders consistently.
               */
              const isCurrent = targetLang === currentLang;
              return h({
                tag: 'li',
                children: [
                  h({
                    tag: 'a',
                    attrs: {
                      href,
                      hreflang: targetLang,
                      lang: targetLang,
                      ...(isCurrent ? { 'aria-current': 'page', } : {}),
                    },
                    text: LANG_NAMES[targetLang],
                  },),
                ],
              },);
            },),
          },),
        ],
      },),
    ],
  },);
}

//endregion HTML
