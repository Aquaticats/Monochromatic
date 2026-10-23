/**
 Tests for the typed half of a stage exchange (2026-09-18): a decision seat
 is asked the stage's question through the client's `decide` and never the
 sheet; a stage with no question, a client with no decisions transport, a
 router refusal and a state past the seat's context all read as a seat out
 of reach, without a chat call. Cat-themed invention.

 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { ProviderHttpError, } from '@monochromatic-dev/translation-repair-provider-stream';
import {
  EveryProviderDryError,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import { modelDarkRefusal, } from '@monochromatic-dev/translation-repair-provider/ts/model-dark-refusal.test-fixture.ts';
import {
  attemptStageCall,
  type StageDecision,
} from '../dist/final/node/index.mjs';
import { warningRecordingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { SEAT_OPENROUTER_DECISIONS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

/**
 Logger the stage writes its progress to.
 */
const l = tagged({ tag: 'stage-decision-call-test', },);

/**
 Guard admitting the read ballot.

 @param value - candidate from the reading

 @returns Whether it carries a numeric best

 @example
 ```ts
 isBallot({ best: 2, },);
 ```
 */
function isBallot(value: unknown,): value is { readonly best: number; } {
  return ((typeof value) === 'object') && (value !== null) && ((typeof (value as { best?: unknown; }).best) === 'number');
}

/**
 One typed question over two cat renderings.
 */
const DECISION: StageDecision = {
  state: { candidates: { '1': 'The cat naps.', '2': 'The cat dozes.', }, },
  questions: {
    best: {
      type: 'choice',
      instructions: 'Which is best?',
      criteria: {
        '0': 'none',
        '1': 'candidate 1',
        '2': 'candidate 2',
      },
    },
  },
  read: function read(answers,) {
    /**
     The one answer.
     */
    const { best, } = answers;
    return ((best !== undefined) && (best.type === 'choice')) ? { best: Number(best.choice,), } : {};
  },
};

/**
 Client whose chat surface must never be reached and whose decide answers
 as scripted.

 @param decide - what a typed exchange does, or nothing for a client without one

 @returns Client for the stage call

 @example
 ```ts
 const client = clientWith({ decide: async () => reply, },);
 ```
 */
function clientWith(
  { decide, }: { readonly decide?: ModelClient['decide']; },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('a decision seat must never be asked for chat',);
    },
    chatJson: async () => {
      throw new Error('a decision seat must never be asked for chat',);
    },
    quotas: async () => {
      throw new Error('quotas unused',);
    },
    ...((decide === undefined) ? {} : { decide, }),
  };
}

/**
 Arguments every case shares.
 */
const SHARED = {
  modelId: SEAT_OPENROUTER_DECISIONS,
  messages: [{ role: 'user' as const, content: 'the sheet a decision seat never sees', },],
  signal: AbortSignal.timeout(HANG_STOP_MS,),
  exchangeTimeoutMs: HANG_STOP_MS,
  responseFormat: {
    type: 'json_schema' as const,
    json_schema: { name: 'ballot', schema: { type: 'object', }, },
  },
  validate: isBallot,
  stage: 'select',
  l,
};

await describe({
  name: 'a decision seat in a stage call',
  children: [
    it({
      name: 'IS ASKED THE TYPED QUESTION and heard through the stage guard',
      fn: async () => {
        const voice = await attemptStageCall({
          ...SHARED,
          client: clientWith({
            decide: async () => ({
              model: 'typesafe/jev-1.13',
              answers: { best: { type: 'choice', choice: '2', }, },
            }),
          },),
          decision: DECISION,
        },);
        expect(voice,).toEqual({
          heard: true,
          value: { best: 2, },
        },);
      },
    },),

    it({
      name: 'READS AS OUT OF REACH without a chat call when the stage has no question or the client no '
        + 'decisions transport (ledger P13: no round can ask it, so no quorum may wait on it), and when the '
        + 'router refuses',
      fn: async () => {
        const noQuestion = await attemptStageCall({
          ...SHARED,
          client: clientWith({
            decide: async () => {
              throw new Error('must not be asked without a question',);
            },
          },),
        },);
        expect(noQuestion,).toEqual({
          heard: false,
          answered: false,
          unreachable: true,
        },);

        const noTransport = await attemptStageCall({
          ...SHARED,
          client: clientWith({},),
          decision: DECISION,
        },);
        expect(noTransport,).toEqual({
          heard: false,
          answered: false,
          unreachable: true,
        },);

        const refused = await attemptStageCall({
          ...SHARED,
          client: clientWith({
            decide: async () => {
              throw new NoProviderForModelError({
                modelId: SEAT_OPENROUTER_DECISIONS,
                reason: 'the decisions endpoint reads dry',
              },);
            },
          },),
          decision: DECISION,
        },);
        expect(refused,).toEqual({
          heard: false,
          answered: false,
          unreachable: true,
        },);
      },
    },),

    it({
      name: 'READS AS OUT OF REACH FOR THE BALLOT when the endpoint refuses a state past the seat\'s context '
        + '(ledger P13: probed 2026-09-28, HTTP 400 max_tokens_exceeded, never a truncation), and as a lost '
        + 'voice on any other 400',
      fn: async () => {
        const overContext = await attemptStageCall({
          ...SHARED,
          client: clientWith({
            decide: async () => {
              throw new ProviderHttpError({
                provider: 'openrouter',
                status: 400,
                bodyText: String.raw`{"error":{"message":"HTTP 400: {\"detail\":{\"error_type\":\"max_tokens_exceeded\"}}","code":400}}`,
              },);
            },
          },),
          decision: DECISION,
        },);
        expect(overContext,).toEqual({
          heard: false,
          answered: false,
          unreachable: true,
        },);

        const otherRefusal = await attemptStageCall({
          ...SHARED,
          client: clientWith({
            decide: async () => {
              throw new ProviderHttpError({
                provider: 'openrouter',
                status: 400,
                bodyText: String.raw`{"error":{"message":"HTTP 400: {\"detail\":{\"error_type\":\"invalid_question\"}}","code":400}}`,
              },);
            },
          },),
          decision: DECISION,
        },);
        expect(otherRefusal,).toEqual({
          heard: false,
          answered: false,
          unreachable: false,
        },);
      },
    },),

    it({
      name: 'LOGS A LOST VOICE BY CLASS AND HTTP STATUS, then the provider\'s own words labelled as its own',
      fn: async () => {
        const warnings: string[] = [];
        await attemptStageCall({
          ...SHARED,
          l: warningRecordingLogger({ base: l, warnings, },),
          client: clientWith({
            decide: async () => {
              throw new ProviderHttpError({
                provider: 'openrouter',
                status: 400,
                bodyText: `the cat named an invalid_question`,
              },);
            },
          },),
          decision: DECISION,
        },);
        expect(warnings,).toEqual([
          `select ${SEAT_OPENROUTER_DECISIONS}: refused by ProviderHttpError from openrouter with HTTP 400 (the provider said: "the cat named an invalid_question"), voice lost`,
        ],);
      },
    },),

    it({
      name: 'READS AS A LOST VOICE rather than out of reach when the refusal status is not 400 even '
        + 'though the body names the marker, since only a 400 max_tokens_exceeded names the state itself',
      fn: async () => {
        const serverError = await attemptStageCall({
          ...SHARED,
          client: clientWith({
            decide: async () => {
              throw new ProviderHttpError({
                provider: 'openrouter',
                status: 503,
                bodyText: 'upstream reported max_tokens_exceeded for its own model',
              },);
            },
          },),
          decision: DECISION,
        },);
        expect(serverError,).toEqual({
          heard: false,
          answered: false,
          unreachable: false,
        },);
      },
    },),

    it({
      name: 'LOSES THE VOICE where the stage guard refuses the reading, since a typed answer that fails '
        + 'the guard is not the value the stage asked for',
      fn: async () => {
        const voice = await attemptStageCall({
          ...SHARED,
          validate: function refuses(_value: unknown,): _value is unknown {
            return false;
          },
          client: clientWith({
            decide: async () => ({
              model: 'typesafe/jev-1.13',
              answers: { best: { type: 'choice', choice: '2', }, },
            }),
          },),
          decision: DECISION,
        },);
        expect(voice,).toEqual({
          heard: false,
          answered: true,
          unreachable: false,
          unreadable: 'off-shape',
        },);
      },
    },),

    it({
      name: 'THROWS the call error where the signal aborted, so steering stops the fan-out',
      fn: async () => {
        await expect(attemptStageCall({
          ...SHARED,
          signal: AbortSignal.abort(),
          client: clientWith({
            decide: async () => {
              throw new Error('cat nap',);
            },
          },),
          decision: DECISION,
        },),).rejects.toThrow('cat nap',);
      },
    },),

    it({
      name: 'CONTAINS A MODEL-DARK REFUSAL FROM THE DECISIONS ROUTE as an unreachable voice, since a dry decisions '
        + 'endpoint costs the decision seat its voice while other providers are wet',
      fn: async () => {
        expect(await attemptStageCall({
          ...SHARED,
          client: clientWith({
            decide: async () => {
              throw modelDarkRefusal({ modelId: SHARED.modelId, },);
            },
          },),
          decision: DECISION,
        },),).toMatchObject({ heard: false, answered: false, unreachable: true, },);
      },
    },),

    it({
      name: 'THROWS an every-provider-dry refusal although the signal did not abort, since it ends the run',
      fn: async () => {
        /**
         The refusal the router raised, held so the assertion proves THAT object propagated.
         */
        const dry = new EveryProviderDryError({ measured: 'meters read every provider dry', },);

        await expect(attemptStageCall({
          ...SHARED,
          client: clientWith({
            decide: async () => {
              throw dry;
            },
          },),
          decision: DECISION,
        },),).rejects.toBe(dry,);
      },
    },),
  ],
},);
