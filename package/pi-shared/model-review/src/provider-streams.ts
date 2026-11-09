/**
 Direct Pi AI provider dispatch for structured reviewer requests.
 
 @module
 */

import type {
  Api,
  AssistantMessageEvent,
  Context,
  Model,
  ProviderStreams,
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
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  cloneReviewMessages,
  isolateReviewContext,
  isolateReviewModel,
  isolateReviewOptions,
  type ReviewSimpleStreamOptions,
} from './provider-data.ts';
import type {
  ScriptedStructuredReviewTransport,
  StructuredReviewRequestSnapshot,
} from './types.ts';

/**
 Provider dispatch inputs.
 
 @example
 ```ts
 const request: StructuredReviewStreamRequest = { model, context, options };
 ```
 */
type StructuredReviewStreamRequest = {
  /**
   Selected reviewer model.
   */
  readonly model: ForeignBorrowed<Model<Api>>;
  /**
   Final provider context.
   */
  readonly context: ForeignBorrowed<Context>;
  /**
   Final provider stream options.
   */
  readonly options: ForeignBorrowed<ReviewSimpleStreamOptions>;
  /**
   Optional deterministic data seam.
   */
  readonly testTransport?: ForeignBorrowed<ScriptedStructuredReviewTransport>;
};

/**
 Lazy Pi AI provider module factory.
 */
type LazyProviderStreams = () => ProviderStreams;

/**
 Lazy provider modules keyed by the API each one implements.

 Adding a provider is one entry here: the lookup key is the selected model's
 recorded API, so no branch per API needs to state or re-assert it.
 */
const PROVIDER_STREAMS_BY_API: Readonly<Record<string, LazyProviderStreams>> = {
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
 Project provider APIs supported by Pi AI direct modules, derived from the table keys.
 */
const SUPPORTED_APIS: string = Object
  .keys(PROVIDER_STREAMS_BY_API,)
  .join(', ',);
/**
 Snapshot one known review context without retaining caller-owned arrays.
 
 @param context - final provider context
 
 @returns isolated primitive request projection
 
 @example
 ```ts
 snapshotContext(context);
 ```
 */
function snapshotContext(
  context: ForeignBorrowed<Context>,
): StructuredReviewRequestSnapshot['context'] {
  /**
   Isolated user-message snapshots.
   */
  const messages = cloneReviewMessages({
    messages: context.messages,
    errorPrefix: 'Structured review request snapshot received unsupported',
  },);
  /**
   Isolated tool-name snapshots.
   */
  const toolNames: string[] = [];
  for (const tool of context.tools ?? [])
    toolNames.push(tool.name,);
  return {
    systemPrompt: context.systemPrompt ?? '',
    messages,
    toolNames,
  };
}

/**
 Snapshot stream options without retaining header or selector objects.
 
 @param options - final provider stream options
 
 @returns isolated primitive options projection
 
 @example
 ```ts
 snapshotOptions({ signal: AbortSignal.timeout(1000) });
 ```
 */
function snapshotOptions(
  options: ForeignBorrowed<ReviewSimpleStreamOptions>,
): StructuredReviewRequestSnapshot['options'] {
  /**
   Provider-specific tool selector.
   */
  const { toolChoice, } = options;
  /**
   Primitive selector type when present.
   */
  const toolChoiceType = (typeof toolChoice) === 'string'
    ? toolChoice
    : ((toolChoice !== null)
      && ((typeof toolChoice) === 'object')
      && ('type' in toolChoice)
      && ((typeof toolChoice.type) === 'string'))
      ? toolChoice.type
      : undefined;
  /**
   Primitive selector tool name when present.
   */
  const toolChoiceName = ((toolChoice !== null)
    && ((typeof toolChoice) === 'object')
    && ('name' in toolChoice)
    && ((typeof toolChoice.name) === 'string'))
    ? toolChoice.name
    : undefined;
  return {
    ...(options.apiKey === undefined ? {} : { apiKey: options.apiKey, }),
    ...(options.headers === undefined
      ? {}
      : { headers: { ...options.headers, }, }),
    hasSignal: options.signal !== undefined,
    ...(options.maxTokens === undefined ? {} : { maxTokens: options.maxTokens, }),
    ...(toolChoiceType === undefined ? {} : { toolChoiceType, }),
    ...(toolChoiceName === undefined ? {} : { toolChoiceName, }),
  };
}

/**
 Consume one scripted response after recording exact provider request data.
 
 @param transport - mutable deterministic script state
 
 @param model - selected reviewer model
 
 @param context - final provider context
 
 @param options - final provider options
 
 @returns next scripted response stream
 
 @mutates transport - advances script and records isolated request snapshot
 
 @example
 ```ts
 scriptedStructuredReviewStream({ transport, model, context, options });
 ```
 */
function scriptedStructuredReviewStream(
  {
    transport,
    model,
    context,
    options,
  }: {
    transport: ScriptedStructuredReviewTransport;
    readonly model: ForeignBorrowed<Model<Api>>;
    readonly context: ForeignBorrowed<Context>;
    readonly options: ForeignBorrowed<ReviewSimpleStreamOptions>;
  },
): AsyncIterable<AssistantMessageEvent> {
  /**
   Script index consumed by this request.
   */
  const index = transport.nextResponseIndex;
  /**
   Scripted response selected before state advances.
   */
  const response = transport.responses[index];
  if (response === undefined)
    throw new Error(`Scripted structured review has no response at index ${index}`,);
  transport.requests[index] = {
    model: {
      api: model.api,
      id: model.id,
      provider: model.provider,
    },
    context: snapshotContext(context,),
    options: snapshotOptions(options,),
  };
  transport.nextResponseIndex = index + 1;
  return response;
}

/**
 Dispatch one stream through concrete implementation matching selected API.
 
 Static lazy-provider imports avoid authored dynamic imports and retained-provider
 capabilities. Isolated request data crosses each lazy stream boundary.
 
 @param model - selected reviewer model
 
 @param context - final provider context
 
 @param options - final provider stream options
 
 @param testTransport - optional deterministic data seam
 
 @returns assistant event stream
 
 @mutates model - concrete provider can inspect selected model data
 
 @mutates context - concrete provider consumes request messages and tools
 
 @mutates options - concrete provider observes auth and cancellation capabilities
 
 @mutates testTransport - deterministic seam advances script and records snapshot
 
 @throws when selected API has no direct implementation
 
 @example
 ```ts
 await streamStructuredReview({ model, context, options });
 ```
 */
function streamStructuredReview(
  {
    model,
    context,
    options,
    testTransport,
  }: ForeignBorrowed<Readonly<StructuredReviewStreamRequest>>,
): AsyncIterable<AssistantMessageEvent> {
  if (testTransport !== undefined) {
    return scriptedStructuredReviewStream({
      transport: testTransport,
      model,
      context,
      options,
    },);
  }
  /**
   Isolated provider context used only beyond external package boundary.
   */
  const providerContext = isolateReviewContext(context,);
  /**
   Isolated provider options used only beyond external package boundary.
   */
  const providerOptions = isolateReviewOptions(options,);
  /**
   Lazy provider module for the selected model's API.
   */
  const createProviderStreams = PROVIDER_STREAMS_BY_API[model.api];
  if (createProviderStreams === undefined) {
    throw new Error(
      `No Pi AI API implementation available for structured reviewer ${model.provider}/${model.id} using ${model.api}. Supported APIs: ${SUPPORTED_APIS}`,
    );
  }
  return createProviderStreams()
    .streamSimple(
    isolateReviewModel(model,),
    providerContext,
    providerOptions,
    );
}

export { streamStructuredReview, };
