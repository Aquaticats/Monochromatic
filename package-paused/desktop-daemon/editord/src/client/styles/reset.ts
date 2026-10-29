/**
 * Global reset and base layout styles for the editord document.
 *
 * Universal reset, `html`/`body` base styles, `#app` flex container,
 * and light-DOM rules that cannot be scoped to a shadow root.
 * Built at compile time into `dist/client/global.css`.
 */

import {
  cssCompounded,
  cssInt,
  cssPercent,
  cssRem,
  cssVar,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

import { MONO_FONT_FAMILY, } from './tokens.ts';

/**
 * Full dimension as percentage.
 */
const FULL = 100;

/**
 * Global reset and base layout rules.
 */
export const STYLES: string = [
  $({
    rule: '*',
    decls: {
      'margin-block': cssInt(0,),
      'margin-inline': cssInt(0,),
      'padding-block': cssInt(0,),
      'padding-inline': cssInt(0,),
      'box-sizing': 'border-box',
    },
  },),
  $({
    rule: 'html, body',
    decls: {
      'block-size': cssPercent(FULL,),
      'inline-size': cssPercent(FULL,),
      'background-color': cssVar('bg',),
      color: cssVar('fg',),
      'font-family': MONO_FONT_FAMILY,
      'scrollbar-color': cssCompounded([
        cssVar('gutter-fg',),
        cssVar('tree-hover-bg',),
      ],),
    },
  },),
  $({
    rule: '#app',
    decls: {
      display: 'flex',
      'block-size': cssPercent(FULL,),
    },
  },),
  $({
    rule: '.ctx-item',
    decls: {
      'padding-block': cssRem(1 / (2 * 2),),
      'padding-inline': cssRem(1 / 2,),
      cursor: 'pointer',
      'white-space': 'nowrap',
    },
    children: [
      $({
        rule: '&:hover, &:focus-visible',
        decls: {
          'background-color': cssVar('tree-hover-bg',),
        },
      },),
    ],
  },),
  $({
    rule: 'references-popup',
    decls: {
      /**
       * `position-anchor` must live in light DOM scope to resolve
       * `anchor-name` across tree scopes (shadow boundary).
       */
      'position-anchor': '--ref-anchor',
    },
  },),
]
  .join('',);
