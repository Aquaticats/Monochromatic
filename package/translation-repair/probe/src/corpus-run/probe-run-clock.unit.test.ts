/**
 Tests for the instant a probe run stamps on its record.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { nowAsIso, } from '../../dist/final/node/index.mjs';

await describe({
  name: nowAsIso.name,
  children: [
    it({
      name: 'READS an ISO 8601 instant between the instants read before and after it',
      fn: async () => {
        /**
         Instant read before the call.
         */
        const before = new Date().toISOString();

        /**
         Instant the function read.
         */
        const read = nowAsIso();

        /**
         Instant read after the call.
         */
        const after = new Date().toISOString();

        expect([
          before <= read,
          read <= after,
          new Date(read,).toISOString() === read,
        ],).toEqual([
          true,
          true,
          true,
        ],);
      },
    },),
  ],
},);
