/**
 Built-artifact tests for provider stream failures reaching review callers.

 A provider that refuses a request terminates its stream with an `error` event
 carrying `errorMessage`.
 A stream carrying that event is not a reviewer that omitted the forced tool,
 so a caller must see the provider's own diagnostic instead of an omitted-tool
 reply followed by a pointless direct-JSON retry.

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

//region Fixtures

/** Fixture reviewer context window. */
const CONTEXT_WINDOW = 128_000;

/** Fixture reviewer output capacity. */
const MAX_TOKENS = 16_384;

/** Test timeout that does not govern transport behavior. */
const TEST_TIMEOUT_MS = 10_000;

/**
 Provider diagnostic a failing scripted stream carries, shaped like a real
 provider refusal so the message read by callers is unmistakable.
 */
const PROVIDER_FAILURE_TEXT = '402: {"message":"out of credits","type":"billing_error"}';

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

//endregion Fixtures

//region Helpers

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
 Build one terminal provider failure stream.

 @param errorMessage - provider diagnostic the terminal event carries

 @param reason - terminal stop reason the provider reported

 @returns one-event failing stream

 @example
 ```ts
 errorStream({ errorMessage: '429: rate limited' });
 ```
 */
function errorStream(
  {
    errorMessage,
    reason = 'error',
  }: {
    readonly errorMessage?: string;
    readonly reason?: 'aborted' | 'error';
  } = {},
): AsyncIterable<AssistantMessageEvent> {
  return eventStream([{
    type: 'error',
    reason,
    error: (errorMessage === undefined ? {} : { errorMessage, }) as never,
  },],);
}

/**
 Build deterministic data transport.

 @param responses - ordered response streams

 @returns mutable script state

 @example
 ```ts
 scriptedTransport([errorStream()]);
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
 Capture async failure without promise matcher indirection.

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

//endregion Helpers

await describe({
  name: 'provider stream failure',
  children: [
    it({
      name: `${runStructuredToolRequest.name} rejects a provider failure event with the provider message instead of reporting an omitted tool`,
      fn: async () => {
        /** Caller-visible provider failure. */
        const error = await captureError(async function runProviderFailure() {
          return runStructuredToolRequest({
            model: MODEL,
            auth: {},
            prompt: { systemPrompt: 'Judge.', userContent: 'Evidence.', },
            signal: structuredReviewSignal({ timeoutMs: TEST_TIMEOUT_MS, },),
            toolName: TOOL.name,
            tool: TOOL,
            testTransport: scriptedTransport([errorStream({
              errorMessage: PROVIDER_FAILURE_TEXT,
            },),],),
          },);
        },);
        expect(error,).toBeInstanceOf(Error,);
        expect((error as Error).message,).toContain(PROVIDER_FAILURE_TEXT,);
        expect((error as Error).message,).not.toContain('omitted',);
      },
    },),
    it({
      name: `${runStructuredToolRequest.name} rejects a provider abort event and names the stop reason it reported`,
      fn: async () => {
        /** Caller-visible provider abort. */
        const error = await captureError(async function runProviderAbort() {
          return runStructuredToolRequest({
            model: MODEL,
            auth: {},
            prompt: { systemPrompt: 'Judge.', userContent: 'Evidence.', },
            signal: structuredReviewSignal({ timeoutMs: TEST_TIMEOUT_MS, },),
            toolName: TOOL.name,
            tool: TOOL,
            testTransport: scriptedTransport([errorStream({ reason: 'aborted', },),],),
          },);
        },);
        expect(error,).toBeInstanceOf(Error,);
        expect((error as Error).message,).toContain('aborted',);
        expect((error as Error).message,).not.toContain('omitted',);
      },
    },),
    it({
      name: `${runStructuredJsonRetries.name} rejects a provider failure event instead of retrying it as empty text`,
      fn: async () => {
        /** Provider failure scripted transport. */
        const transport = scriptedTransport([errorStream({
          errorMessage: PROVIDER_FAILURE_TEXT,
        },),],);
        /** Caller-visible provider failure. */
        const error = await captureError(async function runProviderFailure() {
          return runStructuredJsonRetries({
            model: MODEL,
            auth: {},
            prompt: { systemPrompt: 'Judge JSON.', userContent: 'Retry.', },
            signal: structuredReviewSignal({ timeoutMs: TEST_TIMEOUT_MS, },),
            expectedToolName: TOOL.name,
            testTransport: transport,
          },);
        },);
        expect(transport.requests,).toHaveLength(1,);
        expect(error,).toBeInstanceOf(Error,);
        expect((error as Error).message,).toContain(PROVIDER_FAILURE_TEXT,);
      },
    },),
  ],
},);
