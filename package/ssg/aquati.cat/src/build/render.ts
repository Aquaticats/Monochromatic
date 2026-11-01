/**
 SVG-to-PNG rendering helpers for favicon generation.
 
 Provides functions for rendering SVG source to PNG at various sizes,
 with optional background padding for platform-specific icon formats.
 */
import sharp from 'sharp';

/**
 Path to the SVG favicon source file.
 */
const SVG_SOURCE = 'public/favicon.svg';

/**
 Background color for apple-touch-icon and maskable icon (dark purple).
 */
const BACKGROUND = {
  r: 45,
  g: 27,
  b: 78,
  alpha: 1,
};

/**
 SVG render density for high-quality rasterization.
 */
const RENDER_DENSITY = 384;

/**
 Renders the source SVG to PNG at the specified square dimensions.
 
 @param size - target width and height in pixels
 
 @returns PNG buffer
 
 @example
 ```ts
 const png = await renderPng({ size: 32 });
 ```
 */
export function renderPng({ size, }: { readonly size: number; },): Promise<Buffer> {
  return sharp(
    SVG_SOURCE,
    { density: RENDER_DENSITY, },
  )
    .resize(
    size,
    size,
    )
    .png()
    .toBuffer();
}

/**
 Renders the source SVG centered on a padded background canvas.
 
 @param contentSize - SVG render size for inner content
 
 @param canvasSize - final output dimensions
 
 @returns PNG buffer with content centered on colored background
 
 @example
 ```ts
 const png = await renderPadded({ contentSize: 140, canvasSize: 180 });
 ```
 */
export async function renderPadded(
  {
    contentSize,
    canvasSize,
  }: {
    readonly contentSize: number;
    readonly canvasSize: number;
  },
): Promise<Buffer> {
  /**
   Inner glyph rendered first so the padded outer canvas can centre-composite it.
   */
  const content = await renderPng({ size: contentSize, },);
  return sharp({
    create: {
      width: canvasSize,
      height: canvasSize,
      channels: 4,
      background: BACKGROUND,
    },
  },)
    .composite([{
      input: content,
      gravity: 'centre',
    },],)
    .png()
    .toBuffer();
}
