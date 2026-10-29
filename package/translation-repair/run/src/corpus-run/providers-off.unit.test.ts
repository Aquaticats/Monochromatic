/**
 Tests the providers the owner left off.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isProviderOff,
  PROVIDERS_LEFT_OFF,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'providers left off',
  children: [
    describe({
      name: 'PROVIDERS_LEFT_OFF',
      children: [
        it({
          name: 'NAMES BEDROCK WITH THE OWNER\'S RULING OF 2026-10-20 and nothing else, so no other provider is switched '
            + 'off by default',
          fn: async () => {
            expect(PROVIDERS_LEFT_OFF.map(function providerOf({ provider, },): string {
              return provider;
            },),).toEqual(['bedrock',],);
            expect(PROVIDERS_LEFT_OFF[0]?.reason,).toContain('2026-10-20',);
            expect(PROVIDERS_LEFT_OFF[0]?.reason,).toContain('google.gemma-4-e2b',);
          },
        },),
      ],
    },),

    describe({
      name: isProviderOff.name,
      children: [
        it({
          name: 'IS TRUE FOR A PROVIDER IN THE LIST AND FALSE FOR EVERY OTHER, and false for any provider when the '
            + 'list is empty',
          fn: async () => {
            expect(isProviderOff({ off: PROVIDERS_LEFT_OFF, provider: 'bedrock', },),).toBe(true,);
            expect(isProviderOff({ off: PROVIDERS_LEFT_OFF, provider: 'hyper', },),).toBe(false,);
            expect(isProviderOff({ off: [], provider: 'bedrock', },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
