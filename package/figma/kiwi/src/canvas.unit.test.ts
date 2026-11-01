import {
  crc32,
  deflateRawSync,
  zstdCompressSync,
} from 'node:zlib';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CANVAS_FIG_MAGIC,
  CANVAS_HEADER_SIZE,
  parseCanvasFig,
  parseCanvasHeader,
  parseFigmaFile,
} from '../dist/final/node/index.mjs';

/**
 Format version carried by older Figma exports; its low byte is the ASCII letter `e`.
 */
const VERSION_OLDER = 101;

/**
 Format version carried by October 2026 Figma exports; its low byte is the ASCII letter `j`.
 */
const VERSION_OCTOBER_2026 = 106;

/**
 Version whose second byte is non-zero, proving all four bytes are read little-endian.
 */
const VERSION_MULTI_BYTE = 0x01_02_03_04;

/**
 Byte offset of the little-endian format version in the header.
 */
const VERSION_OFFSET = 8;

/**
 Byte length of a ZIP local file header before the entry name.
 */
const ZIP_LOCAL_HEADER_LENGTH = 30;

/**
 Byte length of a ZIP central directory header before the entry name.
 */
const ZIP_CENTRAL_HEADER_LENGTH = 46;

/**
 Minimal empty Kiwi schema bytes: a definition count of zero.
 */
const EMPTY_SCHEMA_BYTES = new Uint8Array([0,],);

/**
 Document bytes small enough to compress into a single zstd frame.
 */
const DOCUMENT_BYTES = new Uint8Array([
  1,
  2,
  3,
  4,
  5,
],);

/**
 Builds the 16-byte canvas.fig header.

 @param prelude - Eight-byte ASCII prelude.

 @param version - Little-endian uint32 format version.

 @returns Header bytes whose schema chunk length field is zero.
 */
function headerBytes(
  {
    prelude,
    version,
  }: {
    readonly prelude: string;
    readonly version: number;
  },
): Uint8Array {
  /**
   Zero-filled header.
   */
  const bytes = new Uint8Array(CANVAS_HEADER_SIZE,);
  bytes.set(
    new TextEncoder().encode(prelude,),
    0,
  );
  new DataView(bytes.buffer,).setUint32(
    VERSION_OFFSET,
    version,
    true,
  );
  return bytes;
}

/**
 Builds a canvas.fig blob with a deflate schema and an optional zstd document.

 @param prelude - Eight-byte ASCII prelude.

 @param version - Little-endian uint32 format version.

 @param withDocument - Whether to append a size-prefixed zstd frame.

 @returns Complete canvas.fig bytes.
 */
function canvasBytes(
  {
    prelude,
    version,
    withDocument,
  }: {
    readonly prelude: string;
    readonly version: number;
    readonly withDocument: boolean;
  },
): Uint8Array {
  /**
   Deflate-compressed empty schema.
   */
  const schema = deflateRawSync(EMPTY_SCHEMA_BYTES,);
  /**
   Zstd frame of the document bytes.
   */
  const frame = zstdCompressSync(DOCUMENT_BYTES,);
  /**
   Little-endian size prefix of the zstd frame.
   */
  const sizePrefix = new Uint8Array(4,);
  new DataView(sizePrefix.buffer,).setUint32(
    0,
    frame.length,
    true,
  );
  return new Uint8Array([
    ...headerBytes({
      prelude,
      version,
    },),
    ...schema,
    ...(withDocument ? [...sizePrefix, ...frame,] : []),
  ],);
}

/**
 Builds one ZIP local file header followed by its entry name and content.

 @param name - Entry name.
 
 @param content - Uncompressed entry content.
 
 @returns Local record bytes.
 */
function zipLocalRecord(
  {
    name,
    content,
  }: {
    readonly content: Uint8Array;
    readonly name: string;
  },
): Uint8Array {
  /**
   Encoded entry name.
   */
  const nameBytes = new TextEncoder().encode(name,);
  /**
   Local file header, name, and content.
   */
  const record = new Uint8Array(ZIP_LOCAL_HEADER_LENGTH + nameBytes.length + content.length,);
  /**
   View for the local header fields.
   */
  const view = new DataView(record.buffer,);
  view.setUint32(0, 0x04_03_4B_50, true,);
  view.setUint32(14, crc32(content,), true,);
  view.setUint32(18, content.length, true,);
  view.setUint32(22, content.length, true,);
  view.setUint16(26, nameBytes.length, true,);
  record.set(nameBytes, ZIP_LOCAL_HEADER_LENGTH,);
  record.set(content, ZIP_LOCAL_HEADER_LENGTH + nameBytes.length,);
  return record;
}

/**
 Builds one ZIP central directory header followed by its entry name.

 @param name - Entry name.
 
 @param content - Uncompressed entry content.
 
 @param offset - Byte offset of the entry's local record.
 
 @returns Central record bytes.
 */
function zipCentralRecord(
  {
    name,
    content,
    offset,
  }: {
    readonly content: Uint8Array;
    readonly name: string;
    readonly offset: number;
  },
): Uint8Array {
  /**
   Encoded entry name.
   */
  const nameBytes = new TextEncoder().encode(name,);
  /**
   Central directory header and name.
   */
  const record = new Uint8Array(ZIP_CENTRAL_HEADER_LENGTH + nameBytes.length,);
  /**
   View for the central header fields.
   */
  const view = new DataView(record.buffer,);
  view.setUint32(0, 0x02_01_4B_50, true,);
  view.setUint32(16, crc32(content,), true,);
  view.setUint32(20, content.length, true,);
  view.setUint32(24, content.length, true,);
  view.setUint16(28, nameBytes.length, true,);
  view.setUint32(42, offset, true,);
  record.set(nameBytes, ZIP_CENTRAL_HEADER_LENGTH,);
  return record;
}

/**
 Builds a ZIP archive that stores each entry uncompressed.

 @param entries - Entry name and content pairs.
 
 @returns ZIP bytes readable by the package's extractor.
 */
function storedZip(entries: readonly (readonly [string, Uint8Array])[],): Uint8Array {
  /**
   Local records, in entry order.
   */
  const locals = entries.map(([name, content,],) => zipLocalRecord({ content, name, },),);
  /**
   Central records, each pointing at the summed length of the local records before it.
   */
  const centrals = entries.map(([name, content,], index,) =>
    zipCentralRecord({
      content,
      name,
      offset: locals.slice(0, index,).reduce((sum, local,) => sum + local.length, 0,),
    },)
  );
  /**
   Concatenated local records.
   */
  const localBytes = new Uint8Array(locals.flatMap((local,) => [...local,]),);
  /**
   Concatenated central directory.
   */
  const centralDirectory = new Uint8Array(centrals.flatMap((central,) => [...central,]),);
  /**
   End of central directory record.
   */
  const eocd = new Uint8Array(22,);
  /**
   View for the end record fields.
   */
  const eocdView = new DataView(eocd.buffer,);
  eocdView.setUint32(0, 0x06_05_4B_50, true,);
  eocdView.setUint16(10, entries.length, true,);
  eocdView.setUint32(12, centralDirectory.length, true,);
  eocdView.setUint32(16, localBytes.length, true,);
  return new Uint8Array([...localBytes, ...centralDirectory, ...eocd,],);
}

await describe({
  name: 'canvas.fig header',
  children: [
    describe({
      name: parseCanvasHeader.name,
      children: [
        it({
          name: 'maps each file type prelude and records the older version',
          fn: async () => {
            for (const fileType of ['fig', 'deck', 'jam',] as const) {
              /**
               Header for this file type at the older version.
               */
              const header = headerBytes({
                prelude: CANVAS_FIG_MAGIC[fileType],
                version: VERSION_OLDER,
              },);
              expect(parseCanvasHeader(header,),).toEqual({
                fileType,
                version: VERSION_OLDER,
              },);
            }
          },
        },),
        it({
          name: 'accepts the newer revision letter j for every file type',
          fn: async () => {
            for (const fileType of ['fig', 'deck', 'jam',] as const) {
              /**
               Header for this file type at the newer version.
               */
              const header = headerBytes({
                prelude: CANVAS_FIG_MAGIC[fileType],
                version: VERSION_OCTOBER_2026,
              },);
              expect(parseCanvasHeader(header,),).toEqual({
                fileType,
                version: VERSION_OCTOBER_2026,
              },);
            }
          },
        },),
        it({
          name: 'reads all four version bytes little-endian',
          fn: async () => {
            expect(parseCanvasHeader(headerBytes({
              prelude: CANVAS_FIG_MAGIC.fig,
              version: VERSION_MULTI_BYTE,
            },),).version,).toBe(VERSION_MULTI_BYTE,);
          },
        },),
        it({
          name: 'accepts the literal fig-kiwij bytes seen in real October 2026 exports',
          fn: async () => {
            /**
             First 16 bytes of a real export: prelude, version 106, schema chunk length.
             */
            const real = new Uint8Array([
              ...new TextEncoder().encode('fig-kiwi',),
              0x6A, 0x00, 0x00, 0x00,
              0x61, 0x75, 0x00, 0x00,
            ],);
            expect(parseCanvasHeader(real,),).toEqual({
              fileType: 'fig',
              version: VERSION_OCTOBER_2026,
            },);
          },
        },),
        it({
          name: 'names the input and the accepted forms for an unknown prelude',
          fn: async () => {
            expect(() =>
              parseCanvasHeader(headerBytes({
                prelude: 'fig-xxxx',
                version: VERSION_OLDER,
              },),)
            ).toThrow('Unknown canvas.fig magic: "fig-xxxx"',);
            expect(() =>
              parseCanvasHeader(headerBytes({
                prelude: 'fig-xxxx',
                version: VERSION_OLDER,
              },),)
            ).toThrow('accepted: "fig-kiwi" (fig), "fig-deck" (deck), "fig-jam." (jam)',);
          },
        },),
        it({
          name: 'rejects a prelude that differs from an accepted one in a single byte',
          fn: async () => {
            expect(() =>
              parseCanvasHeader(headerBytes({
                prelude: 'fig-kiwj',
                version: VERSION_OLDER,
              },),)
            ).toThrow('Unknown canvas.fig magic: "fig-kiwj"',);
          },
        },),
        it({
          name: 'rejects data shorter than the header',
          fn: async () => {
            expect(() => parseCanvasHeader(new Uint8Array(CANVAS_HEADER_SIZE - 1,),)).toThrow(
              'canvas.fig header too short: 15 bytes (need 16)',
            );
          },
        },),
        it({
          name: 'reads the header from a subarray with a nonzero byte offset',
          fn: async () => {
            /**
             Header preceded by padding so the view starts mid-buffer.
             */
            const padded = new Uint8Array([
              9, 9, 9,
              ...headerBytes({
                prelude: CANVAS_FIG_MAGIC.jam,
                version: VERSION_OCTOBER_2026,
              },),
            ],);
            /**
             View starting after the padding.
             */
            const view = padded.subarray(3,);
            expect(parseCanvasHeader(view,),).toEqual({
              fileType: 'jam',
              version: VERSION_OCTOBER_2026,
            },);
          },
        },),
      ],
    },),
    describe({
      name: parseCanvasFig.name,
      children: [
        it({
          name: 'returns file type, newer version, schema, and zstd document bytes',
          fn: async () => {
            /**
             Sections from a version 106 deck blob.
             */
            const sections = await parseCanvasFig(canvasBytes({
              prelude: CANVAS_FIG_MAGIC.deck,
              version: VERSION_OCTOBER_2026,
              withDocument: true,
            },),);
            expect(sections.fileType,).toBe('deck',);
            expect(sections.version,).toBe(VERSION_OCTOBER_2026,);
            expect([...sections.schemaBytes,],).toEqual([...EMPTY_SCHEMA_BYTES,],);
            expect([...sections.documentBytes,],).toEqual([...DOCUMENT_BYTES,],);
          },
        },),
        it({
          name: 'returns empty document bytes when no zstd frame follows the schema',
          fn: async () => {
            /**
             Sections from a version 101 blob with deflate schema only.
             */
            const sections = await parseCanvasFig(canvasBytes({
              prelude: CANVAS_FIG_MAGIC.fig,
              version: VERSION_OLDER,
              withDocument: false,
            },),);
            expect(sections.version,).toBe(VERSION_OLDER,);
            expect(sections.documentBytes,).toHaveLength(0,);
          },
        },),
        it({
          name: 'rejects an unknown prelude before touching the payload',
          fn: async () => {
            /**
             Blob with an unknown prelude.
             */
            const blob = canvasBytes({
              prelude: 'fig-nope',
              version: VERSION_OLDER,
              withDocument: true,
            },);
            await expect(parseCanvasFig(blob,),).rejects.toThrow('Unknown canvas.fig magic: "fig-nope"',);
          },
        },),
      ],
    },),
    describe({
      name: parseFigmaFile.name,
      children: [
        it({
          name: 'parses an archive whose canvas.fig carries the newer version and records it',
          fn: async () => {
            /**
             Stored ZIP with the two required entries.
             */
            const archive = storedZip([
              [
                'canvas.fig',
                canvasBytes({
                  prelude: CANVAS_FIG_MAGIC.fig,
                  version: VERSION_OCTOBER_2026,
                  withDocument: false,
                },),
              ],
              ['meta.json', new TextEncoder().encode('{}',),],
            ],);
            /**
             Parsed file.
             */
            const file = await parseFigmaFile(archive,);
            expect(file.fileType,).toBe('fig',);
            expect(file.canvasVersion,).toBe(VERSION_OCTOBER_2026,);
          },
        },),
      ],
    },),
  ],
},);
