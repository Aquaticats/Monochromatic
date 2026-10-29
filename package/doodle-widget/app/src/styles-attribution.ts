/**
 Attribution bar CSS styles for the doodle widget.
 
 Styles the bottom attribution bar as a thin, muted strip
 with a source code link. Background matches the toolbar
 for visual symmetry.
 */
import {
  cssOklch,
  cssRem,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

/**
 Attribution bar font size, smaller than body text
 */
const ATTRIBUTION_FONT_SIZE = (1 / 2) + ((1 / 2) / 2);

/**
 Vertical padding for the attribution bar
 */
const ATTRIBUTION_PADDING_BLOCK = 1 / 2
  / 2;

/**
 Horizontal padding for the attribution bar
 */
const ATTRIBUTION_PADDING_INLINE = 1;

/**
 Generates CSS rules for the attribution bar.
 
 @returns array of minified CSS rule strings
 
 @example
 ```ts
 const rules = renderAttributionStyles();
 ```
 */
export function renderAttributionStyles(): string[] {
  return [
    $({
      rule: '.attribution',
      decls: {
        'text-align': 'end',

        'overflow-wrap': 'break-word',
        'padding-block': cssRem(ATTRIBUTION_PADDING_BLOCK,),
        'padding-inline': cssRem(ATTRIBUTION_PADDING_INLINE,),
        'font-family': 'sans-serif',
        'font-size': cssRem(ATTRIBUTION_FONT_SIZE,),
        'background-color': cssOklch({
          l: 0.95,
          c: 0,
          h: 0,
        },),
      },
    },),

    $({
      rule: '.attribution a',
      decls: {
        color: cssOklch({
          l: 0.45,
          c: 0,
          h: 0,
        },),
        'text-decoration': 'none',
      },
    },),

    $({
      rule: '.attribution a:hover',
      decls: {
        'text-decoration': 'underline',
      },
    },),
  ];
}
