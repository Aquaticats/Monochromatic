import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  FIGMA_DOCUMENT_ABSENT,
  createBinaryReader,
  decodeDocument,
  parseCanvasHeader,
  parseKiwiSchema,
  parseMetaJson,
  resolveTypeName,
} from '../dist/final/node/index.mjs';

/**
 Example single-byte varuint payload used by reader tests.
 */
const EXAMPLE_VARUINT_VALUE = 42;

/**
 Primitive type code for Kiwi string values.
 */
const STRING_TYPE_CODE = -6;

/**
 Reserved byte count after the null-terminated fig magic in the header.
 */
const FIG_HEADER_RESERVED_BYTES = 6;

/**
 Header bytes for a fig canvas payload.
 */
const FIG_HEADER = new Uint8Array([
  ...new TextEncoder().encode('fig-kiwie\0',),
  ...new Uint8Array(FIG_HEADER_RESERVED_BYTES,),
],);

/**
 Minimal empty Kiwi schema bytes.
 */
const EMPTY_SCHEMA_BYTES = new Uint8Array([0,],);

await describe({
  name: 'figma kiwi parser',
  children: [
    describe({
      name: createBinaryReader.name,
      children: [
        it({
          name: 'reads single byte varuint',
          fn: async () => {
            /**
             Reader over example varuint bytes.
             */
            const reader = createBinaryReader({
              data: new Uint8Array([EXAMPLE_VARUINT_VALUE,],),
            },);
            expect(reader.readVarUint(),).toBe(EXAMPLE_VARUINT_VALUE,);
            expect(reader.eof,).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: parseKiwiSchema.name,
      children: [
        it({
          name: 'parses empty schema',
          fn: async () => {
            /**
             Parsed empty schema.
             */
            const schema = parseKiwiSchema(EMPTY_SCHEMA_BYTES,);
            expect(schema.definitions,).toHaveLength(0,);
          },
        },),
      ],
    },),
    describe({
      name: resolveTypeName.name,
      children: [
        it({
          name: 'resolves primitive string type',
          fn: async () => {
            /**
             Empty schema still resolves primitive type codes.
             */
            const schema = parseKiwiSchema(EMPTY_SCHEMA_BYTES,);
            expect(resolveTypeName({
              schema,
              typeCode: STRING_TYPE_CODE,
            },),).toBe('string',);
          },
        },),
      ],
    },),
    describe({
      name: decodeDocument.name,
      children: [
        it({
          name: 'returns sentinel for empty document bytes',
          fn: async () => {
            /**
             Empty schema paired with empty document data.
             */
            const schema = parseKiwiSchema(EMPTY_SCHEMA_BYTES,);
            expect(decodeDocument({
              documentData: new Uint8Array(),
              schema,
            },),).toBe(FIGMA_DOCUMENT_ABSENT,);
          },
        },),
      ],
    },),
    describe({
      name: parseCanvasHeader.name,
      children: [
        it({
          name: 'parses fig magic',
          fn: async () => {
            expect(parseCanvasHeader(FIG_HEADER,).fileType,).toBe('fig',);
          },
        },),
      ],
    },),
    describe({
      name: parseMetaJson.name,
      children: [
        it({
          name: 'parses metadata defaults',
          fn: async () => {
            /**
             Empty metadata payload.
             */
            const json = new TextEncoder().encode('{}',);
            /**
             Parsed metadata with defaulted fields.
             */
            const meta = parseMetaJson(json,);
            expect(meta.fileName,).toBe('',);
            expect(meta.backgroundColor.r,).toBe(1,);
          },
        },),
        it({
          name: 'maps every field a real meta.json carries',
          fn: async () => {
            /**
             Metadata payload shaped like the exports Figma writes.
             */
            const json = new TextEncoder().encode(JSON.stringify({
              client_meta: {
                background_color: { a: 0.5, b: 0.25, g: 0.75, r: 0.125, },
                render_coordinates: { height: 40, width: 30, x: 10, y: 20, },
                thumbnail_size: { height: 20, width: 10, },
              },
              developer_related_links: [{ url: 'https://example.test', },],
              exported_at: '2026-10-17T14:22:08Z',
              file_name: 'demo',
            },),);
            /**
             Parsed metadata.
             */
            const meta = parseMetaJson(json,);
            expect(meta.fileName,).toBe('demo',);
            expect(meta.exportedAt,).toBe('2026-10-17T14:22:08Z',);
            expect(meta.developerRelatedLinks,).toHaveLength(1,);
            expect(meta.backgroundColor,).toEqual({ a: 0.5, b: 0.25, g: 0.75, r: 0.125, },);
            expect(meta.thumbnailSize,).toEqual({ height: 20, width: 10, },);
            expect(meta.renderCoordinates,).toEqual({ height: 40, width: 30, x: 10, y: 20, },);
          },
        },),
        it({
          name: 'keeps defaults when the root is not an object',
          fn: async () => {
            /**
             Array root, which carries no metadata fields.
             */
            const json = new TextEncoder().encode('[]',);
            /**
             Parsed metadata with defaults everywhere.
             */
            const meta = parseMetaJson(json,);
            expect(meta.fileName,).toBe('',);
            expect(meta.exportedAt,).toBe('',);
            expect(meta.developerRelatedLinks,).toEqual([],);
            expect(meta.renderCoordinates,).toEqual({ height: 0, width: 0, x: 0, y: 0, },);
          },
        },),
        it({
          name: 'keeps defaults for fields whose value has the wrong type',
          fn: async () => {
            /**
             Metadata whose nested values are not the declared types.
             */
            const json = new TextEncoder().encode(JSON.stringify({
              client_meta: {
                background_color: { r: 'red', },
                thumbnail_size: 'large',
              },
              developer_related_links: 'none',
              file_name: 5,
            },),);
            /**
             Parsed metadata with defaults for every mistyped field.
             */
            const meta = parseMetaJson(json,);
            expect(meta.fileName,).toBe('',);
            expect(meta.developerRelatedLinks,).toEqual([],);
            expect(meta.backgroundColor.r,).toBe(1,);
            expect(meta.thumbnailSize,).toEqual({ height: 0, width: 0, },);
          },
        },),
      ],
    },),
  ],
},);
