import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { generateAllowedIps, } from '../dist/final/node/generate.mjs';

await describe({
  name: generateAllowedIps.name,
  children: [
    it({
      name: 'uses the production boundary for address-only input',
      fn: async () => {
        /**
         Output from public built package entry.
         */
        const output = await generateAllowedIps({
          allowedText: '10.0.0.0/8\n2001:db8::/126',
          disallowedText: '10.0.0.0/9\n2001:db8::/127',
        },);
        expect(output,).toBe('10.128.0.0/9, 2001:db8::2/127\n',);
      },
    },),
  ],
},);
