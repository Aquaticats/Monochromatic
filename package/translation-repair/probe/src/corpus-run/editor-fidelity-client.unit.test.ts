/**
 Tests for the client the editor fidelity probe asks through: the environment it is built over, which holds the Claude
 key and no other provider's, and the guard that refuses any model the probe was not started for.

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
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  CLAUDE_KEY_VARIABLE,
} from '@monochromatic-dev/translation-repair-run';
import {
  cardsClientFrom,
  cardsOnlyClient,
  editorProbeEnv,
} from '../../dist/final/node/index.mjs';
import { statedRefusalMessage, } from '@monochromatic-dev/translation-repair-glossary/ts/stated-refusal-message.test-fixture.ts';
import {
  claudeStreamOf,
  PLAIN_USAGE,
} from '@monochromatic-dev/translation-repair-provider/ts/claude-client.test-fixture.ts';
import { scripted, } from '@monochromatic-dev/translation-repair-provider/ts/direct-client.test-fixture.ts';
import { withFreshRunSeats, } from '@monochromatic-dev/translation-repair-stage/ts/corpus-run/fresh-run-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  HAIKU_CARD,
  HIGH_CARD,
  LOW_CARD,
} from '../editor-fidelity.test-fixture.ts';

/**
 Every key variable a launch under mise might carry, with an invented value.
 */
const FULL_ENV: Readonly<NodeJS.ProcessEnv> = {
  PATH: '/usr/bin',
  HOME: '/home/cat',
  TRANSLATION_REPAIR_NEURALWATT_API_KEY: 'purr-1',
  TRANSLATION_REPAIR_SYNTHETIC_API_KEY: 'purr-2',
  TRANSLATION_REPAIR_CHARM_HYPER_API_KEY: 'purr-3',
  TRANSLATION_REPAIR_OPENROUTER_API_KEY: 'purr-4',
  DEEPSEEK_API_KEY: 'purr-5',
  TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY: 'purr-6',
  [CLAUDE_KEY_VARIABLE]: 'purr-claude',
};

/**
 Client recording the model of every exchange and answering each with an empty edit list.

 @param asked - array each exchange's model id is appended to

 @returns Inner client the guard wraps

 @example
 ```ts
 const inner = recordingClient({ asked: [], },);
 ```
 */
function recordingClient({ asked, }: { readonly asked: string[]; },): ModelClient {
  return {
    chatText: async (request,) => {
      asked.push(request.modelId,);
      return {
        kind: 'ok',
        text: 'meow',
        rawText: 'meow',
      } as unknown as Awaited<ReturnType<ModelClient['chatText']>>;
    },
    chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
      asked.push(request.modelId,);
      return {
        kind: 'ok',
        value: { edits: [], } as ValueT,
        rawText: '{"edits":[]}',
      };
    },
    quotas: async () => {
      throw new Error('quotas unused',);
    },
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: editorProbeEnv.name,
      children: [
        it({
          name: 'KEEPS THE CLAUDE KEY AND NOTHING ELSE of an environment that holds every provider\'s, so a client built '
            + 'over it has one route and cannot be routed anywhere a named card is not served',
          fn: async () => {
            expect(editorProbeEnv({
              env: FULL_ENV,
              cardIds: [LOW_CARD, HIGH_CARD,],
            },),).toStrictEqual({ [CLAUDE_KEY_VARIABLE]: 'purr-claude', },);
          },
        },),

        it({
          name: 'REFUSES NAMING ONLY THE CLAUDE ROUTE when its key is unset or empty, whatever other keys are set, and '
            + 'asks for nothing else',
          fn: async () => {
        /**
         The refusal both environments draw.
         */
            const expected = `${CLAUDE_KEY_VARIABLE} is not set, so the Claude route, the only one this probe asks `
              + `(for ${LOW_CARD}), is unconfigured; run under mise so sops injects it`;
            expect(statedRefusalMessage({
              read: function readsUnset(): void {
                editorProbeEnv({
                  env: {
                    ...FULL_ENV,
                    [CLAUDE_KEY_VARIABLE]: undefined,
                  },
                  cardIds: [LOW_CARD,],
                },);
              },
            },),).toBe(expected,);
            expect(statedRefusalMessage({
              read: function readsEmpty(): void {
                editorProbeEnv({
                  env: {
                    [CLAUDE_KEY_VARIABLE]: '',
                  },
                  cardIds: [LOW_CARD,],
                },);
              },
            },),).toBe(expected,);
          },
        },),

        it({
          name: 'REFUSES A CARD NO CLAUDE ROUTE SERVES, since the environment it keeps would not reach it',
          fn: async () => {
            expect(statedRefusalMessage({
              read: function readsStranger(): void {
                editorProbeEnv({
                  env: FULL_ENV,
                  cardIds: ['hf:zai-org/GLM-5.3-Flash',],
                },);
              },
            },),).toBe('hf:zai-org/GLM-5.3-Flash is not served by the Claude route, the only one this probe asks',);
          },
        },),
      ],
    },),

    describe({
      name: cardsClientFrom.name,
      // SEQUENTIAL: the seat tally of a run client is process-wide.
      concurrency: 1,
      children: [
        it({
          name: 'HANDS THE CLAUDE CLIENT THE CALLER\'S ADMIT, over the Claude key alone: a call for the named card is '
            + 'told to the caller once before its request, though the environment holds every provider\'s key',
          fn: async () => {
            using _fresh = withFreshRunSeats();
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-client-', },);
            const { transport, exchanges, } = scripted({
              replies: [{
                status: 200,
                bodyText: claudeStreamOf({ text: '{"edits":[]}', stopReason: 'end_turn', usage: PLAIN_USAGE, }),
              },],
            },);
            /**
             What the caller was told, with the requests sent when it was told.
             */
            const asks: { readonly label: string; readonly sent: number; }[] = [];
            const client = cardsClientFrom({
              env: FULL_ENV,
              transport,
              promptPayloadDir: scratch.path,
              cardIds: [LOW_CARD,],
              admitClaudeAttempt: function noteAsk({ label, },): void {
                asks.push({ label, sent: exchanges.length, },);
              },
            },);
            const outcome = await client.chatJson({
              modelId: LOW_CARD,
              messages: [{ role: 'user', content: 'Where does the cat nap?', },],
              signal: new AbortController().signal,
              validate: (value: unknown,): value is unknown => value !== undefined,
            },);
            expect(outcome.kind,).toBe('ok',);
            expect(asks,).toStrictEqual([{ label: LOW_CARD, sent: 0, },],);
            expect(exchanges.length,).toBe(1,);
          },
        },),

        it({
          name: 'SENDS THE SAME PROMPT AGAIN UNDER ANOTHER REUSE SCOPE, over the real route: asks of one prompt in pass 1, '
            + 'pass 2 and pass 1 again are two requests on the wire and the scope is on none of them',
          fn: async () => {
            using _fresh = withFreshRunSeats();
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-client-', },);
            const { transport, exchanges, } = scripted({
              replies: [{
                status: 200,
                bodyText: claudeStreamOf({ text: '{"edits":[]}', stopReason: 'end_turn', usage: PLAIN_USAGE, }),
              },],
            },);
            const client = cardsClientFrom({
              env: FULL_ENV,
              transport,
              promptPayloadDir: scratch.path,
              cardIds: [LOW_CARD,],
            },);
            /**
             Asks the one prompt under a scope.
             */
            const askUnder = (reuseScope: string,): Promise<unknown> =>
              client.chatJson({
                modelId: LOW_CARD,
                messages: [{ role: 'user', content: 'Where does the cat nap?', },],
                signal: new AbortController().signal,
                validate: (value: unknown,): value is unknown => value !== undefined,
                reuseScope,
              },);
            await askUnder('pass 1',);
            await askUnder('pass 2',);
            await askUnder('pass 1',);
            expect(exchanges.length,).toBe(2,);
            /**
             Every body the transport was handed, joined.
             */
            const bodies = exchanges.map(function bodyOf({ bodyJson, },): string {
              return bodyJson ?? '';
            },)
              .join('\n',);
            expect(bodies,).not.toContain('pass 1',);
            expect(bodies,).not.toContain('pass 2',);
            expect(bodies,).not.toContain('reuseScope',);
          },
        },),
      ],
    },),

    describe({
      name: cardsOnlyClient.name,
      children: [
        it({
          name: 'PASSES an exchange for a named card and REFUSES one for any other model before it reaches the inner '
            + 'client, JSON and text alike, so a judge or a checker asked by mistake spends nothing',
          fn: async () => {
        /**
         Models the inner client was asked.
         */
            const asked: string[] = [];
            const client = cardsOnlyClient({
              inner: recordingClient({ asked, },),
              cardIds: [LOW_CARD,],
            },);
            const {signal} = new AbortController();
            await client.chatJson({
              modelId: LOW_CARD,
              messages: [],
              signal,
              validate: (value: unknown,): value is unknown => value !== undefined,
            } as unknown as ChatJsonRequest<unknown>,);
            expect(asked,).toStrictEqual([LOW_CARD,],);

        /**
         What asking another model threw, by exchange.
         */
            const caught: unknown[] = [];
            try {
              await client.chatJson({
                modelId: HAIKU_CARD,
                messages: [],
                signal,
                validate: (value: unknown,): value is unknown => value !== undefined,
              } as unknown as ChatJsonRequest<unknown>,);
            }
            catch (error) {
              caught.push(error,);
            }
            try {
              await client.chatText({
                modelId: HAIKU_CARD,
                messages: [],
                signal,
              },);
            }
            catch (error) {
              caught.push(error,);
            }
            expect(caught.length,).toBe(2,);
            expect(caught.every(function names(error,): boolean {
              return (Error.isError(error,)) && error.message.includes(HAIKU_CARD,)
                && error.message.includes(LOW_CARD,);
            },),).toBe(true,);
            expect(asked,).toStrictEqual([LOW_CARD,],);
          },
        },),

        it({
          name: 'KEEPS THE REST OF THE CLIENT it wraps, so the seat tally and the meters the run client carries still answer',
          fn: async () => {
        /**
         Inner client carrying a member the guard does not know.
         */
            const inner = {
              ...recordingClient({ asked: [], },),
              providerHolds: 'held',
            };
            const client = cardsOnlyClient({
              inner,
              cardIds: [LOW_CARD,],
            },);
            expect((client as ModelClient & { readonly providerHolds: string; }).providerHolds,).toBe('held',);
          },
        },),
      ],
    },),
  ],
},);
