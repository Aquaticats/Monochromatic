/**
 Tests for the `pMapStop` sentinel itself. The behavior of a stop inside a map
 lives in `p-map.stop.unit.test.ts`.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  pMapSkip,
  pMapStop,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: 'pMapStop sentinel',
  children: [
    it({
      name: 'carries a descriptive symbol description for readable traces',
      fn: async () => {
        expect(pMapStop.description,).toBe('p-map mapper stop sentinel',);
      },
    },),

    it({
      name: 'is a symbol distinct from pMapSkip',
      fn: async () => {
        expect(typeof pMapStop,).toBe('symbol',);
        expect(pMapStop,).not.toBe(pMapSkip,);
      },
    },),
  ],
},);
