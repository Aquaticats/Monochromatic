import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { Span, } from '@oxlint/plugins';

import { at, } from './range.ts';

//region Structural AST view
// The node fields the invocation-depth walk reads; `parent` is the walker link,
// and nullable oxlint fields are typed non-null here and reconciled at the read
// site so the view carries no `T | null` union.

/**
 AST node fields the invocation-depth walk reads.
 
 oxlint types every node's `parent` and child links as the bare {@link Span}
 shape, which omits the discriminant and the structural links the spine walk
 needs. This self-referential view exposes them; the rule casts its visited
 node to this type once, after which the walk stays cast-free. Every field
 beyond `type` is optional because a single shape covers call, new, dynamic
 import, transparent-wrapper, and leaf nodes alike. Fields oxlint declares as
 nullable (`typeArguments`, `options`, a bare `yield`'s `argument`) are typed
 non-null here and reconciled at the read site with a `?? STOP` or an explicit
 `null` guard, keeping the view free of the banned `T | null` union.
 */
export type SpineNode = Span & {
  /**
   AST node-type discriminant, for example `'CallExpression'`.
   */
  readonly type: string;
  /**
   `CallExpression`/`NewExpression` callee; the rule descends arguments, never this.
   */
  readonly callee?: SpineNode;
  /**
   Type-argument span (`a<T>(x)`); anchors the opening-paren search past `>`.
   */
  readonly typeArguments?: Span;
  /**
   `CallExpression`/`NewExpression` argument list; the spine continues only when length is 1.
   */
  readonly arguments?: readonly SpineNode[];
  /**
   `ImportExpression.source`: the dynamic-import specifier operand.
   */
  readonly source?: SpineNode;
  /**
   `ImportExpression.options`: the second argument; a present value stops the import spine.
   */
  readonly options?: SpineNode;
  /**
   Inner expression of a `ChainExpression` or TypeScript wrapper (`!`, `as`, `satisfies`, `<...>`, type-assertion).
   */
  readonly expression?: SpineNode;
  /**
   Operand of an `await`, unary, `yield`, or spread wrapper.
   */
  readonly argument?: SpineNode;
  /**
   Parent link surfaced on every node by oxlint's visitor walker.
   */
  readonly parent?: SpineNode;
};
//endregion Structural AST view

//region Spine stop and wrapper kinds
// The sentinel that ends a walk, and the two wrapper families the spine passes
// through without counting them.

/**
 Spine-stop sentinel returned where a walk has no further node.
 
 A unique `Symbol` rather than `null`/`undefined` keeps "no node" out of a
 `SpineNode | null` union (banned by `no-nullish-union`), mirroring
 `chain-flatten.ts`'s `LEAF`; the walk gates on `=== STOP`.
 */
const STOP: unique symbol = Symbol('oxlint-stylistic:invocation-spine:stop',);

/**
 Transparent wrappers that hold their inner expression in `.expression`.
 
 The optional-chaining marker and the value-preserving TypeScript wrappers; the
 spine passes through each into its inner expression without counting it.
 */
const WRAPPER_EXPRESSION_TYPES: ReadonlySet<string> = new Set([
  'ChainExpression',
  'TSAsExpression',
  'TSSatisfiesExpression',
  'TSTypeAssertion',
  'TSNonNullExpression',
  'TSInstantiationExpression',
],);

/**
 Transparent wrappers that hold their inner expression in `.argument`.
 
 `await`, unary operators (`void`, `!`, `typeof`, ...), `yield`/`yield*`, and a
 single spread argument; the spine follows visible invocation density through
 each rather than only value-preserving wrappers.
 */
const WRAPPER_ARGUMENT_TYPES: ReadonlySet<string> = new Set([
  'AwaitExpression',
  'UnaryExpression',
  'YieldExpression',
  'SpreadElement',
],);
//endregion Spine stop and wrapper kinds

//region Invocation heads and wrapper descent
// Which node kinds count toward a line's invocation total, and how the walk
// steps through the wrappers around them.

/**
 Reports whether a node is one of the counted invocation heads.
 
 `CallExpression` (including optional calls), `NewExpression`, and
 `ImportExpression` are the three forms whose heads count toward a source
 line's invocation total.
 
 @param node - candidate AST node
 
 @returns whether the node is a counted invocation
 */
function isCounted(node: ForeignBorrowed<SpineNode>,): boolean {
  return (node.type
    === 'CallExpression')
    || (node.type
      === 'NewExpression')
    || (node.type
      === 'ImportExpression');
}

/**
 Reports whether a node type is a transparent spine wrapper.
 
 @param type - AST node-type discriminant
 
 @returns whether the spine passes through the type without counting it
 */
function isTransparentWrapper(type: string,): boolean {
  return WRAPPER_EXPRESSION_TYPES.has(type,)
    || WRAPPER_ARGUMENT_TYPES.has(type,);
}

/**
 Returns the inner expression a transparent wrapper holds, or {@link STOP}.
 
 `.expression` for the chain/TypeScript wrappers, `.argument` for the
 await/unary/yield/spread wrappers; a bare `yield` with no argument yields
 {@link STOP}, which stops the spine.
 
 @param node - transparent-wrapper node
 
 @returns inner expression, or {@link STOP} when the wrapper has no operand
 */
function transparentInner(node: ForeignBorrowed<SpineNode>,): SpineNode | typeof STOP {
  if (WRAPPER_EXPRESSION_TYPES.has(node.type,))
    return node.expression ?? STOP;
  if (WRAPPER_ARGUMENT_TYPES.has(node.type,))
    return node.argument ?? STOP;
  return STOP;
}

/**
 Descends through any transparent wrappers and returns the wrapped node.
 
 Iterative because wrapper nesting (`await void !x`) tracks source length, not
 structural depth, and the repo bans recursion over linear input. A non-wrapper
 node returns itself; a wrapper with no operand returns {@link STOP}.
 
 @param node - node to descend from
 
 @returns innermost wrapped node, or {@link STOP} when a wrapper has no operand
 */
function descendWrappers(node: ForeignBorrowed<SpineNode>,): SpineNode | typeof STOP {
  for (let cursor: SpineNode = node;;) {
    if (!isTransparentWrapper(cursor.type,))
      return cursor;
    /**
     Inner expression of the current wrapper; {@link STOP} ends the descent.
     */
    const inner = transparentInner(cursor,);
    if (inner === STOP)
      return STOP;
    cursor = inner;
  }
}
//endregion Invocation heads and wrapper descent

//region Single-operand access
// The operand a spine step reads, raw for the autofix and thrown-on for a
// reported owner.

/**
 Returns the raw single operand of a counted invocation, or {@link STOP}.
 
 A `CallExpression`/`NewExpression` continues its spine only when it has
 exactly one argument; an `ImportExpression` only when its `options` is absent,
 in which case the operand is `.source`. The returned node is the raw operand
 (a wrapper or the argument itself), not the wrapper-descended invocation; the
 autofix slices from it and {@link nextSpineNode} descends it.
 
 @param node - counted invocation node
 
 @returns raw single operand, or {@link STOP} when the spine does not continue
 */
function operandOf(node: ForeignBorrowed<SpineNode>,): SpineNode | typeof STOP {
  if ((node.type
    === 'CallExpression')
    || (node.type
      === 'NewExpression')) {
    /**
     Argument list; a single argument is the spine operand.
     */
    const args = node.arguments;
    if ((args === undefined) || (args.length
      !== 1))
      return STOP;
    return at({
      arr: args,
      index: 0,
    },);
  }
  if (node.type
    === 'ImportExpression') {
    /**
     Second dynamic-import argument; a present value stops the source spine.
     */
    const { options, } = node;
    // oxlint emits `null` (not absence) for `import(x)`; a real node means two arguments.
    if ((options !== null) && (options !== undefined))
      return STOP;
    return node.source ?? STOP;
  }
  return STOP;
}

/**
 Returns the single operand of a counted invocation, throwing when absent.
 
 The rule reports only invocations whose spine continues, so a reported owner
 always carries a single operand; the autofix calls this to slice it.
 
 @param node - counted invocation known to carry a single operand
 
 @returns raw single operand
 
 @throws when the node has no single operand, which is unreachable for a
   reported owner
 
 @example
 ```ts
 // For `a(b(c()))`, operandOrThrow(a) is the `b(c())` node.
 operandOrThrow(node);
 ```
 */
export function operandOrThrow(node: ForeignBorrowed<SpineNode>,): SpineNode {
  /**
   Raw single operand or the stop sentinel.
   */
  const operand = operandOf(node,);
  if (operand === STOP) {
    throw new Error('counted invocation has no single operand to split',);
  }
  return operand;
}
//endregion Single-operand access

//region Spine walking
// Steps down the operand spine, up through enclosing wrappers, and collects one
// root's whole spine.

/**
 Returns the next counted invocation down a node's operand spine, or {@link STOP}.
 
 Descends the raw operand through transparent wrappers; the spine continues
 only when the descended node is itself a counted invocation. A container,
 literal, or other leaf returns {@link STOP}, so the spine stops there while the
 rule's normal visitor still checks invocations inside it independently.
 
 @param node - counted invocation to step down from
 
 @returns next counted invocation on the spine, or {@link STOP}
 */
function nextSpineNode(node: ForeignBorrowed<SpineNode>,): SpineNode | typeof STOP {
  /**
   Raw single operand; {@link STOP} when arity or import options break the spine.
   */
  const operand = operandOf(node,);
  if (operand === STOP)
    return STOP;
  /**
   Operand past any transparent wrappers.
   */
  const descended = descendWrappers(operand,);
  if ((descended !== STOP) && isCounted(descended,))
    return descended;
  return STOP;
}

/**
 Walks up through any transparent wrappers enclosing a node.
 
 Iterative for the same reason as {@link descendWrappers}: wrapper nesting
 tracks source length. Stops at the first ancestor that is not a transparent
 wrapper of the current node.
 
 @param node - node to walk up from
 
 @returns outermost transparent wrapper enclosing `node`, or `node` itself
 */
function wrapperTop(node: ForeignBorrowed<SpineNode>,): SpineNode {
  for (let top: SpineNode = node;;) {
    /**
     Parent link; absent at program scope.
     */
    const { parent, } = top;
    if (parent === undefined)
      return top;
    if (!isTransparentWrapper(parent.type,))
      return top;
    if (transparentInner(parent,)
      !== top)
      return top;
    top = parent;
  }
}

/**
 Determines whether a counted invocation begins its own operand spine.
 
 A node is a root unless a parent counted invocation continues its spine into
 it: walking up past transparent wrappers, the first non-wrapper ancestor is a
 counted invocation whose single-operand spine descends back to this node.
 Firing only on roots lays out each spine once and keeps callee chains (owned
 by `chain-per-line`) and multi-argument parents (owned by `argument-per-line`)
 from absorbing the node.
 
 @param node - counted invocation under consideration
 
 @returns whether the node should be treated as a spine root
 
 @example
 ```ts
 // For `a(b(c()))`: the outer `a` is a root; `b` and `c` are not.
 isSpineRoot(node);
 ```
 */
export function isSpineRoot(node: ForeignBorrowed<SpineNode>,): boolean {
  /**
   Outermost transparent wrapper around the node; its parent decides absorption.
   */
  const top = wrapperTop(node,);
  /**
   Effective parent: the first ancestor that is not a transparent wrapper.
   */
  const { parent, } = top;
  if (parent === undefined)
    return true;
  if (isCounted(parent,) && (nextSpineNode(parent,)
    === node))
    return false;
  return true;
}

/**
 Collects a root's operand spine, outermost invocation first.
 
 Iterative cursor walk: a single-operand invocation chain is a spine whose
 depth equals its length, so recursion would be linear-depth and overflow on
 long chains. Each element is a counted invocation; the autofix and the
 per-line counting consume the returned order.
 
 @param root - spine root, as decided by {@link isSpineRoot}
 
 @returns counted invocations from the root down, in source-nesting order
 
 @example
 ```ts
 // For `a(b(c()))`: [a, b, c]
 collectSpine(rootNode);
 ```
 */
export function collectSpine(root: ForeignBorrowed<SpineNode>,): readonly SpineNode[] {
  /**
   Spine accumulator seeded with the root; later invocations push on.
   */
  const spine: SpineNode[] = [root,];
  for (
    let cursor = nextSpineNode(root,);
    cursor !== STOP;
    cursor = nextSpineNode(cursor,)
  ) {
    spine.push(cursor,);
  }
  return spine;
}
//endregion Spine walking
