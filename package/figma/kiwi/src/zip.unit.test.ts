import {
  crc32,
  deflateRawSync,
} from 'node:zlib';
import { mkdtemp, writeFile } from 'node:fs/promises';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  extractZipEntries,
  parseFigmaFile,
} from '../dist/final/node/index.mjs';

/**
 ZIP local file header signature.
 */
const LOCAL_SIGNATURE = 0x04_03_4B_50;

/**
 ZIP central directory header signature.
 */
const CENTRAL_SIGNATURE = 0x02_01_4B_50;

/**
 ZIP end of central directory signature.
 */
const EOCD_SIGNATURE = 0x06_05_4B_50;

/**
 Byte length of a ZIP local file header before the entry name.
 */
const LOCAL_HEADER_LENGTH = 30;

/**
 Byte length of a ZIP central directory header before the entry name.
 */
const CENTRAL_HEADER_LENGTH = 46;

/**
 Byte length of a ZIP end of central directory record.
 */
const EOCD_LENGTH = 22;

/**
 Compression method of stored ZIP entries.
 */
const METHOD_STORED = 0;

/**
 Compression method of deflated ZIP entries.
 */
const METHOD_DEFLATE = 8;

/**
 Compression method this extractor does not implement.
 */
const METHOD_UNSUPPORTED = 99;

/**
 Byte offset of the little-endian format version in a canvas.fig header.
 */
const CANVAS_VERSION_OFFSET = 8;

/**
 Eight-byte ASCII prelude of a canvas.fig blob.
 */
const CANVAS_PRELUDE = 'fig-kiwi';

/**
 Four-byte little-endian format version written into the fixture canvas.fig header.
 */
const CANVAS_VERSION = 101;

/**
 Deflate-compressed empty Kiwi schema: a definition count of zero.
 */
const EMPTY_SCHEMA_BYTES = new Uint8Array([0,],);

/**
 One entry of a fixture ZIP archive.
 */
type ZipEntryFixture = {
  readonly compressedSizeOverride?: number;
  readonly content: Uint8Array;
  readonly method?: number;
  readonly name: string;
  readonly uncompressedSizeOverride?: number;
};

/**
 Byte offsets a fixture archive used, so a test can corrupt one field.
 */
type ZipLayout = {
  readonly archive: Uint8Array;
  readonly centralOffsets: readonly number[];
  readonly localOffsets: readonly number[];
};

/**
 Builds one ZIP local file record.
 
 @param entry - Entry descriptor.
 
 @param nameBytes - Encoded entry name.
 
 @param compressed - Compressed entry payload.
 
 @returns Local record bytes.
 */
function localRecord(
  {
    entry,
    nameBytes,
    compressed,
  }: {
    readonly compressed: Uint8Array;
    readonly entry: ZipEntryFixture;
    readonly nameBytes: Uint8Array;
  },
): Uint8Array {
  const record = new Uint8Array(LOCAL_HEADER_LENGTH + nameBytes.length + compressed.length,);
  const view = new DataView(record.buffer,);
  view.setUint32(0, LOCAL_SIGNATURE, true,);
  view.setUint32(14, crc32(entry.content,), true,);
  view.setUint32(18, compressed.length, true,);
  view.setUint32(22, entry.content.length, true,);
  view.setUint16(26, nameBytes.length, true,);
  record.set(nameBytes, LOCAL_HEADER_LENGTH,);
  record.set(compressed, LOCAL_HEADER_LENGTH + nameBytes.length,);
  return record;
}

/**
 Builds one ZIP central directory record.
 
 @param entry - Entry descriptor.
 
 @param nameBytes - Encoded entry name.
 
 @param compressedSize - Compressed payload length.
 
 @param localOffset - Byte offset of the matching local record.
 
 @returns Central record bytes.
 */
function centralRecord(
  {
    entry,
    nameBytes,
    compressedSize,
    localOffset,
  }: {
    readonly compressedSize: number;
    readonly entry: ZipEntryFixture;
    readonly localOffset: number;
    readonly nameBytes: Uint8Array;
  },
): Uint8Array {
  const record = new Uint8Array(CENTRAL_HEADER_LENGTH + nameBytes.length,);
  const view = new DataView(record.buffer,);
  view.setUint32(0, CENTRAL_SIGNATURE, true,);
  view.setUint16(10, entry.method ?? METHOD_STORED, true,);
  view.setUint32(16, crc32(entry.content,), true,);
  view.setUint32(20, compressedSize, true,);
  view.setUint32(24, entry.uncompressedSizeOverride ?? entry.content.length, true,);
  view.setUint16(28, nameBytes.length, true,);
  view.setUint32(42, localOffset, true,);
  record.set(nameBytes, CENTRAL_HEADER_LENGTH,);
  return record;
}

/**
 Builds a ZIP archive readable by the package's extractor.
 
 @param entries - Entry descriptors in archive order.
 
 @returns Archive bytes and the offsets of the records a test may corrupt.
 */
function zipArchive({ entries, }: { readonly entries: readonly ZipEntryFixture[]; },): ZipLayout {
  const nameBytes = entries.map(entry => new TextEncoder().encode(entry.name,),);
  const compressed = entries.map((entry,) => entry.method === METHOD_DEFLATE
    ? new Uint8Array(deflateRawSync(entry.content,),)
    : entry.content);
  const locals = entries.map((entry, index,) => localRecord({
    compressed: compressed[index] as Uint8Array,
    entry,
    nameBytes: nameBytes[index] as Uint8Array,
  },),);
  const localOffsets = recordOffsets(locals, 0,);
  const centrals = entries.map((entry, index,) => centralRecord({
    compressedSize: (compressed[index] as Uint8Array).length,
    entry,
    localOffset: localOffsets[index] as number,
    nameBytes: nameBytes[index] as Uint8Array,
  },),);
  const localBytes = concat(locals,);
  const centralBytes = concat(centrals,);
  const centralOffsets = recordOffsets(centrals, localBytes.length,);
  const eocd = new Uint8Array(EOCD_LENGTH,);
  const eocdView = new DataView(eocd.buffer,);
  eocdView.setUint32(0, EOCD_SIGNATURE, true,);
  eocdView.setUint16(10, entries.length, true,);
  eocdView.setUint32(12, centralBytes.length, true,);
  eocdView.setUint32(16, localBytes.length, true,);
  return {
    archive: concat([localBytes, centralBytes, eocd,],),
    centralOffsets,
    localOffsets,
  };
}

/**
 Computes the byte offset of each record in a sequence, starting at a given offset.
 
 @param records - Record bytes in order.
 
 @param start - Offset of the first record.
 
 @returns Offset of each record, in the same order.
 */
function recordOffsets(records: readonly Uint8Array[], start: number,): number[] {
  return (function offsets(): number[] {
    const result: number[] = [];
    let cursor = start;
    for (const record of records) {
      result.push(cursor,);
      cursor += record.length;
    }
    return result;
  })();
}

/**
 Concatenates byte arrays.
 
 @param chunks - Byte arrays in order.
 
 @returns Single array holding every byte.
 */
function concat(chunks: readonly Uint8Array[],): Uint8Array {
  const total = chunks.reduce((sum, chunk,) => sum + chunk.length, 0,);
  const result = new Uint8Array(total,);
  let offset = 0;
  for (const chunk of chunks) {
    result.set(chunk, offset,);
    offset += chunk.length;
  }
  return result;
}

/**
 Builds the 16-byte canvas.fig header plus a deflate-compressed empty schema.
 
 @returns canvas.fig bytes the parser accepts.
 */
function canvasFigBytes(): Uint8Array {
  const header = new Uint8Array(16,);
  header.set(new TextEncoder().encode(CANVAS_PRELUDE,), 0,);
  new DataView(header.buffer,).setUint32(CANVAS_VERSION_OFFSET, CANVAS_VERSION, true,);
  return concat([
    header,
    new Uint8Array(deflateRawSync(EMPTY_SCHEMA_BYTES,),),
  ],);
}

/**
 Encodes text as UTF-8 bytes.
 
 @param text - Source text.
 
 @returns Encoded bytes.
 */
function utf8(text: string,): Uint8Array {
  return new TextEncoder().encode(text,);
}

await describe({
  name: 'figma export archive',
  children: [
    describe({
      name: extractZipEntries.name,
      children: [
        it({
          name: 'extracts stored entries with their bytes',
          fn: async () => {
            const { archive, } = zipArchive({
              entries: [
                { content: utf8('{}',), name: 'meta.json', },
                { content: new Uint8Array(), name: 'thumbnail.png', },
              ],
            },);
            const entries = await extractZipEntries(archive,);
            expect([...entries.keys(),],).toEqual(['meta.json', 'thumbnail.png',],);
            expect([...entries.get('meta.json',) as Uint8Array,],).toEqual([0x7B, 0x7D,],);
            expect(entries.get('thumbnail.png',),).toHaveLength(0,);
          },
        },),
        it({
          name: 'inflates deflated entries',
          fn: async () => {
            const { archive, } = zipArchive({
              entries: [{ content: utf8('compressed payload',), method: METHOD_DEFLATE, name: 'canvas.fig', },],
            },);
            const entries = await extractZipEntries(archive,);
            const decoded = new TextDecoder().decode(entries.get('canvas.fig',),);
            expect(decoded,).toBe('compressed payload',);
          },
        },),
        it({
          name: 'reads archives whose bytes start inside a larger buffer',
          fn: async () => {
            const { archive, } = zipArchive({ entries: [{ content: utf8('x',), name: 'a.txt', },], },);
            const padded = concat([new Uint8Array([9, 9, 9,],), archive,],);
            const entries = await extractZipEntries(padded.subarray(3,),);
            expect([...entries.get('a.txt',) as Uint8Array,],).toEqual([0x78,],);
          },
        },),
        it({
          name: 'throws for an unsupported compression method',
          fn: async () => {
            const { archive, } = zipArchive({
              entries: [{ content: utf8('x',), method: METHOD_UNSUPPORTED, name: 'a.txt', },],
            },);
            await expect(extractZipEntries(archive,),).rejects.toThrow(
              `Unsupported ZIP compression method ${METHOD_UNSUPPORTED} for "a.txt"`,
            );
          },
        },),
        it({
          name: 'throws when the decompressed size disagrees with the central directory',
          fn: async () => {
            const { archive, } = zipArchive({
              entries: [{ content: utf8('ab',), name: 'a.txt', uncompressedSizeOverride: 5, },],
            },);
            await expect(extractZipEntries(archive,),).rejects.toThrow('Size mismatch for "a.txt": expected 5, got 2',);
          },
        },),
        it({
          name: 'throws when the archive carries no end of central directory record',
          fn: async () => {
            const empty = new Uint8Array(EOCD_LENGTH,);
            await expect(extractZipEntries(empty,),).rejects.toThrow('Cannot find ZIP end of central directory',);
          },
        },),
        it({
          name: 'throws when a central directory entry has the wrong signature',
          fn: async () => {
            const { archive, centralOffsets, } = zipArchive({
              entries: [{ content: utf8('x',), name: 'a.txt', },],
            },);
            const corrupted = new Uint8Array(archive,);
            new DataView(corrupted.buffer,).setUint32(centralOffsets[0] as number, LOCAL_SIGNATURE, true,);
            await expect(extractZipEntries(corrupted,),).rejects.toThrow(
              `Invalid central directory entry signature at offset ${centralOffsets[0]}`,
            );
          },
        },),
        it({
          name: 'throws when a local file header has the wrong signature',
          fn: async () => {
            const { archive, localOffsets, } = zipArchive({
              entries: [{ content: utf8('x',), name: 'a.txt', },],
            },);
            const corrupted = new Uint8Array(archive,);
            new DataView(corrupted.buffer,).setUint32(localOffsets[0] as number, CENTRAL_SIGNATURE, true,);
            await expect(extractZipEntries(corrupted,),).rejects.toThrow(
              `Invalid local file header at offset ${localOffsets[0]}`,
            );
          },
        },),
      ],
    },),

    describe({
      name: parseFigmaFile.name,
      children: [
        it({
          name: 'collects the document, metadata, thumbnail, and image entries',
          fn: async () => {
            const { archive, } = zipArchive({
              entries: [
                { content: canvasFigBytes(), name: 'canvas.fig', },
                {
                  content: utf8(JSON.stringify({ file_name: 'demo', },),),
                  name: 'meta.json',
                },
                { content: new Uint8Array([0x89, 0x50,],), name: 'thumbnail.png', },
                { content: new Uint8Array([1,],), name: 'images/one.png', },
                { content: new Uint8Array([2,],), name: 'images/two.png', },
              ],
            },);
            const file = await parseFigmaFile(archive,);
            expect(file.fileType,).toBe('fig',);
            expect(file.meta.fileName,).toBe('demo',);
            expect([...file.images.keys(),],).toEqual(['one.png', 'two.png',],);
            expect([...file.thumbnail,],).toEqual([0x89, 0x50,],);
          },
        },),
        it({
          name: 'returns empty thumbnail bytes when the archive carries none',
          fn: async () => {
            const { archive, } = zipArchive({
              entries: [
                { content: canvasFigBytes(), name: 'canvas.fig', },
                { content: utf8('{}',), name: 'meta.json', },
              ],
            },);
            const file = await parseFigmaFile(archive,);
            expect(file.thumbnail,).toHaveLength(0,);
          },
        },),
        it({
          name: 'throws when canvas.fig is missing',
          fn: async () => {
            const { archive, } = zipArchive({ entries: [{ content: utf8('{}',), name: 'meta.json', },], },);
            await expect(parseFigmaFile(archive,),).rejects.toThrow('Missing canvas.fig in Figma export file',);
          },
        },),
        it({
          name: 'throws when meta.json is missing',
          fn: async () => {
            const { archive, } = zipArchive({ entries: [{ content: canvasFigBytes(), name: 'canvas.fig', },], },);
            await expect(parseFigmaFile(archive,),).rejects.toThrow('Missing meta.json in Figma export file',);
          },
        },),
        it({
          name: 'reads an export archive from a file path',
          fn: async () => {
            const { archive, } = zipArchive({
              entries: [
                { content: canvasFigBytes(), name: 'canvas.fig', },
                { content: utf8('{}',), name: 'meta.json', },
              ],
            },);
            const directory = await mkdtemp(join(scratchRoot(), 'figma-kiwi-parse-',),);
            const path = join(directory, 'fixture.fig',);
            await writeFile(path, archive,);
            const file = await parseFigmaFile(path,);
            expect(file.fileType,).toBe('fig',);
            expect(file.canvasVersion,).toBe(CANVAS_VERSION,);
            expect(file.document,).toBeTypeOf('symbol',);
          },
        },),
      ],
    },),
  ],
},);
