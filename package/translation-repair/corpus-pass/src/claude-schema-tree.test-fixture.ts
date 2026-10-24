import {
  identifierName,
  isTreeNode,
  literalText,
  memberName,
  nodesUnder,
  type TreeNode,
  unwrapped,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

//region The syntax tree of a source file, read for answer schemas
// What `claude-schema-module.test-fixture.ts` and `claude-schema-sources.test-fixture.ts` read a parsed file through:
// walks over the tree that leave out types, the object literal that is an answer schema's format, the function an
// expression holds, the calls a function returns, and the names a file refers to as dotted paths. The tree is the one
// `parseSource` of `source-scan.test-fixture.ts` produces, so a string, a template, a comment or a type is never a
// node of an expression and cannot be read as a use.

/**
 One node met in a walk, with the node that holds it and the field of that node it sits in.
 */
type Visit = {
  /**
   The node.
   */
  readonly node: TreeNode;

  /**
   The node that holds it, `NO_NODE` for the root of the walk.
   */
  readonly parent: TreeNode;

  /**
   The node that holds the parent, `NO_NODE` for the root of the walk and its children.
   */
  readonly grandparent: TreeNode;

  /**
   The field of the parent that holds it, empty for the root of the walk.
   */
  readonly field: string;
};

/**
 The node that stands where a field holds no node, so a walk that reads a missing field reads a node of no kind.
 */
export const NO_NODE: TreeNode = {
  type: 'NoNode',
  start: 0,
  end: 0,
};

/**
 The kinds of node that are a function: a declaration, an expression and an arrow function.
 */
const FUNCTION_KINDS: ReadonlySet<string> = new Set([
  'ArrowFunctionExpression',
  'FunctionDeclaration',
  'FunctionExpression',
],);

/**
 The kinds of node that put a type mark around an expression and leave the expression a value.
 */
const TYPE_MARKED_EXPRESSION_KINDS: ReadonlySet<string> = new Set([
  'TSAsExpression',
  'TSInstantiationExpression',
  'TSNonNullExpression',
  'TSSatisfiesExpression',
  'TSTypeAssertion',
],);

/**
 The spelling of the property that makes an object literal a format: its key and its value.
 */
const FORMAT_KEY = 'type';

/**
 The value of the property that makes an object literal a format.
 */
const FORMAT_VALUE = 'json_schema';

/**
 The text of a template piece, with escapes read.

 @param value - the `value` field of a template piece, of any kind

 @returns The cooked text, empty when the piece holds none

 @example
 ```ts
 const cooked = cookedTextOf({ value: { raw: 'a', cooked: 'a', }, },);
 // => 'a'
 ```
 */
function cookedTextOf({ value, }: { readonly value: unknown; },): string {
  return (((typeof value) === 'object') && (value !== null)
    && ('cooked' in value)
    && ((typeof value.cooked) === 'string'))
    ? value.cooked
    : '';
}

/**
 The node a field holds.

 @param node - node read

 @param field - name of the field

 @returns The node, `NO_NODE` when the field holds none

 @example
 ```ts
 const callee = nodeAt({ node: call, field: 'callee', },);
 ```
 */
export function nodeAt({
  node,
  field,
}: {
  readonly node: TreeNode;
  readonly field: string
},): TreeNode {
  /**
   What the field holds.
   */
  const value = node[field];
  return isTreeNode(value,) ? value : NO_NODE;
}

/**
 Whether a node is the one that stands for a missing field.

 @param node - node read

 @returns Whether it is `NO_NODE`

 @example
 ```ts
 const missing = isNoNode({ node: nodeAt({ node: call, field: 'callee', },), },);
 ```
 */
export function isNoNode({ node, }: { readonly node: TreeNode; },): boolean {
  return node === NO_NODE;
}

/**
 Whether a value is a node, as a type guard a callback can take.

 @param value - field of a node

 @returns Whether it is a node

 @example
 ```ts
 const nodes = [1, program,].filter(isNodeValue,);
 ```
 */
function isNodeValue(value: unknown,): value is TreeNode {
  return isTreeNode(value,);
}

/**
 The nodes a list field holds.

 @param node - node read

 @param field - name of the field

 @returns The nodes in order, none when the field holds no list

 @example
 ```ts
 const statements = nodesAt({ node: program, field: 'body', },);
 ```
 */
export function nodesAt({
  node,
  field,
}: {
  readonly node: TreeNode;
  readonly field: string
},): readonly TreeNode[] {
  /**
   What the field holds.
   */
  const value = node[field];
  return Array.isArray(value,) ? value.filter(isNodeValue,) : [];
}

/**
 Whether a node is a type or part of one, which holds no value: every kind that starts with `TS` but the kinds that put
 a type mark around an expression.

 @param node - node read

 @returns Whether the walk leaves it out

 @example
 ```ts
 const skipped = isTypeNode({ node: { type: 'TSTypeReference', start: 0, end: 1, }, },);
 // => true
 ```
 */
function isTypeNode({ node, }: { readonly node: TreeNode; },): boolean {
  return node.type
    .startsWith('TS',)
    && (!TYPE_MARKED_EXPRESSION_KINDS.has(node.type,));
}

/**
 Whether a node is a function.

 @param node - node read

 @returns Whether it is a function declaration, a function expression or an arrow function

 @example
 ```ts
 const isFunction = isFunctionNode({ node: declaration, },);
 ```
 */
export function isFunctionNode({ node, }: { readonly node: TreeNode; },): boolean {
  return FUNCTION_KINDS.has(node.type,);
}

/**
 Every node under a root, the root included, with the node that holds it, walked with a stack.

 @param root - node to walk from

 @param skips - whether to leave a node and everything under it out of the walk

 @returns Visits in no particular order

 @example
 ```ts
 const visits = visitsUnder({ root: program, skips: isTypeVisit, },);
 ```
 */
function visitsUnder(
  {
    root,
    skips,
  }: {
    readonly root: TreeNode;
    readonly skips: (visit: Visit,) => boolean;
  },
): readonly Visit[] {
  /**
   Visits met so far.
   */
  const met: Visit[] = [];
  /**
   Visits still to make.
   */
  const pending: Visit[] = [
    {
      node: root,
      parent: NO_NODE,
      grandparent: NO_NODE,
      field: '',
    },
  ];
  for (let visit = pending.pop(); visit !== undefined; visit = pending.pop()) {
    if (!skips(visit,)) {
      met.push(visit,);
      for (const [field, value,] of Object.entries(visit.node,)) {
        for (const child of (Array.isArray(value,) ? value : [value,])) {
          if (isTreeNode(child,)) {
            pending.push({
              node: child,
              parent: visit.node,
              grandparent: visit.parent,
              field,
            },);
          }
        }
      }
    }
  }
  return met;
}

/**
 The text a node holds when it is a string: a string literal, or a template that has no substitution.

 @param node - node read, of any kind or none

 @returns The string, empty for any other node

 @example
 ```ts
 const text = stringValueOf({ node: property.value, },);
 // => 'json_schema'
 ```
 */
export function stringValueOf({ node, }: { readonly node: unknown; },): string {
  if (!isTreeNode(node,))
    return '';
  if (node.type === 'Literal')
    return ((typeof node.value) === 'string') ? literalText({ node, },) : '';
  if ((node.type !== 'TemplateLiteral') || (nodesAt({
    node,
    field: 'expressions',
  },)
    .length
    > 0))
    return '';
  /**
   The one piece of text a template without substitution has.
   */
  const [quasi = NO_NODE,] = nodesAt({
    node,
    field: 'quasis',
  },);
  return cookedTextOf({ value: quasi.value, },);
}

/**
 The name a property or a member is written with: a plain name, or a string in quotes or in brackets.

 @param key - the node that holds the key

 @returns The name, empty for any other key

 @example
 ```ts
 const name = keyNameOf({ key: property.key, },);
 ```
 */
export function keyNameOf({ key, }: { readonly key: TreeNode; },): string {
  /**
   The name of a plain key.
   */
  const plain = identifierName({ node: key, },);
  return (plain === '') ? stringValueOf({ node: key, },) : plain;
}

/**
 Whether a property is written `type: 'json_schema'`, the pair that makes an object literal an answer schema's format.

 @param property - node read

 @returns Whether the node is a property with that key and value

 @example
 ```ts
 const isFormat = isFormatProperty({ property, },);
 ```
 */
function isFormatProperty({ property, }: { readonly property: TreeNode; },): boolean {
  return (property.type === 'Property')
    && (keyNameOf({ key: nodeAt({
      node: property,
      field: 'key',
    },), },) === FORMAT_KEY)
    && (stringValueOf({ node: property.value, },) === FORMAT_VALUE);
}

/**
 Whether a node is an object literal that is an answer schema's format: one of its properties is
 `type: 'json_schema'`. Only an object literal lists properties that hold a string, since an object pattern lists the
 names it takes out.

 @param node - node read

 @returns Whether the node lists a property with that key and that value

 @example
 ```ts
 const isFormat = isFormatObject({ node, },);
 ```
 */
function isFormatObject({ node, }: { readonly node: TreeNode; },): boolean {
  return nodesAt({
    node,
    field: 'properties',
  },)
    .some(function isFormatEntry(property,): boolean {
      return isFormatProperty({ property, },);
    },);
}

/**
 Whether a member of a type is written `type: 'json_schema'`, the type of a format.

 @param node - node read

 @returns Whether the node is a property signature with that key and that literal type

 @example
 ```ts
 const isFormatType = isFormatTypeMember({ node: signature, },);
 ```
 */
function isFormatTypeMember({ node, }: { readonly node: TreeNode; },): boolean {
  if ((node.type !== 'TSPropertySignature') || (keyNameOf({ key: nodeAt({
    node,
    field: 'key',
  },), },) !== FORMAT_KEY))
    return false;
  /**
   The annotation of the member.
   */
  const annotation = nodeAt({
    node,
    field: 'typeAnnotation',
  },);
  /**
   The literal type the annotation holds.
   */
  const literalType = nodeAt({
    node: annotation,
    field: 'typeAnnotation',
  },);
  return (literalType.type === 'TSLiteralType')
    && (stringValueOf({ node: literalType.literal, },) === FORMAT_VALUE);
}

/**
 Whether a tree holds an object literal that is an answer schema's format, anywhere under the root and inside any
 function under it.

 @param root - node to look under

 @returns Whether some object literal has the property `type: 'json_schema'`

 @example
 ```ts
 const holds = holdsFormatValue({ root: declaration, },);
 ```
 */
export function holdsFormatValue({ root, }: { readonly root: TreeNode; },): boolean {
  return nodesUnder({ root, },)
    .some(function isFormat(node,): boolean {
      return isFormatObject({ node, },);
    },);
}

/**
 Whether a tree writes a format or names the type of one: an object literal with the property
 `type: 'json_schema'`, or a member `type: 'json_schema'` of a type or an interface.

 @param root - node to look under

 @returns Whether some node is either

 @example
 ```ts
 const mentions = holdsFormatMention({ root: program, },);
 ```
 */
export function holdsFormatMention({ root, }: { readonly root: TreeNode; },): boolean {
  return nodesUnder({ root, },)
    .some(function isMention(node,): boolean {
      return isFormatObject({ node, },) || isFormatTypeMember({ node, },);
    },);
}

/**
 The function an expression holds: an arrow function or a function expression, also when a call takes it as its first
 argument (`memoize((a) => ...)`) and under a type mark.

 @param node - expression read, of any kind or none

 @returns The function, `NO_NODE` when the expression holds none

 @example
 ```ts
 const fn = functionIn({ node: declarator.init, },);
 ```
 */
export function functionIn({ node, }: { readonly node: unknown; },): TreeNode {
  /**
   The expression without the type marks and parentheses around it.
   */
  const { inner, } = unwrapped({ node, },);
  if (!isTreeNode(inner,))
    return NO_NODE;
  if (FUNCTION_KINDS.has(inner.type,))
    return inner;
  if (inner.type !== 'CallExpression')
    return NO_NODE;
  /**
   The first argument of the call.
   */
  const [first,] = nodesAt({
    node: inner,
    field: 'arguments',
  },);
  return functionIn({ node: first, },);
}

/**
 The names an expression is written with when it is a plain name or a chain of members: `build`, `ns.build`,
 `kit.build`, also `kit['build']`.

 @param node - expression read, of any kind or none

 @returns The names in order, none for any other expression

 @example
 ```ts
 const path = pathOf({ node: call.callee, },);
 // => ['kit', 'build']
 ```
 */
export function pathOf({ node, }: { readonly node: unknown; },): readonly string[] {
  /**
   The expression without the type marks and parentheses around it.
   */
  const { inner, } = unwrapped({ node, },);
  if (!isTreeNode(inner,))
    return [];
  if (inner.type === 'Identifier')
    return [identifierName({ node: inner, },),];
  if (inner.type !== 'MemberExpression')
    return [];
  /**
   The names of the object the member is read off.
   */
  const objectPath = pathOf({ node: inner.object, });
  /**
   The member's own name, empty when it is computed from anything but a string.
   */
  const member = memberName({ node: inner, });
  return ((objectPath.length === 0) || (member === '')) ? [] : [
    ...objectPath,
    member,
  ];
}

/**
 Whether a visit is of a function, which a walk of a function's own body does not enter.

 @param node - the node visited

 @returns Whether the node is a function

 @example
 ```ts
 const skipped = isFunctionVisit({ node: fn, parent: NO_NODE, grandparent: NO_NODE, field: '', },);
 ```
 */
function isFunctionVisit({ node, }: Visit,): boolean {
  return isFunctionNode({ node, },);
}

/**
 The expressions a function returns: an arrow function's expression body, and the argument of each `return` in the
 function's own body, not the `return` of a function inside it.

 @param fn - function read

 @returns The expressions in order

 @example
 ```ts
 const returned = returnedExpressionsOf({ fn: declaration, },);
 ```
 */
function returnedExpressionsOf({ fn, }: { readonly fn: TreeNode; },): readonly TreeNode[] {
  /**
   The body of the function.
   */
  const body = nodeAt({
    node: fn,
    field: 'body',
  },);
  if (isNoNode({ node: body, },))
    return [];
  if (body.type !== 'BlockStatement')
    return [body,];
  return visitsUnder({
    root: body,
    skips: isFunctionVisit,
  },)
    .flatMap(function returnedBy({ node, },): readonly TreeNode[] {
      /**
       What the statement returns.
       */
      const returned = (node.type === 'ReturnStatement') ? nodeAt({
        node,
        field: 'argument',
      },) : NO_NODE;
      return isNoNode({ node: returned, },) ? [] : [returned,];
    },);
}

/**
 The expression a function's return statement or arrow body holds once an `await`, parentheses, type marks and an
 optional chain around it are taken off.

 @param node - expression read

 @returns The expression underneath

 @example
 ```ts
 const call = withoutAwait({ node: returned, },);
 ```
 */
function withoutAwait({ node, }: { readonly node: TreeNode; },): unknown {
  /**
   The expression without type marks, parentheses and an optional chain.
   */
  const { inner, } = unwrapped({ node, },);
  return (isTreeNode(inner,) && (inner.type === 'AwaitExpression'))
    ? withoutAwait({ node: nodeAt({
      node: inner,
      field: 'argument',
    },), })
    : inner;
}

/**
 The names of the functions a function returns the call of: for each returned expression that is a call, once an `await`,
 parentheses, type marks and an optional chain are taken off, the names its callee is written with.

 @param fn - function read

 @returns One path per returned call whose callee is a name or a chain of members

 @example
 ```ts
 const callees = returnedCalleesOf({ fn: declaration, },);
 // => [['contestResponseFormat']]
 ```
 */
export function returnedCalleesOf({ fn, }: { readonly fn: TreeNode; },): readonly (readonly string[])[] {
  return returnedExpressionsOf({ fn, },)
    .flatMap(function calleeOf(returned,): readonly (readonly string[])[] {
      /**
       What the function returns, once the marks around it are taken off.
       */
      const call = withoutAwait({ node: returned, },);
      /**
       The names of the callee, none when the expression is no call or the callee is no name.
       */
      const path = (isTreeNode(call,) && (call.type === 'CallExpression'))
        ? pathOf({ node: call.callee, })
        : [];
      return (path.length === 0) ? [] : [path,];
    },);
}

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
  'VariableDeclarator',
],);

/**
 Whether an identifier, seen in the place it sits, names a value: it is not a key, a declared name, a parameter, a label,
 a member read off an object, an entry of an import or export list, the target of a pattern, the target of an
 assignment or the operand of `typeof`.

 @param visit - the identifier, the node that holds it and the field it sits in

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
 Whether an identifier whose parent is none of the kinds that never hold a value names a value.

 @param visit - the identifier, the node that holds it and the field it sits in

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
 Whether a visit is of a type, which the walk of references leaves out.

 @param node - the node visited

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
  return visitsUnder({
    root,
    skips: isTypeVisit,
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
    },)
    .flatMap(function pathOfReference({ node, },): readonly (readonly string[])[] {
      /**
       The names the reference is written with.
       */
      const path = pathOf({ node, },);
      return (path.length === 0) ? [] : [path,];
    },);
}

//endregion The syntax tree of a source file, read for answer schemas
