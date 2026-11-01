/**
 Option-plan properties: the four customizable entry points agree with the
 options-aware model in `./options-model.ts` on generated plans and trees.

 - `deepmergeCustom`: every merge function absent, `false`, or custom
   (`actions.defaultMerge`, `undefined`, first value, `actions.skip` below the
   root, a probe of the metadata it receives), with and without
   `enableImplicitDefaultMerging`, four `filterValues` choices (one removing
   every value at keys holding only arrays), valid or invalid `maxDepth`
   values, `rootMetaData`, and a tagging `metaDataUpdater` that keeps the
   default hierarchy.
 - `deepmergeFastUnsafeCustom`: the same without skips, probes, and metadata
   options (none exist there) and without `__proto__` keys (documented as
   unguarded).
 - `deepmergeIntoCustom` and `deepmergeIntoFastUnsafeCustom`: plans whose
   functions request the default, write the first value or a probe marker
   into the target slot, or write `actions.defaultMerge` into the slot, on a
   writable target; the model reports the known-defect filter regions, which
   are skipped (`./options-model.ts`).

 Each property tallies the model branches its runs reach
 (`./reach-tally.ts`) and fails when a widened branch is drawn too rarely.

 Run plan and seed policy: see `./fuzz-budget.ts`.

 @module
 */

import {
  array,
  assert,
  oneof,
  property,
  tuple,
} from 'fast-check';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { treeArbitraries, } from './arbitraries.ts';
import { fuzzRunPlan, } from './fuzz-budget.ts';
import {
  intoPlanArbitrary,
  optionsArgumentsArbitrary,
  optionsPlanArbitrary,
  parallelCollectionsArbitrary,
} from './options-arbitraries.ts';
import {
  buildIntoOptions,
  buildOptions,
  ROOT_META,
} from './options-build.ts';
import {
  modelMergeIntoWithOptions,
  modelMergeWithOptions,
} from './options-model.ts';
import type { OptionsPlan, } from './options-plan.ts';
import {
  minimumReach,
  reachTally,
} from './reach-tally.ts';
import {
  NO_MISMATCH,
  shapeMismatch,
  snapshotObject,
} from './shape.ts';
import { target, } from './target.ts';

//region Constants and arbitraries

/**
 Run plan resolved once for every property in this file.
 */
const RUN = fuzzRunPlan();

/**
 Most sources merged into one target.
 */
const MAX_INTO_SOURCES = 3;

/**
 Runs each widened branch must be reached in.
 */
const MINIMUM_REACH = minimumReach(RUN.params.numRuns,);

/**
 `deepmergeCustom` for a plan, with its root metadata when it has some.

 @param plan - Generated plan.

 @returns Customized merge.

 @example
 ```ts
 customMergeOf(plan)(...values);
 ```
 */
function customMergeOf(plan: OptionsPlan,): (...values: readonly unknown[]) => unknown {
  return plan.rootMeta
    ? target.deepmergeCustom(
      buildOptions(plan,),
      ROOT_META,
    )
    : target.deepmergeCustom(buildOptions(plan,),);
}

/**
 `deepmergeIntoCustom` for a plan, with its root metadata when it has some.

 @param plan - Generated into plan.

 @returns Customized into merge.

 @example
 ```ts
 intoMergeOf(plan)(mutableTarget, ...sources);
 ```
 */
function intoMergeOf(plan: OptionsPlan,): (
  into: object,
  ...sources: readonly unknown[]
) => void {
  return plan.rootMeta
    ? target.deepmergeIntoCustom(
      buildIntoOptions(plan,),
      ROOT_META,
    )
    : target.deepmergeIntoCustom(buildIntoOptions(plan,),);
}

/**
 Plans whose array, Set, and Map functions return `undefined` under implicit
 default merging, drawn together with parallel collections so the fallback
 runs in most draws instead of about one in a hundred.

 @param fast - Whether the plan is for `deepmergeFastUnsafeCustom`.

 @returns Generator of arguments and plans.

 @example
 ```ts
 const cases = implicitCollectionCases({ fast: false, });
 ```
 */
function implicitCollectionCases({ fast, }: { readonly fast: boolean; },) {
  return tuple(
    parallelCollectionsArbitrary({ exotic: true, protoKey: !fast, },),
    optionsPlanArbitrary({ fast, },)
      .map(function deferCollections(plan,): OptionsPlan {
        return { ...plan, implicit: true, mergeArrays: 'undefined', mergeMaps: 'undefined', mergeSets: 'undefined', };
      },),
  );
}

/**
 `Into` inputs: a writable plain target and exotic sources, without the
 regions of the known `deepmergeInto` first-value typing defect, plus
 parallel Set or Map holders so the collection functions run.

 @param protoKey - Whether records may use the `__proto__` key.

 @returns Generator of a target and its sources.

 @example
 ```ts
 const inputs = intoInputs({ protoKey: true, });
 ```
 */
function intoInputs({ protoKey, }: { readonly protoKey: boolean; },) {
  return oneof(
    {
      weight: 3,
      arbitrary: tuple(
        treeArbitraries({ exotic: false, objectLeaves: false, protoKey, undefinedLeaves: false, },)
          .record
          .filter(function isPlain(record,) {
            return Object.getPrototypeOf(record,) === Object.prototype;
          },),
        array(treeArbitraries({ exotic: true, objectLeaves: false, protoKey, undefinedLeaves: false, },).record, {
          maxLength: MAX_INTO_SOURCES,
          minLength: 1,
        },),
      ),
    },
    {
      weight: 1,
      arbitrary: parallelCollectionsArbitrary({ exotic: false, objectLeaves: false, protoKey, undefinedLeaves: false, },)
        .map(function splitTarget([first, ...rest]: readonly object[],): readonly [object, readonly object[]] {
          return [first ?? {}, rest,];
        },),
    },
  );
}

//endregion Constants and arbitraries

await describe({
  name: 'custom options agree with the options model',
  children: [
    it({
      name: 'deepmergeCustom matches the model for generated option plans',
      timeout: RUN.timeout,
      fn: async () => {
        /**
         Widened branches this property must reach.
         */
        const customTally = reachTally([
          'filterDroppedAll',
          'nestedSkip',
          'rootMetaProbe',
          'taggedMetaProbe',
        ],);
        assert(
          property(
            optionsArgumentsArbitrary({ exotic: true, },),
            optionsPlanArbitrary({ fast: false, },),
            function customMatchesModel(values, plan,) {
              expect(shapeMismatch({
                actual: customMergeOf(plan,)(...values,),
                expected: modelMergeWithOptions({ fast: false, observe: customTally.hit, plan, values, },),
              },),)
                .toBe(NO_MISMATCH,);
              customTally.endRun();
            },
          ),
          RUN.params,
        );
        expect(customTally.shortfalls(MINIMUM_REACH,),).toEqual([],);
      },
    },),
    it({
      name: 'deepmergeFastUnsafeCustom matches the model for generated option plans',
      timeout: RUN.timeout,
      fn: async () => {
        /**
         Widened branches this property must reach.
         */
        const fastTally = reachTally(['filterDroppedAll',],);
        assert(
          property(
            optionsArgumentsArbitrary({ exotic: true, protoKey: false, },),
            optionsPlanArbitrary({ fast: true, },),
            function fastCustomMatchesModel(values, plan,) {
              expect(shapeMismatch({
                actual: target.deepmergeFastUnsafeCustom(buildOptions(plan,),)(...values,),
                expected: modelMergeWithOptions({ fast: true, observe: fastTally.hit, plan, values, },),
              },),)
                .toBe(NO_MISMATCH,);
              fastTally.endRun();
            },
          ),
          RUN.params,
        );
        expect(fastTally.shortfalls(MINIMUM_REACH,),).toEqual([],);
      },
    },),
    it({
      name: 'implicit default merging falls back for custom array, Set, and Map functions, with and without FastUnsafe',
      timeout: RUN.timeout,
      fn: async () => {
        /**
         Fallback branches this property must reach, per entry point.
         */
        const implicitTally = reachTally([
          'implicitArray',
          'implicitMap',
          'implicitSet',
        ],);
        for (const fast of [false, true,]) {
          assert(
            property(
              implicitCollectionCases({ fast, },),
              function implicitMatchesModel([values, plan,],) {
                /**
                 Entry point under test.
                 */
                const merge = fast
                  ? target.deepmergeFastUnsafeCustom(buildOptions(plan,),)
                  : customMergeOf(plan,);
                expect(shapeMismatch({
                  actual: merge(...values,),
                  expected: modelMergeWithOptions({ fast, observe: implicitTally.hit, plan, values, },),
                },),)
                  .toBe(NO_MISMATCH,);
                implicitTally.endRun();
              },
            ),
            RUN.params,
          );
        }
        expect(implicitTally.shortfalls(MINIMUM_REACH,),).toEqual([],);
      },
    },),
    it({
      name: 'deepmergeIntoCustom leaves the target equal to the model for generated plans',
      timeout: RUN.timeout,
      fn: async () => {
        /**
         Widened branches this property must reach.
         */
        const intoTally = reachTally([
          'intoDropAllKept',
          'intoSlotWrite',
          'rootMetaProbe',
          'slotDefault',
          'taggedMetaProbe',
        ],);
        assert(
          property(
            intoInputs({ protoKey: true, },),
            intoPlanArbitrary({ fast: false, },),
            function intoCustomMatchesModel([generated, sources,], plan,) {
              /**
               Private target this run may mutate.
               */
              const mutableTarget = snapshotObject(generated,);
              /**
               Model prediction from the untouched target.
               */
              const prediction = modelMergeIntoWithOptions({
                fast: false,
                observe: intoTally.hit,
                plan,
                values: [generated, ...sources,],
              },);
              intoTally.endRun();
              if (!prediction.modelled)
                return;
              intoMergeOf(plan,)(mutableTarget, ...sources,);
              expect(shapeMismatch({ actual: mutableTarget, expected: prediction.expected, },),).toBe(NO_MISMATCH,);
            },
          ),
          RUN.params,
        );
        expect(intoTally.shortfalls(MINIMUM_REACH,),).toEqual([],);
      },
    },),
    it({
      name: 'deepmergeIntoFastUnsafeCustom leaves the target equal to the model for generated plans',
      timeout: RUN.timeout,
      fn: async () => {
        /**
         Widened branches this property must reach.
         */
        const fastIntoTally = reachTally([
          'intoDropAllKept',
          'intoSlotWrite',
          'slotDefault',
        ],);
        assert(
          property(
            intoInputs({ protoKey: false, },),
            intoPlanArbitrary({ fast: true, },),
            function fastIntoCustomMatchesModel([generated, sources,], plan,) {
              /**
               Private target this run may mutate.
               */
              const mutableTarget = snapshotObject(generated,);
              /**
               Model prediction from the untouched target.
               */
              const prediction = modelMergeIntoWithOptions({
                fast: true,
                observe: fastIntoTally.hit,
                plan,
                values: [generated, ...sources,],
              },);
              fastIntoTally.endRun();
              if (!prediction.modelled)
                return;
              target.deepmergeIntoFastUnsafeCustom(buildIntoOptions(plan,),)(mutableTarget, ...sources,);
              expect(shapeMismatch({ actual: mutableTarget, expected: prediction.expected, },),).toBe(NO_MISMATCH,);
            },
          ),
          RUN.params,
        );
        expect(fastIntoTally.shortfalls(MINIMUM_REACH,),).toEqual([],);
      },
    },),
  ],
},);
