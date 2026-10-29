/**
 Case categories of the differential check (`./mutation-differential.ts`):
 each draws fresh fixed-seed inputs and calls one deepmerge-ts bundle's
 entry points on them, so two bundles see identical cases.

 @module
 */

import {
  type Arbitrary,
  sample,
  tuple,
} from 'fast-check';

import {
  graphSpecArbitrary,
  materialize,
} from './alias-graph.ts';
import { mergeArgumentsArbitrary, } from './argument-arbitraries.ts';
import { exoticArgumentsArbitrary, } from './exotic-arbitraries.ts';
import { MutationStepError, } from './mutation-container.ts';
import {
  intoPlanArbitrary,
  optionsPlanArbitrary,
} from './options-arbitraries.ts';
import {
  buildIntoOptions,
  buildOptions,
} from './options-build.ts';

/**
 Seed shared by every category, so baseline and mutant see the same cases.
 */
const SEED = 20_260_924;

/**
 A loaded deepmerge-ts bundle's module namespace.
 */
export type Library = object;

/**
 One call: its inputs (labelled before it runs) and the call itself.
 */
export type Call = {
  readonly inputs: readonly unknown[];
  readonly run: () => readonly unknown[];
};

/**
 One category: draws fresh cases for a library.
 */
export type DifferentialCase = {
  readonly name: string;
  readonly calls: (options: {
    readonly library: Library;
    readonly runs: number;
  },) => readonly Call[];
};

/**
 Call a default entry point of a bundle.

 @param library - Bundle exports.

 @param name - Export name, such as `deepmerge`.

 @param args - Arguments.

 @returns Its return value.

 @throws {@link MutationStepError} When the export is not a function.

 @example
 ```ts
 callEntry({ args: [{}, {},], library, name: 'deepmerge', });
 ```
 */
function callEntry(
  {
    library,
    name,
    args,
  }: {
    readonly library: Library;
    readonly name: string;
    readonly args: readonly unknown[];
  },
): unknown {
  /**
   Export under test.
   */
  const entry: unknown = Reflect.get(
    library,
    name,
  );
  if ((typeof entry) !== 'function')
    throw new MutationStepError(`bundle has no function export ${name}`,);
  /**
   Return value of the call.
   */
  const returned: unknown = Reflect.apply(
    entry,
    undefined,
    args,
  );
  return returned;
}

/**
 Call a custom entry point's merge function.

 @param library - Bundle exports.

 @param name - Custom export name, such as `deepmergeCustom`.

 @param options - Built options.

 @param args - Merge arguments.

 @returns The merge function's return value.

 @example
 ```ts
 callCustom({ args, library, name: 'deepmergeCustom', options, });
 ```
 */
function callCustom(
  {
    library,
    name,
    options,
    args,
  }: {
    readonly library: Library;
    readonly name: string;
    readonly options: Readonly<Record<string, unknown>>;
    readonly args: readonly unknown[];
  },
): unknown {
  return callEntry({
    args,
    library: { merge: callEntry({
      args: [options,],
      library,
      name,
    },), },
    name: 'merge',
  },);
}

/**
 Build a category from a generator and a call shape.

 @param name - Category name in the report.

 @param arbitrary - Case generator.

 @param call - Call shape for one drawn case.

 @returns Category drawing fresh cases per library.

 @example
 ```ts
 defineCase({ arbitrary, call, name: 'graph-deepmerge', });
 ```
 */
function defineCase<const TCase,>(
  {
    name,
    arbitrary,
    call,
  }: {
    readonly name: string;
    readonly arbitrary: Arbitrary<TCase>;
    readonly call: (options: {
      readonly library: Library;
      readonly value: TCase;
    },) => Call;
  },
): DifferentialCase {
  return {
    calls: function drawCalls({
      library,
      runs,
    },) {
      return sample(
        arbitrary,
        {
          numRuns: runs,
          seed: SEED,
        },
      )
        .map(function toCall(value,) {
          return call({
            library,
            value,
          },);
        },);
    },
    name,
  };
}

/**
 The six categories of the audit's differential check.
 */
export const CASES: readonly DifferentialCase[] = [
  defineCase({
    arbitrary: graphSpecArbitrary({
      exoticLeaves: true,
      recordRoots: false,
    },),
    call: function graphMerge({
      library,
      value,
    },) {
      /**
       Fresh graph inputs.
       */
      const inputs = materialize(value,);
      return {
        inputs,
        run: function run() {
          return [
            callEntry({
              args: inputs,
              library,
              name: 'deepmerge',
            },),
            ...inputs,
          ];
        },
      };
    },
    name: 'graph-deepmerge',
  },),
  defineCase({
    arbitrary: graphSpecArbitrary({
      exoticLeaves: true,
      recordRoots: true,
    },),
    call: function graphInto({
      library,
      value,
    },) {
      /**
       Fresh graph inputs; the first is the target.
       */
      const inputs = materialize(value,);
      return {
        inputs,
        run: function run() {
          return [
            callEntry({
              args: inputs,
              library,
              name: 'deepmergeInto',
            },),
            ...inputs,
          ];
        },
      };
    },
    name: 'graph-into',
  },),
  defineCase({
    arbitrary: tuple(
      graphSpecArbitrary({
        exoticLeaves: true,
        recordRoots: false,
      },),
      optionsPlanArbitrary({ fast: false, },),
    ),
    call: function graphCustom({
      library,
      value: [spec, plan,],
    },) {
      /**
       Fresh graph inputs.
       */
      const inputs = materialize(spec,);
      return {
        inputs,
        run: function run() {
          return [
            callCustom({
              args: inputs,
              library,
              name: 'deepmergeCustom',
              options: buildOptions(plan,),
            },),
            ...inputs,
          ];
        },
      };
    },
    name: 'graph-custom',
  },),
  defineCase({
    arbitrary: tuple(
      graphSpecArbitrary({
        exoticLeaves: true,
        recordRoots: true,
      },),
      intoPlanArbitrary({ fast: false, },),
    ),
    call: function graphIntoCustom({
      library,
      value: [spec, plan,],
    },) {
      /**
       Fresh graph inputs; the first is the target.
       */
      const inputs = materialize(spec,);
      return {
        inputs,
        run: function run() {
          return [
            callCustom({
              args: inputs,
              library,
              name: 'deepmergeIntoCustom',
              options: buildIntoOptions(plan,),
            },),
            ...inputs,
          ];
        },
      };
    },
    name: 'graph-into-custom',
  },),
  defineCase({
    arbitrary: exoticArgumentsArbitrary,
    call: function exoticAll({
      library,
      value,
    },) {
      return {
        inputs: value,
        run: function run() {
          return [
            callEntry({
              args: value,
              library,
              name: 'deepmerge',
            },),
            callEntry({
              args: value,
              library,
              name: 'deepmergeFastUnsafe',
            },),
            callEntry({
              args: [
                {},
                ...value,
              ],
              library,
              name: 'deepmergeInto',
            },),
            callEntry({
              args: [
                {},
                ...value,
              ],
              library,
              name: 'deepmergeIntoFastUnsafe',
            },),
            ...value,
          ];
        },
      };
    },
    name: 'exotic-all',
  },),
  defineCase({
    arbitrary: tuple(
      mergeArgumentsArbitrary({
        edgeCalls: false,
        exotic: true,
      },),
      optionsPlanArbitrary({ fast: true, },),
    ),
    call: function treeFastCustom({
      library,
      value: [values, plan,],
    },) {
      return {
        inputs: values,
        run: function run() {
          return [
            callCustom({
              args: values,
              library,
              name: 'deepmergeFastUnsafeCustom',
              options: buildOptions(plan,),
            },),
            callEntry({
              args: [
                {},
                ...values,
              ],
              library,
              name: 'deepmergeIntoFastUnsafe',
            },),
            ...values,
          ];
        },
      };
    },
    name: 'tree-fast-custom',
  },),
];
