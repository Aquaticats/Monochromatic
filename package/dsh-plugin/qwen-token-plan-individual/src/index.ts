/**
 Qwen Token Plan Individual provider plugin entry: keeps the
 `qwen-token-plan-individual` provider route written into the shipped pi-ai
 adapter's settings namespace, serving a checked-in model snapshot at mount
 and landing the live Qwen Token Plan listing once the mount fetch resolves. @module
 */

import type {
  Context,
  EventsService,
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
   fetch is about to run. A write refused by the host's open reload
   transaction hands its retry here instead of failing: the plugin-manager
   composes this plugin inside that transaction on every `set_plugin`,
   `install_bundle`, and `set_bundle` (`dsh-plugin-manager` `configure` →
   `hmr.runExclusive`), where `@deepseek-ai/dsh-hmr` refuses every settings
   write as nested (`runExclusive`, `lib/index.js:749`) and the refusing chain
   keeps the transaction's context for good, so the write must be re-run from
   outside it.
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
    onWriteDeferral: function onWriteDeferral(retry: () => Promise<void>): void {
      /**
       Re-run the deferred mount refresh from the plugin-manager's settle
       dispatch: `plugin-manager/changed` is emitted at the tail of `change()`,
       outside the reload transaction (`dsh-plugin-manager` `lib/index.js:3185`,
       typed at `lib/types/types.d.ts:392`), and a listener body runs in the
       emit's own context, so the retried write lands instead of refusing
       again. Registering on each deferral keeps the retry pending across an
       operation whose own settle dispatch has already passed, draining at the
       next one.
       */
      function runDeferredRetry(): void {
        void retry();
      }
      /**
       Event bus carrying the plugin-manager's settle dispatch; held so the
       registration reads as one call step rather than a member chain.
       */
      const events: EventsService = ctx.events;
      events.once(
        'plugin-manager/changed',
        runDeferredRetry,
      );
    },
  },);
}

//endregion Plugin contract
