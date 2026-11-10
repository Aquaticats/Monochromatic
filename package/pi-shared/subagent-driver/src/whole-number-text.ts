/**
 Reads whether text is a whole number, which the commands here need for run folder numbers and process numbers. The rules of
 this repository forbid a regular expression in production code, so the digits are read one at a time.

 @module
 */

/**
 Reads whether text is a whole number.

 @param text - the text to read

 @returns whether it is one or more digits and nothing else

 @example
 ```ts
 const whole = isWholeNumberText({ text: '12', });
 ```
 */
export function isWholeNumberText({
  text,
}: {
  readonly text: string;
},): boolean {
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
