/**
 Tests which names count against the free energy beyond the covered models' own: the tracking aliases
 (`https://docs.neuralwatt.com/guides/tracking-aliases`, fetched 2026-10-20), such as `nw-flash`, which point at one
 model and move to another, so `GET /v1/models` (`metadata.huggingface_id`) is read for what each points at and no
 table of them is kept. A usage row for a call made with an alias is "listed under the model that served it" and
 carries the alias as its `requested_model`.

 Fixtures are the 2026-10-20 listing's names and cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  aliasNamesOf,
  coveredWhOf,
} from '../dist/final/node/index.mjs';
import {
  DEEPSEEK_LISTED_AS,
  KIMI_LISTED_AS,
  LISTED_MODELS,
  modelEntryOf,
  modelsReply,
} from './neuralwatt-models-listing.test-fixture.ts';
import { usageRowOf, } from './neuralwatt-scripted-client.test-fixture.ts';

/**
 The window the cases ask for.
 */
const ASKED = {
  start: new Date('2026-10-20T19:15:08.000Z',),
  end: new Date('2026-10-20T23:15:00.741Z',),
};

/**
 The period a reply echoes for that window.
 */
const ECHO = {
  start: '2026-10-20T19:15:08+00:00',
  end: '2026-10-20T23:15:00.741000+00:00',
};

/**
 A listing body for the given entries.

 @param entries - listing

 @returns Body text
 */
function listingOf({ entries, }: { readonly entries: Parameters<typeof modelsReply>[0]['entries']; },): string {
  return modelsReply({ entries, },).bodyText;
}

/**
 A by-model body for the given rows.

 @param products - rows

 @returns Body text
 */
function usageOf({ products, }: { readonly products: readonly unknown[]; },): string {
  return JSON.stringify({ period: ECHO, models: [], products, },);
}

await describe({
  name: 'NeuralWatt tracking aliases',
  concurrency: 1,
  children: [
    describe({
      name: aliasNamesOf.name,
      children: [
        it({
          name: 'NAMES THE ALIASES AND FLEX TWINS THAT POINT AT A COVERED MODEL, and no other name',
          fn: async () => {
            expect(aliasNamesOf({ bodyText: listingOf({ entries: LISTED_MODELS, }), },),).toEqual({
              known: true,
              names: new Set(['nw-flash', 'nw-flash-flex',],),
            },);
          },
        },),
        it({
          name: 'LEAVES OUT A SPEED LANE however it points, and ADDS THE FLEX TWIN of an alias the listing gives none',
          fn: async () => {
            expect(aliasNamesOf({
              bodyText: listingOf({
                entries: [
                  modelEntryOf({ id: 'deepseek-v4.1-flash', listedAs: DEEPSEEK_LISTED_AS, },),
                  modelEntryOf({ id: 'glm-5.3-flash', listedAs: 'zai-org/GLM-5.3-Flash', },),
                  modelEntryOf({ id: 'nw-flash-speed', listedAs: DEEPSEEK_LISTED_AS, },),
                  modelEntryOf({ id: 'deepseek-v4.1-flash-speed', listedAs: DEEPSEEK_LISTED_AS, },),
                  modelEntryOf({ id: 'nw-glm', listedAs: 'zai-org/GLM-5.3-Flash', },),
                ],
              }),
            },),).toEqual({
              known: true,
              names: new Set(['nw-glm', 'nw-glm-flex',],),
            },);
          },
        },),
        it({
          name: 'FOLLOWS AN ALIAS WHEN THE LISTING MOVES IT: nw-flash pointing at another model is no longer counted',
          fn: async () => {
            expect(aliasNamesOf({
              bodyText: listingOf({
                entries: [
                  modelEntryOf({ id: 'deepseek-v4.1-flash', listedAs: DEEPSEEK_LISTED_AS, },),
                  modelEntryOf({ id: 'glm-5.3-flash', listedAs: 'zai-org/GLM-5.3-Flash', },),
                  modelEntryOf({ id: 'nw-flash', listedAs: KIMI_LISTED_AS, },),
                  modelEntryOf({ id: 'nw-large', listedAs: 'zai-org/GLM-5.3-Flash', },),
                ],
              }),
            },),).toEqual({
              known: true,
              names: new Set(['nw-large', 'nw-large-flex',],),
            },);
          },
        },),
        it({
          name: 'SAYS WHY A LISTING GIVES NO MAP: not JSON, no list, entries with no pointer, or no covered model in it',
          fn: async () => {
            /**
             Each unreadable body and the phrase naming the cause.
             */
            const unreadable: readonly (readonly [string, string])[] = [
              ['not json', 'not valid JSON',],
              ['[]', 'not a JSON object',],
              [JSON.stringify({ object: 'list', },), 'no data list',],
              [listingOf({ entries: [{ id: 'nw-flash', },], }), 'lists neither covered model with a huggingface_id',],
              [
                listingOf({ entries: [modelEntryOf({ id: 'nw-flash', listedAs: DEEPSEEK_LISTED_AS, },),], }),
                'lists neither covered model with a huggingface_id',
              ],
            ];
            for (const [bodyText, phrase,] of unreadable) {
              /**
               What the reader made of this body.
               */
              const map = aliasNamesOf({ bodyText, },);
              expect(map.known,).toBe(false,);
              expect(map.known ? '' : map.why,).toContain(phrase,);
            }
          },
        },),
      ],
    },),
    describe({
      name: `${coveredWhOf.name} with aliases`,
      children: [
        it({
          name: 'COUNTS THE ROWS OF THE NAMED ALIASES with the covered models\', and not the speed lane or the large alias',
          fn: async () => {
            expect(coveredWhOf({
              bodyText: usageOf({
                products: [
                  usageRowOf({ requestedModel: 'deepseek-v4.1-flash', kwh: 0.001, },),
                  usageRowOf({ requestedModel: 'nw-flash', kwh: 0.002, },),
                  usageRowOf({ requestedModel: 'nw-flash-flex', kwh: 0.0005, },),
                  usageRowOf({ requestedModel: 'nw-flash-speed', kwh: 0.2, },),
                  usageRowOf({ requestedModel: 'nw-large', kwh: 0.1, },),
                ],
              }),
              asked: ASKED,
              aliases: { known: true, names: new Set(['nw-flash', 'nw-flash-flex',],), },
            },),).toBeCloseTo(3.5, 9,);
          },
        },),
        it({
          name: 'WITH NO MAP COUNTS EVERY nw- ROW A COVERED MODEL SERVED, by the served name, and not a speed lane, '
            + 'a model that served another name, or a name without the prefix',
          fn: async () => {
            expect(coveredWhOf({
              bodyText: usageOf({
                products: [
                  usageRowOf({ requestedModel: 'nw-flash', kwh: 0.002, servedModelId: 'deepseek-ai/DeepSeek-V4.1-Flash', },),
                  usageRowOf({ requestedModel: 'nw-flash-flex', kwh: 0.0005, servedModelId: 'deepseek-ai/DeepSeek-V4.1-Flash', },),
                  usageRowOf({ requestedModel: 'nw-glm', kwh: 0.001, servedModelId: 'zai-org/GLM-5.3-Flash', },),
                  usageRowOf({ requestedModel: 'nw-flash-speed', kwh: 0.2, servedModelId: 'deepseek-ai/DeepSeek-V4.1-Flash', },),
                  usageRowOf({ requestedModel: 'nw-large', kwh: 0.1, servedModelId: 'moonshotai/Kimi-K3', },),
                  usageRowOf({ requestedModel: 'cat-flash', kwh: 0.1, servedModelId: 'deepseek-ai/DeepSeek-V4.1-Flash', },),
                ],
              }),
              asked: ASKED,
              aliases: { known: false, why: 'the listing could not be read', },
            },),).toBeCloseTo(3.5, 9,);
          },
        },),
      ],
    },),
  ],
},);
