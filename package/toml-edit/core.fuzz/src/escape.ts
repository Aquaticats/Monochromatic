/**
 Independent TOML basic-string escaper for the fuzz generators.
 
 The generators must turn an arbitrary JS string into a valid TOML basic
 string literal so the parser can be asked to decode it back. Reusing the
 package's own emitter would make that test circular (the emitter is itself
 under test), so this module is a deliberately separate, minimal encoder
 verified against the parser in `generators.unit.test.ts`.
 
 It follows the TOML 1.0 basic-string rules: backslash and double quote take
 their short escapes, the named control escapes (`\b \t \n \f \r`) are used
 where they apply, and every other control scalar below U+0020 plus U+007F is
 emitted as a `\uXXXX` sequence. All other scalars, including astral
 characters, pass through verbatim because TOML basic strings accept raw
 Unicode.
 
 @module
 */

/**
 Highest control codepoint that must be escaped at the low end of the table.
 */
const LOW_CONTROL_MAX = 0x1F;

/**
 Delete control codepoint (U+007F), escaped for safety even though some
 parsers tolerate it raw.
 */
const DELETE_CODEPOINT = 0x7F;

/**
 Single-quote codepoint (U+0027), which a literal string cannot contain.
 */
const SINGLE_QUOTE_CODEPOINT = 0x27;

/**
 Number of hex digits in a short `\uXXXX` escape.
 */
const SHORT_UNICODE_WIDTH = 4;

/**
 Named single-character escapes keyed by codepoint.
 */
const NAMED_ESCAPES: Record<number, string> = {
  0x08: String.raw`\b`,
  0x09: String.raw`\t`,
  0x0A: String.raw`\n`,
  0x0C: String.raw`\f`,
  0x0D: String.raw`\r`,
  0x22: String.raw`\"`,
  0x5C: String.raw`\\`,
};

/**
 Emit one scalar as its `\uXXXX` escape.
 
 @returns Six-character escape such as `\u0000`.
 */
function unicodeEscape({ codepoint, }: { readonly codepoint: number; },): string {
  return `\\u${codepoint.toString(16,)
    .toUpperCase()
    .padStart(
      SHORT_UNICODE_WIDTH,
      '0',
    )}`;
}

/**
 Escape one Unicode scalar for a TOML basic string.
 
 @returns Escaped fragment, possibly the scalar itself.
 */
function escapeScalar({ scalar, }: { readonly scalar: string; },): string {
  /**
   Codepoint of the scalar; `for...of` yields whole scalars so astral
   characters arrive as a single two-unit string with a codepoint above
   the BMP, which needs no escaping.
   */
  const codepoint = scalar.codePointAt(0,) ?? 0;
  /**
   Named escape for this codepoint, if one applies.
   */
  const named = NAMED_ESCAPES[codepoint];
  if (named !== undefined) return named;
  if ((codepoint <= LOW_CONTROL_MAX) || (codepoint === DELETE_CODEPOINT)) {
    return unicodeEscape({ codepoint, },);
  }
  return scalar;
}

/**
 Encode `content` as a complete TOML basic-string literal, quotes included.
 
 @param content - Arbitrary JS string to encode.
 
 @returns Quoted basic-string literal whose parsed value is `content`.
 
 @example
 ```ts
 basicStringLiteral({ content: 'a"b\n', },); // '"a\\"b\\n"'
 ```
 */
export function basicStringLiteral({ content, }: { readonly content: string; },): string {
  /**
   Per-scalar escaped fragments joined into the quoted body. `Array.from` with
   a mapper iterates whole code points without the string-spread the linter
   rejects for surrogate safety.
   */
  const body = Array.from(
    content,
    function each(scalar,) {
      return escapeScalar({ scalar, },);
    },
  )
    .join('',);
  return `"${body}"`;
}

/**
 Whether `content` is representable as a TOML literal (single-quoted) string.
 
 Literal strings perform no escaping, so they cannot hold a single quote, a
 newline, a carriage return, or any control scalar.
 
 @returns Whether a literal-string encoding is available for `content`.
 
 @example
 ```ts
 isLiteralStringSafe({ content: "C:\\path", },); // true
 isLiteralStringSafe({ content: "it's", },); // false
 ```
 */
export function isLiteralStringSafe({ content, }: { readonly content: string; },): boolean {
  // `for...of` walks whole code points (the string iterator), avoiding both the
  // string-spread the linter rejects and any surrogate splitting.
  for (const scalar of content) {
    /**
     Codepoint under test; single quote and control scalars are disqualifying.
     */
    const codepoint = scalar.codePointAt(0,) ?? 0;
    if (codepoint === SINGLE_QUOTE_CODEPOINT) return false;
    if (codepoint <= LOW_CONTROL_MAX) return false;
    if (codepoint === DELETE_CODEPOINT) return false;
  }
  return true;
}

/**
 Encode `content` as a TOML literal-string literal, quotes included.
 
 @param content - String that must satisfy {@link isLiteralStringSafe}.
 
 @returns Single-quoted literal whose parsed value is `content`.
 
 @example
 ```ts
 literalStringLiteral({ content: 'C:\\path', },); // "'C:\\path'"
 ```
 */
export function literalStringLiteral({ content, }: { readonly content: string; },): string {
  return `'${content}'`;
}
