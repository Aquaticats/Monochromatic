import {
  identifierName,
  isTreeNode,
  type TreeNode,
  unwrapped,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import { namesBoundBy, } from './claude-schema-blind-positions.test-fixture.ts';
import { staticCallOf, } from './claude-schema-blind-static-calls.test-fixture.ts';
import {
  memberNameOf,
  nodeAt,
  nodesAt,
} from './claude-schema-tree.test-fixture.ts';

//region Where a value made from a parameter is stored, and the names that hold it afterwards
// What `claude-schema-blind-derived.test-fixture.ts` reads a function for besides constants and loop heads: a value
// stored in an object, a list or a map is read through the first name of that object, so the first name of the object
// is a name made from the parameter. The places that store are an assignment to a member or an index, a pattern whose
// target is a member, a call of a method that stores its arguments in the object it is called on (`push`, `set`), a
// call of `Object.assign`, of a method that defines a property or of `Reflect.set`, which store in the first argument
// (also when `Object` is written under a type mark or read off the global object, and when the method is called through
// `call`, `apply` or `Reflect.apply`, with its arguments spread from an array written in place, or as the last operand
// of a comma: `claude-schema-blind-static-calls.test-fixture.ts`), a class or a function declared inside whose code
// reads the parameter, and the default of a pattern, which is a value the pattern takes. A call of any other function
// or method may store its arguments in an object it receives, and is not read; so is a field of `this`.

/**
 A place where names become derived: the names that a statement or an expression binds, and the code whose value they
 take.
 */
export type Link = {
  /**
   The code whose value the names take.
   */
  readonly read: TreeNode;

  /**
   The names bound there.
   */
  readonly bound: readonly string[];
};

/**
 The methods that store their arguments in the object they are called on: the methods of a list, a set, a map and a
 header list.
 */
const STORING_METHODS: ReadonlySet<string> = new Set([
  'add',
  'append',
  'fill',
  'push',
  'set',
  'splice',
  'unshift',
],);

/**
 The methods that store in their first argument, by the object they are read off: `Object.assign`, the two methods of
 `Object` that define properties, and `Reflect.set`.
 */
const STORING_STATICS: ReadonlyMap<string, ReadonlySet<string>> = new Map([
  [
    'Object',
    new Set([
      'assign',
      'defineProperties',
      'defineProperty',
    ],),
  ],
  [
    'Reflect',
    new Set(['set',],),
  ],
]);

/**
 The first name of a chain of members, whether or not a member is computed.

 @param node - chain, or any expression

 @returns Name, empty when the chain starts with anything but a name

 @example
 ```ts
 const head = headNameOf({ node: reference, },);
 // => 'args' for `args[0].name`
 ```
 */
export function headNameOf({ node, }: { readonly node: unknown; },): string {
  /**
   The expression without the type marks and parentheses around it.
   */
  const { inner, } = unwrapped({ node, },);
  if ((!isTreeNode(inner,)) || (inner.type !== 'MemberExpression'))
    return identifierName({ node: inner, },);
  return headNameOf({ node: inner.object, },);
}

/**
 The names a target holds a value in: the name itself, the first name of the object a member is read off, and the same
 for each target inside a pattern.

 @param node - target of an assignment or of a loop head, of any kind or none

 @returns Names in order, none for a target that is a member of something that is no name, like `this`

 @example
 ```ts
 const names = namesStoredInto({ node: assignment.left, });
 // => ['holder'] for `holder.a`, ['a', 'holder'] for `[a, holder.b]`
 ```
 */
export function namesStoredInto({ node, }: { readonly node: unknown; },): readonly string[] {
  /**
   The target without the type marks and parentheses around it.
   */
  const { inner, } = unwrapped({ node, },);
  if (!isTreeNode(inner,))
    return [];
  if (inner.type === 'MemberExpression') {
    /**
     The first name of the object the member is read off.
     */
    const head = headNameOf({ node: inner, },);
    return (head === '') ? [] : [head,];
  }
  if (inner.type === 'ObjectPattern') {
    return nodesAt({
      node: inner,
      field: 'properties',
    },)
      .flatMap(function storedByProperty(property,): readonly string[] {
        return namesStoredInto({ node: (property.type === 'Property') ? property.value : property, },);
      },);
  }
  if (inner.type === 'ArrayPattern') {
    return nodesAt({
      node: inner,
      field: 'elements',
    },)
      .flatMap(function storedByElement(element,): readonly string[] {
        return namesStoredInto({ node: element, },);
      },);
  }
  if (inner.type === 'RestElement')
    return namesStoredInto({ node: inner.argument, },);
  if (inner.type === 'AssignmentPattern')
    return namesStoredInto({ node: inner.left, },);
  return namesBoundBy({ node: inner, },);
}

/**
 The names a call stores its arguments in.

 @param call - call

 @returns First name of the object a storing method is called on, or of the first argument of `Object.assign` and of
 the methods that define properties, none for any other call

 @example
 ```ts
 const names = namesStoredByCall({ call, });
 // => ['holder'] for `holder.push(args)` and for `Object.assign(holder, args)`
 ```
 */
function namesStoredByCall({ call, }: { readonly call: TreeNode; },): readonly string[] {
  /**
   The call of a method of a built-in object that the call is, however it is written.
   */
  const staticCall = staticCallOf({ call, },);
  /**
   The methods of that object that store in their first argument, none for an object that has none.
   */
  const statics = STORING_STATICS.get(staticCall.object,) ?? new Set<string>();
  if (statics.has(staticCall.method,)) {
    return namesStoredInto({ node: staticCall.received
      .at(0,), },);
  }
  /**
   The callee without the type marks and parentheses around it.
   */
  const { inner: callee, } = unwrapped({ node: nodeAt({
    node: call,
    field: 'callee',
  },), },);
  if ((!isTreeNode(callee,)) || (callee.type !== 'MemberExpression'))
    return [];
  /**
   The method the call reads off the object.
   */
  const method = memberNameOf({ node: callee, },);
  return STORING_METHODS.has(method,)
    ? namesStoredInto({ node: callee.object, },)
    : [];
}

/**
 The links a node makes besides a constant, an assignment, a loop head or a catch clause: a call that stores its
 arguments in an object, a class or a function declaration, which holds what it reads, and the default of a pattern.

 @param node - node read

 @returns One link for the node, none for a node that makes none

 @example
 ```ts
 const links = storeLinksOf({ node, });
 ```
 */
export function storeLinksOf({ node, }: { readonly node: TreeNode; },): readonly Link[] {
  if (node.type === 'CallExpression') {
    /**
     The names the call stores its arguments in.
     */
    const bound = namesStoredByCall({ call: node, },);
    return (bound.length === 0)
      ? []
      : [
        {
          read: node,
          bound,
        },
      ];
  }
  if ((node.type === 'ClassDeclaration') || (node.type === 'FunctionDeclaration')) {
    return [
      {
        read: node,
        bound: namesBoundBy({ node: nodeAt({
          node,
          field: 'id',
        },), },),
      },
    ];
  }
  return (node.type === 'AssignmentPattern')
    ? [
      {
        read: nodeAt({
          node,
          field: 'right',
        },),
        bound: namesStoredInto({ node: nodeAt({
          node,
          field: 'left',
        },), },),
      },
    ]
    : [];
}

//endregion Where a value made from a parameter is stored, and the names that hold it afterwards
