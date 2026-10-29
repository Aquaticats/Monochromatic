/**
 Tests for pi extension entry point.
 */

import type {
  AgentEndEvent,
  BeforeAgentStartEvent,
  ExtensionAPI,
  SessionShutdownEvent,
  SessionStartEvent,
} from '@earendil-works/pi-coding-agent';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  MAX_TERMINAL_TITLE_UTF8_BYTES,
  terminalTitleUtf8ByteLength,
} from '@monochromatic-dev/agent-harness-shared-terminal-title/ts';

//region Mock infrastructure

/** Minimal handler signature matching pi event handlers. */
type HandlerFn = (...args: unknown[]) => unknown;

/** Shape of the mock registration map. */
type RegistrationMap = Map<string, HandlerFn[]>;

/**
 Creates a mock ExtensionAPI that records all `on()` registrations.
 
 @returns mock API and registration map
 */
function createMockApi(): {
  readonly api: ExtensionAPI;
  readonly registrations: RegistrationMap;
} {
  /**
   Event handlers registered by extension.
   */
  const registrations: RegistrationMap = new Map();
  /**
   Minimal extension API mock.
   */
  const api = {
    on(
      event: string,
      handler: HandlerFn,
    ) {
      /**
       Existing handlers for event.
       */
      const existing = registrations.get(event,) ?? [];
      existing.push(handler,);
      registrations.set(event, existing,);
    },
  } as unknown as ExtensionAPI;

  return {
    api,
    registrations,
  };
}

/**
 Creates a mock context with setTitle spy.
 
 @returns mock context and captured titles
 */
function createMockContext(): {
  readonly ctx: { readonly ui: { readonly setTitle: (title: string,) => void } };
  readonly titles: string[];
} {
  /**
   Titles captured from setTitle calls.
   */
  const titles: string[] = [];
  return {
    ctx: {
      ui: {
        setTitle(title: string,): void {
          titles.push(title,);
        },
      },
    },
    titles,
  };
}

/**
 Retrieves registered handler for event.
 
 @param registrations - registration map from mock API
 
 @param event - event name to look up
 
 @returns registered handler
 */
function getHandler(
  {
    registrations,
    event,
  }: Readonly<{
    registrations: RegistrationMap;
    event: string;
  }>,
): HandlerFn {
  /**
   Handlers registered for event.
   */
  const handlers = registrations.get(event,);
  if ((handlers === undefined) || (handlers.length === 0))
    throw new Error(`No handler registered for event: ${event}`,);
  /**
   First registered handler.
   */
  const [handler,] = handlers;
  if (handler === undefined)
    throw new Error(`No handler registered for event: ${event}`,);
  return handler;
}

//endregion Mock infrastructure

/**
 Extension default export loaded dynamically for test isolation.
 */
const { default: terminalTitle, } = await import('./index.ts');

await describe({
  name: terminalTitle.name,
  children: [
    it({
      name: 'registers all handled event handlers',
      fn: async () => {
        const { api, registrations, } = createMockApi();
        terminalTitle(api,);
        /**
         Events expected from extension entry point.
         */
        const expectedEvents = [
          'tool_execution_start',
          'tool_execution_end',
          'session_start',
          'session_shutdown',
          'agent_end',
          'before_agent_start',
        ];
        for (const eventName of expectedEvents)
          expect(registrations.get(eventName,),).toHaveLength(1,);
      },
    },),
    it({
      name: 'sets lifecycle command title on tool start',
      fn: async () => {
        const { api, registrations, } = createMockApi();
        terminalTitle(api,);
        const { ctx, titles, } = createMockContext();
        getHandler({ registrations, event: 'tool_execution_start', },)(
          { toolCallId: 'call-1', toolName: 'bash', args: { command: 'npm test', }, },
          ctx,
        );
        expect(titles,).toStrictEqual(['π Running npm test',],);
      },
    },),
    it({
      name: 'reuses start args on tool end',
      fn: async () => {
        const { api, registrations, } = createMockApi();
        terminalTitle(api,);
        const { ctx, titles, } = createMockContext();
        getHandler({ registrations, event: 'tool_execution_start', },)(
          { toolCallId: 'call-1', toolName: 'bash', args: { command: 'ls -l', }, },
          ctx,
        );
        getHandler({ registrations, event: 'tool_execution_end', },)(
          { toolCallId: 'call-1', toolName: 'bash', result: {}, isError: false, },
          ctx,
        );
        expect(titles,).toStrictEqual([
          'π Running ls -l',
          'π Ran ls -l',
        ],);
      },
    },),
    it({
      name: 'clears cached args on session shutdown',
      fn: async () => {
        const { api, registrations, } = createMockApi();
        terminalTitle(api,);
        const { ctx, titles, } = createMockContext();
        getHandler({ registrations, event: 'tool_execution_start', },)(
          { toolCallId: 'call-1', toolName: 'bash', args: { command: 'ls -l', }, },
          ctx,
        );
        getHandler({ registrations, event: 'session_shutdown', },)(
          { type: 'session_shutdown', reason: 'quit', } as SessionShutdownEvent,
          ctx,
        );
        getHandler({ registrations, event: 'tool_execution_end', },)(
          { toolCallId: 'call-1', toolName: 'bash', result: {}, isError: false, },
          ctx,
        );
        expect(titles,).toStrictEqual([
          'π Running ls -l',
          'π Ended session',
          'π Ran command',
        ],);
      },
    },),
    it({
      name: 'sets lifecycle session and agent titles',
      fn: async () => {
        const { api, registrations, } = createMockApi();
        terminalTitle(api,);
        const { ctx, titles, } = createMockContext();
        getHandler({ registrations, event: 'session_start', },)(
          { type: 'session_start', reason: 'startup', } as SessionStartEvent,
          ctx,
        );
        getHandler({ registrations, event: 'agent_end', },)(
          { type: 'agent_end', messages: [], } as AgentEndEvent,
          ctx,
        );
        expect(titles,).toStrictEqual([
          'π Started session: startup',
          'π Stopped agent',
        ],);
      },
    },),
    it({
      name: 'sanitizes control characters in prompt title',
      fn: async () => {
        const { api, registrations, } = createMockApi();
        terminalTitle(api,);
        const { ctx, titles, } = createMockContext();
        getHandler({ registrations, event: 'before_agent_start', },)(
          {
            type: 'before_agent_start',
            messageType: 'user',
            prompt: 'Fix\u001Bauth\u0007bug',
            systemPrompt: '',
            systemPromptOptions: {} as never,
          } as BeforeAgentStartEvent,
          ctx,
        );
        expect(titles,).toStrictEqual(['π Received prompt: Fix␛auth␇bug',],);
      },
    },),
    it({
      name: 'byte-caps emitted prompt titles',
      fn: async () => {
        const { api, registrations, } = createMockApi();
        terminalTitle(api,);
        const { ctx, titles, } = createMockContext();
        getHandler({ registrations, event: 'before_agent_start', },)(
          {
            type: 'before_agent_start',
            messageType: 'user',
            prompt: '😀'.repeat(MAX_TERMINAL_TITLE_UTF8_BYTES,),
            systemPrompt: '',
            systemPromptOptions: {} as never,
          } as BeforeAgentStartEvent,
          ctx,
        );
        expect(terminalTitleUtf8ByteLength(titles[0] ?? '',) <= MAX_TERMINAL_TITLE_UTF8_BYTES,)
          .toBe(true,);
      },
    },),
    it({
      name: 'skips the prompt title for custom-triggered runs',
      fn: async function testCustomTriggerSkipsTitle() {
        const { api, registrations, } = createMockApi();
        terminalTitle(api,);
        const { ctx, titles, } = createMockContext();
        getHandler({ registrations, event: 'before_agent_start', },)(
          {
            type: 'before_agent_start',
            messageType: 'custom',
            message: {
              role: 'custom',
              customType: 'advisor-kickoff',
              content: 'start review',
              display: false,
              timestamp: 0,
            },
            systemPrompt: '',
            systemPromptOptions: {} as never,
          } as BeforeAgentStartEvent,
          ctx,
        );
        expect(titles,).toStrictEqual([],);
      },
    },),
  ],
},);
