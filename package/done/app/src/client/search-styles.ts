/**
 Page-scoped styles for the Search page.
 */
import {
  cssCalc,
  cssRem,
  cssVar,
} from '@monochromatic-dev/hyperscript-core/ts';
import { $ as css, } from './css.ts';
import {
  borderRadiusFull,
  flexCenter,
  whitespaceNowrap,
} from './mixins.ts';

/**
 Tag chip padding in rem (1/2).
 */
const CHIP_PADDING = 1 / 2;

/**
 Tag chip gap in rem (1/4).
 */
const CHIP_GAP = 1 / 2
  / 2;

/**
 Desktop search hint font size in rem (3/2).
 */
const HINT_FONT_SIZE_DESKTOP = (2 + 1) / 2;

/**
 Compiled CSS string for search page styling.
 */
export const searchStyles: string = [
  css({
    rule: '.search-hint',
    decls: {
      color: cssVar('fg-weaker',),
      'font-size': cssRem(1,),
      'line-height': 1.5,
    },
  },),
  css({
    rule: '.tag-chips',
    decls: {
      display: 'flex',
      'flex-wrap': 'wrap',
      gap: cssVar('min-gap',),
    },
  },),
  css({
    rule: '.tag-chip',
    decls: {
      ...flexCenter(),
      ...whitespaceNowrap(),
      ...borderRadiusFull(),
      'border-width': cssCalc(`${cssRem(1,)} / 16`,),
      'border-style': 'solid',
      'border-color': cssVar('fg',),
      'padding-block': cssRem(CHIP_PADDING,),
      'padding-inline': cssRem(CHIP_PADDING,),
      gap: cssRem(CHIP_GAP,),
      cursor: 'pointer',
      'background-color': 'transparent',
      'font-family': 'inherit',
      'font-size': 'inherit',
      'font-style': 'inherit',
      'font-weight': 'inherit',
      'line-height': 'inherit',
    },
    children: [
      css({
        rule: '&:hover',
        decls: { 'background-color': cssVar('hover-bg',), },
      },),
    ],
  },),
  css({
    at: 'media',
    params: '(min-width: 48rem)',
    children: [
      css({
        rule: '.search-hint',
        decls: { 'font-size': cssRem(HINT_FONT_SIZE_DESKTOP,), },
      },),
    ],
  },),
]
  .join('',);
