/**
 Ownership-only direct summary for complete inbound proof.
 
 @module
 */

import {
  type BinaryExpression,
  type ForOfStatement,
  SyntaxKind,
  type VariableDeclaration,
} from 'typescript/ast';
import {
  isBinaryExpression,
  isCallExpression,
  isForOfStatement,
  isVariableDeclaration,
} from 'typescript/ast/is';
import type { Project, } from 'typescript/sync';

import { activeCallableBodyNodes, } from './closure-activity.ts';
import {
  discoverAliasOrigins,
  seedParameterSlots,
} from './effect-binding-origins.ts';
import {
  collectAstNodes,
  type EffectCallableDeclaration,
  type MutableEffectSummary,
} from './effect-summary-model.ts';
import { bindingContainsForeignBorrowed, } from './foreign-borrowed-classifier.ts';
import { parameterSlotTable, } from './effect-parameter-slots.ts';
import {
  asParameterIndex,
  type EffectSlot,
  type ParameterIndex,
} from './effect-slot-identity.ts';
import { addForeignBorrowedCallEdge, } from './foreign-borrowed-call-edge.ts';

/**
 Creates ownership seed needed by foreign ownership fixed point.
 
 @param project - TypeScript project resolving owned calls and argument origins.
 
 @param declaration - Callable whose inbound edges are required.
 
 @returns direct marker and binding facts with empty effect dimensions.
 
 @example
 ```ts
 foreignBorrowedOwnershipSeed({ project, declaration });
 ```
 */
export function foreignBorrowedOwnershipSeed({
  project,
  declaration,
}: {
  readonly project: Project;
  readonly declaration: EffectCallableDeclaration;
}): MutableEffectSummary {
  /**
   Binding origins seeded by callable parameters.
   */
  const bindingOriginBySymbolId = new Map<number, Set<EffectSlot>>();
  /**
   Slots this declaration's parameters own.
   */
  const table = parameterSlotTable({ declaration, },);
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
  /**
   Explicit foreign marker indexes on current declaration.
   */
  const directForeignBorrowed = new Set<ParameterIndex>();
  for (const [parameterIndex, parameter,] of declaration.parameters
    .entries()) {
    if (bindingContainsForeignBorrowed({
      project,
      name: parameter.name,
    },))
      directForeignBorrowed.add(asParameterIndex(parameterIndex,),);
  }
  /**
   Ownership-only summary with effect dimensions intentionally empty.
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
   Callable body absent for source-only signatures.
   */
  const body = 'body' in declaration ? declaration.body : undefined;
  if (body === undefined)
    return summary;
  /**
   Complete body nodes used for alias-origin discovery.
   */
  const allBodyNodes = collectAstNodes(body,);
  /**
   Variable aliases initialized from parameter-reachable state.
   */
  const variableDeclarations = allBodyNodes.filter(function variableDeclaration(node,): node is VariableDeclaration {
    return isVariableDeclaration(node,);
  },);
  /**
   Simple assignments that may establish later aliases.
   */
  const aliasAssignments = allBodyNodes.filter(function aliasAssignment(node,): node is BinaryExpression {
    return isBinaryExpression(node,)
      && (node.operatorToken
        .kind
        === SyntaxKind.EqualsToken);
  },);
  /**
   Iterations binding elements reached through parameter-owned iterables.
   */
  const forOfStatements = allBodyNodes.filter(function forOfStatement(node,): node is ForOfStatement {
    return isForOfStatement(node,);
  },);
  discoverAliasOrigins({
    project,
    variableDeclarations,
    aliasAssignments,
    forOfStatements,
    bindingOriginBySymbolId,
  },);
  return summary;
}

/**
 Creates direct marker and owned-call facts for one callable.
 
 @param project - TypeScript project resolving owned calls and argument origins.
 
 @param declaration - Callable whose body calls are summarized.
 
 @param analysisRoot - Optional external package root admitted as owned.
 
 @returns ownership summary with every active direct call edge.
 
 @example
 ```ts
 foreignBorrowedDirectSummary({ project, declaration });
 ```
 */
export function foreignBorrowedDirectSummary({
  project,
  declaration,
  analysisRoot,
}: {
  readonly project: Project;
  readonly declaration: EffectCallableDeclaration;
  readonly analysisRoot?: string;
}): MutableEffectSummary {
  /**
   Marker and alias-origin seed for current callable.
   */
  const summary = foreignBorrowedOwnershipSeed({
    project,
    declaration,
  },);
  /**
   Callable body absent for source-only signatures.
   */
  const body = 'body' in declaration ? declaration.body : undefined;
  if (body === undefined)
    return summary;
  activeCallableBodyNodes({
    project,
    body,
    boundary: declaration,
    /* This scan looks for owned calls only, and it passes no parameter initializers, so an owned
     * call written in a default is not seen here:
     *
     * ```ts
     * function candidate(value: Value, ignored = ownedHelper(value),): void {}
     * ```
     *
     * An earlier note here claimed a parameter default cannot contain an owned call the body does
     * not, which is simply false. Left as a stated gap rather than closed blind: what a missing
     * owned inbound edge costs this graph is a question about ownership inference rather than
     * about effects, and the scan's cost was the subject of its own measurement, so widening its
     * universe is a change to make against a sweep. Task #81 holds it. */
    parameterInitializerNodes: [],
    bindingOriginBySymbolId: summary.bindingOriginBySymbolId,
  },)
    .forEach(function inspectOwnedCall(node,): void {
      if (!isCallExpression(node,))
        return;
      addForeignBorrowedCallEdge({
        project,
        declaration,
        call: node,
        summary,
        ...(analysisRoot === undefined) ? {} : { analysisRoot, },
      },);
    },);
  return summary;
}
