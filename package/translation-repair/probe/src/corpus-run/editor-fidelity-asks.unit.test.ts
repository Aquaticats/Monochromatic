/**
 Tests for how the editor fidelity probe records how each ask of a trial ended: the end an outcome reads as, the wrapper
 that notes the end of every exchange a trial makes without changing what the exchange returns or throws, and the word a
 row line prints for them.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  ChatJsonOutcome,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  askEndOf,
  recordingAsks,
  stoppedWord,
} from '../../dist/final/node/index.mjs';

/**
 What the stand-in client answers, by exchange.
 */
type Answers = {
  /**
   Outcomes the JSON exchange gives, in order; a rejected promise is a throw.
   */
  readonly json: readonly (ChatJsonOutcome<unknown> | Error)[];

  /**
   Texts the text exchange gives, in order; an error is a throw.
   */
  readonly text?: readonly (string | Error)[];
};

/**
 A client that answers from a script.

 @param answers - what each exchange gives, in order

 @returns Client

 @example
 ```ts
 const client = clientAnswering({ json: [{ kind: 'ok', value: {}, rawText: '{}', },], },);
 ```
 */
function clientAnswering({ answers, }: { readonly answers: Answers; }): ModelClient {
  /**
   Position in each script.
   */
  const next = {
    json: 0,
    text: 0,
  };
  return {
    chatJson: async () => {
      /**
       The scripted answer.
       */
      const answer = answers.json[next.json];
      next.json += 1;
      if (Error.isError(answer,))
        throw answer;
      return answer as ChatJsonOutcome<never>;
    },
    chatText: async () => {
      /**
       The scripted answer.
       */
      const answer = answers.text?.[next.text];
      next.text += 1;
      if (Error.isError(answer,))
        throw answer;
      return { text: answer ?? '', };
    },
    quotas: async () => {
      throw new Error('quotas unused',);
    },
    extra: 'kept',
  } as unknown as ModelClient;
}

/**
 A JSON request of no importance.
 */
const REQUEST = {
  modelId: 'claude-sonnet-5-5-effort-low',
  messages: [],
  signal: new AbortController().signal,
  validate: (value: unknown,): value is unknown => value !== undefined,
} as never;

await describe({
  name: 'editor fidelity asks',
  children: [
    describe({
      name: askEndOf.name,
      children: [
        it({
          name: 'READS EACH OUTCOME AS THE END IT IS: a parsed answer ok, a refusal refused, a cut-off completion or '
            + 'thinking the output cap, unparseable text unparseable and a guard\'s rejection rejected',
          fn: async () => {
            /**
             Outcomes and the end each reads as.
             */
            const cases: readonly (readonly [ChatJsonOutcome<unknown>, string])[] = [
              [{ kind: 'ok', value: {}, rawText: '{}', }, 'ok',],
              [{ kind: 'refusal-shaped', rawText: 'no', marker: 'api-refusal-field', }, 'refused',],
              [{ kind: 'schema-mismatch', rawText: '{', reason: 'truncated-completion', detail: 'cut', }, 'output-cap',],
              [{ kind: 'schema-mismatch', rawText: '', reason: 'truncated-thinking', detail: 'cut', }, 'output-cap',],
              [{ kind: 'schema-mismatch', rawText: 'purr', reason: 'unparseable-json', detail: 'x', }, 'unparseable',],
              [{ kind: 'schema-mismatch', rawText: '{}', reason: 'caller-guard-rejected', detail: 'x', }, 'rejected',],
              [{ kind: 'schema-mismatch', rawText: '{}', detail: 'x', }, 'rejected',],
            ];
            expect(cases.map(function endOf([outcome,],): string {
              return askEndOf({ outcome, },);
            },),).toStrictEqual(cases.map(function wanted([, end,],): string {
              return end;
            },),);
          },
        },),
      ],
    },),

    describe({
      name: stoppedWord.name,
      children: [
        it({
          name: 'JOINS THE ENDS OF A TRIAL WITH A PLUS IN THE ORDER ASKED, and says none for a trial that asked nothing',
          fn: async () => {
            expect(stoppedWord({ ends: ['ok',], },),).toBe('ok',);
            expect(stoppedWord({ ends: ['refused', 'ok',], },),).toBe('refused+ok',);
            expect(stoppedWord({ ends: [], },),).toBe('none',);
          },
        },),
      ],
    },),

    describe({
      name: recordingAsks.name,
      children: [
        it({
          name: 'NOTES THE END OF EVERY EXCHANGE, returns the outcome untouched and empties its notes when they are taken',
          fn: async () => {
            /**
             Outcomes the stand-in gives.
             */
            const given: readonly ChatJsonOutcome<unknown>[] = [
              { kind: 'refusal-shaped', rawText: 'no', marker: 'api-refusal-field', },
              { kind: 'ok', value: { edits: [], }, rawText: '{"edits":[]}', },
            ];
            const { client, takeEnds, } = recordingAsks({
              client: clientAnswering({ answers: { json: given, text: ['meow',], }, },),
            },);
            expect(await client.chatJson(REQUEST,),).toBe(given[0],);
            expect(await client.chatJson(REQUEST,),).toBe(given[1],);
            expect((await client.chatText(REQUEST,)).text,).toBe('meow',);
            expect(takeEnds(),).toStrictEqual(['refused', 'ok', 'ok',],);
            expect(takeEnds(),).toStrictEqual([],);
          },
        },),

        it({
          name: 'NOTES A THROW AS THREW AND RETHROWS THE SAME ERROR, for the JSON exchange and the text one',
          fn: async () => {
            /**
             What the stand-in throws.
             */
            const failure = new Error('the stream broke',);
            const { client, takeEnds, } = recordingAsks({
              client: clientAnswering({ answers: { json: [failure,], text: [failure,], }, },),
            },);
            await expect(client.chatJson(REQUEST,),).rejects.toBe(failure,);
            await expect(client.chatText(REQUEST,),).rejects.toBe(failure,);
            expect(takeEnds(),).toStrictEqual(['threw', 'threw',],);
          },
        },),

        it({
          name: 'KEEPS THE REST OF THE CLIENT it wraps, so the seat tally and the meters the run client carries still answer',
          fn: async () => {
            const { client, } = recordingAsks({ client: clientAnswering({ answers: { json: [], }, },), },);
            expect((client as ModelClient & { readonly extra: string; }).extra,).toBe('kept',);
          },
        },),
      ],
    },),
  ],
},);
