/**
 Shared navigation styles for the `<side-drawer>` web component.
 
 Covers the divider, nav container, anchor links, header bar,
 and close button used by both the inline sidebar and the popover panel.
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

/**
 Full percentage for inline-size.
 */
const FULL_PERCENT = 100;

/**
 Minimum block-size for anchor link rows in rem.
 */
const LINK_HEIGHT = 3;

/**
 Font size for navigation links in rem.
 */
const NAV_FONT_SIZE = 1 + ((1 / 2) / 2);

/**
 Font weight for navigation text.
 */
const FONT_WEIGHT_NORMAL = 400;

/**
 Focus outline offset in rem.
 */
const FOCUS_OFFSET = -(1 / 2
  / 2
  / 2);

/**
 Minimum block-size for the header bar in rem.
 */
const HEADER_HEIGHT = 4;

/**
 Compiled CSS rules for shared `<side-drawer>` navigation elements.
 */
export const SIDE_DRAWER_NAV_STYLES: string[] = [
  css({
    rule: '.divider',
    decls: {
      'block-size': cssCalc(`${cssRem(1,)} / 16`,),
      'background-color': cssVar('bg-weaker',),
      'inline-size': cssPercent(FULL_PERCENT,),
    },
  },),
  css({
    rule: 'nav',
    decls: {
      ...flexColumn(),
      gap: cssVar('min-gap',),
      'flex-grow': 1,
      'padding-block-start': cssVar('min-gap',),
    },
  },),
  css({
    rule: 'a',
    decls: {
      ...flexRow(),
      gap: cssVar('min-gap',),
      'min-block-size': cssRem(LINK_HEIGHT,),
      'padding-block': 0,
      'padding-inline': cssVar('min-gap',),
      color: cssVar('fg',),
      'text-decoration': 'none',
      'font-size': cssRem(NAV_FONT_SIZE,),
      'font-weight': cssInt(FONT_WEIGHT_NORMAL,),
    },
    children: [
      css({
        rule: '&:hover',
        decls: { 'background-color': cssVar('hover-bg',), },
      },),
      css({
        rule: '&:focus-visible',
        decls: focusOutline({ offset: cssRem(FOCUS_OFFSET,), },),
      },),
    ],
  },),
  css({
    rule: '.header',
    decls: {
      ...flexRow(),
      'justify-content': 'space-between',
      'padding-block-start': cssVar('min-gap',),
      'padding-block-end': cssVar('min-padding',),
      'padding-inline-start': cssVar('min-gap',),
      'padding-inline-end': cssVar('min-padding',),
      'min-block-size': cssRem(HEADER_HEIGHT,),
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
        },
      },),
    ],
  },),
];
