import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import type { TreeNode, } from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import type {
  BlindContext,
  ConstantReaders,
} from './claude-schema-blind-positions.test-fixture.ts';
import type { Family, } from './claude-schema-resolve.test-fixture.ts';
import {
  keyOf,
  type KnownFunctions,
} from './claude-schema-usage.test-fixture.ts';

//region What the names of a file are read against
// The first part of a context that the check behind the schema search (`claude-schema-blind-spots.test-fixture.ts`)
// reads a file's names against: the sources, the builders, the forwarders, the modules that carry a builder and the
// functions of the file that are known. The tree around the names is added once the file is read.

/**
 What the names of a file are read against before the tree around them is known: the sources, the builders, the modules
 that carry one and the functions of the file that are known.

 @param family - modules of the sources

 @param file - path of the file

 @param known - builders and wrappers of the sources, by where each sits and by name

 @param forwarding - forwarders of the sources, by where each sits and by name

 @param bearing - paths of the modules that carry a builder or a forwarder

 @param carriers - keys of the objects that hold a builder or a forwarder as a member

 @param constants - where the other files read a constant a file exports

 @returns Context, with no parents and no references yet

 @example
 ```ts
 const base = baseContextOf({ family, file, known, forwarding, bearing, carriers, constants, });
 ```
 */
export function baseContextOf(
  {
    family,
    file,
    known,
    forwarding,
    bearing,
    carriers,
    constants,
  }: {
    readonly family: Family;
    readonly file: string;
    readonly known: KnownFunctions;
    readonly forwarding: KnownFunctions;
    readonly bearing: ReadonlySet<string>;
    readonly carriers: ReadonlySet<string>;
    readonly constants: ConstantReaders;
  },
): BlindContext {
  /**
   What the file declares, binds and refers to, and its syntax tree.
   */
  const facts = nonNullishOrThrow(family.get(file,),);
  return {
    family,
    file,
    facts,
    builders: known,
    forwarders: forwarding,
    bearing,
    carriers,
    recognized: new Set(facts.declared
      .filter(function isKnown({ name, },): boolean {
        return known.keys
          .has(keyOf({
            file,
            name,
          },),);
      },)
      .map(function fnOf({ fn, },): TreeNode {
        return fn;
      },),),
    parents: new Map<TreeNode, TreeNode>(),
    references: [],
    constants,
  };
}

//endregion What the names of a file are read against
