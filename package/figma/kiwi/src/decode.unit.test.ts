import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  KiwiDecodeError,
  createBinaryReader,
  decodeDocument,
  decodeMessage,
  decodeStruct,
  decodeValue,
  type KiwiDefinition,
  type KiwiEnum,
  type KiwiSchema,
  type KiwiStruct,
  type KiwiStructField,
} from '../dist/final/node/index.mjs';

/**
 Type code of the Kiwi bool primitive.
 */
const BOOL_TYPE = -1;

/**
 Type code of the Kiwi byte primitive.
 */
const BYTE_TYPE = -2;

/**
 Type code of the Kiwi int primitive.
 */
const INT_TYPE = -3;

/**
 Type code of the Kiwi uint primitive.
 */
const UINT_TYPE = -4;

/**
 Type code of the Kiwi float primitive.
 */
const FLOAT_TYPE = -5;

/**
 Type code of the Kiwi string primitive.
 */
const STRING_TYPE = -6;

/**
 Type code of a primitive this decoder does not know.
 */
const UNKNOWN_PRIMITIVE_TYPE = -99;

/**
 Type code that no definition in the fixture schemas occupies.
 */
const MISSING_DEFINITION_TYPE = 7;

/**
 Four-byte Kiwi varfloat encoding of 1.0: the IEEE 754 bits rotated left by 23.
 */
const ONE_POINT_ZERO_VARFLOAT = new Uint8Array([0x7F, 0x00, 0x00, 0x00,],);

/**
 Nesting levels fed to a self-referencing message so the depth limit is crossed.
 */
const NESTING_LEVELS_BEYOND_LIMIT = 40;

/**
 Builds a struct field descriptor.
 
 @param name - Field name.
 
 @param type - Field type code.
 
 @param tag - Wire tag or positional index.
 
 @param isArray - Whether the field repeats.
 
 @returns Field descriptor for a hand-built schema.
 */
function field(
  {
    name,
    type,
    tag,
    isArray = false,
  }: {
    readonly isArray?: boolean;
    readonly name: string;
    readonly tag: number;
    readonly type: number;
  },
): KiwiStructField {
  return {
    isArray,
    name,
    type,
    value: tag,
  };
}

/**
 Builds a struct or message definition.
 
 @param kind - Definition kind.
 
 @param name - Definition name.
 
 @param fields - Field descriptors.
 
 @returns Definition for a hand-built schema.
 */
function structDefinition(
  {
    kind,
    name,
    fields,
  }: {
    readonly fields: readonly KiwiStructField[];
    readonly kind: KiwiStruct['kind'];
    readonly name: string;
  },
): KiwiStruct {
  return {
    fields,
    kind,
    name,
  };
}

/**
 Builds an enum definition.
 
 @param name - Definition name.
 
 @param values - Enum field names paired with their wire values.
 
 @returns Enum definition for a hand-built schema.
 */
function enumDefinition(
  {
    name,
    values,
  }: {
    readonly name: string;
    readonly values: readonly (readonly [string, number])[];
  },
): KiwiEnum {
  return {
    fields: values.map(([fieldName, value,],) => ({
      isArray: false,
      name: fieldName,
      value,
    }),),
    kind: 'ENUM',
    name,
  };
}

/**
 Builds a schema whose lookup maps index the given definitions, mirroring the
 maps `parseKiwiSchema` derives, so a hand-built schema reaches the decoder the
 same way a parsed one does.
 
 @param definitions - Definitions in type-code order.
 
 @returns Schema with derived name maps.
 */
function schemaOf({ definitions, }: { readonly definitions: readonly KiwiDefinition[]; },): KiwiSchema {
  return {
    definitions,
    enumByName: new Map(definitions
      .filter(function isEnum(definition,): definition is KiwiEnum {
        return definition.kind === 'ENUM';
      },)
      .map(function enumEntry(definition,): readonly [string, KiwiEnum] {
        return [
          definition.name,
          definition,
        ];
      },),),
    structByName: new Map(definitions
      .filter(function isStruct(definition,): definition is KiwiStruct {
        return definition.kind !== 'ENUM';
      },)
      .map(function structEntry(definition,): readonly [string, KiwiStruct] {
        return [
          definition.name,
          definition,
        ];
      },),),
  };
}

/**
 Names the decoded KiwiDecodeError a call must raise.
 
 @param act - Call expected to refuse the input.
 
 @returns Thrown decode error.
 */
function decodeRefusal(act: () => unknown,): KiwiDecodeError {
  const refusal = caught(act,);
  expect(refusal,).toBeInstanceOf(KiwiDecodeError,);
  return refusal as KiwiDecodeError;
}

await describe({
  name: 'kiwi decoder',
  children: [
    describe({
      name: KiwiDecodeError.name,
      children: [
        it({
          name: 'is an Error subclass named after itself',
          fn: async () => {
            const error = new KiwiDecodeError('fixture',);
            expect(error,).toBeInstanceOf(Error,);
            expect(error.name,).toBe('KiwiDecodeError',);
          },
        },),
      ],
    },),

    describe({
      name: decodeValue.name,
      children: [
        it({
          name: 'decodes every primitive type code',
          fn: async () => {
            const schema = schemaOf({ definitions: [], },);
            const primitives: readonly (readonly [number, Uint8Array, unknown])[] = [
              [BOOL_TYPE, new Uint8Array([1,],), true,],
              [BOOL_TYPE, new Uint8Array([0,],), false,],
              [BYTE_TYPE, new Uint8Array([0x2A,],), 0x2A,],
              [INT_TYPE, new Uint8Array([0x01,],), -1,],
              [UINT_TYPE, new Uint8Array([0x80, 0x01,],), 128,],
              [FLOAT_TYPE, ONE_POINT_ZERO_VARFLOAT, 1,],
              [STRING_TYPE, new Uint8Array([0x61, 0x62, 0x00,],), 'ab',],
            ];
            for (const [typeCode, data, expected,] of primitives) {
              expect(decodeValue({
                depth: 0,
                reader: createBinaryReader({ data, },),
                schema,
                typeCode,
              },),).toEqual(expected,);
            }
          },
        },),
        it({
          name: 'decodes an enum field to its qualified name',
          fn: async () => {
            const schema = schemaOf({
              definitions: [enumDefinition({ name: 'Kind', values: [['FIRST', 0,], ['THIRD', 2,],], },),],
            },);
            expect(decodeValue({
              depth: 0,
              reader: createBinaryReader({ data: new Uint8Array([0x02,],), },),
              schema,
              typeCode: 0,
            },),).toBe('Kind.THIRD',);
          },
        },),
        it({
          name: 'names an enum value the schema does not map',
          fn: async () => {
            const schema = schemaOf({
              definitions: [enumDefinition({ name: 'Kind', values: [['FIRST', 0,],], },),],
            },);
            expect(decodeValue({
              depth: 0,
              reader: createBinaryReader({ data: new Uint8Array([0x05,],), },),
              schema,
              typeCode: 0,
            },),).toBe('Kind(5)',);
          },
        },),
        it({
          name: 'throws when the type code names no definition',
          fn: async () => {
            const refusal = decodeRefusal(() => decodeValue({
              depth: 0,
              reader: createBinaryReader({ data: new Uint8Array([0x00,],), },),
              schema: schemaOf({ definitions: [], },),
              typeCode: MISSING_DEFINITION_TYPE,
            },),);
            expect(refusal.message,).toContain('no definition for type code 7',);
            expect(refusal.message,).toContain('at byte 0',);
          },
        },),
        it({
          name: 'throws when the type code names no primitive',
          fn: async () => {
            const refusal = decodeRefusal(() => decodeValue({
              depth: 0,
              reader: createBinaryReader({ data: new Uint8Array([0x00,],), },),
              schema: schemaOf({ definitions: [], },),
              typeCode: UNKNOWN_PRIMITIVE_TYPE,
            },),);
            expect(refusal.message,).toContain('no primitive for type code -99',);
            expect(refusal.message,).toContain('at byte 0',);
          },
        },),
        it({
          name: 'throws instead of returning a value past the depth limit',
          fn: async () => {
            const recursive = structDefinition({
              fields: [field({ name: 'child', tag: 1, type: 0, },),],
              kind: 'MESSAGE',
              name: 'Nested',
            },);
            const nestedBytes = new Uint8Array(NESTING_LEVELS_BEYOND_LIMIT * 2,);
            for (let index = 0; index < NESTING_LEVELS_BEYOND_LIMIT; index++) {
              nestedBytes[(index * 2)] = 0x01;
              nestedBytes[((index * 2) + 1)] = 0x01;
            }
            const refusal = decodeRefusal(() => decodeValue({
              depth: 0,
              reader: createBinaryReader({ data: nestedBytes, },),
              schema: schemaOf({ definitions: [recursive,], },),
              typeCode: 0,
            },),);
            expect(refusal.message,).toContain('exceeds',);
            expect(refusal.message,).toContain('depth',);
          },
        },),
        it({
          name: 'names the field a failing value belongs to',
          fn: async () => {
            const schema = schemaOf({
              definitions: [structDefinition({
                fields: [field({ name: 'opacity', tag: 1, type: MISSING_DEFINITION_TYPE, },),],
                kind: 'STRUCT',
                name: 'Shape',
              },),],
            },);
            const refusal = decodeRefusal(() => decodeStruct({
              def: schema.definitions[0] as KiwiStruct,
              depth: 0,
              reader: createBinaryReader({ data: new Uint8Array([0x01,],), },),
              schema,
            },),);
            expect(refusal.message,).toContain('field "opacity"',);
          },
        },),
      ],
    },),

    describe({
      name: decodeStruct.name,
      children: [
        it({
          name: 'decodes positional fields in declaration order',
          fn: async () => {
            const shape = structDefinition({
              fields: [
                field({ name: 'id', tag: 0, type: UINT_TYPE, },),
                field({ name: 'name', tag: 1, type: STRING_TYPE, },),
              ],
              kind: 'STRUCT',
              name: 'Shape',
            },);
            expect(decodeStruct({
              def: shape,
              depth: 0,
              reader: createBinaryReader({ data: new Uint8Array([0x07, 0x68, 0x69, 0x00,],), },),
              schema: schemaOf({ definitions: [shape,], },),
            },),).toEqual({
              __type: 'Shape',
              id: 7,
              name: 'hi',
            },);
          },
        },),
        it({
          name: 'throws naming the struct and field when the input ends early',
          fn: async () => {
            const shape = structDefinition({
              fields: [
                field({ name: 'id', tag: 0, type: UINT_TYPE, },),
                field({ name: 'name', tag: 1, type: STRING_TYPE, },),
              ],
              kind: 'STRUCT',
              name: 'Shape',
            },);
            const refusal = decodeRefusal(() => decodeStruct({
              def: shape,
              depth: 0,
              reader: createBinaryReader({ data: new Uint8Array([0x07,],), },),
              schema: schemaOf({ definitions: [shape,], },),
            },),);
            expect(refusal.message,).toContain('Shape',);
            expect(refusal.message,).toContain('field "name"',);
          },
        },),
      ],
    },),

    describe({
      name: decodeMessage.name,
      children: [
        it({
          name: 'decodes tagged fields until the zero tag',
          fn: async () => {
            const sample = structDefinition({
              fields: [
                field({ name: 'count', tag: 1, type: UINT_TYPE, },),
                field({ name: 'label', tag: 2, type: STRING_TYPE, },),
              ],
              kind: 'MESSAGE',
              name: 'Sample',
            },);
            expect(decodeMessage({
              def: sample,
              depth: 0,
              reader: createBinaryReader({ data: new Uint8Array([0x01, 0x05, 0x02, 0x68, 0x69, 0x00, 0x00,],), },),
              schema: schemaOf({ definitions: [sample,], },),
            },),).toEqual({
              __type: 'Sample',
              count: 5,
              label: 'hi',
            },);
          },
        },),
        it({
          name: 'decodes a repeated field from its element count',
          fn: async () => {
            const list = structDefinition({
              fields: [field({
                isArray: true,
                name: 'ids',
                tag: 3,
                type: UINT_TYPE,
              },),],
              kind: 'MESSAGE',
              name: 'List',
            },);
            expect(decodeMessage({
              def: list,
              depth: 0,
              reader: createBinaryReader({ data: new Uint8Array([0x03, 0x02, 0x07, 0x09,],), },),
              schema: schemaOf({ definitions: [list,], },),
            },),).toEqual({
              __type: 'List',
              ids: [7, 9,],
            },);
          },
        },),
        it({
          name: 'throws on a tag no field declares instead of ending the message',
          fn: async () => {
            const sample = structDefinition({
              fields: [field({ name: 'count', tag: 1, type: UINT_TYPE, },),],
              kind: 'MESSAGE',
              name: 'Sample',
            },);
            const refusal = decodeRefusal(() => decodeMessage({
              def: sample,
              depth: 0,
              reader: createBinaryReader({ data: new Uint8Array([0x01, 0x05, 0x02, 0x09,],), },),
              schema: schemaOf({ definitions: [sample,], },),
            },),);
            expect(refusal.message,).toContain('Sample',);
            expect(refusal.message,).toContain('tag 2',);
            expect(refusal.message,).toContain('at byte 2',);
          },
        },),
      ],
    },),

    describe({
      name: decodeDocument.name,
      children: [
        it({
          name: 'decodes the document bytes through the Message definition',
          fn: async () => {
            const message = structDefinition({
              fields: [field({
                isArray: true,
                name: 'nodeChanges',
                tag: 1,
                type: UINT_TYPE,
              },),],
              kind: 'MESSAGE',
              name: 'Message',
            },);
            expect(decodeDocument({
              documentData: new Uint8Array([0x01, 0x02, 0x00, 0x05, 0x00,],),
              schema: schemaOf({ definitions: [message,], },),
            },),).toEqual({
              __type: 'Message',
              nodeChanges: [0, 5,],
            },);
          },
        },),
        it({
          name: 'throws when the schema carries no Message definition',
          fn: async () => {
            const refusal = decodeRefusal(() => decodeDocument({
              documentData: new Uint8Array([0x01,],),
              schema: schemaOf({ definitions: [], },),
            },),);
            expect(refusal.message,).toContain('Message definition not found in schema',);
          },
        },),
      ],
    },),
  ],
},);
