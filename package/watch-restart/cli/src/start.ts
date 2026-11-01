import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

import {
  Child,
  type SpawnFn,
} from './child.ts';
import { composeFilters, } from './filter/compose.ts';
import { contentHashFilter, } from './filter/content-hash.ts';
import { extFilter, } from './filter/ext.ts';
import { gitignoreFilter, } from './filter/gitignore.ts';
import { globFilter, } from './filter/glob.ts';
import { hiddenFilter, } from './filter/hidden.ts';
import { regexFilter, } from './filter/regex.ts';
import { typeFilter, } from './filter/type.ts';
import {
  DEFAULT_MAX_HASH_SIZE_BYTES,
  HashCache,
} from './hash-cache.ts';
import type {
  WatchCtx,
  WatchEntityType,
  WatchEvent,
  WatchEventKind,
  WatchFilter,
} from './types.ts';
import { Watcher, } from './watcher.ts';

/**
 Logger root for watch-restart after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: defaultLogger, },);
 ```
 */
const defaultLogger = tagged({ tag: 'watch-restart', },);

/**
 Default debounce window (ms) between the last qualifying event and the
 actual restart. Two events inside the window coalesce to one restart;
 100 ms is short enough to feel instant on a save and long enough to
 absorb a multi-file format-on-save burst.
 */
export const DEFAULT_DEBOUNCE_MS = 100;

/**
 Real sentinel for "no debounce timer is currently armed". A unique
 `Symbol` rather than `undefined` so the timer container's type stays free
 of a nullish union; the orchestrator compares against it to decide whether
 a pending timer must be cleared.
 */
const NO_TIMER: unique symbol = Symbol('watch restart debounce timer is not armed',);

/**
 Options for {@link startWatchRestart}.
 
 Mirrors the CLI flag surface (`-w`, `-i`, `-e`, `--ext`, ...) so the CLI
 parses straight into this shape with no second translation layer. Each
 field that gates a filter is `?`-optional and unset means "do not
 apply this filter dimension"; `contentChanged === undefined` defaults
 to `true` (the package's reason for being): only an explicit `false`
 opts out.
 */
export type StartWatchRestartOptions = {
  /**
   Watch roots; at least one is expected by {@link Watcher}.
   */
  readonly paths: readonly string[];
  /**
   Command to run; first positional after `--` at the CLI.
   */
  readonly command: string;
  /**
   Argument list for the command; remaining positionals at the CLI.
   */
  readonly args?: readonly string[];
  /**
   Include glob patterns matched against {@link WatchEvent.relativePath}.
   */
  readonly include?: readonly string[];
  /**
   Exclude glob patterns; an exclude match short-circuits to skip.
   */
  readonly exclude?: readonly string[];
  /**
   Include regex patterns matched against {@link WatchEvent.relativePath} (any-match OR).
   */
  readonly includeRegex?: readonly RegExp[];
  /**
   Exclude regex patterns matched against {@link WatchEvent.relativePath} (any-match short-circuits skip).
   */
  readonly excludeRegex?: readonly RegExp[];
  /**
   Extensions admitted (case-insensitive, leading dot optional).
   */
  readonly extensions?: readonly string[];
  /**
   Entity types admitted; `undefined` defaults to `['file']` so the
   dev-server case (the package's reason for being) sees only file
   events. Pass `['file', 'dir']` to include directory create/remove.
   */
  readonly types?: readonly WatchEntityType[];
  /**
   Event kinds admitted; `undefined` admits all kinds reaching the filter.
   */
  readonly events?: readonly WatchEventKind[];
  /**
   Include hidden files and directories (segments starting with `.`).
   Defaults to `false`; an explicit `true` lets dotfiles through.
   */
  readonly hidden?: boolean;
  /**
   Follow symbolic links during traversal. Defaults to `false` (chokidar's
   safer default); opt-in for projects whose watch roots include symlinked
   vendor directories.
   */
  readonly followSymlinks?: boolean;
  /**
   Respect `.gitignore` files in the watched tree. Defaults to `true`;
   `false` lets ignored paths through (e.g. when watching `dist/` is intentional).
   */
  readonly gitignore?: boolean;
  /**
   Extra gitignore-format files whose patterns AND with `.gitignore` (when enabled).
   */
  readonly ignoreFiles?: readonly string[];
  /**
   Maximum directory-descent depth from each watch root; `undefined` is unlimited.
   */
  readonly depth?: number;
  /**
   Polling interval (ms); `undefined` uses native filesystem events.
   */
  readonly poll?: number;
  /**
   Suppress byte-identical writes when `true` (default); `false` disables.
   */
  readonly contentChanged?: boolean;
  /**
   Cap on file size hashed by {@link contentHashFilter}; default 16 MiB.
   */
  readonly maxHashSize?: number;
  /**
   Debounce window (ms); coalesces multi-event bursts. Default 100.
   */
  readonly debounce?: number;
  /**
   SIGTERM-to-SIGKILL grace (ms) for the child; default 5_000.
   */
  readonly stopTimeout?: number;
  /**
   Run the child at start when `true` (default); `false` defers to first event.
   */
  readonly initial?: boolean;
  /**
   Clear the terminal (`\\x1b[2J\\x1b[H`) before each child re-spawn; default `false`.
   */
  readonly clear?: boolean;
  /**
   Signal sent to the child before the SIGKILL fallback; default `'SIGTERM'`.
   Named `killSignal` so it never collides with the orchestrator's internal
   `AbortSignal` shutdown plumbing.
   */
  readonly killSignal?: NodeJS.Signals;
  /**
   Spawn the child as a process-group leader (`detached: true`) and signal
   the whole group (`process.kill(-pid, sig)`). Default `true`; turning off
   limits signals to the direct child pid only.
   */
  readonly processGroup?: boolean;
  /**
   Optional user predicate AND'd onto the internal chain (runs last).
   */
  readonly filter?: WatchFilter;
  /**
   Parent logger; the orchestrator composes a {@link startWatchRestart} tag on top.
   */
  readonly logger?: Logger;
  /**
   Spawn factory forwarded to {@link Child}; tests inject a recording fake.
   */
  readonly spawn?: SpawnFn;
};

/**
 Handle returned by {@link startWatchRestart}.
 
 The orchestrator's lifetime is bounded by `stop()`: a second `stop()`
 call is a no-op (idempotent), matching `Watcher.stop` and `Child.stop`.
 */
export type WatchRestartHandle = {
  /**
   Aborts ctx.signal, clears debounce timer, stops watcher then child.
   */
  readonly stop: () => Promise<void>;
};

/**
 Builds a {@link WatchFilter} that passes only events whose `kind` is in
 the allowed list.
 
 Module-scope helper rather than a generic `filters/event-kind.ts`
 module because this is the only consumer; the orchestrator already
 owns the option-to-filter mapping and pulling a one-line filter into
 its own file would obscure the chain.
 
 @param allowed - event kinds admitted; checked via Set membership
 
 @returns watch filter that returns `true` iff `event.kind` is admitted
 
 @example
 ```ts
 const filter = buildEventKindFilter(['change',],);
 ```
 */
function buildEventKindFilter(
  allowed: readonly WatchEventKind[],
): WatchFilter {
  /**
   Set built once at construction for O(1) lookup on the event hot path.
   */
  const allowedSet: ReadonlySet<WatchEventKind> = new Set<WatchEventKind>(
    allowed,
  );
  return function eventKindFilterFn(
    {
      event,
    }: {
      readonly event: WatchEvent;
      readonly ctx: WatchCtx;
    },
  ): boolean {
    return allowedSet.has(event.kind,);
  };
}

/**
 Assembles the internal filter chain from {@link StartWatchRestartOptions}.
 
 Order matters for efficiency: cheap sync filters first (event kind, ext,
 glob), then I/O (content hash), then the user filter last (opaque cost).
 Within an `all-of` composition this is a strict short-circuit: a non-
 matching ext never causes a disk hash.
 
 `contentChanged === undefined` includes the hash filter (package's
 reason for being); only an explicit `false` opts out. Naive
 `if (options.contentChanged)` would skip the filter when unset and
 silently break the dev-loop contract.
 
 @param options - same shape passed to {@link startWatchRestart}
 
 @returns single composed {@link WatchFilter} (potentially vacuous true)
 
 @example
 ```ts
 const filter = buildInternalFilter({
   paths: ['src',], command: 'node',
   extensions: ['.ts',], contentChanged: true,
 },);
 ```
 */
async function buildInternalFilter(
  options: StartWatchRestartOptions,
): Promise<WatchFilter> {
  /**
   Mutable working list of filters; collected in evaluation order before composition.
   */
  const filters: WatchFilter[] = [];

  /**
   Default `types` to `['file']` so the baseline dev-server case (the
   package's reason for being) ignores the new `addDir`/`unlinkDir`
   events the watcher started emitting in Q6. Callers that want
   directory events pass `['file', 'dir']` or `['dir']`.
   */
  const types: readonly WatchEntityType[] = options.types
    ?? ['file',];
  filters.push(typeFilter(types,),);

  if (options.events
    !== undefined)
    filters.push(buildEventKindFilter(options.events,),);
  if ((options.extensions
    !== undefined) && (options.extensions
      .length
      > 0))
    filters.push(extFilter(options.extensions,),);
  if (
    ((options.include
      !== undefined) && (options.include
        .length
        > 0))
      || ((options.exclude
        !== undefined) && (options.exclude
          .length
          > 0))
  ) {
    filters.push(globFilter({
      ...(options.include
        === undefined ? {} : { include: options.include, }),
      ...(options.exclude
        === undefined ? {} : { exclude: options.exclude, }),
    },),);
  }
  if (
    ((options.includeRegex
      !== undefined) && (options.includeRegex
        .length
        > 0))
      || ((options.excludeRegex
        !== undefined) && (options.excludeRegex
          .length
          > 0))
  ) {
    filters.push(regexFilter({
      ...(options.includeRegex
        === undefined
        ? {}
        : { include: options.includeRegex, }),
      ...(options.excludeRegex
        === undefined
        ? {}
        : { exclude: options.excludeRegex, }),
    },),);
  }
  if (options.hidden
    !== true)
    filters.push(hiddenFilter(),);

  /**
   Resolved `.gitignore` roots: only the watch roots when `--gitignore`
   is on (default); empty when off (`--no-gitignore`). The factory still
   loads from `extraFiles` regardless, per the plan's "separate AND"
   semantics (gitignore and ignore-file are independent dimensions).
   */
  const gitignoreRoots: readonly string[] = options.gitignore
    === false
    ? []
    : options.paths;
  /**
   Resolved extra ignore files; empty when none configured.
   */
  const gitignoreExtraFiles: readonly string[] = options.ignoreFiles
    ?? [];
  if ((gitignoreRoots.length
    > 0) || (gitignoreExtraFiles.length
      > 0)) {
    filters.push(
      await gitignoreFilter({
        roots: gitignoreRoots,
        extraFiles: gitignoreExtraFiles,
      },),
    );
  }

  if (options.contentChanged
    !== false)
    filters.push(contentHashFilter(),);
  if (options.filter
    !== undefined)
    filters.push(options.filter,);

  return composeFilters(filters,);
}

/**
 Starts a long-running watch/restart loop.
 
 Wires the four building blocks together:
 
 1. {@link HashCache} (shared, pre-populated by Watcher's initial walk).
 2. {@link Watcher} (chokidar adapter; absorbs pre-`ready` events into the cache).
 3. Filter chain assembled by {@link buildInternalFilter} (the configured
    flag-derived filters AND'd with the user-supplied predicate).
 4. {@link Child} (the spawn/restart/stop wrapper; receives `SIGTERM`
    with `stopTimeout` grace before `SIGKILL`).
 
 Each post-`ready` event runs through the filter chain. A passing event
 (re)schedules a debounced restart: the timer is reset on every passing
 event, so a burst within the window coalesces to a single restart at
 the end of the debounce period. When `options.initial !== false`, the child is
 spawned immediately after the watcher reaches ready.
 
 The handle's `stop()` aborts the shared abort signal first (so any
 in-flight filter awaiting `ctx.signal` exits cleanly), then clears
 the debounce timer, stops the watcher, and stops the child. Calling
 `stop()` twice is a no-op on every layer (idempotent).
 
 @param options - options object; see {@link StartWatchRestartOptions}
 
 @returns handle whose `stop()` tears the whole loop down cleanly
 
 @example
 ```ts
 const handle = await startWatchRestart({
   paths: ['src/server',],
   command: 'node',
   args: ['src/server/index.ts',],
 },);
 // ... later, on Ctrl+C:
 await handle.stop();
 ```
 */
export async function startWatchRestart(
  options: StartWatchRestartOptions,
): Promise<WatchRestartHandle> {
  /**
   Resolved parent logger; the orchestrator and inner subsystems compose tags onto it.
   */
  const parentLogger: Logger = options.logger
    ?? defaultLogger;
  /**
   Tagged logger for this orchestrator's own log lines.
   */
  const startLogger: Logger = tagged({
    tag: startWatchRestart.name,
    l: parentLogger,
  },);

  /**
   Shared content-hash cache; pre-populated by the {@link Watcher}, read by
   {@link contentHashFilter}.
   */
  const hashCache = new HashCache({
    maxHashSize: options.maxHashSize
      ?? DEFAULT_MAX_HASH_SIZE_BYTES,
  },);

  /**
   Shared abort controller; flipped during `stop()` so in-flight filters can bail.
   */
  const abort = new AbortController();
  /**
   Context handed to every filter invocation.
   */
  const ctx: WatchCtx = {
    logger: startLogger,
    signal: abort.signal,
    hashCache,
  };

  /**
   Composed filter chain assembled once at start; evaluated on every event.
   */
  const internalFilter: WatchFilter = await buildInternalFilter(options,);
  /**
   Resolved debounce delay.
   */
  const debounceMs: number = options.debounce
    ?? DEFAULT_DEBOUNCE_MS;

  /**
   Underlying child manager.
   */
  const child = new Child({
    command: options.command,
    ...(options.args
      === undefined ? {} : { args: options.args, }),
    ...(options.stopTimeout
      === undefined
      ? {}
      : { stopTimeout: options.stopTimeout, }),
    ...(options.killSignal
      === undefined
      ? {}
      : { killSignal: options.killSignal, }),
    ...(options.processGroup
      === undefined
      ? {}
      : { processGroup: options.processGroup, }),
    ...(options.clear
      === undefined ? {} : { clear: options.clear, }),
    logger: startLogger,
    ...(options.spawn
      === undefined ? {} : { spawn: options.spawn, }),
  },);

  /**
   Const-bound mutable container for the debounce timer handle.
   Avoids the function-root `let` ban while keeping the timer addressable
   from {@link scheduleRestart} and `stop`.
   */
  const state: {
    timer: ReturnType<typeof setTimeout> | typeof NO_TIMER;
  } = { timer: NO_TIMER, };

  /**
   Resets the debounce window: a new event ties for "latest", so the
   restart fires `debounceMs` after THIS event, not the original first
   one. Timer handle is cleared before the async restart starts so a
   fresh event arriving mid-restart cannot see a stale handle.
   */
  function scheduleRestart(): void {
    if (state.timer
      !== NO_TIMER)
      clearTimeout(state.timer,);
    state.timer = setTimeout(
      function fireRestart(): void {
        state.timer = NO_TIMER;
        void (async function doRestart(): Promise<void> {
          try {
            await child.restart();
          }
          catch (error) {
            /**
             Human-readable error string used in the restart-failure log line.
             */
            const message = caughtValueText(error,);
            startLogger.error(`restart failed: ${message}`,);
          }
        })();
      },
      debounceMs,
    );
  }

  /**
   Watcher's per-event callback. Runs the filter chain; on `true`, kicks
   the debounce; on `false`, drops the event. Filter exceptions are
   logged but never propagated (a single bad predicate must not crash
   the dev loop).
   
   @param event - normalised event from {@link Watcher}
   */
  async function onEvent(event: WatchEvent,): Promise<void> {
    try {
      /**
       Composed filter verdict; `true` means the event should trigger a debounced restart.
       */
      const passed = await internalFilter({
        event,
        ctx,
      },);
      if (passed)
        scheduleRestart();
    }
    catch (error) {
      /**
       Human-readable error string used in the filter-failure log line.
       */
      const message = caughtValueText(error,);
      startLogger.error(
        `filter chain failed for ${event.path}: ${message}`,
      );
    }
  }

  /**
   Watcher instance owned by this orchestrator; closed during `stop()`.
   */
  const watcher = new Watcher({
    paths: options.paths,
    hashCache,
    onEvent,
    logger: startLogger,
    ...(options.depth
      === undefined ? {} : { depth: options.depth, }),
    ...(options.poll
      === undefined ? {} : { poll: options.poll, }),
    ...(options.followSymlinks
      === undefined
      ? {}
      : { followSymlinks: options.followSymlinks, }),
  },);

  await watcher.untilReady();

  if (options.initial
    !== false)
    await child.start();

  /**
   Tears down the loop. Order matters: abort first so in-flight filter
   awaits on `ctx.signal` return early; then drop any pending debounce
   to prevent a phantom restart during teardown; finally stop the
   watcher (which closes OS handles) and the child (which sends
   SIGTERM/SIGKILL). Both `Watcher.stop` and `Child.stop` are
   idempotent, so a second `handle.stop()` is harmless.
   */
  async function stop(): Promise<void> {
    abort.abort();
    if (state.timer
      !== NO_TIMER) {
      clearTimeout(state.timer,);
      state.timer = NO_TIMER;
    }
    await watcher.stop();
    await child.stop();
  }

  return { stop, };
}
