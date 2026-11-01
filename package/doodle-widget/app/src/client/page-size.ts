/**
 Fixed page dimensions for the doodle widget.
 
 US Letter (8.5 x 11 inches) at 96 DPI, matching the embedded
 SVG background dimensions. Used for export output sizing and
 aspect ratio calculation.
 */

/**
 DPI assumed for CSS pixel mapping
 */
const DPI = 96;

/**
 US Letter width in inches
 */
const LETTER_WIDTH_INCHES = 8.5;

/**
 US Letter height in inches
 */
const LETTER_HEIGHT_INCHES = 11;

/**
 US Letter width at 96 DPI (8.5 inches * 96 px/inch = 816 px)
 */
export const LETTER_WIDTH: number = LETTER_WIDTH_INCHES * DPI;

/**
 US Letter height at 96 DPI (11 inches * 96 px/inch = 1056 px)
 */
export const LETTER_HEIGHT: number = LETTER_HEIGHT_INCHES * DPI;
