/**
 Tests for the provider identity: the order the owner prefers to spend in,
 and the record helpers every provider-keyed shape is built from.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  isProviderName,
  otherProviders,
  PROVIDER_ORDER,
  providerRecord,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'PROVIDER_ORDER',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SPENDS ON NEURALWATT, THEN SYNTHETIC, THEN BEDROCK, THEN DEEPSEEK, THEN XIAOMI, THEN ANTHROPIC, '
            + 'THEN HYPER, THEN OPENROUTER, which is the owner\'s order of 2026-09-03 with the 2026-09-07 insertion, '
            + 'the 2026-10-20 one, the 2026-10-22 pair and the 2026-10-22 Claude key, the last two open to the '
            + 'owner\'s veto: the free daily energy that '
            + 'does not carry over first ("Use the neuralwatt free energy front and center."), the '
            + 'subscription next, the expiring credits that will never be topped up after it, the prepaid '
            + 'DeepSeek balance given to use "as you wish", the per-token Xiaomi route that serves only the '
            + 'seat OpenRouter otherwise serves, the balance that will not be topped up after those, and the '
            + 'per-token marketplace last',
          fn: async () => {
            expect(PROVIDER_ORDER,).toEqual([
              'neuralwatt',
              'synthetic',
              'bedrock',
              'deepseek',
              'xiaomi',
              'anthropic',
              'hyper',
              'openrouter',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: providerRecord.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FILLS every provider from one function, in order, so a shape keyed by provider cannot '
            + 'be built with one missing',
          fn: async () => {
            /**
             Positions each provider was asked in.
             */
            const asked: string[] = [];
            expect(providerRecord({
              of: function position(provider,): number {
                asked.push(provider,);
                return asked.length;
              },
            },),).toEqual({
              neuralwatt: 1,
              synthetic: 2,
              bedrock: 3,
              deepseek: 4,
              xiaomi: 5,
              anthropic: 6,
              hyper: 7,
              openrouter: 8,
            },);
            expect(asked,).toEqual(PROVIDER_ORDER,);
          },
        },),
      ],
    },),

    describe({
      name: otherProviders.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES the others in spending order, whichever one is left out',
          fn: async () => {
            expect(otherProviders({ provider: 'hyper', },),).toEqual([
              'neuralwatt',
              'synthetic',
              'bedrock',
              'deepseek',
              'xiaomi',
              'anthropic',
              'openrouter',
            ],);
            expect(otherProviders({ provider: 'synthetic', },),).toEqual([
              'neuralwatt',
              'bedrock',
              'deepseek',
              'xiaomi',
              'anthropic',
              'hyper',
              'openrouter',
            ],);
            expect(otherProviders({ provider: 'bedrock', },),).toEqual([
              'neuralwatt',
              'synthetic',
              'deepseek',
              'xiaomi',
              'anthropic',
              'hyper',
              'openrouter',
            ],);
            expect(otherProviders({ provider: 'deepseek', },),).toEqual([
              'neuralwatt',
              'synthetic',
              'bedrock',
              'xiaomi',
              'anthropic',
              'hyper',
              'openrouter',
            ],);
            expect(otherProviders({ provider: 'xiaomi', },),).toEqual([
              'neuralwatt',
              'synthetic',
              'bedrock',
              'deepseek',
              'anthropic',
              'hyper',
              'openrouter',
            ],);
            expect(otherProviders({ provider: 'openrouter', },),).toEqual([
              'neuralwatt',
              'synthetic',
              'bedrock',
              'deepseek',
              'xiaomi',
              'anthropic',
              'hyper',
            ],);
            expect(otherProviders({ provider: 'anthropic', },),).toEqual([
              'neuralwatt',
              'synthetic',
              'bedrock',
              'deepseek',
              'xiaomi',
              'hyper',
              'openrouter',
            ],);
            expect(otherProviders({ provider: 'neuralwatt', },),).toEqual([
              'synthetic',
              'bedrock',
              'deepseek',
              'xiaomi',
              'anthropic',
              'hyper',
              'openrouter',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: isProviderName.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ADMITS every provider name and nothing else, since a flag value reaches this unchecked',
          fn: async () => {
            expect(PROVIDER_ORDER.every(function admitted(provider,): boolean {
              return isProviderName(provider,);
            },),).toBe(true,);
            expect(isProviderName('neuralwatt',),).toBe(true,);
            expect(isProviderName('deepseek',),).toBe(true,);
            expect(isProviderName('xiaomi',),).toBe(true,);
            expect(isProviderName('anthropic',),).toBe(true,);
            expect(isProviderName('claude',),).toBe(false,);
            expect(isProviderName('',),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
