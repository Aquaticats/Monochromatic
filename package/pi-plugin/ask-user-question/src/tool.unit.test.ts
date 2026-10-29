import type {
  AgentToolResult,
  ExtensionAPI,
  ExtensionContext,
  Theme,
} from '@earendil-works/pi-coding-agent';
import type { Component, } from '@earendil-works/pi-tui';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import askUserQuestionExtension, {
  ASK_USER_QUESTION_TOOL_NAME,
  AskUserQuestionUnavailableError,
  type AskUserQuestionDetails,
  registerAskUserQuestionExtension,
} from '../dist/final/node/index.mjs';

//region Harness types

/**
 Runtime tool surface exercised through fake Pi host.
 */
type RegisteredTool = {
  readonly name: string;
  readonly executionMode?: string;
  readonly parameters: unknown;
  readonly execute: (
    toolCallId: string,
    params: Readonly<{ question: string; }>,
    signal: AbortSignal,
    onUpdate: unknown,
    ctx: ExtensionContext,
  ) => Promise<AgentToolResult<AskUserQuestionDetails>>;
  readonly renderCall: (
    args: Readonly<{ question: string; }>,
    theme: Theme,
    context: unknown,
  ) => Component;
};

/**
 Captured Pi registrations.
 */
type PiHarness = {
  readonly api: ExtensionAPI;
  readonly state: {
    tool?: RegisteredTool;
    shutdown?: () => void;
  };
};

//endregion Harness types

//region Harness

/**
 Creates fake Pi API for tool and shutdown registration.
 
 @returns fake host and captured registration state
 */
function createPiHarness(): PiHarness {
  /**
   Mutable captured registration slots.
   */
  const state: PiHarness['state'] = {};
  /**
   Minimal fake implementation narrowed through runtime checks.
   */
  const fakeApi = {
    registerTool(tool: unknown): void {
      if (!isRegisteredTool(tool,))
        throw new Error('Extension registered unexpected tool shape.',);
      state.tool = tool;
    },
    on(event: string, handler: unknown): void {
      if (event !== 'session_shutdown')
        throw new Error(`Extension registered unexpected event: ${event}`,);
      if ((typeof handler) !== 'function')
        throw new Error('Session shutdown handler must be a function.',);
      state.shutdown = handler as () => void;
    },
  };
  return {
    api: fakeApi as unknown as ExtensionAPI,
    state,
  };
}

/**
 Narrows unknown registration value to test tool surface.
 
 @param value - registration candidate
 
 @returns whether required tool functions and fields exist
 */
function isRegisteredTool(value: unknown,): value is RegisteredTool {
  if (value === null)
    return false;
  if ((typeof value) !== 'object')
    return false;
  return ('name' in value)
    && ('parameters' in value)
    && ('execute' in value)
    && ((typeof value.execute) === 'function')
    && ('renderCall' in value)
    && ((typeof value.renderCall) === 'function');
}

/**
 Retrieves registered tool or fails test with plain diagnostic.
 
 @param harness - fake Pi registration state
 
 @returns registered ask-user tool
 */
function registeredTool(harness: PiHarness,): RegisteredTool {
  /**
   Optional captured tool after extension registration.
   */
  const { tool, } = harness.state;
  if (tool === undefined)
    throw new Error('Expected ask-user tool registration.',);
  return tool;
}

/**
 Creates minimal external Pi context for execute callback.
 
 @param mode - host run mode
 
 @returns fake context with mode and cwd
 */
function fakeContext(mode: ExtensionContext['mode'],): ExtensionContext {
  return {
    mode,
    cwd: '/tmp/project',
  } as ExtensionContext;
}

/**
 Creates identity-style theme required by call renderer.
 
 @returns fake theme with used styling methods
 */
function fakeTheme(): Theme {
  return {
    fg(_color: string, text: string): string {
      return text;
    },
    bold(text: string): string {
      return text;
    },
  } as Theme;
}

//endregion Harness

await describe({
  name: '',
  children: [
    it({
      name: 'registers sequential tool and shutdown cleanup through default export',
      fn: async () => {
        /**
         Default-extension registration harness.
         */
        const harness = createPiHarness();
        await askUserQuestionExtension(harness.api,);
        const tool = registeredTool(harness,);
        expect(tool.name,)
          .toBe(ASK_USER_QUESTION_TOOL_NAME,);
        expect(tool.executionMode,)
          .toBe('sequential',);
        expect(harness.state.shutdown,)
          .toBeTypeOf('function',);
        harness.state.shutdown?.();
      },
    },),
    it({
      name: 'returns submitted answer from TUI requester',
      fn: async () => {
        /**
         Registration harness with answered requester.
         */
        const harness = createPiHarness();
        registerAskUserQuestionExtension({
          pi: harness.api,
          requestAnswer: async ({ cwd, },) => {
            expect(cwd,)
              .toBe('/tmp/project',);
            return {
              status: 'answered',
              answer: 'multiline\nanswer',
            };
          },
        },);
        const result = await registeredTool(harness,).execute(
          'call-id',
          { question: 'What should happen?', },
          new AbortController().signal,
          undefined,
          fakeContext('tui',),
        );
        expect(result.details,)
          .toEqual({
            status: 'answered',
            answer: 'multiline\nanswer',
          },);
      },
    },),
    it({
      name: 'returns cancellation from TUI requester',
      fn: async () => {
        /**
         Registration harness with cancelling requester.
         */
        const harness = createPiHarness();
        registerAskUserQuestionExtension({
          pi: harness.api,
          requestAnswer: async () => ({ status: 'cancelled', }),
        },);
        const result = await registeredTool(harness,).execute(
          'call-id',
          { question: 'Continue?', },
          new AbortController().signal,
          undefined,
          fakeContext('tui',),
        );
        expect(result.details,)
          .toEqual({ status: 'cancelled', },);
      },
    },),
    it({
      name: 'rejects non-TUI mode without requesting answer',
      fn: async () => {
        /**
         Request invocation evidence.
         */
        const calls: string[] = [];
        /**
         Registration harness for noninteractive execution.
         */
        const harness = createPiHarness();
        registerAskUserQuestionExtension({
          pi: harness.api,
          requestAnswer: async () => {
            calls.push('called',);
            return { status: 'cancelled', };
          },
        },);
        /**
         Captured unavailable-mode failure.
         */
        const caught: { value?: unknown; } = {};
        try {
          await registeredTool(harness,).execute(
            'call-id',
            { question: 'Continue?', },
            new AbortController().signal,
            undefined,
            fakeContext('rpc',),
          );
        }
        catch (error: unknown) {
          caught.value = error;
        }
        expect(caught.value,)
          .toBeInstanceOf(AskUserQuestionUnavailableError,);
        expect(calls,)
          .toHaveLength(0,);
      },
    },),
    it({
      name: 'renders complete wrapped question with visible controls',
      fn: async () => {
        /**
         Registration harness for call rendering.
         */
        const harness = createPiHarness();
        registerAskUserQuestionExtension({
          pi: harness.api,
          requestAnswer: async () => ({ status: 'cancelled', }),
        },);
        /**
         Rendered lines from narrow transcript width.
         */
        const lines = registeredTool(harness,)
          .renderCall(
          { question: 'first line\nsecond line with detail\u001B', },
          fakeTheme(),
          {},
          )
          .render(12,);
        /**
         Rendered content without wrapping whitespace.
         */
        const compact = lines
          .join('',)
          .replaceAll(' ', '',);
        expect(compact,)
          .toContain('firstline',);
        expect(compact,)
          .toContain('secondline',);
        expect(compact,)
          .toContain('<U+001B>',);
      },
    },),
  ],
},);
