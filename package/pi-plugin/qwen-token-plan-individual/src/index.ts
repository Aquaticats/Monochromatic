/**
 Pi extension entry point for the Qwen Token Plan Individual provider.

 The factory registers the provider immediately,
 from the checked-in snapshot,
 so `pi --list-models` and the startup model choice see the full route with no
 network and no credential,
 and pi's provider composition prefers this registration over the builtin
 catalog while the extension is loaded.
 The `session_start` handler refreshes the catalog from the live listing for
 startup and reload only,
 and `session_shutdown` cancels any in-flight refresh.

 @module
 */

import {
  QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE,
} from '@monochromatic-dev/agent-harness-shared-qwen-token-plan-individual-listing/ts';
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
import { createQwenTokenPlanIndividualApiKeyAuth, } from './credential.ts';
import { createQwenTokenPlanIndividualProvider, } from './provider.ts';

export {
  hasResolvedEfforts,
  mapQwenTokenPlanIndividualModelCost,
  projectQwenTokenPlanIndividualModel,
  projectQwenTokenPlanIndividualModels,
  QWEN_TOKEN_PLAN_INDIVIDUAL_IMAGE_INPUT_LIMITS,
  type QwenTokenPlanIndividualPiModel,
} from './projection.ts';
export {
  createQwenTokenPlanIndividualApiKeyAuth,
  isApiKeyCredential,
  resolveQwenTokenPlanIndividualCredential,
  storedCredentialSources,
  type QwenTokenPlanIndividualCredentialResolution,
  type QwenTokenPlanIndividualCredentialSource,
  type QwenTokenPlanIndividualCredentialSources,
} from './credential.ts';
export {
  createQwenTokenPlanIndividualProvider,
  type QwenTokenPlanIndividualProvider,
  type QwenTokenPlanIndividualProviderRuntime,
} from './provider.ts';
export {
  QWEN_TOKEN_PLAN_COST_RATE_ZERO,
  QWEN_TOKEN_PLAN_INDIVIDUAL_BASE_URL,
  QWEN_TOKEN_PLAN_INDIVIDUAL_BUILTIN_CREDENTIAL_ENV,
  QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_GUIDANCE,
  QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_LABEL,
  QWEN_TOKEN_PLAN_INDIVIDUAL_PROVIDER_NAME,
} from './constants.ts';

//region Extension entry

/**
 Register the Qwen Token Plan Individual provider and its session lifecycle.
 The provider lands from the snapshot before any event fires,
 and every handler re-tags the package logger so records name their entry
 point.

 @param pi - pi extension API

 @example
 ```typescript
 import extension from '@monochromatic-dev/pi-plugin-qwen-token-plan-individual';
 extension(pi);
 ```
 */
export default function qwenTokenPlanIndividualProviderExtension(
  pi: ForeignBorrowed<ExtensionAPI>,
): void {
  /**
   Package logger every seam re-tags.
   */
  const l = tagged({ tag: 'pi-plugin-qwen-token-plan-individual', },);
  /**
   Credential seam the provider resolves per request.
   */
  const apiKeyAuth = createQwenTokenPlanIndividualApiKeyAuth({ logger: l, },);
  /**
   Provider carrying the snapshot catalog and its lifecycle teardown hook.
   */
  const {
    provider,
    abortInFlightRefresh,
  } = createQwenTokenPlanIndividualProvider({
    logger: l,
    apiKeyAuth,
  },);
  pi.registerProvider(provider);
  pi.on(
    'session_start',
    async function onQwenTokenPlanIndividualSessionStart(
      event: ForeignBorrowed<SessionStartEvent>,
      ctx,
    ): Promise<void> {
      /**
       Logger tagged for this handler, so records name the entry point.
       */
      const rl = tagged({
        tag: onQwenTokenPlanIndividualSessionStart.name,
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
            providers: [QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE],
            allowNetwork: true,
            force: true,
          },);
        for (const [failedProvider, error,] of result.errors) {
          rl.warn(
            `catalog refresh reported a failure for "${failedProvider}": ${caughtValueText(error,)}`,
          );
        }
        rl.debug(
          `catalog refresh finished for "${QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE}" (aborted: ${String(result.aborted,)})`,
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
    function onQwenTokenPlanIndividualSessionShutdown(
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
export type QwenTokenPlanIndividualLogger = Logger;

//endregion Re-tagged logger type
