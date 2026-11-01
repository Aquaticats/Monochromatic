/**
 Toolbar and button CSS rules for the doodle widget.
 
 Covers the top toolbar bar, a shared base rule for all interactive
 controls (buttons, selects, toggle labels, color picker), and the
 toolbar title. Control-specific overrides live in composed modules:
 {@link renderToolbarControlStyles} and {@link renderToggleGroupStyles}.
 */
import {
  cssOklch,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';
import {
  BG_CONTROL,
  BG_CONTROL_HOVER,
  BORDER_COLOR,
  BORDER_SOLID_DECLS,
  BORDER_WIDTH,
  BUTTON_PADDING_BLOCK,
  BUTTON_PADDING_INLINE,
  BUTTON_RADIUS,
  FONT_WEIGHT_BOLD,
  TOOLBAR_FONT_DECLS,
  TOOLBAR_GAP,
  TOOLBAR_PADDING_BLOCK,
  TOOLBAR_PADDING_INLINE,
} from './style-tokens.ts';
import { renderToolbarControlStyles, } from './styles-toolbar-controls.ts';
import { renderToggleGroupStyles, } from './styles-toolbar-toggle.ts';

/**
 Selector targeting all interactive controls in the toolbar.
 
 Buttons, selects, toggle-option labels, and the color picker
 share the same base styling.
 */
const CONTROL_SELECTOR =
  '.toolbar button, .toolbar select, .toggle-option, #color-picker';

/**
 Generates CSS rules for the toolbar, buttons, and composed sub-groups.
 
 @returns array of minified CSS rule strings
 
 @example
 ```ts
 const css = renderToolbarStyles().join('');
 ```
 */
export function renderToolbarStyles(): string[] {
  return [
    $({
      rule: '.toolbar',
      decls: {
        display: 'flex',
        'flex-wrap': 'wrap',
        'align-items': 'center',
        gap: TOOLBAR_GAP,
        'padding-block': TOOLBAR_PADDING_BLOCK,
        'padding-inline': TOOLBAR_PADDING_INLINE,
        'background-color': cssOklch({
          l: 0.95,
          c: 0,
          h: 0,
        },),
        'border-block-end-style': 'solid',
        'border-block-end-color': BORDER_COLOR,
        'border-block-end-width': BORDER_WIDTH,
        ...TOOLBAR_FONT_DECLS,
      },
    },),

    $({
      rule: '.toolbar-title',
      decls: { 'font-weight': FONT_WEIGHT_BOLD, },
    },),

    /**
     Base styling shared by all interactive toolbar controls
     */
    $({
      rule: CONTROL_SELECTOR,
      decls: {
        'padding-block': BUTTON_PADDING_BLOCK,
        'padding-inline': BUTTON_PADDING_INLINE,
        cursor: 'pointer',
        'border-radius': BUTTON_RADIUS,
        'background-color': BG_CONTROL,
        ...BORDER_SOLID_DECLS,
        ...TOOLBAR_FONT_DECLS,
      },
    },),

    $({
      rule: CONTROL_SELECTOR
        .split(', ',)
        .map(
        function addHover(sel,): string {
          return `${sel}:hover`;
        },
        )
        .join(', ',),
      decls: { 'background-color': BG_CONTROL_HOVER, },
    },),

    ...renderToolbarControlStyles(),
    ...renderToggleGroupStyles(),
  ];
}
