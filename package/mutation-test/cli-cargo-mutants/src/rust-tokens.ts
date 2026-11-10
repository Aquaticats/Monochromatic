/**
 A Rust source file reduced to the tokens the module and import scanners read: identifiers, string literals, numbers
 and punctuation, with comments, character literals and lifetimes dropped.

 It is a scanner for locating `mod` declarations and path prefixes, not a parser: it does not validate Rust, and it
 makes one linear pass over the text.
 */

import { isDigit } from '@monochromatic-dev/agent-harness-shared-text-scan/ts';

/**
 Characters an escaped character literal has at least: the apostrophe, the backslash, the escaped character, so the closing
 apostrophe is searched for from that far in.
 */
const escapedLiteralMinimumLength = 3;

/**
 One token of a Rust source file.
 */
export type RustToken = {
  /**
   `ident` for names and keywords, `string` for a string literal (its text is the content between the quotes, with
   escapes left as written), `number`, or `punct` for one punctuation character (`::` is one token).
   */
  readonly kind: 'ident' | 'number' | 'punct' | 'string';
  readonly text: string;
};

/**
 Check whether a character may continue an identifier.

 @param character - one UTF-16 code unit as text, or the empty text at the end of the text

 @returns true for ASCII letters, digits, the underscore and any non-ASCII character

 @example
 ```ts
 continuesIdentifier('a');
 ```
 */
function continuesIdentifier(character: string): boolean {
  return ((character >= 'a') && (character <= 'z'))
    || ((character >= 'A') && (character <= 'Z'))
    || ((character >= '0') && (character <= '9'))
    || (character === '_')
    || (character > '\u007F');
}

/**
 Find the end of a block comment, which Rust allows to nest.

 @param text - source text

 @param start - index just after the opening slash and star

 @returns the index just after the matching close, or the text length when it is unterminated

 @example
 ```ts
 blockCommentEnd({ text: '/* a *\/ b', start: 2 });
 ```
 */
function blockCommentEnd({
  text,
  start,
}: {
  readonly text: string;
  readonly start: number;
}): number {
  // `depth` counts the open comments, the one being skipped included.
  for (let depth = 1, index = start; index < text.length;) {
    if (text.startsWith(
      '/*',
      index,
    )) {
      depth += 1;
      index += 2;
    }
    else if (text.startsWith(
      '*/',
      index,
    )) {
      depth -= 1;
      index += 2;
      if (depth === 0)
        return index;
    }
    else
      index += 1;
  }
  return text.length;
}

/**
 Find the end of a string literal that escapes with a backslash.

 @param text - source text

 @param start - index just after the opening quote

 @returns the index of the closing quote, or the text length when it is unterminated

 @example
 ```ts
 stringEnd({ text: '"a" b', start: 1 });
 ```
 */
function stringEnd({
  text,
  start,
}: {
  readonly text: string;
  readonly start: number;
}): number {
  for (let index = start; index < text.length; index += text[index] === '\\' ? 2 : 1) {
    if (text[index] === '"')
      return index;
  }
  return text.length;
}

/**
 Find the end of a raw string literal.

 @param text - source text

 @param start - index just after the opening quote

 @param hashes - number of `#` signs the literal opened with

 @returns the index of the closing quote, or the text length when it is unterminated

 @example
 ```ts
 rawStringEnd({ text: 'r#"a"# b', start: 3, hashes: 1 });
 ```
 */
function rawStringEnd({
  text,
  start,
  hashes,
}: {
  readonly text: string;
  readonly start: number;
  readonly hashes: number;
}): number {
  /**
   The closing quote and its hashes.
   */
  const closing = `"${'#'.repeat(hashes)}`;
  /**
   Where the closing text starts.
   */
  const found = text.indexOf(
    closing,
    start,
  );
  return found === (-1) ? text.length : found;
}

/**
 Find the end of a character literal or a lifetime that starts with an apostrophe.

 @param text - source text

 @param start - index of the apostrophe

 @returns the index just after the character literal or lifetime

 @example
 ```ts
 apostropheEnd({ text: "'a' b", start: 0 });
 ```
 */
function apostropheEnd({
  text,
  start,
}: {
  readonly text: string;
  readonly start: number;
}): number {
  if (text[start + 1] === '\\') {
    /**
     The closing apostrophe of an escaped character.
     */
    const closing = text.indexOf(
      '\'',
      start + escapedLiteralMinimumLength,
    );
    return closing === (-1) ? text.length : closing + 1;
  }
  /**
   Length in UTF-16 code units of the character after the apostrophe.
   */
  const width = String.fromCodePoint(text.codePointAt(start + 1) ?? 0)
    .length;
  if (text[start + 1
    + width] === '\'')
    return start + width
      + 2;
  // A lifetime: the apostrophe and the name after it.
  for (let index = start + 1; ; index += 1) {
    if (!continuesIdentifier(text[index] ?? ''))
      return index;
  }
}

/**
 Split Rust source text into tokens.

 @param text - the contents of a `.rs` file

 @returns the tokens in source order; comments, character literals and lifetimes leave none

 @example
 ```ts
 tokenizeRust('use crate::a;');
 ```
 */
export function tokenizeRust(text: string): readonly RustToken[] {
  /**
   Tokens found so far.
   */
  const tokens: RustToken[] = [];
  for (let index = 0; index < text.length;) {
    /**
     The character at the index.
     */
    const character = text[index] ?? '';
    if ((character === ' ') || (character === '\n')
      || (character === '\t')
      || (character === '\r'))
      index += 1;
    else if (text.startsWith(
      '//',
      index,
    )) {
      /**
       The newline that ends the line comment.
       */
      const newline = text.indexOf(
        '\n',
        index,
      );
      index = newline === (-1) ? text.length : newline;
    }
    else if (text.startsWith(
      '/*',
      index,
    ))
      index = blockCommentEnd({
        text,
        start: index + 2,
      });
    else if (character === '"') {
      /**
       Index of the closing quote.
       */
      const end = stringEnd({
        text,
        start: index + 1,
      });
      tokens.push({
        kind: 'string',
        text: text.slice(
          index + 1,
          end,
        ),
      });
      index = end + 1;
    }
    else if (character === '\'')
      index = apostropheEnd({
        text,
        start: index,
      });
    else if (isDigit(character)) {
      /**
       Index just after the number.
       */
      let end = index;
      while (continuesIdentifier(text[end] ?? ''))
        end += 1;
      tokens.push({
        kind: 'number',
        text: text.slice(
          index,
          end,
        ),
      });
      index = end;
    }
    else if (continuesIdentifier(character)) {
      /**
       Index just after the identifier.
       */
      let end = index;
      while (continuesIdentifier(text[end] ?? ''))
        end += 1;
      /**
       The identifier read.
       */
      const word = text.slice(
        index,
        end,
      );
      /**
       Whether the word is a prefix of a string or character literal (`r`, `b`, `br`, `c`, `cr`).
       */
      const prefixesLiteral = [
        'r',
        'b',
        'br',
        'c',
        'cr'
      ].includes(word);
      if (prefixesLiteral && (text[end] === '"')) {
        // A byte or C string with escapes, or a raw string without hashes.
        /**
         Index of the closing quote.
         */
        const close = word.includes('r')
          ? rawStringEnd({
            text,
            start: end + 1,
            hashes: 0,
          })
          : stringEnd({
            text,
            start: end + 1,
          });
        tokens.push({
          kind: 'string',
          text: text.slice(
            end + 1,
            close,
          ),
        });
        index = close + 1;
      }
      else if (prefixesLiteral && word.includes('r')
        && (text[end] === '#')) {
        /**
         Index just after the hashes.
         */
        let afterHashes = end;
        while (text[afterHashes] === '#')
          afterHashes += 1;
        if (text[afterHashes] === '"') {
          /**
           Index of the closing quote.
           */
          const close = rawStringEnd({
            text,
            start: afterHashes + 1,
            hashes: afterHashes - end,
          });
          tokens.push({
            kind: 'string',
            text: text.slice(
              afterHashes + 1,
              close,
            ),
          });
          index = close + 1
            + (afterHashes - end);
        }
        else {
          // A raw identifier such as `r#type`: the name after the hash is the identifier.
          /**
           Index just after the raw identifier.
           */
          let rawEnd = afterHashes;
          while (continuesIdentifier(text[rawEnd] ?? ''))
            rawEnd += 1;
          tokens.push({
            kind: 'ident',
            text: text.slice(
              afterHashes,
              rawEnd,
            ),
          });
          index = rawEnd;
        }
      }
      else if ((word === 'b') && (text[end] === '\''))
        index = apostropheEnd({
          text,
          start: end,
        });
      else {
        tokens.push({
          kind: 'ident',
          text: word,
        });
        index = end;
      }
    }
    else if (text.startsWith(
      '::',
      index,
    )) {
      tokens.push({
        kind: 'punct',
        text: '::',
      });
      index += 2;
    }
    else {
      tokens.push({
        kind: 'punct',
        text: character,
      });
      index += 1;
    }
  }
  return tokens;
}

/**
 Check for a punctuation token at a position. A string literal with the same text (`"["`) is not one.

 @param tokens - source tokens

 @param index - position to check, which may lie past the end

 @param text - the punctuation

 @returns true when the token at the position is that punctuation

 @example
 ```ts
 isPunct({ tokens, index: 0, text: '#' });
 ```
 */
export function isPunct({
  tokens,
  index,
  text,
}: {
  readonly tokens: readonly RustToken[];
  readonly index: number;
  readonly text: string;
}): boolean {
  return (tokens[index]
    ?.kind
    === 'punct') && (tokens[index]
      .text
      === text);
}
