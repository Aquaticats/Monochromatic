/**
 Turn generated option plans (`./options-plan.ts`) into real options objects
 for the customizable deepmerge-ts entry points.

 @module
 */

import {
  type CustomChoice,
  META_TAG_KEY,
  MERGE_FUNCTION_NAMES,
  type OptionsPlan,
  ROOT_META_MARKER,
  TAGGED_META_MARKER,
} from './options-plan.ts';

/**
 Minimal view of the utils deepmerge-ts passes to custom functions.
 */
export type ActionUtils = {
  readonly actions: {
    readonly defaultMerge: symbol;
    readonly skip: symbol;
  };
};

/**
 Target slot deepmerge-ts passes to custom into functions.
 */
type IntoSlot = { value: unknown; };

/**
 Returned by into functions that wrote the slot themselves: any symbol other
 than `actions.defaultMerge` means "no action".
 */
const NO_ACTION: unique symbol = Symbol('into function wrote the target slot itself',);

/**
 Returned by {@link metaMarker} for metadata a probe does not recognize.
 */
const UNRECOGNIZED_META: unique symbol = Symbol('metadata a metaProbe function does not recognize',);

/**
 Raw `maxDepth` value of each invalid choice.
 */
const INVALID_MAX_DEPTH: Readonly<Record<'invalidNaN' | 'invalidNegative' | 'invalidString', unknown>> = {
  invalidNaN: Number.NaN,
  invalidNegative: -1,
  invalidString: '2',
};

/**
 Root metadata every plan with `rootMeta` passes; identity is what
 `metaProbe` functions look for.
 */
export const ROOT_META: Readonly<Record<string, unknown>> = Object.freeze({ probe: 'rootMetaData', },);

/**
 Custom filter dropping `undefined` and `null`.

 @param values - Values at one position.

 @returns Values without `undefined` and `null`.

 @example
 ```ts
 dropNullish([1, null, undefined,]); // [1]
 ```
 */
function dropNullish(values: readonly unknown[],): unknown[] {
  return values.filter(function present(value,) {
    return (value !== undefined) && (value !== null);
  },);
}

/**
 Custom filter dropping every array, `undefined` included in what it keeps.

 @param values - Values at one position.

 @returns Values that are not arrays.

 @example
 ```ts
 dropArrays([[1,], 2, undefined,]); // [2, undefined]
 ```
 */
function dropArrays(values: readonly unknown[],): unknown[] {
  return values.filter(function notArray(value,) {
    return !Array.isArray(value,);
  },);
}

/**
 Hierarchy entries carried by a metadata value.

 @param meta - Metadata deepmerge-ts passed along, possibly absent.

 @returns Its `hierarchy` entries, or none when it has no hierarchy.

 @example
 ```ts
 hierarchyOf(undefined); // []
 ```
 */
function hierarchyOf(meta: unknown,): readonly unknown[] {
  if (((typeof meta) !== 'object') || (meta === null))
    return [];
  /**
   Candidate hierarchy.
   */
  const hierarchy: unknown = Reflect.get(
    meta,
    'hierarchy',
  );
  return Array.isArray(hierarchy,) ? hierarchy : [];
}

/**
 Custom `metaDataUpdater` building the same hierarchy entries as the default
 updater, `result` included, and marking the metadata as tagged.

 @returns Updater taking the parent metadata (absent at the root without
   root metadata) and the merge info of the child position, and returning
   metadata with one more hierarchy entry and `tagged: true`.

 @example
 ```ts
 deepmergeCustom({ metaDataUpdater: taggingUpdater(), });
 ```
 */
function taggingUpdater(): (
  previous: unknown,
  info: Readonly<Record<'key' | 'parents' | 'result' | 'values', unknown>>,
) => Record<string, unknown> {
  return function tagged(
    previous,
    info,
  ) {
    /**
     Entry describing the child position, as the default updater builds it.
     */
    const entry = {
      key: info.key,
      parents: info.parents,
      result: info.result,
      values: info.values,
    };
    return {
      ...entry,
      hierarchy: [
        ...hierarchyOf(previous,),
        entry,
      ],
      tagged: true,
    };
  };
}

/**
 Marker a `metaProbe` function reports for the metadata it received.

 @param meta - Metadata passed to the custom function.

 @returns Root or tagged marker, or {@link UNRECOGNIZED_META} for any other metadata.

 @example
 ```ts
 metaMarker(ROOT_META); // ROOT_META_MARKER
 ```
 */
function metaMarker(meta: unknown,): string | typeof UNRECOGNIZED_META {
  if (meta === ROOT_META)
    return ROOT_META_MARKER;
  if (((typeof meta) !== 'object') || (meta === null))
    return UNRECOGNIZED_META;
  return Reflect.get(
    meta,
    'tagged',
  ) === true
    ? TAGGED_META_MARKER
    : UNRECOGNIZED_META;
}

/**
 Build one custom merge function with deepmerge-ts's `(values, utils, meta)`
 signature.

 @param choice - Behaviour to implement; `absent`, `false`, and the into-only
   `slotDefault` never reach here.

 @returns Custom merge function.

 @example
 ```ts
 const first = customFunction('first');
 ```
 */
function customFunction(choice: CustomChoice,): (
  values: readonly unknown[],
  utils: ActionUtils,
  meta: unknown,
) => unknown {
  return function planned(
    values,
    utils,
    meta,
  ) {
    if (choice === 'defaultMerge')
      return utils.actions
        .defaultMerge;
    if (choice === 'undefined')
      return undefined;
    if (choice === 'first')
      return values[0];
    if (choice === 'metaProbe') {
      /**
       Marker for the metadata received.
       */
      const marker = metaMarker(meta,);
      return marker === UNRECOGNIZED_META ? utils.actions
        .defaultMerge : marker;
    }
    // skipNested: only nested calls carry a hierarchy; skipping at the root leaks the symbol.
    /**
     Hierarchy the call carries; empty at the root and in FastUnsafe.
     */
    const hierarchy = hierarchyOf(meta,);
    return hierarchy.length === 0 ? utils.actions
      .defaultMerge : utils.actions
        .skip;
  };
}

/**
 Turn a plan into options for `deepmergeCustom` or
 `deepmergeFastUnsafeCustom`, including deliberately invalid `maxDepth`.

 @param plan - Generated plan.

 @returns Options object.

 @example
 ```ts
 const options = buildOptions(plan);
 ```
 */
export function buildOptions(plan: OptionsPlan,): Record<string, unknown> {
  /**
   Options accumulated from the plan.
   */
  const options: Record<string, unknown> = { enableImplicitDefaultMerging: plan.implicit, };
  for (const name of MERGE_FUNCTION_NAMES) {
    /**
     Choice for this function.
     */
    const choice = plan[name];
    if (choice === 'false')
      options[name] = false;
    else if (choice !== 'absent')
      options[name] = customFunction(choice,);
  }
  if (plan.filter === 'false')
    options.filterValues = false;
  if (plan.filter === 'dropNull')
    options.filterValues = dropNullish;
  if (plan.filter === 'dropArrays')
    options.filterValues = dropArrays;
  if ((typeof plan.maxDepth) === 'number')
    options.maxDepth = plan.maxDepth;
  else if (plan.maxDepth !== 'absent')
    options.maxDepth = INVALID_MAX_DEPTH[plan.maxDepth];
  if (plan.metaUpdater === 'tagging')
    options.metaDataUpdater = taggingUpdater();
  return options;
}

/**
 Build one custom into function with deepmerge-ts's
 `(target, values, utils, meta)` signature.

 @param choice - Behaviour to implement: `defaultMerge`, `first`,
   `metaProbe`, `metaTag`, or `slotDefault`.

 @returns Custom into function.

 @throws When the choice has no into form.

 @example
 ```ts
 const writeFirst = customIntoFunction('first');
 ```
 */
function customIntoFunction(choice: CustomChoice,): (
  slot: IntoSlot,
  values: readonly unknown[],
  utils: ActionUtils,
  meta: unknown,
) => symbol {
  if ((choice === 'absent')
    || (choice === 'false')
    || (choice === 'skipNested')
    || (choice === 'undefined'))
    throw new Error(`buildIntoOptions: ${choice} has no into form`,);
  return function plannedInto(
    slot,
    values,
    utils,
    meta,
  ) {
    if (choice === 'defaultMerge')
      return utils.actions
        .defaultMerge;
    if (choice === 'first') {
      /**
       Value the slot takes.
       */
      const [first,] = values;
      slot.value = first;
      return NO_ACTION;
    }
    if (choice === 'slotDefault') {
      slot.value = utils.actions
        .defaultMerge;
      return NO_ACTION;
    }
    /**
     Marker for the metadata received, if any.
     */
    const marker = metaMarker(meta,);
    if (marker === UNRECOGNIZED_META)
      return utils.actions
        .defaultMerge;
    if (choice === 'metaTag') {
      /**
       Target record this call merges into.
       */
      const record = slot.value;
      if (((typeof record) !== 'object') || (record === null))
        throw new Error('metaTag: mergeRecords received a non-record target slot',);
      Reflect.defineProperty(
        record,
        META_TAG_KEY,
        {
          configurable: true,
          enumerable: true,
          value: marker,
          writable: true,
        },
      );
      return utils.actions
        .defaultMerge;
    }
    slot.value = marker;
    return NO_ACTION;
  };
}

/**
 Turn an into plan into options for the `Into` variants, whose functions
 take the target reference first.

 @param plan - Plan from `intoPlanArbitrary`.

 @returns Options object.

 @throws When the plan holds a choice the into variants cannot express.

 @example
 ```ts
 const options = buildIntoOptions(plan);
 ```
 */
export function buildIntoOptions(plan: OptionsPlan,): Record<string, unknown> {
  /**
   Non-function options from the shared builder.
   */
  const options: Record<string, unknown> = buildOptions({
    ...plan,
    mergeArrays: 'absent',
    mergeMaps: 'absent',
    mergeOthers: 'absent',
    mergeRecords: 'absent',
    mergeSets: 'absent',
  },);
  for (const name of MERGE_FUNCTION_NAMES) {
    /**
     Choice for this function.
     */
    const choice = plan[name];
    if (choice === 'false')
      options[name] = false;
    else if (choice !== 'absent')
      options[name] = customIntoFunction(choice,);
  }
  return options;
}
