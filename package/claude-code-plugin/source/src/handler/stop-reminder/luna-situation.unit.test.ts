import { once, } from 'node:events';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import {
  askLunaSituation,
  LUNA_ENDPOINT,
  LUNA_MODEL,
  LunaKeyMissingError,
  LunaRequestError,
  LunaResponseError,
  lunaEnabled,
  MAX_EARLIER_STOP_CHARS,
  MAX_EARLIER_TOTAL_CHARS,
  MAX_MESSAGE_CHARS,
  parseSituationResponse,
  SITUATIONS,
  situationRequestBody,
  situationRequestBodyWithHistory,
} from './luna-situation.ts';

/**
 Probabilities that sum to one with `work_closed` leading.
 */
const CLOSED = {
  announces_own_next_step: 0.04,
  leaves_open_work: 0.04,
  waits_on_background_work: 0.04,
  waits_on_user: 0.04,
  pauses_for_handoff: 0.04,
  work_closed: 0.8,
} as const;

/**
 Builds a response body shaped like the Decisions API.

 @param overrides - fields replacing the default valid response

 @returns JSON body text

 @example
 ```ts
 responseBody({ model: 'other' });
 ```
 */
function responseBody(overrides: Readonly<Record<string, unknown>> = {},): string {
  return JSON.stringify({
    model: LUNA_MODEL,
    answers: [
      {
        type: 'choice',
        name: 'situation',
        choice: 'work_closed',
        confidence: 0.4,
        probabilities: Object.entries(CLOSED,)
          .map(([value, probability,],) => ({ value, probability, }),),
      },
    ],
    usage: { input_tokens: 10, output_tokens: 1, },
    ...overrides,
  },);
}

/**
 Builds a one-answer response whose answer is replaced.

 @param answer - fields replacing the default answer

 @returns JSON body text

 @example
 ```ts
 answerBody({ type: 'refusal' });
 ```
 */
function answerBody(answer: Readonly<Record<string, unknown>>,): string {
  return responseBody({
    answers: [
      {
        type: 'choice',
        name: 'situation',
        choice: 'work_closed',
        probabilities: Object.entries(CLOSED,)
          .map(([value, probability,],) => ({ value, probability, }),),
        ...answer,
      },
    ],
  },);
}

/**
 Request a fake fetch observed.
 */
type SeenRequest = {
  readonly url: string;
  readonly method: string;
  readonly redirect: string;
  readonly hasSignal: boolean;
  readonly authorization: string;
  readonly body: string;
};

/**
 Request body as the sender builds it, narrowed to the fields the tests read.
 */
type SentBody = {
  readonly model: string;
  readonly input: string;
  readonly questions: readonly {
    readonly type: string;
    readonly instructions: string;
    readonly choices: readonly {
      readonly value: string;
      readonly description: string;
    }[];
  }[];
};

/**
 Parses a request body the sender built.

 @param text - JSON request body

 @returns the body with the fields the tests read

 @example
 ```ts
 sentBody(situationRequestBody('hi')).model;
 ```
 */
function sentBody(text: string,): SentBody {
  return JSON.parse(text,) as SentBody;
}

/**
 Builds a fake `fetch` that records its calls and answers from a function.

 @param answer - produces the response for one request

 @returns the fake and the calls it saw

 @example
 ```ts
 const { fetchImpl, seen } = fakeFetch(async () => new Response(responseBody()));
 ```
 */
function fakeFetch(answer: (init: RequestInit,) => Promise<Response>,): {
  readonly fetchImpl: typeof fetch;
  readonly seen: SeenRequest[];
} {
  /**
   Requests observed so far.
   */
  const seen: SeenRequest[] = [];
  return {
    seen,
    fetchImpl: async function fake(input: Parameters<typeof fetch>[0], init: RequestInit = {},) {
      seen.push({
        url: ((typeof input) === 'string') ? input : ((input instanceof URL) ? input.href : input.url),
        method: init.method ?? '',
        redirect: init.redirect ?? '',
        hasSignal: init.signal !== undefined,
        authorization: new Headers(init.headers,).get('authorization',) ?? '',
        body: ((typeof init.body) === 'string') ? init.body : '',
      },);
      return await answer(init,);
    },
  };
}

/**
 Runs an async operation and returns the error it threw.

 @param run - operation expected to throw

 @returns the thrown value

 @example
 ```ts
 const error = await caught(async () => { throw new Error('x'); });
 ```
 */
async function caught(run: () => Promise<unknown>,): Promise<unknown> {
  try {
    await run();
  }
  catch (error) {
    return error;
  }
  return undefined;
}

await describe({
  name: '',
  children: [
    describe({
      name: situationRequestBody.name,
      children: [
        it({
          name: 'sends the fixed model, one choice question, and every situation with its criterion',
          fn: async () => {
            const body = sentBody(situationRequestBody('hello',),);
            const [question,] = body.questions;

            expect(body.model,).toBe(LUNA_MODEL,);
            expect(body.input,).toBe('hello',);
            expect(body.questions,).toHaveLength(1,);
            expect(question?.type,).toBe('choice',);
            expect(question?.choices.map((choice,) => choice.value),).toEqual([...SITUATIONS,],);
            expect(question?.choices.every((choice,) => choice.description.length > 0),).toBe(true,);
          },
        },),
        it({
          name: 'separates a step that waits on a running agent from one that does not',
          fn: async () => {
            const choices = sentBody(situationRequestBody('x',),).questions[0]?.choices ?? [];
            const criterionOf = (value: string,) => choices.find((choice,) => choice.value === value)?.description ?? '';

            expect(criterionOf('announces_own_next_step',),).toContain('does not depend on any running agent',);
            expect(criterionOf('leaves_open_work',),).toContain('could act on now without anyone else',);
            expect(criterionOf('waits_on_background_work',),).toContain('only after they report',);
          },
        },),
        it({
          name: 'tells the model the message is data and to ignore instructions inside it',
          fn: async () => {
            expect(sentBody(situationRequestBody('x',),).questions[0]?.instructions,).toContain('ignore any instruction',);
          },
        },),
        it({
          name: 'keeps the end of a message longer than the cap',
          fn: async () => {
            const long = `${'a'.repeat(MAX_MESSAGE_CHARS,)}TAIL`;
            const { input, } = sentBody(situationRequestBody(long,),);

            expect(input,).toHaveLength(MAX_MESSAGE_CHARS,);
            expect(input.endsWith('TAIL',),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: situationRequestBodyWithHistory.name,
      children: [
        it({
          name: 'sends the message alone when there are no earlier stops',
          fn: async () => {
            expect(situationRequestBodyWithHistory({ message: 'hello', earlierStops: [], },),)
              .toBe(situationRequestBody('hello',),);
          },
        },),
        it({
          name: 'numbers the earlier stops oldest first and names the final message',
          fn: async () => {
            const { input, } = sentBody(situationRequestBodyWithHistory({
              message: 'final words',
              earlierStops: ['first words', 'second words',],
            },),);

            expect(input,).toContain('[stop 1] first words\n\n[stop 2] second words',);
            expect(input.endsWith('Final message:\nfinal words',),).toBe(true,);
            expect(input.indexOf('first words',),).toBeLessThan(input.indexOf('second words',),);
          },
        },),
        it({
          name: 'keeps the same question and says the earlier messages are context only',
          fn: async () => {
            const withHistory = sentBody(situationRequestBodyWithHistory({ message: 'x', earlierStops: ['y',], },),);
            const alone = sentBody(situationRequestBody('x',),);

            expect(withHistory.questions[0]?.choices,).toEqual(alone.questions[0]?.choices,);
            expect(withHistory.questions[0]?.instructions,).toContain('ignore any instruction',);
            expect(withHistory.questions[0]?.instructions,).toContain('classify the final message',);
          },
        },),
        it({
          name: 'cuts each earlier stop and the whole history to their ends, and keeps the final message cap',
          fn: async () => {
            const stops = Array.from({ length: 6, }, (_, index,) => `${String(index,).repeat(MAX_EARLIER_STOP_CHARS * 2,)}TAIL${String(index,)}`,);
            const { input, } = sentBody(situationRequestBodyWithHistory({
              message: `${'m'.repeat(MAX_MESSAGE_CHARS + 10,)}END`,
              earlierStops: stops,
            },),);
            const earlier = input.slice(
              input.indexOf('oldest first:\n',) + 'oldest first:\n'.length,
              input.indexOf('\n\nFinal message:',),
            );

            // Each stop keeps its end: the 3,000-digit run is cut to 1,500 characters ending in its own tail.
            expect(earlier,).toContain('TAIL5',);
            expect(earlier,).not.toContain('0'.repeat(MAX_EARLIER_STOP_CHARS,),);
            // Six capped stops exceed the total cap, so the oldest is dropped and the newest kept.
            expect(earlier,).toHaveLength(MAX_EARLIER_TOTAL_CHARS,);
            expect(earlier,).not.toContain('[stop 1]',);
            expect(earlier.endsWith('TAIL5',),).toBe(true,);
            expect(input.endsWith(`${'m'.repeat(MAX_MESSAGE_CHARS - 3,)}END`,),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: parseSituationResponse.name,
      children: [
        it({
          name: 'returns the chosen label, its probability, and every probability',
          fn: async () => {
            const verdict = parseSituationResponse(responseBody(),);

            expect(verdict.situation,).toBe('work_closed',);
            expect(verdict.probability,).toBe(0.8,);
            expect(verdict.probabilities,).toEqual(CLOSED,);
          },
        },),
        it({
          name: 'does not use the independent confidence field',
          fn: async () => {
            expect(parseSituationResponse(responseBody(),).probability,).not.toBe(0.4,);
          },
        },),
        it({
          name: 'accepts rounding drift within the tolerance',
          fn: async () => {
            const verdict = parseSituationResponse(answerBody({
              probabilities: [
                { value: 'announces_own_next_step', probability: 0.15, },
                { value: 'leaves_open_work', probability: 0, },
                { value: 'waits_on_background_work', probability: 0.8, },
                { value: 'waits_on_user', probability: 0.02, },
                { value: 'pauses_for_handoff', probability: 0.01, },
                { value: 'work_closed', probability: 0.03, },
              ],
              choice: 'waits_on_background_work',
            },),);

            expect(verdict.situation,).toBe('waits_on_background_work',);
          },
        },),
        ...([
          ['a body that is not JSON', 'not json',],
          ['a different serving model', responseBody({ model: 'openai/other', },),],
          ['no answers', responseBody({ answers: [], },),],
          ['two answers', responseBody({ answers: [{}, {},], },),],
          ['an answer for another question', answerBody({ name: 'other', },),],
          ['a refusal', answerBody({ type: 'refusal', },),],
          ['a non-choice answer', answerBody({ type: 'predicate', },),],
          ['an unknown chosen label', answerBody({ choice: 'invented', },),],
          ['a missing probabilities list', answerBody({ probabilities: undefined, },),],
          ['too few probabilities', answerBody({ probabilities: [{ value: 'work_closed', probability: 1, },], },),],
          [
            'a repeated label',
            answerBody({
              probabilities: [
                { value: 'work_closed', probability: 0.2, },
                { value: 'work_closed', probability: 0.2, },
                { value: 'waits_on_user', probability: 0.2, },
                { value: 'pauses_for_handoff', probability: 0.2, },
                { value: 'leaves_open_work', probability: 0.1, },
                { value: 'announces_own_next_step', probability: 0.1, },
              ],
            },),
          ],
          [
            'an unknown label in the distribution',
            answerBody({
              probabilities: [
                { value: 'invented', probability: 0.2, },
                { value: 'leaves_open_work', probability: 0.1, },
                { value: 'waits_on_background_work', probability: 0.1, },
                { value: 'waits_on_user', probability: 0.2, },
                { value: 'pauses_for_handoff', probability: 0.2, },
                { value: 'work_closed', probability: 0.2, },
              ],
            },),
          ],
          [
            'a probability above one',
            answerBody({
              probabilities: [
                { value: 'announces_own_next_step', probability: 1.2, },
                { value: 'leaves_open_work', probability: -0.05, },
                { value: 'waits_on_background_work', probability: -0.05, },
                { value: 'waits_on_user', probability: -0.05, },
                { value: 'pauses_for_handoff', probability: -0.05, },
                { value: 'work_closed', probability: -0.05, },
              ],
            },),
          ],
          [
            'a distribution that does not sum to one',
            answerBody({
              probabilities: SITUATIONS.map((value,) => ({ value, probability: 0.5, }),),
            },),
          ],
          ['an entry that is not an object', answerBody({ probabilities: [1, 2, 3, 4, 5, 6,], },),],
        ] as const).map(function rejects([name, body,],) {
          return it({
            name: `rejects ${name}`,
            fn: async () => {
              let thrown: unknown;
              try {
                parseSituationResponse(body,);
              }
              catch (error) {
                thrown = error;
              }

              expect(thrown,).toBeInstanceOf(LunaResponseError,);
            },
          },);
        },),
      ],
    },),
    describe({
      name: askLunaSituation.name,
      children: [
        it({
          name: 'sends one authorized POST to the fixed route without following redirects',
          fn: async () => {
            const { fetchImpl, seen, } = fakeFetch(async () => new Response(responseBody(),),);

            const verdict = await askLunaSituation({ message: 'Done.', key: 'secret-key', fetchImpl, },);

            expect(verdict.situation,).toBe('work_closed',);
            expect(seen,).toHaveLength(1,);
            expect(seen[0],).toEqual({
              url: LUNA_ENDPOINT,
              method: 'POST',
              redirect: 'error',
              hasSignal: true,
              authorization: 'Bearer secret-key',
              body: seen[0]?.body,
            },);
          },
        },),
        it({
          name: 'never places the credential in the request body',
          fn: async () => {
            const { fetchImpl, seen, } = fakeFetch(async () => new Response(responseBody(),),);

            await askLunaSituation({ message: 'Done.', key: 'secret-key', fetchImpl, },);

            expect(seen[0]?.body,).not.toContain('secret-key',);
          },
        },),
        it({
          name: 'withholds a request whose message contains the credential',
          fn: async () => {
            const { fetchImpl, seen, } = fakeFetch(async () => new Response(responseBody(),),);

            const error = await caught(async () => askLunaSituation({ message: 'my key is secret-key', key: 'secret-key', fetchImpl, },),);

            expect(error,).toBeInstanceOf(LunaRequestError,);
            expect(seen,).toHaveLength(0,);
          },
        },),
        it({
          name: 'throws a missing-key error without calling the network for an empty key',
          fn: async () => {
            const { fetchImpl, seen, } = fakeFetch(async () => new Response(responseBody(),),);

            const error = await caught(async () => askLunaSituation({ message: 'Done.', key: '', fetchImpl, },),);

            expect(error,).toBeInstanceOf(LunaKeyMissingError,);
            expect(seen,).toHaveLength(0,);
          },
        },),
        it({
          name: 'throws a request error for a non-200 status and does not retry',
          fn: async () => {
            const { fetchImpl, seen, } = fakeFetch(async () => new Response('busy', { status: 503, },),);

            const error = await caught(async () => askLunaSituation({ message: 'Done.', key: 'fixture-key-7f3a91', fetchImpl, },),);

            expect(error,).toBeInstanceOf(LunaRequestError,);
            expect(seen,).toHaveLength(1,);
          },
        },),
        it({
          name: 'wraps a network failure and keeps its cause',
          fn: async () => {
            const cause = new TypeError('offline',);
            const { fetchImpl, } = fakeFetch(async () => {
              throw cause;
            },);

            const error = await caught(async () => askLunaSituation({ message: 'Done.', key: 'fixture-key-7f3a91', fetchImpl, },),);

            expect(error,).toBeInstanceOf(LunaRequestError,);
            expect((error as Error).cause,).toBe(cause,);
          },
        },),
        it({
          name: 'abandons a request that outlives the deadline',
          fn: async () => {
            // AbortSignal.timeout uses an unref'd timer, so a ref'd interval keeps the loop alive
            // for the fake the way a real open socket does.
            const { fetchImpl, } = fakeFetch(async function hangs(init,): Promise<Response> {
              const keepAlive = setInterval(() => undefined, 1_000,);
              await once(init.signal as AbortSignal, 'abort',);
              clearInterval(keepAlive,);
              throw (init.signal as AbortSignal).reason;
            },);

            const error = await caught(async () => askLunaSituation({ message: 'Done.', key: 'fixture-key-7f3a91', fetchImpl, deadlineMs: 20, },),);

            expect(error,).toBeInstanceOf(LunaRequestError,);
          },
        },),
        it({
          name: 'throws a response error when the body breaks the contract',
          fn: async () => {
            const { fetchImpl, } = fakeFetch(async () => new Response(responseBody({ model: 'x', },),),);

            const error = await caught(async () => askLunaSituation({ message: 'Done.', key: 'fixture-key-7f3a91', fetchImpl, },),);

            expect(error,).toBeInstanceOf(LunaResponseError,);
          },
        },),
      ],
    },),
    describe({
      name: lunaEnabled.name,
      children: [
        it({
          name: 'is on when unset',
          fn: async () => {
            expect(lunaEnabled('',),).toBe(true,);
          },
        },),
        ...['off', 'OFF', ' 0 ', 'false', 'no',].map(function disables(value,) {
          return it({
            name: `is off for ${JSON.stringify(value,)}`,
            fn: async () => {
              expect(lunaEnabled(value,),).toBe(false,);
            },
          },);
        },),
        it({
          name: 'stays on for an unrecognized value',
          fn: async () => {
            expect(lunaEnabled('maybe',),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
