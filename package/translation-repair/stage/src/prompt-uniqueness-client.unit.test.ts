/**
 Tests process-local model and substantive prompt uniqueness.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  alternateModelsFor,
  type ChatTextRequest,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import {
  noteRunSpend,
  resetRunSpend,
  runSpendUsd,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { infoLinesDuring, } from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';
import { MalformedCompletionError, } from '@monochromatic-dev/translation-repair-provider-stream';
import {
  createSeatTally,
  CUT_SHORT_RECOVERY_NUDGE,
  modelPromptDigest,
  OFF_SHAPE_RECOVERY_NUDGE,
  promptPayloadStore,
  promptUniqueClient,
  withheldSeatRefusal,
} from '../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { SEAT_SYNTHETIC_VISION_WITHHELD, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

/**
 Guard accepting any string, so a quoted JSON string is a usable answer and
 anything that is not JSON is not.

 @param value - parsed reply

 @returns Whether it is a string

 @example
 ```ts
 isStringAnswer('windowsill',);
 ```
 */
function isStringAnswer(value: unknown,): value is string {
  return (typeof value) === 'string';
}

/**
 Exact prompt reused across boundary cases.
 */
const REQUEST: ChatTextRequest = {
  modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
  messages: [{ role: 'user', content: 'Judge this one cat sentence.', },],
  signal: AbortSignal.timeout(HANG_STOP_MS,),
};

/**
 Client whose first reply is a reasoning runaway (a length stop with no content) and whose every later reply is a usable
 answer, which writes every request it is handed to a sink.

 @param asked - sink for the requests, in order

 @returns Client over the scripted replies

 @example
 ```ts
 const inner = runawayThenAnswer({ asked: [], },);
 ```
 */
function runawayThenAnswer({ asked, }: { readonly asked: ChatTextRequest[]; },): ModelClient {
  return {
    chatText: async (request,) => {
      asked.push(request,);
      return (asked.length === 1)
        ? { text: '', finishReason: 'length', servedBy: 'neuralwatt', }
        : { text: '"windowsill"', servedBy: 'hyper', };
    },
    chatJson: async () => {
      throw new Error('chatJson bypassed by prompt payload reader',);
    },
    quotas: async () => {
      throw new Error('quotas unused by prompt uniqueness fixture',);
    },
  };
}

await describe({
  name: promptUniqueClient.name,
  // SEQUENTIAL: the meter case diverts console.info and reads the process-wide run spend meter, so cases must not overlap.
  concurrency: 1,
  children: [
    it({
      name: 'REUSES COMPLETED PAYLOAD instead of second provider call',
      fn: async () => {
        /** Provider calls crossing wrapper. */
        let providerCalls = 0;
        const inner: ModelClient = {
          chatText: async () => {
            providerCalls += 1;
            return { text: 'first payload', };
          },
          chatJson: async () => {
            throw new Error('chatJson unused by text fixture',);
          },
          quotas: async () => {
            throw new Error('quotas unused by prompt uniqueness fixture',);
          },
        };
        const client = promptUniqueClient({ inner, },);
        expect(await client.chatText(REQUEST,),).toEqual({ text: 'first payload', },);
        expect(await client.chatText(REQUEST,),).toEqual({ text: 'first payload', },);
        expect(providerCalls,).toBe(1,);
      },
    },),

    it({
      name: 'REUSES JSON PAYLOAD when only response schema metadata changes',
      fn: async () => {
        /** Provider calls crossing wrapper. */
        let providerCalls = 0;
        const inner: ModelClient = {
          chatText: async () => {
            providerCalls += 1;
            return { text: 'not-json', };
          },
          chatJson: async () => {
            throw new Error('chatJson bypassed by prompt payload reader',);
          },
          quotas: async () => {
            throw new Error('quotas unused by prompt uniqueness fixture',);
          },
        };
        const client = promptUniqueClient({ inner, },);
        const first = await client.chatJson({
          ...REQUEST,
          responseFormat: {
            type: 'json_schema',
            json_schema: {
              name: 'first_shape',
              schema: { type: 'object', },
            },
          },
          validate: function acceptsString(value: unknown,): value is string {
            return (typeof value) === 'string';
          },
        },);

        const second = await client.chatJson({
          ...REQUEST,
          responseFormat: {
            type: 'json_schema',
            json_schema: {
              name: 'second_shape',
              schema: { type: 'string', },
            },
          },
          validate: function acceptsString(value: unknown,): value is string {
            return (typeof value) === 'string';
          },
        },);
        expect(first.kind,).toBe('schema-mismatch',);
        expect(second.kind,).toBe('schema-mismatch',);
        expect(providerCalls,).toBe(1,);
      },
    },),

    it({
      name: 'RELEASES IDENTITY after transport throws without payload',
      fn: async () => {
        /** Provider calls crossing wrapper. */
        let providerCalls = 0;
        const inner: ModelClient = {
          chatText: async () => {
            providerCalls += 1;
            if (providerCalls === 1)
              throw new Error('connection reset before payload',);
            return { text: 'recovered payload', };
          },
          chatJson: async () => {
            throw new Error('chatJson unused by text fixture',);
          },
          quotas: async () => {
            throw new Error('quotas unused by prompt uniqueness fixture',);
          },
        };
        const client = promptUniqueClient({ inner, },);
        // ASSERTED OUTSIDE A CATCH: a first call that did not throw would
        // otherwise pass untested (ledger T2).
        await expect(client.chatText(REQUEST,),).rejects.toThrow('connection reset before payload',);
        expect(await client.chatText(REQUEST,),).toEqual({ text: 'recovered payload', },);
        expect(providerCalls,).toBe(2,);
      },
    },),

    it({
      name: 'KEEPS IDENTITY CLAIMED after malformed completed payload throws',
      fn: async () => {
        /** Provider calls crossing wrapper. */
        let providerCalls = 0;
        const inner: ModelClient = {
          chatText: async () => {
            providerCalls += 1;
            throw new MalformedCompletionError({
              wireFormat: 'openai',
              detail: 'completed payload lacked choices',
            },);
          },
          chatJson: async () => {
            throw new Error('chatJson unused by malformed fixture',);
          },
          quotas: async () => {
            throw new Error('quotas unused by prompt uniqueness fixture',);
          },
        };
        const client = promptUniqueClient({ inner, },);
        await expect(client.chatText(REQUEST,),).rejects.toThrow(MalformedCompletionError,);
        await expect(client.chatText(REQUEST,),).rejects.toThrow(MalformedCompletionError,);
        expect(providerCalls,).toBe(1,);
      },
    },),

    it({
      name: 'CANONICALIZES MESSAGE KEYS while preserving changed content',
      fn: async () => {
        /** Digest from role-first message construction. */
        const roleFirst = modelPromptDigest({
          request: {
            ...REQUEST,
            messages: [{ role: 'user', content: 'same content', },],
          },
        },);
        /** Digest from content-first message construction. */
        const contentFirst = modelPromptDigest({
          request: {
            ...REQUEST,
            messages: [{ content: 'same content', role: 'user', },],
          },
        },);
        /** Digest after substantive message change. */
        const changed = modelPromptDigest({
          request: {
            ...REQUEST,
            messages: [{ role: 'user', content: 'different content', },],
          },
        },);
        expect(contentFirst,).toBe(roleFirst,);
        expect(changed,).not.toBe(roleFirst,);
      },
    },),

    it({
      name: 'CANONICALIZES CONTENT PART KEYS in a picture prompt while another picture is another prompt',
      fn: async () => {
        /** Digest from parts written type first. */
        const typeFirst = modelPromptDigest({
          request: {
            ...REQUEST,
            messages: [{
              role: 'user',
              content: [
                { type: 'text', text: 'Read this cat picture.', },
                { type: 'image_url', image_url: { url: 'data:image/png;base64,AAAA', }, },
              ],
            },],
          },
        },);
        /** Digest from the same parts written type last. */
        const typeLast = modelPromptDigest({
          request: {
            ...REQUEST,
            messages: [{
              role: 'user',
              content: [
                { text: 'Read this cat picture.', type: 'text', },
                { image_url: { url: 'data:image/png;base64,AAAA', }, type: 'image_url', },
              ],
            },],
          },
        },);
        /** Digest after the picture changes and the words do not. */
        const otherPicture = modelPromptDigest({
          request: {
            ...REQUEST,
            messages: [{
              role: 'user',
              content: [
                { type: 'text', text: 'Read this cat picture.', },
                { type: 'image_url', image_url: { url: 'data:image/png;base64,BBBB', }, },
              ],
            },],
          },
        },);
        expect(typeLast,).toBe(typeFirst,);
        expect(otherPicture,).not.toBe(typeFirst,);
      },
    },),

    it({
      name: 'SHARES IN-FLIGHT PAYLOAD before second provider call',
      fn: async () => {
        /** Provider calls crossing wrapper. */
        let providerCalls = 0;
        const inner: ModelClient = {
          chatText: async () => {
            providerCalls += 1;
            return { text: 'single payload', };
          },
          chatJson: async () => {
            throw new Error('chatJson unused by text fixture',);
          },
          quotas: async () => {
            throw new Error('quotas unused by prompt uniqueness fixture',);
          },
        };
        const client = promptUniqueClient({ inner, },);
        const first = client.chatText(REQUEST,);
        const second = client.chatText(REQUEST,);
        expect(await second,).toEqual({ text: 'single payload', },);
        expect(await first,).toEqual({ text: 'single payload', },);
        expect(providerCalls,).toBe(1,);
      },
    },),

    it({
      name: 'RE-ASKS ANOTHER PROVIDER WITH A NUDGE when the reply could not be used (ledger P9, owner '
        + '2026-09-28 "Enable with the nudge"): the nudged prompt is a new digest routed away from the '
        + 'provider that served the first, and both exchanges stay claimed',
      fn: async () => {
        /** Every request that crossed the wrapper, in order. */
        const asked: ChatTextRequest[] = [];
        const inner: ModelClient = {
          chatText: async (request,) => {
            asked.push(request,);
            return (asked.length === 1)
              ? { text: 'not-json', servedBy: 'synthetic', }
              : { text: '"windowsill"', servedBy: 'hyper', };
          },
          chatJson: async () => {
            throw new Error('chatJson bypassed by prompt payload reader',);
          },
          quotas: async () => {
            throw new Error('quotas unused by prompt uniqueness fixture',);
          },
        };
        const client = promptUniqueClient({ inner, },);
        const outcome = await client.chatJson({ ...REQUEST, validate: isStringAnswer, },);
        expect(outcome.kind,).toBe('ok',);
        expect(asked,).toHaveLength(2,);
        expect(asked[1]?.otherThan,).toBe('synthetic',);
        expect(asked[1]?.messages.slice(0, -1,),).toStrictEqual(REQUEST.messages,);

        // A WORDING OF ITS OWN, so either of the stage recovery round's nudged
        // prompts is still a new digest rather than a replay of this re-ask.
        expect(asked[1]?.messages.at(-1,),).not.toStrictEqual(OFF_SHAPE_RECOVERY_NUDGE,);
        expect(asked[1]?.messages.at(-1,),).not.toStrictEqual(CUT_SHORT_RECOVERY_NUDGE,);

        // Asked again, both answers come from the claims.
        const again = await client.chatJson({ ...REQUEST, validate: isStringAnswer, },);
        expect(again.kind,).toBe('ok',);
        expect(asked,).toHaveLength(2,);
      },
    },),

    it({
      name: 'RE-ASKS A REASONING RUNAWAY OF ANOTHER MODEL through the same claims (ledger B1072, B1076): a first reply '
        + 'that ended in a length stop with no content is re-asked under another model\'s name with no provider hint and '
        + 'no nudge, so its digest is the one that model\'s own first ask of the question has, and a later ask of the '
        + 'question replays both exchanges',
      fn: async () => {
        /** Every request that crossed the wrapper, in order. */
        const asked: ChatTextRequest[] = [];
        const inner = runawayThenAnswer({ asked, },);
        const client = promptUniqueClient({ inner, },);
        // A RUNAWAY NAMES ITS SEAT, or no other model is asked of it (ledger B1079).
        const seated = { ...REQUEST, validate: isStringAnswer, reaskSeat: { role: 'roster', reask: 'other-model', bench: [], }, } as const;
        const outcome = await client.chatJson(seated,);
        expect(outcome.kind,).toBe('ok',);
        // THE MODEL THAT ANSWERED RIDES ON THE OUTCOME through the wrapper, so a caller can record its answer as its own
        // (ledger B1102).
        expect(outcome.kind === 'ok' ? outcome.answeredBy : 'not ok',).toBe(asked[1]?.modelId,);
        expect(asked,).toHaveLength(2,);
        expect(asked[1]?.modelId,).not.toBe(REQUEST.modelId,);
        expect(asked[1]?.otherThan,).toBeUndefined();
        // THE PLAIN QUESTION: the other model is asked the messages as they stood, with no nudge after them.
        expect(asked[1]?.messages,).toStrictEqual(REQUEST.messages,);
        expect(modelPromptDigest({ request: asked[1] ?? REQUEST, },),).not.toBe(modelPromptDigest({ request: REQUEST, },),);
        // ITS DIGEST IS THAT MODEL'S OWN FIRST ASK OF THE QUESTION, so a payload stored for that ask is replayed.
        expect(modelPromptDigest({ request: asked[1] ?? REQUEST, },),).toBe(
          modelPromptDigest({ request: { ...REQUEST, modelId: asked[1]?.modelId ?? REQUEST.modelId, }, },),
        );

        // Asked again, both answers come from the claims.
        const again = await client.chatJson(seated,);
        expect(again.kind,).toBe('ok',);
        expect(asked,).toHaveLength(2,);
      },
    },),

    it({
      name: 'NEVER RE-ASKS A MODEL THE RUN\'S TALLY WITHHELD (ledger B1162): the tally sits outside this wrapper, so the '
        + 'refusal it gives its own door is handed in and read at the re-ask too, and the next model of the order is asked',
      fn: async () => {
        /**
         Models a runaway of this seat could be re-asked of, in order.
         */
        const { alternates, } = alternateModelsFor({ modelId: REQUEST.modelId, role: 'roster', bench: [], },);
        const [withheld, next,] = alternates;
        if ((withheld === undefined) || (next === undefined))
          throw new Error('the seat has two models to try',);
        /**
         Tally that has seen the first model throw on its only ask.
         */
        const tally = createSeatTally();
        tally.record({ modelId: withheld, outcome: 'threw', },);
        /** Every request that crossed the wrapper, in order. */
        const asked: ChatTextRequest[] = [];
        const inner = runawayThenAnswer({ asked, },);
        const client = promptUniqueClient({ inner, refuseSeat: withheldSeatRefusal({ tally, minAsks: 1, },), },);
        const outcome = await client.chatJson(
          { ...REQUEST, validate: isStringAnswer, reaskSeat: { role: 'roster', bench: [], reask: 'other-model', }, },
        );
        expect(asked.map(function modelOf(request,) {
          return request.modelId;
        },),).toEqual([REQUEST.modelId, next,],);
        expect(outcome.kind === 'ok' ? outcome.answeredBy : 'not ok',).toBe(next,);
      },
    },),

    it({
      name: 'READS A CLAIMED PAYLOAD AGAINST THE RESPONSE FORMAT OF THE REQUEST, taking the schema\'s spelling of a '
        + 'listed value that came back differing only in its capital',
      fn: async () => {
        const inner: ModelClient = {
          chatText: async () => ({ text: '{"spot":"Windowsill"}', servedBy: 'synthetic', }),
          chatJson: async () => {
            throw new Error('chatJson bypassed by prompt payload reader',);
          },
          quotas: async () => {
            throw new Error('quotas unused by prompt uniqueness fixture',);
          },
        };
        const client = promptUniqueClient({ inner, },);
        const outcome = await client.chatJson({
          ...REQUEST,
          responseFormat: {
            type: 'json_schema',
            json_schema: {
              name: 'nap_spot',
              schema: {
                type: 'object',
                properties: { spot: { type: 'string', enum: ['windowsill', 'radiator',], }, },
              },
            },
          },
          validate: function isListedSpot(value: unknown,): value is { readonly spot: string; } {
            return ((typeof value) === 'object') && (value !== null) && ('spot' in value)
              && ((value.spot === 'windowsill') || (value.spot === 'radiator'));
          },
        },);
        expect(outcome.kind === 'ok' ? outcome.value : undefined,).toEqual({ spot: 'windowsill', },);
      },
    },),

    it({
      name: 'KEEPS THE FIRST ANSWER when no other provider can take the re-ask, and releases the nudged '
        + 'prompt, so a later ask of the same question may still buy it',
      fn: async () => {
        /** Every request that crossed the wrapper, in order. */
        const asked: ChatTextRequest[] = [];
        const inner: ModelClient = {
          chatText: async (request,) => {
            asked.push(request,);
            if (asked.length === 2)
              throw new NoProviderForModelError({ modelId: request.modelId, reason: 'no other provider', },);
            return (asked.length === 1)
              ? { text: 'not-json', servedBy: 'synthetic', }
              : { text: '"windowsill"', servedBy: 'hyper', };
          },
          chatJson: async () => {
            throw new Error('chatJson bypassed by prompt payload reader',);
          },
          quotas: async () => {
            throw new Error('quotas unused by prompt uniqueness fixture',);
          },
        };
        const client = promptUniqueClient({ inner, },);
        const first = await client.chatJson({ ...REQUEST, validate: isStringAnswer, },);
        expect(first.kind,).toBe('schema-mismatch',);
        expect(asked,).toHaveLength(2,);

        const later = await client.chatJson({ ...REQUEST, validate: isStringAnswer, },);
        expect(later.kind,).toBe('ok',);
        expect(asked,).toHaveLength(3,);
      },
    },),

    it({
      name: 'ASKS THE MODEL AGAIN FOR THE SAME PROMPT UNDER ANOTHER REUSE SCOPE, and still reuses it inside one scope: '
        + 'two scopes are two provider calls, a third ask under the first scope is the first call\'s payload',
      fn: async () => {
        /** Provider calls crossing wrapper. */
        let providerCalls = 0;
        const inner: ModelClient = {
          chatText: async () => {
            providerCalls += 1;
            return { text: `payload ${String(providerCalls,)}`, };
          },
          chatJson: async () => {
            throw new Error('chatJson unused by text fixture',);
          },
          quotas: async () => {
            throw new Error('quotas unused by prompt uniqueness fixture',);
          },
        };
        const client = promptUniqueClient({ inner, },);
        expect(await client.chatText({ ...REQUEST, reuseScope: 'pass 1', },),).toEqual({ text: 'payload 1', },);
        expect(await client.chatText({ ...REQUEST, reuseScope: 'pass 2', },),).toEqual({ text: 'payload 2', },);
        expect(providerCalls,).toBe(2,);
        expect(await client.chatText({ ...REQUEST, reuseScope: 'pass 1', },),).toEqual({ text: 'payload 1', },);
        expect(providerCalls,).toBe(2,);
      },
    },),

    it({
      name: 'ASKS THE MODEL AGAIN UNDER ANOTHER REUSE SCOPE THROUGH THE DURABLE STORE: a client sharing the first '
        + 'client\'s payload directory replays the payload of its own scope and buys the payload of a scope the '
        + 'store holds none for',
      fn: async () => {
        await using dir = await scratchDir({ prefix: 'prompt-reuse-scope-', },);
        /** Provider calls crossing the wrappers. */
        let providerCalls = 0;
        const inner: ModelClient = {
          chatText: async () => {
            providerCalls += 1;
            return { text: `payload ${String(providerCalls,)}`, };
          },
          chatJson: async () => {
            throw new Error('chatJson unused by text fixture',);
          },
          quotas: async () => {
            throw new Error('quotas unused by prompt uniqueness fixture',);
          },
        };
        const first = promptUniqueClient({ inner, store: promptPayloadStore({ dir: dir.path, },), },);
        await first.chatText({ ...REQUEST, reuseScope: 'pass 1', },);
        expect(providerCalls,).toBe(1,);

        const second = promptUniqueClient({ inner, store: promptPayloadStore({ dir: dir.path, },), },);
        expect(await second.chatText({ ...REQUEST, reuseScope: 'pass 1', },),).toEqual({ text: 'payload 1', },);
        expect(providerCalls,).toBe(1,);
        expect(await second.chatText({ ...REQUEST, reuseScope: 'pass 2', },),).toEqual({ text: 'payload 2', },);
        expect(providerCalls,).toBe(2,);
      },
    },),

    it({
      name: 'BOOKS NOTHING ON THE RUN METER FOR A REUSE, from memory or from the durable store: the meter holds the one '
        + 'priced call, and the log holds one PROMPT-REUSE line for each reuse and no SPEND line beyond the call\'s own, '
        + 'so the meter of a corpus run does not over-read where a prompt is asked twice',
      fn: async () => {
        await using dir = await scratchDir({ prefix: 'prompt-reuse-meter-', },);
        resetRunSpend();
        /** Provider calls crossing the wrappers. */
        let providerCalls = 0;
        const inner: ModelClient = {
          chatText: async () => {
            providerCalls += 1;
            // A PRICED CALL IS WHAT PUTS A LINE ON THE METER, as the provider clients do.
            noteRunSpend({ provider: 'anthropic', costUsd: 0.1, },);
            return { text: 'payload', usage: { prompt_tokens: 10, completion_tokens: 20, }, };
          },
          chatJson: async () => {
            throw new Error('chatJson unused by text fixture',);
          },
          quotas: async () => {
            throw new Error('quotas unused by prompt uniqueness fixture',);
          },
        };
        const first = promptUniqueClient({ inner, store: promptPayloadStore({ dir: dir.path, },), },);
        const second = promptUniqueClient({ inner, store: promptPayloadStore({ dir: dir.path, },), },);
        const lines = await infoLinesDuring({
          run: async () => {
            await first.chatText(REQUEST,);
            await first.chatText(REQUEST,);
            await second.chatText(REQUEST,);
          },
        },);
        expect(providerCalls,).toBe(1,);
        expect(runSpendUsd({ provider: 'anthropic', },),).toBeCloseTo(0.1, 10,);
        expect(lines.logged.filter(function isReuse(line,): boolean {
          return line.includes('PROMPT-REUSE source=memory',) || line.includes('PROMPT-REUSE source=disk',);
        },).length,).toBe(2,);
        expect(lines.logged.filter(function isSpend(line,): boolean {
          return line.includes('SPEND provider=',);
        },),).toStrictEqual([],);
        // POSITIVE CONTROL: a prompt asked under another scope is a priced call, and the same meter moves for it.
        await first.chatText({ ...REQUEST, reuseScope: 'pass 2', },);
        expect(providerCalls,).toBe(2,);
        expect(runSpendUsd({ provider: 'anthropic', },),).toBeCloseTo(0.2, 10,);
      },
    },),

    it({
      name: 'NAMES THE REUSE SCOPE IN THE PROMPT DIGEST ONLY WHEN IT IS GIVEN: a request with no scope keeps the digest '
        + 'it always had, pinned here so no stored payload of a corpus run moves, and each scope is a digest of its own',
      fn: async () => {
        /** Digest of the request as a corpus run sends it. */
        const unscoped = modelPromptDigest({ request: REQUEST, },);
        expect(unscoped,).toBe('5d94598d27a56145bd58cd6eb45cee47ccc3928da84613ae69cf53b6c4ab70aa',);
        /** Digests of the same request under two scopes. */
        const passOne = modelPromptDigest({ request: { ...REQUEST, reuseScope: 'pass 1', }, },);
        const passTwo = modelPromptDigest({ request: { ...REQUEST, reuseScope: 'pass 2', }, },);
        expect(passOne,).not.toBe(unscoped,);
        expect(passTwo,).not.toBe(unscoped,);
        expect(passTwo,).not.toBe(passOne,);
      },
    },),
  ],
},);
