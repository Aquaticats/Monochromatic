/**
 Reads whether the text a caller gave for one count is a whole number, which is the only kind of number a limit or a count may
 be set to.

 A setting that is no whole number is refused rather than read as zero or truncated: a limit of `4.5` read as `4` would start a
 run the owner did not allow, and one read as `0` would stop every start on the machine with no line saying why.

 @module
 */

/**
 Reads whether text is a whole number.

 @param text - the text to read

 @returns whether it is one or more digits and nothing else

 @example
 ```ts
 const whole = isWholeNumber('20',);
 ```
 */
export function isWholeNumber(text: string,): boolean {
  if (text === '') {
    return false;
  }
  for (const character of text) {
    if ((character < '0') || (character > '9')) {
      return false;
    }
  }
  return true;
}
