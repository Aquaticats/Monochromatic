/**
 Metadata handling of the options-aware model (`./options-model.ts`): which
 metadata a custom function receives at a position, and the into
 `mergeRecords` choice `metaTag`, which records it on the target record.

 @module
 */

import type {
  MetaState,
  ModelEvent,
} from './options-model-choice.ts';
import {
  defaultMerge,
  type MergeChild,
  type Slot,
  withProperty,
} from './options-model-merge.ts';
import {
  META_TAG_KEY,
  type OptionsPlan,
  ROOT_META_MARKER,
  TAGGED_META_MARKER,
} from './options-plan.ts';

/**
 Metadata a custom function at this position receives, as a `metaProbe`
 function tells it apart.

 @param plan - Option plan.

 @param fast - Whether the entry point is a FastUnsafe variant (no metadata).

 @param depth - Depth of the position.

 @returns Root metadata at the root when the plan passes it, tagged metadata
   below the root under the tagging updater, otherwise none.

 @example
 ```ts
 metaStateAt({ plan, fast: false, depth: 0, });
 ```
 */
export function metaStateAt(
  {
    plan,
    fast,
    depth,
  }: {
    readonly plan: OptionsPlan;
    readonly fast: boolean;
    readonly depth: number;
  },
): MetaState {
  if (fast)
    return 'none';
  if (depth === 0)
    return plan.rootMeta ? 'root' : 'none';
  return plan.metaUpdater === 'tagging' ? 'tagged' : 'none';
}

/**
 Into `mergeRecords` under `metaTag`: when the metadata is recognized, the
 target record gains {@link META_TAG_KEY} before the default merge. At a
 `target` slot the key then merges like any key the target holds; at a
 `seeded` slot it sits first on the fresh record and the merge never visits
 it.

 @param values - Filtered records.

 @param metaState - Metadata the function recognizes here.

 @param slot - Slot of this position.

 @param observe - Receives the probe event when the metadata is recognized.

 @param mergeChild - Child merge one level deeper.

 @returns Predicted record.

 @throws When the record merge produced a non-object, which it never does.

 @example
 ```ts
 metaTagMerge({ values, metaState: 'root', slot: 'target', observe, mergeChild, });
 ```
 */
export function metaTagMerge(
  {
    values,
    metaState,
    slot,
    observe,
    mergeChild,
  }: {
    readonly values: readonly unknown[];
    readonly metaState: MetaState;
    readonly slot: Slot;
    readonly observe: (event: ModelEvent,) => void;
    readonly mergeChild: MergeChild;
  },
): unknown {
  if (metaState === 'none') {
    return defaultMerge({
      kind: 'record',
      mergeChild,
      slot,
      values,
    },);
  }
  observe(metaState === 'root' ? 'rootMetaProbe' : 'taggedMetaProbe',);
  /**
   Marker the function stores.
   */
  const marker = metaState === 'root' ? ROOT_META_MARKER : TAGGED_META_MARKER;
  /**
   Target record at this slot, before tagging.
   */
  const [first, ...rest] = values;
  if ((slot === 'target')
    && ((typeof first) === 'object')
    && (first !== null)) {
    return defaultMerge({
      kind: 'record',
      mergeChild,
      slot,
      values: [
        withProperty({
          key: META_TAG_KEY,
          record: first,
          value: marker,
        },),
        ...rest,
      ],
    },);
  }
  /**
   Default merge of the sources into the fresh record.
   */
  const merged: unknown = defaultMerge({
    kind: 'record',
    mergeChild,
    slot,
    values,
  },);
  if (((typeof merged) !== 'object') || (merged === null))
    throw new Error('metaTagMerge: a record merge produced a non-object',);
  return Object.defineProperties(
    withProperty({
      key: META_TAG_KEY,
      record: {},
      value: marker,
    },),
    Object.getOwnPropertyDescriptors(merged,),
  );
}
