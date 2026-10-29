/**
 Per-position pieces of the options-aware model (`./options-model.ts`):
 the plan's filter, what a planned custom function returns, the model's
 action markers, and the reach events a property can tally
 (`./reach-tally.ts`).

 @module
 */

import {
  type CustomChoice,
  type FilterChoice,
  ROOT_META_MARKER,
  TAGGED_META_MARKER,
} from './options-plan.ts';

/**
 Marker for "leave this key out", the model's `actions.skip`.
 */
export const SKIP: unique symbol = Symbol('model marker for a skipped key or entry',);

/**
 Marker for "use the default merge", the model's `actions.defaultMerge`.
 */
export const DEFAULT: unique symbol = Symbol('model marker for falling back to the default merge',);

/**
 Metadata a custom function receives at one position, as far as a
 `metaProbe` function can tell: none it recognizes, the plan's root
 metadata, or metadata from the tagging updater.
 */
export type MetaState = 'none' | 'root' | 'tagged';

/**
 Model branch reached while predicting one merge, for reach tallies.

 - `filterDroppedAll`: the array-dropping filter removed every value at a
   position of a returning merge.
 - `intoDropAllKept`: the filter removed every value at a key the into
   target already had, which keeps the target's value.
 - `implicitArray`, `implicitSet`, `implicitMap`: a custom array, Set, or
   Map function returned `undefined` under implicit default merging.
 - `rootMetaProbe`, `taggedMetaProbe`: a `metaProbe` function recognized
   the root or tagged metadata.
 - `nestedSkip`: a `skipNested` function skipped a nested position.
 - `intoSlotWrite`: an into `first` function wrote the target slot.
 - `slotDefault`: an into `mergeOthers` wrote `actions.defaultMerge` into the slot.
 */
export type ModelEvent =
  | 'filterDroppedAll'
    | 'implicitArray'
    | 'implicitMap'
    | 'implicitSet'
    | 'intoDropAllKept'
    | 'intoSlotWrite'
    | 'nestedSkip'
    | 'rootMetaProbe'
    | 'slotDefault'
    | 'taggedMetaProbe';

/**
 Apply the plan's filter.

 @param values - Values at one position.

 @param filter - Filter choice.

 @returns Values that take part in the merge.

 @example
 ```ts
 applyFilter({ values: [1, undefined,], filter: 'absent', }); // [1]
 ```
 */
export function applyFilter({
  values,
  filter,
}: {
  readonly values: readonly unknown[];
  readonly filter: FilterChoice
},): readonly unknown[] {
  if (filter === 'false')
    return values;
  if (filter === 'dropArrays') {
    return values.filter(function notArray(value,) {
      return !Array.isArray(value,);
    },);
  }
  return values.filter(function kept(value,) {
    return (value !== undefined) && ((filter === 'absent') || (value !== null));
  },);
}

/**
 Marker a `metaProbe` function reports for recognized metadata.

 @param metaState - Recognized metadata at this position.

 @returns Root or tagged marker.

 @example
 ```ts
 probeMarker('root'); // ROOT_META_MARKER
 ```
 */
function probeMarker(metaState: Exclude<MetaState, 'none'>,): string {
  return metaState === 'root' ? ROOT_META_MARKER : TAGGED_META_MARKER;
}

/**
 What a planned custom function returns (or, for into functions, leaves in
 the target slot), before fallback.

 @param choice - Custom choice (never `absent` or `false`).

 @param values - Filtered values.

 @param metaAbsent - Whether the call carries no hierarchy (root or FastUnsafe).

 @param metaState - Metadata a `metaProbe` function recognizes here.

 @param observe - Receives the model branches this call reaches.

 @returns Returned value, or a model marker.

 @example
 ```ts
 customResult({ choice: 'first', values: [1, 2,], metaAbsent: true, metaState: 'none', observe, }); // 1
 ```
 */
export function customResult(
  {
    choice,
    values,
    metaAbsent,
    metaState,
    observe,
  }: {
    readonly choice: CustomChoice;
    readonly values: readonly unknown[];
    readonly metaAbsent: boolean;
    readonly metaState: MetaState;
    readonly observe: (event: ModelEvent,) => void;
  },
): unknown {
  if ((choice === 'defaultMerge') || (choice === 'slotDefault'))
    return DEFAULT;
  if (choice === 'undefined')
    return undefined;
  if (choice === 'first')
    return values[0];
  if (choice === 'metaProbe') {
    if (metaState === 'none')
      return DEFAULT;
    observe(metaState === 'root' ? 'rootMetaProbe' : 'taggedMetaProbe',);
    return probeMarker(metaState,);
  }
  if (metaAbsent)
    return DEFAULT;
  observe('nestedSkip',);
  return SKIP;
}
