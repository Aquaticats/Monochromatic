/**
 Fixture transport whose reply lands after a delay on the process's timer function, for the cases that need a call to
 last minutes without waiting for them: the case replaces the timers with fake ones and ticks the clock. The reply is
 cut the way a real exchange is, by the exchange's abort signal, so a signal that fires before the reply lands shows as
 a call that never answered. A case that stops ticking calls `release`, which ends every exchange still waiting and
 every later one, so no call outlives the case. Nothing here reaches a provider.

 @module
 */

import type {
  ModelTransport,
  TransportExchange,
  TransportReply,
} from '@monochromatic-dev/translation-repair-provider-stream';

/**
 What the release answers every exchange with: a refusal no retry ladder repeats.
 */
const RELEASED_REPLY: TransportReply = {
  status: 400,
  bodyText: '{"type":"error","error":{"type":"invalid_request_error",'
    + '"message":"the delayed transport was released"}}',
};

/**
 Builds a transport whose reply lands after the given delay, and which gives up the moment
 the exchange's signal aborts.

 @param landMs - delay in milliseconds, on whichever timer function is global when the exchange is made

 @param reply - what lands

 @returns Transport; the exchanges it was handed; a promise that settles at the first; the reasons the cut ones were
 aborted for, which an exchange the release ended is not among; and the release, which ends every exchange still
 waiting and every later one with an HTTP 400 reply, a refusal the client does not retry, so the walk under test comes
 to an end at once and not after the client's backoffs

 @example
 ```ts
 const { transport, } = transportLandingAfter({ landMs: 400_000, reply: { status: 200, bodyText, }, },);
 ```
 */
export function transportLandingAfter(
  {
    landMs,
    reply,
  }: {
    readonly landMs: number;
    readonly reply: TransportReply;
  },
): {
  readonly transport: ModelTransport;
  readonly exchanges: TransportExchange[];
  readonly firstCall: Promise<void>;
  readonly cut: unknown[];
  readonly release: () => void;
} {
  /**
   Every exchange the transport was handed, in order.
   */
  const exchanges: TransportExchange[] = [];

  /**
   Capability that settles `firstCall`.
   */
  const called = Promise.withResolvers<void>();

  /**
   The reason each exchange the abort signal cut was aborted with, in order.
   */
  const cut: unknown[] = [];

  /**
   Ends of the exchanges still waiting, each clearing its timer and settling.
   */
  const waiting = new Set<() => void>();

  /**
   Whether the case released the transport.
   */
  const state = { released: false, };
  return {
    cut,
    exchanges,
    firstCall: called.promise,
    release: function release(): void {
      state.released = true;
      // A Set drops an element deleted during the loop and does not visit it, so each `end` removing itself is safe.
      for (const end of waiting)
        end();
    },
    transport: function landing(exchange: TransportExchange,): Promise<TransportReply> {
      exchanges.push(exchange,);
      called.resolve();
      if (state.released)
        return Promise.resolve(RELEASED_REPLY,);

      /**
       Capability to settle the exchange from the timer, the abort or the release.
       */
      const {
        promise,
        resolve,
        reject,
      } = Promise.withResolvers<TransportReply>();

      /**
       Ends this exchange at the release with the refusal.
       */
      function end(): void {
        clearTimeout(timer,);
        waiting.delete(end,);
        resolve(RELEASED_REPLY,);
      }

      /**
       Ends this exchange at the abort, with the signal's reason.
       */
      function cutShort(): void {
        clearTimeout(timer,);
        waiting.delete(end,);
        cut.push(exchange.signal
          .reason,);
        reject(exchange.signal
          .reason,);
      }

      /**
       Timer that lands the reply.
       */
      const timer = setTimeout(
        function land(): void {
          waiting.delete(end,);
          resolve(reply,);
        },
        landMs,
      );
      if (exchange.signal
        .aborted) {
        cutShort();
        return promise;
      }
      waiting.add(end,);
      exchange.signal
        .addEventListener(
        'abort',
        cutShort,
        { once: true, },
        );
      return promise;
    },
  };
}
