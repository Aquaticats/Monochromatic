/**
 ASCII character-set constants for tests, parsers, and token builders.
 
 These constants enumerate characters explicitly instead of spreading string
 literals, which avoids Unicode-code-point hazards and satisfies the
 `no-misused-spread` lint rule.
 
 @module
 */

//region ASCII character sets

/**
 Lowercase ASCII letter characters in code-point order.
 
 @example
 ```ts
 const firstLetter = ASCII_LOWERCASE_LETTER_CHARS[0];
 ```
 */
export const ASCII_LOWERCASE_LETTER_CHARS = [
  'a',
  'b',
  'c',
  'd',
  'e',
  'f',
  'g',
  'h',
  'i',
  'j',
  'k',
  'l',
  'm',
  'n',
  'o',
  'p',
  'q',
  'r',
  's',
  't',
  'u',
  'v',
  'w',
  'x',
  'y',
  'z',
] as const;

/**
 Decimal ASCII digit characters in code-point order.
 
 @example
 ```ts
 const firstDigit = ASCII_DECIMAL_DIGIT_CHARS[0];
 ```
 */
export const ASCII_DECIMAL_DIGIT_CHARS = [
  '0',
  '1',
  '2',
  '3',
  '4',
  '5',
  '6',
  '7',
  '8',
  '9',
] as const;

/**
 Lowercase ASCII letters followed by decimal ASCII digits.
 
 Useful for generated identifiers, slugs, and fixture tokens that must avoid
 path separators, whitespace, and case folding.
 
 @example
 ```ts
 const firstTokenChar = ASCII_LOWERCASE_ALPHANUMERIC_CHARS[0];
 ```
 */
export const ASCII_LOWERCASE_ALPHANUMERIC_CHARS = [
  'a',
  'b',
  'c',
  'd',
  'e',
  'f',
  'g',
  'h',
  'i',
  'j',
  'k',
  'l',
  'm',
  'n',
  'o',
  'p',
  'q',
  'r',
  's',
  't',
  'u',
  'v',
  'w',
  'x',
  'y',
  'z',
  '0',
  '1',
  '2',
  '3',
  '4',
  '5',
  '6',
  '7',
  '8',
  '9',
] as const;

//endregion ASCII character sets
