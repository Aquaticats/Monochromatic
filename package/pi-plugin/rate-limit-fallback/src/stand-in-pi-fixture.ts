/**
 Test-only providers and extension API. No transport in this module can reach the network.
 @module
 */

import {
  createAssistantMessageEventStream,
  normalizeContext,
  type Api,
  type AssistantMessage,
  type AssistantMessageEvent,
  type AssistantMessageEventStream,
  type FetchFunction,
  type Model,
  type Provider,
  type SimpleStreamOptions,
  type TranscriptContext,
} from '@earendil-works/pi-ai';
import type { ExtensionAPI, ExtensionHandler, SessionStartEvent, } from '@earendil-works/pi-coding-agent';

//region Fixtures for real Pi shapes

/** Stand-in credential, intentionally different from the virtual provider's caller-supplied key. */
export const MEMBER_KEY = 'stand-in-member-key';

/** Independently pin each expected setup, including authenticated quota origins. */
export const TARGETS = [
  { provider: 'openai', id: 'gpt-6.2-sol', baseUrl: 'https://api.openai.com/v1', },
  {
    provider: 'qwen-token-plan-individual',
    id: 'deepseek-v4.1-flash-1017',
    baseUrl: 'https://qwen.example/v1',
  },
  {
    provider: 'synthetic',
    id: 'hf:deepseek-ai/DeepSeek-V4.1-Flash-1017',
    baseUrl: 'https://api.synthetic.new/v2',
  },
  { provider: 'opencode-go', id: 'deepseek-v4.1-flash', baseUrl: 'https://opencode.ai/inference/go/v1', },
  { provider: 'neuralwatt', id: 'deepseek-v4.1-flash-flex', baseUrl: 'https://api.neuralwatt.com/v1', },
] as const;

/** Transcript that must survive fallback unchanged. */
export const TRANSCRIPT: TranscriptContext = normalizeContext({
  messages: [
    { role: 'system', content: 'Fixture system instruction', timestamp: 0, },
    { role: 'user', content: 'Fixture request', timestamp: 0, },
  ],
},);

/** Catalog records used by both stand-in registry and SDK fixtures. */
export const PHYSICAL_MODELS: readonly Model<Api>[] = TARGETS.map(function toModel(target,) {
  return {
    ...target,
    name: target.id,
    api: 'test-only-chat',
    input: ['text',],
    cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, },
    reasoning: false,
    contextWindow: 16_000,
    maxTokens: 256,
  };
},);

/**
 Build a physical answer using the identity from the registry, just as Pi adapters do.
 @param model - provider and model recorded on the response
 @param errorMessage - terminal refusal when supplied
 @returns physical assistant message
 */
export function assistant({
  model,
  errorMessage,
}: {
  model: Model<Api>;
  errorMessage?: string;
},): AssistantMessage {
  return {
    role: 'assistant',
    provider: model.provider,
    model: model.id,
    api: model.api,
    content: [],
    usage: {
      input: 0, output: 0, cacheRead: 0, cacheWrite: 0, totalTokens: 0,
      cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, total: 0, },
    },
    stopReason: errorMessage === undefined ? 'stop' : 'error',
    ...(errorMessage === undefined ? {} : { errorMessage, }),
    timestamp: Date.now(),
  };
}

/**
 Finish a test stream with a normalized result.
 @param stream - destination physical stream
 @param message - result to record and expose
 */
export function finish({
  stream,
  message,
}: {
  stream: AssistantMessageEventStream;
  message: AssistantMessage;
},): void {
  if (message.stopReason === 'pending')
    throw new Error('Test message must be terminal',);
  stream.push(message.stopReason === 'error' || message.stopReason === 'aborted'
    ? { type: 'error', reason: message.stopReason, error: message, }
    : { type: 'done', reason: message.stopReason, message, },);
  stream.end(message,);
}

/**
 Read the events as well as the terminal result, so a hidden duplicate start is detectable.
 @param stream - stream under test
 @returns ordered events and final message
 */
export async function collect(stream: AssistantMessageEventStream,): Promise<{
  readonly events: AssistantMessageEvent[];
  readonly result: AssistantMessage;
}> {
  const events: AssistantMessageEvent[] = [];
  for await (const event of stream)
    events.push(event,);
  return { events, result: await stream.result(), };
}

//endregion Fixtures for real Pi shapes

//region Stand-in extension host

/** What the member received from the pool without authentication. */
export type MemberCall = {
  readonly model: Model<Api>;
  readonly context: TranscriptContext;
  readonly options: SimpleStreamOptions | undefined;
};

/**
 Build a fake Pi host that exposes its live registry on session start.
 @param extension - built Pi factory
 @param play - scripted member behavior
 @param models - current registry catalog
 @param fetchQuota - scripted quota responses, defaults to unavailable
 @param resolvedBaseUrl - registry auth endpoint override
 @returns provider, request history and controls for the fixture
 */
export async function fixture({
  extension,
  play,
  models = PHYSICAL_MODELS,
  fetchQuota = async function quotaUnavailable() { return new Response(null, { status: 503, },); },
  resolvedBaseUrl,
}: {
  extension: (pi: ExtensionAPI) => void;
  play: (call: MemberCall) => AssistantMessageEventStream;
  models?: readonly Model<Api>[];
  fetchQuota?: FetchFunction;
  resolvedBaseUrl?: string;
},): Promise<{
  readonly calls: MemberCall[];
  readonly provider: Provider;
  readonly options: SimpleStreamOptions;
}> {
  const calls: MemberCall[] = [];
  let provider: Provider | undefined;
  let start: ExtensionHandler<SessionStartEvent> | undefined;
  extension({
    registerProvider(value: Provider,) {
      provider = value;
    },
    on(_event: string, callback: ExtensionHandler<SessionStartEvent>,) {
      start = callback;
      return () => {};
    },
  } as unknown as ExtensionAPI,);
  if (provider === undefined || start === undefined)
    throw new Error('Expected provider registration and session binding',);
  const registry = {
    find(providerId: string, id: string,) {
      return models.find(model => model.provider === providerId && model.id === id,);
    },
    async getApiKeyAndHeaders() {
      return { ok: true, apiKey: MEMBER_KEY, baseUrl: resolvedBaseUrl, };
    },
    streamSimple(model: Model<Api>, context: TranscriptContext, options?: SimpleStreamOptions,) {
      const call = { model, context, options, };
      calls.push(call,);
      return play(call,);
    },
  };
  await start({ type: 'session_start', reason: 'startup', }, {
    modelRegistry: registry,
  } as unknown as Parameters<typeof start>[1],);
  return { calls, provider, options: { fetch: fetchQuota, }, };
}

/**
 Emit a completed response or refusal without using a network.
 @param model - physical model
 @param errorMessage - refusal text when fixture is not a success
 @returns stream with a start and a single terminal event
 */
export function immediate({
  model,
  errorMessage,
}: {
  model: Model<Api>;
  errorMessage?: string;
},): AssistantMessageEventStream {
  const stream = createAssistantMessageEventStream();
  const message = assistant({ model, ...(errorMessage === undefined ? {} : { errorMessage, }), },);
  stream.push({ type: 'start', partial: message, },);
  finish({ stream, message, },);
  return stream;
}

/**
 Select a catalog model supplied by the extension.
 @param provider - registered pool provider
 @param id - id to verify and retrieve
 @returns registered chat model
 */
export function poolModel({ provider, id, }: { provider: Provider; id: string; },): Model<Api> {
  const model = provider.getModels().find(function matches(candidate,): candidate is Model<Api> {
    return (candidate.type === undefined || candidate.type === 'chat') && candidate.id === id;
  },);
  if (model === undefined)
    throw new Error(`Missing pool model ${id}`,);
  return model;
}

//endregion Stand-in extension host
