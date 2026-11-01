/**
 Search icon styles for the `<top-nav>` web component.
 
 Separated from the main top-nav styles to keep each file under the line limit.
 */
import {
  cssCalc,
  cssCompounded,
  cssRem,
  cssRotate,
  cssTurn,
  cssVar,
} from '@monochromatic-dev/hyperscript-core/ts';
import { $ as css, } from '../css.ts';
import { borderRadiusFull, } from '../mixins.ts';

/**
 Circle element size in rem (1 + 3/8).
 */
const CIRCLE_SIZE = 1 + ((1 / 2) / 2)
  + (((1 / 2) / 2) / 2);

/**
 Border width shared by circle and handle in rem.
 */
const BORDER = 1 / 2
  / 2;

/**
 Pixel offset for handle positioning (19/16 rem).
 */
const HANDLE_OFFSET_PX = 19;

/**
 Handle bar height in rem (7/8).
 */
const HANDLE_HEIGHT = 1 - (((1 / 2) / 2) / 2);

/**
 Rotation amount in turns for the handle angle (-1/8 turn).
 */
const HANDLE_TURN = -(1 / 2
  / 2
  / 2);

/**
 CSS rules for the search icon `.search-icon`, `.circle`, and `.handle` elements.
 
 @example
 ```ts
 TOP_NAV_ICON_STYLES.forEach(rule => sheet.insertRule(rule));
 ```
 */
export const TOP_NAV_ICON_STYLES: string[] = [
  css({
    rule: '.search-icon',
    decls: {
      'inline-size': cssRem(2,),
      'block-size': cssRem(2,),
      position: 'relative',
    },
  },),
  css({
    rule: '.circle',
    decls: {
      position: 'absolute',
      'inset-block-start': 0,
      'inset-inline-start': 0,
      'inline-size': cssRem(CIRCLE_SIZE,),
      'block-size': cssRem(CIRCLE_SIZE,),
      'border-width': cssRem(BORDER,),
      'border-style': 'solid',
      'border-color': cssVar('fg',),
      ...borderRadiusFull(),
    },
  },),
  css({
    rule: '.handle',
    decls: {
      position: 'absolute',
      'inset-block-start': cssCalc(`${cssRem(HANDLE_OFFSET_PX,)} / 16`,),
      'inset-inline-start': cssCalc(`${cssRem(HANDLE_OFFSET_PX,)} / 16`,),
      'inline-size': cssRem(BORDER,),
      'block-size': cssRem(HANDLE_HEIGHT,),
      'background-color': cssVar('fg',),
      transform: cssRotate(cssTurn(HANDLE_TURN,),),
      'transform-origin': cssCompounded([
        'top',
        'left',
      ],),
    },
  },),
];
