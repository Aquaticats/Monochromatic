/**
 Tests for the author of the answer of a seat (ledger B1142).

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  answererOf,
  heardVoiceOf,
} from '../dist/final/node/index.mjs';

//region Stage answerer tests

await describe({
  name: `${answererOf.name} and ${heardVoiceOf.name}`,
  concurrency: 1,
  children: [
    it({
      name: 'NAMES THE SEAT AS THE AUTHOR WHERE THE SEAT\'S OWN MODEL ANSWERED, and for a voice that was lost',
      fn: async () => {
        expect(answererOf({
          outcome: { modelId: 'glm-5.3', voice: { heard: true, value: 'purr', }, },
        },),).toBe('glm-5.3',);
        expect(answererOf({
          outcome: { modelId: 'glm-5.3', voice: { heard: false, answered: false, unreachable: false, }, },
        },),).toBe('glm-5.3',);
      },
    },),

    it({
      name: 'NAMES THE MODEL THAT ANSWERED IN PLACE OF A RUNAWAY SEAT as the author',
      fn: async () => {
        expect(answererOf({
          outcome: { modelId: 'glm-5.3', voice: { heard: true, value: 'purr', answeredBy: 'minimax-m3', }, },
        },),).toBe('minimax-m3',);
      },
    },),

    it({
      name: 'MAKES A VOICE OF THE AUTHOR WITH NO SEAT where the seat answered, and logs nothing',
      fn: async () => {
        const lines: string[] = [];
        expect(heardVoiceOf({
          outcome: { modelId: 'glm-5.3', voice: { heard: true, value: 'purr', }, },
          stage: 'panel',
          l: levelCapturingLogger({ lines, },),
        },),).toEqual({ modelId: 'glm-5.3', value: 'purr', },);
        expect(lines,).toEqual([],);
      },
    },),

    it({
      name: 'MAKES A VOICE OF THE AUTHOR BESIDE THE SEAT where another model answered, and says so in one line naming both',
      fn: async () => {
        const lines: string[] = [];
        expect(heardVoiceOf({
          outcome: { modelId: 'glm-5.3', voice: { heard: true, value: 'purr', answeredBy: 'minimax-m3', }, },
          stage: 'panel',
          l: levelCapturingLogger({ lines, },),
        },),).toEqual({ modelId: 'minimax-m3', seat: 'glm-5.3', value: 'purr', },);
        expect(lines,).toEqual([
          'info panel: glm-5.3: the answer was written by minimax-m3, which the re-ask of a runaway asked in its place',
        ],);
      },
    },),

    it({
      name: 'REFUSES AN OUTCOME THAT IS NO HEARD VOICE, which the caller has ruled out',
      fn: async () => {
        expect(function readsALostVoice() {
          heardVoiceOf({
            outcome: { modelId: 'glm-5.3', voice: { heard: false, answered: false, unreachable: false, }, },
            stage: 'panel',
            l: levelCapturingLogger({ lines: [], },),
          },);
        },).toThrow('glm-5.3 is no heard voice',);
      },
    },),
  ],
},);

//endregion Stage answerer tests
