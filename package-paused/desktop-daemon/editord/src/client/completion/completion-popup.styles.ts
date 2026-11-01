/**
 * Shadow DOM styles for the `<completion-popup>` web component.
 */

import {
  cssRem,
  cssVar,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

import { POPUP_HOST_DECLS, } from '../styles/tokens.ts';

/**
 * Maximum height of the completion popup in rem.
 */
const MAX_HEIGHT_REM = 16;

/**
 * Width of the completion popup in rem.
 */
const WIDTH_REM = 20;

/**
 * Item padding in rem: 1/4.
 */
const ITEM_PADDING = 1 / (2 * 2);

/**
 * Shadow DOM styles for the completion popup.
 */
export const STYLES: string = [
  $({
    rule: ':host',
    decls: {
      ...POPUP_HOST_DECLS,
      position: 'fixed',
      'inline-size': cssRem(WIDTH_REM,),
      'max-block-size': cssRem(MAX_HEIGHT_REM,),
      overflow: 'auto',
    },
  },),
  $({
    rule: '.item',
    decls: {
      'padding-block': cssRem(ITEM_PADDING,),
      'padding-inline': cssRem(ITEM_PADDING * 2,),
      cursor: 'pointer',
      'white-space': 'nowrap',
      'text-overflow': 'ellipsis',
      overflow: 'hidden',
    },
  },),
  $({
    rule: '.item[data-selected]',
    decls: {
      'background-color': cssVar('tree-selected-bg',),
    },
  },),
  $({
    rule: '.detail',
    decls: {
      color: cssVar('gutter-fg',),
      'margin-inline-start': cssRem(1 / 2,),
    },
  },),
]
  .join('',);
