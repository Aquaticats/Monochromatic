import type { ProviderRecord, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { MeterState, } from '../dist/final/node/index.mjs';

//region Meter states fixture
// TEST SUPPORT, NOT PACKAGE SOURCE. What a scripted run client answers when a
// case does not care what the meters said: every meter answered wet, so a
// seat reading names every provider as the routed view does.

/**
 What every meter says in a scripted run client: all answered wet.

 @returns Every meter wet

 @example
 ```ts
 const client = { providerMeterStates: metersAllWet, };
 ```
 */
export function metersAllWet(): Promise<ProviderRecord<MeterState>> {
  return Promise.resolve({
    neuralwatt: 'wet',
    synthetic: 'wet',
    bedrock: 'wet',
    deepseek: 'wet',
    xiaomi: 'wet',
    anthropic: 'wet',
    hyper: 'wet',
    openrouter: 'wet',
  },);
}

//endregion Meter states fixture
