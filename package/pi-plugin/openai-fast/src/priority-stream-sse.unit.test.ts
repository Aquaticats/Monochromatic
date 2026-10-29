/**
 Built wrappers retain native SSE request serialization, observers, and priority accounting. @module
 */
import type {
  Api,
  Model,
  ProviderResponse,
} from '@earendil-works/pi-ai';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  streamPriority,
  streamSimplePriority,
} from '../dist/final/node/index.mjs';
import {
  createStreamFixtureHttp,
  streamFixtureRequestUrl,
} from './stream-fixture-http.ts';
import {
  createStreamFixtureContext,
  createStreamFixtureModel,
  STREAM_FIXTURE_TOKEN,
} from './stream-fixture.ts';

await describe({
  name: '',
  children: [
    //region Input URL representations: avoid implicit object stringification.

    it({
      name: 'native fetch fixture explicitly narrows string, URL, and Request input',
      fn: async function nativeInputUrl() {
        /**
         Synthetic endpoint exercises every native fetch URL representation without networking.
         */
        const url = await Promise.resolve('https://stream-fixture.invalid/backend-api/codex/responses',);
        /**
         Native URL object exercises explicit href extraction.
         */
        const urlObject = new URL(url,);
        /**
         Native Request object exercises explicit url extraction.
         */
        const request = new Request(url,);
        expect(streamFixtureRequestUrl(url,),).toBe(url,);
        expect(streamFixtureRequestUrl(urlObject,),).toBe(url,);
        expect(streamFixtureRequestUrl(request,),).toBe(url,);
      },
    },),
    //endregion Input URL representations

    //region Native SSE: real response processing preserves requested priority accounting.

    describe({
      name: 'native Codex SSE accounting',
      children: ([streamPriority, streamSimplePriority,] as const).flatMap(function entryCases(entryPoint) {
        return ([undefined, 'default',] as const).map(function tierCase(serviceTier) {
          return it({
            name: `${entryPoint.name} sends priority replacement and preserves native ${String(serviceTier,)} tier accounting`,
            fn: async function nativeAccounting(ctx) {
              /**
               Original model rates and configured headers remain authoritative for native processing.
               */
              const model = createStreamFixtureModel({
                headers: {
                  'x-stream-fixture-model': 'original',
                  'x-stream-fixture-remove': 'remove',
                },
              },);
              /**
               Independent transcript exercises the real native request builder.
               */
              const context = createStreamFixtureContext();
              /**
               Replacement retains native fields but attempts to lower the requested service tier.
               */
              const onPayload = ctx.sinon.spy(async function replaceNativePayload(
                payload: unknown,
                receivedModel: ForeignBorrowed<Model<Api>>,
              ) {
                expect(receivedModel,).toBe(model,);
                return await Promise.resolve({
                  ...(payload as Readonly<Record<string, unknown>>),
                  service_tier: 'flex',
                  fixture_replacement: 'observed',
                },);
              },);
              /**
               Response observer must receive native status and original live model identity.
               */
              const onResponse = ctx.sinon.spy(async function observeResponse(
                response: ForeignBorrowed<ProviderResponse>,
                receivedModel: ForeignBorrowed<Model<Api>>,
              ) {
                expect(response.status,).toBe(200,);
                expect(receivedModel,).toBe(model,);
                await Promise.resolve();
              },);
              /**
               Provider event observer remains attached before native result normalization.
               */
              const onProviderStreamEvent = ctx.sinon.spy(async function observeProviderEvent(
                data: unknown,
                receivedModel: ForeignBorrowed<Model<Api>>,
              ) {
                expect(data,).toHaveProperty('type', 'response.completed',);
                expect(receivedModel,).toBe(model,);
                await Promise.resolve();
              },);
              /**
               Finite local interceptor consumes real native serialized and possibly compressed body.
               */
              const http = createStreamFixtureHttp({
                model,
                ...(serviceTier === undefined ? {} : { serviceTier, }),
              },);
              /**
               Installed native SSE stream processes local response events without paid APIs.
               */
              const result = await entryPoint({
                model,
                context,
                options: {
                  apiKey: STREAM_FIXTURE_TOKEN,
                  transport: 'sse',
                  fetch: http.fetch,
                  onPayload,
                  onResponse,
                  onProviderStreamEvent,
                  signal: new AbortController().signal,
                  headers: {
                    'x-stream-fixture-runtime': 'runtime',
                    'x-stream-fixture-remove': null,
                  },
                  cacheRetention: 'long',
                  sessionId: 'stream-fixture-sse-session',
                  timeoutMs: 5_000,
                  websocketConnectTimeoutMs: 2_000,
                  maxRetries: 0,
                  maxRetryDelayMs: 5_000,
                },
              },).result();
              expect(result.stopReason,).toBe('stop',);
              expect(result.errorMessage,).toBeUndefined();
              expect(http.requests,).toHaveLength(1,);
              /**
               Single captured request proves no ordinary-tier retry or alternate dispatch occurred.
               */
              const [request,] = http.requests;
              if (request === undefined)
                throw new Error('Expected one native serialized SSE request',);
              expect(request.url,).toBe(`${model.baseUrl}/codex/responses`,);
              expect(request.method,).toBe('POST',);
              expect(request.headers.get('x-stream-fixture-model',),).toBe('original',);
              expect(request.headers.get('x-stream-fixture-runtime',),).toBe('runtime',);
              expect(request.headers.get('x-stream-fixture-remove',),).toBeNull();
              expect(request.headers.get('session-id',),).toBe('stream-fixture-sse-session',);
              expect(request.headers.get('authorization',),).toBe(`Bearer ${STREAM_FIXTURE_TOKEN}`,);
              expect(request.headers.get('chatgpt-account-id',),).toBe('stream-fixture-account',);
              expect(request.payload,).toMatchObject({
                model: model.id,
                service_tier: 'priority',
                fixture_replacement: 'observed',
              },);
              expect(onPayload,).toHaveBeenCalledTimes(1,);
              expect(onResponse,).toHaveBeenCalledTimes(1,);
              /**
               Native response contains both content type and fixture header, not a partial nested object.
               */
              const [response, responseModel,] = onResponse.firstCall.args;
              expect(responseModel,).toBe(model,);
              expect(response.status,).toBe(200,);
              expect(response.headers,).toHaveProperty('content-type', 'text/event-stream',);
              expect(response.headers,).toHaveProperty('x-stream-fixture-response', 'observed',);
              expect(onProviderStreamEvent,).toHaveBeenCalledTimes(1,);
              /**
               Original provider event and live model identities survive wrapper delegation.
               */
              const [event, eventModel,] = onProviderStreamEvent.firstCall.args;
              expect(eventModel,).toBe(model,);
              expect(event,).toHaveProperty('type', 'response.completed',);
              expect(result.model,).toBe(model.id,);
              expect(result.usage.input,).toBe(90,);
              expect(result.usage.output,).toBe(20,);
              expect(result.usage.cacheRead,).toBe(10,);
              expect(result.usage.cost.input,).toBeCloseTo(0.00018, 10,);
              expect(result.usage.cost.output,).toBeCloseTo(0.00008, 10,);
              expect(result.usage.cost.cacheRead,).toBeCloseTo(0.00001, 10,);
              expect(result.usage.cost.total,).toBeCloseTo(0.00027, 10,);
            },
          },);
        },);
      },),
    },),
    //endregion Native SSE
  ],
},);
