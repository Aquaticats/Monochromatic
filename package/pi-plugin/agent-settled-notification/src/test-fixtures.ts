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
import type { Logger, } from '@monochromatic-dev/logger-core/ts';

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

/**
 Messages a recording logger captured, in call order.
 */
type RecordedLogCalls = {
  /**
   Messages logged at debug level.
   */
  readonly debug: readonly string[];

  /**
   Messages logged at warn level.
   */
  readonly warn: readonly string[];
};

/**
 Logger that records its messages and the record it keeps appending to.
 */
type RecordingLoggerHarness = {
  /**
   Logger handed to the code under test.
   */
  readonly logger: Logger;

  /**
   Live view of the messages recorded so far.
   */
  readonly calls: RecordedLogCalls;
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

/**
 Discards a level this package's delivery decision never branches on.

 @param message - message the recording logger deliberately leaves unobserved

 @example
 ```ts
 discardMessage('ignored');
 ```
 */
function discardMessage(message: string,): void {
  void message;
}

/**
 Reports an immediately completed flush, because a recording logger buffers nothing.

 @example
 ```ts
 await flushNothing();
 ```
 */
function flushNothing(): Promise<void> {
  return Promise.resolve();
}

/**
 Creates a logger that records warn and debug messages instead of writing them.

 Delivery outcomes are the only levels this package decides on, so the fixture
 keeps them and discards the rest, which keeps a suite off the desktop sinks.

 @returns logger handed to the code under test plus the record it appends to

 @example
 ```ts
 const { logger, calls } = createRecordingLogger();
 ```
 */
function createRecordingLogger(): RecordingLoggerHarness {
  /**
   Messages logged at debug level, appended in call order.
   */
  const debug: string[] = [];

  /**
   Messages logged at warn level, appended in call order.
   */
  const warn: string[] = [];

  /**
   Logger whose recorded levels stay inspectable after the call returns.
   */
  const logger: Logger = {
    debug: function recordDebug(message: string,): void {
      debug.push(message,);
    },
    error: discardMessage,
    fatal: discardMessage,
    flush: flushNothing,
    info: discardMessage,
    trace: discardMessage,
    warn: function recordWarn(message: string,): void {
      warn.push(message,);
    },
  };

  return {
    logger,
    calls: {
      debug,
      warn,
    },
  };
}

//endregion Harness construction

export {
  createAgentSettledEvent,
  createExtensionContext,
  createFakePiHarness,
  createRecordingLogger,
  getOnlyHandler,
};

export type {
  AgentSettledHandler,
  FakePiHarness,
  RecordedLogCalls,
  RecordingLoggerHarness,
};
