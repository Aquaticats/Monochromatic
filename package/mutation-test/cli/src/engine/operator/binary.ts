/**
 Arithmetic, equality, and logical operator swaps.
 
 Replacement spans cover only the operator token (located with the
 comment-aware scanner), so operand text, parentheses, and comments
 survive untouched.
 
 @example
 ```ts
 binaryOperatorReplacements({ node, parent: undefined, source: 'a + b' });
 ```
 */

import { findOperatorToken, } from '../operator-token.ts';
import { childNode, } from '../node-access.ts';
import type {
  EstreeNode,
  OperatorName,
  Replacement,
} from '../types.ts';

/**
 Arithmetic operator swap table, mirroring Stryker's ArithmeticOperator.
 */
const ARITHMETIC_SWAPS: Readonly<Record<string, string>> = {
  '+': '-',
  '-': '+',
  '*': '/',
  '/': '*',
  '%': '*',
};

/**
 Equality operator swap table, mirroring Stryker's EqualityOperator
 boundary and negation variants.
 */
const EQUALITY_SWAPS: Readonly<Record<string, readonly string[]>> = {
  '<': [
    '<=',
    '>=',
  ],
  '<=': [
    '<',
    '>',
  ],
  '>': [
    '>=',
    '<=',
  ],
  '>=': [
    '>',
    '<',
  ],
  '==': ['!=',],
  '!=': ['==',],
  '===': ['!==',],
  '!==': ['===',],
};

/**
 Logical operator swap table, mirroring Stryker's LogicalOperator.
 */
const LOGICAL_SWAPS: Readonly<Record<string, string>> = {
  '&&': '||',
  '||': '&&',
  '??': '&&',
};

/**
 Builds one token-span replacement for a binary-like expression.
 
 @param options - Expression node, source, family, and new operator text.
 
 @returns Replacement swapping the operator token.
 
 @example
 ```ts
 tokenSwap({ node, source, operator: 'arithmetic', currentToken: '+', newToken: '-' });
 ```
 */
function tokenSwap(options: {
  readonly node: EstreeNode;
  readonly source: string;
  readonly operator: OperatorName;
  readonly currentToken: string;
  readonly newToken: string;
},): Replacement {
  /**
   Operator token start between operand spans.
   */
  const tokenStart = findOperatorToken({
    source: options.source,
    from: childNode({
      node: options.node,
      key: 'left',
    },)
      .end,
    to: childNode({
      node: options.node,
      key: 'right',
    },)
      .start,
    token: options.currentToken,
  },);

  return {
    start: tokenStart,
    end: tokenStart
      + options.currentToken
        .length,
    text: options.newToken,
    operator: options.operator,
    description: `swapped ${options.currentToken} with ${options.newToken}`,
  };
}

/**
 Emits operator swaps for one BinaryExpression or LogicalExpression.
 
 @param options - Node under inspection with parent and source.
 
 @returns Token-span replacements, possibly empty.
 
 @example
 ```ts
 binaryOperatorReplacements({ node, parent: undefined, source });
 ```
 */
export function binaryOperatorReplacements(options: {
  readonly node: EstreeNode;
  readonly parent?: EstreeNode;
  readonly source: string;
},): readonly Replacement[] {
  /**
   Operator token declared by the expression node.
   */
  const token = options.node
    .operator;

  if ((typeof token) !== 'string')
    return [];

  if (options.node
    .type
    === 'BinaryExpression') {
    /**
     Arithmetic swap for this token, when applicable.
     */
    const arithmetic = ARITHMETIC_SWAPS[token];

    if (arithmetic !== undefined)
      return [tokenSwap({
        node: options.node,
        source: options.source,
        operator: 'arithmetic',
        currentToken: token,
        newToken: arithmetic,
      },),];

    /**
     Equality swap list for this token, when applicable.
     */
    const equality = EQUALITY_SWAPS[token];

    if (equality !== undefined)
      return equality.map(function toSwap(newToken,): Replacement {
        return tokenSwap({
          node: options.node,
          source: options.source,
          operator: 'equality',
          currentToken: token,
          newToken,
        },);
      },);

    return [];
  }

  if (options.node
    .type
    === 'LogicalExpression') {
    /**
     Logical swap for this token, when applicable.
     */
    const logical = LOGICAL_SWAPS[token];

    if (logical !== undefined)
      return [tokenSwap({
        node: options.node,
        source: options.source,
        operator: 'logical',
        currentToken: token,
        newToken: logical,
      },),];
  }

  return [];
}
