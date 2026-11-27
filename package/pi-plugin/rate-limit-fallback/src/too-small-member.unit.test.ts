/**
 A member whose own window cannot hold the request is passed over, and the pool advertises its head's
 window (owner, 2026-11-27).
 @module
 */

import {
  normalizeContext,
  type Api,
  type Model,
  type TranscriptContext,
} from '@earendil-works/pi-ai';
import { clampMaxTokensToContext, } from '@earendil-works/pi-ai/api/simple-options';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import extension, { estimateRequestTokens, } from '../dist/final/node/index.mjs';
import { labelOf, WRITER_TARGETS, } from './member-order-fixture.ts';
import {
  MEMBER_LIMITS,
  memberLimitsOf,
  paddingAboveWindow,
  paddingBelowWindow,
  paddingNearWindow,
} from './member-limits-fixture.ts';
import {
  collect,
  fixture,
  immediate,
  poolModel,
  rateLimitRefusal,
} from './stand-in-pi-fixture.ts';

/**
 Writer members in the pool's order: the head, the member the owner's first example passes over, the
 member that answers it, and the last member.
 */
const [HEAD, SECOND, THIRD, LAST,] = WRITER_TARGETS;

/**
 Contract of the four members' windows and output limits, read from the fixture that records them.
 */
const HEAD_LIMITS = memberLimitsOf(HEAD,);

/**
 Limits the second member reports, the one the owner's first example is too large for.
 */
const SECOND_LIMITS = memberLimitsOf(SECOND,);

/**
 Limits the last member reports, the pool's largest window.
 */
const LAST_LIMITS = memberLimitsOf(LAST,);

/**
 One member with the window and output limit it reports, so a request's size decides whether it is
 asked.

 @param target - catalog record naming the member

 @returns that record carrying the member's own reported limits
 */
function withOwnLimits(target: Model<Api>,): Model<Api> {
  return {
    ...target,
    ...memberLimitsOf(target,),
  };
}

/**
 Every member registered with the window and output limit it reports, so a request's size decides
 which of them is asked.
 */
const MEMBER_MODELS: readonly Model<Api>[] = WRITER_TARGETS.map(function each(target,) {
  return withOwnLimits(target,);
},);

/**
 System message every case's transcript opens with, sized like the package's own fixture transcript.

 Made up: it stands for no program's output.
 */
const FIXTURE_SYSTEM_TEXT = 'Fixture system instruction';

/**
 One request carrying the given padding, with a system message in front of it.

 @param padding - user text whose length sets the request's estimated size

 @returns normalized transcript, as Pi hands it to a provider

 @example
 ```ts
 requestCarrying(paddingAboveWindow(SECOND_LIMITS.contextWindow));
 ```
 */
function requestCarrying(padding: string,): TranscriptContext {
  return normalizeContext({
    messages: [
      {
        role: 'system',
        content: FIXTURE_SYSTEM_TEXT,
        timestamp: 0,
      },
      {
        role: 'user',
        content: padding,
        timestamp: 0,
      },
    ],
  },);
}

/**
 Dispatch one request through the writer pool of a stand-in host.

 @param options - request to send and how the stand-in members answer

 @returns member calls, the terminal result and the pool entry Pi would size the request from

 @example
 ```ts
 await dispatch({ request: TRANSCRIPT, play: ({ model }) => immediate({ model }) });
 ```
 */
async function dispatch({
  request,
  play,
}: {
  readonly request: TranscriptContext;
  play: Parameters<typeof fixture>[0]['play'];
},) {
  const { provider, calls, options, } = await fixture({
    extension,
    models: MEMBER_MODELS,
    play,
  },);
  const pool = poolModel({
    provider,
    id: 'writer-pool',
  },);
  const { result, } = await collect(provider.streamSimple(
    pool,
    request,
    {
      ...options,
      // Pi clamps a request's output limit to the routed model before the pool sees it
      // (`@earendil-works/pi-coding-agent` dist/core/model-runtime.js:515), so the pool receives the
      // limit its own catalog advertises.
      maxTokens: pool.maxTokens,
    },
  ),);
  return {
    calls,
    result,
    pool,
  };
}

/**
 Answer with a refusal at the head and a completed answer everywhere else.

 @param call - member request the stand-in host is answering

 @returns the member's own stream
 */
function refuseAtTheHead({ model, }: { readonly model: Model<Api> },) {
  return immediate({
    model,
    ...(labelOf(model,) === labelOf(HEAD,)
      ? { errorMessage: rateLimitRefusal({ model, },), }
      : {}),
  },);
}

await describe({
  name: 'a member too small for the request',
  children: [
    //region The owner's two examples

    it({
      name: 'passes over the second member for a request larger than its window and lets the next one answer',
      fn: async function aboveSecondWindow() {
        /**
         Request larger than the second member's window, sized the way the owner's first example
         sizes one: too big for Synthetic's member and well inside the head's.
         */
        const request = requestCarrying(paddingAboveWindow(SECOND_LIMITS.contextWindow,),);
        // Controls on the case's own claim: the request really is too large for the member it must
        // pass over, and well inside the window of the member that answers.
        expect(estimateRequestTokens(request,),).toBeGreaterThan(SECOND_LIMITS.contextWindow,);
        expect(estimateRequestTokens(request,),).toBeLessThan(HEAD_LIMITS.contextWindow,);
        const { calls, result, } = await dispatch({
          request,
          play: refuseAtTheHead,
        },);
        expect(calls.map(call => labelOf(call.model,),),).toEqual([labelOf(HEAD,), labelOf(THIRD,),],);
        expect(result.provider,).toBe(THIRD.provider,);
        expect(result.stopReason,).toBe('stop',);
      },
    },),
    it({
      name: 'asks the second member for a request its own window can hold',
      fn: async function insideSecondWindow() {
        /**
         Request the second member can hold, sized the way the owner's second example sizes one.
         */
        const request = requestCarrying(paddingBelowWindow(SECOND_LIMITS.contextWindow,),);
        expect(estimateRequestTokens(request,),).toBeLessThan(SECOND_LIMITS.contextWindow,);
        const { calls, result, } = await dispatch({
          request,
          play: refuseAtTheHead,
        },);
        expect(calls.map(call => labelOf(call.model,),),).toEqual([labelOf(HEAD,), labelOf(SECOND,),],);
        expect(result.provider,).toBe(SECOND.provider,);
        expect(result.stopReason,).toBe('stop',);
      },
    },),

    //endregion The owner's two examples

    //region A request no member can hold

    it({
      name: 'names every member and why it was not asked when no member can hold the request',
      fn: async function noMemberHoldsIt() {
        /**
         Request larger than the pool's largest window, so every member reports one too small.
         */
        const request = requestCarrying(paddingAboveWindow(LAST_LIMITS.contextWindow,),);
        const sizes = MEMBER_LIMITS.map(member => member.contextWindow,);
        expect(estimateRequestTokens(request,),).toBeGreaterThan(Math.max(...sizes,),);
        const { calls, result, } = await dispatch({
          request,
          play: refuseAtTheHead,
        },);
        // No member was asked: the size is decided before anything is sent.
        expect(calls,).toEqual([],);
        expect(result.stopReason,).toBe('error',);
        const message = result.errorMessage ?? '';
        expect(message,).toContain('no eligible writer-pool member',);
        for (const member of MEMBER_LIMITS) {
          // Each member is named with the window it reports, so a reader can tell which member was
          // left out and by how much.
          expect(message,).toContain(`${member.provider}/${member.id}`,);
          expect(message,).toContain(`its window is ${String(member.contextWindow,)} tokens`,);
        }
        expect(message,).toContain(`estimated at ${String(estimateRequestTokens(request,),)} tokens`,);
      },
    },),

    //endregion A request no member can hold

    //region One request only

    it({
      name: 'asks the member it passed over again on the next request',
      fn: async function passedOverForOneRequest() {
        /**
         Request too large for the second member, which the third answers.
         */
        const large = requestCarrying(paddingAboveWindow(SECOND_LIMITS.contextWindow,),);
        /**
         Request the second member can hold, sent afterwards to the same pool.
         */
        const small = requestCarrying(paddingBelowWindow(SECOND_LIMITS.contextWindow,),);
        const { provider, calls, options, } = await fixture({
          extension,
          models: MEMBER_MODELS,
          play: refuseAtTheHead,
        },);
        const pool = poolModel({
          provider,
          id: 'writer-pool',
        },);
        // One session, one member pool: the second request is the only thing that could remember the
        // first one's decision.
        const first = await collect(provider.streamSimple(pool, large, options,),);
        expect(first.result.provider,).toBe(THIRD.provider,);
        const second = await collect(provider.streamSimple(pool, small, options,),);
        expect(second.result.provider,).toBe(SECOND.provider,);
        expect(calls.map(call => labelOf(call.model,),),).toEqual([
          labelOf(HEAD,),
          labelOf(THIRD,),
          labelOf(HEAD,),
          labelOf(SECOND,),
        ],);
      },
    },),
    it({
      name: 'asks a member that reports no window rather than passing it over',
      fn: async function noReportedWindow() {
        /**
         Request larger than the second member's own window, which this case's catalog no longer
         reports.
         */
        const request = requestCarrying(paddingAboveWindow(SECOND_LIMITS.contextWindow,),);
        const { provider, calls, options, } = await fixture({
          extension,
          models: MEMBER_MODELS.map(function withoutSecondWindow(model,) {
            return labelOf(model,) === labelOf(SECOND,)
              ? {
                ...model,
                contextWindow: 0,
              }
              : model;
          },),
          play: refuseAtTheHead,
        },);
        const pool = poolModel({
          provider,
          id: 'writer-pool',
        },);
        const { result, } = await collect(provider.streamSimple(pool, request, options,),);
        // A member that reports no window says nothing about what it can hold, so the pool asks it and
        // leaves the answer to the member, as it did before this change.
        expect(calls.map(call => labelOf(call.model,),),).toEqual([labelOf(HEAD,), labelOf(SECOND,),],);
        expect(result.provider,).toBe(SECOND.provider,);
      },
    },),

    //endregion One request only

    //region The edge where the input fits and the output limit does not

    it({
      name: 'asks the second member when its input fits that member’s window but the input plus the output limit does not',
      fn: async function inputFitsOutputDoesNot() {
        /**
         Request whose input fits the second member's window and leaves less room than that member's
         own output limit.
         */
        const request = requestCarrying(paddingNearWindow(SECOND_LIMITS.contextWindow,),);
        /**
         Input tokens this request is estimated to carry.
         */
        const input = estimateRequestTokens(request,);
        // Controls on the case's own edge: the input alone fits, and the input plus the output limit
        // the request would carry there does not.
        expect(input,).toBeLessThan(SECOND_LIMITS.contextWindow,);
        expect(input,).toBeGreaterThan(SECOND_LIMITS.contextWindow - SECOND_LIMITS.maxTokens,);
        expect(input + SECOND_LIMITS.maxTokens,).toBeGreaterThan(SECOND_LIMITS.contextWindow,);
        const { calls, result, } = await dispatch({
          request,
          play: refuseAtTheHead,
        },);
        // The pool asks the member anyway: its window holds the request, and Pi fits the answer into
        // what the window leaves instead of the pool passing the member over for room the answer
        // might want.
        expect(calls.map(call => labelOf(call.model,),),).toEqual([labelOf(HEAD,), labelOf(SECOND,),],);
        expect(result.provider,).toBe(SECOND.provider,);
        /**
         Output limit the request carries once it reaches the second member: that member's own.
         */
        const carried = calls.at(1,)
          ?.options?.maxTokens;
        expect(carried,).toBe(SECOND_LIMITS.maxTokens,);
        // Pi's own clamp is what fits the pair into the window, and it is the member's installed
        // adapter that applies it, so the sum a member receives never exceeds its window.
        const fitted = clampMaxTokensToContext(
          withOwnLimits(SECOND,),
          request,
          carried ?? SECOND_LIMITS.maxTokens,
        );
        expect(fitted,).toBeLessThan(SECOND_LIMITS.maxTokens,);
        expect(input + fitted,).toBeLessThanOrEqual(SECOND_LIMITS.contextWindow,);
      },
    },),

    //endregion The edge where the input fits and the output limit does not

    //region The output limit

    it({
      name: 'advertises the head’s output limit and clamps a request it sends to the second member at that member’s own',
      fn: async function outputLimitClamp() {
        /**
         Request the second member can hold.
         */
        const request = requestCarrying(paddingBelowWindow(SECOND_LIMITS.contextWindow,),);
        const refused = await dispatch({
          request,
          play: refuseAtTheHead,
        },);
        // Control: the pool advertises the head's output limit, which is larger than the second
        // member's own.
        expect(refused.pool.maxTokens,).toBe(HEAD_LIMITS.maxTokens,);
        expect(refused.pool.maxTokens,).toBeGreaterThan(SECOND_LIMITS.maxTokens,);
        // The request the pool sent to the second member carries that member's own limit, not the
        // pool's advertised one.
        const answered = refused.calls.at(1,);
        expect(answered === undefined ? undefined : labelOf(answered.model,),).toBe(labelOf(SECOND,),);
        expect(answered?.options?.maxTokens,).toBe(SECOND_LIMITS.maxTokens,);
        // Control: the same request answered by the head carries the head's own limit, so the clamp is
        // what moved and not a value every attempt already had.
        const atHead = await dispatch({
          request,
          play({ model, }) { return immediate({ model, },); },
        },);
        expect(atHead.calls.at(0,)
          ?.options?.maxTokens,).toBe(HEAD_LIMITS.maxTokens,);
      },
    },),

    //endregion The output limit
  ],
},);
