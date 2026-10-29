import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  ancestorsOf,
  nodesUnder,
  parentsOf,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import {
  callsFromParameter,
  derivedNamesOf,
} from './claude-schema-blind-derived.test-fixture.ts';
import {
  callsAnySender,
  SENDER_KEYS,
} from './claude-schema-blind-senders.test-fixture.ts';
import type { Family, } from './claude-schema-resolve.test-fixture.ts';
import { isFunctionNode, } from './claude-schema-tree.test-fixture.ts';
import {
  keyOf,
  type KnownFunctions,
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
//
// The same reading finds a function that no declaration names (a class method, a static method, a constructor, a
// getter, a setter, a member of an object nested in an object, an anonymous default export, a function expression
// nothing binds). No name stands for it that another file could write, so the files that call it cannot be listed, and
// the check reports the function where it is written, as it reports a function that holds a format and that no
// declared function holds. A getter has no parameter to build from, so the function it gives back is the one reported.
// Of a function and the functions inside it that do the same, only the outermost is reported.

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
        facts: nonNullishOrThrow(family.get(file,),),
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
export function knownIn({ functions, }: { readonly functions: readonly FunctionAt[]; },): KnownFunctions {
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
 The files that refer to a sender or use a forwarder, which are the only files a forwarder can be in.

 @param usages - what the references of each file stand for

 @param forwarders - forwarders known so far

 @returns Usage of each such file, in the order of the sources

 @example
 ```ts
 const referring = referringTo({ usages, forwarders: [], },);
 ```
 */
function referringTo(
  {
    usages,
    forwarders,
  }: {
    readonly usages: readonly Usage[];
    readonly forwarders: readonly FunctionAt[];
  },
): readonly Usage[] {
  /**
   The files that use a forwarder known so far.
   */
  const usingForwarder = new Set(forwarders.flatMap(function usersOfForwarder(forwarder,): readonly string[] {
    return usersIn({
      usages,
      builder: forwarder,
    },);
  },),);
  return usages.filter(function referToSenderOrForwarder({
    file,
    keys,
  },): boolean {
    return usingForwarder.has(file,) || [...SENDER_KEYS,].some(function isReferred(key,): boolean {
      return keys.has(key,);
    },);
  },);
}

/**
 The functions that hand a sender a builder call made from their own parameters, or pass a name made from them to one
 that does, found round by round until a round adds none.

 @param family - modules of the sources

 @param builders - builders and wrappers of the sources

 @param usages - what the references of each file stand for, to tell the files that refer to a sender or a forwarder

 @param found - forwarders found by the rounds before

 @returns Forwarders found before, then those this round and the rounds after it add, in the order of the sources.
 A round adds only a function whose key is in neither the builders nor the forwarders found so far, and each round that
 adds one adds a key no round added, so the search ends after at most as many rounds as the sources declare functions

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
   The functions this round adds.
   */
  const added = referringTo({
    usages,
    forwarders: found,
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
  if (added.length === 0)
    return found;
  return forwardersAfter({
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

/**
 A function that hands a sender a builder call made from its own parameters and that no declaration names.
 */
export type UndeclaredForwarder = {
  /**
   Path of the file that writes it.
   */
  readonly file: string;

  /**
   The function.
   */
  readonly node: TreeNode;
};

/**
 The functions that hand a sender a builder call made from their own parameters, or pass a name made from them to a
 forwarder, and that no declaration names: a class method, a static method, a constructor, a getter, a setter, a member
 of an object nested in an object, an anonymous default export, a function expression that nothing names. Only the
 outermost such function is listed, and none that sits inside a function that does the same, declared or not.

 @param family - modules of the sources

 @param builders - builders and wrappers of the sources

 @param forwarders - forwarders the declarations name

 @param usages - what the references of each file stand for, to tell the files that refer to a sender or a forwarder

 @returns One entry per function, in the order of the sources

 @example
 ```ts
 const undeclared = undeclaredForwardersIn({ family, builders, forwarders, usages, },);
 ```
 */
export function undeclaredForwardersIn(
  {
    family,
    builders,
    forwarders,
    usages,
  }: {
    readonly family: Family;
    readonly builders: readonly FunctionAt[];
    readonly forwarders: readonly FunctionAt[];
    readonly usages: readonly Usage[];
  },
): readonly UndeclaredForwarder[] {
  /**
   The builders and wrappers, and the forwarders the declarations name.
   */
  const known: Known = {
    builders: knownIn({ functions: builders, },),
    forwarders: knownIn({ functions: forwarders, },),
  };
  return referringTo({
    usages,
    forwarders,
  },)
    .flatMap(function undeclaredInFile({ file, },): readonly UndeclaredForwarder[] {
      /**
       What the file declares and its syntax tree.
       */
      const facts = nonNullishOrThrow(family.get(file,),);
      /**
       Parent of each node of the file.
       */
      const parents = parentsOf({ program: facts.program, },);
      /**
       The functions of the file, of every form, that forward.
       */
      const forwarding = new Set(nodesUnder({ root: facts.program, },)
        .filter(function isForwardingFunction(node,): boolean {
          return isFunctionNode({ node, },) && forwardsFrom({
            family,
            file,
            parents,
            known,
            fn: node,
          },);
        },),);
      /**
       The functions the declarations name.
       */
      const declared = new Set(facts.declared
        .map(function fnOf({ fn, },): TreeNode {
          return fn;
        },),);
      return [...forwarding,]
        .filter(function isOutermostUndeclared(node,): boolean {
          return (!declared.has(node,)) && (!ancestorsOf({
            node,
            parents,
          },)
            .some(function isForwardingAncestor(ancestor,): boolean {
              return forwarding.has(ancestor,);
            },));
        },)
        .map(function atFile(node,): UndeclaredForwarder {
          return {
            file,
            node,
          };
        },);
    },);
}

//endregion Functions that hand a sender a builder call made from their own parameters
