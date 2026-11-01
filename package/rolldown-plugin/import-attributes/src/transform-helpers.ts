/**
 Span-based string replacement helpers for AST-driven import attribute transforms.
 
 @module
 */

import type { ESTree, } from 'rolldown/utils';

import { ATTR_QUERY_KEY, } from './patterns.ts';

//region Types

/**
 Source string replacement with start/end byte offsets.
 
 @example
 ```ts
 const r: Replacement = { start: 10, end: 20, text: "'./file.sql?attr=text'" };
 ```
 */
export type Replacement = {
  readonly start: number;
  readonly end: number;
  readonly text: string;
};

//endregion Types

//region Clause scanning

/**
 Returns the cursor past any space/tab/newline/carriage-return run at
 `idx`. Used to skip the whitespace gap between an import path and the
 `with`/`assert` keyword.
 
 Single linear pass with no recursion: the cursor walks right past every
 with-clause whitespace char, so a long run is O(n) time and O(1) stack.
 The prior recursive scan was O(n) stack and overflowed on long input
 under engines without tail-call elimination (V8/Node).
 
 @param s - source string
 
 @param idx - cursor to begin scanning from
 
 @returns first non-whitespace position at or after `idx`
 
 @example
 ```ts
 skipWithClauseWhitespace({ s: '   with', idx: 0 }); // 3
 skipWithClauseWhitespace({ s: 'with', idx: 0 }); // 0
 ```
 */
export function skipWithClauseWhitespace({
  s,
  idx,
}: {
  readonly s: string;
  readonly idx: number;
},): number {
  return (function scan(): number {
    /**
     Cursor; walked right past every with-clause whitespace char so the position is found in one pass.
     */
    let cursor = idx;
    while (cursor < s
      .length) {
      /**
       Char at the cursor; only the four whitespace chars permitted in the with-clause prefix advance the scan.
       */
      const c = s.charAt(cursor,);
      if ((c !== ' ') && (c !== '\t')
        && (c !== '\n')
        && (c !== '\r'))
        return cursor;
      cursor += 1;
    }
    return cursor;
  })();
}

/**
 Finds the start of the `with`/`assert` clause after a source literal
 by scanning whitespace with {@link skipWithClauseWhitespace} then checking
 for the keyword.
 
 @param code - full source code
 
 @param fromPos - position to start scanning (typically `source.end`)
 
 @returns position of the clause start (including leading whitespace), or -1
 */
function findWithClauseStart({
  code,
  fromPos,
}: {
  readonly code: string;
  readonly fromPos: number;
},): number {
  /**
   Position past the whitespace gap, where the `with`/`assert` keyword would begin.
   */
  const keywordStart = skipWithClauseWhitespace({
    s: code,
    idx: fromPos,
  },);
  if (
    (!code.startsWith(
      'with',
      keywordStart,
    ))
      && (!code.startsWith(
        'assert',
        keywordStart,
      ))
  ) {
    return -1;
  }
  return fromPos;
}

/**
 Finds the closing brace `}` of the `with`/`assert` clause after the
 last import attribute entry.
 
 @param code - full source code
 
 @param afterLastAttr - position after the last attribute node's span
 
 @returns position after the closing `}` (or `code.length + 1` if not found)
 */
function findWithClauseEnd({
  code,
  afterLastAttr,
}: {
  readonly code: string;
  readonly afterLastAttr: number;
},): number {
  /**
   Offset of the brace that ends the attributes object; falls through when absent.
   */
  const closingBrace = code.indexOf(
    '}',
    afterLastAttr,
  );
  return (closingBrace === (-1) ? code.length : closingBrace) + 1;
}

//endregion Clause scanning

/**
 Collects replacements for a static import/export declaration with
 attributes, locating the clause to elide via {@link findWithClauseStart}
 and {@link findWithClauseEnd}.
 
 @param source - source literal AST node
 
 @param attributes - import attribute AST nodes
 
 @param attrType - resolved attribute type string
 
 @param code - full source code
 
 @returns span replacements for the specifier and any elided with-clause
 
 @example
 ```ts
 const replacements = collectStaticReplacements({ source: node.source, attributes: node.attributes, attrType: 'text', code, },);
 ```
 */
export function collectStaticReplacements({
  source,
  attributes,
  attrType,
  code,
}: {
  readonly source: ESTree.StringLiteral;
  readonly attributes: readonly ESTree.ImportAttribute[];
  readonly attrType: string;
  readonly code: string;
},): readonly Replacement[] {
  /**
   Quote character preserved so the rewritten specifier matches the source's quoting style.
   */
  const quote = code[source.start];
  /**
   Rewrites the specifier to carry the attribute query; always emitted.
   */
  const specifierReplacement: Replacement = {
    start: source.start,
    end: source.end,
    text: `${quote}${source.value}?${ATTR_QUERY_KEY}=${attrType}${quote}`,
  };

  /**
   Start of the `with`/`assert` clause to elide; -1 if no clause is present.
   */
  const withStart = findWithClauseStart({
    code,
    fromPos: source.end,
  },);
  if (withStart === (-1))
    return [specifierReplacement,];
  /**
   Last attribute entry, used to anchor the end-of-clause scan.
   */
  const lastAttr = attributes.at(-1,);
  if (lastAttr === undefined)
    return [specifierReplacement,];
  /**
   End offset that completes the elision span past the closing brace.
   */
  const withEnd = findWithClauseEnd({
    code,
    afterLastAttr: lastAttr.end,
  },);
  return [
    specifierReplacement,
    {
      start: withStart,
      end: withEnd,
      text: '',
    },
  ];
}
