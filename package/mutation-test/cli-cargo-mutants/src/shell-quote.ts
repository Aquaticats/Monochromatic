import { CargoMutantsToolingError } from './error.ts';

/**
 Single quote character, the only character POSIX shells cannot hold inside single quotes.
 */
const singleQuote = "'";

/**
 Spelling of one single quote inside a single-quoted word: close the quote, add an escaped quote, reopen the quote.
 */
const escapedSingleQuote = String.raw`'\''`;

/**
 Encode a value as one POSIX shell word that expands to exactly the value.

 Single quotes keep every character literal, including spaces, `$`, `;`, backticks, backslashes and newlines;
 each single quote in the value is spelled `'\''`.

 @param value - text that ends up as one shell argument
 
 @returns the quoted word, safe to place anywhere a shell word may stand
 
 @throws CargoMutantsToolingError when the value holds a NUL character, which no shell argument can hold

 @example
 ```ts
 shellQuote("it's");
 ```
 */
export function shellQuote(value: string): string {
  if (value.includes('\0'))
    throw new CargoMutantsToolingError('A shell argument cannot contain a NUL character.');
  return `${singleQuote}${value.replaceAll(
    singleQuote,
    escapedSingleQuote
  )}${singleQuote}`;
}
