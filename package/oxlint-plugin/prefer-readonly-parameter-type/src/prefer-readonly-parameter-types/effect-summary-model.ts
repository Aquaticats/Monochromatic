/**
 Shared data model and AST utilities for callable effect summaries.
 
 @module
 */

import type {
  CallSignatureDeclaration,
  ConstructorTypeNode,
  ConstructSignatureDeclaration,
  FunctionLikeDeclaration,
  FunctionTypeNode,
  MethodSignatureDeclaration,
  Node,
} from 'typescript/ast';
import type {
  EffectSlot,
  ParameterIndex,
} from './effect-slot-identity.ts';
import {
  isCallSignatureDeclaration,
  isConstructorTypeNode,
  isConstructSignatureDeclaration,
  isElementAccessExpression,
  isFunctionLikeDeclaration,
  isFunctionTypeNode,
  isMethodSignatureDeclaration,
  isPropertyAccessExpression,
} from 'typescript/ast/is';

/**
 Sentinel when expression root does not resolve to a callable slot.
 */
export const EFFECT_SLOT_UNAVAILABLE: unique symbol = Symbol(
  'expression root lacks callable effect slot',
);

/**
 Every callable slot one binding can hold, empty when none.
 
 A local reassigned across branches holds state from more than one slot, so a
 single slot cannot describe it. Emptiness replaces the sentinel here rather than
 joining it in a union, because a set already distinguishes "no origin" from "some
 origin" without a second representation of absence.
 */
export type SlotOrigins = ReadonlySet<EffectSlot>;

/**
 Shared empty result for expressions rooted outside callable parameters.
 
 Most identifiers in a body resolve to no slot, so this is returned far more
 often than any populated set and sharing one instance avoids allocating per node.
 Safe only while `SlotOrigins` stays read-only at every boundary: an assertion
 back to `Set<EffectSlot>` anywhere would let one caller poison every other.
 */
export const NO_SLOT_ORIGIN: SlotOrigins = new Set<EffectSlot>();

/**
 Sentinel when semantic call target has no owned callable declaration.
 */
export const OWNED_CALLABLE_UNAVAILABLE: unique symbol = Symbol(
  'call target lacks owned callable declaration',
);

/**
 Callable implementation or bodyless source signature covered by effect contract.
 */
export type EffectCallableDeclaration =
  | FunctionLikeDeclaration
    | CallSignatureDeclaration
    | ConstructSignatureDeclaration
    | MethodSignatureDeclaration
    | FunctionTypeNode
    | ConstructorTypeNode;

/**
 Tests whether node participates in callable effect contract.
 
 @param node - TypeScript AST node to classify.
 
 @returns whether node is callable implementation or supported signature.
 
 @example
 ```ts
 if (isEffectCallableDeclaration(node)) {
   node.parameters;
 }
 ```
 */
export function isEffectCallableDeclaration(node: Node,): node is EffectCallableDeclaration {
  return isFunctionLikeDeclaration(node,)
    || isCallSignatureDeclaration(node,)
    || isConstructSignatureDeclaration(node,)
    || isMethodSignatureDeclaration(node,)
    || isFunctionTypeNode(node,)
    || isConstructorTypeNode(node,);
}

/**
 One callback-parameter relation inferred from owned function body.
 
 `callbackSlot` and `sourceSlot` name slots of the callable that declared the relation.
 `callbackArgumentPosition` is neither: it is the syntactic position of an argument at the
 inner invocation, read against the callback's own summary by the consumer. Keeping the
 three names distinct is deliberate, since all three were `number` and the last one is the
 odd one out.
 */
export type CallbackRelation = {
  readonly callbackSlot: EffectSlot;
  readonly callbackArgumentPosition: number;
  readonly sourceSlot: EffectSlot;
};

/**
 One caller-supplied observer reaching state behind a read-only view receiver.
 
 Recorded when a call satisfies the receiver-structure claim: the member
 belongs to a default-library read-only view, so it cannot restructure the
 receiver, and the only remaining question is what the observer does with the
 receiver state handed to it.
 */
export type ElementApplication = {
  readonly receiverSlot: EffectSlot;
  readonly callbackKey: string;
  readonly observerParameterIndexes: readonly ParameterIndex[];
};

/**
 What a caller does with the result of one owned call, before its origins are known.
 
 The syntax pass can see that a result is mutated or handed back, and cannot see which
 caller parameters that result carries, because a callee's summary does not exist while
 its callers are being walked. Recording the use and deferring the origins is what lets
 the two meet in propagation, where the callee summary and the edge's formal-to-actual
 mapping sit together.
 
 Keyed by call site rather than by callee, because two calls of the same callee in one
 body are separate uses and must not share a verdict.
 */
export type ResultApplication =
  | {
    readonly callSiteKey: string;
    readonly kind: 'mutated' | 'returned';
    readonly provenance?: never;
  }
    | {
    /**
     A result handed to something outliving the call.
     
     Provenance is required here and forbidden on the other two kinds, which is why this
     is a union rather than one shape with an optional field. The retention channel is
     the one that names where a value went, and the diagnostic reads that text to decide
     whether a boundary is a store or a call. A retained application with nothing to say
     would arrive as an unexplained opaque slot and be reported as an unresolved effect
     addressed to an unresolved implementation, which is the exact confusion
     `effect-retention-provenance.ts` exists to prevent.
     
     Written as a union after the optional-field spelling let `{ kind: 'retained' }`
     type-check, leaving a runtime throw as the only thing standing between a
     well-typed literal and a crash inside the fixed point.
     */
      readonly callSiteKey: string;
      readonly kind: 'retained';
      readonly provenance: string;
    };

/**
 One owned call edge with caller-relative argument roots.
 
 Three of these arrays used to share one formal-parameter index. Slots split them, because
 propagation reads a callee's effect set and that set now names slots, while foreign
 ownership is a marker on a whole parameter and has no property-level meaning. Each array
 therefore says in its name what indexes it, and what its values are.
 */
export type CallEdge = {
  readonly callSiteKey: string;
  readonly calleeKey: string;
  readonly calleeFileName: string;
  readonly originsByCalleeSlot: readonly (readonly EffectSlot[])[];
  /* Formal-indexed rather than slot-indexed, because a capture is not inside the value the
   * caller authored and so belongs under no property key. Kept beside `originsByCalleeSlot`
   * rather than unioned into it, since the two license different conclusions: an ordinary
   * origin says the callee received the caller's value, so a write or a return the callee
   * records is a fact about it, while a capture says only that running the callable can
   * reach the parameter. */
  readonly capturedOriginsByFormal: readonly (readonly EffectSlot[])[];
  readonly foreignOriginsByFormal: readonly (readonly ParameterIndex[])[];
  readonly directForeignByFormal: readonly boolean[];
  readonly foreignInbound: boolean;
  readonly callbackKeysByCalleeSlot: readonly (
    string | typeof OWNED_CALLABLE_UNAVAILABLE
  )[];
  readonly callbackFileNamesByCalleeSlot: readonly (
    string | typeof OWNED_CALLABLE_UNAVAILABLE
  )[];
};

/**
 Which parameter owns each slot of one callable.
 
 Carried on the summary rather than recomputed, because a summary restored from the
 persistent cache has no declaration to recompute it from.
 */
export type SlotOwnership = {
  readonly parameterCount: number;
  readonly slotCount: number;
  readonly parameterOfSlot: readonly ParameterIndex[];
  readonly slotsByParameter: readonly (readonly EffectSlot[])[];
};

/**
 Mutable internal summary while fixed point is computed.
 */
export type MutableEffectSummary = {
  readonly slots: SlotOwnership;
  readonly bindingOriginBySymbolId: ReadonlyMap<number, SlotOrigins>;
  readonly directMutated: Set<EffectSlot>;
  readonly directInvoked: Set<EffectSlot>;
  readonly directOpaque: Set<EffectSlot>;
  readonly opaqueProvenanceBySlot: Map<EffectSlot, Set<string>>;
  readonly mutated: Set<EffectSlot>;
  readonly invoked: Set<EffectSlot>;
  readonly opaque: Set<EffectSlot>;
  readonly directForeignBorrowed: ReadonlySet<ParameterIndex>;
  readonly directReturned: Set<EffectSlot>;
  /**
   Returned slots after substitution, seeded from `directReturned`.
   
   Separate from the direct set for the same reason `mutated` is separate from
   `directMutated`: a callable that returns another callable's result carries whatever
   that result carries, and only the fixed point can know it.
   */
  readonly returned: Set<EffectSlot>;
  readonly relations: CallbackRelation[];
  readonly elementApplications: ElementApplication[];
  readonly resultApplications: ResultApplication[];
  readonly calls: CallEdge[];
};

/**
 Builds stable declaration key across TypeScript API node wrapper instances.
 
 @param declaration - Function-like declaration to identify.
 
 @returns source path and span identity.
 
 @example
 ```ts
 const key = callableKey(declaration);
 ```
 */
export function callableKey(declaration: EffectCallableDeclaration,): string {
  /**
   Source file owning declaration.
   */
  const sourceFile = declaration.getSourceFile();
  return `${sourceFile.fileName}:${String(declaration.pos,)}:${String(declaration.end,)}:${String(declaration.kind,)}`;
}

/**
 Builds stable identity for one call site inside its source.
 
 Deliberately a string rather than the node. Propagation runs without a project or an
 AST, and a summary restored from the persistent cache has no declarations to point at,
 so a relation that survives caching cannot hold a node.
 
 @param call - Call expression whose site is identified.
 
 @returns source path and span identity for this call.
 
 @example
 ```ts
 const site = callSiteKey(call);
 ```
 */
export function callSiteKey(call: Node,): string {
  /**
   Source file owning this call.
   */
  const sourceFile = call.getSourceFile();
  return `${sourceFile.fileName}:${String(call.pos,)}:${String(call.end,)}`;
}

/**
 Adds one slot to an effect set.
 
 @param target - Effect set receiving slot.
 
 @param value - Slot to add.
 
 @returns whether value was newly added.
 
 @mutates target - Adds resolved slot.
 
 @example
 ```ts
 addEffectSlot({ target: slots, value: asEffectSlot(0) });
 ```
 */
export function addEffectSlot({
  target,
  value,
}: {
  readonly target: Set<EffectSlot>;
  readonly value: EffectSlot | typeof EFFECT_SLOT_UNAVAILABLE;
},): boolean {
  if (value === EFFECT_SLOT_UNAVAILABLE)
    return false;
  /**
   Size before insertion detects fixed-point progress.
   */
  const priorSize = target.size;
  target.add(value,);
  return target.size !== priorSize;
}

/**
 Adds every slot origin a binding can hold to an effect set.
 
 Separate from `addEffectSlot` rather than a widened parameter on it, because the
 two describe different inputs: propagation edges carry one callee slot at a time,
 while a binding carries the whole set of slots it may alias.
 
 @param target - Effect set receiving slots.
 
 @param values - Slot origins resolved for one binding or expression.
 
 @returns whether any slot was newly added.
 
 @mutates target - Adds every resolved slot origin.
 
 @example
 ```ts
 addEffectSlots({ target: summary.directMutated, values: origins });
 ```
 */
export function addEffectSlots({
  target,
  values,
}: {
  readonly target: Set<EffectSlot>;
  readonly values: SlotOrigins;
},): boolean {
  /**
   Size before insertion detects fixed-point progress.
   */
  const priorSize = target.size;
  values.forEach(function addOrigin(value,): void {
    target.add(value,);
  },);
  return target.size !== priorSize;
}

/**
 Collects all descendants using explicit work stack.
 
 @param root - AST subtree root.
 
 @returns nodes in depth-first order including root.
 
 @example
 ```ts
 const nodes = collectAstNodes(sourceFile);
 ```
 */
export function collectAstNodes(root: Node,): readonly Node[] {
  /**
   Collected nodes in traversal order.
   */
  const nodes: Node[] = [];
  /**
   Explicit traversal stack avoids recursive flat-tree traversal.
   */
  const stack: Node[] = [root,];
  while (stack.length > 0) {
    /**
     Next node, absent only when stack changed unexpectedly.
     */
    const node = stack.pop();
    if (node === undefined)
      continue;
    nodes.push(node,);
    node.forEachChild(function collect(child,): undefined {
      stack.push(child,);
      return undefined;
    },);
  }
  return nodes;
}

/**
 Removes property and element access layers from expression.
 
 @param node - Expression whose receiver root is required.
 
 @returns deepest receiver node.
 
 @example
 ```ts
 const root = expressionRoot(propertyAccess);
 ```
 */
export function expressionRoot(node: Node,): Node {
  /**
   Mutable cursor descending through access expressions.
   */
  const cursor: { current: Node; } = { current: node, };
  while (isPropertyAccessExpression(cursor.current,)
    || isElementAccessExpression(cursor.current,)) {
    cursor.current = cursor.current
      .expression;
  }
  return cursor.current;
}
