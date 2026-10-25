import {
  identifierName,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import type { FoundShape, } from './claude-schema-blind-spots.test-fixture.ts';
import {
  keyNameOf,
  moduleLoadOf,
  nodeAt,
  nodesAt,
  pathOf,
  stringValueOf,
} from './claude-schema-tree.test-fixture.ts';

//region Code made from text by the vm module and by the constructor of a function
// Besides `eval` and `Function` (`claude-schema-blind-unreadable.test-fixture.ts`), code is made from text by the `vm`
// module and by the constructor of a function, which is the `Function` constructor. Both can be told by a read of the
// text. The module is told by its specifier, `vm` or `node:vm`, in an import, an export from, a call of `require`, an
// `import()`, a call of `process.getBuiltinModule` (also read off the global object) or the module of an import-equals
// declaration; an import, an export or an import-equals declaration that names types only (`import type`) loads
// nothing and names no module. A
// pattern that takes `getBuiltinModule` out of a value is reported, since the function is then called by a free name,
// and so is a call of `binding` or `internalBinding` with the string `contextify`, the binding of Node that makes
// scripts, which is a route to `vm.Script` without the module. The constructor is told by a member
// named `constructor` written with a dot or with a string literal in brackets, also through `?.`, and by `Reflect.get`,
// `Reflect.getOwnPropertyDescriptor` and `Object.getOwnPropertyDescriptor` with the string `constructor` as the key. A
// key that is no literal, a pattern that takes `constructor` out of a value, and a method that is written through an
// alias (`const R = Reflect; R.get(f, "constructor")`) are not told, and are limits of the search that the ledger
// states, and so is a loader that `process.getBuiltinModule` was assigned to and is called by another name.

/**
 What a report says of a place that loads the `vm` module.
 */
const LOADS_VM = 'vm runs text the search cannot read';

/**
 What a report says of a pattern that takes `getBuiltinModule` out of a value, since the function it names can load
 `vm` under any name.
 */
const TAKES_LOADER = 'getBuiltinModule taken out of a value loads modules the search cannot read';

/**
 What a report says of a call of the binding of Node that makes scripts.
 */
const BINDS_CONTEXTIFY = 'contextify binding runs text the search cannot read';

/**
 The names a call may read the binding of Node through: `process.binding` and `internalBinding`, and the same
 function called by a free name.
 */
const BINDING_CALLEES: ReadonlySet<string> = new Set([
  'binding',
  'internalBinding',
],);

/**
 What a report says of a place that reads the constructor of a value.
 */
const READS_CONSTRUCTOR = 'constructor of a function builds code from text the search cannot read';

/**
 The specifiers that name the `vm` module.
 */
const VM_SPECIFIERS: ReadonlySet<string> = new Set([
  'node:vm',
  'vm',
],);

/**
 The kinds of node that carry the path of a module in their `source` field.
 */
const SOURCE_KINDS: ReadonlySet<string> = new Set([
  'ExportAllDeclaration',
  'ExportNamedDeclaration',
  'ImportDeclaration',
],);

/**
 The module a node names, when the node is an import, an export from, a load or the module of an import-equals
 declaration. An import or an export that names types only loads nothing, so it names no module.

 @param node - node read, of any kind

 @returns Specifier as written, empty for any other node and for a path that is no single literal

 @example
 ```ts
 const specifier = moduleNamedBy({ node, },);
 ```
 */
function moduleNamedBy({ node, }: { readonly node: TreeNode; },): string {
  if (SOURCE_KINDS.has(node.type,)) {
    return ((node.importKind === 'type') || (node.exportKind === 'type'))
      ? ''
      : stringValueOf({ node: node.source, },);
  }
  if (node.type === 'TSImportEqualsDeclaration') {
    /**
     The reference to the module the declaration names, or what else it names.
     */
    const reference = nodeAt({
      node,
      field: 'moduleReference',
    },);
    return ((node.importKind === 'type') || (reference.type !== 'TSExternalModuleReference'))
      ? ''
      : stringValueOf({ node: reference.expression, },);
  }
  if ((node.type !== 'ImportExpression') && (node.type !== 'CallExpression'))
    return '';
  /**
   The load the node is, if it is one.
   */
  const load = moduleLoadOf({ node, },);
  return load.isLoad ? load.specifier : '';
}

/**
 Whether a member is read by the name `constructor`, with a dot or with a string literal.

 @param node - member expression

 @returns Whether the key is `constructor`

 @example
 ```ts
 const reads = readsConstructor({ node: member, },);
 ```
 */
function readsConstructor({ node, }: { readonly node: TreeNode; },): boolean {
  /**
   The key of the member.
   */
  const key = nodeAt({
    node,
    field: 'property',
  },);
  return (node.computed === true)
    ? (stringValueOf({ node: key, },) === 'constructor')
    : (identifierName({ node: key, },) === 'constructor');
}

/**
 The methods that read a member of a value by a key given as an argument, by the object they are read off:
 `Reflect.get`, `Reflect.getOwnPropertyDescriptor` and `Object.getOwnPropertyDescriptor`.
 */
const KEY_READERS: ReadonlyMap<string, ReadonlySet<string>> = new Map([
  [
    'Object',
    new Set(['getOwnPropertyDescriptor',],),
  ],
  [
    'Reflect',
    new Set([
      'get',
      'getOwnPropertyDescriptor',
    ],),
  ],
]);

/**
 Whether a call reads a member by a key and the key is the string `constructor`.

 @param node - call expression

 @returns Whether the callee is one of the methods that read a member by a key and the second argument is that string

 @example
 ```ts
 const reads = reflectsConstructor({ node: call, },);
 ```
 */
function reflectsConstructor({ node, }: { readonly node: TreeNode; },): boolean {
  /**
   The callee of the call.
   */
  const callee = nodeAt({
    node,
    field: 'callee',
  },);
  /**
   The second argument of the call, absent when it has none.
   */
  const key = nodesAt({
    node,
    field: 'arguments',
  },)
    .at(1,);
  if ((callee.type !== 'MemberExpression') || (node.type !== 'CallExpression'))
    return false;
  if (callee.computed === true)
    return false;
  /**
   The methods of the object the callee reads from that read a member by a key.
   */
  const readers = KEY_READERS.get(identifierName({ node: callee.object, },),);
  /**
   Whether the callee is a method that reads a member by a key.
   */
  const isKeyReader = readers?.has(identifierName({ node: callee.property, },),) ?? false;
  if (!isKeyReader)
    return false;
  return (key !== undefined) && (stringValueOf({ node: key, },) === 'constructor');
}

/**
 Whether a call is `process.getBuiltinModule` of the module `vm`.

 @param node - node read, of any kind

 @returns Whether the node is such a call with the specifier as a string literal

 @example
 ```ts
 const loads = getsBuiltinVm({ node, },);
 ```
 */
function getsBuiltinVm({ node, }: { readonly node: TreeNode; },): boolean {
  if (node.type !== 'CallExpression')
    return false;
  /**
   The callee of the call.
   */
  const callee = nodeAt({
    node,
    field: 'callee',
  },);
  /**
   The first argument of the call, absent when it has none.
   */
  const specifier = nodesAt({
    node,
    field: 'arguments',
  },)
    .at(0,);
  /**
   The names the callee reads through, whatever marks the object is written under.
   */
  const calleePath = pathOf({ node: callee, },);
  /**
   Whether the callee is `process.getBuiltinModule`, whether `process` is read off the global object or not.
   */
  const isGetter = (calleePath.at(-1,) === 'getBuiltinModule') && (calleePath.at(-2,) === 'process');
  /**
   Whether the first argument is a string literal that names `vm`.
   */
  const isVm = (specifier !== undefined) && VM_SPECIFIERS.has(stringValueOf({ node: specifier, },),);
  return isGetter && isVm;
}

/**
 Whether a pattern takes `getBuiltinModule` out of a value.

 @param node - node read, of any kind

 @returns Whether the node is an object pattern with a property whose key is `getBuiltinModule`

 @example
 ```ts
 const takes = takesLoader({ node, },);
 ```
 */
function takesLoader({ node, }: { readonly node: TreeNode; },): boolean {
  if (node.type !== 'ObjectPattern')
    return false;
  return nodesAt({
    node,
    field: 'properties',
  },)
    .some(function isLoaderKey(property,): boolean {
      return (property.type === 'Property') && (keyNameOf({ key: nodeAt({
        node: property,
        field: 'key',
      },), },) === 'getBuiltinModule');
    },);
}

/**
 Whether a call is a call of the binding of Node that makes scripts.

 @param node - node read, of any kind

 @returns Whether the node is a call of `binding` or `internalBinding`, read off any object, with the string
 `contextify` as its first argument

 @example
 ```ts
 const binds = bindsContextify({ node, },);
 ```
 */
function bindsContextify({ node, }: { readonly node: TreeNode; },): boolean {
  if (node.type !== 'CallExpression')
    return false;
  /**
   The first argument of the call, absent when it has none.
   */
  const first = nodesAt({
    node,
    field: 'arguments',
  },)
    .at(0,);
  /**
   The names the callee reads through.
   */
  const calleePath = pathOf({ node: nodeAt({
    node,
    field: 'callee',
  },), },);
  /**
   The name the callee ends in.
   */
  const calleeName = calleePath.at(-1,) ?? '';
  return BINDING_CALLEES.has(calleeName,) && (stringValueOf({ node: first, },) === 'contextify');
}

/**
 What a node says about code made from text.

 @param node - node read, of any kind

 @returns Report text for a node that loads `vm` or reads a constructor, empty for any other node

 @example
 ```ts
 const position = textCodePositionOf({ node, },);
 ```
 */
function textCodePositionOf({ node, }: { readonly node: TreeNode; },): string {
  if (VM_SPECIFIERS.has(moduleNamedBy({ node, },),) || getsBuiltinVm({ node, },))
    return LOADS_VM;
  if (takesLoader({ node, },))
    return TAKES_LOADER;
  if (bindsContextify({ node, },))
    return BINDS_CONTEXTIFY;
  if ((node.type === 'MemberExpression') && readsConstructor({ node, },))
    return READS_CONSTRUCTOR;
  return reflectsConstructor({ node, },) ? READS_CONSTRUCTOR : '';
}

/**
 Whether a node loads `vm` or reads a constructor, wherever it sits.

 @param node - node read, of any kind

 @returns Whether the node is such a place

 @example
 ```ts
 const names = namesTextCode({ node, },);
 ```
 */
export function namesTextCode({ node, }: { readonly node: TreeNode; },): boolean {
  return textCodePositionOf({ node, },) !== '';
}

/**
 The places in a file that load `vm` or read the constructor of a value. A type holds none of the nodes that do.

 @param nodes - every node of the file

 @returns One entry per place

 @example
 ```ts
 const found = textCodeShapesIn({ nodes: nodesUnder({ root: program, },), },);
 ```
 */
export function textCodeShapesIn({ nodes, }: { readonly nodes: readonly TreeNode[]; },): readonly FoundShape[] {
  return nodes.flatMap(function shapesOfNode(node,): readonly FoundShape[] {
    /**
     What the node says about code made from text, empty when nothing.
     */
    const position = textCodePositionOf({ node, },);
    return (position === '')
      ? []
      : [
        {
          shape: 'code-the-search-cannot-read',
          node,
          position,
        },
      ];
  },);
}

//endregion Code made from text by the vm module and by the constructor of a function
