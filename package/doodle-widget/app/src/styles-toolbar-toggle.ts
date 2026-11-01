/**
 Toggle-group CSS overrides for the doodle widget toolbar.
 
 Base control styling (padding, cursor, bg, border, radius, font) is
 handled by the shared control selector in {@link renderToolbarStyles}.
 This module provides toggle-specific overrides: connected border
 joining, hidden radio inputs, and checked-state feedback.
 */
import {
  cssNum,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';
import {
  BG_TOGGLE_CHECKED,
  BG_TOGGLE_CHECKED_HOVER,
  BORDER_COLOR,
  BORDER_WIDTH,
  BUTTON_RADIUS,
} from './style-tokens.ts';

/**
 Generates CSS rules for the radio-based toggle button group.
 
 @returns array of minified CSS rule strings
 
 @example
 ```ts
 const css = renderToggleGroupStyles().join('');
 ```
 */
export function renderToggleGroupStyles(): string[] {
  return [
    $({
      rule: '.toggle-group',
      decls: { display: 'flex', },
    },),

    /**
     Hide the native radio circle
     */
    $({
      rule: '.toggle-option input',
      decls: {
        position: 'absolute',
        appearance: 'none',
        'inline-size': cssNum(0,),
        'block-size': cssNum(0,),
        opacity: cssNum(0,),
      },
    },),

    /**
     Override base border/radius for connected toggle items
     */
    $({
      rule: '.toggle-option',
      decls: {
        display: 'flex',
        'align-items': 'center',
        'border-inline-end-width': cssNum(0,),
        'border-radius': cssNum(0,),
      },
    },),

    $({
      rule: '.toggle-option:first-child',
      decls: {
        'border-start-start-radius': BUTTON_RADIUS,
        'border-end-start-radius': BUTTON_RADIUS,
      },
    },),

    $({
      rule: '.toggle-option:last-child',
      decls: {
        'border-start-end-radius': BUTTON_RADIUS,
        'border-end-end-radius': BUTTON_RADIUS,
        'border-inline-end-width': BORDER_WIDTH,
        'border-inline-end-style': 'solid',
        'border-inline-end-color': BORDER_COLOR,
      },
    },),

    /**
     Active state driven by native :checked pseudo-class
     */
    $({
      rule: '.toggle-option:has(input:checked)',
      decls: { 'background-color': BG_TOGGLE_CHECKED, },
    },),

    $({
      rule: '.toggle-option:has(input:checked):hover',
      decls: { 'background-color': BG_TOGGLE_CHECKED_HOVER, },
    },),
  ];
}
