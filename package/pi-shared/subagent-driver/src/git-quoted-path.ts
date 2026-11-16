/**
 Reads a path git wrote quoted, which it does for a name holding a non-ASCII byte, a quote, a control character or a space at an
 edge: a C-style string with `\n`, `\t`, `\"`, `\\` and three octal digits per byte.

 The path is decoded from its bytes as UTF-8, so the path a reviewer copies and the path a diff names compare as the same text.

 @module
 */

/**
 The byte each escape git writes stands for.
 */
const ESCAPED_BYTES: ReadonlyMap<string, number> = new Map([
  [
    'n',
    '\n'.codePointAt(0,) ?? 0,
  ],
  [
    't',
    '\t'.codePointAt(0,) ?? 0,
  ],
  [
    '"',
    '"'.codePointAt(0,) ?? 0,
  ],
  [
    '\\',
    '\\'.codePointAt(0,) ?? 0,
  ],
  [
    'r',
    '\r'.codePointAt(0,) ?? 0,
  ],
  [
    'b',
    '\b'.codePointAt(0,) ?? 0,
  ],
  [
    'f',
    '\f'.codePointAt(0,) ?? 0,
  ],
]);

/**
 What an escape git does not write stands for, so the escape is kept as written.
 */
const NOT_AN_ESCAPE = -1;

/**
 How many characters a three-digit octal escape takes.
 */
const OCTAL_ESCAPE_LENGTH = 4;

/**
 How many digits an octal escape holds.
 */
const OCTAL_DIGITS = 3;

/**
 The base an octal escape's digits are read in.
 */
const OCTAL_RADIX = 8;

/**
 Reads a path a review or a diff wrote, without the quotes git may put around it.

 @param path - the path as written

 @returns the path itself

 @example
 ```ts
 const path = pathOf('"package/p/r\\303\\251sum\\303\\251.ts"');
 ```
 */
export function pathOf(path: string,): string {
  /**
   The path as written, without the spaces around it.
   */
  const trimmed = path.trim();
  if ((!trimmed.startsWith('"',)) || (!trimmed.endsWith('"',))
    || (trimmed.length < 2)) {
    return trimmed;
  }
  /**
   The path without its quotes.
   */
  const quoted = trimmed.slice(
    1,
    -1,
  );
  /**
   The bytes read so far, kept as bytes so a UTF-8 sequence decodes as one path.
   */
  const bytes: number[] = [];
  /**
   Where the next character is read from.
   */
  const cursor = { at: 0, };
  while (cursor.at < quoted.length) {
    /**
     The character this step reads.
     */
    const character = quoted[cursor.at] ?? '';
    if (character !== '\\') {
      bytes.push(...Buffer.from(
        character,
        'utf8',
      ),);
      cursor.at += 1;
      continue;
    }
    /**
     What the escape stands for, when it is one of git's own.
     */
    const next = quoted[cursor.at + 1] ?? '';
    /**
     The byte the escape stands for, or minus one when it stands for none.
     */
    const escaped = ESCAPED_BYTES.get(next,) ?? NOT_AN_ESCAPE;
    if (escaped !== NOT_AN_ESCAPE) {
      bytes.push(escaped,);
      cursor.at += 2;
      continue;
    }
    /**
     The three octal digits of a byte, when the escape is one.
     */
    const octal = quoted.slice(
      cursor.at + 1,
      cursor.at + OCTAL_ESCAPE_LENGTH,
    );
    if ((octal.length === OCTAL_DIGITS)
      && octal.split('',)
        .every(function isOctal(digit,) {
          return (digit >= '0') && (digit <= '7');
        },)) {
      bytes.push(Number.parseInt(
        octal,
        OCTAL_RADIX,
      ),);
      cursor.at += OCTAL_ESCAPE_LENGTH;
      continue;
    }
    // An escape this reader does not know is kept as written, so the path is still visible.
    bytes.push(...Buffer.from(
      character,
      'utf8',
    ),);
    cursor.at += 1;
  }
  return Buffer.from(bytes,)
    .toString('utf8',);
}
