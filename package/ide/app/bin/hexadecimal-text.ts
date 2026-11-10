// Whether a text is a lowercase hexadecimal number of an asked length, which is the shape of the commit id a runtime
// manifest names.
// What: a length comparison and one walk over the characters, each of which must be a digit or a letter from `a` to
// `f`.
// Why: the bundle inspection validated the manifest's Helix revision through `/^[0-9a-f]{40}$/`, and the repository
// lint bans a pattern where a length check and a character walk answer the same question with the count named in
// code.
// Tested in hexadecimal-text.test.ts.
import { isLowercaseHexadecimalDigit } from './scan-characters.ts';

/**
 Whether a text is a lowercase hexadecimal number of the asked length.
 
 @param options - `text` is the candidate, `length` the count every accepted text holds.
 
 @returns Whether the text is exactly that long and every character is a hexadecimal digit.
 
 @example
 ```ts
 isHexadecimalText({ text: '0123abcd', length: 8 }); // true
 isHexadecimalText({ text: '0123ABCD', length: 8 }); // false
 ```
 */
export function isHexadecimalText({ text, length }: { text: string; length: number }): boolean {
  if (text.length !== length) return false;
  for (let index = 0; index < length; index++) {
    if (!isLowercaseHexadecimalDigit(text[index] ?? '')) return false;
  }
  return true;
}
