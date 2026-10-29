/**
 Shared TOML basic-string escaping.
 
 A basic string must escape for the TOML grammar at the destination:
 backslash, double quote, the named control escapes, and every other control
 scalar (below U+0020, plus U+007F) as a `\uXXXX` sequence. Emitting such a
 control scalar raw produces invalid TOML that the parser rejects, so the
 escaper here is exhaustive rather than handling only the common cases. Both
 the parsed-node emitter (`emit-value-string.ts`) and the from-scratch value
 and key encoders share this module so they cannot drift apart.
 
 @module
 */

//region Escape tables

/**
 Named single-character basic-string escapes keyed by codepoint.
 */
const NAMED_BASIC_ESCAPES: Record<number, string> = {
  0x08: String.raw`\b`,
  0x09: String.raw`\t`,
  0x0A: String.raw`\n`,
  0x0C: String.raw`\f`,
  0x0D: String.raw`\r`,
};

/**
 Backslash codepoint, escaped first in every basic string.
 */
const BACKSLASH_CODEPOINT = 0x5C;

/**
 Double-quote codepoint, escaped in single-line basic strings.
 */
const DQUOTE_CODEPOINT = 0x22;

/**
 Highest control codepoint requiring escaping at the low end.
 */
const LOW_CONTROL_MAX = 0x1F;

/**
 Delete control codepoint (U+007F), which the parser rejects raw.
 */
const DELETE_CODEPOINT = 0x7F;

/**
 Hexadecimal radix for `\uXXXX` escape formatting.
 */
const HEX_RADIX = 16;

/**
 Hex-digit width of a short `\uXXXX` escape.
 */
const SHORT_UNICODE_WIDTH = 4;

//endregion Escape tables

//region Escaping

/**
 Emit one control codepoint as its `\uXXXX` escape.
 
 @param codepoint - Control scalar value to format.
 
 @returns Six-character escape such as `\u0000`.
 
 @example
 ```ts
 unicodeEscape({ codepoint: 0, }); // '\\u0000'
 ```
 */
function unicodeEscape({ codepoint, }: { readonly codepoint: number; },): string {
  return `\\u${
    codepoint.toString(HEX_RADIX,)
      .toUpperCase()
      .padStart(
        SHORT_UNICODE_WIDTH,
        '0',
      )
  }`;
}

/**
 Escape one scalar for a basic string.
 
 @param scalar - Single code point of the source string.
 
 @param multiline - Whether the destination is a `"""` literal.
 
 @returns Escaped fragment, possibly the scalar itself.
 
 @example
 ```ts
 escapeBasicScalar({ scalar: '\n', multiline: false, }); // '\\n'
 ```
 */
function escapeBasicScalar(
  {
    scalar,
    multiline,
  }: {
    readonly scalar: string;
    readonly multiline: boolean;
  },
): string {
  /**
   Codepoint of the scalar; astral characters arrive whole and pass through.
   */
  const codepoint = scalar.codePointAt(0,) ?? 0;
  if (codepoint === BACKSLASH_CODEPOINT) return String.raw`\\`;
  // A single-line literal escapes every quote; a multiline literal leaves bare
  // quotes raw (a run of `"""` is escaped afterward). Control characters,
  // including newlines, are always escaped: a raw leading newline would be
  // trimmed by TOML and a lone carriage return is invalid, so neither is left raw.
  if ((!multiline) && (codepoint === DQUOTE_CODEPOINT)) return String.raw`\"`;
  /**
   Named escape for this codepoint, if one applies.
   */
  const named = NAMED_BASIC_ESCAPES[codepoint];
  if (named !== undefined) return named;
  if ((codepoint <= LOW_CONTROL_MAX) || (codepoint === DELETE_CODEPOINT))
    return unicodeEscape({ codepoint, },);
  return scalar;
}

/**
 Escape a string for emission inside a basic single-line `"..."` literal.
 
 @param value - Raw string value.
 
 @returns Escaped string content.
 
 @example
 ```ts
 escapeBasicSingleLine({ value: 'a\nb', }); // 'a\\nb'
 ```
 */
export function escapeBasicSingleLine({ value, }: { readonly value: string; },): string {
  return Array.from(
    value,
    function each(scalar,) {
      return escapeBasicScalar({
        scalar,
        multiline: false,
      },);
    },
  )
    .join('',);
}

/**
 Escape a string for emission inside a `"""..."""` multiline basic literal.
 
 Bare quotes stay raw, but a literal `"""` run is escaped so it cannot close
 the literal. Every control scalar, including newlines, is escaped: a raw
 leading newline would be trimmed by TOML and a lone carriage return is invalid.
 
 @param value - Raw string value.
 
 @returns Escaped string content.
 
 @example
 ```ts
 escapeBasicMultiline({ value: 'a"""b', }); // 'a\\"\\"\\"b'
 ```
 */
export function escapeBasicMultiline({ value, }: { readonly value: string; },): string {
  return Array.from(
    value,
    function each(scalar,) {
      return escapeBasicScalar({
        scalar,
        multiline: true,
      },);
    },
  )
    .join('',)
    .replaceAll(
    '"""',
    String.raw`\"\"\"`,
    );
}

//endregion Escaping
