/**
 Filename and digest primitives shared by the fingerprinting post-process.
 
 Split out of `postprocess.ts` to keep that orchestrator under the
 max-lines budget; these helpers carry no pipeline state.
 */
import { createHash, } from 'node:crypto';

//region Fingerprint suffix

/**
 Number of hex characters to use from the SHA-256 digest.
 */
const HASH_LENGTH = 10;

/**
 Lower-case hex digit the digest is written with, and the only alphabet a
 fingerprint segment may use.
 */
const LOWEST_HEX_DIGIT = '0';

/**
 Highest digit of the decimal range inside the hex alphabet.
 */
const HIGHEST_DECIMAL_DIGIT = '9';

/**
 Lowest letter of the hex alphabet.
 */
const LOWEST_HEX_LETTER = 'a';

/**
 Highest letter of the hex alphabet.
 */
const HIGHEST_HEX_LETTER = 'f';

/**
 Tests one character against the hex alphabet the digest is written with.
 
 @param character - character to test
 
 @returns whether the character is a lower-case hex digit
 */
function isLowercaseHexDigit(character: string,): boolean {
  if ((character >= LOWEST_HEX_DIGIT) && (character <= HIGHEST_DECIMAL_DIGIT))
    return true;
  return (character >= LOWEST_HEX_LETTER) && (character <= HIGHEST_HEX_LETTER);
}

/**
 Tests a slice for being exactly the digest prefix a fingerprint carries.
 
 @param segment - slice of a basename to test
 
 @returns whether the slice is {@link HASH_LENGTH} lower-case hex digits
 */
function isFingerprintSegment(segment: string,): boolean {
  if (segment.length !== HASH_LENGTH)
    return false;
  for (const character of segment) {
    if (!isLowercaseHexDigit(character,))
      return false;
  }
  return true;
}

/**
 Tests whether a basename already carries a fingerprint.
 
 The shape is the one {@link insertHash} writes: a dot, {@link HASH_LENGTH}
 lower-case hex digits, a dot, and an extension holding no further dot. Reading
 it back this way compares against the same length constant the insert uses, so
 a basename from another length can never pass.
 
 @param name - basename to test, without directories
 
 @returns whether the name is already fingerprinted
 
 @example
 ```ts
 isFingerprintedName('styles.a1b2c3d4ef.css'); // true
 isFingerprintedName('styles.css');            // false
 ```
 */
export function isFingerprintedName(name: string,): boolean {
  /**
   Position of the dot that starts the extension, which is the last dot in a
   fingerprint's tail because the extension holds no further one.
   */
  const extensionDot = name.lastIndexOf('.',);
  if (extensionDot === (-1))
    return false;
  if (extensionDot === (name.length - 1))
    return false;
  /**
   Position of the dot that precedes the digest; absent when the tail is shorter
   than a digest.
   */
  const digestDot = extensionDot
    - HASH_LENGTH
    - 1;
  if (digestDot < 0)
    return false;
  if (name.charAt(digestDot,) !== '.')
    return false;
  return isFingerprintSegment(
    name.slice(
      digestDot + 1,
      extensionDot,
    ),
  );
}

//endregion Fingerprint suffix

/**
 Computes a SHA-256 hex digest of a Buffer.
 
 @param input - binary data to hash
 
 @mutates input through createHash('sha256',).update native-boundary byte access
 
 @returns hex-encoded SHA-256 digest
 
 @example
 ```ts
 const hash = sha256Buffer(await readFile('image.avif'));
 ```
 */
export function sha256Buffer(input: Buffer,): string {
  return createHash('sha256',)
    .update(input,)
    .digest('hex',);
}

/**
 Inserts a content hash before the file extension.
 
 @param name - original filename (basename only, no directory)
 
 @param hash - full hex hash (sliced to HASH_LENGTH internally)
 
 @returns filename with hash inserted before the last extension
 
 @example
 ```ts
 insertHash({ name: 'styles.css', hash: 'a1b2c3d4ef9876543210' });
 // → 'styles.a1b2c3d4ef.css'
 ```
 */
export function insertHash(
  {
    name,
    hash,
  }: {
    readonly name: string;
    readonly hash: string;
  },
): string {
  /**
   Position of the final dot; `-1` indicates the file has no extension.
   */
  const lastDot = name.lastIndexOf('.',);
  if (lastDot === (-1)) {
    return `${name}.${
      hash.slice(
        0,
        HASH_LENGTH,
      )
    }`;
  }
  /**
   Filename portion before the extension; used as the prefix for the hashed name.
   */
  const stem = name.slice(
    0,
    lastDot,
  );
  /**
   Extension portion including the leading dot; appended after the hash.
   */
  const ext = name.slice(lastDot,);
  return `${stem}.${
    hash.slice(
      0,
      HASH_LENGTH,
    )
  }${ext}`;
}
