/**
 Tests for the hand-built Qwen Token Plan Individual pi provider:
 the snapshot baseline,
 the refresh publication path,
 and the cancellation the session lifecycle drives.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  mapQwenTokenPlanIndividualModels,
  qwenTokenPlanIndividualListingSnapshot,
  QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME,
  type QwenTokenPlanListingEntry,
  type QwenTokenPlanListingFetcher,
} from '@monochromatic-dev/agent-harness-shared-qwen-token-plan-individual-listing/ts';
import type {
  ApiKeyAuth,
  ModelsPublication,
  RefreshModelsContext,
} from '@earendil-works/pi-ai';
import type {
  Logger,
} from '@monochromatic-dev/logger-core/ts';
import {
  createQwenTokenPlanIndividualProvider,
  projectQwenTokenPlanIndividualModels,
  QWEN_TOKEN_PLAN_INDIVIDUAL_BUILTIN_CREDENTIAL_ENV,
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
 Api-key auth stub; these tests never resolve credentials through pi.

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
 One listing entry the fetch fixtures serve.

 @param id - model id the entry carries

 @returns a bare listing entry in the endpoint's own shape
 */
function listingEntry(
  id: string,
): QwenTokenPlanListingEntry {
  return {
    id,
  };
}

/**
 Wrap listing entries in the models-list body the shared fetch decodes.

 @param entries - entries the fixture response carries

 @returns an HTTP response whose body carries the entries
 */
function listingResponse(
  entries: readonly QwenTokenPlanListingEntry[],
): Response {
  return new Response(
    JSON.stringify({
      object: 'list',
      data: entries,
    },),
    {
      status: 200,
      headers: { 'content-type': 'application/json', },
    },
  );
}

/**
 Fetcher serving one fixed listing, recording the options it received.

 @param entries - entries the fixture serves

 @returns the fetcher and the options each call carried
 */
function fixtureFetcher(entries: readonly QwenTokenPlanListingEntry[],): {
  readonly fetcher: QwenTokenPlanListingFetcher;
  readonly calls: {
    readonly baseUrl: string;
    readonly apiKey: string | undefined;
    readonly signal: AbortSignal;
  }[];
} {
  /**
   Options each fetch call carried, in order.
   */
  const calls: {
    readonly baseUrl: string;
    readonly apiKey: string | undefined;
    readonly signal: AbortSignal;
  }[] = [];
  return {
    fetcher: function fetcher(options): ReturnType<QwenTokenPlanListingFetcher> {
      calls.push(options,);
      return Promise.resolve(listingResponse(entries),);
    },
    calls,
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
 Ids the checked-in listing earns under the shared mapping and projection.

 @returns the served ids in listing order
 */
function expectedSnapshotIds(): string[] {
  return projectQwenTokenPlanIndividualModels({
    facts: mapQwenTokenPlanIndividualModels({
      listing: qwenTokenPlanIndividualListingSnapshot.entries,
    },),
  },)
    .map(function toId(row,) {
      return row.id;
    },);
}

/**
 A media-kind listing id the mapping never serves,
 named by the facts table's media kind rows.
 */
const MEDIA_LISTING_ID = 'wan3.0-video';

/**
 A chat-kind listing id the facts table serves,
 the fixture listing every refresh test drives.
 */
const FIXTURE_CHAT_ID = 'qwen3.8-max';

//endregion Fixtures

//region Snapshot baseline

await describe({
  name: 'snapshot baseline',
  children: [
    it({
      name: 'serves the projected checked-in listing before any refresh',
      fn: async () => {
        const { provider, } = createQwenTokenPlanIndividualProvider({
          logger: fixtureLogger(),
          apiKeyAuth: fixtureApiKeyAuth(),
        },);
        expect(provider.getModels()
          .map(function toId(row,) {
            return row.id;
          },),)
          .toEqual(expectedSnapshotIds(),);
      },
    },),
    it({
      name: 'serves the same catalog through getAllModels',
      fn: async () => {
        const { provider, } = createQwenTokenPlanIndividualProvider({
          logger: fixtureLogger(),
          apiKeyAuth: fixtureApiKeyAuth(),
        },);
        expect(provider.getAllModels?.()
          .map(function toId(row,) {
            return row.id;
          },),)
          .toEqual(expectedSnapshotIds(),);
      },
    },),
  ],
},);

//endregion Snapshot baseline

//region Refresh publication

await describe({
  name: 'refresh publication',
  children: [
    it({
      name: 'replaces the served catalog through publish update and never persist',
      fn: async () => {
        const { fetcher, } = fixtureFetcher([listingEntry(FIXTURE_CHAT_ID,),],);
        const { provider, } = createQwenTokenPlanIndividualProvider({
          logger: fixtureLogger(),
          apiKeyAuth: fixtureApiKeyAuth(),
          fetcher,
        },);
        const { context, publications, } = fixtureRefreshContext();
        await provider.refreshModels?.(context,);
        expect(publications,).toHaveLength(1);
        const [publication] = publications;
        expect(publication?.persist,).toBe(undefined);
        expect(provider.getModels()
          .map(function toId(row,) {
            return row.id;
          },),)
          .toEqual([FIXTURE_CHAT_ID]);
      },
    },),
    it({
      name: 'keeps the served catalog when the listing fetch fails',
      fn: async () => {
        const { provider, } = createQwenTokenPlanIndividualProvider({
          logger: fixtureLogger(),
          apiKeyAuth: fixtureApiKeyAuth(),
          fetcher: function fetcher(): ReturnType<QwenTokenPlanListingFetcher> {
            return Promise.reject(new Error('fixture transport failure',),);
          },
        },);
        const { context, publications, } = fixtureRefreshContext();
        await provider.refreshModels?.(context,);
        expect(publications,).toHaveLength(0);
        expect(provider.getModels()
          .map(function toId(row,) {
            return row.id;
          },),)
          .toEqual(expectedSnapshotIds(),);
      },
    },),
    it({
      name: 'keeps the served catalog when the listing maps to zero serviceable models',
      fn: async () => {
        const { fetcher, } = fixtureFetcher([listingEntry(MEDIA_LISTING_ID,),],);
        const { provider, } = createQwenTokenPlanIndividualProvider({
          logger: fixtureLogger(),
          apiKeyAuth: fixtureApiKeyAuth(),
          fetcher,
        },);
        const { context, publications, } = fixtureRefreshContext();
        await provider.refreshModels?.(context,);
        expect(publications,).toHaveLength(0);
        expect(provider.getModels(),)
          .toHaveLength(expectedSnapshotIds().length);
      },
    },),
    it({
      name: 'skips the refresh when the host forbids network access',
      fn: async () => {
        const { fetcher, calls, } = fixtureFetcher([listingEntry(FIXTURE_CHAT_ID,),],);
        const { provider, } = createQwenTokenPlanIndividualProvider({
          logger: fixtureLogger(),
          apiKeyAuth: fixtureApiKeyAuth(),
          fetcher,
        },);
        const { context, } = fixtureRefreshContext({ allowNetwork: false, },);
        await provider.refreshModels?.(context,);
        expect(calls,).toHaveLength(0);
        expect(provider.getModels(),)
          .toHaveLength(expectedSnapshotIds().length);
      },
    },),
    it({
      name: 'returns without fetching when the refresh signal is already aborted',
      fn: async () => {
        const { fetcher, calls, } = fixtureFetcher([listingEntry(FIXTURE_CHAT_ID,),],);
        const { provider, } = createQwenTokenPlanIndividualProvider({
          logger: fixtureLogger(),
          apiKeyAuth: fixtureApiKeyAuth(),
          fetcher,
        },);
        const { context, controller, } = fixtureRefreshContext();
        controller.abort();
        await provider.refreshModels?.(context,);
        expect(calls,).toHaveLength(0);
      },
    },),
  ],
},);

//endregion Refresh publication

//region Listing credential

await describe({
  name: 'listing credential',
  children: [
    it({
      name: 'passes the stored credential to the listing fetch',
      fn: async () => {
        const { fetcher, calls, } = fixtureFetcher([listingEntry(FIXTURE_CHAT_ID,),],);
        const { provider, } = createQwenTokenPlanIndividualProvider({
          logger: fixtureLogger(),
          apiKeyAuth: fixtureApiKeyAuth(),
          fetcher,
        },);
        const { context, } = fixtureRefreshContext();
        await provider.refreshModels?.({
          ...context,
          credential: {
            type: 'api_key',
            key: 'stored-key',
          },
        },);
        expect(calls[0]?.apiKey,).toBe('stored-key');
      },
    },),
    it({
      name: 'passes the individual environment key when no stored credential lands',
      fn: async () => {
        const { fetcher, calls, } = fixtureFetcher([listingEntry(FIXTURE_CHAT_ID,),],);
        const { provider, } = createQwenTokenPlanIndividualProvider({
          logger: fixtureLogger(),
          apiKeyAuth: fixtureApiKeyAuth(),
          fetcher,
          environment: {
            [QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME]: 'individual-key',
          },
        },);
        const { context, } = fixtureRefreshContext();
        await provider.refreshModels?.(context,);
        expect(calls[0]?.apiKey,).toBe('individual-key');
      },
    },),
    it({
      name: 'passes the builtin environment key as the last fallback',
      fn: async () => {
        const { fetcher, calls, } = fixtureFetcher([listingEntry(FIXTURE_CHAT_ID,),],);
        const { provider, } = createQwenTokenPlanIndividualProvider({
          logger: fixtureLogger(),
          apiKeyAuth: fixtureApiKeyAuth(),
          fetcher,
          environment: {
            [QWEN_TOKEN_PLAN_INDIVIDUAL_BUILTIN_CREDENTIAL_ENV]: 'builtin-key',
          },
        },);
        const { context, } = fixtureRefreshContext();
        await provider.refreshModels?.(context,);
        expect(calls[0]?.apiKey,).toBe('builtin-key');
      },
    },),
    it({
      name: 'prefers the stored credential over both environment homes',
      fn: async () => {
        const { fetcher, calls, } = fixtureFetcher([listingEntry(FIXTURE_CHAT_ID,),],);
        const { provider, } = createQwenTokenPlanIndividualProvider({
          logger: fixtureLogger(),
          apiKeyAuth: fixtureApiKeyAuth(),
          fetcher,
          environment: {
            [QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME]: 'individual-key',
            [QWEN_TOKEN_PLAN_INDIVIDUAL_BUILTIN_CREDENTIAL_ENV]: 'builtin-key',
          },
        },);
        const { context, } = fixtureRefreshContext();
        await provider.refreshModels?.({
          ...context,
          credential: {
            type: 'api_key',
            key: 'stored-key',
          },
        },);
        expect(calls[0]?.apiKey,).toBe('stored-key');
      },
    },),
    it({
      name: 'fetches with no key when no home answers',
      fn: async () => {
        const { fetcher, calls, } = fixtureFetcher([listingEntry(FIXTURE_CHAT_ID,),],);
        const { provider, } = createQwenTokenPlanIndividualProvider({
          logger: fixtureLogger(),
          apiKeyAuth: fixtureApiKeyAuth(),
          fetcher,
          environment: {},
        },);
        const { context, } = fixtureRefreshContext();
        await provider.refreshModels?.(context,);
        expect(calls[0]?.apiKey,).toBe(undefined);
      },
    },),
  ],
},);

//endregion Listing credential

//region Lifecycle teardown

await describe({
  name: 'session shutdown',
  children: [
    it({
      name: 'aborts the in-flight refresh and keeps the served catalog',
      fn: async () => {
        /**
         Signals the fixture transport received, in order.
         */
        const seenSignals: AbortSignal[] = [];
        const runtime = createQwenTokenPlanIndividualProvider({
          logger: fixtureLogger(),
          apiKeyAuth: fixtureApiKeyAuth(),
          fetcher: function fetcher(options): ReturnType<QwenTokenPlanListingFetcher> {
            seenSignals.push(options.signal,);
            return new Promise(function neverSettles(_resolve, reject) {
              options.signal.addEventListener('abort', function onAbort() {
                reject(new Error('aborted',),);
              }, { once: true, },);
            },);
          },
        },);
        /**
         Refresh context whose publish records nothing.
         */
        const refresh = runtime.provider.refreshModels?.({
          allowNetwork: true,
          signal: new AbortController().signal,
          publish: function publish(): Promise<boolean> {
            return Promise.resolve(true);
          },
        },);
        runtime.abortInFlightRefresh();
        await refresh;
        const [seenSignal] = seenSignals;
        expect(seenSignal?.aborted,).toBe(true);
        expect(runtime.provider.getModels(),)
          .toHaveLength(expectedSnapshotIds().length);
      },
    },),
  ],
},);

//endregion Lifecycle teardown
