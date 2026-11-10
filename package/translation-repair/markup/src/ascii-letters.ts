//region ASCII letters
// The ASCII letter, digit and hexadecimal tests the package's scanners share.
// Modules kept their own copies, as named functions and written inline (audit
// area six, 2026-09-28; ledger B18), several of them named for Latin while
// testing ASCII only, which hid what they do: an accented letter such as `é`
// ends a word for every one of them.
//
// ASCII IS RIGHT WHERE A FORMAT SAYS SO: an HTML tag or attribute name starts
// with an ASCII letter, a corpus directory id is ASCII, and the signer-handle
// floor reads a rendering's basic letters after NFD has split its tone marks
// off. Where a scanner reads English prose, an ASCII test is a choice the
// scanner states; the name here says what is tested so that choice is visible.
//
// THIS MODULE IS THE TRANSLATION-REPAIR FAMILY'S OWNER of those classes, and the
// exemption is deliberate: `@monochromatic-dev/agent-harness-shared-text-scan`
// owns the same ASCII classes for agent-harness code, and the review that asked
// every copy to import it lets a site stay when it says why (issue 1673).
// Two facts keep this one. The family already consolidated eleven modules'
// copies here (`72bbe6500`, audit ledger B18), so importing the agent-harness
// package would put a second home back for the same classes. And the object
// parameter is this family's call convention: the five names are written 204
// times across 41 files, every one as `isAsciiLetter({ character, })` or its
// sibling, so a plain string predicate rewrites all of them without changing a
// verdict. One call site in the family now takes `isLowerAlpha` from the owner
// directly, in `adjudication/src/task-runner-guard.ts`, because that function
// held its own copy rather than calling this module.

/**
 Whether one character is a lower-case ASCII letter, `a` to `z`.

 @param character - one UTF-16 unit, empty past a text's edge

 @returns True for `a` to `z` only

 @example
 ```ts
 isAsciiLowerLetter({ character: 'h', },); // true
 isAsciiLowerLetter({ character: 'H', },); // false
 ```
 */
export function isAsciiLowerLetter({ character, }: { readonly character: string; },): boolean {
  return (character >= 'a') && (character <= 'z');
}

/**
 Whether one character is an ASCII letter, `a` to `z` in either case.

 @param character - one UTF-16 unit, empty past a text's edge

 @returns True for an ASCII letter only; an accented letter is not one

 @example
 ```ts
 isAsciiLetter({ character: 'N', },); // true
 isAsciiLetter({ character: 'é', },); // false
 ```
 */
export function isAsciiLetter({ character, }: { readonly character: string; },): boolean {
  return isAsciiLowerLetter({ character, },) || ((character >= 'A') && (character <= 'Z'));
}

/**
 Whether one character is an ASCII digit.

 @param character - one UTF-16 unit, empty past a text's edge

 @returns True for `0` to `9`

 @example
 ```ts
 isAsciiDigit({ character: '7', },); // true
 ```
 */
export function isAsciiDigit({ character, }: { readonly character: string; },): boolean {
  return (character >= '0') && (character <= '9');
}

/**
 Whether a text is a run of ASCII digits and nothing else, as a count, an
 index or a number word is written.

 @param text - text under test

 @returns True for a non-empty text of `0` to `9` only

 @example
 ```ts
 isAsciiDigits({ text: '611', },); // true
 isAsciiDigits({ text: '10th', },); // false
 isAsciiDigits({ text: '', },); // false
 ```
 */
export function isAsciiDigits({ text, }: { readonly text: string; },): boolean {
  return (text !== '')
    && Array.from(text,)
      .every(function digit(character,): boolean {
        return isAsciiDigit({ character, },);
      },);
}

/**
 Whether one character is an ASCII letter or digit.

 @param character - one UTF-16 unit, empty past a text's edge

 @returns True for `a` to `z`, `A` to `Z` and `0` to `9`

 @example
 ```ts
 isAsciiAlphanumeric({ character: '9', },); // true
 isAsciiAlphanumeric({ character: '_', },); // false
 ```
 */
export function isAsciiAlphanumeric({ character, }: { readonly character: string; },): boolean {
  return isAsciiLetter({ character, },) || isAsciiDigit({ character, },);
}

/**
 Whether one character is a lower-case hexadecimal digit, as a digest or a
 git object id is written. Upper case is refused: every writer here emits
 lower case, so another spelling came from elsewhere, and two spellings of
 one value would count as two.

 @param character - one UTF-16 unit, empty past a text's edge

 @returns True for `0` to `9` and `a` to `f`

 @example
 ```ts
 isLowerHexDigit({ character: 'f', },); // true
 isLowerHexDigit({ character: 'F', },); // false
 ```
 */
export function isLowerHexDigit({ character, }: { readonly character: string; },): boolean {
  return isAsciiDigit({ character, },) || ((character >= 'a') && (character <= 'f'));
}

//endregion ASCII letters
