/**
 Inline sidebar styles for the `<side-drawer>` web component.
 
 Renders the persistent sidebar visible above the desktop breakpoint
 and hidden on narrower viewports where the popover panel is used instead.
 */
import {
  cssCalc,
  cssPercent,
  cssRem,
  cssVar,
} from '@monochromatic-dev/hyperscript-core/ts';
import { $ as css, } from '../css.ts';
import { flexColumn, } from '../mixins.ts';

/** Viewport breakpoint matching the body flex-wrap threshold.
  Below this width the sidebar stacks and the inline nav hides. */
const DESKTOP_BREAKPOINT = '48rem';

/**
 Full percentage for sidebar block-size.
 */
const FULL_PERCENT = 100;

/**
 Compiled CSS rules for the `<side-drawer>` inline sidebar.
 */
export const SIDE_DRAWER_SIDEBAR_STYLES: string[] = [
  css({
    rule: '.sidebar',
    decls: {
      ...flexColumn(),
      'block-size': cssPercent(FULL_PERCENT,),
      'border-inline-end-width': cssCalc(`${cssRem(1,)} / 16`,),
      'border-inline-end-style': 'solid',
      'border-inline-end-color': cssVar('bg-weaker',),
      display: 'none',
    },
    children: [
      css({
        rule: '& .header',
        decls: {
          'padding-block': cssVar('min-padding',),
          'padding-inline-start': cssVar('min-gap',),
          'padding-inline-end': cssVar('min-padding',),
        },
      },),
      css({
        rule: '& .close',
        decls: { display: 'none', },
      },),
    ],
  },),
  css({
    at: 'media',
    params: `(min-width: ${DESKTOP_BREAKPOINT})`,
    children: [
      css({
        rule: '.sidebar',
        decls: { ...flexColumn(), },
      },),
    ],
  },),
];
