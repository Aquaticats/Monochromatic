/**
 Unary and update operator mutations.
 
 @example
 ```ts
 unaryUpdateReplacements({ node, parent: undefined, source });
 ```
 */

import { childNode, } from '../node-access.ts';
import type {
  EstreeNode,
  Replacement,
} from '../types.ts';

/**
 Numeric unary swap table; `~` drops entirely, mirroring Stryker's
 UnaryOperator. `!` belongs to the boolean family instead.
 */
const UNARY_SWAPS: Readonly<Record<string, string>> = {
  '+': '-',
  '-': '+',
};

/**
 Update operator swap table, mirroring Stryker's UpdateOperator.
 */
const UPDATE_SWAPS: Readonly<Record<string, string>> = {
  '++': '--',
  '--': '++',
};

/**
 Emits unary and update operator replacements for one node.
 
 @param options - Node under inspection with parent and source.
 
 @returns Replacements, possibly empty.
 
 @example
 ```ts
 unaryUpdateReplacements({ node: updateExpression, parent: undefined, source });
 ```
 */
export function unaryUpdateReplacements(options: {
  readonly node: EstreeNode;
  readonly parent?: EstreeNode;
  readonly source: string;
},): readonly Replacement[] {
  if (options.node
    .type
    === 'UnaryExpression') {
    /**
     Argument expression text reused by swap and drop variants.
     */
    const argumentText = options.source
      .slice(
      childNode({
        node: options.node,
        key: 'argument',
      },)
        .start,
      childNode({
        node: options.node,
        key: 'argument',
      },)
        .end,
      );
    /**
     Unary operator token declared by the expression.
     */
    const token = (typeof options.node
      .operator) === 'string'
      ? options.node
        .operator
      : '';

    if (token === '~')
      return [{
        start: options.node
          .start,
        end: options.node
          .end,
        text: argumentText,
        operator: 'unary',
        description: 'removed ~ operator',
      },];

    /**
     Swapped unary operator, when the token participates in the family.
     */
    const swapped = UNARY_SWAPS[token];

    if (swapped === undefined)
      return [];

    return [{
      start: options.node
        .start,
      end: options.node
        .end,
      text: `${swapped}${argumentText}`,
      operator: 'unary',
      description: `swapped unary ${token} with ${swapped}`,
    },];
  }

  if (options.node
    .type
    === 'UpdateExpression') {
    /**
     Update operator token declared by the expression.
     */
    const token = (typeof options.node
      .operator) === 'string'
      ? options.node
        .operator
      : '';
    /**
     Swapped update operator for this token.
     */
    const swapped = UPDATE_SWAPS[token];

    if (swapped === undefined)
      return [];

    /**
     Updated variable expression text.
     */
    const argumentText = options.source
      .slice(
      childNode({
        node: options.node,
        key: 'argument',
      },)
        .start,
      childNode({
        node: options.node,
        key: 'argument',
      },)
        .end,
      );

    return [{
      start: options.node
        .start,
      end: options.node
        .end,
      text: options.node
        .prefix
        === true
        ? `${swapped}${argumentText}`
        : `${argumentText}${swapped}`,
      operator: 'update',
      description: `swapped ${token} with ${swapped}`,
    },];
  }

  return [];
}
