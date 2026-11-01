/**
 Desktop media-query styles for the `<top-nav>` web component.
 */
import {
  cssCalc,
  cssRem,
  cssVar,
} from '@monochromatic-dev/hyperscript-core/ts';
import { $ as css, } from '../css.ts';

/**
 Desktop breakpoint overrides for `<top-nav>`: shows full nav, hides hamburger.
 */
export const TOP_NAV_MEDIA_STYLES: string = css({
  at: 'media',
  params: '(min-width: 48rem)',
  children: [
    css({
      rule: ':host',
      decls: {
        'justify-content': 'space-between',
        'padding-inline-start': cssVar('min-gap',),
        'border-block-end-width': cssCalc(`${cssRem(1,)} / 16`,),
        'border-block-end-style': 'solid',
        'border-block-end-color': cssVar('bg-weaker',),
      },
    },),
    css({
      rule: '.menu-toggle',
      decls: { display: 'none', },
    },),
    css({
      rule: 'h1',
      decls: { 'text-align': 'start', },
    },),
  ],
},);
