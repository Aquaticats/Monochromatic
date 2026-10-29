import {
  type FSWatcher,
  watch as chokidarWatch,
} from 'chokidar';
import { once, } from 'node:events';
import { extname, } from 'node:path';
import { caughtValueText as describeError, } from '@monochromatic-dev/caught-value-core/ts';
import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

import {
  type HashCache,
  OVERSIZED,
} from './hash-cache.ts';
import type {
  WatchEntityType,
  WatchEvent,
  WatchEventKind,
} from './types.ts';
import {
  relativePathForRoots,
  sortRootsByLengthDesc,
} from './watcher-paths.ts';
import {
  DEFAULT_AWAIT_WRITE_FINISH,
  type WatcherOptions,
} from './watcher-types.ts';

/**
 Logger root for watch-restart after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: defaultLogger, },);
 ```
 */
const defaultLogger = tagged({ tag: 'watch-restart', },);

/* oxlint-disable no-restricted-syntax/no-class -- per-instance watcher state: one Watcher (owning one chokidar FSWatcher and its pre-populate set) lives per `startWatchRestart()` call, state is `#private`-encapsulated, and the class is an exported library primitive consumers instantiate via `new`; module-level state cannot model multiple concurrent watch sessions. */
/**
 chokidar 5 adapter that owns one {@link FSWatcher}, pre-populates a {@link HashCache}
 during the initial walk (events before `ready`), and forwards live events
 (events after `ready`) to {@link WatcherOptions.onEvent} as normalised
 {@link WatchEvent}s.
 
 Pre-`ready` events are absorbed: their handler hashes the file and stores
 the digest, but does not call `onEvent`. This is the "first sighting is not
 a change" rule the README and HANDOVER document; without it, every file on
 disk would look like an add event the first time the dev loop boots.
 
 `untilReady()` waits for chokidar's `ready` event AND drains any in-flight
 pre-populate hash work. Without the drain, post-`ready` events for files
 whose pre-populate is still running would race the empty cache and fire a
 spurious restart.
 
 @example
 ```ts
 const watcher = new Watcher({
   paths: ['src/server',],
   hashCache,
   onEvent: async function onEvent(event,) { console.log(event,); },
 },);
 await watcher.untilReady();
 // ... later
 await watcher.stop();
 ```
 */
export class Watcher {
  /**
   Underlying chokidar instance.
   */
  readonly #fsw: FSWatcher;
  /**
   Watch roots resolved to absolute, sorted deepest-first for `#relativePathFor`.
   */
  readonly #resolvedRoots: readonly string[];
  /**
   Shared hash cache; the watcher writes during pre-populate, filters read post-ready.
   */
  readonly #hashCache: Readonly<HashCache>;
  /**
   Live-event callback handed in by the orchestrator; awaited per event.
   */
  readonly #onEvent: (event: WatchEvent,) => Promise<void>;
  /**
   Tagged logger composed onto the parent.
   */
  readonly #logger: Logger;
  /**
   In-flight pre-populate promises; drained by {@link untilReady}.
   */
  readonly #prePopulate: Set<Promise<void>> = new Set<Promise<void>>();
  /**
   Flips `true` once chokidar emits `ready`; flips event handling from pre-populate to forward.
   */
  #ready: boolean = false;

  /**
   Wires chokidar up but does not block for the initial walk; call
   {@link untilReady} to wait for the pre-populate to drain.
   
   @param options - construction options
   
   @example
   ```ts
   const watcher = new Watcher({ paths: ['src',], hashCache, onEvent, },);
   await watcher.untilReady();
   ```
   */
  constructor(options: WatcherOptions,) {
    this.#resolvedRoots = sortRootsByLengthDesc(options.paths,);
    this.#hashCache = options.hashCache;
    this.#onEvent = options.onEvent;
    this.#logger = tagged({
      tag: Watcher.name,
      l: options.logger
        ?? defaultLogger,
    },);

    this.#fsw = chokidarWatch(
      [...this.#resolvedRoots,],
      {
        atomic: options.atomic
          ?? true,
        awaitWriteFinish: options.awaitWriteFinish
          ?? DEFAULT_AWAIT_WRITE_FINISH,
        ignoreInitial: false,
        persistent: true,
        // Always pass `followSymlinks` so the package's default (false) is
        // enforced regardless of chokidar's own default value.
        followSymlinks: options.followSymlinks
          === true,
        // Conditional spread keeps `ignored` absent when the caller passes none;
        // ChokidarOptions's `Partial<>` allows missing keys but not explicit `undefined`.
        ...(options.ignored
          === undefined
          ? {}
          : { ignored: [...options.ignored,], }),
        // `depth` absent leaves chokidar's default (unlimited) intact.
        ...(options.depth
          === undefined ? {} : { depth: options.depth, }),
        // Polling: only enable when an explicit interval is given; both
        // `usePolling` and `interval` are spread together so the keys move
        // as one unit rather than leaving a stray `usePolling: false`.
        ...(options.poll
          === undefined
          ? {}
          : {
            usePolling: true,
            interval: options.poll,
          }),
      },
    );

    /**
     Captured `this` for sync chokidar listeners that void-call async members with their own try/catch.
     */
    const self = this;

    /**
     chokidar `ready` listener.
     Synchronously sets the ready flag; no async work to await.
     */
    function onReady(): void {
      self.#ready = true;
      self.#logger
        .info('ready',);
    }

    /**
     chokidar `add` listener.
     Wraps `dispatchAdd` in an IIFE with internal try/catch so chokidar's
     EventEmitter sees a sync function and rejections cannot be dropped.
     
     @param path - absolute path emitted by chokidar
     */
    function onAdd(path: string,): void {
      void (async function dispatchAdd(): Promise<void> {
        try {
          await self.#dispatchAddOrChange(
            'add',
            path,
          );
        }
        catch (error) {
          self.#logger
            .error(`add dispatch failed: ${describeError(error,)}`,);
        }
      })();
    }

    /**
     chokidar `change` listener.
     
     @param path - absolute path emitted by chokidar
     */
    function onChange(path: string,): void {
      void (async function dispatchChange(): Promise<void> {
        try {
          await self.#dispatchAddOrChange(
            'change',
            path,
          );
        }
        catch (error) {
          self.#logger
            .error(`change dispatch failed: ${describeError(error,)}`,);
        }
      })();
    }

    /**
     chokidar `unlink` listener.
     
     @param path - absolute path emitted by chokidar
     */
    function onUnlink(path: string,): void {
      void (async function dispatchUnlink(): Promise<void> {
        try {
          await self.#dispatchUnlink(path,);
        }
        catch (error) {
          self.#logger
            .error(`unlink dispatch failed: ${describeError(error,)}`,);
        }
      })();
    }

    /**
     chokidar `addDir` listener.
     Directories have no content to hash, so pre-`ready` adds are
     silently absorbed (no cache write, no emit); post-`ready` adds
     normalise and emit. The {@link typeFilter} (default `['file']`)
     downstream decides whether the emitted event actually drives a
     restart.
     
     @param path - absolute directory path emitted by chokidar
     */
    function onAddDir(path: string,): void {
      void (async function dispatchAddDir(): Promise<void> {
        try {
          await self.#dispatchDirEvent(
            'addDir',
            path,
          );
        }
        catch (error) {
          self.#logger
            .error(`addDir dispatch failed: ${describeError(error,)}`,);
        }
      })();
    }

    /**
     chokidar `unlinkDir` listener.
     Directory removals never had a hash entry; post-`ready` removals
     emit, pre-`ready` removals are absorbed (rare; would only happen
     if the initial walk catches a directory mid-removal).
     
     @param path - absolute directory path emitted by chokidar
     */
    function onUnlinkDir(path: string,): void {
      void (async function dispatchUnlinkDir(): Promise<void> {
        try {
          await self.#dispatchDirEvent(
            'unlinkDir',
            path,
          );
        }
        catch (error) {
          self.#logger
            .error(
            `unlinkDir dispatch failed: ${describeError(error,)}`,
            );
        }
      })();
    }

    /**
     chokidar `error` listener.
     chokidar keeps the watcher alive after recoverable errors, so this
     does not call `stop()`.
     
     @param error - error value emitted by chokidar (typed `unknown` per chokidar's signature)
     */
    function onError(error: unknown,): void {
      self.#logger
        .error(`watcher error: ${describeError(error,)}`,);
    }

    this.#fsw
      .on(
      'ready',
      onReady,
      );
    this.#fsw
      .on(
      'add',
      onAdd,
      );
    this.#fsw
      .on(
      'change',
      onChange,
      );
    this.#fsw
      .on(
      'unlink',
      onUnlink,
      );
    this.#fsw
      .on(
      'addDir',
      onAddDir,
      );
    this.#fsw
      .on(
      'unlinkDir',
      onUnlinkDir,
      );
    this.#fsw
      .on(
      'error',
      onError,
      );
  }

  /**
   Awaits chokidar's `ready` event then drains in-flight pre-populate hashes.
   Returns immediately on subsequent calls (idempotent).
   Resolves once the cache reflects every file present at start time.
   
   @example
   ```ts
   await watcher.untilReady();
   // hashCache now contains an entry for every file under the watch roots
   ```
   */
  async untilReady(): Promise<void> {
    if (!this.#ready) {
      await once(
        this.#fsw,
        'ready',
      );
    }
    while (this.#prePopulate
      .size
      > 0) {
      /* oxlint-disable-next-line eslint/no-await-in-loop -- intentional drain: each iteration awaits ALL pending pre-populates before re-checking, so the loop body cannot run concurrently */
      await Promise.allSettled(this.#prePopulate,);
    }
  }

  /**
   Closes the chokidar watcher. Idempotent: chokidar tracks a `closed` flag
   so calling twice does not error.
   Resolves once chokidar has released its OS watches.
   
   @example
   ```ts
   await watcher.stop();
   ```
   */
  async stop(): Promise<void> {
    await this.#fsw
      .close();
  }

  /**
   Routes an add/change event: pre-`ready` paths feed the cache (no emit);
   post-`ready` paths normalise and emit a {@link WatchEvent}.
   
   @param kind - `add` for new files, `change` for modifications
   
   @param path - absolute path emitted by chokidar
   */
  async #dispatchAddOrChange(
    kind: 'add' | 'change',
    path: string,
  ): Promise<void> {
    if (!this.#ready) {
      this.#trackPrePopulate(path,);
      return;
    }
    await this.#emitEvent(
      kind,
      path,
    );
  }

  /**
   Routes an unlink event. Cache entry is dropped unconditionally so a
   later re-create starts from a clean slate; the event is only forwarded
   post-`ready`.
   
   @param path - absolute path emitted by chokidar
   */
  async #dispatchUnlink(path: string,): Promise<void> {
    this.#hashCache
      .delete(path,);
    if (!this.#ready)
      return;
    await this.#emitEvent(
      'unlink',
      path,
    );
  }

  /**
   Routes a directory `addDir`/`unlinkDir` event. Directories have no
   content to hash, so pre-`ready` events are silently absorbed; post-
   `ready` events normalise and emit. The downstream filter chain
   (`typeFilter`, default `['file']`) decides whether the emitted event
   drives a restart.
   
   @param kind - `'addDir'` for new directories, `'unlinkDir'` for removals
   
   @param path - absolute directory path emitted by chokidar
   */
  async #dispatchDirEvent(
    kind: 'addDir' | 'unlinkDir',
    path: string,
  ): Promise<void> {
    if (!this.#ready)
      return;
    await this.#emitEvent(
      kind,
      path,
    );
  }

  /**
   Schedules a pre-populate job and detaches its cleanup.
   
   @param path - absolute path to hash and store
   */
  #trackPrePopulate(path: string,): void {
    /**
     Hash-and-store job retained in `#prePopulate` so `untilReady()` can drain it.
     */
    const job = this.#runPrePopulate(path,);
    this.#prePopulate
      .add(job,);
    void this.#drainPrePopulateJob(job,);
  }

  /**
   Waits for a single pre-populate job to settle and removes it from the
   tracking set. Uses `allSettled` so a rejected job still cleans up.
   
   @param job - the job promise to drain
   
   @mutates job through `Promise.allSettled` promise assimilation
   */
  async #drainPrePopulateJob(job: Promise<void>,): Promise<void> {
    await Promise.allSettled([job,],);
    this.#prePopulate
      .delete(job,);
  }

  /**
   Reads a file's bytes, hashes them, stores the digest. Logs and
   swallows errors so a single transient I/O failure does not propagate
   up through chokidar's event dispatch.
   
   @param path - absolute path of the file
   */
  async #runPrePopulate(path: string,): Promise<void> {
    try {
      /**
       Digest computed off the disk read; the OVERSIZED sentinel signals a too-large file that should not be cached.
       */
      const hash = await this.#hashCache
        .hashFile(path,);
      if (hash !== OVERSIZED) {
        this.#hashCache
          .set({
            path,
            hash,
          },);
      }
    }
    catch (error) {
      this.#logger
        .warn(
        `pre-populate hash failed for ${path}: ${describeError(error,)}`,
        );
    }
  }

  /**
   Normalises a chokidar event into a {@link WatchEvent} and invokes the
   orchestrator's callback.
   
   `entity` is derived from `kind`: file kinds (`add`/`change`/`unlink`)
   map to `'file'`; dir kinds (`addDir`/`unlinkDir`) map to `'dir'`.
   Keeping the derivation here means downstream filters never have to
   recompute it from kind strings.
   
   @param kind - event kind
   
   @param path - absolute path emitted by chokidar
   */
  async #emitEvent(
    kind: WatchEventKind,
    path: string,
  ): Promise<void> {
    /**
     Entity derived from kind once; filters reuse rather than re-derive.
     */
    const entity: WatchEntityType = ((kind === 'addDir') || (kind === 'unlinkDir'))
      ? 'dir'
      : 'file';
    /**
     Normalised event handed to the orchestrator's `onEvent` callback.
     */
    const event: WatchEvent = {
      kind,
      entity,
      path,
      relativePath: relativePathForRoots({
        resolvedRoots: this.#resolvedRoots,
        absPath: path,
      },),
      ext: extname(path,),
    };
    await this.#onEvent(event,);
  }
}
/* oxlint-enable no-restricted-syntax/no-class */
