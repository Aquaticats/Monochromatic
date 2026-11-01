/**
 Tests for pi guardrail extension registration and tool-call dispatch.
 
 @module
 */

import type {
  ExtensionAPI,
  ExtensionContext,
  ToolCallEvent,
} from '@earendil-works/pi-coding-agent';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { createPathGuardMatcher, } from './path-guard.ts';
import {
  evaluateToolCall,
  registerGuardrail,
} from './index.ts';
import {
  GUARDRAIL_NOT_BLOCKED,
  type GuardrailConfig,
} from './types.ts';

/**
 Minimal event handler signature captured by the mock API.
 */
type HandlerFn = (event: ToolCallEvent, ctx: ExtensionContext,) => unknown;

/**
 Mock API harness with captured event handlers.
 */
type MockApiHarness = {
  /**
   Mock pi extension API.
   */
  readonly api: ExtensionAPI;
  /**
   Registered handlers by event name.
   */
  readonly handlers: Map<string, HandlerFn[]>;
};

/**
 Creates default guardrail config for tests.
 
 @returns test guardrail config
 */
function testConfig(): GuardrailConfig {
  return {
    pathRules: [{
      pattern: 'pnpm-lock.yaml',
      message: 'run pnpm install',
    },],
    blockBunTest: true,
    source: {
      path: '/tmp/pi-guardrail.json',
      loaded: false,
    },
  };
}

/**
 Creates minimal pi ExtensionAPI mock.
 
 @returns mock API harness
 */
function createMockApi(): MockApiHarness {
  /**
   Registered handlers by event name.
   */
  const handlers = new Map<string, HandlerFn[]>();
  /**
   Mock extension API recording event handlers.
   */
  const api = {
    on(event: string, handler: HandlerFn,) {
      handlers.set(
        event,
        [
          ...(handlers.get(event,) ?? []),
          handler,
        ],
      );
    },
  } as unknown as ExtensionAPI;
  return {
    api,
    handlers,
  };
}

/**
 Retrieves first handler for an event.
 
 @param handlers - handler map
 
 @param event - event name
 
 @returns registered handler
 
 @throws when handler is missing
 */
function getHandler(
  {
    handlers,
    event,
  }: {
    readonly handlers: ReadonlyMap<string, readonly HandlerFn[]>;
    readonly event: string;
  },
): HandlerFn {
  /**
   Event handlers recorded for requested event.
   */
  const eventHandlers = handlers.get(event,);
  if ((eventHandlers === undefined) || (eventHandlers[0] === undefined))
    throw new Error(`Missing handler for ${event}`,);
  return eventHandlers[0];
}

/**
 Builds a minimal tool-call event.
 
 @param toolName - pi tool name
 
 @param input - tool input
 
 @returns tool-call event
 */
function toolEvent(
  {
    toolName,
    input,
  }: {
    readonly toolName: string;
    readonly input: unknown;
  },
): ToolCallEvent {
  return {
    type: 'tool_call',
    toolName,
    toolCallId: `${toolName}-1`,
    input,
  } as ToolCallEvent;
}

/**
 Creates a minimal extension context.
 
 @param notifications - notification message sink
 
 @returns extension context
 */
function context(notifications: string[] = [],): ExtensionContext {
  return {
    cwd: '/repo',
    hasUI: true,
    ui: {
      notify(message: string,) {
        notifications.push(message,);
      },
    },
  } as unknown as ExtensionContext;
}

await describe({
  name: 'pi guardrail extension',
  children: [
    it({
      name: 'registers tool_call handler after loading config',
      fn: async function testRegistration() {
        const { api, handlers, } = createMockApi();
        await registerGuardrail({
          pi: api,
          loadConfig: async function loadConfig(): Promise<GuardrailConfig> {
            return testConfig();
          },
        },);
        expect(handlers.get('tool_call',),).toHaveLength(1,);
      },
    },),
    it({
      name: 'blocks edit and write for matching protected paths',
      fn: async function testBlocksFileMutations() {
        const config = testConfig();
        const matcher = createPathGuardMatcher(config.pathRules,);
        expect(evaluateToolCall({
          event: toolEvent({ toolName: 'edit', input: { path: 'pnpm-lock.yaml', }, },),
          ctx: context(),
          config,
          pathMatcher: matcher,
        },),).toEqual({
          block: true,
          reason: 'run pnpm install',
        },);
        expect(evaluateToolCall({
          event: toolEvent({ toolName: 'write', input: { path: 'package/a/pnpm-lock.yaml', }, },),
          ctx: context(),
          config,
          pathMatcher: matcher,
        },),).toEqual({
          block: true,
          reason: 'run pnpm install',
        },);
      },
    },),
    it({
      name: 'blocks bash bun test only when config enables bash guard',
      fn: async function testBashGuardToggle() {
        const config = testConfig();
        const matcher = createPathGuardMatcher(config.pathRules,);
        const decision = evaluateToolCall({
          event: toolEvent({ toolName: 'bash', input: { command: 'bun test', }, },),
          ctx: context(),
          config,
          pathMatcher: matcher,
        },);
        if (decision === GUARDRAIL_NOT_BLOCKED)
          throw new Error('Expected bun test tool call to be blocked',);
        expect(decision.block,).toBe(true,);
        expect(decision.reason.includes('Blocked: `bun test` invocations are banned',),).toBe(true,);
        expect(evaluateToolCall({
          event: toolEvent({ toolName: 'bash', input: { command: 'bun test', }, },),
          ctx: context(),
          config: {
            ...config,
            blockBunTest: false,
          },
          pathMatcher: matcher,
        },),).toBe(GUARDRAIL_NOT_BLOCKED,);
      },
    },),
    it({
      name: 'allows unrelated tools and non-matching paths',
      fn: async function testAllowsUnrelatedTools() {
        const config = testConfig();
        const matcher = createPathGuardMatcher(config.pathRules,);
        expect(evaluateToolCall({
          event: toolEvent({ toolName: 'read', input: { path: 'pnpm-lock.yaml', }, },),
          ctx: context(),
          config,
          pathMatcher: matcher,
        },),).toBe(GUARDRAIL_NOT_BLOCKED,);
        expect(evaluateToolCall({
          event: toolEvent({ toolName: 'edit', input: { path: 'package.json', }, },),
          ctx: context(),
          config,
          pathMatcher: matcher,
        },),).toBe(GUARDRAIL_NOT_BLOCKED,);
      },
    },),
    it({
      name: 'tool_call handler returns block decision and notifies UI',
      fn: async function testHandlerNotification() {
        const { api, handlers, } = createMockApi();
        const notifications: string[] = [];
        await registerGuardrail({
          pi: api,
          loadConfig: async function loadConfig(): Promise<GuardrailConfig> {
            return testConfig();
          },
        },);
        const handler = getHandler({ handlers, event: 'tool_call', },);
        const result = handler(
          toolEvent({ toolName: 'edit', input: { path: 'pnpm-lock.yaml', }, },),
          context(notifications,),
        );
        expect(result,).toEqual({
          block: true,
          reason: 'run pnpm install',
        },);
        expect(notifications,).toEqual(['run pnpm install',],);
      },
    },),
  ],
},);
