/**
 Tests that a picture reading sent to NeuralWatt's GLM-5.3-Flash carries the
 documented reasoning effort `low`, that nothing else does, and that no other
 provider's card can carry the field.

 WHY: on 2026-10-21 `glm-5.3-flash` read a dense screenshot at NeuralWatt's
 default effort `max` (reasoning is mandatory on this model, the listing's
 `default_effort` is `max`) and spent all 18316 completion tokens of its cap
 reasoning, with 0 content characters (XingZ6016). The owner, 2026-10-21:
 "NeuralWatt is a special case of all the providers: It implements what it
 documents correctly, almost always - I'm lifting the ban on passing params,
 but only for NeuralWatt." The documented parameter is `reasoning_effort`,
 and `low` is the shallowest level the model lists in `supported_efforts`
 (`max`, `high`, `low`; `https://docs.neuralwatt.com/api/chat-completions.md`,
 fetched 2026-10-21).

 WHAT THESE PIN: a picture call to the GLM seat sends `low`; a picture call to
 the DeepSeek seat sends nothing (its transcription was never measured); and
 only the NeuralWatt side of a card may name a picture effort, so no other
 provider's client can be handed one. The text calls of the same seat have
 their own file, `neuralwatt-text-effort.unit.test.ts` (ledger B807).

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
 A request carrying one picture part, as the reading stage builds it.
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

await describe({
  name: 'NeuralWatt picture reasoning effort',
  concurrency: 1,
  children: [
    it({
      name: 'SENDS reasoning_effort low ON A PICTURE CALL TO THE GLM SEAT, the shallowest level the listing '
        + 'supports, so the completion cap leaves room for the answer',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-effort-', },);
        const { client, exchanges, } = scriptedClient({ dir: scratch.path, },);
        await client.chatText({
          modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
          messages: PICTURE_MESSAGES,
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
      name: 'SENDS NO reasoning_effort ON A PICTURE CALL TO THE DEEPSEEK SEAT, whose transcription has '
        + 'never been measured on any stack',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-effort-', },);
        const { client, exchanges, } = scriptedClient({ dir: scratch.path, },);
        await client.chatText({
          modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
          messages: PICTURE_MESSAGES,
          signal: SIGNAL,
        },);
        const [post,] = chatExchangesOf({ exchanges, },);
        expect(Object.hasOwn(jsonRecordOf({ text: post?.bodyJson ?? '{}', },), 'reasoning_effort',),).toBe(false,);
      },
    },),

    it({
      name: 'LETS ONLY THE NEURALWATT SIDE OF A CARD NAME A PICTURE EFFORT, and names low on the GLM card, so '
        + 'no other provider\'s client has a field to send',
      fn: async () => {
        await Promise.resolve();
        /**
         Every side of every card that names a picture effort, as `card.side`.
         */
        const naming: string[] = [];
        for (const [modelId, card,] of Object.entries(MODEL_CARDS,)) {
          for (const [side, value,] of Object.entries(card,)) {
            if (((typeof value) === 'object') && (value !== null) && Object.hasOwn(value, 'pictureReasoningEffort',))
              naming.push(`${modelId}.${side}=${String((value as Record<string, unknown>).pictureReasoningEffort,)}`,);
          }
        }
        expect(naming,).toEqual(['hf:zai-org/GLM-5.3-Flash.neuralwatt=low',],);
      },
    },),
  ],
},);
