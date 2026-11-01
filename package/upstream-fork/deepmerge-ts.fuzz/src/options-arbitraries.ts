/**
 fast-check generators for `deepmergeCustom` option plans
 (see `./options-model.ts`).

 @module
 */

import {
  array,
  boolean,
  constant,
  constantFrom,
  integer,
  oneof,
  record,
  tuple,
  uniqueArray,
  type Arbitrary,
} from 'fast-check';

import {
  keyOfEntry,
  treeArbitraries,
  type TreeOptions,
} from './arbitraries.ts';
import { mergeArgumentsArbitrary, } from './argument-arbitraries.ts';
import type {
  CustomChoice,
  FilterChoice,
  MaxDepthChoice,
  MetaUpdaterChoice,
  OptionsPlan,
} from './options-plan.ts';

/**
 Largest valid `maxDepth` drawn; generated trees nest at most four containers.
 */
const MAX_PLANNED_DEPTH = 5;

/**
 Most collections merged at the shared key, and most elements per collection.
 */
const MAX_PARALLEL = 3;

/**
 Records that each hold a Set, each hold a Map, or each hold an array, under
 the key `k`: the only inputs where two collections of one kind reliably
 meet, so a plan's Set, Map, and array functions run instead of
 `mergeOthers`. The general tree arguments rarely line up that way; arrays
 joined after the recall audit's edge shapes shifted the fixed-seed draws off
 the `deepmergeCustom` array fallback.

 @param options - Leaf and key switches, as for the tree generators.

 @returns Generator of two or three such records.

 @example
 ```ts
 const inputs = parallelCollectionsArbitrary({ exotic: false, });
 ```
 */
export function parallelCollectionsArbitrary(options: TreeOptions,): Arbitrary<readonly object[]> {
  /**
   Values stored in the collections.
   */
  const { tree, } = treeArbitraries(options,);
  /**
   One Set of trees.
   */
  const set = array(
    tree,
    { maxLength: MAX_PARALLEL, },
  )
    .map(function toSet(items,): unknown {
      return new Set(items,);
    },);
  /**
   One Map of trees under a small key pool, so Maps overlap.
   */
  const map = uniqueArray(
    tuple(
      constantFrom(
        'a',
        'b',
      ),
      tree,
    ),
    {
      maxLength: MAX_PARALLEL,
      selector: keyOfEntry,
    },
  )
    .map(function toMap(pairs,): unknown {
      return new Map(pairs,);
    },);
  /**
   One array of trees.
   */
  const list = array(
    tree,
    { maxLength: MAX_PARALLEL, },
  )
    .map(function toArray(items,): unknown {
      return items;
    },);
  return oneof(
    set,
    map,
    list,
  )
    .chain(function sameKind(first,) {
      /**
       Generator for the rest, of the first collection's kind.
       */
      const sameKindArbitrary = (first instanceof Set)
        ? set
        : ((first instanceof Map) ? map : list);
      return array(
        sameKindArbitrary,
        {
          minLength: 1,
          maxLength: MAX_PARALLEL - 1,
        },
      )
        .map(function toRecords(rest,) {
          /**
           Every collection, first one first.
           */
          const collections = [
            first,
            ...rest,
          ];
          return collections.map(function holder(collection,) {
            return { k: collection, };
          },);
        },);
    },);
}

/**
 Merge arguments for option plans: the general tree arguments, plus
 {@link parallelCollectionsArbitrary} often enough to reach the collection
 functions.

 @param options - Leaf and key switches, as for the tree generators.

 @returns Generator of merge arguments.

 @example
 ```ts
 const inputs = optionsArgumentsArbitrary({ exotic: true, });
 ```
 */
export function optionsArgumentsArbitrary(options: TreeOptions,): Arbitrary<readonly unknown[]> {
  return oneof(
    {
      weight: 3,
      arbitrary: mergeArgumentsArbitrary(options,),
    },
    {
      weight: 1,
      arbitrary: parallelCollectionsArbitrary(options,),
    },
  );
}

/**
 Choice generator for one merge function.

 @param fast - Whether the plan is for a FastUnsafe variant: no skips (no
   call has metadata, so a planned skip could not tell the root apart) and
   no `metaProbe` (FastUnsafe takes no metadata options).

 @returns Generator of choices.

 @example
 ```ts
 const choices = choiceArbitrary({ fast: false, });
 ```
 */
function choiceArbitrary({ fast, }: { readonly fast: boolean; },): Arbitrary<CustomChoice> {
  return constantFrom<CustomChoice>(
    'absent',
    'false',
    'defaultMerge',
    'undefined',
    'first',
    ...(fast
      ? []
      : [
        'skipNested',
        'metaProbe',
      ] as const),
  );
}

/**
 Filter generator shared by every plan.
 */
const filterArbitrary: Arbitrary<FilterChoice> = constantFrom<FilterChoice>(
  'absent',
  'false',
  'dropNull',
  'dropArrays',
);

/**
 Root metadata and updater switches; FastUnsafe takes neither.

 @param fast - Whether the plan is for a FastUnsafe variant.

 @returns Generators of the two switches.

 @example
 ```ts
 const meta = metaArbitraries({ fast: false, });
 ```
 */
function metaArbitraries({ fast, }: { readonly fast: boolean; },): {
  readonly rootMeta: Arbitrary<boolean>;
  readonly metaUpdater: Arbitrary<MetaUpdaterChoice>;
} {
  return fast
    ? {
      metaUpdater: constant<MetaUpdaterChoice>('absent',),
      rootMeta: constant(false,),
    }
    : {
      metaUpdater: constantFrom<MetaUpdaterChoice>(
        'absent',
        'tagging',
      ),
      rootMeta: boolean(),
    };
}

/**
 Plan generator.

 @param fast - Whether the plan is for a FastUnsafe variant (no skips, no
   metadata options, `maxDepth` ignored by the library).

 @returns Generator of option plans.

 @example
 ```ts
 const plans = optionsPlanArbitrary({ fast: false, });
 ```
 */
export function optionsPlanArbitrary({ fast, }: { readonly fast: boolean; },): Arbitrary<OptionsPlan> {
  /**
   Choice generator shared by the five functions.
   */
  const choice = choiceArbitrary({ fast, },);
  return record({
    ...metaArbitraries({ fast, },),
    filter: filterArbitrary,
    implicit: boolean(),
    maxDepth: oneof(
      constantFrom<MaxDepthChoice>(
        'absent',
        'invalidNaN',
        'invalidNegative',
        'invalidString',
      ),
      integer({
        min: 0,
        max: MAX_PLANNED_DEPTH,
      },),
    ),
    mergeArrays: choice,
    mergeMaps: choice,
    mergeOthers: choice,
    mergeRecords: choice,
    mergeSets: choice,
  },);
}

/**
 Plan generator for the `Into` variants.

 Into custom functions mutate a target slot, so plans use `absent`,
 `false`, `defaultMerge`, `first` (writes the first value), and `metaProbe`
 (writes a marker), and `mergeOthers` may also write `actions.defaultMerge`
 into the slot (`slotDefault`). `mergeRecords` is never `false` or
 `metaProbe`: at the root either makes the call a silent no-op (known
 defect, `./known-defect-options.unit.test.ts`); it tags the target record
 instead (`metaTag`). The filter regions that
 hit known into defects are excluded by the model, not here.

 @param fast - Whether the plan is for `deepmergeIntoFastUnsafeCustom`.

 @returns Generator of option plans.

 @example
 ```ts
 const plans = intoPlanArbitrary({ fast: false, });
 ```
 */
export function intoPlanArbitrary({ fast, }: { readonly fast: boolean; },): Arbitrary<OptionsPlan> {
  /**
   Choices every container function may take.
   */
  const containerChoices: readonly CustomChoice[] = [
    'absent',
    'false',
    'defaultMerge',
    'first',
    ...(fast ? [] : ['metaProbe',] as const),
  ];
  /**
   Choice generator for the Array, Set, and Map functions.
   */
  const choice = constantFrom<CustomChoice>(...containerChoices,);
  return record({
    ...metaArbitraries({ fast, },),
    filter: filterArbitrary,
    implicit: boolean(),
    maxDepth: fast
      ? constantFrom<MaxDepthChoice>('absent',)
      : oneof(
        constantFrom<MaxDepthChoice>(
          'absent',
          'invalidNaN',
          'invalidNegative',
          'invalidString',
        ),
        integer({
          min: 1,
          max: MAX_PLANNED_DEPTH,
        },),
      ),
    mergeArrays: choice,
    mergeMaps: choice,
    mergeOthers: constantFrom<CustomChoice>(
      ...containerChoices,
      'slotDefault',
    ),
    mergeRecords: constantFrom<CustomChoice>(
      'absent',
      'defaultMerge',
      'first',
      ...(fast ? [] : ['metaTag',] as const),
    ),
    mergeSets: choice,
  },);
}
