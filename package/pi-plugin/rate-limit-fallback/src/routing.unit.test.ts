/**
 Guard tests for ordered request handoff and lossless terminal records.
 @module
 */

import { createAssistantMessageEventStream, lazyStream, } from '@earendil-works/pi-ai';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import extension, { PROVIDER_ID, } from '../dist/final/node/index.mjs';
import {
  assistant,
  collect,
  finish,
  fixture,
  immediate,
  poolModel,
  TARGETS,
  TRANSCRIPT,
} from './stand-in-pi-fixture.ts';

/** Stand-in HTTP refusal in Pi SDK error format. */
const LIMITED = '429 Too many requests';

/** Explicit policy refusal also containing a misleading HTTP 429 prefix. */
const POLICY = '429 Request flagged as a possible risk';

/** Non-capacity failures are never retried down the pool. */
const TERMINAL_ERRORS = [
  POLICY,
  '401 Unauthorized: contact support with request 429',
  'Request timed out',
  '400 Context window exceeded',
] as const;

await describe({
  name: 'ordered provider routing',
  children: [
    it({
      name: 'writer tries the exact pool and keeps only the answering member events',
      fn: async function writerOrder() {
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, }) {
            return immediate({
              model,
              ...(model.provider === TARGETS.at(-1)?.provider ? {} : { errorMessage: LIMITED, }),
            },);
          },
        },);
        expect(provider.id,).toBe(PROVIDER_ID,);
        expect(provider.getModels().map(model => model.id,),).toEqual(['writer', 'reviewer',],);
        const model = poolModel({ provider, id: 'writer', },);
        const { events, result, } = await collect(provider.stream(model, TRANSCRIPT, {
          ...options,
          maxTokens: Number.MAX_SAFE_INTEGER,
        },),);
        expect(calls.map(call => [call.model.provider, call.model.id,],),).toEqual(
          TARGETS.map(target => [target.provider, target.id,],),
        );
        expect(calls.every(call => call.context === TRANSCRIPT && call.options?.maxTokens === call.model.maxTokens,),)
          .toBe(true,);
        expect(events.map(event => event.type,),).toEqual(['start', 'done',],);
        expect([result.provider, result.model,],).toEqual([TARGETS.at(-1)?.provider, TARGETS.at(-1)?.id,],);
      },
    },),
    it({
      name: 'reviewer starts with Qwen, retries Qwen on the next request, and never forwards outer credentials',
      fn: async function reviewerRetry() {
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, }) {
            return calls.length === 1 ? immediate({ model, errorMessage: LIMITED, },) : immediate({ model, },);
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const requestOptions = {
          ...options,
          apiKey: 'stand-in-outer-key',
          headers: { authorization: 'stand-in-outer-header', },
          env: { CREDENTIAL_MARKER: 'stand-in-outer-env', },
        };
        const first = await collect(provider.streamSimple(model, TRANSCRIPT, requestOptions,),);
        const second = await collect(provider.streamSimple(model, TRANSCRIPT, requestOptions,),);
        expect([first.result.provider, second.result.provider,],).toEqual(['synthetic', 'qwen-token-plan-individual',],);
        expect(calls.map(call => call.model.provider,),).toEqual([
          'qwen-token-plan-individual', 'synthetic', 'qwen-token-plan-individual',
        ],);
        for (const { options: actual, } of calls) {
          expect(actual?.apiKey,).toBeUndefined();
          expect(actual?.headers,).toBeUndefined();
          expect(actual?.env,).toBeUndefined();
        }
      },
    },),
    ...TERMINAL_ERRORS.map(function terminalError(errorMessage,) {
      return it({
        name: `does not move on from ${errorMessage}`,
        fn: async function terminalStop() {
          const { provider, calls, options, } = await fixture({
            extension,
            play({ model, }) { return immediate({ model, errorMessage, },); },
          },);
          const model = poolModel({ provider, id: 'writer', },);
          const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          expect(result.errorMessage,).toBe(errorMessage,);
          expect(calls.map(call => call.model.provider,),).toEqual(['openai',],);
        },
      },);
    },),
    it({
      name: 'does not replay after a tool event or when the terminal 429 already contains content',
      fn: async function partialOutput() {
        for (const publish of [true, false,]) {
          const { provider, calls, options, } = await fixture({
            extension,
            play({ model, }) {
              const stream = createAssistantMessageEventStream();
              const message = assistant({ model, errorMessage: LIMITED, },);
              const toolCall = { type: 'toolCall', id: 'tool-fixture', name: 'fixtureTool', arguments: {}, } as const;
              message.content.push(toolCall,);
              if (publish) {
                stream.push({ type: 'start', partial: message, },);
                stream.push({ type: 'toolcall_start', contentIndex: 0, partial: message, },);
              }
              finish({ stream, message, },);
              return stream;
            },
          },);
          const model = poolModel({ provider, id: 'writer', },);
          const { events, result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          expect(result.errorMessage,).toBe(LIMITED,);
          expect(events.map(event => event.type,),).toEqual(publish ? ['start', 'toolcall_start', 'error',] : ['error',],);
          expect(calls.map(call => call.model.provider,),).toEqual(['openai',],);
        }
      },
    },),
    it({
      name: 'keeps an already-exposed attempt even if the terminal error has empty content',
      fn: async function committedStream() {
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, }) {
            const stream = createAssistantMessageEventStream();
            const partial = assistant({ model, },);
            stream.push({ type: 'start', partial, },);
            stream.push({ type: 'text_start', contentIndex: 0, partial, },);
            finish({ stream, message: assistant({ model, errorMessage: LIMITED, },), },);
            return stream;
          },
        },);
        const model = poolModel({ provider, id: 'writer', },);
        const { events, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(events.map(event => event.type,),).toEqual(['start', 'text_start', 'error',],);
        expect(calls.map(call => call.model.provider,),).toEqual(['openai',],);
      },
    },),
    it({
      name: 'aborting the active request prevents a later member from being called',
      fn: async function abortActive() {
        const controller = new AbortController();
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, }) {
            controller.abort();
            return immediate({ model, errorMessage: LIMITED, },);
          },
        },);
        const model = poolModel({ provider, id: 'writer', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, {
          ...options, signal: controller.signal,
        },),);
        expect(result.stopReason,).toBe('aborted',);
        expect(calls.map(call => call.model.provider,),).toEqual(['openai',],);
      },
    },),
    it({
      name: 'returns the last member error when every registered member rate-limits',
      fn: async function exhaustedPool() {
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, }) { return immediate({ model, errorMessage: LIMITED, },); },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const { events, result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(calls.map(call => call.model.provider,),).toEqual(TARGETS.slice(1,).map(target => target.provider,),);
        expect(events.map(event => event.type,),).toEqual(['error',],);
        expect(result.provider,).toBe(TARGETS.at(-1)?.provider,);
        expect(result.errorMessage,).toBe(LIMITED,);
      },
    },),
    it({
      name: 'uses the actual response status and leaves failed non-429 requests alone',
      fn: async function structuredStatus() {
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, options: member, }) {
            return lazyStream(model, async function reply() {
              await member?.onResponse?.({
                status: model.provider === 'openai' ? 429 : 400,
                headers: {},
              }, model,);
              return immediate({ model, errorMessage: 'Provider returned an error', },);
            },);
          },
        },);
        const model = poolModel({ provider, id: 'writer', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(result.provider,).toBe('qwen-token-plan-individual',);
        expect(calls.map(call => call.model.provider,),).toEqual(['openai', 'qwen-token-plan-individual',],);
      },
    },),
    it({
      name: 'a truncated stream is not a refusal and does not hang or invoke another model',
      fn: async function truncated() {
        const { provider, calls, options, } = await fixture({
          extension,
          play() {
            const stream = createAssistantMessageEventStream();
            stream.end();
            return stream;
          },
        },);
        const model = poolModel({ provider, id: 'writer', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(result.errorMessage,).toContain('closed without a terminal assistant event',);
        expect(calls.map(call => call.model.provider,),).toEqual(['openai',],);
      },
    },),
  ],
},);
