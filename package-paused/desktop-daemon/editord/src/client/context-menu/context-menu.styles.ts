/**
 * Global styles for the context menu popover.
 *
 * The context menu anchor and popup live in light DOM (`document.body`)
 * so they need global CSS rules built via the CSS pipeline.
 */

import {
  cssAnchor,
  cssCommaList,
  cssNum,
  cssRem,
  cssVar,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

import {
  MONO_FONT_FAMILY,
  POPUP_BORDER_DECLS,
  POPUP_BORDER_RADIUS,
} from '../styles/tokens.ts';

/**
 * Minimum width of the popup in rem: 10 = 2 * (2 * 2 + 1).
 */
const MIN_WIDTH = 2 * ((2 * 2) + 1);

/**
 * Input field opacity: (10 - 2 - 1) / 10 = 7/10 for subtle appearance.
 */
const INPUT_OPACITY = (10 - 2
  - 1) / 10;

/**
 * Global context menu styles.
 */
export const STYLES: string = [
  $({
    rule: '.ctx-anchor',
    decls: {
      position: 'fixed',
      'anchor-name': '--ctx-anchor',
      'inline-size': cssNum(0,),
      'block-size': cssNum(0,),
      'pointer-events': 'none',
    },
  },),
  $({
    rule: '.ctx-popup',
    decls: {
      inset: 'auto',
      margin: cssNum(0,),
      'position-anchor': '--ctx-anchor',
      'inset-block-start': cssAnchor('end',),
      'inset-inline-start': cssAnchor('start',),
      'position-try-fallbacks': cssCommaList([
        'flip-block',
        'flip-inline',
        'flip-block flip-inline',
      ],),
      'min-inline-size': cssRem(MIN_WIDTH,),
      'background-color': cssVar('bg',),
      ...POPUP_BORDER_DECLS,
      'border-radius': cssRem(POPUP_BORDER_RADIUS,),
      'padding-block': cssRem(1 / (2 * 2),),
      'font-family': MONO_FONT_FAMILY,
      'font-size': cssRem(1,),
      color: cssVar('fg',),
    },
  },),
  $({
    rule: '.ctx-input-row',
    decls: {
      display: 'flex',
      'align-items': 'center',
      gap: cssRem(1 / 2,),
      'padding-block': cssRem(1 / (2 * 2),),
      'padding-inline': cssRem(1 / 2,),
      'white-space': 'nowrap',
    },
  },),
  $({
    rule: '.ctx-input',
    decls: {
      'flex-grow': cssNum(1,),
      'min-inline-size': cssRem(2 * 2
        * 2,),
      'background-color': 'transparent',
      'border-style': 'none',
      'border-block-end-style': 'solid',
      'border-block-end-width': cssRem(1 / 16,),
      'border-block-end-color': cssVar('fg',),
      color: cssVar('fg',),
      'font-family': 'inherit',
      'font-size': 'inherit',
      'outline-style': 'none',
      opacity: cssNum(INPUT_OPACITY,),
    },
  },),
  $({
    rule: '.ctx-label',
    decls: { 'flex-shrink': cssNum(0,), },
  },),
]
  .join('',);
