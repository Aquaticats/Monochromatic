/**
 Tests where one roster model can be reached on NeuralWatt: the two models the
 owner's free daily energy covers, `deepseek-v4.1-flash` and `glm-5.3-flash`
 (`GET https://api.neuralwatt.com/v1/models`, public, read 2026-10-20), spelled
 the way that provider spells them, and whether it shows them a picture.

 Roster models come from the shared seats, so a model leaving the roster
 changes one line there and not this file. Both served models list image
 input today, so the picture case holds the answer to the catalog's own flag
 rather than to a spelled-out boolean.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  NEURALWATT_MODELS,
  neuralWattIdFor,
  neuralWattShowsPictures,
} from '../dist/final/node/index.mjs';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: neuralWattIdFor.name,
      children: [
        it({
          name: 'SPELLS EACH SERVED SEAT THE WAY NEURALWATT SPELLS IT, which differs from the roster\'s id for a model '
            + 'the other providers spell with a prefix',
          fn: async () => {
            expect(neuralWattIdFor({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, },),).toEqual({
              served: true,
              id: 'deepseek-v4.1-flash',
            },);
            expect(neuralWattIdFor({ modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR, },),).toEqual({
              served: true,
              id: 'glm-5.3-flash',
            },);
          },
        },),

        it({
          name: 'SAYS A SEAT THIS PROVIDER HAS NO SIDE FOR IS NOT SERVED, with no identifier to send',
          fn: async () => {
            expect(neuralWattIdFor({ modelId: SEAT_BEDROCK_ONLY_TEXT, },),).toEqual({ served: false, },);
          },
        },),
      ],
    },),

    describe({
      name: neuralWattShowsPictures.name,
      children: [
        it({
          name: 'SHOWS A SERVED MODEL A PICTURE WHEN ITS LISTING REPORTS IMAGE INPUT, read off the catalog row of the '
            + 'identifier it is spelled with',
          fn: async () => {
            for (const modelId of [SEAT_HYPER_OPENROUTER_UNMEASURED, SEAT_HYPER_OPENROUTER_VISION_EDITOR,]) {
              /**
               How NeuralWatt spells this seat.
               */
              const spelling = neuralWattIdFor({ modelId, },);
              if (!spelling.served)
                throw new Error('the seat is served and should have been spelled',);
              expect(neuralWattShowsPictures({ modelId, },),).toBe(NEURALWATT_MODELS[spelling.id].readsImages,);
              expect(neuralWattShowsPictures({ modelId, },),).toBe(true,);
            }
          },
        },),

        it({
          name: 'SHOWS A MODEL THIS PROVIDER DOES NOT SERVE NO PICTURE, whatever the model reads elsewhere',
          fn: async () => {
            expect(neuralWattShowsPictures({ modelId: SEAT_BEDROCK_ONLY_TEXT, },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
