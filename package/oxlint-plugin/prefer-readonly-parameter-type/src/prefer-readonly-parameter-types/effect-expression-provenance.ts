/**
 Which callable parameters one expression's value can be reached from.
 
 The single resolver every origin extractor delegates to. Before it existed,
 `expressionOrigins` and `rootParameterOrigins` each stripped property and element
 access down to an identifier and stopped, so they agreed only by coincidence and
 both stopped at a call. Having one definition is what lets a verified member
 result be followed to its receiver everywhere rather than in one extractor.
 
 @module
 */

import {
  type Node,
  SyntaxKind,
} from 'typescript/ast';
import {
  isArrayLiteralExpression,
  isAssertionExpression,
  isBinaryExpression,
  isCallExpression,
  isConditionalExpression,
  isElementAccessExpression,
  isIdentifier,
  isNonNullExpression,
  isObjectLiteralExpression,
  isParenthesizedExpression,
  isPropertyAssignment,
  isSatisfiesExpression,
  isShorthandPropertyAssignment,
  isSpreadAssignment,
  isSpreadElement,
  isThisExpression,
} from 'typescript/ast/is';
import type { Project, } from 'typescript/sync';

import {
  NOT_A_VERIFIED_READER,
  READER_RESULT_CARRIES_OPERAND,
  verifiedReaderCall,
} from './effect-default-library-reader-authority.ts';
import {
  callResultReceiver,
  RESULT_NOT_RECEIVER_STATE,
} from './effect-member-result-relation.ts';
import type { EffectSlot, } from './effect-slot-identity.ts';
import {
  containerElementReceiver,
  NOT_A_RECEIVER_CONTAINER,
} from './effect-container-element-origin.ts';
import {
  expressionCanCarryMutableState,
  receiverElementsArePrimitive,
} from './effect-primitive-origin.ts';
import {
  expressionRoot,
  NO_SLOT_ORIGIN,
  type SlotOrigins,
} from './effect-summary-model.ts';

/**
 Operators whose value may come from either operand.
 
 `??` is the one that matters most, and a resolver handling only calls would miss
 this package's own blocking shape: `target.get(key) ?? new Set()` is a
 `BinaryExpression`, so following calls alone never reaches the lookup. Measured
 more widely than that: `expressionRoot` strips property access but not a binary
 operator, so before this every alias established through `??` carried no origin at
 all, including `config.eviction ?? []` in `package/kv-store/core`.
 
 `||` belongs here too, since it yields its left operand whenever that is truthy,
 and a mutable object is truthy.
 
 `&&` deliberately does not, and that asymmetry is the point. It yields its left
 operand only when that operand is falsy, and no falsy value is a mutable object, so
 any object the expression produces came from the right operand. Following the left
 one could only over-attribute: `input && new Set()` would credit `input` for a
 `Set` that is always freshly built, and a false mutation record withholds a
 read-only offer the parameter deserves. `&&` is handled as right-operand-only in
 `provenanceSuccessors`.
 
 Arithmetic and comparison operators are absent because their result is a fresh
 primitive, which carries no state to attribute.
 */
const EITHER_OPERAND_OPERATORS: ReadonlySet<SyntaxKind> = new Set([
  SyntaxKind.QuestionQuestionToken,
  SyntaxKind.BarBarToken,
],);

/**
 Operators whose value is always their right operand's.
 
 `&&` for the truthiness reason recorded on `EITHER_OPERAND_OPERATORS`. Simple
 assignment because `holder = facts.get(key)` evaluates to what was assigned, so a
 mutation through the assignment expression's value reaches the right side. The comma
 operator discards its left operand outright.
 */
const RIGHT_OPERAND_OPERATORS: ReadonlySet<SyntaxKind> = new Set([
  SyntaxKind.AmpersandAmpersandToken,
  SyntaxKind.EqualsToken,
  SyntaxKind.CommaToken,
],);

/**
 Sentinel when an expression wraps nothing whose value it passes through.
 
 A sentinel rather than `undefined`, since this repo models absence without nullish
 unions.
 */
export const NOTHING_WRAPPED: unique symbol = Symbol(
  'expression passes through no operand value',
);

/**
 Expressions whose value is exactly their operand's.
 
 Every form here erases at runtime or passes its operand through unchanged, so the
 value that arrives is the operand's own. `as`, an angle-bracket assertion and `satisfies`
 matter as much as parentheses: `facts.get(key) as Set<string>` is the ordinary way
 to narrow a lookup, and treating it as opaque loses attribution for the whole
 expression.
 
 `await` is deliberately absent. Thenable assimilation means an awaited value need
 not be the operand's, so admitting it would assert an identity nothing here proves.
 
 @param node - Expression to unwrap.
 
 @returns inner expression, or sentinel when nothing is wrapped.
 
 @example
 ```ts
 transparentOperand({ node });
 ```
 */
export function transparentOperand(
  { node, }: { readonly node: Node; },
): Node | typeof NOTHING_WRAPPED {
  /**
   Whether this node's value is exactly its operand's.
   */
  const passesThrough = isParenthesizedExpression(node,)
    || isNonNullExpression(node,)
    || isAssertionExpression(node,)
    || isSatisfiesExpression(node,);
  return passesThrough
    ? node.expression
    : NOTHING_WRAPPED;
}

/**
 Operands one of which the expression's own value already is.
 
 The selection family, kept apart from the aggregate family below it because the two
 answer different questions. Everything here yields a value that some operand already
 held, either by erasing at runtime or by choosing between operands, so any walk asking
 "where did this value come from" inherits from these without needing a relation to
 explain the step. An aggregate is the opposite: `[a, b,]` builds a value no operand
 held, and crediting its contents is a separate claim about what a container packages.
 
 Exported because the element walk needs exactly this family and nothing else. Measured
 2026-08-07: `return (rows.slice(0,));` and `return cond ? rows.slice(0,) : [];` recorded
 no element origins while the bare `return (rows);` and `return cond ? rows : [];`
 recorded them correctly, because value provenance saw through the wrappers and the
 container relation did not. Sharing one definition is what stops the two walks
 disagreeing about identical state, which is the same hazard the spread rule on
 `isArrayLiteralExpression` was written to avoid.
 
 @param node - Expression whose selected operands are wanted.
 
 @returns operands whose value this expression's own may be.
 
 @example
 ```ts
 selectedOperandSuccessors({ node });
 ```
 */
export function selectedOperandSuccessors(
  { node, }: { readonly node: Node; },
): readonly Node[] {
  /**
   Operand of a wrapper that changes nothing about the value.
   */
  const unwrapped = transparentOperand({ node, },);
  if (unwrapped !== NOTHING_WRAPPED)
    return [unwrapped,];
  if (isConditionalExpression(node,))
    /* Both results and never the condition. The condition decides which value arrives
     * and is not itself a value that arrives, so descending into it would credit state
     * the expression cannot produce. */
    return [
      node.whenTrue,
      node.whenFalse,
    ];
  if (isBinaryExpression(node,)) {
    if (EITHER_OPERAND_OPERATORS.has(node.operatorToken
      .kind,))
      return [
        node.left,
        node.right,
      ];
    /* Right operand only, for the reason recorded on `EITHER_OPERAND_OPERATORS`: a
     * mutable object produced by `&&` or by an assignment can only be the right
     * operand's value. */
    if (RIGHT_OPERAND_OPERATORS.has(node.operatorToken
      .kind,))
      return [node.right,];
    return [];
  }
  return [];
}

/**
 Successor expressions whose origins the current expression inherits.
 
 Every returned node is a strict AST descendant of the input, which is what bounds
 the walk: no step can revisit an ancestor, so the stack drains without needing a
 visited set or a pass limit.
 
 @param project - TypeScript project proving default-library ownership.
 
 @param node - Expression whose value sources are wanted.
 
 @returns descendants contributing origins to this expression.
 
 @example
 ```ts
 provenanceSuccessors({ project, node });
 ```
 */
function provenanceSuccessors({
  project,
  node,
}: {
  readonly project: Project;
  readonly node: Node;
},): readonly Node[] {
  /**
   Operands this expression's value may already be, when it selects rather than builds.
   */
  const selected = selectedOperandSuccessors({ node, },);
  if (selected.length > 0)
    return selected;
  if (isObjectLiteralExpression(node,))
    /* An aggregate holds whatever was written into it, so a callee writing through one of its
     * properties writes into the value that property holds. The literal's own identity is fresh
     * and is not what is being credited here: this resolver answers which caller parameters an
     * expression's value can reach, and everything a literal packages is reachable through it.
     *
     * Without this a local written once, `const packaged = { named: first, }`, carried no origin
     * at all, so `callee(packaged,)` attributed the callee's write to nothing and offered the row
     * it mutates as read-only. `localLiteralProvenance` in the slot-narrowing fixture measures
     * it. The same values reached through a literal written directly at the call site were always
     * collected, by `parameterIndexes`, so the two paths disagreed about identical state.
     *
     * Property assignments, shorthand and spreads all contribute. A method or accessor does not:
     * what it can reach is a body rather than a value, which `packagedCallableOrigins` answers
     * for the argument walk and which this resolver has no node to hand back for. */
    return node.properties
      .flatMap(function packagedValue(property,): readonly Node[] {
        if (isPropertyAssignment(property,))
          return [property.initializer,];
        if (isShorthandPropertyAssignment(property,))
          return [property.name,];
        if (isSpreadAssignment(property,))
          return [property.expression,];
        return [];
      },);
  if (isArrayLiteralExpression(node,))
    /* Elements for the same reason, with a spread contributing what it spreads.
     *
     * A spread of a container whose elements are all primitive contributes nothing, which is the
     * same rule `parameterIndexes` applies to a spread argument and is applied here so the two
     * walks cannot disagree about identical state. `[...values,]` over a `readonly string[]`
     * builds a fresh array of primitives that shares no object with `values`, so crediting
     * `values` reports a parameter nothing can reach. Measured on `copiedPrimitiveArray` in
     * `readonly-catalog-free-invalid.ts`, whose doc comment states exactly that intent. */
    return node.elements
      .flatMap(function packagedElement(element,): readonly Node[] {
        if (!isSpreadElement(element,))
          return [element,];
        /**
         Type of the container being spread, when the checker resolves one.
         */
        const spreadType = project.checker
          .getTypeAtLocation(element.expression,);
        if ((spreadType !== undefined)
          && receiverElementsArePrimitive({
            checker: project.checker,
            type: spreadType,
          },))
          return [];
        /* Spreading is the fourth element-step spelling. `[...rows.slice()]` builds an array
         * holding the receiver's own rows, and the spread expression's own value carries
         * nothing, so following it as a value loses them exactly as iteration did. Both are
         * returned rather than one: the value answers when a parameter is spread directly,
         * and the receiver answers when a fresh container is. */
        /**
         Receiver whose elements the spread container holds, when that is verified.
         */
        const elementReceiver = containerElementReceiver({
          project,
          checker: project.checker,
          node: element.expression,
        },);
        return (elementReceiver === NOT_A_RECEIVER_CONTAINER)
          ? [element.expression,]
          : [
            element.expression,
            elementReceiver,
          ];
      },);
  if (!isCallExpression(node,))
    return [];
  /* Calling a callable the caller supplied. Whatever comes back was chosen by the
   * caller's function, so it may be caller-owned state, and the callee identifier is the
   * only handle on it this analysis has: a caller maps that parameter to whatever it
   * packaged there. Restricted to an identifier callee on purpose. A member callee would
   * make every `values.map(fn)` credit `values` for a container the member freshly
   * allocated, which is the distinction `FRESH_CONTAINER_MEMBER_NAMES` exists to keep.
   * An identifier that holds no parameter origin contributes nothing, so a call of a
   * local or imported function is unaffected. Measured on `callThroughMethodResult` in
   * the result-provenance fixture, which writes `get().label`. */
  if (isIdentifier(node.expression,))
    return [node.expression,];
  /* A call contributes its receiver only when the result authority verifies that its
   * result is state the receiver held. Absent that, a call is where provenance stops:
   * the result is either fresh or unproven, and neither may be credited. */
  /**
   Checker for the project resolving this call.
   */
  const { checker, } = project;
  /* A verified reader's result holds the values it read, so provenance runs to the value
   * it was given rather than to its receiver, which is a global. */
  /**
   Declaration this call resolves to, when one does.
   */
  const readerDeclaration = checker.getResolvedSignature(node,)
    ?.declaration
    ?.resolve(project,);
  if (readerDeclaration !== undefined) {
    /**
     Verified reader and the value it reads, when this call is one.
     */
    const reader = verifiedReaderCall({
      project,
      checker,
      call: node,
      declaration: readerDeclaration,
    },);
    if ((reader !== NOT_A_VERIFIED_READER)
      && (reader.resultRelation === READER_RESULT_CARRIES_OPERAND))
      return [reader.operand,];
  }
  /**
   Receiver whose state this call's result is verified to be, when any.
   */
  const receiver = callResultReceiver({
    project,
    checker,
    call: node,
  },);
  return receiver === RESULT_NOT_RECEIVER_STATE
    ? []
    : [receiver,];
}

/**
 Resolves every callable parameter one expression's value can be reached from.
 
 Walks the expression with an explicit stack rather than recursion, per `ITR`: the
 shape being followed is a spine of receivers and operands, and branch operators
 make it a small tree, so the stack both flattens it and unions the branches.
 
 @param project - TypeScript project resolving symbols and signatures.
 
 @param bindingOriginBySymbolId - Known parameter and alias origins.
 
 @param node - Expression whose value provenance is wanted.
 
 @returns every parameter origin the value can carry, empty when none.
 
 @example
 ```ts
 expressionValueOrigins({ project, bindingOriginBySymbolId, node });
 ```
 */
export function expressionValueOrigins({
  project,
  bindingOriginBySymbolId,
  node,
}: {
  readonly project: Project;
  readonly bindingOriginBySymbolId: ReadonlyMap<number, SlotOrigins>;
  readonly node: Node;
},): SlotOrigins {
  /**
   Origins accumulated across every branch reached.
   */
  const origins = new Set<EffectSlot>();
  /**
   Expressions still to examine, mostly but not only descendants of one already seen.
   
   The qualification is the point, and the flat claim that stood here was wrong. Successors
   are descendants and `provenanceSuccessors` says so, but the element-access and spread
   branches also queue a `containerElementReceiver` result, and that follows a name to its
   declaration initializer, which is anywhere in the file. So the descendant argument
   covers most of what is queued and not all of it, and this walk keeps no visited set.
   
   Probed rather than left as a worry, on five self-referential and mutually referential
   container declarations, including `const a = [...b]; const b = [...a];` and
   `const a = [a[0]!];`. None failed to terminate. Two exhaust the stack instead, inside
   the checker's own member aggregation rather than here, and `effect-demand-index` catches
   that, logs the omission and leaves the callable without a summary, which the unresolved
   boundary then withholds on. Degraded and safe rather than unsound.
   
   A visited set is therefore not added here. It would cost a set operation per node on one
   of the hottest walks in the rule, against a cycle nothing has produced.
   */
  const pending: Node[] = [node,];
  while (pending.length > 0) {
    /**
     Next expression whose value sources are examined.
     */
    const current = pending.pop();
    if (current === undefined)
      continue;
    /* An element step is where a container's two answers separate. `copy.push(row)` reaches
     * the container, whose identity is fresh and shares nothing with the caller, while
     * `copy[0].label = 'x'` reaches an element, which is the receiver's own row. Stripping
     * both to `copy` and asking one question about it cannot answer them differently, so the
     * element step is answered here, before the strip, and only for a container whose
     * relation is verified.
     *
     * Nothing is queued when the relation is unproven, which leaves the walk exactly as it
     * was: a fresh container of unknown provenance still contributes no origin, and the
     * receiver's own opacity is what withholds the offer until it is discharged. */
    if (isElementAccessExpression(current,)) {
      /**
       Receiver whose elements this container holds, when that is verified.
       */
      const elementReceiver = containerElementReceiver({
        project,
        checker: project.checker,
        node: current.expression,
      },);
      if (elementReceiver !== NOT_A_RECEIVER_CONTAINER) {
        pending.push(elementReceiver,);
        continue;
      }
    }
    /**
     Root after property and element access removal.
     */
    const root = expressionRoot(current,);
    /* A `this` expression is not an identifier, so this branch skipped it and the walk fell
     * through to a successor lookup with nothing to hand back. A callable declaring an explicit
     * `this` formal therefore recorded no write for `this.label = 'written'` at all, which
     * offers a row the callable mutates. `writeThroughThis` in the slot-narrowing fixture
     * measured it.
     *
     * Nothing else was missing. The checker resolves a `this` expression to the same symbol as
     * the `this` parameter's name, measured as symbol identity rather than assumed, and the
     * parameter seeding already registers that symbol against parameter zero. Only the gate in
     * front of the lookup had to widen. */
    if (isIdentifier(root,) || isThisExpression(root,)) {
      /**
       Symbol the root identifier or `this` expression resolves to.
       
       A shorthand property's name resolves to the property rather than to the local it
       reads, so the value symbol has to be asked for separately. Every other walk in this
       package already does: `packagedCallableOrigins`, `parameterIndexes` and the
       `ForeignBorrowed` classifier. This one did not, and the object-literal branch above
       hands it exactly that node, so a returned `{ slice }` recorded no origin while
       `{ slice: slice }` recorded one. Measured: a caller writing through the returned
       object was attributed nothing and kept its read-only offer, while the identical
       write through the explicit form reported the mutation.
       */
      const symbol = isShorthandPropertyAssignment(root.parent,)
        && (root.parent
          .name
          === root)
        ? project.checker
          .getShorthandAssignmentValueSymbol(root.parent,)
        : project.checker
          .getSymbolAtLocation(root,);
      /**
       Origins already recorded for this binding.
       */
      const known = symbol === undefined
        ? NO_SLOT_ORIGIN
        : bindingOriginBySymbolId.get(symbol.id,) ?? NO_SLOT_ORIGIN;
      known.forEach(function collectKnown(origin,): void {
        origins.add(origin,);
      },);
      continue;
    }
    /* A successor that cannot carry mutable identity contributes no origin, however it was
     * derived from a parameter. `{ chars: slice.targetChars }` packages a number, and the
     * walk reached `slice` from it only because `expressionRoot` strips the property access
     * back to the receiver, which answers "what was read" rather than "what can be reached".
     * Measured: the fresh object recorded the callback parameter as a returned origin, so a
     * caller could not tell it apart from `row => row`, which is exactly the distinction the
     * result-provenance decision rests on.
     *
     * This is type evidence used in the one direction the decision permits. A type may prove
     * a value carries no mutable identity, and `typeCanCarryMutableState` fails closed for
     * `any` and `unknown`. It may never prove that a mutable value is fresh, which stays a
     * provenance question and is why this prunes leaves rather than deciding results.
     *
     * The sibling walks already do this at their own boundaries: `packagedCallableOrigins`
     * skips a binding that cannot carry state, the array-literal branch above drops a spread
     * of primitives, and `recordReturnEffects` gates the whole returned expression the same
     * way. Doing it here is what keeps them agreeing on identical state. */
    provenanceSuccessors({
      project,
      node: root,
    },)
      .filter(function carriesIdentity(successor,): boolean {
        return expressionCanCarryMutableState({
          checker: project.checker,
          node: successor,
        },);
      },)
      .forEach(function queueSuccessor(successor,): void {
        pending.push(successor,);
      },);
  }
  return origins.size === 0
    ? NO_SLOT_ORIGIN
    : origins;
}
