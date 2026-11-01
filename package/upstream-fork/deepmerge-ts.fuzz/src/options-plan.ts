/**
 Shapes of generated `deepmergeCustom` option plans, shared by the plan
 generators (`./options-arbitraries.ts`), the builders of real options
 (`./options-build.ts`), and the options-aware model (`./options-model.ts`).

 @module
 */

/**
 Behaviour of one merge function in a plan: left out, `false`, or a custom
 function that
 - `defaultMerge`: returns `actions.defaultMerge`;
 - `undefined`: returns `undefined`;
 - `first`: returns the first value (into: writes it into the target slot);
 - `skipNested`: returns `actions.skip` below the root;
 - `metaProbe`: returns (into: writes) a marker naming the metadata it
   received, root metadata or tagged metadata, and requests the default
   merge otherwise;
 - `slotDefault`: into `mergeOthers` only, writes `actions.defaultMerge`
   into the target slot instead of returning it;
 - `metaTag`: into `mergeRecords` only, adds {@link META_TAG_KEY} holding a
   `metaProbe` marker to the target record when it recognizes the metadata,
   then requests the default merge; the only way an into plan can observe
   `rootMetaData` without replacing the root.
 */
export type CustomChoice =
  | 'absent'
    | 'defaultMerge'
    | 'false'
    | 'first'
    | 'metaProbe'
    | 'metaTag'
    | 'skipNested'
    | 'slotDefault'
    | 'undefined';

/**
 Choice of `filterValues`: left out, `false`, a custom filter dropping both
 `undefined` and `null`, or a custom filter dropping every array (and
 keeping `undefined`), which removes every value at a key holding only
 arrays.
 */
export type FilterChoice = 'absent' | 'dropArrays' | 'dropNull' | 'false';

/**
 Choice of `maxDepth`: left out, a valid depth, or an invalid value that the
 library replaces with its default.
 */
export type MaxDepthChoice = 'absent' | 'invalidNaN' | 'invalidNegative' | 'invalidString' | number;

/**
 Choice of `metaDataUpdater`: left out, or a custom updater that builds the
 same hierarchy entries as the default and also marks the metadata as
 tagged.
 */
export type MetaUpdaterChoice = 'absent' | 'tagging';

/**
 Names of the five merge functions a plan configures.
 */
export type MergeFunctionName = 'mergeArrays' | 'mergeMaps' | 'mergeOthers' | 'mergeRecords' | 'mergeSets';

/**
 Every merge function name, in a fixed order.
 */
export const MERGE_FUNCTION_NAMES: readonly MergeFunctionName[] = [
  'mergeRecords',
  'mergeArrays',
  'mergeSets',
  'mergeMaps',
  'mergeOthers',
];

/**
 One generated option plan.
 */
export type OptionsPlan = Readonly<Record<MergeFunctionName, CustomChoice>> & {
  readonly implicit: boolean;
  readonly filter: FilterChoice;
  readonly maxDepth: MaxDepthChoice;
  /**
   Whether the plan passes `rootMetaData`; never for FastUnsafe, which takes none.
   */
  readonly rootMeta: boolean;
  /**
   Custom `metaDataUpdater`; always `absent` for FastUnsafe, which takes none.
   */
  readonly metaUpdater: MetaUpdaterChoice;
};

/**
 What a `metaProbe` function returns or writes when it receives the plan's
 root metadata.
 */
export const ROOT_META_MARKER = 'metaProbe saw rootMetaData';

/**
 What a `metaProbe` function returns or writes when it receives metadata
 from the tagging updater.
 */
export const TAGGED_META_MARKER = 'metaProbe saw tagged metadata';

/**
 Key a `metaTag` function adds to the into target record it merges into.
 */
export const META_TAG_KEY: unique symbol = Symbol('metaTag marker key',);
