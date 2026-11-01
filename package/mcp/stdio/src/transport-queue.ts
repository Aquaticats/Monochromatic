// Serial request queue: frees the stdin read loop while tool execution stays one at a time.

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import type { JsonRpcId, } from './json-rpc.ts';

//region Queue vocabulary: entries, sentinels, and the public surface

/**
 Marks a queue entry that no `notifications/cancelled` can ever match.
 
 Parse-error and shape-error frames are queued for ordering alone: they carry no
 request id a client could name, so they must not collide with a real id in the
 cancellation bookkeeping.
 */
export const UNCANCELLABLE: unique symbol = Symbol('mcp-stdio uncancellable queue entry',);

/**
 Returned by a producer that has nothing to write, such as a dispatched notification.
 */
export const NO_FRAME: unique symbol = Symbol('mcp-stdio no outbound frame',);

/**
 Runs one queued entry and yields its serialized frame.
 
 Called at most once per entry, and never at all when cancellation arrives before
 this entry reaches the front of the queue.
 */
export type FrameProducer = () => Promise<string | typeof NO_FRAME>;

/**
 Identity a queue entry is cancelled by, or {@link UNCANCELLABLE} when it has none.
 */
export type QueuedId = JsonRpcId | typeof UNCANCELLABLE;

/**
 FIFO queue running one entry at a time and writing every frame through one path.
 
 @example
 ```ts
 const queue = createSerialRequestQueue({ write: async (frame) => { await sink(frame); } });
 queue.enqueue({ id: 1, produce: async () => '{"jsonrpc":"2.0","id":1,"result":{}}' });
 await queue.idle();
 ```
 */
export type SerialRequestQueue = {
  /**
   Accepts one entry and returns immediately, leaving the caller free to keep reading.
   */
  readonly enqueue: (entry: {
    readonly id: QueuedId;
    readonly produce: FrameProducer;
  },) => void;

  /**
   Cancels a queued or running entry, reporting whether any entry matched.
   */
  readonly cancel: (target: { readonly id: JsonRpcId; },) => boolean;

  /**
   Resolves once no entry is queued or running.
   */
  readonly idle: () => Promise<void>;
};

//endregion

//region Queue internals

/**
 One accepted entry awaiting its turn.
 */
type QueuedEntry = {
  /**
   Identity unique to this entry, unlike `id`, which a client may repeat.
   */
  readonly ticket: number;

  /**
   Identity this entry is cancelled by.
   */
  readonly id: QueuedId;

  /**
   Work producing this entry's outbound frame.
   */
  readonly produce: FrameProducer;
};

//endregion

//region Queue construction

/**
 Creates a queue that serializes execution and output for one stdio connection.
 
 The read loop calls {@link SerialRequestQueue.enqueue} and moves on, so a tool running
 for half an hour no longer stops the server from reading `notifications/cancelled`.
 Execution itself stays strictly serial: exactly one producer runs at a time, which is
 what the transport did before by accident and now does by design, so no two tool
 handlers can interleave against a shared backend.
 
 Cancellation follows the revision 2026-07-28 schema, which states a cancellation
 "MAY arrive after the request has already finished" and that "the result will be
 unused, so any associated processing SHOULD cease". An entry cancelled before it runs
 is dropped without ever being produced; one cancelled while running is allowed to
 finish, because aborting a half-created VM is worse than completing it, and its frame
 is discarded unwritten. A cancellation naming an unknown or already-settled id is
 ignored.
 
 @param write - Sink for one serialized frame, awaited before the next entry runs
 
 @returns Queue owning execution order, cancellation, and write order
 
 @example
 ```ts
 const queue = createSerialRequestQueue({
   write: async function writeFrame(frame) {
     await output.write(encoder.encode(`${frame}\n`,),);
   },
 });
 ```
 */
export function createSerialRequestQueue(
  { write, }: {
    readonly write: (frame: string,) => Promise<void>;
  },
): SerialRequestQueue {
  /**
   Entries waiting behind whichever entry is running.
   */
  const waiting: QueuedEntry[] = [];

  /**
   Tickets still live, grouped by the id a client would name to cancel them.
   
   A set rather than one ticket because JSON-RPC ids are the client's to choose and a
   misbehaving one may leave two requests outstanding under the same id. Keying
   cancellation by id alone would let settling either erase the other's state.
   */
  const liveTickets = new Map<JsonRpcId, Set<number>>();

  /**
   Tickets cancelled while live; consulted before producing and again before writing.
   */
  const cancelled = new Set<number>();

  /**
   Source of ticket numbers, held in a `const` so no function-root `let` is needed.
   */
  const tickets = { issued: 0, };

  /**
   Synchronous marker that a drain has been started.
   
   Set before {@link drain} is invoked. The promise holder alone cannot serve this
   purpose: `drain()` is evaluated before its promise is stored, and it runs as far as
   its first suspension, so an entry enqueued from inside that stretch would see no
   active drain and start a second one.
   */
  const running = new Map<'active', true>();

  /**
   Single-key holder for the active drain, avoiding a function-root `let`.
   */
  const draining = new Map<'active', Promise<void>>();

  /**
   Failure that ended a drain, rethrown from {@link SerialRequestQueue.idle}.
   
   Held rather than left on the drain promise so a rejection cannot go unhandled while
   the read loop is still running.
   */
  const failure = new Map<'error', unknown>();

  /**
   Forgets one entry's bookkeeping once it can no longer be cancelled.
   
   Keeps both structures bounded by the number of entries in flight rather than by
   everything this connection has ever handled.
   
   @param entry - Entry that has settled or been dropped
   */
  function release(entry: QueuedEntry,): void {
    cancelled.delete(entry.ticket,);
    if (entry.id === UNCANCELLABLE)
      return;
    /**
     Tickets still outstanding under this entry's id.
     */
    const siblings = liveTickets.get(entry.id,);
    if (siblings === undefined)
      return;
    siblings.delete(entry.ticket,);
    if (siblings.size === 0)
      liveTickets.delete(entry.id,);
  }

  /**
   Runs one entry and writes its frame unless cancellation overtook it.
   
   A producer is expected to answer its own failures with an error frame; one that throws
   anyway would leave its request unanswered forever, so the failure ends the drain rather
   than being logged and stepped over. A write failure ends it for the same reason: the
   frame is already lost and the stream it was lost on cannot carry a report of that.
   
   @param entry - Entry reaching the front of the queue
   
   @mutates entry - Invokes this entry's producer, which dispatches to a tool handler.
   
   @throws Whatever a producer or the write sink threw
   */
  async function settle(entry: QueuedEntry,): Promise<void> {
    // Cancelled before reaching the front: never dispatched at all, which is the one
    // case where processing genuinely ceases rather than merely going unreported.
    if (cancelled.has(entry.ticket,)) {
      release(entry,);
      return;
    }

    // Deliberate catch-and-rethrow: release must happen on every path, and `using` cannot
    // express it here because the disposal has to run before the failure propagates.
    try {
      /**
       Frame this entry produced, or the sentinel when it has nothing to send.
       */
      const frame = await entry.produce();
      if ((frame !== NO_FRAME) && (!cancelled.has(entry.ticket,)))
        await write(frame,);
    }
    catch (error: unknown) {
      release(entry,);
      throw error;
    }
    release(entry,);
  }

  /**
   Runs queued entries in order until none remain, removing each from the waiting list.
   */
  async function drain(): Promise<void> {
    /**
     Clears active-drain state even when an entry throws past {@link settle}.
     */
    using _drainCleanup = {
      [Symbol.dispose](): void {
        running.delete('active',);
        draining.delete('active',);
      },
    };
    while (waiting.length > 0) {
      /**
       Next entry in acceptance order.
       */
      const entry = waiting.shift();
      if (entry === undefined)
        throw new Error('mcp-stdio request queue was empty during dequeue',);
      // oxlint-disable-next-line eslint/no-await-in-loop -- serial execution is this queue's contract; overlapping entries would let two tool handlers mutate one backend at once
      await settle(entry,);
    }
  }

  /**
   Starts a drain when none is running, leaving an active one to pick up new entries.
   
   Records the failure rather than letting the drain promise reject on its own, so a
   rejection cannot go unobserved while the read loop is still consuming stdin.
   
   Publishes the active marker before any drain code runs.
   */
  function startDraining(): void {
    if (running.has('active',))
      return;
    running.set(
      'active',
      true,
    );
    draining.set(
      'active',
      recordFailure(),
    );
  }

  /**
   Runs a drain, storing whatever ended it for {@link SerialRequestQueue.idle} to rethrow.
   */
  async function recordFailure(): Promise<void> {
    // Deliberate catch-and-store: rethrowing here would surface as an unhandled rejection
    // long before `idle` is awaited.
    try {
      await drain();
    }
    catch (error: unknown) {
      console.error(
        '[mcp-stdio] request queue stopped:',
        caughtValueText(error,),
      );
      failure.set(
        'error',
        error,
      );
    }
  }

  return {
    enqueue({
      id,
      produce,
    },): void {
      tickets.issued += 1;
      /**
       This entry's identity, distinct from the id even when a client repeats one.
       */
      const ticket = tickets.issued;
      if (id !== UNCANCELLABLE) {
        /**
         Tickets already outstanding under this id, created on first use.
         */
        const siblings = liveTickets.get(id,) ?? new Set<number>();
        siblings.add(ticket,);
        liveTickets.set(
          id,
          siblings,
        );
      }
      waiting.push({
        ticket,
        id,
        produce,
      },);
      startDraining();
    },

    cancel({ id, },): boolean {
      /**
       Tickets this cancellation names.
       */
      const targets = liveTickets.get(id,);
      // A cancellation for an id that never arrived, or already settled, is expected
      // rather than exceptional: the schema allows one to arrive after completion.
      if (targets === undefined)
        return false;
      for (const ticket of targets)
        cancelled.add(ticket,);

      // Dropping cancelled entries here rather than skipping them at the front releases
      // whatever their producers captured immediately, which matters when the running
      // entry is a tool that takes half an hour.
      /**
       Entries still waiting that this cancellation did not name.
       */
      const survivors = waiting.filter(function isUncancelled(entry,): boolean {
        return !cancelled.has(entry.ticket,);
      },);
      for (const entry of waiting)
        if (cancelled.has(entry.ticket,))
          release(entry,);
      waiting.length = 0;
      for (const entry of survivors)
        waiting.push(entry,);
      return true;
    },

    async idle(): Promise<void> {
      /**
       Active drain, if one is running.
       */
      const active = draining.get('active',);
      if (active !== undefined)
        await active;
      if (!failure.has('error',))
        return;
      /**
       Failure that ended the drain, cleared so a later call reports a fresh one.
       */
      const error = failure.get('error',);
      failure.delete('error',);
      throw error;
    },
  };
}

//endregion
