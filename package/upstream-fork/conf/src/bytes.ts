/**
 Byte and text conversions for encrypted config payloads.
 
 Replaces upstream `conf`'s `uint8array-extras` dependency with the runtime's
 own `TextEncoder`/`TextDecoder` and one concat,
 so the encryption path has no third-party conversion code.
 
 @module
 */

//region Constants

/**
 UTF-8 encoder shared by every payload conversion.
 
 @example
 ```ts
 stringToBytes('hi'); // => Uint8Array [104, 105]
 ```
 */
const utf8Encoder = new TextEncoder();

/**
 UTF-8 decoder shared by every payload conversion.
 
 @example
 ```ts
 bytesToString(new Uint8Array([104, 105])); // 'hi'
 ```
 */
const utf8Decoder = new TextDecoder();

//endregion Constants

//region Conversions

/**
 Encodes text to UTF-8 bytes.
 
 @param text - Plaintext to encode.
 
 @returns UTF-8 encoding of `text`.
 
 @example
 ```ts
 stringToBytes('hi'); // => Uint8Array [104, 105]
 ```
 */
export function stringToBytes(text: string,): Uint8Array {
  return utf8Encoder.encode(text,);
}

/**
 Decodes UTF-8 bytes to text.
 
 @param bytes - Bytes to decode.
 
 @returns UTF-8 decoding of `bytes`.
 
 @example
 ```ts
 bytesToString(new Uint8Array([104, 105])); // 'hi'
 ```
 */
export function bytesToString(bytes: Uint8Array,): string {
  return utf8Decoder.decode(bytes,);
}

/**
 Concatenates byte chunks into one buffer.
 
 @param chunks - Consecutive byte chunks.
 
 @returns One buffer holding every chunk in order.
 
 @example
 ```ts
 concatBytes([
   new Uint8Array([1]),
   new Uint8Array([2]),
 ]); // => Uint8Array [1, 2]
 ```
 */
export function concatBytes(chunks: readonly Uint8Array[],): Uint8Array {
  /**
   Total byte count every chunk contributes to the result.
   */
  const totalLength = chunks.reduce(
    function sumLength(
      total: number,
      chunk: Uint8Array,
    ): number {
      return total + chunk.length;
    },
    0,
  );
  /**
   Result buffer filled left to right by the copy loop below.
   */
  const joined = new Uint8Array(totalLength,);
  /**
   Byte offset where the next chunk lands.
   */
  const cursor = {
    offset: 0,
  };
  for (const chunk of chunks) {
    joined.set(
      chunk,
      cursor.offset,
    );
    cursor.offset += chunk.length;
  }
  return joined;
}

//endregion Conversions
