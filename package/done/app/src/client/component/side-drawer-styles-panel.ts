/**
 Popover panel styles for the `<side-drawer>` web component.
 
 Covers the full-screen popover overlay, slide-in animation,
 scrim fade-in, and the drawer panel itself.
 */
import {
  cssCubicBezier,
  cssInt,
  cssNum,
  cssPercent,
  cssRem,
  cssS,
  cssTranslateX,
  cssVar,
  cssVi,
} from '@monochromatic-dev/hyperscript-core/ts';
import { $ as css, } from '../css.ts';
import {
  flexColumn,
  shadowDomGlobals,
} from '../mixins.ts';

/**
 Full percentage value for sizing declarations.
 */
const FULL_PERCENT = 100;

/**
 Negative full percentage for offscreen slide-out positioning.
 */
const NEG_FULL_PERCENT = -100;

/**
 Z-index for the popover panel overlay.
 */
const Z_INDEX_PANEL = 100;

/**
 Scrim fade-in animation duration in seconds.
 */
const SCRIM_FADE_S = 0.2;

/**
 Drawer slide-in animation duration in seconds.
 */
const DRAWER_SLIDE_S = 1 / 2
  / 2;

/**
 Cubic-bezier x-coordinate for the decelerate easing control point.
 */
const EASE_CONTROL_X = 0.2;

/**
 Fixed inline-size of the drawer panel in rem.
 */
const DRAWER_WIDTH = 20;

/**
 Maximum inline-size of the drawer panel in vi units.
 */
const DRAWER_MAX_VI = 85;

/**
 Popover panel styles and animations for `<side-drawer>`.
 */
export const SIDE_DRAWER_PANEL_STYLES: string = [
  css({
    rule: '.panel',
    decls: {
      position: 'fixed',
      'inset-block': 0,
      'inset-inline': 0,
      'margin-block': 0,
      'margin-inline': 0,
      'padding-block': 0,
      'padding-inline': 0,
      'border-style': 'none',
      'inline-size': cssPercent(FULL_PERCENT,),
      'max-inline-size': cssPercent(FULL_PERCENT,),
      'block-size': cssPercent(FULL_PERCENT,),
      'max-block-size': cssPercent(FULL_PERCENT,),
      'z-index': cssInt(Z_INDEX_PANEL,),
      display: 'flex',
      'background-color': 'transparent',
      'overflow-x': 'visible',
      'overflow-y': 'visible',
    },
  },),
  css({
    rule: '.panel:not(:popover-open)',
    decls: { display: 'none', },
  },),
  css({
    at: 'keyframes',
    params: 'drawer-slide-in',
    children: [
      css({
        rule: 'from',
        decls: {
          transform: cssTranslateX(cssPercent(NEG_FULL_PERCENT,),),
          opacity: 0,
        },
      },),
      css({
        rule: 'to',
        decls: {
          transform: cssTranslateX(cssNum(0,),),
          opacity: 1,
        },
      },),
    ],
  },),
  css({
    at: 'keyframes',
    params: 'scrim-fade-in',
    children: [
      css({
        rule: 'from',
        decls: { 'background-color': 'transparent', },
      },),
      css({
        rule: 'to',
        decls: { 'background-color': cssVar('overlay-bg',), },
      },),
    ],
  },),
  css({
    rule: '.panel:popover-open',
    decls: {
      'animation-name': 'scrim-fade-in',
      'animation-duration': cssS(SCRIM_FADE_S,),
      'animation-timing-function': 'ease-out',
      'animation-fill-mode': 'both',
    },
  },),
  css({
    rule: '.panel:popover-open > .panel-drawer',
    decls: {
      'animation-name': 'drawer-slide-in',
      'animation-duration': cssS(DRAWER_SLIDE_S,),
      'animation-timing-function': cssCubicBezier([
        0,
        0,
        EASE_CONTROL_X,
        1,
      ],),
      'animation-fill-mode': 'both',
    },
  },),
  css({
    rule: '.panel-drawer',
    decls: {
      'background-color': cssVar('bg',),
      'inline-size': cssRem(DRAWER_WIDTH,),
      'max-inline-size': cssVi(DRAWER_MAX_VI,),
      'block-size': cssPercent(FULL_PERCENT,),
      ...flexColumn(),
    },
  },),
  ...shadowDomGlobals(),
]
  .join('',);
