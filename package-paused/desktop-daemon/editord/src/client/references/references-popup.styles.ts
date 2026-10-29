/**
 * Shadow DOM styles for the `<references-popup>` web component.
 *
 * Anchor positioning properties are set as inline styles on the host
 * to avoid shadow DOM scope issues with `anchor-name` resolution.
 * The Popover API provides top-layer rendering and light dismiss.
 */

import {
  cssAnchor,
  cssCommaList,
  cssNum,
  cssRem,
  cssVar,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

import { POPUP_HOST_DECLS, } from '../styles/tokens.ts';

/**
 * Maximum width of the references popup in rem.
 */
const MAX_WIDTH_REM = 30;

/**
 * Maximum height of the references popup in rem.
 */
const MAX_HEIGHT_REM = 16;

/**
 * Item padding in rem: 1/4.
 */
const ITEM_PADDING = 1 / (2 * 2);

/**
 * Shadow DOM styles for the references popup.
 */
export const STYLES: string = [
  $({
    rule: ':host',
    decls: {
      ...POPUP_HOST_DECLS,
      'inset-block-start': cssAnchor('end',),
      'inset-inline-start': cssAnchor('start',),
      'position-try-fallbacks': cssCommaList([
        'flip-block',
        'flip-inline',
        'flip-block flip-inline',
      ],),
      'inline-size': 'max-content',
      'max-inline-size': cssRem(MAX_WIDTH_REM,),
      'max-block-size': cssRem(MAX_HEIGHT_REM,),
      overflow: 'auto',
    },
  },),
  $({
    rule: '.item',
    decls: {
      display: 'flex',
      'align-items': 'baseline',
      'padding-block': cssRem(ITEM_PADDING,),
      'padding-inline': cssRem(ITEM_PADDING * 2,),
      cursor: 'pointer',
      'white-space': 'nowrap',
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
    rule: '.item-path',
    decls: {
      direction: 'rtl',
      'flex-shrink': cssNum(1,),
      overflow: 'hidden',
      'text-overflow': 'ellipsis',
    },
  },),
  $({
    rule: '.line-num',
    decls: {
      color: cssVar('gutter-fg',),
      'flex-shrink': cssNum(0,),
    },
  },),
]
  .join('',);
