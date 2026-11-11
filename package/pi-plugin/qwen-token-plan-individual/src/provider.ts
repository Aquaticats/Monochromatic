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
  Credential,
  Model,
  Provider,
  RefreshModelsContext,
  SimpleStreamOptions,
  TranscriptContext,
} from '@earendil-works/pi-ai';
import type {
  ForeignBorrowed,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  QWEN_TOKEN_PLAN_INDIVIDUAL_BASE_URL,
  QWEN_TOKEN_PLAN_INDIVIDUAL_BUILTIN_CREDENTIAL_ENV,
  QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_GUIDANCE,
  QWEN_TOKEN_PLAN_INDIVIDUAL_PROVIDER_NAME,
} from './constants.ts';
import {
  resolveQwenTokenPlanIndividualCredential,
  storedCredentialSources,
  type QwenTokenPlanIndividualCredentialResolution,
  type QwenTokenPlanIndividualCredentialSources,
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
 Environment homes one refresh reads,
 expressed as the credential sources fragment they fill:
 absence is the fragment carrying no key,
 so no nullish union crosses the seam.

 @param environment - launch environment the two homes read

 @returns the sources fragment each non-empty environment value fills

 @example
 ```typescript
 const sources = environmentCredentialSources({ environment: process.env });
 ```
 */
function environmentCredentialSources({
  environment,
}: {
  readonly environment: ForeignBorrowed<NodeJS.ProcessEnv>;
},): QwenTokenPlanIndividualCredentialSources {
  /**
   Environment value under the credential name the shared core pins.
   */
  const individualEnvironmentKey =
    environment[QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME];
  /**
   Environment value under the builtin's own name, the recorded fallback.
   */
  const builtinEnvironmentKey =
    environment[QWEN_TOKEN_PLAN_INDIVIDUAL_BUILTIN_CREDENTIAL_ENV];
  return {
    ...(individualEnvironmentKey === undefined
      ? {}
      : { individualEnvironmentKey, }),
    ...(builtinEnvironmentKey === undefined
      ? {}
      : { builtinEnvironmentKey, }),
  };
}

/**
 Resolve the listing credential one refresh fetches with:
 the stored credential pi resolved for this route first,
 then the two environment homes,
 exactly the chain the request path walks.

 @param credential - credential pi passed this refresh, absent when it passed none

 @param environment - launch environment the two homes read

 @returns the resolution whose `key` names the winning home's value

 @example
 ```typescript
 const resolution = refreshListingCredential({
   credential: context.credential,
   environment: process.env,
 });
 ```
 */
function refreshListingCredential({
  credential,
  environment,
}: {
  readonly credential?: ForeignBorrowed<Credential>;
  readonly environment: ForeignBorrowed<NodeJS.ProcessEnv>;
},): QwenTokenPlanIndividualCredentialResolution {
  return resolveQwenTokenPlanIndividualCredential({
    sources: {
      ...storedCredentialSources(credential === undefined
        ? {}
        : { credential, }),
      ...environmentCredentialSources({ environment, }),
    },
  },);
}

/**
 Compose the injected listing transport so the cancellation this refresh owns
 reaches it alongside the shared fetch's own signal.
 The shared core owns the request shape,
 so the provider injects a transport only where a test replaces it and
 otherwise leaves `fetchQwenTokenPlanIndividualListing` on its own bounded
 default.

 @param fetcher - transport override, absent outside tests

 @param signal - cancellation this refresh owns

 @returns the fetcher option carrying the wrapped transport,
 or the empty option when no override applies

 @example
 ```typescript
 const options = composedFetcherOptions({ fetcher: testFetch, signal: controller.signal });
 ```
 */
function composedFetcherOptions({
  fetcher,
  signal,
}: {
  readonly fetcher?: QwenTokenPlanListingFetcher;
  readonly signal: AbortSignal;
},): {
  readonly fetcher?: QwenTokenPlanListingFetcher;
} {
  if (fetcher === undefined)
    return {};
  return {
    fetcher: function composedQwenTokenPlanListingFetch(
      options: Parameters<QwenTokenPlanListingFetcher>[0],
    ): ReturnType<QwenTokenPlanListingFetcher> {
      return fetcher({
        ...options,
        signal: AbortSignal.any([
          options.signal,
          signal,
        ],),
      },);
    },
  };
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
  readonly environment?: NodeJS.ProcessEnv;
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
       Listing credential resolution this refresh fetches with,
       walked through the same chain the request path walks.
       */
      const resolution = refreshListingCredential({
        ...(context.credential === undefined
          ? {}
          : { credential: context.credential, }),
        environment,
      },);
      /**
       Winning key the resolution carries; absent when no home answered.
       */
      const apiKey = resolution.key;
      if (apiKey === undefined)
        rl.info(QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_GUIDANCE,);
      try {
        /**
         Fetcher option this refresh composes cancellation into,
         the empty option outside tests.
         */
        const fetcherOptions = composedFetcherOptions({
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
          ...fetcherOptions,
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
