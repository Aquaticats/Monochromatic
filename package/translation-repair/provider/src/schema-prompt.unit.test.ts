/**
 Tests for stating a call's own response schema inside its system prompt.

 THIS IMPLEMENTS A DIRECT OWNER INSTRUCTION: put the full schema into the
 system prompt, because some model and provider pairs behave badly without a
 detailed one and answer in the wrong shape. Before this, seventeen modules
 built a system message and not one mentioned the shape it expected back.

 DERIVED, NEVER COPIED. Every case here renders from the same
 `JsonSchemaResponseFormat` value the request puts on the wire, which is the
 property that makes drift between the two impossible rather than unlikely.
 A test that spelled the expected block out by hand would pass while the two
 diverged, so the cases assert the RELATION between the format and the text.

 IDEMPOTENCE IS LOAD-BEARING, not tidiness. A routed call can cross more than
 one seam and a re-route rebuilds its request, so a transform that appended
 unconditionally would state the schema two or three times and spend tokens
 saying the same thing.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  renderSchemaForPrompt,
  SCHEMA_BLOCK_HEADING,
  withSchemaInSystemPrompt,
} from '../dist/final/node/index.mjs';

/**
 Response format a case sends, shaped like the ones production sends.

 CARRIES AN ARRAY OF OBJECTS deliberately: the measured failure this exists to
 prevent returned a JSON-stringified array where one of these was declared.
 */
const NAP_FORMAT = {
  type: 'json_schema' as const,
  json_schema: {
    name: 'nap_report',
    schema: {
      type: 'object',
      required: ['naps',],
      additionalProperties: false,
      properties: {
        naps: {
          type: 'array',
          items: {
            type: 'object',
            required: [
              'spot',
              'minutes',
            ],
            properties: {
              spot: { type: 'string', },
              minutes: { type: 'integer', },
            },
          },
        },
      },
    },
  },
};

/**
 System message a case starts from.
 */
const SYSTEM = {
  role: 'system' as const,
  content: 'You are a careful cat.',
};

/**
 User message a case starts from, which must come back untouched.
 */
const USER = {
  role: 'user' as const,
  content: 'Where did the cat nap?',
};

/**
 Text of whichever message carries the system role, joined where it has parts.

 @param messages - conversation to read

 @returns System prompt as the model would see it, empty where there is none

 @example
 ```ts
 expect(systemTextOf({ messages, },),).toContain(SCHEMA_BLOCK_HEADING,);
 ```
 */
function systemTextOf(
  { messages, }: { readonly messages: readonly { readonly role: string; readonly content: unknown; }[]; },
): string {
  /**
   First system message, absent where the conversation has none.
   */
  const system = messages.find(function isSystem(message,): boolean {
    return message.role === 'system';
  },);

  if (system === undefined)
    return '';

  /**
   Content as that message carries it.
   */
  const { content, } = system;

  if ((typeof content) === 'string')
    return content;

  return (content as readonly { readonly type: string; readonly text?: string; }[])
    .map(function partText(part,): string {
      return part.text ?? `[${part.type}]`;
    },)
    .join('\n',);
}

/**
 How many times the block heading appears in a conversation's system prompt.

 @param messages - conversation to read

 @returns Occurrence count

 @example
 ```ts
 expect(headingCount({ messages, },),).toBe(1,);
 ```
 */
function headingCount(
  { messages, }: { readonly messages: readonly { readonly role: string; readonly content: unknown; }[]; },
): number {
  return systemTextOf({ messages, },)
    .split(SCHEMA_BLOCK_HEADING,)
    .length - 1;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: renderSchemaForPrompt.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CARRIES the schema itself, not a description of it, so the text '
            + 'and the wire cannot say different things',
          fn: async () => {
            const block = renderSchemaForPrompt({ format: NAP_FORMAT, },);
            expect(block,).toContain('"additionalProperties": false',);
            expect(block,).toContain('"minutes"',);
            // THE WHOLE SCHEMA, rendered from the same value the request sends.
            // "Put even the full tool schema into system prompts" is the
            // instruction, and a block carrying part of it would satisfy the
            // `additionalProperties` and `minutes` assertions while failing the thing that was asked for.
            expect(block,).toContain(JSON.stringify(
              NAP_FORMAT.json_schema.schema,
              null,
              2,
            ),);
          },
        },),

        it({
          name: 'NAMES the schema, so a model asked for one shape by two channels '
            + 'can tell they are the same request',
          fn: async () => {
            expect(renderSchemaForPrompt({ format: NAP_FORMAT, },),).toContain('nap_report',);
          },
        },),

        it({
          name: 'FORBIDS the exact failure measured in production, a JSON string '
            + 'holding an array where an array of objects is declared',
          fn: async () => {
            const block = renderSchemaForPrompt({ format: NAP_FORMAT, },);
            expect(block,).toContain('never a string containing one',);
          },
        },),
      ],
    },),

    describe({
      name: withSchemaInSystemPrompt.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RETURNS THE SAME ARRAY for a free-text call, so a caller with no '
            + 'schema pays nothing and can compare by identity',
          fn: async () => {
            const messages = [
              SYSTEM,
              USER,
            ];
            expect(withSchemaInSystemPrompt({ messages, },),).toBe(messages,);
          },
        },),

        it({
          name: 'APPENDS to the system prompt the caller wrote rather than '
            + 'replacing it, since the framing and rules there are the reason it '
            + 'exists',
          fn: async () => {
            const amended = withSchemaInSystemPrompt({
              messages: [
                SYSTEM,
                USER,
              ],
              responseFormat: NAP_FORMAT,
            },);
            const text = systemTextOf({ messages: amended, },);
            expect(text,).toContain('You are a careful cat.',);
            expect(text,).toContain(SCHEMA_BLOCK_HEADING,);
            expect(text.indexOf('You are a careful cat.',),)
              .toBeLessThan(text.indexOf(SCHEMA_BLOCK_HEADING,),);
          },
        },),

        it({
          name: 'LEAVES every other message exactly as it was',
          fn: async () => {
            const amended = withSchemaInSystemPrompt({
              messages: [
                SYSTEM,
                USER,
              ],
              responseFormat: NAP_FORMAT,
            },);
            expect(amended.at(1,),).toBe(USER,);
          },
        },),

        it({
          name: 'ADDS a system message where the call had none, rather than '
            + 'dropping the schema on exactly the calls that state least',
          fn: async () => {
            const amended = withSchemaInSystemPrompt({
              messages: [USER,],
              responseFormat: NAP_FORMAT,
            },);
            expect(amended.at(0,)?.role,).toBe('system',);
            expect(amended.at(1,),).toBe(USER,);
            expect(systemTextOf({ messages: amended, },),).toContain(SCHEMA_BLOCK_HEADING,);
          },
        },),

        it({
          name: 'ADDS A TEXT PART to a system message carrying parts, so a call '
            + 'that also sends a picture is not the one call that loses its schema',
          fn: async () => {
            const amended = withSchemaInSystemPrompt({
              messages: [
                {
                  role: 'system' as const,
                  content: [{
                    type: 'text' as const,
                    text: 'You are a careful cat.',
                  },],
                },
                USER,
              ],
              responseFormat: NAP_FORMAT,
            },);
            const text = systemTextOf({ messages: amended, },);
            expect(text,).toContain('You are a careful cat.',);
            expect(text,).toContain(SCHEMA_BLOCK_HEADING,);
          },
        },),

        it({
          name: 'STATES IT ONCE however many times it is applied, because a routed '
            + 'call crosses more than one seam and a re-route rebuilds its request',
          fn: async () => {
            const once = withSchemaInSystemPrompt({
              messages: [
                SYSTEM,
                USER,
              ],
              responseFormat: NAP_FORMAT,
            },);
            const twice = withSchemaInSystemPrompt({
              messages: once,
              responseFormat: NAP_FORMAT,
            },);
            expect(headingCount({ messages: once, },),).toBe(1,);
            expect(headingCount({ messages: twice, },),).toBe(1,);
            expect(twice,).toBe(once,);
          },
        },),

        it({
          name: 'AMENDS ONE system message when a call carries two, rather than '
            + 'stating the schema twice',
          fn: async () => {
            const amended = withSchemaInSystemPrompt({
              messages: [
                SYSTEM,
                {
                  role: 'system' as const,
                  content: 'You also count naps.',
                },
                USER,
              ],
              responseFormat: NAP_FORMAT,
            },);
            expect(amended.filter(function statesIt(message,): boolean {
              return ((typeof message.content) === 'string')
                && message.content.includes(SCHEMA_BLOCK_HEADING,);
            },).length,).toBe(1,);
          },
        },),
      ],
    },),
  ],
},);
