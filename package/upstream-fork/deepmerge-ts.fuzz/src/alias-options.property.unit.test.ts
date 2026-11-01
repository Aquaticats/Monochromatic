/**
 Custom `mergeCircularReferences` over generated object graphs
 (`./alias-graph.ts`, `./alias-into-case.ts`), the one merge function the
 tree-based option plans (`./options.property.unit.test.ts`) never reach,
 because trees have no cycles.

 Each property is metamorphic: two configurations the docs say are
 equivalent must produce isomorphic results (`isomorphismMismatch` in
 `./alias-walk.ts`) on the same graph.

 - `deepmergeCustom`: a cycle function returning `actions.defaultMerge`, or
   returning `undefined` under `enableImplicitDefaultMerging`, merges like
   plain `deepmerge`.
 - `deepmergeCustom`: `mergeCircularReferences: false` resolves cycles to the
   last value, like a cycle function returning the last value (the current
   dispatch, pinned as an intent question in
   `./known-defect-options.unit.test.ts`).
 - `deepmergeIntoCustom`: a cycle function requesting the default, by
   returning `actions.defaultMerge` or by writing it into the target slot,
   leaves the target like plain `deepmergeInto`.

 Each property requires its cycle function to have been called in enough
 runs (`./reach-tally.ts`).

 Run plan and seed policy: see `./fuzz-budget.ts`.

 @module
 */

import {
  assert,
  constantFrom,
  property,
  tuple,
} from 'fast-check';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  graphSpecArbitrary,
  materialize,
} from './alias-graph.ts';
import {
  intoCaseArbitrary,
  intoInputsOf,
} from './alias-into-case.ts';
import { isomorphismMismatch, } from './alias-walk.ts';
import { fuzzRunPlan, } from './fuzz-budget.ts';
import type { ActionUtils, } from './options-build.ts';
import {
  minimumReach,
  reachTally,
  type ReachTally,
} from './reach-tally.ts';
import { target, } from './target.ts';

//region Constants and helpers

/**
 Run plan resolved once for every property in this file.
 */
const RUN = fuzzRunPlan();

/**
 Runs in which each property's cycle function must have been called.
 */
const MINIMUM_REACH = minimumReach(RUN.params.numRuns,);

/**
 Graphs with every leaf kind, any root kind.
 */
const graphs = graphSpecArbitrary({
  exoticLeaves: true,
  recordRoots: false,
},);

/**
 How a `deepmergeCustom` cycle function defers to the default.
 */
type DeferringCycleChoice = 'defaultMerge' | 'undefinedImplicit';

/**
 Returned by into cycle functions that wrote the slot themselves.
 */
const NO_ACTION: unique symbol = Symbol('into cycle function wrote the target slot itself',);

/**
 `deepmergeCustom` options whose cycle function defers to the default and
 records each call.

 @param choice - How the function defers.

 @param tally - Counts the call for the current run.

 @returns Options object.

 @example
 ```ts
 const options = deferringCycleOptions({ choice: 'defaultMerge', tally, });
 ```
 */
function deferringCycleOptions(
  {
    choice,
    tally,
  }: {
    readonly choice: DeferringCycleChoice;
    readonly tally: ReachTally;
  },
): Record<string, unknown> {
  return {
    enableImplicitDefaultMerging: choice === 'undefinedImplicit',
    mergeCircularReferences: function deferCycle(
      _values: readonly unknown[],
      _depths: readonly number[],
      utils: ActionUtils,
    ): unknown {
      tally.hit('cycleFunctionCalled',);
      // Under implicit default merging, `undefined` requests the default.
      return choice === 'defaultMerge' ? utils.actions.defaultMerge : undefined;
    },
  };
}

//endregion Constants and helpers

await describe({
  name: 'custom mergeCircularReferences on graphs',
  children: [
    it({
      name: 'deepmergeCustom with a deferring cycle function merges graphs like deepmerge',
      timeout: RUN.timeout,
      fn: async () => {
        /**
         Calls of the cycle function per run.
         */
        const tally = reachTally(['cycleFunctionCalled',],);
        assert(
          property(
            tuple(
              graphs,
              constantFrom<DeferringCycleChoice>(
                'defaultMerge',
                'undefinedImplicit',
              ),
            ),
            function deferringMatchesPlain([spec, choice,],) {
              /**
               Merge inputs built from the graph.
               */
              const values = materialize(spec,);
              /**
               Result with the custom cycle function.
               */
              const custom: unknown = target.deepmergeCustom(deferringCycleOptions({
                choice,
                tally,
              },),)(...values,);
              tally.endRun();
              expect(isomorphismMismatch({
                left: custom,
                right: target.deepmerge(...values,),
              },),)
                .toBe('',);
            },
          ),
          RUN.params,
        );
        expect(tally.shortfalls(MINIMUM_REACH,),).toEqual([],);
      },
    },),
    it({
      name: 'deepmergeCustom with mergeCircularReferences false resolves cycles to the last value',
      timeout: RUN.timeout,
      fn: async () => {
        /**
         Calls of the last-value cycle function per run.
         */
        const tally = reachTally(['cycleFunctionCalled',],);
        assert(
          property(
            graphs,
            function falseMatchesLast(spec,) {
              /**
               Merge inputs built from the graph.
               */
              const values = materialize(spec,);
              /**
               Result with a cycle function returning the last value.
               */
              const last: unknown = target.deepmergeCustom({
                mergeCircularReferences: function lastValue(cyclic: readonly unknown[],): unknown {
                  tally.hit('cycleFunctionCalled',);
                  return cyclic.at(-1,);
                },
              },)(...values,);
              tally.endRun();
              expect(isomorphismMismatch({
                left: target.deepmergeCustom({ mergeCircularReferences: false, },)(...values,),
                right: last,
              },),)
                .toBe('',);
            },
          ),
          RUN.params,
        );
        expect(tally.shortfalls(MINIMUM_REACH,),).toEqual([],);
      },
    },),
    it({
      name: 'deepmergeIntoCustom with a cycle function requesting the default leaves the target like deepmergeInto',
      timeout: RUN.timeout,
      fn: async () => {
        /**
         Calls of the into cycle function per run.
         */
        const tally = reachTally(['cycleFunctionCalled',],);
        assert(
          property(
            tuple(
              intoCaseArbitrary,
              constantFrom(
                'return',
                'slot',
              ),
            ),
            function intoDeferringMatchesPlain([intoCase, how,],) {
              /**
               Inputs for the customized merge.
               */
              const custom = intoInputsOf(intoCase,);
              /**
               Independent inputs for the plain merge.
               */
              const plain = intoInputsOf(intoCase,);
              /**
               Options whose cycle function requests the default the planned way.
               */
              const intoOptions: Record<string, unknown> = {
                mergeCircularReferences: function deferCycle(
                  slot: { value: unknown; },
                  _values: readonly unknown[],
                  utils: ActionUtils,
                ): symbol {
                  tally.hit('cycleFunctionCalled',);
                  if (how === 'return')
                    return utils.actions.defaultMerge;
                  slot.value = utils.actions.defaultMerge;
                  return NO_ACTION;
                },
              };
              target.deepmergeIntoCustom(intoOptions,)(
                custom.intoTarget,
                ...custom.sources,
              );
              tally.endRun();
              target.deepmergeInto(
                plain.intoTarget,
                ...plain.sources,
              );
              expect(isomorphismMismatch({
                left: custom.intoTarget,
                right: plain.intoTarget,
              },),)
                .toBe('',);
            },
          ),
          RUN.params,
        );
        expect(tally.shortfalls(MINIMUM_REACH,),).toEqual([],);
      },
    },),
  ],
},);
