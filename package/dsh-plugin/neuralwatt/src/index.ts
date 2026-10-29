/**
 NeuralWatt provider plugin entry: keeps the `neuralwatt` provider route
 written into the shipped pi-ai adapter's settings namespace, serving a
 checked-in model snapshot at mount and landing the live NeuralWatt listing
 once the mount fetch resolves. @module
 */

import type {
  Context,
} from '@deepseek-ai/cordis';
import type {
  SettingsForms,
} from '@deepseek-ai/dsh-settings';
import {
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import {
  neuralWattLogger,
} from './logging.ts';
import {
  refreshNeuralWattRoute,
} from './providers.ts';

export * from './constants.ts';
export * from './fetch-listing.ts';
export * from './listing.ts';
export * from './logging.ts';
export * from './providers.ts';
export * from './snapshot.ts';

//region Plugin contract

/**
 Host services the plugin reads at activation: the settings surface whose
 `llm-pi-ai` entry the route is written through.
 */
export const inject: readonly string[] = ['settings'];

/**
 Activate the NeuralWatt provider: write the checked-in snapshot route into
 the shipped adapter's settings namespace immediately, then land the live
 listing once the mount fetch resolves.

 @param ctx - loader-scoped registration context

 @example
 ```ts
 apply(ctx);
 ```
 */
export function apply(ctx: Context): void {
  /**
   Logger tagged with the plugin entry name.
   */
  const logger = tagged({
    tag: apply.name,
    l: neuralWattLogger,
  },);
  /**
   Settings surface the route is written through; `inject` guarantees the
   service is present when this runs.
   */
  const settings: SettingsForms = ctx.settings;
  logger.debug('neuralwatt route feed applied over the llm-pi-ai settings entry');
  /**
   Fire-and-forget mount refresh: bounded, and its failure branches log and
   keep the served snapshot instead of throwing.
   */
  void refreshNeuralWattRoute({ settings, },);
}

//endregion Plugin contract
