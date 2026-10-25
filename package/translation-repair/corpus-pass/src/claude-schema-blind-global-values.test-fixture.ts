import {
  identifierName,
  nodesUnder,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import {
  type BlindContext,
  outermostValueOf,
} from './claude-schema-blind-positions.test-fixture.ts';
import { isBoundAround, } from './claude-schema-blind-scopes.test-fixture.ts';
import type { FoundShape, } from './claude-schema-blind-spots.test-fixture.ts';
import {
  nodeAt,
  nodesAt,
  pathOf,
} from './claude-schema-tree.test-fixture.ts';

//region The global object written as a whole value
// A member of the global object computed from anything but a string literal can name `eval` or `Function`
// (`claude-schema-blind-unreadable.test-fixture.ts` reports it). The global object can also be held first and read
// through the holder: kept in a constant, taken apart by a pattern with a computed key, put in an array, handed to
// `Reflect.get`, `Object.assign`, `call` or any function, chosen by a conditional, a logical operand or a comma
// expression, given as a default, exported in an export list (`export { globalThis as gt }`) or exported as the
// default. The check reports the global object (`globalThis`, `window`, `self` or `global`) wherever a file writes it
// as a whole value that no scope binds again. The object of a member (`globalThis.fetch`, `globalThis['fetch']`), the
// operand of `typeof`, a type, a key of an object, a member read off another object and a parameter or a constant of
// the same name are not the global object as a value.

/**
 What a report says of the global object written as a whole value.
 */
const GLOBAL_AS_VALUE = 'global object used as a value the search cannot follow';

/**
 The names the global object is written with, in the runtimes the family runs in and in the browsers.
 */
export const GLOBAL_OBJECTS: ReadonlySet<string> = new Set([
  'global',
  'globalThis',
  'self',
  'window',
],);

/**
 The name of the built-in object a method is read off, whatever marks the object is written under and whether or not it
 is read off the global object.

 @param node - object the method is read off, of any kind

 @returns Name, empty when the object is neither a name nor a name read off the global object

 @example
 ```ts
 const name = builtinNameOf({ node: callee.object, },);
 // => 'Object' for `Object`, `(Object as any)` and `globalThis.Object`
 ```
 */
export function builtinNameOf({ node, }: { readonly node: unknown; },): string {
  /**
   The names the object is written with.
   */
  const path = pathOf({ node, },);
  if (path.length === 1)
    return path[0] ?? '';
  return ((path.length === 2) && GLOBAL_OBJECTS.has(path[0] ?? '',))
    ? (path[1] ?? '')
    : '';
}

/**
 Whether a node is written with the name of the global object.

 @param node - node read, of any kind

 @returns Whether the node is an identifier of that name

 @example
 ```ts
 const isGlobal = namesGlobalObject({ node, },);
 ```
 */
export function namesGlobalObject({ node, }: { readonly node: TreeNode; },): boolean {
  return (node.type === 'Identifier') && GLOBAL_OBJECTS.has(identifierName({ node, },),);
}

/**
 Whether a name reference stands for the global object as a value: no scope around it and no top-level binding of the
 file makes it another thing, it is not the object of a member, and it is not the operand of `typeof`.

 @param context - what the file's names are read against

 @param node - identifier that starts a reference

 @returns Whether the identifier is the global object as a whole value

 @example
 ```ts
 const isValue = isGlobalValue({ context, node, },);
 ```
 */
function isGlobalValue(
  {
    context,
    node,
  }: {
    readonly context: BlindContext;
    readonly node: TreeNode;
  },
): boolean {
  /**
   The name the identifier is written with.
   */
  const name = identifierName({ node, },);
  /**
   Whether a scope around the identifier binds the name.
   */
  const isBoundInScope = isBoundAround({
    node,
    name,
    parents: context.parents,
  },);
  /**
   Whether the top level of the file binds the name.
   */
  const isBoundAtTop = context.facts
    .bindings
    .has(name,);
  /**
   Whether a scope around the identifier or the top level of the file binds the name.
   */
  const isBound = isBoundInScope || isBoundAtTop;
  /**
   The expression under the marks that leave it the same value.
   */
  const top = outermostValueOf({
    node,
    parents: context.parents,
  },);
  /**
   The node that holds the identifier.
   */
  const holder = context.parents
    .get(top,);
  /**
   Whether the identifier is the object of a member, or the operand of `typeof`.
   */
  const isRead = ((holder?.type === 'MemberExpression') && (holder.object === top))
    || ((holder?.type === 'UnaryExpression') && (holder.operator === 'typeof'));
  return (!isBound) && (!isRead);
}

/**
 The identifiers an export of the file hands out under the name of the global object, which no reference visit holds
 because an export list and a default export are no value position: the names in `export { globalThis as gt }` and the
 value of `export default globalThis`. An export from another module names that module's export, not the global object.

 @param context - what the file's names are read against

 @returns Identifiers in order, none that the file binds at its top level

 @example
 ```ts
 const exported = exportedGlobalsIn({ context, },);
 ```
 */
function exportedGlobalsIn({ context, }: { readonly context: BlindContext; },): readonly TreeNode[] {
  /**
   The program of the file.
   */
  const { program, } = context.facts;
  return nodesUnder({ root: program, },)
    .flatMap(function exportedBy(node,): readonly TreeNode[] {
      if (node.type === 'ExportDefaultDeclaration')
        return [nodeAt({
          node,
          field: 'declaration',
        },),];
      if ((node.type !== 'ExportNamedDeclaration') || (node.source !== null))
        return [];
      return nodesAt({
        node,
        field: 'specifiers',
      },)
        .map(function localOf(specifier,): TreeNode {
          return nodeAt({
            node: specifier,
            field: 'local',
          },);
        },);
    },)
    .filter(function isUnboundGlobal(candidate,): boolean {
      return namesGlobalObject({ node: candidate, },)
        && (!context.facts
          .bindings
          .has(identifierName({ node: candidate, },),));
    },);
}

/**
 The places in a file that write the global object as a whole value.

 @param context - what the file's names are read against

 @returns One entry per place

 @example
 ```ts
 const found = globalValueShapesIn({ context, },);
 ```
 */
export function globalValueShapesIn({ context, }: { readonly context: BlindContext; },): readonly FoundShape[] {
  return [
    ...context.references
      .flatMap(function shapeOfReference({ node, },): readonly FoundShape[] {
        /**
         Whether the reference is the global object written as a whole value.
         */
        const isValue = namesGlobalObject({ node, },) && isGlobalValue({
          context,
          node,
        },);
        return isValue
          ? [
            {
              shape: 'code-the-search-cannot-read',
              node,
              position: GLOBAL_AS_VALUE,
            },
          ]
          : [];
      },),
    ...exportedGlobalsIn({ context, },)
      .map(function shapeOfExport(node,): FoundShape {
        return {
          shape: 'code-the-search-cannot-read',
          node,
          position: GLOBAL_AS_VALUE,
        };
      },),
  ];
}

//endregion The global object written as a whole value
