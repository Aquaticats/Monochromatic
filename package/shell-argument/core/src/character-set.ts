/**
 Decimal digits.
 */
export const decimalDigits = '0123456789';

/**
 Ascii letters and decimal digits.
 */
export const alphanumericCharacters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';

/**
 Ascii letters and the underscore.
 */
export const wordStartCharacters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_';

/**
 Check that a text is non-empty and every character of it is one of the allowed characters, in one linear scan.

 @param text - value under test
 
 @param allowed - characters it may use
 
 @returns true when the text is non-empty and uses only allowed characters
 
 @example
 ```ts
 usesOnly({ text: '12G', allowed: '0123456789G' });
 ```
 */
export function usesOnly({
  text,
  allowed,
}: {
  readonly text: string;
  readonly allowed: string;
}): boolean {
  if (text.length === 0)
    return false;
  for (const character of text) {
    if (!allowed.includes(character))
      return false;
  }
  return true;
}

/**
 Check that a text starts with one of the first characters and continues with the rest characters, in one linear scan.

 @param text - value under test
 
 @param first - characters its first character may be
 
 @param rest - characters the others may be
 
 @returns true when the text is non-empty and fits both sets
 
 @example
 ```ts
 startsThenUses({ text: 'a-b', first: 'ab', rest: 'ab-' });
 ```
 */
export function startsThenUses({
  text,
  first,
  rest,
}: {
  readonly text: string;
  readonly first: string;
  readonly rest: string;
}): boolean {
  return usesOnly({
    text: text.slice(
      0,
      1
    ),
    allowed: first
  })
    && ((text.length === 1) || usesOnly({
      text: text.slice(1),
      allowed: rest
    }));
}

/**
 Replace every character outside the allowed characters with a hyphen, in one linear pass.

 @param text - value to sanitize
 
 @param allowed - characters kept as they are
 
 @returns the text with the same number of characters, each kept or replaced
 
 @example
 ```ts
 hyphenateOutside({ text: 'a b', allowed: 'ab' });
 ```
 */
export function hyphenateOutside({
  text,
  allowed,
}: {
  readonly text: string;
  readonly allowed: string;
}): string {
  return Array.from(
    text,
    function keepOrHyphenate(character) {
      return allowed.includes(character) ? character : '-';
    },
  )
    .join('');
}

/**
 Check for a positive whole number written without a sign, leading zero or fraction, short enough to be exact.

 @param text - value under test
 
 @param maximumDigits - longest accepted text
 
 @returns true for `1`, `30`; false for `0`, `007`, `1.5`, the empty text and texts over the limit
 
 @example
 ```ts
 isPositiveWholeNumber({ text: '900', maximumDigits: 10 });
 ```
 */
export function isPositiveWholeNumber({
  text,
  maximumDigits,
}: {
  readonly text: string;
  readonly maximumDigits: number;
}): boolean {
  return usesOnly({
    text,
    allowed: decimalDigits
  })
    && (!text.startsWith('0'))
    && (text.length <= maximumDigits);
}
