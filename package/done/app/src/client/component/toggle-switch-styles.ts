/**
 Shadow DOM styles for the `<toggle-switch>` web component.
 */
import {
  cssCalc,
  cssCommaList,
  cssPercent,
  cssRem,
  cssS,
  cssTranslateY,
  cssVar,
} from '@monochromatic-dev/hyperscript-core/ts';
import { $ as css, } from '../css.ts';
import {
  borderRadiusFull,
  flexCenter,
} from '../mixins.ts';

/**
 Track inline-size in rem.
 */
const TRACK_WIDTH = 3;

/**
 Full percentage for track sizing.
 */
const FULL_PERCENT = 100;

/**
 Half percentage for centering the thumb.
 */
const HALF_PERCENT = 50;

/**
 Negative half percentage for translateY centering.
 */
const NEG_HALF_PERCENT = -50;

/**
 Transition duration for thumb slide in seconds.
 */
const THUMB_TRANSITION_S = 0.15;

/**
 Compiled CSS string for `<toggle-switch>` Shadow DOM.
 */
export const TOGGLE_SWITCH_STYLES: string = [
  css({
    rule: ':host',
    decls: {
      display: 'inline-flex',
      cursor: 'pointer',
      'inline-size': cssRem(TRACK_WIDTH,),
      'block-size': cssRem(2,),
    },
  },),
  css({
    rule: '.track',
    decls: {
      'inline-size': cssPercent(FULL_PERCENT,),
      'block-size': cssPercent(FULL_PERCENT,),
      'border-width': cssCalc(`${cssRem(1,)} / 16`,),
      'border-style': 'solid',
      'border-color': cssVar('fg',),
      ...borderRadiusFull(),
      'background-color': cssVar('bg',),
      position: 'relative',
      'overflow-x': 'hidden',
      'overflow-y': 'hidden',
    },
  },),
  css({
    rule: '.thumb',
    decls: {
      position: 'absolute',
      'inset-block-start': cssPercent(HALF_PERCENT,),
      transform: cssTranslateY(cssPercent(NEG_HALF_PERCENT,),),
      'inline-size': cssRem(2,),
      'block-size': cssRem(2,),
      ...borderRadiusFull(),
      'border-width': cssCalc(`${cssRem(1,)} / 16`,),
      'border-style': 'solid',
      'border-color': cssVar('fg',),
      'background-color': cssVar('bg-stronger',),
      ...flexCenter(),
      'font-size': cssRem(1,),
      'transition-property': cssCommaList([
        'inset-inline-start',
        'inset-inline-end',
      ],),
      'transition-duration': cssS(THUMB_TRANSITION_S,),
    },
  },),
  css({
    rule: '.thumb.on',
    decls: {
      'inset-inline-end': cssCalc(`${cssRem(-1,)} / 16`,),
      'inset-inline-start': 'auto',
    },
  },),
  css({
    rule: '.thumb.off',
    decls: {
      'inset-inline-start': cssCalc(`${cssRem(-1,)} / 16`,),
      'inset-inline-end': 'auto',
    },
  },),
]
  .join('',);
