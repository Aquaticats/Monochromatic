import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  parseKiwiSchema,
  resolveTypeName,
  type KiwiEnum,
  type KiwiStruct,
} from '../dist/final/node/index.mjs';

/**
 Raw schema kind byte for enum definitions.
 */
const ENUM_KIND_BYTE = 0;

/**
 Raw schema kind byte for struct definitions.
 */
const STRUCT_KIND_BYTE = 1;

/**
 Raw schema kind byte for message definitions.
 */
const MESSAGE_KIND_BYTE = 2;

/**
 Kind byte no version of the format defines, used to prove the parser refuses it.
 */
const UNKNOWN_KIND_BYTE = 3;

/**
 Wire value of the second fixture enum field (`SECOND`).
 */
const SECOND_ENUM_VALUE = 2;

/**
 Int primitive type code, whose negative value exercises zigzag encoding.
 */
const INT_TYPE = -3;

/**
 String primitive type code.
 */
const STRING_TYPE = -6;

/**
 Reference type code pointing at the first fixture definition.
 */
const FIRST_DEFINITION_TYPE = 0;

/**
 Type code that no fixture schema definition occupies.
 */
const MISSING_DEFINITION_TYPE = 9;

/**
 Primitive index no Kiwi primitive table entry occupies.
 */
const UNKNOWN_PRIMITIVE_TYPE = -99;

/**
 Encodes text as UTF-8 bytes.
 
 @param text - Source text.
 
 @returns Encoded bytes.
 */
function utf8(text: string,): Uint8Array {
  return new TextEncoder().encode(text,);
}

/**
 Encodes one unsigned LEB128 varuint.
 
 @param value - Non-negative integer.
 
 @returns Encoded bytes.
 */
function varUintBytes(value: number,): Uint8Array {
  return (function encode(): Uint8Array {
    const bytes: number[] = [];
    let remaining = value;
    do {
      const payload = remaining % 128;
      remaining = Math.floor(remaining / 128,);
      bytes.push(remaining > 0 ? (payload | 0x80) : payload,);
    } while (remaining > 0);
    return new Uint8Array(bytes,);
  })();
}

/**
 Encodes one zigzag integer as Kiwi writes field type codes.
 
 @param value - Signed integer.
 
 @returns Encoded bytes.
 */
function varIntBytes(value: number,): Uint8Array {
  return varUintBytes(value < 0 ? (((~value) * 2) + 1) : (value * 2),);
}

/**
 Encodes one null-terminated Kiwi string.
 
 @param value - Source text.
 
 @returns Encoded bytes including the terminator.
 */
function stringBytes(value: string,): Uint8Array {
  return new Uint8Array([...utf8(value,), 0,],);
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
 Builds one enum definition.
 
 @param name - Definition name.
 
 @param fields - Field names paired with their wire values.
 
 @returns Encoded definition.
 */
function enumDefinition(
  {
    name,
    fields,
  }: {
    readonly fields: readonly (readonly [string, number])[];
    readonly name: string;
  },
): Uint8Array {
  return concat([
    stringBytes(name,),
    new Uint8Array([ENUM_KIND_BYTE,],),
    varUintBytes(fields.length,),
    ...fields.flatMap(([fieldName, value,],) => [
      stringBytes(fieldName,),
      varIntBytes(0,),
      new Uint8Array([0,],),
      varUintBytes(value,),
    ]),
  ],);
}

/**
 Builds one struct or message definition.
 
 @param name - Definition name.
 
 @param kindByte - Raw kind byte.
 
 @param fields - Field descriptors in declaration order.
 
 @returns Encoded definition.
 */
function structDefinition(
  {
    name,
    kindByte,
    fields,
  }: {
    readonly fields: readonly {
      readonly isArray?: boolean;
      readonly name: string;
      readonly tag: number;
      readonly type: number;
    }[];
    readonly kindByte: number;
    readonly name: string;
  },
): Uint8Array {
  return concat([
    stringBytes(name,),
    new Uint8Array([kindByte,],),
    varUintBytes(fields.length,),
    ...fields.flatMap(field => [
      stringBytes(field.name,),
      varIntBytes(field.type,),
      new Uint8Array([field.isArray === true ? 1 : 0,],),
      varUintBytes(field.tag,),
    ]),
  ],);
}

/**
 Builds a whole schema from encoded definitions.
 
 @param definitions - Encoded definitions in type-code order.
 
 @returns Schema bytes with the definition count in front.
 */
function schemaBytes(definitions: readonly Uint8Array[],): Uint8Array {
  return concat([varUintBytes(definitions.length,), ...definitions,],);
}

await describe({
  name: 'kiwi schema',
  children: [
    describe({
      name: parseKiwiSchema.name,
      children: [
        it({
          name: 'parses enum, struct, and message definitions with every field property',
          fn: async () => {
            const schema = parseKiwiSchema(schemaBytes([
              enumDefinition({ fields: [['FIRST', 0,], ['SECOND', SECOND_ENUM_VALUE,],], name: 'Kind', },),
              structDefinition({
                fields: [
                  { name: 'id', tag: 1, type: INT_TYPE, },
                  { name: 'labels', isArray: true, tag: 2, type: STRING_TYPE, },
                ],
                kindByte: STRUCT_KIND_BYTE,
                name: 'Shape',
              },),
              structDefinition({
                fields: [{ name: 'shape', tag: 3, type: FIRST_DEFINITION_TYPE, },],
                kindByte: MESSAGE_KIND_BYTE,
                name: 'Message',
              },),
            ],),);
            expect(schema.definitions.map(definition => definition.kind,),).toEqual(['ENUM', 'STRUCT', 'MESSAGE',],);
            const kind = schema.enumByName.get('Kind',) as KiwiEnum;
            expect(kind.fields.map(field => field.name,),).toEqual(['FIRST', 'SECOND',],);
            expect(kind.fields.map(field => field.value,),).toEqual([0, SECOND_ENUM_VALUE,],);
            const shape = schema.structByName.get('Shape',) as KiwiStruct;
            expect(shape.fields.map(field => field.name,),).toEqual(['id', 'labels',],);
            expect(shape.fields.map(field => field.type,),).toEqual([INT_TYPE, STRING_TYPE,],);
            expect(shape.fields.map(field => field.isArray,),).toEqual([false, true,],);
            expect(shape.fields.map(field => field.value,),).toEqual([1, 2,],);
            expect(schema.structByName.get('Message',)?.kind,).toBe('MESSAGE',);
          },
        },),
        it({
          name: 'keeps every definition reachable by its type code',
          fn: async () => {
            const schema = parseKiwiSchema(schemaBytes([
              enumDefinition({ fields: [['FIRST', 0,],], name: 'Kind', },),
              structDefinition({ fields: [], kindByte: STRUCT_KIND_BYTE, name: 'Empty', },),
            ],),);
            expect(schema.definitions,).toHaveLength(2,);
            expect(schema.enumByName.size,).toBe(1,);
            expect(schema.structByName.size,).toBe(1,);
            expect(schema.enumByName.has('Empty',),).toBe(false,);
            expect(schema.structByName.has('Kind',),).toBe(false,);
          },
        },),
        it({
          name: 'throws naming the definition when the kind byte is unknown',
          fn: async () => {
            const unknownKind = concat([
              stringBytes('Thing',),
              new Uint8Array([UNKNOWN_KIND_BYTE,],),
              varUintBytes(0,),
            ],);
            expect(() => parseKiwiSchema(schemaBytes([unknownKind,],),),).toThrow(
              `Unknown Kiwi definition kind ${UNKNOWN_KIND_BYTE} for "Thing"`,
            );
          },
        },),
        it({
          name: 'throws when the definition count promises more bytes than the schema holds',
          fn: async () => {
            expect(() => parseKiwiSchema(varUintBytes(1,),),).toThrow('Unexpected end of Kiwi data',);
          },
        },),
      ],
    },),

    describe({
      name: resolveTypeName.name,
      children: [
        it({
          name: 'names every primitive type code',
          fn: async () => {
            const schema = parseKiwiSchema(schemaBytes([],),);
            expect(resolveTypeName({ schema, typeCode: -4, },),).toBe('uint',);
            expect(resolveTypeName({ schema, typeCode: UNKNOWN_PRIMITIVE_TYPE, },),).toBe('prim[98]',);
          },
        },),
        it({
          name: 'names a definition by its type code',
          fn: async () => {
            const schema = parseKiwiSchema(schemaBytes([
              structDefinition({ fields: [], kindByte: STRUCT_KIND_BYTE, name: 'Paint', },),
            ],),);
            expect(resolveTypeName({ schema, typeCode: FIRST_DEFINITION_TYPE, },),).toBe('Paint',);
          },
        },),
        it({
          name: 'names a type code no definition occupies',
          fn: async () => {
            const schema = parseKiwiSchema(schemaBytes([],),);
            expect(resolveTypeName({ schema, typeCode: MISSING_DEFINITION_TYPE, },),).toBe('ref[9]',);
          },
        },),
      ],
    },),
  ],
},);
