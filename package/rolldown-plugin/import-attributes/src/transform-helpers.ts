/**
 Span-based string replacement helpers for AST-driven import attribute transforms.
 
 @module
 */

import type { ESTree, } from 'rolldown/utils';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

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
 
 Only space, tab, newline and carriage return are skipped. JavaScript also admits
 vertical tab, form feed and the Unicode space separators, which this scan leaves
 alone on purpose: the four characters are what editors and formatters put between
 a specifier and its keyword, and the restriction keeps the pinned behaviour of the
 linear rewrite that replaced the earlier recursive scan.
 
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
 Returns the cursor past the comment starting at `idx`, or `idx` when no
 comment starts there. Used by the clause scanners so a `with` keyword or a
 `}` held inside a comment is not read as clause syntax.
 
 A block comment ends past its closing `*\/`; a line comment ends at the
 newline that terminates it. An unterminated comment runs to the end of the
 source, which is where the parser stops as well.
 
 @param code - source to scan
 
 @param idx - cursor to test
 
 @returns first position after the comment, or `idx` when none starts there
 */
function skipComment({
  code,
  idx,
}: {
  readonly code: string;
  readonly idx: number;
},): number {
  if (code.startsWith(
    '/*',
    idx,
  )) {
    /**
     Offset of the block comment's closing `*\/`; -1 means it never closes.
     */
    const blockEnd = code.indexOf(
      '*/',
      idx + 2,
    );
    return blockEnd === (-1) ? code.length : blockEnd + 2;
  }
  if (code.startsWith(
    '//',
    idx,
  )) {
    /**
     Offset of the newline that ends the line comment; -1 means it runs to the end of the source.
     */
    const lineEnd = code.indexOf(
      '\n',
      idx + 2,
    );
    return lineEnd === (-1) ? code.length : lineEnd;
  }
  return idx;
}

/**
 Returns the cursor past any run of with-clause whitespace and comments
 starting at `idx`, so a comment between a specifier and the keyword, or
 between the keyword and its brace, hides neither.
 
 One linear pass with no recursion: every iteration hands the cursor past a
 whole gap segment, so a long run is O(n) time and O(1) stack.
 
 @param code - source to scan
 
 @param idx - cursor to begin scanning from
 
 @returns first position at or after `idx` that starts neither whitespace nor a comment
 */
function skipClauseGap({
  code,
  idx,
}: {
  readonly code: string;
  readonly idx: number;
},): number {
  return (function scan(): number {
    /**
     Cursor advanced past one gap segment per iteration until none starts here.
     */
    let cursor = idx;
    for (;;) {
      /**
       Position past a whitespace run starting at the cursor.
       */
      const afterWhitespace = skipWithClauseWhitespace({
        s: code,
        idx: cursor,
      },);
      /**
       Position past a comment starting where the whitespace run stopped.
       */
      const afterComment = skipComment({
        code,
        idx: afterWhitespace,
      },);
      if (afterComment === afterWhitespace)
        return afterWhitespace;
      cursor = afterComment;
    }
  })();
}

/**
 Finds the start of the `with`/`assert` clause after a source literal
 by scanning whitespace and comments with {@link skipClauseGap} then checking
 for the keyword.
 
 @param code - full source code
 
 @param fromPos - position to start scanning (typically `source.end`)
 
 @returns position of the clause start (including leading whitespace and comments), or -1
 */
function findWithClauseStart({
  code,
  fromPos,
}: {
  readonly code: string;
  readonly fromPos: number;
},): number {
  /**
   Position past the gap, where the `with`/`assert` keyword would begin.
   */
  const keywordStart = skipClauseGap({
    code,
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
 last import attribute entry, ignoring any `}` that sits inside a comment.
 
 The scan starts past every attribute node, so it never meets the string literal
 an attribute value is written as; the only text between the last value and the
 closing brace is whitespace, a comma and comments.
 
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
  return (function scan(): number {
    /**
     Cursor walked right through the gap after the last attribute; a comment it
     enters is skipped whole, so the braces it holds are never read as the end.
     */
    let cursor = afterLastAttr;
    while (cursor < code
      .length) {
      /**
       Position past a comment starting at the cursor, or the cursor itself when none starts there.
       */
      const commentEnd = skipComment({
        code,
        idx: cursor,
      },);
      if (commentEnd !== cursor) {
        cursor = commentEnd;
        continue;
      }
      if (code.charAt(cursor,)
        === '}')
        return cursor + 1;
      cursor += 1;
    }
    return code.length + 1;
  })();
}

/**
 Returns the offset of the first comma between `fromPos` and `toPos` that sits
 outside a comment, or -1 when the range holds none.
 
 The dynamic-import options argument is separated from its specifier by a comma
 that a comment may precede, and a comment may hold a comma of its own, so the
 separator cannot be found by searching the gap for the next comma.
 
 @param code - full source code
 
 @param fromPos - position the range starts at (typically the specifier's end)
 
 @param toPos - position the range ends before (typically the options' start)
 
 @returns offset of the separating comma, or -1 when the range holds none
 
 @example
 ```ts
 findCommaOutsideComments({ code: "import('./a.sql', { with: {} })", fromPos: 18, toPos: 30 });
 // 17: the comma between the specifier and its options
 ```
 
 With a comment between the two, the separator is the comma after the comment,
 never one the comment itself holds.
 */
export function findCommaOutsideComments({
  code,
  fromPos,
  toPos,
}: {
  readonly code: string;
  readonly fromPos: number;
  readonly toPos: number;
},): number {
  return (function scan(): number {
    /**
     Cursor walked right through the range; a comment it enters is skipped whole,
     so the commas it holds are never read as the separator.
     */
    let cursor = fromPos;
    while (cursor < toPos) {
      /**
       Position past a comment starting at the cursor, or the cursor itself when none starts there.
       */
      const commentEnd = skipComment({
        code,
        idx: cursor,
      },);
      if (commentEnd !== cursor) {
        cursor = commentEnd;
        continue;
      }
      if (code.charAt(cursor,)
        === ',')
        return cursor;
      cursor += 1;
    }
    return -1;
  })();
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
}: ForeignBorrowed<Readonly<{
  source: ESTree.StringLiteral;
  attributes: readonly ESTree.ImportAttribute[];
  attrType: string;
  code: string;
}>>,): readonly Replacement[] {
  /**
   Text of the specifier literal spanning both quotes, escape sequences
   exactly as the source spelled them.
   */
  const rawLiteral = code.slice(
    source.start,
    source.end,
  );
  /**
   Rewrites the specifier to carry the attribute query; always emitted.
   
   The query is inserted before the closing quote of the source text rather
   than rebuilt from the parsed value, so every escape sequence keeps the
   meaning it had (`'.\\file.sql'` stays a form feed rather than becoming a
   literal `f`, and an escaped quote stays escaped).
   */
  const specifierReplacement: Replacement = {
    start: source.start,
    end: source.end,
    text: `${rawLiteral.slice(
      0,
      -1,
    )}?${ATTR_QUERY_KEY}=${attrType}${rawLiteral.slice(-1,)}`,
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
