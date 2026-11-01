/**
 Whether a node is written in one callable body or in a callable nested under it.
 
 @module
 */

import type { Node, } from 'typescript/ast';

import { isPresentNode, } from './effect-value-consumer.ts';
import { isEffectCallableDeclaration, } from './effect-summary-model.ts';

/**
 Tests whether a node is written directly in one body rather than in a callable under it.
 
 One question two separate steps were each answering with their own ascent.
 `resultEscapesCallable` asks it to treat a captured reference as escaping, because a use
 inside a nested callable outlives its reasoning about statement order. The returned-result
 discharge asks it because caller enumeration is performed for the body it was handed, and a
 `return` written inside a nested declaration belongs to that declaration instead:
 
 ```ts
 function outer(rows: Row[],): () => Row[] {
   function inner(): Row[] {
     return rows.slice(0,);
   }
   return inner;
 }
 ```
 
 The call is returned outright and the callers being enumerated are `outer`'s, which
 substitute for `outer` alone. Whoever later calls `inner` writes through the result with
 nothing attributing it, so answering this wrongly is the guarded failure directly.
 
 Ascending rather than descending, because the containment question is asked of one node
 against one body and an ascent visits only that node's ancestors. The walk stops on an
 absent parent as well as a self-referential one, for the reason `effect-value-consumer.ts`
 records: a source file's parent is `undefined` here while the declared type says otherwise,
 so a walk that trusts the type runs past the root.
 
 Both exits answer false, which is the conservative direction for each caller. A node not
 inside the body at all is one neither caller can reason about, and the escape test's own
 note that "every caller passes a node inside `body`" makes that exit unreachable rather
 than merely unlikely.
 
 @param node - Node being placed.
 
 @param body - Body it is expected to belong to.
 
 @returns whether the node's nearest enclosing callable body is exactly that body.
 
 @example
 ```ts
 writtenDirectlyInBody({ node: call, body });
 ```
 */
export function writtenDirectlyInBody({
  node,
  body,
}: {
  readonly node: Node;
  readonly body: Node;
},): boolean {
  /**
   Cursor ascending toward the expected body.
   */
  const cursor: { current: Node; } = { current: node, };
  while (cursor.current !== body) {
    if (isEffectCallableDeclaration(cursor.current,))
      return false;
    /**
     Enclosing node, absent at the root whatever the declared type says.
     */
    const { parent, } = cursor.current;
    if ((!isPresentNode({ candidate: parent, },))
      || (parent === cursor.current))
      return false;
    cursor.current = parent;
  }
  return true;
}
