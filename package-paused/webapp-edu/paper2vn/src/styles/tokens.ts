/**
 * Design tokens for paper2vn CSS.
 *
 * All sizing in `rem` (with `calc()` for derivation), all colors as
 * CSS custom properties so the design language can be retuned in one
 * place. No tokens emit `var(...)` fallbacks per project CSS rules.
 */
import {
  cssCalc,
  cssDvb,
  cssDvi,
  cssNum,
  cssPercent,
  cssRem,
  type CssValue,
} from '@monochromatic-dev/hyperscript-core/ts';
import {
  HALF,
  QUARTER,
  THREE_QUARTERS,
} from '@monochromatic-dev/const-core';

//region Fractional building blocks (max-magnitude 2 per project rules)

/**
 * 1/8
 */
const EIGHTH: number = QUARTER / 2;

/**
 * Tight line height for primary headings (h1)
 */
export const LINE_HEIGHT_TIGHT: number = 1 + (1 / (1 + 2
  + 2));

/**
 * Snug line height for secondary headings (h2)
 */
export const LINE_HEIGHT_SNUG: number = 1 + (1 / (1 + 2));

/**
 * Normal line height for body text
 */
export const LINE_HEIGHT_NORMAL: number = 1 + HALF;

//endregion

//region Whole-unit constants

/**
 * Whole unit (100 for percent and viewport units)
 */
const WHOLE = 100;

/**
 * Bold font weight literal
 */
const BOLD = 700;

/**
 * Touch-target minimum in rem (48px equivalent at 16px root)
 */
const TOUCH_TARGET_REM = 3;

/**
 * Small font size in rem (0.875 ≈ 14px at 16px root)
 */
const SMALL_FONT_SIZE_REM = THREE_QUARTERS + EIGHTH;

/**
 * Heading 2 font size in rem (1.5 ≈ 24px at 16px root)
 */
const H2_FONT_SIZE_REM = 1 + HALF;

/**
 * Chapter card heading font size in rem (2.5 ≈ 40px at 16px root)
 */
const HUGE_FONT_SIZE_REM = 2 + HALF;

/**
 * Textarea minimum block size in rem (8 ≈ 128px at 16px root)
 */
const TEXTAREA_MIN_BLOCK_REM = 8;

/**
 * Vertical offset reserved for stage UI above the character (14 rem)
 */
const STAGE_CHARACTER_OFFSET_REM = 14;

/**
 * Screen content max inline size in rem (48 ≈ 768px at 16px root)
 */
const SCREEN_MAX_INLINE_REM = 48;

//endregion

//region Computed CSS values

/**
 * Full dynamic viewport block size
 */
export const FULL_DVB: CssValue = cssDvb(WHOLE,);

/**
 * Full dynamic viewport inline size
 */
export const FULL_DVI: CssValue = cssDvi(WHOLE,);

/**
 * Full percentage (100%)
 */
export const FULL_PERCENT: CssValue = cssPercent(WHOLE,);

/**
 * Bold weight
 */
export const FONT_WEIGHT_BOLD: CssValue = cssNum(BOLD,);

/**
 * Quarter rem: smallest spacing unit
 */
export const SPACE_QUARTER: CssValue = cssRem(QUARTER,);

/**
 * Half rem
 */
export const SPACE_HALF: CssValue = cssRem(HALF,);

/**
 * Three-quarter rem
 */
export const SPACE_THREE_QUARTERS: CssValue = cssRem(THREE_QUARTERS,);

/**
 * One rem
 */
export const SPACE_ONE: CssValue = cssRem(1,);

/**
 * Two rem
 */
export const SPACE_TWO: CssValue = cssRem(2,);

/**
 * Touch-target minimum size
 */
export const TOUCH_TARGET: CssValue = cssRem(TOUCH_TARGET_REM,);

/**
 * Border radius: subtle
 */
export const RADIUS_SMALL: CssValue = cssRem(QUARTER,);

/**
 * Border radius: pill / rounded
 */
export const RADIUS_LARGE: CssValue = cssRem(1,);

/**
 * Half of full block (used for dialogue box max-block-size)
 */
export const HALF_DVB: CssValue = cssCalc(`${cssDvb(WHOLE,)} * ${HALF}`,);

/**
 * Eighth rem: shadow offset
 */
export const SHADOW_OFFSET: CssValue = cssRem(EIGHTH,);

/**
 * Outline thickness: 1/8 rem
 */
export const OUTLINE_THIN: CssValue = cssRem(EIGHTH,);

/**
 * Stroke thickness: 1/4 rem
 */
export const STROKE_THICK: CssValue = cssRem(QUARTER,);

/**
 * Small font size (label, helper text)
 */
export const FONT_SIZE_SMALL: CssValue = cssRem(SMALL_FONT_SIZE_REM,);

/**
 * Heading 2 font size
 */
export const FONT_SIZE_H2: CssValue = cssRem(H2_FONT_SIZE_REM,);

/**
 * Chapter card heading font size
 */
export const FONT_SIZE_HUGE: CssValue = cssRem(HUGE_FONT_SIZE_REM,);

/**
 * Textarea minimum block size
 */
export const TEXTAREA_MIN_BLOCK: CssValue = cssRem(TEXTAREA_MIN_BLOCK_REM,);

/**
 * Stage character vertical offset
 */
export const STAGE_CHARACTER_OFFSET: CssValue = cssRem(STAGE_CHARACTER_OFFSET_REM,);

/**
 * Screen content max inline size
 */
export const SCREEN_MAX_INLINE: CssValue = cssRem(SCREEN_MAX_INLINE_REM,);

//endregion
