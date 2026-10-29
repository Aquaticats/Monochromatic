/**
 Conditional mutations: forcing boolean contexts to constants.
 
 Mirrors Stryker's ConditionalExpression mutator: comparison and logical
 expressions plus if/ternary tests become `true` and `false`; loop tests
 become `false` only, since a forced-true loop can only end in timeout.
 
 @example
 ```ts
 conditionalReplacements({ node, parent: undefined, source });
 ```
 */

import {
  childNode,
  hasChildNode,
} from '../node-access.ts';
import type {
  EstreeNode,
  Replacement,
} from '../types.ts';

/**
 Comparison operator tokens whose expressions get boolean forcing.
 */
const COMPARISON_TOKENS: ReadonlySet<string> = new Set([
  '==',
  '!=',
  '===',
  '!==',
  '<',
  '<=',
  '>',
  '>=',
],);

/**
 Logical operator tokens whose expressions get boolean forcing.
 */
const LOGICAL_TOKENS: ReadonlySet<string> = new Set([
  '&&',
  '||',
],);

/**
 Statement types whose `test` child is forced to `false` only.
 */
const LOOP_TYPES: ReadonlySet<string> = new Set([
  'WhileStatement',
  'DoWhileStatement',
  'ForStatement',
],);

/**
 Builds boolean-forcing replacements over one span.
 
 @param options - Span to force and constants to force it to.
 
 @returns One replacement per constant.
 
 @example
 ```ts
 forceBoolean({ start: 3, end: 8, texts: ['true', 'false'] });
 ```
 */
function forceBoolean(options: {
  readonly start: number;
  readonly end: number;
  readonly texts: readonly string[];
},): readonly Replacement[] {
  return options.texts
    .map(function toReplacement(text,): Replacement {
      return {
        start: options.start,
        end: options.end,
        text,
        operator: 'conditional',
        description: `forced condition to ${text}`,
      };
    },);
}

/**
 Emits boolean-forcing replacements for one node.
 
 @param options - Node under inspection with parent and source.
 
 @returns Replacements, possibly empty.
 
 @example
 ```ts
 conditionalReplacements({ node: ifStatement, parent: undefined, source });
 ```
 */
export function conditionalReplacements(options: {
  readonly node: EstreeNode;
  readonly parent?: EstreeNode;
  readonly source: string;
},): readonly Replacement[] {
  /**
   Operator token for expression nodes, absent on statements.
   */
  const token = options.node
    .operator;
  /**
   Operator token narrowed to text for set membership checks.
   */
  const tokenText = (typeof token) === 'string' ? token : '';

  if ((options.node
    .type
    === 'BinaryExpression')
    && COMPARISON_TOKENS.has(tokenText,))
    return forceBoolean({
      start: options.node
        .start,
      end: options.node
        .end,
      texts: [
        'true',
        'false',
      ],
    },);

  if ((options.node
    .type
    === 'LogicalExpression')
    && LOGICAL_TOKENS.has(tokenText,))
    return forceBoolean({
      start: options.node
        .start,
      end: options.node
        .end,
      texts: [
        'true',
        'false',
      ],
    },);

  if ((options.node
    .type
    === 'IfStatement')
    || (options.node
      .type
      === 'ConditionalExpression')) {
    if (!hasChildNode({
      node: options.node,
      key: 'test',
    },))
      return [];

    /**
     Condition expression under the statement or ternary.
     */
    const test = childNode({
      node: options.node,
      key: 'test',
    },);

    return forceBoolean({
      start: test.start,
      end: test.end,
      texts: [
        'true',
        'false',
      ],
    },);
  }

  if (LOOP_TYPES.has(options.node
    .type,)) {
    if (!hasChildNode({
      node: options.node,
      key: 'test',
    },))
      return [];

    /**
     Loop condition expression; `for (;;)` has none.
     */
    const test = childNode({
      node: options.node,
      key: 'test',
    },);

    return forceBoolean({
      start: test.start,
      end: test.end,
      texts: ['false',],
    },);
  }

  return [];
}
