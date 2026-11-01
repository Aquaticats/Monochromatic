/**
 Direct callable ownership for foreign inbound call edges.
 
 @module
 */

import type { Node, } from 'typescript/ast';

import {
  callableKey,
  type EffectCallableDeclaration,
  isEffectCallableDeclaration,
} from './effect-summary-model.ts';

/**
 Tests whether call belongs to declaration rather than active nested closure.
 
 @param node - Call node whose nearest callable owner is resolved.
 
 @param declaration - Summary declaration expected as direct owner.
 
 @returns whether declaration directly owns call.
 
 @example
 ```ts
 declarationDirectlyOwnsNode({ node: call, declaration });
 ```
 */
export function declarationDirectlyOwnsNode({
  node,
  declaration,
}: {
  readonly node: Node;
  readonly declaration: EffectCallableDeclaration;
}): boolean {
  /**
   Stable identity of expected direct owner.
   */
  const declarationIdentity = callableKey(declaration,);
  /**
   Parent cursor seeking nearest callable declaration.
   */
  const cursor: { current: Node; } = { current: node.parent, };
  while (!isEffectCallableDeclaration(cursor.current,)) {
    /**
     Next parent or self-parented source boundary.
     */
    const { parent, } = cursor.current;
    if (parent === cursor.current)
      return false;
    cursor.current = parent;
  }
  return callableKey(cursor.current,) === declarationIdentity;
}
