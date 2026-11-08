/**
 Tests for the hand-built NeuralWatt pi provider.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  neuralWattListingSnapshot,
  type NeuralWattListingEntry,
  type NeuralWattListingResponse,
} from '@monochromatic-dev/agent-harness-shared-neuralwatt-listing/ts';
import type {
  ApiKeyAuth,
  ModelsPublication,
  RefreshModelsContext,
} from '@earendil-works/pi-ai';
import type {
  Logger,
} from '@monochromatic-dev/logger-core/ts';
import {
  createNeuralWattProvider,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Discards one log message.
 */
function discardLogMessage(): void {}

/**
 Silent logger the provider tests run under.
 */
function fixtureLogger(): Logger {
  return {
    debug: discardLogMessage,
    error: discardLogMessage,
    fatal: discardLogMessage,
    flush: function flush(): Promise<void> {
      return Promise.resolve();
    },
    info: discardLogMessage,
    trace: discardLogMessage,
    warn: discardLogMessage,
  };
}

/**
 Api-key auth stub; these tests never resolve credentials.

 @returns an auth object whose resolve reports nothing configured
 */
function fixtureApiKeyAuth(): NonNullable<ApiKeyAuth> {
  return {
    name: 'fixture key',
    resolve: function resolve(): Promise<undefined> {
      return Promise.resolve(undefined);
    },
  };
}

/**
 Publication-recording refresh context backed by a fixture signal.

 @param allowNetwork - whether the refresh may reach the network

 @returns the context, its publications, and its abort controller
 */
function fixtureRefreshContext({
  allowNetwork = true,
}: {
  readonly allowNetwork?: boolean;
} = {},): {
  readonly context: RefreshModelsContext;
  readonly publications: ModelsPublication[];
  readonly controller: AbortController;
} {
  /**
   Every publication the refresh made, in order.
   */
  const publications: ModelsPublication[] = [];
  /**
   Cancellation the context carries.
   */
  const controller = new AbortController();
  return {
    context: {
      allowNetwork,
      signal: controller.signal,
      publish: function publish(
        publication: ModelsPublication,
      ): Promise<boolean> {
        publications.push(publication,);
        publication.update?.();
        return Promise.resolve(true,);
      },
    },
    publications,
    controller,
  };
}

/**
 One listing entry the fetch fixtures serve.

 @param id - model id the entry carries

 @returns a serviceable non-reasoning listing entry
 */
function listingEntry(
  id: string,
): NeuralWattListingEntry {
  return {
    id,
    metadata: {
      display_name: id,
      capabilities: {
        vision: false,
        reasoning: false,
      },
      limits: {
        max_context_length: 1_000,
        max_output_tokens: 100,
      },
    },
  };
}

/**
 Fetcher serving one fixed listing payload.

 @param entries - entries the response carries

 @returns a fetcher answering with that listing
 */
function fixtureFetcher(
  entries: readonly NeuralWattListingEntry[],
): () => Promise<NeuralWattListingResponse> {
  return function fetcher(): Promise<NeuralWattListingResponse> {
    return Promise.resolve({
      ok: true,
      status: 200,
      json: function json(): Promise<unknown> {
        return Promise.resolve({ data: [...entries], });
      },
    },);
  };
}

//endregion Fixtures

//region Catalog behavior

await describe({
  name: createNeuralWattProvider.name,
  children: [
    it({
      name: 'serves the projected snapshot before any refresh',
      fn: async () => {
        const { provider, } = createNeuralWattProvider({
          logger: fixtureLogger(),
          apiKeyAuth: fixtureApiKeyAuth(),
        },);
        expect(provider.getModels()).toHaveLength(neuralWattListingSnapshot.length);
      },
    }),
    it({
      name: 'replaces the served catalog and drops models the live listing removed',
      fn: async () => {
        const { provider, } = createNeuralWattProvider({
          logger: fixtureLogger(),
          apiKeyAuth: fixtureApiKeyAuth(),
          fetcher: fixtureFetcher([listingEntry('only-model',),],),
        },);
        const { context, publications, } = fixtureRefreshContext();
        await provider.refreshModels?.(context,);
        expect(publications).toHaveLength(1);
        expect(provider.getModels().map(function readId(model,) {
          return model.id;
        },)).toEqual(['only-model']);
      },
    }),
    it({
      name: 'publishes only an update, never persistence',
      fn: async () => {
        const { provider, } = createNeuralWattProvider({
          logger: fixtureLogger(),
          apiKeyAuth: fixtureApiKeyAuth(),
          fetcher: fixtureFetcher([listingEntry('only-model',),],),
        },);
        const { context, publications, } = fixtureRefreshContext();
        await provider.refreshModels?.(context,);
        expect(publications).toHaveLength(1);
        const [publication] = publications;
        expect(Object.hasOwn(publication ?? {}, 'persist')).toBe(false);
        expect(typeof publication?.update).toBe('function');
      },
    }),
    it({
      name: 'keeps the served catalog when the fetch fails',
      fn: async () => {
        const { provider, } = createNeuralWattProvider({
          logger: fixtureLogger(),
          apiKeyAuth: fixtureApiKeyAuth(),
          fetcher: function fetcher(): Promise<NeuralWattListingResponse> {
            return Promise.reject(new Error('listing unreachable',));
          },
        },);
        const before = provider.getModels().length;
        const { context, publications, } = fixtureRefreshContext();
        await provider.refreshModels?.(context,);
        expect(publications).toHaveLength(0);
        expect(provider.getModels()).toHaveLength(before);
      },
    }),
    it({
      name: 'keeps the served catalog when the live listing maps to zero serviceable models',
      fn: async () => {
        const { provider, } = createNeuralWattProvider({
          logger: fixtureLogger(),
          apiKeyAuth: fixtureApiKeyAuth(),
          fetcher: fixtureFetcher([],),
        },);
        const before = provider.getModels().length;
        const { context, publications, } = fixtureRefreshContext();
        await provider.refreshModels?.(context,);
        expect(publications).toHaveLength(0);
        expect(provider.getModels()).toHaveLength(before);
      },
    }),
    it({
      name: 'skips the fetch when the host forbids network access',
      fn: async () => {
        /**
         Transport calls the refresh made.
         */
        const transportCalls: string[] = [];
        const { provider, } = createNeuralWattProvider({
          logger: fixtureLogger(),
          apiKeyAuth: fixtureApiKeyAuth(),
          fetcher: function fetcher(options: {
            readonly url: string;
            readonly signal: AbortSignal;
          },): Promise<NeuralWattListingResponse> {
            transportCalls.push(options.url,);
            return Promise.reject(new Error('should not run',));
          },
        },);
        const { context, publications, } = fixtureRefreshContext({
          allowNetwork: false,
        },);
        await provider.refreshModels?.(context,);
        expect(transportCalls).toHaveLength(0);
        expect(publications).toHaveLength(0);
      },
    }),
    it({
      name: 'aborts the in-flight fetch at teardown and keeps the served catalog',
      fn: async () => {
        /**
         Signals the fixture transport received, in order.
         */
        const seenSignals: AbortSignal[] = [];
        const runtime = createNeuralWattProvider({
          logger: fixtureLogger(),
          apiKeyAuth: fixtureApiKeyAuth(),
          fetcher: function fetcher(options: {
            readonly url: string;
            readonly signal: AbortSignal;
          },): Promise<NeuralWattListingResponse> {
            seenSignals.push(options.signal,);
            return new Promise(function neverSettles(_resolve, reject) {
              options.signal.addEventListener('abort', function onAbort() {
                reject(new Error('aborted',));
              }, { once: true, },);
            },);
          },
        },);
        const { context, publications, } = fixtureRefreshContext();
        const refresh = runtime.provider.refreshModels?.(context,);
        runtime.abortInFlightRefresh();
        await refresh;
        const [seenSignal] = seenSignals;
        expect(seenSignal?.aborted).toBe(true);
        expect(publications).toHaveLength(0);
        expect(runtime.provider.getModels())
          .toHaveLength(neuralWattListingSnapshot.length);
      },
    }),
    it({
      name: 'composes the host abort signal into the transport',
      fn: async () => {
        /**
         Signals the fixture transport received, in order.
         */
        const seenSignals: AbortSignal[] = [];
        const { provider, } = createNeuralWattProvider({
          logger: fixtureLogger(),
          apiKeyAuth: fixtureApiKeyAuth(),
          fetcher: function fetcher(options: {
            readonly url: string;
            readonly signal: AbortSignal;
          },): Promise<NeuralWattListingResponse> {
            seenSignals.push(options.signal,);
            return new Promise(function neverSettles(_resolve, reject) {
              options.signal.addEventListener('abort', function onAbort() {
                reject(new Error('aborted',));
              }, { once: true, },);
            },);
          },
        },);
        const {
          context,
          controller,
          publications,
        } = fixtureRefreshContext();
        const refresh = provider.refreshModels?.(context,);
        controller.abort();
        await refresh;
        const [seenSignal] = seenSignals;
        expect(seenSignal?.aborted).toBe(true);
        expect(publications).toHaveLength(0);
      },
    }),
  ],
});

//endregion Catalog behavior
