/**
 A member that cannot be reached before any output moves the request on, and is held for a while.
 @module
 */

import {
  createAssistantMessageEventStream,
} from '@earendil-works/pi-ai';
import { streamSimple as streamCompletions, } from '@earendil-works/pi-ai/api/openai-completions';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import extension, { UNREACHABLE_MEMBER_HOLD_MS, } from '../dist/final/node/index.mjs';
import { labelOf, WRITER_TARGETS, } from './member-order-fixture.ts';
import {
  assistant,
  collect,
  finish,
  fixture,
  immediate,
  type MemberCall,
  MEMBER_KEY,
  poolModel,
  TRANSCRIPT,
} from './stand-in-pi-fixture.ts';
import { TRANSPORT_FAILURES, } from './transport-failure-fixture.ts';

/** Writer head, the member every request starts at. */
const [HEAD, SECOND, THIRD,] = WRITER_TARGETS;

/**
 First pool attempt of a case, the one the unreachable member gets.
 */
const FIRST_ATTEMPT = 1;

/**
 Failure whose text carries a refusal status and a transport code, so only the status can decide.

 Made up: boundary is that a member which answered with an HTTP refusal is not a member that could
 not be reached, whatever its body names (`doc/captures.md`, "Made-up texts").
 */
const REFUSAL_NAMING_A_CODE = '400 Connection reset by an upstream proxy (ECONNRESET)';

/**
 Transport failure the whole pool reports when every member is unreachable.
 */
const EXHAUSTED_FAILURE = TRANSPORT_FAILURES['a refused connection'];

/**
 Drive one member that fails at the socket, as the installed Completions adapter reports it.

 @param call - member request the pool made

 @param failure - transport the member's provider stands behind

 @returns the member's own failed stream
 */
function unreachable({
  call,
  failure,
}: {
  readonly call: MemberCall;
  readonly failure: (typeof TRANSPORT_FAILURES)[keyof typeof TRANSPORT_FAILURES];
},) {
  return streamCompletions(
    {
      ...call.model,
      api: 'openai-completions',
      baseUrl: 'http://127.0.0.1:0/v1',
    },
    TRANSCRIPT,
    {
      ...call.options,
      apiKey: MEMBER_KEY,
      fetch: failure.respond,
      maxRetries: 0,
    },
  );
}

await describe({
  name: 'a member that cannot be reached',
  children: [
    //region Before any output

    ...Object.entries(TRANSPORT_FAILURES,)
      .map(function transportFailure([name, failure,],) {
        return it({
          name: `moves on from ${name} before any output`,
          fn: async function unreachableMember() {
            const { provider, calls, options, } = await fixture({
              extension,
              play(call,) {
                return calls.length === FIRST_ATTEMPT
                  ? unreachable({ call, failure, },)
                  : immediate({ model: call.model, },);
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
      name: 'does not move on from an HTTP refusal whose body names a transport code',
      fn: async function refusalNamingACode() {
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, }) { return immediate({ model, errorMessage: REFUSAL_NAMING_A_CODE, },); },
        },);
        const model = poolModel({ provider, id: 'writer-pool', },);
        const { result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        // The member answered, so its refusal is the request's answer; another member was not asked.
        expect(calls.map(call => labelOf(call.model,),),).toEqual([labelOf(HEAD,),],);
        expect(result.errorMessage,).toBe(REFUSAL_NAMING_A_CODE,);
      },
    },),
    it({
      name: 'does not move on from a transport failure after output was exposed',
      fn: async function afterOutput() {
        /**
         Text the member exposed before its connection died, so the caller already saw an answer.
         */
        const shown = 'Fixture answer cut short';
        const { provider, calls, options, } = await fixture({
          extension,
          play({ model, }) {
            const stream = createAssistantMessageEventStream();
            /**
             Partial message whose text the member exposed before the socket closed.
             */
            const partial = assistant({ model, },);
            partial.content.push({ type: 'text', text: shown, },);
            stream.push({ type: 'start', partial, },);
            stream.push({ type: 'text_start', contentIndex: 0, partial, },);
            finish({ stream, message: assistant({
              model,
              errorMessage: TRANSPORT_FAILURES['a reset after a successful response header'].text,
            },), },);
            return stream;
          },
        },);
        const model = poolModel({ provider, id: 'writer-pool', },);
        const { events, result, } = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(events.map(event => event.type,),).toEqual(['start', 'text_start', 'error',],);
        expect(result.errorMessage,).toBe(TRANSPORT_FAILURES['a reset after a successful response header'].text,);
        expect(calls.map(call => labelOf(call.model,),),).toEqual([labelOf(HEAD,),],);
      },
    },),

    //endregion Before any output

    //region The hold

    it({
      name: 'holds the member it could not reach so the next request skips it',
      fn: async function holdUnreachable(ctx,) {
        const clock = { now: Date.now(), };
        ctx.sinon.stub(Date, 'now',).callsFake(() => clock.now,);
        const { provider, calls, options, } = await fixture({
          extension,
          play(call,) {
            return calls.length === FIRST_ATTEMPT
              ? unreachable({ call, failure: EXHAUSTED_FAILURE, },)
              : immediate({ model: call.model, },);
          },
        },);
        const model = poolModel({ provider, id: 'writer-pool', },);
        const first = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(first.result.provider,).toBe(SECOND.provider,);
        // The next request arrives inside the hold, so the head is not asked again.
        clock.now += UNREACHABLE_MEMBER_HOLD_MS / 2;
        const second = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(calls.map(call => labelOf(call.model,),),).toEqual([
          labelOf(HEAD,),
          labelOf(SECOND,),
          labelOf(SECOND,),
        ],);
        expect(second.result.provider,).toBe(SECOND.provider,);
        // The hold is not a demotion: once it passes, the head is asked again.
        clock.now += UNREACHABLE_MEMBER_HOLD_MS;
        await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(calls.map(call => labelOf(call.model,),),).toEqual([
          labelOf(HEAD,),
          labelOf(SECOND,),
          labelOf(SECOND,),
          labelOf(HEAD,),
        ],);
      },
    },),
    it({
      name: 'warns about the move in words of its own, naming the member',
      fn: async function moveLog(ctx,) {
        const warn = ctx.sinon.stub(console, 'warn',);
        const { provider, calls, options, } = await fixture({
          extension,
          play(call,) {
            return calls.length === FIRST_ATTEMPT
              ? unreachable({ call, failure: EXHAUSTED_FAILURE, },)
              : immediate({ model: call.model, },);
          },
        },);
        const model = poolModel({ provider, id: 'writer-pool', },);
        await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        /**
         Lines this case's move produced, apart from anything else the pool logged.
         */
        const moves = warn.getCalls()
          .map(call => String(call.args.at(0,),),)
          .filter(line => line.includes('could not be reached',),);
        expect(moves,).toHaveLength(1,);
        const line = moves.at(0,) ?? '';
        expect(line,).toContain(labelOf(HEAD,),);
        // The wording separates an unreachable member from a member that refused a request.
        expect(line,).toContain(EXHAUSTED_FAILURE.code,);
        expect(line,).toContain(String(UNREACHABLE_MEMBER_HOLD_MS,),);
        expect(line,).not.toContain('429',);
      },
    },),
    it({
      name: 'asks the held members anyway when no other member can be tried',
      fn: async function heldPoolStillTries(ctx,) {
        const clock = { now: Date.now(), };
        ctx.sinon.stub(Date, 'now',).callsFake(() => clock.now,);
        const { provider, calls, options, } = await fixture({
          extension,
          play(call,) { return unreachable({ call, failure: EXHAUSTED_FAILURE, },); },
        },);
        const model = poolModel({ provider, id: 'writer-pool', },);
        const first = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        // Every member failed the same way, so the last refusal is the request's answer, unchanged.
        expect(first.result.stopReason,).toBe('error',);
        expect(first.result.errorMessage,).toBe(EXHAUSTED_FAILURE.text,);
        expect(first.result.provider,).toBe(THIRD.provider,);
        clock.now += UNREACHABLE_MEMBER_HOLD_MS / 2;
        const second = await collect(provider.streamSimple(model, TRANSCRIPT, options,),);
        expect(second.result.errorMessage,).toBe(EXHAUSTED_FAILURE.text,);
        expect(calls.map(call => labelOf(call.model,),),).toEqual([
          labelOf(HEAD,),
          labelOf(SECOND,),
          labelOf(THIRD,),
          labelOf(HEAD,),
          labelOf(SECOND,),
          labelOf(THIRD,),
        ],);
      },
    },),

    //endregion The hold
  ],
},);
