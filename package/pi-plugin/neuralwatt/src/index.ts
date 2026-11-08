/**
 Pi extension entry point for the NeuralWatt provider.

 The factory registers the provider immediately,
 from the checked-in snapshot,
 so `pi --list-models` and the startup model choice see the route with no
 network and no credential.
 The `session_start` handler refreshes the catalog from the live listing for
 startup and reload only,
 and `session_shutdown` cancels any in-flight refresh.

 @module
 */

import { NEURALWATT_ROUTE, } from '@monochromatic-dev/agent-harness-shared-neuralwatt-listing/ts';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';
import type {
  ExtensionAPI,
  SessionShutdownEvent,
  SessionStartEvent,
} from '@earendil-works/pi-coding-agent';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { createNeuralWattApiKeyAuth, } from './credential.ts';
import { createNeuralWattProvider, } from './provider.ts';

export {
  hasEffortsDict,
  mapNeuralWattModelCost,
  NEURALWATT_OPENAI_COMPLETIONS_COMPAT,
  projectNeuralWattModel,
  projectNeuralWattModels,
  type NeuralWattPiModel,
} from './projection.ts';
export {
  createNeuralWattApiKeyAuth,
  deriveRepoEnvLocalJsonPath,
  readNeuralWattStoreValue,
  resolveNeuralWattCredential,
  type NeuralWattCredentialResolution,
  type NeuralWattCredentialSource,
  type NeuralWattCredentialSources,
  type NeuralWattStoreRead,
} from './credential.ts';
export {
  createNeuralWattProvider,
  type NeuralWattPiProvider,
  type NeuralWattProviderRuntime,
} from './provider.ts';
export {
  ENV_LOCAL_JSON_PATH_VARIABLE,
  NEURALWATT_COST_RATE_ZERO,
  NEURALWATT_CREDENTIAL_GUIDANCE,
  NEURALWATT_CREDENTIAL_NAME,
  NEURALWATT_PROVIDER_NAME,
  PI_SCHEMA_DEFAULT_CONTEXT_WINDOW,
  PI_SCHEMA_DEFAULT_MAX_TOKENS,
} from './constants.ts';

//region Extension entry

/**
 Register the NeuralWatt provider and its session lifecycle.
 The provider lands from the snapshot before any event fires,
 and every handler re-tags the package logger so records name their entry
 point.

 @param pi - pi extension API

 @example
 ```typescript
 import extension from '@monochromatic-dev/pi-plugin-neuralwatt';
 extension(pi);
 ```
 */
export default function neuralWattProviderExtension(
  pi: ForeignBorrowed<ExtensionAPI>,
): void {
  /**
   Package logger every seam re-tags.
   */
  const l = tagged({ tag: 'pi-plugin-neuralwatt', },);
  /**
   Credential seam the provider resolves per request.
   */
  const apiKeyAuth = createNeuralWattApiKeyAuth({ logger: l, },);
  /**
   Provider carrying the snapshot catalog and its lifecycle teardown hook.
   */
  const {
    provider,
    abortInFlightRefresh,
  } = createNeuralWattProvider({
    logger: l,
    apiKeyAuth,
  },);
  pi.registerProvider(provider);
  pi.on(
    'session_start',
    async function onNeuralWattSessionStart(
      event: ForeignBorrowed<SessionStartEvent>,
      ctx,
    ): Promise<void> {
      /**
       Logger tagged for this handler, so records name the entry point.
       */
      const rl = tagged({
        tag: onNeuralWattSessionStart.name,
        l,
      },);
      if ((event.reason !== 'startup')
        && (event.reason !== 'reload')) {
        rl.debug(
          `catalog refresh skipped: session start reason "${event.reason}" is not one this route refreshes on`,
        );
        return;
      }
      try {
        /**
         Registry-driven refresh, which reaches this provider's
         `refreshModels` with pi's own publication context.
         */
        const result = await ctx.modelRegistry
          .refresh({
            providers: [NEURALWATT_ROUTE],
            allowNetwork: true,
            force: true,
          },);
        for (const [failedProvider, error,] of result.errors) {
          rl.warn(
            `catalog refresh reported a failure for "${failedProvider}": ${caughtValueText(error,)}`,
          );
        }
        rl.debug(
          `catalog refresh finished for "${NEURALWATT_ROUTE}" (aborted: ${String(result.aborted,)})`,
        );
      } catch (error) {
        rl.warn(
          `catalog refresh failed, so the served catalog stands: ${caughtValueText(error,)}`,
        );
      }
    },
  );
  pi.on(
    'session_shutdown',
    function onNeuralWattSessionShutdown(
      _event: ForeignBorrowed<SessionShutdownEvent>,
    ): void {
      abortInFlightRefresh();
    },
  );
}

//endregion Extension entry

//region Re-tagged logger type

/**
 Logger shape the seams accept, re-exported so consumers type their own
 injection.
 */
export type NeuralWattLogger = Logger;

//endregion Re-tagged logger type
