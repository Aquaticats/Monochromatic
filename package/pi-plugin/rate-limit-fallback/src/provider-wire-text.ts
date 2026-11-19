/**
 Index scans for the ASCII tokens reported in provider error messages and reset headers.

 @module
 */

/**
 First ASCII decimal digit.
 */
const ZERO = '0';

/**
 Last ASCII decimal digit.
 */
const NINE = '9';

/**
 HTTP status indicating a rate limit.
 */
export const HTTP_RATE_LIMIT_STATUS = 429;

/**
 Fold the ASCII comparisons the wire grammars make, including the Unicode equivalents of `k` and `s`.

 @param value - provider message to compare without case

 @returns comparison text

 @example
 ```ts
 foldProviderText('HTTP 429'); // 'http 429'
 ```
 */
export function foldProviderText(value: string,): string {
  return value.toLowerCase()
    .replaceAll(
    'ſ',
    's',
    );
}

/**
 Find the end of the ASCII decimal digits starting at an offset.

 @param value - text containing an unsigned whole-number component

 @param offset - first character to inspect

 @returns exclusive end, or offset when there are no digits

 @example
 ```ts
 digitsEnd({ value: '12ms', offset: 0 }); // 2
 ```
 */
export function digitsEnd({
  value,
  offset,
}: {
  readonly value: string;
  readonly offset: number;
},): number {
  for (let cursor = offset; cursor < value.length; cursor += 1) {
    /**
     Character to compare against the ordered ASCII digit range.
     */
    const digit = value[cursor];
    if ((digit === undefined) || (digit < ZERO)
      || (digit > NINE))
      return cursor;
  }
  return value.length;
}
