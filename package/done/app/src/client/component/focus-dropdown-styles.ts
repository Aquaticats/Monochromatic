/**
 Shadow DOM styles for the `<focus-dropdown>` web component.
 */
import {
  cssCalc,
  cssInt,
  cssPercent,
  cssRem,
  cssVar,
} from '@monochromatic-dev/hyperscript-core/ts';
import { $ as css, } from '../css.ts';
import { buttonOutlined, } from '../mixins-composed.ts';
import { focusOutline, } from '../mixins.ts';

/**
 Full percentage for width declarations.
 */
const FULL_PERCENT = 100;

/**
 Menu vertical padding in rem.
 */
const MENU_PADDING = 1 / 2
  / 2;

/**
 Option padding in rem.
 */
const OPTION_PADDING = 1 / 2;

/**
 Compiled CSS string for `<focus-dropdown>` Shadow DOM.
 */
export const FOCUS_DROPDOWN_STYLES: string = [
  css({
    rule: ':host',
    decls: {
      display: 'block',
      'inline-size': cssPercent(FULL_PERCENT,),
      position: 'relative',
    },
  },),
  css({
    rule: '.trigger',
    decls: {
      ...buttonOutlined(),
      'inline-size': cssPercent(FULL_PERCENT,),
      'text-align': 'start',
    },
    children: [
      css({
        rule: '&:focus-visible',
        decls: focusOutline(),
      },),
    ],
  },),
  css({
    rule: '.text',
    decls: {
      'flex-grow': 1,
      'text-align': 'start',
    },
  },),
  css({
    rule: '.divider',
    decls: {
      'inline-size': cssCalc(`${cssRem(1,)} / 16`,),
      'block-size': cssPercent(FULL_PERCENT,),
      'background-color': cssVar('fg-weaker',),
    },
  },),
  css({
    rule: '.menu',
    decls: {
      position: 'absolute',
      'inset-block-start': cssPercent(FULL_PERCENT,),
      'inset-inline-start': 0,
      'inline-size': cssPercent(FULL_PERCENT,),
      'border-width': cssCalc(`${cssRem(1,)} / 16`,),
      'border-style': 'solid',
      'border-color': cssVar('fg',),
      'background-color': cssVar('bg',),
      'padding-block': cssRem(MENU_PADDING,),
      'padding-inline': 0,
      'margin-block': 0,
      'margin-inline': 0,
      'list-style': 'none',
      'z-index': cssInt(10,),
    },
    children: [
      css({
        rule: '&:not(:popover-open)',
        decls: { display: 'none', },
      },),
    ],
  },),
  css({
    rule: '.option',
    decls: {
      'padding-block': cssRem(OPTION_PADDING,),
      'padding-inline': cssRem(OPTION_PADDING,),
      cursor: 'pointer',
    },
    children: [
      css({
        rule: '&:hover',
        decls: { 'background-color': cssVar('hover-bg',), },
      },),
    ],
  },),
]
  .join('',);
