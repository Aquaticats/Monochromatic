/**
 Export group and draw settings CSS overrides for the doodle widget toolbar.
 
 Base control styling (padding, cursor, bg, border, radius, font) is
 handled by the shared control selector in {@link renderToolbarStyles}.
 This module only provides layout and element-specific overrides.
 */
import {
  cssNum,
  cssRem,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

/**
 Color picker square size
 */
const COLOR_PICKER_SIZE = 2;

/**
 Stroke width slider track length
 */
const SLIDER_INLINE_SIZE = 2 + 2
  + 2;

/**
 Generates CSS rules for the export group and draw settings controls.
 
 @returns array of minified CSS rule strings
 
 @example
 ```ts
 const css = renderToolbarControlStyles().join('');
 ```
 */
export function renderToolbarControlStyles(): string[] {
  return [
    //region Export group (connected button + select)

    $({
      rule: '.export-group',
      decls: { display: 'flex', },
    },),

    /**
     Remove right border and radius to connect with adjacent select
     */
    $({
      rule: '.export-group > button',
      decls: {
        'border-start-end-radius': cssNum(0,),
        'border-end-end-radius': cssNum(0,),
        'border-inline-end-width': cssNum(0,),
      },
    },),

    /**
     Remove left radius to connect with adjacent button
     */
    $({
      rule: '.export-group > select',
      decls: {
        'border-start-start-radius': cssNum(0,),
        'border-end-start-radius': cssNum(0,),
      },
    },),

    //endregion Export group

    //region Draw settings (color picker + size slider)

    $({
      rule: '.draw-settings',
      decls: {
        display: 'flex',
        'align-items': 'center',
        gap: cssRem(1 / 2,),
      },
    },),

    /**
     Color picker size override (base border/radius from shared control rule)
     */
    $({
      rule: '#color-picker',
      decls: {
        'inline-size': cssRem(COLOR_PICKER_SIZE,),
        'block-size': cssRem(COLOR_PICKER_SIZE,),
        'padding-block': cssRem(1 / (2 * 2
          * 2),),
        'padding-inline': cssRem(1 / (2 * 2
          * 2),),
      },
    },),

    $({
      rule: '#size-slider',
      decls: {
        cursor: 'pointer',
        'inline-size': cssRem(SLIDER_INLINE_SIZE,),
      },
    },),
    //endregion Draw settings
  ];
}
