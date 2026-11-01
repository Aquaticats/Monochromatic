import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { BAKED_ENTRYPOINT, } from '../../dist/final/node/index.mjs';

await describe({
  name: 'container entrypoint',
  children: [
    it({
      name: 'points into the package directory copied into the runtime image',
      fn: async () => {
        expect(BAKED_ENTRYPOINT,).toBe('/baked/package/mutation-test/cli/src/container/main.ts',);
      },
    },),
  ],
},);
