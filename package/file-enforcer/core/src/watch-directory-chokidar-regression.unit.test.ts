import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';
import { setTimeout as wait, } from 'node:timers/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type EventKind,
  reset,
  resetWriteTimestamps,
  trackDest,
  trackRead,
  watchDirectory,
} from '../dist/final/node/index.mjs';

/**
 Maximum time allowed for a live chokidar event to reach watchDirectory's callback.
 */
const WATCH_EVENT_TIMEOUT_MS = 2_000;

/**
 Maximum time allowed for watchDirectory to close after abort.
 */
const WATCH_DIRECTORY_STOP_TIMEOUT_MS = 2_000;

/**
 Sentinel returned when a live watch event does not arrive in time.
 */
const WATCH_EVENT_TIMEOUT = Symbol('watchDirectory event timeout');

/**
 Sentinel returned when chokidar reports initial scan readiness.
 */
const WATCH_DIRECTORY_READY = Symbol('watchDirectory ready');

/**
 Sentinel returned when watchDirectory closes normally after abort.
 */
const WATCH_DIRECTORY_CLOSED = Symbol('watchDirectory closed');

/**
 Sentinel returned when watchDirectory does not close after abort in time.
 */
const WATCH_DIRECTORY_STOP_TIMEOUT = Symbol('watchDirectory stop timeout');

/**
 Event observed through watchDirectory's public callback.
 */
type WatchDirectoryObservedEvent = Readonly<{
  /**
   Classification produced by file-enforcer's watch filter.
   */
  kind: EventKind;

  /**
   Filename relative to watched directory.
   */
  filename: string;
}>;

/**
 WatchDirectory completion failure captured without rejecting in the background.
 */
type WatchDirectoryFailure = Readonly<{
  /**
   Error that made watchDirectory reject.
   */
  error: unknown;
}>;

/**
 Result emitted by the background watchDirectory runner.
 */
type WatchDirectoryFinish = typeof WATCH_DIRECTORY_CLOSED | WatchDirectoryFailure;

/**
 Result candidates while waiting for chokidar readiness.
 */
type WatchDirectoryReadyResult =
  | WatchDirectoryFailure
    | typeof WATCH_DIRECTORY_READY
    | typeof WATCH_EVENT_TIMEOUT
    | typeof WATCH_DIRECTORY_CLOSED;

/**
 Result candidates while waiting for one live watch event.
 */
type WatchDirectoryRaceResult =
  | WatchDirectoryObservedEvent
    | WatchDirectoryFailure
    | typeof WATCH_EVENT_TIMEOUT
    | typeof WATCH_DIRECTORY_CLOSED;

/**
 Result candidates while waiting for watchDirectory teardown.
 */
type WatchDirectoryStopResult =
  | WatchDirectoryFailure
    | typeof WATCH_DIRECTORY_CLOSED
    | typeof WATCH_DIRECTORY_STOP_TIMEOUT;

/**
 Mutable controls and promises for one watchDirectory capture.
 */
type WatchDirectoryCapture = Readonly<{
  /**
   Controller used to stop watchDirectory.
   */
  controller: AbortController;

  /**
   First event observed by watchDirectory.
   */
  eventReceived: Promise<WatchDirectoryObservedEvent>;

  /**
   Signal that chokidar's initial scan completed.
   */
  readyReceived: Promise<typeof WATCH_DIRECTORY_READY>;

  /**
   Completion state of the background watchDirectory call.
   */
  watcherFinished: Promise<WatchDirectoryFinish>;
}>;

/**
 Filesystem mutation performed after watchDirectory starts.
 */
type WatchDirectoryTrigger = () => Promise<void>;

/**
 Creates isolated temp directory for chokidar watch regression tests.
 
 @returns Temp directory path.
 
 @example
 ```ts
 const tempDir = await setup();
 ```
 */
async function setup(): Promise<string> {
  return await mkdtemp(join(
    scratchRoot(),
    'file-enforcer-watch-directory-chokidar-',
  ),);
}

/**
 Removes isolated temp directory.
 
 @param tempDir - Directory returned by {@link setup}.
 
 @example
 ```ts
 await teardown(tempDir);
 ```
 */
async function teardown(tempDir: string,): Promise<void> {
  await rm(
    tempDir,
    {
      recursive: true,
      force: true,
    },
  );
}

/**
 Returns whether a race result carries a watchDirectory failure.
 
 @param result - Result returned while waiting for an event or teardown.
 
 @returns Whether result is the failure wrapper.
 
 @example
 ```ts
 const failed = watchDirectoryResultIsFailure(result);
 ```
 */
function watchDirectoryResultIsFailure(
  result: WatchDirectoryRaceResult | WatchDirectoryReadyResult | WatchDirectoryStopResult,
): result is WatchDirectoryFailure {
  if ((typeof result) !== 'object')
    return false;
  if (result === null)
    return false;

  return 'error' in result;
}

/**
 Starts watchDirectory and captures its first callback event.
 
 @param dir - Directory passed to watchDirectory.
 
 @param configPath - Config path passed to watchDirectory.
 
 @returns Capture controls and promises.
 
 @example
 ```ts
 const capture = startWatchDirectoryCapture({ dir: '/tmp/repo', configPath: '/tmp/repo/config.ts' });
 ```
 */
function startWatchDirectoryCapture(
  {
    dir,
    configPath,
  }: {
    readonly configPath: string;
    readonly dir: string;
  },
): WatchDirectoryCapture {
  /**
   Abort controller for stopping the watcher after the assertion event arrives.
   */
  const controller = new AbortController();
  /**
   First event observed by watchDirectory's public callback.
   */
  const eventReceived = Promise.withResolvers<WatchDirectoryObservedEvent>();
  /**
   Chokidar initial-scan readiness signal.
   */
  const readyReceived = Promise.withResolvers<typeof WATCH_DIRECTORY_READY>();
  /**
   Background watchDirectory completion state.
   */
  const watcherFinished = Promise.withResolvers<WatchDirectoryFinish>();

  void (async function runWatchDirectoryCapture(): Promise<void> {
    try {
      await watchDirectory({
        dir,
        signal: controller.signal,
        configPath,
        onReady: function captureReady(): void {
          readyReceived.resolve(WATCH_DIRECTORY_READY,);
        },
        onEvent: function captureEvent(
          kind: EventKind,
          filename: string,
        ): void {
          eventReceived.resolve({
            kind,
            filename,
          },);
        },
      },);
      watcherFinished.resolve(WATCH_DIRECTORY_CLOSED,);
    }
    catch (watchError: unknown) {
      watcherFinished.resolve({ error: watchError, },);
    }
  })();

  return {
    controller,
    eventReceived: eventReceived.promise,
    readyReceived: readyReceived.promise,
    watcherFinished: watcherFinished.promise,
  };
}

/**
 Stops watchDirectory and asserts abort teardown finished cleanly.
 
 @param controller - Controller used to abort watchDirectory.
 
 @param watcherFinished - Background watchDirectory completion state.
 
 @throws When watchDirectory fails or does not stop after abort.
 
 @example
 ```ts
 await stopWatchDirectoryCapture({ controller, watcherFinished });
 ```
 */
async function stopWatchDirectoryCapture(
  {
    controller,
    watcherFinished,
  }: {
    readonly controller: AbortController;
    readonly watcherFinished: Promise<WatchDirectoryFinish>;
  },
): Promise<void> {
  controller.abort();
  /**
   Result of waiting for watchDirectory to finish after abort.
   */
  const stopResult: WatchDirectoryStopResult = await Promise.race([
    watcherFinished,
    wait(
      WATCH_DIRECTORY_STOP_TIMEOUT_MS,
      WATCH_DIRECTORY_STOP_TIMEOUT,
    ),
  ],);

  if (stopResult === WATCH_DIRECTORY_STOP_TIMEOUT)
    throw new Error('watchDirectory did not stop after abort',);
  if (watchDirectoryResultIsFailure(stopResult,))
    throw stopResult.error;
}

/**
 Captures one watchDirectory event after triggering one filesystem mutation.
 
 @param dir - Directory passed to watchDirectory.
 
 @param configPath - Config path passed to watchDirectory.
 
 @param trigger - Mutation expected to produce a watch event.
 
 @returns First observed watchDirectory event.
 
 @throws When no event arrives, watchDirectory fails, or abort teardown fails.
 
 @example
 ```ts
 const event = await captureWatchDirectoryEvent({ dir, configPath, trigger });
 ```
 */
async function captureWatchDirectoryEvent(
  {
    dir,
    configPath,
    trigger,
  }: {
    readonly configPath: string;
    readonly dir: string;
    readonly trigger: WatchDirectoryTrigger;
  },
): Promise<WatchDirectoryObservedEvent> {
  /**
   Background watchDirectory capture for this assertion.
   */
  const capture = startWatchDirectoryCapture({
    dir,
    configPath,
  },);
  /**
   Readiness, timeout, or early watcher completion before mutation.
   */
  const readyResult: WatchDirectoryReadyResult = await Promise.race([
    capture.readyReceived,
    wait(
      WATCH_EVENT_TIMEOUT_MS,
      WATCH_EVENT_TIMEOUT,
    ),
    capture.watcherFinished,
  ],);
  if (readyResult === WATCH_EVENT_TIMEOUT) {
    await stopWatchDirectoryCapture(capture,);
    throw new Error('watchDirectory did not become ready before timeout',);
  }
  if (readyResult === WATCH_DIRECTORY_CLOSED) {
    await stopWatchDirectoryCapture(capture,);
    throw new Error('watchDirectory closed before becoming ready',);
  }
  if (watchDirectoryResultIsFailure(readyResult,)) {
    await stopWatchDirectoryCapture(capture,);
    throw readyResult.error;
  }

  await trigger();
  /**
   First event, timeout, or early watcher completion.
   */
  const eventResult: WatchDirectoryRaceResult = await Promise.race([
    capture.eventReceived,
    wait(
      WATCH_EVENT_TIMEOUT_MS,
      WATCH_EVENT_TIMEOUT,
    ),
    capture.watcherFinished,
  ],);
  await stopWatchDirectoryCapture(capture,);

  if (eventResult === WATCH_EVENT_TIMEOUT)
    throw new Error('watchDirectory did not emit an event before timeout',);
  if (eventResult === WATCH_DIRECTORY_CLOSED)
    throw new Error('watchDirectory closed before emitting an event',);
  if (watchDirectoryResultIsFailure(eventResult,))
    throw eventResult.error;

  return eventResult;
}

await describe({
  name: 'file-enforcer chokidar watchDirectory regressions',
  concurrency: 1,
  children: [
    it({
      name: 'emits source events for live tracked read changes',
      fn: async function emitsSourceEventsForTrackedReadChanges(): Promise<void> {
        const tempDir = await setup();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return teardown(tempDir,);
          },
        };
        reset();
        resetWriteTimestamps();
        const sourcePath = join(
          tempDir,
          'source.txt',
        );
        const configPath = join(
          tempDir,
          'file-enforcer.config.ts',
        );
        await writeFile(
          configPath,
          '',
        );
        await writeFile(
          sourcePath,
          'initial',
        );
        trackRead(sourcePath,);

        const event = await captureWatchDirectoryEvent({
          dir: tempDir,
          configPath,
          trigger: async function writeTrackedSourceChange(): Promise<void> {
            await writeFile(
              sourcePath,
              'changed',
            );
          },
        },);

        expect(event,).toEqual({
          kind: 'source',
          filename: 'source.txt',
        },);
      },
    },),

    it({
      name: 'emits protected events for live managed destination edits',
      fn: async function emitsProtectedEventsForManagedDestinationEdits(): Promise<void> {
        const tempDir = await setup();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return teardown(tempDir,);
          },
        };
        reset();
        resetWriteTimestamps();
        const destinationPath = join(
          tempDir,
          'dest.txt',
        );
        const configPath = join(
          tempDir,
          'file-enforcer.config.ts',
        );
        await writeFile(
          configPath,
          '',
        );
        await writeFile(
          destinationPath,
          'managed',
        );
        trackDest(destinationPath,);

        const event = await captureWatchDirectoryEvent({
          dir: tempDir,
          configPath,
          trigger: async function writeManagedDestinationChange(): Promise<void> {
            await writeFile(
              destinationPath,
              'tampered',
            );
          },
        },);

        expect(event,).toEqual({
          kind: 'protected',
          filename: 'dest.txt',
        },);
      },
    },),
  ],
},);
