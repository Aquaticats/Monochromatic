/**
 Test harness helpers for current-time-context pi extension checks.
 
 @module
 */

import {
  type BeforeAgentStartEvent,
  type BeforeAgentStartEventResult,
  createEventBus,
  type ExecResult,
  type ExtensionAPI,
  type ExtensionContext,
  type ExtensionHandler,
} from '@earendil-works/pi-coding-agent';

//region Types

/**
 Minimal handler signature for `before_agent_start` checks.
 */
type BeforeAgentStartHandler = ExtensionHandler<BeforeAgentStartEvent, BeforeAgentStartEventResult>;

/**
 Fake API harness recording registrations and captured handlers.
 */
type FakePiApiHarness = {
  /**
   Mock pi extension API.
   */
  api: ExtensionAPI;
  /**
   Registration calls observed through fake Pi API.
   */
  registrations: string[];
  /**
   Captured before-agent-start handlers.
   */
  handlers: BeforeAgentStartHandler[];
};

//endregion Types

//region Harness construction

/**
 Builds fake Pi API used to verify extension registration.
 
 @returns fake Pi API harness with captured registrations
 
 @example
 ```typescript
 const harness = fakePiApi();
 ```
 */
function fakePiApi(): FakePiApiHarness {
  /**
   Registration calls observed through fake Pi API.
   */
  const registrations: string[] = [];
  /**
   Captured before-agent-start handlers.
   */
  const handlers: BeforeAgentStartHandler[] = [];

  /**
   Mock pi API implementing the full surface with inert methods.
   */
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- Fake API records extension calls for tests; method implementations below cover the extension surface exercised here.
  const api = {
    on(
      event: string,
      handler: BeforeAgentStartHandler,
    ) {
      registrations.push(`event:${event}`,);
      if (event === 'before_agent_start')
        handlers.push(handler,);
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
    sendMessage(message: { readonly customType: string; },) {
      registrations.push(`message:${message.customType}`,);
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
  };
}

/**
 Retrieves captured `before_agent_start` handler from fake Pi harness.
 
 @param handlers - captured handlers from {@link fakePiApi}
 
 @returns first captured handler
 
 @throws when no handler is captured
 
 @example
 ```typescript
 const handler = getBeforeAgentStartHandler(harness.handlers);
 ```
 */
function getBeforeAgentStartHandler(
  handlers: readonly BeforeAgentStartHandler[],
): BeforeAgentStartHandler {
  /**
   First registered `before_agent_start` handler.
   */
  const [handler,] = handlers;
  if (handler === undefined)
    throw new Error('No handler registered for before_agent_start',);

  return handler;
}

/**
 Creates minimal before-agent-start event for handler invocation.
 
 @returns before-agent-start event with empty prompt metadata
 
 @example
 ```typescript
 const event = createBeforeAgentStartEvent();
 ```
 */
function createBeforeAgentStartEvent(): BeforeAgentStartEvent {
  return {
    type: 'before_agent_start',
    messageType: 'user',
    prompt: 'Reply with ok.',
    systemPrompt: '',
    // Pi 0.87 hands handlers collection-complete options; `normalizeBuildSystemPromptOptions` is not exported.
    systemPromptOptions: {
      cwd: process.cwd(),
      selectedTools: [],
      hiddenTools: [],
      toolSnippets: {},
      toolGuidelines: {},
      promptGuidelines: [],
      appendSystemPrompt: '',
      sections: {},
      contextFiles: [],
      skills: [],
    },
  } satisfies BeforeAgentStartEvent;
}

/**
 Creates minimal extension context for handler invocation.
 
 @returns extension context unused by current-time handler
 
 @example
 ```typescript
 const ctx = createExtensionContext();
 ```
 */
function createExtensionContext(): ExtensionContext {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- Mirrors @earendil-works/pi-coding-agent ExtensionContext at the test boundary; the current-time-context handler ignores ctx, so an empty stand-in suffices for registration verification.
  return {} as unknown as ExtensionContext;
}

//endregion Harness construction

export {
  createBeforeAgentStartEvent,
  createExtensionContext,
  fakePiApi,
  getBeforeAgentStartHandler,
};
export type {
  BeforeAgentStartHandler,
  FakePiApiHarness,
};
