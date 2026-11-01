import type {
  ProviderName,
  ProviderRecord,
} from '@monochromatic-dev/translation-repair-shared-contract';
import type { BudgetView, } from '../dist/final/node/index.mjs';

//region Provider router wet budgets
// A BUDGET VIEW WHERE EVERY PROVIDER BUT SYNTHETIC READS WET, as in the run
// that found the pacing and stream-bound classes this file's siblings guard.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The provider-router-pace and
// provider-router-stream-bound tests kept their own copy of this stub; all
// now import it from here. `readOlderProvidersWet` is the sibling view for
// cases that pin the older providers' order with NeuralWatt out of the way.

/**
 Reads every provider but Synthetic as wet.

 @returns Budget view with only Synthetic dry

 @example
 ```ts
 const view = await readWetBudgets();
 ```
 */
function readWetBudgets(): Promise<BudgetView> {
  return Promise.resolve({
    neuralwatt: false,
    synthetic: true,
    hyper: false,
    bedrock: false,
    deepseek: false,
    xiaomi: false,
    anthropic: false,
    openrouter: false,
  },);
}

/**
 Reads NeuralWatt as dry and the four older providers as wet, so cases that
 pin the older providers' order never reach NeuralWatt; its own cases are in
 `neuralwatt-routing.unit.test.ts`.

 @returns Budget view with only NeuralWatt dry

 @example
 ```ts
 const view = await readOlderProvidersWet();
 ```
 */
export function readOlderProvidersWet(): Promise<BudgetView> {
  return Promise.resolve({
    neuralwatt: true,
    synthetic: false,
    hyper: false,
    bedrock: false,
    deepseek: false,
    xiaomi: false,
    anthropic: false,
    openrouter: false,
  },);
}

/**
 Marks nothing refused, since nothing refuses in these cases.

 @example
 ```ts
 await markNothingRefused();
 ```
 */
function markNothingRefused(): Promise<void> {
  return Promise.resolve();
}

/**
 Reports no provider holding a call.

 @returns Zero holds at every provider

 @example
 ```ts
 const held = holdNothing();
 ```
 */
export function holdNothing(): ProviderRecord<number> {
  return {
    neuralwatt: 0,
    synthetic: 0,
    hyper: 0,
    bedrock: 0,
    deepseek: 0,
    xiaomi: 0,
    anthropic: 0,
    openrouter: 0,
  };
}

/**
 Budget view where every provider but Synthetic reads wet, as in the run
 that found the class.

 @returns Budgets that never mark anything refused

 @example
 ```ts
 const budgets = stubWetBudgets();
 ```
 */
export function stubWetBudgets(): {
  readonly read: () => Promise<BudgetView>;
  readonly markRefused: (args: { readonly provider: ProviderName; },) => Promise<void>;
  readonly holds: () => ProviderRecord<number>;
} {
  return {
    read: readWetBudgets,
    markRefused: markNothingRefused,
    holds: holdNothing,
  };
}

//endregion Provider router wet budgets
