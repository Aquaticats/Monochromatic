/**
 Everything the body scan has to know about its own locals before it starts.
 
 Split from `direct-effect-summary.ts` for the line budget, on the seam the phases already
 had. This phase reads no effects and records none: it collects the syntax that can bind a
 value and answers two questions about every local, which parameter slots it can reach and
 which call filled it. The scan that follows consults both and writes the summary.
 
 Keeping the two discoveries together matters more than keeping either beside its caller.
 They walk the same three node lists, they converge in the same loop shape, and a change
 to what counts as a binding has to reach both or the maps disagree about the same local.
 
 @module
 */

import {
  type BinaryExpression,
  type ForOfStatement,
  type Node,
  type ParameterDeclaration,
  SyntaxKind,
  type VariableDeclaration,
} from 'typescript/ast';
import {
  isBinaryExpression,
  isForOfStatement,
  isVariableDeclaration,
} from 'typescript/ast/is';
import type { Project, } from 'typescript/sync';


import { discoverAliasOrigins, } from './effect-binding-origins.ts';
import { containerLiteralHolderSymbolIds, } from './effect-container-literal-holder.ts';
import { discoverResultBindings, } from './effect-result-binding.ts';
import type { EffectSlot, } from './effect-slot-identity.ts';
import {
  collectAstNodes,
  type EffectCallableDeclaration,
} from './effect-summary-model.ts';

/**
 Assignment operators that bind the right operand's own reference to the target.
 
 The arithmetic forms are absent on purpose: `total += config.rows.length` coerces its operand
 to a primitive, so the target holds a number rather than anything the operand referenced.
 */
const ALIAS_ESTABLISHING_OPERATORS: ReadonlySet<SyntaxKind> = new Set([
  SyntaxKind.EqualsToken,
  SyntaxKind.BarBarEqualsToken,
  SyntaxKind.AmpersandAmpersandEqualsToken,
  SyntaxKind.QuestionQuestionEqualsToken,
],);

/**
 What the body scan needs from the binding discovery phase.
 */
export type BodyBindings = {
  readonly parameterInitializerNodes: readonly Node[];
  readonly resultSitesBySymbolId: ReadonlyMap<number, ReadonlySet<string>>;
  readonly containerLiteralHolders: ReadonlySet<number>;
};

/**
 Resolves what every local in one callable body can be holding.
 
 @param project - TypeScript project resolving binding symbols.
 
 @param declaration - Callable whose parameter initializers join the walk.
 
 @param body - Callable implementation body.
 
 @param bindingOriginBySymbolId - Origin map filled with parameter slots per binding.
 
 @mutates bindingOriginBySymbolId - Adds every parameter slot each binding can reach.
 
 @returns parameter initializer nodes and the call sites recorded per binding.
 
 @example
 ```ts
 discoverBodyBindings({ project, declaration, body, bindingOriginBySymbolId });
 ```
 */
export function discoverBodyBindings({
  project,
  declaration,
  body,
  bindingOriginBySymbolId,
}: {
  readonly project: Project;
  readonly declaration: EffectCallableDeclaration;
  readonly body: Node;
  readonly bindingOriginBySymbolId: Map<number, Set<EffectSlot>>;
},): BodyBindings {
  /* Parameter initializers run on entry and can do anything a body statement can, so they
   * belong to the callable's own effects. A walk bounded by the body never saw them, and
   * `defaultInitializerEffect` in the call-edge fixture reached a mutating call from an
   * initializer and was offered readonly for the parameter it wrote. */
  /**
   Nodes of every parameter initializer, which run on entry before the body.
   */
  const parameterInitializerNodes = declaration.parameters
    .flatMap(function initializerNodes(parameter,): readonly Node[] {
      return parameter.initializer === undefined
        ? []
        : collectAstNodes(parameter.initializer,);
    },);
  /**
   Complete body nodes used to discover origins before escape selection.
   */
  const allBodyNodes = [
    ...parameterInitializerNodes,
    ...collectAstNodes(body,),
  ];
  /**
   Variable declarations that may alias parameter-reachable state.
   */
  const variableDeclarations = allBodyNodes.filter(function variableDeclaration(node,): node is VariableDeclaration {
    return isVariableDeclaration(node,);
  },);
  /**
   Declarations that may bind a call result, this callable's own parameters included.
   
   A parameter default binds a result exactly as a local declaration does, and the alias scan
   beside this one already reads parameter initializers for their origins, so the asymmetry was
   in the result record alone.
   */
  const resultBindingDeclarations: readonly (VariableDeclaration | ParameterDeclaration)[] = [
    ...variableDeclarations,
    ...declaration.parameters,
  ];
  /**
   Assignments that may establish aliases after declaration.
   
   The logical forms belong here beside plain assignment, because each stores the right
   operand's own reference whenever it stores anything. Plain assignment alone was collected,
   so `row ??= firstRow(config,)` bound a result the record never learned about and a later
   write through `row` attributed nothing. Falsified.
   */
  const aliasAssignments = allBodyNodes.filter(function aliasAssignment(node,): node is BinaryExpression {
    return isBinaryExpression(node,)
      && ALIAS_ESTABLISHING_OPERATORS.has(node.operatorToken
        .kind,);
  },);
  /**
   Iteration statements binding elements reached through parameter-owned iterables.
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
  /**
   Call sites each local binding can be holding a result of.
   
   Beside the origins rather than inside them, because the two settle for different
   reasons: origins accumulate parameter slots a binding can reach, while these name a
   syntactic fact about where a value came from, complete as soon as the declarations
   have been walked.
   */
  const resultSitesBySymbolId = new Map<number, Set<string>>();
  discoverResultBindings({
    project,
    variableDeclarations: resultBindingDeclarations,
    aliasAssignments,
    forOfStatements,
    resultSitesBySymbolId,
  },);
  return {
    parameterInitializerNodes,
    resultSitesBySymbolId,
    /* Beside the origins for the same reason the result sites are: it records a syntactic
     * fact about where a binding's value was built rather than which parameters that value
     * can reach, and exactly one consumer asks the first question. */
    containerLiteralHolders: containerLiteralHolderSymbolIds({
      project,
      variableDeclarations,
    },),
  };
}
