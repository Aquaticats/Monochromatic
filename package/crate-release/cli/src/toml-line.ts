import {
  foundValue,
  type Lookup,
  missingValue,
} from './lookup.ts';

/**
 Where the value of a quoted TOML string sits in a line.
 */
export type ValueLocation = {
  readonly value: string;
  /**
   Index of the first character of the value, just after the opening quote.
   */
  readonly valueStart: number;
  /**
   Index of the closing quote.
   */
  readonly valueEnd: number;
};

/**
 Manifest text split into lines, with the carriage returns set aside so they can be put back.
 */
export type SplitManifest = {
  /**
   Line bodies without their carriage return.
   */
  readonly bodies: readonly string[];
  /**
   Flag per line for a trailing carriage return.
   */
  readonly hadReturn: readonly boolean[];
};

/**
 Characters that separate tokens inside a TOML line without carrying meaning.
 */
export const blankCharacters = ' \t';

/**
 Quote characters of a TOML string: basic and literal.
 */
export const quoteCharacters = '"\'';

/**
 Carriage return that ends a line of a CRLF manifest.
 */
export const carriageReturn = '\r';

/**
 Index of the first character that is not a space or tab, starting at a position.

 @param line - line under scan

 @param from - first index to look at

 @returns the index of the first non-blank character, or the line length when only blanks remain

 @example
 ```ts
 skipBlanks({ line: '  a', from: 0 });
 ```
 */
export function skipBlanks({
  line,
  from,
}: {
  readonly line: string;
  readonly from: number;
}): number {
  /**
   Distance from the start position to the first non-blank character, or minus one when only blanks remain.
   */
  const offset = Array.from(line.slice(from))
    .findIndex(function isNonBlank(character) {
      return !blankCharacters.includes(character);
    });
  return offset === (-1) ? line.length : from + offset;
}

/**
 Read `key = "value"` or `key = 'value'` at a position of a line.

 @param line - line under scan, without its carriage return

 @param from - index where the key must begin

 @param key - key expected there

 @returns the value and its place, or a miss when the text there is not that key followed by `=` and a quoted string
 with no backslash

 @example
 ```ts
 locateValue({ line: 'version = "1.0.0"', from: 0, key: 'version' });
 ```
 */
export function locateValue({
  line,
  from,
  key,
}: {
  readonly line: string;
  readonly from: number;
  readonly key: string;
}): Lookup<ValueLocation> {
  if (!line.startsWith(
    key,
    from
  ))
    return missingValue(`The line has no ${key} key at index ${String(from)}.`);
  /**
   Index of the equals sign, after optional blanks.
   */
  const equals = skipBlanks({
    line,
    from: from + key.length
  });
  if (line.charAt(equals) !== '=')
    return missingValue(`The ${key} key is not followed by an equals sign.`);
  /**
   Index of the opening quote, after optional blanks.
   */
  const open = skipBlanks({
    line,
    from: equals + 1
  });
  /**
   The quote character, if the value is a quoted string.
   */
  const quote = line.charAt(open);
  if ((quote === '') || (!quoteCharacters.includes(quote)))
    return missingValue(`The ${key} value is not a quoted string.`);
  /**
   Index of the matching closing quote.
   */
  const close = line.indexOf(
    quote,
    open + 1
  );
  if (close === (-1))
    return missingValue(`The ${key} string is not closed.`);
  /**
   The text between the quotes.
   */
  const value = line.slice(
    open + 1,
    close
  );
  if (value.includes('\\'))
    return missingValue(`The ${key} string holds a backslash escape, which a version never needs.`);
  return foundValue({
    value,
    valueStart: open + 1,
    valueEnd: close
  });
}

/**
 Check that nothing but a comment follows a value on its line.

 @param rest - text after the closing quote

 @returns true for blanks only or blanks then a `#` comment

 @example
 ```ts
 endsWithComment(' # published');
 ```
 */
export function endsWithComment(rest: string): boolean {
  /**
   Text after the blanks.
   */
  const tail = rest.slice(skipBlanks({
    line: rest,
    from: 0
  }));
  return (tail === '') || tail.startsWith('#');
}

/**
 Split a manifest into lines, each without its carriage return, remembering which lines had one.

 @param text - manifest text

 @returns the line bodies and a flag per line for a trailing carriage return

 @example
 ```ts
 splitManifest('a\r\nb');
 ```
 */
export function splitManifest(text: string): SplitManifest {
  /**
   Lines as split at line feeds.
   */
  const rawLines = text.split('\n');
  return {
    bodies: rawLines.map(function stripReturn(line) {
      return line.endsWith(carriageReturn) ? line.slice(
        0,
        -1
      ) : line;
    }),
    hadReturn: rawLines.map(function hasReturn(line) {
      return line.endsWith(carriageReturn);
    }),
  };
}

/**
 Join line bodies back into a manifest with the carriage returns they had.

 @param bodies - line bodies

 @param hadReturn - flag per line for a trailing carriage return

 @returns the manifest text

 @example
 ```ts
 joinManifest({ bodies: ['a'], hadReturn: [false] });
 ```
 */
export function joinManifest({
  bodies,
  hadReturn,
}: SplitManifest): string {
  return bodies.map(function restoreReturn(
    body,
    index
  ) {
    return hadReturn[index] === true ? `${body}${carriageReturn}` : body;
  })
    .join('\n');
}

/**
 Replace the value of one located quoted string in a line body.

 @param body - line body

 @param location - place of the value

 @param replacement - new value text

 @returns the line body with the value replaced and everything else kept

 @example
 ```ts
 spliceValue({ body: 'v = "1"', location: { value: '1', valueStart: 5, valueEnd: 6 }, replacement: '2' });
 ```
 */
export function spliceValue({
  body,
  location,
  replacement,
}: {
  readonly body: string;
  readonly location: ValueLocation;
  readonly replacement: string;
}): string {
  return `${body.slice(
    0,
    location.valueStart
  )}${replacement}${body.slice(location.valueEnd)}`;
}
