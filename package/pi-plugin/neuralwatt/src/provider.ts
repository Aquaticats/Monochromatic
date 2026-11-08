/**
 Hand-built pi provider for the NeuralWatt route.

 The provider serves the checked-in snapshot as its baseline catalog and
 replaces that catalog from the live listing through `refreshModels`,
 publishing only `update` so pi's model stores stay untouched
 (owner answer `persist` in `doc/planning/neuralwatt-pi-plugin.md`).
 Every fetch failure keeps the served catalog,
 mirroring the DSH plugin's snapshot-first behavior.

 @module
 */

import {
  fetchNeuralWattListing,
  mapNeuralWattListing,
  neuralWattListingSnapshot,
  type NeuralWattListingFetcher,
} from '@monochromatic-dev/agent-harness-shared-neuralwatt-listing/ts';
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
  projectNeuralWattModels,
  type NeuralWattPiModel,
} from './projection.ts';
import {
  NEURALWATT_PROVIDER_NAME,
} from './constants.ts';
import {
  NEURALWATT_API,
  NEURALWATT_BASE_URL,
  NEURALWATT_ROUTE,
} from '@monochromatic-dev/agent-harness-shared-neuralwatt-listing/ts';

//region Types

/**
 pi provider the extension registers for the NeuralWatt route.
 */
export type NeuralWattPiProvider = Provider<typeof NEURALWATT_API>;

/**
 Provider plus the teardown hook the session lifecycle drives.
 */
export type NeuralWattProviderRuntime = {
  /**
   Provider object handed to `pi.registerProvider`.
   */
  readonly provider: NeuralWattPiProvider;
  /**
   Abort any in-flight listing refresh, idempotent.
   */
  readonly abortInFlightRefresh: () => void;
};

//endregion Types

//region Catalog state

/**
 Snapshot catalog the provider serves before any successful refresh.
 */
function snapshotModels(): NeuralWattPiModel[] {
  return projectNeuralWattModels({
    facts: mapNeuralWattListing({ listing: neuralWattListingSnapshot, },),
  },);
}

//endregion Catalog state

//region Provider construction

/**
 Build the NeuralWatt provider with its private catalog state.
 The catalog starts as the snapshot projection,
 so the route stands with no network and no credential.

 @param logger - package logger, re-tagged per entry point

 @param apiKeyAuth - credential seam pi resolves per request

 @param fetcher - listing transport override for tests

 @param timeoutMs - listing fetch bound override for tests

 @returns the provider object and its in-flight teardown hook

 @example
 ```typescript
 const { provider } = createNeuralWattProvider({ logger, apiKeyAuth });
 ```
 */
export function createNeuralWattProvider({
  logger,
  apiKeyAuth,
  fetcher,
  timeoutMs,
}: {
  readonly logger: Logger;
  readonly apiKeyAuth: NonNullable<ApiKeyAuth>;
  readonly fetcher?: NeuralWattListingFetcher;
  readonly timeoutMs?: number;
},): NeuralWattProviderRuntime {
  /**
   Logger tagged for this construction, so records name the provider seam.
   */
  const l = tagged({
    tag: createNeuralWattProvider.name,
    l: logger,
  },);
  /**
   Private catalog state, replaced only inside a published update callback.
   */
  let models: NeuralWattPiModel[] = snapshotModels();
  /**
   In-flight refresh cancellation, replaced on every refresh and cleared when
   that refresh settles.
   */
  let inFlight: AbortController | undefined = undefined;
  /**
   OpenAI-completions stream implementation the provider delegates to.
   */
  const streams = openAICompletionsApi();
  /**
   Provider object pi registers and dispatches through.
   */
  const provider: NeuralWattPiProvider = {
    id: NEURALWATT_ROUTE,
    name: NEURALWATT_PROVIDER_NAME,
    baseUrl: NEURALWATT_BASE_URL,
    auth: { apiKey: apiKeyAuth, },
    getModels: function getNeuralWattModels(): NeuralWattPiModel[] {
      return models;
    },
    getAllModels: function getAllNeuralWattModels(): NeuralWattPiModel[] {
      return models;
    },
    stream: function streamNeuralWatt<T extends typeof NEURALWATT_API>(
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
    streamSimple: function streamSimpleNeuralWatt(
      model: Model<typeof NEURALWATT_API>,
      context: TranscriptContext,
      options?: SimpleStreamOptions,
    ): AssistantMessageEventStream {
      return streams.streamSimple(
        model,
        context,
        options,
      );
    },
    refreshModels: async function refreshNeuralWattModels(
      context: RefreshModelsContext,
    ): Promise<void> {
      /**
       Logger tagged for this refresh, so records name the entry point.
       */
      const rl = tagged({
        tag: refreshNeuralWattModels.name,
        l,
      },);
      if (context.signal.aborted)
        return;
      if (!context.allowNetwork) {
        rl.debug(
          'refresh skipped: the host forbids network access, so the served catalog stands',
        );
        return;
      }
      /**
       Cancellation for this refresh, chained to the host's signal.
       */
      const controller = new AbortController();
      inFlight?.abort();
      inFlight = controller;
      const onHostAbort = function onHostAbort(): void {
        controller.abort();
      };
      context.signal.addEventListener(
        'abort',
        onHostAbort,
        { once: true, },
      );
      try {
        /**
         Live listing entries, bounded and abortable through the composed
         fetcher.
         */
        const listing = await fetchNeuralWattListing({
          fetcher: composeFetcher({
            fetcher,
            signal: controller.signal,
          },),
          ...(timeoutMs === undefined ? {} : { timeoutMs, },),
        },);
        /**
         Live catalog the listing earns under the shared mapping.
         */
        const next = projectNeuralWattModels({
          facts: mapNeuralWattListing({ listing, },),
        },);
        if (next.length === 0) {
          rl.warn(
            'live listing mapped to zero serviceable models, so the served catalog stands',
          );
          return;
        }
        await context.publish({
          update: function replaceServedCatalog(): void {
            models = next;
          },
        },);
        rl.debug(
          `served catalog replaced with ${String(next.length,)} live models`,
        );
      } catch (error) {
        rl.warn(
          `live listing refresh failed, so the served catalog stands: ${caughtValueText(error,)}`,
        );
      } finally {
        context.signal.removeEventListener(
          'abort',
          onHostAbort,
        );
        if (inFlight === controller)
          inFlight = undefined;
      }
    },
  };
  return {
    provider,
    abortInFlightRefresh: function abortInFlightRefresh(): void {
      inFlight?.abort();
    },
  };
}

//endregion Provider construction

//region Transport composition

/**
 Compose the listing fetcher so the host's abort signal reaches the transport
 alongside the fetch's own timeout signal.

 @param fetcher - transport override, defaulting to the runtime fetch

 @param signal - cancellation this refresh owns

 @returns a fetcher whose requests die with either signal
 */
function composeFetcher({
  fetcher,
  signal,
}: {
  readonly fetcher: NeuralWattListingFetcher | undefined;
  readonly signal: AbortSignal;
},): NeuralWattListingFetcher {
  /**
   Transport every request goes through.
   */
  const transport: NeuralWattListingFetcher = fetcher
    ?? function runtimeFetch({ url, signal: timeoutSignal, }) {
      return globalThis.fetch(
        url,
        { signal: timeoutSignal, },
      );
    };
  return function composedNeuralWattListingFetch(options: {
    readonly url: string;
    readonly signal: AbortSignal;
  },): ReturnType<NeuralWattListingFetcher> {
    return transport({
      url: options.url,
      signal: AbortSignal.any([options.signal, signal],),
    },);
  };
}

//endregion Transport composition
