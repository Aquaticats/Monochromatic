import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  Comment,
  Context,
  Span,
} from '@oxlint/plugins';

//region Structural AST view
// The node fields the chain walk reads; `parent` is the walker link and every
// other field is optional because one shape covers every node kind.

/**
 AST node fields the chain walk reads.
 
 oxlint types every node's `parent` and child links as the bare {@link Span}
 shape, which omits the discriminant and the structural links the chain walk
 needs. This self-referential view exposes them; the visitor casts its node to
 this type once, after which the walk stays cast-free. Every field beyond
 `type` is optional because a single shape covers member, call, operator,
 wrapper, and leaf nodes alike.
 
 @internal
 */
export type ChainNode = Span & {
  /**
   AST node-type discriminant, for example `'MemberExpression'`.
   */
  readonly type: string;
  /**
   `MemberExpression.object`: the receiver of a member access.
   */
  readonly object?: ChainNode;
  /**
   `CallExpression.callee`: the callee of a call.
   */
  readonly callee?: ChainNode;
  /**
   Inner expression of a `ChainExpression`, `!`, `as`, or `satisfies` wrapper.
   */
  readonly expression?: ChainNode;
  /**
   `MemberExpression.property`: the accessed name or computed key.
   */
  readonly property?: Span;
  /**
   `true` when a `MemberExpression` uses computed `[expr]` access.
   */
  readonly computed?: boolean;
  /**
   Left operand of a `BinaryExpression` or `LogicalExpression`.
   */
  readonly left?: ChainNode;
  /**
   Right operand of a `BinaryExpression` or `LogicalExpression`.
   */
  readonly right?: ChainNode;
  /**
   Operator literal of a `BinaryExpression` or `LogicalExpression`.
   */
  readonly operator?: string;
  /**
   Parent link surfaced on every node by oxlint's visitor walker.
   */
  readonly parent?: ChainNode;
};
//endregion Structural AST view

//region Grouping parentheses
// oxlint strips parentheses from the AST, so grouping is read back from the
// token stream around a node.

/**
 Parameters for {@link parenIsolated}.
 
 Threads the rule {@link Context} rather than its `sourceCode` directly: oxlint's
 `SourceCode` is an anonymous `Readonly<typeof ...>` the readonly-params
 allow-list cannot name-match, whereas {@link Context} is allow-listed by name. The
 token accessors live on `context.sourceCode`.
 */
export type ParenIsolatedParams = {
  /**
   Rule context; its `sourceCode` supplies the surrounding-token lookups.
   */
  readonly context: Context;
  /**
   Node whose immediate token neighbours decide grouping isolation.
   */
  readonly node: Span;
};

/**
 Determines whether a node is wrapped in its own grouping parentheses.
 
 oxlint strips parentheses from the AST (it emits no `ParenthesizedExpression`
 node), so grouping is recovered from the token stream: the node is isolated
 when the token immediately before it is `(` and the token immediately after
 it is `)`. The chain walk only consults this in structural positions (a chain
 receiver, a chain callee, an operator operand, or a chain root) where a
 bracketing `(`/`)` pair can only be grouping; a call argument's or `if`-test's
 parentheses never abut a node in those positions, so the historic
 false-positive on `f(a + b)` cannot misdirect a decision here.
 
 @returns whether `( ... )` brackets the node as a grouping pair
 
 @example
 ```ts
 // For `(a + b).c`, with node = BinaryExpression `a + b`: true
 parenIsolated({ context, node, });
 ```
 */
export function parenIsolated({
  context,
  node,
}: ForeignBorrowed<ParenIsolatedParams>,): boolean {
  /**
   Token immediately before the node; `(` when the node opens a grouping.
   */
  const before = context.sourceCode
    .getTokenBefore(node,);
  /**
   Token immediately after the node; `)` when the node closes a grouping.
   */
  const after = context.sourceCode
    .getTokenAfter(node,);
  return (before !== null)
    && (before.value
      === '(')
    && (after !== null)
    && (after.value
      === ')');
}
//endregion Grouping parentheses

//region Transparent wrappers
// `ChainExpression` and the TypeScript `!`, `as`, and `satisfies` expressions
// enclose a chain without being one, so the walk steps through them.

/**
 Parameters for {@link wrapsChild}.
 */
type WrapsChildParams = {
  /**
   Candidate transparent-wrapper parent.
   */
  readonly parent: ChainNode;
  /**
   Child the parent must wrap to count.
   */
  readonly child: ChainNode;
};

/**
 Reports whether `parent` is a transparent wrapper around `child`.
 
 The transparent wrappers are the optional-chaining `ChainExpression` marker
 and the TypeScript `!`, `as`, and `satisfies` expressions. Each holds its
 inner expression in `.expression`; matching that link by identity confirms
 `child` is wrapped rather than merely adjacent.
 
 @returns whether `parent` transparently wraps `child`
 */
function wrapsChild({
  parent,
  child,
}: ForeignBorrowed<WrapsChildParams>,): boolean {
  /**
   Whether the parent is one of the four transparent wrapper kinds.
   */
  const isWrapper = (parent.type
    === 'ChainExpression')
    || (parent.type
      === 'TSNonNullExpression')
    || (parent.type
      === 'TSAsExpression')
    || (parent.type
      === 'TSSatisfiesExpression');
  return isWrapper && (parent.expression
    === child);
}

/**
 Walks up from a core chain node through any transparent wrappers that enclose
 it and returns the outermost such node.
 
 The chain region ends at this node so trailing wrapper text (`!`, `as T`,
 `satisfies T`) and the optional-chaining marker are inside the region and
 survive the render. A non-wrapped node returns itself.
 
 @param node - core chain node to walk up from
 
 @returns outermost transparent wrapper enclosing `node`, or `node` itself
 
 @example
 ```ts
 // For `a.b.c.d as Foo`, with node = MemberExpression `a.b.c.d`:
 effectiveTop(node); // the enclosing TSAsExpression
 ```
 */
export function effectiveTop(node: ForeignBorrowed<ChainNode>,): ChainNode {
  /**
   Parent link; absent at program scope.
   */
  const { parent, } = node;
  if (parent === undefined)
    return node;
  if (wrapsChild({
    parent,
    child: node,
  },)) {
    return effectiveTop(parent,);
  }
  return node;
}
//endregion Transparent wrappers

//region Chain roots
// A node is laid out only when no larger chain absorbs it, and grouping
// parentheses isolate it exactly as `no-mixed-operators` treats a
// parenthesised subexpression.

/**
 Parameters for {@link isChainRoot}.
 */
export type IsChainRootParams = {
  /**
   Rule context; its `sourceCode` supplies the grouping-parenthesis check.
   */
  readonly context: Context;
  /**
   Visited core node (`MemberExpression`, `CallExpression`, `BinaryExpression`, or `LogicalExpression`).
   */
  readonly node: ChainNode;
};

/**
 Determines whether a visited node is the outermost root of its chain.
 
 A node is the root unless a larger chain absorbs it: the effective parent
 (found past any transparent wrappers) continues the chain as a member
 receiver, a call callee, or an operator operand. Grouping parentheses around
 the node override absorption, isolating it as its own root exactly as
 `no-mixed-operators` treats a parenthesised subexpression. Firing only on the
 root lays out the whole flattened chain in one pass, including chains nested
 as operands of an operator chain.
 
 @returns whether the node should be laid out as a chain root
 
 @example
 ```ts
 // For `a.b.c.d`: the outermost MemberExpression returns true; inner ones false
 isChainRoot({ context, node, });
 ```
 */
export function isChainRoot({
  context,
  node,
}: ForeignBorrowed<IsChainRootParams>,): boolean {
  /**
   Outermost transparent wrapper around the node; the parent of this decides absorption.
   */
  const top = effectiveTop(node,);
  if (parenIsolated({
    context,
    node: top,
  },)) {
    return true;
  }
  /**
   Effective parent: the first ancestor that is not a transparent wrapper.
   */
  const { parent, } = top;
  if (parent === undefined)
    return true;
  if ((parent.type
    === 'MemberExpression') && (parent.object
      === top))
    return false;
  if ((parent.type
    === 'CallExpression') && (parent.callee
      === top))
    return false;
  /**
   Whether the effective parent is an operator that takes `top` as an operand.
   */
  const parentIsOperator = (parent.type
    === 'BinaryExpression')
    || (parent.type
      === 'LogicalExpression');
  if (parentIsOperator && ((parent.left
    === top) || (parent.right
      === top)))
    return false;
  return true;
}
//endregion Chain roots

//region Reflowable comments
// Only a comment inside the collapsible head is moved by the autofix, so only
// that one is asked about.

/**
 Parameters for {@link hasReflowableComment}.
 */
export type HasReflowableCommentParams = {
  /**
   Rule context; its `sourceCode` supplies the comment lookup.
   */
  readonly context: Context;
  /**
   Outermost chain-region node whose interior comments are inspected.
   */
  readonly node: Span;
  /**
   First break offset; a comment before it sits in the collapsible head.
   */
  readonly firstBreak: number;
};

/**
 Reports whether a comment sits where the autofix would reflow it.
 
 The render emits the head slice (region start to the first break) as one
 collapsed unit and every later slice verbatim, inserting a newline only at a
 break offset, which is always a `.`/`?.` or operator token and so never lands
 inside a comment. A comment therefore survives untouched unless it sits in the
 head, the one region where separate segments fold onto a single line; a
 comment at or after the first break rides verbatim on its own continuation
 slice (for example one buried in a trailing call's arguments). Suppressing the
 fix exactly when a comment precedes the first break reinstates the
 no-relocation guarantee while leaving call-argument comments fixable. Comments
 cannot straddle a break offset: the break token would otherwise be inside the
 comment and so not a token at all, so testing each comment's start against the
 first break classifies it fully.
 
 @returns whether any comment precedes the first break offset
 
 @example
 ```ts
 // For `obj.b // note\n  .c.d` with firstBreak at `.c`: true (note precedes .c)
 // For `obj.a.method(\n  // note\n  x,\n)` with firstBreak at `.method`: false
 hasReflowableComment({ context, node, firstBreak, });
 ```
 */
export function hasReflowableComment({
  context,
  node,
  firstBreak,
}: ForeignBorrowed<HasReflowableCommentParams>,): boolean {
  return context.sourceCode
    .getCommentsInside(node,)
    .some(function precedesFirstBreak(comment: ForeignBorrowed<Comment>,): boolean {
      return comment.start
        < firstBreak;
    },);
}
//endregion Reflowable comments
