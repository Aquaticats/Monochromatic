import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  Comment,
  Context,
  Token,
} from '@oxlint/plugins';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  type ChainNode,
  parenIsolated,
} from './chain.ts';
import {
  type ChainBreak,
  type ChainSegment,
  selectBreaks,
} from './chain-render.ts';

/**
 Shared attached-segment value: rides on the previous segment's line, never a break point.
 */
const ATTACHED: ChainSegment = { isBreak: false, };

/**
 Leaf sentinel returned by {@link descentChild} when a chain node has no
 receiver to descend into. A unique `Symbol` rather than `undefined` keeps
 "no child" out of a `ChainNode | undefined` union (banned by
 `no-nullish-union`); the walk gates on `=== LEAF`.
 */
const LEAF: unique symbol = Symbol('oxlint-stylistic:chain-flatten:leaf',);

/**
 Reports whether a node type is a transparent chain wrapper.
 
 The walk passes through these without emitting a segment of their own; their
 source text (`!`, `as T`, `satisfies T`, or nothing for `ChainExpression`)
 folds into the slice of the segment before them.
 
 @param type - AST node-type discriminant
 
 @returns whether the type is `ChainExpression`, `!`, `as`, or `satisfies`
 */
function isTransparentWrapper(type: string,): boolean {
  return (type === 'ChainExpression')
    || (type === 'TSNonNullExpression')
    || (type === 'TSAsExpression')
    || (type === 'TSSatisfiesExpression');
}

/**
 Parameters for {@link memberDotStart}.
 */
type MemberDotStartParams = {
  /**
   Rule context; its `sourceCode` supplies the property-preceding token lookup.
   */
  readonly context: Context;
  /**
   Non-computed `MemberExpression` whose `.`/`?.` offset is wanted.
   */
  readonly member: ChainNode;
};

/**
 Returns the byte offset of a member step's `.` or `?.` token.
 
 The token immediately before the property name is the dot (or optional-chain)
 punctuator; its start is where a continuation line for this step begins.
 
 @returns byte offset of the member step's leading dot token
 
 @throws when the property or its preceding token is absent, which is
   unreachable for a non-computed member access
 */
function memberDotStart({
  context,
  member,
}: ForeignBorrowed<MemberDotStartParams>,): number {
  /**
   Accessed property; its preceding token is the dot punctuator.
   */
  const { property, } = member;
  if (property === undefined) {
    throw new Error('member step missing property',);
  }
  /**
   `.` or `?.` token immediately before the property name.
   */
  const dot = context.sourceCode
    .getTokenBefore(property,);
  if (dot === null) {
    throw new Error('member step missing dot token',);
  }
  return dot.start;
}

/**
 Parameters for {@link operatorTokenStart}.
 */
type OperatorTokenStartParams = {
  /**
   Rule context; its `sourceCode` supplies the operator-token lookup.
   */
  readonly context: Context;
  /**
   `BinaryExpression` or `LogicalExpression` whose operator offset is wanted.
   */
  readonly node: ChainNode;
};

/**
 Returns the byte offset of an operator's token.
 
 The first token after the left operand whose value equals the operator skips
 any closing parenthesis of a parenthesised left operand and lands on the
 operator itself; its start is where the operator's continuation line begins,
 so the operator renders leading (`+ c`).
 
 @returns byte offset of the operator token
 
 @throws when the left operand, operator, or token is absent, which is
   unreachable for a binary or logical expression
 */
function operatorTokenStart({
  context,
  node,
}: ForeignBorrowed<OperatorTokenStartParams>,): number {
  /**
   Left operand; the operator token is the first matching token after it.
   */
  const {
    left,
    operator,
  } = node;
  if ((left === undefined) || (operator === undefined)) {
    throw new Error('operator node missing left operand or operator',);
  }
  /**
   First token after the left operand whose value is the operator literal.
   */
  const token = context.sourceCode
    .getTokenAfter(
    left,
    {
      filter: function matchesOperator(
        candidate: ForeignBorrowed<Comment | Token>,
      ): boolean {
        return candidate.value
          === operator;
      },
    },
    );
  if (token === null) {
    throw new Error('operator node missing operator token',);
  }
  return token.start;
}

/**
 Parameters for the chain-walk functions keyed on a single node.
 */
type ChainWalkParams = {
  /**
   Rule context; its `sourceCode` supplies token lookups during the walk.
   */
  readonly context: Context;
  /**
   Node currently being walked.
   */
  readonly node: ChainNode;
};

/**
 Returns the child a chain node descends into, or `undefined` for a leaf.
 
 Transparent wrappers descend through `.expression`; member access descends
 through `.object`; calls descend through `.callee`. Any other node, including
 `new` expressions and tagged templates, terminates the walk as the leaf.
 
 @param node - chain node whose receiver link is wanted
 
 @returns receiver to descend into, or {@link LEAF} when the node is the leaf
 */
function descentChild(node: ForeignBorrowed<ChainNode>,): ChainNode | typeof LEAF {
  if (isTransparentWrapper(node.type,))
    return node.expression ?? LEAF;
  if (node.type
    === 'MemberExpression')
    return node.object ?? LEAF;
  if (node.type
    === 'CallExpression')
    return node.callee ?? LEAF;
  return LEAF;
}

/**
 Returns the segment a chain node contributes after its receiver.
 
 A non-computed member access is a break point carrying its dot offset anchored
 at the owning member step's start, which on the receiver spine is the chain
 head; a computed access (`[expr]`, `?.[expr]`) and a call are attached; a
 transparent wrapper contributes nothing because its text rides on the prior
 segment.
 
 @returns trailing segment list (empty for transparent wrappers)
 */
function trailingStep({
  context,
  node,
}: ForeignBorrowed<ChainWalkParams>,): ChainSegment[] {
  if (node.type
    === 'MemberExpression') {
    return (node.computed
      === true)
      ? [ATTACHED,]
      : [
        {
          isBreak: true,
          breakOffset: memberDotStart({
            context,
            member: node,
          },),
          anchorOffset: node.start,
        },
      ];
  }
  if (node.type
    === 'CallExpression')
    return [ATTACHED,];
  return [];
}

/**
 Flattens a member/call chain into segments in source order, leaf first.
 
 Descends the receiver spine with a cursor via {@link descentChild}, stopping
 at the leaf or at a grouping-parenthesised receiver flagged by {@link parenIsolated}
 (treated as an opaque leaf), and collects each node that contributes a trailing
 step via {@link trailingStep}. The leaf supplies the head {@link ATTACHED}
 segment; the collected nodes supply their trailing steps in source order
 (innermost receiver first), so member-name steps are the only break points
 this produces. The walk is iterative: a member or call chain is a left-nested
 spine whose depth equals its length, so recursion here would be linear-depth
 and overflow on long chains.
 
 @returns segments in source order, the leaf at index 0
 
 @example
 ```ts
 // For `a.b.c`: [leaf a, break .b, break .c]
 chainSegments({ context, node, });
 ```
 */
export function chainSegments({
  context,
  node,
}: ForeignBorrowed<ChainWalkParams>,): ChainSegment[] {
  /**
   Nodes that each contribute a trailing step, outermost first; the leaf contributes none.
   */
  const contributors: ChainNode[] = [];
  for (
    let cursor: ChainNode = node;
    ;
  ) {
    /**
     Receiver to descend into; {@link LEAF} when the cursor is the chain leaf.
     */
    const child = descentChild(cursor,);
    if (child === LEAF) {
      break;
    }
    contributors.push(cursor,);
    if (parenIsolated({
      context,
      node: child,
    },)) {
      break;
    }
    cursor = child;
  }
  return [
    ATTACHED,
    ...contributors
      .toReversed()
      .flatMap(function step(contributor: ForeignBorrowed<ChainNode>,): ChainSegment[] {
        return trailingStep({
          context,
          node: contributor,
        },);
      },),
  ];
}

/**
 Parameters for {@link collectOperatorChain} and {@link operatorChainBreakOffsets}.
 */
type OperatorChainParams = {
  /**
   Rule context; its `sourceCode` supplies operator-token lookups.
   */
  readonly context: Context;
  /**
   Operator chain root (`BinaryExpression` or `LogicalExpression`).
   */
  readonly root: ChainNode;
};

/**
 Operands and operator breaks of an operator chain, gathered without order.
 
 The two axes are decoupled, so neither list's order matters: operand breaks
 and operator breaks are merged and sorted into source order downstream. Each
 operand is flattened on its own member axis; each operator break is a
 candidate operator-axis break point anchored at its operator node's start.
 */
type OperatorChainParts = {
  /**
   Operand subchains terminating the chain, each flattened independently.
   */
  readonly operands: readonly ChainNode[];
  /**
   Operator break candidates within the chain, root operator included; each
   anchors at its operator node's start.
   */
  readonly operatorBreaks: readonly ChainBreak[];
};

/**
 Gathers the operands and operator offsets of an operator chain iteratively.
 
 Descending both operands of every same-type unparenthesised operator flattens
 left- and right-associative runs alike. The root always continues into its
 operands, so its operator is recorded unconditionally; paren-isolation gates
 only the descendant operands, the boundary `no-mixed-operators` already
 inserts for mixed precedence. A grouping-parenthesised operand becomes one
 opaque operand. The walk uses an explicit work-stack rather than recursion
 because a left-associative operator chain is a left-nested spine whose depth
 equals operand count, so recursion would be linear-depth and overflow.
 
 @returns operands and operator breaks, neither in any guaranteed order
 
 @throws via {@link nonNullishOrThrow} when an operator node lacks an operand,
   which is unreachable for a binary or logical expression
 */
function collectOperatorChain({
  context,
  root,
}: ForeignBorrowed<OperatorChainParams>,): OperatorChainParts {
  /**
   Operand nodes terminating the chain.
   */
  const operands: ChainNode[] = [];
  /**
   Operator break candidates; the root operator is always present.
   */
  const operatorBreaks: ChainBreak[] = [
    {
      breakOffset: operatorTokenStart({
        context,
        node: root,
      },),
      anchorOffset: root.start,
    },
  ];
  /**
   Root operator type; same-type unparenthesised descendants continue the chain.
   */
  const { type: rootType, } = root;
  /**
   Pending nodes to classify; the root's two operands seed the walk.
   */
  const work: ChainNode[] = [
    nonNullishOrThrow(root.left,),
    nonNullishOrThrow(root.right,),
  ];
  while (work.length
    > 0) {
    /**
     Next pending node; the loop guard guarantees the stack is non-empty.
     */
    const node = nonNullishOrThrow(work.pop(),);
    /**
     Whether the node continues the chain rather than ending it as an operand.
     */
    const continues = (node.type
      === rootType)
      && (!parenIsolated({
        context,
        node,
      },));
    if (!continues) {
      operands.push(node,);
      continue;
    }
    operatorBreaks.push({
      breakOffset: operatorTokenStart({
        context,
        node,
      },),
      anchorOffset: node.start,
    },);
    work.push(
      nonNullishOrThrow(node.left,),
      nonNullishOrThrow(node.right,),
    );
  }
  return {
    operands,
    operatorBreaks,
  };
}

/**
 Returns the breaks for an operator chain root, in ascending source order.
 
 Gathers operands and operator breaks via {@link collectOperatorChain}.
 Decoupled axes: each operand is flattened on its own member axis and breaks
 only on its own member-step count via {@link selectBreaks}, while operators
 break on their own count. Neither axis inflates the other, so a single
 operator with a member operand (`a.b === c`) stays on one line. When any
 operand's member chain breaks, every operator also breaks onto its own line;
 otherwise the source-first operator stays on the head line and the rest break
 (two or more operators).
 
 @returns operator and member breaks merged in ascending break-offset order
 */
function operatorChainBreaks({
  context,
  root,
}: ForeignBorrowed<OperatorChainParams>,): readonly ChainBreak[] {
  /**
   Operands and operator breaks, neither ordered; the final sort imposes order.
   */
  const {
    operands,
    operatorBreaks,
  } = collectOperatorChain({
    context,
    root,
  },);
  /**
   Breaks contributed by the operands' own member subchains.
   */
  const operandBreaks = operands.flatMap(function operandBreaksOf(
    operand: ForeignBorrowed<ChainNode>,
  ): readonly ChainBreak[] {
    return selectBreaks(chainSegments({
      context,
      node: operand,
    },),);
  },);
  /**
   Whether any operand's member chain broke; forces every operator onto its own line.
   */
  const anyOperandBroke = operandBreaks.length
    > 0;
  /**
   Source-first operator break offset; it stays on the head line unless an operand broke.
   */
  const firstOperatorOffset = operatorBreaks.reduce(
    function smaller(
      smallest,
      operatorBreak,
    ): number {
      return (operatorBreak.breakOffset < smallest)
        ? operatorBreak.breakOffset
        : smallest;
    },
    Number.POSITIVE_INFINITY,
  );
  /**
   Operator breaks that begin a continuation line.
   */
  const breakingOperators = operatorBreaks.filter(function breaks(operatorBreak,): boolean {
    return anyOperandBroke
      || (operatorBreak.breakOffset !== firstOperatorOffset);
  },);
  return [
    ...operandBreaks,
    ...breakingOperators,
  ].toSorted(function ascending(
    first,
    second,
  ): number {
    return first.breakOffset
      - second.breakOffset;
  },);
}

/**
 Returns the breaks for any chain root, in ascending source order.
 
 Dispatches on the root kind: operator roots flatten across their operands on
 decoupled axes via {@link operatorChainBreaks}; member and call roots flatten
 down their receiver spine via {@link chainSegments}. Each branch returns
 breaks already in source order (member spines ascend by construction; operator
 chains sort their merged axes), ready for the renderer.
 
 @returns breaks in ascending break-offset order, empty when the chain fits on
   one line
 
 @example
 ```ts
 // For `arr.map(f).filter(g)`: [break for `.filter`]
 chainBreaks({ context, node, });
 ```
 
 @internal
 */
export function chainBreaks({
  context,
  node,
}: ForeignBorrowed<ChainWalkParams>,): readonly ChainBreak[] {
  if ((node.type
    === 'BinaryExpression') || (node.type
      === 'LogicalExpression')) {
    return operatorChainBreaks({
      context,
      root: node,
    },);
  }
  return selectBreaks(chainSegments({
    context,
    node,
  },),);
}
