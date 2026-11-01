/**
 Wire-level test that the schema reaches the model on the Synthetic path.

 SEPARATE FILE BY NECESSITY, not preference: `synthetic-client.unit.test.ts`
 is near its line budget, and `MXL` forbids raising one. The seam under test
 is also narrower than that file's subject, which is the whole client.

 WHY ONLY THIS PROVIDER. It was once believed that no system prompt carried
 its schema. Reading the deciding source refuted that for Charm Hyper:
 `buildAnthropicBody` routes every schema-bearing call through
 `renderToolSystemPrompt`, which prints the whole schema into the `system`
 field with its own format rules. The Synthetic path had nothing of the kind,
 sending only the API-level `response_format`, so it is the one that changed.

 READS THE BYTES, not the transform. `schema-prompt.unit.test.ts` covers the
 pure function. This asserts the property that actually matters: what the
 provider receives. A transform that worked and a client that ignored it
 would pass every test in the other file.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  ModelTransport,
  TransportExchange,
} from '@monochromatic-dev/translation-repair-provider-stream';
import {
  createSyntheticClient,
  SCHEMA_BLOCK_HEADING,
} from '../dist/final/node/index.mjs';
import { SEAT_HYPER_OPENROUTER_VISION_EDITOR, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Streamed completion the fake transport replays, shaped as the reader wants.
 */
const COMPLETION_BODY = [
  `data: ${JSON.stringify({ choices: [{ delta: { content: '{"verdict":"pass"}', }, },], },)}`,
  'data: [DONE]',
  '',
].join('\n\n',);

/**
 Response format the call sends, and therefore what the prompt must state.
 */
const NAP_FORMAT = {
  type: 'json_schema' as const,
  json_schema: {
    name: 'nap_report',
    schema: {
      type: 'object',
      required: ['spot',],
      additionalProperties: false,
      properties: { spot: { type: 'string', }, },
    },
  },
};

/**
 Conversation the caller builds, with a system prompt of its own.
 */
const MESSAGES = [
  {
    role: 'system' as const,
    content: 'You are a careful cat.',
  },
  {
    role: 'user' as const,
    content: 'Where did the cat nap?',
  },
];

/**
 Records every exchange and replays one completion for each.

 @returns Transport to inject, and the list it fills

 @example
 ```ts
 const { transport, exchanges, } = recordingTransport();
 ```
 */
function recordingTransport(): {
  readonly transport: ModelTransport;
  readonly exchanges: TransportExchange[];
} {
  /**
   Every exchange the client performed, in order.
   */
  const exchanges: TransportExchange[] = [];

  return {
    transport: async function replay(exchange,) {
      exchanges.push(exchange,);
      return {
        status: 200,
        bodyText: COMPLETION_BODY,
      };
    },
    exchanges,
  };
}

/**
 System prompt of the body the client actually sent.

 @param exchanges - exchanges the transport recorded

 @returns Text of the first system message on the wire

 @throws {@link Error} when no body or no system message was sent

 @example
 ```ts
 expect(sentSystemPrompt({ exchanges, },),).toContain(SCHEMA_BLOCK_HEADING,);
 ```
 */
function sentSystemPrompt(
  { exchanges, }: { readonly exchanges: readonly TransportExchange[]; },
): string {
  /**
   Serialized body of the first exchange.
   */
  const bodyJson = exchanges.at(0,)?.bodyJson;
  if (bodyJson === undefined)
    throw new Error('exchange carried no body',);

  /**
   Body as the provider would parse it.
   */
  const body: unknown = JSON.parse(bodyJson,);

  /**
   Messages field, read defensively because this is a wire assertion.
   */
  const { messages, } = body as { readonly messages?: unknown; };
  if (!Array.isArray(messages,))
    throw new Error('body carried no messages array',);

  /**
   First message whose role is system.
   */
  const system = (messages as readonly { readonly role?: unknown; readonly content?: unknown; }[])
    .find(function isSystem(message,): boolean {
      return message.role === 'system';
    },);

  if ((typeof system?.content) !== 'string')
    throw new Error('body carried no string system prompt',);
  return system.content;
}

await describe({
  name: 'synthetic wire carries the response schema',
  children: [
    it({
      name: 'SENDS the whole schema inside the system prompt, beside the '
        + 'prompt the caller wrote, on a call that declares one',
      fn: async () => {
        const { transport, exchanges, } = recordingTransport();
        await createSyntheticClient({
          apiKey: 'test-key',
          transport,
        },).chatText({
          modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
          messages: MESSAGES,
          signal: new AbortController().signal,
          responseFormat: NAP_FORMAT,
        },);

        /**
         System prompt as the provider would read it.
         */
        const prompt = sentSystemPrompt({ exchanges, },);

        expect(prompt,).toContain('You are a careful cat.',);
        expect(prompt,).toContain(SCHEMA_BLOCK_HEADING,);
        expect(prompt,).toContain(JSON.stringify(
          NAP_FORMAT.json_schema.schema,
          null,
          2,
        ),);
      },
    },),

    it({
      name: 'LEAVES a free-text call untouched, so a caller with no schema '
        + 'sends exactly what it wrote and pays for nothing else',
      fn: async () => {
        const { transport, exchanges, } = recordingTransport();
        await createSyntheticClient({
          apiKey: 'test-key',
          transport,
        },).chatText({
          modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
          messages: MESSAGES,
          signal: new AbortController().signal,
        },);

        expect(sentSystemPrompt({ exchanges, },),).toBe('You are a careful cat.',);
      },
    },),
  ],
},);
