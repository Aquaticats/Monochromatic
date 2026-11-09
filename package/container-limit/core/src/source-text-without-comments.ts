/**
 A file's text with every comment replaced by blank space, so a command named in a comment is not read as one.

 Each language the repository writes its container commands in has its own comment syntax, and a `mise.toml` holds an
 inline `node` script inside a `"""` string, whose `//` comments follow JavaScript's syntax rather than TOML's.
 Blanking preserves offsets and line numbers, so a scan of the blanked text reports the file's own lines.
 */

import {
  blankRange,
  blockCommentEnd,
  closingMultiLineQuote,
  endOfQuotedString,
  endOfSingleLineString,
  isQuote,
  lineEnd,
} from './positions-in-source-text.ts';

//region Vocabulary

/**
 Languages whose comment syntax the scan knows.
 */
export type SourceLanguage = 'toml' | 'typescript' | 'yaml';

/**
 Three of these quotes, read together, open and close a multi-line TOML string.
 */
const TOML_MULTI_LINE_QUOTE = '"""';

//endregion Vocabulary

//region JavaScript comments

/**
 Blanks `//` and block comments and their text, leaving quoted strings alone.

 @param characters - the file's characters, changed in place
 
 @param from - first character to read
 
 @param to - first character after the range to read

 @example
 ```ts
 blankSlashComments({ characters, from: 0, to: characters.length });
 ```
 */
function blankSlashComments({
  characters,
  from,
  to,
}: {
  readonly characters: string[];
  readonly from: number;
  readonly to: number;
},): void {
  for (let index = from; index < to;) {
    if (isQuote(characters[index],)) {
      index = endOfQuotedString({
        characters,
        quote: index,
        to,
      },);
      continue;
    }
    if ((characters[index] === '/') && (characters[index + 1] === '/')) {
      /**
       Index of the newline that ends the line comment.
       */
      const end = lineEnd({
        characters,
        from: index,
        to,
      },);
      blankRange({
        characters,
        from: index,
        to: end,
      },);
      index = end;
      continue;
    }
    if ((characters[index] === '/') && (characters[index + 1] === '*')) {
      /**
       Index past the block comment's closing marker.
       */
      const end = blockCommentEnd({
        characters,
        from: index,
        to,
      },);
      blankRange({
        characters,
        from: index,
        to: end,
      },);
      index = end;
      continue;
    }
    index += 1;
  }
}

//endregion JavaScript comments

//region TOML and YAML comments

/**
 Blanks TOML `#` comments, and the `node` script comments inside `"""` strings.

 @param characters - the file's characters, changed in place

 @example
 ```ts
 blankTomlComments(characters);
 ```
 */
function blankTomlComments(characters: string[],): void {
  for (let index = 0; index < characters.length;) {
    if (characters[index] === '#') {
      /**
       Index of the newline that ends the comment.
       */
      const end = lineEnd({
        characters,
        from: index,
        to: characters.length,
      },);
      blankRange({
        characters,
        from: index,
        to: end,
      },);
      index = end;
      continue;
    }
    if (characters.slice(
      index,
      index + TOML_MULTI_LINE_QUOTE.length,
    )
      .join('',)
      === TOML_MULTI_LINE_QUOTE) {
      /**
       First character of the multi-line string's content.
       */
      const contentStart = index + TOML_MULTI_LINE_QUOTE.length;
      /**
       Index of the quote that closes the multi-line string.
       */
      const contentEnd = closingMultiLineQuote({
        characters,
        from: contentStart,
        marker: TOML_MULTI_LINE_QUOTE,
      },);
      blankSlashComments({
        characters,
        from: contentStart,
        to: contentEnd,
      },);
      index = contentEnd + TOML_MULTI_LINE_QUOTE.length;
      continue;
    }
    if (isQuote(characters[index],)) {
      index = endOfSingleLineString({
        characters,
        quote: index,
        to: characters.length,
      },);
      continue;
    }
    index += 1;
  }
}

/**
 Blanks `#` comments outside quoted scalars, as YAML and shell read them.

 @param characters - the file's characters, changed in place

 @example
 ```ts
 blankHashComments(characters);
 ```
 */
function blankHashComments(characters: string[],): void {
  for (let index = 0; index < characters.length;) {
    if (characters[index] === '#') {
      /**
       Index of the newline that ends the comment.
       */
      const end = lineEnd({
        characters,
        from: index,
        to: characters.length,
      },);
      blankRange({
        characters,
        from: index,
        to: end,
      },);
      index = end;
      continue;
    }
    if (isQuote(characters[index],)) {
      index = endOfSingleLineString({
        characters,
        quote: index,
        to: characters.length,
      },);
      continue;
    }
    index += 1;
  }
}

//endregion TOML and YAML comments

//region The blanked text

/**
 The text of a file with every comment replaced by blank space.

 Offsets and line numbers stay those of the original text, so a scan of this text reports the file's own lines.

 @param text - the file's text
 
 @param language - the language whose comment syntax the text holds

 @returns the text with comments blanked

 @example
 ```ts
 const blanked = stripSourceComments({ text, language: 'toml' });
 ```
 */
export function stripSourceComments({
  text,
  language,
}: {
  readonly text: string;
  readonly language: SourceLanguage;
},): string {
  /**
   The file's characters, whose comment characters this scan blanks.
   */
  const characters = text.split('',);
  if (language === 'typescript')
    blankSlashComments({
      characters,
      from: 0,
      to: characters.length,
    },);
  else if (language === 'toml')
    blankTomlComments(characters,);
  else
    blankHashComments(characters,);
  return characters.join('',);
}

//endregion The blanked text
