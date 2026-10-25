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
  usersIn,
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
// function that calls a forwarder with an argument that reads one of its own parameters is a forwarder as well, with no
// sender of its own to call, since the files that call it write the arguments of the schema as much as the files that
// call the first forwarder do. The forwarders are the fixpoint of that reading, as the wrappers are the fixpoint of the
// reading of a returned call: a chain of any length is followed to its end.

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
 The functions a search of the forwarders tells a call apart by: the builders and wrappers, and the forwarders found so
 far.
 */
type Known = {
  /**
   Builders and wrappers, by where each sits and by name.
   */
  readonly builders: KnownFunctions;

  /**
   Forwarders found so far, by where each sits and by name.
   */
  readonly forwarders: KnownFunctions;
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
 The names a round adds to the names made from a function's parameters: those that a constant binds when its value reads
 one of the names so far.

 @param declarators - every constant the function holds, nested ones too

 @param derived - names so far

 @returns The names, and the names so far, once no later round adds a name

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
function derivedNamesOf({ fn, }: { readonly fn: TreeNode; },): ReadonlySet<string> {
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
function callsFromParameter(
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
 Whether a function hands a sender a builder call made from its own parameters, or passes a name made from them to a
 forwarder.

 @param family - modules of the sources

 @param file - path of the file that declares the function

 @param parents - parent of each node of the file

 @param known - builders, wrappers and the forwarders found so far

 @param fn - function

 @returns Whether it calls a forwarder with an argument that reads a name made from a parameter, or calls a sender and
 calls a builder with such an argument

 @example
 ```ts
 const forwards = forwardsFrom({ family, file, parents, known, fn, },);
 ```
 */
function forwardsFrom(
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
    readonly known: Known;
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
  /**
   Names made from the function's parameters.
   */
  const derived = derivedNamesOf({ fn, },);
  if (calls.some(function passesOn(call,): boolean {
    return callsFromParameter({
      family,
      file,
      known: known.forwarders,
      derived,
      call,
    },);
  },))
    return true;
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
  return calls.some(function buildsFrom(call,): boolean {
    return callsFromParameter({
      family,
      file,
      known: known.builders,
      derived,
      call,
    },);
  },);
}

/**
 The functions of a set, by where each sits and by name.

 @param functions - functions

 @returns Their keys and their names

 @example
 ```ts
 const known = knownIn({ functions: builders, },);
 ```
 */
function knownIn({ functions, }: { readonly functions: readonly FunctionAt[]; },): KnownFunctions {
  return {
    keys: new Set(functions.map(function keyOfFunction(declared,): string {
      return keyOf(declared,);
    },),),
    names: new Set(functions.map(function nameOfFunction({ name, },): string {
      return name;
    },),),
  };
}

/**
 The functions that hand a sender a builder call made from their own parameters, or pass a name made from them to one
 that does, found round by round until a round adds none.

 @param family - modules of the sources

 @param builders - builders and wrappers of the sources

 @param usages - what the references of each file stand for, to tell the files that refer to a sender or a forwarder

 @param found - forwarders found by the rounds before

 @returns The forwarders found before, then those this round and the rounds after it add, in the order of the sources

 @example
 ```ts
 const forwarders = forwardersAfter({ family, builders, usages, found: [], },);
 ```
 */
function forwardersAfter(
  {
    family,
    builders,
    usages,
    found,
  }: {
    readonly family: Family;
    readonly builders: readonly FunctionAt[];
    readonly usages: readonly Usage[];
    readonly found: readonly FunctionAt[];
  },
): readonly FunctionAt[] {
  /**
   The builders and wrappers, and the forwarders so far.
   */
  const known: Known = {
    builders: knownIn({ functions: builders, },),
    forwarders: knownIn({ functions: found, },),
  };
  /**
   The files that use a forwarder found so far.
   */
  const usingForwarder = new Set(found.flatMap(function usersOfForwarder(forwarder,): readonly string[] {
    return usersIn({
      usages,
      builder: forwarder,
    },);
  },),);
  /**
   The functions this round adds.
   */
  const added = usages
    .filter(function referToSenderOrForwarder({
      file,
      keys,
    },): boolean {
      return usingForwarder.has(file,) || [...SENDER_KEYS,].some(function isReferred(key,): boolean {
        return keys.has(key,);
      },);
    },)
    .flatMap(function addedInFile({ file, },): readonly FunctionAt[] {
      /**
       What the file declares and its syntax tree.
       */
      const facts = nonNullishOrThrow(family.get(file,),);
      /**
       Parent of each node of the file.
       */
      const parents = parentsOf({ program: facts.program, },);
      return facts.declared
        .filter(function isNewForwarder(declared,): boolean {
          /**
           Where the function sits.
           */
          const key = keyOf({
            file,
            name: declared.name,
          },);
          /**
           Whether the function is a builder, a wrapper or a forwarder already.
           */
          const isKnown = [
            known.builders,
            known.forwarders,
          ].some(function holdsKey({ keys, },): boolean {
            return keys.has(key,);
          },);
          return (!isKnown) && forwardsFrom({
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
  return (added.length === 0)
    ? found
    : forwardersAfter({
      family,
      builders,
      usages,
      found: [
        ...found,
        ...added,
      ],
    },);
}

/**
 The functions that hand a sender a builder call made from their own parameters, or pass a name made from them to one
 that does, which no builder or wrapper is.

 @param family - modules of the sources

 @param builders - builders and wrappers of the sources

 @param usages - what the references of each file stand for, to tell the files that refer to a sender or a forwarder

 @returns One entry per function, the forwarders of a round after those of the round before

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
  return forwardersAfter({
    family,
    builders,
    usages,
    found: [],
  },);
}

//endregion Functions that hand a sender a builder call made from their own parameters
