import { isAsciiDigits, } from './ascii-letters.ts';

//region Whole number text
// One rule for reading a count back out of text, shared by every reader of a
// count the package wrote itself (a log line, a sheet heading, a declaration)
// and every reader of a count an operator typed (ledger B73).
//
// `Number` IS NOT THAT RULE. It reads an empty or blank text as 0, takes a
// hexadecimal, binary, octal or exponent spelling, a sign and surrounding
// spaces, and reads a digit run past the largest whole number a double holds
// exactly as a neighbouring number. Each of those gave some reader here a
// number nobody wrote: an empty heard count read as a stage that heard nobody,
// a truncated duration as no time at all, a mistyped cap as the default.
//
// DIGITS ONLY, LEADING ZEROS READ AS WRITTEN. `04` names four to anyone who
// types it, and no writer here pads, so reading it costs nothing; a sign, a
// point, a radix prefix or a space is refused, since none is how a count is
// written here.
//
// AN AMOUNT THAT TAKES A FRACTION (minutes, dollars, a rate) is a plain
// decimal: digits, or digits, a point and digits. The same spellings are
// refused, and the point needs a digit on each side.

/**
 How a refusal names the numbers this rule takes, so every reader says the
 same thing about the same rule.
 */
export const WHOLE_NUMBER_RULE: string = `a whole number written in digits, at most ${String(Number.MAX_SAFE_INTEGER,)}`;

/**
 Whether text is a whole number written in ASCII digits that a double holds
 exactly, which is the only count text any reader here takes.

 @param text - count as written

 @returns Whether `Number` reads it as exactly the number written

 @example
 ```ts
 isWholeNumberText({ text: '12', },); // true
 isWholeNumberText({ text: '1e3', },); // false
 ```
 */
export function isWholeNumberText({ text, }: { readonly text: string; },): boolean {
  return isAsciiDigits({ text, },) && Number.isSafeInteger(Number(text,),);
}

/**
 Marker a typed count below zero opens with.
 */
const MINUS = '-';

/**
 Whether text is a minus sign before a whole number above zero, which is how
 an operator types a count below zero.

 TOLD APART FROM OTHER TEXT THE RULE REFUSES so a reader can answer it in
 its own words ("cannot be below zero", "must be at least 1") rather than
 call `-3` no number at all. `-0` IS NOT ONE: it names no count below zero,
 so it draws the rule's own refusal like any other spelling the rule refuses
 (ledger B73).

 @param text - count as typed

 @returns Whether `Number` reads it as exactly the number below zero written

 @example
 ```ts
 isNegativeWholeNumberText({ text: '-3', },); // true
 isNegativeWholeNumberText({ text: '-0', },); // false
 ```
 */
export function isNegativeWholeNumberText({ text, }: { readonly text: string; },): boolean {
  /**
   What follows the sign, the count's size.
   */
  const magnitude = text.slice(MINUS.length,);
  return text.startsWith(MINUS,)
    && isWholeNumberText({ text: magnitude, },)
    && (Number(magnitude,) > 0);
}

/**
 Separator between a decimal's whole and fractional digits.
 */
const DECIMAL_POINT = '.';

/**
 Whether text is a plain decimal written in ASCII digits, with at most one
 point that has digits on both sides.

 NOT WHETHER `Number` READS IT FINITE: a digit run past the largest double
 is a plain decimal that `Number` reads as Infinity, so a reader checks that
 too.

 @param text - amount as written

 @returns Whether it is digits, or digits, a point and digits

 @example
 ```ts
 isDecimalText({ text: '7.5', },); // true
 isDecimalText({ text: '.5', },); // false
 ```
 */
export function isDecimalText({ text, }: { readonly text: string; },): boolean {
  /**
   Digits before the point, those after it where one is written, and any
   text past a second point, which no decimal has.
   */
  const [
    whole = '',
    fraction,
    ...beyond
  ] = text.split(DECIMAL_POINT,);
  return (beyond.length === 0)
    && isAsciiDigits({ text: whole, },)
    && ((fraction === undefined) || isAsciiDigits({ text: fraction, },));
}

/**
 Letters that open a number's exponent, as JSON writes one.
 */
const EXPONENT_MARKS: ReadonlySet<string> = new Set([
  'e',
  'E',
],);

/**
 Signs an exponent may carry.
 */
const EXPONENT_SIGNS: ReadonlySet<string> = new Set([
  '+',
  '-',
],);

/**
 What `findIndex` returns when no character matches.
 */
const NOT_FOUND = -1;

/**
 Whether text is an unsigned number as JSON writes one: a plain decimal, then
 optionally an exponent mark, an optional sign and digits.

 FOR NUMBERS A SERVICE WRITES AS TEXT, not ones a person types. A catalogue
 in OpenRouter's shape (LLM Gateway's) writes a price as `0.042e-6`, the one
 exponent spelling among the 7,866 stored listing prices the audit read
 (ledger B73). Like {@link isDecimalText} it does not ask whether `Number`
 reads the text finite.

 @param text - number as written, currency sign removed

 @returns Whether it is a plain decimal with at most one exponent after it

 @example
 ```ts
 isUnsignedNumberText({ text: '0.042e-6', },); // true
 isUnsignedNumberText({ text: '1e', },); // false
 ```
 */
export function isUnsignedNumberText({ text, }: { readonly text: string; },): boolean {
  /**
   The text's characters, so the mark is found and cut at the same offsets.
   */
  const characters = Array.from(text,);
  /**
   Where the exponent mark stands, when the number has one.
   */
  const markAt = characters.findIndex(function isMark(character,): boolean {
    return EXPONENT_MARKS.has(character,);
  },);
  if (markAt === NOT_FOUND)
    return isDecimalText({ text, },);
  /**
   What follows the mark: an optional sign, then digits. A second mark lands
   here and is no digit.
   */
  const exponent = characters.slice(markAt + 1,);
  /**
   The exponent's digits, past the sign it may open with.
   */
  const digits = EXPONENT_SIGNS.has(exponent[0] ?? '',) ? exponent.slice(1,) : exponent;
  return isDecimalText({
    text: characters.slice(
      0,
      markAt,
    )
      .join('',),
  },)
    && isAsciiDigits({ text: digits.join('',), },);
}

//endregion Whole number text
