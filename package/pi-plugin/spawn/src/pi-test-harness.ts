/**
 Test harness helpers for spawn-pi extension tests.
 
 @module
 */

import {
  createEventBus,
  type ExecResult,
  type ExtensionAPI,
  type ExtensionContext,
  type ExtensionHandler,
  type SessionStartEvent,
  type AgentEndEvent,
  type SessionShutdownEvent,
} from '@earendil-works/pi-coding-agent';

//region Handler types

/**
 Captured session-start handler type.
 */
type SessionStartHandler = ExtensionHandler<SessionStartEvent>;

/**
 Captured session-shutdown handler type.
 */
type SessionShutdownHandler = ExtensionHandler<SessionShutdownEvent>;

/**
 Captured agent-end handler type.
 */
type AgentEndHandler = ExtensionHandler<AgentEndEvent>;


/**
 Message sent through fake Pi API.
 */
type SentMessageRecord = {
  /**
   Custom message payload passed to `sendMessage`.
   */
  readonly message: {
    readonly customType: string;
    readonly content: unknown;
    readonly display: boolean;
  };
  /**
   Delivery options passed to `sendMessage`.
   */
  readonly options: unknown;
};

/**
 Notification sent through fake extension UI.
 */
type NotificationRecord = {
  /**
   Notification text.
   */
  readonly message: string;
  /**
   Notification severity.
   */
  readonly level: string;
};

/**
 Captured event handlers by event name.
 */
type CapturedHandlers = {
  /**
   Registered session-start handlers.
   */
  readonly sessionStart: SessionStartHandler[];
  /**
   Registered session-shutdown handlers.
   */
  readonly sessionShutdown: SessionShutdownHandler[];
  /**
   Registered agent-end handlers.
   */
  readonly agentEnd: AgentEndHandler[];
};

/**
 Fake API harness for spawn-pi extension tests.
 */
type FakePiApiHarness = {
  /**
   Mock Pi extension API.
   */
  readonly api: ExtensionAPI;
  /**
   Registration calls observed through fake Pi API.
   */
  readonly registrations: string[];
  /**
   Captured handlers grouped by event name.
   */
  readonly handlers: CapturedHandlers;
  /**
   Messages queued through fake Pi API.
   */
  readonly sentMessages: SentMessageRecord[];
};

/**
 Options for fake extension context creation.
 */
type ExtensionContextOptions = {
  /**
   Pi session id exposed by fake SessionManager.
   */
  readonly sessionId: string;
  /**
   Optional Pi session file exposed by fake SessionManager.
   */
  readonly sessionFile?: string;
  /**
   Working directory exposed through context.
   */
  readonly cwd?: string;
  /**
   Whether fake context should expose interactive UI.
   */
  readonly hasUI?: boolean;
  /**
   Notifications captured from fake UI.
   */
  readonly notifications?: readonly NotificationRecord[];
};

//endregion Handler types

//region Fake API

/**
 Builds fake Pi API and captures extension registrations.
 
 @returns fake {@link FakePiApiHarness}.
 
 @example
 ```typescript
 const harness = fakePiApi();
 ```
 */
function fakePiApi(): FakePiApiHarness {
  /**
   Registration calls observed through fake API.
   */
  const registrations: string[] = [];
  /**
   Messages queued through fake API.
   */
  const sentMessages: SentMessageRecord[] = [];
  /**
   Event handlers captured by fake API.
   */
  const handlers: CapturedHandlers = {
    sessionStart: [],
    sessionShutdown: [],
    agentEnd: [],
  };

  /**
   Mock Pi API implementing surface used by spawn-pi.
   */
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- fake API records extension interactions; unexercised methods are inert stand-ins.
  const api = {
    on(
      event: string,
      handler: ExtensionHandler<unknown, unknown>,
    ) {
      registrations.push(`event:${event}`,);
      if (event === 'session_start') {
        handlers
          .sessionStart
          .push(
            /**
             Invokes captured extension callback for fake session start.
             
             @param sessionStartEvent - Caller-provided fake host event.
             
             @param ctx - Caller-provided fake extension context.
             
             @mutates sessionStartEvent - `handler` can retain or change supplied event state.
             
             @mutates ctx - `handler` can invoke or change supplied context capabilities.
             */
          async function callSessionStartHandler(
            sessionStartEvent,
            ctx,
          ): Promise<void> {
            await handler(
              sessionStartEvent,
              ctx,
            );
          },
          );
      }
      if (event === 'session_shutdown') {
        handlers
          .sessionShutdown
          .push(
            /**
             Invokes captured extension callback for fake session shutdown.
             
             @param sessionShutdownEvent - Caller-provided fake host event.
             
             @param ctx - Caller-provided fake extension context.
             
             @mutates sessionShutdownEvent - `handler` can retain or change supplied event state.
             
             @mutates ctx - `handler` can invoke or change supplied context capabilities.
             */
          async function callSessionShutdownHandler(
            sessionShutdownEvent,
            ctx,
          ): Promise<void> {
            await handler(
              sessionShutdownEvent,
              ctx,
            );
          },
          );
      }
      if (event === 'agent_end') {
        handlers
          .agentEnd
          .push(
            /**
             Invokes captured extension callback for fake agent completion.
             
             @param agentEndEvent - Caller-provided fake host event.
             
             @param ctx - Caller-provided fake extension context.
             
             @mutates agentEndEvent - `handler` can retain or change supplied event state.
             
             @mutates ctx - `handler` can invoke or change supplied context capabilities.
             */
          async function callAgentEndHandler(
            agentEndEvent,
            ctx,
          ): Promise<void> {
            await handler(
              agentEndEvent,
              ctx,
            );
          },
          );
      }
    },
    registerTool(tool: { readonly name: string; },) {
      registrations.push(`tool:${tool.name}`,);
    },
    registerCommand(name: string,) {
      registrations.push(`command:${name}`,);
    },
    registerShortcut(shortcut: string,) {
      registrations.push(`shortcut:${shortcut}`,);
    },
    registerFlag(name: string,) {
      registrations.push(`flag:${name}`,);
    },
    getFlag() {
      return undefined;
    },
    registerMessageRenderer(customType: string,) {
      registrations.push(`renderer:${customType}`,);
    },
    sendMessage(
      message: SentMessageRecord['message'],
      options: unknown,
    ) {
      sentMessages.push({
        message,
        options,
      },);
    },
    sendUserMessage(content: unknown,) {
      void content;
    },
    appendEntry(customType: string,) {
      registrations.push(`entry:${customType}`,);
    },
    setSessionName(name: string,) {
      void name;
    },
    getSessionName() {
      return undefined;
    },
    setLabel(
      entryId: string,
      label?: string,
    ) {
      void entryId;
      void label;
    },
    exec(
      command: string,
      args: readonly string[],
    ): Promise<ExecResult> {
      void command;
      void args;
      return Promise.resolve({
        stdout: '',
        stderr: '',
        code: 0,
        killed: false,
      },);
    },
    getActiveTools() {
      return [];
    },
    getAllTools() {
      return [];
    },
    setActiveTools(toolNames: readonly string[],) {
      void toolNames;
    },
    getCommands() {
      return [];
    },
    setModel(model: unknown,) {
      void model;
      return Promise.resolve(false,);
    },
    getThinkingLevel() {
      return 'off';
    },
    setThinkingLevel(level: string,) {
      void level;
    },
    registerProvider(name: string,) {
      registrations.push(`provider:${name}`,);
    },
    unregisterProvider(name: string,) {
      registrations.push(`unprovider:${name}`,);
    },
    events: createEventBus(),
  } as unknown as ExtensionAPI;

  return {
    api,
    registrations,
    handlers,
    sentMessages,
  };
}

//endregion Fake API

//region Fake context and events

/**
 Creates minimal extension context for spawn-pi handler tests.
 
 @param options - fake {@link ExtensionContextOptions}.
 
 @returns fake extension context.
 
 @example
 ```typescript
 createExtensionContext({ sessionId: 'parent' });
 ```
 */
function createExtensionContext(options: Readonly<ExtensionContextOptions>,): ExtensionContext {
  /**
   Notification records shared with fake UI.
   */
  const notifications: NotificationRecord[] = [
    ...(options.notifications ?? []),
  ];

  /**
   Minimal context implementing members used by spawn-pi.
   */
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- fake context covers only members used by spawn-pi handlers.
  const ctx = {
    cwd: options.cwd
      ?? process.cwd(),
    hasUI: options.hasUI
      ?? false,
    sessionManager: {
      getSessionId() {
        return options.sessionId;
      },
      getSessionFile() {
        return options.sessionFile;
      },
    },
    ui: {
      notify(
        message: string,
        level: string,
      ) {
        notifications.push({
          message,
          level,
        },);
      },
    },
  } as unknown as ExtensionContext;

  return ctx;
}

/**
 Creates minimal session-start event.
 
 @returns fake session-start event.
 
 @example
 ```typescript
 createSessionStartEvent();
 ```
 */
function createSessionStartEvent(): SessionStartEvent {
  return {
    type: 'session_start',
    reason: 'startup',
  };
}

/**
 Creates minimal session-shutdown event.
 
 @returns fake session-shutdown event.
 
 @example
 ```typescript
 createSessionShutdownEvent();
 ```
 */
function createSessionShutdownEvent(): SessionShutdownEvent {
  return {
    type: 'session_shutdown',
    reason: 'quit',
  };
}

/**
 Creates minimal agent-end event.
 
 @param message - assistant text emitted by fake event.
 
 @returns fake agent-end event.
 
 @example
 ```typescript
 createAgentEndEvent('done');
 ```
 */
function createAgentEndEvent(message: string,): AgentEndEvent {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- test fixture matches AgentEndEvent fields consumed by spawn-pi.
  return {
    type: 'agent_end',
    messages: [
      {
        role: 'assistant',
        content: [
          {
            type: 'text',
            text: message,
          },
        ],
      },
    ],
  } as AgentEndEvent;
}

//endregion Fake context and events

//region Handler accessors

/**
 Retrieves only registered handler from an array.
 
 @param handlers - captured handler array.
 
 @returns first handler.
 
 @throws when no handler was registered.
 
 @example
 ```typescript
 onlyHandler(harness.handlers.sessionStart);
 ```
 */
function onlyHandler<const THandler>(handlers: readonly THandler[],): THandler {
  /**
   First captured handler.
   */
  const [handler,] = handlers;
  if (handler === undefined)
    throw new Error('No handler registered',);
  return handler;
}

//endregion Handler accessors

export {
  createAgentEndEvent,
  createExtensionContext,
  createSessionStartEvent,
  createSessionShutdownEvent,
  fakePiApi,
  onlyHandler,
};

export type {
  CapturedHandlers,
  FakePiApiHarness,
  NotificationRecord,
  SentMessageRecord,
};
