import {
  ancestorsOf,
  identifierName,
  isTreeNode,
  memberName,
  nodesUnder,
  parentsOf,
  parseSource,
  type SourceText,
  type TreeNode,
} from './source-scan.test-fixture.ts';

//region Entry logger finding
// WHAT THE ENTRY-LOGGER SCAN READS (ledger B963), kept apart from the scan so the finding can be run over any text.
// A file's tree is read for every call of `tagged`, under whatever name the file binds it (an import with another local
// name, a const alias), whose `tag` names an entry: the tag, with the const bindings of the file followed, holds the
// identifier `entryId` or reads a member called `id` or `entryId`, in a template literal, a key of any order or any
// spacing. Such a call is found unless it is the `l` of a call of `sliceTagged`, under its own name or an alias, with
// any parenthesis or assertion between them.

/**
 Name the pass imports `tagged` under, which a file is taken to bind without importing it.
 */
const TAGGED = 'tagged';

/**
 Name the pass imports the wrapper under.
 */
export const SLICE_TAGGED = 'sliceTagged';

/**
 Identifier that names an entry in the pass.
 */
const ENTRY_IDENTIFIER = 'entryId';

/**
 Members that read an entry's id: `entry.id`, `pair.id`, `slice.entryId`.
 */
const ENTRY_MEMBERS: ReadonlySet<string> = new Set([
  'id',
  ENTRY_IDENTIFIER,
],);

/**
 Node kinds that wrap an expression without changing the value it holds.
 */
const WRAPPING_KINDS: ReadonlySet<string> = new Set([
  'ParenthesizedExpression',
  'TSAsExpression',
  'TSNonNullExpression',
  'TSSatisfiesExpression',
],);

/**
 What `findIndex` answers when no element fits.
 */
const NOT_FOUND = -1;

/**
 A const name and the expression the file sets it to.
 */
type ConstBinding = readonly [
  string,
  TreeNode,
];

/**
 The nodes a field of a node holds, none for a field that is no list.

 @param value - field of a node

 @returns Tree nodes in the list

 @example
 ```ts
 const properties = nodeListOf({ value: options.properties, },);
 ```
 */
function nodeListOf({ value, }: { readonly value: unknown; },): readonly TreeNode[] {
  return Array.isArray(value,)
    ? value.filter(function isNode(item,): item is TreeNode {
      return isTreeNode(item,);
    },)
    : [];
}

/**
 Names a file binds a function to: its own name, the local names of an import of it, and the names of const aliases.

 @param nodes - every node of the file

 @param imported - function's name where it is exported

 @returns Names a call of it can go by in the file

 @example
 ```ts
 const names = localNamesOf({ nodes, imported: 'tagged', },); // Set { 'tagged', 'makeLogger', 'make' }
 ```
 */
function localNamesOf(
  {
    nodes,
    imported,
  }: {
    readonly nodes: readonly TreeNode[];
    readonly imported: string;
  },
): ReadonlySet<string> {
  /**
   Names bound by an import of the function under another local name.
   */
  const importedAs = nodes
    .filter(function isImportOf(node,): boolean {
      return (node.type === 'ImportSpecifier') && (identifierName({ node: node.imported, },) === imported);
    },)
    .map(function localName(node,): string {
      return identifierName({ node: node.local, },);
    },);

  /**
   Const declarations that bind a name to another name: `const make = tagged`.
   */
  const aliasings = nodes.filter(function isAliasing(node,): boolean {
    return (node.type === 'VariableDeclarator')
      && (identifierName({ node: node.id, },) !== '')
      && (identifierName({ node: node.init, },) !== '');
  },);

  /**
   The names found so far.
   */
  const names = new Set([
    imported,
    ...importedAs,
  ],);
  // Aliases of aliases: pass over the declarations until one pass adds nothing.
  for (let grown = true; grown;) {
    grown = false;
    for (const aliasing of aliasings) {
      /**
       The name the declaration sets.
       */
      const alias = identifierName({ node: aliasing.id, },);

      /**
       The name it sets it from.
       */
      const source = identifierName({ node: aliasing.init, },);
      if (names.has(source,) && (!names.has(alias,))) {
        names.add(alias,);
        grown = true;
      }
    }
  }
  return names;
}

/**
 Whether a tag expression names an entry, following the const names of the file to what they were set to.

 @param tag - tag's expression

 @param bindings - what each const name of the file was set to

 @returns Whether the expression, or anything a const name in it stands for, holds `entryId` or reads an entry's id

 @example
 ```ts
 const names = namesAnEntry({ tag, bindings, },);
 ```
 */
function namesAnEntry(
  {
    tag,
    bindings,
  }: {
    readonly tag: TreeNode;
    readonly bindings: ReadonlyMap<string, TreeNode>;
  },
): boolean {
  /**
   Const names already followed, so a name set from itself ends the walk.
   */
  const followed = new Set<string>();

  /**
   Expressions still to read.
   */
  const pending: TreeNode[] = [tag,];
  for (const expression of pending) {
    for (const node of nodesUnder({ root: expression, },)) {
      if ((node.type === 'MemberExpression') && ENTRY_MEMBERS.has(memberName({ node, },),))
        return true;
      /**
       The name an identifier node carries, empty for any other node.
       */
      const name = identifierName({ node, },);
      if (name === ENTRY_IDENTIFIER)
        return true;
      /**
       What the name was set to, when the file sets it.
       */
      const setTo = bindings.get(name,);
      if ((setTo !== undefined) && (!followed.has(name,))) {
        followed.add(name,);
        pending.push(setTo,);
      }
    }
  }
  return false;
}

/**
 Whether a call is the `l` of a call of the wrapper.

 @param call - call of `tagged`

 @param parents - each node's parent

 @param wrapperNames - names the wrapper goes by in the file

 @returns Whether `sliceTagged({ l: call })` holds it, with any wrapping of the expression between

 @example
 ```ts
 const wrapped = isWrapped({ call, parents, wrapperNames, },);
 ```
 */
function isWrapped(
  {
    call,
    parents,
    wrapperNames,
  }: {
    readonly call: TreeNode;
    readonly parents: ReadonlyMap<TreeNode, TreeNode>;
    readonly wrapperNames: ReadonlySet<string>;
  },
): boolean {
  /**
   The nodes holding the call, nearest first.
   */
  const chain = ancestorsOf({
    node: call,
    parents,
  },);

  /**
   Where in the chain the first node is that does more than wrap the expression.
   */
  const wrapping = chain.findIndex(function isNotWrapping(node,): boolean {
    return !WRAPPING_KINDS.has(node.type,);
  },);
  if (wrapping === NOT_FOUND)
    return false;

  /**
   The property that holds the call, the object that holds the property and the call that holds the object.
   */
  const [property, object, wrapper,] = chain.slice(wrapping,);
  return (property?.type === 'Property')
    && (identifierName({ node: property.key, },) === 'l')
    && (object?.type === 'ObjectExpression')
    && (wrapper?.type === 'CallExpression')
    && wrapperNames.has(identifierName({ node: wrapper.callee, },),);
}

/**
 What each const name of a file was set to.

 @param nodes - every node of the file

 @returns Initializer of every const declaration that names one variable

 @example
 ```ts
 const bindings = constBindingsOf({ nodes, },); // Map { 'id' => <entry.id> }
 ```
 */
function constBindingsOf({ nodes, }: { readonly nodes: readonly TreeNode[]; },): ReadonlyMap<string, TreeNode> {
  return new Map(nodes.flatMap(function bindingOf(node,): readonly ConstBinding[] {
    if (node.type !== 'VariableDeclarator')
      return [];
    /**
     The name the declaration sets.
     */
    const name = identifierName({ node: node.id, },);

    /**
     The expression it sets the name to.
     */
    const { init, } = node;
    if ((name === '') || (!isTreeNode(init,)))
      return [];
    return [
      [
        name,
        init,
      ],
    ];
  },),);
}

/**
 Entry loggers in one file that no `sliceTagged` wraps.

 @param file - file read

 @returns How many calls of `tagged` build a logger for an entry and stand without the wrapper

 @example
 ```ts
 unwrappedEntryLoggersIn({ file: { path: 'corpus-pass/a.ts', text: "const l = tagged({ tag: entryId, },);", isTest: false, }, },); // 1
 ```
 */
export function unwrappedEntryLoggersIn({ file, }: { readonly file: SourceText; },): number {
  /**
   The file's tree.
   */
  const { program, } = parseSource({ file, },);

  /**
   Every node of the file.
   */
  const nodes = nodesUnder({ root: program, },);

  /**
   Each node's parent.
   */
  const parents = parentsOf({ program, },);

  /**
   What each const name of the file was set to.
   */
  const bindings = constBindingsOf({ nodes, },);

  /**
   Names the file calls `tagged` by.
   */
  const taggedNames = localNamesOf({
    nodes,
    imported: TAGGED,
  },);

  /**
   Names the file calls `sliceTagged` by.
   */
  const wrapperNames = localNamesOf({
    nodes,
    imported: SLICE_TAGGED,
  },);

  /**
   Calls of `tagged` in the file.
   */
  const calls = nodes.filter(function isTaggedCall(node,): boolean {
    return (node.type === 'CallExpression') && taggedNames.has(identifierName({ node: node.callee, },),);
  },);
  /**
   Calls of `tagged` that build a logger for an entry and stand without the wrapper.
   */
  const plain = calls.filter(function isPlainEntryLogger(call,): boolean {
    /**
     The `tag` property of the options the call is given.
     */
    const tag = nodeListOf({ value: call.arguments, },)
      .filter(function isOptions(argument,): boolean {
        return argument.type === 'ObjectExpression';
      },)
      .flatMap(function propertiesOf(options,): readonly TreeNode[] {
        return nodeListOf({ value: options.properties, },);
      },)
      .find(function isTag(property,): boolean {
        return (property.type === 'Property') && (identifierName({ node: property.key, },) === 'tag');
      },);
    if ((tag === undefined) || (!isTreeNode(tag.value,)))
      return false;
    return namesAnEntry({
      tag: tag.value,
      bindings,
    },) && (!isWrapped({
      call,
      parents,
      wrapperNames,
    },));
  },);
  return plain.length;
}

//endregion Entry logger finding
