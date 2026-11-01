import { JsoncParseError, } from './errors.ts';

//region Whitespace

/**
 Characters JSON treats as insignificant whitespace.
 */
const JSON_WHITESPACE = new Set([
  ' ',
  '\t',
  '\n',
  '\r',
],);

/**
 Tests whether a character is JSON whitespace.
 
 @param char - Single character to test.
 
 @returns `true` for space, tab, newline, or carriage return.
 
 @example
 ```ts
 isJsonWhitespace(' '); // => true
 ```
 */
export function isJsonWhitespace(char: string,): boolean {
  return JSON_WHITESPACE.has(char,);
}

//endregion Whitespace

//region Scalars

/**
 Result of scanning a scalar token: the decoded value, the original source
 slice, and the offset just past the token.
 */
export type ScalarScan<T,> = {
  readonly value: T;
  readonly raw: string;
  readonly end: number;
};

/**
 Finds the offset just past a double-quoted string's closing quote, honoring
 backslash escapes. The loop's own increment skips one character; an escape
 skips one more, so `\"` and `\\` do not end the string early.
 
 @param source - Full JSONC source.
 
 @param openQuote - Offset of the opening quote.
 
 @returns Offset just past the closing quote.
 
 @throws JsoncParseError when the string is never closed.
 
 @example
 ```ts
 findStringEnd({ source: '"a\\n"', openQuote: 0 }); // => 5
 ```
 */
function findStringEnd({
  source,
  openQuote,
}: {
  readonly source: string;
  readonly openQuote: number;
},): number {
  for (let cursor = openQuote + 1; cursor < source.length; cursor += 1) {
    /**
     Character under the cursor.
     */
    const char = source[cursor];
    if (char === '"')
      return cursor + 1;
    if (char === '\\')
      cursor += 1;
  }
  throw new JsoncParseError({
    message: 'unterminated string',
    offset: openQuote,
  },);
}

/**
 Scans a double-quoted string starting at the opening quote. The decoded value
 is recovered with `JSON.parse` over the raw slice so the JSON grammar, not a
 hand-rolled escape table, defines the decoding.
 
 @param source - Full JSONC source.
 
 @param index - Offset of the opening quote.
 
 @returns Decoded string, raw quoted slice, and end offset.
 
 @throws JsoncParseError when the string is never closed.
 
 @example
 ```ts
 scanString({ source: '"a"', index: 0 });
 // => { value: 'a', raw: '"a"', end: 3 }
 ```
 */
export function scanString({
  source,
  index,
}: {
  readonly source: string;
  readonly index: number;
},): ScalarScan<string> {
  /**
   Offset just past the closing quote.
   */
  const end = findStringEnd({
    source,
    openQuote: index,
  },);
  /**
   Original quoted slice, including both quotes.
   */
  const raw = source.slice(
    index,
    end,
  );
  /* oxlint-disable typescript/no-unsafe-type-assertion -- a complete JSON string literal parses to a string */
  /**
   Decoded string value, recovered through the JSON grammar.
   */
  const value = JSON.parse(raw,) as string;
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  return {
    value,
    raw,
    end,
  };
}

/**
 Characters that may appear inside a JSON number token.
 */
const NUMBER_CHARS = new Set([
  '-',
  '+',
  '.',
  'e',
  'E',
  '0',
  '1',
  '2',
  '3',
  '4',
  '5',
  '6',
  '7',
  '8',
  '9',
],);

/**
 Finds the offset just past the run of number characters starting at `start`.
 
 @param source - Full JSONC source.
 
 @param start - Offset of the number's first character.
 
 @returns Offset just past the number run.
 
 @example
 ```ts
 findNumberEnd({ source: '12,', start: 0 }); // => 2
 ```
 */
function findNumberEnd({
  source,
  start,
}: {
  readonly source: string;
  readonly start: number;
},): number {
  for (let cursor = start; cursor < source.length; cursor += 1) {
    /**
     Character under the cursor.
     */
    const char = source[cursor];
    if ((char === undefined) || (!NUMBER_CHARS.has(char,)))
      return cursor;
  }
  return source.length;
}

/**
 Decodes a number slice with `JSON.parse`, which rejects malformed runs such as
 `1.2.3` or a leading-zero integer.
 
 @param raw - Raw number slice.
 
 @param offset - Offset of the slice, for error reporting.
 
 @returns Decoded number.
 
 @throws JsoncParseError when the slice is not a valid JSON number.
 
 @example
 ```ts
 decodeNumber({ raw: '1.5', offset: 0 }); // => 1.5
 ```
 */
function decodeNumber({
  raw,
  offset,
}: {
  readonly raw: string;
  readonly offset: number;
},): number {
  try {
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- a valid JSON number slice parses to a number
    return JSON.parse(raw,) as number;
  }
  catch (error: unknown) {
    throw new JsoncParseError({
      message: `invalid number ${JSON.stringify(raw,)}: ${String(error,)}`,
      offset,
    },);
  }
}

/**
 Scans a JSON number starting at `index`, then validates and decodes it.
 
 @param source - Full JSONC source.
 
 @param index - Offset of the number's first character.
 
 @returns Decoded number, raw slice, and end offset.
 
 @throws JsoncParseError when the run is not a valid JSON number.
 
 @example
 ```ts
 scanNumber({ source: '1.5,', index: 0 });
 // => { value: 1.5, raw: '1.5', end: 3 }
 ```
 */
export function scanNumber({
  source,
  index,
}: {
  readonly source: string;
  readonly index: number;
},): ScalarScan<number> {
  /**
   Offset just past the number run.
   */
  const end = findNumberEnd({
    source,
    start: index,
  },);
  /**
   Raw slice spanning the number run.
   */
  const raw = source.slice(
    index,
    end,
  );
  /**
   Decoded numeric value.
   */
  const value = decodeNumber({
    raw,
    offset: index,
  },);
  return {
    value,
    raw,
    end,
  };
}

/**
 Tests whether `keyword` appears in `source` exactly at `index`.
 
 @param source - Full JSONC source.
 
 @param index - Offset to test at.
 
 @param keyword - Literal keyword to match (`true`, `false`, or `null`).
 
 @returns `true` when the keyword starts at the offset.
 
 @example
 ```ts
 matchKeyword({ source: 'true,', index: 0, keyword: 'true' }); // => true
 ```
 */
export function matchKeyword({
  source,
  index,
  keyword,
}: {
  readonly source: string;
  readonly index: number;
  readonly keyword: string;
},): boolean {
  return source.startsWith(
    keyword,
    index,
  );
}

//endregion Scalars

//region Comments

/**
 Result of scanning a comment: its body (delimiters stripped, untrimmed) and
 the offset just past it.
 */
export type CommentScan = {
  readonly text: string;
  readonly end: number;
};

/**
 Reports whether an `indexOf` result found a match, dropping the -1 sentinel.
 
 @param offset - Candidate offset from `indexOf`.
 
 @returns Whether that offset exists in the searched source.
 
 @example
 ```ts
 ['// a\r\n', '// b'].map((source) => source.indexOf('\r')).filter(isFoundOffset);
 // => [2]
 ```
 */
function isFoundOffset(offset: number): boolean {
  return offset !== (-1);
}

/**
 Scans a `//` line comment starting at the first slash. The body runs to the
 next line feed, carriage return, or end of input; the terminator itself is not
 consumed.
 
 @param source - Full JSONC source.
 
 @param index - Offset of the first slash.
 
 @returns Comment body and end offset.
 
 @example
 ```ts
 scanLineComment({ source: '// hi\n', index: 0 });
 // => { text: ' hi', end: 5 }
 ```
 */
export function scanLineComment({
  source,
  index,
}: {
  readonly source: string;
  readonly index: number;
},): CommentScan {
  /**
   Offset of the next line feed, or -1 when none follows.
   */
  const lineFeed = source.indexOf(
    '\n',
    index,
  );
  /**
   Offset of the next carriage return, or -1 when none follows. A bare CR is a
   line ending in JSONC source, so a comment must not swallow the rest of the
   document on classic Mac or mixed line endings.
   */
  const carriageReturn = source.indexOf(
    '\r',
    index,
  );
  /**
   Offsets of line endings that actually follow the comment start.
   */
  const terminators = [
    lineFeed,
    carriageReturn,
  ].filter(isFoundOffset,);
  /**
   Offset of the comment body's end: the earliest line ending, or end of input.
   The terminator itself is left for whitespace scanning.
   */
  const end = (terminators.length === 0)
    ? source.length
    : Math.min(...terminators,);
  return {
    text: source.slice(
      index + 2,
      end,
    ),
    end,
  };
}

/**
 Scans a `/* *\/` block comment starting at the slash. The body runs to the
 first `*\/`; C-family block comments do not nest, so the first close wins.
 
 @param source - Full JSONC source.
 
 @param index - Offset of the opening slash.
 
 @returns Comment body and end offset.
 
 @throws JsoncParseError when the block is never closed.
 
 @example
 ```ts
 scanBlockComment({ source: '/* a *\/', index: 0 });
 // => { text: ' a ', end: 7 }
 ```
 */
export function scanBlockComment({
  source,
  index,
}: {
  readonly source: string;
  readonly index: number;
},): CommentScan {
  /**
   Offset of the closing delimiter, searched from just past the opener.
   */
  const close = source.indexOf(
    '*/',
    index + 2,
  );
  if (close === (-1))
    throw new JsoncParseError({
      message: 'unterminated block comment',
      offset: index,
    },);
  return {
    text: source.slice(
      index + 2,
      close,
    ),
    end: close + 2,
  };
}

//endregion Comments
