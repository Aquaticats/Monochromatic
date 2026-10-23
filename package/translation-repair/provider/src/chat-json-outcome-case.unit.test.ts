/**
 Tests for the capitalization of an `enum` or `const` value in a reply, which the structured outputs page of Anthropic
 says it does not guarantee: "Claude might return a value that differs from your schema only in capitalization,
 typically in the first letter of a word following a space" (read 2026-10-23).

 A STAGE'S GUARD COMPARES A LABEL WITH `includes` OR `===`, so a label that differs only in its capital reads as no
 label and costs the voice. These cases hold the reading to the rule that settles it: when the schema the caller sent
 holds exactly one value that matches the reply's value ignoring capitalization, the reader takes the schema's own
 spelling and warns; when none or several match, the reply is read as it came and the guard decides.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  type JsonSchemaResponseFormat,
  readJsonOutcome,
} from '../dist/final/node/index.mjs';
import { SEAT_HYPER_VISION, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  warnLinesDuring,
} from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';

/**
 Model named on every reading, for the log lines only.
 */
const MODEL_ID = SEAT_HYPER_VISION;

/**
 The two verdicts a cat judge may give, the guard's own list.
 */
const VERDICTS: readonly string[] = ['consistent', 'not consistent',];

/**
 Guards a reply whose `verdict` is one of the two spellings the guard knows, compared as a stage's guard does.

 @param value - parsed candidate

 @returns Whether value carries a verdict of the list

 @example
 ```ts
 isVerdictReply({ verdict: 'consistent', },);
 ```
 */
function isVerdictReply(value: unknown,): value is { readonly verdict: string; } {
  return isJsonRecord(value,) && ((typeof value.verdict) === 'string') && VERDICTS.includes(value.verdict as string,);
}

/**
 Wraps a schema as the response format a stage sends.

 @param schema - JSON schema of the reply

 @returns Response format that wraps the schema

 @example
 ```ts
 const format = formatOf({ schema: { type: 'object', }, },);
 ```
 */
function formatOf({ schema, }: { readonly schema: Record<string, unknown>; },): JsonSchemaResponseFormat {
  return {
    type: 'json_schema',
    json_schema: {
      name: 'cat_verdict',
      schema,
    },
  };
}

/**
 Schema of a reply with one `verdict`, listed as the guard lists it.
 */
const VERDICT_SCHEMA: Record<string, unknown> = {
  type: 'object',
  properties: { verdict: { type: 'string', enum: [...VERDICTS,], }, },
  required: ['verdict',],
  additionalProperties: false,
};

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: `${readJsonOutcome.name} enum capitalization`,
      concurrency: 1,
      children: [
        it({
          name: 'TAKES THE SCHEMA\'S SPELLING of a verdict that differs only in its capital, and warns with the field, '
            + 'the value received and the value taken',
          fn: async () => {
            const { result: outcome, warned, } = await warnLinesDuring({
              run: async () =>
                readJsonOutcome({
                  modelId: MODEL_ID,
                  reply: { text: '{"verdict":"Consistent"}', },
                  validate: isVerdictReply,
                  responseFormat: formatOf({ schema: VERDICT_SCHEMA, },),
                },),
            },);

            expect(outcome.kind,).toBe('ok',);
            expect(outcome.kind === 'ok' ? outcome.value : undefined,).toEqual({ verdict: 'consistent', },);
            expect(outcome.rawText,).toBe('{"verdict":"Consistent"}',);
            expect(warned.length,).toBe(1,);
            expect(warned[0],).toContain(MODEL_ID,);
            expect(warned[0],).toContain('verdict',);
            expect(warned[0],).toContain('"Consistent"',);
            expect(warned[0],).toContain('"consistent"',);
          },
        },),

        it({
          name: 'TAKES THE SCHEMA\'S SPELLING of the form the page names, a capital on the word after a space',
          fn: async () => {
            const { result: outcome, warned, } = await warnLinesDuring({
              run: async () =>
                readJsonOutcome({
                  modelId: MODEL_ID,
                  reply: { text: '{"verdict":"not Consistent"}', },
                  validate: isVerdictReply,
                  responseFormat: formatOf({ schema: VERDICT_SCHEMA, },),
                },),
            },);

            expect(outcome.kind === 'ok' ? outcome.value : undefined,).toEqual({ verdict: 'not consistent', },);
            expect(warned.length,).toBe(1,);
            expect(warned[0],).toContain('"not Consistent"',);
            expect(warned[0],).toContain('"not consistent"',);
          },
        },),

        it({
          name: 'TAKES THE SCHEMA\'S SPELLING at every depth the schema lists the value, a member of a list of objects '
            + 'included, and names the field by its path',
          fn: async () => {
            /**
             A list of findings, each with a severity the schema lists.
             */
            const schema = {
              type: 'object',
              properties: {
                findings: {
                  type: 'array',
                  items: {
                    type: 'object',
                    properties: {
                      severity: { type: 'string', enum: ['minor', 'major',], },
                      note: { type: 'string', },
                    },
                  },
                },
              },
            };
            const { result: outcome, warned, } = await warnLinesDuring({
              run: async () =>
                readJsonOutcome({
                  modelId: MODEL_ID,
                  reply: {
                    text: '{"findings":[{"severity":"minor","note":"Purr"},'
                      + '{"severity":"Major","note":"Hiss"}]}',
                  },
                  validate: isJsonRecord,
                  responseFormat: formatOf({ schema, },),
                },),
            },);

            expect(outcome.kind === 'ok' ? outcome.value : undefined,).toEqual({
              findings: [
                {
                  severity: 'minor',
                  note: 'Purr',
                },
                {
                  severity: 'major',
                  note: 'Hiss',
                },
              ],
            },);
            expect(warned.length,).toBe(1,);
            expect(warned[0],).toContain('findings[1].severity',);
          },
        },),

        it({
          name: 'TAKES THE SCHEMA\'S SPELLING of a `const` too',
          fn: async () => {
            const { result: outcome, } = await warnLinesDuring({
              run: async () =>
                readJsonOutcome({
                  modelId: MODEL_ID,
                  reply: { text: '{"kind":"Ballot"}', },
                  validate: function isBallot(value: unknown,): value is { readonly kind: 'ballot'; } {
                    return isJsonRecord(value,) && (value.kind === 'ballot');
                  },
                  responseFormat: formatOf({
                    schema: {
                      type: 'object',
                      properties: { kind: { type: 'string', const: 'ballot', }, },
                    },
                  },),
                },),
            },);

            expect(outcome.kind === 'ok' ? outcome.value : undefined,).toEqual({ kind: 'ballot', },);
          },
        },),

        it({
          name: 'LEAVES A VALUE THE SCHEMA LISTS EXACTLY, with no warning',
          fn: async () => {
            const { result: outcome, warned, } = await warnLinesDuring({
              run: async () =>
                readJsonOutcome({
                  modelId: MODEL_ID,
                  reply: { text: '{"verdict":"not consistent"}', },
                  validate: isVerdictReply,
                  responseFormat: formatOf({ schema: VERDICT_SCHEMA, },),
                },),
            },);

            expect(outcome.kind === 'ok' ? outcome.value : undefined,).toEqual({ verdict: 'not consistent', },);
            expect(warned,).toEqual([],);
          },
        },),

        it({
          name: 'KEEPS THE REFUSAL OF A VALUE NO SPELLING OF THE SCHEMA MATCHES, with no warning',
          fn: async () => {
            const { result: outcome, warned, } = await warnLinesDuring({
              run: async () =>
                readJsonOutcome({
                  modelId: MODEL_ID,
                  reply: { text: '{"verdict":"Maybe"}', },
                  validate: isVerdictReply,
                  responseFormat: formatOf({ schema: VERDICT_SCHEMA, },),
                },),
            },);

            expect(outcome.kind,).toBe('schema-mismatch',);
            expect(
              outcome.kind === 'schema-mismatch' ? outcome.reason : undefined,
            ).toBe('caller-guard-rejected',);
            expect(warned,).toEqual([],);
          },
        },),

        it({
          name: 'KEEPS THE REFUSAL OF A VALUE SEVERAL SPELLINGS OF THE SCHEMA MATCH, since the schema does not say '
            + 'which was meant',
          fn: async () => {
            const { result: outcome, warned, } = await warnLinesDuring({
              run: async () =>
                readJsonOutcome({
                  modelId: MODEL_ID,
                  reply: { text: '{"verdict":"CONSISTENT"}', },
                  validate: isVerdictReply,
                  responseFormat: formatOf({
                    schema: {
                      type: 'object',
                      properties: {
                        verdict: {
                          type: 'string',
                          enum: ['consistent', 'Consistent', 'not consistent',],
                        },
                      },
                    },
                  },),
                },),
            },);

            expect(outcome.kind,).toBe('schema-mismatch',);
            expect(warned,).toEqual([],);
          },
        },),

        it({
          name: 'READS THE REPLY AS IT CAME where the caller sent no response format, which is the control arm',
          fn: async () => {
            const outcome = readJsonOutcome({
              modelId: MODEL_ID,
              reply: { text: '{"verdict":"Consistent"}', },
              validate: isVerdictReply,
            },);

            expect(outcome.kind,).toBe('schema-mismatch',);
          },
        },),

        it({
          name: 'LEAVES A STRING AT A NODE WITH NO LIST OF VALUES, and a value the schema lists no property for',
          fn: async () => {
            const { result: outcome, warned, } = await warnLinesDuring({
              run: async () =>
                readJsonOutcome({
                  modelId: MODEL_ID,
                  reply: { text: '{"verdict":"consistent","note":"Purr","extra":"Consistent"}', },
                  validate: isJsonRecord,
                  responseFormat: formatOf({ schema: VERDICT_SCHEMA, },),
                },),
            },);

            expect(outcome.kind === 'ok' ? outcome.value : undefined,).toEqual({
              verdict: 'consistent',
              note: 'Purr',
              extra: 'Consistent',
            },);
            expect(warned,).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
