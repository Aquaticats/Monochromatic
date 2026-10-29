import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  Child,
  DEFAULT_STOP_TIMEOUT_MS,
  type ExitListener,
  type ExitResult,
  NO_CHILD,
  type ProcessSignalFn,
  type SpawnedChildHandle,
  type SpawnFn,
  type WriteClearFn,
} from './child.ts';

/* oxlint-disable no-restricted-syntax/no-class -- test double implementing the SpawnedChildHandle contract and instantiated via `new` by the recording spawn factory; it carries mutable per-instance state (signalsReceived, exit listeners) that a frozen-object factory cannot model for the state-machine assertions. */
/**
 Fake stand-in for `node:child_process.ChildProcess`.
 
 Records every signal it received so tests can assert SIGTERM-vs-SIGKILL
 sequencing without spawning a real OS process. The fake is honest about
 exit timing: it does NOT exit automatically on `kill()`. Tests drive
 `simulateExit()` to flip the state when they want the child to "die",
 which lets each test pick its own race timeline.
 */
class FakeChild implements SpawnedChildHandle {
  /** Mirrors `ChildProcess.pid`; always a number here (assignable to the handle's `number | undefined`). */
  readonly pid: number;
  /** Mirrors `ChildProcess.exitCode`; `null` until {@link simulateExit}. Type sourced from the handle so the nullish union lives only at its definition. */
  exitCode: SpawnedChildHandle['exitCode'] = null;
  /** Mirrors `ChildProcess.killed`; flips on any {@link kill} call. */
  killed: boolean = false;
  /** Test assertion surface: which signals arrived in what order. */
  readonly signalsReceived: NodeJS.Signals[] = [];
  /** Listeners registered with `once('exit', ...)`; cleared after firing. */
  #exitListeners: ExitListener[] = [];
  /** When true, {@link kill}`('SIGTERM')` triggers an exit synchronously. */
  readonly autoExitOnSigterm: boolean;

  /**
   Constructs a fake child.
   
   @param options - construction options
   
   @example
   ```ts
   const fake = new FakeChild({ pid: 100, autoExitOnSigterm: true, },);
   ```
   */
  constructor(
    {
      pid = 1_234,
      autoExitOnSigterm = false,
    }: {
      readonly pid?: number;
      readonly autoExitOnSigterm?: boolean;
    } = {},
  ) {
    this.pid = pid;
    this.autoExitOnSigterm = autoExitOnSigterm;
  }

  /**
   Records the signal and flips {@link killed}. Optionally auto-exits on SIGTERM.
   
   @param signal - signal name; defaults to `'SIGTERM'` like node
   
   @returns `true` always (matches `ChildProcess.kill`'s sentinel for "delivered")
   */
  kill(signal: NodeJS.Signals | number = 'SIGTERM',): boolean {
    if ((typeof signal) === 'string')
      this.signalsReceived.push(signal,);
    this.killed = true;
    if (this.autoExitOnSigterm && (signal === 'SIGTERM')) {
      this.simulateExit({
        code: null,
        signal: 'SIGTERM',
      },);
    }
    if (signal === 'SIGKILL') {
      this.simulateExit({
        code: null,
        signal: 'SIGKILL',
      },);
    }
    return true;
  }

  /**
   Registers a one-shot exit listener.
   
   @param event - event name; only `'exit'` is supported
   
   @param listener - callback to fire on exit
   */
  once(
    event: 'exit',
    listener: ExitListener,
  ): void {
    if (event === 'exit')
      this.#exitListeners.push(listener,);
  }

  /**
   Removes a previously registered exit listener; no-op if not registered.
   
   @param event - event name; only `'exit'` is supported
   
   @param listener - callback previously passed to {@link once}
   */
  off(
    event: 'exit',
    listener: ExitListener,
  ): void {
    if (event === 'exit') {
      this.#exitListeners = this.#exitListeners.filter(
        function isOther(other,) {
          return other !== listener;
        },
      );
    }
  }

  /**
   Test helper: pretends the OS reported an exit. Fires every registered
   listener once, clears the list, sets {@link exitCode}.
   
   @param result - exit code and signal to pass to listeners
   
   @example
   ```ts
   fake.simulateExit({ code: 0, signal: null, },);
   ```
   */
  simulateExit(
    {
      code = 0,
      signal = null,
    }: {
      readonly code?: ExitResult['code'];
      readonly signal?: ExitResult['signal'];
    } = {},
  ): void {
    this.exitCode = code;
    const listeners = this.#exitListeners;
    this.#exitListeners = [];
    for (const listener of listeners) {
      listener(
        code,
        signal,
      );
    }
  }
}
/* oxlint-enable no-restricted-syntax/no-class */

/**
 Records spawn calls so tests can assert command/args wiring and inspect
 each returned fake handle.
 */
type SpawnRecord = {
  readonly command: string;
  readonly args: readonly string[];
  readonly handle: FakeChild;
};

/**
 Builds a {@link SpawnFn} that returns fresh {@link FakeChild} instances and
 records every call. Tests use the records to assert order of spawns and
 to drive each handle's exit timing.
 
 @param options - per-test fake-child configuration
 
 @returns spawn factory and the captured records
 
 @example
 ```ts
 const { spawn, records, } = makeRecordingSpawn({ autoExitOnSigterm: true, },);
 const child = new Child({ command: 'node', spawn, },);
 ```
 */
function makeRecordingSpawn(
  options: {
    readonly autoExitOnSigterm?: boolean;
  } = {},
): {
  readonly spawn: SpawnFn;
  readonly records: SpawnRecord[];
} {
  const records: SpawnRecord[] = [];
  /**
   Spawn factory captured by the closure: pushes a record per call and
   returns a fresh fake. Declared as a function so the lint rule banning
   variable-assigned function expressions is satisfied; the closure picks
   up `records` and `options` from the enclosing factory.
   
   @param spawnArgs - command and argument list
   
   @returns fake child handle
   */
  function recordingSpawn(
    spawnArgs: {
      readonly command: string;
      readonly args: readonly string[];
    },
  ): FakeChild {
    const handle = new FakeChild({
      pid: 1_000 + records.length,
      autoExitOnSigterm: options.autoExitOnSigterm ?? false,
    },);
    records.push({
      command: spawnArgs.command,
      args: spawnArgs.args,
      handle,
    },);
    return handle;
  }
  return {
    spawn: recordingSpawn,
    records,
  };
}

await describe({
  name: Child.name,
  children: [
    describe({
      name: 'state machine',
      children: [
        it({
          name: 'initial state is idle, current is undefined',
          fn: async () => {
            const { spawn, } = makeRecordingSpawn();
            const child = new Child({
              command: 'node',
              processGroup: false,
              spawn,
            },);

            expect(child.state,).toBe('idle',);
            expect(child.current,).toBe(NO_CHILD,);
          },
        },),
        it({
          name: 'start() transitions idle -> running',
          fn: async () => {
            const { spawn, records, } = makeRecordingSpawn();
            const child = new Child({
              command: 'node',
              processGroup: false,
              args: ['a', 'b',],
              spawn,
            },);

            await child.start();

            expect(child.state,).toBe('running',);
            expect(records,).toHaveLength(1,);
            expect(records[0]?.command,).toBe('node',);
            expect(records[0]?.args,).toEqual(['a', 'b',],);
            expect(child.current,).toBe(records[0]?.handle,);
          },
        },),
        it({
          name: 'start() while running is a no-op (records only one spawn)',
          fn: async () => {
            const { spawn, records, } = makeRecordingSpawn();
            const child = new Child({
              command: 'node',
              processGroup: false,
              spawn,
            },);

            await child.start();
            await child.start();

            expect(records,).toHaveLength(1,);
            expect(child.state,).toBe('running',);
          },
        },),
        it({
          name: 'stop() while idle is a no-op',
          fn: async () => {
            const { spawn, records, } = makeRecordingSpawn();
            const child = new Child({
              command: 'node',
              processGroup: false,
              spawn,
            },);

            await child.stop();

            expect(records,).toHaveLength(0,);
            expect(child.state,).toBe('idle',);
          },
        },),
        it({
          name: 'natural exit transitions running -> idle without orchestrator action',
          fn: async () => {
            const { spawn, records, } = makeRecordingSpawn();
            const child = new Child({
              command: 'node',
              processGroup: false,
              spawn,
            },);

            await child.start();
            expect(child.state,).toBe('running',);

            const { handle, } = nonNullishOrThrow(records[0],);
            handle.simulateExit({
              code: 0,
              signal: null,
            },);

            expect(child.state,).toBe('idle',);
            expect(child.current,).toBe(NO_CHILD,);
          },
        },),
      ],
    },),
    describe({
      name: 'stop semantics',
      children: [
        it({
          name: 'stop() sends SIGTERM and returns once the child exits',
          fn: async () => {
            const { spawn, records, } = makeRecordingSpawn({
              autoExitOnSigterm: true,
            },);
            const child = new Child({
              command: 'node',
              processGroup: false,
              stopTimeout: 200,
              spawn,
            },);

            await child.start();
            await child.stop();

            const { handle, } = nonNullishOrThrow(records[0],);
            expect(handle.signalsReceived,).toEqual(['SIGTERM',],);
            expect(child.state,).toBe('idle',);
            expect(child.current,).toBe(NO_CHILD,);
          },
        },),
        it({
          name: 'stop() does NOT send SIGKILL when SIGTERM is honored',
          fn: async () => {
            const { spawn, records, } = makeRecordingSpawn({
              autoExitOnSigterm: true,
            },);
            const child = new Child({
              command: 'node',
              processGroup: false,
              stopTimeout: 1_000,
              spawn,
            },);

            await child.start();
            await child.stop();

            const { handle, } = nonNullishOrThrow(records[0],);
            expect(handle.signalsReceived,).toEqual(['SIGTERM',],);
            expect(handle.signalsReceived.includes('SIGKILL',),).toBe(false,);
          },
        },),
        it({
          name: 'stop() escalates to SIGKILL after stopTimeout when SIGTERM is ignored',
          fn: async () => {
            const stopTimeoutMs = 50;
            const { spawn, records, } = makeRecordingSpawn({
              autoExitOnSigterm: false,
            },);
            const child = new Child({
              command: 'node',
              processGroup: false,
              stopTimeout: stopTimeoutMs,
              spawn,
            },);

            await child.start();
            const stopStart = performance.now();
            await child.stop();
            const elapsed = performance.now() - stopStart;

            const { handle, } = nonNullishOrThrow(records[0],);
            expect(handle.signalsReceived,).toEqual(['SIGTERM', 'SIGKILL',],);
            // Loose floor: the SIGKILL must come at least one timer tick after SIGTERM,
            // not on the same microtask. 1ms is enough to verify the timer fired.
            expect(elapsed,).toBeGreaterThanOrEqual(1,);
            expect(child.state,).toBe('idle',);
          },
        },),
      ],
    },),
    describe({
      name: 'restart semantics',
      children: [
        it({
          name: 'restart() from idle just spawns (no stop signals)',
          fn: async () => {
            const { spawn, records, } = makeRecordingSpawn();
            const child = new Child({
              command: 'node',
              processGroup: false,
              spawn,
            },);

            await child.restart();

            expect(records,).toHaveLength(1,);
            expect(records[0]?.handle.signalsReceived,).toEqual([],);
            expect(child.state,).toBe('running',);
          },
        },),
        it({
          name: 'restart() from running stops the prior, then spawns a fresh handle',
          fn: async () => {
            const { spawn, records, } = makeRecordingSpawn({
              autoExitOnSigterm: true,
            },);
            const child = new Child({
              command: 'node',
              processGroup: false,
              stopTimeout: 200,
              spawn,
            },);

            await child.start();
            const { handle: priorHandle, } = nonNullishOrThrow(records[0],);
            await child.restart();

            expect(records,).toHaveLength(2,);
            const { handle: newHandle, } = nonNullishOrThrow(records[1],);
            expect(priorHandle.signalsReceived,).toEqual(['SIGTERM',],);
            expect(newHandle.signalsReceived,).toEqual([],);
            expect(newHandle,).not.toBe(priorHandle,);
            expect(child.current,).toBe(newHandle,);
            expect(child.state,).toBe('running',);
          },
        },),
        it({
          name: 'restart() spawns the new child only AFTER the prior has exited',
          fn: async () => {
            // Manually drive the timing: SIGTERM does NOT auto-exit the prior;
            // we await the SIGKILL escalation to confirm restart waits.
            const stopTimeoutMs = 30;
            const { spawn, records, } = makeRecordingSpawn({
              autoExitOnSigterm: false,
            },);
            const child = new Child({
              command: 'node',
              processGroup: false,
              stopTimeout: stopTimeoutMs,
              spawn,
            },);

            await child.start();
            expect(records,).toHaveLength(1,);
            await child.restart();
            const { handle: priorHandle, } = nonNullishOrThrow(records[0],);

            expect(records,).toHaveLength(2,);
            expect(priorHandle.signalsReceived,).toEqual(['SIGTERM', 'SIGKILL',],);
            // Signal-killed processes report `exitCode === null` (signal set instead),
            // so the proof that the prior actually exited before the new spawn is
            // the `killed` flag plus the SIGKILL receipt above.
            expect(priorHandle.killed,).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: 'defaults',
      children: [
        it({
          name: 'stopTimeout defaults to DEFAULT_STOP_TIMEOUT_MS (5_000)',
          fn: async () => {
            const expected = 5_000;
            expect(DEFAULT_STOP_TIMEOUT_MS,).toBe(expected,);
          },
        },),
        it({
          name: 'args default to an empty array when omitted',
          fn: async () => {
            const { spawn, records, } = makeRecordingSpawn();
            const child = new Child({
              command: 'node',
              processGroup: false,
              spawn,
            },);

            await child.start();

            expect(records[0]?.args,).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: 'Q6 options (killSignal, processGroup, clear)',
      children: [
        it({
          name: 'killSignal: SIGHUP sends SIGHUP first, then SIGKILL after timeout',
          fn: async () => {
            const stopTimeoutMs = 50;
            const { spawn, records, } = makeRecordingSpawn({
              autoExitOnSigterm: false,
            },);
            const child = new Child({
              command: 'node',
              processGroup: false,
              killSignal: 'SIGHUP',
              stopTimeout: stopTimeoutMs,
              spawn,
            },);

            await child.start();
            await child.stop();

            const { handle, } = nonNullishOrThrow(records[0],);
            // SIGHUP is the first signal (instead of SIGTERM); SIGKILL is
            // still the escalation because the FakeChild does not exit on
            // SIGHUP in this test fixture.
            expect(handle.signalsReceived,).toEqual(['SIGHUP', 'SIGKILL',],);
            expect(child.state,).toBe('idle',);
          },
        },),
        it({
          name:
            'processGroup: true routes signals through processSignal with negative pid',
          fn: async () => {
            const { spawn, records, } = makeRecordingSpawn();
            const pgSignals: {
              readonly pid: number;
              readonly signal: NodeJS.Signals | number;
            }[] = [];
            /**
             Recording processSignal sink: pushes every received `(pid, signal)`
             and drives the corresponding fake handle's synthetic exit so the
             Promise.race in `#stopRunning` resolves on the `exited` branch
             rather than hitting the SIGKILL timeout.
             
             @param args - pid and signal forwarded from `Child.#sendSignal`
             */
            function recordingProcessSignal(
              args: {
                readonly pid: number;
                readonly signal: NodeJS.Signals | number;
              },
            ): void {
              pgSignals.push(args,);
              /** Recover the FakeChild by absolute pid; negative inputs reach the same handle. */
              const rec = records.find(function matchByPid(r,) {
                return r.handle.pid === Math.abs(args.pid,);
              },);
              if (rec && ((typeof args.signal) === 'string')) {
                rec.handle.simulateExit({
                  code: null,
                  signal: args.signal,
                },);
              }
            }
            const processSignal: ProcessSignalFn = recordingProcessSignal;
            const child = new Child({
              command: 'node',
              processGroup: true,
              stopTimeout: 200,
              spawn,
              processSignal,
            },);

            await child.start();
            await child.stop();

            expect(pgSignals.length,).toBe(1,);
            expect(nonNullishOrThrow(pgSignals[0],).pid,).toBe(-1_000,);
            expect(nonNullishOrThrow(pgSignals[0],).signal,).toBe('SIGTERM',);
            // The direct-handle kill path was NOT taken; `signalsReceived`
            // on the fake stays empty under processGroup mode.
            const { handle, } = nonNullishOrThrow(records[0],);
            expect(handle.signalsReceived,).toEqual([],);
            expect(child.state,).toBe('idle',);
          },
        },),
        it({
          name:
            'clear: true runs writeClear before initial spawn and again before restart',
          fn: async () => {
            const calls: { count: number; } = { count: 0, };
            /**
             Counting writeClear sink so the test can assert call timing
             without polluting stdout. Declared as a function expression
             to match the {@link WriteClearFn} structural type.
             */
            function recordingWriteClear(): void {
              calls.count += 1;
            }
            const writeClear: WriteClearFn = recordingWriteClear;
            const { spawn, } = makeRecordingSpawn({
              autoExitOnSigterm: true,
            },);
            const child = new Child({
              command: 'node',
              processGroup: false,
              clear: true,
              stopTimeout: 200,
              spawn,
              writeClear,
            },);

            await child.start();
            expect(calls.count,).toBe(1,);

            await child.restart();
            expect(calls.count,).toBe(2,);

            await child.stop();
          },
        },),
        it({
          name: 'clear: false (default) never runs writeClear',
          fn: async () => {
            const calls: { count: number; } = { count: 0, };
            function recordingWriteClear(): void {
              calls.count += 1;
            }
            const writeClear: WriteClearFn = recordingWriteClear;
            const { spawn, } = makeRecordingSpawn({
              autoExitOnSigterm: true,
            },);
            const child = new Child({
              command: 'node',
              processGroup: false,
              stopTimeout: 200,
              spawn,
              writeClear,
            },);

            await child.start();
            await child.restart();
            await child.stop();

            expect(calls.count,).toBe(0,);
          },
        },),
      ],
    },),
    describe({
      name: 'reentry guards',
      children: [
        it({
          name: 'stop() during stopping logs warn and returns without further signals',
          fn: async () => {
            // Block SIGTERM from auto-exiting; we'll fire the second stop()
            // before the first one's SIGKILL escalation completes.
            const stopTimeoutMs = 100;
            const { spawn, records, } = makeRecordingSpawn({
              autoExitOnSigterm: false,
            },);
            const child = new Child({
              command: 'node',
              processGroup: false,
              stopTimeout: stopTimeoutMs,
              spawn,
            },);

            await child.start();
            const firstStop = child.stop();
            // Wait for `#stopRunning` to flip state to 'stopping' before
            // the second call; one event-loop tick is enough.
            await wait(0,);
            expect(child.state,).toBe('stopping',);
            const secondStop = child.stop();
            await Promise.all([
              firstStop,
              secondStop,
            ],);

            const { handle, } = nonNullishOrThrow(records[0],);
            // Exactly one SIGTERM + one SIGKILL escalation; the second
            // stop() must not have added more signals.
            expect(handle.signalsReceived,).toEqual(['SIGTERM', 'SIGKILL',],);
            expect(child.state,).toBe('idle',);
          },
        },),
      ],
    },),
  ],
},);
