/**
 Hand-built pi provider for the Qwen Token Plan Individual route.

 The provider serves the checked-in listing snapshot as its baseline catalog
 and replaces that catalog from the live listing through `refreshModels`,
 publishing only `update` so pi's model stores stay untouched
 (settled decision `persist` in
 `doc/planning/qwen-token-plan-individual-provider.md`).
 Every fetch failure keeps the served catalog,
 mirroring the DSH plugin's snapshot-first behavior.
 While the extension is loaded pi's `composeProvider` prefers this registration
 over the builtin catalog,
 so the stale builtin list stops serving and `unregisterProvider` restores it.

 @module
 */

import {
  QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME,
  QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE,
  fetchQwenTokenPlanIndividualListing,
  mapQwenTokenPlanIndividualModels,
  qwenTokenPlanIndividualListingSnapshot,
  type QWEN_TOKEN_PLAN_INDIVIDUAL_API,
  type QwenTokenPlanListingFetcher,
} from '@monochromatic-dev/agent-harness-shared-qwen-token-plan-individual-listing/ts';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';
import { openAICompletionsApi, } from '@earendil-works/pi-ai/api/openai-completions.lazy';
import type {
  ApiKeyAuth,
  ApiStreamOptions,
  AssistantMessageEventStream,
  Model,
  Provider,
  RefreshModelsContext,
  SimpleStreamOptions,
  TranscriptContext,
} from '@earendil-works/pi-ai';
import {
  QWEN_TOKEN_PLAN_INDIVIDUAL_BASE_URL,
  QWEN_TOKEN_PLAN_INDIVIDUAL_BUILTIN_CREDENTIAL_ENV,
  QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_GUIDANCE,
  QWEN_TOKEN_PLAN_INDIVIDUAL_PROVIDER_NAME,
} from './constants.ts';
import {
  resolveQwenTokenPlanIndividualCredential,
  storedCredentialKey,
} from './credential.ts';
import {
  projectQwenTokenPlanIndividualModels,
  type QwenTokenPlanIndividualPiModel,
} from './projection.ts';

//region Types

/**
 pi provider the extension registers for the Qwen Token Plan Individual route.
 */
export type QwenTokenPlanIndividualProvider =
  Provider<typeof QWEN_TOKEN_PLAN_INDIVIDUAL_API>;

/**
 Provider plus the teardown hook the session lifecycle drives.
 */
export type QwenTokenPlanIndividualProviderRuntime = {
  /**
   Provider object handed to `pi.registerProvider`.
   */
  readonly provider: QwenTokenPlanIndividualProvider;
  /**
   Abort any in-flight listing refresh, idempotent.
   */
  readonly abortInFlightRefresh: () => void;
};

/**
 Private catalog state one provider owns.
 A plain state object keeps the refresh loop free of root bindings while
 still carrying the catalog and the in-flight cancellation.
 */
type QwenTokenPlanIndividualProviderState = {
  /**
   Served catalog, replaced only inside a published update callback.
   */
  models: QwenTokenPlanIndividualPiModel[];
  /**
   Cancellation for the refresh currently running, absent when idle.
   */
  inFlight?: AbortController;
};

//endregion Types

//region Catalog state

/**
 Snapshot catalog the provider serves before any successful refresh.

 @returns the projected checked-in snapshot, in listing order
 */
function snapshotModels(): QwenTokenPlanIndividualPiModel[] {
  return projectQwenTokenPlanIndividualModels({
    facts: mapQwenTokenPlanIndividualModels({
      listing: qwenTokenPlanIndividualListingSnapshot.entries,
    },),
  },);
}

//endregion Catalog state

//region Transport

/**
 Compose the injected listing transport so the cancellation this refresh owns
 reaches it alongside the shared fetch's own signal.
 The shared core owns the request shape,
 so the provider injects a transport only where a test replaces it and
 otherwise leaves `fetchQwenTokenPlanIndividualListing` on its own bounded
 default.

 @param fetcher - transport override, absent outside tests

 @param signal - cancellation this refresh owns

 @returns the wrapped transport, or nothing when no override applies

 @example
 ```typescript
 const fetcher = composeFetcher({ fetcher: testFetch, signal: controller.signal });
 ```
 */
function composeFetcher({
  fetcher,
  signal,
}: {
  readonly fetcher?: QwenTokenPlanListingFetcher;
  readonly signal: AbortSignal;
},): QwenTokenPlanListingFetcher | undefined {
  if (fetcher === undefined)
    return undefined;
  return function composedQwenTokenPlanListingFetch(options: {
    readonly baseUrl: string;
    readonly apiKey: string | undefined;
    readonly signal: AbortSignal;
  },): ReturnType<QwenTokenPlanListingFetcher> {
    return fetcher({
      ...options,
      signal: AbortSignal.any([
        options.signal,
        signal,
      ],),
    },);
  };
}

/**
 Resolve the listing credential one refresh fetches with:
 the stored credential pi resolved for this route first,
 then the two environment homes,
 exactly the chain the request path walks.

 @param credential - credential pi passed this refresh

 @param environment - launch environment the two homes read

 @returns the resolved key, or `undefined` when no home answers

 @example
 ```typescript
 const apiKey = refreshListingCredential({
   credential: context.credential,
   environment: process.env,
 });
 ```
 */
function refreshListingCredential({
  credential,
  environment,
}: {
  readonly credential?: RefreshModelsContext['credential'] | undefined;
  readonly environment: Readonly<Record<string, string | undefined>>;
},): string | undefined {
  /**
   Stored key pi resolved for this route, when it carries one.
   */
  const credentialKey = storedCredentialKey({ credential, },);
  /**
   Environment values under both accepted names.
   */
  const individualEnvironmentKey =
    environment[QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME];
  const builtinEnvironmentKey =
    environment[QWEN_TOKEN_PLAN_INDIVIDUAL_BUILTIN_CREDENTIAL_ENV];
  return resolveQwenTokenPlanIndividualCredential({
    sources: {
      ...(credentialKey === undefined ? {} : { credentialKey, }),
      ...(individualEnvironmentKey === undefined
        ? {}
        : { individualEnvironmentKey, }),
      ...(builtinEnvironmentKey === undefined
        ? {}
        : { builtinEnvironmentKey, }),
    },
  },)
    .key;
}

//endregion Transport

//region Provider construction

/**
 Build the Qwen Token Plan Individual provider with its private catalog state.
 The catalog starts as the snapshot projection,
 so the route stands with no network and no credential.

 @param logger - package logger, re-tagged per entry point

 @param apiKeyAuth - credential seam pi resolves per request

 @param fetcher - listing transport override for tests

 @param environment - launch environment the listing credential homes read

 @returns the provider object and its in-flight teardown hook

 @example
 ```typescript
 const { provider } = createQwenTokenPlanIndividualProvider({ logger, apiKeyAuth });
 ```
 */
export function createQwenTokenPlanIndividualProvider({
  logger,
  apiKeyAuth,
  fetcher,
  environment = process.env,
}: {
  readonly logger: Logger;
  readonly apiKeyAuth: NonNullable<ApiKeyAuth>;
  readonly fetcher?: QwenTokenPlanListingFetcher;
  readonly environment?: Readonly<Record<string, string | undefined>>;
},): QwenTokenPlanIndividualProviderRuntime {
  /**
   Logger tagged for this construction, so records name the provider seam.
   */
  const l = tagged({
    tag: createQwenTokenPlanIndividualProvider.name,
    l: logger,
  },);
  /**
   Private catalog state, seeded from the snapshot projection.
   */
  const state: QwenTokenPlanIndividualProviderState = {
    models: snapshotModels(),
  };
  /**
   OpenAI-completions stream implementation the provider delegates to.
   */
  const streams = openAICompletionsApi();
  /**
   Provider object pi registers and dispatches through.
   */
  const provider: QwenTokenPlanIndividualProvider = {
    id: QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE,
    name: QWEN_TOKEN_PLAN_INDIVIDUAL_PROVIDER_NAME,
    baseUrl: QWEN_TOKEN_PLAN_INDIVIDUAL_BASE_URL,
    auth: { apiKey: apiKeyAuth, },
    getModels: function getQwenTokenPlanIndividualModels(): QwenTokenPlanIndividualPiModel[] {
      return state.models;
    },
    getAllModels: function getAllQwenTokenPlanIndividualModels(): QwenTokenPlanIndividualPiModel[] {
      return state.models;
    },
    stream: function streamQwenTokenPlanIndividual<T extends typeof QWEN_TOKEN_PLAN_INDIVIDUAL_API>(
      model: Model<T>,
      context: TranscriptContext,
      options?: ApiStreamOptions<T>,
    ): AssistantMessageEventStream {
      return streams.stream(
        model,
        context,
        options,
      );
    },
    streamSimple: function streamSimpleQwenTokenPlanIndividual(
      model: Model<typeof QWEN_TOKEN_PLAN_INDIVIDUAL_API>,
      context: TranscriptContext,
      options?: SimpleStreamOptions,
    ): AssistantMessageEventStream {
      return streams.streamSimple(
        model,
        context,
        options,
      );
    },
    refreshModels: async function refreshQwenTokenPlanIndividualModels(
      context: RefreshModelsContext,
    ): Promise<void> {
      /**
       Logger tagged for this refresh, so records name the entry point.
       */
      const rl = tagged({
        tag: refreshQwenTokenPlanIndividualModels.name,
        l,
      },);
      if (context.signal
        .aborted)
        return;
      if (!context.allowNetwork) {
        rl.debug(
          'refresh skipped: the host forbids network access, so the served catalog stands',
        );
        return;
      }
      /**
       Cancellation for this refresh, chained to the host's signal below.
       */
      const controller = new AbortController();
      state.inFlight
        ?.abort();
      state.inFlight = controller;
      /**
       Host-abort listener this refresh registers and then removes, declared
       before its first use.
       */
      function onHostAbort(): void {
        controller.abort();
      }
      context.signal
        .addEventListener(
        'abort',
        onHostAbort,
        { once: true, },
        );
      /**
       Listing credential this refresh fetches with, resolved through the same
       chain the request path walks.
       */
      const apiKey = refreshListingCredential({
        credential: context.credential,
        environment,
      },);
      if (apiKey === undefined)
        rl.info(QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_GUIDANCE,);
      try {
        /**
         Transport override this refresh composes cancellation into, absent
         outside tests.
         */
        const composedFetcher = composeFetcher({
          ...(fetcher === undefined ? {} : { fetcher, }),
          signal: controller.signal,
        },);
        /**
         Live listing entries, bounded and abortable through the shared fetch.
         */
        const listing = await fetchQwenTokenPlanIndividualListing({
          baseUrl: context.baseUrl ?? QWEN_TOKEN_PLAN_INDIVIDUAL_BASE_URL,
          apiKey,
          signal: controller.signal,
          ...(composedFetcher === undefined ? {} : { fetcher: composedFetcher, }),
        },);
        /**
         Live catalog the listing earns under the shared mapping.
         */
        const next = projectQwenTokenPlanIndividualModels({
          facts: mapQwenTokenPlanIndividualModels({ listing, },),
        },);
        if (next.length === 0)
          rl.warn(
            'live listing mapped to zero serviceable models, so the served catalog stands',
          );
        else {
          await context.publish({
            update: function replaceServedCatalog(): void {
              state.models = next;
            },
          },);
          rl.debug(
            `served catalog replaced with ${String(next.length,)} live models`,
          );
        }
      } catch (error) {
        rl.warn(
          `live listing refresh failed, so the served catalog stands: ${caughtValueText(error,)}`,
        );
      }
      /**
       Lifecycle cleanup runs on both the published and the failed path,
       since the guarded region above returns on neither.
       */
      context.signal
        .removeEventListener(
        'abort',
        onHostAbort,
        );
      delete state.inFlight;
    },
  };
  return {
    provider,
    abortInFlightRefresh: function abortInFlightRefresh(): void {
      state.inFlight
        ?.abort();
    },
  };
}

//endregion Provider construction
