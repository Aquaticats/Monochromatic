/**
 Aliasing and cycle properties over generated object graphs
 (`./alias-graph.ts`): shared subtrees within and across inputs, self-loops,
 cycles through records, arrays, Sets, and Maps, and interlocking cycles.

 - `deepmerge` agrees with `./alias-bisim.ts` under upstream's own cycle rule:
   acyclic positions must equal the merge of the inputs' unfoldings, and
   cyclic positions must resolve as `mergeCircularReferences` intends.
 - `deepmerge` never mutates its graph inputs.
 - `deepmergeInto` into a tree target agrees the same way, and never mutates
   its graph sources within one call.

 Excluded regions, each pinned in `./known-defect-alias.unit.test.ts`:
 the false-cycle region (a value identical to an ancestor on another input's
 path only), deepmergeInto targets that are not trees (in-place mutation of a
 container reached twice), `undefined` or date leaves in into inputs, and
 writes into source containers the into target reaches after the call, which
 it shares by storing one by reference (checked per node by identity,
 `./alias-rewire.ts`).
 The divergence of upstream's cycle rule from unfolding semantics is an
 intent question, pinned there too, not asserted here.

 Run plan and seed policy: see `./fuzz-budget.ts`.

 @module
 */

import {
  assert,
  property,
} from 'fast-check';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { checkUnfolding, } from './alias-bisim.ts';
import {
  graphSpecArbitrary,
  materialize,
} from './alias-graph.ts';
import {
  cyclicValueAtMissingKey,
  intoCaseArbitrary,
  intoInputsOf,
  treeTargetWithSources,
} from './alias-into-case.ts';
import {
  cloneGraph,
  isomorphismMismatch,
  isTree,
} from './alias-walk.ts';
import {
  reachableContainers,
  rewiredNodes,
  snapshotEdges,
  sharedWithRoot,
} from './alias-rewire.ts';
import { fuzzRunPlan, } from './fuzz-budget.ts';
import {
  minimumReach,
  reachTally,
} from './reach-tally.ts';
import { target, } from './target.ts';

//region Constants and arbitraries

/**
 Run plan resolved once for every property in this file.
 */
const RUN = fuzzRunPlan();

/**
 Graphs with every leaf kind, any root kind.
 */
const graphs = graphSpecArbitrary({ exoticLeaves: true, recordRoots: false, },);

/**
 Runs in which the into properties must merge a cyclic source value at a
 root key the target lacks (`./alias-into-case.ts`).
 */
const MINIMUM_REACH = minimumReach(RUN.params.numRuns,);

/**
 Mismatch between each value and its pre-merge copy, empty when none changed.

 @param values - Inputs after the merge.
 @param copies - Copies taken before the merge.

 @returns First mutation found, or an empty string.

 @example
 ```ts
 mutationOf({ values, copies, });
 ```
 */
function mutationOf({ values, copies, }: { readonly values: readonly unknown[]; readonly copies: readonly unknown[]; },): string {
  return values
    .map(function compare(value, index,) {
      return isomorphismMismatch({ left: value, right: copies[index], },);
    },)
    .find(function found(mismatch,) {
      return mismatch !== '';
    },) ?? '';
}

//endregion Constants and arbitraries

await describe({
  name: 'deepmerge-ts aliasing and cycles',
  children: [
    it({
      name: 'deepmerge on graphs agrees with upstream\'s cycle rule outside the false-cycle region',
      timeout: RUN.timeout,
      fn: async () => {
        assert(
          property(graphs, function graphMerge(spec,) {
            /**
             Merge inputs built from the graph.
             */
            const values = materialize(spec,);
            /**
             Oracle verdict on the merge result.
             */
            const verdict = checkUnfolding({ cycles: 'upstream-rule', result: target.deepmerge(...values,), values, },);
            expect(verdict.falseCycle || (verdict.mismatch === ''),).toBe(true,);
          },),
          RUN.params,
        );
      },
    },),
    it({
      name: 'deepmerge never mutates graph inputs',
      timeout: RUN.timeout,
      fn: async () => {
        assert(
          property(graphs, function graphInputsUnchanged(spec,) {
            /**
             Merge inputs built from the graph.
             */
            const values = materialize(spec,);
            /**
             Pre-merge copies, sharing and cycles included.
             */
            const copies = values.map(function copyOf(value,) {
              return cloneGraph({ copies: new Map(), value, },);
            },);
            target.deepmerge(...values,);
            expect(mutationOf({ copies, values, },),).toBe('',);
          },),
          RUN.params,
        );
      },
    },),
    it({
      name: 'deepmergeInto a tree target from graph sources agrees with upstream\'s cycle rule outside the false-cycle region',
      timeout: RUN.timeout,
      fn: async () => {
        /**
         Seeded-slot cycles this property must reach.
         */
        const tally = reachTally(['cyclicValueAtMissingKey',],);
        assert(
          property(intoCaseArbitrary, function graphInto(intoCase,) {
            /**
             Private target this run mutates, and its sources.
             */
            const inputs = intoInputsOf(intoCase,);
            if (!treeTargetWithSources(inputs,))
              return;
            /**
             Target and sources of the case.
             */
            const {
              intoTarget,
              sources,
            } = inputs;
            if (cyclicValueAtMissingKey({ intoTarget, sources, },))
              tally.hit('cyclicValueAtMissingKey',);
            tally.endRun();
            /**
             Target before the merge, for the oracle.
             */
            const before = cloneGraph({ copies: new Map(), value: intoTarget, },);
            target.deepmergeInto(intoTarget, ...sources,);
            /**
             Oracle verdict on the mutated target.
             */
            const verdict = checkUnfolding({ cycles: 'upstream-rule', result: intoTarget, values: [before, ...sources,], },);
            expect(verdict.falseCycle || (verdict.mismatch === ''),).toBe(true,);
          },),
          RUN.params,
        );
        expect(tally.shortfalls(MINIMUM_REACH,),).toEqual([],);
      },
    },),
    it({
      name: 'deepmergeInto never mutates its graph sources within one call',
      timeout: RUN.timeout,
      fn: async () => {
        /**
         Seeded-slot cycles this property must reach.
         */
        const tally = reachTally(['cyclicValueAtMissingKey',],);
        assert(
          property(intoCaseArbitrary, function graphSourcesUnchanged(intoCase,) {
            /**
             Private target this run mutates, and its sources.
             */
            const {
              intoTarget,
              sources,
            } = intoInputsOf(intoCase,);
            /**
             Children of every source container before the merge.
             */
            const edges = snapshotEdges(reachableContainers(sources,),);
            if (cyclicValueAtMissingKey({ intoTarget, sources, },))
              tally.hit('cyclicValueAtMissingKey',);
            tally.endRun();
            target.deepmergeInto(intoTarget, ...sources,);
            /**
             Source containers the target reaches after the call; writes into them are the pinned defect.
             */
            const shared = sharedWithRoot({ edges, root: intoTarget, },);
            expect(rewiredNodes(edges,).filter(function unexplained(node,) {
              return !shared.has(node,);
            },),).toEqual([],);
          },),
          RUN.params,
        );
        expect(tally.shortfalls(MINIMUM_REACH,),).toEqual([],);
      },
    },),
  ],
},);
