import {
  at,
} from './range.ts';
import {
  effectiveRecordIndexOf,
  type ExpectationToken,
  type IndentNodeRecord,
} from './indent-tree.ts';

//region Line lookup
// Offset to line index mapping shared by every route resolver.

/**
 Parameters for {@link lineIndexOf}.
 */
export type LineIndexOfParams = {
  readonly lineStarts: readonly number[];
  readonly offset: number;
};

/**
 Binary-searches the line holding `offset`.
 
 Routes resolve anchors and bracket bases by line index, so every offset
 lookup funnels through here instead of rescanning the text.
 
 @param params - ascending line-start offsets and target offset
 
 @returns index of the last line whose start is at or before `offset`
 
 @example
 ```ts
 lineIndexOf({ lineStarts: [0, 5], offset: 6 }); // 1
 ```
 */
export function lineIndexOf({
  lineStarts,
  offset,
}: LineIndexOfParams,): number {
  let low = 0;
  let high = lineStarts.length - 1;
  while (low < high) {
    const middle = Math.ceil((low + high) / 2);
    if (at({
      arr: lineStarts,
      index: middle,
    },) <= offset)
      low = middle;
    else
      high = middle - 1;
  }
  return low;
}
//endregion Line lookup

//region Token roles
// Bracket and suffix-keyword classification for line-leading tokens.

/**
 Punctuator values opening a bracket frame whose closer returns to the
 frame's base line.
 */
const OPEN_BRACKET_VALUES = new Set([
  '(',
  '[',
  '{',
]);

/**
 Punctuator values closing a bracket frame opened earlier.
 */
const CLOSE_BRACKET_VALUES = new Set([
  ')',
  ']',
  '}',
]);

/**
 Decides whether a token opens a bracket frame.
 
 @param token - candidate token
 
 @returns whether the token is an open bracket punctuator
 
 @example
 ```ts
 isOpenBracketToken(openParenToken); // true
 ```
 */
export function isOpenBracketToken(token: ExpectationToken,): boolean {
  return (token.type === 'Punctuator')
    && OPEN_BRACKET_VALUES.has(token.value);
}

/**
 Decides whether a token closes a bracket frame.
 
 @param token - candidate token
 
 @returns whether the token is a close bracket punctuator
 
 @example
 ```ts
 isCloseBracketToken(closeBraceToken); // true
 ```
 */
export function isCloseBracketToken(token: ExpectationToken,): boolean {
  return (token.type === 'Punctuator')
    && CLOSE_BRACKET_VALUES.has(token.value);
}

/**
 Returns the matching close bracket for an open bracket value.
 
 @param openValue - open bracket punctuator text
 
 @returns matching close bracket text
 
 @throws when `openValue` is not an open bracket
 
 @example
 ```ts
 closeValueOfOpenBracket('{'); // '}'
 ```
 */
export function closeValueOfOpenBracket(openValue: string,): string {
  switch (openValue) {
    case '(': {
      return ')';
    }
    case '[': {
      return ']';
    }
    case '{': {
      return '}';
    }
    default: {
      throw new Error(`Not an open bracket: ${openValue}`,);
    }
  }
}

/**
 Parameters for {@link isSuffixKeywordToken}.
 */
export type IsSuffixKeywordTokenParams = {
  readonly token: ExpectationToken;
  readonly ownerType: string;
};

/**
 Decides whether a line-leading token is a suffix keyword of its statement:
 `else`, `catch`, `finally`, or do-while `while`.
 
 The owning node kind disambiguates keywords from same-text identifiers, so
 property names and member accesses never take the suffix route.
 
 @param params - candidate token and owning node kind
 
 @returns whether the token aligns with its statement line instead of indenting
 
 @example
 ```ts
 isSuffixKeywordToken({ token, ownerType: 'IfStatement' }); // `else` on its own line
 ```
 */
export function isSuffixKeywordToken({
  token,
  ownerType,
}: IsSuffixKeywordTokenParams,): boolean {
  if (token.type !== 'Keyword')
    return false;
  return ((token.value === 'else') && (ownerType === 'IfStatement'))
    || ((token.value === 'catch') && (ownerType === 'CatchClause'))
    || ((token.value === 'finally') && (ownerType === 'TryStatement'))
    || ((token.value === 'while') && (ownerType === 'DoWhileStatement'));
}
//endregion Token roles

//region Route resolution
// The two indent-bearing routes: content anchors and bracket bases. Both
// return the earlier line whose leading width the checked line builds on.

/**
 ESTree body kinds whose braces and content anchor at the owning construct's
 line when that construct opens on an earlier line: function bodies, bare
 blocks, and namespace bodies. Nested bodies owned by another body never
 remap, so bare blocks inside bare blocks keep their own base.
 */
const BODY_KINDS = new Set([
  'BlockStatement',
  'TSModuleBlock',
]);

/**
 Parameters for {@link anchorLineIndexOf}.
 */
export type AnchorLineIndexOfParams = {
  readonly records: readonly IndentNodeRecord[];
  readonly recordIndex: number;
  readonly lineIndex: number;
  readonly lineStarts: readonly number[];
};

/**
 Finds the anchor line for a line-leading token: the nearest ancestor record
 starting on an earlier line.
 
 A body-kind ancestor remaps to its construct's start line when the construct
 opens earlier and is not itself a body, so hanging signatures and conditions
 anchor at their statement. `Program` means source-rooted content, anchored
 at no line.
 
 @param params - records, collapsed owner record index, and checked line
 
 @returns anchor line index, or `-1` for source-rooted content
 
 @example
 ```ts
 anchorLineIndexOf({ records, recordIndex: owner, lineIndex: 3, lineStarts }); // 1
 ```
 */
export function anchorLineIndexOf({
  records,
  recordIndex,
  lineIndex,
  lineStarts,
}: AnchorLineIndexOfParams,): number {
  let current = effectiveRecordIndexOf({
    records,
    recordIndex,
  },);
  while (current >= 0) {
    /**
     Candidate ancestor record; its start line decides anchor or step-up.
     */
    const record = at({
      arr: records,
      index: current,
    },);
    if (record.node
      .type
      === 'Program')
      return -1;
    /**
     Line where the candidate ancestor starts.
     */
    const recordLine = lineIndexOf({
      lineStarts,
      offset: record.node
        .start,
    },);
    if (recordLine >= lineIndex) {
      current = effectiveRecordIndexOf({
        records,
        recordIndex: record.parentIndex,
      },);
      continue;
    }
    if (!BODY_KINDS.has(record.node
      .type))
      return recordLine;
    /**
     Construct record owning the body; remap target when it opens earlier.
     */
    const constructRecord = (record.parentIndex >= 0)
      ? at({
        arr: records,
        index: record.parentIndex,
      },)
      : undefined;
    if (
      (constructRecord !== undefined)
        && (!BODY_KINDS.has(constructRecord.node
          .type))
        && (lineIndexOf({
          lineStarts,
          offset: constructRecord.node
            .start,
        },) < recordLine)
    ) {
      return lineIndexOf({
        lineStarts,
        offset: constructRecord.node
          .start,
      },);
    }
    return recordLine;
  }
  return -1;
}

/**
 Parameters for {@link bracketBaseLineIndexOf}.
 */
export type BracketBaseLineIndexOfParams = {
  readonly sourceText: string;
  readonly records: readonly IndentNodeRecord[];
  readonly ownerRecordIndex: number;
  readonly openValue: string;
  readonly openOffset: number;
  readonly openLine: number;
  readonly lineStarts: readonly number[];
};

/**
 Finds the base line a bracket's closer returns to.
 
 Open brackets default to their own line. Open braces owned by a body whose
 construct opens earlier remap to the construct's line, and braces owned
 directly by a closing construct (class, interface, enum, switch) use the
 construct's line, so closers never outdent past their statement.
 
 @param params - records, collapsed owner of the open token, and its position
 
 @returns base line index for the matching closer
 
 @example
 ```ts
 bracketBaseLineIndexOf({ ...params, openValue: '{', openLine: 4 }); // 3
 ```
 */
export function bracketBaseLineIndexOf({
  sourceText,
  records,
  ownerRecordIndex,
  openValue,
  openOffset,
  openLine,
  lineStarts,
}: BracketBaseLineIndexOfParams,): number {
  if ((openValue !== '{') || (ownerRecordIndex < 0))
    return openLine;
  /**
   Collapsed owner record of the open brace; its span versus the brace
   position selects the base route.
   */
  const ownerRecord = at({
    arr: records,
    index: ownerRecordIndex,
  },);
  /**
   Owner node span compared against the brace offset.
   */
  const owner = ownerRecord.node;
  if (owner.start === openOffset) {
    if (!BODY_KINDS.has(owner.type))
      return openLine;
    /**
     Construct record owning the body; remap target when it opens earlier.
     */
    const constructRecord = (ownerRecord.parentIndex >= 0)
      ? at({
        arr: records,
        index: ownerRecord.parentIndex,
      },)
      : undefined;
    if (
      (constructRecord !== undefined)
        && (!BODY_KINDS.has(constructRecord.node
          .type))
        && (lineIndexOf({
          lineStarts,
          offset: constructRecord.node
            .start,
        },) < openLine)
    ) {
      return lineIndexOf({
        lineStarts,
        offset: constructRecord.node
          .start,
      },);
    }
    return openLine;
  }
  if (
    (owner.start < openOffset)
      && (sourceText.charCodeAt(owner.end - 1,) === 125)
  ) {
    return lineIndexOf({
      lineStarts,
      offset: owner.start,
    },);
  }
  return openLine;
}
//endregion Route resolution
