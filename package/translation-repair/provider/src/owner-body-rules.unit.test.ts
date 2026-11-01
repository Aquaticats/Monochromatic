/**
 Tests that every provider client's request body keeps the owner's standing
 rules, and that a seat the owner dropped from OpenRouter stays off it
 (ledger P11).

 WHY ONE TABLE. The rules ("never set thinking, budget_tokens or
 reasoning_effort; always send max_tokens") were guarded for the Hyper and
 OpenRouter bodies in one routing test, and the Synthetic and Bedrock bodies
 by nothing: each client builds its own body, so a knob added to one is
 caught only where a test reads that body. And Qwen3.8-27B and glm-5.3 stayed
 off OpenRouter only because their cards carry no OpenRouter block, so
 re-adding one would route them there with nothing to say so.

 NEURALWATT IS THE ONE EXCEPTION, since 2026-10-21 ("I'm lifting the ban on
 passing params, but only for NeuralWatt"): its client sends `reasoning_effort`
 on a picture call to a seat whose card names one, and
 `neuralwatt-picture-effort.unit.test.ts` pins that. The picture pass here is
 the other half: it shows that no OTHER provider's body carries a knob when
 given the same picture.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { MODEL_CARDS, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type BedrockLedger,
  createBedrockClient,
  createHyperClient,
  createOpenRouterClient,
  createSyntheticClient,
  OPENROUTER_DROPPED_SEATS,
  reachOf,
  type ChatTextRequest,
  type ModelClient,
} from '../dist/final/node/index.mjs';
import {
  recordingTransport,
  settled,
} from './recording-transport.test-fixture.ts';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_HYPER_ONLY,
  SEAT_OPENROUTER_ONLY,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Keys the owner's instruction of 2026-08-25 forbids in any body, at any depth.
 */
const FORBIDDEN_KEYS: ReadonlySet<string> = new Set([
  'thinking',
  'budget_tokens',
  'reasoning_effort',
  'reasoning',
  'include_reasoning',
],);

/**
 Retry policy that makes one attempt.
 */
const ONE_ATTEMPT = {
  limit: 0,
  baseMs: 1,
};

/**
 Ledger that holds nothing, for the Bedrock client, which never reaches it here.
 */
const EMPTY_LEDGER: BedrockLedger = {
  path: '/nowhere/bedrock-spend.jsonl',
  note: async function note(): Promise<void> {
    await Promise.resolve();
  },
  read: async function read() {
    return {
      creditUsd: 200,
      spentUsd: 0,
      reckonedUsd: 0,
      remainingUsd: 200,
      calls: 0,
    };
  },
};

/**
 Every key an object carries, at every depth.

 @param value - parsed body or any part of it

 @returns Keys, repeated where they repeat

 @example
 ```ts
 const keys = keysOf(JSON.parse(bodyJson,),);
 ```
 */
function keysOf(value: unknown,): readonly string[] {
  /**
   Values still to walk.
   */
  const pending: unknown[] = [value,];
  /**
   Keys found so far.
   */
  const keys: string[] = [];
  while (pending.length > 0) {
    /**
     Next value.
     */
    const next = pending.pop();
    if (Array.isArray(next,)) {
      for (const item of next as readonly unknown[])
        pending.push(item,);
    }
    else if (((typeof next) === 'object') && (next !== null)) {
      for (const [key, inner,] of Object.entries(next,)) {
        keys.push(key,);
        pending.push(inner,);
      }
    }
  }
  return keys;
}

/**
 Each provider, the seat it alone or first serves, and a client over the
 recording transport.

 @param bodies - where the recorded bodies go

 @returns One row per provider

 @example
 ```ts
 const rows = providerRows({ bodies, },);
 ```
 */
function providerRows(
  { bodies, }: { readonly bodies: string[]; },
): readonly { readonly provider: string; readonly modelId: string; readonly client: Pick<ModelClient, 'chatJson' | 'chatText'>; }[] {
  /**
   Transport shared by every row.
   */
  const transport = recordingTransport({ bodies, },);
  return [
    {
      provider: 'synthetic',
      modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
      client: createSyntheticClient({ apiKey: 'whiskers', transport, retryPolicy: ONE_ATTEMPT, },),
    },
    {
      provider: 'hyper',
      modelId: SEAT_HYPER_ONLY,
      client: createHyperClient({ apiKey: 'whiskers', transport, retryPolicy: ONE_ATTEMPT, },),
    },
    {
      provider: 'openrouter',
      modelId: SEAT_OPENROUTER_ONLY,
      client: createOpenRouterClient({ apiKey: 'whiskers', transport, retryPolicy: ONE_ATTEMPT, },),
    },
    {
      provider: 'bedrock',
      modelId: SEAT_BEDROCK_ONLY_TEXT,
      // No stream bound: the card's own would be armed on the real clock
      // around every attempt (ledger B416).
      client: createBedrockClient({
        apiKey: 'whiskers',
        ledger: EMPTY_LEDGER,
        transport,
        retryPolicy: ONE_ATTEMPT,
        streamBoundMsOverride: 'unbounded',
      },),
    },
  ];
}

/**
 The conversation of the text calls.
 */
const TEXT_MESSAGES = [{ role: 'user', content: 'Where does the cat nap?', },] as const;

/**
 The conversation of the picture calls, one image part as the reading stage builds it.
 */
const PICTURE_MESSAGES = [{
  role: 'user',
  content: [
    {
      type: 'text',
      text: 'Transcribe every word visible in this image.',
    },
    {
      type: 'image_url',
      image_url: { url: 'data:image/webp;base64,UklGRg==', },
    },
  ],
},] as const;

/**
 What each provider's bodies break of the owner's standing rules, for one kind
 of conversation.

 @param messages - conversation each provider's text call and schema'd call send

 @returns One finding per rule a body broke, empty when every body holds

 @example
 ```ts
 const findings = await bodyFindingsFor({ messages: [{ role: 'user', content: 'Hi', },], },);
 ```
 */
async function bodyFindingsFor(
  { messages, }: { readonly messages: ChatTextRequest['messages']; },
): Promise<readonly string[]> {
  /**
   Bodies the transport recorded, in call order.
   */
  const bodies: string[] = [];
  /**
   What each row's two calls sent.
   */
  const findings: string[] = [];
  for (const row of providerRows({ bodies, },)) {
    /**
     Bodies before this row's calls.
     */
    const before = bodies.length;
    // oxlint-disable-next-line no-await-in-loop -- the rows share one recorder, read in order
    await settled(row.client.chatText({
      modelId: row.modelId as never,
      messages,
      signal: new AbortController().signal,
    },),);
    // oxlint-disable-next-line no-await-in-loop -- the rows share one recorder, read in order
    await settled(row.client.chatJson({
      modelId: row.modelId as never,
      messages,
      signal: new AbortController().signal,
      responseFormat: {
        type: 'json_schema',
        json_schema: {
          name: 'nap_spot',
          strict: true,
          schema: {
            type: 'object',
            properties: { spot: { type: 'string', }, },
            required: ['spot',],
            additionalProperties: false,
          },
        },
      },
      validate: function isSpot(value: unknown,): value is { readonly spot: string; } {
        return ((typeof value) === 'object') && (value !== null) && ('spot' in value);
      },
    },),);
    if ((bodies.length - before) < 2)
      findings.push(`${row.provider}: ${String(bodies.length - before,)} bodies recorded`,);
    for (const bodyJson of bodies.slice(before,)) {
      /**
       Body as sent.
       */
      const body = JSON.parse(bodyJson,) as Record<string, unknown>;
      if (!(((typeof body.max_tokens) === 'number') && (body.max_tokens > 0)))
        findings.push(`${row.provider}: no max_tokens`,);
      for (const key of keysOf(body,).filter(function forbidden(found,): boolean {
        return FORBIDDEN_KEYS.has(found,);
      },))
        findings.push(`${row.provider}: ${key}`,);
    }
  }
  return findings;
}

await describe({
  name: 'the owner\'s standing body rules hold on every provider (ledger P11)',
  children: [
    it({
      name: 'EVERY BODY CARRIES max_tokens AND NO THINKING, BUDGET OR REASONING KNOB, on the text call and on '
        + 'the schema\'d call of each of the four providers',
      fn: async () => {
        expect(await bodyFindingsFor({ messages: TEXT_MESSAGES, },),).toEqual([],);
      },
    },),
    it({
      name: 'EVERY BODY CARRIES NO REASONING KNOB ON A PICTURE CALL EITHER, so the lift the owner gave NeuralWatt '
        + '(2026-10-21) reaches no other provider\'s request',
      fn: async () => {
        expect(await bodyFindingsFor({ messages: PICTURE_MESSAGES, },),).toEqual([],);
      },
    },),
    it({
      name: 'A SEAT DROPPED FROM OPENROUTER HAS NO OPENROUTER BLOCK AND NO OPENROUTER REACH, so re-adding a block '
        + 'fails here before a run routes the seat there',
      fn: async () => {
        expect(OPENROUTER_DROPPED_SEATS.size,).toBeGreaterThan(0,);
        for (const modelId of OPENROUTER_DROPPED_SEATS) {
          expect({
            modelId,
            block: MODEL_CARDS[modelId].openrouter !== undefined,
            reach: reachOf({ modelId, },).openrouter,
          },).toEqual({
            modelId,
            block: false,
            reach: false,
          },);
        }
      },
    },),
  ],
},);
