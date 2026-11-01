import {
  failureName,
  monotonicMs,
} from '@monochromatic-dev/translation-repair-markup/ts';
import {
  recordRunLine,
  type RunLog,
} from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type PassStop,
  SECOND_SIGNAL_MIN_GAP_MS,
  STOP_SHIP_WINDOW_MS,
} from './pass-stop.ts';

//region Pass stop signals
// WHAT A PASS DOES WHEN IT IS TOLD TO STOP (ledger B1113). An operator's `systemctl --user stop`, a reboot's SIGTERM and
// a Ctrl-C on a foreground launch end a process that has no handler at once, with nothing in its run folder to say why
// and its lock file left for the next pass to judge stale. A pass that handles the signal records that it stopped and
// which signal said so, flushes its run log, releases the lock of its runs directory and exits with the conventional
// code, 128 and the signal's number.
//
// THE ENTRY IN FLIGHT SHIPS BEFORE THE PASS EXITS (B1247; "no matter what, we must ship"). The handler aborts the signal
// the entry runs under (`pass-stop.ts`), the entry's own `catch` composes its interim page from the progress it holds,
// with no provider call, writes it and its record, and tells the stop; only then does the handler flush, release and
// exit. The wait has a bound (`STOP_SHIP_WINDOW_MS`, well under systemd's default stop timeout of 90 seconds), and a
// second signal ends it at once, as B1113 ended the pass; an entry that has not shipped by then is still pending for the
// next pass, as it is after any kill.
//
// WHAT "A SECOND SIGNAL" MEANS (B1295, B1297). One `systemctl --user stop` of a unit that runs `mise run <task>`
// delivered SIGTERM to the task's node process twice in the same millisecond (measured 2026-11-01 UTC; the unit's
// default `KillMode=control-group` signals the wrapper and the pass, and the wrapper relays), and a Ctrl-C of a
// foreground launch reaches the wrapper and the pass alike. So a later signal counts as the operator's second only when
// it arrives `SECOND_SIGNAL_MIN_GAP_MS` or more after the first, whichever signal it is; an earlier one is one stop
// delivered twice, is ignored, and is recorded with how long after the first it came. A second signal ends the wait for
// the entry in flight and nothing after it: the flush and the release that follow run to their end, take no signal
// and have no time bound of their own, so a disk that hangs there is ended by systemd's kill at the stop timeout.
//
// WHAT SHIPS WHEN THE SIGNAL LANDS MID-WRITE. Every page, record and artifact is written to a temporary name and renamed,
// so a write is either not there or whole. A signal during an interim ship lets that ship finish within the window; a
// window that ends, or a second signal, between the page and its record leaves the new page with no record, which
// `verify-published` names as a page with no artifact and the next pass treats as a standing page. A signal during the
// settled entry's own write lets that write finish within the window as well: the entry does not check the signal after
// its settlement stages, so it settles, and the interim page it would have shipped is not needed.
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

 @param stop - what the pass shares with its entries: the handler aborts its signal and waits for the entry in flight to
 ship its interim page; absent, the pass exits at once as B1113 had it

 @param windowMs - most the handler waits for that page, `STOP_SHIP_WINDOW_MS` by default

 @param now - clock, in milliseconds, that tells a later signal from one stop delivered twice by how long after the first
 it came; the monotonic clock unless a case plants another

 @param source - signals are listened for here, the process unless a case plants another

 @param exit - ends the process with a code, the process's own unless a case plants another

 @returns Disposable taking the handlers off

 @example
 ```ts
 using _signals = handlingStopSignals({ log: runLog, release: () => lock[Symbol.asyncDispose](), stop, },);
 ```
 */
export function handlingStopSignals(
  {
    log,
    release,
    stop,
    windowMs = STOP_SHIP_WINDOW_MS,
    now = monotonicMs,
    source = process,
    exit = exitProcess,
  }: {
    readonly log: RunLog;
    readonly release: () => Promise<void>;
    readonly stop?: PassStop;
    readonly windowMs?: number;
    readonly now?: () => number;
    readonly source?: StopSignalSource;
    readonly exit?: (code: number,) => void;
  },
): Disposable {
  /**
   The stop in progress, which a second signal does not start again.
   */
  const stopping: { current?: Promise<void>; } = {};

  /**
   When the first signal arrived, on the clock `now` reads; absent until it does.
   */
  const firstAt: { ms?: number; } = {};

  /**
   Settles on a second signal, which ends the wait for the entry in flight.
   */
  const secondSignal = Promise.withResolvers<void>();

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
        /**
         This signal's arrival.
         */
        const at = now();
        if (stopping.current !== undefined) {
          /**
           How long after the first signal this one came.
           */
          const sinceFirst = at - (firstAt.ms ?? at);
          if (sinceFirst < SECOND_SIGNAL_MIN_GAP_MS) {
            // ONE STOP DELIVERED TWICE (B1295): the wrapper and the pass both heard it, so this is not the operator's.
            recordRunLine({
              level: 'warn',
              line: `[corpus-pass] ${signal} arrived again ${String(sinceFirst,)} ms after the first, inside the `
                + `${String(SECOND_SIGNAL_MIN_GAP_MS,)} ms in which two signals are one stop delivered twice, so it `
                + 'is ignored',
            },);
            return;
          }
          secondSignal.resolve();
          return;
        }
        firstAt.ms = at;
        stopping.current = stopOn({
          signal,
          log,
          release,
          exit,
          windowMs,
          secondSignal: secondSignal.promise,
          ...((stop === undefined) ? {} : { stop, }),
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

 @param windowMs - most the entry in flight is waited for

 @param secondSignal - settles when a second signal arrives, which ends the wait

 @param stop - what the pass shares with its entries, absent where the pass exits at once

 @example
 ```ts
 await stopOn({ signal: 'SIGTERM', log, release, exit, windowMs, secondSignal, stop, },);
 ```
 */
async function stopOn(
  {
    signal,
    log,
    release,
    exit,
    windowMs,
    secondSignal,
    stop,
  }: {
    readonly signal: StopSignal;
    readonly log: RunLog;
    readonly release: () => Promise<void>;
    readonly exit: (code: number,) => void;
    readonly windowMs: number;
    readonly secondSignal: Promise<void>;
    readonly stop?: PassStop;
  },
): Promise<void> {
  recordRunLine({
    level: 'error',
    line: `[corpus-pass] the pass stopped: ${signal} arrived, so it ended before it finished; the entries it had not `
      + 'settled are still pending for the next pass',
  },);
  if (stop !== undefined) {
    // THE ENTRY IN FLIGHT SHIPS FIRST (B1247): its signal is aborted, its own `catch` writes the interim page and the
    // record, and the pass leaves only when it says so, when the window ends or when a second signal arrives.
    stop.request();
    if (stop.inFlight()) {
      /**
       How the wait for the entry in flight ended.
       */
      const waited = await waitForShip({
        stop,
        windowMs,
        secondSignal,
      },);
      recordRunLine({
        level: 'error',
        line: (waited === 'shipped')
          ? '[corpus-pass] the entry in flight shipped its interim page'
          : ((waited === 'second-signal')
            ? '[corpus-pass] a second signal arrived while the pass waited for the entry in flight, so the pass exits '
              + 'without its interim page'
            : `[corpus-pass] the entry in flight did not ship its interim page within ${String(windowMs,)} ms, so the `
              + 'pass exits without it'),
      },);
    }
  }
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
  stop?.markExited();
}

/**
 Passes one source's settling to the wait that is listening for it.

 @param source - promise whose settling is the outcome

 @param outcome - what the wait reports when the source settles

 @param ended - resolver of the wait, which keeps the first value it is given

 @example
 ```ts
 void relayOutcome({ source: shipped, outcome: 'shipped', ended, },);
 ```
 */
async function relayOutcome(
  {
    source,
    outcome,
    ended,
  }: {
    readonly source: Promise<void>;
    readonly outcome: 'shipped' | 'second-signal';
    readonly ended: PromiseWithResolvers<'shipped' | 'window-ended' | 'second-signal'>;
  },
): Promise<void> {
  await source;
  ended.resolve(outcome,);
}

/**
 Waits for the entry in flight to say its interim page is shipped, for a window, or for a second signal.

 @param stop - what the pass shares with its entries

 @param windowMs - most the wait lasts

 @param secondSignal - settles when a second signal arrives

 @returns What ended the wait

 @example
 ```ts
 const waited = await waitForShip({ stop, windowMs: 30_000, secondSignal, },);
 ```
 */
async function waitForShip(
  {
    stop,
    windowMs,
    secondSignal,
  }: {
    readonly stop: PassStop;
    readonly windowMs: number;
    readonly secondSignal: Promise<void>;
  },
): Promise<'shipped' | 'window-ended' | 'second-signal'> {
  /**
   Settles with whichever of the three ends the wait first; the first value stands and a later one is dropped.
   */
  const ended = Promise.withResolvers<'shipped' | 'window-ended' | 'second-signal'>();
  /**
   The window's timer, cleared when the wait ends first.
   */
  const timer = setTimeout(
    function endWindow(): void {
      ended.resolve('window-ended',);
    },
    windowMs,
  );
  void relayOutcome({
    source: stop.inFlightShipped(),
    outcome: 'shipped',
    ended,
  },);
  void relayOutcome({
    source: secondSignal,
    outcome: 'second-signal',
    ended,
  },);
  /**
   What ended the wait.
   */
  const waited = await ended.promise;
  clearTimeout(timer,);
  return waited;
}

//endregion Pass stop signals
