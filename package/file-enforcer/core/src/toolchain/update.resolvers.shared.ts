/**
 Types and text helpers shared by the `toolchain:update` family resolvers
 (`update.resolvers.ts` re-exports the public part).

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import type { ToolArtifacts, } from './plan.artifacts.data.ts';

//region Fetching

/**
 Minimal HTTP response a resolver needs.
 */
export type FetchedResponse = {
  readonly ok: boolean;
  readonly status: number;
  readonly text: string;
  /**
   Raw bytes for binary payloads (the Android archive hash).
   */
  readonly bytes?: Uint8Array;
};

/**
 Injectable GET used by every resolver (bound to `GITHUB_TOKEN` at runtime).
 */
export type Fetcher = (url: string,) => Promise<FetchedResponse>;

/**
 One tool's re-resolved version and artifact rows.
 */
export type ToolResolution = {
  readonly version: string;
  readonly artifacts: ToolArtifacts;
};

/**
 Artifact rows under construction, keyed by platform id.
 */
export type MutableArtifacts = Record<string, {
  url: string;
  sha256: string;
}>;

//endregion Fetching

//region JSON

/**
 Parsed JSON object whose fields are still unchecked.
 */
export type JsonObject = Readonly<Record<string, unknown>>;

/**
 Whether `value` is a plain (non-array) JSON object.

 @param value - parsed JSON

 @returns whether the value is a non-array object

 @example
 ```ts
 isJsonObject({ a: 1, },); // true
 isJsonObject([],); // false
 ```
 */
export function isJsonObject(value: unknown,): value is JsonObject {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Keeps the plain-object elements of a parsed JSON array.

 @param value - parsed JSON that may be an array

 @returns object elements in order, empty when the value is not an array

 @example
 ```ts
 jsonObjectsOf({ value: [{ a: 1, }, 2, ], },); // [{ a: 1 }]
 ```
 */
export function jsonObjectsOf(
  { value, }: { readonly value: unknown; },
): readonly JsonObject[] {
  return Array.isArray(value,)
    ? value.filter(function isObjectItem(item: unknown,): item is JsonObject {
      return isJsonObject(item,);
    },)
    : [];
}

/**
 Parses one fetched JSON document.

 @param response - response whose text to parse

 @param label - source label that names the document in the error

 @returns parsed document

 @throws Error when the body is not JSON

 @example
 ```ts
 parseJson({ response: { ok: true, status: 200, text: '{"a":1}', }, label: 'demo', },); // { a: 1 }
 ```
 */
export function parseJson(
  {
    response,
    label,
  }: {
    readonly response: FetchedResponse;
    readonly label: string;
  },
): unknown {
  try {
    return JSON.parse(response.text,) as unknown;
  }
  catch (error) {
    throw new Error(
      `${label} returned unparseable JSON: ${caughtValueText(error,)}`,
      { cause: error, },
    );
  }
}

/**
 Reads one JSON field path out of a parsed document.

 @param doc - parsed document

 @param path - field names to follow from the document root

 @returns string value, or empty when the path is absent or not a string

 @example
 ```ts
 stringField({ doc: { info: { version: '1.0.0', }, }, path: ['info', 'version',], },); // '1.0.0'
 ```
 */
export function stringField(
  {
    doc,
    path,
  }: {
    readonly doc: unknown;
    readonly path: readonly string[];
  },
): string {
  /**
   Value at the end of the path.
   */
  const value = path.reduce(
    function step(
      cursor: unknown,
      key: string,
    ): unknown {
      return isJsonObject(cursor,)
        ? cursor[key]
        : undefined;
    },
    doc,
  );
  return (typeof value) === 'string'
    ? value
    : '';
}

//endregion JSON

//region Text

/**
 Removes one leading `prefix` from `text` when present.

 @param text - text to trim

 @param prefix - leading text to remove

 @returns text without the prefix, or unchanged when it does not start with it

 @example
 ```ts
 stripPrefix({ text: 'cargo:ripgrep', prefix: 'cargo:', },); // 'ripgrep'
 ```
 */
export function stripPrefix(
  {
    text,
    prefix,
  }: {
    readonly text: string;
    readonly prefix: string;
  },
): string {
  return text.startsWith(prefix,)
    ? text.slice(prefix.length,)
    : text;
}

/**
 Lowercase hexadecimal digits.
 */
const HEX_DIGITS = '0123456789abcdef';

/**
 Hex digits in a SHA-256 digest.
 */
const SHA256_HEX_LENGTH = 64;

/**
 Whether a character is a lowercase hexadecimal digit.

 @param char - one character

 @returns whether the character is in `0-9a-f`
 */
function isHexDigit(char: string,): boolean {
  return HEX_DIGITS.includes(char,);
}

/**
 Whether `text` is a lowercase hexadecimal SHA-256 digest.

 @param text - candidate digest

 @returns whether the text is exactly 64 lowercase hex digits

 @example
 ```ts
 isSha256Hex({ text: 'a'.repeat(64,), },); // true
 isSha256Hex({ text: 'not-a-digest', },); // false
 ```
 */
export function isSha256Hex(
  { text, }: { readonly text: string; },
): boolean {
  return (text.length === SHA256_HEX_LENGTH)
    && text.split('',)
      .every(isHexDigit,);
}

/**
 Whether `text` is a non-empty run of ASCII digits.

 @param text - candidate number text

 @returns whether every character is `0-9` and the text is not empty

 @example
 ```ts
 isDigits({ text: '2026', },); // true
 isDigits({ text: '', },); // false
 ```
 */
export function isDigits(
  { text, }: { readonly text: string; },
): boolean {
  return (text !== '')
    && text.split('',)
      .every(function isDigit(char,): boolean {
        return ('0123456789').includes(char,);
      },);
}

/**
 Reads the first whitespace-delimited word of a text.

 @param text - text whose first word to read

 @returns first word, or empty when the text is blank

 @example
 ```ts
 firstWord({ text: 'abc *./rustup-init\n', },); // 'abc'
 ```
 */
export function firstWord(
  { text, }: { readonly text: string; },
): string {
  /**
   Text without surrounding whitespace.
   */
  const trimmed = text.trim();
  /**
   Index of the first whitespace character, or -1 when the text is one word.
   */
  const end = trimmed.split('',)
    .findIndex(function isSpace(char,): boolean {
      return char.trim() === '';
    },);
  return end === (-1)
    ? trimmed
    : trimmed.slice(
      0,
      end,
    );
}

/**
 Reads the quoted value of a `key = "value"` line.

 @param line - one line of a TOML-like manifest

 @param key - key the line must start with

 @param quotes - quote characters that may open and close the value

 @returns value between the quotes, or empty when the line is not that assignment

 @example
 ```ts
 quotedAssignment({ line: 'date = "2026-10-26"', key: 'date', quotes: '"', },); // '2026-10-26'
 ```
 */
export function quotedAssignment(
  {
    line,
    key,
    quotes,
  }: {
    readonly line: string;
    readonly key: string;
    readonly quotes: string;
  },
): string {
  if (!line.startsWith(key,))
    return '';
  /**
   Text after the key, without leading blanks.
   */
  const afterKey = line.slice(key.length,)
    .trimStart();
  if (!afterKey.startsWith('=',))
    return '';
  /**
   Text after the equals sign, without leading blanks.
   */
  const afterEquals = afterKey.slice(1,)
    .trimStart();
  if ((!quotes.includes(afterEquals.charAt(0,),)) || (afterEquals === ''))
    return '';
  /**
   Index of the first quote character after the opening one.
   */
  const closeAt = afterEquals.split('',)
    .findIndex(function isCloser(
      char,
      at,
    ): boolean {
      return (at > 0) && quotes.includes(char,);
    },);
  return closeAt === (-1)
    ? ''
    : afterEquals.slice(
      1,
      closeAt,
    );
}

//endregion Text
