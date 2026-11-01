/**
 Binary reader for Kiwi varints, strings, and fixed-width values.
 
 @example
 ```ts
 const reader = createBinaryReader({ data: new Uint8Array([42]) });
 reader.readVarUint();
 // 42
 ```
 */

/**
 Low seven payload bits in each LEB128 byte.
 */
const VARUINT_PAYLOAD_MASK = 0x7F;

/**
 Continuation bit in each LEB128 byte.
 */
const VARUINT_CONTINUATION_BIT = 0x80;

/**
 Payload bit count contributed by each LEB128 byte.
 */
const VARUINT_CHUNK_BITS = 7;

/**
 Byte count in Kiwi's nonzero varfloat representation.
 */
const VARFLOAT_BYTE_LENGTH = 4;

/**
 Bit count in a byte.
 */
const BYTE_BITS = 8;

/**
 Bit rotation restoring Kiwi varfloat payloads to IEEE 754 layout.
 */
const VARFLOAT_LEFT_ROTATE_BITS = 23;

/**
 Complementary right rotation for Kiwi varfloat payloads.
 */
const VARFLOAT_RIGHT_ROTATE_BITS = 9;

/**
 Binary cursor over an immutable byte array.
 
 @example
 ```ts
 const reader = createBinaryReader({ data: new Uint8Array([0]) });
 reader.eof;
 // false
 ```
 */
export type BinaryReader = {
  readonly data: Uint8Array;
  readonly eof: boolean;
  readonly pos: number;
  readonly readByte: () => number;
  readonly readBytes: (count: number,) => Uint8Array;
  readonly readString: () => string;
  readonly readUint32LE: () => number;
  readonly readVarFloat: () => number;
  readonly readVarInt: () => number;
  readonly readVarUint: () => number;
  readonly remaining: number;
};

/**
 Reads byte at index or throws when the buffer is exhausted.
 
 @param data - Byte array.
 
 @param index - Requested index.
 
 @returns Byte value at index.
 
 @example
 ```ts
 byteAt({ data: new Uint8Array([1]), index: 0 });
 // 1
 ```
 */
function byteAt(
  {
    data,
    index,
  }: {
    readonly data: Uint8Array;
    readonly index: number;
  },
): number {
  /**
   Byte read from the requested index.
   */
  const value = data[index];
  if (value === undefined)
    throw new Error(`Unexpected end of Kiwi data at byte ${index}`);
  return value;
}

/**
 Creates a mutable Kiwi binary reader over data.
 
 @param data - Buffer.
 
 @param pos - Optional starting cursor.
 
 @returns {@link BinaryReader} object.
 
 @example
 ```ts
 const reader = createBinaryReader({ data: new Uint8Array([0]) });
 reader.readByte();
 // 0
 ```
 */
export function createBinaryReader(
  {
    data,
    pos = 0,
  }: {
    readonly data: Uint8Array;
    readonly pos?: number;
  },
): BinaryReader {
  /**
   Mutable cursor state hidden behind reader methods.
   */
  const state = { cursor: pos, };
  /**
   Reader object whose methods share and advance one cursor.
   */
  const reader: BinaryReader = {
    data,
    /**
     Current cursor position.
     */
    get pos(): number {
      return state.cursor;
    },
    /**
     Whether cursor reached end of input.
     */
    get eof(): boolean {
      return state.cursor
        >= data
          .length;
    },
    /**
     Remaining unread byte count.
     */
    get remaining(): number {
      return data
        .length
        - state.cursor;
    },
    readByte(): number {
      /**
       Byte at current cursor.
       */
      const value = byteAt({
        data,
        index: state.cursor,
      },);
      state.cursor += 1;
      return value;
    },
    readBytes(count: number,): Uint8Array {
      /**
       Start cursor for returned slice.
       */
      const start = state.cursor;
      state.cursor += count;
      return data
        .subarray(
        start,
        start + count,
        );
    },
    readString(): string {
      /**
       Terminator offset found by a bounded scan.
       */
      const end = findNullTerminator({
        data,
        start: state.cursor,
      },);
      /**
       UTF-8 decoded bytes before the terminator.
       */
      const value = new TextDecoder('utf-8',).decode(data
        .subarray(
        state.cursor,
        end,
        ),);
      state.cursor = end + 1;
      return value;
    },
    readUint32LE(): number {
      /**
       View over the next uint32 payload.
       */
      const view = new DataView(
        data
          .buffer,
        data
          .byteOffset
          + state.cursor,
        VARFLOAT_BYTE_LENGTH,
      );
      state.cursor += VARFLOAT_BYTE_LENGTH;
      return view.getUint32(
        0,
        true,
      );
    },
    readVarFloat(): number {
      return readVarFloat({ reader: this, },);
    },
    readVarInt(): number {
      /**
       Raw zigzag-encoded integer.
       */
      const raw = this.readVarUint();
      if ((raw & 1) === 1)
        return ~(Math.trunc(raw / 2));
      return Math.trunc(raw / 2);
    },
    readVarUint(): number {
      return readVarUint({ reader: this, },);
    },
  };
  return reader;
}

/**
 Finds null terminator starting at a cursor.
 
 @param data - Buffer.
 
 @param start - Start cursor.
 
 @returns Terminator offset, or buffer length when absent.
 
 @example
 ```ts
 findNullTerminator({ data: new Uint8Array([65, 0]), start: 0 });
 // 1
 ```
 */
function findNullTerminator(
  {
    data,
    start,
  }: {
    readonly data: Uint8Array;
    readonly start: number;
  },
): number {
  for (let index = start; index < data.length; index++) {
    if (data[index] === 0)
      return index;
  }
  return data.length;
}

/**
 Reads unsigned LEB128 varint from reader.
 
 @param reader - {@link BinaryReader} to advance.
 
 @returns Unsigned integer value.
 
 @example
 ```ts
 readVarUint({ reader: createBinaryReader({ data: new Uint8Array([128, 1]) }) });
 // 128
 ```
 */
function readVarUint({ reader, }: { readonly reader: BinaryReader; },): number {
  for (let result = 0, shift = 0; !reader.eof; shift += VARUINT_CHUNK_BITS) {
    /**
     Current LEB128 byte.
     */
    const byte = reader.readByte();
    result += (byte & VARUINT_PAYLOAD_MASK) * (2 ** shift);
    if ((byte & VARUINT_CONTINUATION_BIT) === 0)
      return Math.trunc(result);
  }
  return 0;
}

/**
 Reads Kiwi varfloat from reader.
 
 @param reader - {@link BinaryReader} to advance.
 
 @returns Decoded float32 value.
 
 @example
 ```ts
 readVarFloat({ reader: createBinaryReader({ data: new Uint8Array([0]) }) });
 // 0
 ```
 */
function readVarFloat({ reader, }: { readonly reader: BinaryReader; },): number {
  /**
   First byte, special-casing zero.
   */
  const first = reader.readByte();
  if (first === 0)
    return 0;

  /**
   Remaining three bytes after the already-consumed first byte.
   */
  const rest = reader.readBytes(VARFLOAT_BYTE_LENGTH - 1,);
  /**
   Raw rotated uint32 assembled from little-endian bytes.
   */
  const rawBits = first
    | (byteAt({
      data: rest,
      index: 0,
    },) << BYTE_BITS)
    | (byteAt({
      data: rest,
      index: 1,
    },) << (BYTE_BITS * 2))
    | (byteAt({
      data: rest,
      index: 2,
    },) << (BYTE_BITS * (VARFLOAT_BYTE_LENGTH - 1)));
  /**
   IEEE 754 float bits after undoing Kiwi rotation.
   */
  const bits = Math.trunc(
    ((rawBits << VARFLOAT_LEFT_ROTATE_BITS) | (rawBits >>> VARFLOAT_RIGHT_ROTATE_BITS)),
  );
  /**
   Scratch view used to reinterpret uint32 bits as float32.
   */
  const view = new DataView(new ArrayBuffer(VARFLOAT_BYTE_LENGTH,),);
  view.setUint32(
    0,
    bits,
    true,
  );
  return view.getFloat32(
    0,
    true,
  );
}
