/**
 Shared utility CSS classes used across multiple pages.
 */
import {
  cssCalc,
  cssPercent,
  cssRem,
  cssVar,
} from '@monochromatic-dev/hyperscript-core/ts';
import { $ as css, } from './css.ts';

/**
 Full percentage for divider inline-size.
 */
const FULL_PERCENT = 100;

/**
 Task list, divider, and empty-state utility styles.
 */
export const utilities: string = [
  css({
    rule: '.task-list',
    decls: {
      display: 'flex',
      'flex-direction': 'column',
      gap: cssVar('gap',),
      'list-style': 'none',
    },
  },),
  css({
    rule: '.divider',
    decls: {
      'block-size': cssCalc(`${cssRem(1,)} / 16`,),
      'background-color': cssVar('fg',),
      'inline-size': cssPercent(FULL_PERCENT,),
    },
  },),
  css({
    rule: '.empty',
    decls: { color: cssVar('medium',), },
  },),
]
  .join('',);
