/**
 Tests for the build-time favicon: the SVG `w<` wordmark and its
 sharp-rasterized PNG form.
 
 @module
 */

import { Buffer, } from 'node:buffer';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import sharp from 'sharp';

import {
  FAVICON_SIZE,
  renderFaviconPngBase64,
  renderFaviconSvg,
} from './favicon.ts';

/**
 Stick count of the whole wordmark: the w's four strokes plus the
 chevron's two.
 */
const STICK_COUNT = 6;

/**
 Grayscale byte every ground (background) sample must stay at or
 below: `oklch(0.1 0 0)` is nearly black in sRGB.
 */
const GROUND_BYTE_MAX = 10;

/**
 Grayscale byte the ink must reach: `oklch(0.9 0 0)` is bright in
 sRGB.
 */
const INK_BYTE_MIN = 200;

/**
 Per-channel wiggle allowed on an "achromatic" pixel: rasterizer
 rounding may land antialiased channels one step apart.
 */
const ACHROMATIC_TOLERANCE = 1;

/**
 Decoded favicon raster: dimensions plus each pixel's gray value.
 */
type DecodedFavicon = Readonly<{
  /**
   Raster width in pixels.
   */
  width: number;
  /**
   Raster height in pixels.
   */
  height: number;
  /**
   Gray value per pixel, rows top to bottom.
   */
  grays: readonly number[];
}>;

/**
 Reads the gray byte of one hex color inside the SVG markup, right
 after a paint-attribute marker like `fill="#`.
 
 @param svg - SVG markup searched
 
 @param marker - attribute prefix directly before the hex digits
 
 @returns first channel byte of the hex color
 
 @throws Error when the marker is absent
 */
function channelByteAfter(
  {
    svg,
    marker,
  }: Readonly<{
    svg: string;
    marker: string;
  }>,
): number {
  if (!svg.includes(marker,)) {
    throw new Error(`marker ${marker} not found in SVG`,);
  }

  /**
   Index of the marker inside the markup.
   */
  const markerIndex = svg.indexOf(marker,);

  /**
   First two hex digits after the marker.
   */
  const hex = svg.slice(
    markerIndex + marker.length,
    markerIndex + marker.length + 2,
  );

  return Number.parseInt(
    hex,
    16,
  );
}

/**
 Renders the favicon PNG and decodes it back to raw pixels through
 sharp, asserting each pixel is achromatic (all channels within
 {@link ACHROMATIC_TOLERANCE} of the red channel) on the way.
 
 @returns decoded dimensions and per-pixel grays
 */
async function decodeFaviconGrays(): Promise<DecodedFavicon> {
  /**
   Rendered favicon PNG bytes.
   */
  const png = Buffer.from(
    await renderFaviconPngBase64(),
    'base64',
  );

  /**
   Raw-pixel decoder for the rendered PNG.
   */
  const decoder = sharp(png,)
    .raw();

  /**
   Decoded pixel bytes plus raster metadata.
   */
  const { data, info, } = await decoder.toBuffer({ resolveWithObject: true, },);

  /**
   Gray value of every pixel; collecting them asserts each pixel is
   achromatic on the way.
   */
  const grays = Array.from(
    { length: info.width * info.height, },
    function grayOfPixel(
      _unused,
      index,
    ): number {
      /**
       Offset of this pixel's R byte.
       */
      const offset = index * info.channels;

      /**
       Red channel, the gray value under test.
       */
      const gray = data.readUInt8(offset,);

      /**
       Green channel, compared against the red.
       */
      const green = data.readUInt8(offset + 1,);

      /**
       Blue channel, compared against the red.
       */
      const blue = data.readUInt8(offset + 2,);

      expect(Math.abs(green - gray,),).toBeLessThanOrEqual(
        ACHROMATIC_TOLERANCE,
      );
      expect(Math.abs(blue - gray,),).toBeLessThanOrEqual(
        ACHROMATIC_TOLERANCE,
      );

      return gray;
    },
  );

  return {
    width: info.width,
    height: info.height,
    grays,
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: renderFaviconSvg.name,
      children: [
        it({
          name: 'declares a FAVICON_SIZE-square viewport',
          fn: async function declaresViewport(): Promise<void> {
            /**
             Rendered SVG markup.
             */
            const svg = renderFaviconSvg();

            expect(svg,).toContain('<svg xmlns="http://www.w3.org/2000/svg"',);
            expect(svg,).toContain(`width="${FAVICON_SIZE}"`,);
            expect(svg,).toContain(`height="${FAVICON_SIZE}"`,);
            expect(svg,).toContain(
              `viewBox="0 0 ${FAVICON_SIZE} ${FAVICON_SIZE}"`,
            );
          },
        },),
        it({
          name: 'draws six round-capped sticks of one shared width',
          fn: async function drawsSticks(): Promise<void> {
            /**
             Rendered SVG markup.
             */
            const svg = renderFaviconSvg();

            /**
             Move commands inside the path: one per stick spine.
             */
            const moveCommands = svg.split('M ',).length - 1;

            expect(moveCommands,).toBe(STICK_COUNT,);
            expect(svg,).toContain('stroke-linecap="round"',);
            expect(svg,).toContain('stroke-width="4.4"',);
          },
        },),
        it({
          name: 'inks near-white on a near-black ground',
          fn: async function paintsPaletteStops(): Promise<void> {
            /**
             Rendered SVG markup.
             */
            const svg = renderFaviconSvg();

            expect(channelByteAfter(
              {
                svg,
                marker: 'fill="#',
              },
            ),).toBeLessThanOrEqual(GROUND_BYTE_MAX,);
            expect(channelByteAfter(
              {
                svg,
                marker: 'stroke="#',
              },
            ),).toBeGreaterThanOrEqual(INK_BYTE_MIN,);
          },
        },),
        it({
          name: 'is deterministic across calls',
          fn: async function svgIsDeterministic(): Promise<void> {
            expect(renderFaviconSvg(),).toBe(renderFaviconSvg(),);
          },
        },),
      ],
    },),
    describe({
      name: renderFaviconPngBase64.name,
      children: [
        it({
          name: 'rasterizes to a FAVICON_SIZE-square image',
          fn: async function rastersSquarePng(): Promise<void> {
            /**
             Decoded favicon raster.
             */
            const decoded = await decodeFaviconGrays();

            expect(decoded.width,).toBe(FAVICON_SIZE,);
            expect(decoded.height,).toBe(FAVICON_SIZE,);
          },
        },),
        it({
          name: 'draws achromatic ink on an achromatic near-black ground',
          fn: async function drawsGrayscaleWordmark(): Promise<void> {
            /**
             Decoded favicon raster; decoding already asserted every
             pixel is achromatic.
             */
            const decoded = await decodeFaviconGrays();

            /**
             Corner pixel: bare ground, since the wordmark is
             centered.
             */
            const [corner,] = decoded.grays;

            expect(corner,).toBeLessThanOrEqual(GROUND_BYTE_MAX,);
            expect(Math.max(...decoded.grays,),).toBeGreaterThanOrEqual(
              INK_BYTE_MIN,
            );
          },
        },),
        it({
          name: 'is deterministic across calls',
          fn: async function pngIsDeterministic(): Promise<void> {
            expect(await renderFaviconPngBase64(),).toBe(
              await renderFaviconPngBase64(),
            );
          },
        },),
      ],
    },),
  ],
},);
