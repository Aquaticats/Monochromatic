/**
 Registration and lifecycle tests for the Qwen Token Plan Individual pi
 extension entry.

 The fake extension API follows the built-module verification pattern in
 `package/pi-plugin/neuralwatt/src/registration.unit.test.ts`.
 Every test shares one `concurrency: 1` ancestor because the suite stubs
 `globalThis.fetch` and moves the process working directory,
 both process-wide state.

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
  mapQwenTokenPlanIndividualModels,
  qwenTokenPlanIndividualListingSnapshot,
  QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE,
  type QwenTokenPlanListingResponse,
} from '@monochromatic-dev/agent-harness-shared-qwen-token-plan-individual-listing/ts';
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
import qwenTokenPlanIndividualProviderExtension, {
  createQwenTokenPlanIndividualProvider,
  projectQwenTokenPlanIndividualModels,
  type QwenTokenPlanIndividualProvider,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Captured extension registrations and handlers one factory run produced.
 */
type ExtensionCapture = {
  readonly providers: QwenTokenPlanIndividualProvider[];
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
      capture.providers.push(provider as QwenTokenPlanIndividualProvider);
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
 Discards one log message.
 */
function discardLogMessage(): void {}

/**
 Silent logger the registration tests run under.

 @returns a logger discarding every message
 */
function fixtureLogger(): Parameters<typeof createQwenTokenPlanIndividualProvider>[0]['logger'] {
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
): QwenTokenPlanIndividualProvider {
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

 The handle changes back on disposal,
 whether the scope ended well or threw,
 so a test that fails inside the directory never leaves the process there after
 the scope removes it.

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
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'extension registration',
      children: [
        it({
          name: 'registers the route provider at load, listing the snapshot with no credential',
          fn: async () => {
            const { api, capture, } = fakeExtensionApi();
            qwenTokenPlanIndividualProviderExtension(api as ForeignBorrowed<ExtensionAPI>,);
            expect(capture.providers,).toHaveLength(1);
            const provider = registeredProvider(capture);
            expect(provider.id,)
              .toBe(QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE);
            /**
             Ids the checked-in listing earns under the shared mapping and
             projection.
             */
            const expectedIds = projectQwenTokenPlanIndividualModels({
              facts: mapQwenTokenPlanIndividualModels({
                listing: qwenTokenPlanIndividualListingSnapshot.entries,
              },),
            },)
              .map(function toId(row,) {
                return row.id;
              },);
            expect(provider.getModels()
              .map(function toId(row,) {
                return row.id;
              },),)
              .toEqual(expectedIds);
          },
        },),
        it({
          name: 'register-anyway: the provider lists models even when no credential resolves',
          fn: async () => {
            const { api, capture, } = fakeExtensionApi();
            qwenTokenPlanIndividualProviderExtension(api as ForeignBorrowed<ExtensionAPI>,);
            const provider = registeredProvider(capture);
            /**
             Resolved auth the empty environment seam reports.
             */
            const resolved = await provider.auth.apiKey?.resolve({
              ctx: {
                env: function env(): Promise<undefined> {
                  return Promise.resolve(undefined);
                },
                fileExists: function fileExists(): Promise<boolean> {
                  return Promise.resolve(false,);
                },
              },
              signal: new AbortController().signal,
            },);
            expect(resolved,).toBe(undefined);
            expect(provider.getModels()
              .length,)
              .toBeGreaterThan(0);
          },
        },),
        it({
          name: 'registers the session lifecycle handlers',
          fn: async () => {
            const { api, capture, } = fakeExtensionApi();
            qwenTokenPlanIndividualProviderExtension(api as ForeignBorrowed<ExtensionAPI>,);
            expect(capture.sessionStarts,).toHaveLength(1);
            expect(capture.sessionShutdowns,).toHaveLength(1);
          },
        },),
        it({
          name: 'refreshes the catalog at session start for startup and reload only',
          fn: async () => {
            const { api, capture, } = fakeExtensionApi();
            qwenTokenPlanIndividualProviderExtension(api as ForeignBorrowed<ExtensionAPI>,);
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
            expect(refreshCalls,).toHaveLength(2);
            const [firstRefresh, secondRefresh] = refreshCalls;
            expect(firstRefresh?.providers,)
              .toEqual([QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE]);
            expect(secondRefresh?.providers,)
              .toEqual([QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE]);
          },
        },),
      ],
    },),

    //region Lifecycle teardown

    describe({
      name: 'session shutdown',
      children: [
        it({
          name: 'aborts the in-flight refresh the registry started',
          fn: async ctx => {
            /**
             Signals the stubbed transport received, in order.
             */
            const seenSignals: AbortSignal[] = [];
            ctx.sinon
              .stub(globalThis, 'fetch',)
              .callsFake(function stubbedFetch(
                _input: string | URL | Request,
                init?: RequestInit,
              ): Promise<Response> {
                /**
                 Signal the stubbed request carried, when it carried one.
                 */
                const requestSignal = init?.signal ?? undefined;
                if (requestSignal !== undefined)
                  seenSignals.push(requestSignal,);
                return new Promise(function neverSettles(_resolve, reject) {
                  requestSignal
                    ?.addEventListener('abort', function onAbort() {
                      reject(new Error('aborted',),);
                    }, { once: true, },);
                },);
              },);
            const { api, capture, } = fakeExtensionApi();
            qwenTokenPlanIndividualProviderExtension(api as ForeignBorrowed<ExtensionAPI>,);
            const provider = registeredProvider(capture);
            /**
             Refresh the registry would run, left in flight until the shutdown
             handler aborts it.
             */
            const refresh = provider.refreshModels?.({
              allowNetwork: true,
              signal: new AbortController().signal,
              publish: function publish(): Promise<boolean> {
                return Promise.resolve(true);
              },
            },);
            /**
             Shutdown handler the factory registered.
             */
            const [sessionShutdown] = capture.sessionShutdowns;
            await Promise.resolve(sessionShutdown?.(
              {
                type: 'session_shutdown',
                reason: 'quit',
              },
              {} as ForeignBorrowed<ExtensionContext>,
            ),);
            await refresh;
            expect(seenSignals.length,)
              .toBeGreaterThan(0);
            expect(seenSignals[0]?.aborted,)
              .toBe(true);
            expect(provider.getModels()
              .length,)
              .toBeGreaterThan(0);
          },
        },),
      ],
    },),

    //endregion Lifecycle teardown

    //region Never-persist guard

    describe({
      name: 'never-persist guard',
      children: [
        it({
          name: 'leaves models-store.json, models.json, and enabledModels byte-identical across a refresh',
          fn: async () => {
            /**
             Disposable home the guard owns outright,
             removed when the test scope ends.
             */
            await using scratchDirectory = await mkdtempDisposable(join(await scratchRoot(), 'qwen-token-plan-guard-',),);
            const home = scratchDirectory.path;
            /**
             Store files the guard pins byte-for-byte.
             */
            const watched: Record<string, string> = {
              'models-store.json': '{"stores":"fixture"}\n',
              'models.json': '{"providers":{}}\n',
              'settings.json': '{"enabledModels":["qwen-token-plan-individual/fixture-model"]}\n',
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
            const runtime = createQwenTokenPlanIndividualProvider({
              logger: fixtureLogger(),
              apiKeyAuth: {
                name: 'fixture key',
                resolve: function resolve(): Promise<undefined> {
                  return Promise.resolve(undefined);
                },
              },
              fetcher: function fetcher(): Promise<QwenTokenPlanListingResponse> {
                return Promise.resolve(Response.json({
                  object: 'list',
                  data: [{ id: 'qwen3.8-max', }],
                },),);
              },
              environment: {},
            },);
            /**
             Working directory the refresh runs under,
             so relative writes land in the guarded home and the guard can see
             them.
             Restored when this scope ends however it ends,
             so a refresh that throws leaves the process where it found it
             rather than in the directory the scope then removes.
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
              expect(await readFile(join(home, name,), 'utf8',))
                .toBe(watched[name] ?? '');
            },));
            expect((await readdir(home,)).toSorted(),)
              .toEqual(beforeListing);
          },
        },),
      ],
    },),

    //endregion Never-persist guard
  ],
},);
