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
// - the queue reserves the entry's place when it decides to start it, so a signal between that decision and the entry's
//   own registration finds an entry in flight too (B1298).

/**
 Bound on how long a stop signal waits for the entry in flight to ship its interim page, in milliseconds.

 WELL UNDER SYSTEMD'S DEFAULT STOP TIMEOUT OF 90 SECONDS (`TimeoutStopSec`, which the runbook's unit leaves unset), so
 the flush, the release and the exit that follow it still happen before the unit is killed. Composing the page makes no
 provider call and reads what the entry already holds.
 */
export const STOP_SHIP_WINDOW_MS = 30_000;

/**
 Least time, in milliseconds, between the first stop signal and a later one for the later one to count as the operator's
 second signal, which ends the wait for the entry in flight (B1295). A later signal that arrives sooner is one stop
 delivered twice and changes nothing.

 MEASURED, NOT ASSUMED: on 2026-11-01 (UTC) one `systemctl --user stop` of a unit running `mise run <task>` with
 systemd's default `KillMode=control-group` delivered SIGTERM to the task's node process twice in the same millisecond
 (ledger B1295). The probe was one run on a quiet unit, so the value is a margin and not a fit.

 WHY ONE SECOND: it must outlast any gap between the two deliveries of one stop, which the probe put at 0 ms and which
 a loaded host can stretch by a scheduling delay of some milliseconds; and a deliberate second signal comes from an
 operator who has watched the first one not end the pass, which takes longer than a second to see and act on. A
 deliberate second signal that lands sooner than this is read as the duplicate and a third one ends the wait, so the
 cost of too long a gap is one more keypress and the cost of too short a gap is the interim page of the entry in flight.
 */
export const SECOND_SIGNAL_MIN_GAP_MS = 1_000;

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
   Holds a place in flight for the entry the queue has decided to start, from that decision to the end of its attempt
   (B1298). A signal that lands between the decision and the entry's own {@link PassStop.begin} then finds an entry in
   flight and waits for its page; without it the signal would find none, and the handler would flush, release and exit
   while the entry started under an aborted signal. The entry that begins takes the reservation over, so the wait ends
   when that entry says it shipped.
   */
  readonly reserve: () => Disposable;

  /**
   Registers the entry that starts, which the handler waits for; takes over the place {@link PassStop.reserve} holds
   when one is held.
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
  /**
   Place held for the entry the queue decided to start, which that entry's `begin` takes over; one at a time.
   */
  const reserved: { current?: PromiseWithResolvers<void>; } = {};
  return {
    signal: controller.signal,
    request: function request(): void {
      controller.abort(new DOMException(
        'the pass was told to stop',
        'AbortError',
      ),);
    },
    reserve: function reserve(): Disposable {
      /**
       The place held for the entry about to start.
       */
      const flight = Promise.withResolvers<void>();
      reserved.current = flight;
      flights.current = flight;
      return {
        [Symbol.dispose](): void {
          flight.resolve();
          if (reserved.current === flight)
            delete reserved.current;
          if (flights.current === flight)
            delete flights.current;
        },
      };
    },
    begin: function begin(): EntryFlight {
      /**
       This entry's flight: the place the queue reserved for it, or a new one where none was reserved.
       */
      const flight = reserved.current ?? Promise.withResolvers<void>();
      delete reserved.current;
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
