/**
 Quoting of values that are printed as part of a shell command for a person to copy.

 @module
 */

/**
 Characters that a POSIX shell leaves alone in a word,
 so a value made only of them is printed as it is.
 */
const BARE_CHARACTERS = new Set('ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789@%+=:,./_-',);

/**
 Highest code point of the ASCII control characters (the range from NUL to US).
 */
const LAST_C0_CONTROL = 0x1F;

/**
 Lowest code point of the range that holds DEL and the C1 control characters.
 */
const FIRST_DELETE_OR_C1 = 0x7F;

/**
 Lowest code point of the C1 control characters.
 */
const FIRST_C1_CONTROL = 0x80;

/**
 Highest code point of the C1 control characters.
 */
const LAST_C1_CONTROL = 0x9F;

/**
 Reports whether a character is a control character,
 which would move the cursor or hide text when it is printed to a terminal.

 @param character - One character.

 @returns Whether it is a C0 or C1 control character or DEL.

 @example
 ```ts
 isControl({ character: '\n' }); // true
 ```
 */
function isControl({ character, }: { readonly character: string; },): boolean {
  /**
   Code point of the character.
   */
  const code = character.codePointAt(0,) ?? 0;
  return (code <= LAST_C0_CONTROL) || ((code >= FIRST_DELETE_OR_C1) && (code <= LAST_C1_CONTROL));
}

/**
 Spells one character of a value inside an ANSI-C quoted string (`$'...'`).

 @param character - One character.

 @returns Its spelling.

 @example
 ```ts
 ansiCharacter({ character: '\n' }); // '\\n'
 ```
 */
function ansiCharacter({ character, }: { readonly character: string; },): string {
  if (character === '\\')
    return String.raw`\\`;
  if (character === '\'')
    return String.raw`\'`;
  if (character === '\n')
    return String.raw`\n`;
  if (character === '\t')
    return String.raw`\t`;
  if (isControl({ character, },)) {
    /**
     Code point of the control character.
     */
    const code = character.codePointAt(0,) ?? 0;
    /**
     Code point in two hexadecimal digits.
     */
    const hex = code.toString(16,)
      .padStart(
      2,
      '0',
      );
    // A C1 control is two bytes in UTF-8; `\xHH` alone would print one byte that is not valid text.
    return code >= FIRST_C1_CONTROL ? `\\xc2\\x${hex}` : `\\x${hex}`;
  }
  return character;
}

/**
 Quotes a value so a POSIX shell reads it back as that one word.
 A value of safe characters is printed as it is;
 any other value is wrapped in single quotes,
 with each single quote inside it closed, escaped and opened again;
 a value with a control character, such as a newline or an escape,
 is spelled in the ANSI-C form `$'...'` of bash, zsh and ksh with the control characters written as escapes,
 so that what is printed holds no control character and no line break.

 @param value - Text to put in a command.

 @returns The quoted text.

 @example
 ```ts
 shellQuote({ value: "it's" }); // "'it'\\''s'"
 ```
 */
export function shellQuote({ value, }: { readonly value: string; },): string {
  if (value === '')
    return '\'\'';
  /**
   The characters of the value,
   by code point.
   */
  const characters = Array.from(value,);
  if (characters.every(function isBare(character,): boolean {
    return BARE_CHARACTERS.has(character,);
  },))
    return value;
  if (characters.some(function hasControl(character,): boolean {
    return isControl({ character, },);
  },))
    return `$'${characters
      .map(function spell(character,): string {
        return ansiCharacter({ character, },);
      },)
      .join('',)}'`;
  return `'${value.replaceAll(
    '\'',
    String.raw`'\''`,
  )}'`;
}
