/**
 Async event-stream driver exposing the pi-ai completion stream surface the
 shared Advisor completion seam consumes. @module
 */

import type {
  AssistantMessage,
  AssistantMessageEvent,
} from '@earendil-works/pi-ai';

//region Types

/**
 Iteration and terminal-result surface of one assistant response stream.

 Matches the consumed part of pi-ai `AssistantMessageEventStream`: event
 iteration plus the terminal message future. The pi-ai class itself is
 nominal (private fields), so provider construction asserts this surface to
 that type once, at the shim boundary.
 */
export type DshAdvisorStreamSurface = {
  /**
   Iterate streamed assistant events in publish order.
   */
  [Symbol.asyncIterator]: () => AsyncIterator<AssistantMessageEvent>;
  /**
   Resolve with the terminal assistant message once the stream concludes.
   */
  result: () => Promise<AssistantMessage>;
};

/**
 Publisher handle driving one {@link DshAdvisorStreamSurface}.
 */
export type DshAdvisorStreamDriver = {
  /**
   Stream surface handed to the shared completion seam.
   */
  readonly surface: DshAdvisorStreamSurface;
  /**
   Publish one streamed event before the terminal conclusion.
   */
  publish: (event: AssistantMessageEvent) => void;
  /**
   Conclude the stream with its terminal message exactly once.
   */
  conclude: (message: AssistantMessage) => void;
};

//endregion Types

//region Public API

/**
 Create one event-stream driver for one provider response.

 @returns driver whose surface streams events and resolves with the terminal
 message

 @example
 ```ts
 const driver = createDshAdvisorStreamDriver();
 driver.publish({ type: 'start', partial });
 driver.conclude(message);
 ```
 */
export function createDshAdvisorStreamDriver(): DshAdvisorStreamDriver {
  /**
   Events published while no iterator pull is pending.
   */
  const bufferedEvents: AssistantMessageEvent[] = [];
  /**
   Iterator pulls waiting for the next published event.
   */
  const eventWaiters: ((chunk: IteratorResult<AssistantMessageEvent>) => void)[] = [];
  /**
   Conclusion flag shared by publisher and iterator checks.
   */
  const state = {
    concluded: false,
  };
  /**
   Terminal-message future created with its resolver captured by array push.
   */
  const terminal = Promise.withResolvers<AssistantMessage>();
  /**
   Pull one buffered event, or suspend until the publisher provides one or the
   stream concludes.

   @returns next iterator result, resolving when an event arrives
   */
  function next(): Promise<IteratorResult<AssistantMessageEvent>> {
    /**
     Buffered event available without waiting.
     */
    const buffered = bufferedEvents.shift();
    if (buffered !== undefined)
      return Promise.resolve({
        value: buffered,
        done: false,
      },);
    if (state.concluded)
      return Promise.resolve({
        value: undefined,
        done: true,
      },);
    /**
     Pending-event future created with its resolver captured by push.
     */
    const pending = Promise.withResolvers<IteratorResult<AssistantMessageEvent>>();
    eventWaiters.push(pending.resolve,);
    return pending.promise;
  }
  /**
   Stream surface consuming the driver buffers.
   */
  const surface: DshAdvisorStreamSurface = {
    [Symbol.asyncIterator]: function asyncIterator() {
      return {
        next,
      };
    },
    result: function result(): Promise<AssistantMessage> {
      return terminal.promise;
    },
  };
  return {
    surface,
    publish: function publish(event: AssistantMessageEvent): void {
      if (state.concluded)
        throw new Error('advisor: stream driver received an event after conclusion',);
      /**
       Pending iterator pull, when one is waiting.
       */
      const waiter = eventWaiters.shift();
      if (waiter === undefined) {
        bufferedEvents.push(event,);
        return;
      }
      waiter({
        value: event,
        done: false,
      },);
    },
    conclude: function conclude(message: AssistantMessage): void {
      if (state.concluded)
        throw new Error('advisor: stream driver concluded twice',);
      state.concluded = true;
      for (const waiter of eventWaiters.splice(0,))
        waiter({
          value: undefined,
          done: true,
        },);
      terminal.resolve(message,);
    },
  };
}

//endregion Public API
