/**
 Where things begin and end in a file's text, read without regular expressions.

 The scan of container start sites (`container-starts-in-source-text.ts`), the blanking of comments
 (`source-text-without-comments.ts`) and the index of declared names (`definitions-in-source-text.ts`) all walk the same
 characters, and each needs the same small questions answered: where does this string end, where does this bracket pair
 close, is this character part of a name.

 Every position finder returns a position inside the range it was given, so a scan never has to ask whether a position
 exists.

 No regular expression is used: an index walk answers each of these questions.
 */

//region Vocabulary

/**
 Characters that can continue a name, so a program name inside a longer word is not a match.
 */
const NAME_CHARACTERS = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$';

/**
 Quote characters a string can start with.
 */
const QUOTES: ReadonlySet<string> = new Set([
  '"',
  '\'',
  '`',
]);

/**
 Bracket pairs whose contents the scans read as a block.
 */
const BRACKET_PAIRS: ReadonlyMap<string, string> = new Map([
  [
    '[',
    ']',
  ],
  [
    '(',
    ')',
  ],
  [
    '{',
    '}',
  ],
],);

/**
 Newline character, which a line-bounded scan stops at.
 */
const NEWLINE = '\n';

/**
 What a comment's characters become, so offsets and line numbers stay those of the original text.
 */
const BLANK = ' ';

//endregion Vocabulary

//region Character questions

/**
 Whether a character can continue a name.

 @param character - one character of the text, or nothing at the end of it

 @returns true for a letter, a digit, `_` or `$`

 @example
 ```ts
 const continues = isNameCharacter('a');
 ```
 */
export function isNameCharacter(character?: string,): boolean {
  if (character === undefined)
    return false;
  return NAME_CHARACTERS.includes(character,);
}

/**
 Whether a character continues a name that ends right before it.

 A letter after a backslash is the escaped character of a string (`\n` in a one-line `task.ts` script), not the tail of
 a name, so a declaration that follows it still starts a word.

 @param characters - the file's characters
 
 @param index - index of the character to read

 @returns true when the character is part of a name that started earlier

 @example
 ```ts
 const continues = continuesName({ characters, index: 4 });
 ```
 */
export function continuesName({
  characters,
  index,
}: {
  readonly characters: readonly string[];
  readonly index: number;
},): boolean {
  if (!isNameCharacter(characters[index],))
    return false;
  return characters[index - 1] !== '\\';
}

/**
 Whether a character is a quote.

 @param character - one character of the text, or nothing at the end of it

 @returns true for `"`, `'` or a backtick

 @example
 ```ts
 const opens = isQuote('"');
 ```
 */
export function isQuote(character?: string,): boolean {
  return (character !== undefined) && QUOTES.has(character,);
}

/**
 Whether a name starts at an index as a whole word.

 @param characters - the file's characters
 
 @param index - index of the first character to compare
 
 @param name - the name to compare against

 @returns true when the characters spell the name and neither neighbour continues a name

 @example
 ```ts
 const isProgram = nameMatchesAt({ characters, index, name: 'podman' });
 ```
 */
export function nameMatchesAt({
  characters,
  index,
  name,
}: {
  readonly characters: readonly string[];
  readonly index: number;
  readonly name: string;
},): boolean {
  if (continuesName({
    characters,
    index: index - 1,
  },))
    return false;
  if (characters.slice(
    index,
    index + name.length,
  )
    .join('',)
    !== name)
    return false;
  return !continuesName({
    characters,
    index: index + name.length,
  },);
}

//endregion Character questions

//region Blanking

/**
 Replaces the characters of one range with blank space, keeping newlines.

 @param characters - the file's characters, changed in place
 
 @param from - first character to blank
 
 @param to - first character to leave alone

 @example
 ```ts
 blankRange({ characters, from: 0, to: 4 });
 ```
 */
export function blankRange({
  characters,
  from,
  to,
}: {
  readonly characters: string[];
  readonly from: number;
  readonly to: number;
},): void {
  for (let index = from; index < to; index += 1) {
    if (characters[index] !== NEWLINE)
      characters[index] = BLANK;
  }
}

//endregion Blanking

//region Positions

/**
 Index of the newline that ends the line holding an index, or the end of the range.

 @param characters - the file's characters
 
 @param from - index to start looking at
 
 @param to - end of the range to read

 @returns the index of the newline, or the end of the range

 @example
 ```ts
 const end = lineEnd({ characters, from: 0, to: characters.length });
 ```
 */
export function lineEnd({
  characters,
  from,
  to,
}: {
  readonly characters: readonly string[];
  readonly from: number;
  readonly to: number;
},): number {
  /**
   Index the walk reached, which is the newline or the end of the range.
   */
  let index = from;
  while ((index < to) && (characters[index] !== NEWLINE))
    index += 1;
  return index;
}

/**
 Index just past the closing quote of the string that opens at a quote character.

 @param characters - the file's characters
 
 @param quote - index of the quote that opens the string
 
 @param to - end of the range to read

 @returns the index past the closing quote, or the end of the range when the string never closes

 @example
 ```ts
 const end = endOfQuotedString({ characters, quote: 0, to: characters.length });
 ```
 */
export function endOfQuotedString({
  characters,
  quote,
  to,
}: {
  readonly characters: readonly string[];
  readonly quote: number;
  readonly to: number;
},): number {
  /**
   The quote that closes the string is the one that opened it.
   */
  const quoteCharacter = characters[quote];
  /**
   Index past the character the walk reads, which ends past the closing quote.
   */
  let index = quote + 1;
  while (index < to) {
    if (characters[index] === '\\') {
      index = Math.min(
        index + 2,
        to,
      );
      continue;
    }
    if (characters[index] === quoteCharacter)
      return index + 1;
    index += 1;
  }
  return index;
}

/**
 Index just past the closing quote of a string that cannot run past the end of its line.

 TOML single-line strings and YAML and shell scalars end at their line, so an apostrophe in prose is not the start of
 a string: when no closing quote follows on that line, the scan resumes just past the quote character.

 @param characters - the file's characters
 
 @param quote - index of the quote that opens the string
 
 @param to - end of the range to read

 @returns the index past the closing quote, or past the opening one when the line holds none

 @example
 ```ts
 const end = endOfSingleLineString({ characters, quote: 0, to: characters.length });
 ```
 */
export function endOfSingleLineString({
  characters,
  quote,
  to,
}: {
  readonly characters: readonly string[];
  readonly quote: number;
  readonly to: number;
},): number {
  /**
   End of the line that holds the quote, which no single-line string crosses.
   */
  const lineStop = lineEnd({
    characters,
    from: quote,
    to,
  },);
  /**
   Index past the closing quote of a string that ends before the line does.
   */
  const end = endOfQuotedString({
    characters,
    quote,
    to: lineStop + 1,
  },);
  return end > lineStop ? quote + 1 : end;
}

/**
 Index just past a block comment that opens at an index, or the end of the range.

 @param characters - the file's characters
 
 @param from - index of the opening `/*`
 
 @param to - end of the range to read

 @returns the index past the closing `*\/`

 @example
 ```ts
 const end = blockCommentEnd({ characters, from: 4, to: characters.length });
 ```
 */
export function blockCommentEnd({
  characters,
  from,
  to,
}: {
  readonly characters: readonly string[];
  readonly from: number;
  readonly to: number;
},): number {
  for (let index = from + 2; index < to; index += 1) {
    if ((characters[index] === '*') && (characters[index + 1] === '/'))
      return index + 2;
  }
  return to;
}

/**
 Index of the closing triple quote of a multi-line TOML string, or the end of the characters.

 @param characters - the file's characters
 
 @param from - first character of the string's content
 
 @param marker - the three quote characters that close it

 @returns the index of the closing triple quote, or the length of the text

 @example
 ```ts
 const close = closingMultiLineQuote({ characters, from: 10, marker: '"""' });
 ```
 */
export function closingMultiLineQuote({
  characters,
  from,
  marker,
}: {
  readonly characters: readonly string[];
  readonly from: number;
  readonly marker: string;
},): number {
  for (let index = from; index < characters.length; index += 1) {
    if (characters.slice(
      index,
      index + marker.length,
    )
      .join('',)
      === marker)
      return index;
  }
  return characters.length;
}

/**
 Index just past the bracket pair that opens at an index, or the end of the range.

 @param characters - the file's characters
 
 @param opening - index of the bracket that opens the pair
 
 @param to - end of the range to read

 @returns the index past the closing bracket

 @example
 ```ts
 const end = bracketPairEnd({ characters, opening: 4, to: characters.length });
 ```
 */
export function bracketPairEnd({
  characters,
  opening,
  to,
}: {
  readonly characters: readonly string[];
  readonly opening: number;
  readonly to: number;
},): number {
  /**
   The character that closes the pair; a character that opens no pair closes on nothing.
   */
  const closing = BRACKET_PAIRS.get(characters[opening] ?? '',);
  if (closing === undefined)
    return to;
  /**
   The character that opened the pair.
   */
  const opens = characters[opening];
  /**
   Depth of the pairs opened so far.
   */
  let depth = 0;
  /**
   Index past the character the walk reads, which ends past the closing bracket.
   */
  let index = opening;
  while (index < to) {
    if (isQuote(characters[index],)) {
      index = endOfQuotedString({
        characters,
        quote: index,
        to,
      },);
      continue;
    }
    if (characters[index] === opens)
      depth += 1;
    else if (characters[index] === closing) {
      depth -= 1;
      if (depth === 0)
        return index + 1;
    }
    index += 1;
  }
  return index;
}

/**
 Index of the end of the statement that starts at an index: its semicolon, or the end of the text.

 @param characters - the file's characters
 
 @param from - index the statement starts at

 @returns the index past the statement's semicolon, or the length of the text

 @example
 ```ts
 const end = statementEnd({ characters, from: 4 });
 ```
 */
export function statementEnd({
  characters,
  from,
}: {
  readonly characters: readonly string[];
  readonly from: number;
},): number {
  /**
   Depth of the braces opened so far, so a brace inside an import clause does not end the statement.
   */
  let depth = 0;
  /**
   Index past the character the walk reads, which ends past the statement.
   */
  let index = from;
  while (index < characters.length) {
    /**
     Character at the scan's index, which opens a brace, closes one, or ends the statement.
     */
    const character = characters[index];
    if (isQuote(character,)) {
      index = endOfQuotedString({
        characters,
        quote: index,
        to: characters.length,
      },);
      continue;
    }
    if (character === '{')
      depth += 1;
    else if (character === '}')
      depth -= 1;
    else if ((character === ';') && (depth <= 0))
      return index + 1;
    index += 1;
  }
  return index;
}

//endregion Positions
