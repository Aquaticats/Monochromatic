import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  Context,
  Node,
} from '@oxlint/plugins';

import { baseIndentAt, } from './indent.ts';

//region Constants

/**
 Opening brace token text.
 */
const OPEN_BRACE = '{';

/**
 Closing brace token text.
 */
const CLOSE_BRACE = '}';

//endregion Constants

//region Types

/**
 Source unit whose range participates in body-boundary checks.
 */
export type SyntaxUnit = {
  /**
   Token or comment kind.
   */
  readonly type: string;
  /**
   Token or comment source text.
   */
  readonly value: string;
  /**
   Start-inclusive and end-exclusive source range.
   */
  readonly range: readonly [
    number,
    number,
  ];
};

/**
 Opening and closing brace tokens enclosing a body.
 */
export type BodyBraces = {
  /**
   Opening `{` token.
   */
  readonly openBrace: SyntaxUnit;
  /**
   Closing `}` token.
   */
  readonly closeBrace: SyntaxUnit;
};

/**
 Parameters for {@link bracedContent}.
 */
type BracedContentParams = BodyBraces & {
  /**
   Rule context whose `tokensAndComments` list supplies body content.
   */
  readonly context: Context;
};

/**
 Parameters for {@link firstBraceInNode}.
 */
type FirstBraceInNodeParams = {
  /**
   Rule context used for token lookup.
   */
  readonly context: Context;
  /**
   Braced AST node whose first opening brace is wanted.
   */
  readonly node: Node;
};

/**
 Parameters for {@link firstBraceAfterNode}.
 */
type FirstBraceAfterNodeParams = {
  /**
   Rule context used for token lookup.
   */
  readonly context: Context;
  /**
   AST node preceding the wanted opening brace.
   */
  readonly node: Node;
};

/**
 Parameters for {@link lastBraceInNode}.
 */
type LastBraceInNodeParams = {
  /**
   Rule context used for token lookup.
   */
  readonly context: Context;
  /**
   Braced AST node whose closing brace is wanted.
   */
  readonly node: Node;
};

/**
 Parameters for {@link lineOfOffset}.
 */
type LineOfOffsetParams = {
  /**
   Rule context used for offset-to-location conversion.
   */
  readonly context: Context;
  /**
   Source offset whose line is wanted.
   */
  readonly offset: number;
};

/**
 Parameters for {@link lineOfUnitEnd}.
 */
type LineOfUnitEndParams = {
  /**
   Rule context used for offset-to-location conversion.
   */
  readonly context: Context;
  /**
   Token or comment whose final character line is wanted.
   */
  readonly unit: SyntaxUnit;
};

/**
 Parameters for {@link bodyBaseIndent}.
 */
type BodyBaseIndentParams = {
  /**
   Rule context whose token stream supplies same-line brace depth.
   */
  readonly context: Context;
  /**
   Full file source text.
   */
  readonly sourceText: string;
  /**
   Opening brace token for the body being fixed.
   */
  readonly openBrace: SyntaxUnit;
  /**
   One indentation level for body content.
   */
  readonly bodyIndent: string;
};

//endregion Types

//region Token helpers

/**
 Reports whether a source unit is an opening brace token.
 
 @param unit - token or comment from oxlint's source-code API
 
 @returns whether `unit` is `{`
 
 @example
 ```ts
 sourceCode.getFirstToken(node, { filter: isOpeningBrace });
 ```
 */
function isOpeningBrace(unit: SyntaxUnit,): boolean {
  return (unit.type === 'Punctuator') && (unit.value === OPEN_BRACE);
}

/**
 Reports whether a source unit is a closing brace token.
 
 @param unit - token or comment from oxlint's source-code API
 
 @returns whether `unit` is `}`
 
 @example
 ```ts
 sourceCode.getLastToken(node, { filter: isClosingBrace });
 ```
 */
function isClosingBrace(unit: SyntaxUnit,): boolean {
  return (unit.type === 'Punctuator') && (unit.value === CLOSE_BRACE);
}

/**
 Extracts a syntax unit's source range.
 
 @param unit - token or comment whose range is wanted
 
 @returns start-inclusive and end-exclusive range
 
 @example
 ```ts
 const [start, end] = rangeOfUnit(unit);
 ```
 */
export function rangeOfUnit(unit: SyntaxUnit,): readonly [
  number,
  number,
] {
  return unit.range;
}

/**
 Returns first opening brace token inside a node.
 
 @returns opening brace token
 
 @throws when node has no opening brace token
 
 @example
 ```ts
 const openBrace = firstBraceInNode({ context, node });
 ```
 */
export function firstBraceInNode({
  context,
  node,
}: ForeignBorrowed<Readonly<FirstBraceInNodeParams>>,): SyntaxUnit {
  /**
   First `{` token contained by the node.
   */
  const openBrace = context.sourceCode
    .getFirstToken(
    node,
    { filter: isOpeningBrace, },
    );
  if (openBrace === null)
    throw new Error('Expected braced node to contain an opening brace.',);

  return openBrace;
}

/**
 Returns first opening brace token after a node.
 
 Used for `SwitchStatement`, whose node range starts before its discriminant;
 the body brace is the first `{` after the discriminant, not necessarily the
 first `{` inside the whole switch node.
 
 @returns opening brace token after `node`
 
 @throws when no opening brace follows `node`
 
 @example
 ```ts
 const openBrace = firstBraceAfterNode({ context, node: switchNode.discriminant });
 ```
 */
export function firstBraceAfterNode({
  context,
  node,
}: ForeignBorrowed<Readonly<FirstBraceAfterNodeParams>>,): SyntaxUnit {
  /**
   First `{` token after the preceding node.
   */
  const openBrace = context.sourceCode
    .getTokenAfter(
    node,
    { filter: isOpeningBrace, },
    );
  if (openBrace === null)
    throw new Error('Expected preceding node to be followed by an opening brace.',);

  return openBrace;
}

/**
 Returns last closing brace token inside a node.
 
 @returns closing brace token
 
 @throws when node has no closing brace token
 
 @example
 ```ts
 const closeBrace = lastBraceInNode({ context, node });
 ```
 */
export function lastBraceInNode({
  context,
  node,
}: ForeignBorrowed<Readonly<LastBraceInNodeParams>>,): SyntaxUnit {
  /**
   Last `}` token contained by the node.
   */
  const closeBrace = context.sourceCode
    .getLastToken(
    node,
    { filter: isClosingBrace, },
    );
  if (closeBrace === null)
    throw new Error('Expected braced node to contain a closing brace.',);

  return closeBrace;
}

/**
 Returns token and comment units strictly inside body braces.
 
 Comments count as body content for this rule: a comment-only block is not
 empty and must place the comment after `{` and `}` after the comment. A truly
 empty block has no tokens or comments between braces and is ignored.
 
 @returns source units inside the braces, in source order
 
 @example
 ```ts
 const content = bracedContent({ context, openBrace, closeBrace });
 ```
 */
export function bracedContent({
  context,
  openBrace,
  closeBrace,
}: ForeignBorrowed<Readonly<BracedContentParams>>,): readonly SyntaxUnit[] {
  /**
   Offset immediately after the opening brace.
   */
  const [, openEnd,] = rangeOfUnit(openBrace,);
  /**
   Offset where the closing brace starts.
   */
  const [closeStart,] = rangeOfUnit(closeBrace,);

  /**
   Full token and comment stream for the current file.
   */
  const { tokensAndComments, } = context.sourceCode;

  return tokensAndComments.filter(function isInsideBody(
    unit: ForeignBorrowed<SyntaxUnit>,
  ): boolean {
    /**
     Range of the candidate source unit.
     */
    const [unitStart, unitEnd,] = rangeOfUnit(unit,);
    return (unitStart >= openEnd) && (unitEnd <= closeStart);
  },);
}

//endregion Token helpers

//region Line helpers

/**
 Returns the 1-indexed line containing an offset.
 
 @returns source line for `offset`
 
 @example
 ```ts
 const line = lineOfOffset({ context, offset: token.start });
 ```
 */
export function lineOfOffset({
  context,
  offset,
}: ForeignBorrowed<Readonly<LineOfOffsetParams>>,): number {
  return context.sourceCode
    .getLocFromIndex(offset,)
    .line;
}

/**
 Returns the 1-indexed line containing a token or comment's final character.
 
 The end offset points one character past the unit. Subtracting one checks the
 line where the unit actually ends, which matters for multi-line block
 comments before a closing brace.
 
 @returns source line for the unit's final character
 
 @example
 ```ts
 const line = lineOfUnitEnd({ context, unit: lastContent });
 ```
 */
export function lineOfUnitEnd({
  context,
  unit,
}: ForeignBorrowed<Readonly<LineOfUnitEndParams>>,): number {
  /**
   Unit range used to locate its final character.
   */
  const [unitStart, unitEnd,] = rangeOfUnit(unit,);
  /**
   Offset of the last character in this non-empty token or comment.
   */
  const finalOffset = Math.max(
    unitStart,
    unitEnd - 1,
  );
  return lineOfOffset({
    context,
    offset: finalOffset,
  },);
}

//endregion Line helpers

//region Indentation helpers

/**
 Counts unmatched braces before current body on same source line.
 
 @param tokensAndComments - Foreign source units scanned in order.
 
 @param lineStart - First offset on current source line.
 
 @param openStart - Current body opening-brace offset.
 
 @returns unmatched opening-brace depth before current body.
 */
function sameLineBraceDepth({
  tokensAndComments,
  lineStart,
  openStart,
}: {
  readonly tokensAndComments: readonly SyntaxUnit[];
  readonly lineStart: number;
  readonly openStart: number;
},): number {
  /**
   Running unmatched opening-brace depth.
   */
  let depth = 0;
  for (const unit of tokensAndComments) {
    /**
     Candidate unit start offset.
     */
    const [unitStart,] = rangeOfUnit(unit,);
    if ((unitStart < lineStart) || (unitStart >= openStart))
      continue;
    if (isOpeningBrace(unit,)) {
      depth++;
      continue;
    }
    if (isClosingBrace(unit,)) {
      depth = Math.max(
        0,
        depth - 1,
      );
    }
  }
  return depth;
}

/**
 Returns indentation for the closing brace of the current body.
 
 When several braced bodies are dense on one line, inner body braces begin on
 the parent's pre-fix line. Counting still-open same-line braces before the
 current `{` predicts the indentation level that parent boundary fixes will
 create, so `class C {static {x;}}` fixes to a four-space static-block body.
 
 @returns base indentation for current body
 
 @example
 ```ts
 const baseIndent = bodyBaseIndent({ context, sourceText, openBrace, bodyIndent: '  ' });
 ```
 */
export function bodyBaseIndent({
  context,
  sourceText,
  openBrace,
  bodyIndent,
}: ForeignBorrowed<Readonly<BodyBaseIndentParams>>,): string {
  /**
   Opening brace start offset for the body being checked.
   */
  const [openStart,] = rangeOfUnit(openBrace,);
  /**
   First offset on the line containing the current opening brace.
   */
  const lineStart = sourceText.lastIndexOf(
    '\n',
    openStart - 1,
  ) + 1;
  /**
   Source-code accessor for the current file.
   */
  const { sourceCode, } = context;
  /**
   Tokens and comments used to inspect same-line brace depth.
   */
  const { tokensAndComments, } = sourceCode;
  /**
   Syntactic brace depth already open on this source line before this body.
   */
  const sameLineDepth = sameLineBraceDepth({
    tokensAndComments,
    lineStart,
    openStart,
  },);

  return `${
    baseIndentAt({
      sourceText,
      offset: openStart,
    },)
  }${bodyIndent.repeat(sameLineDepth,)}`;
}

//endregion Indentation helpers
