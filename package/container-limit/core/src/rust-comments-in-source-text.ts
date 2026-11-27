/**
 A Rust source with its comments replaced by blank space, so a start written in one is not read as a start.

 Rust's comment syntax is not the one the other slash language the scan reads holds: its block comments nest, so a
 comment inside a comment ends at the inner marker and not the outer one, and it writes raw strings, char literals and
 lifetimes, whose quotes and apostrophes a reader that only knows JavaScript's strings pairs with each other and blanks
 the wrong text with.

 A start in a Rust source is a call that names the container program (`Command::new("podman")`) followed by the calls
 that hand it the arguments, so a mention inside a comment, a doc comment or a message is not a start, and the `//`, `///`
 and `//!` forms of a comment are all blanked here.
 */

import {
  blankRange,
  endOfQuotedString,
  lineEnd,
} from './positions-in-source-text.ts';

//region Vocabulary

/**
 Characters that turn a raw string into a byte string or a C string, so its `r` comes second.
 */
const RUST_STRING_PREFIXES = 'bc';

/**
 The character that opens a raw string, whose text holds no escape and may hold any quote.
 */
const RUST_RAW_STRING_MARK = 'r';

/**
 The character that marks how many of them wrap the quote that opens and closes a raw string.
 */
const RUST_RAW_STRING_MARK_CHARACTER = '#';

/**
 The quote that opens and closes a Rust string.
 */
const QUOTE = '"';

/**
 The apostrophe that opens a char literal or names a lifetime.
 */
const APOSTROPHE = '\'';

/**
 The character that opens an escape inside a Rust literal.
 */
const BACKSLASH = '\\';

/**
 What a `\u{...}` escape writes between the backslash and its digits.
 */
const UNICODE_ESCAPE_OPENING = 'u{';

/**
 The character that closes the braces of a `\u{...}` escape.
 */
const UNICODE_ESCAPE_CLOSING = '}';

/**
 The width of a backslash and the one character it escapes.
 */
const ESCAPED_CHARACTER_WIDTH = BACKSLASH.length + 1;

/**
 The index of a char literal's single character, counted from the apostrophe that opens it.
 */
const CHAR_LITERAL_CHARACTER = APOSTROPHE.length;

/**
 The index of a char literal's closing apostrophe, counted from the apostrophe that opens it.
 */
const CHAR_LITERAL_CLOSING = CHAR_LITERAL_CHARACTER + APOSTROPHE.length;

/**
 The width of a char literal that holds one character, past the apostrophe that opens it.
 */
const CHAR_LITERAL_WIDTH = CHAR_LITERAL_CLOSING + APOSTROPHE.length;

//endregion Vocabulary

//region Block comments

/**
 Index just past the Rust block comment that opens at an index.

 Rust's block comments nest (`/* a /* b *\/ c *\/` is one comment), unlike the two other slash languages the scan reads,
 so the walk counts the comments it opens instead of stopping at the first closing marker.

 @param characters - the file's characters
 
 @param from - index of the opening `/*`
 
 @param to - end of the range to read

 @returns the index past the outer comment's closing marker, or the end of the range when it never closes

 @example
 ```ts
 const end = rustBlockCommentEnd({ characters, from: 4, to: characters.length });
 ```
 */
function rustBlockCommentEnd({
  characters,
  from,
  to,
}: {
  readonly characters: readonly string[];
  readonly from: number;
  readonly to: number;
},): number {
  /**
   Comments opened so far and not yet closed.
   */
  let depth = 0;
  /**
   Index past the character the walk reads.
   */
  let index = from;
  /**
   Index past the closing marker of the outermost comment, or the end of the range when none closes it.
   */
  let end = to;
  while (index < to) {
    if ((characters[index] === '/') && (characters[index + 1] === '*')) {
      depth += 1;
      index += 2;
      continue;
    }
    if ((characters[index] === '*') && (characters[index + 1] === '/')) {
      depth -= 1;
      index += 2;
      if (depth === 0) {
        end = index;
        break;
      }
      continue;
    }
    index += 1;
  }
  return end;
}

//endregion Block comments

//region Literals

/**
 Index just past the escape that opens with a backslash at an index.

 @param characters - the file's characters
 
 @param index - index of the backslash
 
 @param to - end of the range to read

 @returns the index past the escaped character, or past the braces of a `\u{...}` escape

 @example
 ```ts
 const end = rustEscapeEnd({ characters, index: 20, to: characters.length });
 ```
 */
function rustEscapeEnd({
  characters,
  index,
  to,
}: {
  readonly characters: readonly string[];
  readonly index: number;
  readonly to: number;
},): number {
  /**
   Index of the first character of the escape's body, past the backslash.
   */
  const body = index + BACKSLASH.length;
  /**
   Index of the first digit of a `\u{...}` escape, past its opening.
   */
  const digits = body + UNICODE_ESCAPE_OPENING.length;
  if (characters.slice(
    body,
    digits,
  )
    .join('',)
    === UNICODE_ESCAPE_OPENING) {
    for (let scan = digits; scan < to; scan += 1) {
      if (characters[scan] === UNICODE_ESCAPE_CLOSING)
        return scan + 1;
    }
    return to;
  }
  return Math.min(
    index + ESCAPED_CHARACTER_WIDTH,
    to,
  );
}

/**
 Index just past the Rust char literal that opens at an apostrophe.

 An apostrophe also names a lifetime (`&'a str`), which is code: only an apostrophe whose single character or escape is
 followed by another apostrophe opens a literal.

 @param characters - the file's characters
 
 @param index - index of the apostrophe
 
 @param to - end of the range to read

 @returns the index past the literal, and nothing when the apostrophe names a lifetime

 @example
 ```ts
 const { end } = rustCharLiteralEnd({ characters, index: 12, to: characters.length });
 ```
 */
function rustCharLiteralEnd({
  characters,
  index,
  to,
}: {
  readonly characters: readonly string[];
  readonly index: number;
  readonly to: number;
},): { readonly end?: number; } {
  if (characters[index + CHAR_LITERAL_CHARACTER] === BACKSLASH) {
    /**
     Index past the escape the literal holds.
     */
    const escapeEnd = rustEscapeEnd({
      characters,
      index: index + CHAR_LITERAL_CHARACTER,
      to,
    },);
    if (characters[escapeEnd] === APOSTROPHE)
      return { end: escapeEnd + APOSTROPHE.length, };
    return {};
  }
  if ((characters[index + CHAR_LITERAL_CLOSING] === APOSTROPHE)
    && (characters[index + CHAR_LITERAL_CHARACTER] !== APOSTROPHE))
    return { end: index + CHAR_LITERAL_WIDTH, };
  return {};
}

/**
 How many marks wrap the quote that opens a raw string, and where that quote sits.

 @param characters - the file's characters
 
 @param from - index where the marks start, just past the `r`
 
 @param to - end of the range to read

 @returns the opening quote's index and the number of marks, and nothing when the opening is not a raw string

 @example
 ```ts
 const { quote, marks } = rustRawStringOpening({ characters, index: 4, to: characters.length });
 ```
 */
function rustRawStringOpening({
  characters,
  from,
  to,
}: {
  readonly characters: readonly string[];
  readonly from: number;
  readonly to: number;
},): {
  readonly marks?: number;
  readonly quote?: number;
} {
  /**
   Marks written before the quote.
   */
  const marks = rustRawStringMarkCount({
    characters,
    from,
    to,
  },);
  /**
   Index of the quote, past every mark before it.
   */
  const quote = from + marks;
  if (characters[quote] !== QUOTE)
    return {};
  return {
    marks,
    quote,
  };
}

/**
 How many marks the raw string that opens at an index writes before its quote.

 @param characters - the file's characters
 
 @param from - index where the marks start, just past the `r`
 
 @param to - end of the range to read

 @returns the number of marks, which is none for the plain `r"..."` form

 @example
 ```ts
 const marks = rustRawStringMarkCount({ characters, from: 5, to: characters.length });
 ```
 */
function rustRawStringMarkCount({
  characters,
  from,
  to,
}: {
  readonly characters: readonly string[];
  readonly from: number;
  readonly to: number;
},): number {
  /**
   Marks read so far.
   */
  let marks = 0;
  /**
   Index past the character the walk reads.
   */
  let index = from;
  while ((index < to) && (characters[index] === RUST_RAW_STRING_MARK_CHARACTER)) {
    marks += 1;
    index += 1;
  }
  return marks;
}

/**
 Whether a raw string's closing marks sit at an index.

 @param characters - the file's characters
 
 @param index - index just past the quote the marks would follow
 
 @param marks - how many marks the string's opening wrote
 
 @param to - end of the range to read

 @returns true when the quote closes the string, which is when as many marks follow it as opened it

 @example
 ```ts
 const closes = rustRawStringClosesAt({ characters, index: 40, marks: 2, to: characters.length });
 ```
 */
function rustRawStringClosesAt({
  characters,
  index,
  marks,
  to,
}: {
  readonly characters: readonly string[];
  readonly index: number;
  readonly marks: number;
  readonly to: number;
},): boolean {
  /**
   Index just past the marks that would close the string, which the range must still hold.
   */
  const pastTheMarks = index + marks;
  if (pastTheMarks > to)
    return false;
  for (let read = 0; read < marks; read += 1) {
    if (characters[index + read] !== RUST_RAW_STRING_MARK_CHARACTER)
      return false;
  }
  return true;
}

/**
 Index just past the closing marks of a raw string.

 @param characters - the file's characters
 
 @param from - index of the first character of the string's text
 
 @param marks - how many marks the string's opening wrote
 
 @param to - end of the range to read

 @returns the index past the closing quote and its marks, or the end of the range when nothing closes it

 @example
 ```ts
 const end = rustRawStringEnd({ characters, from: 8, marks: 1, to: characters.length });
 ```
 */
function rustRawStringEnd({
  characters,
  from,
  marks,
  to,
}: {
  readonly characters: readonly string[];
  readonly from: number;
  readonly marks: number;
  readonly to: number;
},): number {
  /**
   Index past the string's closing marks, or the end of the range when none closes it.
   */
  let end = to;
  for (let index = from; index < to; index += 1) {
    if (characters[index] !== QUOTE)
      continue;
    if (!rustRawStringClosesAt({
      characters,
      index: index + QUOTE.length,
      marks,
      to,
    },))
      continue;
    /**
     Index just past the quote that closes the string.
     */
    const quoteEnd = index + QUOTE.length;
    end = quoteEnd + marks;
    break;
  }
  return end;
}

/**
 Index just past the Rust raw string that opens at an index, when one opens there.

 A raw string is `r"..."`, `r#"..."#`, `r##"..."##` or more, with `b` or `c` before the `r` for a byte or C string: its
 text holds no escape, so the marks around the closing quote are what end it, and a `"` inside it closes nothing.

 @param characters - the file's characters
 
 @param index - index where the prefix and the `r` start
 
 @param to - end of the range to read

 @returns the index past the closing marks, and nothing when no raw string opens here

 @example
 ```ts
 const { end } = rustRawStringEnd({ characters, index: 4, to: characters.length });
 ```
 */
function rustRawStringLiteralEnd({
  characters,
  index,
  to,
}: {
  readonly characters: readonly string[];
  readonly index: number;
  readonly to: number;
},): { readonly end?: number; } {
  /**
   Index of the `r` that opens the raw string, past a byte or C string's prefix when one is there.
   */
  const mark = RUST_STRING_PREFIXES.includes(characters[index] ?? '',)
    ? index + 1
    : index;
  if (characters[mark] !== RUST_RAW_STRING_MARK)
    return {};
  /**
   The opening quote and how many marks it wrote.
   */
  const {
    marks,
    quote,
  } = rustRawStringOpening({
    characters,
    from: mark + RUST_RAW_STRING_MARK.length,
    to,
  },);
  if ((marks === undefined) || (quote === undefined))
    return {};
  return {
    end: rustRawStringEnd({
      characters,
      from: quote + QUOTE.length,
      marks,
      to,
    },),
  };
}

/**
 Index just past the Rust string or char literal that opens at an index, when one opens there.

 @param characters - the file's characters
 
 @param index - index to read at
 
 @param to - end of the range to read

 @returns the index past the literal, and nothing when the character opens none

 @example
 ```ts
 const { end } = rustLiteralEnd({ characters, index: 4, to: characters.length });
 ```
 */
function rustLiteralEnd({
  characters,
  index,
  to,
}: {
  readonly characters: readonly string[];
  readonly index: number;
  readonly to: number;
},): { readonly end?: number; } {
  if (characters[index] === QUOTE)
    return {
      end: endOfQuotedString({
        characters,
        quote: index,
        to,
      },),
    };
  if (characters[index] === APOSTROPHE)
    return rustCharLiteralEnd({
      characters,
      index,
      to,
    },);
  return rustRawStringLiteralEnd({
    characters,
    index,
    to,
  },);
}

//endregion Literals

//region The blanked text

/**
 Blanks Rust's `//` comments, its nested block comments, and its doc comments, which are those two forms again.

 @param characters - the file's characters, changed in place

 @example
 ```ts
 blankRustComments(characters);
 ```
 */
export function blankRustComments(characters: string[],): void {
  for (let index = 0; index < characters.length;) {
    if ((characters[index] === '/') && (characters[index + 1] === '/')) {
      /**
       Index of the newline that ends the line comment.
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
    if ((characters[index] === '/') && (characters[index + 1] === '*')) {
      /**
       Index past the block comment, including the comments nested inside it.
       */
      const end = rustBlockCommentEnd({
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
    /**
     Index past the string or char literal that opens here, when one does, so the comment markers inside it stay text.
     */
    const { end: literalEnd, } = rustLiteralEnd({
      characters,
      index,
      to: characters.length,
    },);
    if (literalEnd !== undefined) {
      index = literalEnd;
      continue;
    }
    index += 1;
  }
}

//endregion The blanked text
