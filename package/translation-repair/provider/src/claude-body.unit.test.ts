/**
 Tests for the body the Claude client puts on the wire (`claude-body.ts`), read off the exchange a recorded transport
 saw. The rules come from the owner's brief of 2026-10-22 and the probes it cites: Sonnet 5.5 answers
 `output_config.format` with adaptive thinking and refuses a forced tool with HTTP 400, Haiku 5.5 under a forced tool
 returned no thinking and the wrong answer on every hard call, thinking can be neither disabled nor given a budget on
 either model, sampling knobs and an assistant prefill are refused with 400.

 NOTHING HERE REACHES A PROVIDER: every transport is an in-memory script, and every key is invented. Fixtures are
 cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  ANTHROPIC_SERVED_IDS,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { createClaudeClient, } from '../dist/final/node/index.mjs';
import {
  firstBodyOf,
  NAP_SCHEMA,
  ONE_QUICK_RETRY,
  scripted,
  SIGNAL,
} from './direct-client.test-fixture.ts';
import {
  CLAUDE_CAT_KEY,
  claudeStreamOf,
  PLAIN_USAGE,
} from './claude-client.test-fixture.ts';

/**
 Asks one question of one Claude card over a recorded transport and hands back the body it sent.

 @param modelId - roster card addressed

 @param messages - conversation to send

 @param withSchema - whether the call states the nap schema

 @param schema - a schema of the caller's own, which replaces the nap schema

 @param extra - loose members the request carries beyond its declared ones, as a caller might add

 @returns The first exchange's body, parsed

 @example
 ```ts
 const body = await bodySentFor({ modelId: 'claude-haiku-5-5', withSchema: true, },);
 ```
 */
async function bodySentFor(
  {
    modelId,
    messages = [
      { role: 'system', content: 'You find the cat.', },
      { role: 'user', content: 'Where does the cat sleep?', },
    ],
    withSchema = false,
    schema,
    extra = {},
  }: {
    readonly modelId: RosterModelId;
    readonly messages?: Parameters<ReturnType<typeof createClaudeClient>['chatText']>[0]['messages'];
    readonly withSchema?: boolean;
    readonly schema?: Readonly<Record<string, unknown>>;
    readonly extra?: Readonly<Record<string, unknown>>;
  },
): Promise<Record<string, unknown>> {
  const { transport, exchanges, } = scripted({
    replies: [{
      status: 200,
      bodyText: claudeStreamOf({ text: '{"spot":"sunbeam"}', stopReason: 'end_turn', usage: PLAIN_USAGE, }),
    },],
  },);
  await createClaudeClient({ apiKey: CLAUDE_CAT_KEY, transport, retryPolicy: ONE_QUICK_RETRY, },)
    .chatText({
      modelId,
      messages,
      signal: SIGNAL,
      ...(schema === undefined
        ? (withSchema ? { responseFormat: NAP_SCHEMA, } : {})
        : { responseFormat: { type: 'json_schema', json_schema: { name: 'cat_naps', strict: true, schema, }, }, }),
      ...extra,
    },);
  return firstBodyOf({ exchanges, },);
}

await describe({
  name: 'the Claude body',
  concurrency: 1,
  children: [
    it({
      name: 'ASKS FOR STRUCTURED OUTPUT BY output_config.format AND NEVER BY A FORCED TOOL, on all eight cards: Sonnet '
        + '5.5 refuses a forced tool with HTTP 400 and Haiku 5.5 under one answered 6 of 6 hard questions wrong with no '
        + 'thinking',
      fn: async () => {
        await Promise.all(ANTHROPIC_SERVED_IDS.map(async function checkCard(modelId,) {
          const body = await bodySentFor({ modelId, withSchema: true, },);
          expect(body.tools,).toBeUndefined();
          expect(body.tool_choice,).toBeUndefined();
          expect(body.output_config,).toMatchObject({
            format: { type: 'json_schema', schema: { ...NAP_SCHEMA.json_schema.schema, additionalProperties: false, }, },
          },);
        },),);
      },
    },),

    it({
      name: 'ASKS FOR NO FORMAT on a free-text call, and offers no tool then either',
      fn: async () => {
        const body = await bodySentFor({ modelId: 'claude-haiku-5-5', },);
        expect(JSON.stringify(body,),).not.toContain('json_schema',);
        expect(body.tools,).toBeUndefined();
        expect(body.tool_choice,).toBeUndefined();
      },
    },),
    it({
      name: 'SENDS ADAPTIVE THINKING WITH ITS SUMMARY SHOWN ON EVERY CARD AND NEVER A WAY TO TURN IT OFF OR BUDGET IT: '
        + 'no disabled, no between_tools, no budget_tokens (the API answers 400 to a budget on both models), because '
        + 'the owner ruled "we should never disable thinking on anything"; display summarized because the default '
        + 'omitted streams empty thinking blocks, so the idle guard would see nothing for the whole think',
      fn: async () => {
        await Promise.all(ANTHROPIC_SERVED_IDS.map(async function checkCard(modelId,) {
          await Promise.all([false, true,].map(async function checkSchemaChoice(withSchema,) {
            const body = await bodySentFor({ modelId, withSchema, },);
            expect(body.thinking,).toEqual({ type: 'adaptive', display: 'summarized', },);
            expect(JSON.stringify(body,),).not.toContain('disabled',);
            expect(JSON.stringify(body,),).not.toContain('between_tools',);
            expect(JSON.stringify(body,),).not.toContain('budget_tokens',);
            expect(JSON.stringify(body,),).not.toContain('omitted',);
          },),);
        },),);
      },
    },),

    it({
      name: 'SENDS THE EFFORT THE CARD NAMES as output_config.effort, one level per effort card, and none at all on '
        + 'the card that names none, which leaves the model its own default (Sonnet high, Haiku medium)',
      fn: async () => {
        await Promise.all(([
          ['claude-sonnet-5-5', undefined,],
          ['claude-sonnet-5-5-effort-low', 'low',],
          ['claude-sonnet-5-5-effort-medium', 'medium',],
          ['claude-sonnet-5-5-effort-high', 'high',],
          ['claude-haiku-5-5', undefined,],
          ['claude-haiku-5-5-effort-low', 'low',],
          ['claude-haiku-5-5-effort-medium', 'medium',],
          ['claude-haiku-5-5-effort-high', 'high',],
        ] as const).map(async function checkEffortCard([modelId, effort,],) {
          await Promise.all([false, true,].map(async function checkSchemaChoice(withSchema,) {
            const body = await bodySentFor({ modelId, withSchema, },);
            /**
             The output config the body carries, empty where it carries none.
             */
            const config: Readonly<Record<string, unknown>> = Object.hasOwn(body, 'output_config',)
              ? (body.output_config as Readonly<Record<string, unknown>>)
              : {};
            expect(config.effort,).toBe(effort,);
            expect(body.model,).toBe(modelId.startsWith('claude-sonnet',) ? 'claude-sonnet-5-5' : 'claude-haiku-5-5',);
          },),);
        },),);
      },
    },),
    it({
      name: 'SENDS NO SAMPLING KNOB: temperature, top_p and top_k are refused with 400 at any non-default value, so '
        + 'a request that happens to carry them reaches the wire with only the members this client names',
      fn: async () => {
        const body = await bodySentFor({
          modelId: 'claude-sonnet-5-5',
          withSchema: true,
          extra: { temperature: 0.2, top_p: 0.9, top_k: 40, },
        },);
        expect(Object.keys(body,).toSorted(),).toEqual([
          'max_tokens',
          'messages',
          'model',
          'output_config',
          'stream',
          'system',
          'thinking',
        ],);
      },
    },),

    it({
      name: 'REFUSES A CONVERSATION THAT ENDS ON AN ASSISTANT TURN, by a named error before any request is built, '
        + 'because an assistant prefill is refused with 400 on both models',
      fn: async () => {
        const { transport, exchanges, } = scripted({
          replies: [{
            status: 200,
            bodyText: claudeStreamOf({ text: '{"spot":"sunbeam"}', stopReason: 'end_turn', usage: PLAIN_USAGE, }),
          },],
        },);

        const failure: unknown = await rejectionOf(
          () =>
            createClaudeClient({ apiKey: CLAUDE_CAT_KEY, transport, retryPolicy: ONE_QUICK_RETRY, },)
              .chatText({
                modelId: 'claude-haiku-5-5',
                messages: [
                  { role: 'user', content: 'Where does the cat sleep?', },
                  { role: 'assistant', content: '{"spot":"', },
                ],
                signal: SIGNAL,
              },),
        );

        expect((failure as Error).name,).toBe('ClaudePrefillError',);
        expect(exchanges.length,).toBe(0,);
      },
    },),
    it({
      name: 'MARKS THE STABLE SYSTEM PREFIX FOR THE FIVE-MINUTE CACHE with one ephemeral breakpoint on the last block of '
        + 'the system field, which every call of a stage shares, and none on the turns that differ per call',
      fn: async () => {
        await Promise.all((['claude-sonnet-5-5', 'claude-haiku-5-5-effort-low',] as const).map(async function checkCachedCard(modelId,) {
          const body = await bodySentFor({ modelId, },);
          expect(body.system,).toEqual([
            { type: 'text', text: 'You find the cat.', cache_control: { type: 'ephemeral', }, },
          ],);
          expect(JSON.stringify(body.messages,),).not.toContain('cache_control',);
        },),);
      },
    },),

    it({
      name: 'SENDS NO SYSTEM FIELD AND NO BREAKPOINT when the call has no system message, rather than caching an '
        + 'empty prefix',
      fn: async () => {
        const body = await bodySentFor({
          modelId: 'claude-haiku-5-5',
          messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
        },);
        expect(body.system,).toBeUndefined();
        expect(JSON.stringify(body,),).not.toContain('cache_control',);
      },
    },),

    it({
      name: 'SENDS A SCHEMA THE STRUCTURED-OUTPUT FEATURE ACCEPTS: additionalProperties false on every object, '
        + 'including those under properties and items, and none of the numeric, string or array bounds it does not '
        + 'support, with the caller\'s own schema left untouched',
      fn: async () => {
        /**
         A schema that breaks each rule the feature states: bare objects at three depths, an object that allows more
         members, and a bound of every unsupported kind.
         */
        const written = {
          type: 'object',
          properties: {
            spot: { type: 'string', minLength: 1, maxLength: 40, },
            hours: { type: 'integer', minimum: 1, maximum: 24, multipleOf: 1, },
            naps: {
              type: 'array',
              minItems: 1,
              maxItems: 9,
              uniqueItems: true,
              items: {
                type: 'object',
                properties: {
                  paragraph: { type: 'integer', minimum: 1, exclusiveMinimum: 0, },
                  inner: {
                    type: 'object',
                    properties: { whisker: { type: 'string', enum: ['left', 'right',], }, },
                    required: ['whisker',],
                    additionalProperties: true,
                  },
                },
                required: ['paragraph', 'inner',],
              },
            },
          },
          required: ['spot', 'hours', 'naps',],
        };
        /**
         The same schema as written, to show the call did not edit it.
         */
        const copy = structuredClone(written,);
        const body = await bodySentFor({ modelId: 'claude-haiku-5-5', schema: written, },);
        expect(body.output_config,).toEqual({
          format: {
            type: 'json_schema',
            schema: {
              type: 'object',
              properties: {
                spot: { type: 'string', description: '{minLength: 1, maxLength: 40}', },
                hours: { type: 'integer', description: '{minimum: 1, maximum: 24, multipleOf: 1}', },
                naps: {
                  type: 'array',
                  minItems: 1,
                  description: '{maxItems: 9, uniqueItems: true}',
                  items: {
                    type: 'object',
                    properties: {
                      paragraph: { type: 'integer', description: '{minimum: 1, exclusiveMinimum: 0}', },
                      inner: {
                        type: 'object',
                        properties: { whisker: { type: 'string', enum: ['left', 'right',], }, },
                        required: ['whisker',],
                        additionalProperties: false,
                      },
                    },
                    required: ['paragraph', 'inner',],
                    additionalProperties: false,
                  },
                },
              },
              required: ['spot', 'hours', 'naps',],
              additionalProperties: false,
            },
          },
        },);
        expect(written,).toEqual(copy,);
      },
    },),

    it({
      name: 'SENDS THE CALLER\'S CEILING WHEN LOWER THAN THE MEASURED CAP, and the cap otherwise, as a whole number '
        + 'of tokens',
      fn: async () => {
        const capped = await bodySentFor({ modelId: 'claude-haiku-5-5', },);
        expect(Number.isSafeInteger(capped.max_tokens,),).toBe(true,);
        expect(capped.max_tokens as number,).toBeGreaterThan(1_000,);
        // Room for the thinking, which counts against it: the pooled p99 of 13,082 measured on other providers
        // is not enough.
        expect(capped.max_tokens,).toBe(32_000,);
        const lowered = await bodySentFor({ modelId: 'claude-haiku-5-5', extra: { maxTokens: 777, }, },);
        expect(lowered.max_tokens,).toBe(777,);
      },
    },),
  ],
},);
