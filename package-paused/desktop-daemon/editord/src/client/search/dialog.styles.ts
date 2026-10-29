/**
 * Dialog and input styles for the search overlay.
 *
 * Covers the `<dialog>`, backdrop, and search input styling.
 * Result-related styles are in the main styles file.
 */

import {
  cssCommaList,
  cssDvb,
  cssOklch,
  cssPercent,
  cssRem,
  cssVar,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

/**
 * Overlay width as percentage of viewport inline size: 60%.
 */
const OVERLAY_WIDTH = 60;

/**
 * Maximum overlay height in dynamic viewport block units: 70.
 */
const OVERLAY_MAX_HEIGHT = 70;

/**
 * Input padding in `rem`: 0.75 = (2 + 1) / (2 * 2).
 */
const INPUT_PADDING = (2 + 1) / (2 * 2);

/**
 * Backdrop opacity: 0.5 = 1 / 2.
 */
const BACKDROP_ALPHA = 1 / 2;

/**
 * Dialog and input styles for the search overlay.
 */
export const DIALOG_STYLES: string = [
  $({
    rule: 'dialog[open]',
    decls: {
      'inline-size': cssPercent(OVERLAY_WIDTH,),
      'max-block-size': cssDvb(OVERLAY_MAX_HEIGHT,),
      'border-block-style': 'none',
      'border-inline-style': 'none',
      'border-radius': cssRem(1 / 2,),
      'padding-block': cssRem(0,),
      'padding-inline': cssRem(0,),
      'background-color': cssVar('bg',),
      color: cssVar('fg',),
      'font-family': cssCommaList([
        "'JetBrains Mono'",
        'monospace',
      ],),
      'font-size': cssRem(1,),
      overflow: 'hidden',
      display: 'flex',
      'flex-direction': 'column',
    },
  },),
  $({
    rule: 'dialog::backdrop',
    decls: {
      'background-color': cssOklch({
        l: 0,
        c: 0,
        h: 0,
        a: BACKDROP_ALPHA,
      },),
    },
  },),
  $({
    rule: '.search-input',
    decls: {
      'inline-size': cssPercent(100,),
      'padding-block': cssRem(INPUT_PADDING,),
      'padding-inline': cssRem(INPUT_PADDING,),
      'border-block-style': 'none',
      'border-inline-style': 'none',
      'border-block-end-width': cssRem(1 / (2 * 2
        * 2
        * 2),),
      'border-block-end-style': 'solid',
      'border-block-end-color': cssVar('gutter-fg',),
      'background-color': cssVar('bg',),
      color: cssVar('fg',),
      'font-family': 'inherit',
      'font-size': cssRem(1,),
      outline: 'none',
      'box-sizing': 'border-box',
    },
  },),
  $({
    rule: '.search-input::placeholder',
    decls: {
      color: cssVar('gutter-fg',),
    },
  },),
]
  .join('',);
