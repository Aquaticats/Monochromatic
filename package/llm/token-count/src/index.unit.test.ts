import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { DEFAULT_MODEL, } from '@monochromatic-dev/llm-token-count';

await describe({
  name: 'token-count lib (built bundle smoke test)',
  children: [
    //region Re-export integrity: importing a pure constant by-name executes the built index.mjs without any API call

    it({
      name: 'exposes DEFAULT_MODEL as a Claude model id',
      fn: async () => {
        expect(typeof DEFAULT_MODEL,).toBe('string',);
        expect(DEFAULT_MODEL,).toContain('claude',);
      },
    },),

    //endregion Re-export integrity
  ],
},);
