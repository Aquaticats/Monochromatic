/**
 Holds the pool a re-ask of an editor or a refiner draws from to the chairs the editor calibration seats (ledger B1100).

 THE EDITOR CALIBRATION SEATS THE ROSTER'S WRITERS IN BOTH CHAIRS, then the run's own named editors or refiners that are
 not among them (`EDITOR_LANE_EDITORS`, `EDITOR_LANE_REFINERS`). The re-ask of a runaway editor or refiner asks only a
 model of the pool of its role (`rolePoolOf` in the shared contract). The two are derived in different packages, so
 these cases fail when the pool and the chair drift apart.

 NOTHING HERE REACHES A PROVIDER.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  MODEL_CARDS,
  rolePoolOf,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  EDITOR_LANE_EDITORS,
  EDITOR_LANE_REFINERS,
} from './editor-calibrate-lane.ts';

//region Editor lane pool tests

await describe({
  name: 'the pool of the editor and refiner roles',
  concurrency: 1,
  children: [
    it({
      name: 'IS THE CHAIR THE EDITOR CALIBRATION SEATS, but for the Claude route: the models a re-ask of an editor or a '
        + 'refiner may ask are the models the calibration measures in that chair',
      fn: async () => {
        for (
          const [role, chair,] of [
            ['editor', EDITOR_LANE_EDITORS,],
            ['refiner', EDITOR_LANE_REFINERS,],
          ] as const
        ) {
          expect(rolePoolOf({ role, },).filter(function isNotClaude(modelId,): boolean {
            return MODEL_CARDS[modelId].anthropic === undefined;
          },),).toEqual(chair.filter(function isNotClaude(modelId,): boolean {
            return MODEL_CARDS[modelId].anthropic === undefined;
          },),);
        }
      },
    },),
  ],
},);

//endregion Editor lane pool tests
