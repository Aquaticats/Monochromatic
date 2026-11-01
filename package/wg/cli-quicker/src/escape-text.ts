import {
  ESCAPE_DIGITS,
  ESCAPE_RADIX,
  MARKUP_ENTITIES,
  UNSAFE_CODE_POINT_RANGES,
} from './escape-text.data.ts';

/**
 Reports a code point that can change how a terminal or a reader shows text:
 a control character,
 or an invisible direction or joining mark that reorders or hides text.

 @param codePoint - One code point.

 @returns Whether it must be escaped.

 @example
 ```ts
 isUnsafeCodePoint({ codePoint: 0x1B }); // true
 ```
 */
function isUnsafeCodePoint({ codePoint, }: { readonly codePoint: number; },): boolean {
  return UNSAFE_CODE_POINT_RANGES.some(function inRange(range,): boolean {
    return (codePoint >= range.first) && (codePoint <= range.last);
  },);
}

/**
 Escapes untrusted text for a line of a log,
 the journal,
 or a terminal:
 every character that could start an escape sequence,
 end the line,
 or reorder the text becomes `\u{XXXX}`.

 @param text - Text that may hold names a user chose.

 @returns Text safe to print on one line.

 @example
 ```ts
 sanitizeText({ text: 'a\nb' }); // 'a\\u{000a}b'
 ```
 */
export function sanitizeText({ text, }: { readonly text: string; },): string {
  return Array.from(
    text,
    function escapeOne(character,): string {
      /**
       Code point of the character.
       */
      const codePoint = character.codePointAt(0,) ?? 0;
      return isUnsafeCodePoint({ codePoint, },)
        ? `\\u{${codePoint.toString(ESCAPE_RADIX,)
          .padStart(
            ESCAPE_DIGITS,
            '0',
          )}}`
        : character;
    },
  )
    .join('',);
}

/**
 Escapes text for the body of a desktop notification,
 whose markup subset treats `<`,
 `>` and `&` as syntax.

 @param text - Text to show.

 @returns Text with markup characters turned into entities.

 @example
 ```ts
 escapeMarkup({ text: '<b>' }); // '&lt;b&gt;'
 ```
 */
export function escapeMarkup({ text, }: { readonly text: string; },): string {
  return Array.from(
    text,
    function escapeOne(character,): string {
      return MARKUP_ENTITIES.get(character,) ?? character;
    },
  )
    .join('',);
}
