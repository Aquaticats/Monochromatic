/**
 Payload-capture tests for the NeuralWatt dispatch pins.

 Each test drives the provider's own stream path with pi's injectable fetch and
 payload observer, so the assertions read the serialized request pi would send.
 The capture pattern is ported from
 `package/pi-plugin/openai-fast/src/host-fixture-http.ts`,
 narrowed here to chat-completions events and to pi's plain `FetchFunction`
 seam, which carries no borrowed marker of its own.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  normalizeContext,
  type ApiKeyAuth,
  type AssistantMessageEvent,
  type OpenAICompletionsOptions,
} from '@earendil-works/pi-ai';
import type {
  Logger,
} from '@monochromatic-dev/logger-core/ts';
import {
  createNeuralWattProvider,
  NEURALWATT_OPENAI_COMPLETIONS_COMPAT,
  projectNeuralWattModel,
  type NeuralWattPiModel,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Discards one log message.
 */
function discardLogMessage(): void {}

/**
 Silent logger the dispatch tests run under.
 */
function fixtureLogger(): Logger {
  return {
    debug: discardLogMessage,
    error: discardLogMessage,
    fatal: discardLogMessage,
    flush: function flush(): Promise<void> {
      return Promise.resolve();
    },
    info: discardLogMessage,
    trace: discardLogMessage,
    warn: discardLogMessage,
  };
}

/**
 Api-key auth stub; these tests never resolve credentials.

 @returns an auth object whose resolve reports nothing configured
 */
function fixtureApiKeyAuth(): NonNullable<ApiKeyAuth> {
  return {
    name: 'fixture key',
    resolve: function resolve(): Promise<undefined> {
      return Promise.resolve(undefined);
    },
  };
}

/**
 Build a completed chat-completions SSE response the stream can drain.

 @param text - synthetic assistant content the completion carries

 @returns a finite SSE response with one text delta and one stop chunk

 @example
 ```typescript
 const response = chatCompletionsSseResponse({ text: 'Fixture reply.' });
 ```
 */
function chatCompletionsSseResponse({
  text,
}: {
  readonly text: string;
},): Response {
  /**
   Chunk lines the response body carries.
   */
  const chunkLines = [
    JSON.stringify({
      choices: [{
        index: 0,
        delta: {
          role: 'assistant',
          content: text,
        },
        finish_reason: null,
      },],
    },),
    JSON.stringify({
      choices: [{
        index: 0,
        delta: {},
        finish_reason: 'stop',
      },],
    },),
  ].map(function toChunkLine(payload,) {
    return `data: ${payload}\n\n`;
  },);
  /**
   Complete SSE body, terminated by the protocol marker.
   */
  const body = `${chunkLines.join('')}data: [DONE]\n\n`;
  return new Response(
    body,
    {
      status: 200,
      headers: { 'content-type': 'text/event-stream', },
    },
  );
}

/**
 Captured request one dispatch test asserts on.
 */
type CapturedRequest = {
  readonly payload: Record<string, unknown>;
};

/**
 Drain one stream under a capturing transport.

 @param model - projected model under test

 @param options - dispatch options the caller varies per case

 @returns the request payloads pi serialized before sending
 */
async function captureDispatch({
  model,
  options,
}: {
  readonly model: NeuralWattPiModel;
  readonly options: OpenAICompletionsOptions;
},): Promise<CapturedRequest[]> {
  /**
   Requests pi serialized, in order.
   */
  const requests: CapturedRequest[] = [];
  /**
   Provider under test, carrying only the stream seam this test needs.
   */
  const { provider, } = createNeuralWattProvider({
    logger: fixtureLogger(),
    apiKeyAuth: fixtureApiKeyAuth(),
  },);
  /**
   Stream run for this capture, over a synthetic transcript.
   */
  const events = provider.stream(
    model,
    normalizeContext({
      systemPrompt: 'You are a fixture assistant.',
      messages: [{
        role: 'user',
        content: 'Fixture question.',
        timestamp: Date.now(),
      }],
    },),
    {
      ...options,
      apiKey: 'fixture-key',
      signal: new AbortController().signal,
      onPayload: function onPayload(payload,) {
        requests.push({
          payload: payload as Record<string, unknown>,
        });
        return undefined;
      },
      fetch: function fetch(): Promise<Response> {
        return Promise.resolve(chatCompletionsSseResponse({
          text: 'Fixture reply.',
        },),);
      },
    },
  );
  /**
   Events drained so the request lifecycle finishes before assertions.
   */
  const drained: AssistantMessageEvent[] = [];
  for await (const event of events)
    drained.push(event,);
  void drained;
  return requests;
}

/**
 Read the serialized chat payload one capture produced.

 @param requests - captures the dispatch produced

 @returns the request body pi built
 */
function payloadOf(
  requests: readonly CapturedRequest[],
): Record<string, unknown> {
  expect(requests).toHaveLength(1);
  /**
   The single request the capture produced.
   */
  const [captured] = requests;
  return captured?.payload ?? {};
}

/**
 Read the serialized message roles one payload carries.

 @param payload - request body pi built

 @returns the role strings in message order
 */
function rolesOf(
  payload: Record<string, unknown>,
): string[] {
  /**
   Messages the payload serialized, narrowed to unknown members.
   */
  const messages: readonly unknown[] = Array.isArray(payload.messages)
    ? payload.messages
    : [];
  return messages.flatMap(function readRole(entry: unknown,) {
    if (((typeof entry) === 'object')
      && (entry !== null)
      && ('role' in entry)
      && ((typeof entry.role) === 'string'))
      return [entry.role];
    return [];
  },);
}

//endregion Fixtures

//region Reasoning effort dispatch

await describe({
  name: 'dispatch payload',
  children: [
    it({
      name: 'dispatches the mapped wire value for the selected thinking level',
      fn: async () => {
        const requests = await captureDispatch({
          model: projectNeuralWattModel({
            facts: {
              id: 'fixture-reasoning',
              name: 'Fixture Reasoning',
              input: ['text'],
              reasoningEfforts: { medium: 'MEDIUM', },
            },
          },),
          options: { reasoningEffort: 'medium', },
        },);
        const payload = payloadOf(requests);
        expect(payload.reasoning_effort).toBe('MEDIUM');
      },
    }),
    it({
      name: 'observed verdict: auto-detection without the reasoning pins still sends the mapped wire value, so the pins are redundant-but-kept-for-parity',
      fn: async () => {
        const pinned = projectNeuralWattModel({
          facts: {
            id: 'fixture-reasoning',
            name: 'Fixture Reasoning',
            input: ['text'],
            reasoningEfforts: { medium: 'MEDIUM', },
          },
        },);
        /**
         The same model with the reasoning pins removed: `supportsReasoningEffort`
         and `thinkingFormat` fall back to pi's URL auto-detection while the role
         pin stays, so only the reasoning path differs.
         */
        const autoDetected: NeuralWattPiModel = {
          ...pinned,
          compat: {
            supportsDeveloperRole:
              NEURALWATT_OPENAI_COMPLETIONS_COMPAT.supportsDeveloperRole,
          },
        };
        const requests = await captureDispatch({
          model: autoDetected,
          options: { reasoningEffort: 'medium', },
        },);
        const payload = payloadOf(requests);
        expect(payload.reasoning_effort).toBe('MEDIUM');
      },
    }),
    it({
      name: 'sends the off wire value when no thinking level is selected',
      fn: async () => {
        const requests = await captureDispatch({
          model: projectNeuralWattModel({
            facts: {
              id: 'fixture-reasoning',
              name: 'Fixture Reasoning',
              input: ['text'],
              reasoningEfforts: {
                off: 'none',
                high: 'high',
              },
            },
          },),
          options: {},
        },);
        const payload = payloadOf(requests);
        expect(payload.reasoning_effort).toBe('none');
      },
    }),
    it({
      name: 'travels as the system role for a reasoning model',
      fn: async () => {
        const requests = await captureDispatch({
          model: projectNeuralWattModel({
            facts: {
              id: 'fixture-reasoning',
              name: 'Fixture Reasoning',
              input: ['text'],
              reasoningEfforts: { high: 'high', },
            },
          },),
          options: { reasoningEffort: 'high', },
        },);
        const roles = rolesOf(payloadOf(requests));
        expect(roles.includes('system',)).toBe(true);
        expect(roles.includes('developer',)).toBe(false);
      },
    }),
    it({
      name: 'positive control: the same model without the pin travels as developer',
      fn: async () => {
        const pinned = projectNeuralWattModel({
          facts: {
            id: 'fixture-reasoning',
            name: 'Fixture Reasoning',
            input: ['text'],
            reasoningEfforts: { high: 'high', },
          },
        },);
        /**
         The same model with the role pin lifted, as the control arm.
         */
        const unpinned: NeuralWattPiModel = {
          ...pinned,
          compat: {
            ...pinned.compat,
            supportsDeveloperRole: true,
          },
        };
        const requests = await captureDispatch({
          model: unpinned,
          options: { reasoningEffort: 'high', },
        },);
        const roles = rolesOf(payloadOf(requests));
        expect(roles.includes('developer',)).toBe(true);
        expect(roles.includes('system',)).toBe(false);
      },
    }),
  ],
});

//endregion Reasoning effort dispatch
