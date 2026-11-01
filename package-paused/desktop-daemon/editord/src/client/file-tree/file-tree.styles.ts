/**
 * Shadow DOM styles for the `<file-tree>` web component.
 *
 * Composes host layout and entry-level styles
 * into a single stylesheet string for the shadow root.
 */

import {
  cssCommaList,
  cssNum,
  cssPercent,
  cssRem,
  cssVar,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

import { ENTRY_STYLES, } from './entry.styles.ts';

/**
 * Sidebar width in `rem`: 16 = 2 * 2 * 2 * 2.
 */
const SIDEBAR_WIDTH = 2 * 2
  * 2
  * 2;

/**
 * Shadow DOM styles for the file tree.
 */
export const STYLES: string = [
  $({
    rule: ':host',
    decls: {
      display: 'block',
      'inline-size': cssRem(SIDEBAR_WIDTH,),
      'block-size': cssPercent(100,),
      'flex-shrink': cssNum(0,),
      overflow: 'auto',
      resize: 'inline',
      'font-family': cssCommaList([
        "'JetBrains Mono'",
        'monospace',
      ],),
      'font-size': cssRem(1,),
      'line-height': cssNum((2 + 1) / 2,),
      color: cssVar('fg',),
      'user-select': 'none',
    },
  },),
  $({
    rule: '.tree, .children',
    decls: {
      display: 'flex',
      'flex-direction': 'column',
    },
  },),
  $({
    rule: 'details',
    decls: {
      'padding-inline-start': cssRem(1,),
    },
  },),
  ENTRY_STYLES,
]
  .join('',);
