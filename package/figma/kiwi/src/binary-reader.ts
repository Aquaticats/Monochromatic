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
 Byte count of a little-endian uint32.
 */
const UINT32_BYTE_LENGTH = 4;

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
 Builds the error a read throws when it would run past the end of the buffer.
 All truncation failures in this module share one message shape, so a caller can
 recognize a truncated file from the prefix alone.
 
 @param detail - What the read wanted and what was left.
 
 @returns Error naming the input and the requested read.
 
 @example
 ```ts
 truncationError({ detail: 'readBytes(10) at byte 0 needs 10 bytes, 2 remain' });
 ```
 */
function truncationError({ detail, }: { readonly detail: string; },): Error {
  return new Error(`Unexpected end of Kiwi data: ${detail}`);
}

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
    throw truncationError({ detail: `byte ${index} does not exist`, },);
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
      if ((start + count) > data.length)
        throw truncationError({ detail: `readBytes(${count}) at byte ${start} needs ${count} bytes, ${data.length - start} remain`, },);
      state.cursor += count;
      return data
        .subarray(
        start,
        start + count,
        );
    },
    readString(): string {
      /**
       Start cursor of the string payload.
       */
      const start = state.cursor;
      /**
       Terminator offset, or a negative index when the payload runs to the end.
       */
      const end = data.indexOf(
        0,
        start,
      );
      if (end === (-1))
        throw truncationError({ detail: `string starting at byte ${start} has no null terminator`, },);
      /**
       UTF-8 decoded bytes before the terminator.
       */
      const value = new TextDecoder('utf-8',).decode(data
        .subarray(
        start,
        end,
        ),);
      state.cursor = end + 1;
      return value;
    },
    readUint32LE(): number {
      /**
       Start cursor of the uint32 payload.
       */
      const start = state.cursor;
      if ((start + UINT32_BYTE_LENGTH) > data.length)
        throw truncationError({ detail: `uint32 at byte ${start} needs ${UINT32_BYTE_LENGTH} bytes, ${data.length - start} remain`, },);
      /**
       View over the next uint32 payload.
       */
      const view = new DataView(
        data
          .buffer,
        data
          .byteOffset
          + start,
        UINT32_BYTE_LENGTH,
      );
      state.cursor += UINT32_BYTE_LENGTH;
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
  /**
   Offset of the varuint's first byte, for the truncation message.
   */
  const start = reader.pos;
  for (let result = 0, shift = 0; !reader.eof; shift += VARUINT_CHUNK_BITS) {
    /**
     Current LEB128 byte.
     */
    const byte = reader.readByte();
    result += (byte & VARUINT_PAYLOAD_MASK) * (2 ** shift);
    if ((byte & VARUINT_CONTINUATION_BIT) === 0)
      return Math.trunc(result);
  }
  throw truncationError({ detail: `varuint starting at byte ${start} has no terminator`, },);
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
