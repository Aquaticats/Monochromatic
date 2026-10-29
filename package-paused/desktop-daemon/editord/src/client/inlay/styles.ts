/**
 * CSS rules for inlay hint annotations rendered via `::before`.
 *
 * Includes the hint styling and severity-colored variants.
 * Line number offset for annotated lines is also defined here.
 */

import {
  cssCommaList,
  cssNum,
  cssVar,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

/**
 * Editor line height multiplier: 1.5 = (2 + 1) / 2.
 */
const LINE_HEIGHT = (2 + 1) / 2;

/**
 * CSS rules for inlay hint annotations.
 */
export const INLAY_STYLES: string = [
  $({
    rule: '.editor > div[data-inlay]::after',
    decls: {
      'inset-block-start': cssVar('line-num-offset',),
    },
  },),
  $({
    rule: '.editor > div[data-inlay]::before',
    decls: {
      content: 'attr(data-inlay)',
      display: 'block',
      'inline-size': 'fit-content',
      color: cssVar('inlay-fg',),
      'font-family': cssCommaList([
        "'Inter'",
        'sans-serif',
      ],),
      'pointer-events': 'none',
      'user-select': 'none',
      'white-space': 'pre-wrap',
      'line-height': cssNum(LINE_HEIGHT,),
    },
  },),
  $({
    rule: '.editor > div[data-inlay-severity="error"]::before',
    decls: {
      color: cssVar('diag-error',),
    },
  },),
  $({
    rule: '.editor > div[data-inlay-severity="warning"]::before',
    decls: {
      color: cssVar('diag-warning',),
    },
  },),
  $({
    rule: '.editor > div[data-inlay-severity="info"]::before',
    decls: {
      color: cssVar('diag-info',),
    },
  },),
]
  .join('',);
