/**
 Pi lifecycle fixtures shared by the settled-notification unit tests and verifiers.

 Every event here carries the fields Pi's own `AgentSettledEvent` requires, and
 every handler invocation receives a handler context, so no suite can pass
 against an event shape Pi never delivers.

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

  /**
   Messages logged at error level.
   */
  readonly error: readonly string[];
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

  /* oxlint-disable typescript/no-unsafe-type-assertion -- Fake API records lifecycle registrations; only `on()` is exercised here. */
  /**
   Fake Pi API implementing only the event-registration surface under test.
   */
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
  /* oxlint-enable typescript/no-unsafe-type-assertion */

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

 Pi's `ExtensionContext` has no complete stand-in short of a live Pi runtime, so
 this returns an object carrying only the fields the settled handler reads,
 asserted to the contract. That stays safe only while the handler reads nothing
 else from the context, which the extension does not.

 @param cwd - working folder Pi reports for the settled agent

 @returns extension context stub carrying the agent's working folder

 @example
 ```ts
 createExtensionContext({ cwd: '/work/agent-7', });
 ```
 */
function createExtensionContext(
  {
    cwd = '/work/agent-7',
  }: {
    readonly cwd?: string;
  } = {},
): ExtensionContext {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- Mirrors Pi's ExtensionContext at the fixture boundary; the settled handler reads only the working folder, so a stand-in carrying it keeps the handler's real call shape.
  return { cwd, } as unknown as ExtensionContext;
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
 Creates a logger that records debug, warn and error messages instead of writing them.

 Delivery outcomes and defects are the only levels this package decides on, so the
 fixture keeps them and discards the rest, which keeps a suite off the desktop sinks.

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
   Messages logged at error level, appended in call order.
   */
  const error: string[] = [];

  /**
   Logger whose recorded levels stay inspectable after the call returns.
   */
  const logger: Logger = {
    debug: function recordDebug(message: string,): void {
      debug.push(message,);
    },
    error: function recordError(message: string,): void {
      error.push(message,);
    },
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
      error,
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
