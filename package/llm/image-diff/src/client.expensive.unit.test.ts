import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { compareAll, } from './client.multi.compare.ts';
import { embedAll, } from './client.multi.ts';
import {
  compare,
  embed,
  embedBatch,
} from './client.ts';
import type { Provider, } from './types.ts';

/**
 Generate a minimal 1x1 PNG as a base64 data URI.
 The color parameter controls the RGBA pixel value to produce distinct images.
 
 @param red - red channel value (0-255)
 @param green - green channel value (0-255)
 @param blue - blue channel value (0-255)
 @returns 1x1 PNG as base64 data URI
 */
function makeMinimalPngDataUri({
  red,
  green,
  blue,
}: {
  red: number;
  green: number;
  blue: number;
},): string {
  /**
   Minimal 1x1 RGBA PNG built from raw bytes.
   Structure: PNG signature + IHDR + IDAT (zlib-compressed scanline) + IEND.
   */
  const pngSignature = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,];

  /** CRC32 lookup table, computed once. */
  const crcTable: number[] = Array.from(
    { length: 256, },
    function buildCrcEntry(_, n,) {
      return Array.from({ length: 8, },).reduce(
        function step(c: number,) {
          return c & 1 ? 0xED_B8_83_20 ^ (c >>> 1) : c >>> 1;
        },
        n,
      );
    },
  );

  /** Compute CRC32 of a byte array. */
  function crc32(bytes: number[],): number {
    const finalCrc = bytes.reduce(
      function updateCrc(crc, byte,) {
        return (crcTable[(crc ^ byte) & 0xFF] ?? 0) ^ (crc >>> 8);
      },
      0xFF_FF_FF_FF,
    );
    // oxlint-disable-next-line prefer-math-trunc -- final XOR; bitwise truncation is intentional for CRC32
    return Math.trunc((finalCrc ^ 0xFF_FF_FF_FF) >>> 0,);
  }

  /** Encode a 4-byte big-endian unsigned integer. */
  function uint32be(value: number,): number[] {
    return [
      (value >>> 24) & 0xFF,
      (value >>> 16) & 0xFF,
      (value >>> 8) & 0xFF,
      value & 0xFF,
    ];
  }

  /** Build a PNG chunk: length + type + data + CRC. */
  function makeChunk({
    type,
    data,
  }: {
    type: number[];
    data: number[];
  },): number[] {
    const typeAndData = [...type, ...data,];
    return [...uint32be(data.length,), ...typeAndData,
      ...uint32be(crc32(typeAndData,),),];
  }

  /** IHDR: 1x1, 8-bit RGBA */
  const ihdr = makeChunk({
    type: [0x49, 0x48, 0x44, 0x52,],
    data: [...uint32be(1,), ...uint32be(1,), 8, 6, 0, 0, 0,],
  },);

  /** Raw scanline: filter byte (0=None) + RGBA pixel. */
  const rawScanline = [0, red, green, blue, 255,];

  /** Adler-32 checksum of the raw scanline data. */
  const {
    adlerA,
    adlerB,
  } = rawScanline.reduce(
    function updateAdler(acc, byte,) {
      const nextA = (acc.adlerA + byte) % 65_521;
      return {
        adlerA: nextA,
        adlerB: (acc.adlerB + nextA) % 65_521,
      };
    },
    { adlerA: 1, adlerB: 0, },
  );
  // oxlint-disable-next-line prefer-math-trunc -- Adler-32 packing; bitwise truncation is intentional
  const adler32 = Math.trunc(((adlerB << 16) | adlerA) >>> 0,);

  /**
   Zlib wrapper around a single uncompressed deflate block.
   CMF=0x78 (deflate, 32K window), FLG=0x01 (check bits for CMF).
   BFINAL=1, BTYPE=00 (no compression), LEN=5, NLEN=~5.
   */
  const len = rawScanline.length;
  const nlen = len ^ 0xFF_FF;
  const deflateBlock = [
    0x78,
    0x01,
    0x01,
    len & 0xFF,
    (len >>> 8) & 0xFF,
    nlen & 0xFF,
    (nlen >>> 8) & 0xFF,
    ...rawScanline,
    ...uint32be(adler32,),
  ];

  const idat = makeChunk({
    type: [0x49, 0x44, 0x41, 0x54,],
    data: deflateBlock,
  },);
  const iend = makeChunk({
    type: [0x49, 0x45, 0x4E, 0x44,],
    data: [],
  },);

  const pngBytes = new Uint8Array([...pngSignature, ...ihdr, ...idat, ...iend,],);
  const binary = Array
    .from(
    pngBytes,
    function byteToChar(byte,) {
      return String.fromCodePoint(byte,);
    },
    )
    .join('',);
  return `data:image/png;base64,${btoa(binary,)}`;
}

await describe({
  name: '',
  children: [
    //region Single-provider tests (Voyage)

    describe({
      name: 'embed (voyage)',
      children: [
        it({
          name: 'returns an embedding vector from a minimal PNG',
          fn: async () => {
            const dataUri = makeMinimalPngDataUri({ red: 255, green: 0, blue: 0, },);
            const result = await embed({
              input: { base64: dataUri, },
              config: { provider: 'voyage', },
            },);

            expect(result.embedding.length,).toBeGreaterThan(0,);
            expect(result.usage.totalTokens,).toBeGreaterThan(0,);
          },
        },),
      ],
    },),
    describe({
      name: 'embedBatch (voyage)',
      children: [
        it({
          name: 'returns embeddings for multiple images',
          fn: async () => {
            const red = makeMinimalPngDataUri({ red: 255, green: 0, blue: 0, },);
            const blue = makeMinimalPngDataUri({ red: 0, green: 0, blue: 255, },);
            const result = await embedBatch({
              inputs: [{ base64: red, }, { base64: blue, },],
              config: { provider: 'voyage', },
            },);

            expect(result.embeddings.length,).toBe(2,);
            const [first, second,] = result.embeddings;
            if ((first === undefined) || (second === undefined))
              throw new Error('missing embeddings',);
            expect(first.length,).toBeGreaterThan(0,);
            expect(second.length,).toBeGreaterThan(0,);
          },
        },),
      ],
    },),
    describe({
      name: 'compare (voyage)',
      children: [
        it({
          name: 'identical images have similarity near 1',
          fn: async () => {
            const red = makeMinimalPngDataUri({ red: 255, green: 0, blue: 0, },);
            const result = await compare({
              imageA: { base64: red, },
              imageB: { base64: red, },
              config: { provider: 'voyage', },
            },);

            expect(result.similarity,).toBeGreaterThan(0.99,);
            expect(result.distance,).toBeLessThan(0.01,);
          },
        },),
        it({
          name: 'different-colored images have lower similarity',
          fn: async () => {
            const red = makeMinimalPngDataUri({ red: 255, green: 0, blue: 0, },);
            const blue = makeMinimalPngDataUri({ red: 0, green: 0, blue: 255, },);
            const result = await compare({
              imageA: { base64: red, },
              imageB: { base64: blue, },
              config: { provider: 'voyage', },
            },);

            expect(result.similarity,).toBeLessThan(1,);
            expect(result.distance,).toBeGreaterThan(0,);
            expect(result.embeddingA.length,).toBe(result.embeddingB.length,);
          },
        },),
      ],
    },),

    //endregion Single-provider tests (Voyage)

    //region Single-provider tests (Gemini)

    describe({
      name: 'embed (gemini)',
      children: [
        it({
          name: 'returns an embedding vector from a minimal PNG',
          fn: async () => {
            const dataUri = makeMinimalPngDataUri({ red: 255, green: 0, blue: 0, },);
            const result = await embed({
              input: { base64: dataUri, },
              config: { provider: 'gemini', },
            },);

            expect(result.embedding.length,).toBeGreaterThan(0,);
          },
        },),
      ],
    },),
    describe({
      name: 'embedBatch (gemini)',
      children: [
        it({
          name: 'returns embeddings for multiple images',
          fn: async () => {
            const red = makeMinimalPngDataUri({ red: 255, green: 0, blue: 0, },);
            const blue = makeMinimalPngDataUri({ red: 0, green: 0, blue: 255, },);
            const result = await embedBatch({
              inputs: [{ base64: red, }, { base64: blue, },],
              config: { provider: 'gemini', },
            },);

            expect(result.embeddings.length,).toBe(2,);
            const [first, second,] = result.embeddings;
            if ((first === undefined) || (second === undefined))
              throw new Error('missing embeddings',);
            expect(first.length,).toBeGreaterThan(0,);
            expect(second.length,).toBeGreaterThan(0,);
          },
        },),
      ],
    },),
    describe({
      name: 'compare (gemini)',
      children: [
        it({
          name: 'identical images have similarity near 1',
          fn: async () => {
            const red = makeMinimalPngDataUri({ red: 255, green: 0, blue: 0, },);
            const result = await compare({
              imageA: { base64: red, },
              imageB: { base64: red, },
              config: { provider: 'gemini', },
            },);

            expect(result.similarity,).toBeGreaterThan(0.99,);
            expect(result.distance,).toBeLessThan(0.01,);
          },
        },),
        it({
          name: 'different-colored images have lower similarity',
          fn: async () => {
            const red = makeMinimalPngDataUri({ red: 255, green: 0, blue: 0, },);
            const blue = makeMinimalPngDataUri({ red: 0, green: 0, blue: 255, },);
            const result = await compare({
              imageA: { base64: red, },
              imageB: { base64: blue, },
              config: { provider: 'gemini', },
            },);

            expect(result.similarity,).toBeLessThan(1,);
            expect(result.distance,).toBeGreaterThan(0,);
            expect(result.embeddingA.length,).toBe(result.embeddingB.length,);
          },
        },),
      ],
    },),

    //endregion Single-provider tests (Gemini)

    //region Multi-provider tests

    describe({
      name: compareAll.name,
      children: [
        it({
          name: 'returns results from both providers',
          fn: async () => {
            const red = makeMinimalPngDataUri({ red: 255, green: 0, blue: 0, },);
            const blue = makeMinimalPngDataUri({ red: 0, green: 0, blue: 255, },);
            const results = await compareAll({
              imageA: { base64: red, },
              imageB: { base64: blue, },
            },);

            expect(results.length,).toBe(2,);
            const providers = results.map(function getProvider(r,) {
              return r.provider;
            },);
            expect(providers,).toContain('voyage' as Provider,);
            expect(providers,).toContain('gemini' as Provider,);

            for (const entry of results) {
              expect(entry.result.similarity,).toBeLessThan(1,);
              expect(entry.result.embeddingA.length,).toBeGreaterThan(0,);
            }
          },
        },),
      ],
    },),
    describe({
      name: embedAll.name,
      children: [
        it({
          name: 'returns embeddings from both providers',
          fn: async () => {
            const red = makeMinimalPngDataUri({ red: 255, green: 0, blue: 0, },);
            const results = await embedAll({ base64: red, },);

            expect(results.length,).toBe(2,);
            for (const entry of results)
              expect(entry.result.embedding.length,).toBeGreaterThan(0,);
          },
        },),
      ],
    },),
    //endregion Multi-provider tests
  ],
},);
