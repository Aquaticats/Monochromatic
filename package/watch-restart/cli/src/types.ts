import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { HashCache, } from './hash-cache.ts';

/**
 Kind of filesystem event the watcher surfaces.
 
 Mirrors chokidar's file events (add, change, unlink) and directory
 events (addDir, unlinkDir). The orchestrator's `typeFilter` (default
 file-only) gates whether directory events reach the restart driver;
 callers that want directory events pass `--type dir` or include both
 tokens to keep both.
 
 @example
 ```ts
 if (event.kind === 'unlink') cache.delete(event.path,);
 if (event.kind === 'addDir') logger.info(`new dir: ${event.path}`,);
 ```
 */
export type WatchEventKind =
  | 'add'
    | 'change'
    | 'unlink'
    | 'addDir'
    | 'unlinkDir';

/**
 Filesystem entity that an event affects.
 
 Orthogonal to {@link WatchEventKind}: an event's kind names the action
 (created/modified/removed), `entity` names what it acted on. Pre-computed
 by the watcher so filters do not have to inspect kind strings.
 
 @example
 ```ts
 if (event.entity === 'dir') {
   // skip directory events; the dev-server only cares about files
   return false;
 }
 ```
 */
export type WatchEntityType = 'file' | 'dir';

/**
 Normalised filesystem event delivered to filters and the restart driver.
 
 Fields are pre-computed so filters do not have to (re)compute them on
 the hot path: `path` is absolute, `relativePath` is relative to the
 matching watch root, `ext` is `path.extname(path)` (with dot), `entity`
 is `'file'` for `add`/`change`/`unlink` and `'dir'` for `addDir`/`unlinkDir`.
 
 @example
 ```ts
 function tsOnly(event: WatchEvent,): boolean {
   return event.ext === '.ts';
 }
 ```
 */
export type WatchEvent = {
  /**
   Kind of event (see {@link WatchEventKind}).
   */
  readonly kind: WatchEventKind;
  /**
   Filesystem entity affected (file or directory).
   */
  readonly entity: WatchEntityType;
  /**
   Absolute path of the affected file or directory.
   */
  readonly path: string;
  /**
   Path relative to the matching watch root (deepest match when roots nest).
   */
  readonly relativePath: string;
  /**
   File extension including the dot, e.g. `'.ts'`; empty string for extension-less files or directories.
   */
  readonly ext: string;
};

/**
 Context handed to every filter and to filter-internal work.
 
 `hashCache` is shared between the watcher (which pre-populates during the
 initial walk) and `contentHashFilter` (which compares on live events).
 Filters that ignore content do not have to touch it; filters that
 compare content do not have to own it.
 
 `signal` lets long-running filter predicates bail out when the watcher
 shuts down. The orchestrator wires this to its own AbortController so
 `await handle.stop()` cancels in-flight filter work as well as the watcher.
 
 @example
 ```ts
 async function asyncFilter(event: WatchEvent, ctx: WatchCtx,): Promise<boolean> {
   if (ctx.signal.aborted) return false;
   return true;
 }
 ```
 */
export type WatchCtx = {
  /**
   Tagged logger; filters compose deeper tags with `tagged({ tag, l, },)`.
   */
  readonly logger: Logger;
  /**
   Aborts when the orchestrator stops; long-running filters should respect it.
   */
  readonly signal: AbortSignal;
  /**
   Shared content-hash cache; read by `contentHashFilter`, written by the watcher.
   Readonly property prevents cache replacement while preserving its intentional
   stateful capability methods.
   */
  readonly hashCache: HashCache;
};

/**
 Predicate that decides whether an event triggers a restart.
 
 Returning `true` (or a promise resolving to `true`) means "fire";
 `false` means "skip". The orchestrator composes filters with
 `composeFilters(filters,)` (all-of) or `anyFilter(filters,)` (any-of).
 
 Single destructured-object parameter so 2+ logical inputs stay
 named-at-the-call-site; matches AGENTS.md's named-params rule.
 
 @example
 ```ts
 function tsFilter(
   { event, }: { readonly event: WatchEvent; readonly ctx: WatchCtx; },
 ): boolean {
   return event.ext === '.ts';
 }
 ```
 */
export type WatchFilter = (
  args: {
    readonly event: WatchEvent;
    readonly ctx: WatchCtx;
  },
) => boolean | Promise<boolean>;
