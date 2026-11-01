/**
 Computes SHA-256 hash of a string and returns hexadecimal representation.
 
 Uses the Web Crypto API to create a cryptographically secure hash suitable for
 data integrity verification, unique identifiers, or security applications.
 
 Internal helper for deriving stable keys from content when an empty key is
 passed to the async store; not part of the package public API.
 
 @param value - string to hash
 
 @returns hexadecimal string representation of the SHA-256 hash
 
 @example
 ```ts
 const hash = await hashString('hello world');
 console.log(hash); // "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
 ```
 */
export async function hashString(value: string,): Promise<string> {
  /**
   Text encoder for converting string to UTF-8 bytes
   */
  const encoder = new TextEncoder();
  /**
   UTF-8 encoded bytes of the input string
   */
  const data = encoder.encode(value,);
  /**
   SHA-256 hash digest as ArrayBuffer
   */
  const hashBuffer = await crypto.subtle
    .digest(
    'SHA-256',
    data,
    );
  /**
   Convert ArrayBuffer to array of bytes for processing
   */
  const hashArray = [...new Uint8Array(hashBuffer,),];
  /**
   Convert bytes to hexadecimal string with zero-padding
   */
  const hashHex = hashArray
    .map(function toHex(b,) {
      /**
       Hexadecimal base for byte-to-hex conversion
       */
      const HEX_BASE = 16;
      return b.toString(HEX_BASE,)
        .padStart(
        2,
        '0',
        );
    },)
    .join('',);
  return hashHex;
}
