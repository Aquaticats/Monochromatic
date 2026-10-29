/**
 Theme toggle component.
 
 A checkbox styled as a button that inverts the site color scheme.
 The real checkbox is visually hidden but remains focusable.
 The adjacent label acts as the visible toggle, displaying
 a Material Symbols `invert_colors` icon.
 
 **Cross-component dependency**: the checkbox uses `id="theme-toggle"`,
 which `tokens.ts` references via `:root:has(#theme-toggle:checked)`
 to flip light/dark tokens. Changing this ID requires updating
 `tokens.ts` as well.
 */
import {
  cssCalc,
  cssVar,
  hCss as $,
  hHtml as h,
} from '@monochromatic-dev/hyperscript-core/ts';

import {
  i18n,
  type Locale,
} from '../i18n/index.ts';
import { icon, } from '../lib/icon/icon.ts';
import { BORDER_WIDTH_REM, } from '../style/constants.ts';

/**
 Material Symbols PUA codepoint for the theme toggle icon.
 
 Uses `invert_colors`: a single icon that represents both light
 and dark modes, replacing the previous sun/moon SVG pair. The
 literal icon name is kept in the `icon('...')` call so the
 subset-fonts source scan can pick it up.
 */
const THEME_ICON = icon('invert_colors',);

//region CSS

/**
 Theme toggle checkbox-as-button styles.
 
 @returns CSS string for the `<theme-toggle>` element
 
 @example
 ```ts
 const styles = css();
 ```
 */
export function css(): string {
  return [
    $({
      rule: 'theme-toggle input',
      decls: {
        position: 'absolute',
        'inline-size': cssCalc(BORDER_WIDTH_REM,),
        'block-size': cssCalc(BORDER_WIDTH_REM,),
        overflow: 'hidden',
        clip: 'rect(0 0 0 0)',
        'white-space': 'nowrap',
      },
      children: [
        $({
          rule: '&:focus-visible + label',
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
      rule: 'theme-toggle label',
      decls: {
        display: 'inline-flex',
        'align-items': 'center',
        'justify-content': 'center',
        cursor: 'pointer',
      },
    },),
  ]
    .join('\n',);
}

//endregion CSS

//region HTML

/**
 Renders the theme toggle as a `<theme-toggle>` custom element.
 
 @param lang - locale code resolving the aria-label
 
 @returns HTML string for the theme toggle
 
 @example
 ```ts
 const markup = html('en');
 ```
 */
export function html(lang: Locale,): string {
  return h({
    tag: 'theme-toggle',
    attrs: { 'data-is': '', },
    children: [
      h({
        tag: 'input',
        attrs: {
          type: 'checkbox',
          id: 'theme-toggle',
        },
      },),
      h({
        tag: 'label',
        attrs: {
          for: 'theme-toggle',
          'aria-label': i18n.label(
            lang,
            'themeToggle',
          ),
        },
        children: [
          h({
            tag: 'span',
            class: 'material-symbols-outlined',
            text: THEME_ICON,
          },),
        ],
      },),
    ],
  },);
}

//endregion HTML
