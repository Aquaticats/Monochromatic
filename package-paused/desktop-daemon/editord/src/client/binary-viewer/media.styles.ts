/**
 * Media element styles for the `<binary-viewer>` component.
 *
 * Layout rules for image, audio, and video elements.
 * Split from binary-viewer.styles.ts to stay under max-lines.
 */

import {
  cssNum,
  cssPercent,
  cssRem,
  cssVar,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';
import { HALF, } from '@monochromatic-dev/const-core';

/**
 * Full dimension as percentage.
 */
const FULL = 100;

/**
 * Repeat button resting opacity.
 */
const REPEAT_RESTING_OPACITY = 0.8;

/**
 * Minimum touch target for the repeat button in rem: 5/2.
 */
const REPEAT_MIN_SIZE = ((2 * 2) + 1) / 2;

/**
 * Media element and control styles.
 */
export const MEDIA_STYLES: string = [
  $({
    rule: 'img',
    decls: {
      'max-inline-size': cssPercent(FULL,),
      'max-block-size': cssPercent(FULL,),
      'object-fit': 'contain',
    },
  },),
  $({
    rule: 'video',
    decls: {
      'max-inline-size': cssPercent(FULL,),
      'max-block-size': cssPercent(FULL,),
    },
  },),
  $({
    rule: '.audio-controls',
    decls: {
      display: 'flex',
      'align-items': 'center',
      gap: cssRem(1,),
    },
  },),
  $({
    rule: '.repeat-btn',
    decls: {
      display: 'flex',
      'align-items': 'center',
      'justify-content': 'center',
      'min-inline-size': cssRem(REPEAT_MIN_SIZE,),
      'min-block-size': cssRem(REPEAT_MIN_SIZE,),
      'border-radius': cssPercent(HALF * FULL,),
      border: 'none',
      'background-color': cssVar('gutter-fg',),
      color: cssVar('bg',),
      cursor: 'pointer',
      opacity: cssNum(REPEAT_RESTING_OPACITY,),
      transition: 'opacity 0.15s',
    },
    children: [
      $({
        rule: '&:hover',
        decls: { opacity: cssNum(1,), },
      },),
      $({
        rule: '&[data-active]',
        decls: {
          opacity: cssNum(1,),
          'background-color': cssVar('fg',),
        },
      },),
    ],
  },),
]
  .join('',);
