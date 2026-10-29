/**
 * Stage scenery (background, character, dialogue), chapter cards, log
 * pane, toolbar, and the `[hidden]` overriding rule.
 */
import {
  cssCalc,
  cssCommaList,
  cssCompounded,
  cssNum,
  cssOklch,
  cssRem,
  cssVar,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

import {
  FONT_SIZE_HUGE,
  FONT_WEIGHT_BOLD,
  FULL_DVB,
  FULL_DVI,
  FULL_PERCENT,
  HALF_DVB,
  RADIUS_LARGE,
  SHADOW_OFFSET,
  SPACE_HALF,
  SPACE_ONE,
  SPACE_THREE_QUARTERS,
  SPACE_TWO,
  STAGE_CHARACTER_OFFSET,
  STROKE_THICK,
  TOUCH_TARGET,
} from './tokens.ts';

/**
 * Stage, chapter card, log pane, toolbar, and [hidden] rules.
 *
 * @returns ordered array of compiled rule strings
 *
 * @example
 * ```ts
 * const css = stageRules().join('\n');
 * ```
 */
export function stageRules(): string[] {
  return [
    $({
      rule: '.stage',
      decls: {
        position: 'relative',
        'block-size': FULL_DVB,
        'inline-size': FULL_DVI,
        overflow: 'hidden',
        'background-color': cssVar('bg-board',),
      },
    },),
    $({
      rule: '.stage-bg',
      decls: {
        position: 'absolute',
        'inset-block-start': cssNum(0,),
        'inset-block-end': cssNum(0,),
        'inset-inline-start': cssNum(0,),
        'inset-inline-end': cssNum(0,),
        'background-size': 'cover',
        'background-position': 'center',
      },
    },),
    $({
      rule: '.stage-character',
      decls: {
        position: 'absolute',
        'inset-block-end': cssNum(0,),
        'inset-inline-start': FULL_PERCENT,
        transform: cssCompounded(['translate(-50%, 0)',],),
        'block-size': cssCalc(`${FULL_DVB} - ${STAGE_CHARACTER_OFFSET}`,),
        'aspect-ratio': cssCompounded([
          '240',
          '/',
          '480',
        ],),
      },
    },),
    $({
      rule: '.stage-character img',
      decls: {
        'block-size': FULL_PERCENT,
        'inline-size': 'auto',
      },
    },),
    $({
      rule: '.stage-controls',
      decls: {
        position: 'absolute',
        'inset-block-start': SPACE_ONE,
        'inset-inline-end': SPACE_ONE,
        display: 'flex',
        gap: SPACE_HALF,
      },
    },),
    $({
      rule: '.stage-dialogue',
      decls: {
        position: 'absolute',
        'inset-inline-start': SPACE_ONE,
        'inset-inline-end': SPACE_ONE,
        'inset-block-end': SPACE_ONE,
        'background-color': cssVar('bg-elevated',),
        'border-start-start-radius': RADIUS_LARGE,
        'border-start-end-radius': RADIUS_LARGE,
        'border-end-start-radius': RADIUS_LARGE,
        'border-end-end-radius': RADIUS_LARGE,
        'padding-block': SPACE_ONE,
        'padding-inline': SPACE_ONE,
        'max-block-size': HALF_DVB,
        'overflow-y': 'auto',
        'box-shadow': cssCommaList([
          `0 ${SHADOW_OFFSET} ${cssRem(1,)} ${cssVar('shadow',)}`,
        ],),
      },
    },),
    $({
      rule: '.stage-dialogue header',
      decls: {
        display: 'flex',
        'align-items': 'center',
        'justify-content': 'space-between',
        'margin-block-end': SPACE_HALF,
      },
    },),
    $({
      rule: '.speaker-name',
      decls: {
        'font-weight': FONT_WEIGHT_BOLD,
        color: cssVar('accent',),
      },
    },),
    $({
      rule: '.dialogue-text',
      decls: {
        'white-space': 'pre-wrap',
        'overflow-wrap': 'anywhere',
      },
    },),
    $({
      rule: '.chapter-card',
      decls: {
        position: 'absolute',
        'inset-block-start': cssNum(0,),
        'inset-block-end': cssNum(0,),
        'inset-inline-start': cssNum(0,),
        'inset-inline-end': cssNum(0,),
        display: 'grid',
        'place-items': 'center',
        'background-color': cssOklch({
          l: 0,
          c: 0,
          h: 0,
          a: 0.5,
        },),
        color: cssOklch({
          l: 0.99,
          c: 0,
          h: 0,
        },),
        'text-align': 'center',
        'padding-block': SPACE_TWO,
        'padding-inline': SPACE_TWO,
      },
    },),
    $({
      rule: '.chapter-card h2',
      decls: {
        'font-size': FONT_SIZE_HUGE,
      },
    },),
    $({
      rule: '.log-pane',
      decls: {
        position: 'fixed',
        'inset-block-start': cssNum(0,),
        'inset-block-end': cssNum(0,),
        'inset-inline-start': cssNum(0,),
        'inset-inline-end': cssNum(0,),
        'background-color': cssVar('bg',),
        'overflow-y': 'auto',
        'padding-block': SPACE_TWO,
        'padding-inline': SPACE_TWO,
        'z-index': cssNum(10,),
      },
    },),
    $({
      rule: '.log-entry',
      decls: {
        'border-inline-start-width': STROKE_THICK,
        'border-inline-start-style': 'solid',
        'border-inline-start-color': cssVar('accent',),
        'padding-block': SPACE_HALF,
        'padding-inline-start': SPACE_THREE_QUARTERS,
        'margin-block-end': SPACE_THREE_QUARTERS,
      },
    },),
    $({
      rule: '.toolbar',
      decls: {
        display: 'flex',
        gap: SPACE_HALF,
      },
    },),
    $({
      rule: '.toolbar button',
      decls: {
        'min-inline-size': TOUCH_TARGET,
      },
    },),
    /**
     * `[hidden]` must override layout-specific `display` declarations
     * (`.chapter-card` uses `display: grid`, dialogue uses `display:
     * flex`, etc.). Placed last so equal-specificity rules order-win.
     */
    $({
      rule: '[hidden]',
      decls: {
        display: 'none',
      },
    },),
  ];
}
