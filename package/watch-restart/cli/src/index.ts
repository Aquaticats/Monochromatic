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
  type SpawnedChildHandle,
  type SpawnFn,
} from './child.ts';
export {
  anyFilter,
  composeFilters,
} from './filter/compose.ts';
export { contentHashFilter, } from './filter/content-hash.ts';
export { extFilter, } from './filter/ext.ts';
export { globFilter, } from './filter/glob.ts';
export { typeFilter, } from './filter/type.ts';
export {
  DEFAULT_MAX_HASH_SIZE_BYTES,
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
export { Watcher, } from './watcher.ts';
export type {
  AwaitWriteFinishOptions,
  IgnoredPredicate,
  WatcherOptions,
} from './watcher-types.ts';
