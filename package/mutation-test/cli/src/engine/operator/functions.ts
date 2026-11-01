/**
 Function-shaped mutations: block emptying and arrow body forcing.
 
 @example
 ```ts
 functionReplacements({ node, parent, source });
 ```
 */

import { childNode, } from '../node-access.ts';
import type {
  EstreeNode,
  Replacement,
} from '../types.ts';

/**
 Emits block and arrow replacements for one node.
 
 Non-empty blocks empty out (Stryker's BlockStatement); expression-bodied
 arrows return `undefined` instead (Stryker's ArrowFunction). Block-bodied
 arrows are covered by the block family, so the arrow variant skips them.
 
 @param options - Node under inspection with parent and source.
 
 @returns Replacements, possibly empty.
 
 @example
 ```ts
 functionReplacements({ node: blockStatement, parent, source });
 ```
 */
export function functionReplacements(options: {
  readonly node: EstreeNode;
  readonly parent?: EstreeNode;
  readonly source: string;
},): readonly Replacement[] {
  if (options.node
    .type
    === 'BlockStatement') {
    /**
     Whether the block holds any statements.
     */
    const hasBody = Array.isArray(options.node
      .body,)
      && (options.node
        .body
        .length
        > 0);

    if (!hasBody)
      return [];

    return [{
      start: options.node
        .start,
      end: options.node
        .end,
      text: '{}',
      operator: 'block',
      description: 'emptied block',
    },];
  }

  if ((options.node
    .type
    === 'ArrowFunctionExpression')
    && (options.node
      .expression
      === true)) {
    /**
     Expression body of the arrow function.
     */
    const body = childNode({
      node: options.node,
      key: 'body',
    },);

    if ((body.type === 'Identifier') && (body.name === 'undefined'))
      return [];

    return [{
      start: body.start,
      end: body.end,
      text: 'undefined',
      operator: 'arrow',
      description: 'forced arrow body to undefined',
    },];
  }

  return [];
}
