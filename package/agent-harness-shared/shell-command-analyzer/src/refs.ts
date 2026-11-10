/**
 Raw reference and path helpers shared by shell command analysis consumers.
 
 @module
 */

import { isWordChar, } from '@monochromatic-dev/agent-harness-shared-text-scan/ts';

//region Identifier helpers

/**
 Whether character starts POSIX-like shell identifier.
 
 @param c - one-character string to inspect
 
 @returns whether character can start variable name
 
 @example
 ```ts
 isIdentifierStart('_');
 ```
 */
function isIdentifierStart(c: string,): boolean {
  return ((c >= 'A') && (c <= 'Z'))
    || ((c >= 'a') && (c <= 'z'))
    || (c === '_');
}

//endregion Identifier helpers

//region Param reference extraction

/**
 Sentinel returned when no identifier starts at requested offset.
 */
const IDENTIFIER_NOT_FOUND: unique symbol = Symbol('shell identifier not found at requested offset',);

/**
 Identifier read result.
 */
type IdentifierRead = {
  /**
   Identifier text.
   */
  readonly name: string;
  /**
   Offset immediately after identifier.
   */
  readonly next: number;
};

/**
 Find end offset for identifier that starts at `start`.
 
 @param source - source text to scan
 
 @param start - offset where identifier starts
 
 @returns first offset after identifier continuation run
 
 @example
 ```ts
 identifierEnd({ source: 'API_KEY}', start: 0 });
 ```
 */
function identifierEnd(
  {
    source,
    start,
  }: {
    readonly source: string;
    readonly start: number;
  },
): number {
  for (let cursor = start + 1;; cursor += 1) {
    if (!isWordChar(source.charAt(cursor,),))
      return cursor;
  }
}

/**
 Read identifier from `source` starting at `start`.
 
 @param source - source text to scan
 
 @param start - offset where identifier may start
 
 @returns identifier text plus next offset, or sentinel when no identifier starts there
 
 @example
 ```ts
 readIdentifier({ source: 'API_KEY}', start: 0 });
 ```
 */
function readIdentifier(
  {
    source,
    start,
  }: {
    readonly source: string;
    readonly start: number;
  },
): IdentifierRead | typeof IDENTIFIER_NOT_FOUND {
  if (!isIdentifierStart(source.charAt(start,),))
    return IDENTIFIER_NOT_FOUND;

  /**
   Cursor advanced across identifier continuation characters.
   */
  const cursor = identifierEnd({
    source,
    start,
  },);

  return {
    name: source.slice(
      start,
      cursor,
    ),
    next: cursor,
  };
}

/**
 Extract `$VAR` and `${VAR}` references with a linear raw-text scan.
 
 This intentionally runs before parsing so malformed commands still surface
 possible secret variable references to guardrails.
 
 @param command - raw shell command string
 
 @returns unique variable names found in source order
 
 @example
 ```ts
 extractParamRefs('curl $API_KEY ${TOKEN}');
 // ['API_KEY', 'TOKEN']
 ```
 */
function extractParamRefs(command: string,): string[] {
  /**
   Ordered set of references discovered during scan.
   */
  const refs = new Set<string>();

  for (let cursor = 0; cursor < command.length; cursor += 1) {
    if (command.charAt(cursor,) !== '$')
      continue;
    if (command.charAt(cursor + 1,) === '$') {
      cursor += 1;
      continue;
    }
    if (command.charAt(cursor + 1,) === '(')
      continue;
    if (command.charAt(cursor + 1,) === '{') {
      /**
       Identifier after `${`.
       */
      const braced = readIdentifier({
        source: command,
        start: cursor + '${'.length,
      },);
      if ((braced !== IDENTIFIER_NOT_FOUND) && (command.charAt(braced.next,) === '}')) {
        refs.add(braced.name,);
        cursor = braced.next;
      }
      continue;
    }
    /**
     Identifier after bare `$`.
     */
    const simple = readIdentifier({
      source: command,
      start: cursor + '$'.length,
    },);
    if (simple !== IDENTIFIER_NOT_FOUND) {
      refs.add(simple.name,);
      cursor = simple.next - 1;
    }
  }

  return [...refs,];
}

//endregion Param reference extraction

//region Path heuristics

/**
 Heuristic for path-like shell words.
 
 @param value - shell word value to inspect
 
 @returns whether value looks like filesystem path
 
 @example
 ```ts
 looksLikePath('./src/index.ts');
 ```
 */
function looksLikePath(value: string,): boolean {
  return value.startsWith('/',)
    || value.startsWith('./',)
    || value.startsWith('../',)
    || value.startsWith('~',)
    || value.includes('/',)
    || value.startsWith('.',);
}

//endregion Path heuristics

export {
  extractParamRefs,
  looksLikePath,
};
