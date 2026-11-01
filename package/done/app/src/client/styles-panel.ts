/**
 New-task panel keyframe animation and positioning styles.
 */
import {
  cssCalc,
  cssCompounded,
  cssCubicBezier,
  cssDvb,
  cssOklch,
  cssPercent,
  cssRem,
  cssS,
  cssScale,
  cssVar,
} from '@monochromatic-dev/hyperscript-core/ts';
import { $ as css, } from './css.ts';

/**
 Initial scale for the FAB-to-surface animation.
 */
const FAB_SCALE_FROM = 0.15;

/**
 Percentage for circular border-radius in the "from" keyframe.
 */
const HALF_PERCENT = 50;

/**
 Opacity at the start of the FAB-to-surface animation.
 */
const FAB_OPACITY_FROM = 0.6;

/**
 Panel inline-size expressed as a pixel-to-rem ratio.
 */
const PANEL_WIDTH_PX = 393;

/**
 Maximum block-size of the panel in dvb units.
 */
const PANEL_MAX_DVB = 80;

/**
 Panel slide-in animation duration in seconds.
 */
const PANEL_SLIDE_S = 1 / 2
  / 2;

/**
 First cubic-bezier x control point for the panel easing.
 */
const EASE_X1 = 0.4;

/**
 Second cubic-bezier x control point for the panel easing.
 */
const EASE_X2 = 0.2;

/**
 Shadow alpha channel for oklch shadow color.
 */
const SHADOW_ALPHA = 0.2;

/**
 Keyframe animation and fixed-position styles for the new-task slide-up panel.
 */
export const newTaskPanel: string = [
  css({
    at: 'keyframes',
    params: 'fab-to-surface',
    children: [
      css({
        rule: 'from',
        decls: {
          transform: cssScale(FAB_SCALE_FROM,),
          'border-radius': cssPercent(HALF_PERCENT,),
          opacity: FAB_OPACITY_FROM,
        },
      },),
      css({
        rule: 'to',
        decls: {
          transform: cssScale(1,),
          'border-start-start-radius': cssRem(1,),
          'border-start-end-radius': cssRem(1,),
          'border-end-start-radius': 0,
          'border-end-end-radius': 0,
          opacity: 1,
        },
      },),
    ],
  },),
  css({
    rule: '.new-task-panel',
    decls: {
      position: 'fixed',
      'inset-block-start': 'auto',
      'inset-block-end': 0,
      'inset-inline-start': 'auto',
      'inset-inline-end': cssRem(1,),
      'margin-block': 0,
      'margin-inline': 0,
      'border-style': 'none',
      'padding-block': 0,
      'padding-inline': 0,
      'background-color': cssVar('bg',),
      color: cssVar('fg',),
      'inline-size': cssCalc(`${cssRem(PANEL_WIDTH_PX,)} / 16`,),
      'max-block-size': cssDvb(PANEL_MAX_DVB,),
      'overflow-y': 'auto',
      'border-start-start-radius': cssRem(1,),
      'border-start-end-radius': cssRem(1,),
      'border-end-start-radius': 0,
      'border-end-end-radius': 0,
      'box-shadow': cssCompounded([
        0,
        cssRem(-(1 / 2
          / 2),),
        cssRem(1,),
        cssOklch({
          l: 0,
          c: 0,
          h: 0,
          a: SHADOW_ALPHA,
        },),
      ],),
      'transform-origin': cssCompounded([
        'bottom',
        'right',
      ],),
    },
    children: [
      css({
        rule: '&[data-animating]',
        decls: {
          'animation-name': 'fab-to-surface',
          'animation-duration': cssS(PANEL_SLIDE_S,),
          'animation-timing-function': cssCubicBezier([
            EASE_X1,
            0,
            EASE_X2,
            1,
          ],),
          'animation-fill-mode': 'both',
        },
      },),
    ],
  },),
]
  .join('',);
