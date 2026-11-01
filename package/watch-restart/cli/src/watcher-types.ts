import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { HashCache, } from './hash-cache.ts';
import type { WatchEvent, } from './types.ts';

/**
 Predicate testing whether a path should be skipped by chokidar's traversal.
 
 `Matcher` in chokidar 5 admits string (literal path), RegExp, function, or
 `{ path, recursive }`. The orchestrator compiles all flag inputs to
 functions ahead of time (via picomatch), so this adapter accepts the
 function form only and ignores the others.
 
 @example
 ```ts
 const isNodeModules: IgnoredPredicate = function isNm(p,) {
   return p.includes('/node_modules/',);
 };
 ```
 */
export type IgnoredPredicate = (path: string,) => boolean;

/**
 Stability/polling tuning for chokidar's `awaitWriteFinish`.
 Splitting it out keeps the {@link WatcherOptions} shape narrow.
 */
export type AwaitWriteFinishOptions = {
  /**
   Milliseconds the file size must hold steady before an event emits.
   */
  readonly stabilityThreshold: number;
  /**
   Milliseconds between size re-stats during the stability period.
   */
  readonly pollInterval: number;
};

/**
 Default `awaitWriteFinish.stabilityThreshold` (milliseconds).
 Tuned for editors that write the final byte then move on; matches
 chokidar's recommended floor for "saved by a human" timing.
 */
const DEFAULT_STABILITY_THRESHOLD_MS = 50;

/**
 Default `awaitWriteFinish.pollInterval` (milliseconds).
 How often chokidar re-stats a candidate file while waiting for stability;
 10ms keeps the latency low without burning CPU on a typical dev box.
 */
const DEFAULT_POLL_INTERVAL_MS = 10;

/**
 Default `awaitWriteFinish` block applied by {@link Watcher} when the caller
 passes none. Kept here next to {@link AwaitWriteFinishOptions} so the
 watcher module stays under its line cap.
 */
export const DEFAULT_AWAIT_WRITE_FINISH: AwaitWriteFinishOptions = {
  stabilityThreshold: DEFAULT_STABILITY_THRESHOLD_MS,
  pollInterval: DEFAULT_POLL_INTERVAL_MS,
};

/**
 Construction options for {@link Watcher}.
 */
export type WatcherOptions = {
  /**
   Roots to watch. Resolved to absolute on construction.
   */
  readonly paths: readonly string[];
  /**
   Shared content-hash cache; pre-populated during the initial walk.
   Readonly property prevents replacement while preserving intentional
   stateful capability methods.
   */
  readonly hashCache: HashCache;
  /**
   Predicates that skip files/directories during traversal (efficiency only; not a filter substitute).
   */
  readonly ignored?: readonly IgnoredPredicate[];
  /**
   Forwarded to chokidar's `atomic` option; defaults to `true`.
   */
  readonly atomic?: boolean | number;
  /**
   Forwarded to chokidar's `awaitWriteFinish`; defaults to `{ stabilityThreshold: 50, pollInterval: 10 }`.
   */
  readonly awaitWriteFinish?: AwaitWriteFinishOptions;
  /**
   Maximum subdirectory depth chokidar will descend from each root.
   `undefined` is chokidar's default (unlimited). `0` watches only the
   root directory's direct files.
   */
  readonly depth?: number;
  /**
   Polling interval (ms). When set, chokidar uses `usePolling: true`
   with this value for `interval`. When `undefined`, native filesystem
   events are used (chokidar default). Set this on filesystems without
   inotify support (NFS mounts, WSL1-on-Windows-FS, some Docker setups).
   */
  readonly poll?: number;
  /**
   Whether chokidar follows symbolic links when traversing watch roots.
   Defaults to `false` (this package's safer default), regardless of
   chokidar's own default; passed explicitly so the value is not
   silently flipped by a chokidar version bump.
   */
  readonly followSymlinks?: boolean;
  /**
   Callback fired for each post-`ready` event; awaited by the watcher so
   filter work completes before the next dispatch settles. Async by
   contract (the orchestrator's handler hashes files); chokidar itself
   does not await it.
   */
  readonly onEvent: (event: WatchEvent,) => Promise<void>;
  /**
   Parent logger; the watcher composes a {@link Watcher} tag on top.
   */
  readonly logger?: Logger;
};
