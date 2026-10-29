/**
 Shadow DOM styles for the `<task-card>` web component.
 */
import {
  cssRem,
  cssVar,
} from '@monochromatic-dev/hyperscript-core/ts';
import { $ as css, } from '../css.ts';
import {
  appearanceNone,
  flexCenter,
  flexColumn,
  flexRow,
  focusOutline,
  scrollRow,
  whitespaceNowrap,
} from '../mixins.ts';

/**
 Checkbox box size in rem (1 3/4).
 */
const CHECKBOX_SIZE = 1 + (1 / 2)
  + ((1 / 2) / 2);

/**
 Checkbox border width in rem (1/4).
 */
const CHECKBOX_BORDER = 1 / 2
  / 2;

/**
 Title font size in rem (1 1/4).
 */
const TITLE_FONT_SIZE = 1 + ((1 / 2) / 2);

/**
 Chip gap in rem (1/4).
 */
const CHIP_GAP = 1 / 2
  / 2;

/**
 Compiled CSS string for `<task-card>` Shadow DOM.
 */
export const TASK_CARD_STYLES: string = [
  css({
    rule: ':host',
    decls: {
      ...flexColumn(),
      gap: cssVar('min-gap',),
      'background-color': cssVar('bg',),
      'overflow-x': 'hidden',
      'overflow-y': 'hidden',
      cursor: 'pointer',
    },
  },),
  css({
    rule: '.row',
    decls: {
      ...flexRow(),
      gap: cssVar('min-gap',),
      'align-items': 'flex-start',
    },
  },),
  css({
    rule: '.checkbox',
    decls: {
      ...appearanceNone(),
      ...flexCenter(),
      'inline-size': cssRem(2,),
      'block-size': cssRem(2,),
    },
    children: [
      css({
        rule: '&:focus-visible',
        decls: focusOutline(),
      },),
    ],
  },),
  css({
    rule: '.checkbox-box',
    decls: {
      'inline-size': cssRem(CHECKBOX_SIZE,),
      'block-size': cssRem(CHECKBOX_SIZE,),
      'border-width': cssRem(CHECKBOX_BORDER,),
      'border-style': 'solid',
      'border-color': cssVar('fg',),
    },
  },),
  css({
    rule: '.title',
    decls: {
      'font-size': cssRem(TITLE_FONT_SIZE,),
      'font-weight': 400,
      'line-height': 'normal',
      'flex-grow': 1,
      'min-inline-size': 0,
    },
  },),
  css({
    rule: '.chips',
    decls: scrollRow(),
  },),
  css({
    rule: '.chips::-webkit-scrollbar',
    decls: { display: 'none', },
  },),
  css({
    rule: '.chip',
    decls: {
      ...flexRow(),
      ...whitespaceNowrap(),
      gap: cssRem(CHIP_GAP,),
      'font-size': cssRem(1,),
      'line-height': 1.5,
    },
  },),
  css({
    rule: '.chip.blocked',
    decls: {
      'border-color': cssVar('red-fg',),
      color: cssVar('red-fg',),
    },
  },),
]
  .join('',);
