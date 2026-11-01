/**
 * Layout containers: `.screen` flex column, `.menu` panel, form `.field`,
 * `.row` inline group, and `.muted` / `.error` text utilities.
 */
import {
  cssCommaList,
  cssNum,
  cssOklch,
  cssRem,
  cssVar,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

import {
  FONT_SIZE_SMALL,
  FULL_DVB,
  RADIUS_LARGE,
  SCREEN_MAX_INLINE,
  SHADOW_OFFSET,
  SPACE_HALF,
  SPACE_ONE,
  SPACE_QUARTER,
  SPACE_TWO,
} from './tokens.ts';

/**
 * Screen and panel layout styles.
 *
 * @returns ordered array of compiled rule strings
 *
 * @example
 * ```ts
 * const css = screenRules().join('\n');
 * ```
 */
export function screenRules(): string[] {
  return [
    $({
      rule: '.screen',
      decls: {
        display: 'flex',
        'flex-direction': 'column',
        'min-block-size': FULL_DVB,
        'padding-block': SPACE_TWO,
        'padding-inline': SPACE_TWO,
        gap: SPACE_ONE,
        'max-inline-size': SCREEN_MAX_INLINE,
        'margin-inline': 'auto',
      },
    },),
    $({
      rule: '.screen[data-screen="lecture"]',
      decls: {
        'max-inline-size': 'none',
        'padding-block': cssNum(0,),
        'padding-inline': cssNum(0,),
      },
    },),
    $({
      rule: '.menu',
      decls: {
        display: 'flex',
        'flex-direction': 'column',
        gap: SPACE_HALF,
        'padding-block': SPACE_ONE,
        'padding-inline': SPACE_ONE,
        'background-color': cssVar('bg-elevated',),
        'border-start-start-radius': RADIUS_LARGE,
        'border-start-end-radius': RADIUS_LARGE,
        'border-end-start-radius': RADIUS_LARGE,
        'border-end-end-radius': RADIUS_LARGE,
        'box-shadow': cssCommaList([
          `0 ${SHADOW_OFFSET} ${cssRem(1,)} ${cssVar('shadow',)}`,
        ],),
      },
    },),
    $({
      rule: '.menu button',
      decls: {
        'text-align': 'start',
      },
    },),
    $({
      rule: '.field',
      decls: {
        display: 'flex',
        'flex-direction': 'column',
        gap: SPACE_QUARTER,
      },
    },),
    $({
      rule: '.field > label',
      decls: {
        color: cssVar('fg-muted',),
        'font-size': FONT_SIZE_SMALL,
      },
    },),
    $({
      rule: '.row',
      decls: {
        display: 'flex',
        'flex-wrap': 'wrap',
        gap: SPACE_HALF,
        'align-items': 'center',
      },
    },),
    $({
      rule: '.muted',
      decls: {
        color: cssVar('fg-muted',),
      },
    },),
    $({
      rule: '.error',
      decls: {
        color: cssOklch({
          l: 0.55,
          c: 0.18,
          h: 28,
        },),
      },
    },),
  ];
}
