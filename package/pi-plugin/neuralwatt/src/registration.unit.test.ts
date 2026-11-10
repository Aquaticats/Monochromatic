/**
 Registration and lifecycle tests for the NeuralWatt pi extension entry.

 The fake extension API follows the built-module verification pattern in
 `package/pi-plugin/thinking-default/src/mise.verify-extension.ts`.

 @module
 */

import {
  mkdtempDisposable,
  readFile,
  readdir,
  writeFile,
} from 'node:fs/promises';
import {
  join,
} from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  scratchRoot,
} from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  neuralWattListingSnapshot,
  NEURALWATT_ROUTE,
  type NeuralWattListingEntry,
  type NeuralWattListingResponse,
} from '@monochromatic-dev/agent-harness-shared-neuralwatt-listing/ts';
import type {
  ExtensionAPI,
  ExtensionContext,
  ExtensionHandler,
  SessionShutdownEvent,
  SessionStartEvent,
} from '@earendil-works/pi-coding-agent';
import type {
  ForeignBorrowed,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import neuralWattProviderExtension, {
  createNeuralWattProvider,
  ENV_LOCAL_JSON_PATH_VARIABLE,
  type NeuralWattPiProvider,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Captured extension registrations and handlers one factory run produced.
 */
type ExtensionCapture = {
  readonly providers: NeuralWattPiProvider[];
  readonly sessionStarts: ExtensionHandler<SessionStartEvent>[];
  readonly sessionShutdowns: ExtensionHandler<SessionShutdownEvent>[];
  readonly unsubscribed: string[];
};

/**
 Fake pi extension API capturing what the factory registers.

 @returns the fake API and the capture it fills
 */
function fakeExtensionApi(): {
  readonly api: ExtensionAPI;
  readonly capture: ExtensionCapture;
} {
  /**
   Registrations the factory made.
   */
  const capture: ExtensionCapture = {
    providers: [],
    sessionStarts: [],
    sessionShutdowns: [],
    unsubscribed: [],
  };
  /**
   Fake API exposing only the members the factory touches.
   */
  const api = {
    registerProvider: function registerProvider(
      provider: unknown,
    ): void {
      capture.providers.push(provider as NeuralWattPiProvider);
    },
    on: function on(
      event: string,
      handler: unknown,
    ): () => void {
      if (event === 'session_start')
        capture.sessionStarts.push(handler as ExtensionCapture['sessionStarts'][number]);
      if (event === 'session_shutdown')
        capture.sessionShutdowns.push(handler as ExtensionCapture['sessionShutdowns'][number]);
      return function unsubscribe(): void {
        capture.unsubscribed.push(event,);
      };
    },
  } as unknown as ExtensionAPI;
  return { api, capture, };
}

/**
 Session context whose model registry records refresh calls.

 @returns the context and the refresh options it was called with
 */
function fakeSessionContext(): {
  readonly ctx: ExtensionContext;
  readonly refreshCalls: Record<string, unknown>[];
} {
  /**
   Refresh options the registry was called with, in order.
   */
  const refreshCalls: Record<string, unknown>[] = [];
  /**
   Fake context exposing only the registry seam the handler touches.
   */
  const ctx = {
    modelRegistry: {
      refresh: function refresh(options: Record<string, unknown>,): Promise<{
        aborted: boolean;
        errors: Map<string, Error>;
      }> {
        refreshCalls.push(options,);
        return Promise.resolve({
          aborted: false,
          errors: new Map<string, Error>(),
        });
      },
    },
  } as unknown as ExtensionContext;
  return { ctx, refreshCalls, };
}

/**
 Listing entry the fetch fixtures serve.

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
 Discards one log message.
 */
function discardLogMessage(): void {}

/**
 Silent logger the registration tests run under.

 @returns a logger discarding every message
 */
function fixtureLogger(): Parameters<typeof createNeuralWattProvider>[0]['logger'] {
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
 The provider the factory registered.

 @param capture - registrations one factory run produced

 @returns the single registered provider

 @throws when the factory registered none
 */
function registeredProvider(
  capture: ExtensionCapture,
): NeuralWattPiProvider {
  /**
   The registered provider, absent when the factory registered none.
   */
  const [provider] = capture.providers;
  if (provider === undefined)
    throw new Error('factory registered no provider');
  return provider;
}

/**
 Moves the process working directory for the rest of one test scope.

 The handle changes back on disposal, whether the scope ended well or threw, so
 a test that fails inside the directory never leaves the process there after the
 scope removes it.

 @param directory - Directory the rest of the scope runs in

 @returns handle restoring the previous working directory

 @example
 ```ts
 using _restoredWorkingDirectory = scopedCwd({ directory: home });
 ```
 */
function scopedCwd({ directory, }: { readonly directory: string; },): Disposable {
  /**
   Working directory restored when the scope exits.
   */
  const previousWorkingDirectory = process.cwd();
  process.chdir(directory,);
  return {
    [Symbol.dispose](): void {
      process.chdir(previousWorkingDirectory,);
    },
  };
}

//endregion Fixtures

//region Registration

await describe({
  name: 'extension registration',
  concurrency: 1,
  children: [
    it({
      name: 'registers the neuralwatt provider at load, listing the snapshot with no credential',
      fn: async () => {
        const { api, capture, } = fakeExtensionApi();
        neuralWattProviderExtension(api as ForeignBorrowed<ExtensionAPI>,);
        expect(capture.providers).toHaveLength(1);
        const provider = registeredProvider(capture);
        expect(provider.id).toBe(NEURALWATT_ROUTE);
        expect(provider.getModels()).toHaveLength(neuralWattListingSnapshot.length);
      },
    }),
    it({
      name: 'register-anyway: the provider lists models even when no credential resolves',
      fn: async () => {
        const { api, capture, } = fakeExtensionApi();
        neuralWattProviderExtension(api as ForeignBorrowed<ExtensionAPI>,);
        const provider = registeredProvider(capture);
        /**
         Store override pointing at nothing, so no home can answer in this
         case and the resolution must stay absent despite the real store this
         checkout carries.
         */
        process.env[ENV_LOCAL_JSON_PATH_VARIABLE] =
          '/nonexistent-neuralwatt-guard/.env.local.json';
        const resolved = await provider.auth.apiKey?.resolve({
          ctx: {
            env: function env(): Promise<undefined> {
              return Promise.resolve(undefined);
            },
            fileExists: function fileExists(): Promise<boolean> {
              return Promise.resolve(false);
            },
          },
          signal: new AbortController().signal,
        },);
        Reflect.deleteProperty(
          process.env,
          ENV_LOCAL_JSON_PATH_VARIABLE,
        );
        expect(resolved).toBe(undefined);
        expect(provider.getModels()).toHaveLength(neuralWattListingSnapshot.length);
      },
    }),
    it({
      name: 'registers the session lifecycle handlers',
      fn: async () => {
        const { api, capture, } = fakeExtensionApi();
        neuralWattProviderExtension(api as ForeignBorrowed<ExtensionAPI>,);
        expect(capture.sessionStarts).toHaveLength(1);
        expect(capture.sessionShutdowns).toHaveLength(1);
      },
    }),
    it({
      name: 'refreshes the catalog at session start for startup and reload only',
      fn: async () => {
        const { api, capture, } = fakeExtensionApi();
        neuralWattProviderExtension(api as ForeignBorrowed<ExtensionAPI>,);
        const { ctx, refreshCalls, } = fakeSessionContext();
        /**
         Session start reasons the handler sees, in order.
         */
        const reasons: SessionStartEvent['reason'][] = [
          'startup',
          'reload',
          'new',
          'resume',
          'fork',
        ];
        /**
         Handler the factory registered, invoked once per reason.
         */
        const [sessionStart] = capture.sessionStarts;
        await Promise.all(reasons.map(function runHandler(reason,) {
          return Promise.resolve(sessionStart?.(
            {
              type: 'session_start',
              reason,
            },
            ctx,
          ),);
        },));
        expect(refreshCalls).toHaveLength(2);
        const [firstRefresh, secondRefresh] = refreshCalls;
        expect(firstRefresh?.providers).toEqual([NEURALWATT_ROUTE]);
        expect(secondRefresh?.providers).toEqual([NEURALWATT_ROUTE]);
      },
    }),
  ],
});

//endregion Registration

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
        const runtime = createNeuralWattProvider({
          logger: fixtureLogger(),
          apiKeyAuth: {
            name: 'fixture key',
            resolve: function resolve(): Promise<undefined> {
              return Promise.resolve(undefined);
            },
          },
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
        expect(seenSignal?.aborted).toBe(true);
        expect(runtime.provider.getModels())
          .toHaveLength(neuralWattListingSnapshot.length);
      },
    }),
  ],
});

//endregion Lifecycle teardown

//region Never-persist guard

await describe({
  name: 'never-persist guard',
  children: [
    it({
      name: 'leaves models-store.json, models.json, and enabledModels byte-identical across a refresh',
      fn: async () => {
        /**
         Disposable home the guard owns outright, removed when the test scope ends.
         */
        await using scratchDirectory = await mkdtempDisposable(join(await scratchRoot(), 'neuralwatt-guard-',),);
        const home = scratchDirectory.path;
        /**
         Store files the guard pins byte-for-byte.
         */
        const watched: Record<string, string> = {
          'models-store.json': '{"stores":"fixture"}\n',
          'models.json': '{"providers":{}}\n',
          'settings.json': '{"enabledModels":["neuralwatt/fixture-model"]}\n',
        };
        await Promise.all(Object.keys(watched).map(function writeWatched(name: string,) {
          return writeFile(join(home, name,), watched[name] ?? '', 'utf8',);
        },));
        /**
         Directory listing before the refresh.
         */
        const beforeListing = (await readdir(home,)).toSorted();
        /**
         Provider under test with a fixture listing of a single model.
         */
        const runtime = createNeuralWattProvider({
          logger: fixtureLogger(),
          apiKeyAuth: {
            name: 'fixture key',
            resolve: function resolve(): Promise<undefined> {
              return Promise.resolve(undefined);
            },
          },
          fetcher: function fetcher(): Promise<NeuralWattListingResponse> {
            return Promise.resolve({
              ok: true,
              status: 200,
              json: function json(): Promise<unknown> {
                return Promise.resolve({ data: [listingEntry('only-model',),], });
              },
            },);
          },
        },);
        /**
         Working directory the refresh runs under, so relative writes land in the
         guarded home and the guard can see them. Restored when this scope ends
         however it ends, so a refresh that throws leaves the process where it
         found it rather than in the directory the scope then removes.
         */
        using _restoredWorkingDirectory = scopedCwd({ directory: home, },);
        await runtime.provider.refreshModels?.({
          allowNetwork: true,
          signal: new AbortController().signal,
          publish: function publish(): Promise<boolean> {
            return Promise.resolve(true);
          },
        },);
        await Promise.all(Object.keys(watched).map(async function assertWatched(name: string,) {
          expect(await readFile(join(home, name,), 'utf8',)).toBe(watched[name] ?? '');
        },));
        expect((await readdir(home,)).toSorted()).toEqual(beforeListing);
      },
    }),
  ],
});

//endregion Never-persist guard
