/**
 Toast notification CSS styles for the doodle widget.
 
 Styles the zoom instruction popover as a bottom-centered
 toast that fades in and out.
 */
import {
  cssCommaList,
  cssNum,
  cssOklch,
  cssRem,
  cssS,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

/**
 Toast corner radius in rem
 */
const TOAST_RADIUS = 1 / 2;

/**
 Toast vertical offset from bottom in rem
 */
const TOAST_BOTTOM_OFFSET = 2;

/**
 Toast font size in rem (slightly smaller than body text)
 */
const TOAST_FONT_SIZE = 1 - (1 / (2 + 2
  + 2
  + 2
  + 2));

/**
 Transition duration in seconds for fade in/out
 */
const TRANSITION_DURATION = (1 / 2) - (1 / (2 + 2
  + 1));

/**
 Generates CSS rules for the zoom toast popover.
 
 @returns array of CSS rule strings
 
 @example
 ```ts
 const rules = renderToastStyles();
 ```
 */
export function renderToastStyles(): string[] {
  return [
    $({
      rule: '#zoom-toast',
      decls: {
        'inset-block-end': cssRem(TOAST_BOTTOM_OFFSET,),
        'inset-inline': cssNum(0,),
        'inset-block-start': 'auto',
        'inline-size': 'fit-content',
        'margin-inline': 'auto',
        'padding-block': cssRem(1 / 2,),
        'padding-inline': cssRem(1,),
        'border-radius': cssRem(TOAST_RADIUS,),
        'border-block-style': 'none',
        'border-inline-style': 'none',
        'background-color': cssOklch({
          l: 0.2,
          c: 0,
          h: 0,
          a: 0.85,
        },),
        color: cssOklch({
          l: 0.95,
          c: 0,
          h: 0,
        },),
        'font-family': 'sans-serif',
        'font-size': cssRem(TOAST_FONT_SIZE,),
        'pointer-events': 'none',
        'transition-property': cssCommaList([
          'opacity',
          'display',
        ],),
        'transition-duration': cssS(TRANSITION_DURATION,),
        'transition-behavior': 'allow-discrete',
      },
    },),

    $({
      rule: '#zoom-toast:not(:popover-open)',
      decls: {
        opacity: cssNum(0,),
      },
    },),
  ];
}
