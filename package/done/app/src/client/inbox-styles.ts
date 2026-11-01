/**
 Page-scoped CSS for the Inbox page controls.
 
 Separated from inbox.ts to keep the entry script focused on
 hydration logic rather than style declarations.
 */
import {
  cssCalc,
  cssPercent,
  cssRem,
  cssVar,
} from '@monochromatic-dev/hyperscript-core/ts';
import { $ as css, } from './css.ts';

/**
 Indentation for task children in rem.
 */
const CHILDREN_INDENT = 1 + (1 / 2);

/**
 Border width for task children separator in rem (1/8).
 */
const CHILDREN_BORDER = 1 / 2
  / 2
  / 2;

/**
 Padding for task children in rem (3/4).
 */
const CHILDREN_PADDING = (1 / 2) + ((1 / 2) / 2);

/**
 Full percentage for control group min-inline-size.
 */
const FULL_PERCENT = 100;

/**
 Font size for subsection headings in rem.
 */
const SUBSECTION_FONT_SIZE = 1 + ((1 / 2) / 2);

/**
 Font size numerator for subsection description (15/16 rem).
 */
const DESC_FONT_SIZE_PX = 15;

/**
 Minimum block-size for location options row in rem.
 */
const LOCATION_MIN_HEIGHT = 3;

/**
 Inbox-specific styles for task children, controls, and location options.
 */
export const inboxStyles: string = [
  css({
    rule: '.task-children',
    decls: {
      'margin-inline-start': cssRem(CHILDREN_INDENT,),
      'border-inline-start-width': cssRem(CHILDREN_BORDER,),
      'border-inline-start-style': 'solid',
      'border-inline-start-color': cssVar('bg-weaker',),
      'padding-inline-start': cssRem(CHILDREN_PADDING,),
    },
  },),
  css({
    rule: '.controls',
    decls: {
      display: 'flex',
      'flex-wrap': 'wrap',
      gap: cssVar('gap',),
      'align-items': 'flex-start',
    },
  },),
  css({
    rule: '.control-group',
    decls: {
      display: 'flex',
      'flex-direction': 'column',
      gap: cssVar('min-padding',),
      'flex-grow': 1,
      'flex-shrink': 0,
      'flex-basis': 0,
      'min-inline-size': cssPercent(FULL_PERCENT,),
      'overflow-x': 'hidden',
      'overflow-y': 'hidden',
    },
  },),
  css({
    rule: '.subsection-heading',
    decls: {
      'font-size': cssRem(SUBSECTION_FONT_SIZE,),
      'font-weight': 400,
    },
  },),
  css({
    rule: '.subsection-desc',
    decls: {
      'font-size': cssCalc(`${cssRem(DESC_FONT_SIZE_PX,)} / 16`,),
      'line-height': 1.5,
      color: cssVar('fg-weaker',),
    },
  },),
  css({
    rule: '.location-options',
    decls: {
      display: 'flex',
      gap: cssVar('min-gap',),
      'align-items': 'center',
      'min-block-size': cssRem(LOCATION_MIN_HEIGHT,),
      'flex-wrap': 'wrap',
    },
  },),
  css({
    rule: '.autodetect-toggle',
    decls: {
      display: 'flex',
      gap: cssVar('min-padding',),
      'align-items': 'center',
      cursor: 'pointer',
      'background-color': 'transparent',
      'border-style': 'none',
      'font-family': 'inherit',
      'font-size': 'inherit',
      'font-style': 'inherit',
      'font-weight': 'inherit',
      'line-height': 'inherit',
      color: cssVar('fg',),
      'padding-block': 0,
      'padding-inline': 0,
    },
  },),
]
  .join('',);
