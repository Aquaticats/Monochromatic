/**
 Tests the reading cluster's refusal (ledger T8): fewer than two readings
 leave no pair to compare, which the cluster refuses rather than answering
 with nothing vouched. Its one caller in the pipeline stops before this with
 fewer than two, so the refusal guards the contract, and this case holds it.
 Readings are invented.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  clusterReadings,
  ReadingClusterError,
} from '../dist/final/node/index.mjs';

await describe({
  name: clusterReadings.name,
  children: [
    it({
      name: 'REFUSES ONE READING OR NONE, which leave no pair to compare',
      fn: async () => {
        expect(() => clusterReadings({ readings: [{ text: 'a cat asleep on a sill', },], },),).toThrow(ReadingClusterError,);
        expect(() => clusterReadings({ readings: [], },),).toThrow(ReadingClusterError,);
      },
    },),
  ],
},);
