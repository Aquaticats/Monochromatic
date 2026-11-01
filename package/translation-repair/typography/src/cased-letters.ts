//region Cased letters
// ONE READING OF A CASED LETTER, by general category (ledger B21). The quote
// neighbours read a cased letter as Lu, Ll or Lt since class ninety-six: the
// mathematical script letters the corpus writes handles in are cased letters
// with no case mapping. The Canadian passes read one by case mapping, so a
// script letter was a letter beside an apostrophe and no letter beside a date
// or a listed word, and a capital or small letter test that compared a
// character with its case mapping read a script capital as neither. Every
// test here takes one whole character (`codePointAt`).

/* oxlint-disable no-restricted-syntax/no-regex -- the input is one code point, anchored at both ends with no quantifier, so the test is bounded and cannot backtrack; the Unicode general categories have no string API */
/**
 Whether one code point is a cased letter.
 */
const CASED_LETTER = /^(?:\p{Lu}|\p{Ll}|\p{Lt})$/u;

/**
 Whether one code point is a capital: upper or title case.
 */
const CAPITAL_LETTER = /^(?:\p{Lu}|\p{Lt})$/u;
/* oxlint-enable no-restricted-syntax/no-regex */

/* oxlint-disable no-restricted-syntax/no-regex -- the input is one code point, anchored at both ends with no quantifier, so the test is bounded and cannot backtrack; the Unicode general category has no string API */
/**
 Whether one code point is a small letter.
 */
const SMALL_LETTER = /^\p{Ll}$/u;
/* oxlint-enable no-restricted-syntax/no-regex */

/**
 Whether one character is a letter with case, in any script: upper, lower or
 title case by general category.

 @param character - one whole character, as `codePointAt` reads it; empty
 past a text's edge

 @returns Whether it is a cased letter

 @example
 ```ts
 isCasedLetter({ character: '\u{1D4DC}', },); // true: a script capital
 ```
 */
export function isCasedLetter({ character, }: { readonly character: string; },): boolean {
  return CASED_LETTER.test(character,);
}

/**
 Whether one character is a capital letter, in any script.

 @param character - one whole character, as `codePointAt` reads it; empty
 past a text's edge

 @returns Whether it is upper or title case

 @example
 ```ts
 isCapitalLetter({ character: 'É', },); // true
 ```
 */
export function isCapitalLetter({ character, }: { readonly character: string; },): boolean {
  return CAPITAL_LETTER.test(character,);
}

/**
 Whether one character is a small letter, in any script.

 @param character - one whole character, as `codePointAt` reads it; empty
 past a text's edge

 @returns Whether it is lower case

 @example
 ```ts
 isSmallLetter({ character: '\u{1D501}', },); // true: a script small x
 ```
 */
export function isSmallLetter({ character, }: { readonly character: string; },): boolean {
  return SMALL_LETTER.test(character,);
}

//endregion Cased letters
