/**
 Tests the picture size read off a file header.

 The headers are built by hand: a lossy, a lossless and an extended WebP, a
 PNG and a JPEG with a thumbnail segment before its frame header, plus
 files no reader here understands. Fixtures are cat-themed invention: headers
 and padding, no pixels.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  imagePixelSizeOf,
} from '../dist/final/node/index.mjs';
import {
  lossy,
  webpOf,
} from './picture-headers.test-fixture.ts';

await describe({
  name: imagePixelSizeOf.name,
  children: [
    it({
      name: 'READS A LOSSY WEBP, the coding of the corpus\'s pictures',
      fn: async () => {
        await Promise.resolve();
        expect(imagePixelSizeOf({ bytes: lossy({ width: 2_400, height: 2_616, }), },),).toEqual({
          kind: 'sized',
          width: 2_400,
          height: 2_616,
        },);
      },
    },),

    it({
      name: 'READS A LOSSLESS WEBP, whose size is stored less one in 14 bits each',
      fn: async () => {
        await Promise.resolve();
        /**
         Signature byte, then 14 bits of width less one and 14 of height less one.
         */
        const chunk = Buffer.alloc(40,);
        chunk[0] = 0x2F;
        chunk.writeUInt32LE((575) | (1_279 << 14), 1,);
        expect(imagePixelSizeOf({ bytes: webpOf({ fourcc: 'VP8L', chunk, },), },),).toEqual({
          kind: 'sized',
          width: 576,
          height: 1_280,
        },);
      },
    },),

    it({
      name: 'READS AN EXTENDED WEBP, whose canvas is stored less one in 24 bits each',
      fn: async () => {
        await Promise.resolve();
        /**
         Flags, reserved bytes, then the canvas.
         */
        const chunk = Buffer.alloc(40,);
        chunk.writeUIntLE(1_224, 4, 3,);
        chunk.writeUIntLE(532, 7, 3,);
        expect(imagePixelSizeOf({ bytes: webpOf({ fourcc: 'VP8X', chunk, },), },),).toEqual({
          kind: 'sized',
          width: 1_225,
          height: 533,
        },);
      },
    },),

    it({
      name: 'READS A PNG off its header chunk',
      fn: async () => {
        await Promise.resolve();
        /**
         Signature, header chunk length and name, then the size.
         */
        const bytes = Buffer.alloc(40,);
        Buffer.from([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,],).copy(bytes, 0,);
        bytes.writeUInt32BE(13, 8,);
        bytes.write('IHDR', 12,);
        bytes.writeUInt32BE(300, 16,);
        bytes.writeUInt32BE(200, 20,);
        expect(imagePixelSizeOf({ bytes, },),).toEqual({ kind: 'sized', width: 300, height: 200, },);
      },
    },),

    it({
      name: 'READS A JPEG PAST A SEGMENT BEFORE ITS FRAME HEADER, such as the metadata a camera writes first',
      fn: async () => {
        await Promise.resolve();
        /**
         Start of image, a 6-byte application segment, then the frame header.
         */
        const bytes = Buffer.alloc(40,);
        Buffer.from([0xFF, 0xD8, 0xFF, 0xE1, 0x00, 0x06, 0x01, 0x02, 0x03, 0x04, 0xFF, 0xC0, 0x00, 0x11, 0x08,],).copy(bytes, 0,);
        bytes.writeUInt16BE(1_280, 15,);
        bytes.writeUInt16BE(576, 17,);
        expect(imagePixelSizeOf({ bytes, },),).toEqual({ kind: 'sized', width: 576, height: 1_280, },);
      },
    },),

    it({
      name: 'REPORTS UNSIZED for a file in no format it reads, a header cut short, and a zero size',
      fn: async () => {
        await Promise.resolve();
        expect(imagePixelSizeOf({ bytes: Buffer.alloc(300, 7,), },),).toEqual({ kind: 'unsized', },);
        expect(imagePixelSizeOf({ bytes: Buffer.from('RIFF',), },),).toEqual({ kind: 'unsized', },);
        expect(imagePixelSizeOf({ bytes: Buffer.alloc(0,), },),).toEqual({ kind: 'unsized', },);
        expect(imagePixelSizeOf({ bytes: lossy({ width: 0, height: 10, }), },),).toEqual({ kind: 'unsized', },);
        expect(imagePixelSizeOf({ bytes: Buffer.from([0xFF, 0xD8, 0xFF, 0xC0,],), },),).toEqual({ kind: 'unsized', },);
      },
    },),
  ],
},);
