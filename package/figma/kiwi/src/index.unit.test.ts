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
      ],
    },),
  ],
},);
