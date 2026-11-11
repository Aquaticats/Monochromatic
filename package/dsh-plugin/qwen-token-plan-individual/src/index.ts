/**
 Qwen Token Plan Individual provider plugin entry: keeps the
 `qwen-token-plan-individual` provider route written into the shipped pi-ai
 adapter's settings namespace, serving a checked-in model snapshot at mount
 and landing the live Qwen Token Plan listing once the mount fetch resolves. @module
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
import type {
  QwenTokenPlanIndividualCredentialsLookupResult,
} from './fetch-listing.ts';
import {
  qwenTokenPlanIndividualLogger,
} from './logging.ts';
import {
  refreshQwenTokenPlanIndividualRoute,
} from './providers.ts';

export * from './constants.ts';
export * from './fetch-listing.ts';
export * from './listing.ts';
export * from './logging.ts';
export * from './model-profile.ts';
export * from './providers.ts';
export * from './snapshot.ts';

//region Plugin contract

/**
 Host services the plugin reads at activation: the settings surface whose
 `llm-pi-ai` entry the route is written through. The credentials service is
 looked up lazily through `ctx.get` when the listing fetch runs, so a
 composition without one keeps the served snapshot instead of refusing to
 load, and one the loader registers after this plugin's `apply` is still
 seen.
 */
export const inject: readonly string[] = ['settings'];

/**
 Activate the Qwen Token Plan Individual provider: write the checked-in
 snapshot route into the shipped adapter's settings namespace immediately,
 then land the live listing once the mount fetch resolves.

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
    l: qwenTokenPlanIndividualLogger,
  });
  /**
   Settings surface the route is written through; `inject` guarantees the
   service is present when this runs.
   */
  const settings: SettingsForms = ctx.settings;
  logger
    .debug('qwen-token-plan-individual route feed applied over the llm-pi-ai settings entry');
  /**
   Fire-and-forget mount refresh: bounded, and its failure branches log and
   keep the served snapshot instead of throwing. The credentials lookup is a
   thunk, not a capture, because the loader can register the credentials
   service after this `apply` runs; the refresh calls it when the listing
   fetch is about to run.
   */
  void refreshQwenTokenPlanIndividualRoute({
    settings,
    getCredentials: function getCredentials(): QwenTokenPlanIndividualCredentialsLookupResult {
      /**
       Credentials service mounted at lookup time; absent here reads as "none
       yet", which the refresh outlasts within its mount-race budget.
       */
      const credentials = ctx.get('credentials');
      return credentials === undefined
        ? {}
        : { credentials, };
    },
  },);
}

//endregion Plugin contract
