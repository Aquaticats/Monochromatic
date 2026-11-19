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
