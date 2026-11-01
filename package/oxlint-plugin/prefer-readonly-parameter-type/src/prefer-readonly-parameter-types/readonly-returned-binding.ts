/**
 Semantic proof that one local binding reaches callable return.
 
 @module
 */

import type {
  FunctionLikeDeclaration,
  Identifier,
  Node,
} from 'typescript/ast';
import {
  isFunctionLikeDeclaration,
  isIdentifier,
  isReturnStatement,
} from 'typescript/ast/is';
import type { Project, } from 'typescript/sync';

/**
 Tests whether callable directly returns exact local binding symbol.
 
 Nested callables are excluded because their returns do not produce outer callable result.
 
 @param callable - Callable lexically containing local producer.
 
 @param binding - Local binding naming producer value.
 
 @param project - Project resolving symbol identities.
 
 @returns whether same binding symbol appears as return expression.
 
 @example
 ```ts
 callableReturnsBinding({ callable, binding, project });
 ```
 */
export function callableReturnsBinding({
  callable,
  binding,
  project,
}: {
  readonly callable: FunctionLikeDeclaration;
  readonly binding: Identifier;
  readonly project: Project;
}): boolean {
  /**
   Callable body whose returns are relevant.
   */
  const body = 'body' in callable ? callable.body : undefined;
  if (body === undefined)
    return false;
  /**
   Exact local binding symbol required at return sites.
   */
  const bindingSymbol = project.checker
    .getSymbolAtLocation(binding,);
  if (bindingSymbol === undefined)
    return false;
  /**
   Body nodes awaiting linear structural traversal.
   */
  const pending: Node[] = [body,];
  while (pending.length > 0) {
    /**
     Next body node,
     absent only after unexpected stack mutation.
     */
    const current = pending.pop();
    if (current === undefined)
      continue;
    if (isReturnStatement(current,)) {
      /**
       Returned expression candidate naming local binding.
       */
      const returned = current.expression;
      if (returned === undefined)
        continue;
      if (!isIdentifier(returned,))
        continue;
      /**
       Semantic symbol for exact return identifier.
       */
      const returnedSymbol = project.checker
        .getSymbolAtLocation(returned,);
      if (returnedSymbol?.id === bindingSymbol.id)
        return true;
    }
    current.forEachChild(function enqueue(child,): undefined {
      if (!isFunctionLikeDeclaration(child,))
        pending.push(child,);
      return undefined;
    },);
  }
  return false;
}
