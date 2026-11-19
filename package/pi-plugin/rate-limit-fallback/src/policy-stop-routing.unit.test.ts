/**
 A policy stop moves the request on before output and ends it once output was exposed.
 @module
 */

import {
  isRetryableAssistantError,
  type AssistantMessageEventStream,
} from '@earendil-works/pi-ai';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import extension from '../dist/final/node/index.mjs';
import {
  assistant,
  collect,
  fixture,
  immediate,
  type MemberCall,
  POLICY_STOP_TEXT,
  policyStopStream,
  poolModel,
  rateLimitRefusal,
  TARGETS,
  TRANSCRIPT,
} from './stand-in-pi-fixture.ts';

/** Physical writer head and reviewer head, followed by the reviewer's backup. */
const [OPENAI, QWEN, SYNTHETIC,] = TARGETS;

/** Wording that identifies a policy stop without HTTP status evidence. */
const POLICY_MESSAGES = {
  'legacy wording without a 429 status': 'Request flagged as a possible risk',
  'verbatim OpenAI wording with a curly apostrophe': POLICY_STOP_TEXT,
  'OpenAI wording truncated after the first sentence': 'This content was flagged for possible cybersecurity risk.',
  'OpenAI wording with a straight apostrophe': POLICY_STOP_TEXT.replace('’', '\'',),
  'content-filter label': 'Rejected by the content-filter. Contact support.',
  'safety_policy label': 'Blocked by safety_policy. Contact support.',
  'safety violation label': 'Blocked for a safety violation. Contact support.',
  'policy violation label': 'Blocked for a policy violation. Contact support.',
} as const;

/** Member-attributed preface the package writes on a terminal policy stop. */
function stoppedBy(member: {
  readonly provider: string;
  readonly id: string;
},): string {
  return `${member.provider}/${member.id} stopped the request under its content policy:`;
}

/** Independent classifier examples across the literal, separated, status, and fetch match shapes. */
const TRANSIENT_SPANS = [
  'Service is temporarily unavailable',
  'Provider/returned_an error',
  'networkerror',
  'HTTP2 stream failed',
  'HTTP/2 stream failed',
  'request to https://stand-in.example/path failed, reason:',
  'ResourceExhausted',
  'timed out',
  'server_busy',
  '503',
] as const;

/** Near matches and Unicode text must be preserved, not removed with the retry spans. */
const RETAINED_DETAIL = 'Reference İ😀abc429xyz, 1503, and rate\nlimit.';

/**
 Build the policy stop through the installed adapter, forwarding the pool's member options.

 @param call - member request the pool dispatched

 @param exposedText - text the member streams before the stop

 @returns physical stream from the installed Responses adapter
 */
function adapterStop({
  call: {
    model,
    context,
    options,
  },
  exposedText,
}: {
  readonly call: MemberCall;
  readonly exposedText?: string;
},): AssistantMessageEventStream {
  return policyStopStream({
    model,
    context,
    ...(options === undefined ? {} : { options, }),
    ...(exposedText === undefined ? {} : { exposedText, }),
  },);
}

await describe({
  name: 'policy stop routing',
  children: [
    it({
      name: 'moves to the next member when a policy stop arrives before any output',
      fn: async function policyMove() {
        const { provider, calls, options, } = await fixture({
          extension,
          play(call) {
            return call.model.provider === OPENAI.provider ? adapterStop({ call, },) : immediate({ model: call.model, },);
          },
        },);
        const model = poolModel({ provider, id: 'writer', },);
        const { events, result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(calls.map(call => call.model.provider,),).toEqual([OPENAI.provider, QWEN.provider,],);
        expect(events.map(event => event.type,),).toEqual(['start', 'done',],);
        expect(result.provider,).toBe(QWEN.provider,);
        expect(result.stopReason,).toBe('stop',);
        expect(result.errorMessage,).toBeUndefined();
      },
    },),
    it({
      name: 'names the member that answered after a policy move in the assistant record',
      fn: async function answeredAfterMove() {
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, options: forwarded, }) {
            if (model.provider === QWEN.provider)
              return immediate({ model, errorMessage: POLICY_STOP_TEXT, },);
            return immediate({ model, },);
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(calls.map(call => call.model.provider,),).toEqual([QWEN.provider, SYNTHETIC.provider,],);
        expect([result.provider, result.model,],).toEqual([SYNTHETIC.provider, SYNTHETIC.id,],);
        expect(result.stopReason,).toBe('stop',);
      },
    },),
    it({
      name: 'ends after exposed output with the member and the provider advisory',
      fn: async function exposedOutput() {
        const { provider, calls, options, } = await fixture({
          extension,
          play(call) {
            return adapterStop({
              call,
              exposedText: 'Partial answer',
            },);
          },
        },);
        const model = poolModel({ provider, id: 'writer', },);
        const { events, result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(calls.map(call => call.model.provider,),).toEqual([OPENAI.provider,],);
        expect(events.at(0,)
          ?.type,)
          .toBe('start',);
        expect(events.some(event => event.type === 'text_delta',),).toBe(true,);
        expect(events.at(-1,)
          ?.type,)
          .toBe('error',);
        expect(result.stopReason,).toBe('error',);
        expect(result.errorMessage,).toBe(`${stoppedBy(OPENAI,)} ${POLICY_STOP_TEXT}`,);
        expect(isRetryableAssistantError(result,),).toBe(false,);
      },
    },),
    it({
      name: 'ends with the last stopped member when every member stops before output',
      fn: async function everyMemberStopped() {
        /**
         Last physical member the writer pool asks.
         */
        const last = TARGETS.at(-1,);
        if (last === undefined)
          throw new Error('Expected a physical pool member',);
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, }) { return immediate({ model, errorMessage: POLICY_STOP_TEXT, },); },
        },);
        const model = poolModel({ provider, id: 'writer', },);
        const { events, result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(calls.map(call => call.model.provider,),).toEqual(TARGETS.map(target => target.provider,),);
        expect(events.map(event => event.type,),).toEqual(['error',],);
        expect(result.provider,).toBe(last.provider,);
        expect(result.errorMessage,).toBe(`${stoppedBy(last,)} ${POLICY_STOP_TEXT}`,);
        expect(isRetryableAssistantError(result,),).toBe(false,);
      },
    },),
    ...Object.entries(POLICY_MESSAGES,).map(function policyMessage([label, errorMessage,],) {
      return it({
        name: `moves on from a policy stop recognised by ${label}`,
        fn: async function policyStop() {
          const { provider, calls, options, } = await fixture({
            extension,
            play({ model, }) {
              return model.provider === OPENAI.provider
                ? immediate({ model, errorMessage, },)
                : immediate({ model, },);
            },
          },);
          const model = poolModel({ provider, id: 'writer', },);
          const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          expect(calls.map(call => call.model.provider,),).toEqual([OPENAI.provider, QWEN.provider,],);
          expect(result.provider,).toBe(QWEN.provider,);
          expect(result.stopReason,).toBe('stop',);
        },
      },);
    },),
    it({
      name: 'OpenAI policy opening outranks an adapter 429 and still moves on',
      fn: async function sanitizeRefusal() {
        const refusal = rateLimitRefusal({
          model: OPENAI,
          message: `${POLICY_STOP_TEXT}. Server_busy, RAtE-Limit, and please retry your request. Reference abc429xyz.`,
        },);
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, }) {
            return model.provider === OPENAI.provider
              ? immediate({ model, errorMessage: refusal, },)
              : immediate({ model, },);
          },
        },);
        const model = poolModel({ provider, id: 'writer', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(calls.map(call => call.model.provider,),).toEqual([OPENAI.provider, QWEN.provider,],);
        expect(result.provider,).toBe(QWEN.provider,);
      },
    },),
    ...TRANSIENT_SPANS.map(function retrySpan(span,) {
      return it({
        name: `masks Pi retry span ${span} without changing adjacent advisory text`,
        fn: async function maskOnlyTransientSpan() {
          /**
           Advisory carrying one Pi retry span beside text that must survive.
           */
          const errorMessage = `${POLICY_STOP_TEXT}. ${RETAINED_DETAIL} (${span})`;
          /**
           Last member the writer pool asks.
           */
          const last = TARGETS.at(-1,);
          if (last === undefined)
            throw new Error('Expected a physical pool member',);
          const { provider, options, } = await fixture({
            extension,
            play({ model, }) {
              expect(
                isRetryableAssistantError(assistant({ model, errorMessage, },),),
              ).toBe(true,);
              return immediate({ model, errorMessage, },);
            },
          },);
          const model = poolModel({ provider, id: 'writer', },);
          const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          expect(result.errorMessage,).toBe(`${stoppedBy(last,)} ${POLICY_STOP_TEXT}. ${RETAINED_DETAIL} ([omitted])`,);
          expect(isRetryableAssistantError(result,),).toBe(false,);
        },
      },);
    },),
    ...['possible risk', 'possible cybersecurity risk',].map(function genericRisk(phrase,) {
      return it({
        name: `hands off on a Completions 429 that merely mentions ${phrase}`,
        fn: async function riskMentionInRateLimit() {
          const { provider, calls, options, } = await fixture({
            extension,
            play({ model, }) {
              return immediate({
                model,
                ...(model.provider === QWEN.provider
                  ? { errorMessage: rateLimitRefusal({ model, message: `Too many requests while we assess ${phrase}`, },), }
                  : {}),
              },);
            },
          },);
          const model = poolModel({ provider, id: 'reviewer', },);
          const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          expect(result.stopReason,).toBe('stop',);
          expect(calls.map(call => call.model.provider,),).toEqual([QWEN.provider, SYNTHETIC.provider,],);
        },
      },);
    },),
    it({
      name: 'an ordinary error still ends the request without a policy move',
      fn: async function ordinaryError() {
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, }) { return immediate({ model, errorMessage: '400 Context window exceeded', },); },
        },);
        const model = poolModel({ provider, id: 'writer', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(result.errorMessage,).toBe('400 Context window exceeded',);
        expect(calls.map(call => call.model.provider,),).toEqual([OPENAI.provider,],);
      },
    },),
    it({
      name: 'keeps the reviewer pool unchanged',
      fn: async function reviewerPool() {
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, }) { return immediate({ model, errorMessage: rateLimitRefusal({ model, },), },); },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(calls.map(call => `${call.model.provider}/${call.model.id}`,),).toEqual(
          TARGETS.slice(1,)
            .map(target => `${target.provider}/${target.id}`,),
        );
      },
    },),
  ],
},);
