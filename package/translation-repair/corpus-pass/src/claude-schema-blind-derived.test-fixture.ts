import {
  identifierName,
  isTreeNode,
  nodesUnder,
  type TreeNode,
  unwrapped,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import { namesBoundBy, } from './claude-schema-blind-positions.test-fixture.ts';
import {
  isPureTypeVisit,
  referenceVisitsUnder,
} from './claude-schema-reference.test-fixture.ts';
import type { Family, } from './claude-schema-resolve.test-fixture.ts';
import {
  isNoNode,
  nodeAt,
  nodesAt,
  pathOf,
} from './claude-schema-tree.test-fixture.ts';
import {
  type KnownFunctions,
  standsForKnown,
} from './claude-schema-usage.test-fixture.ts';

//region Names made from a function's parameters, and the calls that take them
// What `claude-schema-blind-forwarders.test-fixture.ts` reads a function for: the names its own parameters bind, the
// constants inside it that are made from them, and whether a call of a known function takes an argument that reads one
// of those names.

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
function headNameOf({ node, }: { readonly node: unknown; },): string {
  /**
   The expression without the type marks and parentheses around it.
   */
  const { inner, } = unwrapped({ node, },);
  if ((!isTreeNode(inner,)) || (inner.type !== 'MemberExpression'))
    return identifierName({ node: inner, },);
  return headNameOf({ node: inner.object, },);
}

/**
 Whether the code under a node reads a name of the set, at the head of a chain of members.

 @param root - node to look under

 @param names - names looked for

 @returns Whether some reference under the node starts with one of them

 @example
 ```ts
 const reads = readsOneOf({ root: argument, names: new Set(['args',],), },);
 ```
 */
function readsOneOf(
  {
    root,
    names,
  }: {
    readonly root: TreeNode;
    readonly names: ReadonlySet<string>;
  },
): boolean {
  return referenceVisitsUnder({
    root,
    skips: isPureTypeVisit,
  },)
    .some(function startsWithName({ node, },): boolean {
      return names.has(headNameOf({ node, },),);
    },);
}

/**
 A place where names become derived: the names that a statement or an expression binds, and the code whose value they
 take.
 */
type Link = {
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
 The names the left side of a loop head binds: the names of the declaration it makes, or the targets it assigns.

 @param left - left side of a `for...in` or `for...of` head

 @returns Names in order

 @example
 ```ts
 const names = namesOfLoopHead({ left: loop.left, },);
 ```
 */
function namesOfLoopHead({ left, }: { readonly left: TreeNode; },): readonly string[] {
  return (left.type === 'VariableDeclaration')
    ? nodesAt({
      node: left,
      field: 'declarations',
    },)
      .flatMap(function boundByDeclarator(declarator,): readonly string[] {
        return namesBoundBy({ node: declarator.id, },);
      },)
    : namesBoundBy({ node: left, },);
}

/**
 The places in a function where a name takes the value of other code: a constant, an assignment, the head of a loop
 over a value, and the parameter of a `catch` clause for what the `try` block throws.

 @param fn - function

 @returns One link per place, nested functions included

 @example
 ```ts
 const links = linksIn({ fn, },);
 ```
 */
function linksIn({ fn, }: { readonly fn: TreeNode; },): readonly Link[] {
  return nodesUnder({ root: fn, },)
    .flatMap(function linksOfNode(node,): readonly Link[] {
      if (node.type === 'VariableDeclarator') {
        return [
          {
            read: nodeAt({
              node,
              field: 'init',
            },),
            bound: namesBoundBy({ node: node.id, },),
          },
        ];
      }
      if (node.type === 'AssignmentExpression') {
        return [
          {
            read: nodeAt({
              node,
              field: 'right',
            },),
            bound: namesBoundBy({ node: node.left, },),
          },
        ];
      }
      if ((node.type === 'ForInStatement') || (node.type === 'ForOfStatement')) {
        return [
          {
            read: nodeAt({
              node,
              field: 'right',
            },),
            bound: namesOfLoopHead({ left: nodeAt({
              node,
              field: 'left',
            },), },),
          },
        ];
      }
      if ((node.type !== 'TryStatement') || isNoNode({ node: nodeAt({
        node,
        field: 'handler',
      },), },))
        return [];
      /**
       The names the `catch` clause binds.
       */
      const caught = namesBoundBy({ node: nodeAt({
        node: nodeAt({
          node,
          field: 'handler',
        },),
        field: 'param',
      },), },);
      return nodesUnder({ root: nodeAt({
        node,
        field: 'block',
      },), },)
        .filter(function isThrow({ type, },): boolean {
          return type === 'ThrowStatement';
        },)
        .map(function linkOfThrow(thrown,): Link {
          return {
            read: nodeAt({
              node: thrown,
              field: 'argument',
            },),
            bound: caught,
          };
        },);
    },);
}

/**
 The names a round adds to the names made from a function's parameters: those that a link binds when the code it reads
 reads one of the names so far.

 @param links - every place in the function where a name takes a value

 @param derived - names so far

 @returns Names, and the names so far, once no later round adds a name

 @example
 ```ts
 const names = derivedFrom({ links, derived: new Set(['args',],), },);
 ```
 */
function derivedFrom(
  {
    links,
    derived,
  }: {
    readonly links: readonly Link[];
    readonly derived: ReadonlySet<string>;
  },
): ReadonlySet<string> {
  /**
   The names the links bind from a name made from a parameter, which are not known yet.
   */
  const added = links.flatMap(function addedBy({
    read,
    bound,
  },): readonly string[] {
    return readsOneOf({
      root: read,
      names: derived,
    },)
      ? bound.filter(function isNew(name,): boolean {
        return !derived.has(name,);
      },)
      : [];
  },);
  return (added.length === 0)
    ? derived
    : derivedFrom({
      links,
      derived: new Set([
        ...derived,
        ...added,
      ],),
    },);
}

/**
 The names a function's own parameters bind, the `arguments` object, and the names made from either through a constant,
 an assignment, a loop over a value or a `catch` clause, to the end of the chain.

 @param fn - function

 @returns Names, the parameters' first

 @example
 ```ts
 const derived = derivedNamesOf({ fn, },);
 ```
 */
export function derivedNamesOf({ fn, }: { readonly fn: TreeNode; },): ReadonlySet<string> {
  return derivedFrom({
    links: linksIn({ fn, },),
    derived: new Set([
      'arguments',
      ...nodesAt({
        node: fn,
        field: 'params',
      },)
        .flatMap(function boundByParameter(parameter,): readonly string[] {
          return namesBoundBy({ node: parameter, },);
        },),
    ],),
  },);
}

/**
 Whether a call is a call of a known function with an argument that reads a name made from the function's parameters.

 @param family - modules of the sources

 @param file - path of the file that declares the function

 @param known - functions the callee is looked for among

 @param derived - names made from the function's parameters

 @param call - call

 @returns Whether the callee stands for one of them and an argument reads a derived name

 @example
 ```ts
 const isFromParameter = callsFromParameter({ family, file, known, derived, call, },);
 ```
 */
export function callsFromParameter(
  {
    family,
    file,
    known,
    derived,
    call,
  }: {
    readonly family: Family;
    readonly file: string;
    readonly known: KnownFunctions;
    readonly derived: ReadonlySet<string>;
    readonly call: TreeNode;
  },
): boolean {
  /**
   The names the callee is written with.
   */
  const callee = pathOf({ node: call.callee, },);
  if (callee.length === 0)
    return false;
  if (!standsForKnown({
    family,
    file,
    path: callee,
    known,
  },))
    return false;
  return nodesAt({
    node: call,
    field: 'arguments',
  },)
    .some(function readsDerived(argument,): boolean {
      return readsOneOf({
        root: argument,
        names: derived,
      },);
    },);
}

//endregion Names made from a function's parameters, and the calls that take them
