/**
 Shadow DOM styles for the `<search-bar>` web component.
 */
import {
  cssCalc,
  cssPercent,
  cssRem,
  cssVar,
} from '@monochromatic-dev/hyperscript-core/ts';
import { $ as css, } from '../css.ts';
import { stickyBar, } from '../mixins-composed.ts';
import {
  appearanceNone,
  flexCenter,
  focusOutline,
  minTouchTarget,
  shadowDomGlobals,
} from '../mixins.ts';

/**
 Back button and heading font size in rem.
 */
const BACK_FONT_SIZE = 1 + (1 / 2);

/**
 Focus outline offset in rem (-1/8).
 */
const FOCUS_OFFSET = -(1 / 2
  / 2
  / 2);

/**
 Full percentage for input block-size.
 */
const FULL_PERCENT = 100;

/**
 Compiled CSS string for `<search-bar>` Shadow DOM.
 */
export const SEARCH_BAR_STYLES: string = [
  css({
    rule: ':host',
    decls: stickyBar(),
  },),
  css({
    rule: '.back',
    decls: {
      ...appearanceNone(),
      ...flexCenter(),
      ...minTouchTarget(),
      'font-size': cssRem(BACK_FONT_SIZE,),
      color: cssVar('fg',),
    },
    children: [
      css({
        rule: '&:focus-visible',
        decls: focusOutline({ offset: cssRem(FOCUS_OFFSET,), },),
      },),
    ],
  },),
  css({
    rule: 'input',
    decls: {
      'flex-grow': 1,
      'border-style': 'none',
      'background-color': 'transparent',
      'font-size': cssRem(1,),
      'font-family': 'inherit',
      color: cssVar('fg',),
      'outline-style': 'none',
      'block-size': cssPercent(FULL_PERCENT,),
    },
  },),
  ...shadowDomGlobals(),
  css({
    at: 'media',
    params: '(min-width: 48rem)',
    children: [
      css({
        rule: ':host',
        decls: {
          'border-block-end-width': cssCalc(`${cssRem(1,)} / 16`,),
          'border-block-end-style': 'solid',
          'border-block-end-color': cssVar('bg-weaker',),
        },
      },),
      css({
        rule: '.back',
        decls: { display: 'none', },
      },),
      css({
        rule: 'input',
        decls: { 'font-size': cssRem(BACK_FONT_SIZE,), },
      },),
    ],
  },),
]
  .join('',);
