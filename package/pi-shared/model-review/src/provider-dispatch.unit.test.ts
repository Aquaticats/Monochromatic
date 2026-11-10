/**
 Provider dispatch coverage for the keyed lazy-provider table.

 The dispatch shape is data rather than branches, so these tests read the
 authored source for the single call site, drive the public API for the
 derived supported-API list, and drive the table itself with fake provider
 modules so each entry is reachable without its optional SDK.

 @module
 */

import { readFile, } from 'node:fs/promises';

import {
  createAssistantMessageEventStream,
  normalizeContext,
  type Api,
  type AssistantMessageEventStream,
  type Model,
  type ProviderStreams,
  type SimpleStreamOptions,
  type Tool,
  type TranscriptContext,
} from '@earendil-works/pi-ai';
import { anthropicMessagesApi, } from '@earendil-works/pi-ai/api/anthropic-messages.lazy';
import { azureOpenAIResponsesApi, } from '@earendil-works/pi-ai/api/azure-openai-responses.lazy';
import { bedrockConverseStreamApi, } from '@earendil-works/pi-ai/api/bedrock-converse-stream.lazy';
import { googleGenerativeAIApi, } from '@earendil-works/pi-ai/api/google-generative-ai.lazy';
import { googleVertexApi, } from '@earendil-works/pi-ai/api/google-vertex.lazy';
import { mistralConversationsApi, } from '@earendil-works/pi-ai/api/mistral-conversations.lazy';
import { openAICodexResponsesApi, } from '@earendil-works/pi-ai/api/openai-codex-responses.lazy';
import { openAICompletionsApi, } from '@earendil-works/pi-ai/api/openai-completions.lazy';
import { openAIResponsesApi, } from '@earendil-works/pi-ai/api/openai-responses.lazy';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  dispatchProviderStream,
  type LazyProviderStreams,
  PROVIDER_STREAMS_BY_API,
  runStructuredToolRequest,
} from '../dist/final/node/index.mjs';

/**
 Production provider dispatch source.
 */
const PROVIDER_STREAMS_SOURCE = new URL(
  'provider-streams.ts',
  import.meta.url,
);

/**
 Fixture reviewer context window.
 */
const CONTEXT_WINDOW = 128_000;

/**
 Fixture reviewer output capacity.
 */
const MAX_TOKENS = 16_384;

/**
 Fixture structured verdict tool.
 */
const TOOL = {
  name: 'submit_fixture_review',
  description: 'Submit fixture review.',
  parameters: {
    type: 'object',
    properties: {
      approved: { type: 'boolean', },
    },
    required: ['approved',],
  },
} satisfies Tool;

/**
 Fixture APIs the package must dispatch directly.
 */
const EXPECTED_DISPATCHED_APIS: readonly string[] = [
  'anthropic-messages',
  'azure-openai-responses',
  'bedrock-converse-stream',
  'google-generative-ai',
  'google-vertex',
  'mistral-conversations',
  'openai-codex-responses',
  'openai-completions',
  'openai-responses',
];

/**
 Provider module each API key must resolve to, keyed by that API.

 Importing these lazy factories needs no optional SDK: each loads its
 implementation, and that implementation's SDK, only when called.
 */
const PROVIDER_MODULE_BY_API: Readonly<Record<string, LazyProviderStreams>> = {
  'anthropic-messages': anthropicMessagesApi,
  'azure-openai-responses': azureOpenAIResponsesApi,
  'bedrock-converse-stream': bedrockConverseStreamApi,
  'google-generative-ai': googleGenerativeAIApi,
  'google-vertex': googleVertexApi,
  'mistral-conversations': mistralConversationsApi,
  'openai-codex-responses': openAICodexResponsesApi,
  'openai-completions': openAICompletionsApi,
  'openai-responses': openAIResponsesApi,
};

/**
 Fixture API key the fake provider modules accept as an opaque credential.
 */
const FIXTURE_API_KEY = 'fixture-key';

/**
 Fixture review context carrying the verdict tool.

 Normalized first, because dispatch receives what
 {@link isolateReviewContext} produces rather than a raw caller context.
 */
const REVIEW_CONTEXT: TranscriptContext = normalizeContext({
  systemPrompt: 'Judge the evidence.',
  messages: [
    {
      role: 'user',
      content: 'Evidence.',
      timestamp: 0,
    },
  ],
  tools: [TOOL,],
},);

/**
 Fixture provider options carrying the fixture credential.
 */
const REVIEW_OPTIONS: SimpleStreamOptions = {
  apiKey: FIXTURE_API_KEY,
  maxTokens: MAX_TOKENS,
};

/**
 Build one reviewer model fixture recording an API.
 
 @param api - Pi AI API the fixture model records
 
 @returns complete reviewer model
 
 @example
 ```ts
 fixtureModel('google-vertex');
 ```
 */
function fixtureModel(api: string,): Model<Api> {
  return {
    id: `reviewer-${api}`,
    name: 'Reviewer',
    api,
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
  };
}

/**
 One recorded call into a fake provider module.
 */
type RecordedProviderCall = {
  /**
   Method the dispatch called on the fake module.
   */
  readonly method: 'stream' | 'streamSimple';
  /**
   API key the fake module was installed under.
   */
  readonly moduleApi: string;
  /**
   Model the dispatch handed the fake module.
   */
  readonly model: Model<Api>;
};

/**
 Fake provider module plus the stream it answers with.
 */
type FakeProviderModule = {
  /**
   Lazy factory a test installs under one API key.
   */
  readonly factory: LazyProviderStreams;
  /**
   Stream the fake answers every request with.
   */
  readonly stream: AssistantMessageEventStream;
};

/**
 Build a fake provider module that records every dispatch call it receives.
 
 @param api - API key the fake stands in for
 
 @param calls - shared recording sink
 
 @returns lazy factory and the stream it answers with
 
 @example
 ```ts
 fakeProviderModule({ api: 'google-vertex', calls });
 ```
 */
function fakeProviderModule(
  {
    api,
    calls,
  }: {
    readonly api: string;
    readonly calls: RecordedProviderCall[];
  },
): FakeProviderModule {
  /**
   Stream the fake answers every request with.
   */
  const stream = createAssistantMessageEventStream();
  /**
   Provider streams the fake factory returns.
   */
  const streams: ProviderStreams = {
    stream: function recordStream(model,): AssistantMessageEventStream {
      calls.push({
        method: 'stream',
        moduleApi: api,
        model,
      },);
      return stream;
    },
    streamSimple: function recordStreamSimple(model,): AssistantMessageEventStream {
      calls.push({
        method: 'streamSimple',
        moduleApi: api,
        model,
      },);
      return stream;
    },
  };
  return {
    stream,
    factory: function createFakeProviderStreams(): ProviderStreams {
      return streams;
    },
  };
}

/**
 Capture an async error without promise matcher indirection.
 
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

/**
 Count non-overlapping occurrences of one literal text.
 
 @param source - text to search
 
 @param needle - literal text to count
 
 @returns number of occurrences
 
 @example
 ```ts
 countOccurrences({ source: 'a b a', needle: 'a' });
 ```
 */
function countOccurrences(
  {
    source,
    needle,
  }: {
    readonly source: string;
    readonly needle: string;
  },
): number {
  return source.split(needle,).length - 1;
}

await describe({
  name: 'provider dispatch',
  children: [
    it({
      name: 'calls the lazy provider module from one shared dispatch site',
      fn: async () => {
        /**
         Authored provider dispatch source text.
         */
        const source = await readFile(
          PROVIDER_STREAMS_SOURCE,
          'utf8',
        );
        expect(countOccurrences({
          source,
          needle: '.streamSimple(',
        },),).toBe(1,);
        expect(countOccurrences({
          source,
          needle: 'if (model.api === ',
        },),).toBe(0,);
      },
    },),
    it({
      name: 'names every dispatched API when the selected model has no implementation',
      fn: async () => {
        /**
         Dispatch failure for an API the package does not implement.
         */
        const error = await captureError(function dispatchUnimplementedApi() {
          return runStructuredToolRequest({
            model: {
              id: 'reviewer',
              name: 'Reviewer',
              api: 'pi-messages',
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
            },
            auth: { apiKey: 'fixture-key', },
            prompt: {
              systemPrompt: 'Judge.',
              userContent: 'Evidence.',
            },
            tool: TOOL,
            toolName: TOOL.name,
            signal: AbortSignal.timeout(1_000,),
          },);
        },);
        expect(error,).toBeInstanceOf(Error,);
        /**
         Dispatch failure message listing supported APIs.
         */
        const { message, } = error as Error;
        expect(message,).toContain('No Pi AI API implementation available',);
        /**
         Supported-API list appended to the dispatch failure message.
         */
        const listedApis = message
          .slice(message.indexOf('Supported APIs: ',) + 'Supported APIs: '.length,)
          .split(', ',)
          .toSorted();
        expect(listedApis,).toEqual(EXPECTED_DISPATCHED_APIS.toSorted(),);
      },
    },),
    describe({
      name: dispatchProviderStream.name,
      children: EXPECTED_DISPATCHED_APIS.map(function apiBranch(api,) {
        return it({
          name: `dispatches ${api} through the lazy provider module registered for it`,
          fn: async function testApiBranch(): Promise<void> {
            expect(PROVIDER_STREAMS_BY_API[api],).toBe(PROVIDER_MODULE_BY_API[api],);
            /**
             Calls every fake provider module recorded during this test.
             */
            const calls: RecordedProviderCall[] = [];
            /**
             Fake provider module for every table entry, so a lookup by any
             other key is recorded rather than silently passing.
             */
            const fakesByApi: Readonly<Record<string, FakeProviderModule>> = Object.fromEntries(
              EXPECTED_DISPATCHED_APIS.map(function buildFake(apiUnderTest,) {
                return [apiUnderTest, fakeProviderModule({
                  api: apiUnderTest,
                  calls,
                },),];
              },),
            );
            /**
             Lazy factory table the dispatch reads, keyed like the production table.
             */
            const providerStreamsByApi = Object.fromEntries(
              Object
                .entries(fakesByApi,)
                .map(function lazyFactory([tableApi, fake,],) {
                  return [tableApi, fake.factory,];
                },),
            );
            /**
             Reviewer model recording the API under test.
             */
            const model = fixtureModel(api,);
            /**
             Stream the dispatch returned.
             */
            const stream = dispatchProviderStream({
              model,
              context: REVIEW_CONTEXT,
              options: REVIEW_OPTIONS,
              providerStreamsByApi,
            },);
            /**
             Fake module installed under the API under test.
             */
            const fake = fakesByApi[api];
            if (fake === undefined)
              throw new Error(`Missing fake provider module for ${api}`,);
            expect(stream,).toBe(fake.stream,);
            expect(calls,).toHaveLength(1,);
            expect(calls[0]?.method,).toBe('streamSimple',);
            expect(calls[0]?.moduleApi,).toBe(api,);
            expect(calls[0]?.model.api,).toBe(api,);
            expect(calls[0]?.model,).toEqual(model,);
            expect(calls[0]?.model,).not.toBe(model,);
          },
        },);
      },),
    },),
  ],
},);
