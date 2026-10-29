/**
 Collection literal mutations: arrays and objects.
 
 @example
 ```ts
 collectionReplacements({ node, parent: undefined, source });
 ```
 */

import { FILLER_TEXT, } from './literals.ts';
import type {
  EstreeNode,
  Replacement,
} from '../types.ts';

/**
 Emits collection replacements for one node.
 
 Non-empty arrays and objects empty out; empty arrays gain one filler
 element, mirroring Stryker's ArrayDeclaration and ObjectLiteral.
 
 @param options - Node under inspection with parent and source.
 
 @returns Replacements, possibly empty.
 
 @example
 ```ts
 collectionReplacements({ node: arrayExpression, parent: undefined, source });
 ```
 */
export function collectionReplacements(options: {
  readonly node: EstreeNode;
  readonly parent?: EstreeNode;
  readonly source: string;
},): readonly Replacement[] {
  if (options.node
    .type
    === 'ArrayExpression') {
    /**
     Whether the array literal has any elements.
     */
    const hasElements = Array.isArray(options.node
      .elements,)
      && (options.node
        .elements
        .length
        > 0);

    return [{
      start: options.node
        .start,
      end: options.node
        .end,
      text: hasElements ? '[]' : `['${FILLER_TEXT}']`,
      operator: 'array',
      description: hasElements ? 'emptied array' : 'filled empty array',
    },];
  }

  if (options.node
    .type
    === 'ObjectExpression') {
    /**
     Whether the object literal has any properties.
     */
    const hasProperties = Array.isArray(options.node
      .properties,)
      && (options.node
        .properties
        .length
        > 0);

    if (!hasProperties)
      return [];

    return [{
      start: options.node
        .start,
      end: options.node
        .end,
      text: '{}',
      operator: 'object',
      description: 'emptied object',
    },];
  }

  return [];
}
