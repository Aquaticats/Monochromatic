/**
 Shadow DOM styles for the `<top-nav>` web component.
 */
import {
  cssInt,
  cssRem,
  cssVar,
} from '@monochromatic-dev/hyperscript-core/ts';
import { $ as css, } from '../css.ts';
import { stickyBar, } from '../mixins-composed.ts';
import {
  appearanceNone,
  flexCenter,
  flexColumn,
  focusOutline,
  minTouchTarget,
} from '../mixins.ts';
import { TOP_NAV_ICON_STYLES, } from './top-nav-styles-icons.ts';
import { TOP_NAV_MEDIA_STYLES, } from './top-nav-styles-media.ts';

/**
 Heading font size in rem.
 */
const HEADING_FONT_SIZE = 1 + (1 / 2);

/**
 Normal font weight for heading.
 */
const FONT_WEIGHT_NORMAL = 400;

/**
 Focus outline offset in rem (-1/8).
 */
const FOCUS_OFFSET = -(1 / 2
  / 2
  / 2);

/**
 Gap between hamburger lines in rem (3/8).
 */
const HAMBURGER_GAP = ((1 / 2) / 2) + (((1 / 2) / 2) / 2);

/**
 Hamburger line width in rem (1 3/4).
 */
const LINE_WIDTH = 1 + (1 / 2)
  + ((1 / 2) / 2);

/**
 Hamburger line height in rem (1/4).
 */
const LINE_HEIGHT = 1 / 2
  / 2;

/**
 Compiled CSS string for `<top-nav>` Shadow DOM.
 */
export const TOP_NAV_STYLES: string = [
  css({
    rule: ':host',
    decls: {
      ...stickyBar(),
      'justify-content': 'center',
    },
  },),
  css({
    rule: 'h1',
    decls: {
      'flex-grow': 1,
      'text-align': 'center',
      'font-size': cssRem(HEADING_FONT_SIZE,),
      'font-weight': cssInt(FONT_WEIGHT_NORMAL,),
      'line-height': 'normal',
      'margin-block': 0,
      'margin-inline': 0,
    },
  },),
  css({
    rule: '.action',
    decls: {
      ...appearanceNone(),
      ...flexCenter(),
      ...minTouchTarget(),
      color: cssVar('fg',),
      'text-decoration': 'none',
    },
    children: [
      css({
        rule: '&:focus-visible',
        decls: focusOutline({ offset: cssRem(FOCUS_OFFSET,), },),
      },),
    ],
  },),
  css({
    rule: '.hamburger',
    decls: {
      'inline-size': cssRem(2,),
      'block-size': cssRem(2,),
      ...flexColumn(),
      'justify-content': 'center',
      'align-items': 'center',
      gap: cssRem(HAMBURGER_GAP,),
    },
  },),
  css({
    rule: '.line',
    decls: {
      'inline-size': cssRem(LINE_WIDTH,),
      'block-size': cssRem(LINE_HEIGHT,),
      'background-color': cssVar('fg',),
      display: 'block',
    },
  },),
  ...TOP_NAV_ICON_STYLES,
  TOP_NAV_MEDIA_STYLES,
]
  .join('',);
