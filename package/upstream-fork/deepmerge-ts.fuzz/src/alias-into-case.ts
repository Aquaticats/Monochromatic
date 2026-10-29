/**
 `deepmergeInto` cases over generated graphs (`./alias-graph.ts`) whose
 target may lack every key its sources hold.

 A target cloned from a generated root holds most keys its sources hold, so
 two cyclic source values almost never meet at a key the target lacks. Only
 there does the merge descend into a seeded slot, where it must remap cycles
 onto the fresh target container rather than the first source's. Half of
 the cases therefore merge into an empty target, one source per generated
 root holding that root under {@link WRAP_KEY}: the roots, often cyclic
 through themselves, then meet at a key the target lacks.
 {@link cyclicValueAtMissingKey} tells a property whether a case reached
 that path.

 @module
 */

import {
  boolean,
  record,
  type Arbitrary,
} from 'fast-check';

import {
  graphSpecArbitrary,
  type GraphSpec,
  materialize,
} from './alias-graph.ts';
import { reachableContainers, } from './alias-rewire.ts';
import {
  childrenOf,
  cloneGraph,
  isContainer,
  isTree,
} from './alias-walk.ts';

/**
 Key under which an empty-target case wraps each generated root.
 */
export const WRAP_KEY = 'k';

/**
 One into case: a graph with record roots, and whether the target starts
 empty (every root wrapped in its own source) or as a copy of the first root.
 */
export type IntoCase = {
  readonly spec: GraphSpec;
  readonly emptyTarget: boolean;
};

/**
 Into cases: record-rooted graphs without the leaves of the known
 first-value typing defect.
 */
export const intoCaseArbitrary: Arbitrary<IntoCase> = record({
  emptyTarget: boolean(),
  spec: graphSpecArbitrary({
    exoticLeaves: false,
    recordRoots: true,
  },),
},);

/**
 Inputs of one into case.
 */
export type IntoInputs = {
  readonly first: unknown;
  readonly intoTarget: object;
  readonly sources: readonly unknown[];
};

/**
 Build a case's private target and its sources from fresh containers.

 @param intoCase - Generated case.

 @returns The target the merge may mutate, the sources, and the first root
   (the target's original, `{}` for an empty target); `undefined`-free
   inputs even when the copied first root is not a tree, which callers
   that need a tree check with {@link isTree} on `first`.

 @throws When the first root is not a record, which record-rooted graphs never produce.

 @example
 ```ts
 const { intoTarget, sources, } = intoInputsOf(intoCase);
 ```
 */
export function intoInputsOf(intoCase: IntoCase,): IntoInputs {
  /**
   Roots of the graph.
   */
  const roots = materialize(intoCase.spec,);
  if (intoCase.emptyTarget) {
    return {
      first: {},
      intoTarget: {},
      sources: roots.map(function wrap(root,) {
        return { [WRAP_KEY]: root, };
      },),
    };
  }
  /**
   First root and the other roots.
   */
  const [first, ...sources] = roots;
  /**
   Copy of the first root.
   */
  const copy = cloneGraph({
    copies: new Map(),
    value: first,
  },);
  if (((typeof copy) !== 'object') || (copy === null))
    throw new Error('intoInputsOf: the first root is not a record',);
  return {
    first,
    intoTarget: copy,
    sources,
  };
}

/**
 Whether an into case is usable by a property that needs a tree target and
 at least one source.

 @param inputs - Inputs from {@link intoInputsOf}.

 @returns Whether the target was a tree and there is a source.

 @example
 ```ts
 if (!treeTargetWithSources(inputs)) return;
 ```
 */
export function treeTargetWithSources(inputs: IntoInputs,): boolean {
  /**
   Original first root and the sources.
   */
  const {
    first,
    sources,
  } = inputs;
  return (sources.length > 0) && isTree(first,);
}

/**
 Whether a child key can name a record property.

 @param key - Key from {@link childrenOf}.

 @returns Whether `key` is a string, number, or symbol.

 @example
 ```ts
 isPropertyKey('a'); // true
 ```
 */
function isPropertyKey(key: unknown,): key is PropertyKey {
  return ((typeof key) === 'string')
    || ((typeof key) === 'number')
    || ((typeof key) === 'symbol');
}

/**
 Whether two or more sources hold containers at a root key the target lacks
 and one of them, not itself a source root, leads back to itself. The merge
 then descends into a seeded slot and meets a cycle back to that slot, where
 the hierarchy's `result` (the fresh target container) and `parents[0]` (the
 first source's container) differ, so remapping the cycle onto the target
 instead of the source is observable. A cycle back to a source root remaps
 to the root level, where both are the target, and shows nothing.

 @param intoTarget - Target before the merge.

 @param sources - Merge sources.

 @returns Whether the case reached that path.

 @example
 ```ts
 cyclicValueAtMissingKey({ intoTarget, sources, });
 ```
 */
export function cyclicValueAtMissingKey(
  {
    intoTarget,
    sources,
  }: {
    readonly intoTarget: object;
    readonly sources: readonly unknown[];
  },
): boolean {
  /**
   Container values the sources hold at root keys the target lacks, by key.
   */
  const byKey = new Map<PropertyKey, object[]>();
  for (const source of sources) {
    if (!isContainer(source,))
      continue;
    for (const [key, value,] of childrenOf(source,)) {
      if (isContainer(value,)
        && isPropertyKey(key,)
        && (!Object.hasOwn(
          intoTarget,
          key,
        ))) {
        byKey.set(
          key,
          [
            ...(byKey.get(key,) ?? []),
            value,
          ],
        );
      }
    }
  }
  return [...byKey.values(),].some(function cyclicMerge(values,) {
    return (values.length >= 2) && values.some(function leadsBack(value,) {
      if (sources.includes(value,))
        return false;
      /**
       Containers reachable from the value's children.
       */
      const below = reachableContainers(childrenOf(value,)
        .map(function childOf([, child,],) {
          return child;
        },),);
      return below.has(value,);
    },);
  },);
}
