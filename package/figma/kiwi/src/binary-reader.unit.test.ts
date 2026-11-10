import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { createBinaryReader, } from '../dist/final/node/index.mjs';

/**
 Two-byte LEB128 varuint encoding of 128: one continuation byte and one terminator.
 */
const MULTI_BYTE_VARUINT = new Uint8Array([0x80, 0x01,],);

/**
 Four-byte Kiwi varfloat encoding of 1.0: the IEEE 754 bits rotated left by 23.
 */
const ONE_POINT_ZERO_VARFLOAT = new Uint8Array([0x7F, 0x00, 0x00, 0x00,],);

/**
 UTF-8 bytes of the two-byte string `é` followed by its null terminator.
 */
const TERMINATED_STRING = new Uint8Array([0xC3, 0xA9, 0x00,],);

/**
 ASCII bytes with no null terminator, standing for a string cut off mid-payload.
 */
const UNTERMINATED_STRING = new Uint8Array([0x61, 0x62,],);

/**
 Little-endian bytes of the uint32 0x01_02_03_04.
 */
const LITTLE_ENDIAN_UINT32 = new Uint8Array([0x04, 0x03, 0x02, 0x01,],);

await describe({
  name: createBinaryReader.name,
  children: [
    describe({
      name: 'readByte',
      children: [
        it({
          name: 'returns the byte under the cursor and advances it',
          fn: async () => {
            const reader = createBinaryReader({ data: new Uint8Array([7, 8,],), },);
            expect(reader.readByte(),).toBe(7,);
            expect(reader.pos,).toBe(1,);
            expect(reader.remaining,).toBe(1,);
            expect(reader.eof,).toBe(false,);
          },
        },),
        it({
          name: 'throws naming the byte index when the buffer is exhausted',
          fn: async () => {
            const reader = createBinaryReader({ data: new Uint8Array([7,],), },);
            reader.readByte();
            expect(() => reader.readByte(),).toThrow('Unexpected end of Kiwi data: byte 1 does not exist',);
          },
        },),
      ],
    },),
    describe({
      name: 'readVarUint',
      children: [
        it({
          name: 'decodes a single-byte varuint and reaches the end of the buffer',
          fn: async () => {
            const reader = createBinaryReader({ data: new Uint8Array([0x2A,],), },);
            expect(reader.readVarUint(),).toBe(0x2A,);
            expect(reader.eof,).toBe(true,);
          },
        },),
        it({
          name: 'decodes a multi-byte varuint from its continuation bits',
          fn: async () => {
            const reader = createBinaryReader({ data: MULTI_BYTE_VARUINT, },);
            expect(reader.readVarUint(),).toBe(128,);
          },
        },),
        it({
          name: 'throws when the buffer ends before the terminating byte',
          fn: async () => {
            expect(() => createBinaryReader({ data: new Uint8Array([0x80,],), },).readVarUint(),)
              .toThrow('Unexpected end of Kiwi data',);
            expect(() => createBinaryReader({ data: new Uint8Array([0x80,],), },).readVarUint(),)
              .toThrow('varuint starting at byte 0',);
          },
        },),
        it({
          name: 'throws for a varuint cut off after several continuation bytes',
          fn: async () => {
            expect(() => createBinaryReader({ data: new Uint8Array([0xFF, 0xFF, 0xFF,],), },).readVarUint(),)
              .toThrow('varuint starting at byte 0',);
          },
        },),
      ],
    },),
    describe({
      name: 'readVarInt',
      children: [
        it({
          name: 'decodes zero from a zero varuint',
          fn: async () => {
            const reader = createBinaryReader({ data: new Uint8Array([0,],), },);
            expect(reader.readVarInt(),).toBe(0,);
          },
        },),
        it({
          name: 'decodes a positive varuint without flipping its sign',
          fn: async () => {
            const reader = createBinaryReader({ data: MULTI_BYTE_VARUINT, },);
            expect(reader.readVarInt(),).toBe(64,);
          },
        },),
        it({
          name: 'decodes an odd zigzag varuint as the matching negative number',
          fn: async () => {
            const reader = createBinaryReader({ data: new Uint8Array([0x01,],), },);
            expect(reader.readVarInt(),).toBe(-1,);
          },
        },),
        it({
          name: 'decodes a multi-byte odd zigzag varuint as the matching negative number',
          fn: async () => {
            const reader = createBinaryReader({ data: new Uint8Array([0x81, 0x01,],), },);
            expect(reader.readVarInt(),).toBe(-65,);
          },
        },),
      ],
    },),
    describe({
      name: 'readVarFloat',
      children: [
        it({
          name: 'decodes a rotated varfloat payload as its IEEE 754 float',
          fn: async () => {
            const reader = createBinaryReader({ data: ONE_POINT_ZERO_VARFLOAT, },);
            expect(reader.readVarFloat(),).toBe(1,);
            expect(reader.eof,).toBe(true,);
          },
        },),
        it({
          name: 'reads a zero float from a single byte without consuming the next value',
          fn: async () => {
            const reader = createBinaryReader({ data: new Uint8Array([0x00, 0x2A,],), },);
            expect(reader.readVarFloat(),).toBe(0,);
            expect(reader.readByte(),).toBe(0x2A,);
          },
        },),
        it({
          name: 'throws when fewer than four bytes remain',
          fn: async () => {
            const reader = createBinaryReader({ data: new Uint8Array([0x7F, 0x00,],), },);
            expect(() => reader.readVarFloat(),).toThrow('Unexpected end of Kiwi data',);
          },
        },),
      ],
    },),
    describe({
      name: 'readString',
      children: [
        it({
          name: 'decodes UTF-8 bytes before the null terminator and skips it',
          fn: async () => {
            const reader = createBinaryReader({ data: TERMINATED_STRING, },);
            expect(reader.readString(),).toBe('é',);
            expect(reader.eof,).toBe(true,);
          },
        },),
        it({
          name: 'reads an empty string from a leading terminator',
          fn: async () => {
            const reader = createBinaryReader({ data: new Uint8Array([0x00, 0x2A,],), },);
            expect(reader.readString(),).toBe('',);
            expect(reader.readByte(),).toBe(0x2A,);
          },
        },),
        it({
          name: 'throws when the input ends without a null terminator',
          fn: async () => {
            expect(() => createBinaryReader({ data: UNTERMINATED_STRING, },).readString(),)
              .toThrow('Unexpected end of Kiwi data',);
            expect(() => createBinaryReader({ data: UNTERMINATED_STRING, },).readString(),)
              .toThrow('string starting at byte 0',);
          },
        },),
      ],
    },),
    describe({
      name: 'readBytes',
      children: [
        it({
          name: 'returns exactly the requested bytes and advances the cursor',
          fn: async () => {
            const reader = createBinaryReader({ data: new Uint8Array([1, 2, 3,],), },);
            expect([...reader.readBytes(2,),],).toEqual([1, 2,],);
            expect(reader.remaining,).toBe(1,);
          },
        },),
        it({
          name: 'throws instead of clamping when the request exceeds the buffer',
          fn: async () => {
            const shortReader = () => createBinaryReader({ data: new Uint8Array([1, 2,],), },);
            expect(() => shortReader().readBytes(10,),).toThrow('Unexpected end of Kiwi data',);
            expect(() => shortReader().readBytes(10,),).toThrow('readBytes(10) at byte 0 needs 10 bytes',);
          },
        },),
        it({
          name: 'throws when the request exceeds a partly consumed buffer',
          fn: async () => {
            const reader = createBinaryReader({ data: new Uint8Array([1, 2, 3,],), },);
            reader.readByte();
            expect(() => reader.readBytes(3,),).toThrow('readBytes(3) at byte 1 needs 3 bytes',);
          },
        },),
      ],
    },),
    describe({
      name: 'readUint32LE',
      children: [
        it({
          name: 'reads four little-endian bytes',
          fn: async () => {
            const reader = createBinaryReader({ data: LITTLE_ENDIAN_UINT32, },);
            expect(reader.readUint32LE(),).toBe(0x01_02_03_04,);
            expect(reader.eof,).toBe(true,);
          },
        },),
        it({
          name: 'throws when fewer than four bytes remain',
          fn: async () => {
            const reader = createBinaryReader({ data: new Uint8Array([1, 2, 3,],), },);
            expect(() => reader.readUint32LE(),).toThrow('Unexpected end of Kiwi data',);
          },
        },),
        it({
          name: 'throws rather than reading past the end of a view into a longer buffer',
          fn: async () => {
            const reader = createBinaryReader({ data: new Uint8Array([1, 2, 3, 4, 5, 6,],).subarray(0, 2,), },);
            expect(() => reader.readUint32LE(),).toThrow('Unexpected end of Kiwi data',);
          },
        },),
        it({
          name: 'reads from a view that starts inside a longer buffer',
          fn: async () => {
            const reader = createBinaryReader({ data: new Uint8Array([9, 9, 0x04, 0x03, 0x02, 0x01,],).subarray(2,), },);
            expect(reader.readUint32LE(),).toBe(0x01_02_03_04,);
          },
        },),
      ],
    },),
    describe({
      name: 'cursor accessors',
      children: [
        it({
          name: 'starts at the requested position',
          fn: async () => {
            const reader = createBinaryReader({
              data: new Uint8Array([1, 2, 3,],),
              pos: 2,
            },);
            expect(reader.pos,).toBe(2,);
            expect(reader.remaining,).toBe(1,);
            expect(reader.eof,).toBe(false,);
          },
        },),
        it({
          name: 'reports end of input for an empty buffer',
          fn: async () => {
            const reader = createBinaryReader({ data: new Uint8Array(), },);
            expect(reader.eof,).toBe(true,);
            expect(reader.remaining,).toBe(0,);
          },
        },),
      ],
    },),
  ],
},);
