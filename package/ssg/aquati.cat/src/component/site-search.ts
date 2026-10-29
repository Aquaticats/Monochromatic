/**
 Expanding search input and results dropdown component.
 
 The input is icon-sized by default with a Material Symbols `search`
 icon centered over it. On `:focus` the input expands, reveals
 placeholder text, and shifts the icon to the inline-start edge.
 The results dropdown positions absolutely below the search wrapper.
 
 Internal class names (`.search-icon`, `.search-input`, `.search-results`,
 `.search-title`, `.search-excerpt`) are scoped to this file's CSS.
 The `.search-title` and `.search-excerpt` classes are also generated
 in client-side innerHTML by `src/client/search.ts`.
 */
import {
  cssCalc,
  cssCommaList,
  cssInt,
  cssRem,
  cssS,
  type CssValue,
  cssVar,
  hCss as $,
  hHtml as h,
} from '@monochromatic-dev/hyperscript-core/ts';

import {
  i18n,
  type Locale,
} from '../i18n/index.ts';
import { icon, } from '../lib/icon/icon.ts';
import {
  BORDER_WIDTH_REM,
  FONT_SIZE_SMALL,
  GAP,
  GAP_SMALL,
  TOUCH_TARGET,
} from '../style/constants.ts';

//region Constants

/**
 Collapsed search input size in rem (matches touch target).
 */
const SEARCH_COLLAPSED = TOUCH_TARGET;

/**
 Expanded search input width in rem.
 */
const SEARCH_EXPANDED = 16;

/**
 Transition duration for the search input expand/collapse.
 */
const SEARCH_TRANSITION = cssS(1 / 2
  / 2,);

//endregion Constants

//region CSS

/**
 Search input, icon overlay, and results dropdown styles.
 
 @returns CSS string for the `<site-search>` element
 
 @example
 ```ts
 const styles = css();
 ```
 */
export function css(): string {
  return [
    $({
      rule: 'site-search search',
      decls: {
        position: 'relative',
        display: 'inline-flex',
        'align-items': 'center',
      },
    },),
    $({
      rule: 'site-search .search-icon',
      decls: {
        position: 'absolute',
        inset: '0',
        display: 'flex',
        'align-items': 'center',
        'justify-content': 'center',
        'pointer-events': 'none',
        // oxlint-disable-next-line no-unsafe-type-assertion -- StrictValue<TransitionProperty> rejects individual property names
        'transition-property': 'justify-content' as CssValue,
        'transition-duration': SEARCH_TRANSITION,
        'transition-timing-function': 'ease-out',
      },
    },),
    $({
      rule: 'site-search .search-input',
      decls: {
        'inline-size': cssRem(SEARCH_COLLAPSED,),
        'block-size': cssRem(SEARCH_COLLAPSED,),
        'padding-block': cssInt(0,),
        'padding-inline': cssInt(0,),
        'border-style': 'solid',
        'border-width': cssCalc(BORDER_WIDTH_REM,),
        'border-color': 'transparent',
        'border-radius': cssRem(SEARCH_COLLAPSED / 2,),
        'background-color': 'transparent',
        color: 'transparent',
        'font-size': cssRem(1,),
        cursor: 'pointer',
        'transition-property': cssCommaList([
          'inline-size',
          'padding-inline',
          'border-color',
          'background-color',
          'color',
        ],),
        'transition-duration': SEARCH_TRANSITION,
        'transition-timing-function': 'ease-out',
      },
      children: [
        $({
          rule: '&:focus',
          decls: {
            color: 'inherit',
            'inline-size': cssRem(SEARCH_EXPANDED,),
            'padding-inline-start': cssRem(2 + (1 / 2),),
            'padding-inline-end': cssRem(GAP_SMALL,),
            'border-color': cssVar('color-border',),
            'background-color': cssVar('color-bg',),
            cursor: 'text',
            'outline-color': cssVar('color-focus-ring',),
            'outline-style': 'solid',
            'outline-width': cssCalc(BORDER_WIDTH_REM,),
            'outline-offset': cssCalc(BORDER_WIDTH_REM,),
          },
        },),
        $({
          rule: '&::placeholder',
          decls: {
            color: 'transparent',
            // oxlint-disable-next-line no-unsafe-type-assertion -- StrictValue<TransitionProperty> rejects individual property names
            'transition-property': 'color' as CssValue,
            'transition-duration': SEARCH_TRANSITION,
          },
        },),
        $({
          rule: '&:focus::placeholder',
          decls: {
            color: cssVar('color-muted',),
          },
        },),
        $({
          rule: '&:focus ~ .search-icon',
          decls: {
            'justify-content': 'start',
            'padding-inline-start': cssRem(GAP_SMALL,),
          },
        },),
      ],
    },),
    $({
      rule: 'site-search .search-results',
      decls: {
        position: 'absolute',
        inset: `${cssRem(SEARCH_COLLAPSED,)} 0 auto auto`,
        'min-inline-size': cssRem(SEARCH_EXPANDED,),
        'max-block-size': cssRem(16 + (2 * 2
          * 2),),
        'overflow-y': 'auto',
        'margin-block': cssInt(0,),
        'padding-block': cssRem(GAP_SMALL,),
        'padding-inline': cssInt(0,),
        'list-style-type': 'none',
        'background-color': cssVar('color-bg',),
        'border-style': 'solid',
        'border-width': cssCalc(BORDER_WIDTH_REM,),
        'border-color': cssVar('color-border',),
        'border-radius': cssRem(GAP_SMALL,),
        // oxlint-disable-next-line no-unsafe-type-assertion -- template literal produces valid box-shadow but doesn't narrow to StrictValue
        'box-shadow': `0 ${cssRem(GAP_SMALL,)} ${
          cssRem(GAP,)
        } rgba(0, 0, 0, 0.1)` as CssValue,
        'z-index': 10,
      },
      children: [
        $({
          rule: '&:empty',
          decls: {
            display: 'none',
          },
        },),
        $({
          rule: 'li',
          decls: {
            'padding-block': cssRem(GAP_SMALL,),
            'padding-inline': cssRem(GAP,),
          },
          children: [
            $({
              rule: '&:hover, &[data-active]',
              decls: {
                'background-color': cssVar('color-code-bg',),
              },
            },),
          ],
        },),
        $({
          rule: 'a',
          decls: {
            'text-decoration-line': 'none',
            color: 'inherit',
            display: 'block',
          },
        },),
        $({
          rule: '.search-title',
          decls: {
            'font-weight': 600,
          },
        },),
        $({
          rule: '.search-excerpt',
          decls: {
            'font-size': cssRem(FONT_SIZE_SMALL,),
            color: cssVar('color-muted',),
            'margin-block-start': cssRem(GAP_SMALL / 2,),
          },
        },),
        $({
          rule: 'mark',
          decls: {
            'background-color': 'transparent',
            color: cssVar('color-link',),
            'font-weight': 600,
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
 Renders the expanding search input with results dropdown
 as a `<site-search>` custom element.
 
 The `<search>` landmark wraps an `<input type="search">` that collapses
 to icon size when unfocused, with a Material Symbols `search` icon
 overlaid via absolute positioning, and an empty `<ul>` for Pagefind
 results populated client-side.
 
 @param lang - locale code resolving the placeholder text
 
 @returns HTML string for the search widget
 
 @example
 ```ts
 const markup = html('en');
 ```
 */
export function html(lang: Locale,): string {
  return h({
    tag: 'site-search',
    attrs: { 'data-is': '', },
    children: [
      h({
        tag: 'search',
        children: [
          h({
            tag: 'input',
            attrs: {
              type: 'search',
              id: 'search-input',
              placeholder: i18n.label(
                lang,
                'searchPlaceholder',
              ),
              autocomplete: 'off',
              role: 'combobox',
              'aria-label': i18n.label(
                lang,
                'searchPlaceholder',
              ),
              'aria-controls': 'search-results',
              'aria-expanded': 'false',
              'aria-autocomplete': 'list',
            },
            class: 'search-input',
          },),
          h({
            tag: 'span',
            class: 'material-symbols-outlined search-icon',
            attrs: { 'aria-hidden': 'true', },
            text: icon('search',),
          },),
          h({
            tag: 'ul',
            attrs: {
              id: 'search-results',
              role: 'listbox',
              'aria-label': i18n.label(
                lang,
                'searchPlaceholder',
              ),
            },
            class: 'search-results',
          },),
        ],
      },),
    ],
  },);
}

//endregion HTML
