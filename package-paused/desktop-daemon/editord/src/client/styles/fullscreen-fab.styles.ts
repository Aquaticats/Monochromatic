/**
 * Global styles for the fullscreen FAB button.
 *
 * Injected into the document head via the CSS build pipeline.
 * Hover opacity is handled via CSS `:hover` instead of JS event listeners.
 */

import {
  cssCommaList,
  cssNum,
  cssPercent,
  cssRem,
  cssS,
  cssVar,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

/**
 * FAB stacking order above all other fixed elements.
 */
const Z_INDEX = 9_999;

/**
 * Default opacity: 6/10.
 */
const DEFAULT_OPACITY = (2 + 1) / ((2 * 2) + 1);

/**
 * Opacity transition denominator: 20, expressed with exempt literals.
 */
const TRANSITION_DURATION_DENOMINATOR = (2 * 2) * ((2 * 2) + 1);

/**
 * Opacity transition duration in seconds: 0.15 = 3/20.
 */
const TRANSITION_DURATION = (2 + 1) / TRANSITION_DURATION_DENOMINATOR;

/**
 * Circle percentage value: 50, expressed with exempt literals.
 */
const CIRCLE_PERCENT = ((2 * 2) + 1) * ((2 * 2) + 1)
  * 2;

/**
 * Minimum touch target size in rem: 3.
 */
const TOUCH_TARGET = 2 + 1;

/**
 * Global fullscreen FAB styles.
 */
export const STYLES: string = $({
  rule: '.fullscreen-fab',
  decls: {
    position: 'fixed',
    'inset-block-end': cssRem(1,),
    'inset-inline-start': cssRem(1,),
    'z-index': cssNum(Z_INDEX,),
    display: 'flex',
    'align-items': 'center',
    'justify-content': 'center',
    'min-inline-size': cssRem(TOUCH_TARGET,),
    'min-block-size': cssRem(TOUCH_TARGET,),
    'border-radius': cssPercent(CIRCLE_PERCENT,),
    'border-style': 'none',
    'background-color': cssVar('gutter-fg',),
    color: cssVar('bg',),
    cursor: 'pointer',
    opacity: cssNum(DEFAULT_OPACITY,),
    'transition-property': cssCommaList(['opacity',],),
    'transition-duration': cssS(TRANSITION_DURATION,),
  },
  children: [
    $({
      rule: '&:hover',
      decls: { opacity: cssNum(1,), },
    },),
    $({
      rule: '&:focus-visible',
      decls: {
        'outline-color': cssVar('fg',),
        'outline-offset': cssRem(1 / (2 * 2),),
      },
    },),
  ],
},);
