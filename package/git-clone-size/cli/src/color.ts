import {
  styleText,
  type InspectColor,
} from 'node:util';

/**
 Color mode: `auto` decides from the stream and environment, `always`/`never`
 force the choice.
 */
export type ColorMode = 'auto' | 'always' | 'never';

/**
 Characters that may appear inside a JSON number token.
 */
const NUMBER_CHARS = '-+.eE0123456789';

/**
 Decides whether to emit ANSI color. `styleText` under Bun does not reliably
 honor a non-TTY stream or `NO_COLOR`, so the decision is made explicitly here
 and `styleText` is invoked only when color is wanted. Honors `--color`,
 `NO_COLOR`, `FORCE_COLOR`, and the stream's TTY status, in that order.
 
 @param mode - requested color mode
 
 @param stream - destination stream, checked for `isTTY` under `auto`
 
 @returns true when ANSI color should be emitted
 
 @example
 ```ts
 const on = shouldColor({ mode: 'auto', stream: process.stdout });
 ```
 */
export function shouldColor(
  {
    mode,
    stream,
  }: {
    readonly mode: ColorMode;
    readonly stream: { readonly isTTY?: boolean; }
  },
): boolean {
  if (mode === 'never')
    return false;
  if (mode === 'always')
    return true;
  /**
   `NO_COLOR` disables color when present and non-empty.
   */
  const noColor = process.env
    .NO_COLOR;
  if ((noColor !== undefined) && (noColor !== ''))
    return false;
  /**
   `FORCE_COLOR` enables color unless explicitly `0`/`false`/empty.
   */
  const forceColor = process.env
    .FORCE_COLOR;
  if (forceColor !== undefined)
    return (forceColor !== '0') && (forceColor !== 'false')
      && (forceColor !== '');
  return stream.isTTY === true;
}

/**
 Whether a character can start a JSON number token.
 
 @param ch - single character
 
 @returns true for a digit or leading minus sign
 */
function isNumberStart(ch: string,): boolean {
  return ((ch >= '0') && (ch <= '9')) || (ch === '-');
}

/**
 Reads a complete JSON string token (including quotes), honoring backslash
 escapes.
 
 @param json - serialized JSON
 
 @param start - index of the opening quote
 
 @returns the quoted token and the index just past the closing quote
 */
function readString(
  {
    json,
    start,
  }: {
    readonly json: string;
    readonly start: number
  },
): {
  readonly token: string;
  readonly next: number
} {
  /**
   Scan cursor starting just inside the opening quote.
   */
  const cursor = { i: start + 1, };
  while (cursor.i < json.length) {
    /**
     Current character, undefined past the end.
     */
    const ch = json[cursor.i];
    if (ch === undefined)
      break;
    if (ch === '\\') {
      cursor.i += 2;
      continue;
    }
    if (ch === '"') {
      cursor.i += 1;
      break;
    }
    cursor.i += 1;
  }
  return {
    next: cursor.i,
    token: json.slice(
      start,
      cursor.i,
    ),
  };
}

/**
 Reads a complete JSON number token.
 
 @param json - serialized JSON
 
 @param start - index of the first number character
 
 @returns the number token and the index just past it
 */
function readNumber(
  {
    json,
    start,
  }: {
    readonly json: string;
    readonly start: number
  },
): {
  readonly token: string;
  readonly next: number
} {
  /**
   Scan cursor over the number characters.
   */
  const cursor = { i: start, };
  while (cursor.i < json.length) {
    /**
     Current character, undefined past the end.
     */
    const ch = json[cursor.i];
    if ((ch === undefined) || (!NUMBER_CHARS.includes(ch,)))
      break;
    cursor.i += 1;
  }
  return {
    next: cursor.i,
    token: json.slice(
      start,
      cursor.i,
    ),
  };
}

/**
 First non-space character at or after an index, or empty string at the end.
 
 @param json - serialized JSON
 
 @param from - starting index
 
 @returns the next non-space character, or `''`
 */
function peekNonSpace({
  json,
  from,
}: {
  readonly json: string;
  readonly from: number
},): string {
  /**
   Scan cursor skipping spaces.
   */
  const cursor = { i: from, };
  while ((cursor.i < json.length) && (json[cursor.i] === ' '))
    cursor.i += 1;
  return json[cursor.i] ?? '';
}

/**
 Applies ANSI styling after {@link shouldColor} already decided color is allowed.
 
 `node:util`'s {@link styleText} checks `NO_COLOR` by default. That is right for
 direct calls, but this module already centralizes that policy in
 {@link shouldColor}. Disabling the second check keeps explicit color-on output
 deterministic during tests that temporarily mutate color environment variables.
 
 @param format - ANSI style names accepted by {@link styleText}
 
 @param text - JSON token text to wrap
 
 @returns ANSI-styled text without another environment-variable check
 
 @example
 ```ts
 forceStyleText({ format: 'cyan', text: '"key"' });
 ```
 */
function forceStyleText({
  format,
  text,
}: {
  readonly format: InspectColor;
  readonly text: string;
},): string {
  return styleText(
    format,
    text,
    { validateStream: false, },
  );
}

/**
 ANSI-highlights the tokens of a serialized JSON string via a single linear
 scan (no regex): keys cyan, string values green, numbers yellow, booleans
 magenta, null dim. Stripping the ANSI escapes restores the exact JSON.
 
 @param json - serialized JSON line
 
 @returns the same JSON with ANSI color applied to its tokens
 
 @example
 ```ts
 colorizeJson({ json: '{"a":1}' });
 ```
 */
export function colorizeJson({ json, }: { readonly json: string; },): string {
  /**
   Output fragments accumulated in scan order.
   */
  const out: string[] = [];
  /**
   Scan cursor over the JSON text.
   */
  const cursor = { i: 0, };
  while (cursor.i < json.length) {
    /**
     Current character, undefined past the end.
     */
    const ch = json[cursor.i];
    if (ch === undefined)
      break;
    if (ch === '"') {
      /**
       String token (with quotes) and the index past it.
       */
      const read = readString({
        json,
        start: cursor.i,
      },);
      /**
       A string is a key when the next non-space character is a colon.
       */
      const isKey = peekNonSpace({
        json,
        from: read.next,
      },) === ':';
      out.push(forceStyleText({
        format: isKey ? 'cyan' : 'green',
        text: read.token,
      },),);
      cursor.i = read.next;
      continue;
    }
    if (isNumberStart(ch,)) {
      /**
       Number token and the index past it.
       */
      const read = readNumber({
        json,
        start: cursor.i,
      },);
      out.push(forceStyleText({
        format: 'yellow',
        text: read.token,
      },),);
      cursor.i = read.next;
      continue;
    }
    if (json.startsWith(
      'true',
      cursor.i,
    ) || json.startsWith(
      'false',
      cursor.i,
    )) {
      /**
       Boolean literal length at this position.
       */
      const length = json.startsWith(
        'true',
        cursor.i,
      ) ? 'true'.length : 'false'.length;
      out.push(forceStyleText({
        format: 'magenta',
        text: json.slice(
          cursor.i,
          cursor.i + length,
        ),
      },),);
      cursor.i += length;
      continue;
    }
    if (json.startsWith(
      'null',
      cursor.i,
    )) {
      out.push(forceStyleText({
        format: 'dim',
        text: 'null',
      },),);
      cursor.i += 'null'.length;
      continue;
    }
    out.push(ch,);
    cursor.i += 1;
  }
  return out.join('',);
}
