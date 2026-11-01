/**
 Shared CSS sizing constants for the h-css style generators.
 
 These TypeScript-side values feed into `cssRem()` calls across
 style modules. CSS custom properties live in `tokens.ts`.
 */

import { HALF, } from '@monochromatic-dev/const-core/ts';

/**
 Maximum content width in rem.
 */
export const MAX_WIDTH = 48;

/**
 Base line-height ratio.
 */
export const LINE_HEIGHT = 1.6;

/**
 Standard gap between elements in rem.
 */
export const GAP = 1;

/**
 Small gap in rem.
 */
export const GAP_SMALL: number = HALF;

/**
 Border width in rem, equivalent to 1px at default font size.
 */
export const BORDER_WIDTH_REM = '1 / 16 * 1rem';

/**
 Minimum interactive touch target size in rem (48px equivalent).
 */
export const TOUCH_TARGET = 3;

/**
 Minimum width for the post grid column in rem.
 */
export const POST_GRID_MIN = 16;

/**
 Font size for h2 headings in rem.
 */
export const FONT_SIZE_H2 = 1.25;

/**
 Small font size for tags and dates in rem.
 */
export const FONT_SIZE_SMALL = 0.875;

/**
 Full width percentage for inputs.
 */
export const FULL_WIDTH = 100;
