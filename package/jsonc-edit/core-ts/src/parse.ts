import type { JsoncComment, } from './comment.ts';
import { JsoncParseError, } from './errors.ts';
import {
  closeArray,
  closeRecord,
} from './parse-close.ts';
import {
  parseScalar,
  type ValueScan,
} from './parse-scalar.ts';
import {
  appendComments,
  captureTrailing,
  prependComments,
  skipTrivia,
} from './parse-trivia.ts';
import { scanString, } from './scan.ts';
import type {
  JsoncKey,
  JsoncValue,
} from './value.ts';

//region Constants

/**
 Maximum container nesting depth. Structural recursion is bounded so a
 deeply-nested attacker-controlled document cannot overflow the call stack;
 beyond this the parser throws rather than crashing.
 */
const MAX_DEPTH = 512;

//endregion Constants

//region Parser

/**
 Parses one JSONC value starting at `index`, recursing into containers and
 delegating scalars. Comments strictly inside the value are attached here; the
 leading and trailing comments around it are attached by the caller.
 
 @param source - Full JSONC source.
 
 @param index - Offset of the value's first character.
 
 @param depth - Current nesting depth, guarded against overflow.
 
 @returns Parsed node and end offset.
 
 @throws JsoncParseError on malformed input or excessive nesting.
 
 @example
 ```ts
 parseValue({ source: 'true', index: 0, depth: 0 });
 // => { node: { kind: 'boolean', value: true }, end: 4 }
 ```
 */
export function parseValue({
  source,
  index,
  depth,
}: {
  readonly source: string;
  readonly index: number;
  readonly depth: number;
},): ValueScan {
  if (depth > MAX_DEPTH)
    throw new JsoncParseError({
      message: 'nesting too deep',
      offset: index,
    },);

  /**
   First character of the value, selecting container versus scalar parsing.
   */
  const char = source[index];
  if (char === '{')
    return parseRecord({
      source,
      index,
      depth,
    },);
  if (char === '[')
    return parseArray({
      source,
      index,
      depth,
    },);
  return parseScalar({
    source,
    index,
  },);
}

/**
 Parses an array body starting at the `[`. Elements carry their leading and
 trailing comments; a trailing comma before the close is tolerated.
 
 @param source - Full JSONC source.
 
 @param index - Offset of the opening bracket.
 
 @param depth - Current nesting depth.
 
 @returns Parsed array node and end offset.
 
 @throws JsoncParseError when the array is malformed or unterminated.
 
 @example
 ```ts
 parseArray({ source: '[1, 2,]', index: 0, depth: 0 });
 // => { node: { kind: 'array', elements: [...] }, end: 7 }
 ```
 */
function parseArray({
  source,
  index,
  depth,
}: {
  readonly source: string;
  readonly index: number;
  readonly depth: number;
},): ValueScan {
  /**
   Accumulated element nodes, built in place during the scan.
   */
  const elements: JsoncValue[] = [];
  for (let cursor = index + 1; ;) {
    /**
     Leading comments and next significant offset.
     */
    const lead = skipTrivia({
      source,
      index: cursor,
    },);
    cursor = lead.end;
    if (cursor >= source.length)
      throw new JsoncParseError({
        message: 'unterminated array',
        offset: index,
      },);
    if (source[cursor] === ']')
      return {
        node: closeArray({
          elements,
          dangling: lead.comments,
        },),
        end: cursor + 1,
      };
    /**
     Element value scanned at the cursor.
     */
    const valueScan = parseValue({
      source,
      index: cursor,
      depth: depth + 1,
    },);
    cursor = valueScan.end;
    /**
     Trailing same-line comments and comma after the element.
     */
    const trailing = captureTrailing({
      source,
      index: cursor,
    },);
    cursor = trailing.end;
    elements.push(
      appendComments({
        node: prependComments({
          node: valueScan.node,
          comments: lead.comments,
        },),
        comments: trailing.comments,
      },),
    );
    if (!trailing.commaSeen) {
      /**
       Lookahead past trivia: a comma may follow the element on a later line, and
       only a comma or the close may appear here.
       */
      const peek = skipTrivia({
        source,
        index: cursor,
      },);
      if (source[peek.end] === ',') {
        /**
         Element the trailing comments describe, if the array is not empty.
         */
        const last = elements.at(-1,);
        /**
         Comments that appeared before the later-line separator.
         */
        const beforeSeparator = peek.comments;
        // Comments before a later-line comma belong to the element that precedes
        // them, not to whatever follows the comma.
        if ((last !== undefined) && (beforeSeparator.length > 0))
          elements[elements.length - 1] = appendComments({
            node: last,
            comments: beforeSeparator,
          },);
        cursor = peek.end + 1;
        continue;
      }
      if (source[peek.end] !== ']')
        throw new JsoncParseError({
          message: 'expected , or ] in array',
          offset: peek.end,
        },);
    }
  }
}

/**
 Parses an object body starting at the `{`. Each key carries the comment that
 precedes it; each value carries the comment between colon and value plus its
 trailing comment. Duplicate keys are preserved as separate entries, and a
 trailing comma before the close is tolerated.
 
 @param source - Full JSONC source.
 
 @param index - Offset of the opening brace.
 
 @param depth - Current nesting depth.
 
 @returns Parsed record node and end offset.
 
 @throws JsoncParseError when the object is malformed or unterminated.
 
 @example
 ```ts
 parseRecord({ source: '{"a":1,}', index: 0, depth: 0 });
 // => { node: { kind: 'record', entries: [...] }, end: 8 }
 ```
 */
function parseRecord({
  source,
  index,
  depth,
}: {
  readonly source: string;
  readonly index: number;
  readonly depth: number;
},): ValueScan {
  /**
   Accumulated key-to-value entries.
   */
  const entries: {
    key: JsoncKey;
    value: JsoncValue
  }[] = [];
  for (let cursor = index + 1; ;) {
    /**
     Leading comments before the key, and next significant offset.
     */
    const lead = skipTrivia({
      source,
      index: cursor,
    },);
    cursor = lead.end;
    if (cursor >= source.length)
      throw new JsoncParseError({
        message: 'unterminated object',
        offset: index,
      },);
    if (source[cursor] === '}')
      return {
        node: closeRecord({
          entries,
          dangling: lead.comments,
        },),
        end: cursor + 1,
      };
    if (source[cursor] !== '"')
      throw new JsoncParseError({
        message: 'expected string key or } in object',
        offset: cursor,
      },);
    /**
     Parsed entry, with cursor advanced past its value and trailing comma.
     */
    const entry = parseEntry({
      source,
      index: cursor,
      depth,
      leadComments: lead.comments,
    },);
    cursor = entry.end;
    entries.push({
      key: entry.key,
      value: entry.value,
    },);
    if (!entry.commaSeen) {
      /**
       Lookahead past trivia: a comma may follow the member on a later line, and
       only a comma or the close may appear here.
       */
      const peek = skipTrivia({
        source,
        index: cursor,
      },);
      if (source[peek.end] === ',') {
        /**
         Member the trailing comments describe, if the record is not empty.
         */
        const last = entries.at(-1,);
        /**
         Comments that appeared before the later-line separator.
         */
        const beforeSeparator = peek.comments;
        // Comments before a later-line comma belong to the value that precedes
        // them, not to the key that follows the comma.
        if ((last !== undefined) && (beforeSeparator.length > 0))
          entries[entries.length - 1] = {
            key: last.key,
            value: appendComments({
              node: last.value,
              comments: beforeSeparator,
            },),
          };
        cursor = peek.end + 1;
        continue;
      }
      if (source[peek.end] !== '}')
        throw new JsoncParseError({
          message: 'expected , or } in object',
          offset: peek.end,
        },);
    }
  }
}

/**
 Parses one `"key": value` entry, attaching the comment before the key, any
 comment between key and colon, the comment between colon and value, and the
 value's trailing comment.
 
 @param source - Full JSONC source.
 
 @param index - Offset of the key's opening quote.
 
 @param depth - Current nesting depth.
 
 @param leadComments - Comments that preceded the key.
 
 @returns Key, value, whether a comma followed, and the end offset.
 
 @throws JsoncParseError when the colon is missing.
 
 @example
 ```ts
 parseEntry({ source: '"a": 1', index: 0, depth: 0, leadComments: [] });
 // => { key: {...}, value: {...}, commaSeen: false, end: 6 }
 ```
 */
function parseEntry({
  source,
  index,
  depth,
  leadComments,
}: {
  readonly source: string;
  readonly index: number;
  readonly depth: number;
  readonly leadComments: readonly JsoncComment[];
},): {
  readonly key: JsoncKey;
  readonly value: JsoncValue;
  readonly commaSeen: boolean;
  readonly end: number;
} {
  /**
   Scanned key string token.
   */
  const keyScan = scanString({
    source,
    index,
  },);
  /**
   Comments sitting between the key and its colon.
   */
  const beforeColon = skipTrivia({
    source,
    index: keyScan.end,
  },);
  if (source[beforeColon.end] !== ':')
    throw new JsoncParseError({
      message: 'expected : after object key',
      offset: beforeColon.end,
    },);
  /**
   Comments sitting between the colon and the value.
   */
  const beforeValue = skipTrivia({
    source,
    index: beforeColon.end + 1,
  },);
  /**
   Value scanned after the colon.
   */
  const valueScan = parseValue({
    source,
    index: beforeValue.end,
    depth: depth + 1,
  },);
  /**
   Trailing same-line comments and comma after the value.
   */
  const trailing = captureTrailing({
    source,
    index: valueScan.end,
  },);
  /**
   Bare key node before its leading and inter-colon comments attach.
   */
  const baseKey: JsoncKey = {
    value: keyScan.value,
    raw: keyScan.raw,
  };
  return {
    key: appendComments({
      node: prependComments({
        node: baseKey,
        comments: leadComments,
      },),
      comments: beforeColon.comments,
    },),
    value: appendComments({
      node: prependComments({
        node: valueScan.node,
        comments: beforeValue.comments,
      },),
      comments: trailing.comments,
    },),
    commaSeen: trailing.commaSeen,
    end: trailing.end,
  };
}

//endregion Parser
