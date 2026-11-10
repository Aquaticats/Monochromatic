/**
 The sha256 digest of a client token, and the check that a `clients` row holds one.

 A client token is 32 random bytes, so a plain digest already makes a leaked row
 useless: there is no low-entropy secret to guess and no salt to store next to it.
 Digesting the presented token also fixes the compared values at one length, which
 is what the constant-time comparison needs.

 @module
 */

/**
 Length of a sha256 digest in lowercase hex characters.
 */
export const SHA256_HEX_LENGTH = 64;

/**
 Lowercase hex digits a digest may hold.
 */
const HEX_DIGITS = '0123456789abcdef';

/**
 Whether `character` is one lowercase hex digit.

 @param character - one UTF-16 code unit

 @returns `true` for `0` to `9` and `a` to `f`
 */
function isLowercaseHexDigit(character: string,): boolean {
  return HEX_DIGITS.includes(character,);
}

/**
 Spell one byte as the two lowercase hex characters a digest is written with.

 @param byte - one byte of a digest

 @returns the byte as two hex characters, zero-padded

 @example
 ```ts
 toHexPair(10); // '0a'
 ```
 */
function toHexPair(byte: number,): string {
  /**
   Byte in base sixteen.
   */
  const hex = byte.toString(16,);
  return hex.padStart(
    2,
    '0',
  );
}

/**
 Whether `value` is exactly one lowercase hex sha256 digest.

 @param value - text to inspect

 @returns `true` when the text is 64 lowercase hex characters

 @example
 ```ts
 isSha256Hex(await sha256Hex('abc')); // true
 isSha256Hex('ABC'); // false
 ```
 */
export function isSha256Hex(value: string,): boolean {
  if (value.length !== SHA256_HEX_LENGTH) {
    return false;
  }
  for (const character of value) {
    if (!isLowercaseHexDigit(character,)) {
      return false;
    }
  }
  return true;
}

/**
 Digest `text` with sha256.

 @param text - UTF-8 text to digest; a token or other secret, never a stored digest

 @returns lowercase hex digest

 @example
 ```ts
 await sha256Hex('abc');
 // 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad'
 ```
 */
export async function sha256Hex(text: string,): Promise<string> {
  /**
   UTF-8 bytes of `text`, which is what sha256 digests.
   */
  const bytes = new TextEncoder()
    .encode(text,);
  /**
   Web Crypto surface the digest is asked of.
   */
  const {subtle} = crypto;
  /**
   Digest of the UTF-8 encoding of `text`.
   */
  const digest = await subtle.digest(
    'SHA-256',
    bytes,
  );
  return [...new Uint8Array(digest,),]
    .map(toHexPair,)
    .join('',);
}
