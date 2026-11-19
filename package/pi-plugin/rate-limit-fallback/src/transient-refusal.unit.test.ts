/**
 A transient provider failure moves the request on, so Pi's own retry never restarts at that member.
 @module
 */

import {
  createAssistantMessageEventStream,
  lazyStream,
} from '@earendil-works/pi-ai';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import extension from '../dist/final/node/index.mjs';
import { labelOf, WRITER_TARGETS, } from './member-order-fixture.ts';
import {
  assistant,
  collect,
  finish,
  fixture,
  immediate,
  poolModel,
  TRANSCRIPT,
} from './stand-in-pi-fixture.ts';

/**
 Writer head, the member every request starts at.
 */
const [HEAD, SECOND,] = WRITER_TARGETS;

/**
 First attempt a pool makes, the one the refusing member gets.
 */
const FIRST_ATTEMPT = 1;

/**
 HTTP status NeuralWatt documents as fleet capacity.
 */
const UNAVAILABLE_STATUS = 503;

/**
 Cloudflare's status for an origin it cannot reach.
 */
const UNREACHABLE_STATUS = 523;

/**
 Cloudflare's status for an origin that timed out.
 */
const TIMED_OUT_STATUS = 522;

/**
 The 503 status in the Completions adapter's text form.

 Made up: no capture of a NeuralWatt 503 exists on this machine. The boundary is the status read from
 a refusal's leading token, and the body shape is the installed Completions adapter's `<status>:
 <body>` (`@earendil-works/pi-ai` dist/utils/error-body.js, `formatProviderError`).
 */
const UNAVAILABLE_503 = `${UNAVAILABLE_STATUS}: {"error":{"message":"Service temporarily unavailable","type":"server_error"}}`;

/**
 Cloudflare 523, as that error page heads it.

 Source, not a capture: "Error 523: origin is unreachable"
 (https://developers.cloudflare.com/support/troubleshooting/http-status-codes/cloudflare-5xx-errors/error-523/,
 read 2026-11-19). A proxied API returns that page to the SDK, and the adapter prefixes the status.
 */
const UNREACHABLE_523 = `${UNREACHABLE_STATUS}: Error ${UNREACHABLE_STATUS}: origin is unreachable`;

/**
 Cloudflare 522, as that error page heads it.

 Source, not a capture: "Error 522: connection timed out"
 (https://developers.cloudflare.com/support/troubleshooting/http-status-codes/cloudflare-5xx-errors/error-522/,
 read 2026-11-19). Its words match Pi's `timed? out` retry pattern
 (`@earendil-works/pi-ai` dist/utils/retry.js), which is why the pool must consume it before Pi sees it.
 */
const TIMED_OUT_522 = `${TIMED_OUT_STATUS}: Error ${TIMED_OUT_STATUS}: connection timed out`;

/**
 Path of NeuralWatt's rate-limit endpoint, the one read a refusal makes for that member.

 The member's quota read also fetches `/v1/quota`, so the count below names the rate-limit request the
 read starts with.
 */
const RATE_LIMITS_PATH = '/v1/rate_limits';

/**
 Read the URL from a fetch input without stringifying an arbitrary object.

 @param input - quota fetch destination in the fixture

 @returns URL string to compare
 */
function quotaUrl(input: Parameters<typeof globalThis.fetch>[0],): string {
  if ((typeof input) === 'string')
    return input;
  if (input instanceof URL)
    return input.href;
  return input.url;
}

/**
 A 429 whose body quotes a Responses failure label for a Cloudflare status.

 Made up: boundary is that the refusal's own leading status decides, so a 429 body cannot be
 reclassified as transient by what it quotes (`doc/captures.md`, "Made-up texts").
 */
const QUOTED_LABEL_IN_429 = '429: OpenAI API error (523): upstream unavailable';

/**
 The same quote with no space after the 429's colon, which an adapter never writes.

 Made up: boundary is the same one, at the tightest spelling a reader might meet.
 */
const TIGHT_QUOTED_LABEL_IN_429 = `429:openai api error (523): upstream unavailable`;

/**
 503 bodies that also carry content-policy wording.

 Made up: the boundary is that the transient status the refusal opens with decides the handoff, so a
 policy label or the `possible risk` phrase inside the body cannot turn a provider's capacity 503
 into a stop that ends the request at that member.
 */
const POLICY_WORDED_503S = {
  'a safety-policy label': '503: {"error":{"message":"Request blocked by safety policy","type":"server_error"}}',
  'the possible-risk phrase': '503: {"error":{"message":"Upstream says a possible risk applies","type":"server_error"}}',
} as const;

/**
 Every transient refusal the suite replays, named by its source class.
 */
const TRANSIENT_REFUSALS = {
  'the 503 status a provider uses for fleet capacity': UNAVAILABLE_503,
  'Cloudflare 523': UNREACHABLE_523,
  'Cloudflare 522': TIMED_OUT_522,
} as const;

/**
 Retry-After seconds a transient refusal names, far enough ahead to hold the member.
 */
const RETRY_AFTER_SECONDS = 60;

await describe({
  name: 'transient provider refusals',
  children: [
    ...Object.entries(TRANSIENT_REFUSALS,)
      .map(function refusal([name, errorMessage,],) {
        return it({
          name: `moves on from ${name} before any output`,
          fn: async function transientRefusal() {
            const { provider, calls, options, } = await fixture({
              extension,
              play({ model, }) {
                return calls.length === FIRST_ATTEMPT
                  ? immediate({ model, errorMessage, },)
                  : immediate({ model, },);
              },
            },);
            const model = poolModel({ provider, id: 'writer-pool', },);
            const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
            expect(calls.map(call => labelOf(call.model,),),).toEqual([labelOf(HEAD,), labelOf(SECOND,),],);
            expect(result.provider,).toBe(SECOND.provider,);
            expect(result.model,).toBe(SECOND.id,);
            expect(result.stopReason,).toBe('stop',);
          },
        },);
      },),
    ...Object.entries(POLICY_WORDED_503S,)
      .map(function policyWording([name, errorMessage,],) {
        return it({
          name: `moves on from a 503 whose body carries ${name}`,
          fn: async function transientWithPolicyWording() {
            const { provider, calls, options, } = await fixture({
              extension,
              play({ model, }) {
                return calls.length === FIRST_ATTEMPT
                  ? immediate({ model, errorMessage, },)
                  : immediate({ model, },);
              },
            },);
            const model = poolModel({ provider, id: 'writer-pool', },);
            const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
            expect(calls.map(call => labelOf(call.model,),),).toEqual([labelOf(HEAD,), labelOf(SECOND,),],);
            expect(result.provider,).toBe(SECOND.provider,);
            expect(result.stopReason,).toBe('stop',);
          },
        },);
      },),
    ...Object.entries(POLICY_WORDED_503S,)
      .map(function policyWordingAfterOutput([name, errorMessage,],) {
        return it({
          name: `does not move on from a 503 carrying ${name} after output was exposed`,
          fn: async function afterOutputWithPolicyWording() {
            const { provider, calls, options, } = await fixture({
              extension,
              play({ model, }) {
                const stream = createAssistantMessageEventStream();
                /**
                 Partial message the member exposes before it fails.
                 */
                const partial = assistant({ model, },);
                stream.push({ type: 'start', partial, },);
                stream.push({ type: 'text_start', contentIndex: 0, partial, },);
                finish({ stream, message: assistant({ model, errorMessage, },), },);
                return stream;
              },
            },);
            const model = poolModel({ provider, id: 'writer-pool', },);
            const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
            // Pi has seen this member's output, so the refusal ends the request there as it arrived,
            // not rewritten into a policy stop.
            expect(calls.map(call => labelOf(call.model,),),).toEqual([labelOf(HEAD,),],);
            expect(result.stopReason,).toBe('error',);
            expect(result.errorMessage,).toBe(errorMessage,);
          },
        },);
      },),
    ...[
      {
        name: 'a 429 whose body quotes a Responses failure label',
        errorMessage: QUOTED_LABEL_IN_429,
      },
      {
        name: 'a 429 whose body quotes one with no space after its colon',
        errorMessage: TIGHT_QUOTED_LABEL_IN_429,
      },
    ].map(function quotedLabel(entry,) {
      return it({
        name: `reads ${entry.name} as the rate limit it opens with`,
        fn: async function leadingStatusWins() {
          const reads = { count: 0, };
          const { provider, calls, options, } = await fixture({
            extension,
            play({ model, }) {
              return calls.length === FIRST_ATTEMPT
                ? immediate({ model, errorMessage: entry.errorMessage, },)
                : immediate({ model, },);
            },
            async fetchQuota(input,) {
              if (quotaUrl(input,).includes(RATE_LIMITS_PATH,))
                reads.count += 1;
              return Response.json({});
            },
          },);
          const model = poolModel({ provider, id: 'writer-pool', },);
          const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
          // A 429 reads the member's quota endpoint; a transient status reads nothing.
          expect(reads.count,).toBe(1,);
          expect(result.stopReason,).toBe('stop',);
          expect(calls.map(call => labelOf(call.model,),),).toEqual([labelOf(HEAD,), labelOf(SECOND,),],);
        },
      },);
    },),
    it({
      name: 'moves on from a Responses failure label that opens the text',
      fn: async function labelForm() {
        /**
         Refusal in the Responses adapter's label form, as a non-OpenAI provider's failure is
         formatted.
         */
        const labelled = 'standin API error (523): origin is unreachable';
        const reads = { count: 0, };
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, }) {
            return calls.length === FIRST_ATTEMPT
              ? immediate({ model, errorMessage: labelled, },)
              : immediate({ model, },);
          },
          async fetchQuota(input,) {
            if (quotaUrl(input,).includes(RATE_LIMITS_PATH,))
              reads.count += 1;
            return Response.json({});
          },
        },);
        const model = poolModel({ provider, id: 'writer-pool', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(reads.count,).toBe(0,);
        expect(result.stopReason,).toBe('stop',);
        expect(calls.map(call => labelOf(call.model,),),).toEqual([labelOf(HEAD,), labelOf(SECOND,),],);
      },
    },),
    it({
      name: 'moves on from a transient status its adapter exposed without text',
      fn: async function exposedStatus() {
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, options: member, }) {
            if (calls.length !== FIRST_ATTEMPT)
              return immediate({ model, },);
            return lazyStream(model, async function reply() {
              await member?.onResponse?.({
                status: UNREACHABLE_STATUS,
                headers: {},
              }, model,);
              // The refusal text names no status, so only the exposed one can move the request on.
              return immediate({ model, errorMessage: 'Provider returned an error', },);
            },);
          },
        },);
        const model = poolModel({ provider, id: 'writer-pool', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(calls.map(call => labelOf(call.model,),),).toEqual([labelOf(HEAD,), labelOf(SECOND,),],);
        expect(result.stopReason,).toBe('stop',);
      },
    },),
    it({
      name: 'does not move on from a transient status after output was exposed',
      fn: async function afterOutput() {
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, }) {
            const stream = createAssistantMessageEventStream();
            /**
             Partial message the member exposes before it fails.
             */
            const partial = assistant({ model, },);
            stream.push({ type: 'start', partial, },);
            stream.push({ type: 'text_start', contentIndex: 0, partial, },);
            finish({ stream, message: assistant({ model, errorMessage: UNREACHABLE_523, },), },);
            return stream;
          },
        },);
        const model = poolModel({ provider, id: 'writer-pool', },);
        const { events, result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(events.map(event => event.type,),).toEqual(['start', 'text_start', 'error',],);
        expect(result.errorMessage,).toBe(UNREACHABLE_523,);
        expect(calls.map(call => labelOf(call.model,),),).toEqual([labelOf(HEAD,),],);
      },
    },),
    it({
      name: 'returns the last transient refusal as it arrived when every member fails the same way',
      fn: async function exhaustedPool() {
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, }) { return immediate({ model, errorMessage: TIMED_OUT_522, },); },
        },);
        const model = poolModel({ provider, id: 'writer-pool', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(calls.map(call => labelOf(call.model,),),).toEqual(WRITER_TARGETS.map(target => labelOf(target,),),);
        expect(result.stopReason,).toBe('error',);
        expect(result.provider,).toBe(WRITER_TARGETS.at(-1,)
          ?.provider,);
        expect(result.errorMessage,).toBe(TIMED_OUT_522,);
      },
    },),
    // Stand-in shape, recorded in `doc/captures.md`: the `onResponse` call in this case is made by
    // hand on a failed response, which no installed adapter does, so this covers the header path
    // alone.
    it({
      name: 'holds a member that refused with a Retry-After until that time',
      fn: async function transientHold(ctx,) {
        const clock = { now: Date.now(), };
        ctx.sinon.stub(Date, 'now',).callsFake(() => clock.now,);
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, options: member, }) {
            if (calls.length !== FIRST_ATTEMPT)
              return immediate({ model, },);
            return lazyStream(model, async function refused() {
              await member?.onResponse?.({
                status: UNAVAILABLE_STATUS,
                headers: { 'Retry-After': String(RETRY_AFTER_SECONDS,), },
              }, model,);
              return immediate({ model, errorMessage: UNAVAILABLE_503, },);
            },);
          },
        },);
        const model = poolModel({ provider, id: 'writer-pool', },);
        await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        const held = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(calls.map(call => labelOf(call.model,),),).toEqual([
          labelOf(HEAD,),
          labelOf(SECOND,),
          labelOf(SECOND,),
        ],);
        expect(held.result.provider,).toBe(SECOND.provider,);
      },
    },),
  ],
},);
