/**
 TSDoc comment discovery and parsing utilities.
 
 Extracted from `tsdoc-utils.ts` to keep files under 100 countable lines.
 
 @module
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  Comment,
  Context,
  Span,
} from '@oxlint/plugins';
import type { ReadonlyDeep, } from 'type-fest';

import { splitDocComment, } from './tsdoc-blocks.ts';
import type { TsdocParseResult as ParsedTsdocResult, } from './tsdoc-doc-model.ts';
import { collectStructuralMessages, } from './tsdoc-structural-messages.ts';

export type { TsdocParseResult, } from './tsdoc-doc-model.ts';

/**
 Parsed comment facts cached independently from declaration ownership.
 */
export type ParsedCommentFacts = {
  /**
   Scanned document model shared by every rule visiting comment.
   */
  readonly docComment: ParsedTsdocResult['docComment'];
  /**
   Structural diagnostics shared by every rule visiting comment.
   */
  readonly messages: ParsedTsdocResult['messages'];
};

/**
 Maximum distinct comment bodies retained during one CLI process.
 */
const MAX_CACHED_COMMENT_TEXTS = 4_096;

/**
 Bounded content cache shared across rule contexts whose host comment wrappers are not identity-stable.
 */
const PARSED_COMMENT_FACTS_BY_TEXT = new Map<string, ParsedCommentFacts>();

/**
 Absence marker meaning "node has no TSDoc comment"; never a real comment
 or parse result.
 
 Shared by {@link findTsdocComment} and {@link parseTsdocForNode} because the
 parse-absence is derived directly from the find-absence (the parser only
 runs once a comment exists); both narrow with `=== NO_TSDOC`. Optionality
 discipline bans `T | undefined` returns, so these functions return
 `T | typeof NO_TSDOC` instead.
 
 @example
 ```ts
 const found = findTsdocComment({ node, context, });
 if (found === NO_TSDOC)
   return;
 ```
 */
export const NO_TSDOC: unique symbol = Symbol('node has no TSDoc comment',);

/**
 Checks whether a block comment is a TSDoc comment (starts with `*`).
 
 @param comment - AST comment node
 
 @returns true for `/** ... *\/` style comments
 
 @example
 ```ts
 isTsdocBlock(commentNode);
 ```
 */
export function isTsdocBlock(comment: ReadonlyDeep<Comment>,): boolean {
  return (comment.type === 'Block')
    && comment.value
      .startsWith('*',);
}

/**
 Node types where `getCommentsBefore` may fail to find TSDoc because
 the comment is attached to a parent scope (export declaration).
 
 FunctionExpression and ArrowFunctionExpression are intentionally
 excluded because their comments belong to the enclosing
 VariableDeclaration or MethodDefinition which owns the TSDoc.
 */
export const FALLBACK_ELIGIBLE_TYPES: ReadonlySet<string> = new Set([
  'FunctionDeclaration',
  'VariableDeclaration',
  'ClassDeclaration',
  'MethodDefinition',
  'TSAbstractMethodDefinition',
  'PropertyDefinition',
  'TSEnumDeclaration',
  'TSEnumMember',
  'TSTypeAliasDeclaration',
  'TSInterfaceDeclaration',
],);

/**
 Parameters for {@link findTsdocComment} and {@link parseTsdocForNode}.
 */
export type TsdocLookupParams = {
  /**
   AST node to find TSDoc for.
   */
  readonly node: Span;
  /**
   Oxlint rule context providing sourceCode.
   */
  readonly context: Context;
};

/**
 Finds the TSDoc block comment associated with a node.
 
 Uses `getCommentsBefore` first (cheapest lookup), then falls back to
 scanning `getAllComments()` for the closest preceding TSDoc comment
 whose end position is on the line immediately before the node start.
 This fallback handles `export` declarations where `getCommentsBefore`
 returns nothing because the comment is attached to a parent
 ExportNamedDeclaration scope boundary.
 
 The fallback only applies to declaration-level node types listed in
 {@link FALLBACK_ELIGIBLE_TYPES}, not to FunctionExpression or
 ArrowFunctionExpression, because their TSDoc is owned by the enclosing
 VariableDeclaration or MethodDefinition.
 
 @returns block comment starting with `*`, or {@link NO_TSDOC} when none precedes the node
 
 @example
 ```ts
 const comment = findTsdocComment({ node, context });
 ```
 */
export function findTsdocComment({
  node,
  context,
}: ForeignBorrowed<TsdocLookupParams>,): Comment | typeof NO_TSDOC {
  // Fast path: getCommentsBefore works for most declarations
  /**
   Leading comments returned by the standard API; scanned back-to-front for nearest TSDoc.
   */
  const comments = context.sourceCode
    .getCommentsBefore(node,);
  for (let loopIndex = comments.length
    - 1; loopIndex >= 0; loopIndex--) {
    /**
     Single comment candidate at index `i`; checked for the TSDoc block marker.
     */
    const c = comments[loopIndex];
    if ((c !== undefined) && isTsdocBlock(c,))
      return c;
  }

  // Only fall back for declaration-level nodes, not expressions inside them
  /**
   Node type string; gates whether the slow whole-file fallback is allowed.
   */
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- oxlint plugin API is untyped
  const nodeType = (node as unknown as Record<string, unknown>).type;
  if (((typeof nodeType)
    !== 'string') || (!FALLBACK_ELIGIBLE_TYPES.has(nodeType,)))
    return NO_TSDOC;

  // Fallback: scan all comments for the closest TSDoc ending on the line
  // immediately before this node. Handles exported declarations where
  // getCommentsBefore returns nothing because the comment is before the
  // `export` keyword rather than the inner declaration.
  /**
   Starting line of the declaration; comments must end exactly one line above.
   */
  const nodeStartLine = node.loc
    .start
    .line;
  /**
   Full comment table for the file; needed because `getCommentsBefore` misses cross-scope ones.
   */
  const allComments = context.sourceCode
    .getAllComments();

  /**
   Closest TSDoc comment found so far, tracked as the loop scans the whole comment table.
   */
  let best: Comment | typeof NO_TSDOC = NO_TSDOC;
  for (const candidate of allComments) {
    if (!isTsdocBlock(candidate,))
      continue;
    /**
     End line of the candidate comment; must immediately precede `nodeStartLine`.
     */
    const candidateEndLine = candidate.loc
      .end
      .line;
    if (candidateEndLine >= nodeStartLine)
      continue;
    if ((nodeStartLine - candidateEndLine) > 1)
      continue;
    if ((best === NO_TSDOC) || (candidate.loc
      .end
      .line
      > best
        .loc
        .end
        .line))
      best = candidate;
  }

  return best;
}

/**
 Scans one TSDoc body once and reuses facts across sibling plugin rules.
 
 @param comment - Host comment whose normalized body supplies cache key.
 
 @returns parsed document model and structural messages.
 
 @example
 ```ts
 const facts = parseTsdocComment(comment);
 console.log(facts.docComment.params.blocks.length);
 ```
 */
export function parseTsdocComment(comment: ReadonlyDeep<Comment>,): ParsedCommentFacts {
  /**
   Exact comment body used because parsed facts are independent of absolute source location.
   */
  const cacheKey = comment.value;
  /**
   Previously scanned facts for identical comment body.
   */
  const cached = PARSED_COMMENT_FACTS_BY_TEXT.get(cacheKey,);
  if (cached !== undefined) {
    PARSED_COMMENT_FACTS_BY_TEXT.delete(cacheKey,);
    PARSED_COMMENT_FACTS_BY_TEXT.set(
      cacheKey,
      cached,
    );
    return cached;
  }

  /**
   Newly scanned facts retained in bounded least-recently-used order.
   */
  const parsed = {
    docComment: splitDocComment({ comment, },),
    messages: collectStructuralMessages({ comment, },),
  };
  PARSED_COMMENT_FACTS_BY_TEXT.set(
    cacheKey,
    parsed,
  );
  if (PARSED_COMMENT_FACTS_BY_TEXT.size > MAX_CACHED_COMMENT_TEXTS) {
    /**
     Oldest insertion-order key evicted to bound CLI memory.
     */
    const oldest = PARSED_COMMENT_FACTS_BY_TEXT.keys()
      .next()
      .value;
    if (oldest !== undefined)
      PARSED_COMMENT_FACTS_BY_TEXT.delete(oldest,);
  }
  return parsed;
}

/**
 Extracts and caches parsed TSDoc facts for given declaration node.
 
 @param node - Declaration whose leading comment is resolved.
 
 @param context - Host context providing comments and filename.
 
 @returns parsed result or absence marker when declaration has no TSDoc.
 
 @example
 ```ts
 const result = parseTsdocForNode({ node, context });
 ```
 */
export function parseTsdocForNode({
  node,
  context,
}: ForeignBorrowed<TsdocLookupParams>,): ParsedTsdocResult | typeof NO_TSDOC {
  /**
   Located TSDoc comment for the node; absent means nothing to parse.
   */
  const comment = findTsdocComment({
    node,
    context,
  },);
  if (comment === NO_TSDOC)
    return NO_TSDOC;

  return {
    comment,
    ...parseTsdocComment(comment,),
  };
}
