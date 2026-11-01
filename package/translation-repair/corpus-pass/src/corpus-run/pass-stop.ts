//region Pass stop
// WHAT A STOP SIGNAL SHARES WITH THE ENTRY IN FLIGHT (B1247). A pass told to stop used to exit at once (B1113), so the
// entry it was working on shipped nothing; the owner's rule is "no matter what, we must ship". The signal handler and the
// entry meet here, in an object the pass makes and hands to both, with no module global between them:
//
// - `signal` is aborted when a stop signal arrives, and the entry's deadline forwards it, so every stage of the entry
//   ends at its next check and the entry's own `catch` ships the interim page from the progress it holds. No provider is
//   asked again for it.
// - an entry registers a flight when it starts, tells the stop that its page is shipped, and leaves; the handler waits
//   for that, within a window it chooses, before it flushes the records and releases the lock.

/**
 Bound on how long a stop signal waits for the entry in flight to ship its interim page, in milliseconds.

 WELL UNDER SYSTEMD'S DEFAULT STOP TIMEOUT OF 90 SECONDS (`TimeoutStopSec`, which the runbook's unit leaves unset), so
 the flush, the release and the exit that follow it still happen before the unit is killed. Composing the page makes no
 provider call and reads what the entry already holds.
 */
export const STOP_SHIP_WINDOW_MS = 30_000;

/**
 One entry's place in the stop: held while the entry runs, told when its page is shipped.
 */
export type EntryFlight = Disposable & {
  /**
   Says the page of the stop is shipped (or none was due), which ends the wait of the signal handler early.
   */
  readonly shipped: () => void;
};

/**
 What the signal handler and the entry in flight share.
 */
export type PassStop = {
  /**
   Aborted when a stop signal arrives.
   */
  readonly signal: AbortSignal;

  /**
   Aborts {@link PassStop.signal}, which a stop signal does once.
   */
  readonly request: () => void;

  /**
   Registers the entry that starts, which the handler waits for.
   */
  readonly begin: () => EntryFlight;

  /**
   Whether an entry is in flight, which a stop signal waits for.
   */
  readonly inFlight: () => boolean;

  /**
   Settles when no entry is in flight, or when the entry in flight has shipped or has ended.
   */
  readonly inFlightShipped: () => Promise<void>;

  /**
   Settles after the handler has called the exit, which a test's stand-in returns from and the process's own does not.
   */
  readonly exited: Promise<void>;

  /**
   Says the handler has called the exit.
   */
  readonly markExited: () => void;
};

/**
 Makes the stop one pass shares between its signal handler and its entries.

 @returns Stop that nothing has requested and no entry is in flight in

 @example
 ```ts
 const stop = createPassStop();
 ```
 */
export function createPassStop(): PassStop {
  /**
   Aborts the signal.
   */
  const controller = new AbortController();
  /**
   Settles when the handler has called the exit.
   */
  const exit = Promise.withResolvers<void>();
  /**
   Entry in flight, whose page the handler waits for; one entry runs at a time.
   */
  const flights: { current?: PromiseWithResolvers<void>; } = {};
  return {
    signal: controller.signal,
    request: function request(): void {
      controller.abort(new DOMException(
        'the pass was told to stop',
        'AbortError',
      ),);
    },
    begin: function begin(): EntryFlight {
      /**
       This entry's flight.
       */
      const flight = Promise.withResolvers<void>();
      flights.current = flight;
      return {
        shipped: function shipped(): void {
          flight.resolve();
        },
        [Symbol.dispose](): void {
          flight.resolve();
          if (flights.current === flight)
            delete flights.current;
        },
      };
    },
    inFlight: function inFlight(): boolean {
      return flights.current !== undefined;
    },
    inFlightShipped: function inFlightShipped(): Promise<void> {
      /**
       The entry in flight, absent between entries.
       */
      const { current, } = flights;
      return (current === undefined) ? Promise.resolve() : current.promise;
    },
    exited: exit.promise,
    markExited: function markExited(): void {
      exit.resolve();
    },
  };
}

//endregion Pass stop
