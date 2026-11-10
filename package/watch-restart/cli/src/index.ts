/**
 Public library entry for `@monochromatic-dev/watch-restart-cli`.
 Re-exports the implementations that consumers compose into a watcher.
 
 Further exports (startWatchRestart, filter helpers) land as each
 underlying module is implemented; the package is staged so each
 commit ships a working build + tests for that module.
 */
export {
  Child,
  type ChildOptions,
  type ChildState,
  DEFAULT_STOP_TIMEOUT_MS,
  type ExitListener,
  type ExitResult,
  NO_CHILD,
  type ProcessSignalFn,
  type SpawnedChildHandle,
  type SpawnFn,
  type WriteClearFn,
} from './child.ts';
export {
  anyFilter,
  composeFilters,
} from './filter/compose.ts';
export { contentHashFilter, } from './filter/content-hash.ts';
export { extFilter, } from './filter/ext.ts';
export { gitignoreFilter, } from './filter/gitignore.ts';
export { globFilter, } from './filter/glob.ts';
export { hiddenFilter, } from './filter/hidden.ts';
export { regexFilter, } from './filter/regex.ts';
export { typeFilter, } from './filter/type.ts';
export {
  DEFAULT_MAX_HASH_SIZE_BYTES,
  DEFAULT_MAX_HASH_SIZE_MIB,
  HashCache,
  type HashCacheOptions,
  OVERSIZED,
} from './hash-cache.ts';
export {
  DEFAULT_DEBOUNCE_MS,
  startWatchRestart,
  type StartWatchRestartOptions,
  type WatchRestartHandle,
} from './start.ts';
export type {
  WatchCtx,
  WatchEntityType,
  WatchEvent,
  WatchEventKind,
  WatchFilter,
} from './types.ts';
//endregion Filters

//region Command-line arguments

/**
 Argument parsing and the flag grammar behind it.

 Imported by the unit suites, which is why the helpers and the parsed shape are
 re-exported here: `cli.ts` is side-effect-free, so this entry can carry them.

 @internal
 */
export {
  argsToOptions,
  parseArgs,
} from './cli.ts';
export type { ParsedArgs, } from './cli-types.ts';
export {
  compileRegex,
  FLAG_UNSET,
  parseKillSignal,
  parseTypeToken,
  resolveBoolPair,
} from './cli-helpers.ts';

//endregion Command-line arguments

export { Watcher, } from './watcher.ts';
export type {
  AwaitWriteFinishOptions,
  IgnoredPredicate,
  WatcherOptions,
} from './watcher-types.ts';
