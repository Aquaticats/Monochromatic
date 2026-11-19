/**
 A policy stop moves the request on only for OpenAI's own opening, and ends it once output was exposed.
 @module
 */

import {
  isRetryableAssistantError,
  lazyStream,
  type Api,
  type AssistantMessage,
  type AssistantMessageEventStream,
  type Model,
  type SimpleStreamOptions,
} from '@earendil-works/pi-ai';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import extension from '../dist/final/node/index.mjs';
import { POLICY_STOP_TEXT, policyStopStream, } from './policy-stop-fixture.ts';
import { labelOf, REVIEWER_TARGETS, WRITER_TARGETS, } from './member-order-fixture.ts';
import {
  assistant,
  collect,
  fixture,
  immediate,
  poolModel,
  rateLimitRefusal,
  RATE_LIMIT_STATUS,
  TRANSCRIPT,
} from './stand-in-pi-fixture.ts';

/**
 Reviewer pool in its decided order; OpenAI's advisory reaches a member only as relayed text.
 No pool has an OpenAI member since 2026-11-19.
 */
const [QWEN, SYNTHETIC,] = REVIEWER_TARGETS;

/** Writer pool in its decided order; the last member is what an exhausted pool returns. */
const [WRITER_HEAD, WRITER_SECOND, WRITER_LAST,] = WRITER_TARGETS;

/** Retry timing a 429 carries, so a member that was rate-limited is held afterwards. */
const RETRY_AFTER_SECONDS = 60;

/**
 Advisories that carry OpenAI's own policy-stop opening, which is what may move a request on.

 The first is the recorded text the fixture documents. The other two are variants of that capture,
 not captures of their own: one truncated after the first sentence, one with a straight apostrophe.
 */
const OPENAI_POLICY_MESSAGES = {
  'the recorded advisory': POLICY_STOP_TEXT,
  'the advisory truncated after its first sentence': 'This content was flagged for possible cybersecurity risk.',
  'the recorded advisory with a straight apostrophe': POLICY_STOP_TEXT.replace('’', '\'',),
} as const;

/**
 Policy text that names no OpenAI opening.

 Hand-written stand-ins: no producer source and no capture on record emits these labels. They exist
 only to prove that another vendor's content stop ends the request instead of moving it on.
 */
const OTHER_POLICY_MESSAGES = {
  'a possible-risk sentence': 'Request flagged as a possible risk',
  'a content-filter label': 'Rejected by the content-filter. Contact support.',
  'a safety_policy label': 'Blocked by safety_policy. Contact support.',
  'a safety-violation label': 'Blocked for a safety violation. Contact support.',
  'a policy-violation label': 'Blocked for a policy violation. Contact support.',
} as const;

/** Member-attributed preface the package writes on a terminal policy stop. */
function stoppedBy(member: {
  readonly provider: string;
  readonly id: string;
},): string {
  return `${labelOf(member,)} stopped the request under its content policy:`;
}

/**
 `provider/id` of an assistant record, whose `model` field names the physical model that answered.

 @param message - terminal assistant record

 @returns exact member label the record names
 */
function recordLabel(message: AssistantMessage,): string {
  return `${message.provider}/${message.model}`;
}

/**
 Injected retry wording, a hand-written stand-in that no producer emits: it exists to show that
 every span Pi's classifier would retry is removed while the advisory around it survives.
 */
const INJECTED_RETRY_WORDS = 'Server_busy, RAtE-Limit, and please retry your request. Reference abc429xyz.';

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
 Build the policy stop through the installed adapter for the transcript under test.

 @param model - member whose stream carries the advisory

 @param options - request options the pool forwarded to the member

 @param exposedText - text the member streams before the stop

 @returns physical stream from the installed Responses adapter
 */
function advisoryStop({
  model,
  options,
  exposedText,
}: {
  readonly model: Model<Api>;
  readonly options?: SimpleStreamOptions;
  readonly exposedText?: string;
},): AssistantMessageEventStream {
  return policyStopStream({
    model,
    context: TRANSCRIPT,
    ...(options === undefined ? {} : { options, }),
    ...(exposedText === undefined ? {} : { exposedText, }),
  },);
}

/**
 Answer one member with a 429 that carries retry timing through the response hook.

 Stand-in for a shape no installed adapter emits: each of them calls `onResponse` after a successful
 response alone (`@earendil-works/pi-ai` dist/api/openai-completions.js:203, dist/api/openai-
 responses.js:155), so a real 429 never reaches the plugin this way. The hook path is kept for an
 adapter that starts exposing failed responses.

 @param model - physical member

 @param options - request options the pool forwarded

 @param message - provider text inside the refusal body

 @returns physical stream that fails with the refusal
 */
function rateLimited({
  model,
  options,
  message,
}: {
  readonly model: Model<Api>;
  readonly options?: SimpleStreamOptions;
  readonly message: string;
},): AssistantMessageEventStream {
  return lazyStream(model, async function refused() {
    await options?.onResponse?.({
      status: RATE_LIMIT_STATUS,
      headers: { 'Retry-After': String(RETRY_AFTER_SECONDS,), },
    }, model,);
    return immediate({
      model,
      errorMessage: rateLimitRefusal({
        model,
        message,
      },),
    },);
  },);
}

await describe({
  name: 'policy stop routing',
  children: [
    it({
      name: 'moves on when a member relays OpenAI’s advisory before any output',
      fn: async function policyMove() {
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, options: member, }) {
            return labelOf(model,) === labelOf(WRITER_HEAD,)
              ? advisoryStop({
                model,
                ...(member === undefined ? {} : { options: member, }),
              },)
              : immediate({ model, },);
          },
        },);
        const model = poolModel({ provider, id: 'writer', },);
        const { events, result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(calls.map(call => labelOf(call.model,),),).toEqual([labelOf(WRITER_HEAD,), labelOf(WRITER_SECOND,),],);
        expect(events.map(event => event.type,),).toEqual(['start', 'done',],);
        expect(recordLabel(result,),).toBe(labelOf(WRITER_SECOND,),);
        expect(result.stopReason,).toBe('stop',);
        expect(result.errorMessage,).toBeUndefined();
      },
    },),
    it({
      name: 'names the member that answered after a policy move in the assistant record',
      fn: async function answeredAfterMove() {
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, }) {
            if (model.provider === QWEN.provider)
              return immediate({ model, errorMessage: POLICY_STOP_TEXT, },);
            return immediate({ model, },);
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(calls.map(call => labelOf(call.model,),),).toEqual([labelOf(QWEN,), labelOf(SYNTHETIC,),],);
        expect([result.provider, result.model,],).toEqual([SYNTHETIC.provider, SYNTHETIC.id,],);
        expect(result.stopReason,).toBe('stop',);
      },
    },),
    it({
      name: 'ends after exposed output with the member and the provider advisory',
      fn: async function exposedOutput() {
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, options: member, }) {
            return advisoryStop({
              model,
              ...(member === undefined ? {} : { options: member, }),
              exposedText: 'Partial answer',
            },);
          },
        },);
        const model = poolModel({ provider, id: 'writer', },);
        const { events, result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(calls.map(call => labelOf(call.model,),),).toEqual([labelOf(WRITER_HEAD,),],);
        expect(events.at(0,)
          ?.type,)
          .toBe('start',);
        expect(events.some(event => event.type === 'text_delta',),).toBe(true,);
        expect(events.at(-1,)
          ?.type,)
          .toBe('error',);
        expect(result.stopReason,).toBe('error',);
        expect(result.errorMessage,).toBe(`${stoppedBy(WRITER_HEAD,)} ${POLICY_STOP_TEXT}`,);
        expect(isRetryableAssistantError(result,),).toBe(false,);
      },
    },),
    it({
      name: 'ends with the last stopped member when every member stops before output',
      fn: async function everyMemberStopped() {
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, }) { return immediate({ model, errorMessage: POLICY_STOP_TEXT, },); },
        },);
        const model = poolModel({ provider, id: 'writer', },);
        const { events, result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(calls.map(call => labelOf(call.model,),),).toEqual(WRITER_TARGETS.map(target => labelOf(target,),),);
        expect(events.map(event => event.type,),).toEqual(['error',],);
        expect(recordLabel(result,),).toBe(labelOf(WRITER_LAST,),);
        expect(result.errorMessage,).toBe(`${stoppedBy(WRITER_LAST,)} ${POLICY_STOP_TEXT}`,);
        expect(isRetryableAssistantError(result,),).toBe(false,);
      },
    },),
    ...Object.entries(OPENAI_POLICY_MESSAGES,).map(function policyMessage([label, errorMessage,],) {
      return it({
        name: `moves on from ${label} before any output`,
        fn: async function policyStop() {
          const { provider, calls, options, } = await fixture({
            extension,
            play({ model, }) {
              return labelOf(model,) === labelOf(WRITER_HEAD,)
                ? immediate({ model, errorMessage, },)
                : immediate({ model, },);
            },
          },);
          const model = poolModel({ provider, id: 'writer', },);
          const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          expect(calls.map(call => labelOf(call.model,),),).toEqual([labelOf(WRITER_HEAD,), labelOf(WRITER_SECOND,),],);
          expect(recordLabel(result,),).toBe(labelOf(WRITER_SECOND,),);
          expect(result.stopReason,).toBe('stop',);
        },
      },);
    },),
    ...Object.entries(OTHER_POLICY_MESSAGES,).map(function otherVendorStop([label, errorMessage,],) {
      return it({
        name: `ends the request on ${label} before any output`,
        fn: async function leftAlone() {
          const { provider, calls, options, } = await fixture({
            extension,
            play({ model, }) { return immediate({ model, errorMessage, },); },
          },);
          const model = poolModel({ provider, id: 'writer', },);
          const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          expect(calls.map(call => labelOf(call.model,),),).toEqual([labelOf(WRITER_HEAD,),],);
          expect(result.stopReason,).toBe('error',);
          expect(result.errorMessage,).toBe(`${stoppedBy(WRITER_HEAD,)} ${errorMessage}`,);
          expect(isRetryableAssistantError(result,),).toBe(false,);
        },
      },);
    },),
    it({
      name: 'does not move on when a refusal merely quotes OpenAI’s sentence',
      fn: async function quotedOpening() {
        /**
         Hand-written stand-in: a refusal whose own text quotes the stock sentence as evidence of a
         repeated prompt, the shape the second review measured for a quoting refusal.
         */
        const quoting = `400 invalid_request_error: the prompt repeats ${POLICY_STOP_TEXT} three times`;
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, }) { return immediate({ model, errorMessage: quoting, },); },
        },);
        const model = poolModel({ provider, id: 'writer', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(calls.map(call => labelOf(call.model,),),).toEqual([labelOf(WRITER_HEAD,),],);
        expect(result.stopReason,).toBe('error',);
        expect(result.errorMessage,).toContain(quoting,);
        expect(isRetryableAssistantError(result,),).toBe(false,);
      },
    },),
    it({
      name: 'ends the request when only a 429 body quotes OpenAI’s opening',
      fn: async function sanitizeRefusal() {
        const message = `${POLICY_STOP_TEXT}. ${INJECTED_RETRY_WORDS}`;
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, }) {
            return labelOf(model,) === labelOf(WRITER_HEAD,)
              ? immediate({ model, errorMessage: rateLimitRefusal({ model, message, },), },)
              : immediate({ model, },);
          },
        },);
        const model = poolModel({ provider, id: 'writer', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(calls.map(call => labelOf(call.model,),),).toEqual([labelOf(WRITER_HEAD,),],);
        expect(result.stopReason,).toBe('error',);
        expect(result.errorMessage,).toContain(stoppedBy(WRITER_HEAD,),);
        expect(result.errorMessage,).toContain(POLICY_STOP_TEXT,);
        expect(isRetryableAssistantError(result,),).toBe(false,);
      },
    },),
    it({
      name: 'a 429 that merely mentions possible risk stays a retryable rate limit',
      fn: async function riskMentionInRateLimit() {
        /**
         Completions refusal that names risk without OpenAI's opening.
         Hand-written stand-in: no capture on record pairs this wording with a 429, and it exists to
         prove that a risk mention inside a rate limit stays a rate limit.
         */
        const message = 'Too many requests while we assess possible risk';
        const { provider, calls, options, } = await fixture({
          extension,
          models: [QWEN,],
          play({ model, options: member, }) {
            return rateLimited({
              model,
              ...(member === undefined ? {} : { options: member, }),
              message,
            },);
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const refused = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(refused.result.errorMessage,).toBe(rateLimitRefusal({ model: QWEN, message, },),);
        expect(isRetryableAssistantError(refused.result,),).toBe(true,);
        const held = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(held.result.errorMessage,).toContain(`${labelOf(QWEN,)} is held until`,);
        expect(calls.length,).toBe(1,);
      },
    },),
    it({
      name: 'a 429 that carries OpenAI’s opening is a policy stop and records no retry window',
      fn: async function openingOutranksRateLimit() {
        const { provider, calls, options, } = await fixture({
          extension,
          models: [QWEN,],
          play({ model, options: member, }) {
            return rateLimited({
              model,
              ...(member === undefined ? {} : { options: member, }),
              message: POLICY_STOP_TEXT,
            },);
          },
        },);
        const model = poolModel({ provider, id: 'reviewer', },);
        const stopped = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        // The refusal's bare `429` status is a retry span Pi would act on, so the record masks it
        // and keeps the advisory that follows it.
        expect(stopped.result.errorMessage,).toContain(stoppedBy(QWEN,),);
        expect(stopped.result.errorMessage,).toContain(POLICY_STOP_TEXT,);
        expect(isRetryableAssistantError(stopped.result,),).toBe(false,);
        const askedAgain = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(calls.length,).toBe(2,);
        expect(askedAgain.result.errorMessage,).toBe(stopped.result.errorMessage,);
      },
    },),
    it({
      name: 'yields the set-aside policy stop when every later member is rate-limited',
      fn: async function setAsideStopWins() {
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, options: member, }) {
            const forwarded = member === undefined ? {} : { options: member, };
            return labelOf(model,) === labelOf(WRITER_HEAD,)
              ? advisoryStop({
                model,
                ...forwarded,
              },)
              : rateLimited({
                model,
                ...forwarded,
                message: 'Too many requests',
              },);
          },
        },);
        const model = poolModel({ provider, id: 'writer', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(calls.map(call => labelOf(call.model,),),).toEqual(WRITER_TARGETS.map(target => labelOf(target,),),);
        // The attributed stop is what Pi must see, so its outer retry cannot restart at the head.
        expect(result.errorMessage,).toBe(`${stoppedBy(WRITER_HEAD,)} ${POLICY_STOP_TEXT}`,);
        expect(isRetryableAssistantError(result,),).toBe(false,);
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
          expect(result.errorMessage,).toBe(`${stoppedBy(WRITER_LAST,)} ${POLICY_STOP_TEXT}. ${RETAINED_DETAIL} ([omitted])`,);
          expect(isRetryableAssistantError(result,),).toBe(false,);
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
        expect(calls.map(call => labelOf(call.model,),),).toEqual([labelOf(WRITER_HEAD,),],);
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
        expect(calls.map(call => labelOf(call.model,),),).toEqual(REVIEWER_TARGETS.map(target => labelOf(target,),),);
      },
    },),
  ],
},);
