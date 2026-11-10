/**
 Pi lifecycle fixtures shared by the settled-notification unit tests and verifiers.

 Every value here mirrors the contract Pi itself hands an extension handler, so a
 suite that passes proves the handler accepted a complete event and a context,
 rather than an object Pi would never deliver.

 @module
 */

import type {
  AgentSettledEvent,
  ExtensionAPI,
  ExtensionContext,
  ExtensionHandler,
} from '@earendil-works/pi-coding-agent';

//region Types

/**
 Lifecycle callback Pi registers for final agent settlement.
 */
type AgentSettledHandler = ExtensionHandler<AgentSettledEvent>;

/**
 Fake Pi API and captured lifecycle registrations.
 */
type FakePiHarness = {
  /**
   API supplied to the extension under test.
   */
  readonly api: ExtensionAPI;

  /**
   Handlers grouped by lifecycle event name.
   */
  readonly handlersByEvent: ReadonlyMap<string, readonly AgentSettledHandler[]>;
};

//endregion Types

//region Harness construction

/**
 Creates a fake Pi extension API that records lifecycle subscriptions.

 @returns fake API and captured event handlers

 @example
 ```ts
 const harness = createFakePiHarness();
 ```
 */
function createFakePiHarness(): FakePiHarness {
  /**
   Mutable registrations populated through the fake `on()` method.
   */
  const handlersByEvent = new Map<string, AgentSettledHandler[]>();

  /**
   Fake Pi API implementing only the event-registration surface under test.
   */
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- Fake API records lifecycle registrations; only `on()` is exercised here.
  const api = {
    on(
      event: string,
      handler: AgentSettledHandler,
    ): void {
      /**
       Existing handlers registered for this lifecycle event.
       */
      const existingHandlers = handlersByEvent.get(event,)
        ?? [];
      handlersByEvent.set(
        event,
        [
          ...existingHandlers,
          handler,
        ],
      );
    },
  } as unknown as ExtensionAPI;

  return {
    api,
    handlersByEvent,
  };
}

/**
 Retrieves the only handler registered for an event.

 @param handlersByEvent - captured Pi registrations

 @param event - lifecycle event whose callback is needed

 @returns registered lifecycle callback

 @throws when registration is absent or duplicated

 @example
 ```ts
 getOnlyHandler({ handlersByEvent: new Map(), event: 'agent_settled' });
 ```
 */
function getOnlyHandler(
  {
    handlersByEvent,
    event,
  }: {
    readonly handlersByEvent: ReadonlyMap<string, readonly AgentSettledHandler[]>;
    readonly event: string;
  },
): AgentSettledHandler {
  /**
   Captured handlers for the requested event.
   */
  const handlers = handlersByEvent.get(event,);
  if (handlers === undefined)
    throw new Error(`Missing handler registration: ${event}`,);
  if (handlers.length !== 1)
    throw new Error(`Expected one handler for ${event}, received ${handlers.length}`,);

  /**
   Sole handler registered for the requested lifecycle event.
   */
  const [handler,] = handlers;
  if (handler === undefined)
    throw new Error(`Missing handler registration: ${event}`,);
  return handler;
}

/**
 Creates the settled-event value Pi delivers after a run finishes.

 @param aborted - whether a user abort ended the run instead of completion

 @returns settled event carrying every field Pi's contract requires

 @example
 ```ts
 createAgentSettledEvent({ aborted: false });
 ```
 */
function createAgentSettledEvent(
  {
    aborted,
  }: {
    readonly aborted: boolean;
  },
): AgentSettledEvent {
  return {
    type: 'agent_settled',
    aborted,
  };
}

/**
 Creates the handler context Pi passes alongside every lifecycle event.

 @returns extension context stub, which the settled handler deliberately ignores

 @example
 ```ts
 createExtensionContext();
 ```
 */
function createExtensionContext(): ExtensionContext {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- Mirrors Pi's ExtensionContext at the fixture boundary; the settled handler reads nothing from the context, so an empty stand-in keeps the handler's real call shape.
  return {} as unknown as ExtensionContext;
}

//endregion Harness construction

export {
  createAgentSettledEvent,
  createExtensionContext,
  createFakePiHarness,
  getOnlyHandler,
};

export type {
  AgentSettledHandler,
  FakePiHarness,
};
