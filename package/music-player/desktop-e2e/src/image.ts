/**
 Screenshots as pixels: loading a PNG through ImageMagick and comparing two screenshots.
 
 @example
 ```ts
 import { diffBox, loadImage, } from './image.ts';
 
 const delta = diffBox({ a: await loadImage(first,), b: await loadImage(second,), },);
 ```
 
 @packageDocumentation
 */

import {
  readFile,
  rm,
} from 'node:fs/promises';

import { runCommand, } from './command.ts';
import { expect, } from './errors.ts';

/**
 Bytes per pixel in the raw RGB data that ImageMagick writes.
 */
const CHANNELS = 3;

/**
 A decoded screenshot: width, height and RGB bytes row by row, three bytes per pixel.
 */
export type Image = {
  readonly data: Uint8Array;
  readonly height: number;
  readonly width: number;
};

/**
 A rectangle of pixels.
 */
export type Box = {
  readonly height: number;
  readonly width: number;
  readonly x: number;
  readonly y: number;
};

/**
 How two screenshots differ: the number of differing pixels and, when there are any, their bounding box.
 */
export type Difference = {
  readonly box?: Box;
  readonly count: number;
};

/**
 Reads the size ImageMagick printed for an image.
 
 @param text - output of `magick identify -format %wx%h`, such as `1000x800`
 
 @returns width and height
 
 @throws CheckFailureError when the text is not two numbers joined by `x`
 
 @example
 ```ts
 parseSize('1000x800',); // { width: 1000, height: 800 }
 ```
 */
export function parseSize(text: string,): {
  readonly height: number;
  readonly width: number
} {
  /**
   The two parts around the `x`.
   */
  const [widthText = '', heightText = '',] = text.trim()
    .split('x',);

  /**
   Width as a number.
   */
  const width = Number(widthText,);

  /**
   Height as a number.
   */
  const height = Number(heightText,);
  expect({
    condition: (widthText !== '') && (heightText !== '')
      && Number.isInteger(width,)
      && Number.isInteger(height,),
    message: `the image size "${text}" is not WIDTHxHEIGHT`,
  },);
  return {
    width,
    height,
  };
}

/**
 Loads a PNG as raw RGB pixels. ImageMagick writes the pixels to a file next to the PNG, which is removed again.
 
 @param png - path of the screenshot
 
 @returns the decoded image
 
 @throws CheckFailureError when ImageMagick fails
 
 @example
 ```ts
 const image = await loadImage('/work/shots/wide-1-plain.png',);
 ```
 */
export async function loadImage(png: string,): Promise<Image> {
  /**
   Size line of the image.
   */
  const identified = await runCommand({
    command: 'magick',
    args: [
      'identify',
      '-format',
      '%wx%h',
      png,
    ],
  },);
  expect({
    condition: identified.status === 0,
    message: `magick identify failed: ${identified.stderr}`,
  },);

  /**
   File the raw pixels go to.
   */
  const rawFile = `${png}.rgb`;

  /**
   Result of the conversion.
   */
  const converted = await runCommand({
    command: 'magick',
    args: [
      png,
      '-alpha',
      'off',
      '-depth',
      '8',
      `rgb:${rawFile}`,
    ],
  },);
  expect({
    condition: converted.status === 0,
    message: `magick convert failed: ${converted.stderr}`,
  },);

  /**
   The raw pixels.
   */
  const data = await readFile(rawFile,);
  await rm(
    rawFile,
    { force: true, },
  );
  return {
    ...parseSize(identified.stdout,),
    data,
  };
}

/**
 Offset of a pixel in the raw data.
 
 @param image - the image
 
 @param x - column
 
 @param y - row
 
 @returns index of the pixel's first byte
 
 @example
 ```ts
 pixelOffset({ image, x: 0, y: 1, },);
 ```
 */
function pixelOffset(
  {
    image,
    x,
    y,
  }: {
    readonly image: Image;
    readonly x: number;
    readonly y: number;
  },
): number {
  return ((y * image.width) + x) * CHANNELS;
}

/**
 Tells whether the pixel at a position differs between two images.
 
 @param a - first image
 
 @param b - second image
 
 @param x - column
 
 @param y - row
 
 @returns whether any channel differs
 
 @example
 ```ts
 pixelDiffers({ a, b, x: 3, y: 4, },);
 ```
 */
function pixelDiffers(
  {
    a,
    b,
    x,
    y,
  }: {
    readonly a: Image;
    readonly b: Image;
    readonly x: number;
    readonly y: number;
  },
): boolean {
  /**
   Where the pixel starts in the first image.
   */
  const offsetA = pixelOffset({
    image: a,
    x,
    y,
  },);

  /**
   Where the pixel starts in the second image.
   */
  const offsetB = pixelOffset({
    image: b,
    x,
    y,
  },);
  return (a.data[offsetA] !== b.data[offsetB])
    || (a.data[offsetA + 1] !== b.data[offsetB + 1])
    || (a.data[offsetA + 2] !== b.data[offsetB + 2]);
}

/**
 Counts the pixels that differ between two images and finds their bounding box. Only the area both images share is
 compared.
 
 @param a - first image
 
 @param b - second image
 
 @returns the count, and the bounding box when the count is not zero
 
 @example
 ```ts
 const { count, box, } = diffBox({ a, b, },);
 ```
 */
export function diffBox(
  {
    a,
    b,
  }: {
    readonly a: Image;
    readonly b: Image;
  },
): Difference {
  /**
   Width of the shared area.
   */
  const width = Math.min(
    a.width,
    b.width,
  );

  /**
   Height of the shared area.
   */
  const height = Math.min(
    a.height,
    b.height,
  );

  /**
   Running count and bounds of the differing pixels, in a one-slot holder.
   */
  const found = {
    count: 0,
    minX: Infinity,
    minY: Infinity,
    maxX: -1,
    maxY: -1,
  };
  for (let y = 0; y < height; y++)
    for (let x = 0; x < width; x++)
      if (pixelDiffers({
        a,
        b,
        x,
        y,
      },)) {
        found.count++;
        found.minX = Math.min(
          found.minX,
          x,
        );
        found.minY = Math.min(
          found.minY,
          y,
        );
        found.maxX = Math.max(
          found.maxX,
          x,
        );
        found.maxY = Math.max(
          found.maxY,
          y,
        );
      }
  if (found.maxX < 0)
    return { count: found.count, };
  return {
    count: found.count,
    box: {
      x: found.minX,
      y: found.minY,
      width: (found.maxX - found.minX) + 1,
      height: (found.maxY - found.minY) + 1,
    },
  };
}

/**
 A color as red, green and blue bytes.
 */
export type Color = readonly [
  red: number,
  green: number,
  blue: number,
];

/**
 Reads the color of one pixel.

 @param image - the image
 
 @param x - column
 
 @param y - row
 
 @returns the pixel's red, green and blue bytes
 
 @example
 ```ts
 colorAt({ image, x: 10, y: 20, },); // [69, 84, 129]
 ```
 */
export function colorAt(
  {
    image,
    x,
    y,
  }: {
    readonly image: Image;
    readonly x: number;
    readonly y: number;
  },
): Color {
  /**
   Where the pixel starts in the raw data.
   */
  const offset = pixelOffset({
    image,
    x,
    y,
  },);
  return [
    image.data[offset] ?? 0,
    image.data[offset + 1] ?? 0,
    image.data[offset + 2] ?? 0,
  ];
}

/**
 Counts the pixels of a rectangle that differ between two images.
 
 @param a - first image
 
 @param b - second image
 
 @param box - the rectangle to compare, which both images must contain
 
 @returns how many pixels of the rectangle differ
 
 @example
 ```ts
 differencesWithin({ a, b, box: { x: 10, y: 10, width: 30, height: 20, }, },);
 ```
 */
export function differencesWithin(
  {
    a,
    b,
    box,
  }: {
    readonly a: Image;
    readonly b: Image;
    readonly box: Box;
  },
): number {
  /**
   Running count in a one-slot holder.
   */
  const found = { count: 0, };

  /**
   The rectangle's edges.
   */
  const {
    x: left,
    y: top,
    width,
    height,
  } = box;
  for (let y = top; y < (top + height); y++)
    for (let x = left; x < (left + width); x++)
      if (pixelDiffers({
        a,
        b,
        x,
        y,
      },))
        found.count++;
  return found.count;
}

/**
 Tells whether two colors are the same in every channel.
 
 @param a - first color
 
 @param b - second color
 
 @returns whether red, green and blue are all equal
 
 @example
 ```ts
 sameColor({ a: [1, 2, 3,], b: [1, 2, 3,], },); // true
 ```
 */
export function sameColor(
  {
    a,
    b,
  }: {
    readonly a: Color;
    readonly b: Color;
  },
): boolean {
  return (a[0] === b[0])
    && (a[1] === b[1])
    && (a[2] === b[2]);
}

/**
 Counts the pixels of a rectangle that are exactly one color.
 
 @param image - the image
 
 @param box - the rectangle to count in
 
 @param color - the color to count
 
 @returns how many pixels of the rectangle have exactly that color
 
 @example
 ```ts
 countColor({ image, box, color: [69, 84, 129], },);
 ```
 */
export function countColor(
  {
    image,
    box,
    color,
  }: {
    readonly box: Box;
    readonly color: Color;
    readonly image: Image;
  },
): number {
  /**
   Running count in a one-slot holder.
   */
  const found = { count: 0, };

  /**
   The rectangle's edges.
   */
  const {
    x: left,
    y: top,
    width,
    height,
  } = box;
  for (let y = top; y < (top + height); y++)
    for (let x = left; x < (left + width); x++)
      if (sameColor({
        a: colorAt({
          image,
          x,
          y,
        },),
        b: color,
      },))
        found.count++;
  return found.count;
}

/**
 Finds the bounding box of the pixels that differ between two images inside a rectangle, and counts them.
 
 @param a - first image
 
 @param b - second image
 
 @param area - the rectangle to look in, which both images must contain
 
 @returns the count, and the bounding box when the count is not zero
 
 @example
 ```ts
 const { count, box, } = diffBoxWithin({ a, b, area: { x: 0, y: 0, width: 100, height: 60, }, },);
 ```
 */
export function diffBoxWithin(
  {
    a,
    b,
    area,
  }: {
    readonly a: Image;
    readonly area: Box;
    readonly b: Image;
  },
): Difference {
  /**
   Running count and bounds in a one-slot holder.
   */
  const found = {
    count: 0,
    minX: Infinity,
    minY: Infinity,
    maxX: -1,
    maxY: -1,
  };

  /**
   The rectangle's edges.
   */
  const {
    x: left,
    y: top,
    width,
    height,
  } = area;
  for (let y = top; y < (top + height); y++)
    for (let x = left; x < (left + width); x++)
      if (pixelDiffers({
        a,
        b,
        x,
        y,
      },)) {
        found.count++;
        found.minX = Math.min(
          found.minX,
          x,
        );
        found.minY = Math.min(
          found.minY,
          y,
        );
        found.maxX = Math.max(
          found.maxX,
          x,
        );
        found.maxY = Math.max(
          found.maxY,
          y,
        );
      }
  if (found.maxX < 0)
    return { count: found.count, };
  return {
    count: found.count,
    box: {
      x: found.minX,
      y: found.minY,
      width: (found.maxX - found.minX) + 1,
      height: (found.maxY - found.minY) + 1,
    },
  };
}
