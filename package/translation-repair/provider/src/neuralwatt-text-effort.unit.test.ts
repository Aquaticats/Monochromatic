/**
 Tests that a text call sent to NeuralWatt's GLM-5.3-Flash carries the
 documented reasoning effort `low`, that no other NeuralWatt call carries
 one on its text side, and that no other provider's card can name one.

 WHY: `glm-5.3-flash` is a reasoning-only model on NeuralWatt whose listing
 gives `default_effort` `max`. A text call that sent no effort ran at `max`
 and spent all 18316 completion tokens of its cap reasoning, with 0 content
 characters, 11 times in 288 text calls over the pass logs from `zheermao19`
 to `zheermao22` (4 of them in `zheermao22`, 29 percent of that run's
 NeuralWatt energy). The owner, 2026-10-21: "NeuralWatt is a special case of
 all the providers: It implements what it documents correctly, almost
 always - I'm lifting the ban on passing params, but only for NeuralWatt."
 The lift was written for pictures only (ledger B719) and covers text calls
 as well.

 WHAT THESE PIN: a text call to the GLM seat sends `low`; a text call to the
 DeepSeek seat sends nothing (the repo records no `supported_efforts` list
 for it, and its default `high` capped on 12 of 819 text calls); the
 effort is one of the three the listing documents; and only the NeuralWatt
 side of a card may name a text effort, so no other provider's client has a
 field to send.

 Fixtures are cat-themed invention. No call reaches NeuralWatt.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { MODEL_CARDS, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  chatExchangesOf,
  scriptedClient,
  SIGNAL,
} from './neuralwatt-scripted-client.test-fixture.ts';
import { jsonRecordOf, } from './neuralwatt-ledger.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Efforts NeuralWatt's listing documents for `glm-5.3-flash`.
 */
const DOCUMENTED_EFFORTS: readonly string[] = ['max', 'high', 'low',];

await describe({
  name: 'NeuralWatt text reasoning effort',
  concurrency: 1,
  children: [
    it({
      name: 'SENDS reasoning_effort low ON A TEXT CALL TO THE GLM SEAT, so a heavy block review cannot spend its '
        + 'whole completion cap reasoning at the provider default max',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-text-effort-', },);
        const { client, exchanges, } = scriptedClient({ dir: scratch.path, },);
        await client.chatText({
          modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
          messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
          signal: SIGNAL,
        },);
        const [post,] = chatExchangesOf({ exchanges, },);
        const body = jsonRecordOf({ text: post?.bodyJson ?? '{}', },);
        expect(body.reasoning_effort,).toBe('low',);
        for (const absent of ['reasoning', 'thinking', 'thinking_token_budget', 'temperature', 'store',])
          expect(Object.hasOwn(body, absent,),).toBe(false,);
      },
    },),

    it({
      name: 'SENDS reasoning_effort low ON A SCHEMA CALL TO THE GLM SEAT as well, since the effort does not depend '
        + 'on the reply shape',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-text-effort-', },);
        const { client, exchanges, } = scriptedClient({ dir: scratch.path, },);
        await client.chatText({
          modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
          messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
          signal: SIGNAL,
          responseFormat: {
            type: 'json_schema',
            json_schema: {
              name: 'spot',
              strict: true,
              schema: { type: 'object', properties: { spot: { type: 'string', }, }, required: ['spot',], },
            },
          },
        },);
        const [post,] = chatExchangesOf({ exchanges, },);
        expect(jsonRecordOf({ text: post?.bodyJson ?? '{}', },).reasoning_effort,).toBe('low',);
      },
    },),

    it({
      name: 'SENDS NO reasoning_effort ON A TEXT CALL TO THE DEEPSEEK SEAT, whose supported efforts the repo does '
        + 'not record',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-text-effort-', },);
        const { client, exchanges, } = scriptedClient({ dir: scratch.path, },);
        await client.chatText({
          modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
          messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
          signal: SIGNAL,
        },);
        const [post,] = chatExchangesOf({ exchanges, },);
        expect(Object.hasOwn(jsonRecordOf({ text: post?.bodyJson ?? '{}', },), 'reasoning_effort',),).toBe(false,);
      },
    },),

    it({
      name: 'LETS ONLY THE NEURALWATT SIDE OF A CARD NAME A TEXT EFFORT, names low on the GLM card alone, and '
        + 'names a level the listing documents',
      fn: async () => {
        await Promise.resolve();
        /**
         Every side of every card that names a text effort, as `card.side=effort`.
         */
        const naming: string[] = [];
        for (const [modelId, card,] of Object.entries(MODEL_CARDS,)) {
          for (const [side, value,] of Object.entries(card,)) {
            if (((typeof value) === 'object') && (value !== null) && Object.hasOwn(value, 'textReasoningEffort',)) {
              /**
               The effort this side names.
               */
              const effort = String((value as Record<string, unknown>).textReasoningEffort,);
              expect(DOCUMENTED_EFFORTS,).toContain(effort,);
              naming.push(`${modelId}.${side}=${effort}`,);
            }
          }
        }
        expect(naming,).toEqual(['hf:zai-org/GLM-5.3-Flash.neuralwatt=low',],);
      },
    },),
  ],
},);
