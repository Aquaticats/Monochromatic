/**
 Browser tests for the built single-file page, exercising it the way
 an end user does: opened straight from disk over `file://`. The
 frequency bars are native `<progress>` elements whose fill each
 engine paints from vendor pseudo-elements (`::-moz-progress-bar`,
 `::-webkit-progress-value`), and computed styles cannot prove those
 painted (Chromium reports a near-white `accent-color` it then
 ignores), so the assertions decode an element screenshot and check
 the actual pixels: achromatic everywhere (an engine-default blue or
 green fill fails instantly) and mostly ink for a full-width bar.
 
 @module
 */

import type { Buffer, } from 'node:buffer';
import { join, } from 'node:path';
import process from 'node:process';

import {
  expect,
  test,
  type Page,
} from '@playwright/test';
import sharp from 'sharp';

/**
 Built page opened by every test; playwright's cwd is the repo root
 (the config directory), on host and in the podman container alike.
 */
const PAGE_URL = `file://${
  join(
    process.cwd(),
    'package/wc/app/dist/final/index.html',
  )
}`;

/**
 Sample text: "apple" repeats most, so the first frequency row's bar
 is at 100% of its own maximum and the screenshot is nearly all fill.
 */
const SAMPLE_TEXT = 'apple apple apple apple banana banana cherry cherry';

/**
 Largest per-channel skew a pixel may show and still count as
 achromatic. Blends of two grays stay gray, so a few steps of
 rasterizer rounding is all that's allowed; Firefox's default blue
 fill skews over 200 and Chromium's default green over 100.
 */
const ACHROMATIC_SKEW_MAX = 8;

/**
 Gray byte at or above which a dark-scheme pixel counts as ink
 (`--color-fg-strong` is the near-white stop there).
 */
const DARK_INK_MIN = 200;

/**
 Gray byte at or below which a light-scheme pixel counts as ink
 (`--color-fg-strong` is the near-black stop there).
 */
const LIGHT_INK_MAX = 60;

/**
 Fraction of the bar's pixels the fill must cover; the first row's
 bar is 100% full, so ink dominating the crop proves the fill
 painted (edges and rounding keep it below every pixel).
 */
const INK_FRACTION_MIN = 1 / 2;

/**
 One decoded screenshot pixel.
 */
type Pixel = Readonly<{
  /**
   Red channel byte.
   */
  r: number;
  /**
   Green channel byte.
   */
  g: number;
  /**
   Blue channel byte.
   */
  b: number;
}>;

/**
 Decodes a PNG screenshot into per-pixel channel triplets through
 sharp.
 
 @param png - screenshot bytes
 
 @returns decoded pixels, rows top to bottom
 */
async function decodePixels(png: Buffer,): Promise<readonly Pixel[]> {
  /**
   Raw-pixel decoder for the screenshot.
   */
  const decoder = sharp(png,)
    .raw();

  /**
   Decoded pixel bytes plus raster metadata.
   */
  const { data, info, } = await decoder.toBuffer({ resolveWithObject: true, },);

  return Array.from(
    { length: info.width * info.height, },
    function pixelAt(
      _unused,
      index,
    ): Pixel {
      /**
       Offset of this pixel's R byte.
       */
      const offset = index * info.channels;

      return {
        r: data.readUInt8(offset,),
        g: data.readUInt8(offset + 1,),
        b: data.readUInt8(offset + 2,),
      };
    },
  );
}

/**
 Largest channel deviation from red across one pixel; zero for a
 perfect gray.
 
 @param pixel - decoded pixel
 
 @returns per-channel skew in bytes
 */
function channelSkew(pixel: Pixel,): number {
  return Math.max(
    Math.abs(pixel.g - pixel.r,),
    Math.abs(pixel.b - pixel.r,),
  );
}

/**
 Opens the built page, types the sample text, and screenshots the
 first frequency row's bar once the debounced compute renders it.
 
 @param page - playwright page fixture
 
 @returns decoded pixels of the bar screenshot
 */
async function shootFirstBar(
  { page, }: Readonly<{ page: Page; }>,
): Promise<readonly Pixel[]> {
  /**
   Uncaught page errors and error-level console messages; the page
   must produce none, and asserting on the collected text surfaces
   the actual failure when the client script dies.
   */
  const pageErrors: string[] = [];

  page.on(
    'pageerror',
    function collectPageError(error,): void {
      pageErrors.push(String(error,),);
    },
  );
  page.on(
    'console',
    function collectConsoleError(message,): void {
      if (message.type() === 'error') {
        pageErrors.push(message.text(),);
      }
    },
  );

  await page.goto(PAGE_URL,);
  await page
    .locator('.wc-input',)
    .fill(SAMPLE_TEXT,);

  expect(pageErrors,).toEqual([],);

  /**
   First frequency row's native progress bar.
   */
  const bar = page
    .locator('.frequency-body .frequency-row .freq-bar',)
    .first();

  await expect(bar,).toBeVisible();

  return decodePixels(await bar.screenshot(),);
}

test.describe('frequency bar fill, dark scheme', () => {
  test.use({ colorScheme: 'dark', },);

  test('paints the near-white stop, not the engine default', async ({ page, },) => {
    /**
     Decoded pixels of the first bar.
     */
    const pixels = await shootFirstBar({ page, },);

    /**
     Worst per-channel skew across the crop; any engine-default
     colored fill pushes this into the hundreds.
     */
    const worstSkew = Math.max(
      ...pixels.map(function skewOf(pixel,): number {
        return channelSkew(pixel,);
      },),
    );

    /**
     Share of pixels at or above the dark-scheme ink stop.
     */
    const inkFraction = pixels
      .filter(function isInk(pixel,): boolean {
        return pixel.r >= DARK_INK_MIN;
      },)
      .length / pixels.length;

    expect(worstSkew,).toBeLessThanOrEqual(ACHROMATIC_SKEW_MAX,);
    expect(inkFraction,).toBeGreaterThanOrEqual(INK_FRACTION_MIN,);
  },);
},);

test.describe('frequency bar fill, light scheme', () => {
  test.use({ colorScheme: 'light', },);

  test('paints the near-black stop, not the engine default', async ({ page, },) => {
    /**
     Decoded pixels of the first bar.
     */
    const pixels = await shootFirstBar({ page, },);

    /**
     Worst per-channel skew across the crop.
     */
    const worstSkew = Math.max(
      ...pixels.map(function skewOf(pixel,): number {
        return channelSkew(pixel,);
      },),
    );

    /**
     Share of pixels at or below the light-scheme ink stop.
     */
    const inkFraction = pixels
      .filter(function isInk(pixel,): boolean {
        return pixel.r <= LIGHT_INK_MAX;
      },)
      .length / pixels.length;

    expect(worstSkew,).toBeLessThanOrEqual(ACHROMATIC_SKEW_MAX,);
    expect(inkFraction,).toBeGreaterThanOrEqual(INK_FRACTION_MIN,);
  },);
},);
