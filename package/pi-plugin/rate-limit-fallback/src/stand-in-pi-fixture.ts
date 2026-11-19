/**
 Test-only providers and extension API. No transport in this module can reach the network.
 @module
 */

import { streamSimple as streamResponses, } from '@earendil-works/pi-ai/api/openai-responses';
import {
  createAssistantMessageEventStream,
  normalizeContext,
  type Api,
  type AssistantMessage,
  type AssistantMessageEvent,
  type AssistantMessageEventStream,
  type Context,
  type FetchFunction,
  type Model,
  type Provider,
  type ProviderHeaders,
  type SimpleStreamOptions,
  type TranscriptContext,
} from '@earendil-works/pi-ai';
import type {
  ModelRegistry,
} from '@earendil-works/pi-coding-agent';
import type builtExtension from '../dist/final/node/index.mjs';

/**
 Only the host capabilities requested by the built extension.
 */
type FallbackHost = Parameters<typeof builtExtension>[0];

/**
 Session handler passed to the stand-in host.
 */
type SessionBinding = Parameters<FallbackHost['on']>[1];

//region Fixtures for real Pi shapes

/**
 Stand-in credential, intentionally different from the virtual provider's caller-supplied key.
 */
export const MEMBER_KEY = 'stand-in-member-key';

/**
 HTTP status used in both adapter and response-hook fixtures.
 */
export const RATE_LIMIT_STATUS = 429;

/**
 Independently pin each expected setup, including authenticated quota origins.
 */
export const TARGETS = [
  {
    provider: 'openai',
    api: 'openai-responses',
    id: 'gpt-6.2-sol',
    baseUrl: 'https://api.openai.com/v1',
  },
  {
    provider: 'qwen-token-plan-individual',
    api: 'openai-completions',
    id: 'deepseek-v4.1-flash-1017',
    baseUrl: 'https://qwen.example/v1',
  },
  {
    provider: 'synthetic',
    api: 'openai-completions',
    id: 'hf:deepseek-ai/DeepSeek-V4.1-Flash-1017',
    baseUrl: 'https://api.synthetic.new/v2',
  },
  {
    provider: 'opencode-go',
    api: 'openai-completions',
    id: 'deepseek-v4.1-flash',
    baseUrl: 'https://opencode.ai/inference/go/v1',
  },
  {
    provider: 'neuralwatt',
    api: 'openai-completions',
    id: 'deepseek-v4.1-flash-flex',
    baseUrl: 'https://api.neuralwatt.com/v1',
  },
] as const;

/**
 Transcript that must survive fallback unchanged.
 */
export const TRANSCRIPT: TranscriptContext = normalizeContext({
  messages: [
    {
      role: 'system',
      content: 'Fixture system instruction',
      timestamp: 0,
    },
    {
      role: 'user',
      content: 'Fixture request',
      timestamp: 0,
    },
  ],
},);

/**
 Catalog records used by both stand-in registry and SDK fixtures.
 */
export const PHYSICAL_MODELS: readonly Model<Api>[] = TARGETS.map(function toModel(target,) {
  return {
    ...target,
    name: target.id,
    input: ['text',],
    cost: {
      input: 0,
      output: 0,
      cacheRead: 0,
      cacheWrite: 0,
    },
    reasoning: false,
    contextWindow: 16_000,
    maxTokens: 256,
  };
},);

/**
 Model fields the fake assistant response uses, without holding a writable catalog entry.
 */
export type ModelIdentity = Pick<Readonly<Model<Api>>, 'api' | 'id' | 'provider'>;

/**
 Compose a refusal body in the forms produced by Pi's OpenAI Responses and Completions adapters.

 @param model - API selection that formats the refusal

 @param message - provider's error text inside the HTTP body

 @returns terminal SDK error text

 @example
 ```ts
 rateLimitRefusal({ model: { api: 'openai-responses' } });
 ```
 */
export function rateLimitRefusal({
  model,
  message = 'Too many requests',
}: {
  readonly model: Pick<ModelIdentity, 'api'>;
  readonly message?: string;
},): string {
  /**
   The OpenAI SDK unwraps the HTTP body's `error` field before Pi serializes it.
   */
  const body = JSON.stringify({ message, },);
  return model.api === 'openai-responses'
    ? `OpenAI API error (${RATE_LIMIT_STATUS}): ${body}`
    : `${RATE_LIMIT_STATUS}: ${body}`;
}

/**
 Build a physical answer using the identity from the registry, just as Pi adapters do.

 @param model - provider and model recorded on the response

 @param errorMessage - terminal refusal when supplied

 @returns physical assistant message

 @example
 ```ts
 const answer = assistant({ model });
 ```
 */
export function assistant({
  model,
  errorMessage,
}: {
  readonly model: ModelIdentity;
  readonly errorMessage?: string;
},): AssistantMessage {
  return {
    role: 'assistant',
    provider: model.provider,
    model: model.id,
    api: model.api,
    content: [],
    usage: {
      input: 0,
      output: 0,
      cacheRead: 0,
      cacheWrite: 0,
      totalTokens: 0,
      cost: {
        input: 0,
        output: 0,
        cacheRead: 0,
        cacheWrite: 0,
        total: 0,
      },
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

 @example
 ```ts
 finish({ stream, message: assistant({ model }) });
 ```
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
  stream.push((message.stopReason === 'error') || (message.stopReason === 'aborted')
    ? {
      type: 'error',
      reason: message.stopReason,
      error: message,
    }
    : {
      type: 'done',
      reason: message.stopReason,
      message,
    },);
  stream.end(message,);
}

/**
 Read the events as well as the terminal result, so a hidden duplicate start is detectable.

 @param stream - stream under test

 @returns ordered events and final message

 @example
 ```ts
 const { events, result } = await collect(stream);
 ```
 */
export async function collect(stream: AssistantMessageEventStream,): Promise<{
  readonly events: AssistantMessageEvent[];
  readonly result: AssistantMessage;
}> {
  /**
   Normalized events observed by the consumer in order.
   */
  const events: AssistantMessageEvent[] = [];
  for await (const event of stream)
    events.push(event,);
  return {
    events,
    result: await stream.result(),
  };
}

//endregion Fixtures for real Pi shapes

//region Policy stop built by the installed adapter

/**
 OpenAI's policy stop, verbatim as two recorded runs on this machine report it:
 `pi-runs/prewarm-of-a-new-worktree-acts-only-on-the-entry-it-checked-1/summary.json`
 and `pi-codemode-abort-crash-troubleshooting-2026-11-18/run-1/summary.json`,
 both under `${HOME}/temp/agent`, field `modelError`.
 */
export const POLICY_STOP_TEXT = 'This content was flagged for possible cybersecurity risk. If this seems wrong, '
  + 'try rephrasing your request. If you’re doing authorized security work that requires more cyber permissive '
  + 'safeguards, apply for Daybreak access via https://platform.openai.com/settings/organization/status-and-access '
  + 'before retrying.';

/**
 Serialize provider events as the OpenAI SDK's server-sent-event body reads them.

 @param events - Events the adapter consumes in order

 @returns response body with one `event` and one `data` line per event
 */
function serverSentEvents(events: readonly {
  readonly type: string;
}[]): string {
  return events.map(function toFrame(event,) {
    return `event: ${event.type}\ndata: ${JSON.stringify(event,)}\n\n`;
  },)
    .join('',);
}

/**
 Build OpenAI's policy stop through the installed Responses adapter, over an in-memory
 stream that answered HTTP 200 as the recorded runs observed.

 @param model - physical model whose adapter reads the body

 @param context - transcript the adapter is asked about

 @param options - request options the pool forwarded to the member

 @param exposedText - text the member streams before the stop, or empty to stop before any output

 @returns physical stream that fails with the policy stop

 @example
 ```ts
 policyStopStream({ model: PHYSICAL_MODELS[0], context: TRANSCRIPT, exposedText: 'partial' });
 ```
 */
export function policyStopStream({
  model,
  context,
  options,
  exposedText = '',
}: {
  readonly model: Model<Api>;
  readonly context: TranscriptContext;
  readonly options?: SimpleStreamOptions;
  readonly exposedText?: string;
},): AssistantMessageEventStream {
  /**
   Events the SDK reads: a created response, any exposed text, then the stop.
   */
  const events = [
    {
      type: 'response.created',
      response: {
        id: 'resp_policy',
        status: 'in_progress',
      },
    },
    ...(exposedText === '' ? [] : [
      {
        type: 'response.output_item.added',
        output_index: 0,
        item: {
          type: 'message',
          id: 'msg_policy',
          role: 'assistant',
          status: 'in_progress',
          content: [],
        },
      },
      {
        type: 'response.output_text.delta',
        output_index: 0,
        content_index: 0,
        item_id: 'msg_policy',
        delta: exposedText,
      },
    ]),
    {
      type: 'error',
      code: 'invalid_request_error',
      message: POLICY_STOP_TEXT,
    },
  ];
  /**
   One in-memory body; the adapter reaches no network.
   */
  const body = serverSentEvents(events,);
  return streamResponses(
    model,
    context,
    {
      ...options,
      apiKey: MEMBER_KEY,
      maxRetries: 0,
      fetch() {
        return Promise.resolve(new Response(
          body,
          {
            status: 200,
            headers: { 'content-type': 'text/event-stream', },
          },
        ),);
      },
    },
  );
}

//endregion Policy stop built by the installed adapter

//region Stand-in extension host

/**
 What the member received from the pool without authentication.
 */
export type MemberCall = {
  readonly model: Model<Api>;
  readonly context: Context;
  readonly options?: SimpleStreamOptions;
};

/**
 Return the default fixture quota response when a test does not script quota data.

 @returns unavailable quota response without touching the network
 */
function quotaUnavailable(): Promise<Response> {
  return Promise.resolve(new Response(
    null,
    { status: 503, },
  ),);
}

/**
 Build a fake Pi host that exposes its live registry on session start.

 @param extension - built Pi factory

 @param play - scripted member behavior

 @param models - current registry catalog

 @param fetchQuota - scripted quota responses, defaults to unavailable

 @param resolvedBaseUrl - registry auth endpoint override

 @param resolvedHeaders - header-only registry credential when supplied

 @param resolveAuth - registry authorization override for error-path tests

 @param findModel - catalog lookup override for registry-contract tests

 @param startSession - whether the registration has received a session binding

 @returns provider, request history and controls for the fixture

 @example
 ```ts
 const host = await fixture({ extension, play: ({ model }) => immediate({ model }) });
 ```
 */
export function fixture({
  extension,
  play,
  models = PHYSICAL_MODELS,
  fetchQuota = quotaUnavailable,
  resolvedBaseUrl,
  resolvedHeaders,
  resolveAuth,
  findModel,
  startSession = true,
}: {
  extension: typeof builtExtension;
  play: (call: MemberCall) => AssistantMessageEventStream;
  models?: readonly Model<Api>[];
  fetchQuota?: FetchFunction;
  resolvedBaseUrl?: string;
  resolvedHeaders?: ProviderHeaders;
  resolveAuth?: ModelRegistry['getApiKeyAndHeaders'];
  findModel?: ModelRegistry['find'];
  startSession?: boolean;
},): Promise<{
  readonly calls: MemberCall[];
  readonly provider: Provider;
  readonly options: SimpleStreamOptions;
}> {
  /**
   Physical requests, including discarded attempts, captured by the fixture.
   */
  const calls: MemberCall[] = [];
  /**
   Owner of registrations received from the factory.
   */
  const registration: {
    /**
     Provider captured by the fake host.
     */
    provider?: Provider;
    /**
     Handler captured for the active session.
     */
    start?: SessionBinding;
  } = {};
  extension({
    registerProvider(value: Provider,) {
      registration.provider = value;
    },
    on(
      _event: string,
      handler: SessionBinding,
    ) {
      registration.start = handler;
      return function unsubscribe(): void {
        delete registration.start;
      };
    },
  },);
  /**
   Extension entry points required before starting the stand-in session.
   */
  const {
    provider,
    start,
  } = registration;
  if ((provider === undefined) || (start === undefined))
    throw new Error('Expected provider registration and session binding',);
  /**
   Test implementation of the extension's registry boundary.
   */
  const registry: Pick<ModelRegistry, 'find' | 'getApiKeyAndHeaders' | 'streamSimple'> = {
    find(
      providerId,
      id,
    ) {
      if (findModel !== undefined)
        return findModel(
          providerId,
          id,
        );
      for (const model of models) {
        if ((model.provider === providerId) && (model.id === id))
          return model;
      }
      return undefined;
    },
    getApiKeyAndHeaders(model,) {
      if (resolveAuth !== undefined)
        return resolveAuth(model,);
      return Promise.resolve({
        ok: true,
        ...(resolvedHeaders === undefined ? { apiKey: MEMBER_KEY, } : { headers: resolvedHeaders, }),
        ...(resolvedBaseUrl === undefined ? {} : { baseUrl: resolvedBaseUrl, }),
      },);
    },
    streamSimple(
      model,
      context,
      options,
    ) {
      /**
       Captured physical request delivered to the scripted member.
       */
      const call = {
        model,
        context,
        ...(options === undefined ? {} : { options, }),
      };
      calls.push(call,);
      return play(call,);
    },
  };
  if (startSession)
    start(
      {
        type: 'session_start',
        reason: 'startup',
      },
      {
        modelRegistry: registry,
      },
    );
  return Promise.resolve({
    calls,
    provider,
    options: { fetch: fetchQuota, },
  },);
}

/**
 Emit a completed response or refusal without using a network.

 @param model - physical model

 @param errorMessage - refusal text when fixture is not a success

 @returns stream with a start and a single terminal event

 @example
 ```ts
 const stream = immediate({ model, errorMessage: '429 fixture limit' });
 ```
 */
export function immediate({
  model,
  errorMessage,
}: {
  readonly model: ModelIdentity;
  readonly errorMessage?: string;
},): AssistantMessageEventStream {
  /**
   Normalized stream whose terminal event decides whether to stop or hand off.
   */
  const stream = createAssistantMessageEventStream();
  /**
   Member-attributed result sent on the fixture stream.
   */
  const message = assistant({
    model,
    ...(errorMessage === undefined ? {} : { errorMessage, }),
  },);
  stream.push({
    type: 'start',
    partial: message,
  },);
  finish({
    stream,
    message,
  },);
  return stream;
}

/**
 Select a catalog model supplied by the extension.

 @param provider - registered pool provider

 @param id - id to verify and retrieve

 @returns registered chat model

 @example
 ```ts
 const writer = poolModel({ provider, id: 'writer' });
 ```
 */
export function poolModel({
  provider,
  id,
}: {
  provider: Provider;
  id: string
},): Model<Api> {
  for (const candidate of provider.getModels()) {
    if (((candidate.type === undefined) || (candidate.type === 'chat')) && (candidate.id === id))
      return candidate;
  }
  throw new Error(`Missing pool model ${id}`,);
}

//endregion Stand-in extension host
