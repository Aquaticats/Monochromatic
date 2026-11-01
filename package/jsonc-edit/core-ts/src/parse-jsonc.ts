import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { StringJsonc, } from './brand.ts';
import { JsoncParseError, } from './errors.ts';
import { parseValue, } from './parse.ts';
import {
  appendComments,
  prependComments,
  skipTrivia,
} from './parse-trivia.ts';
import type { JsoncValue, } from './value.ts';

/**
 Module logger for the parse entry point.
 */
const l = tagged({ tag: 'parse-jsonc', },);


//region Entry

/**
 Parses a JSONC document into a structured, comment-preserving value.
 
 Every document goes through the structured parser, so number spellings, comment
 ownership and the 512-container nesting limit behave identically for clean and
 commented input. Comments attach to keys and values, trailing commas are
 tolerated, and the document must be an object or array at the top level; a bare
 scalar is rejected.
 
 @param source - Branded JSONC source string.
 
 @returns Parsed value, with comments preserved.
 
 @throws JsoncParseError on malformed input or a non-container top level.
 
 @example
 ```ts
 parseJsonc({ source: '{ "a": 1 } // note' as StringJsonc });
 // => { kind: 'record', entries: [...], comment: { type: 'inline', text: ' note' } }
 ```
 */
export function parseJsonc({
  source,
}: {
  readonly source: StringJsonc;
},): JsoncValue {
  tagged({
    tag: parseJsonc.name,
    l,
  },)
    .trace(`parsing ${String(source.length,)} bytes of JSONC structurally`,);

  /**
   Leading document comments and the offset of the top-level value.
   */
  const lead = skipTrivia({
    source,
    index: 0,
  },);
  /**
   First significant character; only a container may open the document.
   */
  const first = source[lead.end];
  if ((first !== '[') && (first !== '{'))
    throw new JsoncParseError({
      message: 'a JSONC document must be an object or array at the top level',
      offset: lead.end,
    },);

  /**
   Parsed top-level value and the offset just past it.
   */
  const valueScan = parseValue({
    source,
    index: lead.end,
    depth: 0,
  },);
  /**
   Trailing document comments and the offset past all trailing trivia.
   */
  const trailing = skipTrivia({
    source,
    index: valueScan.end,
  },);
  if (trailing.end !== source.length)
    throw new JsoncParseError({
      message: 'unexpected trailing content after top-level value',
      offset: trailing.end,
    },);

  return appendComments({
    node: prependComments({
      node: valueScan.node,
      comments: lead.comments,
    },),
    comments: trailing.comments,
  },);
}

//endregion Entry
