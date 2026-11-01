/**
 Design tokens for the doodle widget CSS.
 
 Defines shared constants (spacing, colors, font weights, border radii)
 consumed by style rule modules.
 */
import {
  type CssDeclarations,
  cssDvb,
  cssNum,
  cssOklch,
  cssPercent,
  cssRem,
  type CssValue,
} from '@monochromatic-dev/hyperscript-core/ts';
import {
  HALF,
  QUARTER,
  THREE_QUARTERS,
} from '@monochromatic-dev/const-core/ts';

//region Fractional building blocks

/**
 Whole unit (100 for viewport/percent)
 */
const WHOLE = 100;

/**
 Bold font weight value
 */
const BOLD = 700;

/**
 3/8
 */
const THREE_EIGHTHS: number = THREE_QUARTERS / 2;

/**
 1/8
 */
const EIGHTH: number = QUARTER / 2;

/**
 1/16
 */
const SIXTEENTH = EIGHTH / 2;

//endregion Fractional building blocks

//region Computed CSS values

/**
 Full dynamic viewport block size
 */
export const FULL_DVB: CssValue = cssDvb(WHOLE,);

/**
 Full percentage (100%)
 */
export const FULL_PERCENT: CssValue = cssPercent(WHOLE,);

/**
 Bold font weight
 */
export const FONT_WEIGHT_BOLD: CssValue = cssNum(BOLD,);

/**
 Standard toolbar gap between items
 */
export const TOOLBAR_GAP: CssValue = cssRem(THREE_QUARTERS,);

/**
 Vertical padding for toolbar
 */
export const TOOLBAR_PADDING_BLOCK: CssValue = cssRem(HALF,);

/**
 Horizontal padding for toolbar
 */
export const TOOLBAR_PADDING_INLINE: CssValue = cssRem(1,);

/**
 Vertical padding for buttons
 */
export const BUTTON_PADDING_BLOCK: CssValue = cssRem(THREE_EIGHTHS,);

/**
 Horizontal padding for buttons
 */
export const BUTTON_PADDING_INLINE: CssValue = cssRem(THREE_QUARTERS,);

/**
 Button corner radius
 */
export const BUTTON_RADIUS: CssValue = cssRem(QUARTER,);

/**
 Shared border width for toolbar controls (1/16 rem ~ 1px at default font size)
 */
export const BORDER_WIDTH: CssValue = cssRem(SIXTEENTH,);

/**
 Shared border color for toolbar controls
 */
export const BORDER_COLOR: CssValue = cssOklch({
  l: 0.8,
  c: 0,
  h: 0,
},);

/**
 Background color for the canvas viewport area behind the page
 */
export const BG_VIEWPORT: CssValue = cssOklch({
  l: 0.93,
  c: 0,
  h: 0,
},);

/**
 Border color for the page frame
 */
export const PAGE_FRAME_COLOR: CssValue = cssOklch({
  l: 0.7,
  c: 0,
  h: 0,
},);

/**
 Eight, used for US Letter width base
 */
const EIGHT = 2 * 2
  * 2;

/**
 US Letter width in inches (8.5)
 */
export const LETTER_WIDTH_IN: number = EIGHT + HALF;

/**
 US Letter height in inches
 */
export const LETTER_HEIGHT_IN = 11;

/* oxlint-disable typescript-eslint/no-unsafe-type-assertion -- branded CssValue requires assertion from template string */
/**
 US Letter aspect ratio (8.5 / 11) for CSS `aspect-ratio`
 */
export const LETTER_ASPECT_RATIO: CssValue = `${String(LETTER_WIDTH_IN,)} / ${
  String(LETTER_HEIGHT_IN,)
}` satisfies string as CssValue;
/* oxlint-enable typescript-eslint/no-unsafe-type-assertion */

/**
 Padding around the page inside the viewport
 */
export const VIEWPORT_PADDING: CssValue = cssRem(1,);

//endregion Computed CSS values

//region Shared declaration fragments

/**
 Default background color for interactive controls
 */
export const BG_CONTROL: CssValue = cssOklch({
  l: 0.97,
  c: 0,
  h: 0,
},);

/**
 Hover background color for interactive controls
 */
export const BG_CONTROL_HOVER: CssValue = cssOklch({
  l: 0.92,
  c: 0,
  h: 0,
},);

/**
 Background color for checked toggle controls
 */
export const BG_TOGGLE_CHECKED: CssValue = cssOklch({
  l: 0.82,
  c: 0,
  h: 0,
},);

/**
 Hover background color for checked toggle controls
 */
export const BG_TOGGLE_CHECKED_HOVER: CssValue = cssOklch({
  l: 0.78,
  c: 0,
  h: 0,
},);

/**
 Solid border on both block and inline axes
 */
export const BORDER_SOLID_DECLS: CssDeclarations = {
  'border-block-style': 'solid',
  'border-block-color': BORDER_COLOR,
  'border-block-width': BORDER_WIDTH,
  'border-inline-style': 'solid',
  'border-inline-color': BORDER_COLOR,
  'border-inline-width': BORDER_WIDTH,
};

/**
 Toolbar and control font stack
 */
export const TOOLBAR_FONT_DECLS: CssDeclarations = {
  'font-family': 'sans-serif',
  'font-size': cssRem(1,),
};

/**
 Absolute positioning filling parent via zero insets
 */
export const INSET_ZERO_DECLS: CssDeclarations = {
  position: 'absolute',
  'inset-block': cssNum(0,),
  'inset-inline': cssNum(0,),
};

//endregion Shared declaration fragments
