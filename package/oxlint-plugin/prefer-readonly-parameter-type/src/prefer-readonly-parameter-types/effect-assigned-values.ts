/**
 Values a binding is given by assignment, beside the one its declaration wrote.
 
 The walks that follow a binding to a callable follow its initializer, and a binding filled by assignment
 names a value just as surely. Measured, with bodies identical except for how the binding is filled:
 
 ```text
 storeAssignedSelector      opq=[1]     nothing recorded
 storeInitializedSelector   opq=[1,0]   charged
 ```
 
 ## Where this may be asked, and where it may not
 
 Two placements were tried before this one, and each failed differently, which is why the boundary is
 written here rather than left to be rediscovered.
 
 In the general value walk it fixed the shape and broke an invariant two modules away, because that walk
 feeds owned call edges: widening what a value can be widens what must already be summarised, and a newly
 reachable callee had none when the index asserted completeness.
 
 In the completion gate it was a no-op, because that gate decides whether a completion can carry state and
 says nothing about which origins are reachable.
 
 The reach walk is the placement that works. It records opacity and builds no edges, so widening it can
 only withhold more, and it is what decides whether the caller's binding is reachable from a handed
 closure at all.
 
 ## What it answers, and what it declines
 
 The enclosing body is found by ascending from the declaration, so no node universe is threaded in. An
 earlier note claimed this fix needed one; it does not.
 
 Every assignment counts, whichever one a given read reaches, which over-approximates and therefore
 withholds more: attributing too much costs an offer, and missing a value loses an escape. A binding
 assigned from outside its declaring body is not reported, which withholds rather than claiming a value
 the source never gave.
 
 @module
 */

import {
  type Node,
  SyntaxKind,
} from 'typescript/ast';
import {
  isBinaryExpression,
  isIdentifier,
  isSourceFile,
  isVariableDeclaration,
} from 'typescript/ast/is';
import type { Project, } from 'typescript/sync';

import {
  collectAstNodes,
  isEffectCallableDeclaration,
} from './effect-summary-model.ts';

/**
 Names every value assigned to the binding one identifier resolves to.
 
 @param project - TypeScript project resolving the identifier and assignment targets.
 
 @param node - Expression being followed, answered for only when it is an identifier.
 
 @returns right-hand sides of assignments to that binding.
 
 @example
 ```ts
 assignedValuesInScope({ project, node });
 ```
 */
export function assignedValuesInScope({
  project,
  node,
}: {
  readonly project: Project;
  readonly node: Node;
},): readonly Node[] {
  if (!isIdentifier(node,))
    return [];
  /**
   Symbol the identifier resolves to.
   */
  const target = project.checker
    .getResolvedSymbol(node,);
  if (target === undefined)
    return [];
  /**
   Declaration carrying that symbol's value, answered for only when it is a plain local.
   */
  const declared = target.valueDeclaration
    ?.resolve(project,);
  if ((declared === undefined) || (!isVariableDeclaration(declared,)))
    return [];
  return enclosingScope(declared,)
    .flatMap(function assignmentsInScope(scope,): readonly Node[] {
      return collectAstNodes(scope,)
        .flatMap(function assignedValue(candidate,): readonly Node[] {
          return assignedRight({
            project,
            candidate,
            targetName: node,
          },);
        },);
    },);
}

/**
 Names the value one candidate assigns to the followed binding, when it assigns one.
 
 @param project - TypeScript project resolving the assignment target.
 
 @param candidate - Node that may be a plain assignment.
 
 @param targetName - Identifier whose binding is being followed.
 
 @returns assigned value, empty when this candidate assigns something else.
 
 @example
 ```ts
 assignedRight({ project, candidate, targetName });
 ```
 */
function assignedRight({
  project,
  candidate,
  targetName,
}: {
  readonly project: Project;
  readonly candidate: Node;
  readonly targetName: Node;
},): readonly Node[] {
  if (!isBinaryExpression(candidate,))
    return [];
  /**
   Operator deciding whether this is a plain assignment.
   */
  const { kind, } = candidate.operatorToken;
  if (kind !== SyntaxKind.EqualsToken)
    return [];
  /**
   Target the assignment writes to.
   */
  const { left, } = candidate;
  if ((!isIdentifier(left,)) || (!isIdentifier(targetName,)))
    return [];
  /**
   Symbol that target resolves to.
   */
  const assigned = project.checker
    .getResolvedSymbol(left,);
  /**
   Symbol the followed identifier resolves to.
   */
  const followed = project.checker
    .getResolvedSymbol(targetName,);
  return ((assigned !== undefined) && (assigned === followed))
    ? [candidate.right,]
    : [];
}

/**
 Names the nearest node whose subtree can hold every assignment to one declaration.
 
 @param declaration - Declaration whose scope is wanted.
 
 @returns one enclosing callable body or source file, empty when the ascent finds neither.
 
 @example
 ```ts
 enclosingScope(declaration);
 ```
 */
function enclosingScope(declaration: Node,): readonly Node[] {
  /**
   Cursor ascending towards a body that can contain assignments.
   */
  const cursor: { current: Node; } = { current: declaration, };
  /**
   Scope found so far, and whether the ascent can still take a step.
   */
  const state: {
    scope: readonly Node[];
    rising: boolean;
  } = {
    scope: [],
    rising: true,
  };
  while (state.rising) {
    /**
     Node one step above the cursor.
     */
    const above = cursor.current
      .parent;
    if (above === cursor.current) {
      state.rising = false;
      continue;
    }
    cursor.current = above;
    if (isEffectCallableDeclaration(above,) || isSourceFile(above,)) {
      state.scope = [above,];
      state.rising = false;
    }
  }
  return state.scope;
}
