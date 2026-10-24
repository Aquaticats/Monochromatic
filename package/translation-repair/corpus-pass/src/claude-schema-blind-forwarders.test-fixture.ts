import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  nodesUnder,
  parentsOf,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import { namesBoundBy, } from './claude-schema-blind-positions.test-fixture.ts';
import {
  callsAnySender,
  SENDER_KEYS,
} from './claude-schema-blind-senders.test-fixture.ts';
import { referencesIn, } from './claude-schema-reference.test-fixture.ts';
import type { Family, } from './claude-schema-resolve.test-fixture.ts';
import {
  isNoNode,
  nodeAt,
  nodesAt,
  pathOf,
} from './claude-schema-tree.test-fixture.ts';
import {
  keyOf,
  type KnownFunctions,
  standsForKnown,
  type Usage,
} from './claude-schema-usage.test-fixture.ts';

//region Functions that hand a sender a builder call made from their own parameters
// A function that makes an answer schema from what its caller passes, and hands it to a sender
// (`claude-schema-blind-senders.test-fixture.ts`), takes the arguments of the schema from its callers. The search lists
// the callers as it lists the callers of a wrapper, since a caller of a wrapper is where the schema's arguments are
// written. The function is no builder: what it gives back is the sender's answer, so a caller that uses the answer is
// listed and the check does not follow the answer as a schema.
//
// The function is recognised by its text alone: it calls a sender, and it calls a builder with an argument that reads
// one of its own parameters, or a constant made from one, however the schema gets from the builder to the sender. A
// function that forwards through another function that forwards is not followed past the first.

/**
 A function, by the file that declares it.
 */
export type FunctionAt = {
  /**
   The function's name, `object.member` for the member of an object.
   */
  readonly name: string;

  /**
   Path of the file that declares it.
   */
  readonly file: string;
};

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
 The names a function's own parameters bind, and the constants inside it made from them, to the end of the chain.

 @param fn - function

 @returns Names, the parameters' first

 @example
 ```ts
 const derived = derivedNamesOf({ fn, },);
 ```
 */
function derivedNamesOf({ fn, }: { readonly fn: TreeNode; },): ReadonlySet<string> {
  /**
   Names known to stand for what a parameter holds.
   */
  const derived = new Set(nodesAt({
    node: fn,
    field: 'params',
  },)
    .flatMap(function boundByParameter(parameter,): readonly string[] {
      return namesBoundBy({ node: parameter, },);
    },),);
  /**
   Every constant the function holds, nested ones too.
   */
  const declarators = nodesUnder({ root: fn, },)
    .filter(function isDeclarator({ type, },): boolean {
      return type === 'VariableDeclarator';
    },);
  for (let grew = true; grew;) {
    grew = false;
    for (const declarator of declarators) {
      /**
       The value the constant holds, `NO_NODE` when it holds none.
       */
      const init = nodeAt({
        node: declarator,
        field: 'init',
      },);
      /**
       Whether the value reads a name made from a parameter.
       */
      const readsDerived = isNoNode({ node: init, },)
        ? false
        : readsOneOf({
          root: init,
          names: derived,
        },);
      for (const name of namesBoundBy({ node: declarator.id, },)) {
        if (readsDerived && (!derived.has(name,))) {
          derived.add(name,);
          grew = true;
        }
      }
    }
  }
  return derived;
}

/**
 Whether a call is a call of a known builder with an argument that reads a name made from the function's parameters.

 @param family - modules of the sources

 @param file - path of the file that declares the function

 @param known - builders and wrappers of the sources

 @param derived - names made from the function's parameters

 @param call - call

 @returns Whether the callee stands for a builder and an argument reads a derived name

 @example
 ```ts
 const isFromParameter = buildsFromParameter({ family, file, known, derived, call, },);
 ```
 */
function buildsFromParameter(
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

/**
 Whether a function hands a sender a builder call made from its own parameters.

 @param family - modules of the sources

 @param file - path of the file that declares the function

 @param parents - parent of each node of the file

 @param known - builders and wrappers of the sources

 @param fn - function

 @returns Whether it calls a sender and calls a builder with an argument that reads a name made from a parameter

 @example
 ```ts
 const forwards = forwardsToSender({ family, file, parents, known, fn, },);
 ```
 */
function forwardsToSender(
  {
    family,
    file,
    parents,
    known,
    fn,
  }: {
    readonly family: Family;
    readonly file: string;
    readonly parents: ReadonlyMap<TreeNode, TreeNode>;
    readonly known: KnownFunctions;
    readonly fn: TreeNode;
  },
): boolean {
  /**
   The calls the function holds, nested functions' too.
   */
  const calls = nodesUnder({ root: fn, },)
    .filter(function isCall({ type, },): boolean {
      return type === 'CallExpression';
    },);
  if (!calls.some(function callsSender(call,): boolean {
    return callsAnySender({
      scope: {
        family,
        file,
        parents,
      },
      call,
    },);
  },))
    return false;
  /**
   Names made from the function's parameters.
   */
  const derived = derivedNamesOf({ fn, },);
  return calls.some(function buildsFrom(call,): boolean {
    return buildsFromParameter({
      family,
      file,
      known,
      derived,
      call,
    },);
  },);
}

/**
 The functions that hand a sender a builder call made from their own parameters, which no builder or wrapper is.

 @param family - modules of the sources

 @param builders - builders and wrappers of the sources

 @param usages - what the references of each file stand for, to tell the files that refer to a sender

 @returns One entry per function, in the order of the sources

 @example
 ```ts
 const forwarders = forwardersIn({ family, builders, usages, },);
 ```
 */
export function forwardersIn(
  {
    family,
    builders,
    usages,
  }: {
    readonly family: Family;
    readonly builders: readonly FunctionAt[];
    readonly usages: readonly Usage[];
  },
): readonly FunctionAt[] {
  /**
   The builders and wrappers, by where each sits and by name.
   */
  const known: KnownFunctions = {
    keys: new Set(builders.map(function keyOfBuilder(builder,): string {
      return keyOf(builder,);
    },),),
    names: new Set(builders.map(function nameOfBuilder({ name, },): string {
      return name;
    },),),
  };
  return usages
    .filter(function referToSender({ keys, },): boolean {
      return [...SENDER_KEYS,].some(function isReferred(key,): boolean {
        return keys.has(key,);
      },);
    },)
    .flatMap(function forwardersOfFile({ file, },): readonly FunctionAt[] {
      /**
       What the file declares and its syntax tree.
       */
      const facts = nonNullishOrThrow(family.get(file,),);
      /**
       Parent of each node of the file.
       */
      const parents = parentsOf({ program: facts.program, },);
      return facts.declared
        .filter(function isForwarder(declared,): boolean {
          /**
           Whether the function is a builder or a wrapper already.
           */
          const isKnown = known.keys
            .has(keyOf({
              file,
              name: declared.name,
            },),);
          return isKnown ? false : forwardsToSender({
            family,
            file,
            parents,
            known,
            fn: declared.fn,
          },);
        },)
        .map(function atFile({ name, },): FunctionAt {
          return {
            name,
            file,
          };
        },);
    },);
}

//endregion Functions that hand a sender a builder call made from their own parameters
