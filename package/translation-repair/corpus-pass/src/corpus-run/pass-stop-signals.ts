import { failureName, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  recordRunLine,
  type RunLog,
} from '@monochromatic-dev/translation-repair-typography/ts';

//region Pass stop signals
// WHAT A PASS DOES WHEN IT IS TOLD TO STOP (ledger B1113). An operator's `systemctl --user stop`, a reboot's SIGTERM and
// a Ctrl-C on a foreground launch end a process that has no handler at once, with nothing in its run folder to say why
// and its lock file left for the next pass to judge stale. A pass that handles the signal records that it stopped and
// which signal said so, flushes its run log, releases the lock of its runs directory and exits with the conventional
// code, 128 and the signal's number. What it does not do is wait for the entry in flight: an entry's artifact is
// written to a temporary name and renamed, so a stop mid-write leaves nothing half-written, and the entry is still
// pending for the next pass, as it is after any kill.
//
// WHAT HOLDS, SAID PLAINLY: the run log is flushed on a return and on a throw out of the pass body and on these two
// signals. It is NOT flushed on SIGKILL, on a crash of the process or on a power loss, which no handler sees; a record
// written after the last flush is then lost, and the lock is left for the next pass to find stale.

/**
 The signals that stop a pass, with the codes it exits with: 128 and the signal's number.
 */
const STOP_SIGNALS = {
  SIGINT: 130,
  SIGTERM: 143,
} as const;

/**
 Name of a signal that stops a pass.
 */
type StopSignal = keyof typeof STOP_SIGNALS;

/**
 Every signal that stops a pass, in the order handlers are registered.
 */
const STOP_SIGNAL_NAMES: readonly StopSignal[] = [
  'SIGINT',
  'SIGTERM',
];

/**
 What the pass listens for signals on: the process, or a stand-in a case plants.
 */
export type StopSignalSource = {
  /**
   Registers a handler for a signal.
   */
  readonly on: (
    signal: StopSignal,
    handler: () => void,
  ) => unknown;

  /**
   Takes a handler off.
   */
  readonly off: (
    signal: StopSignal,
    handler: () => void,
  ) => unknown;
};

/**
 Ends the process, as the process's own `exit` does.

 @param code - exit code

 @example
 ```ts
 exitProcess(143,);
 ```
 */
function exitProcess(code: number,): void {
  // oxlint-disable-next-line unicorn/no-process-exit -- the pass is still running when the signal arrives, so only an exit ends it with the signal's conventional code (ledger B1113)
  process.exit(code,);
}

/**
 Handles the signals that stop a pass until the returned disposable is disposed.

 @param log - run log to flush

 @param release - releases the lock of the runs directory

 @param source - signals are listened for here, the process unless a case plants another

 @param exit - ends the process with a code, the process's own unless a case plants another

 @returns Disposable taking the handlers off

 @example
 ```ts
 using _signals = handlingStopSignals({ log: runLog, release: () => lock[Symbol.asyncDispose](), },);
 ```
 */
export function handlingStopSignals(
  {
    log,
    release,
    source = process,
    exit = exitProcess,
  }: {
    readonly log: RunLog;
    readonly release: () => Promise<void>;
    readonly source?: StopSignalSource;
    readonly exit?: (code: number,) => void;
  },
): Disposable {
  /**
   The stop in progress, which a second signal does not start again.
   */
  const stopping: { current?: Promise<void>; } = {};

  /**
   Handlers by signal, kept so they can be taken off.
   */
  const handlers = STOP_SIGNAL_NAMES.map(function handlerFor(signal,): readonly [
    StopSignal,
    () => void,
  ] {
    return [
      signal,
      function onSignal(): void {
        stopping.current ??= stopOn({
          signal,
          log,
          release,
          exit,
        },);
      },
    ];
  },);
  for (const [signal, handler,] of handlers)
    source.on(
      signal,
      handler,
    );
  return {
    [Symbol.dispose](): void {
      for (const [signal, handler,] of handlers)
        source.off(
          signal,
          handler,
        );
    },
  };
}

/**
 Stops the pass: records why, flushes, releases the lock and exits.

 @param signal - signal that arrived

 @param log - run log to flush

 @param release - releases the lock

 @param exit - ends the process

 @example
 ```ts
 await stopOn({ signal: 'SIGTERM', log, release, exit, },);
 ```
 */
async function stopOn(
  {
    signal,
    log,
    release,
    exit,
  }: {
    readonly signal: StopSignal;
    readonly log: RunLog;
    readonly release: () => Promise<void>;
    readonly exit: (code: number,) => void;
  },
): Promise<void> {
  recordRunLine({
    level: 'error',
    line: `[corpus-pass] the pass stopped: ${signal} arrived, so it ended before it finished; the entries it had not `
      + 'settled are still pending for the next pass',
  },);
  // EACH STEP IS TRIED WHATEVER THE ONE BEFORE DID: a flush that fails must not keep the lock, and a lock that will not
  // release must not keep the process from exiting.
  for (const step of [
    log.flush,
    release,
  ]) {
    try {
      // oxlint-disable-next-line no-await-in-loop -- the flush finishes before the lock is let go, so the folder holds the reason when the next pass starts
      await step();
    } catch (error) {
      console.error(`corpus-pass: a step of stopping on ${signal} failed (${failureName({ error, },)})`,);
    }
  }
  exit(STOP_SIGNALS[signal],);
}

//endregion Pass stop signals
