/**
 Literal mutations: booleans, negations, strings, and templates.
 
 @example
 ```ts
 literalReplacements({ node, parent, source });
 ```
 */

import { childNode, } from '../node-access.ts';
import type {
  EstreeNode,
  Replacement,
} from '../types.ts';

/**
 Filler text injected when an empty string or template is mutated,
 mirroring Stryker's "Stryker was here!" marker under our own name.
 */
export const FILLER_TEXT = 'mutation-test was here!';

/**
 Parent types whose string children must never be mutated: module
 specifiers and import attributes break resolution rather than logic,
 and TS literal types are type-position only.
 */
const PROTECTED_STRING_PARENTS: ReadonlySet<string> = new Set([
  'ImportDeclaration',
  'ImportExpression',
  'ExportNamedDeclaration',
  'ExportAllDeclaration',
  'ImportAttribute',
  'TSLiteralType',
],);

/**
 Returns whether a string literal sits in a position that must not be
 mutated: protected parents, non-computed object keys, and directives.
 
 @param options - Literal node and its structural parent.
 
 @returns Whether mutation must be skipped.
 
 @example
 ```ts
 isProtectedString({ node, parent });
 ```
 */
function isProtectedString(options: {
  readonly node: EstreeNode;
  readonly parent?: EstreeNode;
},): boolean {
  if (options.parent === undefined)
    return false;

  if (PROTECTED_STRING_PARENTS.has(options.parent
    .type,))
    return true;

  if ((options.parent
    .type
    === 'Property')
    && (options.parent
      .key
      === options.node)
    && (options.parent
      .computed
      !== true))
    return true;

  return (options.parent
    .type
    === 'ExpressionStatement')
    && ((typeof options.parent
      .directive) === 'string');
}

/**
 Emits literal replacements for one node.
 
 Booleans swap; `!expr` drops its negation (Stryker files that under
 BooleanLiteral); strings and templates empty out, or gain filler text
 when already empty. Tagged template quasis stay untouched because the
 tag may depend on exact quasi structure.
 
 @param options - Node under inspection with parent and source.
 
 @returns Replacements, possibly empty.
 
 @example
 ```ts
 literalReplacements({ node, parent, source });
 ```
 */
export function literalReplacements(options: {
  readonly node: EstreeNode;
  readonly parent?: EstreeNode;
  readonly source: string;
},): readonly Replacement[] {
  if ((options.node
    .type
    === 'UnaryExpression')
    && (options.node
      .operator
      === '!'))
    return [{
      start: options.node
        .start,
      end: options.node
        .end,
      text: options.source
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
        ),
      operator: 'boolean',
      description: 'removed ! negation',
    },];

  if (options.node
    .type
    === 'Literal') {
    /**
     Runtime literal value discriminating boolean, string, and regex.
     */
    const {value} = options.node;

    if ((typeof value) === 'boolean')
      return [{
        start: options.node
          .start,
        end: options.node
          .end,
        text: value ? 'false' : 'true',
        operator: 'boolean',
        description: `swapped ${String(value,)} with ${String(!value,)}`,
      },];

    if (((typeof value) === 'string')
      && (!isProtectedString({
        node: options.node,
        ...(options.parent === undefined ? {} : { parent: options.parent, }),
      },)))
      return [{
        start: options.node
          .start,
        end: options.node
          .end,
        text: value === '' ? `'${FILLER_TEXT}'` : "''",
        operator: 'string',
        description: value === ''
          ? 'filled empty string'
          : 'emptied string',
      },];

    return [];
  }

  if ((options.node
    .type
    === 'TemplateLiteral')
    && (options.parent
      ?.type
      !== 'TaggedTemplateExpression')) {
    /**
     Whether the template already produces an empty string.
     */
    const isEmpty = (options.node
      .end
      - options.node
        .start) === 2;

    return [{
      start: options.node
        .start,
      end: options.node
        .end,
      text: isEmpty ? `\`${FILLER_TEXT}\`` : '``',
      operator: 'string',
      description: isEmpty
        ? 'filled empty template'
        : 'emptied template',
    },];
  }

  return [];
}
