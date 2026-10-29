/**
 * Shadow DOM styles for the `<editor-pane>` web component.
 *
 * Core layout rules for the editor container, line divs, and line numbers.
 * Highlight and inlay styles are imported from dedicated modules.
 */

import {
  cssCh,
  cssCommaList,
  cssInt,
  cssLh,
  cssNum,
  cssPercent,
  cssRem,
  cssVar,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

import { HIGHLIGHT_STYLES, } from '../highlight/styles.ts';
import { INLAY_STYLES, } from '../inlay/styles.ts';

/**
 * Full viewport height as percentage.
 */
const FULL_HEIGHT = 100;

/**
 * Editor line height multiplier: 1.5 = (2 + 1) / 2.
 */
const LINE_HEIGHT = (2 + 1) / 2;

/**
 * Width of the line number gutter in `ch` units: 5 = 2 * 2 + 1.
 */
const GUTTER_WIDTH = (2 * 2) + 1;

/**
 * Total left padding (gutter + gap) in `ch` units: 6 = (2 + 1) * 2.
 */
const LINE_PADDING = (2 + 1) * 2;

/**
 * Shadow DOM styles for the editor pane.
 */
export const STYLES: string = [
  $({
    rule: ':host',
    decls: {
      display: 'block',
      'flex-grow': cssInt(1,),
      'flex-shrink': cssInt(1,),
      'flex-basis': cssInt(0,),
      overflow: 'auto',
    },
  },),
  $({
    rule: '.editor',
    decls: {
      'min-block-size': cssPercent(FULL_HEIGHT,),
      'padding-block': cssVar('editor-padding',),
      'padding-inline': cssVar('editor-padding',),
      'outline-style': 'none',
      'white-space': 'pre-wrap',
      'overflow-wrap': 'break-word',
      'font-family': cssCommaList([
        "'JetBrains Mono'",
        'monospace',
      ],),
      'font-size': cssRem(1,),
      'line-height': cssNum(LINE_HEIGHT,),
      'tab-size': cssInt(2,),
      color: cssVar('fg',),
      'caret-color': cssVar('fg',),
      'counter-reset': 'line',
    },
  },),
  $({
    rule: '.editor > div',
    decls: {
      'min-block-size': cssLh(1,),
      'counter-increment': 'line',
      position: 'relative',
      'padding-inline-start': cssCh(LINE_PADDING,),
    },
  },),
  $({
    rule: '.editor > div::after',
    decls: {
      content: 'counter(line)',
      position: 'absolute',
      'inset-inline-start': cssInt(0,),
      'inset-block-start': cssInt(0,),
      'inline-size': cssCh(GUTTER_WIDTH,),
      'text-align': 'end',
      color: cssVar('gutter-fg',),
      'user-select': 'none',
      'pointer-events': 'none',
    },
  },),
  HIGHLIGHT_STYLES,
  INLAY_STYLES,
]
  .join('',);
