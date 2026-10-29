/**
 Build-time favicon generator: derives the `w<` wordmark's stick
 spines from first principles (no font), emits them as an SVG
 document, and rasterizes that same SVG to a PNG through sharp, so
 the raster can never drift from the vector. `./build.ts` inlines
 both as data URIs: the SVG for engines that take vector icons, the
 PNG as the raster fallback.
 
 Geometry decree: every stick shares one length and one width; the
 w's four sticks tilt so every turn closes at 30 degrees, and the
 chevron's two sticks close at 60 degrees. Ink is `oklch(0.9 0 0)` on
 an `oklch(0.1 0 0)` ground, matching the page's dark palette stops;
 both are spelled as hex in the SVG so librsvg (sharp's SVG
 rasterizer) and browsers paint identical bytes.
 */
import { Buffer, } from 'node:buffer';

import sharp from 'sharp';

/**
 Favicon edge length in pixels, exported so tests can assert the
 SVG viewport and the rasterized dimensions.
 */
export const FAVICON_SIZE: number = 64;

/**
 Shared stick length in pixels; every stroke of both glyphs uses it.
 */
const STICK_LENGTH = 24;

/**
 Half the shared stick width in pixels; the SVG stroke width is twice
 this.
 */
const STICK_HALF_WIDTH = 2.2;

/**
 Stick count of the w glyph (down, up, down, up).
 */
const W_STICK_COUNT = 2 * 2;

/**
 Quarter turn in radians, the base the glyph angles derive from.
 */
const QUARTER_TURN_RAD = Math.PI / 2;

/**
 The w's tilt off vertical in radians: 15 degrees, so two adjacent
 sticks close each turn at 30 degrees.
 */
const W_TILT_RAD = QUARTER_TURN_RAD / ((2 * 2) + 2);

/**
 The chevron's half-angle off horizontal in radians: 30 degrees, so
 its two sticks close at 60 degrees.
 */
const CHEVRON_HALF_RAD = QUARTER_TURN_RAD / (2 + 1);

/**
 Gap between the two glyphs, as a fraction of {@link STICK_LENGTH}.
 */
const GLYPH_GAP_RATIO = 0.1 * ((2 + 1) + (1 / 2));

/**
 Ink lightness: the palette's near-white stop, `oklch(0.9 0 0)`.
 */
const INK_LIGHTNESS = 1 - 0.1;

/**
 Ground lightness: the palette's near-black stop, `oklch(0.1 0 0)`.
 */
const GROUND_LIGHTNESS = 0.1;

/**
 Luminance below which sRGB's transfer function stays linear.
 */
const SRGB_LINEAR_THRESHOLD = 0.0031308;

/**
 Slope of sRGB's linear segment.
 */
const SRGB_LINEAR_SLOPE = 12.92;

/**
 Scale of sRGB's gamma segment.
 */
const SRGB_GAMMA_SCALE = 1.055;

/**
 Offset of sRGB's gamma segment.
 */
const SRGB_GAMMA_OFFSET = 0.055;

/**
 Exponent denominator of sRGB's gamma segment.
 */
const SRGB_GAMMA = 2.4;

/**
 Decimal places kept for SVG coordinates; hundredths sit far below
 one raster pixel, and fixing the precision keeps the markup
 deterministic.
 */
const SVG_COORD_DECIMALS = 2;

/**
 One line segment: a stick's spine from `(ax, ay)` to `(bx, by)`.
 */
type Segment = Readonly<{
  /**
   Spine start x.
   */
  ax: number;
  /**
   Spine start y.
   */
  ay: number;
  /**
   Spine end x.
   */
  bx: number;
  /**
   Spine end y.
   */
  by: number;
}>;

/**
 Converts an achromatic OKLCH lightness to an 8-bit sRGB channel
 value. For grays, OKLab lightness is the cube root of CIE luminance
 Y, so Y is recovered as lightness cubed, then pushed through sRGB's
 piecewise transfer function.
 
 @param lightness - OKLCH lightness in 0 to 1
 
 @returns sRGB channel byte in 0 to 255
 
 @example
 ```ts
 achromaticLightnessToSrgbByte(1); // 255
 ```
 */
function achromaticLightnessToSrgbByte(lightness: number,): number {
  /**
   CIE luminance Y recovered from the achromatic OKLab lightness.
   */
  const luminance = lightness ** (2 + 1);

  /**
   Nonlinear sRGB channel value in 0 to 1, via the piecewise sRGB
   transfer function.
   */
  const channel = luminance <= SRGB_LINEAR_THRESHOLD
    ? luminance * SRGB_LINEAR_SLOPE
    : (SRGB_GAMMA_SCALE * (luminance ** (1 / SRGB_GAMMA))) - SRGB_GAMMA_OFFSET;

  return Math.round(channel * 255,);
}

/**
 Converts an achromatic OKLCH lightness to a hex sRGB gray for SVG
 paint attributes.
 
 @param lightness - OKLCH lightness in 0 to 1
 
 @returns six-digit hex color, e.g. `#dedede`
 */
function lightnessToHexGray(lightness: number,): string {
  /**
   Shared gray byte of all three channels.
   */
  const byte = achromaticLightnessToSrgbByte(lightness,);

  /**
   Hex digits of the gray byte, before zero-padding.
   */
  const hexByte = byte.toString(16,);

  /**
   Two-digit hex form of the gray byte.
   */
  const channel = hexByte.padStart(
    2,
    '0',
  );

  return `#${channel}${channel}${channel}`;
}

/**
 Formats one SVG coordinate: fixed decimal precision with trailing
 zeros dropped, so the markup stays compact and deterministic.
 
 @param value - coordinate in SVG user units
 
 @returns formatted coordinate string
 */
function svgNum(value: number,): string {
  /**
   Fixed-precision form, possibly carrying trailing zeros.
   */
  const fixed = value.toFixed(SVG_COORD_DECIMALS,);

  return String(Number(fixed,),);
}

/**
 Builds the spines of all six sticks: the w's four alternating
 down/up strokes, then the chevron's two strokes meeting at a left
 apex, horizontally centered as one group and vertically centered on
 the icon.
 
 @returns stick spines in drawing order
 */
function buildGlyphSegments(): readonly Segment[] {
  /**
   Horizontal run of one w stick.
   */
  const wStickDx = STICK_LENGTH * Math.sin(W_TILT_RAD,);

  /**
   Vertical run of one w stick.
   */
  const wStickDy = STICK_LENGTH * Math.cos(W_TILT_RAD,);

  /**
   Horizontal run of one chevron stick.
   */
  const chevronDx = STICK_LENGTH * Math.cos(CHEVRON_HALF_RAD,);

  /**
   Vertical run of one chevron stick.
   */
  const chevronDy = STICK_LENGTH * Math.sin(CHEVRON_HALF_RAD,);

  /**
   Full inline extent of the w glyph.
   */
  const wWidth = wStickDx * W_STICK_COUNT;

  /**
   Gap between the w and the chevron in pixels.
   */
  const glyphGap = STICK_LENGTH * GLYPH_GAP_RATIO;

  /**
   Inline extent of both glyphs plus the gap between them.
   */
  const contentWidth = (wWidth + glyphGap) + chevronDx;

  /**
   Left edge of the w, centering the whole wordmark horizontally.
   */
  const startX = (FAVICON_SIZE - contentWidth) / 2;

  /**
   Vertical center both glyphs align on.
   */
  const centerY = FAVICON_SIZE / 2;

  /**
   Top y of the w's peaks.
   */
  const wTop = centerY - (wStickDy / 2);

  /**
   Bottom y of the w's valleys.
   */
  const wBottom = centerY + (wStickDy / 2);

  /**
   The w's four sticks: vertices alternate top, bottom, top, bottom,
   top from left to right.
   */
  const wSegments = Array.from(
    { length: W_STICK_COUNT, },
    function wStick(
      _unused,
      index,
    ): Segment {
      /**
       Whether this stick starts at a peak (even index) or a valley.
       */
      const startsAtTop = (index % 2) === 0;

      return {
        ax: startX + (wStickDx * index),
        ay: startsAtTop ? wTop : wBottom,
        bx: startX + (wStickDx * (index + 1)),
        by: startsAtTop ? wBottom : wTop,
      };
    },
  );

  /**
   X of the chevron's left-pointing apex.
   */
  const apexX = (startX + wWidth) + glyphGap;

  return [
    ...wSegments,
    {
      ax: apexX,
      ay: centerY,
      bx: apexX + chevronDx,
      by: centerY - chevronDy,
    },
    {
      ax: apexX,
      ay: centerY,
      bx: apexX + chevronDx,
      by: centerY + chevronDy,
    },
  ];
}

/**
 Renders the wordmark as a standalone SVG document: a ground rect
 under a single round-capped path holding all six stick spines as
 move/line subpaths (round caps match the round stick ends the
 geometry promises).
 
 @returns SVG markup string
 
 @example
 ```ts
 const href = `data:image/svg+xml;base64,${
   Buffer.from(renderFaviconSvg(),).toString('base64',)
 }`;
 ```
 */
export function renderFaviconSvg(): string {
  /**
   Path data: one `M`/`L` subpath per stick spine.
   */
  const pathData = buildGlyphSegments()
    .map(function segmentToSubpath(segment,): string {
      return `M ${svgNum(segment.ax,)} ${svgNum(segment.ay,)}`
        + ` L ${svgNum(segment.bx,)} ${svgNum(segment.by,)}`;
    },)
    .join(' ',);

  /**
   Shared stroke width of every stick.
   */
  const strokeWidth = svgNum(STICK_HALF_WIDTH * 2,);

  /**
   Root element open tag with an explicit raster size so sharp
   rasterizes at exactly {@link FAVICON_SIZE}.
   */
  const svgOpen = `<svg xmlns="http://www.w3.org/2000/svg"`
    + ` width="${FAVICON_SIZE}" height="${FAVICON_SIZE}"`
    + ` viewBox="0 0 ${FAVICON_SIZE} ${FAVICON_SIZE}">`;

  /**
   Ground rect covering the whole icon.
   */
  const ground = `<rect width="${FAVICON_SIZE}" height="${FAVICON_SIZE}"`
    + ` fill="${lightnessToHexGray(GROUND_LIGHTNESS,)}"/>`;

  /**
   The six sticks as one stroked path.
   */
  const sticks = `<path d="${pathData}" fill="none"`
    + ` stroke="${lightnessToHexGray(INK_LIGHTNESS,)}"`
    + ` stroke-width="${strokeWidth}" stroke-linecap="round"/>`;

  return `${svgOpen}${ground}${sticks}</svg>`;
}

/**
 Rasterizes the SVG wordmark ({@link renderFaviconSvg}) to a PNG
 through sharp, base64-encoded for inlining as a data URI.
 
 @returns base64 PNG bytes
 
 @example
 ```ts
 const href = `data:image/png;base64,${await renderFaviconPngBase64()}`;
 ```
 */
export async function renderFaviconPngBase64(): Promise<string> {
  /**
   SVG bytes handed to sharp's rasterizer.
   */
  const svg = Buffer.from(renderFaviconSvg(),);

  /**
   Complete favicon PNG bytes.
   */
  const png = await sharp(svg,)
    .png()
    .toBuffer();

  return png.toString('base64',);
}
