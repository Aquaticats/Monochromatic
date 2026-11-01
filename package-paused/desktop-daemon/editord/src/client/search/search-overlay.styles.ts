/**
 * Shadow DOM styles for the `<search-overlay>` web component.
 *
 * Uses a native `<dialog>` element for the overlay.
 * The input sits at the top; results scroll below.
 */

import {
  cssNum,
  cssRem,
  cssVar,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

import { DIALOG_STYLES, } from './dialog.styles.ts';

/**
 * Result item padding in `rem`: 0.5 = 1 / 2.
 */
const RESULT_PADDING = 1 / 2;

/**
 * Result line height multiplier: 1.5 = (2 + 1) / 2.
 */
const RESULT_LINE_HEIGHT = (2 + 1) / 2;

/**
 * Shadow DOM styles for the search overlay.
 */
export const STYLES: string = DIALOG_STYLES + [
  $({
    rule: '.results',
    decls: {
      flex: '1',
      'overflow-y': 'auto',
      'overflow-x': 'hidden',
    },
  },),
  $({
    rule: '.result',
    decls: {
      display: 'flex',
      'align-items': 'baseline',
      'padding-block': cssRem(RESULT_PADDING / 2,),
      'padding-inline': cssRem(RESULT_PADDING,),
      cursor: 'pointer',
      'line-height': cssNum(RESULT_LINE_HEIGHT,),
      'white-space': 'nowrap',
      overflow: 'hidden',
      'text-overflow': 'ellipsis',
    },
  },),
  $({
    rule: '.result:hover, .result[data-selected]',
    decls: {
      'background-color': cssVar('tree-hover-bg',),
    },
  },),
  $({
    rule: '.result-path',
    decls: {
      'flex-shrink': cssNum(1,),
      overflow: 'hidden',
      'text-overflow': 'ellipsis',
    },
  },),
  $({
    rule: '.result-line',
    decls: {
      color: cssVar('gutter-fg',),
      'flex-shrink': cssNum(0,),
    },
  },),
  $({
    rule: '.result-text',
    decls: {
      color: cssVar('gutter-fg',),
      'margin-inline-start': cssRem(1,),
      overflow: 'hidden',
      'text-overflow': 'ellipsis',
      'flex-shrink': cssNum(1,),
    },
  },),
  $({
    rule: '.empty',
    decls: {
      'padding-block': cssRem(RESULT_PADDING,),
      'padding-inline': cssRem(RESULT_PADDING,),
      color: cssVar('gutter-fg',),
    },
  },),
  $({
    rule: '::highlight(hl-search-match)',
    decls: {
      'background-color': cssVar('search-match-bg',),
    },
  },),
]
  .join('',);
