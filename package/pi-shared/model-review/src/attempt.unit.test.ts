/**
 Built-artifact tests for callback-free structured reviewer transport.
 
 @module
 */

import type {
  Api,
  AssistantMessageEvent,
  Model,
  Tool,
} from '@earendil-works/pi-ai';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  runStructuredJsonRetries,
  runStructuredToolRequest,
  type ScriptedStructuredReviewTransport,
  structuredReviewSignal,
} from '../dist/final/node/index.mjs';

/** Fixture reviewer context window. */
const CONTEXT_WINDOW = 128_000;

/** Fixture reviewer output capacity. */
const MAX_TOKENS = 16_384;

/** Test timeout that does not govern transport behavior. */
const TEST_TIMEOUT_MS = 10_000;

/** Fixture model consumed only by scripted transport. */
const MODEL = {
  id: 'reviewer',
  name: 'Reviewer',
  api: 'openai-responses',
  provider: 'test',
  baseUrl: 'https://example.invalid',
  reasoning: false,
  input: ['text',],
  cost: {
    input: 1,
    output: 1,
    cacheRead: 0,
    cacheWrite: 0,
  },
  contextWindow: CONTEXT_WINDOW,
  maxTokens: MAX_TOKENS,
} satisfies Model<Api>;

/** Fixture structured verdict tool. */
const TOOL = {
  name: 'submit_fixture_review',
  description: 'Submit fixture review.',
  parameters: {
    type: 'object',
    properties: {
      approved: { type: 'boolean', },
      feedback: { type: 'string', },
    },
    required: ['approved', 'feedback',],
  },
} satisfies Tool;

/**
 Build one async event stream from fixed event list.
 
 @param events - ordered provider events
 
 @returns async event stream
 
 @example
 ```ts
 eventStream([]);
 ```
 */
async function* eventStream(
  events: readonly AssistantMessageEvent[],
): AsyncIterable<AssistantMessageEvent> {
  for (const event of events)
    yield event;
}

/**
 Build finalized text event stream.
 
 @param content - finalized provider text
 
 @returns one-event stream
 
 @example
 ```ts
 textStream('{}');
 ```
 */
function textStream(content: string,): AsyncIterable<AssistantMessageEvent> {
  return eventStream([{
    type: 'text_end',
    contentIndex: 0,
    content,
    partial: {} as never,
  },],);
}

/**
 Build expected tool-call event stream.
 
 @param argumentsValue - unknown structured arguments
 
 @param toolName - emitted tool name
 
 @returns one-event stream
 
 @example
 ```ts
 toolStream({ approved: true });
 ```
 */
function toolStream(
  argumentsValue: unknown,
  toolName = TOOL.name,
): AsyncIterable<AssistantMessageEvent> {
  return eventStream([{
    type: 'toolcall_end',
    contentIndex: 0,
    toolCall: {
      type: 'toolCall',
      id: 'review-1',
      name: toolName,
      arguments: argumentsValue as never,
    },
    partial: {} as never,
  },],);
}

/**
 Build deterministic data transport.
 
 @param responses - ordered response streams
 
 @returns mutable script state
 
 @example
 ```ts
 scriptedTransport([textStream('{}')]);
 ```
 */
function scriptedTransport(
  responses: readonly AsyncIterable<AssistantMessageEvent>[],
): ScriptedStructuredReviewTransport {
  return {
    nextResponseIndex: 0,
    responses,
    requests: [],
  };
}

/**
 Capture async error without promise matcher indirection.
 
 @param action - action expected to fail
 
 @returns thrown value
 
 @example
 ```ts
 await captureError(async () => { throw new Error('x'); });
 ```
 */
async function captureError(action: () => Promise<unknown>,): Promise<unknown> {
  try {
    await action();
  }
  catch (error) {
    return error;
  }
  throw new Error('expected action to fail',);
}

await describe({
  name: runStructuredToolRequest.name,
  children: [
    it({
      name: 'returns expected tool arguments and records final provider request snapshot',
      fn: async () => {
        /** Deterministic provider transport. */
        const transport = scriptedTransport([
          toolStream({ approved: true, feedback: 'complete', },),
        ],);
        /** Initial structured result. */
        const result = await runStructuredToolRequest({
          model: MODEL,
          auth: { apiKey: 'fixture-key', },
          prompt: {
            systemPrompt: 'Judge.',
            userContent: 'Evidence.',
          },
          signal: structuredReviewSignal({ timeoutMs: TEST_TIMEOUT_MS, },),
          toolName: TOOL.name,
          tool: TOOL,
          maxOutputTokens: 512,
          testTransport: transport,
        },);
        expect(result,).toEqual({
          kind: 'toolCall',
          arguments: { approved: true, feedback: 'complete', },
        },);
        expect(transport.requests,).toHaveLength(1,);
        /** Captured final provider request. */
        const [request,] = transport.requests;
        if (request === undefined)
          throw new Error('Expected captured provider request.',);
        expect(request.model,).toEqual({
          api: MODEL.api,
          id: MODEL.id,
          provider: MODEL.provider,
        },);
        expect(request.context.systemPrompt,).toBe('Judge.',);
        expect(request.context.toolNames,).toEqual([TOOL.name,],);
        expect(request.context.messages[0]?.role,).toBe('user',);
        expect(request.context.messages[0]?.content,).toBe('Evidence.',);
        expect(request.options,).toMatchObject({
          apiKey: 'fixture-key',
          hasSignal: true,
          maxTokens: 512,
          toolChoiceType: 'required',
        },);
      },
    },),
    it({
      name: 'returns omitted-tool text and rejects unexpected tool name',
      fn: async () => {
        /** Omitted-tool scripted transport. */
        const omittedTransport = scriptedTransport([textStream('tool omitted',),],);
        /** Omitted-tool result. */
        const omitted = await runStructuredToolRequest({
          model: MODEL,
          auth: {},
          prompt: { systemPrompt: 'Judge.', userContent: 'Evidence.', },
          signal: structuredReviewSignal({ timeoutMs: TEST_TIMEOUT_MS, },),
          toolName: TOOL.name,
          tool: TOOL,
          testTransport: omittedTransport,
        },);
        expect(omitted,).toEqual({ kind: 'noToolCall', textContent: 'tool omitted', },);

        /** Unexpected-tool failure. */
        const error = await captureError(async function runUnexpectedTool() {
          return runStructuredToolRequest({
            model: MODEL,
            auth: {},
            prompt: { systemPrompt: 'Judge.', userContent: 'Evidence.', },
            signal: structuredReviewSignal({ timeoutMs: TEST_TIMEOUT_MS, },),
            toolName: TOOL.name,
            tool: TOOL,
            testTransport: scriptedTransport([toolStream({}, 'different_tool',),],),
          },);
        },);
        expect(error,).toBeInstanceOf(Error,);
        expect((error as Error).message,).toContain('different_tool',);
      },
    },),
  ],
},);

await describe({
  name: runStructuredJsonRetries.name,
  children: [
    it({
      name: 'parses direct JSON and records request without tools',
      fn: async () => {
        /** Direct-JSON scripted transport. */
        const transport = scriptedTransport([
          textStream('{"approved":false,"feedback":"missing evidence"}',),
        ],);
        /** Unknown parsed direct JSON value. */
        const value = await runStructuredJsonRetries({
          model: MODEL,
          auth: {},
          prompt: { systemPrompt: 'Judge JSON.', userContent: 'Retry.', },
          signal: structuredReviewSignal({ timeoutMs: TEST_TIMEOUT_MS, },),
          expectedToolName: TOOL.name,
          testTransport: transport,
        },);
        expect(value,).toEqual({ approved: false, feedback: 'missing evidence', },);
        /** Captured direct-JSON request. */
        const [request,] = transport.requests;
        if (request === undefined)
          throw new Error('Expected captured direct-JSON request.',);
        expect(request.context.systemPrompt,).toBe('Judge JSON.',);
        expect(request.context.toolNames,).toEqual([],);
        expect(request.context.messages[0]?.role,).toBe('user',);
        expect(request.context.messages[0]?.content,).toBe('Retry.',);
      },
    },),
    it({
      name: 'retries empty text once and rejects malformed final text',
      fn: async () => {
        /** Empty then valid direct-JSON transport. */
        const transport = scriptedTransport([
          textStream('',),
          textStream('prefix {"approved":true,"feedback":"second"} suffix',),
        ],);
        /** Parsed second direct-JSON response. */
        const value = await runStructuredJsonRetries({
          model: MODEL,
          auth: {},
          prompt: { systemPrompt: 'Judge JSON.', userContent: 'Retry.', },
          signal: structuredReviewSignal({ timeoutMs: TEST_TIMEOUT_MS, },),
          expectedToolName: TOOL.name,
          testTransport: transport,
        },);
        expect(value,).toEqual({ approved: true, feedback: 'second', },);
        expect(transport.requests,).toHaveLength(2,);

        /** Malformed direct-JSON failure. */
        const error = await captureError(async function runMalformedJson() {
          return runStructuredJsonRetries({
            model: MODEL,
            auth: {},
            prompt: { systemPrompt: 'Judge JSON.', userContent: 'Retry.', },
            signal: structuredReviewSignal({ timeoutMs: TEST_TIMEOUT_MS, },),
            expectedToolName: TOOL.name,
            testTransport: scriptedTransport([textStream('not structured',),],),
          },);
        },);
        expect(error,).toBeInstanceOf(Error,);
      },
    },),
  ],
},);

await describe({
  name: structuredReviewSignal.name,
  children: [
    it({
      name: 'propagates already-active caller cancellation',
      fn: async () => {
        /** Already-aborted caller cancellation. */
        const controller = new AbortController();
        controller.abort(new Error('caller cancelled review',),);
        /** Composite cancellation signal. */
        const signal = structuredReviewSignal({
          signal: controller.signal,
          timeoutMs: TEST_TIMEOUT_MS,
        },);
        expect(signal.aborted,).toBe(true,);
        expect((signal.reason as Error).message,).toBe('caller cancelled review',);
      },
    },),
  ],
},);
