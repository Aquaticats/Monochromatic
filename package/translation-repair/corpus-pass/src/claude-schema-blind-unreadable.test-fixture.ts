import {
  ancestorsOf,
  identifierName,
  type TreeNode,
  unwrapped,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import {
  GLOBAL_OBJECTS,
  globalValueShapesIn,
  namesGlobalObject,
} from './claude-schema-blind-global-values.test-fixture.ts';
import type { BlindContext, } from './claude-schema-blind-positions.test-fixture.ts';
import type { FoundShape, } from './claude-schema-blind-spots.test-fixture.ts';
import {
  namesTextCode,
  textCodeShapesIn,
} from './claude-schema-blind-text-code.test-fixture.ts';
import { isTypeWithoutValue, } from './claude-schema-reference.test-fixture.ts';
import {
  keyNameOf,
  nodeAt,
  stringValueOf,
} from './claude-schema-tree.test-fixture.ts';

//region Code the schema search cannot read
// The first total follows `import()` and a call of `require` whose path is one literal. Code that builds a loader,
// reaches `require` by another name, or makes code from text can load a module that carries a builder with no path the
// search could read, so each of these is reported where it is written, whether or not a builder is near it:
// `createRequire` in any form (a call, an import of it, a member of a module), `require` anywhere but as the callee of
// a direct call (`module.require`, an alias, `require.call`, a comma expression), `eval`, and `Function` with or
// without `new`, also as the member of another object, and a member of `globalThis`, `window`, `self` or `global`
// whose key is computed from anything but a string literal (`globalThis['ev' + 'al']`), since it can be either. The
// global object written as a whole value, which a holder can read with such a key, is reported too
// (`claude-schema-blind-global-values.test-fixture.ts`). A word inside a string, a longer name and a type are not
// code. The `vm` module and the constructor of a function, which make code from text as well, are told by their text
// and reported in `claude-schema-blind-text-code.test-fixture.ts`. Three limits stay, held by no report and stated in
// the ledger: a computed key that is no string literal, a pattern that takes `constructor` out of a value, and a
// `Reflect.get` with a key that is no literal reach them without a name the search could tell.

/**
 What a report says of a place that builds a loader.
 */
const MAKES_LOADER = 'createRequire builds a loader the search cannot follow';

/**
 What a report says of a place that reaches `require` by another way than a direct call.
 */
const REACHES_REQUIRE = 'require used beyond a direct call';

/**
 What a report says of a place that runs text as code.
 */
const RUNS_TEXT = 'eval runs text the search cannot read';

/**
 What a report says of a place that makes a function from text.
 */
const BUILDS_FUNCTION = 'Function builds code from text the search cannot read';

/**
 What a report says of a member of the global object whose key is no string the search can read. The key can be
 `eval` or `Function`, and nothing in the file says which.
 */
const COMPUTES_GLOBAL = 'global object member computed from a key the search cannot read';

/**
 The name that stands for a member of the global object with a key the search cannot read. It is no identifier, so no
 file can write it, and no name a file writes can be taken for it.
 */
const COMPUTED_GLOBAL_KEY = '[computed global key]';

/**
 The names that make code the search cannot read, each with what a report says of it.
 */
const UNREADABLE_NAMES: ReadonlyMap<string, string> = new Map(Object.entries({
  createRequire: MAKES_LOADER,
  eval: RUNS_TEXT,
  Function: BUILDS_FUNCTION,
  require: REACHES_REQUIRE,
  [COMPUTED_GLOBAL_KEY]: COMPUTES_GLOBAL,
},),);

/**
 The name of a member of the global object whose key is no string the search can read.

 @param node - member expression read

 @returns Name that stands for it among the unreadable names, empty for any other member

 @example
 ```ts
 const name = computedGlobalNameOf({ node, },);
 // => COMPUTED_GLOBAL_KEY for globalThis[key]
 ```
 */
function computedGlobalNameOf({ node, }: { readonly node: TreeNode; },): string {
  if (node.computed !== true)
    return '';
  /**
   The key when it is a string the search can read, empty otherwise: a name in brackets is a variable's value, not a
   string.
   */
  const key = stringValueOf({
    node: nodeAt({
      node,
      field: 'property',
    },),
  },);
  if (key !== '')
    return '';
  /**
   The object the member is read from, without the marks that leave it the same.
   */
  const { inner: object, } = unwrapped({ node: node.object, },);
  return GLOBAL_OBJECTS.has(identifierName({ node: object, },),) ? COMPUTED_GLOBAL_KEY : '';
}

/**
 The name an identifier or a member computed from a string is written with when it makes code the search cannot read.

 @param node - node read, of any kind

 @returns Name, empty for any node that is neither an identifier nor a member computed from a string, and for a name
 that makes no such code

 @example
 ```ts
 const name = unreadableNameOf({ node, },);
 // => 'eval'
 ```
 */
function unreadableNameOf({ node, }: { readonly node: TreeNode; },): string {
  /**
   The name the node is written with.
   */
  const name = (node.type === 'MemberExpression')
    ? ((node.computed === true)
      ? keyNameOf({ key: nodeAt({
        node,
        field: 'property',
      },), },)
      : '')
    : identifierName({ node, },);
  if (node.type === 'MemberExpression') {
    /**
     The name of a member of the global object whose key is no string, empty for any other member.
     */
    const computedGlobal = computedGlobalNameOf({ node, },);
    if (computedGlobal !== '')
      return computedGlobal;
  }
  return UNREADABLE_NAMES.has(name,) ? name : '';
}

/**
 Whether a node is written with a name that makes code the search cannot read, wherever it sits.

 @param node - node read, of any kind

 @returns Whether the node is an identifier of that name, or a member computed from a string of that name

 @example
 ```ts
 const names = namesUnreadableCode({ node, },);
 ```
 */
export function namesUnreadableCode({ node, }: { readonly node: TreeNode; },): boolean {
  /**
   Whether the node is the global object or a place that loads `vm` or reads a constructor.
   */
  const isOtherPlace = namesGlobalObject({ node, },) || namesTextCode({ node, },);
  return (unreadableNameOf({ node, },) !== '') || isOtherPlace;
}

/**
 Whether a node is the identifier `require` as the callee of a direct call, which is the one place `require` is read.

 @param name - name that makes the code, as `unreadableNameOf` reads it

 @param node - node read

 @param parents - each node's parent

 @returns Whether the node is that identifier and a call takes it as its callee

 @example
 ```ts
 const isCall = isRequireCall({ name: 'require', node, parents, },);
 ```
 */
function isRequireCall(
  {
    name,
    node,
    parents,
  }: {
    readonly name: string;
    readonly node: TreeNode;
    readonly parents: ReadonlyMap<TreeNode, TreeNode>;
  },
): boolean {
  if ((name !== 'require') || (node.type !== 'Identifier'))
    return false;
  /**
   The node that holds the identifier.
   */
  const holder = parents.get(node,);
  return (holder?.type === 'CallExpression') && (holder.callee === node);
}

/**
 The places in a file that are written with a name that makes code the search cannot read.

 @param context - what the file's names are read against

 @param nodes - every node of the file

 @returns One entry per place, outside types, a direct call of `require` excepted

 @example
 ```ts
 const found = namedShapesIn({ context, nodes: nodesUnder({ root: program, },), },);
 ```
 */
function namedShapesIn(
  {
    context,
    nodes,
  }: {
    readonly context: BlindContext;
    readonly nodes: readonly TreeNode[];
  },
): readonly FoundShape[] {
  return nodes.flatMap(function shapesOfNode(node,): readonly FoundShape[] {
    /**
     The name that makes the code, empty for a node that does not.
     */
    const name = unreadableNameOf({ node, },);
    if (name === '')
      return [];
    if (isRequireCall({
      name,
      node,
      parents: context.parents,
    },))
      return [];
    if (ancestorsOf({
      node,
      parents: context.parents,
    },)
      .some(function isType(ancestor,): boolean {
        return isTypeWithoutValue({ node: ancestor, },);
      },))
      return [];
    return [
      {
        shape: 'code-the-search-cannot-read',
        node,
        position: UNREADABLE_NAMES.get(name,) ?? name,
      },
    ];
  },);
}

/**
 The places in a file that make code the search cannot read: a name that makes it, the global object written as a
 whole value, and `vm` and the constructor of a function.

 @param context - what the file's names are read against

 @param nodes - every node of the file

 @returns One entry per place

 @example
 ```ts
 const found = unreadableShapesIn({ context, nodes: nodesUnder({ root: program, },), },);
 ```
 */
export function unreadableShapesIn(
  {
    context,
    nodes,
  }: {
    readonly context: BlindContext;
    readonly nodes: readonly TreeNode[];
  },
): readonly FoundShape[] {
  return [
    ...namedShapesIn({
      context,
      nodes,
    },),
    ...globalValueShapesIn({ context, },),
    ...textCodeShapesIn({
      context,
      nodes,
    },),
  ];
}

//endregion Code the schema search cannot read
