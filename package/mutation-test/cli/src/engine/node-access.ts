/**
 Typed access to ESTree child nodes.
 
 @example
 ```ts
 childNode({ node: binaryExpression, key: 'left' });
 ```
 */

import type { EstreeNode, } from './types.ts';

/**
 Returns whether a value looks like an ESTree node.
 
 Keeps operators structural: they narrow parser output through this
 guard instead of depending on the parser's full node union.
 
 @param value - Candidate value from a node property.
 
 @returns Whether value carries `type` plus span offsets.
 
 @example
 ```ts
 isEstreeNode({ type: 'Literal', start: 0, end: 1 });
 // true
 ```
 */
export function isEstreeNode(value: unknown,): value is EstreeNode {
  return (value !== null)
    && ((typeof value) === 'object')
    && ((typeof (value as { type?: unknown; }).type) === 'string')
    && ((typeof (value as { start?: unknown; }).start) === 'number')
    && ((typeof (value as { end?: unknown; }).end) === 'number');
}

/**
 Reads one required child node property, throwing on shape mismatch.
 
 Operators only run on node types they matched, so a missing child means
 the AST shape assumption broke; failing loudly beats emitting a mutant
 with a garbage span.
 
 @param options - Parent node and child property key.
 
 @returns Child node.
 
 @throws Error when property is not an ESTree node.
 
 @example
 ```ts
 childNode({ node, key: 'left' });
 ```
 */
export function childNode(options: {
  readonly node: EstreeNode;
  readonly key: string;
},): EstreeNode {
  /**
   Raw child property value before shape validation.
   */
  const value = options.node[options.key];

  if (!isEstreeNode(value,))
    throw new Error(`expected ${options.node
      .type}.${options.key} to be a node`,);

  return value;
}

/**
 Returns whether one child property holds a node.
 
 Pairs with {@link childNode} for optional children: guard with this,
 then read with childNode, keeping absence handling at the call site.
 
 @param options - Parent node and child property key.
 
 @returns Whether property is an ESTree node.
 
 @example
 ```ts
 hasChildNode({ node: forStatement, key: 'test' });
 ```
 */
export function hasChildNode(options: {
  readonly node: EstreeNode;
  readonly key: string;
},): boolean {
  return isEstreeNode(options.node[options.key],);
}
