/**
 Linear scanners that turn project source text into the Material Symbols icons
 the site uses.
 
 The icon charset pass needs the `icon('NAME')` call sites of every source file
 with the comments taken out. This module does that with one character walk per
 step rather than a regular expression: a block comment stripper, a line comment
 stripper, and the call-site matcher. Each walk resumes where the previous match
 ended, so the whole pass stays linear in the length of the text and no
 backtracking state has to be reasoned about.
 
 The two strippers keep the order the pass depends on: block comments are
 removed before line comments, because a line comment marker inside a block
 comment would otherwise eat the line that closes the block comment.
 
 @module
 */

//region Delimiters

/**
 Opening delimiter of a block comment.
 */
const BLOCK_COMMENT_OPENING = '/*';

/**
 Closing delimiter of a block comment.
 */
const BLOCK_COMMENT_CLOSING = '*/';

/**
 Opening delimiter of a line comment.
 */
const LINE_COMMENT_OPENING = '//';

//endregion Delimiters

//region Character classes

/**
 Characters JavaScript's `\s` class matches, which is what the deleted pattern
 skipped between a call's tokens.
 */
const WHITESPACE_CHARACTERS = new Set([
  '\t',
  '\n',
  '\v',
  '\f',
  '\r',
  ' ',
  '\u00A0',
  '\u1680',
  '\u2000',
  '\u2001',
  '\u2002',
  '\u2003',
  '\u2004',
  '\u2005',
  '\u2006',
  '\u2007',
  '\u2008',
  '\u2009',
  '\u200A',
  '\u2028',
  '\u2029',
  '\u202F',
  '\u205F',
  '\u3000',
  '\uFEFF',
],);

/**
 Tests one character against JavaScript's `\s` class.
 
 Past the end of the text a character read is the empty string, which no
 whitespace class matches, so a scan always ends at the text's end.
 
 @param character - single character, or the empty string at the text's end
 
 @returns whether the character is whitespace
 */
function isWhitespace(character: string,): boolean {
  return WHITESPACE_CHARACTERS.has(character,);
}

/**
 Tests one character for one of the half-width ASCII digits.
 
 @param character - character to test
 
 @returns whether the character is `0` through `9`
 */
function isAsciiDigit(character: string,): boolean {
  return (character >= '0') && (character <= '9');
}

/**
 Tests one character for a lowercase ASCII letter.
 
 @param character - character to test
 
 @returns whether the character is `a` through `z`
 */
function isLowercaseAsciiLetter(character: string,): boolean {
  return (character >= 'a') && (character <= 'z');
}

/**
 Tests one character against the pattern's `\b` word class, which is ASCII
 letters, digits, and the underscore.
 
 @param character - character to test
 
 @returns whether the character continues a word
 */
function isAsciiWordCharacter(character: string,): boolean {
  if ((character >= 'A') && (character <= 'Z'))
    return true;
  return isLowercaseAsciiLetter(character,)
    || isAsciiDigit(character,)
    || (character === '_');
}

/**
 Tests one character against the pattern's icon name alphabet after the first.
 
 @param character - character to test
 
 @returns whether the character can continue an icon name
 */
function isIconNameCharacter(character: string,): boolean {
  return isLowercaseAsciiLetter(character,)
    || isAsciiDigit(character,)
    || (character === '_');
}

/**
 Tests one character for a quote at either end of an icon name.
 
 Single and double quotes are interchangeable and need not pair up, which is
 what the deleted pattern's `['"]` class allowed.
 
 @param character - character to test
 
 @returns whether the character quotes an icon name
 */
function isQuote(character: string,): boolean {
  return (character === '\'') || (character === '"');
}

//endregion Character classes

//region Block comment removal

/**
 Removes every block comment from source text.
 
 Delimiters pair up the way the previous non-greedy pattern paired them: the
 first closing delimiter after an opening `/*` ends the comment, so a comment
 marker inside a comment closes it there. An opening `/*` with no closing
 delimiter after it is not a comment and stays in the text, which is what an
 unterminated block comment leaves the pass with today.
 
 @param text - source text to strip
 
 @returns the text with every closed block comment removed
 
 @example
 ```ts
 // A commented-out call no longer reaches the icon charset.
 const stripped = stripBlockComments(sourceText,);
 ```
 */
export function stripBlockComments(text: string,): string {
  /**
   Text runs between comments, joined once at the end so the walk never rebuilds
   the result character by character.
   */
  const kept: string[] = [];
  for (let cursor = 0;;) {
    /**
     Next opening delimiter at or after the cursor; absent when the text is done.
     */
    const opening = text.indexOf(
      BLOCK_COMMENT_OPENING,
      cursor,
    );
    /**
     First closing delimiter after that opening; absent when the comment never closes.
     */
    const closing = opening === (-1)
      ? (-1)
      : text.indexOf(
        BLOCK_COMMENT_CLOSING,
        opening + BLOCK_COMMENT_OPENING.length,
      );
    if ((opening === (-1)) || (closing === (-1))) {
      kept.push(text.slice(cursor,),);
      return kept.join('',);
    }
    kept.push(
      text.slice(
        cursor,
        opening,
      ),
    );
    cursor = closing + BLOCK_COMMENT_CLOSING.length;
  }
}

//endregion Block comment removal

//region Line comment removal

/**
 Removes every `//` line comment from source text.
 
 A comment runs to the newline that ends its line, and the newline itself
 stays: the text keeps its line structure, so a position reported from the
 stripped text still names the same line. A `//` with no newline after it runs
 to the end of the text, and markers inside a block comment are ordinary text
 here, which is why {@link stripBlockComments} runs first.
 
 @param text - source text to strip
 
 @returns the text with every line comment removed
 
 @example
 ```ts
 // A commented-out call no longer reaches the icon charset.
 const stripped = stripLineComments(sourceText,);
 ```
 */
export function stripLineComments(text: string,): string {
  /**
   Text runs between comments, joined once at the end so the walk never rebuilds
   the result character by character.
   */
  const kept: string[] = [];
  for (let cursor = 0;;) {
    /**
     Next comment marker at or after the cursor; absent when there is no comment left.
     */
    const opening = text.indexOf(
      LINE_COMMENT_OPENING,
      cursor,
    );
    if (opening === (-1)) {
      kept.push(text.slice(cursor,),);
      return kept.join('',);
    }
    /**
     Character before the comment, kept because the marker starts after it.
     */
    kept.push(
      text.slice(
        cursor,
        opening,
      ),
    );
    /**
     Newline that ends the comment; absent when the comment runs to the end of the text.
     */
    const newline = text.indexOf(
      '\n',
      opening + LINE_COMMENT_OPENING.length,
    );
    if (newline === (-1))
      return kept.join('',);
    cursor = newline;
  }
}

//endregion Line comment removal

//region Icon call sites

/**
 Opening token of an icon call.
 */
const ICON_CALL_OPENING = 'icon(';

/**
 Sentinel returned by {@link parseIconCall} when the text at the opening token is
 not a call site. A unique symbol rather than `undefined`, so "not a call" is
 distinguishable from a call whose name happens to be empty.
 */
const NO_ICON_CALL: unique symbol = Symbol('text at the opening token is not an icon call',);

/**
 One icon call found in source text.
 */
type IconCall = {
  /**
   Bare icon name between the quotes.
   */
  readonly name: string;
  /**
   Index one past the call's closing parenthesis, where the next search starts.
   */
  readonly end: number;
};

/**
 Advances past every whitespace character starting at an index.
 
 @param text - text to read
 
 @param from - index of the first character to test
 
 @returns index of the first character that is not whitespace, or the text's length
 */
function skipWhitespace(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): number {
  for (let index = from;;) {
    if (!isWhitespace(text.charAt(index,),))
      return index;
    index += 1;
  }
}

/**
 Finds the first character that cannot continue an icon name.
 
 @param text - text to read
 
 @param from - index of the first character after the name's first
 
 @returns index of the first character outside the name alphabet
 */
function endOfIconName(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): number {
  for (let index = from;;) {
    if (!isIconNameCharacter(text.charAt(index,),))
      return index;
    index += 1;
  }
}

/**
 Reads one icon call starting at an opening token.
 
 The shape is the deleted pattern's, token for token: a word boundary before
 `icon(`, whitespace, a quote, a name over `[a-z][a-z0-9_]*`, a quote,
 whitespace, an optional comma, whitespace, and the closing parenthesis.
 
 @param text - text to read
 
 @param opening - index of the `i` of the opening token
 
 @returns the call's name and end index, or {@link NO_ICON_CALL} when the text there is not a call
 */
function parseIconCall(
  {
    text,
    opening,
  }: {
    readonly text: string;
    readonly opening: number;
  },
): IconCall | typeof NO_ICON_CALL {
  if (isAsciiWordCharacter(text.charAt(opening - 1,),))
    return NO_ICON_CALL;
  /**
   Index of the character after the opening token.
   */
  const afterOpening = opening + ICON_CALL_OPENING.length;
  /**
   Index of the first token after the whitespace that follows the opening parenthesis.
   */
  const nameQuote = skipWhitespace({
    text,
    from: afterOpening,
  },);
  if (!isQuote(text.charAt(nameQuote,),))
    return NO_ICON_CALL;
  /**
   Index of the name's first character.
   */
  const nameStart = nameQuote + 1;
  if (!isLowercaseAsciiLetter(text.charAt(nameStart,),))
    return NO_ICON_CALL;
  /**
   Index of the closing quote, past every character the name alphabet allows.
   */
  const closingQuote = endOfIconName({
    text,
    from: nameStart + 1,
  },);
  if (!isQuote(text.charAt(closingQuote,),))
    return NO_ICON_CALL;
  /**
   Index of the first token after the closing quote.
   */
  const afterQuote = skipWhitespace({
    text,
    from: closingQuote + 1,
  },);
  /**
   Index where the closing parenthesis must sit, past an optional comma and its
   whitespace.
   */
  const closingParenthesis = text.charAt(afterQuote,) === ','
    ? skipWhitespace({
      text,
      from: afterQuote + 1,
    },)
    : afterQuote;
  if (text.charAt(closingParenthesis,) !== ')')
    return NO_ICON_CALL;
  return {
    name: text.slice(
      nameStart,
      closingQuote,
    ),
    end: closingParenthesis + 1,
  };
}

/**
 Collects every `icon('NAME')` call site in source text, in order.
 
 Names repeat when a call repeats: the caller owns deduplication, because a
 component may legitimately call the same icon twice. Text at a candidate that
 turns out not to be a call is skipped one character at a time, so an `icon(`
 nested inside a failed candidate is still found.
 
 @param text - comment-free source text to scan
 
 @returns icon names in the order their calls appear
 
 @example
 ```ts
 collectIconCallNames("icon('star'); icon(\"cat\");");  // ['star', 'cat']
 ```
 */
export function collectIconCallNames(text: string,): readonly string[] {
  /**
   Names in call order, one entry per call site.
   */
  const names: string[] = [];
  for (let cursor = 0;;) {
    /**
     Next opening token at or after the cursor; absent when the text is done.
     */
    const opening = text.indexOf(
      ICON_CALL_OPENING,
      cursor,
    );
    if (opening === (-1))
      return names;
    /**
     The call read at this opening, or the sentinel when it is not one.
     */
    const call = parseIconCall({
      text,
      opening,
    },);
    if (call === NO_ICON_CALL) {
      cursor = opening + 1;
      continue;
    }
    names.push(call.name,);
    cursor = call.end;
  }
}

//endregion Icon call sites
