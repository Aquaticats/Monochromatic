/**
 Project-owned caller resolution and collection-observer graph insertion.
 
 @module
 */

import type {
  CallExpression,
  Node,
  SourceFile,
} from 'typescript/ast';
import type { Project, } from 'typescript/sync';

import {
  callableKey,
  type EffectCallableDeclaration,
  isEffectCallableDeclaration,
  type MutableEffectSummary,
  OWNED_CALLABLE_UNAVAILABLE,
} from './effect-summary-model.ts';
import { foreignBorrowedOwnershipSeed, } from './foreign-borrowed-direct-summary.ts';
import { addForeignObserverInbound, } from './foreign-borrowed-observer-edge.ts';

/**
 Finds nearest callable owner admitted by effect source policy.
 
 @param node - Call expression whose caller is needed.
 
 @param indexedSourceFiles - Sources admitted as owned.
 
 @returns caller declaration or unavailable sentinel.
 
 @example
 ```ts
 nearestForeignOwnedCallable({ node: call, indexedSourceFiles });
 ```
 */
export function nearestForeignOwnedCallable({
  node,
  indexedSourceFiles,
}: {
  readonly node: Node;
  readonly indexedSourceFiles: ReadonlyMap<string, SourceFile>;
}): EffectCallableDeclaration | typeof OWNED_CALLABLE_UNAVAILABLE {
  /**
   Parent cursor seeking callable boundary.
   */
  const cursor: { current: Node; } = { current: node.parent, };
  while (!isEffectCallableDeclaration(cursor.current,)) {
    /**
     Next parent,
     self-parented source boundary,
     or absent past source root.
     */
    const { parent, } = cursor.current;
    if ((parent === undefined) || (parent === cursor.current))
      return OWNED_CALLABLE_UNAVAILABLE;
    cursor.current = parent;
  }
  return indexedSourceFiles.has(cursor.current
    .getSourceFile()
    .fileName,)
    ? cursor.current
    : OWNED_CALLABLE_UNAVAILABLE;
}

/**
 Adds one exact collection-observer inbound and queues its enclosing caller.
 
 @param project - Project resolving observer relation.
 
 @param indexedSourceFiles - Sources admitted as owned.
 
 @param summaries - Mutable ownership graph summaries.
 
 @param queue - Caller declarations awaiting inbound discovery.
 
 @param call - Collection call carrying observer.
 
 @param observerDeclaration - Observer declaration receiving receiver state.
 
 @returns whether supported observer edge was added.
 
 @mutates summaries - Adds caller ownership seed and observer edge.
 
 @mutates queue - Appends enclosing caller declaration.
 
 @example
 ```ts
 addForeignObserverCaller({
   project,
   indexedSourceFiles,
   summaries,
   queue,
   call,
   observerDeclaration,
 });
 ```
 */
export function addForeignObserverCaller({
  project,
  indexedSourceFiles,
  summaries,
  queue,
  call,
  observerDeclaration,
}: {
  readonly project: Project;
  readonly indexedSourceFiles: ReadonlyMap<string, SourceFile>;
  readonly summaries: Map<string, MutableEffectSummary>;
  readonly queue: EffectCallableDeclaration[];
  readonly call: CallExpression;
  readonly observerDeclaration: EffectCallableDeclaration;
}): boolean {
  /**
   Nearest project-owned callable containing collection call.
   */
  const caller = nearestForeignOwnedCallable({
    node: call,
    indexedSourceFiles,
  },);
  if (caller === OWNED_CALLABLE_UNAVAILABLE)
    return false;
  /**
   Stable caller identity for graph summary.
   */
  const callerKey = callableKey(caller,);
  /**
   Caller ownership seed receiving synthetic observer edge.
   */
  const callerSummary = summaries.get(callerKey,)
    ?? foreignBorrowedOwnershipSeed({
      project,
      declaration: caller,
    },);
  summaries.set(
    callerKey,
    callerSummary,
  );
  queue.push(caller,);
  return addForeignObserverInbound({
    project,
    call,
    observerDeclaration,
    callerSummary,
  },);
}
