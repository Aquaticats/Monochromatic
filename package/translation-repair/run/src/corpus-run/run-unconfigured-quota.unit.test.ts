/**
 Tests the Synthetic quota a run reads when Synthetic is not configured.

 The router receives an absent Synthetic as dry directly, so this reading exists only to keep the client's surface
 stable for observational callers; what it must never do is read as a budget with room left.

 Fixtures are cat-themed invention; no meter is read and no HTTP is made.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { unconfiguredSyntheticQuota, } from '../../dist/final/node/index.mjs';

await describe({
  name: unconfiguredSyntheticQuota.name,
  concurrency: 1,
  children: [
    it({
      name: 'READS AS EXHAUSTED IN BOTH WINDOWS, the five-hour one limited and the weekly one empty, with no instant '
        + 'named for either to return',
      fn: async () => {
        expect(await unconfiguredSyntheticQuota(),).toEqual({
          fiveHour: {
            remaining: 0,
            max: 0,
            limited: true,
            nextTickAt: '',
          },
          weekly: {
            percentRemaining: 0,
            nextRegenAt: '',
          },
        },);
      },
    },),
  ],
},);
