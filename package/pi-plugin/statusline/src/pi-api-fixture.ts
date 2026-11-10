/**
 Test harness helpers for pi-statusline extension registration checks.
 
 @module
 */

import type {
  ExtensionAPI,
  ExtensionContext,
  ExtensionHandler,
  SessionShutdownEvent,
  SessionStartEvent,
} from '@earendil-works/pi-coding-agent';

//region Types

/**
 Minimal after-provider-response event shape used by the fake harness.
 */
type AfterProviderResponseEvent = {
  /**
   Pi event discriminant.
   */
  readonly type: 'after_provider_response';
  /**
   HTTP status code reported by provider transport.
   */
  readonly status: number;
  /**
   Provider response headers.
   */
  readonly headers: Record<string, string>;
};

/**
 Captured `after_provider_response` handler signature.
 */
type AfterProviderResponseHandler = ExtensionHandler<AfterProviderResponseEvent>;

/**
 Captured `session_start` handler signature.
 */
type SessionStartHandler = ExtensionHandler<SessionStartEvent>;

/**
 Captured `session_shutdown` handler signature.
 */
type SessionShutdownHandler = ExtensionHandler<SessionShutdownEvent>;

/**
 Fake Pi API harness with captured event handlers.
 */
type FakePiApiHarness = {
  /**
   Mock Pi extension API.
   */
  readonly api: ExtensionAPI;
  /**
   Observed registration calls.
   */
  readonly registrations: readonly string[];
  /**
   Captured provider response handlers.
   */
  readonly afterProviderResponseHandlers: readonly AfterProviderResponseHandler[];
  /**
   Captured session-start handlers.
   */
  readonly sessionStartHandlers: readonly SessionStartHandler[];
  /**
   Captured session-shutdown handlers.
   */
  readonly sessionShutdownHandlers: readonly SessionShutdownHandler[];
};

/**
 Fake extension context harness with status writes.
 */
type FakeExtensionContextHarness = {
  /**
   Mock Pi extension context.
   */
  readonly ctx: ExtensionContext;
  /**
   Status values keyed by extension status key.
   */
  readonly statuses: Map<string, string>;
};

//endregion Types

//region Fake Pi API

/**
 Builds fake Pi API capturing event registrations used by this extension.
 
 @returns {@link FakePiApiHarness}
 
 @example
 ```ts
 const harness = fakePiApi();
 ```
 */
function fakePiApi(): FakePiApiHarness {
  /**
   Registration call log.
   */
  const registrations: string[] = [];
  /**
   Provider response handlers captured through `pi.on`.
   */
  const afterProviderResponseHandlers: AfterProviderResponseHandler[] = [];
  /**
   Session-start handlers captured through `pi.on`.
   */
  const sessionStartHandlers: SessionStartHandler[] = [];
  /**
   Session-shutdown handlers captured through `pi.on`.
   */
  const sessionShutdownHandlers: SessionShutdownHandler[] = [];

  /**
   Minimal fake API. The extension only calls `on`, so other API methods are not needed for this harness.
   */
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- Test double intentionally implements only the API surface exercised by pi-statusline.
  const api = {
    on(
      event: string,
      handler: AfterProviderResponseHandler | SessionStartHandler | SessionShutdownHandler,
    ): void {
      registrations.push(`event:${event}`,);
      if (event === 'after_provider_response') {
        // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- Event branch determines handler shape in this fake `on` implementation.
        afterProviderResponseHandlers.push(handler as AfterProviderResponseHandler,);
        return;
      }
      if (event === 'session_start') {
        // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- Event branch determines handler shape in this fake `on` implementation.
        sessionStartHandlers.push(handler as SessionStartHandler,);
        return;
      }
      if (event === 'session_shutdown')
        // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- Event branch determines handler shape in this fake `on` implementation.
        sessionShutdownHandlers.push(handler as SessionShutdownHandler,);
    },
  } as unknown as ExtensionAPI;

  return {
    api,
    registrations,
    afterProviderResponseHandlers,
    sessionStartHandlers,
    sessionShutdownHandlers,
  };
}

/**
 Retrieves first provider response handler from fake Pi harness.
 
 @param handlers - captured handlers
 
 @returns first captured handler
 
 @throws when no handler is captured
 
 @example
 ```ts
 const handler = getAfterProviderResponseHandler(harness.afterProviderResponseHandlers);
 ```
 */
function getAfterProviderResponseHandler(
  handlers: readonly AfterProviderResponseHandler[],
): AfterProviderResponseHandler {
  /**
   First captured handler.
   */
  const [handler,] = handlers;
  if (handler === undefined)
    throw new Error('No handler registered for after_provider_response',);

  return handler;
}

/**
 Retrieves first session-start handler from fake Pi harness.
 
 @param handlers - captured handlers
 
 @returns first captured handler
 
 @throws when no handler is captured
 
 @example
 ```ts
 const handler = getSessionStartHandler(harness.sessionStartHandlers);
 ```
 */
function getSessionStartHandler(
  handlers: readonly SessionStartHandler[],
): SessionStartHandler {
  /**
   First captured handler.
   */
  const [handler,] = handlers;
  if (handler === undefined)
    throw new Error('No handler registered for session_start',);

  return handler;
}

//endregion Fake Pi API

//region Events and context

/**
 Creates provider response event with supplied headers.
 
 @param headers - provider response headers
 
 @returns after-provider-response event
 
 @example
 ```ts
 createAfterProviderResponseEvent({});
 ```
 */
function createAfterProviderResponseEvent(
  headers: Readonly<Record<string, string>>,
): AfterProviderResponseEvent {
  return {
    type: 'after_provider_response',
    status: 200,
    headers: {
      ...headers,
    },
  } satisfies AfterProviderResponseEvent;
}

/**
 Creates session-start event.
 
 @returns session-start event
 
 @example
 ```ts
 createSessionStartEvent();
 ```
 */
function createSessionStartEvent(): SessionStartEvent {
  return {
    type: 'session_start',
    reason: 'startup',
  } satisfies SessionStartEvent;
}

/**
 Creates fake extension context with status capture.
 
 @param hasUI - whether context should report UI availability
 
 @returns {@link FakeExtensionContextHarness}
 
 @example
 ```ts
 const { ctx, statuses } = createExtensionContext();
 ```
 */
function createExtensionContext(hasUI = true,): FakeExtensionContextHarness {
  /**
   Captured footer status writes.
   */
  const statuses = new Map<string, string>();
  /**
   Minimal extension context with UI methods used by pi-statusline.
   */
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- Test double provides the context properties used by pi-statusline and intentionally omits unrelated Pi APIs.
  const ctx = {
    hasUI,
    ui: {
      setStatus(
        key: string,
        value?: string,
      ): void {
        if (value === undefined) {
          statuses.delete(key,);
          return;
        }

        statuses.set(
          key,
          value,
        );
      },
      theme: {
        fg(
          color: string,
          text: string,
        ): string {
          return `<${color}>${text}</${color}>`;
        },
      },
    },
  } as unknown as ExtensionContext;

  return {
    ctx,
    statuses,
  };
}

//endregion Events and context

export {
  createAfterProviderResponseEvent,
  createExtensionContext,
  createSessionStartEvent,
  fakePiApi,
  getAfterProviderResponseHandler,
  getSessionStartHandler,
};
export type {
  AfterProviderResponseHandler,
  FakeExtensionContextHarness,
  FakePiApiHarness,
  SessionStartHandler,
};
