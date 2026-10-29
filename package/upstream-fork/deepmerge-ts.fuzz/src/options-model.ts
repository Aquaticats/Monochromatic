/**
 Reference model of the customizable entry points for generated option plans
 (`./options-plan.ts`), predicting results from `docs/API.md` and
 `docs/deepmergeCustom.md`.

 Where the docs are silent, the model follows the current dispatch, and each
 such point is pinned separately as an intent question in
 `./known-defect-options.unit.test.ts`:

 - a container present in only one input, and every value at or past
   `maxDepth`, goes to `mergeOthers`, never to the container's function;
 - an option set to `false` resolves to the default `mergeOthers` (the last
   value), not a custom one;
 - invalid `maxDepth` values fall back to 1000.

 Plans never return `actions.skip` at the root: that leaks the symbol to the
 caller (known defect).

 The into variants mutate the target, so the into model tracks whether each
 position is the target's own value (`target`) or a key the target lacks
 (`seeded`). When the filter removes every value at a key the target has,
 the target keeps its value (`./mutation-custom.unit.test.ts`). Known
 regions reported as excluded rather than predicted: every value removed at
 a key the target lacks, and the first value removed while a later one
 survives (`./known-defect-mutation.unit.test.ts`,
 `./known-defect.unit.test.ts`); a Map entry only the target holds, which
 the into merge never visits, when a custom function would change it
 (`./known-defect-options.unit.test.ts`).

 @module
 */

import {
  DEFAULT_MAX_DEPTH,
  kindOf,
  type ValueKind,
} from './model.ts';
import {
  applyFilter,
  customResult,
  DEFAULT,
  type ModelEvent,
  SKIP,
} from './options-model-choice.ts';
import {
  type ChildPosition,
  defaultMerge,
  type MergeChild,
} from './options-model-merge.ts';
import {
  metaStateAt,
  metaTagMerge,
} from './options-model-meta.ts';
import type {
  MaxDepthChoice,
  MergeFunctionName,
  OptionsPlan,
} from './options-plan.ts';

/**
 Option key of each value kind.
 */
const FUNCTION_OF_KIND: Readonly<Record<ValueKind, MergeFunctionName>> = {
  array: 'mergeArrays',
  map: 'mergeMaps',
  other: 'mergeOthers',
  record: 'mergeRecords',
  set: 'mergeSets',
};

/**
 Context shared by every step of one model merge.
 */
type ModelContext = {
  readonly plan: OptionsPlan;
  readonly maxDepth: number;
  readonly fast: boolean;
  readonly into: boolean;
  readonly observe: (event: ModelEvent,) => void;
};

/**
 One position of a model merge, with the context of the whole merge.
 */
type Position = ChildPosition & {
  readonly depth: number;
  readonly context: ModelContext;
};

/**
 Thrown inside the into model when a position falls in a known-defect
 region; {@link modelMergeIntoWithOptions} turns it into an excluded
 prediction.
 */
class ExcludedRegionError extends Error {
  /**
   @param region - Which known-defect region the position falls in.
   */
  constructor(region: string,) {
    super(region,);
    this.name = 'ExcludedRegionError';
  }
}

/**
 Receives model events when the caller does not tally them.

 @param _event - Ignored event.

 @example
 ```ts
 modelMergeWithOptions({ values, plan, fast: false, observe: ignoreEvent, });
 ```
 */
function ignoreEvent(_event: ModelEvent,): void {
  // Intentionally empty: tallies are optional.
}

/**
 Effective `maxDepth` after the invalid-option fallback.

 @param choice - Plan choice.

 @returns Depth the merge stops at.

 @example
 ```ts
 effectiveMaxDepth('invalidNaN'); // 1000
 ```
 */
export function effectiveMaxDepth(choice: MaxDepthChoice,): number {
  return ((typeof choice) === 'number') && (choice >= 0) ? choice : DEFAULT_MAX_DEPTH;
}

/**
 Call the function the plan resolves for a kind, with default fallback.

 @param kind - Kind the dispatch chose (`other` for the mergeOthers path).

 @param position - Filtered values, depth, slot, and context.

 @param mergeChild - Child merge one level deeper.

 @returns Result, possibly the model's skip marker.

 @example
 ```ts
 const merged = dispatch({ kind: 'other', position, mergeChild, });
 ```
 */
function dispatch(
  {
    kind,
    position,
    mergeChild,
  }: {
    readonly kind: ValueKind;
    readonly position: Position;
    readonly mergeChild: MergeChild;
  },
): unknown {
  /**
   Position parts the dispatch reads.
   */
  const {
    context,
    depth,
    slot,
    values,
  } = position;
  /**
   Plan and observer of this merge.
   */
  const {
    plan,
    observe,
  } = context;
  /**
   Plan choice for this kind's function.
   */
  const choice = plan[FUNCTION_OF_KIND[kind]];
  if (choice === 'absent') {
    return defaultMerge({
      kind,
      mergeChild,
      slot,
      values,
    },);
  }
  // `false` resolves to the default mergeOthers: the last value.
  if (choice === 'false')
    return values.at(-1,);
  /**
   Metadata a `metaProbe` function recognizes here.
   */
  const metaState = metaStateAt({
    depth,
    fast: context.fast,
    plan,
  },);
  if (choice === 'metaTag') {
    return metaTagMerge({
      mergeChild,
      metaState,
      observe,
      slot,
      values,
    },);
  }
  /**
   Custom result before fallback.
   */
  const result = customResult({
    choice,
    metaAbsent: context.fast || (depth === 0),
    metaState,
    observe,
    values,
  },);
  if (context.into
    && (choice === 'first')
    && (depth > 0))
    observe('intoSlotWrite',);
  if (context.into && (choice === 'slotDefault'))
    observe('slotDefault',);
  /**
   Whether implicit default merging turns an `undefined` result into the default.
   */
  const implicitFallback = (!context.into)
    && plan.implicit
    && (result === undefined);
  if (implicitFallback && (kind === 'array'))
    observe('implicitArray',);
  if (implicitFallback && ((kind === 'set') || (kind === 'map')))
    observe(kind === 'set' ? 'implicitSet' : 'implicitMap',);
  if ((result === DEFAULT) || implicitFallback) {
    return defaultMerge({
      kind,
      mergeChild,
      slot,
      values,
    },);
  }
  return result;
}

/**
 Values at a position after the plan's filter, with the into regions
 checked first.

 @param position - Unfiltered values (the target's own value first at a
   `target` slot), slot, and context.

 @returns Filtered values.

 @throws {@link ExcludedRegionError} For a known-defect into region.

 @example
 ```ts
 const present = presentValues(position);
 ```
 */
function presentValues(position: Position,): readonly unknown[] {
  /**
   Position parts the filter step reads.
   */
  const {
    context,
    slot,
    values,
  } = position;
  /**
   Filter choice of the plan.
   */
  const { filter, } = context.plan;
  /**
   Values after the plan's filter.
   */
  const present = applyFilter({
    filter,
    values,
  },);
  if (slot === 'none') {
    if ((present.length === 0)
      && (values.length > 0)
      && (filter === 'dropArrays'))
      context.observe('filterDroppedAll',);
    return present;
  }
  if ((present.length === 0) && (slot === 'seeded'))
    throw new ExcludedRegionError('filterValues removed every value at a key the target lacks',);
  if (present.length === 0)
    return present;
  /**
   First value, if the filter keeps it.
   */
  const firstKept = applyFilter({
    filter,
    values: values.slice(
      0,
      1,
    ),
  },);
  if (firstKept.length === 0)
    throw new ExcludedRegionError('filterValues removed the first value while a later one survives',);
  return present;
}

/**
 Recursive model step.

 @param position - Unfiltered values, depth, slot, and context.

 @returns Merged value or the skip marker.

 @example
 ```ts
 const merged = mergeAt({ values, depth: 0, slot: 'none', context, });
 ```
 */
function mergeAt(position: Position,): unknown {
  /**
   Position parts the recursion reads.
   */
  const {
    context,
    depth,
    slot,
    values,
  } = position;
  if (slot === 'targetMapOnly') {
    /**
     What the merge would make of the entry if it visited it; its events never happen.
     */
    const visited = mergeAt({
      ...position,
      context: {
        ...context,
        observe: ignoreEvent,
      },
      slot: 'target',
    },);
    if (!Object.is(
      visited,
      values[0],
    ))
      throw new ExcludedRegionError('deepmergeInto never visits a Map entry only the target holds',);
    return visited;
  }
  /**
   Values after the plan's filter.
   */
  const present = presentValues(position,);
  if (present.length === 0) {
    if (slot !== 'target')
      return undefined;
    context.observe('intoDropAllKept',);
    return values[0];
  }
  /**
   Merge one child position one level deeper.

   @param child - Values and slot under one key.

   @returns Merged child, possibly the skip marker.
   */
  function mergeChild(child: ChildPosition,): unknown {
    return mergeAt({
      ...child,
      context,
      depth: depth + 1,
    },);
  }
  /**
   Kind of the first value.
   */
  const kind = kindOf(present[0],);
  /**
   Whether the values go to mergeOthers instead of their kind's function.
   */
  const toOthers = (depth >= context.maxDepth)
    || (present.length === 1)
    || (kind === 'other')
    || present.some(function differs(value,) {
      return kindOf(value,) !== kind;
    },);
  return dispatch({
    kind: toOthers ? 'other' : kind,
    mergeChild,
    position: {
      ...position,
      values: present,
    },
  },);
}

/**
 Context for one prediction.

 @param plan - Option plan.

 @param fast - Whether to model a FastUnsafe variant (no depth limit, no metadata).

 @param into - Whether to model an into variant.

 @param observe - Receives the model branches the prediction reaches.

 @returns Model context.

 @example
 ```ts
 const context = contextOf({ plan, fast: false, into: false, observe, });
 ```
 */
function contextOf(
  {
    plan,
    fast,
    into,
    observe,
  }: {
    readonly plan: OptionsPlan;
    readonly fast: boolean;
    readonly into: boolean;
    readonly observe: (event: ModelEvent,) => void;
  },
): ModelContext {
  return {
    fast,
    into,
    maxDepth: fast ? Number.POSITIVE_INFINITY : effectiveMaxDepth(plan.maxDepth,),
    observe,
    plan,
  };
}

/**
 Predict a customized merge for a plan: `deepmergeCustom`, or
 `deepmergeFastUnsafeCustom` when `fast` is set.

 @param values - Tree inputs.

 @param plan - Option plan.

 @param fast - Whether to model a FastUnsafe variant.

 @param observe - Receives the model branches the prediction reaches.

 @returns Predicted result.

 @throws When a planned skip reaches the root, which plans must not allow.

 @example
 ```ts
 const expected = modelMergeWithOptions({ values, plan, fast: false, });
 ```
 */
export function modelMergeWithOptions(
  {
    values,
    plan,
    fast,
    observe = ignoreEvent,
  }: {
    readonly values: readonly unknown[];
    readonly plan: OptionsPlan;
    readonly fast: boolean;
    readonly observe?: (event: ModelEvent,) => void;
  },
): unknown {
  /**
   Merge result.
   */
  const merged = mergeAt({
    context: contextOf({
      fast,
      into: false,
      observe,
      plan,
    },),
    depth: 0,
    slot: 'none',
    values,
  },);
  if (merged === SKIP)
    throw new Error('modelMergeWithOptions: a planned skip reached the root',);
  return merged;
}

/**
 Prediction of an into merge: the target's final shape, or the known-defect
 region the inputs fall in.
 */
export type IntoPrediction =
  | {
    readonly modelled: false;
    readonly region: string;
  }
    | {
      readonly modelled: true;
      readonly expected: unknown;
    };

/**
 Predict the target of `deepmergeIntoCustom` (or
 `deepmergeIntoFastUnsafeCustom` when `fast` is set) for a plan.

 @param values - Target first, then the sources.

 @param plan - Into option plan.

 @param fast - Whether to model the FastUnsafe variant.

 @param observe - Receives the model branches the prediction reaches, only
   when the inputs are modelled.

 @returns Predicted target, or the excluded region.

 @throws When the model rejects the plan for another reason.

 @example
 ```ts
 const prediction = modelMergeIntoWithOptions({ values: [target, ...sources,], plan, fast: false, });
 ```
 */
export function modelMergeIntoWithOptions(
  {
    values,
    plan,
    fast,
    observe = ignoreEvent,
  }: {
    readonly values: readonly unknown[];
    readonly plan: OptionsPlan;
    readonly fast: boolean;
    readonly observe?: (event: ModelEvent,) => void;
  },
): IntoPrediction {
  /**
   Events of this prediction, forwarded only when it is modelled.
   */
  const events: ModelEvent[] = [];
  try {
    /**
     Predicted target.
     */
    const expected = mergeAt({
      context: contextOf({
        fast,
        into: true,
        observe: function record(event,) {
          events.push(event,);
        },
        plan,
      },),
      depth: 0,
      slot: 'target',
      values,
    },);
    for (const event of events)
      observe(event,);
    return {
      expected,
      modelled: true,
    };
  } catch (error) {
    if (error instanceof ExcludedRegionError) {
      return {
        modelled: false,
        region: error.message,
      };
    }
    throw error;
  }
}
