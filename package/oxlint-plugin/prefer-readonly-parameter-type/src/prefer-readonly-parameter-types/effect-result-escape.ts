/**
 Whether a tracked call result can leave the callable that produced it.
 
 This is the condition that licenses discharging a verified member call's receiver
 opacity. The opacity report exists because nothing tracked the result as an alias;
 once provenance tracks it, the report is redundant exactly while every use of the
 result is one the analysis attributes. A use that leaves the callable is not, so the
 report has to stay for those.
 
 Returning parameter-reachable state is benign by accepted policy, since the caller
 already holds the parameter. That policy is about the callee not being blamed, not
 about the value becoming untracked: until a caller substitutes through
 `directReturned`, a returned result is still a use this analysis cannot follow, so it
 counts as an escape here.
 
 @module
 */

import {
  type CallExpression,
  type Node,
  SyntaxKind,
} from 'typescript/ast';
import {
  isArrayLiteralExpression,
  isBinaryExpression,
  isBindingElement,
  isCallExpression,
  isElementAccessExpression,
  isExpressionStatement,
  isForOfStatement,
  isIdentifier,
  isObjectLiteralExpression,
  isPrefixUnaryExpression,
  isPropertyAccessExpression,
  isReturnStatement,
  isSpreadElement,
  isTypeOfExpression,
  isVariableDeclaration,
  isVoidExpression,
  isYieldExpression,
} from 'typescript/ast/is';
import type { Project, } from 'typescript/sync';

import {
  assignmentStoreEscapes,
  isPresentNode,
  valueConsumer,
} from './effect-value-consumer.ts';
import { collectAstNodes, } from './effect-summary-model.ts';
import { resultReachableSymbolIds, } from './effect-result-holders.ts';
import { writtenDirectlyInBody, } from './effect-enclosing-callable.ts';

/**
 Binary operators that only test a value and keep no reference to it.
 */
const TESTING_OPERATORS: ReadonlySet<SyntaxKind> = new Set([
  SyntaxKind.EqualsEqualsEqualsToken,
  SyntaxKind.ExclamationEqualsEqualsToken,
  SyntaxKind.EqualsEqualsToken,
  SyntaxKind.ExclamationEqualsToken,
  SyntaxKind.InKeyword,
  SyntaxKind.InstanceOfKeyword,
  SyntaxKind.LessThanToken,
  SyntaxKind.GreaterThanToken,
  SyntaxKind.LessThanEqualsToken,
  SyntaxKind.GreaterThanEqualsToken,
],);

/**
 Prefix operators that only test a value and keep no reference to it.
 */
const TESTING_PREFIX_OPERATORS: ReadonlySet<SyntaxKind> = new Set([
  SyntaxKind.ExclamationToken,
],);

/**
 Tests whether a literal is handed directly to a call as an argument.
 
 Nesting is allowed, because `parameterIndexes` walks nested object and array
 literals and spreads alike, so a value inside `{ options: { target } }` reaches the
 same argument analysis as one inside `{ target }`.
 
 @param literal - Object or array literal holding a tracked value.
 
 @returns whether the literal, or a literal enclosing it, is a call argument.
 
 @example
 ```ts
 literalIsCallArgument({ literal });
 ```
 */
function literalIsCallArgument(
  { literal, }: { readonly literal: Node; },
): boolean {
  /**
   Cursor ascending through enclosing literals to their consumer.
   */
  const cursor: { current: Node; } = { current: literal, };
  while (isEnclosingLiteral({ node: cursor.current, },)) {
    cursor.current = cursor.current
      .parent;
  }
  /**
   Expression consuming the outermost enclosing literal.
   */
  const consumer = valueConsumer({ node: cursor.current, },);
  /**
   Call receiving that literal, when one does.
   */
  const { parent, } = consumer;
  if (!isCallExpression(parent,))
    return false;
  return parent.arguments
    .some(function isThisArgument(argument,): boolean {
      return argument === consumer;
    },);
}

/**
 Tests whether a node's parent is itself an object or array literal.
 
 @param node - Literal whose enclosing literal is sought.
 
 @returns whether another literal encloses this one.
 
 @example
 ```ts
 isEnclosingLiteral({ node: literal });
 ```
 */
function isEnclosingLiteral({ node, }: { readonly node: Node; },): boolean {
  /**
   Context possibly nesting this literal inside another.
   */
  const { parent, } = node;
  return isObjectLiteralExpression(parent,) || isArrayLiteralExpression(parent,);
}

/**
 Ascends spread-into-array-literal steps to the literal that carries the value.
 
 `[...pairs.entries(),].flatMap(compare)` puts one collection's elements into a literal
 whose own position decides everything: as a call's receiver or argument the obligation
 transfers to that call, and stored or returned it leaves. Classifying the spread instead
 asks about a node whose parent is always a literal, which answers "is that literal an
 argument", and for a literal used as a receiver that is no.
 
 Starts from the node itself rather than its parent, because `valueConsumer` already
 returns the spread element. Ascending from the parent instead looks one node too high,
 finds no spread, and changes nothing, which is exactly what a first attempt at this
 measured.
 
 Iterative because the step composes: `[...[...pairs,],]` is two hops asking the identical
 question one node further out.
 
 @param node - Expression possibly sitting inside one or more spreads.
 
 @returns outermost literal carrying this value, or node itself when it is not spread.
 
 @example
 ```ts
 spreadCarrier({ node: spreadElement });
 ```
 */
function spreadCarrier({ node, }: { readonly node: Node; },): Node {
  /**
   Value reached so far while ascending spread steps.
   */
  let carried = node;
  while (isSpreadElement(carried,) && isArrayLiteralExpression(carried.parent,))
    carried = carried.parent;
  return carried;
}

/**
 Tests whether a node sits in a position this analysis cannot follow.
 
 Attributed positions are deliberately enumerated rather than inferred from what is
 left over, so an unfamiliar construct counts as an escape by default.
 
 @param node - Expression whose enclosing use is classified.
 
 @param elementStepsAttributed - Whether a caller walks this result's elements.
 
 @param returnsAttributed - Whether returning counts as followed rather than as leaving.
 
 @param body - Body whose own returns may be attributed, absent when none may be.
 
 @returns whether the value at this position leaves attributed tracking.
 
 @example
 ```ts
 useEscapes({ node: identifier, elementStepsAttributed, returnsAttributed });
 ```
 */
function useEscapes({
  node,
  elementStepsAttributed,
  returnsAttributed,
  body,
}: {
  readonly node: Node;
  readonly elementStepsAttributed: boolean;
  readonly returnsAttributed: boolean;
  readonly body?: Node;
},): boolean {
  /* A spread carries its operand's elements into the enclosing array literal, so the
   * question is about where that literal goes rather than about the spread. Gated with
   * the iterated position below: only a caller that walks elements may treat reaching one
   * as attributed. */
  /**
   Value whose position decides this, with spread-into-literal steps ascended.
   */
  const carried = elementStepsAttributed
    ? spreadCarrier({ node, },)
    : node;
  /**
   Syntactic context consuming this value.
   */
  const { parent, } = carried;
  if (isYieldExpression(parent,))
    return true;
  if (isReturnStatement(parent,)) {
    /* Returning is the one escape whose destination this analysis can follow, and only a
     * caller asking for that may treat it as followed. `yield` stays an escape beside it: a
     * generator's consumer is not enumerable the way a call site is.
     *
     * The return must belong to the body being scanned. A `return` written inside a nested
     * declaration hands the value to whoever calls *that*, and this scan's callers are the
     * outer callable's, so attributing it would credit substitution nobody performs. */
    return !(returnsAttributed
      && (body !== undefined)
      && writtenDirectlyInBody({
        node: parent,
        body,
      },));
  }
  /* The other element step reaching a value without writing an access node.
   * `containerElementWriteEffect` consumes its container through `copy[0]`, which the
   * access branch below already answers, while `iteratedContainerWriteEffect` reaches the
   * same elements through `for...of` and was answered as leaving, so a container that
   * never leaves reported anyway.
   *
   * Gated rather than unconditional, because the position is only attributed where
   * something walks the elements. `effect-call-analysis.ts` asks the same question about a
   * call result reaching an argument, and there nothing does: widening it globally cleared
   * an argument-side obligation that arrives by propagation from a callee, measured on
   * `formatUsageWarningStatus`. */
  if (elementStepsAttributed
    && isForOfStatement(parent,)
    && (parent.expression === carried))
    return false;
  /* Placed in an object or array literal. Whether that is an escape depends entirely
   * on where the literal goes, and getting this wrong defeats discharge throughout
   * this repository: `ST9` makes every multi-argument call pass one object literal, so
   * treating literal membership as an escape outright leaves almost nothing
   * dischargeable. A literal handed straight to a call is walked by
   * `parameterIndexes`, which collects the origins of its properties and shorthand
   * values, so the obligation transfers to that call exactly as a direct argument
   * does. A literal that is stored or bound is not walked by anything, so the value
   * inside it becomes untracked and does escape. */
  if (isObjectLiteralExpression(parent,) || isArrayLiteralExpression(parent,))
    return !literalIsCallArgument({ literal: parent, },);
  /* Assignment is deliberately absent here and cannot be added back. Every caller hands
   * this predicate `valueConsumer` of a node, and that ascends the right operand of an
   * assignment, so a node contributing to one never arrives. `assignmentStoreEscapes` in
   * `effect-value-consumer.ts` classifies the store during the ascent instead, which is
   * the only position where both the store and the assignment's own consumer are visible. */
  /* Reached as the object of a property or element access: attributed through
   * provenance, which follows the access down to this value's own origins. */
  if (isPropertyAccessExpression(parent,) || isElementAccessExpression(parent,))
    return false;
  if (isVariableDeclaration(parent,))
    return false;
  /* Only tested, never retained. An equality or relational comparison, `in`,
   * `instanceof`, `typeof`, `void` and logical negation all produce a fresh primitive
   * and keep no reference, so the value cannot leave through them. Without this,
   * the ubiquitous `if (stored === undefined) return;` guard counted as an escape and
   * nothing behind a null check could ever discharge. */
  if (isBinaryExpression(parent,)
    && TESTING_OPERATORS.has(parent.operatorToken
      .kind,))
    return false;
  if (isPrefixUnaryExpression(parent,)
    && TESTING_PREFIX_OPERATORS.has(parent.operator,))
    return false;
  if (isTypeOfExpression(parent,) || isVoidExpression(parent,))
    return false;
  /* Discarded outright. A value used as an expression statement is not stored, tested
   * or passed anywhere, so it cannot leave: `facts.get(key)?.add('x')` evaluates for
   * the mutation alone and drops the result. Without this, every chained mutation on a
   * looked-up value counted as an escape. */
  if (isExpressionStatement(parent,))
    return false;
  /* Passed as a call argument. The argument analysis already resolves each argument's
   * origins and records opacity against them when the callee is unresolved, so the
   * obligation has moved to that sink rather than disappearing. This is the one case
   * where "not escaping" means "reported elsewhere" instead of "fully attributed". */
  if (isCallExpression(parent,)) {
    /**
     Whether this value is one of the call's arguments.
     */
    const isArgument = parent.arguments
      .some(function isThisArgument(argument,): boolean {
        return argument === node;
      },);
    if (isArgument)
      return false;
  }
  return true;
}

/**
 Tests whether an identifier occurrence establishes a binding rather than using it.
 
 Once the holder set follows aliases, the bindings it collects have declaration and
 assignment-target occurrences of their own, and those are not uses of the result. A
 destructured leaf's identifier has a `BindingElement` parent, which no attributed
 position matches, so it would fall through to escaping and make every destructuring
 declaration report. An assignment target's identifier has a binary-expression parent
 whose operator is not a testing operator, so it would report every assignment alias.
 Neither occurrence reads the value, so both are skipped before classification.
 
 @param node - Identifier occurrence resolving to a holder.
 
 @returns whether this occurrence declares or overwrites rather than reads.
 
 @example
 ```ts
 occurrenceEstablishesBinding({ node: identifier });
 ```
 */
function occurrenceEstablishesBinding({ node, }: { readonly node: Node; },): boolean {
  /**
   Syntactic context this occurrence sits in.
   */
  const { parent, } = node;
  if (isVariableDeclaration(parent,) || isBindingElement(parent,))
    return parent.name === node;
  return isBinaryExpression(parent,)
    && (parent.operatorToken
      .kind
      === SyntaxKind.EqualsToken)
    && (parent.left === node);
}

/**
 Tests whether a verified call's result can leave the callable body.
 
 @param project - TypeScript project resolving symbols.
 
 @param body - Body of the callable containing the call.
 
 @param call - Verified member call whose result is tracked.
 
 @returns whether any use of the result is one this analysis cannot follow.
 
 @example
 ```ts
 resultEscapesCallable({ project, body, call });
 ```
 */
export function resultEscapesCallable({
  project,
  body,
  call,
  elementStepsAttributed,
  returnsAttributed = false,
}: {
  readonly project: Project;
  readonly body: Node;
  readonly call: CallExpression;
  readonly elementStepsAttributed: boolean;
  readonly returnsAttributed?: boolean;
},): boolean {
  /* The call's own position first. A call whose result is returned outright, or
   * placed straight into a container, escapes without ever being bound. */
  if (useEscapes({
    node: valueConsumer({ node: call, },),
    elementStepsAttributed,
    returnsAttributed,
    body,
  },))
    return true;
  /* Then any store performed on the way to that position. `sink.value = facts.get(k)`
   * consumes the assignment expression as a discarded statement, so the position test
   * above answers about the assignment rather than about the store beneath it. */
  if (assignmentStoreEscapes({
    project,
    node: call,
    body,
  },))
    return true;
  /**
   Bindings that hold state reachable from this call's result.
   */
  const holders = resultReachableSymbolIds({
    project,
    call,
    body,
  },);
  if (holders.size === 0)
    /* Not bound and not escaping by position: the result is consumed in place, which
     * the enumerated attributed positions already cover. */
    return false;
  return collectAstNodes(body,)
    .some(function referenceEscapes(node,): boolean {
      if ((!isIdentifier(node,)) || occurrenceEstablishesBinding({ node, },))
        return false;
      /**
       Symbol this identifier resolves to.
       */
      const symbol = project.checker
        .getSymbolAtLocation(node,);
      if ((symbol === undefined) || (!holders.has(symbol.id,)))
        return false;
      /* A reference inside a nested callable outlives this scan's reasoning about
       * order, so treat any captured use as escaping. */
      return enclosedByNestedCallable({
        node,
        body,
      },)
        || useEscapes({
          node: valueConsumer({ node, },),
          elementStepsAttributed,
          returnsAttributed,
          body,
        },)
        || assignmentStoreEscapes({
          project,
          node,
          body,
        },);
    },);
}

/**
 Tests whether a node sits inside a callable nested under the given body.
 
 The ascent itself moved to `effect-enclosing-callable.ts`, where the returned-result
 discharge asks the same containment question for a different reason. Two walks answering
 one question is what `AGENTS.md` calls a shared definition, and keeping them apart had
 already let them disagree about the case neither reaches: this one answered "not nested"
 when it ran off the root, which is the permissive direction, while the discharge needs the
 conservative one. Unified on the conservative answer, since the note this replaces states
 that every caller passes a node inside `body` and so neither can observe the difference.
 
 @param node - Reference being classified.
 
 @param body - Outer callable body boundary.
 
 @returns whether a nested callable encloses the reference.
 
 @example
 ```ts
 enclosedByNestedCallable({ node, body });
 ```
 */
function enclosedByNestedCallable({
  node,
  body,
}: {
  readonly node: Node;
  readonly body: Node;
},): boolean {
  return !writtenDirectlyInBody({
    node: node.parent,
    body,
  },);
}
