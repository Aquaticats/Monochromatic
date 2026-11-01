/**
 Intercepted native Codex HTTP and finite response packets, never WebSockets. @module
 */
import { promisify, } from 'node:util';
import { zstdDecompress, } from 'node:zlib';
import {
  hasApi,
  type Provider,
  type StreamOptions,
  type Api,
  type Model,
  type TranscriptContext,
  type AssistantMessageEventStream,
  type SimpleStreamOptions,
} from '@earendil-works/pi-ai';
import {
  stream as nativeStream,
  streamSimple as nativeSimple,
} from '@earendil-works/pi-ai/api/openai-codex-responses';
import { openaiProvider, } from '@earendil-works/pi-ai/providers/openai';
import {
  stream as nativeOpenAIStream,
  streamSimple as nativeOpenAISimple,
} from '@earendil-works/pi-ai/api/openai-responses';
import {
  CODEX_PROVIDER,
  OPENAI_PROVIDER,
  OPENAI_API,
} from '../dist/final/node/index.mjs';
import { requireCodexModel, } from './host-fixture-model.ts';
import { fixtureProvider, } from './host-fixture-provider.ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

//region SSE fixtures: use the installed native parser and complete packet protocol.

/**
 Native Codex SSE bodies are zstd bytes when the installed host supports compression.
 */
const decompressZstd = promisify(zstdDecompress,);

/**
 Create a completed text or tool response accepted by native Codex streaming.

 @param text - synthetic assistant content for text-completion assertions

 @param tool - emit a completed tool call to exercise native continuation

 @returns finite SSE response with native lifecycle and usage packets

 @example
 ```ts
 const response = nativeResponse({ text: 'Offline reply.' });
 ```
 */
export function nativeResponse({
  text = 'Native fixture answer.',
  tool = false,
}: {
  readonly text?: string;
  readonly tool?: boolean;
} = {},): Response {
  /**
   Completed native output item drives either tool continuation or final text.
   */
  const item = tool
    ? {
      type: 'function_call',
      id: 'fc_fixture',
      call_id: 'call_fixture',
      name: 'fixture_tool',
      arguments: '{}',
      status: 'completed',
    }
    : {
      type: 'message',
      id: 'msg_fixture',
      role: 'assistant',
      status: 'completed',
      content: [{
        type: 'output_text',
        text,
        annotations: [],
      },],
    };
  /**
   Complete protocol includes creation, item lifecycle, and native terminal usage.
   */
  const events = [
    {
      type: 'response.created',
      response: { id: 'resp_fixture', },
    },
    {
      type: 'response.output_item.added',
      output_index: 0,
      item,
    },
    {
      type: 'response.output_item.done',
      output_index: 0,
      item,
    },
    {
      type: 'response.completed',
      response: {
        id: 'resp_fixture',
        status: 'completed',
        output: [item,],
        usage: {
          input_tokens: 10,
          output_tokens: 2,
          total_tokens: 12,
          input_tokens_details: { cached_tokens: 0, },
        },
      },
    },
  ];
  return new Response(
    events.map(function packet(event): string { return `data: ${JSON.stringify(event,)}\n\n`; },)
      .join('',),
    {
      status: 200,
      headers: { 'content-type': 'text/event-stream', },
    },
  );
}

/**
 Deliver native provider failure packets without depending on normalized fake errors.

 @param code - native failure identifier for parser and retry assertions

 @param message - synthetic diagnostic retained in canonical assistant history

 @returns finite SSE failure response for the installed native parser

 @example
 ```ts
 const response = nativeFailureResponse({ code: 'rate_limit_exceeded', message: 'Synthetic rate limit.' });
 ```
 */
export function nativeFailureResponse({
  code,
  message,
}: {
  readonly code: string;
  readonly message: string
},): Response {
  /**
   Native failure packet exercises pi's real parser and diagnostic propagation.
   */
  const event = {
    type: 'response.failed',
    response: {
      id: 'resp_fixture_failed',
      status: 'failed',
      output: [],
      error: {
        code,
        message,
      },
    },
  };
  return new Response(
    `data: ${JSON.stringify(event,)}\n\n`,
    {
      status: 200,
      headers: { 'content-type': 'text/event-stream', },
    },
  );
}

/**
 Captured final serialized HTTP input, including headers after registry auth resolution.
 */
export type FixtureHttpCall = {
  readonly url: string;
  readonly payload: unknown;
  readonly headers: Headers
};

/**
 Create a native provider whose HTTP adapter is bounded and cannot reach a real network.

 @param responses - ordered response factories whose length bounds request dispatch

 @param providerId - native provider identity retained through intercepted transport

 @returns captured final HTTP requests and synthetic provider with intercepted transport

 @example
 ```ts
 const http = fixtureHttp({ responses: [nativeResponse] });
 ```
 */
export function fixtureHttp({
  responses = [
    nativeResponse,
    nativeResponse,
  ],
  providerId = CODEX_PROVIDER,
}: {
  readonly responses?: readonly (() => Response)[];
  readonly providerId?: typeof CODEX_PROVIDER | typeof OPENAI_PROVIDER;
} = {},): {
  readonly requests: FixtureHttpCall[];
  readonly source: ReturnType<typeof fixtureProvider>
} {
  /**
   Synthetic provider records auth and native request dispatch independently.
   */
  const source = fixtureProvider({ dynamic: false, },);
  if (providerId === OPENAI_PROVIDER) {
    source.state
      .models = source.state
        .models
        .map(function nativeOpenAIModel(model: ForeignBorrowed<Model<Api>>,) {
          return {
            ...model,
            provider: OPENAI_PROVIDER,
            api: OPENAI_API,
            baseUrl: 'https://api.openai.com/v1',
          };
        },);
  }
  /**
   Final serialized HTTP inputs include registry-resolved headers.
   */
  const requests: FixtureHttpCall[] = [];
  /**
   Reject every non-fixture endpoint and unexpected additional request.
   
   @param input - fetch URL form dictated by the native transport
   
   @param init - final native request supplied by pi
   
   @returns finite synthetic response through fetch's asynchronous contract
   */
  async function fetchFixture({
    input,
    init,
  }: {
    readonly input: ForeignBorrowed<RequestInfo | URL>;
    readonly init?: ForeignBorrowed<RequestInit>;
  },): Promise<Response> {
    /**
     URL forms are narrowed explicitly instead of coerced from arbitrary objects.
     */
    const url = input instanceof Request ? input.url : input instanceof URL ? input.href : input;
    if (!(providerId === OPENAI_PROVIDER ? url.startsWith('https://api.openai.com/v1/',) : url.startsWith('https://host-fixture.invalid/',)))
      throw new Error(`Native fixture attempted an unexpected endpoint: ${url}`,);
    /**
     Native transport may serialize JSON directly or compress it into zstd bytes.
     */
    const body = init?.body;
    if (((typeof body) !== 'string') && (!(body instanceof Uint8Array)))
      throw new Error('Native fixture expected serialized JSON or native compressed bytes.',);
    /**
     Registry-resolved native headers also identify the serialized body encoding.
     */
    const headers = new Headers(init?.headers,);
    /**
     Decode the actual final native request rather than disabling its compression path.
     */
    const serialized = (typeof body) === 'string' ? body
      : headers.get('content-encoding',) === 'zstd' ? (await decompressZstd(body,)).toString('utf8',)
        : new TextDecoder().decode(body,);
    /**
     Response budget bounds the fixture and makes fallback requests observable.
     */
    const respond = responses[requests.length];
    if (respond === undefined)
      throw new Error('Native fixture exceeded its finite response budget.',);
    requests.push({
      url,
      payload: JSON.parse(serialized,),
      headers,
    },);
    init?.signal
      ?.throwIfAborted();
    return await Promise.resolve(respond(),);
  }
  /**
   Foreign fetch callback preserves native positional arguments at the transport boundary.
   */
  const transport = {
    fetch: function fetch(
      this: void,
      input: ForeignBorrowed<RequestInfo | URL>,
      init?: ForeignBorrowed<RequestInit>,
    ): Promise<Response> {
      return fetchFixture({
        input,
        ...(init === undefined ? {} : { init, }),
      },);
    },
  };
  /**
   Native Codex adapter is intercepted only at its HTTP capability boundary.
   */
  const provider: Provider = {
    ...source.provider,
    id: providerId,
    ...(providerId === OPENAI_PROVIDER ? {
      auth: openaiProvider()
        .auth,
      baseUrl: 'https://api.openai.com/v1',
    } : {}),
    stream: function stream(
      model: ForeignBorrowed<Model<Api>>,
      context: ForeignBorrowed<TranscriptContext>,
      options?: ForeignBorrowed<StreamOptions>,
    ): AssistantMessageEventStream {
      source.state
        .calls
        .push({
          kind: 'full',
          model,
          context,
          ...(options === undefined ? {} : { options, }),
        },);
      if (hasApi(
        model,
        OPENAI_API,
      ))
        return nativeOpenAIStream(
          model,
          context,
          {
            ...options,
            fetch: transport.fetch,
            maxRetries: 0,
          },
        );
      return nativeStream(
        requireCodexModel(model,),
        context,
        {
          ...options,
          fetch: transport.fetch,
          transport: 'sse',
          maxRetries: 0,
        },
      );
    },
    streamSimple: function streamSimple(
      model: ForeignBorrowed<Model<Api>>,
      context: ForeignBorrowed<TranscriptContext>,
      options?: ForeignBorrowed<SimpleStreamOptions>,
    ): AssistantMessageEventStream {
      source.state
        .calls
        .push({
          kind: 'simple',
          model,
          context,
          ...(options === undefined ? {} : { options, }),
        },);
      if (hasApi(
        model,
        OPENAI_API,
      ))
        return nativeOpenAISimple(
          model,
          context,
          {
            ...options,
            fetch: transport.fetch,
            maxRetries: 0,
          },
        );
      return nativeSimple(
        requireCodexModel(model,),
        context,
        {
          ...options,
          fetch: transport.fetch,
          transport: 'sse',
          maxRetries: 0,
        },
      );
    },
  };
  return {
    requests,
    source: {
      provider,
      state: source.state,
    },
  };
}

//endregion SSE fixtures
