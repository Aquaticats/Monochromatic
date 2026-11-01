/**
 Direct Pi AI provider dispatch for structured reviewer requests.
 
 @module
 */

import type {
  Api,
  AssistantMessageEvent,
  Context,
  Model,
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
  assertModelUsesApi,
  isolateReviewContext,
  isolateReviewModel,
  isolateReviewOptions,
  type ReviewSimpleStreamOptions,
} from './provider-data.ts';
import type {
  ScriptedStructuredReviewTransport,
  StructuredReviewMessageSnapshot,
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
 Project provider APIs supported by Pi AI direct modules.
 */
const SUPPORTED_APIS = 'anthropic-messages, azure-openai-responses, bedrock-converse-stream, google-generative-ai, google-vertex, mistral-conversations, openai-codex-responses, openai-completions, openai-responses';

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
  const messages: StructuredReviewMessageSnapshot[] = [];
  for (const message of context.messages) {
    if ((message.role !== 'user') || ((typeof message.content) !== 'string')) {
      throw new Error(
        `Structured review request snapshot received unsupported ${message.role} message content`,
      );
    }
    messages.push({
      role: 'user',
      content: message.content,
      timestamp: message.timestamp,
    },);
  }
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
  if (model.api === 'anthropic-messages') {
    /**
     Model narrowed by exact Anthropic API assertion.
     */
    const selected = {
      model,
      api: 'anthropic-messages',
    } as const;
    assertModelUsesApi(selected,);
    return anthropicMessagesApi()
      .streamSimple(
      isolateReviewModel(selected.model,),
      providerContext,
      providerOptions,
      );
  }
  if (model.api === 'azure-openai-responses') {
    /**
     Model narrowed by exact Azure OpenAI API assertion.
     */
    const selected = {
      model,
      api: 'azure-openai-responses',
    } as const;
    assertModelUsesApi(selected,);
    return azureOpenAIResponsesApi()
      .streamSimple(
      isolateReviewModel(selected.model,),
      providerContext,
      providerOptions,
      );
  }
  if (model.api === 'bedrock-converse-stream') {
    /**
     Model narrowed by exact Bedrock API assertion.
     */
    const selected = {
      model,
      api: 'bedrock-converse-stream',
    } as const;
    assertModelUsesApi(selected,);
    return bedrockConverseStreamApi()
      .streamSimple(
      isolateReviewModel(selected.model,),
      providerContext,
      providerOptions,
      );
  }
  if (model.api === 'google-generative-ai') {
    /**
     Model narrowed by exact Google Generative AI assertion.
     */
    const selected = {
      model,
      api: 'google-generative-ai',
    } as const;
    assertModelUsesApi(selected,);
    return googleGenerativeAIApi()
      .streamSimple(
      isolateReviewModel(selected.model,),
      providerContext,
      providerOptions,
      );
  }
  if (model.api === 'google-vertex') {
    /**
     Model narrowed by exact Google Vertex API assertion.
     */
    const selected = {
      model,
      api: 'google-vertex',
    } as const;
    assertModelUsesApi(selected,);
    return googleVertexApi()
      .streamSimple(
      isolateReviewModel(selected.model,),
      providerContext,
      providerOptions,
      );
  }
  if (model.api === 'mistral-conversations') {
    /**
     Model narrowed by exact Mistral API assertion.
     */
    const selected = {
      model,
      api: 'mistral-conversations',
    } as const;
    assertModelUsesApi(selected,);
    return mistralConversationsApi()
      .streamSimple(
      isolateReviewModel(selected.model,),
      providerContext,
      providerOptions,
      );
  }
  if (model.api === 'openai-codex-responses') {
    /**
     Model narrowed by exact OpenAI Codex API assertion.
     */
    const selected = {
      model,
      api: 'openai-codex-responses',
    } as const;
    assertModelUsesApi(selected,);
    return openAICodexResponsesApi()
      .streamSimple(
      isolateReviewModel(selected.model,),
      providerContext,
      providerOptions,
      );
  }
  if (model.api === 'openai-completions') {
    /**
     Model narrowed by exact OpenAI Completions API assertion.
     */
    const selected = {
      model,
      api: 'openai-completions',
    } as const;
    assertModelUsesApi(selected,);
    return openAICompletionsApi()
      .streamSimple(
      isolateReviewModel(selected.model,),
      providerContext,
      providerOptions,
      );
  }
  if (model.api === 'openai-responses') {
    /**
     Model narrowed by exact OpenAI Responses API assertion.
     */
    const selected = {
      model,
      api: 'openai-responses',
    } as const;
    assertModelUsesApi(selected,);
    return openAIResponsesApi()
      .streamSimple(
      isolateReviewModel(selected.model,),
      providerContext,
      providerOptions,
      );
  }
  throw new Error(
    `No Pi AI API implementation available for structured reviewer ${model.provider}/${model.id} using ${model.api}. Supported APIs: ${SUPPORTED_APIS}`,
  );
}

export { streamStructuredReview, };
