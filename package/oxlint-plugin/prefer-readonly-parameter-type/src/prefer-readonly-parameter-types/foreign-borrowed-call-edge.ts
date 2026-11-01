/**
 Exact ownership edge for one TypeScript call usage.
 
 @module
 */

import type { CallExpression, } from 'typescript/ast';
import type { Project, } from 'typescript/sync';

import {
  callableDeclaration,
  parameterIndexes,
} from './effect-call-resolution.ts';
import { declarationDirectlyOwnsNode, } from './effect-foreign-inbound.ts';
import type { EffectSlot, } from './effect-slot-identity.ts';
import { addOwnedCallEdge, } from './effect-owned-call-edge.ts';
import {
  type EffectCallableDeclaration,
  isEffectCallableDeclaration,
  type MutableEffectSummary,
  OWNED_CALLABLE_UNAVAILABLE,
} from './effect-summary-model.ts';

/**
 Adds exact owned edge represented by one call expression.
 
 @param project - TypeScript project resolving call signature.
 
 @param declaration - Callable expected to own call directly.
 
 @param call - Exact usage call requiring ownership edge.
 
 @param summary - Caller ownership summary receiving edge.
 
 @param analysisRoot - Optional external package root admitted as owned.
 
 @returns whether call resolved to owned callable edge.
 
 @example
 ```ts
 addForeignBorrowedCallEdge({ project, declaration, call, summary });
 ```
 */
export function addForeignBorrowedCallEdge({
  project,
  declaration,
  call,
  summary,
  analysisRoot,
}: {
  readonly project: Project;
  readonly declaration: EffectCallableDeclaration;
  readonly call: CallExpression;
  readonly summary: MutableEffectSummary;
  readonly analysisRoot?: string;
}): boolean {
  /**
   Resolved source declaration selected by overload when available.
   */
  const resolvedDeclaration = project.checker
    .getResolvedSignature(call,)
    ?.declaration
    ?.resolve(project,);
  /**
   Owned callee selected from signature before expression fallback.
   */
  const signatureCallee = (resolvedDeclaration !== undefined)
    && isEffectCallableDeclaration(resolvedDeclaration,)
    ? callableDeclaration({
      project,
      node: resolvedDeclaration,
      ...(analysisRoot === undefined) ? {} : { analysisRoot, },
    },)
    : OWNED_CALLABLE_UNAVAILABLE;
  /**
   Final owned callee declaration.
   */
  const callee = signatureCallee === OWNED_CALLABLE_UNAVAILABLE
    ? callableDeclaration({
      project,
      node: call.expression,
      ...(analysisRoot === undefined) ? {} : { analysisRoot, },
    },)
    : signatureCallee;
  if (callee === OWNED_CALLABLE_UNAVAILABLE)
    return false;
  /**
   Caller parameter roots corresponding to exact call arguments.
   */
  const allArgumentIndexes = call.arguments
    .map(function argumentIndexes(argument,): readonly EffectSlot[] {
      return parameterIndexes({
        project,
        bindingOriginBySymbolId: summary.bindingOriginBySymbolId,
        node: argument,
      },);
    },);
  addOwnedCallEdge({
    project,
    call,
    callee,
    allArgumentIndexes,
    summary,
    foreignInbound: declarationDirectlyOwnsNode({
      node: call,
      declaration,
    },),
    ...(analysisRoot === undefined) ? {} : { analysisRoot, },
  },);
  return true;
}
