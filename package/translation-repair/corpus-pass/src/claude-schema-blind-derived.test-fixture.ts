import {
  nodesUnder,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import { namesBoundBy, } from './claude-schema-blind-positions.test-fixture.ts';
import { referencesIn, } from './claude-schema-reference.test-fixture.ts';
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
  return referencesIn({ root, },)
    .some(function startsWithName(path,): boolean {
      return names.has(path.at(0,) ?? '',);
    },);
}

/**
 The names a round adds to the names made from a function's parameters: those that a constant binds when its value reads
 one of the names so far.

 @param declarators - every constant the function holds, nested ones too

 @param derived - names so far

 @returns Names, and the names so far, once no later round adds a name

 @example
 ```ts
 const names = derivedFrom({ declarators, derived: new Set(['args',],), },);
 ```
 */
function derivedFrom(
  {
    declarators,
    derived,
  }: {
    readonly declarators: readonly TreeNode[];
    readonly derived: ReadonlySet<string>;
  },
): ReadonlySet<string> {
  /**
   The names the constants bind from a name made from a parameter, which are not known yet.
   */
  const added = declarators.flatMap(function addedBy(declarator,): readonly string[] {
    /**
     The value the constant holds, `NO_NODE` when it holds none.
     */
    const init = nodeAt({
      node: declarator,
      field: 'init',
    },);
    return (isNoNode({ node: init, },) || (!readsOneOf({
      root: init,
      names: derived,
    },)))
      ? []
      : namesBoundBy({ node: declarator.id, },)
        .filter(function isNew(name,): boolean {
          return !derived.has(name,);
        },);
  },);
  return (added.length === 0)
    ? derived
    : derivedFrom({
      declarators,
      derived: new Set([
        ...derived,
        ...added,
      ],),
    },);
}

/**
 The names a function's own parameters bind, and the constants inside it made from them, to the end of the chain.

 @param fn - function

 @returns Names, the parameters' first

 @example
 ```ts
 const derived = derivedNamesOf({ fn, },);
 ```
 */
export function derivedNamesOf({ fn, }: { readonly fn: TreeNode; },): ReadonlySet<string> {
  return derivedFrom({
    declarators: nodesUnder({ root: fn, },)
      .filter(function isDeclarator({ type, },): boolean {
        return type === 'VariableDeclarator';
      },),
    derived: new Set(nodesAt({
      node: fn,
      field: 'params',
    },)
      .flatMap(function boundByParameter(parameter,): readonly string[] {
        return namesBoundBy({ node: parameter, },);
      },),),
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
