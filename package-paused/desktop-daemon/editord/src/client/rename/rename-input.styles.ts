/**
 * Shadow DOM styles for the `<rename-input>` web component.
 */

import {
  cssNum,
  cssRem,
  cssVar,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

import {
  MONO_FONT_FAMILY,
  POPUP_BORDER_DECLS,
  POPUP_BORDER_RADIUS,
  POPUP_FONT_SIZE,
} from '../styles/tokens.ts';

/**
 * Padding inside the input field in rem.
 */
const INPUT_PADDING = 1 / (2 * 2);

/**
 * Z-index for the rename input overlay, above other fixed-position elements.
 */
const OVERLAY_Z_INDEX = 1_000;

/**
 * Shadow DOM styles for the rename input.
 */
export const STYLES: string = [
  $({
    rule: ':host',
    decls: {
      inset: 'auto',
      margin: cssNum(0,),
      'background-color': cssVar('hover-bg',),
      color: cssVar('fg',),
      'border-radius': cssRem(POPUP_BORDER_RADIUS,),
      ...POPUP_BORDER_DECLS,
      position: 'fixed',
      'z-index': cssNum(OVERLAY_Z_INDEX,),
    },
  },),
  $({
    rule: 'input',
    decls: {
      'font-family': MONO_FONT_FAMILY,
      'font-size': cssRem(POPUP_FONT_SIZE,),
      'background-color': cssVar('bg',),
      color: cssVar('fg',),
      'border-block-width': cssRem(1 / 16,),
      'border-block-style': 'solid',
      'border-block-color': cssVar('hover-border',),
      'border-inline-width': cssRem(1 / 16,),
      'border-inline-style': 'solid',
      'border-inline-color': cssVar('hover-border',),
      'border-radius': cssRem(POPUP_BORDER_RADIUS,),
      'padding-block': cssRem(INPUT_PADDING,),
      'padding-inline': cssRem(INPUT_PADDING,),
      outline: 'none',
      'min-inline-size': cssRem(10,),
    },
  },),
  $({
    rule: 'input:focus-visible',
    decls: {
      'border-inline-color': cssVar('accent',),
      'border-block-color': cssVar('accent',),
    },
  },),
]
  .join('',);
