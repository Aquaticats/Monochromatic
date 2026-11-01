/**
 Shadow DOM styles for the `<side-drawer>` web component.
 
 Assembles nav, sidebar, and panel styles into a single CSS string.
 */
import { cssPercent, } from '@monochromatic-dev/hyperscript-core/ts';
import { $ as css, } from '../css.ts';
import { SIDE_DRAWER_NAV_STYLES, } from './side-drawer-styles-nav.ts';
import { SIDE_DRAWER_PANEL_STYLES, } from './side-drawer-styles-panel.ts';
import { SIDE_DRAWER_SIDEBAR_STYLES, } from './side-drawer-styles-sidebar.ts';

/**
 Full percentage for wrapper block-size.
 */
const FULL_PERCENT = 100;

/**
 Compiled CSS string for `<side-drawer>` Shadow DOM.
 */
export const SIDE_DRAWER_STYLES: string = [
  css({
    rule: ':host',
    decls: { display: 'block', },
  },),
  css({
    rule: '.wrapper',
    decls: { 'block-size': cssPercent(FULL_PERCENT,), },
  },),
  ...SIDE_DRAWER_NAV_STYLES,
  ...SIDE_DRAWER_SIDEBAR_STYLES,
  SIDE_DRAWER_PANEL_STYLES,
]
  .join('',);
