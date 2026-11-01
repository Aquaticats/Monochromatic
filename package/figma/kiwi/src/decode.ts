/**
 Kiwi document decoding helpers.
 
 @example
 ```ts
 decodeDocument({ documentData: new Uint8Array(), schema: { definitions: [], enumByName: new Map(), structByName: new Map() } });
 // FIGMA_DOCUMENT_ABSENT
 ```
 */

import {
  type BinaryReader,
  createBinaryReader,
} from './binary-reader.ts';
import {
  FIGMA_DOCUMENT_ABSENT,
  KIWI_VALUE_ABSENT,
  type KiwiDecodedValue,
  type KiwiSchema,
  type KiwiStruct,
  type KiwiStructField,
} from './types.ts';

/**
 Maximum recursive decode depth before yielding a sentinel.
 */
const MAX_DECODE_DEPTH = 20;

/**
 Primitive bool index in Kiwi's primitive table.
 */
const PRIMITIVE_BOOL = 0;

/**
 Primitive byte index in Kiwi's primitive table.
 */
const PRIMITIVE_BYTE = 1;

/**
 Primitive int index in Kiwi's primitive table.
 */
const PRIMITIVE_INT = 2;

/**
 Primitive uint index in Kiwi's primitive table.
 */
const PRIMITIVE_UINT = 3;

/**
 Primitive float index in Kiwi's primitive table.
 */
const PRIMITIVE_FLOAT = 4;

/**
 Primitive string index in Kiwi's primitive table.
 */
const PRIMITIVE_STRING = 5;

/**
 Primitive int64 index in Kiwi's primitive table.
 */
const PRIMITIVE_INT64 = 6;

/**
 Primitive uint64 index in Kiwi's primitive table.
 */
const PRIMITIVE_UINT64 = 7;

/**
 Decodes a single value from a binary reader given its type code.
 
 @param reader - {@link BinaryReader}.
 
 @param schema - Parsed {@link KiwiSchema}.
 
 @param typeCode - Type code.
 
 @param depth - Recursion depth.
 
 @returns Decoded JavaScript value, or {@link KIWI_VALUE_ABSENT} when absent.
 
 @example
 ```ts
 decodeValue({ reader: createBinaryReader({ data: new Uint8Array([1]) }), schema: emptySchema(), typeCode: -1, depth: 0 });
 // true
 ```
 */
export function decodeValue(
  {
    reader,
    schema,
    typeCode,
    depth,
  }: {
    readonly depth: number;
    readonly reader: BinaryReader;
    readonly schema: KiwiSchema;
    readonly typeCode: number;
  },
): KiwiDecodedValue {
  if (depth > MAX_DECODE_DEPTH)
    return KIWI_VALUE_ABSENT;

  if (typeCode < 0)
    return decodePrimitive({
      reader,
      primitiveIndex: ~typeCode,
    },);

  /**
   Schema definition referenced by type code.
   */
  const definition = schema.definitions[typeCode];
  if (definition === undefined)
    return KIWI_VALUE_ABSENT;

  if (definition.kind === 'ENUM') {
    /**
     Enum wire value.
     */
    const value = reader.readVarUint();
    /**
     Matching enum field.
     */
    const enumField = definition.fields
      .find(function matchesValue(field,): boolean {
        return field.value === value;
      },);
    return enumField === undefined ? `${definition.name}(${value})` : `${definition.name}.${enumField.name}`;
  }

  if (definition.kind === 'STRUCT')
    return decodeStruct({
      reader,
      schema,
      def: definition,
      depth: depth + 1,
    },);

  return decodeMessage({
    reader,
    schema,
    def: definition,
    depth: depth + 1,
  },);
}

/**
 Decodes a Kiwi primitive value.
 
 @param reader - {@link BinaryReader}.
 
 @param primitiveIndex - Primitive index.
 
 @returns Decoded primitive value, or {@link KIWI_VALUE_ABSENT} when absent.
 
 @example
 ```ts
 decodePrimitive({ reader: createBinaryReader({ data: new Uint8Array([0]) }), primitiveIndex: 0 });
 // false
 ```
 */
function decodePrimitive(
  {
    reader,
    primitiveIndex,
  }: {
    readonly primitiveIndex: number;
    readonly reader: BinaryReader;
  },
): KiwiDecodedValue {
  if (primitiveIndex === PRIMITIVE_BOOL)
    return reader.readByte() !== 0;
  if (primitiveIndex === PRIMITIVE_BYTE)
    return reader.readByte();
  if ((primitiveIndex === PRIMITIVE_INT) || (primitiveIndex === PRIMITIVE_INT64))
    return reader.readVarInt();
  if ((primitiveIndex === PRIMITIVE_UINT) || (primitiveIndex === PRIMITIVE_UINT64))
    return reader.readVarUint();
  if (primitiveIndex === PRIMITIVE_FLOAT)
    return reader.readVarFloat();
  if (primitiveIndex === PRIMITIVE_STRING)
    return reader.readString();
  return KIWI_VALUE_ABSENT;
}

/**
 Decodes a Kiwi struct from a binary reader.
 
 @param reader - {@link BinaryReader}.
 
 @param schema - Parsed {@link KiwiSchema}.
 
 @param def - {@link KiwiStruct} definition.
 
 @param depth - Recursion depth.
 
 @returns Decoded struct as a plain object.
 
 @example
 ```ts
 decodeStruct({ reader: createBinaryReader({ data: new Uint8Array() }), schema: emptySchema(), def: { kind: 'STRUCT', name: 'Empty', fields: [] }, depth: 0 });
 // { __type: 'Empty' }
 ```
 */
export function decodeStruct(
  {
    reader,
    schema,
    def,
    depth,
  }: {
    readonly def: KiwiStruct;
    readonly depth: number;
    readonly reader: BinaryReader;
    readonly schema: KiwiSchema;
  },
): Record<string, unknown> {
  return Object.fromEntries([
    [
      '__type',
      def.name,
    ],
    ...def.fields
      .map(function decodedStructField(field,): readonly [
        string,
        unknown
      ] {
        return [
          field.name,
          reader.eof
            ? KIWI_VALUE_ABSENT
            : decodeFieldValue({
              reader,
              schema,
              field,
              depth,
            }),
        ];
      },),
  ],);
}

/**
 Decodes a Kiwi message from a binary reader.
 
 @param reader - {@link BinaryReader}.
 
 @param schema - Parsed {@link KiwiSchema}.
 
 @param def - Message {@link KiwiStruct} definition.
 
 @param depth - Recursion depth.
 
 @returns Decoded message as a plain object.
 
 @example
 ```ts
 decodeMessage({ reader: createBinaryReader({ data: new Uint8Array([0]) }), schema: emptySchema(), def: { kind: 'MESSAGE', name: 'Empty', fields: [] }, depth: 0 });
 // { __type: 'Empty' }
 ```
 */
export function decodeMessage(
  {
    reader,
    schema,
    def,
    depth,
  }: {
    readonly def: KiwiStruct;
    readonly depth: number;
    readonly reader: BinaryReader;
    readonly schema: KiwiSchema;
  },
): Record<string, unknown> {
  /**
   Lookup from wire tag to field metadata.
   */
  const fieldByTag = new Map(def.fields
    .map(function fieldEntry(field,): readonly [
      number,
      KiwiStructField
    ] {
      return [
        field.value,
        field,
      ];
    },),);
  /**
   Decoded message entries collected from present fields.
   */
  const entries: [
    string,
    unknown
  ][] = [[
    '__type',
    def.name,
  ],];

  while (!reader.eof) {
    /**
     Next wire tag.
     */
    const tag = reader.readVarUint();
    if (tag === 0)
      break;

    /**
     Field metadata for tag.
     */
    const field = fieldByTag.get(tag,);
    if (field === undefined)
      break;
    entries.push([
      field.name,
      decodeFieldValue({
        reader,
        schema,
        field,
        depth,
      }),
    ],);
  }

  return Object.fromEntries(entries,);
}

/**
 Decodes field value, including repeated fields.
 
 @param reader - {@link BinaryReader}.
 
 @param schema - Parsed {@link KiwiSchema}.
 
 @param field - {@link KiwiStructField} metadata.
 
 @param depth - Recursion depth.
 
 @returns Decoded field value.
 
 @example
 ```ts
 decodeFieldValue({ reader: createBinaryReader({ data: new Uint8Array([1]) }), schema: emptySchema(), field: { name: 'flag', type: -1, isArray: false, value: 1 }, depth: 0 });
 // true
 ```
 */
function decodeFieldValue(
  {
    reader,
    schema,
    field,
    depth,
  }: {
    readonly depth: number;
    readonly field: KiwiStructField;
    readonly reader: BinaryReader;
    readonly schema: KiwiSchema;
  },
): KiwiDecodedValue {
  if (!field.isArray)
    return decodeValue({
      reader,
      schema,
      typeCode: field.type,
      depth,
    },);

  /**
   Repeated field item count.
   */
  const count = reader.readVarUint();
  return Array.from(
    { length: count, },
    function decodeRepeatedValue(): KiwiDecodedValue {
      return decodeValue({
        reader,
        schema,
        typeCode: field.type,
        depth,
      },);
    },
  );
}

/**
 Decodes document data section of a Figma file.
 
 @param documentData - Document bytes.
 
 @param schema - Parsed {@link KiwiSchema}.
 
 @returns Decoded document object, or {@link FIGMA_DOCUMENT_ABSENT} when absent.
 
 @example
 ```ts
 decodeDocument({ documentData: new Uint8Array(), schema: emptySchema() });
 // FIGMA_DOCUMENT_ABSENT
 ```
 */
export function decodeDocument(
  {
    documentData,
    schema,
  }: {
    readonly documentData: Uint8Array;
    readonly schema: KiwiSchema;
  },
): Record<string, unknown> | typeof FIGMA_DOCUMENT_ABSENT {
  if (documentData.length === 0)
    return FIGMA_DOCUMENT_ABSENT;

  /**
   Reader over document bytes.
   */
  const reader = createBinaryReader({ data: documentData, },);
  /**
   Top-level message definition.
   */
  const messageDef = schema.structByName
    .get('Message',);
  if (messageDef === undefined)
    throw new Error('Message definition not found in schema');

  return decodeMessage({
    reader,
    schema,
    def: messageDef,
    depth: 0,
  },);
}

/**
 Creates empty schema for examples.
 
 @returns Empty {@link KiwiSchema}.
 
 @example
 ```ts
 emptySchema().definitions.length;
 // 0
 ```
 */
function emptySchema(): KiwiSchema {
  return {
    definitions: [],
    enumByName: new Map(),
    structByName: new Map(),
  };
}
