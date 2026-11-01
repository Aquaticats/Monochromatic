import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';
import type {
  ExitListener,
  ExitResult,
  SpawnedChildHandle,
  SpawnFn,
} from './child.ts';
import {
  DEFAULT_DEBOUNCE_MS,
  startWatchRestart,
  type WatchRestartHandle,
} from './start.ts';

/**
 Synthetic post-event wait: chokidar's default `awaitWriteFinish.stabilityThreshold`
 is 50 ms, our debounce is `DEFAULT_DEBOUNCE_MS` (100 ms), and a safety
 margin absorbs setImmediate / setTimeout scheduling jitter on slow CI.
 Used by every "wait for the dust to settle" assertion.
 */
const POST_EVENT_WAIT_MS: number = 50 + DEFAULT_DEBOUNCE_MS + 150;

/**
 Buffer used by "did NOT restart" assertions; same shape as the
 positive wait so a slow restart cannot hide as a fast skip.
 */
const NO_EVENT_WAIT_MS: number = POST_EVENT_WAIT_MS;

/* oxlint-disable no-restricted-syntax/no-class -- test double implementing the SpawnedChildHandle contract and instantiated via `new` by the recording spawn factory; it carries mutable per-instance state (exit listeners, exited guard) that a frozen-object factory cannot model for the orchestrator's lifecycle assertions. */
/**
 In-memory stand-in for `node:child_process.ChildProcess`.
 
 Implements only the slice {@link SpawnedChildHandle} declares: `once` /
 `off` for `exit`, `kill` to flip `killed` and schedule a deferred
 synthetic exit, and the readable fields the orchestrator inspects.
 Auto-exit on SIGTERM is wired through {@link kill} so the orchestrator's
 `child.stop()` resolves on the next event-loop turn without burning
 the 5-second SIGTERM-grace window every test.
 
 @example
 ```ts
 const handle = new FakeChild();
 handle.kill('SIGTERM',); // schedules a synchronous-ish exit
 ```
 */
class FakeChild implements SpawnedChildHandle {
  /** Mutable so the orchestrator's lifecycle logging reads consistent. */
  pid: number = 1_000;
  /** Set when {@link simulateExit} fires; mirrors `ChildProcess.exitCode`. Type sourced from the handle so the nullish union lives only at its definition. */
  exitCode: SpawnedChildHandle['exitCode'] = null;
  /** Flips on {@link kill}; mirrors `ChildProcess.killed`. */
  killed: boolean = false;

  /** Listeners waiting on the `exit` event. */
  readonly #exitListeners: ExitListener[] = [];
  /** Guard against double-firing the synthetic exit. */
  #exited: boolean = false;

  /**
   Registers a one-shot exit listener.
   
   @param event - must be `'exit'`; other events are ignored
   
   @param listener - exit callback (code, signal)
   */
  once(event: 'exit', listener: ExitListener,): void {
    if (event !== 'exit')
      return;
    this.#exitListeners.push(listener,);
  }

  /**
   Removes a previously-registered exit listener.
   
   @param event - must be `'exit'`; other events are ignored
   
   @param listener - the same reference passed to {@link once}
   */
  off(event: 'exit', listener: ExitListener,): void {
    if (event !== 'exit')
      return;
    const idx = this.#exitListeners.indexOf(listener,);
    if (idx !== (-1))
      this.#exitListeners.splice(idx, 1,);
  }

  /**
   Flips `killed` and schedules a synthetic exit on the next macrotask
   (via `setImmediate`) so the orchestrator's pre-registered exit
   listener observes the resolution. Mirrors real OS behavior where
   `kill('SIGTERM')` produces an asynchronous exit, not a synchronous
   one.
   
   @param signal - signal name (or numeric); defaults to `SIGTERM`
   
   @returns `true`, matching `ChildProcess.kill`'s "signal sent" return
   */
  kill(signal?: NodeJS.Signals | number,): boolean {
    this.killed = true;
    const resolvedSignal: NodeJS.Signals = ((typeof signal) === 'string')
      ? signal
      : 'SIGTERM';
    /** Captured `this` so the setImmediate closure can reach private state. */
    const self = this;
    setImmediate(function emitSyntheticExit() {
      self.#simulateExit(null, resolvedSignal,);
    },);
    return true;
  }

  /**
   Synchronously fires the synthetic exit and clears listeners.
   Test entry point for "exit by some non-signal cause" scenarios;
   tests in this file rely only on the SIGTERM path through {@link kill}.
   
   @param code - exit code (or `null` when killed by signal)
   
   @param signal - signal name (or `null` when exited normally)
   */
  #simulateExit(
    code: ExitResult['code'],
    signal: ExitResult['signal'],
  ): void {
    if (this.#exited)
      return;
    this.#exited = true;
    this.exitCode = code;
    /** Snapshot the listeners before clearing so a listener that re-registers does not loop. */
    const listeners = this.#exitListeners.splice(0,);
    for (const listener of listeners)
      listener(code, signal,);
  }
}
/* oxlint-enable no-restricted-syntax/no-class */

/**
 Spawn record produced by {@link makeRecordingSpawn}; one entry per
 spawn the orchestrator requests.
 */
type SpawnRecord = {
  readonly command: string;
  readonly args: readonly string[];
  readonly handle: FakeChild;
};

/**
 Builds a {@link SpawnFn} that records every call into a shared array.
 
 @returns spawn factory and the live records array
 
 @example
 ```ts
 const { spawn, records, } = makeRecordingSpawn();
 ```
 */
function makeRecordingSpawn(): {
  spawn: SpawnFn;
  records: SpawnRecord[];
} {
  const records: SpawnRecord[] = [];
  function recordingSpawn(
    args: {
      readonly command: string;
      readonly args: readonly string[];
    },
  ): SpawnedChildHandle {
    const handle = new FakeChild();
    records.push({
      command: args.command,
      args: args.args,
      handle,
    },);
    return handle;
  }
  return {
    spawn: recordingSpawn,
    records,
  };
}

/**
 Returns a fresh temp directory dedicated to one test run.
 
 @returns absolute path of a freshly-created temp directory
 
 @example
 ```ts
 const dir = await makeTmpDir();
 ```
 */
async function makeTmpDir(): Promise<string> {
  return mkdtemp(join(scratchRoot(), 'watch-restart-start-',),);
}

/**
 Stops the handle ignoring errors. Used in test teardown where the
 orchestrator's `stop()` failing is its own bug, not the failure under
 test.
 
 @param handle - handle to stop
 */
async function safeStop(handle: WatchRestartHandle,): Promise<void> {
  try {
    await handle.stop();
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    // Test teardown; suppress.
  }
}

await describe({
  name: startWatchRestart.name,
  children: [
    it({
      name: 'initial: true (default) spawns the child immediately after ready',
      fn: async function defaultsInitialTrue() {
        const dir = await makeTmpDir();
        const { spawn, records, } = makeRecordingSpawn();

        const handle = await startWatchRestart({
          paths: [dir,],
          command: 'noop',
          processGroup: false,
          spawn,
        },);

        expect(records.length,).toBe(1,);
        expect(nonNullishOrThrow(records[0],).command,).toBe('noop',);

        await safeStop(handle,);
        await rm(dir, { recursive: true, },);
      },
    },),
    it({
      name: 'initial: false defers spawn until the first qualifying event',
      fn: async function noInitial() {
        const dir = await makeTmpDir();
        const { spawn, records, } = makeRecordingSpawn();

        const handle = await startWatchRestart({
          paths: [dir,],
          command: 'noop',
          processGroup: false,
          initial: false,
          spawn,
        },);

        expect(records.length,).toBe(0,);

        await writeFile(join(dir, 'first.ts',), 'content',);
        await wait(POST_EVENT_WAIT_MS,);

        expect(records.length,).toBe(1,);

        await safeStop(handle,);
        await rm(dir, { recursive: true, },);
      },
    },),
    it({
      name: 'byte-identical write produces no restart',
      fn: async function byteIdenticalSkips() {
        const dir = await makeTmpDir();
        const file = join(dir, 'same.ts',);
        await writeFile(file, 'unchanged',);

        const { spawn, records, } = makeRecordingSpawn();
        const handle = await startWatchRestart({
          paths: [dir,],
          command: 'noop',
          processGroup: false,
          initial: false,
          spawn,
        },);

        await writeFile(file, 'unchanged',);
        await wait(NO_EVENT_WAIT_MS,);

        expect(records.length,).toBe(0,);

        await safeStop(handle,);
        await rm(dir, { recursive: true, },);
      },
    },),
    it({
      name: 'no-content-changed lets byte-identical writes through',
      fn: async function noContentChanged() {
        const dir = await makeTmpDir();
        const file = join(dir, 'same.ts',);
        await writeFile(file, 'unchanged',);

        const { spawn, records, } = makeRecordingSpawn();
        const handle = await startWatchRestart({
          paths: [dir,],
          command: 'noop',
          processGroup: false,
          initial: false,
          contentChanged: false,
          spawn,
        },);

        await writeFile(file, 'unchanged',);
        await wait(POST_EVENT_WAIT_MS,);

        expect(records.length,).toBe(1,);

        await safeStop(handle,);
        await rm(dir, { recursive: true, },);
      },
    },),
    it({
      name: 'two writes to different files within debounce coalesce to one restart',
      fn: async function debounceCoalesce() {
        const dir = await makeTmpDir();

        const { spawn, records, } = makeRecordingSpawn();
        const handle = await startWatchRestart({
          paths: [dir,],
          command: 'noop',
          processGroup: false,
          initial: false,
          spawn,
        },);

        // Two distinct files: chokidar emits two events; our debounce
        // must coalesce them to a single restart.
        await writeFile(join(dir, 'a.ts',), 'a',);
        await writeFile(join(dir, 'b.ts',), 'b',);
        await wait(POST_EVENT_WAIT_MS,);

        expect(records.length,).toBe(1,);

        await safeStop(handle,);
        await rm(dir, { recursive: true, },);
      },
    },),
    it({
      name: 'ext filter rejects non-matching extensions; admits matching',
      fn: async function extFilters() {
        const dir = await makeTmpDir();

        const { spawn, records, } = makeRecordingSpawn();
        const handle = await startWatchRestart({
          paths: [dir,],
          command: 'noop',
          processGroup: false,
          initial: false,
          extensions: ['.ts',],
          spawn,
        },);

        await writeFile(join(dir, 'styles.css',), 'body{}',);
        await wait(NO_EVENT_WAIT_MS,);
        expect(records.length,).toBe(0,);

        await writeFile(join(dir, 'index.ts',), 'export {};',);
        await wait(POST_EVENT_WAIT_MS,);
        expect(records.length,).toBe(1,);

        await safeStop(handle,);
        await rm(dir, { recursive: true, },);
      },
    },),
    it({
      name: 'exclude beats include for overlapping files',
      fn: async function excludeBeatsInclude() {
        const dir = await makeTmpDir();

        const { spawn, records, } = makeRecordingSpawn();
        const handle = await startWatchRestart({
          paths: [dir,],
          command: 'noop',
          processGroup: false,
          initial: false,
          exclude: ['*.test.ts',],
          spawn,
        },);

        await writeFile(join(dir, 'a.test.ts',), 'test',);
        await wait(NO_EVENT_WAIT_MS,);
        expect(records.length,).toBe(0,);

        await writeFile(join(dir, 'a.ts',), 'real',);
        await wait(POST_EVENT_WAIT_MS,);
        expect(records.length,).toBe(1,);

        await safeStop(handle,);
        await rm(dir, { recursive: true, },);
      },
    },),
    it({
      name: 'stop() tears down without leaking events',
      fn: async function stopTearsDown() {
        const dir = await makeTmpDir();

        const { spawn, records, } = makeRecordingSpawn();
        const handle = await startWatchRestart({
          paths: [dir,],
          command: 'noop',
          processGroup: false,
          initial: false,
          spawn,
        },);

        await handle.stop();

        await writeFile(join(dir, 'late.ts',), 'no',);
        await wait(NO_EVENT_WAIT_MS,);

        expect(records.length,).toBe(0,);

        await rm(dir, { recursive: true, },);
      },
    },),
    it({
      name: 'stop() is idempotent: a second call is a no-op',
      fn: async function stopIdempotent() {
        const dir = await makeTmpDir();
        const { spawn, } = makeRecordingSpawn();
        const handle = await startWatchRestart({
          paths: [dir,],
          command: 'noop',
          processGroup: false,
          initial: false,
          spawn,
        },);

        await handle.stop();
        // Second call must resolve without throwing.
        await handle.stop();

        await rm(dir, { recursive: true, },);
      },
    },),
  ],
},);
