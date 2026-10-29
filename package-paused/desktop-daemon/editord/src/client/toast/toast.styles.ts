/**
 * Global styles for ephemeral toast notifications.
 *
 * Injected once into the document head by {@link toast.ts}.
 * Dynamic positional properties are set inline per toast instance.
 */

import {
  cssNum,
  cssPercent,
  cssRem,
  cssTranslateX,
  cssVar,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

import {
  MONO_FONT_FAMILY,
  POPUP_BORDER_RADIUS,
  POPUP_FONT_SIZE,
} from '../styles/tokens.ts';

/**
 * Toast opacity: (10-1)/10.
 */
const OPACITY = (10 - 1) / 10;

/**
 * Toast stacking order above other fixed elements.
 */
const Z_INDEX = 200;

/**
 * Horizontal centering offset as percentage.
 */
const CENTER_OFFSET = -50;

/**
 * Horizontal centering position as percentage.
 */
const CENTER_POSITION = 50;

/**
 * Global toast styles with nested variant rules.
 */
export const STYLES: string = $({
  rule: '.toast',
  decls: {
    position: 'fixed',
    'z-index': cssNum(Z_INDEX,),
    'background-color': cssVar('hover-bg',),
    color: cssVar('fg',),
    'border-radius': cssRem(POPUP_BORDER_RADIUS,),
    'font-family': MONO_FONT_FAMILY,
    'font-size': cssRem(POPUP_FONT_SIZE,),
    'pointer-events': 'none',
    opacity: cssNum(OPACITY,),
  },
  children: [
    $({
      rule: '&[data-variant="fixed"]',
      decls: {
        'inset-block-start': cssRem(1,),
        'inset-inline-start': cssPercent(CENTER_POSITION,),
        transform: cssTranslateX(cssPercent(CENTER_OFFSET,),),
        'padding-block': cssRem(1 / 2,),
        'padding-inline': cssRem(1,),
      },
    },),
    $({
      rule: '&[data-variant="cursor"]',
      decls: {
        'padding-block': cssRem(1 / (2 * 2),),
        'padding-inline': cssRem(1 / 2,),
      },
    },),
  ],
},);
