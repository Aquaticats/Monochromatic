/**
 Offline native runtime initialization shared only by disposable host fixtures.
 
 @module
 */
import { writeFile, } from 'node:fs/promises';
import {
  InMemoryCredentialStore,
  InMemoryModelsStore,
  type Model,
} from '@earendil-works/pi-ai';
import {
  ModelRuntime,
  SettingsManager,
} from '@earendil-works/pi-coding-agent';
import { CODEX_PROVIDER, } from '../dist/final/node/index.mjs';
import type { FixtureHome, } from './host-fixture-home.ts';
import { requireCodexModel, } from './host-fixture-model.ts';
import {
  fixtureCredential,
  type fixtureProvider,
} from './host-fixture-provider.ts';

//region Runtime setup: only synthetic credentials and test-created config.

/**
 Actual runtime services created independently for one host fixture.
 */
export type FixtureRuntime = {
  readonly credentials: InMemoryCredentialStore;
  readonly runtime: ModelRuntime;
  readonly effective: NonNullable<ReturnType<ModelRuntime['getProvider']>>;
  readonly base: Model<'openai-codex-responses'>;
  readonly settings: SettingsManager;
};

/**
 Prepare original provider config, synthetic auth, and real offline services.
 
 @param home - disposable model configuration location
 
 @param source - finite native provider registered before the keyless adapter
 
 @param expired - stale synthetic access token makes native refresh observable
 
 @param configured - exercise actual native provider and model header overrides
 
 @returns independent SDK services with the original native selection
 
 @example
 ```ts
 const services = await fixtureRuntime({ home, source: fixtureProvider(), expired: false, configured: false });
 ```
 */
export async function fixtureRuntime({
  home,
  source,
  expired,
  configured,
}: {
  readonly home: FixtureHome;
  readonly source: ReturnType<typeof fixtureProvider>;
  readonly expired: boolean;
  readonly configured: boolean;
},): Promise<FixtureRuntime> {
  if (configured) {
    await writeFile(
      home.modelsPath,
      JSON.stringify({ providers: { 'openai-codex': {
        headers: {
          'x-config-provider': 'configured-provider',
          'X-Shared': 'provider-value',
        },
        modelOverrides: { 'gpt-host-fixture': {
          name: 'Configured fixture',
          contextWindow: 333_333,
          maxTokens: 16_384,
          headers: {
            'x-config-model': 'original-model-only',
            'x-shared': 'model-value',
          },
        }, },
      }, }, },),
    );
  }
  /**
   Auth store contains only credentials constructed by this test.
   */
  const credentials = new InMemoryCredentialStore();
  await credentials.modify(
    CODEX_PROVIDER,
    function seed(): Promise<ReturnType<typeof fixtureCredential>> {
      return Promise.resolve(fixtureCredential({ expired, },),);
    },
  );
  /**
   Real runtime resolves config and auth without reading ambient stores.
   */
  const runtime = await ModelRuntime.create({
    credentials,
    modelsPath: configured ? home.modelsPath : null,
    modelsStore: new InMemoryModelsStore(),
    refreshOnCreate: false,
    allowModelNetwork: false,
  },);
  runtime.registerNativeProvider(source.provider,);
  /**
   Effective original provider includes configuration without replacement.
   */
  const effective = runtime.getProvider(CODEX_PROVIDER,);
  if (effective === undefined)
    throw new Error('Synthetic Codex provider was not registered.',);
  /**
   Initial ordinary selection is resolved from the actual native registry.
   */
  const base = requireCodexModel(runtime.getModel(
    CODEX_PROVIDER,
    source.state
      .models[0]
      ?.id
      ?? '',
  ),);
  /**
   Settings disable unrelated background work without changing fast defaults.
   */
  const settings = SettingsManager.inMemory({
    defaultProvider: CODEX_PROVIDER,
    defaultModel: base.id,
    defaultThinkingLevel: 'high',
    enabledModels: [`${CODEX_PROVIDER}/${base.id}`,],
    transport: 'auto',
    compaction: { enabled: false, },
    retry: {
      enabled: false,
      provider: { maxRetries: 0, },
    },
    cacheWarming: 'off',
    enableAnalytics: false,
    enableInstallTelemetry: false,
  },);
  return {
    credentials,
    runtime,
    effective,
    base,
    settings,
  };
}

//endregion Runtime setup
