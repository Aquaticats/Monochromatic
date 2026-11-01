/**
 Shared TSDoc visitor factories for oxlint rule implementations.
 
 Provides reusable visitor constructors so each rule file does not duplicate
 node-type enumeration, ignored-file preambles, or comment location arithmetic.
 
 @module
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  Comment,
  Context,
  ESTree,
  LineColumn,
  Span,
  VisitorWithHooks,
} from '@oxlint/plugins';
import type { ReadonlyDeep, } from 'type-fest';

import type { ReadonlyRecord, } from '../ast-access.ts';
import {
  findTsdocComment,
  NO_TSDOC,
  parseTsdocForNode,
  shouldIgnoreFile,
  type TsdocParseResult,
} from '../tsdoc-utils.ts';

export {
  getCommentLines,
  stripCommentLineMarker,
} from '../comment-text.ts';

/**
 Parameters for {@link shouldSkipIgnoredFile}.
 */
export type IgnoredFileBeforeHookParams = {
  /**
   Oxlint rule context carrying filename.
   */
  readonly context: Context;
};

/**
 Parameters for {@link commentLineReportLoc}.
 */
export type CommentLineReportLocParams = {
  /**
   TSDoc comment whose start line anchors the diagnostic.
   */
  readonly comment: ReadonlyDeep<Comment>;
  /**
   Zero-based line offset from the comment start line.
   */
  readonly lineOffset: number;
  /**
   Report column, defaulting to the start of the line.
   */
  readonly column?: number;
};

/**
 Parameters for documentable-node visitor construction.
 */
type CreateDocumentableVisitorParams = {
  /**
   Oxlint rule context.
   */
  readonly context: Context;
  /**
   Invoked for each documentable node type.
   */
  readonly check: (node: ForeignBorrowed<Span>) => void;
};

/**
 Checks whether current file is excluded from TSDoc validation, delegating
 to {@link shouldIgnoreFile}.
 
 @param params - rule context carrying current filename
 
 @returns whether visitor should skip current file
 
 @example
 ```ts
 if (shouldSkipIgnoredFile({ context })) return false;
 ```
 */
export function shouldSkipIgnoredFile(params: ForeignBorrowed<IgnoredFileBeforeHookParams>,): boolean {
  /**
   Rule context carrying current filename.
   */
  const { context, } = params;
  return shouldIgnoreFile(context.filename,);
}

/**
 Builds a fresh report location from a comment's span.
 
 `context.report` needs a mutable `Ranged` / location, but rule handlers
 receive deeply-readonly comments; copying the numeric positions into a
 new object crosses that readonly boundary without an assertion.
 
 @param comment - TSDoc comment whose span locates the diagnostic
 
 @returns location object accepted by `context.report({ loc })`
 
 @example
 ```ts
 context.report({ loc: commentReportLoc(comment), messageId: 'x' });
 ```
 */
export function commentReportLoc(comment: ReadonlyDeep<Comment>,): {
  start: LineColumn;
  end: LineColumn;
} {
  return {
    start: {
      line: comment.loc
        .start
        .line,
      column: comment.loc
        .start
        .column,
    },
    end: {
      line: comment.loc
        .end
        .line,
      column: comment.loc
        .end
        .column,
    },
  };
}

/**
 Builds a report location anchored to one line inside a comment.
 
 @param params - comment, relative line offset, and optional column
 
 @returns location object with a copied mutable start position
 
 @example
 ```ts
 context.report({ loc: commentLineReportLoc({ comment, lineOffset: index }) });
 ```
 */
export function commentLineReportLoc(
  params: CommentLineReportLocParams,
): { start: LineColumn; } {
  /**
   Comment location inputs for the copied report location.
   */
  const {
    comment,
    lineOffset,
    column = 0,
  } = params;
  return {
    start: {
      line: comment.loc
        .start
        .line
        + lineOffset,
      column,
    },
  };
}

/**
 Creates a visitor over every documentable node type, skipping ignored
 files via {@link shouldSkipIgnoredFile}.
 
 @param params - rule context and per-node callback
 
 @returns visitor with ignored-file preamble and node handlers
 
 @example
 ```ts
 createDocumentableVisitor({ context, check });
 ```
 */
function createDocumentableVisitor(params: ForeignBorrowed<CreateDocumentableVisitorParams>,): VisitorWithHooks {
  /**
   Rule context and callback used by every documentable-node visitor.
   */
  const {
    context,
    check,
  } = params;
  return {
    before() {
      if (shouldSkipIgnoredFile({ context, }))
        return false;
      return undefined;
    },
    FunctionDeclaration: check,
    FunctionExpression: check,
    ArrowFunctionExpression: check,
    ClassDeclaration: check,
    MethodDefinition: check,
    TSInterfaceDeclaration: check,
    TSTypeAliasDeclaration: check,
    TSEnumDeclaration: check,
    VariableDeclaration: check,
    PropertyDefinition: check,
    TSEnumMember: check,
    Property(node: ForeignBorrowed<
      | ESTree.AssignmentTargetProperty
        | ESTree.BindingProperty
        | ESTree.ObjectProperty
      >,): void {
      if ((node.kind
        === 'get') || (node.kind
          === 'set'))
        check(node,);
    },
  } as VisitorWithHooks;
}

/**
 Parameters for {@link createTsdocVisitor}.
 */
export type CreateTsdocVisitorParams = {
  /**
   Oxlint rule context.
   */
  readonly context: Context;
  /**
   Invoked for each (node, comment) pair.
   */
  readonly handler: (
    node: ForeignBorrowed<Span>,
    comment: ReadonlyDeep<Comment>,
  ) => void;
};

/**
 Creates a visitor (built on {@link createDocumentableVisitor}) that
 iterates over all documentable node types and calls the provided handler
 when {@link findTsdocComment} finds a TSDoc comment.
 
 @param params - rule context and comment handler
 
 @returns visitor with hooks
 
 @example
 ```ts
 return createTsdocVisitor({ context, handler });
 ```
 */
export function createTsdocVisitor(params: ForeignBorrowed<CreateTsdocVisitorParams>,): VisitorWithHooks {
  /**
   Rule context and handler for located TSDoc comments.
   */
  const {
    context,
    handler,
  } = params;
  /**
   Checks node and fires handler when TSDoc exists.
   
   @param node - AST node to check
   */
  function check(node: ForeignBorrowed<Span>,): void {
    /**
     Located TSDoc comment for the node; only when present does the handler fire.
     */
    const comment = findTsdocComment({
      node,
      context,
    },);
    if (comment !== NO_TSDOC) {
      handler(
        node,
        comment,
      );
    }
  }

  return createDocumentableVisitor({
    context,
    check,
  },);
}

/**
 Parameters for {@link createParsedTsdocVisitor}.
 */
export type CreateParsedTsdocVisitorParams = {
  /**
   Oxlint rule context.
   */
  readonly context: Context;
  /**
   Invoked with node and parsed TSDoc for each documentable node.
   */
  readonly handler: (
    node: ForeignBorrowed<Span>,
    result: ReadonlyDeep<TsdocParseResult>,
  ) => void;
};

/**
 Creates a visitor (built on {@link createDocumentableVisitor}) over every
 documentable node and supplies parsed TSDoc via {@link parseTsdocForNode}.
 
 @param params - rule context and parsed-comment handler
 
 @returns visitor with hooks
 
 @example
 ```ts
 return createParsedTsdocVisitor({ context, handler });
 ```
 */
export function createParsedTsdocVisitor(params: ForeignBorrowed<CreateParsedTsdocVisitorParams>,): VisitorWithHooks {
  /**
   Rule context and handler for parsed TSDoc comments.
   */
  const {
    context,
    handler,
  } = params;
  /**
   Checks node and fires handler when parsed TSDoc exists.
   
   @param node - AST node to check
   */
  function check(node: ForeignBorrowed<Span>,): void {
    /**
     Parsed TSDoc bundle for the node; absent means no TSDoc and the handler is skipped.
     */
    const result = parseTsdocForNode({
      node,
      context,
    },);
    if (result === NO_TSDOC)
      return;
    handler(
      node,
      result,
    );
  }

  return createDocumentableVisitor({
    context,
    check,
  },);
}

/**
 Parameters for {@link createFunctionTsdocVisitor}.
 */
export type CreateFunctionTsdocVisitorParams = {
  /**
   Oxlint rule context.
   */
  readonly context: Context;
  /**
   Whether arrow functions should be visited.
   */
  readonly includeArrowFunctions?: boolean;
  /**
   Whether bodyless TypeScript signature declarations should be visited.
   */
  readonly includeTypeSignatures?: boolean;
  /**
   Invoked with node and parsed TSDoc for each function-like node.
   */
  readonly handler: (
    node: ForeignBorrowed<Span & ReadonlyRecord>,
    result: ReadonlyDeep<TsdocParseResult>,
  ) => void;
};

/**
 Creates a visitor for function-like nodes that have TSDoc comments, parsed
 via {@link parseTsdocForNode}.
 
 Covers runtime callables by default.
 Callers may include bodyless TypeScript signatures explicitly or exclude
 arrow functions.
 
 @param params - rule context, arrow-function toggle, and parsed-comment handler
 
 @returns visitor with hooks
 
 @example
 ```ts
 return createFunctionTsdocVisitor({ context, handler });
 ```
 */
export function createFunctionTsdocVisitor(
  params: ForeignBorrowed<CreateFunctionTsdocVisitorParams>,
): VisitorWithHooks {
  /**
   Rule context, arrow toggle, and handler for parsed function TSDoc comments.
   */
  const {
    context,
    includeArrowFunctions = true,
    includeTypeSignatures = false,
    handler,
  } = params;
  /**
   Checks a function-like node for TSDoc and invokes handler.
   
   @param node - AST node to check
   */
  function check(node: ForeignBorrowed<Span>,): void {
    /**
     Parsed TSDoc bundle for the node; absent means no TSDoc and the handler is skipped.
     */
    const result = parseTsdocForNode({
      node,
      context,
    },);
    if (result === NO_TSDOC)
      return;
    /**
     Narrowed view that exposes the host AST's untyped extra properties to the handler.
     */
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- oxlint plugin API is untyped
    const typedNode = node as Span & Record<string, unknown>;
    handler(
      typedNode,
      result,
    );
  }

  /**
   Visitor object built before the unsafe cast that satisfies the host's index-signature type.
   */
  const visitor = {
    before() {
      if (shouldSkipIgnoredFile({ context, }))
        return false;
      return undefined;
    },
    FunctionDeclaration: check,
    FunctionExpression: check,
    ...includeArrowFunctions
      ? { ArrowFunctionExpression: check, }
      : {},
    MethodDefinition: check,
    ...includeTypeSignatures
      ? {
        TSAbstractMethodDefinition: check,
        TSDeclareFunction: check,
        TSMethodSignature: check,
        TSCallSignatureDeclaration: check,
        TSConstructSignatureDeclaration: check,
      }
      : {},
  } as VisitorWithHooks;
  return visitor;
}
