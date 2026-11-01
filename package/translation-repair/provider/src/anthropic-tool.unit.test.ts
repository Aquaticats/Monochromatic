/**
 Tests for the answer tool, in both of the places it is described.

 THE ANTI-DRIFT CASE IS THE POINT OF THIS FILE. The owner's instruction is
 that the full tool schema goes into the system prompt as well as into
 `tools`, because some model and provider pairs emit the wrong call format
 without it. Two renderings of one schema can disagree, and a disagreement
 here teaches a model to call a tool that is not the one being offered, so the
 cases in this file check the name and the schema body in BOTH renderings rather than
 checking each rendering alone.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  answerToolDefinition,
  answerToolName,
  renderToolSystemPrompt,
  UnnameableToolError,
} from '../dist/final/node/index.mjs';

/**
 Structured-output constraint standing in for a real stage's.

 NESTED ON PURPOSE: a renderer that printed only top-level keys would pass a
 flat fixture, and every real schema in this pipeline nests.
 */
const catFormat = {
  type: 'json_schema',
  json_schema: {
    name: 'whisker_report',
    strict: true,
    schema: {
      type: 'object',
      required: ['verdict', 'paws',],
      additionalProperties: false,
      properties: {
        verdict: {
          type: 'string',
          enum: ['purring', 'affronted',],
        },
        paws: {
          type: 'array',
          items: {
            type: 'object',
            required: ['toebeans',],
            properties: { toebeans: { type: 'integer', }, },
          },
        },
      },
    },
  },
} as const;

/**
 Builds a constraint carrying a chosen name over the same schema body.

 @param name - schema name under test

 @returns Constraint the functions under test take

 @example
 ```ts
 const format = namedAs({ name: 'whisker_report', },);
 ```
 */
function namedAs(
  { name, }: { readonly name: string; },
): typeof catFormat {
  return {
    ...catFormat,
    json_schema: {
      ...catFormat.json_schema,
      name,
    },
  } as typeof catFormat;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: answerToolName.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS the schema its own name, so one stage is described identically to whichever '
            + 'provider serves it',
          fn: async () => {
            expect(answerToolName({ responseFormat: catFormat, },),).toBe('whisker_report',);
          },
        },),

        it({
          name: 'REFUSES an empty name, which the Messages API rejects at the request',
          fn: async () => {
            expect(() => {
              answerToolName({ responseFormat: namedAs({ name: '', },), },);
            },).toThrow(UnnameableToolError,);
          },
        },),

        it({
          name: 'REFUSES a name past the 64-character ceiling rather than letting the provider '
            + 'answer 400 on every call of that stage',
          fn: async () => {
            expect(() => {
              answerToolName({ responseFormat: namedAs({ name: 'c'.repeat(65,), },), },);
            },).toThrow(UnnameableToolError,);

            expect(answerToolName({ responseFormat: namedAs({ name: 'c'.repeat(64,), },), },),)
              .toBe('c'.repeat(64,),);
          },
        },),

        it({
          name: 'REFUSES every character outside letters, digits, underscore and hyphen, including '
            + 'the dot and space a schema name could plausibly carry',
          fn: async () => {
            for (const name of ['whisker.report', 'whisker report', 'whisker/report', '猫报告',]) {
              expect(() => {
                answerToolName({ responseFormat: namedAs({ name, },), },);
              },).toThrow(UnnameableToolError,);
            }
          },
        },),

        it({
          name: 'ACCEPTS the hyphen and underscore the protocol allows',
          fn: async () => {
            expect(answerToolName({ responseFormat: namedAs({ name: 'whisker-report_2', },), },),)
              .toBe('whisker-report_2',);
          },
        },),
      ],
    },),

    describe({
      name: answerToolDefinition.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CARRIES the schema body itself as input_schema, rather than a copy that could be '
            + 'reshaped on the way',
          fn: async () => {
            expect(answerToolDefinition({ responseFormat: catFormat, },).input_schema,)
              .toEqual(catFormat.json_schema.schema,);
          },
        },),

        it({
          name: 'NAMES the tool in its description, so a model choosing between tools reads what '
            + 'calling this one means',
          fn: async () => {
            /**
             Tool entry as the request body would carry it.
             */
            const tool = answerToolDefinition({ responseFormat: catFormat, },);

            expect(tool.name,).toBe('whisker_report',);
            expect(tool.description.includes('whisker_report',),).toBe(true,);
          },
        },),

        it({
          name: 'FORWARDS the naming refusal rather than building a tool the provider rejects',
          fn: async () => {
            expect(() => {
              answerToolDefinition({ responseFormat: namedAs({ name: 'whisker report', },), },);
            },).toThrow(UnnameableToolError,);
          },
        },),
      ],
    },),

    describe({
      name: renderToolSystemPrompt.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PUTS the caller instruction first, because that is the task and the answer protocol '
            + 'is only how to hand its result back',
          fn: async () => {
            /**
             System text as the request body would carry it.
             */
            const system = renderToolSystemPrompt({
              instruction: 'You count the toebeans of one cat.',
              responseFormat: catFormat,
            },);

            expect(system.startsWith('You count the toebeans of one cat.',),).toBe(true,);
            expect(system.indexOf('HOW TO ANSWER',),)
              .toBeGreaterThan(system.indexOf('toebeans of one cat',),);
          },
        },),

        it({
          name: 'CARRIES the whole schema, nested property names included, which is the owner '
            + 'instruction this rendering exists for',
          fn: async () => {
            /**
             System text as the request body would carry it.
             */
            const system = renderToolSystemPrompt({
              instruction: 'You count the toebeans of one cat.',
              responseFormat: catFormat,
            },);

            for (const fragment of ['verdict', 'paws', 'toebeans', 'affronted', 'additionalProperties',]) {
              expect(system.includes(fragment,),).toBe(true,);
            }
          },
        },),

        it({
          name: 'RENDERS the schema as JSON a model can parse, not as a description of it',
          fn: async () => {
            /**
             System text as the request body would carry it.
             */
            const system = renderToolSystemPrompt({
              instruction: '',
              responseFormat: catFormat,
            },);

            /**
             Schema as it was printed into the prompt, cut out and read back.
             */
            const printed: unknown = JSON.parse(system.slice(
              system.indexOf('{',),
              system.lastIndexOf('}',) + 1,
            ),);

            expect(printed,).toEqual(catFormat.json_schema.schema,);
          },
        },),

        it({
          name: 'OPENS on the protocol where the caller sent no instruction, rather than on blank '
            + 'lines a model has to read past',
          fn: async () => {
            expect(renderToolSystemPrompt({
              instruction: '',
              responseFormat: catFormat,
            },).startsWith('HOW TO ANSWER',),).toBe(true,);
          },
        },),

        it({
          name: 'NAMES the same tool the definition does, so the prose cannot teach a model to call '
            + 'a tool that was never offered',
          fn: async () => {
            expect(renderToolSystemPrompt({
              instruction: '',
              responseFormat: catFormat,
            },).includes(answerToolDefinition({ responseFormat: catFormat, },).name,),).toBe(true,);
          },
        },),

        it({
          name: 'STATES every call-format mistake the owner named, because a model that reads only '
            + 'the schema still gets the envelope wrong',
          fn: async () => {
            /**
             System text as the request body would carry it.
             */
            const system = renderToolSystemPrompt({
              instruction: '',
              responseFormat: catFormat,
            },);

            for (const rule of [
              'fenced',
              'string that contains JSON',
              'top level of the tool input',
              'Do not rename one',
              'empty array',
              'Call the tool once',
            ]) {
              expect(system.includes(rule,),).toBe(true,);
            }
          },
        },),

        it({
          name: 'FORWARDS the naming refusal rather than telling a model to call an unnameable tool',
          fn: async () => {
            expect(() => {
              renderToolSystemPrompt({
                instruction: '',
                responseFormat: namedAs({ name: '', },),
              },);
            },).toThrow(UnnameableToolError,);
          },
        },),
      ],
    },),
  ],
},);
