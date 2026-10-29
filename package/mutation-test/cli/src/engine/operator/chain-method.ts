/**
 Optional chaining removal and method expression mutations.
 
 @example
 ```ts
 chainMethodReplacements({ node, parent: undefined, source });
 ```
 */

import { findOperatorToken, } from '../operator-token.ts';
import { childNode, } from '../node-access.ts';
import type {
  EstreeNode,
  Replacement,
} from '../types.ts';

/**
 Method name swap pairs, mirroring Stryker's MethodExpression table.
 */
const METHOD_SWAPS: Readonly<Record<string, string>> = {
  startsWith: 'endsWith',
  endsWith: 'startsWith',
  every: 'some',
  some: 'every',
  toLowerCase: 'toUpperCase',
  toUpperCase: 'toLowerCase',
  toLocaleLowerCase: 'toLocaleUpperCase',
  toLocaleUpperCase: 'toLocaleLowerCase',
  trimStart: 'trimEnd',
  trimEnd: 'trimStart',
  min: 'max',
  max: 'min',
};

/**
 Method names whose whole call collapses to its receiver, mirroring
 Stryker's MethodExpression removal list.
 */
const METHOD_REMOVALS: ReadonlySet<string> = new Set([
  'filter',
  'reverse',
  'slice',
  'sort',
  'substr',
  'substring',
  'trim',
],);

/**
 Emits optional-chaining removal for one optional member or call.
 
 `a?.b` becomes `a.b`, `a?.[b]` becomes `a[b]`, and `f?.()` becomes
 `f()`; only the `?.` token span changes.
 
 @param options - Optional node and source.
 
 @returns Token replacement for the `?.` occurrence.
 
 @example
 ```ts
 optionalRemoval({ node: optionalMember, source });
 ```
 */
function optionalRemoval(options: {
  readonly node: EstreeNode;
  readonly source: string;
},): Replacement {
  /**
   Left neighbour whose end bounds the `?.` token scan.
   */
  const scanFrom = childNode({
    node: options.node,
    key: options.node
      .type
      === 'CallExpression' ? 'callee' : 'object',
  },)
    .end;
  /**
   Start offset of the `?.` token.
   */
  const tokenStart = findOperatorToken({
    source: options.source,
    from: scanFrom,
    to: options.node
      .end,
    token: '?.',
  },);
  /**
   Whether plain `.` must replace the token; computed members and calls
   drop it entirely.
   */
  const needsDot = (options.node
    .type
    === 'MemberExpression')
    && (options.node
      .computed
      !== true);

  return {
    start: tokenStart,
    end: tokenStart + 2,
    text: needsDot ? '.' : '',
    operator: 'optional-chaining',
    description: 'removed ?. optional chaining',
  };
}

/**
 Emits chaining and method replacements for one node.
 
 @param options - Node under inspection with parent and source.
 
 @returns Replacements, possibly empty.
 
 @example
 ```ts
 chainMethodReplacements({ node: callExpression, parent: undefined, source });
 ```
 */
export function chainMethodReplacements(options: {
  readonly node: EstreeNode;
  readonly parent?: EstreeNode;
  readonly source: string;
},): readonly Replacement[] {
  /**
   Collected replacements for this node.
   */
  const replacements: Replacement[] = [];

  if (((options.node
    .type
    === 'MemberExpression')
    || (options.node
      .type
      === 'CallExpression'))
    && (options.node
      .optional
      === true))
    replacements.push(optionalRemoval({
      node: options.node,
      source: options.source,
    },),);

  if (options.node
    .type
    === 'CallExpression') {
    /**
     Callee expression, a member expression for method calls.
     */
    const callee = childNode({
      node: options.node,
      key: 'callee',
    },);

    if ((callee.type === 'MemberExpression')
      && (callee.computed !== true)) {
      /**
       Method name identifier under the member expression.
       */
      const property = childNode({
        node: callee,
        key: 'property',
      },);
      /**
       Called method name.
       */
      const {name} = property;

      if ((typeof name) === 'string') {
        /**
         Swapped method name for this call, when in the family.
         */
        const swapped = METHOD_SWAPS[name];

        if (swapped !== undefined)
          replacements.push({
            start: property.start,
            end: property.end,
            text: swapped,
            operator: 'method',
            description: `swapped .${name} with .${swapped}`,
          },);

        if (METHOD_REMOVALS.has(name,)) {
          /**
           Receiver expression the collapsed call reduces to.
           */
          const object = childNode({
            node: callee,
            key: 'object',
          },);
          replacements.push({
            start: options.node
              .start,
            end: options.node
              .end,
            text: options.source
              .slice(
              object.start,
              object.end,
              ),
            operator: 'method',
            description: `removed .${name} call`,
          },);
        }
      }
    }
  }

  return replacements;
}
