import { JsoncParseError, } from './errors.ts';
import {
  matchKeyword,
  scanNumber,
  scanString,
} from './scan.ts';
import type { JsoncValue, } from './value.ts';

//region Constants

/**
 `true` literal keyword.
 */
const KEYWORD_TRUE = 'true';

/**
 `false` literal keyword.
 */
const KEYWORD_FALSE = 'false';

/**
 `null` literal keyword.
 */
const KEYWORD_NULL = 'null';

/**
 Digits that may begin a number (a leading `-` is handled separately).
 */
const DIGITS = new Set([
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

//endregion Constants

//region Value scan result

/**
 A parsed node paired with the offset just past it.
 */
export type ValueScan = {
  readonly node: JsoncValue;
  readonly end: number;
};

//endregion Value scan result

//region Scalar parser

/**
 Parses a non-container value (string, number, `true`, `false`, or `null`) at
 `index`. Throws when the character there opens no valid value.
 
 @param source - Full JSONC source.
 
 @param index - Offset of the value's first character.
 
 @returns Parsed scalar node and end offset.
 
 @throws JsoncParseError when no scalar value starts at the offset.
 
 @example
 ```ts
 parseScalar({ source: 'true', index: 0 });
 // => { node: { kind: 'boolean', value: true }, end: 4 }
 ```
 */
export function parseScalar({
  source,
  index,
}: {
  readonly source: string;
  readonly index: number;
},): ValueScan {
  /**
   First character of the value, selecting which scalar production to parse.
   */
  const char = source[index];
  if (char === '"') {
    /**
     Scanned string token.
     */
    const scan = scanString({
      source,
      index,
    },);
    return {
      node: {
        kind: 'string',
        value: scan.value,
        raw: scan.raw,
      },
      end: scan.end,
    };
  }
  if (matchKeyword({
    source,
    index,
    keyword: KEYWORD_TRUE,
  },))
    return {
      node: {
        kind: 'boolean',
        value: true,
      },
      end: index + KEYWORD_TRUE.length,
    };
  if (matchKeyword({
    source,
    index,
    keyword: KEYWORD_FALSE,
  },))
    return {
      node: {
        kind: 'boolean',
        value: false,
      },
      end: index + KEYWORD_FALSE.length,
    };
  if (matchKeyword({
    source,
    index,
    keyword: KEYWORD_NULL,
  },))
    return {
      node: { kind: 'null', },
      end: index + KEYWORD_NULL.length,
    };
  if ((char === '-') || ((char !== undefined) && DIGITS.has(char,))) {
    /**
     Scanned number token.
     */
    const scan = scanNumber({
      source,
      index,
    },);
    return {
      node: {
        kind: 'number',
        value: scan.value,
        raw: scan.raw,
      },
      end: scan.end,
    };
  }
  throw new JsoncParseError({
    message: `unexpected character ${JSON.stringify(char ?? '<eof>',)}`,
    offset: index,
  },);
}

//endregion Scalar parser
