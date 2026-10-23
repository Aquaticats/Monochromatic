/**
 Tests for the nudged re-ask.

 THE FIRST ANSWER STANDS when the re-ask brings nothing usable: it came from
 the provider the policy preferred, and the caller's handling is written
 against it. The re-ask's log names the provider that answered, or another
 provider where the reply named none.

 FIXTURES ARE CAT-THEMED.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { ProviderHttpError, } from '@monochromatic-dev/translation-repair-provider-stream';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  type ChatTextReply,
  type ChatTextRequest,
  reaskElsewhereNudged,
} from '../dist/final/node/index.mjs';
import { warnLinesDuring, } from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';
import { quotingFailure, } from '@monochromatic-dev/translation-repair-test-fixture/ts/quoting-failure.test-fixture.ts';
import { SEAT_SYNTHETIC_TEXT_EVERYWHERE, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

//region Nudged reask tests

/**
 What the cat questions answer with.
 */
type Cat = { readonly spot: string; };

/**
 Request under test, asking one question of one seat.
 */
const REQUEST: ChatJsonRequest<Cat> = {
  modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  messages: [{ role: 'user', content: 'the sheet', },],
  signal: new AbortController().signal,
  responseFormat: {
    type: 'json_schema',
    json_schema: { name: 'cat', schema: { type: 'object', }, },
  },
  validate: function isCat(value: unknown,): value is Cat {
    return ((typeof value) === 'object') && (value !== null) && ('spot' in value) && ((typeof value.spot) === 'string');
  },
};

/**
 First reply, served by the provider the re-ask must avoid.
 */
const FIRST_REPLY: ChatTextReply = { text: 'not json at all', servedBy: 'synthetic', };

/**
 What reading the first reply produced.
 */
const FIRST_UNUSABLE: ChatJsonOutcome<Cat> = {
  kind: 'schema-mismatch',
  rawText: 'not json at all',
  detail: 'fixture',
};

await describe({
  name: 'nudged re-ask',
  concurrency: 1,
  children: [
    it({
      name: 'RE-ASKS only where the first answer failed, returning the second where it is usable and the '
        + 'FIRST where both fail, each path distinguished by the value it returns',
      fn: async () => {
        const usable = await reaskElsewhereNudged({
          request: REQUEST,
          first: { reply: FIRST_REPLY, outcome: FIRST_UNUSABLE, },
          ask: async () => ({ text: '{"spot":"radiator"}', }),
        },);
        expect(usable,).toEqual({ kind: 'ok', value: { spot: 'radiator', }, rawText: '{"spot":"radiator"}', },);

        const bothFail = await reaskElsewhereNudged({
          request: REQUEST,
          first: { reply: FIRST_REPLY, outcome: FIRST_UNUSABLE, },
          ask: async () => ({ text: '{"spot":3}', servedBy: 'hyper', }),
        },);
        // THE FIRST ANSWER IS RETURNED, carrying how the re-ask failed too, so the lost voice can name both failures.
        expect(bothFail,).toEqual({
          kind: 'schema-mismatch',
          rawText: 'not json at all',
          detail: 'fixture',
          reaskFailure: {
            servedBy: 'hyper',
            kind: 'schema-mismatch',
            rawText: '{"spot":3}',
            cause: 'content parsed as JSON but failed the caller schema guard',
          },
        },);
      },
    },),

    it({
      name: 'RECORDS A REFUSAL-SHAPED RE-ASK AS SUCH, naming the kind it failed as and the marker for its cause, not the detail '
        + 'of a schema mismatch, beside the first answer that is returned',
      fn: async () => {
        const bothFail = await reaskElsewhereNudged({
          request: REQUEST,
          first: { reply: FIRST_REPLY, outcome: FIRST_UNUSABLE, },
          ask: async () => ({ text: '', refusal: '不能回答。', servedBy: 'hyper', }),
        },);
        expect(bothFail,).toEqual({
          kind: 'schema-mismatch',
          rawText: 'not json at all',
          detail: 'fixture',
          reaskFailure: {
            servedBy: 'hyper',
            kind: 'refusal-shaped',
            rawText: '不能回答。',
            cause: 'api-refusal-field',
          },
        },);
      },
    },),

    it({
      name: 'ANSWERS WITH A USABLE FIRST ANSWER at once, asking no second provider',
      fn: async () => {
        /**
         Calls the re-ask received.
         */
        const asks: ChatTextRequest[] = [];
        const usableFirst: ChatJsonOutcome<Cat> = {
          kind: 'ok',
          value: { spot: 'radiator', },
          rawText: '{"spot":"radiator"}',
        };
        const outcome = await reaskElsewhereNudged({
          request: REQUEST,
          first: { reply: { text: '{"spot":"radiator"}', servedBy: 'synthetic', }, outcome: usableFirst, },
          ask: async (nudged,) => {
            asks.push(nudged,);
            throw new Error('a usable first answer must not buy a re-ask',);
          },
        },);
        expect(outcome,).toEqual({ kind: 'ok', value: { spot: 'radiator', }, rawText: '{"spot":"radiator"}', },);
        expect(asks,).toEqual([],);
      },
    },),

    it({
      name: 'READS THE RE-ASK AGAINST THE RESPONSE FORMAT OF THE REQUEST, taking the schema\'s spelling of a listed '
        + 'value that came back differing only in its capital',
      fn: async () => {
        const listed = await reaskElsewhereNudged({
          request: {
            ...REQUEST,
            responseFormat: {
              type: 'json_schema',
              json_schema: {
                name: 'cat',
                schema: {
                  type: 'object',
                  properties: { spot: { type: 'string', enum: ['radiator', 'windowsill',], }, },
                },
              },
            },
            validate: function isListedSpot(value: unknown,): value is Cat {
              return ((typeof value) === 'object') && (value !== null) && ('spot' in value)
                && ((value.spot === 'radiator') || (value.spot === 'windowsill'));
            },
          },
          first: { reply: FIRST_REPLY, outcome: FIRST_UNUSABLE, },
          ask: async () => ({ text: '{"spot":"Radiator"}', servedBy: 'hyper', }),
        },);
        expect(listed.kind === 'ok' ? listed.value : undefined,).toEqual({ spot: 'radiator', },);
      },
    },),

    it({
      name: 'ASKS THE SAME QUESTION AGAIN with the nudge as the last message and the first provider excluded',
      fn: async () => {
        /**
         Calls the re-ask received.
         */
        const asks: ChatTextRequest[] = [];
        await reaskElsewhereNudged({
          request: REQUEST,
          first: { reply: FIRST_REPLY, outcome: FIRST_UNUSABLE, },
          ask: async (nudged,) => {
            asks.push(nudged,);
            return { text: '{"spot":"radiator"}', };
          },
        },);
        expect(asks,).toEqual([
          {
            ...REQUEST,
            messages: [
              { role: 'user', content: 'the sheet', },
              {
                role: 'user',
                content: 'Your previous reply could not be used: it was not the JSON object the question asks for. '
                  + 'Answer the same question again, replying with ONLY that JSON object, nothing before or after it.',
              },
            ],
            otherThan: 'synthetic',
          },
        ],);
      },
    },),

    it({
      name: 'ANSWERS FROM AN UNTAGGED FIRST REPLY with no re-ask, since no provider names where else to '
        + 'look',
      fn: async () => {
        let asked = false;
        const untagged = await reaskElsewhereNudged({
          request: REQUEST,
          first: { reply: { text: 'not json at all', }, outcome: FIRST_UNUSABLE, },
          ask: async () => {
            asked = true;
            throw new Error('the untagged reply must not buy a re-ask',);
          },
        },);
        expect(asked,).toBe(false,);
        expect(untagged,).toEqual({ kind: 'schema-mismatch', rawText: 'not json at all', detail: 'fixture', },);
      },
    },),

    it({
      name: 'KEEPS THE FIRST ANSWER where the re-ask exchange fails and the caller stands, answering from the '
        + 'first',
      fn: async () => {
        const kept = await reaskElsewhereNudged({
          request: REQUEST,
          first: { reply: FIRST_REPLY, outcome: FIRST_UNUSABLE, },
          ask: async () => {
            throw new Error('the cat swatted the cable',);
          },
        },);
        expect(kept,).toEqual({ kind: 'schema-mismatch', rawText: 'not json at all', detail: 'fixture', },);
      },
    },),

    it({
      name: 'WARNS OF A FAILED RE-ASK BY THE FAILURE\'S CLASS and never by its message, which an unsendable header '
        + 'quotes',
      fn: async () => {
        const { warned, } = await warnLinesDuring({
          run: async () =>
            reaskElsewhereNudged({
              request: REQUEST,
              first: { reply: FIRST_REPLY, outcome: FIRST_UNUSABLE, },
              ask: async () => {
                throw quotingFailure();
              },
            },),
        },);
        expect(warned,).toEqual([
          `[translation-repair] [reaskElsewhereNudged] ${SEAT_SYNTHETIC_TEXT_EVERYWHERE}: no re-ask elsewhere (refused by TypeError); keeping the first answer`,
        ],);
      },
    },),

    it({
      name: 'WARNS OF A REFUSED RE-ASK BY CLASS AND STATUS, then the provider\'s own words labelled as its own',
      fn: async () => {
        const { warned, } = await warnLinesDuring({
          run: async () =>
            reaskElsewhereNudged({
              request: REQUEST,
              first: { reply: FIRST_REPLY, outcome: FIRST_UNUSABLE, },
              ask: async () => {
                throw new ProviderHttpError({ provider: 'synthetic', status: 400, bodyText: 'the cat refused this passage', },);
              },
            },),
        },);
        expect(warned,).toEqual([
          `[translation-repair] [reaskElsewhereNudged] ${SEAT_SYNTHETIC_TEXT_EVERYWHERE}: no re-ask elsewhere `
            + '(refused by ProviderHttpError from synthetic with HTTP 400 (the provider said: "the cat refused this passage")); '
            + 'keeping the first answer',
        ],);
      },
    },),

    it({
      name: 'THROWS the re-ask error where the caller aborted, so steering stops the second chance',
      fn: async () => {
        const controller = new AbortController();
        controller.abort();
        await expect(reaskElsewhereNudged({
          request: {
            ...REQUEST,
            signal: controller.signal,
          },
          first: { reply: FIRST_REPLY, outcome: FIRST_UNUSABLE, },
          ask: async () => {
            throw new Error('the cat swatted the cable',);
          },
        },),).rejects.toThrow('the cat swatted the cable',);
      },
    },),
  ],
},);

//endregion Nudged reask tests
