/**
 Reads whether one word's source spelling expands at run time.
 
 A guard that bans run-time paths needs the difference between a path written
 out and a path the shell computes.
 Both spell the same characters after quote removal,
 so only the source spelling separates them:
 `rm -rf "$d"` asks the shell for the path,
 while `rm -rf '$d'` deletes a directory named `$d`.
 
 Called on a word's original spelling,
 this reads parameter expansion,
 command substitution (dollar form and backtick form),
 and arithmetic expansion,
 and treats single quotes,
 ANSI-C quoting,
 and backslash escapes as the literal text they are.
 
 @module
 */

//region Constants

/**
 Character that makes the next one literal outside single quotes.
 */
const BACKSLASH = '\\';

/**
 Quote character whose region expands nothing.
 */
const SINGLE_QUOTE = '\'';

/**
 Quote character whose region still expands parameters, substitutions, and backticks.
 */
const DOUBLE_QUOTE = '"';

/**
 Character that introduces an expansion.
 */
const DOLLAR_SIGN = '$';

/**
 Character that runs the text it encloses as a command.
 */
const BACKTICK = '`';

/**
 Character that opens a command substitution or an arithmetic expansion.
 */
const SUBSTITUTION_OPEN = '(';

/**
 Characters that continue a parameter expansion beyond the dollar sign.
 */
const PARAMETER_EXPANSION_CHARACTERS = '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz'
  + '{@*#?$!-';

//endregion Constants

//region Readers

/**
 Finds the quote closing an ANSI-C quoted region, where a backslash escapes.
 
 @param sourceText - word source spelling
 
 @param from - offset just after the opening quote
 
 @returns offset of the closing quote, or -1 when the region never closes
 
 @example
 ```ts
 ansiCQuotedRegionEnd({ sourceText: String.raw`$'a\'b'`, from: 2, }); // 6
 ```
 */
function ansiCQuotedRegionEnd({
  sourceText,
  from,
}: {
  readonly sourceText: string;
  readonly from: number;
},): number {
  for (let offset = from; offset < sourceText.length; offset += 1) {
    /**
     Character at the inspected offset.
     */
    const character = sourceText[offset];
    if (character === BACKSLASH) {
      offset += 1;
      continue;
    }
    if (character === SINGLE_QUOTE)
      return offset;
  }
  return (-1);
}

/**
 Reports whether an expansion starts at one offset inside a word.
 
 @param sourceText - word source spelling
 
 @param offset - offset of the `$` or the backtick being read
 
 @returns whether the shell would compute part of this word at run time
 
 @example
 ```ts
 expansionStartsAt({ sourceText: '"$d"', offset: 1, }); // true
 ```
 */
function expansionStartsAt({
  sourceText,
  offset,
}: {
  readonly sourceText: string;
  readonly offset: number;
},): boolean {
  /**
   Character at the inspected offset.
   */
  const character = sourceText[offset];
  if (character === BACKTICK)
    return true;
  if (character !== DOLLAR_SIGN)
    return false;
  /**
   Character after the dollar sign, or absent at the end of the word.
   */
  const next = sourceText[offset + 1];
  if (next === SUBSTITUTION_OPEN)
    return true;
  return (next !== undefined)
    && PARAMETER_EXPANSION_CHARACTERS.includes(next,);
}

/**
 Reports whether one source-spelled word expands at run time.
 
 @param sourceText - word as written, including its quotes and escapes
 
 @returns whether the shell would compute part of this word at run time
 
 @example
 ```ts
 shellWordCarriesExpansion('"$d"'); // true
 shellWordCarriesExpansion(`'$d'`); // false
 shellWordCarriesExpansion('$((1 + 1))'); // true
 ```
 */
function shellWordCarriesExpansion(sourceText: string,): boolean {
  return (function readsWordText(): boolean {
    /**
     Offset of the character being read.
     */
    let offset = 0;
    /**
     Whether the offset sits inside a double-quoted region, where a single quote is text.
     */
    let inDoubleQuote = false;
    while (offset < sourceText.length) {
      /**
       Character at the inspected offset.
       */
      const character = sourceText[offset];
      if (character === BACKSLASH) {
        offset += 2;
        continue;
      }
      if (character === DOUBLE_QUOTE) {
        inDoubleQuote = !inDoubleQuote;
        offset += 1;
        continue;
      }
      if (expansionStartsAt({
        sourceText,
        offset,
      },))
        return true;
      if ((!inDoubleQuote) && (character === SINGLE_QUOTE)) {
        /**
         Offset of the quote closing this literal region, or -1 when it never closes.
         */
        const closingOffset = sourceText.indexOf(
          SINGLE_QUOTE,
          offset + 1,
        );
        if (closingOffset === (-1))
          return false;
        offset = closingOffset + 1;
        continue;
      }
      if ((!inDoubleQuote)
        && (character === DOLLAR_SIGN)
        && (sourceText[offset + 1] === SINGLE_QUOTE)) {
        /**
         Offset of the quote ending the ANSI-C region, which decodes escapes only.
         */
        const closingOffset = ansiCQuotedRegionEnd({
          sourceText,
          from: offset + 2,
        },);
        if (closingOffset === (-1))
          return false;
        offset = closingOffset + 1;
        continue;
      }
      offset += 1;
    }
    return false;
  })();
}

//endregion Readers

export {
  ansiCQuotedRegionEnd,
  shellWordCarriesExpansion,
};
