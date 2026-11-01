/**
 * Shadow DOM styles for the `<binary-viewer>` web component.
 *
 * Host container and text display rules.
 * Media element styles are in binary-viewer-media.styles.ts.
 */

import {
  cssNum,
  cssRem,
  cssVar,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

import {
  CODE_LINE_HEIGHT,
  MONO_FONT_FAMILY,
} from '../styles/tokens.ts';
import { MEDIA_STYLES, } from './media.styles.ts';

/**
 * Shadow DOM styles for the binary viewer.
 */
export const STYLES: string = [
  $({
    rule: ':host',
    decls: {
      display: 'none',
      flex: '1',
      overflow: 'auto',
      'flex-direction': 'column',
      'align-items': 'center',
      'justify-content': 'center',
      'background-color': cssVar('bg',),
    },
  },),
  MEDIA_STYLES,
  $({
    rule: '.media-info',
    decls: {
      'font-family': MONO_FONT_FAMILY,
      'line-height': cssNum(CODE_LINE_HEIGHT,),
      'padding-block': cssRem(1,),
      'padding-inline': cssRem(1,),
      margin: '0',
      color: cssVar('fg',),
      'white-space': 'pre-wrap',
    },
  },),
  $({
    rule: 'pre',
    decls: {
      'font-family': MONO_FONT_FAMILY,
      'font-size': cssRem(1,),
      'line-height': cssNum(CODE_LINE_HEIGHT,),
      'padding-block': cssRem(1,),
      'padding-inline': cssRem(1,),
      margin: '0',
      overflow: 'auto',
      'align-self': 'start',
      color: cssVar('fg',),
      'white-space': 'pre',
    },
  },),
]
  .join('',);
