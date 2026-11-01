/**
 * Element-level base styles: box-model reset, html/body/main, headings,
 * default form controls, and focus ring.
 */
import {
  cssCalc,
  cssNum,
  cssRem,
  cssVar,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

import {
  FONT_SIZE_H2,
  FONT_WEIGHT_BOLD,
  FULL_DVB,
  FULL_DVI,
  FULL_PERCENT,
  LINE_HEIGHT_NORMAL,
  LINE_HEIGHT_SNUG,
  LINE_HEIGHT_TIGHT,
  OUTLINE_THIN,
  RADIUS_SMALL,
  SPACE_HALF,
  SPACE_ONE,
  SPACE_THREE_QUARTERS,
  TEXTAREA_MIN_BLOCK,
  TOUCH_TARGET,
} from './tokens.ts';

/**
 * Element-level base styles.
 *
 * @returns ordered array of compiled rule strings
 *
 * @example
 * ```ts
 * const css = elementRules().join('\n');
 * ```
 */
export function elementRules(): string[] {
  return [
    $({
      rule: '*, *::before, *::after',
      decls: {
        'box-sizing': 'border-box',
        'margin-block-start': cssNum(0,),
        'margin-block-end': cssNum(0,),
        'margin-inline-start': cssNum(0,),
        'margin-inline-end': cssNum(0,),
        'padding-block-start': cssNum(0,),
        'padding-block-end': cssNum(0,),
        'padding-inline-start': cssNum(0,),
        'padding-inline-end': cssNum(0,),
      },
    },),
    $({
      rule: 'html, body',
      decls: {
        'block-size': FULL_PERCENT,
        'inline-size': FULL_PERCENT,
        'background-color': cssVar('bg',),
        color: cssVar('fg',),
        'font-size': cssCalc(`1rem * ${cssVar('font-scale',)}`,),
      },
    },),
    $({
      rule: 'body',
      decls: {
        'line-height': cssNum(LINE_HEIGHT_NORMAL,),
      },
    },),
    $({
      rule: 'main#app',
      decls: {
        display: 'block',
        'min-block-size': FULL_DVB,
        'min-inline-size': FULL_DVI,
      },
    },),
    $({
      rule: 'button, input, select, textarea',
      decls: {
        font: 'inherit',
        color: 'inherit',
      },
    },),
    $({
      rule: 'button',
      decls: {
        cursor: 'pointer',
        'min-block-size': TOUCH_TARGET,
        'min-inline-size': TOUCH_TARGET,
        'background-color': cssVar('bg-elevated',),
        'border-block-start-width': cssRem(1 / 16,),
        'border-block-end-width': cssRem(1 / 16,),
        'border-inline-start-width': cssRem(1 / 16,),
        'border-inline-end-width': cssRem(1 / 16,),
        'border-block-start-style': 'solid',
        'border-block-end-style': 'solid',
        'border-inline-start-style': 'solid',
        'border-inline-end-style': 'solid',
        'border-block-start-color': cssVar('border',),
        'border-block-end-color': cssVar('border',),
        'border-inline-start-color': cssVar('border',),
        'border-inline-end-color': cssVar('border',),
        'border-start-start-radius': RADIUS_SMALL,
        'border-start-end-radius': RADIUS_SMALL,
        'border-end-start-radius': RADIUS_SMALL,
        'border-end-end-radius': RADIUS_SMALL,
        'padding-block': SPACE_HALF,
        'padding-inline': SPACE_ONE,
      },
    },),
    $({
      rule: 'button[data-variant="primary"]',
      decls: {
        'background-color': cssVar('accent',),
        color: cssVar('accent-fg',),
        'border-block-start-color': cssVar('accent',),
        'border-block-end-color': cssVar('accent',),
        'border-inline-start-color': cssVar('accent',),
        'border-inline-end-color': cssVar('accent',),
        'font-weight': FONT_WEIGHT_BOLD,
      },
    },),
    $({
      rule: 'button[data-variant="ghost"]',
      decls: {
        'background-color': 'transparent',
        'border-block-start-color': 'transparent',
        'border-block-end-color': 'transparent',
        'border-inline-start-color': 'transparent',
        'border-inline-end-color': 'transparent',
      },
    },),
    $({
      rule: ':focus-visible',
      decls: {
        outline: `${OUTLINE_THIN} solid ${cssVar('accent',)}`,
        'outline-offset': OUTLINE_THIN,
      },
    },),
    $({
      rule: '.h1, h1',
      decls: {
        'font-size': cssRem(2,),
        'font-weight': FONT_WEIGHT_BOLD,
        'line-height': cssNum(LINE_HEIGHT_TIGHT,),
      },
    },),
    $({
      rule: '.h2, h2',
      decls: {
        'font-size': FONT_SIZE_H2,
        'font-weight': FONT_WEIGHT_BOLD,
        'line-height': cssNum(LINE_HEIGHT_SNUG,),
      },
    },),
    $({
      rule: 'input, select, textarea',
      decls: {
        'background-color': cssVar('bg-elevated',),
        color: cssVar('fg',),
        'min-block-size': TOUCH_TARGET,
        'padding-block': SPACE_HALF,
        'padding-inline': SPACE_THREE_QUARTERS,
        'border-block-start-width': cssRem(1 / 16,),
        'border-block-end-width': cssRem(1 / 16,),
        'border-inline-start-width': cssRem(1 / 16,),
        'border-inline-end-width': cssRem(1 / 16,),
        'border-block-start-style': 'solid',
        'border-block-end-style': 'solid',
        'border-inline-start-style': 'solid',
        'border-inline-end-style': 'solid',
        'border-block-start-color': cssVar('border',),
        'border-block-end-color': cssVar('border',),
        'border-inline-start-color': cssVar('border',),
        'border-inline-end-color': cssVar('border',),
        'border-start-start-radius': RADIUS_SMALL,
        'border-start-end-radius': RADIUS_SMALL,
        'border-end-start-radius': RADIUS_SMALL,
        'border-end-end-radius': RADIUS_SMALL,
      },
    },),
    $({
      rule: 'textarea',
      decls: {
        'min-block-size': TEXTAREA_MIN_BLOCK,
        'font-family': 'inherit',
      },
    },),
  ];
}
