/**
 Pill, button, and interactive element styles for `<task-detail>`.
 */
import {
  cssCalc,
  cssRem,
  cssVar,
} from '@monochromatic-dev/hyperscript-core/ts';
import { $ as css, } from '../css.ts';
import { buttonOutlined, } from '../mixins-composed.ts';
import {
  borderRadiusFull,
  flexCenter,
  flexRow,
  focusOutline,
  scrollRow,
  shadowDomGlobals,
  whitespaceNowrap,
} from '../mixins.ts';

/**
 Pill padding in rem (1/2).
 */
const PILL_PADDING = 1 / 2;

/**
 Pill gap in rem (1/4).
 */
const PILL_GAP = 1 / 2
  / 2;

/**
 Button row gap in rem (1/2).
 */
const BTN_ROW_GAP = 1 / 2;

/**
 Pill, button row, and shadow DOM global styles for `<task-detail>`.
 */
export const TASK_DETAIL_INTERACTIVE_STYLES: string = [
  css({
    rule: '.pills',
    decls: {
      ...scrollRow(),
      'flex-wrap': 'wrap',
    },
  },),
  css({
    rule: '.pill',
    decls: {
      ...flexCenter(),
      ...whitespaceNowrap(),
      'border-width': cssCalc(`${cssRem(1,)} / 16`,),
      'border-style': 'solid',
      'border-color': cssVar('fg',),
      ...borderRadiusFull(),
      'padding-block': cssRem(PILL_PADDING,),
      'padding-inline': cssRem(PILL_PADDING,),
      gap: cssRem(PILL_GAP,),
      'font-size': cssRem(1,),
      'line-height': 1.5,
    },
    children: [
      css({
        rule: '&[data-autofilled]',
        decls: {
          'border-color': cssVar('red-fg',),
          color: cssVar('red-fg',),
        },
      },),
      css({
        rule: '&[data-loading]',
        decls: { opacity: 0.5, },
      },),
    ],
  },),
  css({
    rule: '.btn-row',
    decls: {
      ...flexRow(),
      gap: cssRem(BTN_ROW_GAP,),
      'flex-wrap': 'wrap',
      'margin-block-start': cssRem(1,),
    },
    children: [
      css({
        rule: '&[data-hidden]',
        decls: { display: 'none', },
      },),
    ],
  },),
  css({
    rule: '.btn-outline',
    decls: buttonOutlined(),
    children: [
      css({
        rule: '&:focus-visible',
        decls: focusOutline(),
      },),
    ],
  },),
  css({
    rule: '.btn-primary',
    decls: {
      ...buttonOutlined(),
      'background-color': cssVar('fg',),
      color: cssVar('bg',),
    },
    children: [
      css({
        rule: '&:focus-visible',
        decls: focusOutline(),
      },),
    ],
  },),
  ...shadowDomGlobals(),
]
  .join('',);
