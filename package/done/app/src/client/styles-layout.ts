/**
 Global CSS resets and layout shell rules.
 */
import {
  cssCalc,
  cssCommaList,
  cssDvb,
  cssRem,
  cssVar,
} from '@monochromatic-dev/hyperscript-core/ts';
import { $ as css, } from './css.ts';

/**
 Disabled button opacity.
 */
const DISABLED_OPACITY = 0.45;

/**
 Sidebar flex-basis in rem.
 */
const SIDEBAR_BASIS = 22;

/**
 Body max-inline-size numerator in px (1194/16 rem).
 */
const BODY_MAX_WIDTH_PX = 1_194;

/**
 Full viewport block-size in dvb units.
 */
const FULL_DVB = 100;

//region Resets

/**
 Box-model reset and form element font inheritance.
 */
export const resets: string = [
  css({
    rule: '*, *::before, *::after',
    decls: {
      'box-sizing': 'border-box',
      'margin-block': 0,
      'margin-inline': 0,
      'padding-block': 0,
      'padding-inline': 0,
    },
  },),
  css({
    rule: 'input, textarea, select, button',
    decls: {
      'font-family': 'inherit',
      'font-size': 'inherit',
      'font-style': 'inherit',
      'font-weight': 'inherit',
      'line-height': 'inherit',
    },
  },),
  css({
    rule: 'button:disabled',
    decls: {
      opacity: DISABLED_OPACITY,
      cursor: 'not-allowed',
    },
  },),
  css({
    rule: 'input::placeholder, textarea::placeholder',
    decls: { color: cssVar('medium',), },
  },),
]
  .join('',);

//endregion Resets

//region Layout shell

/**
 Body, sidebar, page-wrapper, and app container layout rules.
 */
export const layoutShell: string = [
  css({
    rule: ':root',
    decls: { '--sidebar-basis': cssRem(SIDEBAR_BASIS,), },
  },),
  css({
    rule: 'body',
    decls: {
      'font-family': cssCommaList([
        'Inter',
        'system-ui',
        'sans-serif',
      ],),
      color: cssVar('fg',),
      'background-color': cssVar('bg',),
      'max-inline-size': cssCalc(`${cssRem(BODY_MAX_WIDTH_PX,)} / 16`,),
      'margin-inline': 'auto',
      'overflow-x': 'hidden',
      'min-block-size': cssDvb(FULL_DVB,),
      display: 'flex',
      'flex-direction': 'row',
    },
  },),
  css({
    rule: 'side-drawer',
    decls: {
      'flex-shrink': 0,
      'flex-basis': 0,
      position: 'sticky',
      'inset-block-start': 0,
      'align-self': 'flex-start',
      'max-block-size': cssDvb(FULL_DVB,),
    },
  },),
  css({
    at: 'media',
    params: '(min-width: 48rem)',
    children: [
      css({
        rule: 'side-drawer',
        decls: { 'flex-basis': cssVar('sidebar-basis',), },
      },),
    ],
  },),
  css({
    rule: '.page-wrapper',
    decls: {
      display: 'flex',
      'flex-direction': 'column',
      'min-block-size': cssDvb(FULL_DVB,),
      'flex-grow': 1,
      'min-inline-size': 0,
    },
  },),
  css({
    rule: '#app',
    decls: {
      display: 'flex',
      'flex-direction': 'column',
      'padding-block': cssVar('gap',),
      'padding-inline': cssVar('min-gap',),
      gap: cssVar('gap',),
      'overflow-x': 'hidden',
    },
  },),
]
  .join('',);

//endregion Layout shell

// Multi-page utilities are in styles-utilities.ts
