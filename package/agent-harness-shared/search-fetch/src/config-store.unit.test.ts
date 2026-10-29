/**
 Unit tests for encrypted-store precedence and degraded startup.
 
 @module
 */

import {
  mkdir,
  mkdtemp,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  loadLinkupConfig,
} from '../dist/final/node/index.mjs';
import {
  createIdentityFixture,
  PLAINTEXT_SENTINEL,
  writeEncryptedStore,
  writePlaintextStore,
} from './config-store-fixture.ts';

//region Fixtures

/**
 Synthetic store values.
 */
const STORE_VALUES = {
  DEEPSEEK_API_KEY: 'store-deepseek-sentinel',
  EXA_API_KEY: 'store-exa-sentinel',
  LINKUP_API_KEY: 'store-linkup-sentinel',
} as const;

/**
 Disposable fixture identity and store.
 */
const fixture = await createIdentityFixture();
/**
 Encrypted store fixture.
 */
const encrypted = await writeEncryptedStore({
  root: fixture.root,
  identityPath: fixture.identityPath,
  values: STORE_VALUES,
});
/**
 Isolated Pi config home.
 */
const configHome = await mkdtemp(join(scratchRoot(), 'search-fetch-config-home-',),);

//endregion Fixtures

await describe({
  name: 'loadLinkupConfig encrypted store',
  children: [
    it({
      name: 'reads keys from the store when the environment has none',
      fn: async () => {
        /**
         Local value for config.
         */
        const config = await loadLinkupConfig({
          home: configHome,
          cwd: fixture.root,
          env: encrypted.env,
        },);

        expect(config.deepseekApiKey,).toBe(STORE_VALUES.DEEPSEEK_API_KEY,);
        expect(config.exaApiKey,).toBe(STORE_VALUES.EXA_API_KEY,);
        expect(config.linkupApiKey,).toBe(STORE_VALUES.LINKUP_API_KEY,);
        expect(config.deepseekApiKeyOrigin,).toBe('store',);
        expect(config.exaApiKeyOrigin,).toBe('store',);
        expect(config.linkupApiKeyOrigin,).toBe('store',);
        expect(config.source.storeLoaded,).toBe(true,);
        expect(config.source.storePath,).toBe(encrypted.storePath,);
      },
    },),
    it({
      name: 'lets the environment beat the store per key',
      fn: async () => {
        /**
         Local value for config.
         */
        const config = await loadLinkupConfig({
          home: configHome,
          cwd: fixture.root,
          env: {
            ...encrypted.env,
            EXA_API_KEY: 'env-exa-sentinel',
          },
        },);

        expect(config.exaApiKey,).toBe('env-exa-sentinel',);
        expect(config.exaApiKeyOrigin,).toBe('env',);
        expect(config.deepseekApiKey,).toBe(STORE_VALUES.DEEPSEEK_API_KEY,);
        expect(config.deepseekApiKeyOrigin,).toBe('store',);
        expect(config.linkupApiKey,).toBe(STORE_VALUES.LINKUP_API_KEY,);
        expect(config.linkupApiKeyOrigin,).toBe('store',);
      },
    },),
    it({
      name: 'does not touch the store when every environment key is set',
      fn: async () => {
        /**
         Local value for config.
         */
        const config = await loadLinkupConfig({
          home: configHome,
          cwd: fixture.root,
          env: {
            ...encrypted.env,
            DEEPSEEK_API_KEY: 'env-deepseek-sentinel',
            EXA_API_KEY: 'env-exa-sentinel',
            LINKUP_API_KEY: 'env-linkup-sentinel',
          },
        },);

        expect(config.deepseekApiKeyOrigin,).toBe('env',);
        expect(config.exaApiKeyOrigin,).toBe('env',);
        expect(config.linkupApiKeyOrigin,).toBe('env',);
        expect(config.source.storeLoaded,).toBeUndefined();
        expect(config.source.storePath,).toBeUndefined();
      },
    },),
    it({
      name: 'degrades with a value-free diagnostic for a plaintext store',
      fn: async () => {
        /**
         Local value for plaintextRoot.
         */
        const plaintextRoot = await createIdentityFixture();
        await writePlaintextStore({
          root: plaintextRoot.root,
          values: {
            DEEPSEEK_API_KEY: PLAINTEXT_SENTINEL,
            EXA_API_KEY: PLAINTEXT_SENTINEL,
            LINKUP_API_KEY: PLAINTEXT_SENTINEL,
          },
        },);

        /**
         Local value for config.
         */
        const config = await loadLinkupConfig({
          home: configHome,
          cwd: plaintextRoot.root,
          env: {
            PATH: process.env.PATH ?? '',
            HOME: plaintextRoot.root,
          },
        },);

        expect(config.deepseekApiKey,).toBeUndefined();
        expect(config.exaApiKey,).toBeUndefined();
        expect(config.linkupApiKey,).toBeUndefined();
        expect(config.source.storeError,).toBeTypeOf('string',);
        expect(config.source.storeError,).not.toContain(PLAINTEXT_SENTINEL,);
      },
    },),
    it({
      name: 'degrades with a value-free diagnostic for a missing identity',
      fn: async () => {
        /**
         Local value for config.
         */
        const config = await loadLinkupConfig({
          home: configHome,
          cwd: fixture.root,
          env: {
            PATH: process.env.PATH ?? '',
            HOME: fixture.root,
          },
        },);

        expect(config.deepseekApiKey,).toBeUndefined();
        expect(config.exaApiKey,).toBeUndefined();
        expect(config.source.storeError,).toContain('identity',);
      },
    },),
    it({
      name: 'degrades when the explicit store path is missing',
      fn: async () => {
        /**
         Local value for config.
         */
        const config = await loadLinkupConfig({
          home: configHome,
          cwd: fixture.root,
          envLocalJsonPath: join(fixture.root, 'absent.env.local.json',),
          env: encrypted.env,
        },);

        expect(config.deepseekApiKey,).toBeUndefined();
        expect(config.exaApiKey,).toBeUndefined();
        expect(config.source.storeError,).toContain('env-local-json path',);
      },
    },),
    it({
      name: 'discovers a store at a [monorepo] repo root',
      fn: async () => {
        /**
         Local value for repoRoot.
         */
        const repoRoot = await createIdentityFixture();
        await writeFile(join(repoRoot.root, 'mise.toml',), '[monorepo]\n',);
        await writeEncryptedStore({
          root: repoRoot.root,
          identityPath: repoRoot.identityPath,
          values: STORE_VALUES,
        },);
        /**
         Local value for nestedCwd.
         */
        const nestedCwd = join(repoRoot.root, 'sub', 'project',);
        await mkdir(nestedCwd, { recursive: true, },);

        /**
         Local value for config.
         */
        const config = await loadLinkupConfig({
          home: configHome,
          cwd: nestedCwd,
          env: {
            PATH: process.env.PATH ?? '',
            HOME: repoRoot.root,
            SOPS_AGE_KEY_FILE: repoRoot.identityPath,
          },
        },);

        expect(config.deepseekApiKey,).toBe(STORE_VALUES.DEEPSEEK_API_KEY,);
        expect(config.exaApiKey,).toBe(STORE_VALUES.EXA_API_KEY,);
        expect(config.source.storePath,).toBe(join(repoRoot.root, '.env.local.json',),);
      },
    },),
    it({
      name: 'does not leave the environment mutated',
      fn: async () => {
        /**
         Local value for before.
         */
        const before = JSON.stringify(process.env,);
        /**
         Local value for env.
         */
        const env = {
          ...encrypted.env,
          EXA_API_KEY: 'env-exa-sentinel',
        };

        await loadLinkupConfig({
          home: configHome,
          cwd: fixture.root,
          env,
        },);

        expect(JSON.stringify(process.env,),).toBe(before,);
        expect(env.EXA_API_KEY,).toBe('env-exa-sentinel',);
      },
    },),
  ],
},);
