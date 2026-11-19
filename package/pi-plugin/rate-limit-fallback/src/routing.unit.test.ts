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
  rateLimitRefusal,
  RATE_LIMIT_STATUS,
  TRANSCRIPT,
} from './stand-in-pi-fixture.ts';
import { labelOf, REVIEWER_TARGETS, WRITER_TARGETS, } from './member-order-fixture.ts';

/** Writer head, the member every request starts at. */
const [HEAD,] = WRITER_TARGETS;

/**
 The recorded Synthetic refusal in the quoted form, verbatim: the `modelError` of
 `${HOME}/temp/agent/pi-runs/kimi-review-since-alpha8-landing-waits-1/summary.json`,
 138 characters.
 */
const RECORDED_QUOTED_REFUSAL: string = `429 "You've exceeded your subscription rate limits. Upgrade, or try again later. You can view your usage at https://synthetic.new/billing"`;

/**
 Other non-capacity failures are returned unchanged.

 The last entry is a hand-written number that merely begins with `429`: another package's tests hold
 the same shape (`package/pi-shared/subagent-driver/src/retries-of-a-model-error.unit.test.ts`),
 and a digit after the status token means the text is not a refusal.
 */
const TERMINAL_ERRORS = [
  '401 Unauthorized: contact support with request 429',
  'Request timed out',
  '400 Context window exceeded',
  '4290 tokens exceeded the window',
] as const;

await describe({
  name: 'ordered provider routing',
  children: [
    it({
      name: 'moves on from the Responses refusal form under any provider name',
      fn: async function responsesPrefix() {
        /**
         Writer head speaking the Responses protocol, as a non-OpenAI member would if Pi routed one
         through that adapter; its refusal carries the adapter's own `<provider> API error (429):`
         label.
         */
        const responses = { ...HEAD, api: 'openai-responses', };
        const { provider, calls, options, } = await fixture({
          extension,
          models: [responses, WRITER_TARGETS[1],],
          play({ model, }) {
            return calls.length === 1
              ? immediate({ model, errorMessage: rateLimitRefusal({ model, },), },)
              : immediate({ model, },);
          },
        },);
        const model = poolModel({ provider, id: 'writer', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(calls.map(call => labelOf(call.model,),),).toEqual([
          labelOf(responses,),
          labelOf(WRITER_TARGETS[1],),
        ],);
        expect(result.stopReason,).toBe('stop',);
        expect(result.provider,).toBe(WRITER_TARGETS[1].provider,);
      },
    },),
    it({
      name: 'moves on from a recorded 429 in the quoted form',
      fn: async function quotedRefusal() {
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, }) {
            // The reviewer head refuses in the Completions form, and the recorded Synthetic
            // refusal follows on the member it belongs to.
            if (model.provider === REVIEWER_TARGETS[0].provider)
              return immediate({ model, errorMessage: rateLimitRefusal({ model, },), },);
            if (model.provider === REVIEWER_TARGETS[1].provider)
              return immediate({ model, errorMessage: RECORDED_QUOTED_REFUSAL, },);
            return immediate({ model, },);
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(calls.map(call => labelOf(call.model,),),).toEqual([
          labelOf(REVIEWER_TARGETS[0],),
          labelOf(REVIEWER_TARGETS[1],),
          labelOf(REVIEWER_TARGETS[2],),
        ],);
        expect(result.stopReason,).toBe('stop',);
      },
    },),
    it({
      name: 'writer tries the exact pool and keeps only the answering member events',
      fn: async function writerOrder() {
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, }) {
            return immediate({
              model,
              ...(model.provider === WRITER_TARGETS.at(-1,)
                ?.provider ? {} : { errorMessage: rateLimitRefusal({ model, },), }),
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
          WRITER_TARGETS.map(target => [target.provider, target.id,],),
        );
        expect(calls.every(call => (call.context === TRANSCRIPT) && (call.options?.maxTokens === call.model.maxTokens),),)
          .toBe(true,);
        expect(events.map(event => event.type,),).toEqual(['start', 'done',],);
        expect([result.provider, result.model,],).toEqual([
          WRITER_TARGETS.at(-1,)
            ?.provider,
          WRITER_TARGETS.at(-1,)
            ?.id,
        ],);
      },
    },),
    it({
      name: 'reviewer starts with Qwen, retries Qwen on the next request, and never forwards outer credentials',
      fn: async function reviewerRetry() {
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, }) {
            return calls.length === 1 ? immediate({ model, errorMessage: rateLimitRefusal({ model, },), },) : immediate({ model, },);
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
          expect(calls.map(call => call.model.provider,),).toEqual([HEAD.provider,],);
        },
      },);
    },),
    it({
      name: 'does not replay after a tool event or when the terminal 429 already contains content',
      fn: async function partialOutput() {
        await Promise.all([true, false,].map(async function verifyOutputBoundary(publish,): Promise<void> {
          const { provider, calls, options, } = await fixture({
            extension,
            play({ model, }) {
              const stream = createAssistantMessageEventStream();
              const message = assistant({ model, errorMessage: rateLimitRefusal({ model, },), },);
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
          expect(result.errorMessage,).toBe(rateLimitRefusal({ model: HEAD, },),);
          expect(events.map(event => event.type,),).toEqual(publish ? ['start', 'toolcall_start', 'error',] : ['error',],);
          expect(calls.map(call => call.model.provider,),).toEqual([HEAD.provider,],);
        },),);
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
            finish({ stream, message: assistant({ model, errorMessage: rateLimitRefusal({ model, },), },), },);
            return stream;
          },
        },);
        const model = poolModel({ provider, id: 'writer', },);
        const { events, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(events.map(event => event.type,),).toEqual(['start', 'text_start', 'error',],);
        expect(calls.map(call => call.model.provider,),).toEqual([HEAD.provider,],);
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
            return immediate({ model, errorMessage: rateLimitRefusal({ model, },), },);
          },
        },);
        const model = poolModel({ provider, id: 'writer', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, {
          ...options, signal: controller.signal,
        },),);
        expect(result.stopReason,).toBe('aborted',);
        expect(calls.map(call => call.model.provider,),).toEqual([HEAD.provider,],);
      },
    },),
    it({
      name: 'returns the last member error when every registered member rate-limits',
      fn: async function exhaustedPool() {
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, }) { return immediate({ model, errorMessage: rateLimitRefusal({ model, },), },); },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const { events, result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(calls.map(call => call.model.provider,),).toEqual(REVIEWER_TARGETS.map(target => target.provider,),);
        expect(events.map(event => event.type,),).toEqual(['error',],);
        expect(result.provider,).toBe(REVIEWER_TARGETS.at(-1,)
          ?.provider,);
        expect(result.errorMessage,).toBe(rateLimitRefusal({ model: result, },),);
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
                status: calls.length === 1 ? RATE_LIMIT_STATUS : 400,
                headers: {},
              }, model,);
              return immediate({ model, errorMessage: 'Provider returned an error', },);
            },);
          },
        },);
        const model = poolModel({ provider, id: 'writer', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(result.provider,).toBe(WRITER_TARGETS[1].provider,);
        expect(result.model,).toBe(WRITER_TARGETS[1].id,);
        expect(calls.map(call => call.model.provider,),).toEqual([
          WRITER_TARGETS[0].provider,
          WRITER_TARGETS[1].provider,
        ],);
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
        expect(calls.map(call => call.model.provider,),).toEqual([HEAD.provider,],);
      },
    },),
  ],
},);
