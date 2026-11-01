/**
 Shadow DOM styles for the `<task-detail>` web component.
 */
import {
  cssCalc,
  cssInt,
  cssPercent,
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
  minTouchTarget,
} from '../mixins.ts';
import { TASK_DETAIL_INTERACTIVE_STYLES, } from './task-detail-styles-interactive.ts';

/**
 Focus outline offset in rem (-1/8).
 */
const FOCUS_OFFSET = -(1 / 2
  / 2
  / 2);

/**
 SVG stroke width for close icon.
 */
const STROKE_WIDTH = 4;

/**
 Heading and title-input font size in rem.
 */
const HEADING_FONT_SIZE = 1 + (1 / 2);

/**
 Normal font weight for headings and inputs.
 */
const FONT_WEIGHT_NORMAL = 400;

/**
 Full percentage for input inline-size.
 */
const FULL_PERCENT = 100;

/**
 Padding for title-input and description blocks in rem (1/4).
 */
const SMALL_PADDING = 1 / 2
  / 2;

/**
 Padding for description input in rem (1/2).
 */
const DESC_PADDING = 1 / 2;

/**
 Minimum block-size for description textarea in rem.
 */
const TEXTAREA_MIN_HEIGHT = 4.5;

/**
 Compiled CSS string for `<task-detail>` Shadow DOM.
 */
export const TASK_DETAIL_STYLES: string = [
  css({
    rule: ':host',
    decls: {
      ...flexColumn(),
      gap: cssRem(1,),
      'padding-block': cssRem(1,),
      'padding-inline': cssRem(1,),
    },
  },),
  css({
    rule: '.header',
    decls: {
      ...flexRow(),
      'justify-content': 'space-between',
    },
  },),
  css({
    rule: '.close',
    decls: {
      ...appearanceNone(),
      ...flexCenter(),
      ...minTouchTarget(),
    },
    children: [
      css({
        rule: '&:focus-visible',
        decls: focusOutline({ offset: cssRem(FOCUS_OFFSET,), },),
      },),
      css({
        rule: '& svg',
        decls: {
          'inline-size': cssRem(2,),
          'block-size': cssRem(2,),
          stroke: cssVar('fg',),
          'stroke-width': cssInt(STROKE_WIDTH,),
        },
      },),
    ],
  },),
  css({
    rule: '.heading',
    decls: {
      'font-size': cssRem(HEADING_FONT_SIZE,),
      'font-weight': cssInt(FONT_WEIGHT_NORMAL,),
    },
  },),
  css({
    rule: '.title-input',
    decls: {
      'font-size': cssRem(HEADING_FONT_SIZE,),
      'font-weight': cssInt(FONT_WEIGHT_NORMAL,),
      'border-style': 'none',
      'border-block-end-width': cssCalc(`${cssRem(1,)} / 16`,),
      'border-block-end-style': 'solid',
      'border-block-end-color': cssVar('fg',),
      'background-color': 'transparent',
      'inline-size': cssPercent(FULL_PERCENT,),
      'padding-block': cssRem(SMALL_PADDING,),
      'padding-inline': 0,
      'outline-style': 'none',
      'font-family': 'inherit',
      color: cssVar('fg',),
    },
  },),
  css({
    rule: '.desc-input',
    decls: {
      'border-width': cssCalc(`${cssRem(1,)} / 16`,),
      'border-style': 'solid',
      'border-color': cssVar('fg',),
      'padding-block': cssRem(DESC_PADDING,),
      'padding-inline': cssRem(DESC_PADDING,),
      'min-block-size': cssRem(TEXTAREA_MIN_HEIGHT,),
      resize: 'vertical',
      'font-family': 'inherit',
      'font-size': 'inherit',
      'font-style': 'inherit',
      'font-weight': 'inherit',
      'line-height': 'inherit',
      color: cssVar('fg',),
      'background-color': 'transparent',
    },
  },),
  css({
    rule: '.actions',
    decls: {
      display: 'flex',
      gap: cssRem(1,),
    },
  },),
  TASK_DETAIL_INTERACTIVE_STYLES,
]
  .join('',);
