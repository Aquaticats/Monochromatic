/**
 Bindings that can hold state reachable from one tracked call result.
 
 Split from `effect-result-escape.ts`, which collected only the identifier a call
 directly initializes while its own doc claimed holders "directly or by alias". One
 alias hop defeated it, and the discharge that rests on it produced an offer the rule's
 own fixer applies, that type-checks, and that rewrites the caller's state at runtime.
 The measurement is in `doc/planning/prefer-readonly-return-substitution.md`, section
 "A false offer on the structural path".
 
 The closure is call-specific on purpose. Deriving it from `bindingOriginBySymbolId`
 would make `const first = rows.at(0,)` and `const second = rows.at(1,)` share a verdict,
 because they share a `rows` origin while holding results of different calls.
 
 It is flow-insensitive, which is a deliberate over-approximation rather than an
 oversight. A binding reassigned away from the result stays a holder, and a transfer
 written before the one that establishes its source still propagates. Both directions
 only add holders, and adding a holder can only keep opacity, so the imprecision costs
 offers rather than soundness.
 
 @module
 */

import {
  type Node,
  SyntaxKind,
} from 'typescript/ast';
import {
  isArrayBindingPattern,
  isAssertionExpression,
  isBinaryExpression,
  isBindingElement,
  isConditionalExpression,
  isElementAccessExpression,
  isIdentifier,
  isNonNullExpression,
  isObjectBindingPattern,
  isOmittedExpression,
  isParenthesizedExpression,
  isPropertyAccessExpression,
  isSatisfiesExpression,
  isVariableDeclaration,
} from 'typescript/ast/is';
import type { Project, } from 'typescript/sync';

import {
  EITHER_OPERAND_PASSES,
  RIGHT_OPERAND_PASSES,
  targetIsCallableLocal,
} from './effect-value-consumer.ts';
import { collectAstNodes, } from './effect-summary-model.ts';
import { expressionCanCarryMutableState, } from './effect-primitive-origin.ts';

/**
 One place a value moves from an expression into a binding.
 */
type TransferSite = {
  /**
   Expression whose value is stored.
   */
  readonly source: Node;
  /**
   Binding name, binding pattern, or assignment target receiving that value.
   */
  readonly target: Node;
};

/**
 Descends one expression to the sub-expressions its own value can be.
 
 The descending mirror of `passesValueOutward` for operators, sharing its operator sets
 so the two directions cannot disagree about which operand an expression's value comes
 from. A mirror of the operators only: `provenanceSuccessors` in
 `effect-expression-provenance.ts` also descends object and array literals, and this
 walk does not, so `const box = { selected, }` never makes `box` a holder. That case
 fails closed elsewhere, because `useEscapes` counts a stored literal as escaping.
 
 @param node - Expression whose value sources are wanted.
 
 @returns strict descendants that can supply this expression's value.
 
 @example
 ```ts
 carrierSuccessors({ node: initializer });
 ```
 */
function carrierSuccessors({ node, }: { readonly node: Node; },): readonly Node[] {
  if (isParenthesizedExpression(node,)
    || isNonNullExpression(node,)
    || isAssertionExpression(node,)
    || isSatisfiesExpression(node,))
    return [node.expression,];
  if (isConditionalExpression(node,))
    return [
      node.whenTrue,
      node.whenFalse,
    ];
  if (!isBinaryExpression(node,))
    return [];
  /**
   Operator deciding which operands can be this expression's value.
   */
  const operator = node.operatorToken
    .kind;
  if (EITHER_OPERAND_PASSES.has(operator,))
    return [
      node.left,
      node.right,
    ];
  return RIGHT_OPERAND_PASSES.has(operator,) ? [node.right,] : [];
}

/**
 Tests whether an expression can hand on state reachable from the tracked result.
 
 A property or element projection qualifies only when the projected value can itself
 carry mutable state, so `const size = held.length` does not make `size` a holder while
 `const child = held.child` does.
 
 @param project - TypeScript project resolving symbols and types.
 
 @param source - Expression being stored.
 
 @param call - Call whose result is tracked.
 
 @param holders - Symbol ids already known to hold reachable state.
 
 @returns whether storing this expression can propagate the result.
 
 @example
 ```ts
 sourceCarriesResult({ project, source, call, holders });
 ```
 */
function sourceCarriesResult({
  project,
  source,
  call,
  holders,
}: {
  readonly project: Project;
  readonly source: Node;
  readonly call: Node;
  readonly holders: ReadonlySet<number>;
},): boolean {
  /**
   Expressions still to examine, each a strict descendant of one already seen.
   */
  const pending: Node[] = [source,];
  while (pending.length > 0) {
    /**
     Next expression whose value sources are examined.
     */
    const current = pending.pop();
    if (current === undefined)
      continue;
    if (current === call)
      return true;
    if (isIdentifier(current,)) {
      /**
       Symbol this identifier resolves to.
       */
      const symbol = project.checker
        .getSymbolAtLocation(current,);
      if ((symbol !== undefined) && holders.has(symbol.id,))
        return true;
      continue;
    }
    if (isPropertyAccessExpression(current,) || isElementAccessExpression(current,)) {
      if (expressionCanCarryMutableState({
        checker: project.checker,
        node: current,
      },))
        pending.push(current.expression,);
      continue;
    }
    pending.push(...carrierSuccessors({ node: current, },),);
  }
  return false;
}

/**
 Records one binding name as a holder when its type can carry mutable state.
 
 @param project - TypeScript project resolving the binding symbol.
 
 @param name - Identifier declaring or naming the binding.
 
 @param holders - Holder set being grown.
 
 @mutates holders - Adds the resolved symbol id.
 
 @returns whether the set gained an id it did not have.
 
 @example
 ```ts
 recordLeaf({ project, name, holders });
 ```
 */
function recordLeaf({
  project,
  name,
  holders,
}: {
  readonly project: Project;
  readonly name: Node;
  readonly holders: Set<number>;
},): boolean {
  if (!expressionCanCarryMutableState({
    checker: project.checker,
    node: name,
  },))
    return false;
  /**
   Symbol declared by this binding name.
   */
  const symbol = project.checker
    .getSymbolAtLocation(name,);
  if ((symbol === undefined) || holders.has(symbol.id,))
    return false;
  holders.add(symbol.id,);
  return true;
}

/**
 Records every leaf of one transfer target that can carry mutable state.
 
 Binding patterns are walked with a work stack rather than by recursion, per `ITR`:
 nesting is the only reason to descend and the pattern is a bounded structure.
 Renames, defaults and rest elements all reach the same leaf handling, because a
 `BindingElement` names its local binding through `name` in every one of those forms.
 Array elisions are `OmittedExpression` and bind nothing.
 
 @param project - TypeScript project resolving binding symbols.
 
 @param target - Binding name, binding pattern, or assignment target.
 
 @param holders - Holder set being grown.
 
 @mutates holders - Adds each qualifying leaf symbol id.
 
 @returns whether the set grew.
 
 @example
 ```ts
 recordTargetLeaves({ project, target: declaration.name, holders });
 ```
 */
function recordTargetLeaves({
  project,
  target,
  holders,
}: {
  readonly project: Project;
  readonly target: Node;
  readonly holders: Set<number>;
},): boolean {
  /**
   Targets still to examine, each a strict descendant of one already seen.
   */
  const pending: Node[] = [target,];
  /**
   Whether any leaf joined the holder set during this walk.
   */
  const growth: { any: boolean; } = { any: false, };
  while (pending.length > 0) {
    /**
     Next target whose leaves are recorded.
     */
    const current = pending.pop();
    if ((current === undefined) || isOmittedExpression(current,))
      continue;
    if (isIdentifier(current,)) {
      if (recordLeaf({
        project,
        name: current,
        holders,
      },))
        growth.any = true;
      continue;
    }
    if (isBindingElement(current,)) {
      /* A binding element without a name binds nothing this walk can follow, which
       * happens only for recovered syntax. Skipping it leaves the leaf unrecorded, and
       * an unrecorded leaf keeps opacity rather than discharging it.
       *
       * An object rest reaches this like any other element, and a narrowing that skipped
       * one whose type enumerated only primitives was reverted rather than repaired. A
       * TypeScript object type states which members a value must have and never which it
       * may have besides, so no reading of a rest's declared members establishes what the
       * rest copied. `leakExcessRestMember` in `readonly-structural-store-invalid.ts`
       * measures the offer that narrowing produced. */
      if (current.name !== undefined)
        pending.push(current.name,);
      continue;
    }
    if (isObjectBindingPattern(current,) || isArrayBindingPattern(current,))
      pending.push(...current.elements,);
  }
  return growth.any;
}

/**
 Collects the initialized declarations and direct local assignments in one body.
 
 Not every transfer, and the shortfall is deliberate rather than unnoticed. A
 declaration transfers into its name, which may be a pattern. An assignment transfers
 into its target only when that target is a plain identifier naming a binding local to
 this callable, so a property, an element, or an outer binding is left to
 `assignmentStoreEscapes`, which classifies each as a store this analysis cannot follow.
 
 The forms left out all fail closed, checked one at a time rather than assumed. A
 destructuring assignment target is not an identifier, so it reaches the same store
 classification. A compound or logical assignment operator is in neither operand set, so
 `useEscapes` receives the operand itself and falls through to escaping. An iteration
 binding has no initializer, so a holder used as the iterable reaches an unfamiliar
 consumer and escapes. Failing closed costs offers, which is the affordable direction.
 
 @param project - TypeScript project resolving target symbols.
 
 @param body - Body of the callable being analysed.
 
 @returns transfer sites in body order.
 
 @example
 ```ts
 transferSites({ project, body });
 ```
 */
function transferSites({
  project,
  body,
}: {
  readonly project: Project;
  readonly body: Node;
},): readonly TransferSite[] {
  return collectAstNodes(body,)
    .flatMap(function siteAt(node,): readonly TransferSite[] {
      if (isVariableDeclaration(node,))
        return node.initializer === undefined ? [] : [
          {
            source: node.initializer,
            target: node.name,
          },
        ];
      if ((!isBinaryExpression(node,))
        || (node.operatorToken
          .kind
          !== SyntaxKind.EqualsToken)
        || (!targetIsCallableLocal({
          project,
          target: node.left,
          body,
        },)))
        return [];
      return [
        {
          source: node.right,
          target: node.left,
        },
      ];
    },);
}

/**
 Collects the local bindings that can hold state reachable from one call's result.
 
 Least fixed point over the body's transfer sites, seeded with the call itself. It
 terminates because the candidate symbol universe inside a callable is finite and the
 set only ever grows, so every symbol can be inserted at most once.
 
 @param project - TypeScript project resolving symbols and types.
 
 @param call - Call whose result is tracked.
 
 @param body - Body of the callable containing the call.
 
 @returns symbol ids holding reachable state, empty when the result is never bound.
 
 @example
 ```ts
 resultReachableSymbolIds({ project, call, body });
 ```
 */
export function resultReachableSymbolIds({
  project,
  call,
  body,
}: {
  readonly project: Project;
  readonly call: Node;
  readonly body: Node;
},): ReadonlySet<number> {
  /**
   Symbol ids reachable from this call's result in the transfer graph.
   
   Membership is a graph relation over transfer sites, not a runtime fact. An alias
   overwritten before it escapes stays a member, a transfer written above the one that
   establishes its source still propagates, and an object rest of a primitive-only row
   qualifies on its type alone. Each of those keeps opacity for a callable that would
   have been offered soundly.
   */
  const holders = new Set<number>();
  /**
   Every transfer this body performs, examined repeatedly until nothing changes.
   */
  const sites = transferSites({
    project,
    body,
  },);
  /**
   Whether the last pass added a holder, which is what licenses another pass.
   */
  const pass: { grew: boolean; } = { grew: true, };
  while (pass.grew) {
    pass.grew = false;
    for (const site of sites) {
      if (!sourceCarriesResult({
        project,
        source: site.source,
        call,
        holders,
      },))
        continue;
      if (recordTargetLeaves({
        project,
        target: site.target,
        holders,
      },))
        pass.grew = true;
    }
  }
  return holders;
}
