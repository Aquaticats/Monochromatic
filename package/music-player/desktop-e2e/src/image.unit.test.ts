/**
 Tests for the screenshot helpers: size parsing, the pixel difference, and loading a real PNG through ImageMagick.
 
 @module
 */

import {
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runCommand, } from './command.ts';
import {
  colorAt,
  countColor,
  diffBox,
  diffBoxWithin,
  differencesWithin,
  type Image,
  loadImage,
  parseSize,
  sameColor,
} from './image.ts';

/**
 Builds a solid image.
 
 @param width - width in pixels
 
 @param height - height in pixels
 
 @param fill - byte value of every channel
 
 @returns the image
 */
function solid(width: number, height: number, fill: number,): Image {
  return { width, height, data: new Uint8Array(width * height * 3,).fill(fill,), };
}

/**
 Copies an image with one pixel changed.
 
 @param image - source
 
 @param x - column
 
 @param y - row
 
 @returns the changed copy
 */
function withPixel(image: Image, x: number, y: number,): Image {
  const data = new Uint8Array(image.data,);
  data[(((y * image.width) + x) * 3) + 1] = 200;
  return { ...image, data, };
}

await describe({
  name: parseSize.name,
  children: [
    it({
      name: 'reads WIDTHxHEIGHT with surrounding white space',
      fn: async ({ expect, }) => {
        expect(parseSize('1000x800',),).toEqual({ width: 1_000, height: 800, },);
        expect(parseSize(' 3x4\n',),).toEqual({ width: 3, height: 4, },);
      },
    },),
    it({
      name: 'rejects text that is not two integers joined by x',
      fn: async ({ expect, }) => {
        for (const text of ['', '10', 'x5', '5x', 'axb', '1.5x2',])
          expect(() => parseSize(text,),).toThrow('is not WIDTHxHEIGHT',);
      },
    },),
  ],
},);

await describe({
  name: diffBox.name,
  children: [
    it({
      name: 'counts no difference between equal images and gives no box',
      fn: async ({ expect, }) => {
        expect(diffBox({ a: solid(4, 3, 9,), b: solid(4, 3, 9,), },),).toEqual({ count: 0, },);
      },
    },),
    it({
      name: 'counts one changed pixel with a one-pixel box',
      fn: async ({ expect, }) => {
        const a = solid(4, 3, 9,);
        expect(diffBox({ a, b: withPixel(a, 2, 1,), },),).toEqual({ count: 1, box: { x: 2, y: 1, width: 1, height: 1, }, },);
      },
    },),
    it({
      name: 'bounds several changed pixels',
      fn: async ({ expect, }) => {
        const a = solid(6, 5, 0,);
        const b = withPixel(withPixel(a, 1, 1,), 4, 3,);
        expect(diffBox({ a, b, },),).toEqual({ count: 2, box: { x: 1, y: 1, width: 4, height: 3, }, },);
      },
    },),
    it({
      name: 'compares only the area both images share',
      fn: async ({ expect, }) => {
        const result = diffBox({ a: solid(4, 4, 1,), b: solid(2, 2, 2,), },);
        expect(result.count,).toBe(4,);
        expect(result.box,).toEqual({ x: 0, y: 0, width: 2, height: 2, },);
      },
    },),
    it({
      name: 'sees a change in each of the three channels',
      fn: async ({ expect, }) => {
        const a = solid(2, 1, 0,);
        for (const channel of [0, 1, 2,]) {
          const data = new Uint8Array(a.data,);
          data[channel] = 1;
          expect(diffBox({ a, b: { ...a, data, }, },).count,).toBe(1,);
        }
      },
    },),
  ],
},);

await describe({
  name: diffBoxWithin.name,
  children: [
    it({
      name: 'ignores a difference outside the area and bounds the ones inside it',
      fn: async ({ expect, }) => {
        const a = solid(8, 6, 0,);
        const b = withPixel(withPixel(withPixel(a, 1, 1,), 3, 2,), 7, 5,);
        expect(diffBoxWithin({ a, b, area: { x: 0, y: 0, width: 5, height: 4, }, },),).toEqual({ count: 2, box: { x: 1, y: 1, width: 3, height: 2, }, },);
        expect(diffBoxWithin({ a, b, area: { x: 4, y: 0, width: 3, height: 4, }, },),).toEqual({ count: 0, },);
      },
    },),
  ],
},);

await describe({
  name: sameColor.name,
  children: [
    it({
      name: 'is true only when all three channels are equal',
      fn: async ({ expect, }) => {
        expect(sameColor({ a: [1, 2, 3,], b: [1, 2, 3,], },),).toBe(true,);
        for (const other of [[0, 2, 3,], [1, 0, 3,], [1, 2, 0,],] as const)
          expect(sameColor({ a: [1, 2, 3,], b: other, },),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: colorAt.name,
  children: [
    it({
      name: 'reads the three channels of one pixel',
      fn: async ({ expect, }) => {
        const image = withPixel(solid(3, 2, 7,), 1, 1,);
        expect(colorAt({ image, x: 1, y: 1, },),).toEqual([7, 200, 7,],);
        expect(colorAt({ image, x: 0, y: 0, },),).toEqual([7, 7, 7,],);
      },
    },),
  ],
},);

await describe({
  name: differencesWithin.name,
  children: [
    it({
      name: 'counts the differing pixels inside the rectangle only',
      fn: async ({ expect, }) => {
        const a = solid(6, 5, 0,);
        const b = withPixel(withPixel(a, 1, 1,), 5, 4,);
        expect(differencesWithin({ a, b, box: { x: 0, y: 0, width: 3, height: 3, }, },),).toBe(1,);
        expect(differencesWithin({ a, b, box: { x: 0, y: 0, width: 6, height: 5, }, },),).toBe(2,);
        expect(differencesWithin({ a, b, box: { x: 2, y: 0, width: 3, height: 3, }, },),).toBe(0,);
      },
    },),
  ],
},);

await describe({
  name: countColor.name,
  children: [
    it({
      name: 'counts the pixels that are exactly the color, inside the rectangle only',
      fn: async ({ expect, }) => {
        const image = withPixel(solid(4, 4, 7,), 2, 2,);
        expect(countColor({ image, box: { x: 0, y: 0, width: 4, height: 4, }, color: [7, 7, 7,], },),).toBe(15,);
        expect(countColor({ image, box: { x: 0, y: 0, width: 2, height: 2, }, color: [7, 7, 7,], },),).toBe(4,);
        expect(countColor({ image, box: { x: 0, y: 0, width: 4, height: 4, }, color: [7, 200, 7,], },),).toBe(1,);
      },
    },),
  ],
},);

await describe({
  name: loadImage.name,
  children: [
    it({
      name: 'decodes a PNG made by ImageMagick into its size and RGB bytes, and removes its scratch file',
      fn: async ({ expect, }) => {
        const dir = await mkdtemp(join(scratchRoot(), 'e2e-image-',),);
        const png = join(dir, 'red.png',);
        const made = await runCommand({ command: 'magick', args: ['-size', '4x3', 'xc:#FF0000', png,], },);
        expect(made.status,).toBe(0,);
        const image = await loadImage(png,);
        expect(image.width,).toBe(4,);
        expect(image.height,).toBe(3,);
        expect(image.data.length,).toBe(4 * 3 * 3,);
        expect([...image.data.subarray(0, 3,),],).toEqual([255, 0, 0,],);
        const listing = await runCommand({ command: 'ls', args: [dir,], },);
        expect(listing.stdout,).toBe('red.png',);
        await rm(dir, { recursive: true, force: true, },);
      },
      timeout: 30_000,
    },),
    it({
      name: 'rejects a file that is not an image',
      fn: async ({ expect, }) => {
        let caught: unknown;
        try {
          await loadImage(join(scratchRoot(), 'e2e-no-such-image.png',),);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect((caught as Error).message,).toContain('magick identify failed',);
      },
      timeout: 30_000,
    },),
  ],
},);
