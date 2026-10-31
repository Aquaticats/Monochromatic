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
} from '@monochromatic-dev/test-harness-core/ts';

import { ProviderHttpError, } from '@monochromatic-dev/translation-repair-provider-stream';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import {
  alternateModelsFor,
  type ChatJsonOutcome,
  type ChatJsonRequest,
  type ChatTextReply,
  type ChatTextRequest,
  EveryProviderDryError,
  reaskElsewhereNudged,
} from '../dist/final/node/index.mjs';
import {
  infoLinesDuring,
  warnLinesDuring,
} from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';
import { quotingFailure, } from '@monochromatic-dev/translation-repair-test-fixture/ts/quoting-failure.test-fixture.ts';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_OPENROUTER_ONLY,
  SEAT_OPENROUTER_ONLY_CHECKER,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

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

/**
 Request asking the model that ran away, on NeuralWatt, in the evidence (`deepseek-v4.1-flash`).
 */
const RUNAWAY_REQUEST: ChatJsonRequest<Cat> = {
  ...REQUEST,
  modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
};

/**
 Reply of a reasoning runaway: the length stop with no content, as the 2026-10-22 runs logged them.
 */
const RUNAWAY_REPLY: ChatTextReply = { text: '', finishReason: 'length', servedBy: 'neuralwatt', };

/**
 What reading that reply produced.
 */
const RUNAWAY: ChatJsonOutcome<Cat> = {
  kind: 'schema-mismatch',
  rawText: '',
  reason: 'truncated-completion',
  detail: 'provider reported a truncating completion (model stopped with finish_reason=length)',
};

/**
 The nudge a re-ask adds, in its words.
 */
const NUDGE_CONTENT = 'Your previous reply could not be used: it was not the JSON object the question asks for. '
  + 'Answer the same question again, replying with ONLY that JSON object, nothing before or after it.';

/**
 The models a reasoning runaway of `deepseek-v4.1-flash` is re-asked of when its request names no seat, in the order
 they are tried.

 @returns Models in the census order, without the ones a roster-wide seat holds out

 @example
 ```ts
 const tried = triedForDeepseek();
 ```
 */
function triedForDeepseek(): readonly string[] {
  return alternateModelsFor({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, },).alternates;
}

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

    it({
      name: 'RE-ASKS A REASONING RUNAWAY OF ANOTHER MODEL WITH THE PLAIN QUESTION, not of the same one on another route: '
        + 'the first reply ended in a length stop with no content, so the ask names the model with the lowest share of '
        + 'runaways, carries no provider hint and no nudge (the owner\'s answer of 2026-10-30, "Plain question")',
      fn: async () => {
        /**
         Calls the re-ask received.
         */
        const asks: ChatTextRequest[] = [];
        const outcome = await reaskElsewhereNudged({
          request: RUNAWAY_REQUEST,
          first: { reply: RUNAWAY_REPLY, outcome: RUNAWAY, },
          ask: async (nudged,) => {
            asks.push(nudged,);
            return { text: '{"spot":"radiator"}', servedBy: 'synthetic', };
          },
        },);
        expect(outcome,).toEqual({ kind: 'ok', value: { spot: 'radiator', }, rawText: '{"spot":"radiator"}', },);
        expect(asks,).toEqual([
          {
            ...RUNAWAY_REQUEST,
            modelId: SEAT_HYPER_TEXT_BEDROCK,
            messages: [{ role: 'user', content: 'the sheet', },],
          },
        ],);
        expect(asks[0]?.otherThan,).toBeUndefined();
      },
    },),

    it({
      name: 'CHOOSES THE MODEL WITH THE LOWEST SHARE OF RUNAWAYS THAT IS NOT THE ONE THAT RAN AWAY, skipping a '
        + 'decision-only seat and the Claude route, and the first model of the order for the one that stands in it',
      fn: async () => {
        /**
         Model each runaway's re-ask was addressed to, by the model that ran away.
         */
        const chosen: (readonly [string, string])[] = [];
        for (const ranAway of [SEAT_HYPER_TEXT_BEDROCK, SEAT_HYPER_OPENROUTER_UNMEASURED, 'claude-haiku-5-5',] as const) {
          /**
           What this runaway's re-ask was asked.
           */
          const asked: ChatTextRequest[] = [];
          // oxlint-disable-next-line no-await-in-loop -- one runaway at a time, so each re-ask's calls stay apart
          await reaskElsewhereNudged({
            request: { ...REQUEST, modelId: ranAway, },
            first: { reply: RUNAWAY_REPLY, outcome: RUNAWAY, },
            ask: async (nudged,) => {
              asked.push(nudged,);
              return { text: '{"spot":"radiator"}', };
            },
          },);
          // ONE ASK EACH, so the models joined are the one model asked.
          chosen.push([ranAway, asked.map(function modelOf(request,): string {
            return request.modelId;
          },)
            .join(' ',),],);
        }
        expect(chosen,).toEqual([
          [SEAT_HYPER_TEXT_BEDROCK, SEAT_BEDROCK_ONLY_TEXT,],
          [SEAT_HYPER_OPENROUTER_UNMEASURED, SEAT_HYPER_TEXT_BEDROCK,],
          ['claude-haiku-5-5', SEAT_HYPER_TEXT_BEDROCK,],
        ],);
        expect(triedForDeepseek().includes('typesafe/jev-1.13',),).toBe(false,);
        expect(triedForDeepseek().includes('claude-haiku-5-5',),).toBe(false,);
      },
    },),

    it({
      name: 'RE-ASKS THE SAME MODEL ELSEWHERE, as before, where the first reply is not a runaway: a schema mismatch '
        + 'that holds content, a length stop that holds content, a refusal, and an unparseable reply with no content',
      fn: async () => {
        /**
         Replies that could not be used, none of them a length stop with no content.
         */
        const notRunaways: readonly (readonly [ChatTextReply, ChatJsonOutcome<Cat>])[] = [
          [{ text: 'not json at all', servedBy: 'neuralwatt', }, FIRST_UNUSABLE,],
          [
            { text: '{"spot":"rad', finishReason: 'length', servedBy: 'neuralwatt', },
            { kind: 'schema-mismatch', rawText: '{"spot":"rad', reason: 'truncated-completion', detail: 'fixture', },
          ],
          [
            { text: '不能回答。', servedBy: 'neuralwatt', },
            { kind: 'refusal-shaped', rawText: '不能回答。', marker: 'fixture', },
          ],
          [
            { text: '', finishReason: 'stop', servedBy: 'neuralwatt', },
            { kind: 'schema-mismatch', rawText: '', reason: 'unparseable-json', detail: 'fixture', },
          ],
        ];
        for (const [reply, outcome,] of notRunaways) {
          /**
           What this reply's re-ask was asked.
           */
          const asked: ChatTextRequest[] = [];
          // oxlint-disable-next-line no-await-in-loop -- one reply at a time, so each re-ask's calls stay apart
          await reaskElsewhereNudged({
            request: RUNAWAY_REQUEST,
            first: { reply, outcome, },
            ask: async (nudged,) => {
              asked.push(nudged,);
              return { text: '{"spot":"radiator"}', };
            },
          },);
          expect(asked.map(function askOf(request,) {
            return [request.modelId, request.otherThan,];
          },),).toEqual([[SEAT_HYPER_OPENROUTER_UNMEASURED, 'neuralwatt',],],);
        }
      },
    },),

    it({
      name: 'MOVES ON TO THE NEXT MODEL when a model cannot take the re-ask, no provider serving it with budget, and '
        + 'asks nothing of the runaway while another model is left',
      fn: async () => {
        /**
         Models asked, in order.
         */
        const askedOf: string[] = [];
        const outcome = await reaskElsewhereNudged({
          request: RUNAWAY_REQUEST,
          first: { reply: RUNAWAY_REPLY, outcome: RUNAWAY, },
          ask: async (nudged,) => {
            askedOf.push(nudged.modelId,);
            if (nudged.modelId === SEAT_HYPER_TEXT_BEDROCK) {
              throw new NoProviderForModelError({
                modelId: nudged.modelId,
                reason: 'every provider serving this model is out of budget',
              },);
            }
            return { text: '{"spot":"radiator"}', servedBy: 'hyper', };
          },
        },);
        expect(outcome.kind,).toBe('ok',);
        expect(askedOf,).toEqual([SEAT_HYPER_TEXT_BEDROCK, SEAT_BEDROCK_ONLY_TEXT,],);
      },
    },),

    it({
      name: 'FALLS BACK TO THE RUNAWAY ITSELF ON ANOTHER PROVIDER, saying so on a warning, when no other model can take '
        + 'the re-ask: it is the only model left, so the re-ask is what it was before, and the nudge rides on that ask '
        + 'alone while every other model got the plain question',
      fn: async () => {
        /**
         Calls the re-ask received.
         */
        const asks: ChatTextRequest[] = [];
        const { result, warned, } = await warnLinesDuring({
          run: async () =>
            reaskElsewhereNudged({
              request: RUNAWAY_REQUEST,
              first: { reply: RUNAWAY_REPLY, outcome: RUNAWAY, },
              ask: async (nudged,) => {
                asks.push(nudged,);
                if (nudged.modelId !== SEAT_HYPER_OPENROUTER_UNMEASURED) {
                  throw new NoProviderForModelError({
                    modelId: nudged.modelId,
                    reason: 'every provider serving this model is out of budget',
                  },);
                }
                return { text: '{"spot":"radiator"}', servedBy: 'hyper', };
              },
            },),
        },);
        expect(result.kind,).toBe('ok',);
        /**
         Models other than the runaway that were tried first.
         */
        const tried = triedForDeepseek();
        expect(asks.map(function modelOf(request,) {
          return request.modelId;
        },),).toEqual([...tried, SEAT_HYPER_OPENROUTER_UNMEASURED,],);
        expect(asks.at(-1,)?.otherThan,).toBe('neuralwatt',);
        expect(asks.at(-1,)?.messages.at(-1,)?.content,).toBe(NUDGE_CONTENT,);
        // THE NUDGE IS ON THE FALL BACK ALONE: every other model was asked the question as it stood.
        for (const plain of asks.slice(0, -1,)) {
          expect(plain.messages,).toEqual([{ role: 'user', content: 'the sheet', },],);
          expect(plain.otherThan,).toBeUndefined();
        }
        expect(warned,).toEqual([
          `[translation-repair] [reaskElsewhereNudged] [askOtherModels] ${SEAT_HYPER_OPENROUTER_UNMEASURED}: no other model `
            + `could take the re-ask of a reasoning runaway (${String(tried.length,)} tried); re-asking `
            + `${SEAT_HYPER_OPENROUTER_UNMEASURED} on another provider, as before`,
        ],);
      },
    },),

    it({
      name: 'LOGS A RE-ASK MOVED OFF THE RUNAWAY with the model, the reason, the model chosen and each model left out '
        + 'with why, and the answer under the model that gave it',
      fn: async () => {
        const { logged, } = await infoLinesDuring({
          run: async () =>
            reaskElsewhereNudged({
              request: RUNAWAY_REQUEST,
              first: { reply: RUNAWAY_REPLY, outcome: RUNAWAY, },
              ask: async () => ({ text: '{"spot":"radiator"}', servedBy: 'synthetic', }),
            },),
        },);
        expect(logged,).toEqual([
          `[translation-repair] [reaskElsewhereNudged] [askOtherModels] ${SEAT_HYPER_OPENROUTER_UNMEASURED}: reasoning runaway `
            + `on neuralwatt (a length stop with no content); re-asking ${SEAT_HYPER_TEXT_BEDROCK} instead, plain question; `
            + `left out: ${SEAT_SYNTHETIC_TEXT_EVERYWHERE} (held out of the roster role by owner-culled), `
            + `${SEAT_BEDROCK_ONLY_VISION_UNSEATED} (held out of the roster role by judge-unmeasured), `
            + `${SEAT_OPENROUTER_ONLY_CHECKER} (held out of the roster role by judge-unmeasured)`,
          `[translation-repair] [reaskElsewhereNudged] ${SEAT_HYPER_TEXT_BEDROCK}: synthetic answered usably on the re-ask`,
        ],);
      },
    },),

    it({
      name: 'LOGS THE SAME-MODEL RE-ASK UNDER THE FUNCTION\'S OWN TAG, the line the run logs and their readers search '
        + 'for (`re-asking another provider, nudged`), unchanged by the runaway rule',
      fn: async () => {
        const { logged, } = await infoLinesDuring({
          run: async () =>
            reaskElsewhereNudged({
              request: REQUEST,
              first: { reply: FIRST_REPLY, outcome: FIRST_UNUSABLE, },
              ask: async () => ({ text: '{"spot":"radiator"}', servedBy: 'hyper', }),
            },),
        },);
        expect(logged,).toEqual([
          `[translation-repair] [reaskElsewhereNudged] ${SEAT_SYNTHETIC_TEXT_EVERYWHERE}: schema-mismatch on synthetic; `
            + 're-asking another provider, nudged',
          `[translation-repair] [reaskElsewhereNudged] ${SEAT_SYNTHETIC_TEXT_EVERYWHERE}: hyper answered usably on the re-ask`,
        ],);
      },
    },),

    it({
      name: 'NAMES THE MODEL ASKED when the re-ask of another model failed too, in the cause the first answer carries, '
        + 'so a lost voice does not read as the runaway answering twice',
      fn: async () => {
        const bothFail = await reaskElsewhereNudged({
          request: RUNAWAY_REQUEST,
          first: { reply: RUNAWAY_REPLY, outcome: RUNAWAY, },
          ask: async () => ({ text: '{"spot":3}', servedBy: 'hyper', }),
        },);
        expect(bothFail,).toEqual({
          ...RUNAWAY,
          reaskFailure: {
            servedBy: 'hyper',
            kind: 'schema-mismatch',
            rawText: '{"spot":3}',
            cause: `content parsed as JSON but failed the caller schema guard; asked of ${SEAT_HYPER_TEXT_BEDROCK}`,
            askedModelId: SEAT_HYPER_TEXT_BEDROCK,
          },
        },);
      },
    },),

    it({
      name: 'SKIPS A MODEL THE ROLE HOLDS OUT: for a wide judge the models the card drops from the wide seats are never '
        + 'asked, though every model before them had no provider, and the fall back to the runaway follows',
      fn: async () => {
        /**
         Models asked, in order.
         */
        const askedOf: string[] = [];
        await reaskElsewhereNudged({
          request: { ...RUNAWAY_REQUEST, reaskSeat: { role: 'wide-judge', bench: [], }, },
          first: { reply: RUNAWAY_REPLY, outcome: RUNAWAY, },
          ask: async (nudged,) => {
            askedOf.push(nudged.modelId,);
            if (nudged.modelId !== SEAT_HYPER_OPENROUTER_UNMEASURED) {
              throw new NoProviderForModelError({ modelId: nudged.modelId, reason: 'fixture', },);
            }
            return { text: '{"spot":"radiator"}', servedBy: 'hyper', };
          },
        },);
        expect(askedOf.includes(SEAT_HYPER_OPENROUTER_VISION_EDITOR,),).toBe(false,);
        expect(askedOf.includes(SEAT_SYNTHETIC_VISION_NO_OPENROUTER,),).toBe(false,);
        expect(askedOf.at(-1,),).toBe(SEAT_HYPER_OPENROUTER_UNMEASURED,);
        expect(askedOf.length,).toBe(
          alternateModelsFor({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, role: 'wide-judge', },).alternates.length + 1,
        );
      },
    },),

    it({
      name: 'SKIPS A MODEL ON THE BENCH OF THE QUESTION and names it in the log with why, so the answer recorded as the '
        + 'runaway\'s voice is never a second voice of a model already seated',
      fn: async () => {
        /**
         Models asked, in order.
         */
        const askedOf: string[] = [];
        const { logged, } = await infoLinesDuring({
          run: async () =>
            reaskElsewhereNudged({
              request: {
                ...RUNAWAY_REQUEST,
                reaskSeat: { role: 'roster', bench: [SEAT_HYPER_TEXT_BEDROCK, SEAT_BEDROCK_ONLY_TEXT,], },
              },
              first: { reply: RUNAWAY_REPLY, outcome: RUNAWAY, },
              ask: async (nudged,) => {
                askedOf.push(nudged.modelId,);
                return { text: '{"spot":"radiator"}', servedBy: 'hyper', };
              },
            },),
        },);
        expect(askedOf,).toEqual([SEAT_OPENROUTER_ONLY,],);
        expect(logged[0],).toContain(`${SEAT_HYPER_TEXT_BEDROCK} (seated on the bench of this question)`,);
        expect(logged[0],).toContain(`${SEAT_BEDROCK_ONLY_TEXT} (seated on the bench of this question)`,);
        expect(logged[0],).toContain(`re-asking ${SEAT_OPENROUTER_ONLY} instead, plain question`,);
      },
    },),

    it({
      name: 'FALLS BACK TO THE RUNAWAY WITH THE WARNING when the bench holds every model that could take the re-ask: '
        + 'no other model is asked, and the one ask is the runaway\'s own, nudged, on another provider',
      fn: async () => {
        /**
         Calls the re-ask received.
         */
        const asks: ChatTextRequest[] = [];
        const { result, warned, } = await warnLinesDuring({
          run: async () =>
            reaskElsewhereNudged({
              request: {
                ...RUNAWAY_REQUEST,
                reaskSeat: { role: 'roster', bench: triedForDeepseek(), },
              },
              first: { reply: RUNAWAY_REPLY, outcome: RUNAWAY, },
              ask: async (nudged,) => {
                asks.push(nudged,);
                return { text: '{"spot":"radiator"}', servedBy: 'hyper', };
              },
            },),
        },);
        expect(result.kind,).toBe('ok',);
        expect(asks.map(function askOf(request,) {
          return [request.modelId, request.otherThan,];
        },),).toEqual([[SEAT_HYPER_OPENROUTER_UNMEASURED, 'neuralwatt',],],);
        expect(asks[0]?.messages.at(-1,)?.content,).toBe(NUDGE_CONTENT,);
        expect(warned,).toEqual([
          `[translation-repair] [reaskElsewhereNudged] [askOtherModels] ${SEAT_HYPER_OPENROUTER_UNMEASURED}: no other model `
            + 'could take the re-ask of a reasoning runaway (0 tried); re-asking '
            + `${SEAT_HYPER_OPENROUTER_UNMEASURED} on another provider, as before`,
        ],);
      },
    },),

    it({
      name: 'THROWS THE DRY REFUSAL a re-ask of another model meets, since every provider reading dry ends the run, '
        + 'where an exchange failure is kept as a warning and the first answer stands',
      fn: async () => {
        await expect(reaskElsewhereNudged({
          request: RUNAWAY_REQUEST,
          first: { reply: RUNAWAY_REPLY, outcome: RUNAWAY, },
          ask: async () => {
            throw new EveryProviderDryError({ measured: 'fixture', },);
          },
        },),).rejects.toThrow(EveryProviderDryError,);
      },
    },),
  ],
},);

//endregion Nudged reask tests
