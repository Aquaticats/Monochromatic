/**
 Tests for the Messages API request body.

 THE CEILING CASES PIN AN OWNER DECISION, not a preference: `max_tokens` is
 the lower of the measured answer bound and the model's own cap, and a
 caller's own ceiling may lower it further but never raise it. A body that
 asked for more than the model can emit would be answered with a truncation
 the pipeline reads as a schema mismatch, which costs a call and blames a
 model.

 THE ALTERNATION CASES PIN A PROTOCOL DIFFERENCE. The OpenAI-compatible
 provider takes the system prompt as a message and does not care what follows
 it. This one takes the system prompt as a field and requires the conversation
 to open on a user turn, so the same message array has to be rearranged rather
 than forwarded.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  answerCeilingFor,
  HYPER_MODELS,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  buildAnthropicBody,
  EmptyConversationError,
  speakingTurns,
  systemTextOf,
} from '../dist/final/node/index.mjs';
import { hyperServesLabel, } from './catalog-lookups.test-fixture.ts';

/**
 Structured-output constraint standing in for a real stage's.
 */
const catFormat = {
  type: 'json_schema',
  json_schema: {
    name: 'whisker_report',
    schema: {
      type: 'object',
      required: ['verdict',],
      properties: { verdict: { type: 'string', }, },
    },
  },
} as const;

/**
 Shortest conversation a stage sends: one instruction and one question.
 */
const catMessages = [
  {
    role: 'system',
    content: 'You count the toebeans of one cat.',
  },
  {
    role: 'user',
    content: 'How many on the front left?',
  },
] as const;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: systemTextOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LIFTS the system message out, since this protocol takes it as a field rather than '
            + 'as the first message',
          fn: async () => {
            expect(systemTextOf({ messages: catMessages, },),).toBe('You count the toebeans of one cat.',);
          },
        },),

        it({
          name: 'JOINS several system messages rather than keeping only one, because dropping any of '
            + 'them would silently change what a stage asked',
          fn: async () => {
            expect(systemTextOf({
              messages: [
                {
                  role: 'system',
                  content: 'First rule.',
                },
                {
                  role: 'user',
                  content: 'Question.',
                },
                {
                  role: 'system',
                  content: 'Second rule.',
                },
              ],
            },),).toBe('First rule.\n\nSecond rule.',);
          },
        },),

        it({
          name: 'ANSWERS with empty text where no system message was sent, rather than inventing one',
          fn: async () => {
            expect(systemTextOf({
              messages: [
                {
                  role: 'user',
                  content: 'Question.',
                },
              ],
            },),).toBe('',);
          },
        },),

        it({
          name: 'REFUSES a system message carrying pictures, which this protocol has no field for',
          fn: async () => {
            expect(() => {
              systemTextOf({
                messages: [
                  {
                    role: 'system',
                    content: [
                      {
                        type: 'text',
                        text: 'Rule.',
                      },
                    ],
                  },
                ],
              },);
            },).toThrow(EmptyConversationError,);
          },
        },),
      ],
    },),

    describe({
      name: speakingTurns.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'DROPS the system turn and keeps the rest in order, which is the whole rearrangement '
            + 'this protocol needs',
          fn: async () => {
            expect(speakingTurns({ messages: catMessages, },),).toEqual([
              {
                role: 'user',
                content: [
                  {
                    type: 'text',
                    text: 'How many on the front left?',
                  },
                ],
              },
            ],);
          },
        },),

        it({
          name: 'KEEPS a repair conversation alternating, which is the shape an invalid-candidate re-ask sends: the question, '
            + 'the answer that failed, and the correction',
          fn: async () => {
            expect(speakingTurns({
              messages: [
                {
                  role: 'system',
                  content: 'Rules.',
                },
                {
                  role: 'user',
                  content: 'Translate.',
                },
                {
                  role: 'assistant',
                  content: '{"translation":"…"}',
                },
                {
                  role: 'user',
                  content: 'Findings.',
                },
              ],
            },).map(function named(turn,): string {
              return turn.role;
            },),).toEqual(['user', 'assistant', 'user',],);
          },
        },),

        it({
          name: 'MERGES consecutive same-role turns rather than refusing them, because the other '
            + 'provider accepts them and a call routed either way must ask the same question',
          fn: async () => {
            /**
             Two user turns in a row, as only a merge can express here.
             */
            const turns = speakingTurns({
              messages: [
                {
                  role: 'user',
                  content: 'First half.',
                },
                {
                  role: 'user',
                  content: 'Second half.',
                },
              ],
            },);

            expect(turns,).toEqual([
              {
                role: 'user',
                content: [
                  {
                    type: 'text',
                    text: 'First half.',
                  },
                  {
                    type: 'text',
                    text: 'Second half.',
                  },
                ],
              },
            ],);
          },
        },),

        it({
          name: 'REFUSES a conversation that is only a system prompt, which this protocol rejects and '
            + 'which would otherwise cost a call to find out',
          fn: async () => {
            expect(() => {
              speakingTurns({
                messages: [
                  {
                    role: 'system',
                    content: 'Rules.',
                  },
                ],
              },);
            },).toThrow(EmptyConversationError,);
          },
        },),

        it({
          name: 'REFUSES a conversation opening on an assistant turn, which this protocol requires to '
            + 'be a user turn',
          fn: async () => {
            expect(() => {
              speakingTurns({
                messages: [
                  {
                    role: 'system',
                    content: 'Rules.',
                  },
                  {
                    role: 'assistant',
                    content: 'Unprompted.',
                  },
                ],
              },);
            },).toThrow(EmptyConversationError,);
          },
        },),
      ],
    },),

    describe({
      name: buildAnthropicBody.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'STREAMS ALWAYS, because the owner requires it and every runaway guard '
            + 'reads a live stream',
          fn: async () => {
            expect(buildAnthropicBody({
              modelId: 'kimi-k3',
              messages: catMessages,
            },).stream,).toBe(true,);
          },
        },),

        it({
          name: 'ASKS FOR THE PER-MODEL CEILING where the caller named none, which is the lower of '
            + "the measured answer bound and the model's own cap",
          fn: async () => {
            // The ceilings themselves are measurements pinned in
            // `hyper-catalog.unit.test.ts`; this case pins that the body asks for them.
            /**
             Ceiling asked for and ceiling owed, per catalog model.
             */
            const asked = Object.keys(HYPER_MODELS,).map(function askedAndOwed(modelId,): readonly [number, number,] {
              if (!hyperServesLabel(modelId,))
                throw new Error(`catalog key ${modelId} is not a served label`,);
              return [
                buildAnthropicBody({
                  modelId,
                  messages: catMessages,
                },).max_tokens,
                answerCeilingFor({ modelId, },),
              ];
            },);
            // Ceilings that differ across models, or one constant ask would pass too.
            expect(new Set(asked.map(function owedOf([, owed,],): number {
              return owed;
            },),).size,).toBeGreaterThan(1,);
            for (const [sent, owed,] of asked)
              expect(sent,).toBe(owed,);
          },
        },),

        it({
          name: "LOWERS the ask to the caller's ceiling, and never raises it above the model's, so a "
            + 'caller cannot ask for a length the model would truncate',
          fn: async () => {
            /**
             The model's own ceiling, which the caller's may only lower.
             */
            const ceiling = answerCeilingFor({ modelId: 'kimi-k3', },);
            /**
             A caller ceiling under the model's.
             */
            const lower = Math.floor(ceiling / 2,);

            expect(buildAnthropicBody({
              modelId: 'kimi-k3',
              messages: catMessages,
              maxTokens: lower,
            },).max_tokens,).toBe(lower,);

            expect(buildAnthropicBody({
              modelId: 'kimi-k3',
              messages: catMessages,
              maxTokens: ceiling * 2,
            },).max_tokens,).toBe(ceiling,);
          },
        },),

        it({
          name: 'CARRIES the instruction as the system field and the question as the only turn',
          fn: async () => {
            /**
             Body as it would be serialised.
             */
            const body = buildAnthropicBody({
              modelId: 'kimi-k3',
              messages: catMessages,
            },);

            expect(body.system,).toBe('You count the toebeans of one cat.',);
            expect(body.messages.length,).toBe(1,);
            expect(body.model,).toBe('kimi-k3',);
          },
        },),

        it({
          name: 'OFFERS NO TOOL where the caller stated no schema, and leaves the system field as the '
            + 'bare instruction, since there is no answer protocol to state',
          fn: async () => {
            /**
             Body as it would be serialised.
             */
            const body = buildAnthropicBody({
              modelId: 'kimi-k3',
              messages: catMessages,
            },);

            expect(body.tools,).toBe(undefined,);
            expect(body.tool_choice,).toBe(undefined,);
            expect(body.system.includes('HOW TO ANSWER',),).toBe(false,);
          },
        },),

        it({
          name: 'OFFERS the answer tool and states its whole schema in the system field, which is the '
            + "owner's instruction for models that emit the wrong call format",
          fn: async () => {
            /**
             Body as it would be serialised.
             */
            const body = buildAnthropicBody({
              modelId: 'kimi-k3',
              messages: catMessages,
              responseFormat: catFormat,
            },);

            /**
             Answer tool the body offers, of which there is exactly one.
             */
            const [tool,] = body.tools ?? [];

            expect(body.tools?.length,).toBe(1,);
            expect(tool?.input_schema,).toEqual(catFormat.json_schema.schema,);
            expect(body.system.includes('You count the toebeans of one cat.',),).toBe(true,);
            expect(body.system.includes('HOW TO ANSWER',),).toBe(true,);
            expect(body.system.includes('verdict',),).toBe(true,);
          },
        },),

        it({
          name: 'FORCES the tool by the same name it offers, so the choice cannot name a tool that '
            + 'was never defined',
          fn: async () => {
            /**
             Body as it would be serialised.
             */
            const body = buildAnthropicBody({
              modelId: 'kimi-k3',
              messages: catMessages,
              responseFormat: catFormat,
            },);

            expect(body.tool_choice,).toEqual({
              type: 'tool',
              name: 'whisker_report',
            },);
            expect(body.tools?.at(0,)?.name,).toBe('whisker_report',);
          },
        },),

        it({
          name: 'FORWARDS the conversation refusal rather than sending a body this protocol rejects',
          fn: async () => {
            expect(() => {
              buildAnthropicBody({
                modelId: 'kimi-k3',
                messages: [
                  {
                    role: 'system',
                    content: 'Rules.',
                  },
                ],
              },);
            },).toThrow(EmptyConversationError,);
          },
        },),
      ],
    },),
  ],
},);
