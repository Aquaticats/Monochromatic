/**
 Direct syntax and call-edge extraction for callable effect summaries.
 
 @module
 */

import {
  type BinaryExpression,
  type ForOfStatement,
  type Node,
  SyntaxKind,
  type VariableDeclaration,
} from 'typescript/ast';
import {
  isAssignmentOperator,
  isBinaryExpression,
  isBlock,
  isCallExpression,
  isDeleteExpression,
  isForOfStatement,
  isIdentifier,
  isPostfixUnaryExpression,
  isPrefixUnaryExpression,
  isReturnStatement,
  isVariableDeclaration,
} from 'typescript/ast/is';
import type { Project, } from 'typescript/sync';

import { activeCallableBodyNodes, } from './closure-activity.ts';
import {
  recordAssignmentStore,
  recordIterationStore,
} from './effect-assignment-store.ts';
import { discoverBodyBindings, } from './effect-body-bindings.ts';
import { recordOutwardHandoffs, } from './effect-outward-handoff.ts';
import {
  recordReturnEffects,
  returnBelongsToCallable,
} from './effect-return-effects.ts';
import { recordReturnedCallableCapture, } from './effect-returned-callable.ts';
import { inspectDirectWrite, } from './effect-direct-write.ts';
import { recordBodylessEffects, } from './direct-bodyless-summary.ts';
import { recordResultApplication, } from './effect-result-substitution.ts';
import {
  bindingOriginsFor,
  discoverAliasOrigins,
  expressionOrigins,
  seedParameterSlots,
} from './effect-binding-origins.ts';
import { parameterSlotTable, } from './effect-parameter-slots.ts';
import {
  asEffectSlot,
  asParameterIndex,
  type EffectSlot,
  type ParameterIndex,
} from './effect-slot-identity.ts';
import { bindingContainsForeignBorrowed, } from './foreign-borrowed-classifier.ts';
import { inspectEffectCall, } from './effect-call-analysis.ts';
import { declarationDirectlyOwnsNode, } from './effect-foreign-inbound.ts';
import { addOpaqueEffect, } from './effect-call-resolution.ts';
import {
  chargeDrainedIterator,
  recordIterationEffects,
} from './effect-drain-charge.ts';
import type { ExternalCallableEffectResolver, } from './external-callable-effect.ts';
import { expressionCanCarryMutableState, } from './effect-primitive-origin.ts';
import {
  MUTATION_CONTRACT_UNAVAILABLE,
  mutationContractsForDeclaration,
  mutationTargetIndexes,
} from './mutation-contract-query.ts';
import {
  addEffectSlot,
  addEffectSlots,
  callableKey,
  type EffectCallableDeclaration,
  collectAstNodes,
  type MutableEffectSummary,
  EFFECT_SLOT_UNAVAILABLE,
  type SlotOrigins,
} from './effect-summary-model.ts';

/**
 Builds direct syntax facts and call edges for one callable.
 
 @param project - TypeScript project owning declaration.
 
 @param declaration - Callable declaration to inspect.
 
 @param analysisRoot - Optional external implementation root accepted for transitive calls.
 
 @param externalEffectResolver - Demand-driven package implementation analyzer.
 
 @returns mutable summary seeded with direct effects.
 
 @example
 ```ts
 const summary = directEffectSummary({ project, declaration });
 ```
 */
export function directEffectSummary({
  project,
  declaration,
  analysisRoot,
  externalEffectResolver,
}: {
  readonly project: Project;
  readonly declaration: EffectCallableDeclaration;
  readonly analysisRoot?: string;
  readonly externalEffectResolver: ExternalCallableEffectResolver;
},): MutableEffectSummary {
  /**
   TypeScript checker for current project.
   */
  const { checker, } = project;
  /**
   Slots this callable's parameters own, allocated from the declaration alone.
   */
  const table = parameterSlotTable({ declaration, },);
  /**
   Binding symbol origins seeded by callable parameters.
   */
  const bindingOriginBySymbolId = new Map<number, Set<EffectSlot>>();
  declaration.parameters
    .forEach(function registerParameter(
      parameter,
      parameterIndex,
    ): void {
      seedParameterSlots({
        project,
        parameter,
        parameterIndex,
        table,
        bindingOriginBySymbolId,
      },);
    },);
  /* A default initializer naming an earlier parameter makes the two aliases. Omitting the
   * argument, or passing `undefined`, means a write through the later name reaches the
   * earlier parameter's value, so both indexes have to answer for it. Registered after
   * every parameter has its own index, because an initializer can only name a parameter
   * declared before it. `mutateDefaultAlias` in the call-edge fixture measured the gap: it
   * recorded a write on the aliasing formal alone and offered the aliased one readonly. */
  declaration.parameters
    .forEach(function registerDefaultAlias(parameter,): void {
      if (parameter.initializer === undefined)
        return;
      addEffectSlots({
        target: bindingOriginsFor({
          project,
          name: parameter.name,
          bindingOriginBySymbolId,
        },),
        values: expressionOrigins({
          project,
          bindingOriginBySymbolId,
          node: parameter.initializer,
        },),
      },);
    },);
  /**
   Parameter indexes explicitly carrying exact foreign ownership marker.
   */
  const directForeignBorrowed = new Set<ParameterIndex>();
  /**
   Parameter entries paired with declaration indexes.
   */
  const parameterEntries = declaration.parameters
    .entries();
  for (const [parameterIndex, parameter,] of parameterEntries) {
    if (bindingContainsForeignBorrowed({
      project,
      name: parameter.name,
    },))
      directForeignBorrowed.add(asParameterIndex(parameterIndex,),);
  }
  /**
   Mutable summary receiving direct and propagated effects.
   */
  const summary: MutableEffectSummary = {
    slots: table,
    bindingOriginBySymbolId,
    directMutated: new Set(),
    directInvoked: new Set(),
    directOpaque: new Set(),
    opaqueProvenanceBySlot: new Map(),
    mutated: new Set(),
    invoked: new Set(),
    opaque: new Set(),
    directForeignBorrowed,
    directReturned: new Set(),
    returned: new Set(),
    relations: [],
    elementApplications: [],
    resultApplications: [],
    calls: [],
  };
  /**
   Callable implementation body, absent for source signatures.
   */
  const body = 'body' in declaration ? declaration.body : undefined;
  if (body === undefined) {
    recordBodylessEffects({
      checker,
      declaration,
      summary,
    },);
    return summary;
  }
  /**
   What every local in this body can be holding, by parameter slot and by call site.
   */
  const {
    parameterInitializerNodes,
    resultSitesBySymbolId,
    containerLiteralHolders,
  } = discoverBodyBindings({
    project,
    declaration,
    body,
    bindingOriginBySymbolId,
  },);
  /**
   Body nodes selected after aliases expose caller-reachable closure storage.
   
   Parameter initializers join the node universe the closure selection gates rather than the
   selected set directly, and the boundary the ancestry walk ascends to is the declaration
   rather than the body, because an initializer node's parents reach the parameter and never
   the body.
   
   An initializer expression runs on entry every time the argument is omitted, which is true of
   the expression and not of everything inside it. One initializer can package a callable:
   
   ```ts
   function f(config: Config, callback = (): void => { config.row.label = 'x'; },): void {}
   ```
   
   Adding those nodes to the selected set outright attributed that write to `config` although
   nothing invokes the closure. Measured `mutated=[0]` then, `mutated=[]` now, with the invoked
   form keeping the write it genuinely performs because a call to a parameter resolves through
   its default.
   */
  const bodyNodes = activeCallableBodyNodes({
    project,
    body,
    boundary: declaration,
    parameterInitializerNodes,
    bindingOriginBySymbolId,
  },);
  bodyNodes.forEach(function inspect(node,): void {
    if (isBinaryExpression(node,)
      && isAssignmentOperator(node.operatorToken
        .kind,)) {
      inspectDirectWrite({
        project,
        bindingOriginBySymbolId,
        resultSitesBySymbolId,
        summary,
        node: node.left,
      },);
      /* Two questions about one assignment, and `inspectDirectWrite` answers only the
       * first. It asks which parameter the write lands on and returns early for an
       * identifier target, correctly, because `held = row` changes no object the caller
       * can see. What that leaves is where the value went, and an identifier target is
       * exactly where it matters: a binding outside this callable keeps the value after
       * the call returns. */
      recordAssignmentStore({
        project,
        bindingOriginBySymbolId,
        resultSitesBySymbolId,
        summary,
        node,
        body,
      },);
      return;
    }
    if (isDeleteExpression(node,)) {
      inspectDirectWrite({
        project,
        bindingOriginBySymbolId,
        resultSitesBySymbolId,
        summary,
        node: node.expression,
      },);
      return;
    }
    if ((isPrefixUnaryExpression(node,) || isPostfixUnaryExpression(node,))
      && ((node.operator === SyntaxKind.PlusPlusToken)
        || (node.operator === SyntaxKind.MinusMinusToken))) {
      inspectDirectWrite({
        project,
        bindingOriginBySymbolId,
        resultSitesBySymbolId,
        summary,
        node: node.operand,
      },);
      return;
    }
    if (isReturnStatement(node,)
      && (node.expression !== undefined)
      /* A nested callable's return belongs to the nested callable. Its body is scanned inline, so
       * every return inside an active nested callable reached this branch and was recorded as this
       * callable's own, and `storeLocalFunctionResult` claimed a returned origin while returning
       * nothing at all.
       *
       * A returned origin is a positive capability claim, saying a caller can reach that parameter
       * through this result, so claiming one where there is no result is the same kind of wrong
       * fact the activation gate removed. Nothing discharges on a returned set today, which is why
       * it cost no offer, and it would have become unsound the moment something did.
       *
       * The origins those nested bodies do supply are unaffected: a write inside one still
       * attributes, and what a nested callable hands back is answered by the capture walk at the
       * call site rather than by this branch. */
      && returnBelongsToCallable({
        node,
        body,
      },)) {
      recordReturnEffects({
        project,
        checker,
        bindingOriginBySymbolId,
        resultSitesBySymbolId,
        summary,
        returned: node.expression,
      },);
      return;
    }
    if (isForOfStatement(node,)) {
      recordIterationEffects({
        project,
        bindingOriginBySymbolId,
        resultSitesBySymbolId,
        summary,
        node,
        body,
      },);
      return;
    }
    /* Asked of every node rather than inside another branch, because a construction, a yield, a
     * tagged template and a throw are none of them calls or stores or returns, which is why
     * nothing answered for any of them. Each was found by walking escape channels and each was
     * falsified. Collected behind one call for this file's line budget. */
    /* Draining an iterator is a call, and the baseline trusts only one the library declares.
     * Asked of every node, since a spread and a `for...of` ask one question from two
     * branches. */
    chargeDrainedIterator({
      project,
      bindingOriginBySymbolId,
      summary,
      node,
    },);
    recordOutwardHandoffs({
      project,
      bindingOriginBySymbolId,
      summary,
      node,
    },);
    if (isCallExpression(node,)) {
      inspectEffectCall({
        project,
        checker,
        bindingOriginBySymbolId,
        resultSitesBySymbolId,
        containerLiteralHolders,
        call: node,
        summary,
        ...(analysisRoot === undefined) ? {} : { analysisRoot, },
        externalEffectResolver,
        body,
        foreignInbound: declarationDirectlyOwnsNode({
          node,
          declaration,
        },),
      },);
    }
  },);
  /* And the result a concise arrow hands back, which no statement in the walk above can carry. The
   * returned branch there tests `isReturnStatement`, and a concise body is the callable's own body
   * expression with no return statement anywhere, so `(row: Row,): Row => row` recorded an empty
   * returned set. Measured at top level rather than only through a default, which is what showed the
   * body form to be the cause: storing what a concise identity handed back offered its caller's
   * configuration while the same identity with a block body withheld. Falsified.
   *
   * The nested case needs no gate here, unlike the statement branch. `body` is this callable's own
   * body, so a nested concise arrow inside it is never what this reads, and the return a nested
   * callable performs stays the nested callable's own exactly as #77 established. */
  if (!isBlock(body,)) {
    recordReturnEffects({
      project,
      checker,
      bindingOriginBySymbolId,
      resultSitesBySymbolId,
      summary,
      returned: body,
    },);
  }
  summary.directMutated
    .forEach(function seed(index,): void {
      summary.mutated
        .add(index,);
    },);
  summary.directInvoked
    .forEach(function seedInvocation(index,): void {
      summary.invoked
        .add(index,);
    },);
  summary.directOpaque
    .forEach(function seedOpacity(index,): void {
      summary.opaque
        .add(index,);
    },);
  return summary;
}
