/**
 * Shadow DOM styles for the `<hover-popup>` web component.
 */

import {
  cssNum,
  cssPercent,
  cssRem,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

import {
  CODE_LINE_HEIGHT,
  POPUP_HOST_DECLS,
} from '../styles/tokens.ts';

/**
 * Invisible bridge that extends upward so the mouse can reach the popup.
 */
const BRIDGE_HEIGHT_REM = 1;

/**
 * Maximum width of the hover popup as percentage of viewport.
 */
const MAX_WIDTH_PERCENT = 60;

/**
 * Maximum height of the hover popup in rem.
 */
const MAX_HEIGHT_REM = 20;

/**
 * Padding in rem.
 */
const PADDING = 1 / 2;

/**
 * Shadow DOM styles for the hover popup.
 */
export const STYLES: string = [
  $({
    rule: ':host',
    decls: {
      ...POPUP_HOST_DECLS,
      position: 'fixed',
      'max-inline-size': cssPercent(MAX_WIDTH_PERCENT,),
      'padding-block': cssRem(PADDING,),
      'padding-inline': cssRem(PADDING,),
      'white-space': 'pre-wrap',
      'word-break': 'break-word',
    },
  },),
  $({
    rule: '.content',
    decls: {
      'max-block-size': cssRem(MAX_HEIGHT_REM,),
      overflow: 'auto',
    },
  },),
  $({
    rule: ':host::before',
    decls: {
      content: "''",
      position: 'absolute',
      'inset-inline-start': cssNum(0,),
      'inset-inline-end': cssNum(0,),
      'inset-block-start': cssRem(-BRIDGE_HEIGHT_REM,),
      'block-size': cssRem(BRIDGE_HEIGHT_REM,),
    },
  },),
]
  .join('',);
