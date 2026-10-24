import type { TreeNode, } from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import {
  isTypeNode,
  NO_NODE,
  pathOf,
  type Visit,
  visitsUnder,
} from './claude-schema-tree.test-fixture.ts';

//region The names a file's code refers to
// What `claude-schema-module.test-fixture.ts` reads a parsed file for: the names its code refers to as values, each
// as a path, and, for the check behind the search, the names the code inside a namespace, an enum or a parameter
// property refers to. A reference is a name or a chain of members where a value sits; types, keys, declared names,
// labels, pattern targets and entries of an import or export list are not.

/**
 The kinds of node whose identifier children are never a value: the entries of an import or export list, a label, the
 parameter of a catch clause, and the targets a pattern takes a value apart into.
 */
const NON_VALUE_PARENT_KINDS: ReadonlySet<string> = new Set([
  'ArrayPattern',
  'BreakStatement',
  'CatchClause',
  'ContinueStatement',
  'ExportAllDeclaration',
  'ExportDefaultDeclaration',
  'ExportSpecifier',
  'ImportDefaultSpecifier',
  'ImportNamespaceSpecifier',
  'ImportSpecifier',
  'LabeledStatement',
  'ObjectPattern',
  'RestElement',
],);

/**
 The kinds of node that declare a name in their `id` field, which is no value.
 */
const DECLARING_KINDS: ReadonlySet<string> = new Set([
  'ArrowFunctionExpression',
  'ClassDeclaration',
  'ClassExpression',
  'FunctionDeclaration',
  'FunctionExpression',
  'TSEnumDeclaration',
  'TSEnumMember',
  'TSModuleDeclaration',
  'VariableDeclarator',
],);

/**
 Whether an identifier whose parent is none of the kinds that never hold a value names a value.

 @param visit - identifier, the node that holds it and the field it sits in

 @returns Whether the identifier is a reference to a value

 @example
 ```ts
 const isReference = namesValueUnder({ visit, },);
 ```
 */
function namesValueUnder({ visit, }: { readonly visit: Visit; },): boolean {
  /**
   The node that holds the identifier, the node that holds that node and the field of the first the identifier sits in.
   */
  const {
    parent,
    grandparent,
    field,
  } = visit;
  if (parent === NO_NODE)
    return true;
  if (parent.type === 'Property') {
    return (grandparent.type !== 'ObjectPattern')
      && ((field === 'value') || (parent.computed === true));
  }
  if (parent.type === 'MemberExpression')
    return (field !== 'property') || (parent.computed === true);
  if ((parent.type === 'MethodDefinition') || (parent.type === 'PropertyDefinition'))
    return (field !== 'key') || (parent.computed === true);
  if (DECLARING_KINDS.has(parent.type,))
    return (field !== 'id') && (field !== 'params');
  if ((parent.type === 'AssignmentPattern') || (parent.type === 'AssignmentExpression'))
    return field !== 'left';
  if (parent.type === 'UnaryExpression')
    return parent.operator !== 'typeof';
  return true;
}

/**
 The kinds of node that start with `TS` and hold a value the program uses, which are no type: a namespace, an enum, a
 parameter property, an export assignment and the two forms of import-equals. The search leaves them out with the types,
 and the check behind it reads what they hold.
 */
const VALUE_HOLDING_TS_KINDS: ReadonlySet<string> = new Set([
  'TSEnumBody',
  'TSEnumDeclaration',
  'TSEnumMember',
  'TSExportAssignment',
  'TSExternalModuleReference',
  'TSImportEqualsDeclaration',
  'TSModuleBlock',
  'TSModuleDeclaration',
  'TSParameterProperty',
],);

/**
 Whether an identifier, seen in the place it sits, names a value: it is not a key, a declared name, a parameter, a
 label, a member read off an object, an entry of an import or export list, the target of a pattern, the target of an
 assignment or the operand of `typeof`.

 @param visit - identifier, the node that holds it and the field it sits in

 @returns Whether the identifier is a reference to a value

 @example
 ```ts
 const isReference = namesValue({ visit, },);
 ```
 */
function namesValue({ visit, }: { readonly visit: Visit; },): boolean {
  return (visit.parent === NO_NODE)
    || ((!NON_VALUE_PARENT_KINDS.has(visit.parent
      .type,)) && namesValueUnder({ visit, }));
}

/**
 Whether a visit is of a type, which the walk of references leaves out.

 @param node - node visited

 @returns Whether the node is a type or part of one

 @example
 ```ts
 const skipped = isTypeVisit({ node: reference, parent: NO_NODE, grandparent: NO_NODE, field: '', },);
 ```
 */
function isTypeVisit({ node, }: Visit,): boolean {
  return isTypeNode({ node, },);
}

/**
 Whether a visit is of a type that holds no value, which the walk inside a namespace, an enum or a parameter property
 leaves out.

 @param node - node visited

 @returns Whether the node is a type or part of one

 @example
 ```ts
 const skipped = isPureTypeVisit({ node: reference, parent: NO_NODE, grandparent: NO_NODE, field: '', },);
 ```
 */
export function isPureTypeVisit({ node, }: Visit,): boolean {
  return isTypeNode({ node, },) && (!VALUE_HOLDING_TS_KINDS.has(node.type,));
}

/**
 The names a file's code refers to as values, each as a path: `build`, `ns.build`, `kit.build`. A reference is a name
 or a chain of members that sits where a value does; types, keys, declared names and import or export list entries are
 left out, and so is the part of a chain that is only its start.

 @param root - node to look under

 @returns One path per reference

 @example
 ```ts
 const references = referencesIn({ root: program, },);
 // => [['contestResponseFormat'], ['contest', 'contestResponseFormat']]
 ```
 */
export function referencesIn({ root, }: { readonly root: TreeNode; },): readonly (readonly string[])[] {
  return referencesUnder({
    root,
    skips: isTypeVisit,
  },);
}

/**
 The visits under a root that start a reference: a name or a chain of members that sits where a value does, with the
 chain kept whole whatever its members are.

 @param root - node to look under

 @param skips - whether to leave a visit and everything under it out

 @returns One visit per chain, in no particular order

 @example
 ```ts
 const starts = referenceVisitsUnder({ root: program, skips: isTypeVisit, },);
 ```
 */
export function referenceVisitsUnder(
  {
    root,
    skips,
  }: {
    readonly root: TreeNode;
    readonly skips: (visit: Visit,) => boolean;
  },
): readonly Visit[] {
  return visitsUnder({
    root,
    skips,
  },)
    .filter(function isReference(visit,): boolean {
      /**
       Whether the node starts a chain of members rather than sitting inside one as the object a member is read off.
       */
      const startsChain = (visit.node
        .type
        === 'Identifier') || (visit.node
          .type
          === 'MemberExpression');
      /**
       Whether the node is the object of a member, so the chain it starts is only part of a longer one.
       */
      const insideChain = (visit.parent
        .type
        === 'MemberExpression') && (visit.field === 'object');
      return startsChain && (!insideChain)
        && namesValue({ visit, });
    },);
}

/**
 The references under a root, leaving out the visits a test names.

 @param root - node to look under

 @param skips - whether to leave a visit and everything under it out

 @returns One path per reference

 @example
 ```ts
 const references = referencesUnder({ root: program, skips: isTypeVisit, },);
 ```
 */
function referencesUnder(
  {
    root,
    skips,
  }: {
    readonly root: TreeNode;
    readonly skips: (visit: Visit,) => boolean;
  },
): readonly (readonly string[])[] {
  return referenceVisitsUnder({
    root,
    skips,
  },)
    .flatMap(function pathOfReference({ node, },): readonly (readonly string[])[] {
      /**
       The names the reference is written with.
       */
      const path = pathOf({ node, },);
      return (path.length === 0) ? [] : [path,];
    },);
}

//endregion The names a file's code refers to
