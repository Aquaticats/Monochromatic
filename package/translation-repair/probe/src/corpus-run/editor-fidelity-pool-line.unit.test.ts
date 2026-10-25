/**
 Tests for the line the editor fidelity probe prints to say what it drew from the frozen pool: how many trials, over
 how many entries, in which defect families.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  EDITOR_DAMAGE_KINDS,
  editorFidelityTrials,
  poolLine,
} from '../../dist/final/node/index.mjs';
import { inventedReference, } from '../editor-fidelity.test-fixture.ts';

await describe({
  name: poolLine.name,
  children: [
    it({
      name: 'SAYS HOW MANY TRIALS A RUN DREW, over how many entries and in which families, so the pool each arm saw '
        + 'can be read off its log',
      fn: async () => {
        /**
         The two trials of the invented reference.
         */
        const trials = editorFidelityTrials({
          references: [inventedReference(),],
          damageKinds: EDITOR_DAMAGE_KINDS,
        },);
        expect(poolLine({ trials, },),).toBe('pool: 2 trials over 1 entry (insertion 1, alteration 1)',);
        expect(poolLine({
          trials: [nonNullishOrThrow(trials[0],),],
        },),).toBe('pool: 1 trial over 1 entry (insertion 1, alteration 0)',);
      },
    },),

    it({
      name: 'COUNTS ENTRIES, NOT REFERENCES: two trials of one entry are one entry, and trials of two entries are two',
      fn: async () => {
        /**
         The invented reference's two trials.
         */
        const [insertion, alteration,] = editorFidelityTrials({
          references: [inventedReference(),],
          damageKinds: EDITOR_DAMAGE_KINDS,
        },);
        /**
         A trial of another entry, the invented one renamed.
         */
        const elsewhere = {
          ...nonNullishOrThrow(alteration,),
          spec: {
            ...nonNullishOrThrow(alteration,).spec,
            entryId: 'ginger',
          },
        };
        expect(poolLine({
          trials: [nonNullishOrThrow(insertion,), elsewhere,],
        },),).toBe('pool: 2 trials over 2 entries (insertion 1, alteration 1)',);
      },
    },),
  ],
},);
