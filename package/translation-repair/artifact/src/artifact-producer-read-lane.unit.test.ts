/**
 Tests for reading a lane producer back out of an artifact (class forty,
 2026-09-17).

 A consolidation slate may carry a lane text as a candidate, and a settled
 artifact records who produced each candidate; the reader must give the
 lane back as it was written and refuse a lane name the contest never had.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  requireProducer,
} from '../dist/final/node/index.mjs';
import { SEAT_HYPER_OPENROUTER_VISION_EDITOR, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

await describe({
  name: 'artifact producer read of a lane candidate (class forty, 2026-09-17)',
  children: [
    it({
      name: 'READS a lane producer back with its lane and the models that reproduced it',
      fn: async () => {
        expect(requireProducer({
          value: {
            kind: 'lane',
            lane: 'translate',
            matched: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
          },
          path: 'slate[1].producer',
        },),).toEqual({
          kind: 'lane',
          lane: 'translate',
          matched: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
        },);
      },
    },),

    it({
      name: 'REFUSES a lane name the contest never had',
      fn: async () => {
        expect(function read(): unknown {
          return requireProducer({
            value: {
              kind: 'lane',
              lane: 'archive',
              matched: [],
            },
            path: 'slate[1].producer',
          },);
        },).toThrow();
      },
    },),
  ],
},);
